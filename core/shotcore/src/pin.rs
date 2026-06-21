//! Pin-to-screen model.
//!
//! SloerShot floats screenshots above all windows as frameless references. The
//! native shell owns the always-on-top window; this module owns the cross-
//! platform state: which shots are pinned, where they float, their opacity and
//! stacking, and whether they are locked. Serializable so a set of pins survives
//! a restart.

use crate::geometry::Rect;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;
use uuid::Uuid;

/// One floating, pinned screenshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pin {
    pub id: Uuid,
    pub image_path: String,
    /// Where the pin floats, in logical screen coordinates.
    pub frame: Rect,
    /// Opacity from 0.0 (transparent) to 1.0 (opaque).
    pub opacity: f32,
    /// Stacking order among pins; higher floats on top.
    pub z: i32,
    /// Locked pins ignore move and resize.
    pub locked: bool,
}

impl Pin {
    pub fn new(image_path: impl Into<String>, frame: Rect) -> Self {
        Self {
            id: Uuid::new_v4(),
            image_path: image_path.into(),
            frame,
            opacity: 1.0,
            z: 0,
            locked: false,
        }
    }
}

/// A collection of pinned shots with stacking and persistence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinBoard {
    pins: Vec<Pin>,
}

impl PinBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.pins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }

    pub fn pins(&self) -> &[Pin] {
        &self.pins
    }

    fn top_z(&self) -> i32 {
        self.pins.iter().map(|p| p.z).max().unwrap_or(0)
    }

    /// Add a pin on top of the stack, returning its id.
    pub fn add(&mut self, mut pin: Pin) -> Uuid {
        pin.z = self.top_z() + 1;
        let id = pin.id;
        self.pins.push(pin);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Pin> {
        self.pins.iter().find(|p| p.id == id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Pin> {
        self.pins.iter_mut().find(|p| p.id == id)
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Pin> {
        let idx = self.pins.iter().position(|p| p.id == id)?;
        Some(self.pins.remove(idx))
    }

    /// Move/resize a pin unless it is locked. Returns false if missing or locked.
    pub fn move_to(&mut self, id: Uuid, frame: Rect) -> bool {
        match self.get_mut(id) {
            Some(p) if !p.locked => {
                p.frame = frame;
                true
            }
            _ => false,
        }
    }

    /// Set opacity, clamped to the range 0.0 to 1.0.
    pub fn set_opacity(&mut self, id: Uuid, opacity: f32) -> bool {
        match self.get_mut(id) {
            Some(p) => {
                p.opacity = opacity.clamp(0.0, 1.0);
                true
            }
            None => false,
        }
    }

    pub fn set_locked(&mut self, id: Uuid, locked: bool) -> bool {
        match self.get_mut(id) {
            Some(p) => {
                p.locked = locked;
                true
            }
            None => false,
        }
    }

    /// Raise a pin above all others.
    pub fn raise(&mut self, id: Uuid) -> bool {
        let top = self.top_z();
        match self.get_mut(id) {
            Some(p) => {
                p.z = top + 1;
                true
            }
            None => false,
        }
    }

    /// Pins ordered back-to-front for drawing.
    pub fn ordered(&self) -> Vec<&Pin> {
        let mut v: Vec<&Pin> = self.pins.iter().collect();
        v.sort_by(|a, b| a.z.cmp(&b.z));
        v
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Option<PinBoard> {
        serde_json::from_str(s).ok()
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::write(path, self.to_json())
    }

    pub fn load(path: impl AsRef<Path>) -> io::Result<PinBoard> {
        let s = std::fs::read_to_string(path)?;
        PinBoard::from_json(&s)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid pin board json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(path: &str) -> Pin {
        Pin::new(path, Rect::new(0.0, 0.0, 100.0, 80.0))
    }

    #[test]
    fn add_assigns_increasing_z_and_returns_id() {
        let mut b = PinBoard::new();
        let a = b.add(pin("a.png"));
        let c = b.add(pin("b.png"));
        assert_eq!(b.len(), 2);
        assert!(b.get(a).unwrap().z < b.get(c).unwrap().z);
    }

    #[test]
    fn remove_works() {
        let mut b = PinBoard::new();
        let a = b.add(pin("a.png"));
        assert!(b.remove(a).is_some());
        assert!(b.remove(a).is_none());
        assert!(b.is_empty());
    }

    #[test]
    fn move_respects_lock() {
        let mut b = PinBoard::new();
        let a = b.add(pin("a.png"));
        assert!(b.move_to(a, Rect::new(10.0, 10.0, 50.0, 50.0)));
        assert_eq!(b.get(a).unwrap().frame, Rect::new(10.0, 10.0, 50.0, 50.0));
        b.set_locked(a, true);
        assert!(!b.move_to(a, Rect::new(99.0, 99.0, 10.0, 10.0)));
        assert_eq!(b.get(a).unwrap().frame, Rect::new(10.0, 10.0, 50.0, 50.0));
    }

    #[test]
    fn opacity_is_clamped() {
        let mut b = PinBoard::new();
        let a = b.add(pin("a.png"));
        b.set_opacity(a, 2.5);
        assert_eq!(b.get(a).unwrap().opacity, 1.0);
        b.set_opacity(a, -1.0);
        assert_eq!(b.get(a).unwrap().opacity, 0.0);
    }

    #[test]
    fn raise_brings_to_front() {
        let mut b = PinBoard::new();
        let a = b.add(pin("a.png"));
        let c = b.add(pin("b.png"));
        assert_eq!(
            b.ordered().iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![a, c]
        );
        b.raise(a);
        assert_eq!(
            b.ordered().iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![c, a]
        );
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pins.json");
        let mut b = PinBoard::new();
        b.add(pin("a.png"));
        b.add(pin("b.png"));
        b.save(&path).unwrap();
        let loaded = PinBoard::load(&path).unwrap();
        assert_eq!(loaded.len(), 2);
    }
}
