//! Code-screenshot card: turn a code snippet into a beautiful shareable image card
//! (window chrome, theme, line numbers, padding) in the spirit of ray.so / Carbon.
//! This models the card and computes its layout; native code does the syntax painting.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeTheme {
 Dark,
 Light,
 Dracula,
 Nord,
 SolarizedDark,
 Midnight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowControls {
 None,
 MacTraffic,
 Plain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeCard {
 pub code: String,
 pub language: String,
 pub theme: CodeTheme,
 pub controls: WindowControls,
 pub line_numbers: bool,
 pub padding: u32,
 pub font_size: f32,
 pub tab_width: u32,
}

impl Default for CodeCard {
 fn default() -> Self {
 Self {
 code: String::new(),
 language: "text".to_string(),
 theme: CodeTheme::Dark,
 controls: WindowControls::MacTraffic,
 line_numbers: true,
 padding: 32,
 font_size: 14.0,
 tab_width: 4,
 }
 }
}

impl CodeCard {
 pub fn line_count(&self) -> usize {
 if self.code.is_empty() {
 1
 } else {
 self.code.split('\n').count()
 }
 }

 /// Width of the widest line in characters, counting a tab as tab_width columns.
 pub fn max_line_chars(&self) -> usize {
 let tw = self.tab_width.max(1) as usize;
 self.code
 .split('\n')
 .map(|line| line.chars().map(|c| if c == '\t' { tw } else { 1 }).sum::<usize>())
 .max()
 .unwrap_or(0)
 }

 /// Number of digits needed for the line-number gutter.
 pub fn gutter_digits(&self) -> usize {
 self.line_count().to_string().len()
 }

 /// Estimated card pixel size given a monospace cell width and line height.
 pub fn estimated_size(&self, char_w: f32, line_h: f32) -> (u32, u32) {
 let gutter = if self.line_numbers { (self.gutter_digits() + 1) as f32 * char_w } else { 0.0 };
 let content_w = self.max_line_chars() as f32 * char_w + gutter;
 let titlebar = if matches!(self.controls, WindowControls::None) { 0.0 } else { line_h * 1.5 };
 let content_h = self.line_count() as f32 * line_h + titlebar;
 let pad = self.padding as f32 * 2.0;
 ((content_w + pad).ceil() as u32, (content_h + pad).ceil() as u32)
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn counts_lines_and_width() {
 let mut c = CodeCard::default();
 c.code = "fn main() {\n println!();\n}".to_string();
 assert_eq!(c.line_count(), 3);
 assert_eq!(c.max_line_chars(), " println!();".len());
 assert_eq!(c.gutter_digits(), 1);
 }

 #[test]
 fn tabs_count_as_tab_width() {
 let mut c = CodeCard::default();
 c.tab_width = 4;
 c.code = "\tx".to_string();
 assert_eq!(c.max_line_chars(), 5);
 }

 #[test]
 fn empty_code_is_one_line() {
 assert_eq!(CodeCard::default().line_count(), 1);
 }

 #[test]
 fn size_grows_with_padding_and_chrome() {
 let mut c = CodeCard::default();
 c.code = "abcdef".to_string();
 c.line_numbers = false;
 let (w, h) = c.estimated_size(8.0, 18.0);
 // 6 chars * 8 + 64 padding = 112 wide
 assert_eq!(w, 112);
 // 1 line * 18 + 1.5*18 titlebar + 64 = 109
 assert_eq!(h, 109);
 }
}
