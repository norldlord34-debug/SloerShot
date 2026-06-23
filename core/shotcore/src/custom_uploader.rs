//! ShareX-compatible custom uploader engine: parse a custom uploader config, build an HTTP
//! request plan, and resolve the response with the ShareX {function:param|param} syntax
//! ({response} {json:path} {regex:pat|n} {xml:tag} {header:name} {base64:..} {input} {filename} {random:..}).
//! Pure logic; the shell performs the actual HTTP call using the plan.
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CustomUploaderBody {
 #[default]
 None,
 MultipartFormData,
 FormURLEncoded,
 JSON,
 XML,
 Binary,
}
impl CustomUploaderBody {
 pub fn name(&self) -> &'static str {
 match self {
 CustomUploaderBody::None => "None",
 CustomUploaderBody::MultipartFormData => "MultipartFormData",
 CustomUploaderBody::FormURLEncoded => "FormURLEncoded",
 CustomUploaderBody::JSON => "JSON",
 CustomUploaderBody::XML => "XML",
 CustomUploaderBody::Binary => "Binary",
 }
 }
}

/// ShareX .sxcu custom uploader config (PascalCase keys).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CustomUploaderConfig {
 #[serde(rename = "Name")] pub name: String,
 #[serde(rename = "RequestMethod")] pub request_method: String,
 #[serde(rename = "RequestURL")] pub request_url: String,
 #[serde(rename = "Parameters")] pub parameters: BTreeMap<String, String>,
 #[serde(rename = "Headers")] pub headers: BTreeMap<String, String>,
 #[serde(rename = "Body")] pub body: CustomUploaderBody,
 #[serde(rename = "Arguments")] pub arguments: BTreeMap<String, String>,
 #[serde(rename = "FileFormName")] pub file_form_name: String,
 #[serde(rename = "Data")] pub data: String,
 #[serde(rename = "URL")] pub url: String,
 #[serde(rename = "ThumbnailURL")] pub thumbnail_url: String,
 #[serde(rename = "DeletionURL")] pub deletion_url: String,
}

/// The resolved HTTP request the shell should perform.
#[derive(Debug, Clone, Serialize)]
pub struct RequestPlan {
 pub method: String,
 pub url: String,
 pub headers: BTreeMap<String, String>,
 pub body: String,
 pub arguments: BTreeMap<String, String>,
 pub file_form_name: String,
 pub data: String,
}

/// The links extracted from a response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ResponseResult {
 pub url: String,
 pub thumbnail_url: String,
 pub deletion_url: String,
}


// Resolution context. Owns its data so no lifetimes leak into the C ABI helpers.
struct Ctx {
 input: String,
 filename: String,
 response: String,
 headers: BTreeMap<String, String>,
}

