//! Object boundary detection (PixelSnap-style "snaps to the object"): given a rough
//! search region, find the tight bounding box of the foreground by trimming pixels
//! that match the background color within a tolerance. Pure pixel analysis.
use crate::geometry::Rect;
use image::RgbaImage;

fn within_tol(p: [u8; 4], bg: [u8; 4], tol: u8) -> bool {
 let d = |a: u8, b: u8| if a > b { a - b } else { b - a };
 d(p[0], bg[0]) <= tol && d(p[1], bg[1]) <= tol && d(p[2], bg[2]) <= tol
}

/// Snap `search` to the tight bounds of the object inside it. Background is sampled
/// at the search top-left; pixels within `tolerance` per channel are treated as empty.
/// Returns a zero-size rect at the search origin when the region is all background.
pub fn snap_to_object(img: &RgbaImage, search: Rect, tolerance: u8) -> Rect {
 let bounds = Rect::new(0.0, 0.0, img.width() as f64, img.height() as f64);
 let s = search.normalized().clamp_to(&bounds).round_out();
 let x0 = s.x as u32;
 let y0 = s.y as u32;
 let x1 = (s.x + s.w) as u32;
 let y1 = (s.y + s.h) as u32;
 if x1 <= x0 || y1 <= y0 {
 return Rect::new(x0 as f64, y0 as f64, 0.0, 0.0);
 }
 let bg = img.get_pixel(x0, y0).0;
 let (mut minx, mut miny, mut maxx, mut maxy) = (u32::MAX, u32::MAX, 0u32, 0u32);
 let mut found = false;
 for y in y0..y1 {
 for x in x0..x1 {
 if !within_tol(img.get_pixel(x, y).0, bg, tolerance) {
 found = true;
 if x < minx { minx = x; }
 if y < miny { miny = y; }
 if x > maxx { maxx = x; }
 if y > maxy { maxy = y; }
 }
 }
 }
 if !found {
 return Rect::new(x0 as f64, y0 as f64, 0.0, 0.0);
 }
 Rect::new(minx as f64, miny as f64, (maxx - minx + 1) as f64, (maxy - miny + 1) as f64)
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 fn scene() -> RgbaImage {
 let mut img = RgbaImage::from_pixel(20, 20, Rgba([255, 255, 255, 255]));
 for y in 5..11u32 {
 for x in 5..13u32 {
 img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 img
 }

 #[test]
 fn snaps_to_object_bounds() {
 let r = snap_to_object(&scene(), Rect::new(0.0, 0.0, 20.0, 20.0), 10);
 assert_eq!(r, Rect::new(5.0, 5.0, 8.0, 6.0));
 }

 #[test]
 fn all_background_is_empty() {
 let img = RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255]));
 let r = snap_to_object(&img, Rect::new(0.0, 0.0, 10.0, 10.0), 5);
 assert_eq!(r.w, 0.0);
 assert_eq!(r.h, 0.0);
 }

 #[test]
 fn tolerance_ignores_near_background_noise() {
 let mut img = RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255]));
 img.put_pixel(3, 3, Rgba([250, 250, 250, 255]));
 img.put_pixel(6, 6, Rgba([0, 0, 0, 255]));
 let r = snap_to_object(&img, Rect::new(0.0, 0.0, 10.0, 10.0), 10);
 assert_eq!(r, Rect::new(6.0, 6.0, 1.0, 1.0));
 }

 #[test]
 fn clamps_search_to_image() {
 let r = snap_to_object(&scene(), Rect::new(-50.0, -50.0, 1000.0, 1000.0), 10);
 assert_eq!(r, Rect::new(5.0, 5.0, 8.0, 6.0));
 }
}
