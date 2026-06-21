//! Pure-Rust QR code encode + decode (byte mode, versions 1-3, ECC level M, single
//! Reed-Solomon block). Screenshots render QR symbols crisply and axis-aligned, so
//! SloerShot both GENERATES and SCANS QR codes fully on-device with no native
//! dependency. GF(256) arithmetic with primitive polynomial 0x11d; format info via
//! BCH(15,5). This is the real in-core decode path the recognize module documented.
use image::RgbaImage;

/// Errors from QR encode/decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QrError {
 TooLong,
 NotFound,
 Format,
 Decode,
}
impl std::fmt::Display for QrError {
 fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
 let s = match self {
 QrError::TooLong => "data too long for supported versions (1-3, level M)",
 QrError::NotFound => "no QR symbol found in image",
 QrError::Format => "could not read QR format information",
 QrError::Decode => "could not decode QR data",
 };
 write!(f, "{}", s)
 }
}
impl std::error::Error for QrError {}

// ---------- GF(256) field ----------
struct Gf {
 exp: [u8; 512],
 log: [u8; 256],
}
impl Gf {
 fn new() -> Self {
 let mut exp = [0u8; 512];
 let mut log = [0u8; 256];
 let mut x: u16 = 1;
 for i in 0..255 {
 exp[i] = x as u8;
 log[x as usize] = i as u8;
 x <<= 1;
 if x & 0x100 != 0 {
 x ^= 0x11d;
 }
 }
 for i in 255..512 {
 exp[i] = exp[i - 255];
 }
 Gf { exp, log }
 }
 fn mul(&self, a: u8, b: u8) -> u8 {
 if a == 0 || b == 0 {
 return 0;
 }
 self.exp[self.log[a as usize] as usize + self.log[b as usize] as usize]
 }
 fn pow(&self, n: i32) -> u8 {
 let m = (((n % 255) + 255) % 255) as usize;
 self.exp[m]
 }
 fn inv(&self, a: u8) -> u8 {
 self.exp[255 - self.log[a as usize] as usize]
 }
 fn div(&self, a: u8, b: u8) -> u8 {
 if a == 0 {
 return 0;
 }
 self.exp[(self.log[a as usize] as usize + 255 - self.log[b as usize] as usize) % 255]
 }
}

fn rs_generator(gf: &Gf, ec: usize) -> Vec<u8> {
 let mut g = vec![1u8];
 for i in 0..ec {
 let a = gf.pow(i as i32);
 let mut ng = vec![0u8; g.len() + 1];
 for (j, &c) in g.iter().enumerate() {
 ng[j] ^= c;
 ng[j + 1] ^= gf.mul(c, a);
 }
 g = ng;
 }
 g
}

fn rs_encode(gf: &Gf, data: &[u8], ec: usize) -> Vec<u8> {
 let gen = rs_generator(gf, ec);
 let mut res = vec![0u8; data.len() + ec];
 res[..data.len()].copy_from_slice(data);
 for i in 0..data.len() {
 let coef = res[i];
 if coef != 0 {
 for j in 0..gen.len() {
 let g = gf.mul(gen[j], coef);
 res[i + j] ^= g;
 }
 }
 }
 res[data.len()..].to_vec()
}

// ---------- version specs (level M, single block) ----------
#[derive(Clone, Copy)]
struct Spec {
 version: usize,
 size: usize,
 data_cw: usize,
 ec_cw: usize,
 align: usize,
}
const SPECS: [Spec; 3] = [
 Spec { version: 1, size: 21, data_cw: 16, ec_cw: 10, align: 0 },
 Spec { version: 2, size: 25, data_cw: 28, ec_cw: 16, align: 18 },
 Spec { version: 3, size: 29, data_cw: 44, ec_cw: 26, align: 22 },
];

fn spec_for_len(n: usize) -> Option<Spec> {
 SPECS.iter().copied().find(|s| n * 8 + 12 + 4 <= s.data_cw * 8)
}
fn spec_for_size(size: usize) -> Option<Spec> {
 SPECS.iter().copied().find(|s| s.size == size)
}

