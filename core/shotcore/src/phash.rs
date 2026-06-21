//! Perceptual image hashing (average hash + difference hash) for the capture history:
//! detect near-duplicate screenshots and find visually similar captures. Pure pixel math,
//! no native dependency. 64-bit hashes compared by Hamming distance.
use image::imageops::FilterType;
use image::RgbaImage;

fn luma(p: [u8; 4]) -> f32 {
 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
}

/// 64-bit average hash: downscale to 8x8 grayscale, set each bit where the pixel luma is at
/// or above the mean.
pub fn ahash(img: &RgbaImage) -> u64 {
 let small = image::imageops::resize(img, 8, 8, FilterType::Triangle);
 let mut lumas = [0f32; 64];
 let mut sum = 0f32;
 for (i, px) in small.pixels().enumerate() {
 let l = luma(px.0);
 lumas[i] = l;
 sum += l;
 }
 let mean = sum / 64.0;
 let mut hash = 0u64;
 for i in 0..64 {
 if lumas[i] >= mean {
 hash |= 1u64 << i;
 }
 }
 hash
}

/// 64-bit difference hash: downscale to 9x8 grayscale, set each bit where a pixel is brighter
/// than its right neighbor (8 comparisons per row, 8 rows).
pub fn dhash(img: &RgbaImage) -> u64 {
 let small = image::imageops::resize(img, 9, 8, FilterType::Triangle);
 let mut hash = 0u64;
 let mut bit = 0u32;
 for y in 0..8u32 {
 for x in 0..8u32 {
 let left = luma(small.get_pixel(x, y).0);
 let right = luma(small.get_pixel(x + 1, y).0);
 if left > right {
 hash |= 1u64 << bit;
 }
 bit += 1;
 }
 }
 hash
}

/// Hamming distance between two hashes (0..=64).
pub fn hamming(a: u64, b: u64) -> u32 {
 (a ^ b).count_ones()
}

/// Whether two image hashes are perceptually similar within a max Hamming distance.
pub fn is_similar(a: u64, b: u64, max_distance: u32) -> bool {
 hamming(a, b) <= max_distance
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 fn gradient(invert: bool) -> RgbaImage {
 let mut img = RgbaImage::new(64, 64);
 for y in 0..64u32 {
 for x in 0..64u32 {
 let v = ((x * 4) % 256) as u8;
 let g = if invert { 255 - v } else { v };
 img.put_pixel(x, y, Rgba([g, g, g, 255]));
 }
 }
 img
 }

 #[test]
 fn identical_images_match() {
 let a = gradient(false);
 let b = gradient(false);
 assert_eq!(hamming(ahash(&a), ahash(&b)), 0);
 assert_eq!(hamming(dhash(&a), dhash(&b)), 0);
 }

 #[test]
 fn near_duplicate_is_similar_but_inverse_is_not() {
 let base = gradient(false);
 let mut near = gradient(false);
 for y in 0..6u32 {
 for x in 0..6u32 {
 near.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 assert!(is_similar(ahash(&base), ahash(&near), 10));
 let inv = gradient(true);
 assert!(hamming(dhash(&base), dhash(&inv)) >= 24);
 }
}
