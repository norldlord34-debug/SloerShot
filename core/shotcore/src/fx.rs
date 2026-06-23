//! Image-editing effects and transforms (competitive parity with Snagit/ShareX/CleanShot).
//!
//! Pure-Rust operations on RGBA images: crop, rotate, flip, resize, color filters,
//! brightness/contrast, blur, vignette, borders, spotlight, watermark, an
//! eyedropper, and multi-format export. All run on-device.

use crate::geometry::Rect;
use crate::model::Color;
use image::{Rgba, RgbaImage};

fn clampu8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Crop to a rect (clamped to the image bounds).
pub fn crop(img: &RgbaImage, rect: &Rect) -> RgbaImage {
    let r = rect.normalized();
    let x = (r.x.max(0.0) as u32).min(img.width());
    let y = (r.y.max(0.0) as u32).min(img.height());
    let w = ((r.w as u32).min(img.width().saturating_sub(x))).max(1);
    let h = ((r.h as u32).min(img.height().saturating_sub(y))).max(1);
    image::imageops::crop_imm(img, x, y, w, h).to_image()
}

/// Rotate 90 degrees clockwise.
pub fn rotate90(img: &RgbaImage) -> RgbaImage {
    image::imageops::rotate90(img)
}

/// Rotate 180 degrees.
pub fn rotate180(img: &RgbaImage) -> RgbaImage {
    image::imageops::rotate180(img)
}

/// Rotate 270 degrees clockwise (90 counter-clockwise).
pub fn rotate270(img: &RgbaImage) -> RgbaImage {
    image::imageops::rotate270(img)
}

/// Mirror horizontally.
pub fn flip_horizontal(img: &RgbaImage) -> RgbaImage {
    image::imageops::flip_horizontal(img)
}

/// Mirror vertically.
pub fn flip_vertical(img: &RgbaImage) -> RgbaImage {
    image::imageops::flip_vertical(img)
}

/// Resize to an exact width and height (Lanczos3).
pub fn resize(img: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    image::imageops::resize(
        img,
        width.max(1),
        height.max(1),
        image::imageops::FilterType::Lanczos3,
    )
}

/// Scale by a factor (1.0 = unchanged).
pub fn scale(img: &RgbaImage, factor: f32) -> RgbaImage {
    let f = factor.max(0.01);
    let w = ((img.width() as f32) * f).round() as u32;
    let h = ((img.height() as f32) * f).round() as u32;
    resize(img, w, h)
}

/// Sample the color at a pixel (the eyedropper). None if out of bounds.
pub fn pick_color(img: &RgbaImage, x: u32, y: u32) -> Option<Color> {
    if x >= img.width() || y >= img.height() {
        return None;
    }
    let p = img.get_pixel(x, y).0;
    Some(Color::rgba(p[0], p[1], p[2], p[3]))
}

/// Grayscale via luminance, preserving alpha.
pub fn grayscale(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        let l = clampu8(0.299 * p.0[0] as f32 + 0.587 * p.0[1] as f32 + 0.114 * p.0[2] as f32);
        p.0[0] = l;
        p.0[1] = l;
        p.0[2] = l;
    }
    out
}

/// Warm sepia tone.
pub fn sepia(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        let (r, g, b) = (p.0[0] as f32, p.0[1] as f32, p.0[2] as f32);
        p.0[0] = clampu8(0.393 * r + 0.769 * g + 0.189 * b);
        p.0[1] = clampu8(0.349 * r + 0.686 * g + 0.168 * b);
        p.0[2] = clampu8(0.272 * r + 0.534 * g + 0.131 * b);
    }
    out
}

/// Invert colors (negative).
pub fn invert(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        p.0[0] = 255 - p.0[0];
        p.0[1] = 255 - p.0[1];
        p.0[2] = 255 - p.0[2];
    }
    out
}

/// Add (or subtract) brightness across all channels.
pub fn adjust_brightness(img: &RgbaImage, delta: i32) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        for c in 0..3 {
            p.0[c] = (p.0[c] as i32 + delta).clamp(0, 255) as u8;
        }
    }
    out
}

