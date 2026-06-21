//! Export and compose pipeline.
//!
//! Rasterizes the vector annotation document onto a copy of the source image.
//! Drawing happens in z-order, and a Redact annotation destroys the pixels of
//! whatever sits beneath it (blur or pixelate), so the flattened export can
//! never recover the obscured content. The source image and sidecar are never
//! modified by this module.

use crate::geometry::{Point, Rect};
use crate::hit::{point_in_ellipse, point_segment_distance};
use crate::model::{Annotation, Color, Document, RedactStyle, ShapeKind};
use image::{Rgba, RgbaImage};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),
}

fn to_rgba(c: Color) -> Rgba<u8> {
    Rgba([c.r, c.g, c.b, c.a])
}

/// Source-over blend of `src` (times an opacity multiplier) onto an opaque `dst`.
fn blend_over(dst: Rgba<u8>, src: Color, opacity: f32) -> Rgba<u8> {
    let sa = (src.a as f32 / 255.0) * opacity.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return dst;
    }
    let inv = 1.0 - sa;
    let r = (src.r as f32 * sa + dst.0[0] as f32 * inv)
        .round()
        .clamp(0.0, 255.0) as u8;
    let g = (src.g as f32 * sa + dst.0[1] as f32 * inv)
        .round()
        .clamp(0.0, 255.0) as u8;
    let b = (src.b as f32 * sa + dst.0[2] as f32 * inv)
        .round()
        .clamp(0.0, 255.0) as u8;
    Rgba([r, g, b, 255])
}

fn put_blend(img: &mut RgbaImage, x: i64, y: i64, color: Color, opacity: f32) {
    if x < 0 || y < 0 || x >= img.width() as i64 || y >= img.height() as i64 {
        return;
    }
    let (xu, yu) = (x as u32, y as u32);
    let dst = *img.get_pixel(xu, yu);
    img.put_pixel(xu, yu, blend_over(dst, color, opacity));
}

/// Integer pixel span [x0,x1) x [y0,y1) of a rect clamped to the image.
fn clamp_span(rect: &Rect, w: u32, h: u32) -> (i64, i64, i64, i64) {
    let r = rect.normalized();
    let x0 = r.x.floor().max(0.0) as i64;
    let y0 = r.y.floor().max(0.0) as i64;
    let x1 = (r.right().ceil() as i64).min(w as i64);
    let y1 = (r.bottom().ceil() as i64).min(h as i64);
    (x0, y0, x1, y1)
}

fn fill_rect(img: &mut RgbaImage, rect: &Rect, color: Color, opacity: f32) {
    let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            put_blend(img, x, y, color, opacity);
        }
    }
}

fn stroke_rect(img: &mut RgbaImage, rect: &Rect, color: Color, opacity: f32, width: f64) {
    let hw = (width / 2.0).max(0.5);
    let outer = rect.normalized().inflate(hw);
    let inner = rect.normalized().inflate(-hw);
    let (x0, y0, x1, y1) = clamp_span(&outer, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
            if outer.contains_point_inclusive(p) && !inner.contains_point(p) {
                put_blend(img, x, y, color, opacity);
            }
        }
    }
}

fn fill_ellipse(img: &mut RgbaImage, rect: &Rect, color: Color, opacity: f32) {
    let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
            if point_in_ellipse(p, rect) {
                put_blend(img, x, y, color, opacity);
            }
        }
    }
}

fn stroke_ellipse(img: &mut RgbaImage, rect: &Rect, color: Color, opacity: f32, width: f64) {
    let hw = (width / 2.0).max(0.5);
    let outer = rect.normalized().inflate(hw);
    let inner = rect.normalized().inflate(-hw);
    let (x0, y0, x1, y1) = clamp_span(&outer, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
            if point_in_ellipse(p, &outer) && !point_in_ellipse(p, &inner) {
                put_blend(img, x, y, color, opacity);
            }
        }
    }
}

fn point_in_round_rect(p: Point, rect: &Rect, radius: f64) -> bool {
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

fn fill_round_rect(img: &mut RgbaImage, rect: &Rect, radius: f64, color: Color, opacity: f32) {
    let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
            if point_in_round_rect(p, rect, radius) {
                put_blend(img, x, y, color, opacity);
            }
        }
    }
}