fn push_bits(bits: &mut Vec<bool>, val: u32, n: usize) {
 for i in (0..n).rev() {
 bits.push((val >> i) & 1 == 1);
 }
}

fn byte_codewords(bytes: &[u8], data_cw: usize) -> Vec<u8> {
 let mut bits: Vec<bool> = Vec::new();
 push_bits(&mut bits, 0b0100, 4);
 push_bits(&mut bits, bytes.len() as u32, 8);
 for &b in bytes {
 push_bits(&mut bits, b as u32, 8);
 }
 let cap = data_cw * 8;
 let term = core::cmp::min(4, cap - bits.len());
 for _ in 0..term {
 bits.push(false);
 }
 while bits.len() % 8 != 0 {
 bits.push(false);
 }
 let mut cw: Vec<u8> = bits
 .chunks(8)
 .map(|c| {
 let mut v = 0u8;
 for (i, &bit) in c.iter().enumerate() {
 if bit {
 v |= 1 << (7 - i);
 }
 }
 v
 })
 .collect();
 let pads = [0xECu8, 0x11u8];
 let mut k = 0;
 while cw.len() < data_cw {
 cw.push(pads[k % 2]);
 k += 1;
 }
 cw
}

// ---------- module matrix ----------
#[derive(Clone)]
struct Matrix {
 size: usize,
 cell: Vec<Option<bool>>,
 func: Vec<bool>,
}
impl Matrix {
 fn new(size: usize) -> Self {
 Matrix { size, cell: vec![None; size * size], func: vec![false; size * size] }
 }
 fn at(&self, r: usize, c: usize) -> usize {
 r * self.size + c
 }
 fn set(&mut self, r: usize, c: usize, v: bool, f: bool) {
 let i = self.at(r, c);
 self.cell[i] = Some(v);
 if f {
 self.func[i] = true;
 }
 }
 fn get(&self, r: usize, c: usize) -> bool {
 self.cell[self.at(r, c)].unwrap_or(false)
 }
 fn is_func(&self, r: usize, c: usize) -> bool {
 self.func[self.at(r, c)]
 }
}

fn place_finder(m: &mut Matrix, r0: i32, c0: i32) {
 for dr in -1i32..=7 {
 for dc in -1i32..=7 {
 let r = r0 + dr;
 let c = c0 + dc;
 if r < 0 || c < 0 || r >= m.size as i32 || c >= m.size as i32 {
 continue;
 }
 let inside = (0..=6).contains(&dr) && (0..=6).contains(&dc);
 let dark = if inside {
 let edge = dr == 0 || dr == 6 || dc == 0 || dc == 6;
 let core = (2..=4).contains(&dr) && (2..=4).contains(&dc);
 edge || core
 } else {
 false
 };
 m.set(r as usize, c as usize, dark, true);
 }
 }
}

fn place_alignment(m: &mut Matrix, center: usize) {
 let cc = center as i32;
 for dr in -2i32..=2 {
 for dc in -2i32..=2 {
 let r = (cc + dr) as usize;
 let c = (cc + dc) as usize;
 let edge = dr.abs() == 2 || dc.abs() == 2;
 let mid = dr == 0 && dc == 0;
 m.set(r, c, edge || mid, true);
 }
 }
}

fn place_function(m: &mut Matrix, spec: Spec) {
 let n = m.size;
 place_finder(m, 0, 0);
 place_finder(m, 0, n as i32 - 7);
 place_finder(m, n as i32 - 7, 0);
 // timing patterns
 for i in 8..(n - 8) {
 let v = i % 2 == 0;
 if !m.is_func(6, i) {
 m.set(6, i, v, true);
 }
 if !m.is_func(i, 6) {
 m.set(i, 6, v, true);
 }
 }
 // alignment pattern (v2-3)
 if spec.align != 0 {
 place_alignment(m, spec.align);
 }
 // always-dark module
 m.set(n - 8, 8, true, true);
 // reserve the two format-info strips (value filled later)
 for (r, c) in format_positions_primary(n) {
 m.set(r, c, false, true);
 }
 for (r, c) in format_positions_secondary(n) {
 m.set(r, c, false, true);
 }
}

