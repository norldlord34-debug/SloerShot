//! WCAG color-contrast checking for accessible annotations and exports.
//! Implements the WCAG 2.x relative-luminance contrast ratio and AA/AAA rating.
use crate::model::Color;
use serde::{Deserialize, Serialize};

fn channel_linear(c: u8) -> f64 {
 let cs = c as f64 / 255.0;
 if cs <= 0.03928 {
 cs / 12.92
 } else {
 ((cs + 0.055) / 1.055).powf(2.4)
 }
}

/// WCAG relative luminance of a color in 0.0..=1.0.
pub fn relative_luminance(c: Color) -> f64 {
 0.2126 * channel_linear(c.r) + 0.7152 * channel_linear(c.g) + 0.0722 * channel_linear(c.b)
}

/// WCAG contrast ratio between two colors in 1.0..=21.0.
pub fn contrast_ratio(a: Color, b: Color) -> f64 {
 let la = relative_luminance(a);
 let lb = relative_luminance(b);
 let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
 (hi + 0.05) / (lo + 0.05)
}

/// WCAG conformance level for a contrast ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WcagLevel {
 Fail,
 AA,
 AAA,
}

/// Rate a contrast ratio for normal or large text.
pub fn rate(ratio: f64, large_text: bool) -> WcagLevel {
 let (aa, aaa) = if large_text { (3.0, 4.5) } else { (4.5, 7.0) };
 if ratio >= aaa {
 WcagLevel::AAA
 } else if ratio >= aa {
 WcagLevel::AA
 } else {
 WcagLevel::Fail
 }
}

/// Pick black or white as the most readable text color over a background.
pub fn readable_on(bg: Color) -> Color {
 if contrast_ratio(Color::BLACK, bg) >= contrast_ratio(Color::WHITE, bg) {
 Color::BLACK
 } else {
 Color::WHITE
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn black_on_white_is_maximal() {
 let r = contrast_ratio(Color::BLACK, Color::WHITE);
 assert!((r - 21.0).abs() < 0.01);
 assert_eq!(rate(r, false), WcagLevel::AAA);
 }

 #[test]
 fn same_color_fails() {
 let r = contrast_ratio(Color::WHITE, Color::WHITE);
 assert!((r - 1.0).abs() < 1e-9);
 assert_eq!(rate(r, false), WcagLevel::Fail);
 }

 #[test]
 fn ratio_is_symmetric() {
 let a = Color::rgb(30, 136, 229);
 let b = Color::WHITE;
 assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-12);
 }

 #[test]
 fn large_text_threshold_is_lower() {
 // a ratio of 3.5 fails normal but passes AA for large text
 assert_eq!(rate(3.5, false), WcagLevel::Fail);
 assert_eq!(rate(3.5, true), WcagLevel::AA);
 }

 #[test]
 fn readable_text_color() {
 assert_eq!(readable_on(Color::WHITE), Color::BLACK);
 assert_eq!(readable_on(Color::BLACK), Color::WHITE);
 }
}
