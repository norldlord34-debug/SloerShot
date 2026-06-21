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
