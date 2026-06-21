//! Automatic document detection: find the page quadrilateral in a capture without manual
//! corner picking. Binarize against the top-left background pixel, then take the foreground
//! x +/- y extremes as the TL/TR/BR/BL corners. Pairs with perspective::unwarp for auto-scan.
use crate::geometry::Point;
use image::RgbaImage;

fn luma(img: &RgbaImage, x: u32, y: u32) -> i32 {
 let p = img.get_pixel(x, y).0;
 (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).round() as i32
}

/// Detect the document quadrilateral as [TL, TR, BR, BL]. Foreground = luma differs from the
/// top-left background pixel by more than `tol`. Returns None if too few foreground pixels.
pub fn detect_document(img: &RgbaImage, tol: u8) -> Option<[Point; 4]> {
 let (w, h) = img.dimensions();
 if w == 0 || h == 0 {
 return None;
 }
 let bg = luma(img, 0, 0);
 let mut count = 0u64;
 let (mut tl, mut tr, mut br, mut bl) = (None, None, None, None);
 let (mut tlv, mut brv, mut trv, mut blv) = (i64::MAX, i64::MIN, i64::MIN, i64::MAX);
 for y in 0..h {
 for x in 0..w {
 if (luma(img, x, y) - bg).unsigned_abs() > tol as u32 {
 count += 1;
 let (xi, yi) = (x as i64, y as i64);
 let sp = xi + yi;
 let dp = xi - yi;
 let pt = Point::new(x as f64, y as f64);
 if sp < tlv { tlv = sp; tl = Some(pt); }
 if sp > brv { brv = sp; br = Some(pt); }
 if dp > trv { trv = dp; tr = Some(pt); }
 if dp < blv { blv = dp; bl = Some(pt); }
 }
 }
 }
 if count < 16 {
 return None;
 }
 Some([tl?, tr?, br?, bl?])
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn finds_document_corners() {
 let mut img = RgbaImage::from_pixel(60, 50, Rgba([0, 0, 0, 255]));
 for y in 10..30u32 {
 for x in 10..40u32 {
 img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
 }
 }
 let q = detect_document(&img, 60).unwrap();
 assert!((q[0].x - 10.0).abs() <= 1.0 && (q[0].y - 10.0).abs() <= 1.0, "TL {:?}", q[0]);
 assert!((q[2].x - 39.0).abs() <= 1.0 && (q[2].y - 29.0).abs() <= 1.0, "BR {:?}", q[2]);
 }

 #[test]
 fn none_when_blank() {
 let img = RgbaImage::from_pixel(20, 20, Rgba([12, 12, 12, 255]));
 assert!(detect_document(&img, 60).is_none());
 }
}
