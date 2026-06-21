//! Speech-bubble / callout tool: a rounded bubble plus a tail that points at an
//! anchor (the thing being annotated). Pure geometry; the canvas paints the shape.
use crate::geometry::{Point, Rect};
use serde::{Deserialize, Serialize};

/// Which edge of the bubble the tail leaves from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TailSide {
 Top,
 Bottom,
 Left,
 Right,
}

/// A callout: the bubble box, the anchor it points to, and corner rounding.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Callout {
 pub bubble: Rect,
 pub anchor: Point,
 pub corner_radius: f32,
}

impl Callout {
 /// The edge nearest the anchor, chosen by normalized offset from the center.
 pub fn tail_side(&self) -> TailSide {
 let c = self.bubble.center();
 let dx = self.anchor.x - c.x;
 let dy = self.anchor.y - c.y;
 if dx.abs() * self.bubble.h >= dy.abs() * self.bubble.w {
 if dx >= 0.0 { TailSide::Right } else { TailSide::Left }
 } else if dy >= 0.0 {
 TailSide::Bottom
 } else {
 TailSide::Top
 }
 }

 /// The three tail-triangle points: two base points on the bubble edge plus the tip
 /// at the anchor. `base_width` is the width of the tail base in pixels.
 pub fn tail_points(&self, base_width: f64) -> [Point; 3] {
 let r = &self.bubble;
 let hw = (base_width / 2.0).min(r.w.min(r.h) / 2.0).max(0.0);
 match self.tail_side() {
 TailSide::Bottom => {
 let bx = self.anchor.x.clamp(r.x + hw, r.right() - hw);
 [Point::new(bx - hw, r.bottom()), Point::new(bx + hw, r.bottom()), self.anchor]
 }
 TailSide::Top => {
 let bx = self.anchor.x.clamp(r.x + hw, r.right() - hw);
 [Point::new(bx - hw, r.y), Point::new(bx + hw, r.y), self.anchor]
 }
 TailSide::Right => {
 let by = self.anchor.y.clamp(r.y + hw, r.bottom() - hw);
 [Point::new(r.right(), by - hw), Point::new(r.right(), by + hw), self.anchor]
 }
 TailSide::Left => {
 let by = self.anchor.y.clamp(r.y + hw, r.bottom() - hw);
 [Point::new(r.x, by - hw), Point::new(r.x, by + hw), self.anchor]
 }
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 fn co(anchor: Point) -> Callout {
 Callout { bubble: Rect::new(0.0, 0.0, 100.0, 60.0), anchor, corner_radius: 8.0 }
 }

 #[test]
 fn side_below_is_bottom() {
 assert_eq!(co(Point::new(50.0, 200.0)).tail_side(), TailSide::Bottom);
 }

 #[test]
 fn side_right_is_right() {
 assert_eq!(co(Point::new(300.0, 30.0)).tail_side(), TailSide::Right);
 }

 #[test]
 fn side_above_is_top() {
 assert_eq!(co(Point::new(50.0, -100.0)).tail_side(), TailSide::Top);
 }

 #[test]
 fn tail_tip_is_anchor_and_base_on_edge() {
 let c = co(Point::new(50.0, 200.0));
 let pts = c.tail_points(20.0);
 assert_eq!(pts[2], Point::new(50.0, 200.0));
 assert_eq!(pts[0].y, 60.0);
 assert_eq!(pts[1].y, 60.0);
 assert!((pts[1].x - pts[0].x - 20.0).abs() < 1e-9);
 }

 #[test]
 fn base_clamped_within_edge() {
 let c = co(Point::new(500.0, 200.0));
 let pts = c.tail_points(20.0);
 // tail side becomes Right for a far-right anchor; base stays on the right edge
 assert_eq!(pts[0].x, 100.0);
 assert_eq!(pts[1].x, 100.0);
 }
}