/// Scale contrast around mid-gray (1.0 = unchanged, >1 more, <1 less).
pub fn adjust_contrast(img: &RgbaImage, factor: f32) -> RgbaImage {
    let mut out = img.clone();
    let f = factor.max(0.0);
    for p in out.pixels_mut() {
        for c in 0..3 {
            p.0[c] = clampu8((p.0[c] as f32 - 128.0) * f + 128.0);
        }
    }
    out
}

/// Gaussian blur the whole image.
pub fn blur(img: &RgbaImage, sigma: f32) -> RgbaImage {
    image::imageops::blur(img, sigma.max(0.1))
}

/// Darken toward the edges (vignette). strength 0.0 to 1.0.
pub fn vignette(img: &RgbaImage, strength: f32) -> RgbaImage {
    let mut out = img.clone();
    let (w, h) = (img.width() as f32, img.height() as f32);
    let (cx, cy) = (w / 2.0, h / 2.0);
    let maxd = (cx * cx + cy * cy).sqrt().max(1.0);
    let s = strength.clamp(0.0, 1.0);
    for y in 0..out.height() {
        for x in 0..out.width() {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt() / maxd;
            let factor = (1.0 - s * d * d).clamp(0.0, 1.0);
            let p = out.get_pixel_mut(x, y);
            for c in 0..3 {
                p.0[c] = clampu8(p.0[c] as f32 * factor);
            }
        }
    }
    out
}

/// Wrap the image in a solid border, returning a larger image.
pub fn add_border(img: &RgbaImage, thickness: u32, color: Color) -> RgbaImage {
    let t = thickness;
    let mut out = RgbaImage::from_pixel(
        img.width() + 2 * t,
        img.height() + 2 * t,
        Rgba([color.r, color.g, color.b, color.a]),
    );
    image::imageops::replace(&mut out, img, t as i64, t as i64);
    out
}

/// Dim everything outside `rect` to focus attention (spotlight). dim 0.0 to 1.0.
pub fn spotlight(img: &RgbaImage, rect: &Rect, dim: f32) -> RgbaImage {
    let r = rect.normalized();
    let keep = 1.0 - dim.clamp(0.0, 1.0);
    let mut out = img.clone();
    for y in 0..out.height() {
        for x in 0..out.width() {
            let inside = (x as f64) >= r.x
                && (x as f64) < r.right()
                && (y as f64) >= r.y
                && (y as f64) < r.bottom();
            if !inside {
                let p = out.get_pixel_mut(x, y);
                for c in 0..3 {
                    p.0[c] = clampu8(p.0[c] as f32 * keep);
                }
            }
        }
    }
    out
}

/// Stamp a text watermark at (x, y) using the given font.
pub fn watermark_text(
    img: &RgbaImage,
    text: &str,
    x: i32,
    y: i32,
    size: f32,
    color: Color,
    font: &ab_glyph::FontVec,
) -> RgbaImage {
    let mut out = img.clone();
    let scale = ab_glyph::PxScale::from(size);
    imageproc::drawing::draw_text_mut(
        &mut out,
        Rgba([color.r, color.g, color.b, color.a]),
        x,
        y,
        scale,
        font,
        text,
    );
    out
}

