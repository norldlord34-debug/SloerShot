//! Screen-recording overlay + settings models: click highlight, keystroke display,
//! camera overlay, and capture settings (cursor, audio, do-not-disturb, countdown).
//!
//! Native recorders read these to draw overlays and configure the encoder; the data
//! and validation live here so Windows and macOS behave identically.
use crate::model::Color;
use serde::{Deserialize, Serialize};

/// A screen corner (and center) for positioning overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Corner {
 TopLeft,
 TopRight,
 BottomLeft,
 BottomRight,
 Center,
}

/// How mouse clicks are visualized on a recording.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClickHighlight {
 pub color: Color,
 pub radius: f32,
 pub filled: bool,
 pub animate: bool,
}

impl Default for ClickHighlight {
 fn default() -> Self {
 Self { color: Color::YELLOW.with_alpha(160), radius: 28.0, filled: true, animate: true }
 }
}

/// On-screen keystroke display configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeystrokeDisplay {
 pub position: Corner,
 pub dark: bool,
 /// Show only modifier combos (e.g. Cmd+S) instead of every key.
 pub command_keys_only: bool,
}

impl Default for KeystrokeDisplay {
 fn default() -> Self {
 Self { position: Corner::BottomRight, dark: true, command_keys_only: false }
 }
}

/// Camera (webcam) overlay shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraShape {
 Circle,
 Square,
 Rectangle,
}

/// Webcam overlay configuration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraOverlay {
 pub shape: CameraShape,
 pub position: Corner,
 /// Size as a fraction of the shorter screen edge, 0.05..=1.0.
 pub size_frac: f32,
 pub fullscreen: bool,
 pub mirrored: bool,
}

impl Default for CameraOverlay {
 fn default() -> Self {
 Self { shape: CameraShape::Circle, position: Corner::BottomLeft, size_frac: 0.18, fullscreen: false, mirrored: true }
 }
}

impl CameraOverlay {
 /// Pixel size of the overlay on a screen of the given dimensions.
 pub fn pixel_size(&self, screen_w: u32, screen_h: u32) -> u32 {
 if self.fullscreen {
 return screen_w.min(screen_h);
 }
 let short = screen_w.min(screen_h) as f32;
 (short * self.size_frac.clamp(0.05, 1.0)).round() as u32
 }
}

/// Recording capture settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RecordSettings {
 pub fps: u32,
 pub record_mic: bool,
 pub record_system_audio: bool,
 pub show_cursor: bool,
 pub hide_desktop_icons: bool,
 pub do_not_disturb: bool,
 /// Countdown before recording starts, in seconds.
 pub countdown_secs: u32,
}

impl Default for RecordSettings {
 fn default() -> Self {
 Self {
 fps: 30,
 record_mic: false,
 record_system_audio: false,
 show_cursor: true,
 hide_desktop_icons: false,
 do_not_disturb: true,
 countdown_secs: 3,
 }
 }
}

impl RecordSettings {
 /// Clamp to encoder-friendly values.
 pub fn normalized(mut self) -> Self {
 self.fps = self.fps.clamp(1, 120);
 self.countdown_secs = self.countdown_secs.min(10);
 self
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn defaults_are_sane() {
 let c = ClickHighlight::default();
 assert!(c.animate && c.filled);
 let k = KeystrokeDisplay::default();
 assert_eq!(k.position, Corner::BottomRight);
 let cam = CameraOverlay::default();
 assert_eq!(cam.shape, CameraShape::Circle);
 }

 #[test]
 fn camera_pixel_size() {
 let cam = CameraOverlay::default();
 assert_eq!(cam.pixel_size(1920, 1080), (1080.0 * 0.18f32).round() as u32);
 let full = CameraOverlay { fullscreen: true, ..CameraOverlay::default() };
 assert_eq!(full.pixel_size(1920, 1080), 1080);
 }

 #[test]
 fn settings_clamp() {
 let s = RecordSettings { fps: 1000, countdown_secs: 99, ..RecordSettings::default() }.normalized();
 assert_eq!(s.fps, 120);
 assert_eq!(s.countdown_secs, 10);
 }
}
