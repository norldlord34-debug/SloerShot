//! Video editor model: trim, resolution scaling, audio (mute/mono/volume), GIF export.
//!
//! Pure parameter math mirroring the CleanShot Video Editor. Native code reads these
//! values to drive the platform encoder (Media Foundation / AVFoundation); the math
//! that decides output duration and dimensions lives here and is unit tested.
use serde::{Deserialize, Serialize};

/// A non-destructive set of edits applied to a source recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoEdit {
 /// Trim window in milliseconds [start, end). end == 0 means to the source end.
 pub trim_start_ms: u64,
 pub trim_end_ms: u64,
 /// Output resolution scale in 0.0..=1.0 of the source (1.0 = original).
 pub scale: f32,
 pub mute: bool,
 /// Downmix stereo to mono.
 pub mono: bool,
 /// Linear gain in 0.0..=2.0 applied to audio (1.0 = unchanged).
 pub volume: f32,
}

impl Default for VideoEdit {
 fn default() -> Self {
 Self {
 trim_start_ms: 0,
 trim_end_ms: 0,
 scale: 1.0,
 mute: false,
 mono: false,
 volume: 1.0,
 }
 }
}

impl VideoEdit {
 /// Output duration in ms given the source duration, honoring the trim window.
 pub fn output_duration_ms(&self, source_ms: u64) -> u64 {
 let end = if self.trim_end_ms == 0 || self.trim_end_ms > source_ms {
 source_ms
 } else {
 self.trim_end_ms
 };
 end.saturating_sub(self.trim_start_ms.min(end))
 }

 /// Output pixel size after scaling, each dimension at least 1 and even.
 pub fn output_dimensions(&self, w: u32, h: u32) -> (u32, u32) {
 let s = self.scale.clamp(0.05, 1.0);
 let ow = ((w as f32 * s).round() as u32).max(1);
 let oh = ((h as f32 * s).round() as u32).max(1);
 // encoders prefer even dimensions
 ((ow - (ow & 1)).max(2), (oh - (oh & 1)).max(2))
 }

 /// Effective per-sample gain after mute/volume.
 pub fn effective_gain(&self) -> f32 {
 if self.mute {
 0.0
 } else {
 self.volume.clamp(0.0, 2.0)
 }
 }

 /// Number of audio channels after downmix.
 pub fn output_channels(&self, source_channels: u16) -> u16 {
 if self.mute {
 0
 } else if self.mono {
 1.min(source_channels).max(if source_channels > 0 { 1 } else { 0 })
 } else {
 source_channels
 }
 }
}

/// GIF export options derived from a clip.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GifExport {
 pub fps: u32,
 pub max_width: u32,
 pub looping: bool,
}

impl Default for GifExport {
 fn default() -> Self {
 Self { fps: 15, max_width: 720, looping: true }
 }
}

impl GifExport {
 /// Total frame count for a clip of the given duration at the chosen fps.
 pub fn frame_count(&self, duration_ms: u64) -> u64 {
 let fps = self.fps.max(1) as u64;
 (duration_ms * fps + 999) / 1000
 }

 /// Scaled width/height capped to max_width, preserving aspect.
 pub fn scaled_size(&self, w: u32, h: u32) -> (u32, u32) {
 if w <= self.max_width || w == 0 {
 return (w, h);
 }
 let ratio = self.max_width as f64 / w as f64;
 (self.max_width, ((h as f64 * ratio).round() as u32).max(1))
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn trim_duration() {
 let mut e = VideoEdit::default();
 assert_eq!(e.output_duration_ms(10_000), 10_000);
 e.trim_start_ms = 2_000;
 e.trim_end_ms = 8_000;
 assert_eq!(e.output_duration_ms(10_000), 6_000);
 e.trim_end_ms = 99_000;
 assert_eq!(e.output_duration_ms(10_000), 8_000);
 }

 #[test]
 fn scaling_dimensions() {
 let mut e = VideoEdit::default();
 e.scale = 0.5;
 assert_eq!(e.output_dimensions(1920, 1080), (960, 540));
 }

 #[test]
 fn audio_gain_and_channels() {
 let mut e = VideoEdit::default();
 assert_eq!(e.effective_gain(), 1.0);
 assert_eq!(e.output_channels(2), 2);
 e.mono = true;
 assert_eq!(e.output_channels(2), 1);
 e.mute = true;
 assert_eq!(e.effective_gain(), 0.0);
 assert_eq!(e.output_channels(2), 0);
 }

 #[test]
 fn gif_frames_and_scale() {
 let g = GifExport::default();
 assert_eq!(g.frame_count(1000), 15);
 assert_eq!(g.scaled_size(1440, 900), (720, 450));
 assert_eq!(g.scaled_size(600, 400), (600, 400));
 }
}