// Parse ShareX {function:param|param} syntax. Byte-based scan (structural chars are ASCII).
fn parse_syntax(text: &str, ctx: &Ctx) -> String {
 let (out, _) = parse_seq(text.as_bytes(), 0, ctx, false);
 String::from_utf8_lossy(&out).into_owned()
}
fn parse_seq(bytes: &[u8], mut i: usize, ctx: &Ctx, stop_on_param: bool) -> (Vec<u8>, usize) {
 let mut out: Vec<u8> = Vec::new();
 while i < bytes.len() {
 let c = bytes[i];
 if c == 92 && i + 1 < bytes.len() {
 out.push(bytes[i + 1]);
 i += 2;
 continue;
 }
 if stop_on_param && (c == 124 || c == 125) {
 return (out, i);
 }
 if c == 123 {
 let (val, ni) = parse_func(bytes, i + 1, ctx);
 out.extend_from_slice(val.as_bytes());
 i = ni;
 continue;
 }
 out.push(c);
 i += 1;
 }
 (out, i)
}
fn parse_func(bytes: &[u8], mut i: usize, ctx: &Ctx) -> (String, usize) {
 let mut name: Vec<u8> = Vec::new();
 while i < bytes.len() && bytes[i] != 58 && bytes[i] != 125 {
 if bytes[i] == 92 && i + 1 < bytes.len() {
 name.push(bytes[i + 1]);
 i += 2;
 } else {
 name.push(bytes[i]);
 i += 1;
 }
 }
 let mut params: Vec<String> = Vec::new();
 if i < bytes.len() && bytes[i] == 58 {
 i += 1;
 loop {
 let (pv, ni) = parse_seq(bytes, i, ctx, true);
 params.push(String::from_utf8_lossy(&pv).into_owned());
 i = ni;
 if i < bytes.len() && bytes[i] == 124 {
 i += 1;
 continue;
 }
 break;
 }
 }
 if i < bytes.len() && bytes[i] == 125 {
 i += 1;
 }
 let nm = String::from_utf8_lossy(&name).trim().to_string();
 (call_function(&nm, &params, ctx), i)
}
fn call_function(name: &str, params: &[String], ctx: &Ctx) -> String {
 let p0 = params.get(0).map(|s| s.as_str()).unwrap_or("");
 match name.to_ascii_lowercase().as_str() {
 "response" => ctx.response.clone(),
 "input" => ctx.input.clone(),
 "filename" => ctx.filename.clone(),
 "json" => json_extract(&ctx.response, p0),
 "regex" => regex_extract(&ctx.response, params),
 "xml" => xml_extract(&ctx.response, p0),
 "header" => ctx.headers.get(p0.trim()).cloned().unwrap_or_default(),
 "base64" => base64::engine::general_purpose::STANDARD.encode(p0.as_bytes()),
 "random" => {
 if params.is_empty() { String::new() } else { params[rand::random::<usize>() % params.len()].clone() }
 }
 _ => String::new(),
 }
}
fn value_to_string(v: &serde_json::Value) -> String {
 match v {
 serde_json::Value::String(s) => s.clone(),
 serde_json::Value::Null => String::new(),
 other => other.to_string(),
 }
}
fn json_extract(json_text: &str, path: &str) -> String {
 let v: serde_json::Value = match serde_json::from_str(json_text) {
 Ok(v) => v,
 Err(_) => return String::new(),
 };
 let mut cur = &v;
 let pb = path.as_bytes();
 let mut i = 0usize;
 while i < pb.len() {
 let c = pb[i];
 if c == 46 {
 i += 1;
 continue;
 }
 if c == 91 {
 i += 1;
 let mut num = 0usize;
 let mut any = false;
 while i < pb.len() && pb[i] >= 48 && pb[i] <= 57 {
 num = num * 10 + (pb[i] - 48) as usize;
 i += 1;
 any = true;
 }
 while i < pb.len() && pb[i] != 93 {
 i += 1;
 }
 if i < pb.len() {
 i += 1;
 }
 if any {
 match cur.get(num) {
 Some(n) => cur = n,
 None => return String::new(),
 }
 }
 } else {
 let start = i;
 while i < pb.len() && pb[i] != 46 && pb[i] != 91 {
 i += 1;
 }
 let key = std::str::from_utf8(&pb[start..i]).unwrap_or("");
 match cur.get(key) {
 Some(n) => cur = n,
 None => return String::new(),
 }
 }
 }
 value_to_string(cur)
}
fn regex_extract(text: &str, params: &[String]) -> String {
 let pat = match params.get(0) {
 Some(p) => p,
 None => return String::new(),
 };
 let re = match regex::Regex::new(pat) {
 Ok(r) => r,
 Err(_) => return String::new(),
 };
 let caps = match re.captures(text) {
 Some(c) => c,
 None => return String::new(),
 };
 match params.get(1).map(|s| s.trim().to_string()) {
 Some(g) => {
 if let Ok(n) = g.parse::<usize>() {
 caps.get(n).map(|m| m.as_str().to_string()).unwrap_or_default()
 } else {
 caps.name(&g).map(|m| m.as_str().to_string()).unwrap_or_default()
 }
 }
 None => caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
 }
}
fn xml_extract(text: &str, path: &str) -> String {
 let seg = path.rsplit(|c: char| c as u32 == 47).next().unwrap_or(path).trim();
 if seg.is_empty() {
 return String::new();
 }
 let open = format!("<{}>", seg);
 let close = format!("</{}>", seg);
 if let Some(s) = text.find(&open) {
 let start = s + open.len();
 if let Some(e) = text[start..].find(&close) {
 return text[start..start + e].to_string();
 }
 }
 String::new()
}

