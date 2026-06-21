//! Connected-components labeling on a binarized image: bounding boxes of distinct foreground
//! regions, for click-to-select-element and auto-annotation. 4-connectivity BFS. Tested.
use crate::geometry::Rect;
use image::RgbaImage;
use std::collections::VecDeque;

fn luma(img: &RgbaImage, x: u32, y: u32) -> i32 {
 let p = img.get_pixel(x, y).0;
 (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).round() as i32
}

/// Bounding boxes of connected foreground regions. A pixel is foreground when its luma differs
/// from the background luma (sampled at the top-left corner) by more than `tol`. Regions below
/// `min_area` pixels are dropped. Largest region first.
pub fn regions(img: &RgbaImage, tol: u8, min_area: u32) -> Vec<Rect> {
 let (w, h) = img.dimensions();
 if w == 0 || h == 0 {
 return Vec::new();
 }
 let bg = luma(img, 0, 0);
 let fg = |x: u32, y: u32| (luma(img, x, y) - bg).unsigned_abs() > tol as u32;
 let mut visited = vec![false; (w * h) as usize];
 let mut found: Vec<(Rect, u32)> = Vec::new();
 for sy in 0..h {
 for sx in 0..w {
 let idx = (sy * w + sx) as usize;
 if visited[idx] || !fg(sx, sy) {
 continue;
 }
 let mut queue = VecDeque::new();
 queue.push_back((sx as i32, sy as i32));
 visited[idx] = true;
 let (mut minx, mut miny, mut maxx, mut maxy) = (sx, sy, sx, sy);
 let mut area = 0u32;
 while let Some((cx, cy)) = queue.pop_front() {
 area += 1;
 let (ux, uy) = (cx as u32, cy as u32);
 minx = minx.min(ux);
 miny = miny.min(uy);
 maxx = maxx.max(ux);
 maxy = maxy.max(uy);
 let neighbors = [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)];
 for (nx, ny) in neighbors {
 if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
 continue;
 }
 let nidx = (ny as u32 * w + nx as u32) as usize;
 if !visited[nidx] && fg(nx as u32, ny as u32) {
 visited[nidx] = true;
 queue.push_back((nx, ny));
 }
 }
 }
 if area >= min_area {
 let r = Rect::new(minx as f64, miny as f64, (maxx - minx + 1) as f64, (maxy - miny + 1) as f64);
 found.push((r, area));
 }
 }
 }
 found.sort_by(|a, b| b.1.cmp(&a.1));
 found.into_iter().map(|(r, _)| r).collect()
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 fn canvas() -> RgbaImage {
 let mut img = RgbaImage::from_pixel(20, 20, Rgba([255, 255, 255, 255]));
 for y in 2..5u32 {
 for x in 2..5u32 {
 img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 for y in 12..16u32 {
 for x in 12..16u32 {
 img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 img
 }

 #[test]
 fn finds_two_regions_largest_first() {
 let regions = regions(&canvas(), 50, 4);
 assert_eq!(regions.len(), 2);
 assert_eq!(regions[0], Rect::new(12.0, 12.0, 4.0, 4.0));
 assert_eq!(regions[1], Rect::new(2.0, 2.0, 3.0, 3.0));
 }

 #[test]
 fn min_area_filters_specks() {
 let mut img = RgbaImage::from_pixel(20, 20, Rgba([255, 255, 255, 255]));
 img.put_pixel(5, 5, Rgba([0, 0, 0, 255]));
 assert!(regions(&img, 50, 4).is_empty());
 }
}
