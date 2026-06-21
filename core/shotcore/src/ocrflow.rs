//! OCR capture flow (CleanShot Text Recognition). The native engine (Windows.Media.Ocr /
//! Apple Vision) fills an OcrResult; this module restricts it to a selected region and
//! formats it for the clipboard in natural reading order. Pure logic, identical on both
//! platforms.
use crate::geometry::Rect;
use crate::ocr::OcrResult;
use std::cmp::Ordering;

fn center_in(r: Rect, region: Rect) -> bool {
 let cx = r.x + r.w / 2.0;
 let cy = r.y + r.h / 2.0;
 cx >= region.x && cx <= region.x + region.w && cy >= region.y && cy <= region.y + region.h
}

// Group words into reading-order rows (top rows first, left-to-right within a row).
fn group_rows(mut words: Vec<(String, Rect)>) -> Vec<String> {
 words.sort_by(|a, b| a.1.y.partial_cmp(&b.1.y).unwrap_or(Ordering::Equal));
 let mut rows: Vec<Vec<(String, Rect)>> = Vec::new();
 for w in words {
 let mut placed = false;
 if let Some(last) = rows.last_mut() {
 let ry = last[0].1.y;
 let rh = last[0].1.h.max(1.0);
 if (w.1.y - ry).abs() <= rh * 0.6 {
 last.push(w.clone());
 placed = true;
 }
 }
 if !placed {
 rows.push(vec![w]);
 }
 }
 rows.into_iter()
 .map(|mut row| {
 row.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap_or(Ordering::Equal));
 row.into_iter().map(|(t, _)| t).collect::<Vec<_>>().join(" ")
 })
 .collect()
}

fn words_in_region(ocr: &OcrResult, region: Rect) -> Vec<(String, Rect)> {
 let mut out = Vec::new();
 for line in &ocr.lines {
 for w in &line.words {
 if center_in(w.bbox, region) {
 out.push((w.text.clone(), w.bbox));
 }
 }
 }
 out
}

/// Recognized text whose word centers fall inside `region`, in reading order
/// (rows separated by newlines). This is the copy-text-from-selection result.
pub fn text_in_region(ocr: &OcrResult, region: Rect) -> String {
 group_rows(words_in_region(ocr, region)).join("\n")
}

/// Number of recognized words whose center falls inside `region`.
pub fn word_count_in_region(ocr: &OcrResult, region: Rect) -> usize {
 words_in_region(ocr, region).len()
}

/// The whole result as a single clipboard line (rows joined by spaces, no newlines).
pub fn to_single_line(ocr: &OcrResult) -> String {
 ocr.lines
 .iter()
 .map(|l| l.text.trim())
 .filter(|t| !t.is_empty())
 .collect::<Vec<_>>()
 .join(" ")
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::ocr::{OcrLine, OcrWord};

 fn word(t: &str, x: f64, y: f64, w: f64) -> OcrWord {
 OcrWord { text: t.to_string(), bbox: Rect::new(x, y, w, 16.0), confidence: 0.95 }
 }

 fn sample() -> OcrResult {
 OcrResult::new(vec![
 OcrLine::from_words(vec![word("Hello", 10.0, 10.0, 50.0), word("World", 70.0, 10.0, 50.0)]),
 OcrLine::from_words(vec![word("Second", 10.0, 40.0, 60.0), word("line", 80.0, 40.0, 40.0)]),
 ])
 }

 #[test]
 fn region_extracts_reading_order() {
 let ocr = sample();
 let all = text_in_region(&ocr, Rect::new(0.0, 0.0, 500.0, 500.0));
 assert_eq!(all, "Hello World\nSecond line");
 let top = text_in_region(&ocr, Rect::new(0.0, 0.0, 500.0, 25.0));
 assert_eq!(top, "Hello World");
 assert_eq!(word_count_in_region(&ocr, Rect::new(0.0, 0.0, 500.0, 25.0)), 2);
 }

 #[test]
 fn single_line_join() {
 assert_eq!(to_single_line(&sample()), "Hello World Second line");
 }
}
