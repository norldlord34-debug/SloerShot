//! Image diff for before/after screenshots (tutorials, QA, change tracking). Compares two
//! equal-size RGBA images and reports how many pixels changed, the percentage, and the
//! bounding box of the change. Pure pixel math, tested.
use crate::geometry::Rect;
use image::RgbaImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffResult {
 pub changed_pixels: u64,
 pub total_pixels: u64,
 pub percent: f64,
 pub bounds: Option<Rect>,
}

/// Compare two images. A pixel counts as changed when any RGB channel differs by more than
/// `tol`. Different sizes are reported as fully changed over the first image bounds.
pub fn diff(a: &RgbaImage, b: &RgbaImage, tol: u8) -> DiffResult {
 let (aw, ah) = a.dimensions();
 let (bw, bh) = b.dimensions();
 let total = aw as u64 * ah as u64;
 if aw != bw || ah != bh {
 return DiffResult {
 changed_pixels: total,
 total_pixels: total,
 percent: 100.0,
 bounds: Some(Rect::new(0.0, 0.0, aw as f64, ah as f64)),
 };
 }
 let mut changed = 0u64;
 let (mut minx, mut miny, mut maxx, mut maxy) = (aw, ah, 0u32, 0u32);
 let mut any = false;
 for y in 0..ah {
 for x in 0..aw {
 let pa = a.get_pixel(x, y).0;
 let pb = b.get_pixel(x, y).0;
 let d = |i: usize| if pa[i] > pb[i] { pa[i] - pb[i] } else { pb[i] - pa[i] };
 if d(0) > tol || d(1) > tol || d(2) > tol {
 changed += 1;
 any = true;
 minx = minx.min(x);
 miny = miny.min(y);
 maxx = maxx.max(x);
 maxy = maxy.max(y);
 }
 }
 }
 let percent = if total == 0 { 0.0 } else { changed as f64 / total as f64 * 100.0 };
 let bounds = if any {
 Some(Rect::new(minx as f64, miny as f64, (maxx - minx + 1) as f64, (maxy - miny + 1) as f64))
 } else {
 None
 };
 DiffResult { changed_pixels: changed, total_pixels: total, percent, bounds }
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn identical_has_no_change() {
 let a = RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255]));
 let b = a.clone();
 let r = diff(&a, &b, 0);
 assert_eq!(r.changed_pixels, 0);
 assert!(r.bounds.is_none());
 assert_eq!(r.percent, 0.0);
 }

 #[test]
 fn locates_changed_block() {
 let a = RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255]));
 let mut b = a.clone();
 for y in 2..5u32 {
 for x in 2..5u32 {
 b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 let r = diff(&a, &b, 10);
 assert_eq!(r.changed_pixels, 9);
 assert_eq!(r.bounds, Some(Rect::new(2.0, 2.0, 3.0, 3.0)));
 }

 #[test]
 fn size_mismatch_is_full_change() {
 let a = RgbaImage::from_pixel(4, 4, Rgba([0, 0, 0, 255]));
 let b = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 255]));
 assert_eq!(diff(&a, &b, 0).percent, 100.0);
 }
}
