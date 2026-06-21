//! Color palette: default + recent swatches, opacity, and eyedropper sampling.
//!
//! Backs the annotation color picker (custom colors, opacity slider, recent colors)
//! and the eyedropper, sampling a pixel straight out of an RGBA8 image buffer.
use crate::model::Color;
use serde::{Deserialize, Serialize};

/// Maximum remembered recent colors.
pub const MAX_RECENT: usize = 12;

/// A color palette: fixed default swatches plus a recent-colors ring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
 pub recent: Vec<Color>,
}

impl Default for Palette {
 fn default() -> Self {
 Self { recent: Vec::new() }
 }
}

impl Palette {
 /// The built-in semantic swatches shown first in the picker.
 pub fn defaults() -> [Color; 8] {
 [
 Color::RED,
 Color::ORANGE,
 Color::YELLOW,
 Color::GREEN,
 Color::BLUE,
 Color::PURPLE,
 Color::BLACK,
 Color::WHITE,
 ]
 }

 /// Record a freshly used color: most-recent-first, de-duplicated, capped.
 pub fn push_recent(&mut self, color: Color) {
 self.recent.retain(|c| *c != color);
 self.recent.insert(0, color);
 if self.recent.len() > MAX_RECENT {
 self.recent.truncate(MAX_RECENT);
 }
 }
}

/// Apply an opacity fraction (0.0..=1.0) to a color, returning a new color whose
/// alpha is scaled. Values outside the range are clamped.
pub fn with_opacity(color: Color, opacity: f32) -> Color {
 let f = opacity.clamp(0.0, 1.0);
 let a = (color.a as f32 * f).round() as u8;
 color.with_alpha(a)
}

/// Sample one RGBA8 pixel from a tightly packed buffer (row-major, 4 bytes/px).
/// Returns None when out of bounds or the buffer is too small.
pub fn eyedrop(rgba: &[u8], width: u32, height: u32, x: u32, y: u32) -> Option<Color> {
 if x >= width || y >= height {
 return None;
 }
 let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
 if idx + 3 >= rgba.len() {
 return None;
 }
 Some(Color::rgba(rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]))
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn defaults_have_eight_swatches() {
 assert_eq!(Palette::defaults().len(), 8);
 assert_eq!(Palette::defaults()[0], Color::RED);
 }

 #[test]
 fn recent_is_most_recent_first_and_dedups() {
 let mut p = Palette::default();
 p.push_recent(Color::RED);
 p.push_recent(Color::BLUE);
 p.push_recent(Color::RED);
 assert_eq!(p.recent, vec![Color::RED, Color::BLUE]);
 }

 #[test]
 fn recent_is_capped() {
 let mut p = Palette::default();
 for i in 0..(MAX_RECENT as u8 + 5) {
 p.push_recent(Color::rgb(i, 0, 0));
 }
 assert_eq!(p.recent.len(), MAX_RECENT);
 }

 #[test]
 fn opacity_scales_alpha() {
 let c = with_opacity(Color::rgba(10, 20, 30, 200), 0.5);
 assert_eq!(c.a, 100);
 let full = with_opacity(Color::rgba(10, 20, 30, 200), 2.0);
 assert_eq!(full.a, 200);
 let none = with_opacity(Color::rgba(10, 20, 30, 200), -1.0);
 assert_eq!(none.a, 0);
 }

 #[test]
 fn eyedrop_reads_pixel() {
 // 2x1 image: red, green
 let buf = vec![255u8, 0, 0, 255, 0, 255, 0, 255];
 assert_eq!(eyedrop(&buf, 2, 1, 0, 0), Some(Color::rgba(255, 0, 0, 255)));
 assert_eq!(eyedrop(&buf, 2, 1, 1, 0), Some(Color::rgba(0, 255, 0, 255)));
 assert_eq!(eyedrop(&buf, 2, 1, 2, 0), None);
 }
}
