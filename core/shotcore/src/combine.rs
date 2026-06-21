//! Combine multiple images into one canvas (CleanShot combine: drag images together,
//! stack them vertically or horizontally, or place them freely). Layout math is pure;
//! compose() blits the images onto an RGBA canvas.
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

/// One image placed at a pixel offset on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placement {
 /// Index into the images slice.
 pub image: usize,
 pub x: i64,
 pub y: i64,
}

/// A canvas size plus the placements of each source image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
 pub canvas_w: u32,
 pub canvas_h: u32,
 pub placements: Vec<Placement>,
}

/// Stack images top-to-bottom, horizontally centered, separated by `gap` px.
pub fn stack_vertical(sizes: &[(u32, u32)], gap: u32) -> Layout {
 let canvas_w = sizes.iter().map(|s| s.0).max().unwrap_or(0);
 let mut placements = Vec::with_capacity(sizes.len());
 let mut y: i64 = 0;
 for (i, (w, h)) in sizes.iter().enumerate() {
 let x = ((canvas_w as i64) - (*w as i64)) / 2;
 placements.push(Placement { image: i, x, y });
 y += *h as i64 + gap as i64;
 }
 let canvas_h = if sizes.is_empty() { 0 } else { (y - gap as i64).max(0) as u32 };
 Layout { canvas_w, canvas_h, placements }
}

/// Stack images left-to-right, vertically centered, separated by `gap` px.
pub fn stack_horizontal(sizes: &[(u32, u32)], gap: u32) -> Layout {
 let canvas_h = sizes.iter().map(|s| s.1).max().unwrap_or(0);
 let mut placements = Vec::with_capacity(sizes.len());
 let mut x: i64 = 0;
 for (i, (w, h)) in sizes.iter().enumerate() {
 let y = ((canvas_h as i64) - (*h as i64)) / 2;
 placements.push(Placement { image: i, x, y });
 x += *w as i64 + gap as i64;
 }
 let canvas_w = if sizes.is_empty() { 0 } else { (x - gap as i64).max(0) as u32 };
 Layout { canvas_w, canvas_h, placements }
}

/// Tight canvas size needed to contain a free-form set of placements.
pub fn bounding(sizes: &[(u32, u32)], placements: &[Placement]) -> (u32, u32) {
 let mut w: i64 = 0;
 let mut h: i64 = 0;
 for p in placements {
 if let Some((iw, ih)) = sizes.get(p.image) {
 w = w.max(p.x + *iw as i64);
 h = h.max(p.y + *ih as i64);
 }
 }
 (w.max(0) as u32, h.max(0) as u32)
}

/// Compose images onto a new canvas filled with `bg`, honoring the layout.
pub fn compose(images: &[RgbaImage], layout: &Layout, bg: Rgba<u8>) -> RgbaImage {
 let mut canvas = RgbaImage::from_pixel(layout.canvas_w.max(1), layout.canvas_h.max(1), bg);
 for p in &layout.placements {
 if let Some(src) = images.get(p.image) {
 image::imageops::overlay(&mut canvas, src, p.x, p.y);
 }
 }
 canvas
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn vertical_stack_dimensions_and_centering() {
 let sizes = [(100u32, 40u32), (60u32, 30u32)];
 let l = stack_vertical(&sizes, 10);
 assert_eq!(l.canvas_w, 100);
 assert_eq!(l.canvas_h, 40 + 10 + 30);
 assert_eq!(l.placements[0], Placement { image: 0, x: 0, y: 0 });
 assert_eq!(l.placements[1], Placement { image: 1, x: 20, y: 50 });
 }

 #[test]
 fn horizontal_stack_dimensions() {
 let sizes = [(40u32, 100u32), (30u32, 60u32)];
 let l = stack_horizontal(&sizes, 5);
 assert_eq!(l.canvas_h, 100);
 assert_eq!(l.canvas_w, 40 + 5 + 30);
 assert_eq!(l.placements[1].x, 45);
 assert_eq!(l.placements[1].y, 20);
 }

 #[test]
 fn bounding_of_free_placements() {
 let sizes = [(20u32, 20u32), (20u32, 20u32)];
 let pl = [Placement { image: 0, x: 0, y: 0 }, Placement { image: 1, x: 30, y: 15 }];
 assert_eq!(bounding(&sizes, &pl), (50, 35));
 }

 #[test]
 fn compose_blits_images() {
 let red = RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255]));
 let blue = RgbaImage::from_pixel(4, 4, Rgba([0, 0, 255, 255]));
 let l = stack_vertical(&[(4, 4), (4, 4)], 0);
 let out = compose(&[red, blue], &l, Rgba([0, 0, 0, 255]));
 assert_eq!(out.width(), 4);
 assert_eq!(out.height(), 8);
 assert_eq!(out.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
 assert_eq!(out.get_pixel(0, 4), &Rgba([0, 0, 255, 255]));
 }
}