/// Encode as JPEG at the given quality (1-100). Alpha is flattened to RGB.
pub fn to_jpeg_bytes(img: &RgbaImage, quality: u8) -> Result<Vec<u8>, image::ImageError> {
    let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
    let mut buf = Vec::new();
    let mut enc =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality.clamp(1, 100));
    enc.encode_image(&rgb)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: u32, h: u32) -> RgbaImage {
        let mut im = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                im.put_pixel(x, y, Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255]));
            }
        }
        im
    }

    #[test]
    fn crop_dims_and_origin() {
        let base = img(20, 20);
        let c = crop(&base, &Rect::new(5.0, 5.0, 8.0, 6.0));
        assert_eq!(c.dimensions(), (8, 6));
        assert_eq!(c.get_pixel(0, 0), base.get_pixel(5, 5));
    }

    #[test]
    fn rotate90_swaps_dims() {
        assert_eq!(rotate90(&img(10, 4)).dimensions(), (4, 10));
    }

    #[test]
    fn rotate180_twice_keeps_pixel() {
        let base = img(6, 8);
        let r = rotate180(&rotate180(&base));
        assert_eq!(r.dimensions(), (6, 8));
        assert_eq!(r.get_pixel(2, 3), base.get_pixel(2, 3));
    }

    #[test]
    fn flip_horizontal_mirrors() {
        let base = img(10, 3);
        assert_eq!(flip_horizontal(&base).get_pixel(0, 0), base.get_pixel(9, 0));
    }

    #[test]
    fn resize_and_scale_dims() {
        let base = img(20, 10);
        assert_eq!(resize(&base, 40, 5).dimensions(), (40, 5));
        assert_eq!(scale(&base, 0.5).dimensions(), (10, 5));
    }

    #[test]
    fn grayscale_makes_channels_equal() {
        let g = grayscale(&img(4, 4));
        let p = g.get_pixel(1, 2).0;
        assert_eq!(p[0], p[1]);
        assert_eq!(p[1], p[2]);
    }

    #[test]
    fn invert_flips() {
        let base = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 128, 255]));
        assert_eq!(invert(&base).get_pixel(0, 0).0, [0, 255, 127, 255]);
    }

    #[test]
    fn brightness_increases_and_clamps() {
        let base = RgbaImage::from_pixel(2, 2, Rgba([100, 100, 100, 255]));
        assert_eq!(adjust_brightness(&base, 50).get_pixel(0, 0).0[0], 150);
        assert_eq!(adjust_brightness(&base, -200).get_pixel(0, 0).0[0], 0);
    }

    #[test]
    fn contrast_clamps() {
        let base = RgbaImage::from_pixel(1, 1, Rgba([200, 200, 200, 255]));
        assert_eq!(adjust_contrast(&base, 2.0).get_pixel(0, 0).0[0], 255);
    }

    #[test]
    fn add_border_grows_and_colors() {
        let base = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255]));
        let b = add_border(&base, 3, Color::rgb(255, 0, 0));
        assert_eq!(b.dimensions(), (10, 10));
        assert_eq!(b.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(b.get_pixel(5, 5).0, [10, 20, 30, 255]);
    }

    #[test]
    fn spotlight_dims_outside_only() {
        let base = RgbaImage::from_pixel(10, 10, Rgba([200, 200, 200, 255]));
        let s = spotlight(&base, &Rect::new(2.0, 2.0, 4.0, 4.0), 0.5);
        assert_eq!(s.get_pixel(3, 3).0[0], 200);
        assert!(s.get_pixel(0, 0).0[0] < 200);
    }

    #[test]
    fn vignette_darkens_corner() {
        let base = RgbaImage::from_pixel(20, 20, Rgba([200, 200, 200, 255]));
        let v = vignette(&base, 0.8);
        assert_eq!(v.get_pixel(10, 10).0[0], 200);
        assert!(v.get_pixel(0, 0).0[0] < 200);
    }

    #[test]
    fn pick_color_reads_pixel() {
        let base = RgbaImage::from_pixel(3, 3, Rgba([1, 2, 3, 255]));
        assert_eq!(pick_color(&base, 1, 1), Some(Color::rgba(1, 2, 3, 255)));
        assert_eq!(pick_color(&base, 9, 9), None);
    }

    #[test]
    fn jpeg_export_has_soi_marker() {
        let bytes = to_jpeg_bytes(&img(16, 16), 85).unwrap();
        assert!(bytes.len() > 2 && bytes[0] == 0xFF && bytes[1] == 0xD8);
    }

    #[test]
    fn sepia_and_blur_run() {
        assert_eq!(sepia(&img(4, 4)).dimensions(), (4, 4));
        assert_eq!(blur(&img(8, 8), 2.0).dimensions(), (8, 8));
    }
}

