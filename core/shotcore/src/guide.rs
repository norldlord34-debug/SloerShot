//! Step-by-step guide builder: an ordered list of captured steps with captions,
//! exportable to Markdown or HTML (auto-numbered click-through tutorials).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
 pub index: u32,
 pub image_path: String,
 pub caption: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Guide {
 pub title: String,
 pub steps: Vec<Step>,
}

fn esc(s: &str) -> String {
 s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
}

fn esc_attr(s: &str) -> String {
 esc(s).replace("\"", "&quot;")
}

impl Guide {
 pub fn new(title: impl Into<String>) -> Self {
 Self { title: title.into(), steps: Vec::new() }
 }

 /// Append a step, auto-assigning the next 1-based number; returns it.
 pub fn add_step(&mut self, image_path: impl Into<String>, caption: impl Into<String>) -> u32 {
 let index = self.steps.len() as u32 + 1;
 self.steps.push(Step { index, image_path: image_path.into(), caption: caption.into() });
 index
 }

 pub fn to_markdown(&self) -> String {
 let mut out = format!("# {}\n\n", self.title);
 for s in &self.steps {
 out.push_str(&format!("{}. {}\n\n ![Step {}]({})\n\n", s.index, s.caption, s.index, s.image_path));
 }
 out
 }

 pub fn to_html(&self) -> String {
 let mut out = format!("<h1>{}</h1>\n<ol>\n", esc(&self.title));
 for s in &self.steps {
 out.push_str(&format!(
 " <li><p>{}</p><img src=\"{}\" alt=\"Step {}\"></li>\n",
 esc(&s.caption),
 esc_attr(&s.image_path),
 s.index
 ));
 }
 out.push_str("</ol>\n");
 out
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 fn sample() -> Guide {
 let mut g = Guide::new("How to export");
 g.add_step("step1.png", "Open the file menu");
 g.add_step("step2.png", "Click Export");
 g
 }

 #[test]
 fn steps_auto_number() {
 let g = sample();
 assert_eq!(g.steps.len(), 2);
 assert_eq!(g.steps[0].index, 1);
 assert_eq!(g.steps[1].index, 2);
 }

 #[test]
 fn markdown_has_numbered_steps_and_images() {
 let md = sample().to_markdown();
 assert!(md.starts_with("# How to export"));
 assert!(md.contains("1. Open the file menu"));
 assert!(md.contains("![Step 2](step2.png)"));
 }

 #[test]
 fn html_escapes_and_lists() {
 let mut g = Guide::new("A <b> & C");
 g.add_step("a\".png", "Use <tag> & go");
 let html = g.to_html();
 assert!(html.contains("<h1>A &lt;b&gt; &amp; C</h1>"));
 assert!(html.contains("Use &lt;tag&gt; &amp; go"));
 assert!(html.contains("src=\"a&quot;.png\""));
 assert!(html.contains("<ol>") && html.contains("</ol>"));
 }
}
