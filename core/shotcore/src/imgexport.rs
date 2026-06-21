//! Export helpers: output formats, export settings (sRGB, JPEG quality, Retina
//! downscaling), and file-name templating with tokens and auto-increment.
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Supported still-image output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureFormat {
 Png,
 Jpeg,
 Bmp,
 Tiff,
}

impl CaptureFormat {
 pub fn extension(&self) -> &'static str {
 match self {
 CaptureFormat::Png => "png",
 CaptureFormat::Jpeg => "jpg",
 CaptureFormat::Bmp => "bmp",
 CaptureFormat::Tiff => "tiff",
 }
 }

 pub fn from_extension(ext: &str) -> Option<CaptureFormat> {
 match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
 "png" => Some(CaptureFormat::Png),
 "jpg" | "jpeg" => Some(CaptureFormat::Jpeg),
 "bmp" => Some(CaptureFormat::Bmp),
 "tif" | "tiff" => Some(CaptureFormat::Tiff),
 _ => None,
 }
 }

 /// JPEG cannot carry alpha; the rest can.
 pub fn supports_alpha(&self) -> bool {
 !matches!(self, CaptureFormat::Jpeg)
 }
}

/// Export settings mirroring the competition options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSettings {
 pub format: CaptureFormat,
 /// Convert/tag output to the sRGB color profile.
 pub srgb: bool,
 /// JPEG quality 1..=100 (ignored for other formats).
 pub jpeg_quality: u8,
 /// Downscale a Retina capture to 1x on export.
 pub scale_retina_to_1x: bool,
}

impl Default for ExportSettings {
 fn default() -> Self {
 Self { format: CaptureFormat::Png, srgb: true, jpeg_quality: 90, scale_retina_to_1x: false }
 }
}

impl ExportSettings {
 pub fn normalized(mut self) -> Self {
 self.jpeg_quality = self.jpeg_quality.clamp(1, 100);
 self
 }
}

/// Downscale a Retina image by `scale_factor` (e.g. 2.0 halves it). 1.0 is a no-op clone.
pub fn scale_to_1x(img: &RgbaImage, scale_factor: f32) -> RgbaImage {
 let sf = scale_factor.max(1.0);
 if (sf - 1.0).abs() < f32::EPSILON {
 return img.clone();
 }
 let w = ((img.width() as f32) / sf).round().max(1.0) as u32;
 let h = ((img.height() as f32) / sf).round().max(1.0) as u32;
 image::imageops::resize(img, w, h, image::imageops::FilterType::Lanczos3)
}

/// Expand {token} placeholders in a file-name template from a variable map.
/// Unknown tokens are left untouched so nothing is silently dropped.
pub fn expand_filename(template: &str, vars: &BTreeMap<String, String>) -> String {
 let mut out = template.to_string();
 for (k, v) in vars {
 out = out.replace(&format!("{{{}}}", k), v);
 }
 out
}

/// Zero-pad an auto-increment counter to `width` digits.
pub fn counter_token(n: u32, width: usize) -> String {
 format!("{:0>width$}", n, width = width)
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn format_extensions_and_alpha() {
 assert_eq!(CaptureFormat::Jpeg.extension(), "jpg");
 assert!(!CaptureFormat::Jpeg.supports_alpha());
 assert!(CaptureFormat::Png.supports_alpha());
 assert_eq!(CaptureFormat::from_extension(".JPEG"), Some(CaptureFormat::Jpeg));
 assert_eq!(CaptureFormat::from_extension("webp"), None);
 }

 #[test]
 fn settings_clamp_quality() {
 let s = ExportSettings { jpeg_quality: 200, ..ExportSettings::default() }.normalized();
 assert_eq!(s.jpeg_quality, 100);
 }

 #[test]
 fn retina_downscale_halves() {
 let img = RgbaImage::new(40, 20);
 let out = scale_to_1x(&img, 2.0);
 assert_eq!((out.width(), out.height()), (20, 10));
 let same = scale_to_1x(&img, 1.0);
 assert_eq!((same.width(), same.height()), (40, 20));
 }

 #[test]
 fn filename_templating() {
 let mut vars = BTreeMap::new();
 vars.insert("app".to_string(), "Safari".to_string());
 vars.insert("counter".to_string(), counter_token(7, 4));
 let name = expand_filename("Shot {app} {counter}.png", &vars);
 assert_eq!(name, "Shot Safari 0007.png");
 }
}