/// Pixelate: replace each square block of `block` px with its average color.
pub fn pixelate(img: &RgbaImage, block: u32) -> RgbaImage {
 let b = block.max(1);
 let (w, h) = (img.width(), img.height());
 let mut out = img.clone();
 let mut by = 0;
 while by < h {
 let mut bx = 0;
 while bx < w {
 let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
 let ex = (bx + b).min(w);
 let ey = (by + b).min(h);
 for y in by..ey {
 for x in bx..ex {
 let p = img.get_pixel(x, y).0;
 sr += p[0] as u64; sg += p[1] as u64; sb += p[2] as u64; sa += p[3] as u64; n += 1;
 }
 }
 if n > 0 {
 let px = Rgba([(sr / n) as u8, (sg / n) as u8, (sb / n) as u8, (sa / n) as u8]);
 for y in by..ey {
 for x in bx..ex {
 out.put_pixel(x, y, px);
 }
 }
 }
 bx += b;
 }
 by += b;
 }
 out
}

/// Gamma correction via a lookup table. Alpha is preserved.
pub fn gamma(img: &RgbaImage, g: f32) -> RgbaImage {
 let inv = 1.0 / g.max(0.01);
 let mut lut = [0u8; 256];
 for i in 0..256 {
 lut[i] = clampu8(255.0 * (i as f32 / 255.0).powf(inv));
 }
 let mut out = img.clone();
 for p in out.pixels_mut() {
 p.0[0] = lut[p.0[0] as usize];
 p.0[1] = lut[p.0[1] as usize];
 p.0[2] = lut[p.0[2] as usize];
 }
 out
}

/// Posterize / reduce color depth to `levels` levels per channel (2..=256).
pub fn posterize(img: &RgbaImage, levels: u32) -> RgbaImage {
 let l = levels.clamp(2, 256) as f32;
 let step = 255.0 / (l - 1.0);
 let mut out = img.clone();
 for p in out.pixels_mut() {
 for c in 0..3 {
 p.0[c] = clampu8((p.0[c] as f32 / step).round() * step);
 }
 }
 out
}

/// Threshold to black and white using luminance.
pub fn black_white(img: &RgbaImage, threshold: u8) -> RgbaImage {
 let mut out = img.clone();
 for p in out.pixels_mut() {
 let lum = 0.299 * p.0[0] as f32 + 0.587 * p.0[1] as f32 + 0.114 * p.0[2] as f32;
 let val = if lum >= threshold as f32 { 255 } else { 0 };
 p.0[0] = val; p.0[1] = val; p.0[2] = val;
 }
 out
}

/// Solarize: invert channels at or above the threshold (Sabattier effect).
pub fn solarize(img: &RgbaImage, threshold: u8) -> RgbaImage {
 let mut out = img.clone();
 for p in out.pixels_mut() {
 for c in 0..3 {
 if p.0[c] >= threshold { p.0[c] = 255 - p.0[c]; }
 }
 }
 out
}

/// Tint a luminance map toward `color` (colorize).
pub fn colorize(img: &RgbaImage, color: Color) -> RgbaImage {
 let mut out = img.clone();
 for p in out.pixels_mut() {
 let lum = (0.299 * p.0[0] as f32 + 0.587 * p.0[1] as f32 + 0.114 * p.0[2] as f32) / 255.0;
 p.0[0] = clampu8(color.r as f32 * lum);
 p.0[1] = clampu8(color.g as f32 * lum);
 p.0[2] = clampu8(color.b as f32 * lum);
 }
 out
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
 let max = r.max(g).max(b);
 let min = r.min(g).min(b);
 let d = max - min;
 let mut h = 0.0;
 if d > 0.0 {
 if max == r { h = 60.0 * (((g - b) / d) % 6.0); }
 else if max == g { h = 60.0 * (((b - r) / d) + 2.0); }
 else { h = 60.0 * (((r - g) / d) + 4.0); }
 }
 if h < 0.0 { h += 360.0; }
 let s = if max <= 0.0 { 0.0 } else { d / max };
 (h, s, max)
}
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
 let c = v * s;
 let hh = (((h % 360.0) + 360.0) % 360.0) / 60.0;
 let x = c * (1.0 - (hh % 2.0 - 1.0).abs());
 let (r1, g1, b1) = match hh as i32 {
 0 => (c, x, 0.0),
 1 => (x, c, 0.0),
 2 => (0.0, c, x),
 3 => (0.0, x, c),
 4 => (x, 0.0, c),
 _ => (c, 0.0, x),
 };
 let m = v - c;
 (r1 + m, g1 + m, b1 + m)
}