// ---------- format information (level M = indicator bits 00) ----------
const ECL_M: u32 = 0b00;

fn format_bits(ecl: u32, mask: u32) -> u32 {
 let data = (ecl << 3) | mask; // 5 bits
 let mut rem = data << 10;
 for i in (0..5).rev() {
 if (rem >> (10 + i)) & 1 == 1 {
 rem ^= 0x537 << i;
 }
 }
 ((data << 10) | rem) ^ 0x5412
}

fn format_positions_primary(n: usize) -> [(usize, usize); 15] {
 [
 (8, 0), (8, 1), (8, 2), (8, 3), (8, 4), (8, 5), (8, 7), (8, 8),
 (7, 8), (5, 8), (4, 8), (3, 8), (2, 8), (1, 8), (0, 8),
 ]
 .map(|(r, c)| (r, c))
 .map(|p| {
 let _ = n;
 p
 })
}

fn format_positions_secondary(n: usize) -> [(usize, usize); 15] {
 [
 (n - 1, 8), (n - 2, 8), (n - 3, 8), (n - 4, 8), (n - 5, 8), (n - 6, 8), (n - 7, 8),
 (8, n - 8), (8, n - 7), (8, n - 6), (8, n - 5), (8, n - 4), (8, n - 3), (8, n - 2), (8, n - 1),
 ]
}

fn write_format(m: &mut Matrix, mask: u32) {
 let bits = format_bits(ECL_M, mask);
 let n = m.size;
 for (k, (r, c)) in format_positions_primary(n).iter().enumerate() {
 let bit = (bits >> (14 - k)) & 1 == 1;
 m.set(*r, *c, bit, true);
 }
 for (k, (r, c)) in format_positions_secondary(n).iter().enumerate() {
 let bit = (bits >> (14 - k)) & 1 == 1;
 m.set(*r, *c, bit, true);
 }
}

fn read_format_mask(m: &Matrix) -> Option<u32> {
 let n = m.size;
 let mut raw = 0u32;
 for (k, (r, c)) in format_positions_primary(n).iter().enumerate() {
 if m.get(*r, *c) {
 raw |= 1 << (14 - k);
 }
 }
 // match against the 8 valid level-M format strings by Hamming distance
 let mut best = (16u32, 0u32);
 for mask in 0..8u32 {
 let cand = format_bits(ECL_M, mask);
 let dist = (raw ^ cand).count_ones();
 if dist < best.0 {
 best = (dist, mask);
 }
 }
 if best.0 <= 3 {
 Some(best.1)
 } else {
 None
 }
}

// ---------- data placement (zigzag) + masking ----------
fn data_order(m: &Matrix) -> Vec<(usize, usize)> {
 let n = m.size as i32;
 let mut order = Vec::new();
 let mut col = n - 1;
 let mut upward = true;
 while col > 0 {
 if col == 6 {
 col -= 1;
 }
 let mut t = 0;
 while t < n {
 let row = if upward { n - 1 - t } else { t };
 for dc in 0..2 {
 let c = col - dc;
 let idx = (row as usize) * m.size + (c as usize);
 if !m.func[idx] {
 order.push((row as usize, c as usize));
 }
 }
 t += 1;
 }
 col -= 2;
 upward = !upward;
 }
 order
}

fn place_data(m: &mut Matrix, codewords: &[u8]) {
 let order = data_order(m);
 let mut bits = Vec::with_capacity(codewords.len() * 8);
 for &b in codewords {
 for i in (0..8).rev() {
 bits.push((b >> i) & 1 == 1);
 }
 }
 for (k, (r, c)) in order.iter().enumerate() {
 let v = if k < bits.len() { bits[k] } else { false };
 m.set(*r, *c, v, false);
 }
}

