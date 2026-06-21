//! On-device OCR result model.
//!
//! The native shells run OCR (Windows.Media.Ocr or Apple Vision) and populate
//! this structure. shotcore owns the cross-platform representation: words and
//! lines with bounding boxes, plus helpers to extract plain text and search.

use crate::geometry::Rect;
use serde::{Deserialize, Serialize};

/// A single recognized word with its bounding box and confidence (0.0 to 1.0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub bbox: Rect,
    pub confidence: f32,
}

/// A line of recognized text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrLine {
    pub text: String,
    pub bbox: Rect,
    pub words: Vec<OcrWord>,
}

impl OcrLine {
    /// Build a line from words, joining text with single spaces and unioning boxes.
    pub fn from_words(words: Vec<OcrWord>) -> Self {
        let text = words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let bbox = union_bbox(words.iter().map(|w| w.bbox));
        OcrLine { text, bbox, words }
    }
}

/// The full OCR result for one image.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OcrResult {
    pub lines: Vec<OcrLine>,
}

impl OcrResult {
    pub fn new(lines: Vec<OcrLine>) -> Self {
        Self { lines }
    }

    /// All recognized text joined with newlines.
    pub fn plain_text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn word_count(&self) -> usize {
        self.lines.iter().map(|l| l.words.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Case-insensitive substring search across all line text.
    pub fn contains(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        !q.is_empty()
            && self
                .lines
                .iter()
                .any(|l| l.text.to_lowercase().contains(&q))
    }

    /// Mean word confidence, or 0.0 when there are no words.
    pub fn mean_confidence(&self) -> f32 {
        let mut sum = 0.0;
        let mut n = 0u32;
        for line in &self.lines {
            for w in &line.words {
                sum += w.confidence;
                n += 1;
            }
        }
        if n == 0 {
            0.0
        } else {
            sum / n as f32
        }
    }
}

fn union_bbox<I: Iterator<Item = Rect>>(mut it: I) -> Rect {
    match it.next() {
        Some(first) => it.fold(first, |acc, r| acc.union(&r)),
        None => Rect::new(0.0, 0.0, 0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(text: &str, x: f64, y: f64, ww: f64, h: f64) -> OcrWord {
        OcrWord {
            text: text.to_string(),
            bbox: Rect::new(x, y, ww, h),
            confidence: 0.9,
        }
    }

    #[test]
    fn line_from_words_joins_and_unions() {
        let line = OcrLine::from_words(vec![
            w("hello", 0.0, 0.0, 50.0, 10.0),
            w("world", 60.0, 0.0, 50.0, 10.0),
        ]);
        assert_eq!(line.text, "hello world");
        assert_eq!(line.bbox, Rect::new(0.0, 0.0, 110.0, 10.0));
    }

    #[test]
    fn plain_text_joins_lines() {
        let r = OcrResult::new(vec![
            OcrLine::from_words(vec![w("first", 0.0, 0.0, 10.0, 10.0)]),
            OcrLine::from_words(vec![w("second", 0.0, 20.0, 10.0, 10.0)]),
        ]);
        assert_eq!(r.plain_text(), "first\nsecond");
        assert_eq!(r.word_count(), 2);
    }

    #[test]
    fn contains_is_case_insensitive() {
        let r = OcrResult::new(vec![OcrLine::from_words(vec![w(
            "Invoice", 0.0, 0.0, 10.0, 10.0,
        )])]);
        assert!(r.contains("invoice"));
        assert!(r.contains("VOIC"));
        assert!(!r.contains("paid"));
        assert!(!r.contains(""));
    }

    #[test]
    fn mean_confidence_average() {
        let mut a = w("a", 0.0, 0.0, 1.0, 1.0);
        a.confidence = 0.8;
        let mut b = w("b", 0.0, 0.0, 1.0, 1.0);
        b.confidence = 0.6;
        let r = OcrResult::new(vec![OcrLine::from_words(vec![a, b])]);
        assert!((r.mean_confidence() - 0.7).abs() < 1e-6);
        assert_eq!(OcrResult::default().mean_confidence(), 0.0);
    }
}
