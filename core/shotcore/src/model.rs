//! The non-destructive annotation document model.
//!
//! A `Document` describes the source image plus an ordered list of vector
//! `Annotation`s. It is the single source of truth that the sidecar persists,
//! the undo engine snapshots, and the export pipeline rasterizes. Editing it
//! never touches the underlying image until an explicit flatten on export.

use crate::geometry::{Point, Rect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 8-bit RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub const fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
    pub fn is_opaque(&self) -> bool {
        self.a == 255
    }

    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(229, 57, 53);
    pub const ORANGE: Color = Color::rgb(251, 140, 0);
    pub const YELLOW: Color = Color::rgb(253, 216, 53);
    pub const GREEN: Color = Color::rgb(67, 160, 71);
    pub const BLUE: Color = Color::rgb(30, 136, 229);
    pub const PURPLE: Color = Color::rgb(142, 36, 170);

    /// Parse "#RRGGBB" or "#RRGGBBAA"; the leading hash is optional.
    pub fn from_hex(s: &str) -> Option<Color> {
        let h = s.strip_prefix("#").unwrap_or(s);
        let parse = |slice: &str| u8::from_str_radix(slice, 16).ok();
        match h.len() {
            6 => Some(Color::rgb(
                parse(&h[0..2])?,
                parse(&h[2..4])?,
                parse(&h[4..6])?,
            )),
            8 => Some(Color::rgba(
                parse(&h[0..2])?,
                parse(&h[2..4])?,
                parse(&h[4..6])?,
                parse(&h[6..8])?,
            )),
            _ => None,
        }
    }

    /// Format as "#RRGGBB" with alpha dropped.
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Format as "#RRGGBBAA".
    pub fn to_hex_rgba(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::RED
    }
}

/// How a redaction obscures pixels. Both destroy the underlying pixels on export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedactStyle {
    Blur,
    Pixelate,
 /// Solid black fill (Black Out).
 BlackOut,
 /// Pixelate then blur so detail cannot be recovered (secure blur).
 BlurSecure,
 /// Pixelate with per-block noise to defeat de-pixelization tools.
 PixelateRandomized,
}

/// The nine annotation tools, identifying a shape without its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    Arrow,
    Rectangle,
    Ellipse,
    Line,
    Freehand,
    Text,
    Counter,
    Highlighter,
    Redact,
}

/// Arrow rendering styles: straight, curved, double-headed, thin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowStyle {
 Straight,
 Curved,
 DoubleHeaded,
 Thin,
}
impl Default for ArrowStyle {
 fn default() -> Self {
 ArrowStyle::Straight
 }
}

/// One of seven predefined text styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextStyleId {
 Plain,
 Outline,
 Filled,
 Rounded,
 Bubble,
 Highlight,
 Shadow,
}
impl Default for TextStyleId {
 fn default() -> Self {
 TextStyleId::Plain
 }
}

/// serde default helper for boolean fields that default to true.
pub fn default_true() -> bool {
 true
}

/// Shared visual styling for stroked and filled shapes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShapeStyle {
    pub stroke: Color,
    pub fill: Option<Color>,
    pub stroke_width: f32,
    /// Overall opacity multiplier in the range 0.0 to 1.0.
    pub opacity: f32,
 /// Arrow rendering style for Arrow shapes.
 #[serde(default)]
 pub arrow_style: ArrowStyle,
 /// Fill rectangles/ellipses solid (filled-shape tools).
 #[serde(default)]
 pub filled: bool,
 /// Predefined text style for Text shapes.
 #[serde(default)]
 pub text_style: TextStyleId,
 /// Snap highlighter to OCR word boxes (smart highlighter).
 #[serde(default)]
 pub highlighter_smart: bool,
 /// Auto-smooth freehand pencil strokes.
 #[serde(default = "default_true")]
 pub pencil_smooth: bool,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            stroke: Color::RED,
            fill: None,
            stroke_width: 4.0,
            opacity: 1.0,
 arrow_style: ArrowStyle::Straight,
 filled: false,
 text_style: TextStyleId::Plain,
 highlighter_smart: false,
 pencil_smooth: true,
        }
    }
}

/// The geometry and per-shape data for each of the nine tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeKind {
    Arrow {
        from: Point,
        to: Point,
    },
    Rectangle {
        rect: Rect,
        corner_radius: f32,
    },
    Ellipse {
        rect: Rect,
    },
    Line {
        from: Point,
        to: Point,
    },
    Freehand {
        points: Vec<Point>,
    },
    Text {
        position: Point,
        content: String,
        font_size: f32,
    },
    Counter {
        center: Point,
        radius: f32,
        number: u32,
    },
    Highlighter {
        rect: Rect,
    },
    Redact {
        rect: Rect,
        style: RedactStyle,
        strength: u32,
    },
}