fn read_data(m: &Matrix) -> Vec<u8> {
 let order = data_order(m);
 let mut bytes = Vec::new();
 let mut cur = 0u8;
 let mut nb = 0u32;
 for (r, c) in order {
 cur = (cur << 1) | (m.get(r, c) as u8);
 nb += 1;
 if nb == 8 {
 bytes.push(cur);
 cur = 0;
 nb = 0;
 }
 }
 bytes
}

fn mask_cond(mask: u32, r: usize, c: usize) -> bool {
 match mask {
 0 => (r + c) % 2 == 0,
 1 => r % 2 == 0,
 2 => c % 3 == 0,
 3 => (r + c) % 3 == 0,
 4 => (r / 2 + c / 3) % 2 == 0,
 5 => (r * c) % 2 + (r * c) % 3 == 0,
 6 => ((r * c) % 2 + (r * c) % 3) % 2 == 0,
 7 => ((r + c) % 2 + (r * c) % 3) % 2 == 0,
 _ => false,
 }
}

fn apply_mask(m: &mut Matrix, mask: u32) {
 for r in 0..m.size {
 for c in 0..m.size {
 if !m.is_func(r, c) && mask_cond(mask, r, c) {
 let i = m.at(r, c);
 if let Some(v) = m.cell[i] {
 m.cell[i] = Some(!v);
 }
 }
 }
 }
}

fn penalty(m: &Matrix) -> u32 {
 let n = m.size;
 let mut score = 0u32;
 for r in 0..n {
 let mut run = 1u32;
 for c in 1..n {
 if m.get(r, c) == m.get(r, c - 1) {
 run += 1;
 } else {
 if run >= 5 {
 score += 3 + (run - 5);
 }
 run = 1;
 }
 }
 if run >= 5 {
 score += 3 + (run - 5);
 }
 }
 for c in 0..n {
 let mut run = 1u32;
 for r in 1..n {
 if m.get(r, c) == m.get(r - 1, c) {
 run += 1;
 } else {
 if run >= 5 {
 score += 3 + (run - 5);
 }
 run = 1;
 }
 }
 if run >= 5 {
 score += 3 + (run - 5);
 }
 }
 for r in 0..n - 1 {
 for c in 0..n - 1 {
 let v = m.get(r, c);
 if m.get(r, c + 1) == v && m.get(r + 1, c) == v && m.get(r + 1, c + 1) == v {
 score += 3;
 }
 }
 }
 let pat1 = [true, false, true, true, true, false, true, false, false, false, false];
 let pat2 = [false, false, false, false, true, false, true, true, true, false, true];
 for r in 0..n {
 for c in 0..n {
 if c + 11 <= n {
 let row_run: Vec<bool> = (0..11).map(|k| m.get(r, c + k)).collect();
 if row_run == pat1 || row_run == pat2 {
 score += 40;
 }
 }
 if r + 11 <= n {
 let col_run: Vec<bool> = (0..11).map(|k| m.get(r + k, c)).collect();
 if col_run == pat1 || col_run == pat2 {
 score += 40;
 }
 }
 }
 }
 let dark = (0..n * n).filter(|&i| m.cell[i] == Some(true)).count();
 let total = n * n;
 let percent = dark * 100 / total;
 let lower = percent - percent % 5;
 let upper = lower + 5;
 let d1 = (lower as i32 - 50).abs();
 let d2 = (upper as i32 - 50).abs();
 score += (d1.min(d2) as u32) / 5 * 10;
 score
}

fn best_mask(m: &Matrix) -> u32 {
 let mut best = (u32::MAX, 0u32);
 for mask in 0..8u32 {
 let mut trial = m.clone();
 apply_mask(&mut trial, mask);
 write_format(&mut trial, mask);
 let p = penalty(&trial);
 if p < best.0 {
 best = (p, mask);
 }
 }
 best.1
}

