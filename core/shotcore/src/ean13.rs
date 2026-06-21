//! EAN-13 1D barcode: encode 13 digits to a 95-module pattern (and render bars), and decode
//! from an RGBA buffer by reading the middle-row run lengths. Pure logic implementing the
//! public EAN-13 spec (L/G/R encodings, first-digit parity, mod-10 checksum). Complements the
//! QR codec - SloerShot reads and writes 1D barcodes on-device with no native dependency.
use image::{Rgba, RgbaImage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EanError {
 BadLength,
 BadDigit,
 BadChecksum,
 NotFound,
 Decode,
}

const L_CODE: [[u8; 7]; 10] = [
 [0, 0, 0, 1, 1, 0, 1],
 [0, 0, 1, 1, 0, 0, 1],
 [0, 0, 1, 0, 0, 1, 1],
 [0, 1, 1, 1, 1, 0, 1],
 [0, 1, 0, 0, 0, 1, 1],
 [0, 1, 1, 0, 0, 0, 1],
 [0, 1, 0, 1, 1, 1, 1],
 [0, 1, 1, 1, 0, 1, 1],
 [0, 1, 1, 0, 1, 1, 1],
 [0, 0, 0, 1, 0, 1, 1],
];

const PARITY: [[u8; 6]; 10] = [
 [0, 0, 0, 0, 0, 0],
 [0, 0, 1, 0, 1, 1],
 [0, 0, 1, 1, 0, 1],
 [0, 0, 1, 1, 1, 0],
 [0, 1, 0, 0, 1, 1],
 [0, 1, 1, 0, 0, 1],
 [0, 1, 1, 1, 0, 0],
 [0, 1, 0, 1, 0, 1],
 [0, 1, 0, 1, 1, 0],
 [0, 1, 1, 0, 1, 0],
];

fn r_code(d: u8) -> [u8; 7] {
 let l = L_CODE[d as usize];
 let mut r = [0u8; 7];
 for i in 0..7 {
 r[i] = 1 - l[i];
 }
 r
}
fn g_code(d: u8) -> [u8; 7] {
 let r = r_code(d);
 let mut g = [0u8; 7];
 for i in 0..7 {
 g[i] = r[6 - i];
 }
 g
}

/// EAN-13 check digit for the first 12 digits.
pub fn checksum(d: &[u8]) -> u8 {
 let mut sum = 0u32;
 for (i, &v) in d.iter().take(12).enumerate() {
 sum += if i % 2 == 0 { v as u32 } else { 3 * v as u32 };
 }
 ((10 - (sum % 10)) % 10) as u8
}

fn push(bits: &mut Vec<bool>, pat: &[u8]) {
 for &b in pat {
 bits.push(b == 1);
 }
}

/// Encode 12 digits (check appended) or 13 digits (check validated) into a 95-module pattern.
pub fn encode(digits: &[u8]) -> Result<Vec<bool>, EanError> {
 if digits.len() != 12 && digits.len() != 13 {
 return Err(EanError::BadLength);
 }
 if digits.iter().any(|&x| x > 9) {
 return Err(EanError::BadDigit);
 }
 let mut d = digits.to_vec();
 if d.len() == 12 {
 let cs = checksum(&d);
 d.push(cs);
 }
 if checksum(&d[..12]) != d[12] {
 return Err(EanError::BadChecksum);
 }
 let mut bits: Vec<bool> = Vec::with_capacity(95);
 push(&mut bits, &[1, 0, 1]);
 let parity = PARITY[d[0] as usize];
 for i in 0..6 {
 let digit = d[1 + i];
 let pat = if parity[i] == 0 { L_CODE[digit as usize] } else { g_code(digit) };
 push(&mut bits, &pat);
 }
 push(&mut bits, &[0, 1, 0, 1, 0]);
 for i in 0..6 {
 push(&mut bits, &r_code(d[7 + i]));
 }
 push(&mut bits, &[1, 0, 1]);
 Ok(bits)
}

/// Render an EAN-13 module pattern to a barcode image (bars span the full height).
pub fn render(pattern: &[bool], scale: u32, height: u32, quiet: u32) -> RgbaImage {
 let w = (pattern.len() as u32 + 2 * quiet) * scale.max(1);
 let h = height.max(1);
 let mut img = RgbaImage::from_pixel(w.max(1), h, Rgba([255, 255, 255, 255]));
 for (i, &dark) in pattern.iter().enumerate() {
 if dark {
 let x0 = (i as u32 + quiet) * scale.max(1);
 for x in x0..x0 + scale.max(1) {
 for y in 0..h {
 img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
 }
 }
 }
 }
 img
}

fn is_dark(img: &RgbaImage, x: u32, y: u32) -> bool {
 let p = img.get_pixel(x, y).0;
 if p[3] < 128 {
 return false;
 }
 (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) < 128.0
}

fn group_eq(bits: &[bool], code: &[u8; 7]) -> bool {
 bits.len() == 7 && (0..7).all(|i| bits[i] == (code[i] == 1))
}

fn match_left(g: &[bool]) -> Option<(u8, u8)> {
 for d in 0..10u8 {
 if group_eq(g, &L_CODE[d as usize]) {
 return Some((d, 0));
 }
 if group_eq(g, &g_code(d)) {
 return Some((d, 1));
 }
 }
 None
}

fn match_right(g: &[bool]) -> Option<u8> {
 (0..10u8).find(|&d| group_eq(g, &r_code(d)))
}

/// Decode an EAN-13 barcode from an RGBA image (clean, axis-aligned). Returns the 13 digits.
pub fn decode(img: &RgbaImage) -> Result<String, EanError> {
 let (w, h) = img.dimensions();
 if w == 0 || h == 0 {
 return Err(EanError::NotFound);
 }
 let y = h / 2;
 let mut x0: Option<u32> = None;
 let mut x1 = 0u32;
 for x in 0..w {
 if is_dark(img, x, y) {
 if x0.is_none() {
 x0 = Some(x);
 }
 x1 = x;
 }
 }
 let x0 = x0.ok_or(EanError::NotFound)?;
 let module = (x1 - x0 + 1) as f64 / 95.0;
 if module < 0.5 {
 return Err(EanError::NotFound);
 }
 let mut bits: Vec<bool> = Vec::with_capacity(95);
 for i in 0..95 {
 let cx = x0 as f64 + (i as f64 + 0.5) * module;
 let xi = (cx as u32).min(w - 1);
 bits.push(is_dark(img, xi, y));
 }
 if !(bits[0] && !bits[1] && bits[2]) {
 return Err(EanError::Decode);
 }
 let mut parity = [0u8; 6];
 let mut left = [0u8; 6];
 for i in 0..6 {
 match match_left(&bits[3 + i * 7..3 + i * 7 + 7]) {
 Some((d, p)) => {
 left[i] = d;
 parity[i] = p;
 }
 None => return Err(EanError::Decode),
 }
 }
 let first = PARITY.iter().position(|row| row == &parity).ok_or(EanError::Decode)? as u8;
 let c = 45;
 if !(!bits[c] && bits[c + 1] && !bits[c + 2] && bits[c + 3] && !bits[c + 4]) {
 return Err(EanError::Decode);
 }
 let mut right = [0u8; 6];
 for i in 0..6 {
 match match_right(&bits[50 + i * 7..50 + i * 7 + 7]) {
 Some(d) => right[i] = d,
 None => return Err(EanError::Decode),
 }
 }
 let mut all: Vec<u8> = Vec::with_capacity(13);
 all.push(first);
 all.extend_from_slice(&left);
 all.extend_from_slice(&right);
 if checksum(&all[..12]) != all[12] {
 return Err(EanError::BadChecksum);
 }
 Ok(all.iter().map(|d| (48u8 + d) as char).collect())
}

#[cfg(test)]
mod tests {
 use super::*;

 fn roundtrip(d12: &[u8], expected: &str) {
 let pattern = encode(d12).unwrap();
 assert_eq!(pattern.len(), 95);
 let img = render(&pattern, 3, 40, 10);
 assert_eq!(decode(&img).unwrap(), expected);
 }

 #[test]
 fn roundtrips_known_codes() {
 roundtrip(&[5, 9, 0, 1, 2, 3, 4, 1, 2, 3, 4, 5], "5901234123457");
 roundtrip(&[4, 0, 0, 6, 3, 8, 1, 3, 3, 3, 9, 3], "4006381333931");
 }

 #[test]
 fn rejects_bad_input() {
 assert_eq!(encode(&[1, 2, 3]), Err(EanError::BadLength));
 assert_eq!(encode(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10]), Err(EanError::BadDigit));
 }

 #[test]
 fn checksum_matches_spec() {
 assert_eq!(checksum(&[5, 9, 0, 1, 2, 3, 4, 1, 2, 3, 4, 5]), 7);
 assert_eq!(checksum(&[4, 0, 0, 6, 3, 8, 1, 3, 3, 3, 9, 3]), 1);
 }
}
