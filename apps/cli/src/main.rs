//! SloerShot demo CLI (`shotcli`).
//!
//! Exercises the shotcore pipeline end to end on the current machine: builds a
//! synthetic capture, annotates it through the undo engine, persists a
//! non-destructive sidecar, flattens an export, beautifies it, and records the
//! result in a searchable history. Also verifies backend-issued license tokens,
//! proving Rust/Node entitlement interop.

use clap::{Parser, Subcommand};
mod parity;

#[derive(Parser)]
#[command(name = "shotcli", version, about = "SloerShot shared-core demo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the SloerShot core version.
    Version,
    /// Run the full pipeline and write demo artifacts to a directory.
    Demo {
        #[arg(short, long, default_value = "assets/out")]
        out: String,
    },
    /// Showcase new features: code-card, contrast, mockup, zoom, captions, tags, svg, guide.
 Showcase,
 /// Exercise the newest parity modules (objdetect/snap, measure, align, table, mask, callout, and more).
 Parity,
 /// Verify a license token against a hex public key (proves backend interop).
    VerifyLicense {
        #[arg(long)]
        pubkey: String,
        #[arg(long)]
        token: String,
        #[arg(long, default_value_t = 0)]
        now: i64,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Version => {
            println!("SloerShot core {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Demo { out } => run_demo(&out),
        Commands::Showcase => run_showcase(),
 Commands::Parity => parity::run_parity(),
 Commands::VerifyLicense { pubkey, token, now } => verify_license(&pubkey, &token, now),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn verify_license(pubkey: &str, token: &str, now: i64) -> Result<(), Box<dyn std::error::Error>> {
    use shotcore::license::{verify, verifying_key_from_hex, LicenseError};
    let vk = verifying_key_from_hex(pubkey).map_err(|_| "invalid public key hex")?;
    match verify(&vk, token, now) {
        Ok(ent) => {
            println!(
                "VALID subject={} plan={:?} expires_at={}",
                ent.subject, ent.plan, ent.expires_at
            );
            Ok(())
        }
        Err(LicenseError::Expired) => Err("EXPIRED".into()),
        Err(e) => Err(format!("INVALID: {e}").into()),
    }
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Load a system font for text rendering, if one can be found.
fn load_system_font() -> Option<ab_glyph::FontVec> {
    let candidates = [
        "C:/Windows/Fonts/segoeui.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/calibri.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    for c in candidates {
        if let Ok(bytes) = std::fs::read(c) {
            if let Ok(font) = ab_glyph::FontVec::try_from_vec(bytes) {
                return Some(font);
            }
        }
    }
    None
}

fn run_demo(out_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    use shotcore::beautify::{beautify, Background, BeautifyOptions, GradientPreset};
    use shotcore::export;
    use shotcore::geometry::{Point, Rect};
    use shotcore::history::{HistoryEntry, HistoryStore};
    use shotcore::model::{Annotation, Color, Document, RedactStyle, ShapeKind, ShapeStyle};
    use shotcore::sidecar;
    use shotcore::undo::History;

    std::fs::create_dir_all(out_dir)?;
    let dir = std::path::Path::new(out_dir);

    let (w, h) = (800u32, 500u32);
    let mut base = image::RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = (x * 255 / w) as u8;
            let g = (y * 255 / h) as u8;
            base.put_pixel(x, y, image::Rgba([r, g, 160, 255]));
        }
    }
    let base_path = dir.join("base.png");
    base.save(&base_path)?;

    let mut history = History::new(Document::new(w, h).with_image_path("base.png"));
    history.edit(|d| {
        d.add(
            Annotation::new(ShapeKind::Rectangle {
                rect: Rect::new(60.0, 60.0, 240.0, 150.0),
                corner_radius: 12.0,
            })
            .with_style(ShapeStyle {
                stroke: Color::RED,
                stroke_width: 5.0,
                ..Default::default()
            }),
        );
    });
    history.edit(|d| {
        d.add(
            Annotation::new(ShapeKind::Arrow {
                from: Point::new(330.0, 120.0),
                to: Point::new(520.0, 240.0),
            })
            .with_style(ShapeStyle {
                stroke: Color::YELLOW,
                stroke_width: 6.0,
                ..Default::default()
            }),
        );
    });
    history.edit(|d| {
        d.add(
            Annotation::new(ShapeKind::Ellipse {
                rect: Rect::new(540.0, 280.0, 180.0, 120.0),
            })
            .with_style(ShapeStyle {
                stroke: Color::GREEN,
                fill: Some(Color::GREEN.with_alpha(60)),
                stroke_width: 4.0,
                ..Default::default()
            }),
        );
    });
    history.edit(|d| {
        let n = d.next_counter_number();
        d.add(
            Annotation::new(ShapeKind::Counter {
                center: Point::new(120.0, 360.0),
                radius: 22.0,
                number: n,
            })
            .with_style(ShapeStyle {
                stroke: Color::BLUE,
                ..Default::default()
            }),
        );
    });
    history.edit(|d| {
        d.add(
            Annotation::new(ShapeKind::Highlighter {
                rect: Rect::new(300.0, 400.0, 220.0, 40.0),
            })
            .with_style(ShapeStyle {
                stroke: Color::YELLOW,
                ..Default::default()
            }),
        );
    });
    history.edit(|d| {
        d.add(Annotation::new(ShapeKind::Redact {
            rect: Rect::new(600.0, 60.0, 150.0, 60.0),
            style: RedactStyle::Pixelate,
            strength: 14,
        }));
    });

    let count_full = history.current().len();
    history.undo();
    let after_undo = history.current().len();
    history.redo();
    let after_redo = history.current().len();
    let doc = history.current().clone();

    let sidecar_path = sidecar::save_beside_image(&doc, &base_path)?;
    let reloaded = sidecar::load_beside_image(&base_path)?;
    let reload_ok = reloaded == doc;

    let font = load_system_font();
    let mut doc2 = doc.clone();
    doc2.add(
        Annotation::new(ShapeKind::Text {
            position: Point::new(70.0, 235.0),
            content: "SloerShot".to_string(),
            font_size: 34.0,
        })
        .with_style(ShapeStyle {
            stroke: Color::WHITE,
            ..Default::default()
        }),
    );

    let annotated = export::compose(&base, &doc2, font.as_ref());
    let annotated_path = dir.join("annotated.png");
    annotated.save(&annotated_path)?;

    let opts = BeautifyOptions {
        background: Background::Preset(GradientPreset::Indigo),
        padding: 60,
        corner_radius: 20.0,
        ..Default::default()
    };
    let pretty = beautify(&annotated, &opts);
    let pretty_path = dir.join("beautified.png");
    pretty.save(&pretty_path)?;

    let mut store = HistoryStore::new();
    store.upsert(
        HistoryEntry::new(
            annotated_path.to_string_lossy().to_string(),
            now_unix(),
            annotated.width(),
            annotated.height(),
        )
        .with_ocr_text("SloerShot demo capture")
        .with_tags(vec!["demo".to_string(), "screenshot".to_string()]),
    );
    let history_path = dir.join("history.json");
    store.save(&history_path)?;
    let hits = store.search("sloershot").len();

    // Record a short synthetic animation (a marker sweeping across the capture) as a GIF.
    use shotcore::video::Recording;
    let gif_frames = 10u32;
    let mut recording = Recording::new(10);
    for i in 0..gif_frames {
        let mut fdoc = Document::new(w, h);
        let fx = 40.0 + (i as f64) * ((w as f64 - 120.0) / (gif_frames as f64 - 1.0));
        fdoc.add(
            Annotation::new(ShapeKind::Rectangle {
                rect: Rect::new(fx, 200.0, 80.0, 80.0),
                corner_radius: 12.0,
            })
            .with_style(ShapeStyle {
                stroke: Color::WHITE,
                fill: Some(Color::RED.with_alpha(180)),
                stroke_width: 4.0,
                ..Default::default()
            }),
        );
        let composed = export::compose(&base, &fdoc, None);
        recording.push(image::imageops::resize(
            &composed,
            400,
            250,
            image::imageops::FilterType::Triangle,
        ));
    }
    let gif_path = dir.join("demo.gif");
    recording.save_gif(&gif_path)?;

    println!("SloerShot demo complete. Artifacts in {}", dir.display());
    println!("- base: {} ({}x{})", base_path.display(), w, h);
    println!(
        "- gif: {} ({} frames @ {} fps, {} ms)",
        gif_path.display(),
        recording.len(),
        recording.fps,
        recording.duration_ms()
    );
    println!(
        "- sidecar: {} ({} annotations, reload_ok={})",
        sidecar_path.display(),
        reloaded.len(),
        reload_ok
    );
    println!("- annotated: {}", annotated_path.display());
    println!(
        "- beautified: {} ({}x{})",
        pretty_path.display(),
        pretty.width(),
        pretty.height()
    );
    println!(
        "- history: {} (search sloershot -> {} hit(s))",
        history_path.display(),
        hits
    );
    println!(
        "- undo/redo: full={} after_undo={} after_redo={}",
        count_full, after_undo, after_redo
    );
    println!(
        "- font: {}",
        if font.is_some() {
            "loaded (text + counter numbers rendered)"
        } else {
            "none found (text skipped)"
        }
    );
    Ok(())
}

fn run_showcase() -> Result<(), Box<dyn std::error::Error>> {
 let mut card = shotcore::codeshot::CodeCard::default();
 card.code = "fn main() {\n println!();\n}".to_string();
 let (cw, ch) = card.estimated_size(8.0, 18.0);
 println!("- code-card size: {}x{}", cw, ch);

 let ratio = shotcore::contrast::contrast_ratio(shotcore::model::Color::BLACK, shotcore::model::Color::WHITE);
 println!("- contrast black/white: {:.1} ({:?})", ratio, shotcore::contrast::rate(ratio, false));

 let m = shotcore::mockup::Mockup { frame: shotcore::mockup::Frame::Browser, content_w: 800, content_h: 600 };
 let (ow, oh) = m.outer_size();
 println!("- browser mockup outer: {}x{}", ow, oh);

 let kf = shotcore::zoompan::auto_zoom_keyframes(&[shotcore::zoompan::ClickEvent { t_ms: 1000, x: 50.0, y: 60.0 }], 2.0, 300, 800);
 println!("- auto-zoom keyframes: {}", kf.len());

 let srt = shotcore::captions::to_srt(&[shotcore::captions::Segment { start_ms: 0, end_ms: 1500, text: "Hello".to_string() }]);
 println!("- srt first cue: {}", srt.lines().nth(1).unwrap_or(""));

 let tags = shotcore::autotag::extract_tags("Invoice invoice total due now", 3);
 println!("- auto tags: {:?}", tags);

 let mut doc = shotcore::model::Document::new(200, 120);
 doc.add(shotcore::model::Annotation::new(shotcore::model::ShapeKind::Rectangle {
 rect: shotcore::geometry::Rect::new(10.0, 10.0, 80.0, 40.0),
 corner_radius: 6.0,
 }));
 println!("- svg export bytes: {}", shotcore::svgexport::to_svg(&doc).len());

 let mut g = shotcore::guide::Guide::new("Quick guide");
 g.add_step("s1.png", "Open the menu");
 g.add_step("s2.png", "Click Export");
 println!("- guide markdown bytes: {}", g.to_markdown().len());

 println!("showcase complete");
 Ok(())
}
