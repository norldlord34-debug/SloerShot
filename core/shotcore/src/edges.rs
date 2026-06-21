//! Sobel edge detection: a gradient-magnitude edge map plus a strong-edge count. Pure 3x3
//! convolution over grayscale, used for auto-annotation, smart selection, and guide
//! detection. Tested.
use image::{GrayImage, Luma, RgbaImage};

fn to_gray(img: &RgbaImage) -> GrayImage {
 let (w, h) = img.dimensions();
 let mut g = GrayImage::new(w, h);
 for y in 0..h {
 for x in 0..w {
 let p = img.get_pixel(x, y).0;
 let l = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).round() as u8;
 g.put_pixel(x, y, Luma([l]));
 }
 }
 g
}

/// Sobel gradient magnitude as a grayscale image (0..255 clamped). A 1px border stays 0.
pub fn sobel(img: &RgbaImage) -> GrayImage {
 let gray = to_gray(img);
 let (w, h) = gray.dimensions();
 let mut out = GrayImage::new(w, h);
 if w < 3 || h < 3 {
 return out;
 }
 let at = |x: u32, y: u32| gray.get_pixel(x, y).0[0] as i32;
 for y in 1..h - 1 {
 for x in 1..w - 1 {
 let gx = -at(x - 1, y - 1) - 2 * at(x - 1, y) - at(x - 1, y + 1)
 + at(x + 1, y - 1) + 2 * at(x + 1, y) + at(x + 1, y + 1);
 let gy = -at(x - 1, y - 1) - 2 * at(x, y - 1) - at(x + 1, y - 1)
 + at(x - 1, y + 1) + 2 * at(x, y + 1) + at(x + 1, y + 1);
 let mag = (((gx * gx + gy * gy) as f64).sqrt()).min(255.0) as u8;
 out.put_pixel(x, y, Luma([mag]));
 }
 }
 out
}

/// Count of strong edge pixels (gradient magnitude >= threshold).
pub fn edge_count(edges: &GrayImage, threshold: u8) -> u64 {
 edges.pixels().filter(|p| p.0[0] >= threshold).count() as u64
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn solid_has_no_edges() {
 let img = RgbaImage::from_pixel(16, 16, Rgba([120, 120, 120, 255]));
 let e = sobel(&img);
 assert_eq!(edge_count(&e, 40), 0);
 }

 #[test]
 fn vertical_boundary_has_edges() {
 let mut img = RgbaImage::from_pixel(16, 16, Rgba([0, 0, 0, 255]));
 for y in 0..16u32 {
 for x in 8..16u32 {
 img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
 }
 }
 let e = sobel(&img);
 assert!(edge_count(&e, 100) > 0);
 }
}
