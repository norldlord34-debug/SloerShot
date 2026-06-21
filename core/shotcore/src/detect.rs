//! On-device sensitive-data detection and auto-redaction.
//!
//! Scans OCR output for sensitive tokens (emails, credit-card numbers validated
//! with the Luhn checksum, phone numbers, IPv4 addresses, and API keys or
//! tokens) and emits Redact annotations over the matching word boxes. Pure Rust,
//! no dependencies, so it runs fully on-device with no cloud round-trip.

use crate::geometry::Rect;
use crate::model::{Annotation, RedactStyle, ShapeKind};
use crate::ocr::OcrResult;
use serde::{Deserialize, Serialize};

/// A category of sensitive data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sensitive {
    Email,
    CreditCard,
    Phone,
    IpV4,
    Token,
}

fn digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn luhn_ok(d: &str) -> bool {
    let mut sum = 0u32;
    let mut alt = false;
    for ch in d.chars().rev() {
        let mut n = match ch.to_digit(10) {
            Some(v) => v,
            None => return false,
        };
        if alt {
            n *= 2;
            if n > 9 {
                n -= 9;
            }
        }
        sum += n;
        alt = !alt;
    }
    sum % 10 == 0
}

fn is_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split("@").collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty()
        || domain.len() < 3
        || !domain.contains(".")
        || domain.starts_with(".")
        || domain.ends_with(".")
    {
        return false;
    }
    local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c))
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-.".contains(c))
}

fn is_credit_card(s: &str) -> bool {
    if s.chars().any(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    let d = digits(s);
    d.len() >= 13 && d.len() <= 19 && luhn_ok(&d)
}

fn is_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split(".").collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| {
        !p.is_empty()
            && p.len() <= 3
            && p.chars().all(|c| c.is_ascii_digit())
            && p.parse::<u32>().map(|n| n <= 255).unwrap_or(false)
    })
}

fn is_token(s: &str) -> bool {
    let prefixes = [
        "sk-",
        "sk_live_",
        "sk_test_",
        "pk_live_",
        "ghp_",
        "gho_",
        "github_pat_",
        "xoxb-",
        "xoxp-",
        "AKIA",
        "AIza",
        "ya29.",
        "eyJ",
    ];
    if prefixes.iter().any(|pre| s.starts_with(*pre)) {
        return s.len() >= 8;
    }
    if s.len() >= 24 {
        let body_ok = s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_/+=.".contains(c));
        let has_alpha = s.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = s.chars().any(|c| c.is_ascii_digit());
        return body_ok && has_alpha && has_digit;
    }
    false
}

fn is_phone(s: &str) -> bool {
    if !s
        .chars()
        .all(|c| c.is_ascii_digit() || "+()-. ".contains(c))
    {
        return false;
    }
    let n = digits(s).len();
    (7..=15).contains(&n)
}

/// Classify a single word, returning its sensitive category if any.
pub fn classify_word(word: &str) -> Option<Sensitive> {
    let w = word
        .trim()
        .trim_matches(|c: char| "()[]{}<>,;:".contains(c));
    if w.is_empty() {
        return None;
    }
    if is_email(w) {
        Some(Sensitive::Email)
    } else if is_credit_card(w) {
        Some(Sensitive::CreditCard)
    } else if is_ipv4(w) {
        Some(Sensitive::IpV4)
    } else if is_token(w) {
        Some(Sensitive::Token)
    } else if is_phone(w) {
        Some(Sensitive::Phone)
    } else {
        None
    }
}

/// Detect sensitive words in an OCR result. An empty `categories` slice matches all.
pub fn detect_in_ocr(ocr: &OcrResult, categories: &[Sensitive]) -> Vec<(Sensitive, Rect)> {
    let mut out = Vec::new();
    for line in &ocr.lines {
        for w in &line.words {
            if let Some(cat) = classify_word(&w.text) {
                if categories.is_empty() || categories.contains(&cat) {
                    out.push((cat, w.bbox));
                }
            }
        }
    }
    out
}

/// Build Redact annotations over every detected sensitive word box.
pub fn auto_redact(
    ocr: &OcrResult,
    categories: &[Sensitive],
    style: RedactStyle,
    strength: u32,
) -> Vec<Annotation> {
    detect_in_ocr(ocr, categories)
        .into_iter()
        .map(|(_, rect)| {
            Annotation::new(ShapeKind::Redact {
                rect,
                style,
                strength,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocr::{OcrLine, OcrResult, OcrWord};

    #[test]
    fn detects_email() {
        assert_eq!(classify_word("alice@example.com"), Some(Sensitive::Email));
        assert_eq!(classify_word("alice@example.com,"), Some(Sensitive::Email));
        assert_eq!(classify_word("not-an-email"), None);
        assert_eq!(classify_word("a@b"), None);
    }

    #[test]
    fn detects_credit_card_with_luhn() {
        assert_eq!(
            classify_word("4242 4242 4242 4242"),
            Some(Sensitive::CreditCard)
        );
        assert_eq!(
            classify_word("4242424242424242"),
            Some(Sensitive::CreditCard)
        );
        assert_ne!(
            classify_word("4242 4242 4242 4243"),
            Some(Sensitive::CreditCard)
        );
    }

    #[test]
    fn detects_ipv4() {
        assert_eq!(classify_word("192.168.0.1"), Some(Sensitive::IpV4));
        assert_eq!(classify_word("10.0.0.255"), Some(Sensitive::IpV4));
        assert_eq!(classify_word("999.1.1.1"), None);
    }

    #[test]
    fn detects_tokens() {
        assert_eq!(
            classify_word("sk_live_abc123def456ghi789"),
            Some(Sensitive::Token)
        );
        assert_eq!(
            classify_word("AKIAIOSFODNN7EXAMPLE"),
            Some(Sensitive::Token)
        );
        assert_eq!(
            classify_word("ghp_0123456789abcdef"),
            Some(Sensitive::Token)
        );
        assert_eq!(classify_word("hello"), None);
    }

    #[test]
    fn detects_phone() {
        assert_eq!(classify_word("+14155552671"), Some(Sensitive::Phone));
        assert_eq!(classify_word("2024"), None);
    }

    #[test]
    fn auto_redact_over_ocr_with_filter() {
        let line = OcrLine::from_words(vec![
            OcrWord {
                text: "Contact".to_string(),
                bbox: Rect::new(0.0, 0.0, 50.0, 10.0),
                confidence: 0.9,
            },
            OcrWord {
                text: "alice@example.com".to_string(),
                bbox: Rect::new(60.0, 0.0, 120.0, 10.0),
                confidence: 0.9,
            },
        ]);
        let ocr = OcrResult::new(vec![line]);
        let all = auto_redact(&ocr, &[], RedactStyle::Pixelate, 12);
        assert_eq!(all.len(), 1);
        match &all[0].kind {
            ShapeKind::Redact { rect, .. } => assert_eq!(*rect, Rect::new(60.0, 0.0, 120.0, 10.0)),
            _ => panic!("expected redact"),
        }
        assert_eq!(
            auto_redact(&ocr, &[Sensitive::CreditCard], RedactStyle::Pixelate, 12).len(),
            0
        );
        assert_eq!(
            auto_redact(&ocr, &[Sensitive::Email], RedactStyle::Blur, 8).len(),
            1
        );
    }
}
