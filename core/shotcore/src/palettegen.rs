//! Dominant-color extraction (median-cut quantization) from a screenshot. Used to suggest a
//! matching gradient background (Background tool) or a color palette. Pure pixel math, tested.
use crate::model::Color;
use image::RgbaImage;

fn channel_range(bucket: &[[u8; 3]], ch: usize) -> u8 {
 let mut lo = 255u8;
 let mut hi = 0u8;
 for p in bucket {
 lo = lo.min(p[ch]);
 hi = hi.max(p[ch]);
 }
 hi - lo
}

fn widest_channel(bucket: &[[u8; 3]]) -> usize {
 let r = channel_range(bucket, 0);
 let g = channel_range(bucket, 1);
 let b = channel_range(bucket, 2);
 if r >= g && r >= b {
 0
 } else if g >= b {
 1
 } else {
 2
 }
}

fn average(bucket: &[[u8; 3]]) -> Color {
 let n = bucket.len().max(1) as u64;
 let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
 for p in bucket {
 r += p[0] as u64;
 g += p[1] as u64;
 b += p[2] as u64;
 }
 Color::rgba((r / n) as u8, (g / n) as u8, (b / n) as u8, 255)
}

/// Extract up to `k` dominant colors via median-cut, largest bucket first.
pub fn dominant_colors(img: &RgbaImage, k: usize) -> Vec<Color> {
 if k == 0 {
 return Vec::new();
 }
 let total = (img.width() * img.height()) as usize;
 let step = (total / 4096).max(1);
 let mut pixels: Vec<[u8; 3]> = Vec::new();
 for (i, px) in img.pixels().enumerate() {
 if i % step == 0 && px.0[3] >= 8 {
 pixels.push([px.0[0], px.0[1], px.0[2]]);
 }
 }
 if pixels.is_empty() {
 return Vec::new();
 }
 let mut buckets: Vec<Vec<[u8; 3]>> = vec![pixels];
 while buckets.len() < k {
 let mut best = None;
 let mut best_range = 0u8;
 for (i, b) in buckets.iter().enumerate() {
 if b.len() < 2 {
 continue;
 }
 let rng = channel_range(b, widest_channel(b));
 if best.is_none() || rng > best_range {
 best = Some(i);
 best_range = rng;
 }
 }
 let idx = match best {
 Some(i) => i,
 None => break,
 };
 let mut b = buckets.remove(idx);
 let ch = widest_channel(&b);
 b.sort_by_key(|p| p[ch]);
 let right = b.split_off(b.len() / 2);
 buckets.push(b);
 buckets.push(right);
 }
 buckets.sort_by(|a, b| b.len().cmp(&a.len()));
 buckets.iter().map(|b| average(b)).collect()
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn splits_two_regions() {
 let mut img = RgbaImage::new(64, 64);
 for y in 0..64u32 {
 for x in 0..64u32 {
 let c = if x < 32 { Rgba([200, 0, 0, 255]) } else { Rgba([0, 0, 200, 255]) };
 img.put_pixel(x, y, c);
 }
 }
 let cols = dominant_colors(&img, 2);
 assert_eq!(cols.len(), 2);
 assert!(cols.iter().any(|c| c.r > 100));
 assert!(cols.iter().any(|c| c.b > 100));
 }

 #[test]
 fn solid_and_zero() {
 let img = RgbaImage::from_pixel(16, 16, Rgba([20, 40, 60, 255]));
 let cols = dominant_colors(&img, 4);
 assert_eq!(cols[0], Color::rgba(20, 40, 60, 255));
 assert!(dominant_colors(&img, 0).is_empty());
 }
}
