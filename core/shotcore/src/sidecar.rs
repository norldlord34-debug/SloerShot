//! Non-destructive sidecar persistence.
//!
//! Annotations live in a small JSON file next to the source image, for example
//! `screenshot.png` paired with `screenshot.png.sloershot.json`. The image bytes
//! are never modified; reopening the image reloads every editable annotation.

use crate::model::{Document, SCHEMA_VERSION};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Suffix appended to the image file name to form the sidecar path.
pub const SIDECAR_SUFFIX: &str = "sloershot.json";

#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("unsupported sidecar schema version {found}; this build supports {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
}

/// Compute the sidecar path that sits next to the given image path.
pub fn sidecar_path_for(image_path: impl AsRef<Path>) -> PathBuf {
    let p = image_path.as_ref();
    let mut name = p.file_name().map(|s| s.to_os_string()).unwrap_or_default();
    name.push(".");
    name.push(SIDECAR_SUFFIX);
    p.with_file_name(name)
}

/// Serialize a document to pretty JSON.
pub fn to_json(doc: &Document) -> Result<String, SidecarError> {
    Ok(serde_json::to_string_pretty(doc)?)
}

/// Deserialize a document from JSON, validating the schema version.
pub fn from_json(json: &str) -> Result<Document, SidecarError> {
    let doc: Document = serde_json::from_str(json)?;
    if doc.schema_version > SCHEMA_VERSION {
        return Err(SidecarError::UnsupportedSchema {
            found: doc.schema_version,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(doc)
}

/// Save the document to an explicit sidecar path.
pub fn save(doc: &Document, sidecar_path: impl AsRef<Path>) -> Result<(), SidecarError> {
    let json = to_json(doc)?;
    std::fs::write(sidecar_path, json)?;
    Ok(())
}

/// Load a document from an explicit sidecar path.
pub fn load(sidecar_path: impl AsRef<Path>) -> Result<Document, SidecarError> {
    let json = std::fs::read_to_string(sidecar_path)?;
    from_json(&json)
}

/// Save the document to the sidecar next to its image path; returns the sidecar path.
pub fn save_beside_image(
    doc: &Document,
    image_path: impl AsRef<Path>,
) -> Result<PathBuf, SidecarError> {
    let path = sidecar_path_for(image_path);
    save(doc, &path)?;
    Ok(path)
}

/// Load the document from the sidecar next to the given image path.
pub fn load_beside_image(image_path: impl AsRef<Path>) -> Result<Document, SidecarError> {
    load(sidecar_path_for(image_path))
}

/// Whether a sidecar exists next to the given image.
pub fn exists_beside_image(image_path: impl AsRef<Path>) -> bool {
    sidecar_path_for(image_path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point, Rect};
    use crate::model::{Annotation, ShapeKind};

    fn sample_doc() -> Document {
        let mut doc = Document::new(640, 480).with_image_path("shot.png");
        doc.add(Annotation::new(ShapeKind::Arrow {
            from: Point::new(10.0, 10.0),
            to: Point::new(100.0, 80.0),
        }));
        doc.add(Annotation::new(ShapeKind::Rectangle {
            rect: Rect::new(20.0, 20.0, 50.0, 40.0),
            corner_radius: 6.0,
        }));
        doc
    }

    #[test]
    fn sidecar_path_is_beside_image() {
        let p = sidecar_path_for("a/b/screenshot.png");
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(name, "screenshot.png.sloershot.json");
    }

    #[test]
    fn json_roundtrip_preserves_document() {
        let doc = sample_doc();
        let json = to_json(&doc).unwrap();
        let back = from_json(&json).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn save_and_load_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let side = dir.path().join("x.sloershot.json");
        let doc = sample_doc();
        save(&doc, &side).unwrap();
        let back = load(&side).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn save_and_load_beside_image() {
        let dir = tempfile::tempdir().unwrap();
        let img = dir.path().join("capture.png");
        let doc = sample_doc();
        assert!(!exists_beside_image(&img));
        let side = save_beside_image(&doc, &img).unwrap();
        assert!(side.exists());
        assert!(exists_beside_image(&img));
        let back = load_beside_image(&img).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn newer_schema_is_rejected() {
        let bad = r#"{"schema_version":999,"image_width":10,"image_height":10,"image_path":null,"annotations":[],"counter_seq":0}"#;
        match from_json(bad) {
            Err(SidecarError::UnsupportedSchema { found, supported }) => {
                assert_eq!(found, 999);
                assert_eq!(supported, SCHEMA_VERSION);
            }
            other => panic!("expected UnsupportedSchema, got {other:?}"),
        }
    }
}
