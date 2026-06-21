//! Multi-page printing for scrolling captures: split a tall image into overlapping
//! pages. Slice math is pure; slice_pages() crops the actual page images.
use crate::geometry::Rect;
use image::RgbaImage;

/// Slice rectangles for each print page of a `content_w` x `content_h` image, using
/// pages of height `page_h` that overlap by `overlap` px. x is always 0, w = content_w.
pub fn page_slices(content_w: u32, content_h: u32, page_h: u32, overlap: u32) -> Vec<Rect> {
 if content_w == 0 || content_h == 0 {
 return Vec::new();
 }
 let page_h = page_h.max(1);
 if content_h <= page_h {
 return vec![Rect::new(0.0, 0.0, content_w as f64, content_h as f64)];
 }
 let overlap = overlap.min(page_h - 1);
 let step = (page_h - overlap) as i64;
 let mut out = Vec::new();
 let mut y = 0i64;
 loop {
 let h = ((content_h as i64) - y).min(page_h as i64);
 out.push(Rect::new(0.0, y as f64, content_w as f64, h as f64));
 if y + page_h as i64 >= content_h as i64 {
 break;
 }
 y += step;
 }
 out
}

/// Number of pages the content splits into.
pub fn page_count(content_h: u32, page_h: u32, overlap: u32) -> usize {
 page_slices(1, content_h, page_h, overlap).len()
}

/// Crop a tall image into page images.
pub fn slice_pages(img: &RgbaImage, page_h: u32, overlap: u32) -> Vec<RgbaImage> {
 page_slices(img.width(), img.height(), page_h, overlap)
 .into_iter()
 .map(|r| {
 image::imageops::crop_imm(img, r.x as u32, r.y as u32, r.w as u32, r.h as u32).to_image()
 })
 .collect()
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn single_page_when_short() {
 let s = page_slices(100, 80, 100, 10);
 assert_eq!(s.len(), 1);
 assert_eq!(s[0], Rect::new(0.0, 0.0, 100.0, 80.0));
 }

 #[test]
 fn paginates_with_overlap() {
 let s = page_slices(100, 250, 100, 20);
 assert_eq!(s.len(), 3);
 assert_eq!(s[0].y, 0.0);
 assert_eq!(s[1].y, 80.0);
 assert_eq!(s[2].y, 160.0);
 // last page is shorter and ends exactly at content bottom
 assert_eq!(s[2].bottom(), 250.0);
 }

 #[test]
 fn page_count_matches() {
 assert_eq!(page_count(250, 100, 20), 3);
 assert_eq!(page_count(100, 100, 0), 1);
 assert_eq!(page_count(0, 100, 0), 0);
 }

 #[test]
 fn slice_pages_crops_images() {
 let img = RgbaImage::new(10, 25);
 let pages = slice_pages(&img, 10, 0);
 assert_eq!(pages.len(), 3);
 assert_eq!(pages[0].height(), 10);
 assert_eq!(pages[2].height(), 5);
 assert_eq!(pages[0].width(), 10);
 }
}