fn stroke_round_rect(
    img: &mut RgbaImage,
    rect: &Rect,
    radius: f64,
    color: Color,
    opacity: f32,
    width: f64,
) {
    let hw = (width / 2.0).max(0.5);
    let outer = rect.normalized().inflate(hw);
    let inner = rect.normalized().inflate(-hw);
    let (x0, y0, x1, y1) = clamp_span(&outer, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
            if point_in_round_rect(p, &outer, radius + hw)
                && !point_in_round_rect(p, &inner, (radius - hw).max(0.0))
            {
                put_blend(img, x, y, color, opacity);
            }
        }
    }
}

fn draw_segment(img: &mut RgbaImage, a: Point, b: Point, color: Color, opacity: f32, width: f64) {
    let hw = (width / 2.0).max(0.5);
    let bbox = Rect::from_corners(a, b).inflate(hw + 1.0);
    let (x0, y0, x1, y1) = clamp_span(&bbox, img.width(), img.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
            if point_segment_distance(p, a, b) <= hw {
                put_blend(img, x, y, color, opacity);
            }
        }
    }
}

fn draw_arrow(img: &mut RgbaImage, from: Point, to: Point, color: Color, opacity: f32, width: f64) {
    draw_segment(img, from, to, color, opacity, width);
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-6 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let head = (width * 3.5).max(10.0);
    let angle = 0.5_f64;
    let (ca, sa) = (angle.cos(), angle.sin());
    let left = Point::new(
        to.x - head * (ux * ca - uy * sa),
        to.y - head * (uy * ca + ux * sa),
    );
    let right = Point::new(
        to.x - head * (ux * ca + uy * sa),
        to.y - head * (uy * ca - ux * sa),
    );
    draw_segment(img, to, left, color, opacity, width);
    draw_segment(img, to, right, color, opacity, width);
}

fn draw_polyline(img: &mut RgbaImage, points: &[Point], color: Color, opacity: f32, width: f64) {
    for seg in points.windows(2) {
        draw_segment(img, seg[0], seg[1], color, opacity, width);
    }
}

/// Average each block within `rect` and flatten it, irreversibly destroying detail.
fn pixelate_region(img: &mut RgbaImage, rect: &Rect, block: u32) {
    let block = block.max(2) as i64;
    let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
    let mut by = y0;
    while by < y1 {
        let mut bx = x0;
        while bx < x1 {
            let xe = (bx + block).min(x1);
            let ye = (by + block).min(y1);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for y in by..ye {
                for x in bx..xe {
                    let px = img.get_pixel(x as u32, y as u32);
                    sr += px.0[0] as u64;
                    sg += px.0[1] as u64;
                    sb += px.0[2] as u64;
                    sa += px.0[3] as u64;
                    n += 1;
                }
            }
            if n > 0 {
                let avg = Rgba([
                    (sr / n) as u8,
                    (sg / n) as u8,
                    (sb / n) as u8,
                    (sa / n) as u8,
                ]);
                for y in by..ye {
                    for x in bx..xe {
                        img.put_pixel(x as u32, y as u32, avg);
                    }
                }
            }
            bx += block;
        }
        by += block;
    }
}

/// Gaussian-blur the pixels within `rect`, irreversibly destroying detail.
fn blur_region(img: &mut RgbaImage, rect: &Rect, sigma: f32) {
    let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
    let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
    if w == 0 || h == 0 {
        return;
    }
    let sub = image::imageops::crop_imm(img, x0 as u32, y0 as u32, w, h).to_image();
    let blurred = image::imageops::blur(&sub, sigma.max(0.5));
    image::imageops::replace(img, &blurred, x0, y0);
}

/// Fill the region with opaque black, destroying the pixels (Black Out).
fn blackout_region(img: &mut RgbaImage, rect: &Rect) {
 let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
 for y in y0..y1 {
 for x in x0..x1 {
 img.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
 }
 }
}

/// Secure blur: pixelate then Gaussian blur so detail cannot be recovered.
fn secure_blur_region(img: &mut RgbaImage, rect: &Rect, strength: u32) {
 let block = strength.max(6).min(64);
 pixelate_region(img, rect, block);
 blur_region(img, rect, (block as f32) * 0.5);
}

