//! Vector export: render an annotation Document to standalone SVG markup. Useful for
//! infinite-resolution sharing and editing in vector tools. Redactions export as solid
//! fills (SVG cannot carry the destructive blur/pixelate that raster export applies).
use crate::model::{Annotation, Color, Document, ShapeKind};

fn esc(s: &str) -> String {
 s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
}

fn fill_attr(c: Color, mult: f32) -> String {
 format!("fill=\"rgb({},{},{})\" fill-opacity=\"{:.3}\"", c.r, c.g, c.b, (c.a as f32 / 255.0) * mult)
}

fn stroke_attr(c: Color, w: f32, mult: f32) -> String {
 format!(
 "stroke=\"rgb({},{},{})\" stroke-opacity=\"{:.3}\" stroke-width=\"{}\" fill=\"none\"",
 c.r, c.g, c.b, (c.a as f32 / 255.0) * mult, w
 )
}

fn shape_svg(a: &Annotation) -> String {
 let st = &a.style;
 let op = st.opacity;
 match &a.kind {
 ShapeKind::Rectangle { rect, corner_radius } => {
 let fill = match st.fill {
 Some(c) => fill_attr(c, op),
 None => "fill=\"none\"".to_string(),
 };
 format!(
 "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" {} stroke=\"rgb({},{},{})\" stroke-width=\"{}\" />",
 rect.x, rect.y, rect.w, rect.h, corner_radius, fill, st.stroke.r, st.stroke.g, st.stroke.b, st.stroke_width
 )
 }
 ShapeKind::Ellipse { rect } => format!(
 "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" {} />",
 rect.x + rect.w / 2.0, rect.y + rect.h / 2.0, rect.w / 2.0, rect.h / 2.0, stroke_attr(st.stroke, st.stroke_width, op)
 ),
 ShapeKind::Line { from, to } => format!(
 "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" {} />",
 from.x, from.y, to.x, to.y, stroke_attr(st.stroke, st.stroke_width, op)
 ),
 ShapeKind::Arrow { from, to } => {
 let (dx, dy) = (to.x - from.x, to.y - from.y);
 let len = (dx * dx + dy * dy).sqrt();
 let mut s = format!(
 "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" {} />",
 from.x, from.y, to.x, to.y, stroke_attr(st.stroke, st.stroke_width, op)
 );
 if len > 0.0 {
 let (ux, uy) = (dx / len, dy / len);
 let (px, py) = (-uy, ux);
 let size = (st.stroke_width as f64 * 3.0).max(8.0);
 let b1 = (to.x - ux * size + px * size * 0.5, to.y - uy * size + py * size * 0.5);
 let b2 = (to.x - ux * size - px * size * 0.5, to.y - uy * size - py * size * 0.5);
 s.push_str(&format!(
 "<polygon points=\"{},{} {},{} {},{}\" fill=\"rgb({},{},{})\" />",
 to.x, to.y, b1.0, b1.1, b2.0, b2.1, st.stroke.r, st.stroke.g, st.stroke.b
 ));
 }
 s
 }
 ShapeKind::Freehand { points } => {
 let pts: Vec<String> = points.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
 format!("<polyline points=\"{}\" {} />", pts.join(" "), stroke_attr(st.stroke, st.stroke_width, op))
 }
 ShapeKind::Text { position, content, font_size } => format!(
 "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"rgb({},{},{})\">{}</text>",
 position.x, position.y, font_size, st.stroke.r, st.stroke.g, st.stroke.b, esc(content)
 ),
 ShapeKind::Counter { center, radius, number } => format!(
 "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"rgb({},{},{})\" /><text x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"white\">{}</text>",
 center.x, center.y, radius, st.stroke.r, st.stroke.g, st.stroke.b, center.x, center.y, radius, number
 ),
 ShapeKind::Highlighter { rect } => format!(
 "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"rgb({},{},{})\" fill-opacity=\"{:.3}\" />",
 rect.x, rect.y, rect.w, rect.h, st.stroke.r, st.stroke.g, st.stroke.b, 0.4 * op
 ),
 ShapeKind::Redact { rect, .. } => format!(
 "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"black\" />",
 rect.x, rect.y, rect.w, rect.h
 ),
 }
}

/// Render the whole document to standalone SVG markup.
pub fn to_svg(doc: &Document) -> String {
 let mut s = format!(
 "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
 doc.image_width, doc.image_height, doc.image_width, doc.image_height
 );
 for a in doc.render_order() {
 s.push_str(&shape_svg(a));
 s.push('\n');
 }
 s.push_str("</svg>\n");
 s
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::geometry::{Point, Rect};
 use crate::model::{Annotation, ShapeKind};

 #[test]
 fn svg_wraps_and_sizes() {
 let doc = Document::new(100, 50);
 let svg = to_svg(&doc);
 assert!(svg.starts_with("<svg"));
 assert!(svg.contains("width=\"100\""));
 assert!(svg.contains("viewBox=\"0 0 100 50\""));
 assert!(svg.trim_end().ends_with("</svg>"));
 }

 #[test]
 fn rectangle_becomes_rect() {
 let mut doc = Document::new(100, 100);
 doc.add(Annotation::new(ShapeKind::Rectangle { rect: Rect::new(10.0, 20.0, 30.0, 40.0), corner_radius: 0.0 }));
 let svg = to_svg(&doc);
 assert!(svg.contains("<rect x=\"10\" y=\"20\" width=\"30\" height=\"40\""));
 }

 #[test]
 fn text_is_escaped() {
 let mut doc = Document::new(100, 100);
 doc.add(Annotation::new(ShapeKind::Text { position: Point::new(5.0, 5.0), content: "a<b>&c".to_string(), font_size: 16.0 }));
 let svg = to_svg(&doc);
 assert!(svg.contains(">a&lt;b&gt;&amp;c</text>"));
 }

 #[test]
 fn arrow_has_line_and_head() {
 let mut doc = Document::new(100, 100);
 doc.add(Annotation::new(ShapeKind::Arrow { from: Point::new(0.0, 0.0), to: Point::new(50.0, 0.0) }));
 let svg = to_svg(&doc);
 assert!(svg.contains("<line"));
 assert!(svg.contains("<polygon"));
 }
}
