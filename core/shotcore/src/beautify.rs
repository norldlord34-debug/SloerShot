//! Background beautification for share-ready shots.
//!
//! Wraps a flattened capture in a padded solid or gradient background with
//! rounded corners and a soft drop shadow. Seven gradient presets ship by
//! default; padding, corner radius, and shadow are all adjustable.

use crate::geometry::{Point, Rect};
use crate::model::Color;
use image::{GrayImage, Luma, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

/// Built-in gradient presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GradientPreset {
    Indigo,
    Sunset,
    Ocean,
    Forest,
    Candy,
    Midnight,
    Graphite,
}

impl GradientPreset {
    /// All presets in display order.
    pub fn all() -> [GradientPreset; 7] {
        [
            GradientPreset::Indigo,
            GradientPreset::Sunset,
            GradientPreset::Ocean,
            GradientPreset::Forest,
            GradientPreset::Candy,
            GradientPreset::Midnight,
            GradientPreset::Graphite,
        ]
    }

    /// Start and end colors of the gradient.
    pub fn colors(&self) -> (Color, Color) {
        match self {
            GradientPreset::Indigo => (Color::rgb(99, 102, 241), Color::rgb(168, 85, 247)),
            GradientPreset::Sunset => (Color::rgb(249, 115, 22), Color::rgb(236, 72, 153)),
            GradientPreset::Ocean => (Color::rgb(14, 165, 233), Color::rgb(34, 211, 238)),
            GradientPreset::Forest => (Color::rgb(34, 197, 94), Color::rgb(16, 185, 129)),
            GradientPreset::Candy => (Color::rgb(236, 72, 153), Color::rgb(251, 191, 36)),
            GradientPreset::Midnight => (Color::rgb(15, 23, 42), Color::rgb(51, 65, 85)),
            GradientPreset::Graphite => (Color::rgb(38, 38, 38), Color::rgb(82, 82, 82)),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            GradientPreset::Indigo => "Indigo",
            GradientPreset::Sunset => "Sunset",
            GradientPreset::Ocean => "Ocean",
            GradientPreset::Forest => "Forest",
            GradientPreset::Candy => "Candy",
            GradientPreset::Midnight => "Midnight",
            GradientPreset::Graphite => "Graphite",
        }
    }
}

/// The background behind a beautified capture.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Background {
    Solid(Color),
    Gradient {
        start: Color,
        end: Color,
        angle_deg: f32,
    },
    Preset(GradientPreset),
}

impl Background {
    fn resolve(&self) -> (Color, Color, f32) {
        match self {
            Background::Solid(c) => (*c, *c, 0.0),
            Background::Gradient {
                start,
                end,
                angle_deg,
            } => (*start, *end, *angle_deg),
            Background::Preset(p) => {
                let (a, b) = p.colors();
                (a, b, 135.0)
            }
        }
    }
}

/// A soft drop shadow under the capture.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shadow {
    pub color: Color,
    pub blur: f32,
    pub dx: f32,
    pub dy: f32,
    pub opacity: f32,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            blur: 24.0,
            dx: 0.0,
            dy: 16.0,
            opacity: 0.35,
        }
    }
}

/// Knobs for the beautify pass.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BeautifyOptions {
    pub background: Background,
    pub padding: u32,
    pub corner_radius: f32,
    pub shadow: Option<Shadow>,
}

impl Default for BeautifyOptions {
    fn default() -> Self {
        Self {
            background: Background::Preset(GradientPreset::Indigo),
            padding: 64,
            corner_radius: 16.0,
            shadow: Some(Shadow::default()),
        }
    }
}

fn blend_rgba(dst: Rgba<u8>, src: Rgba<u8>) -> Rgba<u8> {
    let sa = src.0[3] as f32 / 255.0;
    if sa <= 0.0 {
        return dst;
    }
    let inv = 1.0 - sa;
    Rgba([
        (src.0[0] as f32 * sa + dst.0[0] as f32 * inv).round() as u8,
        (src.0[1] as f32 * sa + dst.0[1] as f32 * inv).round() as u8,
        (src.0[2] as f32 * sa + dst.0[2] as f32 * inv).round() as u8,
        255,
    ])
}

fn lerp_rgba(a: Color, b: Color, t: f32) -> Rgba<u8> {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Rgba([l(a.r, b.r), l(a.g, b.g), l(a.b, b.b), 255])
}

