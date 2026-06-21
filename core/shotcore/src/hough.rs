//! Hough transform for straight-line detection. From the Sobel edge map it accumulates
//! (rho, theta) votes and returns the dominant lines and the dominant angle. Used for
//! deskew (auto-straighten) and table/grid detection. Pure math; theta sampled per degree.
use crate::edges::sobel;
use image::RgbaImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoughLine {
 pub theta_deg: f64,
 pub rho: f64,
 pub votes: u32,
}

struct Accumulator {
 acc: Vec<u32>,
 nrho: usize,
 diag: i32,
}

fn accumulate(img: &RgbaImage, edge_threshold: u8) -> Accumulator {
 let e = sobel(img);
 let (w, h) = e.dimensions();
 let diag = (((w * w + h * h) as f64).sqrt().ceil()) as i32;
 let nrho = (2 * diag + 1) as usize;
 let mut cos = vec![0.0f64; 180];
 let mut sin = vec![0.0f64; 180];
 for t in 0..180 {
 let r = (t as f64) * std::f64::consts::PI / 180.0;
 cos[t] = r.cos();
 sin[t] = r.sin();
 }
 let mut acc = vec![0u32; 180 * nrho];
 for y in 0..h {
 for x in 0..w {
 if e.get_pixel(x, y).0[0] < edge_threshold {
 continue;
 }
 for t in 0..180 {
 let rho = (x as f64) * cos[t] + (y as f64) * sin[t];
 let bin = rho.round() as i32 + diag;
 if bin >= 0 && bin < nrho as i32 {
 acc[t * nrho + bin as usize] += 1;
 }
 }
 }
 }
 Accumulator { acc, nrho, diag }
}

/// Detect up to `max_lines` dominant lines from the image edges, peak-suppressed.
pub fn detect_lines(img: &RgbaImage, edge_threshold: u8, max_lines: usize) -> Vec<HoughLine> {
 let a = accumulate(img, edge_threshold);
 let max_votes = *a.acc.iter().max().unwrap_or(&0);
 if max_votes == 0 {
 return Vec::new();
 }
 let cutoff = ((max_votes as f64 * 0.5) as u32).max(3);
 let mut peaks: Vec<(u32, usize, usize)> = Vec::new();
 for t in 0..180 {
 for b in 0..a.nrho {
 let v = a.acc[t * a.nrho + b];
 if v >= cutoff {
 peaks.push((v, t, b));
 }
 }
 }
 peaks.sort_by(|x, y| y.0.cmp(&x.0));
 let mut chosen: Vec<(usize, usize)> = Vec::new();
 let mut out = Vec::new();
 for (v, t, b) in peaks {
 let far = chosen.iter().all(|&(ct, cb)| {
 (t as i32 - ct as i32).abs() > 5 || (b as i32 - cb as i32).abs() > 10
 });
 if far {
 chosen.push((t, b));
 out.push(HoughLine { theta_deg: t as f64, rho: (b as i32 - a.diag) as f64, votes: v });
 if out.len() >= max_lines {
 break;
 }
 }
 }
 out
}

/// Dominant line angle in degrees (theta of the most-voted cell), or None when there are no
/// strong edges. A horizontal line yields ~90 degrees, a vertical line ~0.
pub fn dominant_angle(img: &RgbaImage, edge_threshold: u8) -> Option<f64> {
 let a = accumulate(img, edge_threshold);
 let mut best = 0u32;
 let mut best_t = 0usize;
 for t in 0..180 {
 for b in 0..a.nrho {
 let v = a.acc[t * a.nrho + b];
 if v > best {
 best = v;
 best_t = t;
 }
 }
 }
 if best == 0 {
 None
 } else {
 Some(best_t as f64)
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn detects_horizontal_line_angle() {
 let mut img = RgbaImage::from_pixel(40, 40, Rgba([255, 255, 255, 255]));
 for x in 0..40u32 {
 img.put_pixel(x, 20, Rgba([0, 0, 0, 255]));
 }
 let ang = dominant_angle(&img, 80).unwrap();
 assert!((85.0..=95.0).contains(&ang), "angle {} should be ~90", ang);
 assert!(!detect_lines(&img, 80, 4).is_empty());
 }

 #[test]
 fn blank_has_no_lines() {
 let img = RgbaImage::from_pixel(20, 20, Rgba([128, 128, 128, 255]));
 assert!(dominant_angle(&img, 80).is_none());
 assert!(detect_lines(&img, 80, 4).is_empty());
 }
}
