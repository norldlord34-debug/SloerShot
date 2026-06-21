//! Hit-testing and geometric transforms for the annotation editor.
//!
//! These functions let the native canvas answer what the user clicked and where
//! a shape moves or resizes to, purely from the document model, so both
//! platforms share identical selection behavior.

use crate::geometry::{Point, Rect};
use crate::model::{Annotation, Document, ShapeKind};
use uuid::Uuid;

fn dist(a: Point, b: Point) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Shortest distance from point `p` to the segment a-b.
pub fn point_segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq <= f64::EPSILON {
        return dist(p, a);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq).clamp(0.0, 1.0);
    dist(p, Point::new(a.x + t * dx, a.y + t * dy))
}

/// Whether `p` lies inside the ellipse inscribed in `rect`.
pub fn point_in_ellipse(p: Point, rect: &Rect) -> bool {
    let r = rect.normalized();
    let rx = r.w / 2.0;
    let ry = r.h / 2.0;
    if rx <= 0.0 || ry <= 0.0 {
        return false;
    }
    let nx = (p.x - (r.x + rx)) / rx;
    let ny = (p.y - (r.y + ry)) / ry;
    nx * nx + ny * ny <= 1.0
}

fn bounds_of_points(points: &[Point]) -> Rect {
    if points.is_empty() {
        return Rect::new(0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Axis-aligned bounding box of a shape in image pixel coordinates.
pub fn shape_bounds(kind: &ShapeKind) -> Rect {
    match kind {
        ShapeKind::Arrow { from, to } | ShapeKind::Line { from, to } => {
            Rect::from_corners(*from, *to)
        }
        ShapeKind::Rectangle { rect, .. }
        | ShapeKind::Ellipse { rect }
        | ShapeKind::Highlighter { rect }
        | ShapeKind::Redact { rect, .. } => rect.normalized(),
        ShapeKind::Freehand { points } => bounds_of_points(points),
        ShapeKind::Text {
            position,
            content,
            font_size,
        } => {
            let w = (content.chars().count().max(1) as f64) * (*font_size as f64) * 0.6;
            let h = (*font_size as f64) * 1.3;
            Rect::new(position.x, position.y, w, h)
        }
        ShapeKind::Counter { center, radius, .. } => {
            let r = *radius as f64;
            Rect::new(center.x - r, center.y - r, 2.0 * r, 2.0 * r)
        }
    }
}

fn pick_tol(ann: &Annotation, tolerance: f64) -> f64 {
    tolerance + (ann.style.stroke_width as f64) / 2.0
}

/// Whether a click at `p` with the given base tolerance selects `ann`.
pub fn hit_test(ann: &Annotation, p: Point, tolerance: f64) -> bool {
    if ann.hidden {
        return false;
    }
    let tol = pick_tol(ann, tolerance);
    match &ann.kind {
        ShapeKind::Arrow { from, to } | ShapeKind::Line { from, to } => {
            point_segment_distance(p, *from, *to) <= tol
        }
        ShapeKind::Freehand { points } => points
            .windows(2)
            .any(|w| point_segment_distance(p, w[0], w[1]) <= tol),
        ShapeKind::Rectangle { rect, .. } => {
            let r = rect.normalized();
            if ann.style.fill.is_some() {
                r.inflate(tol).contains_point_inclusive(p)
            } else {
                r.inflate(tol).contains_point_inclusive(p)
                    && !r.inflate(-tol).contains_point_inclusive(p)
            }
        }
        ShapeKind::Ellipse { rect } => {
            let r = rect.normalized();
            if ann.style.fill.is_some() {
                point_in_ellipse(p, &r.inflate(tol))
            } else {
                point_in_ellipse(p, &r.inflate(tol)) && !point_in_ellipse(p, &r.inflate(-tol))
            }
        }
        ShapeKind::Highlighter { rect } | ShapeKind::Redact { rect, .. } => rect
            .normalized()
            .inflate(tolerance)
            .contains_point_inclusive(p),
        ShapeKind::Text { .. } => shape_bounds(&ann.kind)
            .inflate(tolerance)
            .contains_point_inclusive(p),
        ShapeKind::Counter { center, radius, .. } => {
            dist(p, *center) <= (*radius as f64) + tolerance
        }
    }
}

/// Topmost annotation hit at `p`, scanning front-to-back.
pub fn topmost_at(doc: &Document, p: Point, tolerance: f64) -> Option<Uuid> {
    doc.render_order()
        .into_iter()
        .rev()
        .find(|a| hit_test(a, p, tolerance))
        .map(|a| a.id)
}

/// Move every defining point of a shape by (dx, dy).
pub fn translate(kind: &ShapeKind, dx: f64, dy: f64) -> ShapeKind {
    let mv = |p: Point| Point::new(p.x + dx, p.y + dy);
    let mv_rect = |r: &Rect| Rect::new(r.x + dx, r.y + dy, r.w, r.h);
    match kind {
        ShapeKind::Arrow { from, to } => ShapeKind::Arrow {
            from: mv(*from),
            to: mv(*to),
        },
        ShapeKind::Line { from, to } => ShapeKind::Line {
            from: mv(*from),
            to: mv(*to),
        },
        ShapeKind::Rectangle {
            rect,
            corner_radius,
        } => ShapeKind::Rectangle {
            rect: mv_rect(rect),
            corner_radius: *corner_radius,
        },
        ShapeKind::Ellipse { rect } => ShapeKind::Ellipse {
            rect: mv_rect(rect),
        },
        ShapeKind::Highlighter { rect } => ShapeKind::Highlighter {
            rect: mv_rect(rect),
        },
        ShapeKind::Redact {
            rect,
            style,
            strength,
        } => ShapeKind::Redact {
            rect: mv_rect(rect),
            style: *style,
            strength: *strength,
        },
        ShapeKind::Freehand { points } => ShapeKind::Freehand {
            points: points.iter().map(|p| mv(*p)).collect(),
        },
        ShapeKind::Text {
            position,
            content,
            font_size,
        } => ShapeKind::Text {
            position: mv(*position),
            content: content.clone(),
            font_size: *font_size,
        },
        ShapeKind::Counter {
            center,
            radius,
            number,
        } => ShapeKind::Counter {
            center: mv(*center),
            radius: *radius,
            number: *number,
        },
    }
}

/// Reposition and rescale a shape so its bounding box becomes `new`.
pub fn set_bounds(kind: &ShapeKind, new: Rect) -> ShapeKind {
    let old = shape_bounds(kind);
    let new = new.normalized();
    let sx = if old.w.abs() > f64::EPSILON {
        new.w / old.w
    } else {
        1.0
    };
    let sy = if old.h.abs() > f64::EPSILON {
        new.h / old.h
    } else {
        1.0
    };
    let map = |p: Point| Point::new(new.x + (p.x - old.x) * sx, new.y + (p.y - old.y) * sy);
    match kind {
        ShapeKind::Arrow { from, to } => ShapeKind::Arrow {
            from: map(*from),
            to: map(*to),
        },
        ShapeKind::Line { from, to } => ShapeKind::Line {
            from: map(*from),
            to: map(*to),
        },
        ShapeKind::Rectangle { corner_radius, .. } => ShapeKind::Rectangle {
            rect: new,
            corner_radius: *corner_radius,
        },
        ShapeKind::Ellipse { .. } => ShapeKind::Ellipse { rect: new },
        ShapeKind::Highlighter { .. } => ShapeKind::Highlighter { rect: new },
        ShapeKind::Redact {
            style, strength, ..
        } => ShapeKind::Redact {
            rect: new,
            style: *style,
            strength: *strength,
        },
        ShapeKind::Freehand { points } => ShapeKind::Freehand {
            points: points.iter().map(|p| map(*p)).collect(),
        },
        ShapeKind::Text {
            content, font_size, ..
        } => {
            let scale = if old.h.abs() > f64::EPSILON {
                (new.h / old.h) as f32
            } else {
                1.0
            };
            ShapeKind::Text {
                position: Point::new(new.x, new.y),
                content: content.clone(),
                font_size: (*font_size) * scale,
            }
        }
        ShapeKind::Counter { number, .. } => ShapeKind::Counter {
            center: new.center(),
            radius: (new.w.min(new.h) / 2.0) as f32,
            number: *number,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Color, ShapeStyle};

    fn ann(kind: ShapeKind) -> Annotation {
        Annotation::new(kind)
    }

    #[test]
    fn segment_distance_and_ellipse() {
        let d = point_segment_distance(
            Point::new(5.0, 5.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        );
        assert!((d - 5.0).abs() < 1e-9);
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(point_in_ellipse(Point::new(5.0, 5.0), &r));
        assert!(!point_in_ellipse(Point::new(0.0, 0.0), &r));
    }

    #[test]
    fn bounds_cover_shapes() {
        let lb = shape_bounds(&ShapeKind::Line {
            from: Point::new(10.0, 20.0),
            to: Point::new(0.0, 5.0),
        });
        assert_eq!(lb, Rect::new(0.0, 5.0, 10.0, 15.0));
        let fb = shape_bounds(&ShapeKind::Freehand {
            points: vec![
                Point::new(1.0, 2.0),
                Point::new(5.0, 1.0),
                Point::new(3.0, 9.0),
            ],
        });
        assert_eq!(fb, Rect::new(1.0, 1.0, 4.0, 8.0));
        let cb = shape_bounds(&ShapeKind::Counter {
            center: Point::new(50.0, 50.0),
            radius: 12.0,
            number: 1,
        });
        assert_eq!(cb, Rect::new(38.0, 38.0, 24.0, 24.0));
    }

    #[test]
    fn hit_test_line_segment() {
        let a = ann(ShapeKind::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(100.0, 0.0),
        });
        assert!(hit_test(&a, Point::new(50.0, 1.0), 3.0));
        assert!(!hit_test(&a, Point::new(50.0, 40.0), 3.0));
    }

    #[test]
    fn hit_test_filled_vs_outline_rectangle() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let filled = ann(ShapeKind::Rectangle {
            rect,
            corner_radius: 0.0,
        })
        .with_style(ShapeStyle {
            fill: Some(Color::BLUE),
            ..ShapeStyle::default()
        });
        let outline = ann(ShapeKind::Rectangle {
            rect,
            corner_radius: 0.0,
        });
        let center = Point::new(50.0, 50.0);
        assert!(hit_test(&filled, center, 2.0));
        assert!(!hit_test(&outline, center, 2.0));
        assert!(hit_test(&outline, Point::new(0.0, 50.0), 2.0));
    }

    #[test]
    fn topmost_prefers_front() {
        let mut doc = Document::new(200, 200);
        let back = doc.add(
            ann(ShapeKind::Rectangle {
                rect: Rect::new(0.0, 0.0, 100.0, 100.0),
                corner_radius: 0.0,
            })
            .with_style(ShapeStyle {
                fill: Some(Color::RED),
                ..Default::default()
            }),
        );
        let front = doc.add(
            ann(ShapeKind::Rectangle {
                rect: Rect::new(0.0, 0.0, 100.0, 100.0),
                corner_radius: 0.0,
            })
            .with_style(ShapeStyle {
                fill: Some(Color::BLUE),
                ..Default::default()
            }),
        );
        assert_eq!(topmost_at(&doc, Point::new(50.0, 50.0), 1.0), Some(front));
        assert_ne!(topmost_at(&doc, Point::new(50.0, 50.0), 1.0), Some(back));
    }

    #[test]
    fn translate_moves_all_geometry() {
        let moved = translate(
            &ShapeKind::Arrow {
                from: Point::new(0.0, 0.0),
                to: Point::new(10.0, 10.0),
            },
            5.0,
            -3.0,
        );
        match moved {
            ShapeKind::Arrow { from, to } => {
                assert_eq!(from, Point::new(5.0, -3.0));
                assert_eq!(to, Point::new(15.0, 7.0));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn set_bounds_rescales() {
        let line = ShapeKind::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(10.0, 10.0),
        };
        match set_bounds(&line, Rect::new(100.0, 100.0, 20.0, 20.0)) {
            ShapeKind::Line { from, to } => {
                assert_eq!(from, Point::new(100.0, 100.0));
                assert_eq!(to, Point::new(120.0, 120.0));
            }
            _ => panic!("wrong variant"),
        }
        let rectk = ShapeKind::Rectangle {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            corner_radius: 4.0,
        };
        match set_bounds(&rectk, Rect::new(5.0, 5.0, 50.0, 30.0)) {
            ShapeKind::Rectangle {
                rect,
                corner_radius,
            } => {
                assert_eq!(rect, Rect::new(5.0, 5.0, 50.0, 30.0));
                assert_eq!(corner_radius, 4.0);
            }
            _ => panic!("wrong variant"),
        }
    }
}
