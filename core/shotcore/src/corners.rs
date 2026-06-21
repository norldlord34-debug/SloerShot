//! Harris corner detection: gradient structure tensor -> corner response -> threshold +
//! non-max suppression -> corner points. Useful for perspective/crop hints and snapping to
//! UI element corners. Pure math, tested.
use crate::geometry::Point;
use image::RgbaImage;

/// Detect corner points whose Harris response exceeds `rel_threshold` * max response.
/// `min_distance` suppresses near-duplicate corners. Strongest first.
pub fn detect(img: &RgbaImage, k: f64, rel_threshold: f64, min_distance: u32) -> Vec<Point> {
 let (w, h) = img.dimensions();
 if w < 5 || h < 5 {
 return Vec::new();
 }
 let mut g = vec![0f64; (w * h) as usize];
 for y in 0..h {
 for x in 0..w {
 let p = img.get_pixel(x, y).0;
 g[(y * w + x) as usize] = 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
 }
 }
 let idx = |x: u32, y: u32| (y * w + x) as usize;
 let mut ix = vec![0f64; (w * h) as usize];
 let mut iy = vec![0f64; (w * h) as usize];
 for y in 1..h - 1 {
 for x in 1..w - 1 {
 ix[idx(x, y)] = (g[idx(x + 1, y)] - g[idx(x - 1, y)]) / 2.0;
 iy[idx(x, y)] = (g[idx(x, y + 1)] - g[idx(x, y - 1)]) / 2.0;
 }
 }
 let mut resp = vec![0f64; (w * h) as usize];
 let mut max_r = 0.0f64;
 for y in 2..h - 2 {
 for x in 2..w - 2 {
 let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
 for dy in -1i32..=1 {
 for dx in -1i32..=1 {
 let nx = (x as i32 + dx) as u32;
 let ny = (y as i32 + dy) as u32;
 let i = idx(nx, ny);
 sxx += ix[i] * ix[i];
 syy += iy[i] * iy[i];
 sxy += ix[i] * iy[i];
 }
 }
 let det = sxx * syy - sxy * sxy;
 let trace = sxx + syy;
 let r = det - k * trace * trace;
 resp[idx(x, y)] = r;
 if r > max_r {
 max_r = r;
 }
 }
 }
 if max_r <= 0.0 {
 return Vec::new();
 }
 let thr = rel_threshold * max_r;
 let mut cands: Vec<(f64, u32, u32)> = Vec::new();
 for y in 2..h - 2 {
 for x in 2..w - 2 {
 let r = resp[idx(x, y)];
 if r > thr {
 cands.push((r, x, y));
 }
 }
 }
 cands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
 let min_d2 = (min_distance as f64) * (min_distance as f64);
 let mut out: Vec<Point> = Vec::new();
 for (_, x, y) in cands {
 let far = out.iter().all(|p| {
 let dx = p.x - x as f64;
 let dy = p.y - y as f64;
 dx * dx + dy * dy >= min_d2
 });
 if far {
 out.push(Point::new(x as f64, y as f64));
 }
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn finds_square_corners() {
 let mut img = RgbaImage::from_pixel(60, 60, Rgba([0, 0, 0, 255]));
 for y in 20..40u32 {
 for x in 20..40u32 {
 img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
 }
 }
 let corners = detect(&img, 0.04, 0.2, 5);
 assert!(corners.len() >= 4, "expected >=4 corners, got {}", corners.len());
 let near = |cx: f64, cy: f64| {
 corners.iter().any(|p| ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt() <= 4.0)
 };
 assert!(near(20.0, 20.0), "missing top-left corner");
 assert!(near(39.0, 39.0), "missing bottom-right corner");
 }

 #[test]
 fn blank_has_no_corners() {
 let img = RgbaImage::from_pixel(30, 30, Rgba([100, 100, 100, 255]));
 assert!(detect(&img, 0.04, 0.2, 5).is_empty());
 }
}
