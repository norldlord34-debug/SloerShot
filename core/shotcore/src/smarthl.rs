//! Smart highlighter: snap a rough highlight selection to the OCR word boxes it
//! overlaps, so highlights hug the text exactly (CleanShot Smart Highlighter).
use crate::geometry::Rect;
use crate::ocr::OcrResult;

/// Union of all OCR word boxes that overlap `selection` at all. None when the
/// selection covers no words (caller keeps the freehand rect).
pub fn snap_highlight(ocr: &OcrResult, selection: Rect) -> Option<Rect> {
 snap_highlight_min(ocr, selection, 0.0)
}

/// Like `snap_highlight` but only includes a word when the overlap covers at least
/// `min_overlap_frac` (0.0..=1.0) of that word box area.
pub fn snap_highlight_min(ocr: &OcrResult, selection: Rect, min_overlap_frac: f64) -> Option<Rect> {
 let mut acc: Option<Rect> = None;
 for line in &ocr.lines {
 for word in &line.words {
 if let Some(inter) = selection.intersection(&word.bbox) {
 let wa = word.bbox.area();
 let frac = if wa > 0.0 { inter.area() / wa } else { 0.0 };
 if frac >= min_overlap_frac {
 acc = Some(match acc {
 Some(r) => r.union(&word.bbox),
 None => word.bbox,
 });
 }
 }
 }
 }
 acc
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::ocr::{OcrLine, OcrResult, OcrWord};

 fn word(text: &str, x: f64, y: f64, w: f64, h: f64) -> OcrWord {
 OcrWord { text: text.to_string(), bbox: Rect::new(x, y, w, h), confidence: 0.99 }
 }

 fn sample() -> OcrResult {
 let words = vec![word("Hello", 10.0, 10.0, 40.0, 12.0), word("World", 55.0, 10.0, 40.0, 12.0)];
 OcrResult::new(vec![OcrLine::from_words(words)])
 }

 #[test]
 fn snaps_to_overlapping_words() {
 let ocr = sample();
 // selection grazes both words; union spans from x=10 to x=95
 let snapped = snap_highlight(&ocr, Rect::new(0.0, 12.0, 100.0, 4.0)).unwrap();
 assert_eq!(snapped.x, 10.0);
 assert_eq!(snapped.right(), 95.0);
 }

 #[test]
 fn snaps_to_single_word() {
 let ocr = sample();
 let snapped = snap_highlight(&ocr, Rect::new(12.0, 12.0, 5.0, 4.0)).unwrap();
 assert_eq!(snapped.x, 10.0);
 assert_eq!(snapped.right(), 50.0);
 }

 #[test]
 fn no_overlap_returns_none() {
 let ocr = sample();
 assert!(snap_highlight(&ocr, Rect::new(0.0, 100.0, 10.0, 10.0)).is_none());
 }

 #[test]
 fn min_overlap_filters_grazing_words() {
 let ocr = sample();
 // a sliver overlap on the first word only
 let sel = Rect::new(48.0, 10.0, 3.0, 12.0);
 assert!(snap_highlight_min(&ocr, sel, 0.5).is_none());
 }
}
