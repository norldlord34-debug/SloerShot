//! End-to-end integration test across the public API: capture -> edit (via the
//! editor controller) -> smart auto-redaction -> sidecar round-trip -> export ->
//! beautify. Exercises many modules together the way a native app would.

use image::{Rgba, RgbaImage};
use shotcore::beautify::{beautify, Background, BeautifyOptions, GradientPreset};
use shotcore::detect::{auto_redact, Sensitive};
use shotcore::editor::{Editor, Tool};
use shotcore::export;
use shotcore::geometry::{Point, Rect};
use shotcore::model::{Color, RedactStyle, ShapeStyle};
use shotcore::ocr::{OcrLine, OcrResult, OcrWord};
use shotcore::sidecar;

#[test]
fn capture_edit_redact_export_beautify_pipeline() {
    let base = RgbaImage::from_pixel(200, 120, Rgba([240, 240, 240, 255]));

    let mut editor = Editor::new(200, 120);
    editor.set_tool(Tool::Rectangle);
    editor.set_style(ShapeStyle {
        fill: Some(Color::RED),
        stroke_width: 0.0,
        ..ShapeStyle::default()
    });
    editor.pointer_down(Point::new(20.0, 20.0));
    editor.pointer_drag(Point::new(80.0, 60.0));
    editor.pointer_up(Point::new(80.0, 60.0));
    assert_eq!(editor.document().len(), 1);

    let ocr = OcrResult::new(vec![OcrLine::from_words(vec![OcrWord {
        text: "bob@acme.io".to_string(),
        bbox: Rect::new(110.0, 80.0, 80.0, 16.0),
        confidence: 0.95,
    }])]);
    let mut doc = editor.document().clone();
    for ann in auto_redact(&ocr, &[Sensitive::Email], RedactStyle::Pixelate, 10) {
        doc.add(ann);
    }
    assert_eq!(doc.len(), 2);

    let json = sidecar::to_json(&doc).unwrap();
    assert_eq!(sidecar::from_json(&json).unwrap(), doc);

    let flat = export::compose(&base, &doc, None);
    assert_eq!(flat.dimensions(), (200, 120));
    assert_eq!(flat.get_pixel(50, 40), &Rgba([229, 57, 53, 255])); // Color::RED palette value

    let pretty = beautify(
        &flat,
        &BeautifyOptions {
            background: Background::Preset(GradientPreset::Ocean),
            padding: 24,
            corner_radius: 12.0,
            shadow: None,
        },
    );
    assert_eq!(pretty.width(), 248);
    assert_eq!(pretty.height(), 168);
}
