//! Shape-mask crop: knock out the alpha outside an ellipse or rounded rectangle so a
//! capture can be cropped to a non-rectangular shape (then beautified/pinned).
use image::RgbaImage;
use serde::{Deserialize, Serialize};

/// The mask shape, inscribed in the image bounds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskShape {
 Ellipse,
 RoundedRect { radius: f32 },
}

fn in_ellipse(x: u32, y: u32, w: u32, h: u32) -> bool {
 let (rx, ry) = (w as f64 / 2.0, h as f64 / 2.0);
 if rx <= 0.0 || ry <= 0.0 {
 return false;
 }
 let nx = (x as f64 + 0.5 - rx) / rx;
 let ny = (y as f64 + 0.5 - ry) / ry;
 nx * nx + ny * ny <= 1.0
}

fn in_rounded_rect(x: u32, y: u32, w: u32, h: u32, radius: f32) -> bool {
 let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
 let r = (radius as f64).min(w as f64 / 2.0).min(h as f64 / 2.0).max(0.0);
 let cx = if px < r {
 r
 } else if px > w as f64 - r {
 w as f64 - r
 } else {
 px
 };
 let cy = if py < r {
 r
 } else if py > h as f64 - r {
 h as f64 - r
 } else {
 py
 };
 let (dx, dy) = (px - cx, py - cy);
 dx * dx + dy * dy <= r * r + 1e-9
}

/// Return a copy of `img` with pixels outside `shape` made fully transparent.
pub fn apply_mask(img: &RgbaImage, shape: MaskShape) -> RgbaImage {
 let (w, h) = (img.width(), img.height());
 let mut out = img.clone();
 for y in 0..h {
 for x in 0..w {
 let inside = match shape {
 MaskShape::Ellipse => in_ellipse(x, y, w, h),
 MaskShape::RoundedRect { radius } => in_rounded_rect(x, y, w, h, radius),
 };
 if !inside {
 let mut px = *out.get_pixel(x, y);
 px.0[3] = 0;
 out.put_pixel(x, y, px);
 }
 }
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 fn white(w: u32, h: u32) -> RgbaImage {
 RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
 }

 #[test]
 fn ellipse_clears_corners_keeps_center() {
 let out = apply_mask(&white(10, 10), MaskShape::Ellipse);
 assert_eq!(out.get_pixel(0, 0).0[3], 0);
 assert_eq!(out.get_pixel(9, 9).0[3], 0);
 assert_eq!(out.get_pixel(5, 5).0[3], 255);
 }

 #[test]
 fn rounded_rect_clears_only_corners() {
 let out = apply_mask(&white(10, 10), MaskShape::RoundedRect { radius: 4.0 });
 assert_eq!(out.get_pixel(0, 0).0[3], 0);
 // edge midpoints stay opaque
 assert_eq!(out.get_pixel(0, 5).0[3], 255);
 assert_eq!(out.get_pixel(5, 0).0[3], 255);
 assert_eq!(out.get_pixel(5, 5).0[3], 255);
 }

 #[test]
 fn zero_radius_keeps_everything() {
 let out = apply_mask(&white(4, 4), MaskShape::RoundedRect { radius: 0.0 });
 for y in 0..4 {
 for x in 0..4 {
 assert_eq!(out.get_pixel(x, y).0[3], 255);
 }
 }
 }
}