/// Build the HTTP request plan by resolving request-time syntax in URL/params/headers/args/data.
pub fn build_request_plan(cfg: &CustomUploaderConfig, input: &str, filename: &str) -> RequestPlan {
 let ctx = Ctx { input: input.to_string(), filename: filename.to_string(), response: String::new(), headers: BTreeMap::new() };
 let method = if cfg.request_method.trim().is_empty() { "POST".to_string() } else { cfg.request_method.to_uppercase() };
 let mut url = parse_syntax(&cfg.request_url, &ctx);
 let mut query: Vec<String> = Vec::new();
 for (k, v) in &cfg.parameters {
 let ek = url_encode(&parse_syntax(k, &ctx));
 let ev = url_encode(&parse_syntax(v, &ctx));
 query.push(format!("{}={}", ek, ev));
 }
 if !query.is_empty() {
 if url.contains("?") { url.push_str("&"); } else { url.push_str("?"); }
 url.push_str(&query.join("&"));
 }
 let mut headers = BTreeMap::new();
 for (k, v) in &cfg.headers {
 headers.insert(parse_syntax(k, &ctx), parse_syntax(v, &ctx));
 }
 let mut arguments = BTreeMap::new();
 for (k, v) in &cfg.arguments {
 arguments.insert(parse_syntax(k, &ctx), parse_syntax(v, &ctx));
 }
 let data = parse_syntax(&cfg.data, &ctx);
 RequestPlan { method, url, headers, body: cfg.body.name().to_string(), arguments, file_form_name: cfg.file_form_name.clone(), data }
}

/// Resolve the response links (URL/thumbnail/deletion) using full ShareX syntax.
pub fn resolve_response(cfg: &CustomUploaderConfig, response: &str, headers: &BTreeMap<String, String>, input: &str, filename: &str) -> ResponseResult {
 let ctx = Ctx { input: input.to_string(), filename: filename.to_string(), response: response.to_string(), headers: headers.clone() };
 let url = if cfg.url.trim().is_empty() { response.to_string() } else { parse_syntax(&cfg.url, &ctx) };
 ResponseResult { url, thumbnail_url: parse_syntax(&cfg.thumbnail_url, &ctx), deletion_url: parse_syntax(&cfg.deletion_url, &ctx) }
}