// ---------- public encode + render ----------
/// Encode text into a QR module grid (true = dark). Returns (version, grid).
pub fn encode(text: &str) -> Result<(usize, Vec<Vec<bool>>), QrError> {
 let bytes = text.as_bytes();
 let spec = spec_for_len(bytes.len()).ok_or(QrError::TooLong)?;
 let gf = Gf::new();
 let data = byte_codewords(bytes, spec.data_cw);
 let ec = rs_encode(&gf, &data, spec.ec_cw);
 let mut all = data;
 all.extend_from_slice(&ec);
 let mut m = Matrix::new(spec.size);
 place_function(&mut m, spec);
 place_data(&mut m, &all);
 let mask = best_mask(&m);
 apply_mask(&mut m, mask);
 write_format(&mut m, mask);
 let grid = (0..spec.size)
 .map(|r| (0..spec.size).map(|c| m.get(r, c)).collect())
 .collect();
 Ok((spec.version, grid))
}

/// Render a QR grid to an RGBA image at `scale` px per module with a `quiet`-module border.
pub fn render(grid: &[Vec<bool>], scale: u32, quiet: u32) -> RgbaImage {
 let n = grid.len() as u32;
 let dim = ((n + 2 * quiet) * scale).max(1);
 let mut img = RgbaImage::from_pixel(dim, dim, image::Rgba([255, 255, 255, 255]));
 for (r, row) in grid.iter().enumerate() {
 for (c, &dark) in row.iter().enumerate() {
 if dark {
 let x0 = (c as u32 + quiet) * scale;
 let y0 = (r as u32 + quiet) * scale;
 for y in y0..y0 + scale {
 for x in x0..x0 + scale {
 img.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
 }
 }
 }
 }
 }
 img
}

// ---------- decode: image -> module grid ----------
fn is_dark(img: &RgbaImage, x: u32, y: u32) -> bool {
 let p = img.get_pixel(x, y);
 if p[3] < 128 {
 return false;
 }
 let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
 lum < 128.0
}

fn dark_bbox(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
 let (w, h) = img.dimensions();
 let (mut minx, mut miny, mut maxx, mut maxy) = (w, h, 0u32, 0u32);
 let mut found = false;
 for y in 0..h {
 for x in 0..w {
 if is_dark(img, x, y) {
 found = true;
 minx = minx.min(x);
 miny = miny.min(y);
 maxx = maxx.max(x);
 maxy = maxy.max(y);
 }
 }
 }
 if found {
 Some((minx, miny, maxx, maxy))
 } else {
 None
 }
}

fn sample_grid(img: &RgbaImage) -> Option<(usize, Vec<Vec<bool>>)> {
 let (minx, miny, maxx, maxy) = dark_bbox(img)?;
 let span_x = maxx - minx + 1;
 let span_y = maxy - miny + 1;
 let mut run = 0u32;
 let mut x = minx;
 while x <= maxx && is_dark(img, x, miny) {
 run += 1;
 x += 1;
 }
 if run == 0 {
 return None;
 }
 let module_px = (run as f32 / 7.0).max(1.0);
 let size = (span_x.max(span_y) as f32 / module_px).round() as usize;
 let spec = spec_for_size(size)?;
 let n = spec.size;
 let step_x = span_x as f32 / n as f32;
 let step_y = span_y as f32 / n as f32;
 let mut grid = vec![vec![false; n]; n];
 for r in 0..n {
 for c in 0..n {
 let px = minx as f32 + (c as f32 + 0.5) * step_x;
 let py = miny as f32 + (r as f32 + 0.5) * step_y;
 let xi = (px as u32).min(img.width() - 1);
 let yi = (py as u32).min(img.height() - 1);
 grid[r][c] = is_dark(img, xi, yi);
 }
 }
 Some((n, grid))
}

