//! Unsharp-mask sharpening and the box blur it builds on. sharp = clamp(original + amount *
//! (original - blurred)). Crisps up scaled or soft screenshots. Pure convolution, tested.
use image::{Rgba, RgbaImage};

/// Box blur of the given radius (RGB averaged over the neighborhood; alpha preserved).
/// radius 0 returns a copy.
pub fn box_blur(img: &RgbaImage, radius: u32) -> RgbaImage {
 if radius == 0 {
 return img.clone();
 }
 let (w, h) = img.dimensions();
 let r = radius as i32;
 let mut out = RgbaImage::new(w, h);
 for y in 0..h {
 for x in 0..w {
 let (mut sr, mut sg, mut sb, mut n) = (0u32, 0u32, 0u32, 0u32);
 for dy in -r..=r {
 for dx in -r..=r {
 let nx = x as i32 + dx;
 let ny = y as i32 + dy;
 if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
 let p = img.get_pixel(nx as u32, ny as u32).0;
 sr += p[0] as u32;
 sg += p[1] as u32;
 sb += p[2] as u32;
 n += 1;
 }
 }
 }
 let a = img.get_pixel(x, y).0[3];
 out.put_pixel(x, y, Rgba([(sr / n) as u8, (sg / n) as u8, (sb / n) as u8, a]));
 }
 }
 out
}

/// Unsharp mask: sharp = clamp(original + amount * (original - blurred)). `radius` is the blur
/// radius; `amount` is the strength (e.g. 0.5 to 2.0).
pub fn unsharp(img: &RgbaImage, radius: u32, amount: f32) -> RgbaImage {
 let blurred = box_blur(img, radius.max(1));
 let (w, h) = img.dimensions();
 let mut out = RgbaImage::new(w, h);
 for y in 0..h {
 for x in 0..w {
 let o = img.get_pixel(x, y).0;
 let b = blurred.get_pixel(x, y).0;
 let mut px = [0u8; 4];
 for c in 0..3 {
 let val = o[c] as f32 + amount * (o[c] as f32 - b[c] as f32);
 px[c] = val.clamp(0.0, 255.0) as u8;
 }
 px[3] = o[3];
 out.put_pixel(x, y, Rgba(px));
 }
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 fn step() -> RgbaImage {
 let mut img = RgbaImage::new(40, 40);
 for y in 0..40u32 {
 for x in 0..40u32 {
 let v = if x < 20 { 100 } else { 160 };
 img.put_pixel(x, y, Rgba([v, v, v, 255]));
 }
 }
 img
 }

 #[test]
 fn sharpens_edges_overshoot() {
 let s = unsharp(&step(), 2, 1.0);
 assert!(s.get_pixel(20, 20).0[0] > 160, "right edge should overshoot");
 assert!(s.get_pixel(19, 20).0[0] < 100, "left edge should undershoot");
 let flat = s.get_pixel(2, 20).0[0] as i32;
 assert!((flat - 100).abs() <= 3, "flat region should be unchanged");
 }

 #[test]
 fn box_blur_smooths() {
 let s = box_blur(&step(), 3);
 let edge = s.get_pixel(20, 20).0[0];
 assert!(edge > 100 && edge < 160, "blurred edge should be between the two levels");
 assert_eq!(box_blur(&step(), 0).get_pixel(0, 0).0[0], 100);
 }
}
