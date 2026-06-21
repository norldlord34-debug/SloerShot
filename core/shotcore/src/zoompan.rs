//! Recording auto-zoom and cursor smoothing (Screen Studio style): generate eased
//! zoom keyframes around click moments and smooth a noisy cursor path. Pure math;
//! the native compositor applies the resulting transform per frame.
use serde::{Deserialize, Serialize};

/// A pointer click during a recording.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClickEvent {
 pub t_ms: u64,
 pub x: f64,
 pub y: f64,
}

/// A zoom keyframe: scale and the focal center to zoom toward.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoomKeyframe {
 pub t_ms: u64,
 pub scale: f64,
 pub cx: f64,
 pub cy: f64,
}

/// Cubic ease-in-out in 0.0..=1.0.
pub fn ease_in_out_cubic(t: f64) -> f64 {
 let t = t.clamp(0.0, 1.0);
 if t < 0.5 {
 4.0 * t * t * t
 } else {
 let u = -2.0 * t + 2.0;
 1.0 - (u * u * u) / 2.0
 }
}

/// Build zoom keyframes that zoom toward each click, hold, then return to 1.0.
pub fn auto_zoom_keyframes(clicks: &[ClickEvent], scale: f64, ease_ms: u64, hold_ms: u64) -> Vec<ZoomKeyframe> {
 let mut out = Vec::new();
 for c in clicks {
 out.push(ZoomKeyframe { t_ms: c.t_ms, scale: 1.0, cx: c.x, cy: c.y });
 out.push(ZoomKeyframe { t_ms: c.t_ms + ease_ms, scale, cx: c.x, cy: c.y });
 out.push(ZoomKeyframe { t_ms: c.t_ms + ease_ms + hold_ms, scale, cx: c.x, cy: c.y });
 out.push(ZoomKeyframe { t_ms: c.t_ms + 2 * ease_ms + hold_ms, scale: 1.0, cx: c.x, cy: c.y });
 }
 out
}

/// Sample the (scale, cx, cy) transform at time t_ms by eased interpolation.
pub fn sample(keyframes: &[ZoomKeyframe], t_ms: u64) -> (f64, f64, f64) {
 match keyframes.first() {
 None => return (1.0, 0.0, 0.0),
 Some(k) if t_ms <= k.t_ms => return (k.scale, k.cx, k.cy),
 _ => {}
 }
 let last = keyframes.last().unwrap();
 if t_ms >= last.t_ms {
 return (last.scale, last.cx, last.cy);
 }
 for w in keyframes.windows(2) {
 let (a, b) = (&w[0], &w[1]);
 if t_ms >= a.t_ms && t_ms <= b.t_ms {
 let span = (b.t_ms - a.t_ms) as f64;
 let f = if span > 0.0 { ease_in_out_cubic((t_ms - a.t_ms) as f64 / span) } else { 1.0 };
 return (
 a.scale + (b.scale - a.scale) * f,
 a.cx + (b.cx - a.cx) * f,
 a.cy + (b.cy - a.cy) * f,
 );
 }
 }
 (last.scale, last.cx, last.cy)
}

/// Exponential-moving-average smoothing of a cursor path (alpha in 0.0..=1.0).
pub fn smooth_cursor(points: &[(f64, f64)], alpha: f64) -> Vec<(f64, f64)> {
 let a = alpha.clamp(0.0, 1.0);
 let mut out = Vec::with_capacity(points.len());
 let mut prev: Option<(f64, f64)> = None;
 for &(x, y) in points {
 let p = match prev {
 None => (x, y),
 Some((px, py)) => (px + (x - px) * a, py + (y - py) * a),
 };
 out.push(p);
 prev = Some(p);
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn easing_endpoints() {
 assert!((ease_in_out_cubic(0.0) - 0.0).abs() < 1e-9);
 assert!((ease_in_out_cubic(1.0) - 1.0).abs() < 1e-9);
 assert!((ease_in_out_cubic(0.5) - 0.5).abs() < 1e-9);
 }

 #[test]
 fn keyframes_per_click() {
 let kf = auto_zoom_keyframes(&[ClickEvent { t_ms: 1000, x: 50.0, y: 60.0 }], 2.0, 300, 800);
 assert_eq!(kf.len(), 4);
 assert_eq!(kf[0].scale, 1.0);
 assert_eq!(kf[1].scale, 2.0);
 assert_eq!(kf[1].t_ms, 1300);
 assert_eq!(kf[3].t_ms, 2400);
 }

 #[test]
 fn sample_reaches_full_zoom_during_hold() {
 let kf = auto_zoom_keyframes(&[ClickEvent { t_ms: 0, x: 10.0, y: 20.0 }], 2.0, 200, 500);
 let (s, cx, cy) = sample(&kf, 400);
 assert!((s - 2.0).abs() < 1e-9);
 assert_eq!((cx, cy), (10.0, 20.0));
 let (s0, _, _) = sample(&kf, 0);
 assert_eq!(s0, 1.0);
 }

 #[test]
 fn cursor_smoothing_lags_toward_target() {
 let pts = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 0.0)];
 let s = smooth_cursor(&pts, 0.5);
 assert_eq!(s[0], (0.0, 0.0));
 assert_eq!(s[1], (5.0, 0.0));
 assert_eq!(s[2], (7.5, 0.0));
 }
}
