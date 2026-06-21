//! Capture/annotation presets: a named bundle of after-capture actions, output
//! format, and beautify choice that can be applied with one shortcut (repeatable
//! presets that pro users ask for).
use serde::{Deserialize, Serialize};

/// Actions to run automatically after a capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AfterCapture {
 pub copy_to_clipboard: bool,
 pub save_to_disk: bool,
 pub open_annotate: bool,
 pub upload: bool,
 pub pin: bool,
}

impl AfterCapture {
 /// The enabled actions as stable identifiers, in execution order.
 pub fn actions(&self) -> Vec<&'static str> {
 let mut v = Vec::new();
 if self.copy_to_clipboard { v.push("copy"); }
 if self.save_to_disk { v.push("save"); }
 if self.open_annotate { v.push("annotate"); }
 if self.upload { v.push("upload"); }
 if self.pin { v.push("pin"); }
 v
 }
}

/// A reusable capture preset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
 pub name: String,
 pub after: AfterCapture,
 /// Output format extension, e.g. png or jpg.
 pub format: String,
 /// Optional beautify preset name to apply.
 pub beautify: Option<String>,
 /// Hold Shift while capturing to bypass this preset.
 pub bypass_with_shift: bool,
}

impl Default for Preset {
 fn default() -> Self {
 Self {
 name: "Default".to_string(),
 after: AfterCapture { copy_to_clipboard: true, ..AfterCapture::default() },
 format: "png".to_string(),
 beautify: None,
 bypass_with_shift: true,
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn default_copies_to_clipboard() {
 let p = Preset::default();
 assert_eq!(p.after.actions(), vec!["copy"]);
 assert_eq!(p.format, "png");
 }

 #[test]
 fn actions_are_ordered() {
 let a = AfterCapture { copy_to_clipboard: true, save_to_disk: true, open_annotate: false, upload: true, pin: false };
 assert_eq!(a.actions(), vec!["copy", "save", "upload"]);
 }

 #[test]
 fn empty_preset_has_no_actions() {
 assert!(AfterCapture::default().actions().is_empty());
 }
}
