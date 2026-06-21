//! SloerShot shared core (`shotcore`).
//!
//! Pure-Rust, cross-platform logic powering the Windows and macOS apps:
//! capture geometry, a non-destructive annotation model, an export/compose
//! pipeline, background beautification, a searchable history index, an OCR
//! result model, scroll-capture stitching, and offline license validation.
//!
//! The native shells (WinUI 3 / SwiftUI) own capture and rendering surfaces;
//! every platform-independent concern lives here and is exposed over a C ABI.

#![allow(clippy::needless_range_loop)]

pub mod align;
pub mod beautify;
pub mod analyze;
pub mod autotag;
pub mod callout;
pub mod captions;
pub mod cloud;
pub mod codeshot;
pub mod combine;
pub mod contrast;
pub mod corners;
pub mod crop;
pub mod deskew;
pub mod detect;
pub mod docdetect;
pub mod edges;
pub mod ean13;
pub mod editor;
pub mod export;
pub mod ffi;
pub mod fx;
pub mod geometry;
pub mod guide;
pub mod history;
pub mod imgexport;
pub mod hough;
pub mod hit;
pub mod imagediff;
pub mod license;
pub mod magnifier;
pub mod mask;
pub mod measure;
pub mod mockup;
pub mod model;
pub mod objdetect;
pub mod ocr;
pub mod ocrflow;
pub mod overlay;
pub mod phash;
pub mod pdf;
pub mod perspective;
pub mod pin;
pub mod preset;
pub mod print;
pub mod qrcode;
pub mod sidecar;
pub mod segment;
pub mod sharpen;
pub mod stitch;
pub mod svgexport;
pub mod table;
pub mod translate;
pub mod undo;
pub mod zoompan;
pub mod urlscheme;
pub mod palette;
pub mod palettegen;
pub mod recognize;
pub mod record;
pub mod recordcompose;
pub mod session;
pub mod settings;
pub mod share;
pub mod smarthl;
pub mod smooth;
pub mod video;
pub mod videoedit;
pub mod whitebalance;