// ---------- Reed-Solomon decode (BM + Chien + GF Gaussian) ----------
fn gf_solve(gf: &Gf, mut a: Vec<Vec<u8>>, mut b: Vec<u8>) -> Option<Vec<u8>> {
 let t = b.len();
 for col in 0..t {
 let mut prow = None;
 for r in col..t {
 if a[r][col] != 0 {
 prow = Some(r);
 break;
 }
 }
 let pr = prow?;
 a.swap(col, pr);
 b.swap(col, pr);
 let inv = gf.inv(a[col][col]);
 for j in col..t {
 a[col][j] = gf.mul(a[col][j], inv);
 }
 b[col] = gf.mul(b[col], inv);
 for r in 0..t {
 if r != col && a[r][col] != 0 {
 let f = a[r][col];
 for j in col..t {
 let m = gf.mul(f, a[col][j]);
 a[r][j] ^= m;
 }
 let m = gf.mul(f, b[col]);
 b[r] ^= m;
 }
 }
 }
 Some(b)
}

fn rs_decode(gf: &Gf, received: &[u8], ec: usize) -> Result<Vec<u8>, QrError> {
 let n = received.len();
 let mut syn = vec![0u8; ec];
 let mut nonzero = false;
 for i in 0..ec {
 let mut s = 0u8;
 let x = gf.pow(i as i32);
 for &b in received {
 s = gf.mul(s, x) ^ b;
 }
 syn[i] = s;
 if s != 0 {
 nonzero = true;
 }
 }
 if !nonzero {
 return Ok(received[..n - ec].to_vec());
 }
 let mut c = vec![1u8];
 let mut bpoly = vec![1u8];
 let mut l = 0usize;
 let mut mm = 1usize;
 let mut bb = 1u8;
 for nn in 0..ec {
 let mut delta = syn[nn];
 for i in 1..=l {
 if i < c.len() {
 delta ^= gf.mul(c[i], syn[nn - i]);
 }
 }
 if delta == 0 {
 mm += 1;
 } else if 2 * l <= nn {
 let t = c.clone();
 let coef = gf.div(delta, bb);
 if c.len() < bpoly.len() + mm {
 c.resize(bpoly.len() + mm, 0);
 }
 for i in 0..bpoly.len() {
 let m = gf.mul(coef, bpoly[i]);
 c[i + mm] ^= m;
 }
 l = nn + 1 - l;
 bpoly = t;
 bb = delta;
 mm = 1;
 } else {
 let coef = gf.div(delta, bb);
 if c.len() < bpoly.len() + mm {
 c.resize(bpoly.len() + mm, 0);
 }
 for i in 0..bpoly.len() {
 let m = gf.mul(coef, bpoly[i]);
 c[i + mm] ^= m;
 }
 mm += 1;
 }
 }
 let nerr = l;
 if nerr == 0 || nerr > ec / 2 {
 return Err(QrError::Decode);
 }
 let mut err_pos = Vec::new();
 for i in 0..n {
 let mut v = 0u8;
 for (j, &cj) in c.iter().enumerate() {
 v ^= gf.mul(cj, gf.pow(-(i as i32) * (j as i32)));
 }
 if v == 0 {
 err_pos.push(n - 1 - i);
 }
 }
 if err_pos.len() != nerr {
 return Err(QrError::Decode);
 }
 let t = err_pos.len();
 let mut amat = vec![vec![0u8; t]; t];
 let mut bvec = vec![0u8; t];
 for j in 0..t {
 for ll in 0..t {
 let e = (n - 1 - err_pos[ll]) as i32;
 amat[j][ll] = gf.pow(e * (j as i32));
 }
 bvec[j] = syn[j];
 }
 let ys = gf_solve(gf, amat, bvec).ok_or(QrError::Decode)?;
 let mut corrected = received.to_vec();
 for (ll, &pos) in err_pos.iter().enumerate() {
 corrected[pos] ^= ys[ll];
 }
 for i in 0..ec {
 let mut s = 0u8;
 let x = gf.pow(i as i32);
 for &b in &corrected {
 s = gf.mul(s, x) ^ b;
 }
 if s != 0 {
 return Err(QrError::Decode);
 }
 }
 Ok(corrected[..n - ec].to_vec())
}

