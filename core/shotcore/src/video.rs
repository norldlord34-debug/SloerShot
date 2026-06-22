//! Video and GIF capture pipeline.
//!
//! Cross-platform recording lives here: frames captured by the native shells
//! (Windows.Graphics.Capture, ScreenCaptureKit) are pushed into a `Recording`
//! and encoded. Animated GIF is implemented in pure Rust. MP4 is produced by the
//! platform encoders behind the `VideoSink` trait (Media Foundation on Windows,
//! VideoToolbox on macOS), which consume the same frame stream.

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, RgbaImage};
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no frames to encode")]
    Empty,
}

/// A captured recording: a sequence of equal-size frames plus a target frame rate.
#[derive(Debug, Clone, Default)]
pub struct Recording {
    frames: Vec<RgbaImage>,
    pub fps: u32,
}

impl Recording {
    pub fn new(fps: u32) -> Self {
        Self {
            frames: Vec::new(),
            fps: fps.max(1),
        }
    }

    pub fn push(&mut self, frame: RgbaImage) {
        self.frames.push(frame);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn frames(&self) -> &[RgbaImage] {
        &self.frames
    }

    /// Per-frame delay in milliseconds derived from the frame rate.
    pub fn frame_delay_ms(&self) -> u32 {
        (1000.0 / self.fps.max(1) as f64).round().max(1.0) as u32
    }

    /// Total duration in milliseconds.
    pub fn duration_ms(&self) -> u32 {
        self.frame_delay_ms() * self.frames.len() as u32
    }

    /// Encode the recording as an animated, looping GIF to any writer.
    pub fn encode_gif<W: Write>(&self, writer: W) -> Result<(), VideoError> {
        encode_gif(&self.frames, self.fps, writer)
    }

    /// Encode and save the recording as a GIF file.
    pub fn save_gif(&self, path: impl AsRef<std::path::Path>) -> Result<(), VideoError> {
        let file = std::fs::File::create(path)?;
        self.encode_gif(std::io::BufWriter::new(file))
    }

    /// Feed every frame into a native sink (for example an MP4 encoder).
    pub fn drain_into(&self, sink: &mut dyn VideoSink) -> Result<(), VideoError> {
        for f in &self.frames {
            sink.push_frame(f)?;
        }
        Ok(())
    }
}

/// Encode a sequence of equal-size frames as an animated, infinitely looping GIF.
pub fn encode_gif<W: Write>(frames: &[RgbaImage], fps: u32, writer: W) -> Result<(), VideoError> {
    if frames.is_empty() {
        return Err(VideoError::Empty);
    }
    let delay_ms = (1000.0 / fps.max(1) as f64).round().max(1.0) as u32;
    let mut encoder = GifEncoder::new(writer);
    encoder.set_repeat(Repeat::Infinite)?;
    for f in frames {
        let frame = Frame::from_parts(f.clone(), 0, 0, Delay::from_numer_denom_ms(delay_ms, 1));
        encoder.encode_frame(frame)?;
    }
    Ok(())
}

/// Stream-encode GIF from image files on disk one at a time (bounded memory),
/// optionally downscaling each frame to max_width (0 = no limit).
pub fn encode_gif_from_files<W: Write>(paths: &[std::path::PathBuf], fps: u32, max_width: u32, writer: W) -> Result<(), VideoError> {
 if paths.is_empty() {
 return Err(VideoError::Empty);
 }
 let delay_ms = (1000.0 / fps.max(1) as f64).round().max(1.0) as u32;
 let mut encoder = GifEncoder::new(writer);
 encoder.set_repeat(Repeat::Infinite)?;
 let mut any = false;
 for p in paths {
 if let Ok(img) = image::open(p) {
 let mut rgba = img.to_rgba8();
 if max_width > 0 && rgba.width() > max_width {
 let h = ((rgba.height() as f64) * (max_width as f64) / (rgba.width() as f64)).round().max(1.0) as u32;
 rgba = image::imageops::resize(&rgba, max_width, h, image::imageops::FilterType::Triangle);
 }
 let frame = Frame::from_parts(rgba, 0, 0, Delay::from_numer_denom_ms(delay_ms, 1));
 encoder.encode_frame(frame)?;
 any = true;
 }
 }
 if !any {
 return Err(VideoError::Empty);
 }
 Ok(())
}
/// A sink native encoders implement to produce formats beyond GIF, such as MP4
/// via Media Foundation (Windows) or VideoToolbox (macOS). The core feeds frames;
/// the platform owns the codec.
pub trait VideoSink {
    fn push_frame(&mut self, frame: &RgbaImage) -> Result<(), VideoError>;
    fn finish(self: Box<Self>) -> Result<(), VideoError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{AnimationDecoder, Rgba};

    fn frame(w: u32, h: u32, c: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba(c))
    }

    #[test]
    fn recording_timing_math() {
        let mut r = Recording::new(10);
        assert_eq!(r.frame_delay_ms(), 100);
        r.push(frame(8, 8, [255, 0, 0, 255]));
        r.push(frame(8, 8, [0, 255, 0, 255]));
        assert_eq!(r.len(), 2);
        assert_eq!(r.duration_ms(), 200);
    }

    #[test]
    fn fps_is_clamped_to_at_least_one() {
        let r = Recording::new(0);
        assert_eq!(r.fps, 1);
        assert_eq!(r.frame_delay_ms(), 1000);
    }

    #[test]
    fn empty_recording_fails_to_encode() {
        let r = Recording::new(10);
        let mut buf = Vec::new();
        assert!(matches!(r.encode_gif(&mut buf), Err(VideoError::Empty)));
    }

    #[test]
    fn gif_roundtrip_preserves_frame_count_and_size() {
        let mut r = Recording::new(20);
        r.push(frame(16, 12, [200, 30, 30, 255]));
        r.push(frame(16, 12, [30, 200, 30, 255]));
        r.push(frame(16, 12, [30, 30, 200, 255]));
        let mut bytes = Vec::new();
        r.encode_gif(&mut bytes).unwrap();
        assert!(bytes.starts_with(b"GIF"));
        let decoder = GifEncoderRoundtrip::decode(&bytes);
        assert_eq!(decoder.0, 3);
        assert_eq!(decoder.1, (16, 12));
    }

    struct GifEncoderRoundtrip(usize, (u32, u32));
    impl GifEncoderRoundtrip {
        fn decode(bytes: &[u8]) -> Self {
            let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(bytes)).unwrap();
            let frames = decoder.into_frames().collect_frames().unwrap();
            let dims = frames[0].buffer().dimensions();
            GifEncoderRoundtrip(frames.len(), dims)
        }
    }
}
