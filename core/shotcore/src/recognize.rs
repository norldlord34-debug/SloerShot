//! Recognition helpers: barcode/QR payload classification and link extraction from
//! OCR text. QR symbols are now decoded in-core by the qrcode module (pure Rust);
//! this module classifies the decoded string and pulls links out of recognized text.
use serde::{Deserialize, Serialize};

/// The semantic kind of a decoded barcode/QR payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadKind {
 Url,
 Email,
 Phone,
 Wifi,
 Geo,
 Text,
}

/// A decoded barcode/QR payload plus its classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarcodePayload {
 pub raw: String,
 pub kind: PayloadKind,
}

impl BarcodePayload {
 pub fn new(raw: impl Into<String>) -> Self {
 let raw = raw.into();
 let kind = classify(&raw);
 Self { raw, kind }
 }
}

/// Classify a decoded payload string by its scheme/shape.
pub fn classify(raw: &str) -> PayloadKind {
 let t = raw.trim();
 let lower = t.to_ascii_lowercase();
 if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.") {
 PayloadKind::Url
 } else if lower.starts_with("mailto:") || (t.contains('@') && t.contains('.') && !t.contains(' ')) {
 PayloadKind::Email
 } else if lower.starts_with("tel:") || lower.starts_with("sms:") {
 PayloadKind::Phone
 } else if lower.starts_with("wifi:") {
 PayloadKind::Wifi
 } else if lower.starts_with("geo:") {
 PayloadKind::Geo
 } else {
 PayloadKind::Text
 }
}

/// Extract http(s) and www links from a block of recognized text.
pub fn extract_links(text: &str) -> Vec<String> {
 let mut out = Vec::new();
 for tokenraw in text.split(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '"') {
 let token = tokenraw.trim_matches(|c: char| matches!(c, '.' | ',' | ';' | ')' | '(' | '[' | ']'));
 let lower = token.to_ascii_lowercase();
 let is_link = lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.");
 if is_link && token.len() > 4 && !out.iter().any(|x| x == token) {
 out.push(token.to_string());
 }
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn classifies_payloads() {
 assert_eq!(classify("https://sloershot.app"), PayloadKind::Url);
 assert_eq!(classify("www.example.com"), PayloadKind::Url);
 assert_eq!(classify("mailto:a@b.com"), PayloadKind::Email);
 assert_eq!(classify("a@b.com"), PayloadKind::Email);
 assert_eq!(classify("tel:+15551234"), PayloadKind::Phone);
 assert_eq!(classify("WIFI:S:net;T:WPA;P:pass;;"), PayloadKind::Wifi);
 assert_eq!(classify("geo:37.33,-122.03"), PayloadKind::Geo);
 assert_eq!(classify("just some text"), PayloadKind::Text);
 }

 #[test]
 fn payload_new_sets_kind() {
 let p = BarcodePayload::new("https://x.io");
 assert_eq!(p.kind, PayloadKind::Url);
 }

 #[test]
 fn extracts_and_dedups_links() {
 let text = "see https://a.com and (https://a.com) plus www.b.org. not aplain word";
 let links = extract_links(text);
 assert_eq!(links, vec!["https://a.com".to_string(), "www.b.org".to_string()]);
 }

 #[test]
 fn no_links_returns_empty() {
 assert!(extract_links("nothing to see here").is_empty());
 }
}