/// Rotate hue by `degrees` in HSV space (alpha preserved).
pub fn hue_rotate(img: &RgbaImage, degrees: f32) -> RgbaImage {
 let mut out = img.clone();
 for p in out.pixels_mut() {
 let (h, s, v) = rgb_to_hsv(p.0[0] as f32 / 255.0, p.0[1] as f32 / 255.0, p.0[2] as f32 / 255.0);
 let (r, g, b) = hsv_to_rgb(h + degrees, s, v);
 p.0[0] = clampu8(r * 255.0); p.0[1] = clampu8(g * 255.0); p.0[2] = clampu8(b * 255.0);
 }
 out
}
/// Scale saturation by `factor` (0 = grayscale, 1 = unchanged, >1 = more vivid).
pub fn saturate(img: &RgbaImage, factor: f32) -> RgbaImage {
 let mut out = img.clone();
 for p in out.pixels_mut() {
 let (h, s, v) = rgb_to_hsv(p.0[0] as f32 / 255.0, p.0[1] as f32 / 255.0, p.0[2] as f32 / 255.0);
 let (r, g, b) = hsv_to_rgb(h, (s * factor).clamp(0.0, 1.0), v);
 p.0[0] = clampu8(r * 255.0); p.0[1] = clampu8(g * 255.0); p.0[2] = clampu8(b * 255.0);
 }
 out
}

fn convolve3x3(img: &RgbaImage, k: &[[f32; 3]; 3], bias: f32) -> RgbaImage {
 let (w, h) = (img.width(), img.height());
 let mut out = img.clone();
 for y in 0..h {
 for x in 0..w {
 let (mut ar, mut ag, mut ab) = (0.0f32, 0.0f32, 0.0f32);
 for ky in 0..3usize {
 for kx in 0..3usize {
 let sx = (x as i32 + kx as i32 - 1).clamp(0, w as i32 - 1) as u32;
 let sy = (y as i32 + ky as i32 - 1).clamp(0, h as i32 - 1) as u32;
 let p = img.get_pixel(sx, sy).0;
 let kv = k[ky][kx];
 ar += p[0] as f32 * kv; ag += p[1] as f32 * kv; ab += p[2] as f32 * kv;
 }
 }
 let a = img.get_pixel(x, y).0[3];
 out.put_pixel(x, y, Rgba([clampu8(ar + bias), clampu8(ag + bias), clampu8(ab + bias), a]));
 }
 }
 out
}

/// Emboss filter (3x3 convolution with mid-gray bias).
pub fn emboss(img: &RgbaImage) -> RgbaImage {
 let kernel: [[f32; 3]; 3] = [[-2.0, -1.0, 0.0], [-1.0, 1.0, 1.0], [0.0, 1.0, 2.0]];
 convolve3x3(img, &kernel, 128.0)
}

/// Edge detection (Sobel) rendered as a grayscale RGBA image.
pub fn edge_detect(img: &RgbaImage) -> RgbaImage {
 let gray = crate::edges::sobel(img);
 let mut out = RgbaImage::new(img.width(), img.height());
 for (x, y, p) in out.enumerate_pixels_mut() {
 let val = gray.get_pixel(x, y).0[0];
 *p = Rgba([val, val, val, 255]);
 }
 out
}