/// Pixelate with deterministic per-block noise to defeat de-pixelization.
fn pixelate_randomized_region(img: &mut RgbaImage, rect: &Rect, block: u32) {
 let block = block.max(2) as i64;
 let (x0, y0, x1, y1) = clamp_span(rect, img.width(), img.height());
 let mut by = y0;
 while by < y1 {
 let mut bx = x0;
 while bx < x1 {
 let xe = (bx + block).min(x1);
 let ye = (by + block).min(y1);
 let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
 for y in by..ye {
 for x in bx..xe {
 let px = img.get_pixel(x as u32, y as u32);
 sr += px.0[0] as u64;
 sg += px.0[1] as u64;
 sb += px.0[2] as u64;
 sa += px.0[3] as u64;
 n += 1;
 }
 }
 if n > 0 {
 let seed = (bx as u64).wrapping_mul(73856093) ^ (by as u64).wrapping_mul(19349663);
 let jit = |c: u64, k: u64| -> u8 {
 let h = (seed ^ k.wrapping_mul(0x9E3779B97F4A7C15)).wrapping_mul(0x2545F4914F6CDD1D);
 let delta = ((h >> 56) as i64 % 25) - 12;
 (((c / n) as i64) + delta).clamp(0, 255) as u8
 };
 let noisy = Rgba([jit(sr, 1), jit(sg, 2), jit(sb, 3), (sa / n) as u8]);
 for y in by..ye {
 for x in bx..xe {
 img.put_pixel(x as u32, y as u32, noisy);
 }
 }
 }
 bx += block;
 }
 by += block;
 }
}

fn draw_text(
    img: &mut RgbaImage,
    text: &str,
    pos: Point,
    size: f32,
    color: Color,
    font: &ab_glyph::FontVec,
) {
    let scale = ab_glyph::PxScale::from(size);
    imageproc::drawing::draw_text_mut(
        img,
        to_rgba(color),
        pos.x.round() as i32,
        pos.y.round() as i32,
        scale,
        font,
        text,
    );
}

fn draw_centered_text(
    img: &mut RgbaImage,
    text: &str,
    center: Point,
    size: f32,
    color: Color,
    font: &ab_glyph::FontVec,
) {
    let tw = size * 0.55 * (text.chars().count().max(1) as f32);
    let x = center.x as f32 - tw / 2.0;
    let y = center.y as f32 - size * 0.62;
    draw_text(img, text, Point::new(x as f64, y as f64), size, color, font);
}

/// Draw one annotation onto the canvas in place.
fn draw_annotation(img: &mut RgbaImage, ann: &Annotation, font: Option<&ab_glyph::FontVec>) {
    let s = &ann.style;
    let op = s.opacity;
    let w = s.stroke_width as f64;
    match &ann.kind {
        ShapeKind::Line { from, to } => draw_segment(img, *from, *to, s.stroke, op, w),
        ShapeKind::Arrow { from, to } => draw_arrow(img, *from, *to, s.stroke, op, w),
        ShapeKind::Freehand { points } => draw_polyline(img, points, s.stroke, op, w),
        ShapeKind::Rectangle {
            rect,
            corner_radius,
        } => {
            let cr = *corner_radius as f64;
            if let Some(fill) = s.fill {
                if cr > 0.0 {
                    fill_round_rect(img, rect, cr, fill, op);
                } else {
                    fill_rect(img, rect, fill, op);
                }
            }
            if w > 0.0 {
                if cr > 0.0 {
                    stroke_round_rect(img, rect, cr, s.stroke, op, w);
                } else {
                    stroke_rect(img, rect, s.stroke, op, w);
                }
            }
        }
        ShapeKind::Ellipse { rect } => {
            if let Some(fill) = s.fill {
                fill_ellipse(img, rect, fill, op);
            }
            if w > 0.0 {
                stroke_ellipse(img, rect, s.stroke, op, w);
            }
        }
        ShapeKind::Highlighter { rect } => fill_rect(img, rect, s.stroke, op * 0.4),
        ShapeKind::Redact {
            rect,
            style,
            strength,
        } => match style {
            RedactStyle::Pixelate => pixelate_region(img, rect, *strength),
            RedactStyle::Blur => blur_region(img, rect, *strength as f32),
 RedactStyle::BlackOut => blackout_region(img, rect),
 RedactStyle::BlurSecure => secure_blur_region(img, rect, *strength),
 RedactStyle::PixelateRandomized => pixelate_randomized_region(img, rect, *strength),
        },
        ShapeKind::Counter {
            center,
            radius,
            number,
        } => {
            let r = *radius as f64;
            let bbox = Rect::new(center.x - r, center.y - r, 2.0 * r, 2.0 * r);
            let fill = s.fill.unwrap_or(s.stroke);
            fill_ellipse(img, &bbox, fill, op);
            if let Some(f) = font {
                draw_centered_text(
                    img,
                    &number.to_string(),
                    *center,
                    (r * 1.1) as f32,
                    Color::WHITE,
                    f,
                );
            }
        }
        ShapeKind::Text {
            position,
            content,
            font_size,
        } => {
            if let Some(f) = font {
                draw_text(img, content, *position, *font_size, s.stroke, f);
            }
        }
    }
}

