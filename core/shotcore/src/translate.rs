//! OCR translation contract: translate recognized text on-device. The core defines
//! the request/result shape and a Translator trait; native code plugs in a real
//! engine, while a dictionary translator backs tests and offline word maps.
use crate::ocr::OcrResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationRequest {
 pub text: String,
 /// Source language code, or None to auto-detect.
 pub source: Option<String>,
 pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationResult {
 pub text: String,
 pub source: String,
 pub target: String,
}

/// Pluggable translation engine.
pub trait Translator {
 fn translate(&self, req: &TranslationRequest) -> TranslationResult;
}

/// Translate each OCR line, returning one result per line.
pub fn translate_lines(ocr: &OcrResult, t: &dyn Translator, source: Option<&str>, target: &str) -> Vec<TranslationResult> {
 ocr.lines
 .iter()
 .map(|l| {
 t.translate(&TranslationRequest {
 text: l.text.clone(),
 source: source.map(|s| s.to_string()),
 target: target.to_string(),
 })
 })
 .collect()
}

/// A word-dictionary translator: deterministic and fully offline.
pub struct WordMapTranslator {
 pub map: HashMap<String, String>,
 pub source: String,
}

impl Translator for WordMapTranslator {
 fn translate(&self, req: &TranslationRequest) -> TranslationResult {
 let text = req
 .text
 .split_whitespace()
 .map(|w| self.map.get(w).cloned().unwrap_or_else(|| w.to_string()))
 .collect::<Vec<_>>()
 .join(" ");
 TranslationResult {
 text,
 source: req.source.clone().unwrap_or_else(|| self.source.clone()),
 target: req.target.clone(),
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::ocr::{OcrLine, OcrResult, OcrWord};
 use crate::geometry::Rect;

 fn es() -> WordMapTranslator {
 let mut map = HashMap::new();
 map.insert("hello".to_string(), "hola".to_string());
 map.insert("world".to_string(), "mundo".to_string());
 WordMapTranslator { map, source: "en".to_string() }
 }

 #[test]
 fn translates_known_words_keeps_unknown() {
 let r = es().translate(&TranslationRequest { text: "hello bright world".to_string(), source: None, target: "es".to_string() });
 assert_eq!(r.text, "hola bright mundo");
 assert_eq!(r.source, "en");
 assert_eq!(r.target, "es");
 }

 #[test]
 fn translates_ocr_lines() {
 let w = OcrWord { text: "hello".to_string(), bbox: Rect::new(0.0, 0.0, 1.0, 1.0), confidence: 1.0 };
 let ocr = OcrResult::new(vec![OcrLine::from_words(vec![w])]);
 let out = translate_lines(&ocr, &es(), Some("en"), "es");
 assert_eq!(out.len(), 1);
 assert_eq!(out[0].text, "hola");
 }
}