// ---------- decode: parse byte segment + public entry ----------
fn read_bits(data: &[u8], start: usize, count: usize) -> u32 {
 let mut v = 0u32;
 for k in 0..count {
 let pos = start + k;
 let byte = pos / 8;
 let bit = 7 - (pos % 8);
 let b = (data[byte] >> bit) & 1;
 v = (v << 1) | (b as u32);
 }
 v
}

fn parse_bytes(data: &[u8]) -> Result<String, QrError> {
 let total = data.len() * 8;
 if total < 12 {
 return Err(QrError::Decode);
 }
 let mode = read_bits(data, 0, 4);
 if mode != 0b0100 {
 return Err(QrError::Decode);
 }
 let len = read_bits(data, 4, 8) as usize;
 let start = 12;
 if start + len * 8 > total {
 return Err(QrError::Decode);
 }
 let mut out = Vec::with_capacity(len);
 for i in 0..len {
 out.push(read_bits(data, start + i * 8, 8) as u8);
 }
 String::from_utf8(out).map_err(|_| QrError::Decode)
}

/// Decode the first QR symbol in an RGBA image to its text payload.
pub fn decode(img: &RgbaImage) -> Result<String, QrError> {
 let (size, grid) = sample_grid(img).ok_or(QrError::NotFound)?;
 let spec = spec_for_size(size).ok_or(QrError::NotFound)?;
 let gf = Gf::new();
 let mut m = Matrix::new(size);
 place_function(&mut m, spec);
 for r in 0..size {
 for c in 0..size {
 let f = m.is_func(r, c);
 m.set(r, c, grid[r][c], f);
 }
 }
 let mask = read_format_mask(&m).ok_or(QrError::Format)?;
 apply_mask(&mut m, mask);
 let codewords = read_data(&m);
 let need = spec.data_cw + spec.ec_cw;
 if codewords.len() < need {
 return Err(QrError::Decode);
 }
 let data = rs_decode(&gf, &codewords[..need], spec.ec_cw)?;
 parse_bytes(&data)
}

#[cfg(test)]
mod tests {
 use super::*;

 fn roundtrip(text: &str) {
 let (ver, grid) = encode(text).expect("encode");
 assert!((1..=3).contains(&ver));
 let img = render(&grid, 4, 4);
 let got = decode(&img).expect("decode");
 assert_eq!(got, text);
 }

 #[test]
 fn gf_inverse_holds() {
 let gf = Gf::new();
 assert_eq!(gf.mul(0, 5), 0);
 assert_eq!(gf.mul(1, 7), 7);
 for a in 1..=255u8 {
 assert_eq!(gf.mul(a, gf.inv(a)), 1);
 }
 }

 #[test]
 fn roundtrip_text_and_url() {
 roundtrip("HELLO");
 roundtrip("SloerShot");
 roundtrip("https://sloershot.app/s/abc123");
 }

 #[test]
 fn picks_version_by_length() {
 assert_eq!(encode("hi").unwrap().0, 1);
 let medium = "x".repeat(20);
 assert_eq!(encode(&medium).unwrap().0, 2);
 let longer = "y".repeat(35);
 assert_eq!(encode(&longer).unwrap().0, 3);
 roundtrip(&medium);
 roundtrip(&longer);
 }

 #[test]
 fn rejects_too_long() {
 let big = "z".repeat(60);
 assert_eq!(encode(&big), Err(QrError::TooLong));
 }

 #[test]
 fn corrects_injected_errors() {
 let (_, grid) = encode("RESILIENT").unwrap();
 let mut img = render(&grid, 4, 4);
 let n = grid.len();
 for (mr, mc) in [(n - 1, n - 1), (n - 1, n - 2)] {
 let x0 = (mc as u32 + 4) * 4;
 let y0 = (mr as u32 + 4) * 4;
 for y in y0..y0 + 4 {
 for x in x0..x0 + 4 {
 let p = *img.get_pixel(x, y);
 let inv = image::Rgba([255 - p[0], 255 - p[1], 255 - p[2], 255]);
 img.put_pixel(x, y, inv);
 }
 }
 }
 assert_eq!(decode(&img).unwrap(), "RESILIENT");
 }
}
