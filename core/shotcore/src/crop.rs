//! Crop tool: aspect-ratio presets, ratio locking, edge snapping, and canvas expand.
//!
//! Pure geometry mirroring the CleanShot crop experience (aspect ratios incl 5:4 and
//! 9:16, snapping to image edges, and expanding the canvas with a background fill).
use crate::geometry::Rect;
use serde::{Deserialize, Serialize};

/// Selectable crop aspect ratios. Custom carries width/height.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AspectRatio {
 Free,
 Square,
 R4x3,
 R3x2,
 R16x9,
 R5x4,
 R9x16,
 Custom { w: f64, h: f64 },
}

impl AspectRatio {
 /// The width/height value, or None when unconstrained.
 pub fn value(&self) -> Option<f64> {
 match self {
 AspectRatio::Free => None,
 AspectRatio::Square => Some(1.0),
 AspectRatio::R4x3 => Some(4.0 / 3.0),
 AspectRatio::R3x2 => Some(3.0 / 2.0),
 AspectRatio::R16x9 => Some(16.0 / 9.0),
 AspectRatio::R5x4 => Some(5.0 / 4.0),
 AspectRatio::R9x16 => Some(9.0 / 16.0),
 AspectRatio::Custom { w, h } if *h > 0.0 => Some(w / h),
 AspectRatio::Custom { .. } => None,
 }
 }

 /// All presets in toolbar order (Custom excluded).
 pub fn presets() -> [AspectRatio; 7] {
 [
 AspectRatio::Free,
 AspectRatio::Square,
 AspectRatio::R4x3,
 AspectRatio::R3x2,
 AspectRatio::R16x9,
 AspectRatio::R5x4,
 AspectRatio::R9x16,
 ]
 }
}

/// Adjust a selection so width/height match the ratio, holding the top-left corner.
/// The shorter dimension is preserved and the other recomputed.
pub fn constrain(rect: Rect, ratio: AspectRatio) -> Rect {
 let r = match ratio.value() {
 Some(v) if v > 0.0 => v,
 _ => return rect,
 };
 let cur = if rect.h != 0.0 { rect.w / rect.h } else { 0.0 };
 if cur > r {
 // too wide: shrink width to match
 let w = rect.h * r;
 Rect::new(rect.x, rect.y, w, rect.h)
 } else {
 // too tall: shrink height to match
 let h = if r != 0.0 { rect.w / r } else { rect.h };
 Rect::new(rect.x, rect.y, rect.w, h)
 }
}

/// Snap edges of `rect` to `bounds` when within `threshold` pixels.
pub fn snap_to_bounds(rect: Rect, bounds: Rect, threshold: f64) -> Rect {
 let mut x = rect.x;
 let mut y = rect.y;
 let mut w = rect.w;
 let mut h = rect.h;
 if (x - bounds.x).abs() <= threshold {
 w += x - bounds.x;
 x = bounds.x;
 }
 if (y - bounds.y).abs() <= threshold {
 h += y - bounds.y;
 y = bounds.y;
 }
 if (rect.right() - bounds.right()).abs() <= threshold {
 w = bounds.right() - x;
 }
 if (rect.bottom() - bounds.bottom()).abs() <= threshold {
 h = bounds.bottom() - y;
 }
 Rect::new(x, y, w, h)
}

/// Clamp a crop rect to the image and return integer-friendly pixel bounds.
pub fn apply(image_w: u32, image_h: u32, crop: Rect) -> Rect {
 let bounds = Rect::new(0.0, 0.0, image_w as f64, image_h as f64);
 crop.normalized().clamp_to(&bounds).round_out()
}

/// Expand the canvas to `new_bounds`, returning the offset where existing content
/// (size content_w x content_h) should be placed for the given alignment fractions
/// (ax, ay in 0.0..=1.0; 0.5 = centered).
pub fn expand_offset(content_w: f64, content_h: f64, new_w: f64, new_h: f64, ax: f64, ay: f64) -> (f64, f64) {
 let ox = (new_w - content_w) * ax.clamp(0.0, 1.0);
 let oy = (new_h - content_h) * ay.clamp(0.0, 1.0);
 (ox.max(0.0), oy.max(0.0))
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn ratio_values() {
 assert_eq!(AspectRatio::Free.value(), None);
 assert_eq!(AspectRatio::Square.value(), Some(1.0));
 assert_eq!(AspectRatio::R16x9.value(), Some(16.0 / 9.0));
 assert_eq!(AspectRatio::R9x16.value(), Some(9.0 / 16.0));
 assert_eq!(AspectRatio::Custom { w: 3.0, h: 0.0 }.value(), None);
 assert_eq!(AspectRatio::presets().len(), 7);
 }

 #[test]
 fn constrain_shrinks_wider_selection() {
 let r = constrain(Rect::new(0.0, 0.0, 200.0, 100.0), AspectRatio::Square);
 assert!((r.w - r.h).abs() < 1e-9);
 assert_eq!(r.h, 100.0);
 assert_eq!(r.w, 100.0);
 }

 #[test]
 fn constrain_shrinks_taller_selection() {
 let r = constrain(Rect::new(0.0, 0.0, 100.0, 400.0), AspectRatio::Square);
 assert_eq!(r.w, 100.0);
 assert_eq!(r.h, 100.0);
 }

 #[test]
 fn constrain_free_is_identity() {
 let r = Rect::new(1.0, 2.0, 3.0, 4.0);
 assert_eq!(constrain(r, AspectRatio::Free), r);
 }

 #[test]
 fn snapping_pulls_to_edges() {
 let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
 let r = snap_to_bounds(Rect::new(2.0, 3.0, 90.0, 90.0), bounds, 5.0);
 assert_eq!(r.x, 0.0);
 assert_eq!(r.y, 0.0);
 }

 #[test]
 fn snapping_ignores_far_edges() {
 let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
 let r = snap_to_bounds(Rect::new(20.0, 20.0, 30.0, 30.0), bounds, 5.0);
 assert_eq!(r.x, 20.0);
 assert_eq!(r.y, 20.0);
 }

 #[test]
 fn apply_clamps_to_image() {
 let r = apply(100, 100, Rect::new(-10.0, -10.0, 200.0, 50.0));
 assert_eq!(r.x, 0.0);
 assert_eq!(r.y, 0.0);
 assert_eq!(r.w, 100.0);
 assert_eq!(r.h, 40.0);
 }

 #[test]
 fn expand_centers_content() {
 let (ox, oy) = expand_offset(100.0, 100.0, 200.0, 200.0, 0.5, 0.5);
 assert_eq!(ox, 50.0);
 assert_eq!(oy, 50.0);
 }
}