/// Compose the document annotations over a copy of `base`, returning a new image.
pub fn compose(base: &RgbaImage, doc: &Document, font: Option<&ab_glyph::FontVec>) -> RgbaImage {
    let mut canvas = base.clone();
    for ann in doc.render_order() {
        draw_annotation(&mut canvas, ann, font);
    }
    canvas
}

/// Encode an image to PNG bytes.
pub fn to_png_bytes(img: &RgbaImage) -> Result<Vec<u8>, ExportError> {
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    Ok(buf)
}

/// Compose and save the flattened result; the format is inferred from the path.
pub fn export_to_path(
    base: &RgbaImage,
    doc: &Document,
    font: Option<&ab_glyph::FontVec>,
    path: impl AsRef<std::path::Path>,
) -> Result<(), ExportError> {
    let out = compose(base, doc, font);
    out.save(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Annotation, ShapeStyle};

    fn white(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
    }

    fn doc_with(w: u32, h: u32, kind: ShapeKind, style: ShapeStyle) -> Document {
        let mut d = Document::new(w, h);
        d.add(Annotation::new(kind).with_style(style));
        d
    }

    #[test]
    fn filled_rectangle_paints_interior() {
        let base = white(40, 40);
        let style = ShapeStyle {
            fill: Some(Color::rgb(10, 20, 30)),
            stroke: Color::BLACK,
            stroke_width: 0.0,
            opacity: 1.0,
 ..Default::default()
        };
        let doc = doc_with(
            40,
            40,
            ShapeKind::Rectangle {
                rect: Rect::new(10.0, 10.0, 20.0, 20.0),
                corner_radius: 0.0,
            },
            style,
        );
        let out = compose(&base, &doc, None);
        assert_eq!(out.get_pixel(20, 20), &Rgba([10, 20, 30, 255]));
        assert_eq!(out.get_pixel(2, 2), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn z_order_top_shape_wins() {
        let base = white(20, 20);
        let mut doc = Document::new(20, 20);
        doc.add(
            Annotation::new(ShapeKind::Rectangle {
                rect: Rect::new(0.0, 0.0, 20.0, 20.0),
                corner_radius: 0.0,
            })
            .with_style(ShapeStyle {
                fill: Some(Color::rgb(255, 0, 0)),
                stroke_width: 0.0,
                ..Default::default()
            }),
        );
        doc.add(
            Annotation::new(ShapeKind::Rectangle {
                rect: Rect::new(0.0, 0.0, 20.0, 20.0),
                corner_radius: 0.0,
            })
            .with_style(ShapeStyle {
                fill: Some(Color::rgb(0, 0, 255)),
                stroke_width: 0.0,
                ..Default::default()
            }),
        );
        let out = compose(&base, &doc, None);
        assert_eq!(out.get_pixel(10, 10), &Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn black_out_destroys_to_black() {
 let base = white(6, 6);
 let doc = doc_with(
 6,
 6,
 ShapeKind::Redact {
 rect: Rect::new(0.0, 0.0, 6.0, 6.0),
 style: RedactStyle::BlackOut,
 strength: 4,
 },
 ShapeStyle::default(),
 );
 let out = compose(&base, &doc, None);
 for y in 0..6u32 {
 for x in 0..6u32 {
 assert_eq!(out.get_pixel(x, y), &Rgba([0, 0, 0, 255]));
 }
 }
 }

 #[test]
 fn randomized_pixelate_destroys_and_adds_noise() {
 let mut base = white(8, 8);
 for y in 0..8u32 {
 for x in 0..8u32 {
 base.put_pixel(x, y, Rgba([(x * 30) as u8, (y * 30) as u8, 0, 255]));
 }
 }
 let mk = |style: RedactStyle| {
 let doc = doc_with(
 8,
 8,
 ShapeKind::Redact {
 rect: Rect::new(0.0, 0.0, 8.0, 8.0),
 style,
 strength: 3,
 },
 ShapeStyle::default(),
 );
 compose(&base, &doc, None)
 };
 let plain = mk(RedactStyle::Pixelate);
 let rand = mk(RedactStyle::PixelateRandomized);
 let mut differs = false;
 for y in 0..8u32 {
 for x in 0..8u32 {
 if plain.get_pixel(x, y) != rand.get_pixel(x, y) {
 differs = true;
 }
 }
 }
 assert!(differs, "randomized pixelation should add noise");
 assert_ne!(rand.get_pixel(0, 0), base.get_pixel(0, 0));
 }

 #[test]
 fn pixelate_makes_block_uniform() {
        let mut base = white(4, 4);
        for y in 0..4u32 {
            for x in 0..4u32 {
                base.put_pixel(x, y, Rgba([(x * 60) as u8, (y * 60) as u8, 0, 255]));
            }
        }
        let doc = doc_with(
            4,
            4,
            ShapeKind::Redact {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                style: RedactStyle::Pixelate,
                strength: 4,
            },
            ShapeStyle::default(),
        );
        let out = compose(&base, &doc, None);
        let first = *out.get_pixel(0, 0);
        for y in 0..4u32 {
            for x in 0..4u32 {
                assert_eq!(out.get_pixel(x, y), &first);
            }
        }
    }

    #[test]
    fn blur_changes_a_sharp_edge() {
        let mut base = white(20, 20);
        for y in 0..20u32 {
            for x in 0..10u32 {
                base.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let before = *base.get_pixel(9, 10);
        let doc = doc_with(
            20,
            20,
            ShapeKind::Redact {
                rect: Rect::new(0.0, 0.0, 20.0, 20.0),
                style: RedactStyle::Blur,
                strength: 4,
            },
            ShapeStyle::default(),
        );
        let out = compose(&base, &doc, None);
        assert_ne!(out.get_pixel(9, 10), &before);
    }

    #[test]
    fn ellipse_fill_inside_and_outside() {
        let base = white(40, 40);
        let style = ShapeStyle {
            fill: Some(Color::rgb(0, 200, 0)),
            stroke_width: 0.0,
            ..Default::default()
        };
        let doc = doc_with(
            40,
            40,
            ShapeKind::Ellipse {
                rect: Rect::new(0.0, 0.0, 40.0, 40.0),
            },
            style,
        );
        let out = compose(&base, &doc, None);
        assert_eq!(out.get_pixel(20, 20), &Rgba([0, 200, 0, 255]));
        assert_eq!(out.get_pixel(1, 1), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn line_draws_on_canvas() {
        let base = white(30, 30);
        let style = ShapeStyle {
            stroke: Color::rgb(0, 0, 0),
            stroke_width: 2.0,
            ..Default::default()
        };
        let doc = doc_with(
            30,
            30,
            ShapeKind::Line {
                from: Point::new(0.0, 15.0),
                to: Point::new(29.0, 15.0),
            },
            style,
        );
        let out = compose(&base, &doc, None);
        assert_eq!(out.get_pixel(15, 15), &Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn export_to_path_writes_png() {
        let base = white(16, 16);
        let doc = Document::new(16, 16);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.png");
        export_to_path(&base, &doc, None, &path).unwrap();
        let loaded = image::open(&path).unwrap();
        assert_eq!(loaded.width(), 16);
        assert_eq!(loaded.height(), 16);
    }
}
