//! Aggregate application settings (CleanShot highly-customizable Settings). Typed, serde,
//! with sane defaults + normalization. References the default capture mode, save format,
//! after-capture actions, recording settings, and the Quick Access Overlay. The native
//! settings UI reads/writes this; validation lives here.
use crate::overlay::QuickAccessOverlay;
use crate::record::RecordSettings;
use crate::session::CaptureMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
 pub default_capture_mode: CaptureMode,
 pub save_format: String,
 pub after_capture_copy: bool,
 pub after_capture_save: bool,
 pub after_capture_open_editor: bool,
 pub capture_dir: String,
 pub recording: RecordSettings,
 pub overlay: QuickAccessOverlay,
 pub theme_dark: bool,
 pub capture_hotkey: String,
}
impl Default for Settings {
 fn default() -> Self {
 Self {
 default_capture_mode: CaptureMode::Area,
 save_format: String::from("png"),
 after_capture_copy: true,
 after_capture_save: true,
 after_capture_open_editor: false,
 capture_dir: String::from("~/Pictures/SloerShot"),
 recording: RecordSettings::default(),
 overlay: QuickAccessOverlay::default(),
 theme_dark: false,
 capture_hotkey: String::from("Ctrl+Shift+4"),
 }
 }
}
impl Settings {
 /// Clamp to valid values: known save format, encoder-friendly recording.
 pub fn normalized(mut self) -> Self {
 self.recording = self.recording.normalized();
 let f = self.save_format.to_lowercase();
 let ok = ["png", "jpg", "jpeg", "bmp", "tiff"];
 self.save_format = if ok.contains(&f.as_str()) { f } else { String::from("png") };
 self
 }
 pub fn to_json(&self) -> String {
 serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"))
 }
 pub fn from_json(s: &str) -> Option<Settings> {
 serde_json::from_str(s).ok()
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn defaults_and_normalize() {
 let s = Settings::default();
 assert_eq!(s.save_format, "png");
 assert!(s.after_capture_copy);
 let bad = Settings { save_format: String::from("webp"), ..Settings::default() }.normalized();
 assert_eq!(bad.save_format, "png");
 let good = Settings { save_format: String::from("JPG"), ..Settings::default() }.normalized();
 assert_eq!(good.save_format, "jpg");
 }

 #[test]
 fn json_round_trip() {
 let s = Settings::default();
 let j = s.to_json();
 let back = Settings::from_json(&j).unwrap();
 assert_eq!(s, back);
 assert!(Settings::from_json("not json").is_none());
 }
}
