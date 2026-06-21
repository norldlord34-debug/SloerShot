//! URL Scheme API: parse and emit sloershot:// commands so other apps and scripts
//! can drive captures (mirrors the CleanShot URL Scheme API). Pure string handling.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The URL scheme this app registers.
pub const SCHEME: &str = "sloershot";

/// A parsed scheme command: an action plus its query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemeCommand {
 pub action: String,
 pub params: BTreeMap<String, String>,
}

/// A recognized high-level action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
 CaptureArea { x: i64, y: i64, w: i64, h: i64 },
 CaptureFullscreen,
 CaptureWindow,
 Record,
 ScrollingCapture,
 OpenAnnotate { path: String },
 Upload { path: String },
 Pin { path: String },
 Unknown(String),
}

fn hex_val(b: u8) -> Option<u8> {
 match b {
 b'0'..=b'9' => Some(b - b'0'),
 b'a'..=b'f' => Some(b - b'a' + 10),
 b'A'..=b'F' => Some(b - b'A' + 10),
 _ => None,
 }
}

fn percent_decode(s: &str) -> String {
 let bytes = s.as_bytes();
 let mut out = Vec::with_capacity(bytes.len());
 let mut i = 0;
 while i < bytes.len() {
 match bytes[i] {
 b'%' if i + 2 < bytes.len() => match (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
 (Some(h), Some(l)) => {
 out.push(h * 16 + l);
 i += 3;
 }
 _ => {
 out.push(bytes[i]);
 i += 1;
 }
 },
 b'+' => {
 out.push(b' ');
 i += 1;
 }
 c => {
 out.push(c);
 i += 1;
 }
 }
 }
 String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode(s: &str) -> String {
 let mut out = String::new();
 for b in s.bytes() {
 match b {
 b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
 out.push(b as char)
 }
 _ => out.push_str(&format!("%{:02X}", b)),
 }
 }
 out
}

/// Parse a sloershot:// URL into a command. Returns None when the scheme or action is absent.
pub fn parse(url: &str) -> Option<SchemeCommand> {
 let prefix = format!("{}://", SCHEME);
 let rest = url.strip_prefix(&prefix)?;
 let (action_part, query) = match rest.split_once('?') {
 Some((a, q)) => (a, q),
 None => (rest, ""),
 };
 let action = action_part.trim_end_matches('/').to_string();
 if action.is_empty() {
 return None;
 }
 let mut params = BTreeMap::new();
 for pair in query.split('&').filter(|s| !s.is_empty()) {
 match pair.split_once('=') {
 Some((k, v)) => {
 params.insert(k.to_string(), percent_decode(v));
 }
 None => {
 params.insert(pair.to_string(), String::new());
 }
 }
 }
 Some(SchemeCommand { action, params })
}

impl SchemeCommand {
 pub fn get(&self, k: &str) -> Option<&str> {
 self.params.get(k).map(|s| s.as_str())
 }

 pub fn get_i64(&self, k: &str) -> Option<i64> {
 self.get(k)?.parse().ok()
 }

 /// Serialize back to a sloershot:// URL (params sorted by key).
 pub fn to_url(&self) -> String {
 let mut s = format!("{}://{}", SCHEME, self.action);
 if !self.params.is_empty() {
 s.push('?');
 let q: Vec<String> = self
 .params
 .iter()
 .map(|(k, v)| format!("{}={}", k, percent_encode(v)))
 .collect();
 s.push_str(&q.join("&"));
 }
 s
 }

 /// Classify into a high-level Action.
 pub fn action_kind(&self) -> Action {
 let path = || self.get("path").unwrap_or("").to_string();
 match self.action.as_str() {
 "capture-area" => Action::CaptureArea {
 x: self.get_i64("x").unwrap_or(0),
 y: self.get_i64("y").unwrap_or(0),
 w: self.get_i64("width").or_else(|| self.get_i64("w")).unwrap_or(0),
 h: self.get_i64("height").or_else(|| self.get_i64("h")).unwrap_or(0),
 },
 "capture-fullscreen" => Action::CaptureFullscreen,
 "capture-window" => Action::CaptureWindow,
 "record" => Action::Record,
 "scrolling-capture" => Action::ScrollingCapture,
 "open-annotate" => Action::OpenAnnotate { path: path() },
 "upload" => Action::Upload { path: path() },
 "pin" => Action::Pin { path: path() },
 other => Action::Unknown(other.to_string()),
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn parses_action_and_params() {
 let c = parse("sloershot://capture-area?x=10&y=20&width=300&height=200").unwrap();
 assert_eq!(c.action, "capture-area");
 assert_eq!(c.get_i64("x"), Some(10));
 assert_eq!(c.get_i64("height"), Some(200));
 }

 #[test]
 fn percent_decodes_path() {
 let c = parse("sloershot://open-annotate?path=%2Ftmp%2Fmy+shot.png").unwrap();
 assert_eq!(c.get("path"), Some("/tmp/my shot.png"));
 }

 #[test]
 fn classifies_actions() {
 let a = parse("sloershot://capture-area?x=1&y=2&w=3&h=4").unwrap().action_kind();
 assert_eq!(a, Action::CaptureArea { x: 1, y: 2, w: 3, h: 4 });
 assert_eq!(parse("sloershot://record").unwrap().action_kind(), Action::Record);
 assert_eq!(parse("sloershot://wat").unwrap().action_kind(), Action::Unknown("wat".to_string()));
 }

 #[test]
 fn rejects_non_scheme() {
 assert!(parse("https://example.com").is_none());
 assert!(parse("sloershot://").is_none());
 }

 #[test]
 fn url_roundtrip() {
 let c = parse("sloershot://upload?path=%2Fa%2Fb.png").unwrap();
 let url = c.to_url();
 let again = parse(&url).unwrap();
 assert_eq!(again.get("path"), Some("/a/b.png"));
 }
}