/// Minimal RFC-3986 percent-encoding for query values.
fn url_encode(s: &str) -> String {
 let mut out = String::new();
 for b in s.bytes() {
 match b {
 65..=90 | 97..=122 | 48..=57 | 45 | 95 | 46 | 126 => out.push(b as char),
 32 => out.push_str("%20"),
 _ => out.push_str(&format!("%{:02X}", b)),
 }
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn json_path_simple() {
 let r = json_extract("{\"data\":{\"url\":\"https://x.com/a.png\"}}", "data.url");
 assert_eq!(r, "https://x.com/a.png");
 }

 #[test]
 fn json_path_array_index() {
 let r = json_extract("{\"files\":[{\"u\":\"one\"},{\"u\":\"two\"}]}", "files[1].u");
 assert_eq!(r, "two");
 }

 #[test]
 fn resolve_url_from_json() {
 let cfg = CustomUploaderConfig { url: "{json:link}".to_string(), ..Default::default() };
 let res = resolve_response(&cfg, "{\"link\":\"https://h.st/abc\"}", &BTreeMap::new(), "", "");
 assert_eq!(res.url, "https://h.st/abc");
 }

 #[test]
 fn regex_capture_group() {
 let ctx = Ctx { input: String::new(), filename: String::new(), response: "id=12345;".to_string(), headers: BTreeMap::new() };
 let out = parse_syntax("https://x.com/{regex:id=([0-9]+)|1}", &ctx);
 assert_eq!(out, "https://x.com/12345");
 }

 #[test]
 fn header_and_filename() {
 let mut h = BTreeMap::new();
 h.insert("Location".to_string(), "https://loc/9".to_string());
 let ctx = Ctx { input: "myfile".to_string(), filename: "myfile.png".to_string(), response: String::new(), headers: h };
 assert_eq!(parse_syntax("{header:Location}", &ctx), "https://loc/9");
 assert_eq!(parse_syntax("{filename}", &ctx), "myfile.png");
 }

 #[test]
 fn build_plan_query_and_headers() {
 let mut params = BTreeMap::new();
 params.insert("k".to_string(), "{filename}".to_string());
 let mut headers = BTreeMap::new();
 headers.insert("Authorization".to_string(), "Bearer xyz".to_string());
 let cfg = CustomUploaderConfig { request_method: "post".to_string(), request_url: "https://api.test/up".to_string(), parameters: params, headers, file_form_name: "file".to_string(), ..Default::default() };
 let plan = build_request_plan(&cfg, "in", "shot.png");
 assert_eq!(plan.method, "POST");
 assert!(plan.url.contains("k=shot.png"));
 assert!(plan.url.contains("?"));
 assert_eq!(plan.headers.get("Authorization").unwrap(), "Bearer xyz");
 assert_eq!(plan.file_form_name, "file");
 }

 #[test]
 fn config_from_sxcu_json() {
 let sxcu = "{\"Name\":\"My Host\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://up.test\",\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:url}\"}";
 let cfg: CustomUploaderConfig = serde_json::from_str(sxcu).unwrap();
 assert_eq!(cfg.name, "My Host");
 assert_eq!(cfg.body.name(), "MultipartFormData");
 assert_eq!(cfg.file_form_name, "file");
 assert_eq!(cfg.url, "{json:url}");
 }

 #[test]
 fn xml_extract_tag() {
 let r = xml_extract("<root><link>https://x/abc</link></root>", "link");
 assert_eq!(r, "https://x/abc");
 }

 #[test]
 fn escape_braces() {
 let ctx = Ctx { input: String::new(), filename: String::new(), response: String::new(), headers: BTreeMap::new() };
 assert_eq!(parse_syntax("a\\{b\\}c", &ctx), "a{b}c");
 }

 #[test]
 fn builtin_sloershot_config() {
 let cfg_json = "{\"Name\":\"SloerShot Backend\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://h.test/v1/upload\",\"Body\":\"Binary\",\"URL\":\"{json:url}\"}";
 let cfg: CustomUploaderConfig = serde_json::from_str(cfg_json).unwrap();
 let plan = build_request_plan(&cfg, "shot.png", "shot.png");
 assert_eq!(plan.method, "POST");
 assert_eq!(plan.url, "https://h.test/v1/upload");
 assert_eq!(plan.body, "Binary");
 let links = resolve_response(&cfg, "{\"url\":\"https://h.test/f/abc.png\"}", &BTreeMap::new(), "shot.png", "shot.png");
 assert_eq!(links.url, "https://h.test/f/abc.png");
 }

 #[test]
 fn builtin_imgur_config() {
 let cfg_json = "{\"Name\":\"Imgur\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://api.imgur.com/3/image\",\"Headers\":{\"Authorization\":\"Client-ID ABC\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"image\",\"URL\":\"{json:data.link}\",\"DeletionURL\":\"https://imgur.com/delete/{json:data.deletehash}\"}";
 let cfg: CustomUploaderConfig = serde_json::from_str(cfg_json).unwrap();
 let plan = build_request_plan(&cfg, "shot.png", "shot.png");
 assert_eq!(plan.body, "MultipartFormData");
 assert_eq!(plan.file_form_name, "image");
 assert_eq!(plan.headers.get("Authorization").unwrap(), "Client-ID ABC");
 let resp = "{\"data\":{\"link\":\"https://i.imgur.com/x.png\",\"deletehash\":\"DEL123\"},\"success\":true}";
 let links = resolve_response(&cfg, resp, &BTreeMap::new(), "shot.png", "shot.png");
 assert_eq!(links.url, "https://i.imgur.com/x.png");
 assert_eq!(links.deletion_url, "https://imgur.com/delete/DEL123");
 }
}