fn in_round_rect(p: Point, rect: &Rect, radius: f64) -> bool {
    let r = rect.normalized();
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    if rad <= 0.0 {
        return r.contains_point_inclusive(p);
    }
    if p.x < r.x || p.x > r.right() || p.y < r.y || p.y > r.bottom() {
        return false;
    }
    let cx = p.x.clamp(r.x + rad, r.right() - rad);
    let cy = p.y.clamp(r.y + rad, r.bottom() - rad);
    let dx = p.x - cx;
    let dy = p.y - cy;
    dx * dx + dy * dy <= rad * rad
}

fn fill_background(out: &mut RgbaImage, bg: &Background) {
    let (start, end, angle) = bg.resolve();
    let (w, h) = (out.width() as f32, out.height() as f32);
    let rad = angle.to_radians();
    let (dx, dy) = (rad.cos(), rad.sin());
    let projs = [0.0, w * dx, h * dy, w * dx + h * dy];
    let min = projs.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = projs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max - min).max(1e-3);
    for y in 0..out.height() {
        for x in 0..out.width() {
            let proj = (x as f32) * dx + (y as f32) * dy;
            let t = ((proj - min) / range).clamp(0.0, 1.0);
            out.put_pixel(x, y, lerp_rgba(start, end, t));
        }
    }
}

fn draw_shadow(out: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, radius: f32, sh: &Shadow) {
    let (ow, oh) = (out.width(), out.height());
    let mut mask = GrayImage::new(ow, oh);
    let rect = Rect::new((x + sh.dx) as f64, (y + sh.dy) as f64, w as f64, h as f64);
    for py in 0..oh {
        for px in 0..ow {
            let p = Point::new(px as f64 + 0.5, py as f64 + 0.5);
            if in_round_rect(p, &rect, radius as f64) {
                mask.put_pixel(px, py, Luma([255]));
            }
        }
    }
    let blurred = image::imageops::blur(&mask, sh.blur.max(0.5));
    for py in 0..oh {
        for px in 0..ow {
            let cov = blurred.get_pixel(px, py).0[0] as f32 / 255.0 * sh.opacity.clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }
            let dst = *out.get_pixel(px, py);
            let src = Rgba([
                sh.color.r,
                sh.color.g,
                sh.color.b,
                (cov * 255.0).round() as u8,
            ]);
            out.put_pixel(px, py, blend_rgba(dst, src));
        }
    }
}

fn composite_rounded(out: &mut RgbaImage, top: &RgbaImage, ox: i64, oy: i64, radius: f32) {
    let rect = Rect::new(0.0, 0.0, top.width() as f64, top.height() as f64);
    for ty in 0..top.height() {
        for tx in 0..top.width() {
            let p = Point::new(tx as f64 + 0.5, ty as f64 + 0.5);
            if !in_round_rect(p, &rect, radius as f64) {
                continue;
            }
            let dx = ox + tx as i64;
            let dy = oy + ty as i64;
            if dx < 0 || dy < 0 || dx >= out.width() as i64 || dy >= out.height() as i64 {
                continue;
            }
            let src = *top.get_pixel(tx, ty);
            let dst = *out.get_pixel(dx as u32, dy as u32);
            out.put_pixel(dx as u32, dy as u32, blend_rgba(dst, src));
        }
    }
}

