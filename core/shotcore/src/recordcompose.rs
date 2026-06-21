//! Real recording-engine compositing + scheduling. Native recorders grab raw frames
//! and call these to stamp click rings, a cursor highlight, and keystroke/camera
//! overlays onto each RGBA frame, and to compute frame timestamps and the elapsed
//! time label. Pure pixel math so Windows and macOS produce identical recordings.
use crate::geometry::Rect;
use crate::model::Color;
use crate::record::{CameraOverlay, Corner, KeystrokeDisplay};
use image::{Rgba, RgbaImage};

fn blend_px(img: &mut RgbaImage, x: i32, y: i32, color: Color, alpha: f32) {
 if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
 return;
 }
 let a = alpha.clamp(0.0, 1.0);
 let p = img.get_pixel(x as u32, y as u32).0;
 let mix = |s: u8, d: u8| ((s as f32 * a) + (d as f32 * (1.0 - a))).round() as u8;
 img.put_pixel(
 x as u32,
 y as u32,
 Rgba([mix(color.r, p[0]), mix(color.g, p[1]), mix(color.b, p[2]), 255]),
 );
}

/// Draw a click indicator centered at (cx, cy). progress 0..1 drives the expand+fade
/// animation when the highlight is animated; a static ring/disc otherwise.
pub fn draw_click(img: &mut RgbaImage, cx: f64, cy: f64, hl: &crate::record::ClickHighlight, progress: f32) {
 let prog = progress.clamp(0.0, 1.0);
 let radius = if hl.animate { hl.radius * (0.4 + 0.6 * prog) } else { hl.radius } as f64;
 let base_alpha = hl.color.a as f32 / 255.0;
 let alpha = if hl.animate { base_alpha * (1.0 - prog) } else { base_alpha };
 let ring = if hl.filled { radius } else { 3.0 };
 let inner = (radius - ring).max(0.0);
 let (r2, i2) = (radius * radius, inner * inner);
 let x0 = (cx - radius).floor() as i32;
 let x1 = (cx + radius).ceil() as i32;
 let y0 = (cy - radius).floor() as i32;
 let y1 = (cy + radius).ceil() as i32;
 for y in y0..=y1 {
 for x in x0..=x1 {
 let dx = x as f64 + 0.5 - cx;
 let dy = y as f64 + 0.5 - cy;
 let d2 = dx * dx + dy * dy;
 if d2 <= r2 && d2 >= i2 {
 blend_px(img, x, y, hl.color, alpha);
 }
 }
 }
}

/// Soft filled circle drawn under the cursor (the show-cursor highlight).
pub fn draw_cursor_highlight(img: &mut RgbaImage, cx: f64, cy: f64, radius: f64, color: Color) {
 let r2 = radius * radius;
 let base = color.a as f32 / 255.0;
 let x0 = (cx - radius).floor() as i32;
 let x1 = (cx + radius).ceil() as i32;
 let y0 = (cy - radius).floor() as i32;
 let y1 = (cy + radius).ceil() as i32;
 for y in y0..=y1 {
 for x in x0..=x1 {
 let dx = x as f64 + 0.5 - cx;
 let dy = y as f64 + 0.5 - cy;
 let d2 = dx * dx + dy * dy;
 if d2 <= r2 {
 let falloff = 1.0 - (d2 / r2) as f32;
 blend_px(img, x, y, color, base * falloff);
 }
 }
 }
}

/// Fill a rectangle (used for the keystroke/HUD bar background).
pub fn draw_filled_rect(img: &mut RgbaImage, rect: Rect, color: Color, alpha: f32) {
 let x0 = rect.x.floor().max(0.0) as i32;
 let y0 = rect.y.floor().max(0.0) as i32;
 let x1 = (rect.x + rect.w).ceil() as i32;
 let y1 = (rect.y + rect.h).ceil() as i32;
 for y in y0..y1 {
 for x in x0..x1 {
 blend_px(img, x, y, color, alpha);
 }
 }
}

