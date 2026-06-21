//! Device / browser mockup frames: wrap a screenshot in a browser, window, phone, or
//! laptop frame. This computes the outer canvas size and where the content sits; the
//! native layer paints the chrome.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frame {
 Browser,
 Window,
 Phone,
 Laptop,
}

impl Frame {
 /// Chrome insets (left, top, right, bottom) in pixels.
 pub fn insets(&self) -> (u32, u32, u32, u32) {
 match self {
 Frame::Browser => (1, 40, 1, 1),
 Frame::Window => (1, 28, 1, 1),
 Frame::Phone => (16, 60, 16, 60),
 Frame::Laptop => (24, 24, 24, 80),
 }
 }
}

/// A content image wrapped in a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mockup {
 pub frame: Frame,
 pub content_w: u32,
 pub content_h: u32,
}

impl Mockup {
 /// Outer canvas size including the frame chrome.
 pub fn outer_size(&self) -> (u32, u32) {
 let (l, t, r, b) = self.frame.insets();
 (self.content_w + l + r, self.content_h + t + b)
 }

 /// Top-left pixel where the content is placed inside the frame.
 pub fn content_origin(&self) -> (u32, u32) {
 let (l, t, _, _) = self.frame.insets();
 (l, t)
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn browser_chrome() {
 let m = Mockup { frame: Frame::Browser, content_w: 800, content_h: 600 };
 assert_eq!(m.outer_size(), (802, 641));
 assert_eq!(m.content_origin(), (1, 40));
 }

 #[test]
 fn phone_bezel() {
 let m = Mockup { frame: Frame::Phone, content_w: 300, content_h: 600 };
 assert_eq!(m.outer_size(), (332, 720));
 assert_eq!(m.content_origin(), (16, 60));
 }

 #[test]
 fn laptop_has_tall_base() {
 let m = Mockup { frame: Frame::Laptop, content_w: 100, content_h: 100 };
 let (_, h) = m.outer_size();
 assert_eq!(h, 100 + 24 + 80);
 }
}