impl ShapeKind {
    /// The tool that produced this shape.
    pub fn tool(&self) -> Tool {
        match self {
            ShapeKind::Arrow { .. } => Tool::Arrow,
            ShapeKind::Rectangle { .. } => Tool::Rectangle,
            ShapeKind::Ellipse { .. } => Tool::Ellipse,
            ShapeKind::Line { .. } => Tool::Line,
            ShapeKind::Freehand { .. } => Tool::Freehand,
            ShapeKind::Text { .. } => Tool::Text,
            ShapeKind::Counter { .. } => Tool::Counter,
            ShapeKind::Highlighter { .. } => Tool::Highlighter,
            ShapeKind::Redact { .. } => Tool::Redact,
        }
    }

    /// True for tools whose export destroys the underlying pixels irreversibly.
    pub fn is_destructive(&self) -> bool {
        matches!(self, ShapeKind::Redact { .. })
    }
}

/// One annotation: a shape plus its style and stacking metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    pub kind: ShapeKind,
    pub style: ShapeStyle,
    /// Stacking order; higher draws on top.
    pub z: i32,
    pub hidden: bool,
    /// Clockwise rotation in degrees about the shape center.
    pub rotation: f32,
}

impl Annotation {
    /// Create an annotation with a fresh id and default style.
    pub fn new(kind: ShapeKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            style: ShapeStyle::default(),
            z: 0,
            hidden: false,
            rotation: 0.0,
        }
    }

    pub fn with_style(mut self, style: ShapeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn tool(&self) -> Tool {
        self.kind.tool()
    }
}

/// The current sidecar schema version, bumped on breaking format changes.
pub const SCHEMA_VERSION: u32 = 1;

/// A non-destructive editing document: the source image plus its annotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub schema_version: u32,
    pub image_width: u32,
    pub image_height: u32,
    /// Path to the source image the sidecar sits next to (relative preferred).
    pub image_path: Option<String>,
    pub annotations: Vec<Annotation>,
    /// Monotonic counter feeding numbered Counter badges.
    counter_seq: u32,
}