fn corner_rect(corner: Corner, frame_w: u32, frame_h: u32, w: f64, h: f64, margin: f64) -> Rect {
 let (fw, fh) = (frame_w as f64, frame_h as f64);
 let (x, y) = match corner {
 Corner::TopLeft => (margin, margin),
 Corner::TopRight => (fw - w - margin, margin),
 Corner::BottomLeft => (margin, fh - h - margin),
 Corner::BottomRight => (fw - w - margin, fh - h - margin),
 Corner::Center => ((fw - w) / 2.0, (fh - h) / 2.0),
 };
 Rect::new(x.max(0.0), y.max(0.0), w, h)
}

/// Placement rect for the webcam overlay on a frame.
pub fn camera_rect(cam: &CameraOverlay, frame_w: u32, frame_h: u32) -> Rect {
 if cam.fullscreen {
 return Rect::new(0.0, 0.0, frame_w as f64, frame_h as f64);
 }
 let size = cam.pixel_size(frame_w, frame_h) as f64;
 let (w, h) = match cam.shape {
 crate::record::CameraShape::Rectangle => (size * 1.6, size),
 _ => (size, size),
 };
 corner_rect(cam.position, frame_w, frame_h, w, h, 24.0)
}

/// Placement rect for the keystroke HUD bar, sized to the label length.
pub fn keystroke_bar_rect(disp: &KeystrokeDisplay, frame_w: u32, frame_h: u32, text_len: usize) -> Rect {
 let w = (text_len.max(1) as f64) * 18.0 + 48.0;
 corner_rect(disp.position, frame_w, frame_h, w, 56.0, 32.0)
}

/// Timestamps (ms) of each captured frame for a target fps over a duration.
pub fn frame_timestamps(fps: u32, duration_ms: u64) -> Vec<u64> {
 let mut v = Vec::new();
 if fps == 0 {
 return v;
 }
 let mut i: u64 = 0;
 loop {
 let t = i * 1000 / fps as u64;
 if t >= duration_ms {
 break;
 }
 v.push(t);
 i += 1;
 }
 v
}

/// Number of frames captured for a target fps over a duration.
pub fn frame_count(fps: u32, duration_ms: u64) -> usize {
 frame_timestamps(fps, duration_ms).len()
}

/// Format an elapsed time in ms as the menu-bar recording label (m:ss).
pub fn format_elapsed(ms: u64) -> String {
 let secs = ms / 1000;
 format!("{}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::record::{CameraOverlay, ClickHighlight, KeystrokeDisplay};

 fn white(w: u32, h: u32) -> RgbaImage {
 RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]))
 }

 #[test]
 fn click_marks_the_center() {
 let mut img = white(100, 100);
 draw_click(&mut img, 50.0, 50.0, &ClickHighlight::default(), 0.0);
 assert_ne!(img.get_pixel(50, 50).0, [255, 255, 255, 255]);
 assert_eq!(img.get_pixel(0, 0).0, [255, 255, 255, 255]);
 }

 #[test]
 fn cursor_and_rect_composite() {
 let mut img = white(60, 60);
 draw_cursor_highlight(&mut img, 30.0, 30.0, 10.0, Color::rgba(255, 0, 0, 200));
 assert_ne!(img.get_pixel(30, 30).0, [255, 255, 255, 255]);
 let mut img2 = white(60, 60);
 draw_filled_rect(&mut img2, Rect::new(5.0, 5.0, 20.0, 10.0), Color::rgba(0, 0, 0, 255), 1.0);
 assert_eq!(img2.get_pixel(6, 6).0, [0, 0, 0, 255]);
 assert_eq!(img2.get_pixel(40, 40).0, [255, 255, 255, 255]);
 }

 #[test]
 fn schedule_and_label() {
 let ts = frame_timestamps(30, 1000);
 assert_eq!(ts.len(), 30);
 assert_eq!(ts[0], 0);
 assert_eq!(frame_count(0, 1000), 0);
 assert_eq!(format_elapsed(65_000), "1:05");
 assert_eq!(format_elapsed(3_000), "0:03");
 }

 #[test]
 fn overlay_placement() {
 let cam = CameraOverlay::default();
 let r = camera_rect(&cam, 1920, 1080);
 assert!(r.x < 100.0 && r.y > 800.0);
 let short = keystroke_bar_rect(&KeystrokeDisplay::default(), 1920, 1080, 4);
 let long = keystroke_bar_rect(&KeystrokeDisplay::default(), 1920, 1080, 20);
 assert!(long.w > short.w);
 }
}