/// Sharpen via unsharp masking (radius 2).
pub fn sharpen(img: &RgbaImage, amount: f32) -> RgbaImage {
 crate::sharpen::unsharp(img, 2, amount)
}

#[cfg(test)]
mod effect_tests {
 use super::*;
 fn sample() -> RgbaImage {
 let mut i = RgbaImage::new(8, 8);
 for (x, y, p) in i.enumerate_pixels_mut() {
 *p = Rgba([(x * 30) as u8, (y * 30) as u8, 64, 255]);
 }
 i
 }
 #[test]
 fn pixelate_keeps_dims() { assert_eq!(pixelate(&sample(), 4).dimensions(), (8, 8)); }
 #[test]
 fn gamma_identity_at_one() {
 let i = sample();
 assert_eq!(gamma(&i, 1.0).get_pixel(3, 3).0, i.get_pixel(3, 3).0);
 }
 #[test]
 fn black_white_is_binary() {
 for p in black_white(&sample(), 128).pixels() { assert!(p.0[0] == 0 || p.0[0] == 255); }
 }
 #[test]
 fn posterize_two_levels_binary() {
 for p in posterize(&sample(), 2).pixels() { assert!(p.0[0] == 0 || p.0[0] == 255); }
 }
 #[test]
 fn hue_rotate_360_near_identity() {
 let i = sample();
 let o = hue_rotate(&i, 360.0);
 let a = i.get_pixel(5, 2).0; let b = o.get_pixel(5, 2).0;
 for c in 0..3 { assert!((a[c] as i32 - b[c] as i32).abs() <= 3); }
 }
 #[test]
 fn saturate_zero_is_gray() {
 let p = saturate(&sample(), 0.0).get_pixel(6, 1).0;
 assert!((p[0] as i32 - p[1] as i32).abs() <= 2 && (p[1] as i32 - p[2] as i32).abs() <= 2);
 }
 #[test]
 fn structural_effects_keep_dims() {
 assert_eq!(edge_detect(&sample()).dimensions(), (8, 8));
 assert_eq!(emboss(&sample()).dimensions(), (8, 8));
 assert_eq!(sharpen(&sample(), 1.0).dimensions(), (8, 8));
 assert_eq!(colorize(&sample(), Color::rgba(255, 0, 0, 255)).dimensions(), (8, 8));
 assert_eq!(solarize(&sample(), 128).dimensions(), (8, 8));
 }
}

/// Split an image into a rows x cols grid of tiles (row-major). Returns (row, col, tile).
/// The last row/column absorbs any remainder pixels.
pub fn split_grid(img: &RgbaImage, rows: u32, cols: u32) -> Vec<(u32, u32, RgbaImage)> {
 let rows = rows.max(1);
 let cols = cols.max(1);
 let (w, h) = (img.width(), img.height());
 let tw = (w / cols).max(1);
 let th = (h / rows).max(1);
 let mut out = Vec::new();
 for r in 0..rows {
 for c in 0..cols {
 let x = c * tw;
 let y = r * th;
 if x >= w || y >= h { continue; }
 let cw = if c == cols - 1 { w - x } else { tw };
 let ch = if r == rows - 1 { h - y } else { th };
 let tile = image::imageops::crop_imm(img, x, y, cw, ch).to_image();
 out.push((r, c, tile));
 }
 }
 out
}

#[cfg(test)]
mod split_tests {
 use super::*;
 #[test]
 fn splits_into_grid() {
 let img = RgbaImage::new(10, 10);
 let tiles = split_grid(&img, 2, 2);
 assert_eq!(tiles.len(), 4);
 for (_, _, t) in &tiles { assert_eq!(t.dimensions(), (5, 5)); }
 }
 #[test]
 fn remainder_goes_to_last() {
 let img = RgbaImage::new(11, 7);
 let tiles = split_grid(&img, 1, 2);
 assert_eq!(tiles.len(), 2);
 assert_eq!(tiles[0].2.dimensions(), (5, 7));
 assert_eq!(tiles[1].2.dimensions(), (6, 7));
 }
}
