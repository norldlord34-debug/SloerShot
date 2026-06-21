//! Table extraction: cluster OCR words into a grid by column position and export it
//! as CSV or Markdown (turn a screenshot of a table into editable data).
use crate::ocr::OcrResult;

/// Detect a grid of cells from OCR lines. Each line is a row; columns are inferred
/// from word left edges clustered within col_tol pixels.
pub fn detect_table(ocr: &OcrResult, col_tol: f64) -> Vec<Vec<String>> {
 let mut anchors: Vec<f64> = Vec::new();
 for line in &ocr.lines {
 for w in &line.words {
 let x = w.bbox.x;
 if !anchors.iter().any(|a| (a - x).abs() <= col_tol) {
 anchors.push(x);
 }
 }
 }
 anchors.sort_by(|a, b| a.partial_cmp(b).unwrap());
 let ncols = anchors.len();
 let mut grid = Vec::new();
 for line in &ocr.lines {
 let mut row = vec![String::new(); ncols];
 for w in &line.words {
 let ci = anchors
 .iter()
 .enumerate()
 .min_by(|(_, a), (_, b)| {
 (**a - w.bbox.x).abs().partial_cmp(&(**b - w.bbox.x).abs()).unwrap()
 })
 .map(|(i, _)| i)
 .unwrap_or(0);
 if !row[ci].is_empty() {
 row[ci].push(' ');
 }
 row[ci].push_str(&w.text);
 }
 grid.push(row);
 }
 grid
}

fn csv_field(s: &str) -> String {
 if s.contains(",") || s.contains("\"") || s.contains("\n") {
 format!("\"{}\"", s.replace("\"", "\"\""))
 } else {
 s.to_string()
 }
}

/// Export a grid as CSV.
pub fn to_csv(grid: &[Vec<String>]) -> String {
 grid.iter()
 .map(|row| row.iter().map(|c| csv_field(c)).collect::<Vec<_>>().join(","))
 .collect::<Vec<_>>()
 .join("\n")
}

fn md_row(row: &[String], ncols: usize) -> String {
 let mut s = String::from("|");
 for i in 0..ncols {
 let cell = row.get(i).map(|c| c.replace("|", "\\|")).unwrap_or_default();
 s.push_str(&format!(" {} |", cell));
 }
 s.push('\n');
 s
}

/// Export a grid as a GitHub-flavored Markdown table (first row is the header).
pub fn to_markdown(grid: &[Vec<String>]) -> String {
 if grid.is_empty() {
 return String::new();
 }
 let ncols = grid.iter().map(|r| r.len()).max().unwrap_or(0);
 let mut out = md_row(&grid[0], ncols);
 out.push('|');
 out.push_str(&" --- |".repeat(ncols));
 out.push('\n');
 for row in &grid[1..] {
 out.push_str(&md_row(row, ncols));
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::geometry::Rect;
 use crate::ocr::{OcrLine, OcrResult, OcrWord};

 fn word(text: &str, x: f64) -> OcrWord {
 OcrWord { text: text.to_string(), bbox: Rect::new(x, 0.0, 30.0, 10.0), confidence: 1.0 }
 }

 fn table_ocr() -> OcrResult {
 let r1 = OcrLine::from_words(vec![word("Name", 0.0), word("Age", 100.0)]);
 let r2 = OcrLine::from_words(vec![word("Alice", 2.0), word("30", 101.0)]);
 OcrResult::new(vec![r1, r2])
 }

 #[test]
 fn detects_two_by_two_grid() {
 let g = detect_table(&table_ocr(), 10.0);
 assert_eq!(g, vec![vec!["Name".to_string(), "Age".to_string()], vec!["Alice".to_string(), "30".to_string()]]);
 }

 #[test]
 fn csv_export() {
 let g = detect_table(&table_ocr(), 10.0);
 assert_eq!(to_csv(&g), "Name,Age\nAlice,30");
 }

 #[test]
 fn csv_quotes_fields_with_commas() {
 let g = vec![vec!["a,b".to_string(), "c".to_string()]];
 assert_eq!(to_csv(&g), "\"a,b\",c");
 }

 #[test]
 fn markdown_export_has_header_divider() {
 let g = detect_table(&table_ocr(), 10.0);
 let md = to_markdown(&g);
 assert!(md.contains("| Name | Age |"));
 assert!(md.contains("| --- | --- |"));
 assert!(md.contains("| Alice | 30 |"));
 }
}
