//! On-screen measurement and alignment (PixelSnap style): distances between points,
//! alignment guides derived from element rects, and snapping a coordinate to guides.
use crate::geometry::{Point, Rect};
use serde::{Deserialize, Serialize};

/// Signed delta (dx, dy) from a to b.
pub fn delta(a: Point, b: Point) -> (f64, f64) {
 (b.x - a.x, b.y - a.y)
}

/// Euclidean distance between two points.
pub fn distance(a: Point, b: Point) -> f64 {
 let (dx, dy) = delta(a, b);
 (dx * dx + dy * dy).sqrt()
}

/// Angle of the a->b vector in degrees (0 = east, 90 = south in image space).
pub fn angle_deg(a: Point, b: Point) -> f64 {
 let (dx, dy) = delta(a, b);
 dy.atan2(dx).to_degrees()
}

/// Candidate alignment guide lines from a set of rects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Guides {
 pub xs: Vec<f64>,
 pub ys: Vec<f64>,
}

fn push_unique(v: &mut Vec<f64>, value: f64) {
 if !v.iter().any(|x| (x - value).abs() < 0.5) {
 v.push(value);
 }
}

/// Left/center/right vertical guides and top/middle/bottom horizontal guides.
pub fn guides_from_rects(rects: &[Rect]) -> Guides {
 let mut g = Guides::default();
 for r in rects {
 push_unique(&mut g.xs, r.x);
 push_unique(&mut g.xs, r.center().x);
 push_unique(&mut g.xs, r.right());
 push_unique(&mut g.ys, r.y);
 push_unique(&mut g.ys, r.center().y);
 push_unique(&mut g.ys, r.bottom());
 }
 g.xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
 g.ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
 g
}

/// Snap a value to the nearest guide within threshold; returns the snapped value.
pub fn snap(value: f64, guides: &[f64], threshold: f64) -> f64 {
 let mut best = value;
 let mut best_d = threshold;
 for &g in guides {
 let d = (g - value).abs();
 if d <= best_d {
 best_d = d;
 best = g;
 }
 }
 best
}

/// Snap a point to the guides on both axes.
pub fn snap_point(p: Point, guides: &Guides, threshold: f64) -> Point {
 Point::new(snap(p.x, &guides.xs, threshold), snap(p.y, &guides.ys, threshold))
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn distance_and_delta() {
 let a = Point::new(0.0, 0.0);
 let b = Point::new(3.0, 4.0);
 assert_eq!(delta(a, b), (3.0, 4.0));
 assert!((distance(a, b) - 5.0).abs() < 1e-9);
 }

 #[test]
 fn angle_east_and_south() {
 assert!((angle_deg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)) - 0.0).abs() < 1e-9);
 assert!((angle_deg(Point::new(0.0, 0.0), Point::new(0.0, 1.0)) - 90.0).abs() < 1e-9);
 }

 #[test]
 fn guides_from_one_rect() {
 let g = guides_from_rects(&[Rect::new(10.0, 20.0, 100.0, 40.0)]);
 assert_eq!(g.xs, vec![10.0, 60.0, 110.0]);
 assert_eq!(g.ys, vec![20.0, 40.0, 60.0]);
 }

 #[test]
 fn snapping_within_threshold() {
 let g = guides_from_rects(&[Rect::new(10.0, 20.0, 100.0, 40.0)]);
 let p = snap_point(Point::new(12.0, 100.0), &g, 5.0);
 assert_eq!(p.x, 10.0);
 assert_eq!(p.y, 100.0);
 }

 #[test]
 fn no_snap_when_far() {
 assert_eq!(snap(50.0, &[10.0, 110.0], 5.0), 50.0);
 }
}
