//! Align and distribute objects: move a set of rects to share an edge/center, or
//! space them evenly. Mirrors the align/distribute controls of a real editor.
use crate::geometry::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
 Left,
 HCenter,
 Right,
 Top,
 VCenter,
 Bottom,
}

fn bounds(rects: &[Rect]) -> Rect {
 let mut b = rects[0];
 for r in &rects[1..] {
 b = b.union(r);
 }
 b
}

/// Align every rect to the shared bounding box on the given edge/center.
pub fn align(rects: &[Rect], edge: Edge) -> Vec<Rect> {
 if rects.is_empty() {
 return Vec::new();
 }
 let b = bounds(rects);
 rects
 .iter()
 .map(|r| {
 let mut nr = *r;
 match edge {
 Edge::Left => nr.x = b.x,
 Edge::Right => nr.x = b.right() - r.w,
 Edge::HCenter => nr.x = b.center().x - r.w / 2.0,
 Edge::Top => nr.y = b.y,
 Edge::Bottom => nr.y = b.bottom() - r.h,
 Edge::VCenter => nr.y = b.center().y - r.h / 2.0,
 }
 nr
 })
 .collect()
}

/// Distribute rects horizontally with equal gaps; first and last stay put.
pub fn distribute_horizontal(rects: &[Rect]) -> Vec<Rect> {
 if rects.len() < 3 {
 return rects.to_vec();
 }
 let mut idx: Vec<usize> = (0..rects.len()).collect();
 idx.sort_by(|&a, &b| rects[a].x.partial_cmp(&rects[b].x).unwrap());
 let first_x = rects[idx[0]].x;
 let last = rects[*idx.last().unwrap()];
 let total_w: f64 = rects.iter().map(|r| r.w).sum();
 let span = last.right() - first_x;
 let gap = (span - total_w) / (rects.len() as f64 - 1.0);
 let mut out = rects.to_vec();
 let mut cursor = first_x;
 for &i in &idx {
 out[i].x = cursor;
 cursor += rects[i].w + gap;
 }
 out
}

/// Distribute rects vertically with equal gaps; first and last stay put.
pub fn distribute_vertical(rects: &[Rect]) -> Vec<Rect> {
 if rects.len() < 3 {
 return rects.to_vec();
 }
 let mut idx: Vec<usize> = (0..rects.len()).collect();
 idx.sort_by(|&a, &b| rects[a].y.partial_cmp(&rects[b].y).unwrap());
 let first_y = rects[idx[0]].y;
 let last = rects[*idx.last().unwrap()];
 let total_h: f64 = rects.iter().map(|r| r.h).sum();
 let span = last.bottom() - first_y;
 let gap = (span - total_h) / (rects.len() as f64 - 1.0);
 let mut out = rects.to_vec();
 let mut cursor = first_y;
 for &i in &idx {
 out[i].y = cursor;
 cursor += rects[i].h + gap;
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn align_left_uses_min_x() {
 let rs = [Rect::new(10.0, 0.0, 20.0, 10.0), Rect::new(40.0, 0.0, 30.0, 10.0)];
 let a = align(&rs, Edge::Left);
 assert_eq!(a[0].x, 10.0);
 assert_eq!(a[1].x, 10.0);
 }

 #[test]
 fn align_hcenter_centers_each() {
 let rs = [Rect::new(0.0, 0.0, 20.0, 10.0), Rect::new(0.0, 0.0, 100.0, 10.0)];
 let a = align(&rs, Edge::HCenter);
 let cx = 50.0;
 assert_eq!(a[0].x, cx - 10.0);
 assert_eq!(a[1].x, cx - 50.0);
 }

 #[test]
 fn distribute_h_equalizes_gaps_and_pins_ends() {
 let rs = [
 Rect::new(0.0, 0.0, 10.0, 10.0),
 Rect::new(15.0, 0.0, 10.0, 10.0),
 Rect::new(80.0, 0.0, 10.0, 10.0),
 ];
 let d = distribute_horizontal(&rs);
 assert_eq!(d[0].x, 0.0);
 assert_eq!(d[2].x, 80.0);
 // total width 30, span 90 -> free 60 over 2 gaps -> gap 30; middle at 0+10+30=40
 assert_eq!(d[1].x, 40.0);
 }

 #[test]
 fn distribute_needs_three() {
 let rs = [Rect::new(0.0, 0.0, 10.0, 10.0), Rect::new(50.0, 0.0, 10.0, 10.0)];
 assert_eq!(distribute_horizontal(&rs), rs.to_vec());
 }
}
