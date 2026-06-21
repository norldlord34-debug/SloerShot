//! Quick Access Overlay model (CleanShot): the post-capture HUD shown in a screen corner
//! for viewing/annotating/sharing the latest capture. Holds position, size, auto-close
//! timing, a bounded stack of shown items, and a recently-closed stack for restore. The
//! native shell draws the HUD; all behavior + bookkeeping live here and are unit tested.
use crate::record::Corner;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlaySize {
 Small,
 Medium,
 Large,
}
impl OverlaySize {
 /// Thumbnail edge length in pixels.
 pub fn pixels(&self) -> u32 {
 match self {
 OverlaySize::Small => 140,
 OverlaySize::Medium => 200,
 OverlaySize::Large => 280,
 }
 }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayItem {
 pub id: String,
 pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickAccessOverlay {
 pub corner: Corner,
 pub size: OverlaySize,
 /// Seconds of inactivity before auto-hiding; 0 = never auto-close.
 pub auto_close_secs: u32,
 pub visible: bool,
 pub items: Vec<OverlayItem>,
 pub closed: Vec<OverlayItem>,
 pub max_items: usize,
}
impl Default for QuickAccessOverlay {
 fn default() -> Self {
 Self {
 corner: Corner::BottomRight,
 size: OverlaySize::Medium,
 auto_close_secs: 8,
 visible: false,
 items: Vec::new(),
 closed: Vec::new(),
 max_items: 5,
 }
 }
}
impl QuickAccessOverlay {
 /// Show a new capture in the overlay (newest first, de-duplicated, stack capped).
 pub fn push(&mut self, id: impl Into<String>, created_at: i64) {
 let id = id.into();
 self.items.retain(|it| it.id != id);
 self.items.insert(0, OverlayItem { id, created_at });
 while self.items.len() > self.max_items.max(1) {
 self.items.pop();
 }
 self.visible = true;
 }
 /// Close the newest item, moving it to the recently-closed stack.
 pub fn close_top(&mut self) -> Option<OverlayItem> {
 if self.items.is_empty() {
 return None;
 }
 let it = self.items.remove(0);
 self.closed.insert(0, it.clone());
 if self.items.is_empty() {
 self.visible = false;
 }
 Some(it)
 }
 /// Restore the most recently closed item back into the overlay.
 pub fn restore_recent(&mut self) -> bool {
 if let Some(it) = self.closed.first().cloned() {
 self.closed.remove(0);
 self.items.insert(0, it);
 self.visible = true;
 true
 } else {
 false
 }
 }
 pub fn hide(&mut self) {
 self.visible = false;
 }
 /// Whether the overlay should auto-hide given the current time and last interaction.
 pub fn should_auto_close(&self, now: i64, last_interaction: i64) -> bool {
 self.visible && self.auto_close_secs > 0 && now - last_interaction >= self.auto_close_secs as i64
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn push_caps_and_dedupes() {
 let mut o = QuickAccessOverlay { max_items: 2, ..Default::default() };
 o.push("a", 1);
 o.push("b", 2);
 o.push("a", 3);
 assert_eq!(o.items.len(), 2);
 assert_eq!(o.items[0].id, "a");
 assert!(o.visible);
 }

 #[test]
 fn close_and_restore() {
 let mut o = QuickAccessOverlay::default();
 o.push("x", 1);
 let closed = o.close_top().unwrap();
 assert_eq!(closed.id, "x");
 assert!(!o.visible);
 assert!(o.restore_recent());
 assert_eq!(o.items[0].id, "x");
 assert!(o.visible);
 assert!(!o.restore_recent());
 }

 #[test]
 fn auto_close_and_size() {
 let o = QuickAccessOverlay { visible: true, auto_close_secs: 5, ..Default::default() };
 assert!(o.should_auto_close(110, 100));
 assert!(!o.should_auto_close(103, 100));
 assert_eq!(OverlaySize::Large.pixels(), 280);
 }
}
