//! Crosshair + magnifier loupe + tolerance helpers for precise capture (CleanShot
//! advanced capture modes and PixelSnap crosshair/tolerance). Pure pixel + geometry
//! math; the native overlay draws the HUD from these readouts.
use crate::geometry::Rect;
use image::RgbaImage;

/// Hex color (#RRGGBB) of the pixel at (x,y), or None if out of bounds.
pub fn pixel_hex(img: &RgbaImage, x: u32, y: u32) -> Option<String> {
 if x >= img.width() || y >= img.height() {
 return None;
 }
 let p = img.get_pixel(x, y).0;
 Some(format!("#{:02X}{:02X}{:02X}", p[0], p[1], p[2]))
}

/// Magnifier loupe: sample a src_size x src_size region centered on (cx,cy) and upscale
/// each source pixel into a zoom x zoom block. Out-of-bounds samples are clamped to edges.
pub fn loupe(img: &RgbaImage, cx: i32, cy: i32, src_size: u32, zoom: u32) -> RgbaImage {
 let src = src_size.max(1);
 let z = zoom.max(1);
 let half = (src / 2) as i32;
 let out_dim = src * z;
 let mut out = RgbaImage::new(out_dim, out_dim);
 let (w, h) = (img.width() as i32, img.height() as i32);
 for sy in 0..src as i32 {
 for sx in 0..src as i32 {
 let px = (cx - half + sx).clamp(0, w - 1);
 let py = (cy - half + sy).clamp(0, h - 1);
 let color = *img.get_pixel(px as u32, py as u32);
 for dy in 0..z {
 for dx in 0..z {
 out.put_pixel(sx as u32 * z + dx, sy as u32 * z + dy, color);
 }
 }
 }
 }
 out
}

/// Crosshair guide lines through (cx,cy): a full-width horizontal line and a full-height
/// vertical line, each as a thickness-1 rect. The native overlay strokes these.
pub fn crosshair_lines(cx: f64, cy: f64, w: f64, h: f64) -> (Rect, Rect) {
 (Rect::new(0.0, cy, w, 1.0), Rect::new(cx, 0.0, 1.0, h))
}

/// True when two RGBA colors match within a per-channel tolerance (PixelSnap tolerance
/// mode, for shadows / low-contrast edges).
pub fn within_tolerance(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
 let d = |x: u8, y: u8| if x > y { x - y } else { y - x };
 d(a[0], b[0]) <= tol && d(a[1], b[1]) <= tol && d(a[2], b[2]) <= tol
}

/// Average color (#RRGGBB) over a region - the loupe-center eyedropper read.
pub fn region_average_hex(img: &RgbaImage, rect: Rect) -> String {
 let x0 = rect.x.max(0.0) as u32;
 let y0 = rect.y.max(0.0) as u32;
 let x1 = ((rect.x + rect.w) as u32).min(img.width());
 let y1 = ((rect.y + rect.h) as u32).min(img.height());
 let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
 for y in y0..y1 {
 for x in x0..x1 {
 let p = img.get_pixel(x, y).0;
 r += p[0] as u64;
 g += p[1] as u64;
 b += p[2] as u64;
 n += 1;
 }
 }
 if n == 0 {
 return String::from("#000000");
 }
 format!("#{:02X}{:02X}{:02X}", (r / n) as u8, (g / n) as u8, (b / n) as u8)
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn hex_and_average() {
 let mut img = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
 img.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
 assert_eq!(pixel_hex(&img, 1, 1).unwrap(), "#FF0000");
 assert_eq!(pixel_hex(&img, 9, 9), None);
 let solid = RgbaImage::from_pixel(3, 3, Rgba([10, 20, 30, 255]));
 assert_eq!(region_average_hex(&solid, Rect::new(0.0, 0.0, 3.0, 3.0)), "#0A141E");
 }

 #[test]
 fn loupe_upscales_center() {
 let mut img = RgbaImage::from_pixel(5, 5, Rgba([255, 255, 255, 255]));
 img.put_pixel(2, 2, Rgba([255, 0, 0, 255]));
 let out = loupe(&img, 2, 2, 3, 4);
 assert_eq!(out.dimensions(), (12, 12));
 assert_eq!(out.get_pixel(4, 4).0, [255, 0, 0, 255]);
 assert_eq!(out.get_pixel(0, 0).0, [255, 255, 255, 255]);
 }

 #[test]
 fn crosshair_and_tolerance() {
 let (hz, vt) = crosshair_lines(50.0, 30.0, 200.0, 100.0);
 assert_eq!((hz.y, hz.w), (30.0, 200.0));
 assert_eq!((vt.x, vt.h), (50.0, 100.0));
 assert!(within_tolerance([100, 100, 100, 255], [108, 95, 100, 255], 10));
 assert!(!within_tolerance([100, 100, 100, 255], [130, 100, 100, 255], 10));
 }
}