/// Wrap `image` in the configured background, returning a new, larger image.
pub fn beautify(image: &RgbaImage, opts: &BeautifyOptions) -> RgbaImage {
    let pad = opts.padding;
    let (iw, ih) = (image.width(), image.height());
    let mut out = RgbaImage::new(iw + pad * 2, ih + pad * 2);
    fill_background(&mut out, &opts.background);
    if let Some(sh) = opts.shadow {
        draw_shadow(
            &mut out,
            pad as f32,
            pad as f32,
            iw as f32,
            ih as f32,
            opts.corner_radius,
            &sh,
        );
    }
    composite_rounded(&mut out, image, pad as i64, pad as i64, opts.corner_radius);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_img(w: u32, h: u32, c: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba(c))
    }

    #[test]
    fn dimensions_include_padding() {
        let img = solid_img(100, 50, [255, 0, 0, 255]);
        let opts = BeautifyOptions {
            padding: 20,
            ..Default::default()
        };
        let out = beautify(&img, &opts);
        assert_eq!(out.width(), 140);
        assert_eq!(out.height(), 90);
    }

    #[test]
    fn solid_background_fills_border() {
        let img = solid_img(4, 4, [255, 0, 0, 255]);
        let opts = BeautifyOptions {
            background: Background::Solid(Color::rgb(0, 128, 0)),
            padding: 5,
            corner_radius: 0.0,
            shadow: None,
        };
        let out = beautify(&img, &opts);
        assert_eq!(out.get_pixel(0, 0), &Rgba([0, 128, 0, 255]));
    }

    #[test]
    fn gradient_origin_is_start_color() {
        let img = solid_img(4, 4, [255, 255, 255, 255]);
        let opts = BeautifyOptions {
            background: Background::Gradient {
                start: Color::rgb(255, 0, 0),
                end: Color::rgb(0, 0, 255),
                angle_deg: 0.0,
            },
            padding: 10,
            corner_radius: 0.0,
            shadow: None,
        };
        let out = beautify(&img, &opts);
        assert_eq!(out.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
        let right = out.get_pixel(out.width() - 1, 0);
        assert!(right.0[2] > right.0[0]);
    }

    #[test]
    fn rounded_corner_shows_background_not_capture() {
        let img = solid_img(20, 20, [255, 0, 0, 255]);
        let opts = BeautifyOptions {
            background: Background::Solid(Color::rgb(0, 0, 255)),
            padding: 0,
            corner_radius: 8.0,
            shadow: None,
        };
        let out = beautify(&img, &opts);
        assert_eq!(out.get_pixel(0, 0), &Rgba([0, 0, 255, 255]));
        assert_eq!(out.get_pixel(10, 10), &Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn presets_are_seven_and_distinct() {
        let all = GradientPreset::all();
        assert_eq!(all.len(), 7);
        let mut names: Vec<&str> = all.iter().map(|p| p.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 7);
    }
}

/// Content alignment within an expanded/padded canvas (Background tool).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
 TopLeft,
 TopCenter,
 TopRight,
 CenterLeft,
 Center,
 CenterRight,
 BottomLeft,
 BottomCenter,
 BottomRight,
}

impl Alignment {
 /// Horizontal/vertical placement fractions in 0.0..=1.0 (0.5 = centered).
 pub fn fractions(&self) -> (f32, f32) {
 let h = match self {
 Alignment::TopLeft | Alignment::CenterLeft | Alignment::BottomLeft => 0.0,
 Alignment::TopCenter | Alignment::Center | Alignment::BottomCenter => 0.5,
 Alignment::TopRight | Alignment::CenterRight | Alignment::BottomRight => 1.0,
 };
 let v = match self {
 Alignment::TopLeft | Alignment::TopCenter | Alignment::TopRight => 0.0,
 Alignment::CenterLeft | Alignment::Center | Alignment::CenterRight => 0.5,
 Alignment::BottomLeft | Alignment::BottomCenter | Alignment::BottomRight => 1.0,
 };
 (h, v)
 }
}

/// Ten extra named gradient backgrounds beyond the seven core presets.
pub fn extra_gradients() -> [(&'static str, Color, Color); 10] {
 [
 ("Sky", Color::rgb(56, 189, 248), Color::rgb(59, 130, 246)),
 ("Mint", Color::rgb(52, 211, 153), Color::rgb(16, 185, 129)),
 ("Peach", Color::rgb(251, 191, 36), Color::rgb(249, 115, 22)),
 ("Rose", Color::rgb(251, 113, 133), Color::rgb(225, 29, 72)),
 ("Lavender", Color::rgb(167, 139, 250), Color::rgb(124, 58, 237)),
 ("Slate", Color::rgb(100, 116, 139), Color::rgb(51, 65, 85)),
 ("Sand", Color::rgb(245, 222, 179), Color::rgb(214, 178, 122)),
 ("Teal", Color::rgb(45, 212, 191), Color::rgb(13, 148, 136)),
 ("Coral", Color::rgb(255, 138, 101), Color::rgb(244, 81, 30)),
 ("Noir", Color::rgb(38, 38, 38), Color::rgb(10, 10, 10)),
 ]
}

/// Auto Balance: padding insets (left, top, right, bottom) that visually center
/// content of size (cw, ch) inside a canvas of (w, h). Negative space is split
/// evenly; never returns negative insets.
pub fn auto_balance(cw: f64, ch: f64, w: f64, h: f64) -> (f64, f64, f64, f64) {
 let hx = ((w - cw) / 2.0).max(0.0);
 let hy = ((h - ch) / 2.0).max(0.0);
 (hx, hy, hx, hy)
}

#[cfg(test)]
mod ext_tests {
 use super::*;

 #[test]
 fn alignment_fractions() {
 assert_eq!(Alignment::TopLeft.fractions(), (0.0, 0.0));
 assert_eq!(Alignment::Center.fractions(), (0.5, 0.5));
 assert_eq!(Alignment::BottomRight.fractions(), (1.0, 1.0));
 }

 #[test]
 fn ten_extra_gradients() {
 assert_eq!(extra_gradients().len(), 10);
 assert_eq!(extra_gradients()[0].0, "Sky");
 }

 #[test]
 fn auto_balance_centers() {
 assert_eq!(auto_balance(100.0, 100.0, 200.0, 300.0), (50.0, 100.0, 50.0, 100.0));
 assert_eq!(auto_balance(400.0, 100.0, 200.0, 100.0), (0.0, 0.0, 0.0, 0.0));
 }
}

/// How a custom background image fills the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundFit {
 Cover,
 Contain,
 Center,
 Tile,
}

/// Canvas size for content (cw,ch) with uniform padding, optionally forced to a
/// width/height aspect ratio. The canvas only grows; content is never cropped.
pub fn canvas_size_for_aspect(cw: u32, ch: u32, padding: u32, ratio: Option<f64>) -> (u32, u32) {
 let base_w = cw + padding * 2;
 let base_h = ch + padding * 2;
 match ratio {
 Some(r) if r > 0.0 => {
 let bw = base_w as f64;
 let bh = base_h as f64;
 if bw / bh > r {
 let h = (bw / r).ceil() as u32;
 (base_w, h.max(base_h))
 } else {
 let w = (bh * r).ceil() as u32;
 (w.max(base_w), base_h)
 }
 }
 _ => (base_w, base_h),
 }
}

/// Render a custom background image into a (w,h) canvas using the given fit mode.
pub fn render_image_background(bg: &RgbaImage, w: u32, h: u32, fit: BackgroundFit) -> RgbaImage {
 let (w, h) = (w.max(1), h.max(1));
 let (bw, bh) = (bg.width().max(1), bg.height().max(1));
 let mut canvas = RgbaImage::new(w, h);
 match fit {
 BackgroundFit::Cover | BackgroundFit::Contain => {
 let sx = w as f64 / bw as f64;
 let sy = h as f64 / bh as f64;
 let scale = if matches!(fit, BackgroundFit::Cover) { sx.max(sy) } else { sx.min(sy) };
 let sw = ((bw as f64) * scale).round().max(1.0) as u32;
 let sh = ((bh as f64) * scale).round().max(1.0) as u32;
 let resized = image::imageops::resize(bg, sw, sh, image::imageops::FilterType::Triangle);
 let ox = ((w as i64) - (sw as i64)) / 2;
 let oy = ((h as i64) - (sh as i64)) / 2;
 image::imageops::overlay(&mut canvas, &resized, ox, oy);
 }
 BackgroundFit::Center => {
 let ox = ((w as i64) - (bw as i64)) / 2;
 let oy = ((h as i64) - (bh as i64)) / 2;
 image::imageops::overlay(&mut canvas, bg, ox, oy);
 }
 BackgroundFit::Tile => {
 let mut y = 0i64;
 while y < h as i64 {
 let mut x = 0i64;
 while x < w as i64 {
 image::imageops::overlay(&mut canvas, bg, x, y);
 x += bw as i64;
 }
 y += bh as i64;
 }
 }
 }
 canvas
}

#[cfg(test)]
mod bg_ext_tests {
 use super::*;

 #[test]
 fn aspect_canvas_no_ratio() {
 assert_eq!(canvas_size_for_aspect(100, 50, 10, None), (120, 70));
 }

 #[test]
 fn aspect_canvas_square_grows_smaller_side() {
 assert_eq!(canvas_size_for_aspect(100, 50, 10, Some(1.0)), (120, 120));
 }

 #[test]
 fn aspect_canvas_wide_grows_width() {
 let (w, h) = canvas_size_for_aspect(100, 50, 10, Some(16.0 / 9.0));
 assert!(w >= 120);
 assert_eq!(h, 70);
 }

 #[test]
 fn cover_fills_canvas_exactly() {
 let bg = RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255]));
 let out = render_image_background(&bg, 8, 8, BackgroundFit::Cover);
 assert_eq!((out.width(), out.height()), (8, 8));
 }

 #[test]
 fn tile_repeats_pattern() {
 let bg = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]));
 let out = render_image_background(&bg, 5, 5, BackgroundFit::Tile);
 assert_eq!(out.get_pixel(0, 0), &Rgba([1, 2, 3, 255]));
 assert_eq!(out.get_pixel(4, 4), &Rgba([1, 2, 3, 255]));
 }
}
