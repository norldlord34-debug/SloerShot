//! Adaptive thresholding (Otsu) + line/guide detection. Otsu picks the global threshold that
//! maximizes between-class variance of the luma histogram; line detection collapses rows or
//! columns of high edge density into guide positions (auto-guides, table-grid hints). Pure
//! math over the Sobel edge map. Tested.
use crate::edges::sobel;
use image::{GrayImage, Luma, RgbaImage};

fn histogram(img: &RgbaImage) -> [u64; 256] {
 let mut hist = [0u64; 256];
 for px in img.pixels() {
 let p = px.0;
 let l = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).round() as usize;
 hist[l.min(255)] += 1;
 }
 hist
}

/// Otsu threshold (0..=255) maximizing between-class variance of the luma histogram.
pub fn otsu_threshold(img: &RgbaImage) -> u8 {
 let hist = histogram(img);
 let total: u64 = hist.iter().sum();
 if total == 0 {
 return 128;
 }
 let sum_all: f64 = (0..256).map(|i| i as f64 * hist[i] as f64).sum();
 let mut sum_bg = 0.0f64;
 let mut w_bg = 0u64;
 let mut best_t = 0u8;
 let mut best_var = -1.0f64;
 for t in 0..256 {
 w_bg += hist[t];
 if w_bg == 0 {
 continue;
 }
 let w_fg = total - w_bg;
 if w_fg == 0 {
 break;
 }
 sum_bg += t as f64 * hist[t] as f64;
 let mu_bg = sum_bg / w_bg as f64;
 let mu_fg = (sum_all - sum_bg) / w_fg as f64;
 let var = w_bg as f64 * w_fg as f64 * (mu_bg - mu_fg) * (mu_bg - mu_fg);
 if var > best_var {
 best_var = var;
 best_t = t as u8;
 }
 }
 best_t
}

/// Binarize to a grayscale 0/255 image: foreground (dark, luma below threshold) -> 255.
pub fn binarize(img: &RgbaImage, threshold: u8) -> GrayImage {
 let (w, h) = img.dimensions();
 let mut out = GrayImage::new(w, h);
 for y in 0..h {
 for x in 0..w {
 let p = img.get_pixel(x, y).0;
 let l = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).round() as u8;
 out.put_pixel(x, y, Luma([if l <= threshold { 255 } else { 0 }]));
 }
 }
 out
}

fn collapse(positions: &[u32]) -> Vec<u32> {
 let mut out = Vec::new();
 let mut i = 0;
 while i < positions.len() {
 let start = positions[i];
 let mut end = start;
 while i + 1 < positions.len() && positions[i + 1] == positions[i] + 1 {
 i += 1;
 end = positions[i];
 }
 out.push((start + end) / 2);
 i += 1;
 }
 out
}

fn line_positions(img: &RgbaImage, frac: f32, horizontal: bool) -> Vec<u32> {
 let e = sobel(img);
 let (w, h) = e.dimensions();
 let n = if horizontal { h } else { w };
 let mut density = vec![0u32; n as usize];
 for y in 0..h {
 for x in 0..w {
 if e.get_pixel(x, y).0[0] >= 50 {
 let k = if horizontal { y } else { x };
 density[k as usize] += 1;
 }
 }
 }
 let max = *density.iter().max().unwrap_or(&0);
 if max == 0 {
 return Vec::new();
 }
 let cutoff = ((max as f32 * frac) as u32).max(1);
 let qualifying: Vec<u32> = (0..n).filter(|&k| density[k as usize] >= cutoff).collect();
 collapse(&qualifying)
}

/// Y positions of horizontal guide lines (rows of high edge density).
pub fn horizontal_lines(img: &RgbaImage, frac: f32) -> Vec<u32> {
 line_positions(img, frac, true)
}

/// X positions of vertical guide lines (columns of high edge density).
pub fn vertical_lines(img: &RgbaImage, frac: f32) -> Vec<u32> {
 line_positions(img, frac, false)
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn otsu_splits_bimodal() {
 let mut img = RgbaImage::new(20, 20);
 for y in 0..20u32 {
 for x in 0..20u32 {
 let v = if x < 10 { 64 } else { 192 };
 img.put_pixel(x, y, Rgba([v, v, v, 255]));
 }
 }
 let t = otsu_threshold(&img);
 assert!(t >= 64 && t < 192, "threshold {} should separate 64 and 192", t);
 let bin = binarize(&img, t);
 assert_eq!(bin.get_pixel(0, 0).0[0], 255);
 assert_eq!(bin.get_pixel(15, 0).0[0], 0);
 }

 #[test]
 fn detects_a_horizontal_and_vertical_line() {
 let mut img = RgbaImage::from_pixel(40, 40, Rgba([255, 255, 255, 255]));
 for x in 0..40u32 {
 img.put_pixel(x, 20, Rgba([0, 0, 0, 255]));
 }
 for y in 0..40u32 {
 img.put_pixel(10, y, Rgba([0, 0, 0, 255]));
 }
 let hl = horizontal_lines(&img, 0.5);
 let vl = vertical_lines(&img, 0.5);
 assert!(hl.iter().any(|&y| (18..=22).contains(&y)));
 assert!(vl.iter().any(|&x| (8..=12).contains(&x)));
 }
}
