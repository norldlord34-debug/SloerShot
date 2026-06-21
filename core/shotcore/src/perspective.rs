//! 4-point perspective unwarp: given the corners of a quadrilateral (e.g. a tilted document
//! detected by corners.rs), compute the homography and remap to an axis-aligned rectangle.
//! The document-scanner flatten step. Pure math: an 8x8 solve for the homography plus
//! bilinear inverse sampling.
use crate::geometry::Point;
use image::{Rgba, RgbaImage};

fn solve8(mut a: [[f64; 8]; 8], mut b: [f64; 8]) -> Option<[f64; 8]> {
 for col in 0..8 {
 let mut piv = col;
 for r in col + 1..8 {
 if a[r][col].abs() > a[piv][col].abs() {
 piv = r;
 }
 }
 if a[piv][col].abs() < 1e-12 {
 return None;
 }
 a.swap(col, piv);
 b.swap(col, piv);
 let d = a[col][col];
 for j in col..8 {
 a[col][j] /= d;
 }
 b[col] /= d;
 for r in 0..8 {
 if r != col {
 let f = a[r][col];
 for j in col..8 {
 a[r][j] -= f * a[col][j];
 }
 b[r] -= f * b[col];
 }
 }
 }
 Some(b)
}

// Homography mapping dst (output rect) corners to src (source quad) corners.
fn homography(dst: &[Point; 4], src: &[Point; 4]) -> Option<[f64; 9]> {
 let mut a = [[0.0f64; 8]; 8];
 let mut b = [0.0f64; 8];
 for i in 0..4 {
 let (x, y) = (dst[i].x, dst[i].y);
 let (u, v) = (src[i].x, src[i].y);
 a[2 * i] = [x, y, 1.0, 0.0, 0.0, 0.0, -x * u, -y * u];
 b[2 * i] = u;
 a[2 * i + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -x * v, -y * v];
 b[2 * i + 1] = v;
 }
 let h = solve8(a, b)?;
 Some([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], 1.0])
}

fn sample_bilinear(img: &RgbaImage, u: f64, v: f64) -> Rgba<u8> {
 let (w, h) = img.dimensions();
 if w == 0 || h == 0 {
 return Rgba([255, 255, 255, 255]);
 }
 let uc = u.clamp(0.0, (w - 1) as f64);
 let vc = v.clamp(0.0, (h - 1) as f64);
 let x0 = uc.floor() as u32;
 let y0 = vc.floor() as u32;
 let x1 = (x0 + 1).min(w - 1);
 let y1 = (y0 + 1).min(h - 1);
 let fx = uc - x0 as f64;
 let fy = vc - y0 as f64;
 let mut out = [0u8; 4];
 for c in 0..4 {
 let p00 = img.get_pixel(x0, y0).0[c] as f64;
 let p10 = img.get_pixel(x1, y0).0[c] as f64;
 let p01 = img.get_pixel(x0, y1).0[c] as f64;
 let p11 = img.get_pixel(x1, y1).0[c] as f64;
 let top = p00 * (1.0 - fx) + p10 * fx;
 let bot = p01 * (1.0 - fx) + p11 * fx;
 out[c] = (top * (1.0 - fy) + bot * fy).round() as u8;
 }
 Rgba(out)
}

/// Unwarp the quadrilateral `corners` (top-left, top-right, bottom-right, bottom-left) from
/// `img` into an out_w x out_h axis-aligned image via bilinear sampling.
pub fn unwarp(img: &RgbaImage, corners: [Point; 4], out_w: u32, out_h: u32) -> RgbaImage {
 let (ow, oh) = (out_w.max(1), out_h.max(1));
 let dst = [
 Point::new(0.0, 0.0),
 Point::new(ow as f64, 0.0),
 Point::new(ow as f64, oh as f64),
 Point::new(0.0, oh as f64),
 ];
 let h = match homography(&dst, &corners) {
 Some(h) => h,
 None => return RgbaImage::new(ow, oh),
 };
 let mut out = RgbaImage::new(ow, oh);
 for y in 0..oh {
 for x in 0..ow {
 let (xf, yf) = (x as f64, y as f64);
 let denom = h[6] * xf + h[7] * yf + h[8];
 if denom.abs() < 1e-12 {
 continue;
 }
 let u = (h[0] * xf + h[1] * yf + h[2]) / denom;
 let v = (h[3] * xf + h[4] * yf + h[5]) / denom;
 out.put_pixel(x, y, sample_bilinear(img, u, v));
 }
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 fn split() -> RgbaImage {
 let mut img = RgbaImage::new(100, 100);
 for y in 0..100u32 {
 for x in 0..100u32 {
 let c = if x < 50 { Rgba([220, 0, 0, 255]) } else { Rgba([0, 0, 220, 255]) };
 img.put_pixel(x, y, c);
 }
 }
 img
 }

 #[test]
 fn identity_reproduces_image() {
 let img = split();
 let corners = [Point::new(0.0, 0.0), Point::new(100.0, 0.0), Point::new(100.0, 100.0), Point::new(0.0, 100.0)];
 let out = unwarp(&img, corners, 100, 100);
 assert!(out.get_pixel(10, 10).0[0] > 150);
 assert!(out.get_pixel(90, 10).0[2] > 150);
 }

 #[test]
 fn extracts_left_half_to_full() {
 let img = split();
 let corners = [Point::new(0.0, 0.0), Point::new(50.0, 0.0), Point::new(50.0, 100.0), Point::new(0.0, 100.0)];
 let out = unwarp(&img, corners, 80, 80);
 let p = out.get_pixel(40, 40).0;
 assert!(p[0] > 150 && p[2] < 100, "center should be red, got {:?}", p);
 }
}
