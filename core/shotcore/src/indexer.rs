//! Folder indexer: walk a directory tree and render it as HTML, plain text, or JSON.
//! Pure std::fs walk; used by the Tools > Folder Indexer feature (ShareX IndexerLib parity).
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct IndexNode {
 pub name: String,
 pub is_dir: bool,
 pub size: u64,
 pub children: Vec<IndexNode>,
}

pub struct IndexOptions {
 pub max_depth: u32,
}
impl Default for IndexOptions {
 fn default() -> Self { IndexOptions { max_depth: 32 } }
}

/// Build a recursive index of `root`.
pub fn build_index(root: &Path, opts: &IndexOptions) -> IndexNode {
 build_node(root, 0, opts)
}

fn build_node(path: &Path, depth: u32, opts: &IndexOptions) -> IndexNode {
 let name = path
 .file_name()
 .map(|n| n.to_string_lossy().into_owned())
 .unwrap_or_else(|| path.to_string_lossy().into_owned());
 if !path.is_dir() {
 let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
 return IndexNode { name, is_dir: false, size, children: Vec::new() };
 }
 let mut children: Vec<IndexNode> = Vec::new();
 if depth < opts.max_depth {
 if let Ok(rd) = std::fs::read_dir(path) {
 let mut entries: Vec<std::path::PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
 entries.sort_by(|a, b| {
 let (ad, bd) = (a.is_dir(), b.is_dir());
 if ad != bd { return bd.cmp(&ad); }
 a.file_name().cmp(&b.file_name())
 });
 for e in entries {
 children.push(build_node(&e, depth + 1, opts));
 }
 }
 }
 let size = children.iter().map(|c| c.size).sum();
 IndexNode { name, is_dir: true, size, children }
}

pub fn to_json(node: &IndexNode) -> String {
 serde_json::to_string_pretty(node).unwrap_or_else(|_| String::from("{}"))
}

fn human_size(bytes: u64) -> String {
 let units = ["B", "KB", "MB", "GB", "TB"];
 let mut s = bytes as f64;
 let mut u = 0usize;
 while s >= 1024.0 && u < units.len() - 1 {
 s /= 1024.0;
 u += 1;
 }
 if u == 0 { format!("{} {}", bytes, units[0]) } else { format!("{:.1} {}", s, units[u]) }
}

pub fn to_text(node: &IndexNode) -> String {
 let mut out = String::new();
 text_node(node, 0, &mut out);
 out
}
fn text_node(node: &IndexNode, depth: usize, out: &mut String) {
 for _ in 0..depth { out.push_str("\t"); }
 if node.is_dir {
 out.push_str(&format!("[{}]\n", node.name));
 } else {
 out.push_str(&format!("{} ({})\n", node.name, human_size(node.size)));
 }
 for c in &node.children { text_node(c, depth + 1, out); }
}

fn html_escape(s: &str) -> String {
 s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
}
pub fn to_html(node: &IndexNode) -> String {
 let mut body = String::new();
 html_node(node, &mut body);
 let title = html_escape(&node.name);
 format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Index of {}</title><style>body{{font-family:Segoe UI,Arial,sans-serif;background:#1b1b1f;color:#e6e6e6;padding:20px}} ul{{list-style:none;padding-left:18px}} .dir{{font-weight:600;color:#7aa2ff}} .size{{color:#888;font-size:12px;margin-left:8px}}</style></head><body><h2>Index of {}</h2>{}</body></html>", title, title, body)
}
fn html_node(node: &IndexNode, out: &mut String) {
 out.push_str("<ul>");
 for c in &node.children {
 if c.is_dir {
 out.push_str(&format!("<li><span class=\"dir\">{}/</span>", html_escape(&c.name)));
 html_node(c, out);
 out.push_str("</li>");
 } else {
 out.push_str(&format!("<li>{}<span class=\"size\">{}</span></li>", html_escape(&c.name), human_size(c.size)));
 }
 }
 out.push_str("</ul>");
}

#[cfg(test)]
mod tests {
 use super::*;
 use std::fs;
 #[test]
 fn indexes_tree() {
 let dir = std::env::temp_dir().join("shotcore_idx_test");
 let _ = fs::remove_dir_all(&dir);
 fs::create_dir_all(dir.join("sub")).unwrap();
 fs::write(dir.join("a.txt"), b"hello").unwrap();
 fs::write(dir.join("sub").join("b.txt"), b"hi").unwrap();
 let node = build_index(&dir, &IndexOptions::default());
 assert!(node.is_dir);
 assert_eq!(node.children.len(), 2);
 assert!(node.children[0].is_dir);
 assert_eq!(node.children[0].name, "sub");
 assert!(node.size >= 7);
 assert!(to_json(&node).contains("a.txt"));
 assert!(to_text(&node).contains("a.txt"));
 let html = to_html(&node);
 assert!(html.contains("<ul>") && html.contains("a.txt"));
 let _ = fs::remove_dir_all(&dir);
 }
 #[test]
 fn html_escapes_special_chars() {
 assert_eq!(html_escape("a<b>&c"), "a&lt;b&gt;&amp;c");
 }
}
