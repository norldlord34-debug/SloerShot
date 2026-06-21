//! Color correction: gray-world white balance and per-channel auto-contrast (levels stretch).
//! Pure pixel math, tested.
use image::{Rgba, RgbaImage};

/// Gray-world white balance: scale each channel so its mean matches the overall gray mean,
// neutralizing a global color cast.
pub fn gray_world(img: &RgbaImage) -> RgbaImage {
 let (w, h) = img.dimensions();
 let n = (w * h).max(1) as f64;
 let (mut sr, mut sg, mut sb) = (0f64, 0f64, 0f64);
 for p in img.pixels() {
 sr += p.0[0] as f64;
 sg += p.0[1] as f64;
 sb += p.0[2] as f64;
 }
 let (ar, ag, ab) = (sr / n, sg / n, sb / n);
 let gray = (ar + ag + ab) / 3.0;
 let scale = |avg: f64| if avg > 0.001 { gray / avg } else { 1.0 };
 let (kr, kg, kb) = (scale(ar), scale(ag), scale(ab));
 let mut out = RgbaImage::new(w, h);
 for (x, y, p) in img.enumerate_pixels() {
 let r = (p.0[0] as f64 * kr).clamp(0.0, 255.0) as u8;
 let g = (p.0[1] as f64 * kg).clamp(0.0, 255.0) as u8;
 let b = (p.0[2] as f64 * kb).clamp(0.0, 255.0) as u8;
 out.put_pixel(x, y, Rgba([r, g, b, p.0[3]]));
 }
 out
}

/// Per-channel auto-contrast: stretch each channel min..max to the full 0..255 range.
pub fn auto_contrast(img: &RgbaImage) -> RgbaImage {
 let (w, h) = img.dimensions();
 let mut mn = [255u8; 3];
 let mut mx = [0u8; 3];
 for p in img.pixels() {
 for c in 0..3 {
 mn[c] = mn[c].min(p.0[c]);
 mx[c] = mx[c].max(p.0[c]);
 }
 }
 let mut out = RgbaImage::new(w, h);
 for (x, y, p) in img.enumerate_pixels() {
 let mut px = [0u8; 4];
 for c in 0..3 {
 let range = mx[c].saturating_sub(mn[c]);
 px[c] = if range == 0 {
 p.0[c]
 } else {
 (((p.0[c] - mn[c]) as u32 * 255) / range as u32) as u8
 };
 }
 px[3] = p.0[3];
 out.put_pixel(x, y, Rgba(px));
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn gray_world_neutralizes_cast() {
 let img = RgbaImage::from_pixel(8, 8, Rgba([200, 100, 100, 255]));
 let out = gray_world(&img);
 let p = out.get_pixel(0, 0).0;
 assert!((p[0] as i32 - p[1] as i32).abs() <= 4 && (p[1] as i32 - p[2] as i32).abs() <= 4, "got {:?}", p);
 }

 #[test]
 fn auto_contrast_stretches() {
 let mut img = RgbaImage::new(10, 4);
 for y in 0..4u32 {
 for x in 0..10u32 {
 let v = if x < 5 { 100 } else { 150 };
 img.put_pixel(x, y, Rgba([v, v, v, 255]));
 }
 }
 let out = auto_contrast(&img);
 assert_eq!(out.get_pixel(0, 0).0[0], 0);
 assert_eq!(out.get_pixel(9, 0).0[0], 255);
 }
}