impl Document {
    pub fn new(image_width: u32, image_height: u32) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            image_width,
            image_height,
            image_path: None,
            annotations: Vec::new(),
            counter_seq: 0,
        }
    }

    pub fn with_image_path(mut self, path: impl Into<String>) -> Self {
        self.image_path = Some(path.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    /// Next number for a Counter badge (1-based), advancing the sequence.
    pub fn next_counter_number(&mut self) -> u32 {
        self.counter_seq += 1;
        self.counter_seq
    }

    /// Append an annotation, placing it on top, and return its id.
    pub fn add(&mut self, mut ann: Annotation) -> Uuid {
        ann.z = self.top_z() + 1;
        let id = ann.id;
        self.annotations.push(ann);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Annotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Annotation> {
        self.annotations.iter_mut().find(|a| a.id == id)
    }

    /// Remove an annotation by id, returning it when present.
    pub fn remove(&mut self, id: Uuid) -> Option<Annotation> {
        let idx = self.annotations.iter().position(|a| a.id == id)?;
        Some(self.annotations.remove(idx))
    }

    fn top_z(&self) -> i32 {
        self.annotations.iter().map(|a| a.z).max().unwrap_or(0)
    }

    fn bottom_z(&self) -> i32 {
        self.annotations.iter().map(|a| a.z).min().unwrap_or(0)
    }

    /// Raise an annotation above all others.
    pub fn bring_to_front(&mut self, id: Uuid) -> bool {
        let top = self.top_z();
        match self.get_mut(id) {
            Some(a) => {
                a.z = top + 1;
                true
            }
            None => false,
        }
    }

    /// Lower an annotation below all others.
    pub fn send_to_back(&mut self, id: Uuid) -> bool {
        let bottom = self.bottom_z();
        match self.get_mut(id) {
            Some(a) => {
                a.z = bottom - 1;
                true
            }
            None => false,
        }
    }

    /// Annotations ordered back-to-front for rendering (stable on ties).
    pub fn render_order(&self) -> Vec<&Annotation> {
        let mut v: Vec<&Annotation> = self.annotations.iter().filter(|a| !a.hidden).collect();
        v.sort_by(|a, b| a.z.cmp(&b.z));
        v
    }

    /// Full canvas rect in image pixel coordinates.
    pub fn canvas_rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.image_width as f64, self.image_height as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
 fn shape_style_extras_default_and_back_compat() {
 let s = ShapeStyle::default();
 assert_eq!(s.arrow_style, ArrowStyle::Straight);
 assert!(!s.filled);
 assert_eq!(s.text_style, TextStyleId::Plain);
 assert!(s.pencil_smooth);
 let old = "{\"stroke\":{\"r\":1,\"g\":2,\"b\":3,\"a\":4},\"fill\":null,\"stroke_width\":2.0,\"opacity\":1.0}";
 let parsed: ShapeStyle = serde_json::from_str(old).unwrap();
 assert_eq!(parsed.arrow_style, ArrowStyle::Straight);
 assert!(parsed.pencil_smooth);
 }

 fn render_ids(doc: &Document) -> Vec<Uuid> {
        doc.render_order().iter().map(|a| a.id).collect()
    }

    #[test]
    fn color_hex_roundtrip() {
        let c = Color::rgb(30, 136, 229);
        assert_eq!(c.to_hex(), "#1E88E5");
        assert_eq!(Color::from_hex("#1E88E5"), Some(c));
        assert_eq!(Color::from_hex("1e88e5"), Some(c));
        let witha = Color::rgba(1, 2, 3, 4);
        assert_eq!(witha.to_hex_rgba(), "#01020304");
        assert_eq!(Color::from_hex("#01020304"), Some(witha));
        assert_eq!(Color::from_hex("nope"), None);
        assert_eq!(Color::from_hex("#12345"), None);
    }

    #[test]
    fn tool_mapping_covers_nine_tools() {
        let kinds = vec![
            ShapeKind::Arrow {
                from: Point::new(0.0, 0.0),
                to: Point::new(1.0, 1.0),
            },
            ShapeKind::Rectangle {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                corner_radius: 0.0,
            },
            ShapeKind::Ellipse {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            },
            ShapeKind::Line {
                from: Point::new(0.0, 0.0),
                to: Point::new(1.0, 1.0),
            },
            ShapeKind::Freehand { points: vec![] },
            ShapeKind::Text {
                position: Point::new(0.0, 0.0),
                content: String::from("hi"),
                font_size: 12.0,
            },
            ShapeKind::Counter {
                center: Point::new(0.0, 0.0),
                radius: 10.0,
                number: 1,
            },
            ShapeKind::Highlighter {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            },
            ShapeKind::Redact {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                style: RedactStyle::Blur,
                strength: 8,
            },
        ];
        let tools: Vec<Tool> = kinds.iter().map(|k| k.tool()).collect();
        assert_eq!(
            tools,
            vec![
                Tool::Arrow,
                Tool::Rectangle,
                Tool::Ellipse,
                Tool::Line,
                Tool::Freehand,
                Tool::Text,
                Tool::Counter,
                Tool::Highlighter,
                Tool::Redact
            ]
        );
        assert!(kinds[8].is_destructive());
        assert!(!kinds[0].is_destructive());
    }

    #[test]
    fn add_assigns_increasing_z_and_returns_id() {
        let mut doc = Document::new(800, 600);
        let a = doc.add(Annotation::new(ShapeKind::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        }));
        let b = doc.add(Annotation::new(ShapeKind::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        }));
        assert_eq!(doc.len(), 2);
        assert!(doc.get(a).unwrap().z < doc.get(b).unwrap().z);
    }

    #[test]
    fn z_order_reordering() {
        let mut doc = Document::new(100, 100);
        let a = doc.add(Annotation::new(ShapeKind::Ellipse {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }));
        let b = doc.add(Annotation::new(ShapeKind::Ellipse {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }));
        let c = doc.add(Annotation::new(ShapeKind::Ellipse {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }));
        assert_eq!(render_ids(&doc), vec![a, b, c]);
        doc.bring_to_front(a);
        assert_eq!(render_ids(&doc), vec![b, c, a]);
        doc.send_to_back(c);
        assert_eq!(render_ids(&doc), vec![c, b, a]);
    }

    #[test]
    fn hidden_annotations_excluded_from_render() {
        let mut doc = Document::new(100, 100);
        let a = doc.add(Annotation::new(ShapeKind::Ellipse {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }));
        doc.get_mut(a).unwrap().hidden = true;
        assert!(doc.render_order().is_empty());
    }

    #[test]
    fn counter_sequence_advances() {
        let mut doc = Document::new(10, 10);
        assert_eq!(doc.next_counter_number(), 1);
        assert_eq!(doc.next_counter_number(), 2);
        assert_eq!(doc.next_counter_number(), 3);
    }

    #[test]
    fn remove_returns_annotation() {
        let mut doc = Document::new(10, 10);
        let a = doc.add(Annotation::new(ShapeKind::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        }));
        assert!(doc.remove(a).is_some());
        assert!(doc.remove(a).is_none());
        assert!(doc.is_empty());
    }
}
