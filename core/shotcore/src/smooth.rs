//! Stroke smoothing and curved-arrow geometry.
//!
//! Pencil auto-smoothing uses Chaikin corner-cutting; curved arrows are sampled from
//! a quadratic Bezier whose control point is offset perpendicular to the chord.
use crate::geometry::Point;

/// One Chaikin corner-cutting pass, keeping the first and last points fixed.
fn chaikin_pass(points: &[Point]) -> Vec<Point> {
 if points.len() < 3 {
 return points.to_vec();
 }
 let mut out = Vec::with_capacity(points.len() * 2);
 out.push(points[0]);
 for w in points.windows(2) {
 let (a, b) = (w[0], w[1]);
 out.push(Point::new(0.75 * a.x + 0.25 * b.x, 0.75 * a.y + 0.25 * b.y));
 out.push(Point::new(0.25 * a.x + 0.75 * b.x, 0.25 * a.y + 0.75 * b.y));
 }
 out.push(points[points.len() - 1]);
 out
}

/// Smooth a freehand stroke with `iterations` Chaikin passes (auto-smoothing).
pub fn smooth_stroke(points: &[Point], iterations: u32) -> Vec<Point> {
 let mut pts = points.to_vec();
 for _ in 0..iterations {
 pts = chaikin_pass(&pts);
 }
 pts
}

/// A point on the quadratic Bezier defined by p0, control c, p1 at parameter t.
pub fn quad_bezier(p0: Point, c: Point, p1: Point, t: f64) -> Point {
 let mt = 1.0 - t;
 let x = mt * mt * p0.x + 2.0 * mt * t * c.x + t * t * p1.x;
 let y = mt * mt * p0.y + 2.0 * mt * t * c.y + t * t * p1.y;
 Point::new(x, y)
}

/// Sample a curved arrow shaft from `from` to `to`. `bow` is the perpendicular
/// offset of the control point as a fraction of the chord length (0 = straight).
/// Returns `segments + 1` points along the curve.
pub fn curved_arrow_path(from: Point, to: Point, bow: f64, segments: u32) -> Vec<Point> {
 let segments = segments.max(1);
 let mid = Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
 let dx = to.x - from.x;
 let dy = to.y - from.y;
 let len = (dx * dx + dy * dy).sqrt();
 // perpendicular unit vector
 let (px, py) = if len > 0.0 { (-dy / len, dx / len) } else { (0.0, 0.0) };
 let off = bow * len;
 let c = Point::new(mid.x + px * off, mid.y + py * off);
 (0..=segments)
 .map(|i| quad_bezier(from, c, to, i as f64 / segments as f64))
 .collect()
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn short_strokes_unchanged() {
 let pts = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
 assert_eq!(smooth_stroke(&pts, 2), pts);
 }

 #[test]
 fn smoothing_grows_and_keeps_endpoints() {
 let pts = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(10.0, 10.0)];
 let s = smooth_stroke(&pts, 1);
 assert!(s.len() > pts.len());
 assert_eq!(s[0], pts[0]);
 assert_eq!(*s.last().unwrap(), *pts.last().unwrap());
 }

 #[test]
 fn straight_bezier_midpoint() {
 let p = quad_bezier(Point::new(0.0, 0.0), Point::new(5.0, 0.0), Point::new(10.0, 0.0), 0.5);
 assert!((p.x - 5.0).abs() < 1e-9);
 assert!((p.y - 0.0).abs() < 1e-9);
 }

 #[test]
 fn curved_path_bows_out_and_hits_endpoints() {
 let from = Point::new(0.0, 0.0);
 let to = Point::new(10.0, 0.0);
 let path = curved_arrow_path(from, to, 0.5, 8);
 assert_eq!(path.len(), 9);
 assert_eq!(path[0], from);
 assert_eq!(*path.last().unwrap(), to);
 // midpoint is offset off the straight chord
 assert!(path[4].y.abs() > 1.0);
 }

 #[test]
 fn zero_bow_is_straight() {
 let path = curved_arrow_path(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 0.0, 4);
 for p in &path {
 assert!(p.y.abs() < 1e-9);
 }
 }
}
