//! Capture-session model: All-In-One mode (one shortcut, all modes, remembered last
//! selection, aspect lock) and the self-timer countdown.
use crate::crop::{constrain, AspectRatio};
use crate::geometry::Rect;
use serde::{Deserialize, Serialize};

/// The capture mode chosen within a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMode {
 Area,
 Window,
 Fullscreen,
}

/// All-In-One capture session state, persisted between captures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureSession {
 pub mode: CaptureMode,
 /// Last area selection, recalled to retake quickly.
 pub last_selection: Option<Rect>,
 pub aspect_lock: AspectRatio,
 /// Self-timer countdown in seconds before capture fires.
 pub self_timer_secs: u32,
}

impl Default for CaptureSession {
 fn default() -> Self {
 Self {
 mode: CaptureMode::Area,
 last_selection: None,
 aspect_lock: AspectRatio::Free,
 self_timer_secs: 0,
 }
 }
}

impl CaptureSession {
 /// Record a new selection (after applying any aspect lock).
 pub fn remember(&mut self, selection: Rect) {
 self.last_selection = Some(self.apply_lock(selection));
 }

 /// The selection to use now: the given one if present, else the remembered one,
 /// always passed through the aspect lock.
 pub fn effective(&self, proposed: Option<Rect>) -> Option<Rect> {
 proposed
 .or(self.last_selection)
 .map(|r| self.apply_lock(r))
 }

 fn apply_lock(&self, r: Rect) -> Rect {
 constrain(r, self.aspect_lock)
 }

 /// Clamp the self-timer to a sane range and report whether it is armed.
 pub fn arm_timer(&mut self, secs: u32) -> bool {
 self.self_timer_secs = secs.min(60);
 self.self_timer_secs > 0
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn remembers_last_selection() {
 let mut s = CaptureSession::default();
 assert!(s.effective(None).is_none());
 s.remember(Rect::new(0.0, 0.0, 100.0, 50.0));
 assert_eq!(s.effective(None), Some(Rect::new(0.0, 0.0, 100.0, 50.0)));
 }

 #[test]
 fn aspect_lock_applies_to_selection() {
 let mut s = CaptureSession::default();
 s.aspect_lock = AspectRatio::Square;
 s.remember(Rect::new(0.0, 0.0, 200.0, 100.0));
 let r = s.last_selection.unwrap();
 assert_eq!(r.w, r.h);
 }

 #[test]
 fn proposed_overrides_remembered() {
 let mut s = CaptureSession::default();
 s.remember(Rect::new(0.0, 0.0, 10.0, 10.0));
 let r = s.effective(Some(Rect::new(5.0, 5.0, 20.0, 20.0))).unwrap();
 assert_eq!(r, Rect::new(5.0, 5.0, 20.0, 20.0));
 }

 #[test]
 fn timer_arms_and_clamps() {
 let mut s = CaptureSession::default();
 assert!(!s.arm_timer(0));
 assert!(s.arm_timer(5));
 assert!(s.arm_timer(999));
 assert_eq!(s.self_timer_secs, 60);
 }
}
