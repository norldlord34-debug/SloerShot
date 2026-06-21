//! Auto-deskew: estimate the skew angle of a tilted capture and rotate it straight. The angle
//! is found by maximizing a horizontal projection-profile score (aligned content concentrates
//! dark pixels into few rows) over candidate angles. Pure math + a nearest-neighbor rotation
//! about the image center. Rotations compose additively, so the estimate and the applied
//! correction are self-consistent.
use image::{Rgba, RgbaImage};
use std::f64::consts::PI;

/// Rotate an image by `degrees` about its center (nearest-neighbor). Out-of-source areas
/// become white; the output keeps the source dimensions.
pub fn rotate(img: &RgbaImage, degrees: f64) -> RgbaImage {
 let (w, h) = img.dimensions();
 let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
 let rad = degrees * PI / 180.0;
 let (s, c) = (rad.sin(), rad.cos());
 let mut out = RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]));
 for y in 0..h {
 for x in 0..w {
 let dx = x as f64 - cx;
 let dy = y as f64 - cy;
 let sx = cx + dx * c + dy * s;
 let sy = cy - dx * s + dy * c;
 if sx >= 0.0 && sy >= 0.0 {
 let (ix, iy) = (sx.round() as i64, sy.round() as i64);
 if ix >= 0 && iy >= 0 && (ix as u32) < w && (iy as u32) < h {
 out.put_pixel(x, y, *img.get_pixel(ix as u32, iy as u32));
 }
 }
 }
 }
 out
}

fn for_estimate(img: &RgbaImage) -> RgbaImage {
 let (w, h) = img.dimensions();
 if w <= 200 {
 img.clone()
 } else {
 let nh = ((h as f64) * 200.0 / w as f64).max(1.0) as u32;
 image::imageops::resize(img, 200, nh, image::imageops::FilterType::Triangle)
 }
}

fn projection_score(img: &RgbaImage) -> f64 {
 let (w, h) = img.dimensions();
 let mut score = 0.0;
 for y in 0..h {
 let mut count = 0u32;
 for x in 0..w {
 let p = img.get_pixel(x, y).0;
 let l = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
 if l < 128.0 {
 count += 1;
 }
 }
 score += count as f64 * count as f64;
 }
 score
}

/// Estimate the skew correction angle (degrees) that best straightens the content. Apply it
/// with rotate(img, angle).
pub fn estimate_skew_degrees(img: &RgbaImage) -> f64 {
 let small = for_estimate(img);
 let mut best_a = 0.0;
 let mut best_score = -1.0;
 let mut a = -15.0;
 while a <= 15.0 + 1e-9 {
 let score = projection_score(&rotate(&small, a));
 if score > best_score {
 best_score = score;
 best_a = a;
 }
 a += 0.5;
 }
 best_a
}

/// Straighten a tilted capture by estimating and applying the skew correction.
pub fn deskew(img: &RgbaImage) -> RgbaImage {
 rotate(img, estimate_skew_degrees(img))
}

#[cfg(test)]
mod tests {
 use super::*;

 fn bar() -> RgbaImage {
 let mut img = RgbaImage::from_pixel(80, 80, Rgba([255, 255, 255, 255]));
 for y in 36..44u32 {
 for x in 10..70u32 {
 img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 img
 }

 #[test]
 fn recovers_known_tilt() {
 let tilted = rotate(&bar(), 6.0);
 let est = estimate_skew_degrees(&tilted);
 assert!((est + 6.0).abs() <= 2.0, "estimate {} should be near -6", est);
 }

 #[test]
 fn straight_image_needs_little_correction() {
 let est = estimate_skew_degrees(&bar());
 assert!(est.abs() <= 1.0, "estimate {} should be near 0", est);
 }
}
