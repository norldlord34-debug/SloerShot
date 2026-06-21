//! Parity showcase: exercises the newest CleanShot and PixelSnap parity modules
//! end to end so the whole shared-core surface is demonstrably callable from a
//! real binary. Run with: cargo run -p shotcli -- parity
use shotcore::geometry::{Point, Rect};
use shotcore::ocr::{OcrLine, OcrResult, OcrWord};

fn sample_ocr() -> OcrResult {
 let mk = |t: &str, x: f64, y: f64, ww: f64| OcrWord {
 text: t.to_string(),
 bbox: Rect::new(x, y, ww, 16.0),
 confidence: 0.95,
 };
 OcrResult::new(vec![
 OcrLine::from_words(vec![mk("Name", 10.0, 10.0, 50.0), mk("Total", 140.0, 10.0, 50.0)]),
 OcrLine::from_words(vec![mk("Invoice", 10.0, 40.0, 70.0), mk("42", 140.0, 40.0, 30.0)]),
 ])
}

pub fn run_parity() -> Result<(), Box<dyn std::error::Error>> {
 println!("== SloerShot parity showcase (newest modules) ==");

 let mut img = image::RgbaImage::from_pixel(40, 40, image::Rgba([255, 255, 255, 255]));
 for y in 10..30 {
 for x in 8..24 {
 img.put_pixel(x, y, image::Rgba([10, 10, 10, 255]));
 }
 }

 let snapped = shotcore::objdetect::snap_to_object(&img, Rect::new(0.0, 0.0, 40.0, 40.0), 16);
 println!("- snap_to_object -> x={} y={} w={} h={}", snapped.x as i64, snapped.y as i64, snapped.w as i64, snapped.h as i64);

 let dist = shotcore::measure::distance(Point::new(0.0, 0.0), Point::new(3.0, 4.0));
 let angle = shotcore::measure::angle_deg(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
 let guides = shotcore::measure::guides_from_rects(&[Rect::new(10.0, 10.0, 100.0, 50.0)]);
 let sp = shotcore::measure::snap_point(Point::new(11.0, 9.0), &guides, 3.0);
 println!("- measure -> distance={} angle={} snap=({},{})", dist as i64, angle as i64, sp.x as i64, sp.y as i64);

 let cr = shotcore::crop::constrain(Rect::new(0.0, 0.0, 320.0, 320.0), shotcore::crop::AspectRatio::R16x9);
 println!("- crop 16x9 -> {}x{} ({} presets)", cr.w as i64, cr.h as i64, shotcore::crop::AspectRatio::presets().len());

 let rects = [Rect::new(0.0, 0.0, 20.0, 20.0), Rect::new(50.0, 5.0, 20.0, 20.0), Rect::new(90.0, 12.0, 20.0, 20.0)];
 let aligned = shotcore::align::align(&rects, shotcore::align::Edge::Top);
 let ys: Vec<i64> = aligned.iter().map(|r| r.y as i64).collect();
 println!("- align Top -> ys={:?}", ys);

 let masked = shotcore::mask::apply_mask(&img, shotcore::mask::MaskShape::Ellipse);
 println!("- mask ellipse -> corner_alpha={} center_alpha={}", masked.get_pixel(0, 0)[3], masked.get_pixel(20, 20)[3]);

 let callout = shotcore::callout::Callout { bubble: Rect::new(100.0, 100.0, 120.0, 60.0), anchor: Point::new(60.0, 220.0), corner_radius: 8.0 };
 println!("- callout -> tail_side={:?} tail_points={}", callout.tail_side(), callout.tail_points(20.0).len());

 let ocr = sample_ocr();
 let grid = shotcore::table::detect_table(&ocr, 40.0);
 println!("- table -> {} rows x {} cols, csv_bytes={}", grid.len(), grid.first().map(|r| r.len()).unwrap_or(0), shotcore::table::to_csv(&grid).len());

 let hl = shotcore::smarthl::snap_highlight(&ocr, Rect::new(5.0, 5.0, 70.0, 22.0));
 println!("- smart highlight -> snapped={}", hl.is_some());

 let mut map = std::collections::HashMap::new();
 map.insert("Invoice".to_string(), "Factura".to_string());
 let tr = shotcore::translate::WordMapTranslator { map, source: "en".to_string() };
 let lines = shotcore::translate::translate_lines(&ocr, &tr, None, "es");
 println!("- translate -> {} lines, first={:?}", lines.len(), lines.first().map(|l| l.text.clone()).unwrap_or_default());

 let pts = [Point::new(0.0, 0.0), Point::new(10.0, 30.0), Point::new(40.0, 0.0)];
 let arc = shotcore::smooth::curved_arrow_path(Point::new(0.0, 0.0), Point::new(100.0, 0.0), 20.0, 12);
 println!("- smooth -> chaikin {} pts, curved arrow {} pts", shotcore::smooth::smooth_stroke(&pts, 2).len(), arc.len());

 let eye = shotcore::palette::eyedrop(img.as_raw(), img.width(), img.height(), 16, 20);
 println!("- palette -> {} default swatches, eyedrop_dark={}", shotcore::palette::Palette::defaults().len(), eye.is_some());

 let layout = shotcore::combine::stack_vertical(&[(100, 50), (80, 40)], 10);
 println!("- combine -> canvas {}x{} with {} placements", layout.canvas_w, layout.canvas_h, layout.placements.len());

 println!("- recognize -> mailto class={:?}, links_found={}", shotcore::recognize::classify("mailto:a@b.com"), shotcore::recognize::extract_links("see https://sloer.sh and https://x.io").len());

 if let Some(cmd) = shotcore::urlscheme::parse("sloershot://capture-area?x=10&y=20&width=300&height=200") {
 println!("- urlscheme -> action={:?} roundtrip={}", cmd.action_kind(), cmd.to_url());
 }

 let mut sess = shotcore::session::CaptureSession::default();
 sess.remember(Rect::new(4.0, 4.0, 200.0, 120.0));
 let armed = sess.arm_timer(3);
 println!("- session -> remembered={} timer_armed={}", sess.effective(None).is_some(), armed);

 let preset = shotcore::preset::Preset::default();
 println!("- preset -> {} after-capture actions", preset.after.actions().len());

 let settings = shotcore::imgexport::ExportSettings::default().normalized();
 let mut vars = std::collections::BTreeMap::new();
 vars.insert("counter".to_string(), shotcore::imgexport::counter_token(7, 4));
 println!("- imgexport -> fmt={:?} q={} name={}", settings.format, settings.jpeg_quality, shotcore::imgexport::expand_filename("shot-{counter}", &vars));

 println!("- print -> {} pages for a 2000px tall capture", shotcore::print::page_count(2000, 800, 40));

 let (qver, qgrid) = shotcore::qrcode::encode("https://sloershot.app").expect("qr encode");
 let qimg = shotcore::qrcode::render(&qgrid, 3, 4);
 let qdecoded = shotcore::qrcode::decode(&qimg).expect("qr decode");
 println!("- qrcode -> v{} {}x{}px decoded={}", qver, qimg.width(), qimg.height(), qdecoded);

 println!("- recordcompose -> elapsed(65s)={} frames@30fps/1s={}", shotcore::recordcompose::format_elapsed(65_000), shotcore::recordcompose::frame_count(30, 1000));

 let mag = image::RgbaImage::from_pixel(4, 4, image::Rgba([12, 34, 56, 255]));
 println!("- magnifier -> pixel(0,0)={}", shotcore::magnifier::pixel_hex(&mag, 0, 0).unwrap_or_default());

 let ocrres = shotcore::ocr::OcrResult::new(vec![shotcore::ocr::OcrLine::from_words(vec![shotcore::ocr::OcrWord { text: "Copy".to_string(), bbox: Rect::new(0.0, 0.0, 40.0, 16.0), confidence: 0.9 }])]);
 println!("- ocrflow -> single_line={}", shotcore::ocrflow::to_single_line(&ocrres));

 let mut overlay = shotcore::overlay::QuickAccessOverlay::default();
 overlay.push("shot-1", 1000);
 overlay.push("shot-2", 2000);
 let closed = overlay.close_top().map(|i| i.id).unwrap_or_default();
 let restored = overlay.restore_recent();
 println!("- overlay -> {} items, closed={}, restored={}", overlay.items.len(), closed, restored);

 let settings = shotcore::settings::Settings::default().normalized();
 println!("- settings -> mode={:?} fmt={} hotkey={}", settings.default_capture_mode, settings.save_format, settings.capture_hotkey);

 let link = shotcore::cloud::share_link("https://sloer.sh", "{\"id\":\"demo\",\"url\":\"/s/demo\"}").unwrap_or_default();
 println!("- cloud -> share link {}", link);

 println!("parity showcase complete: 25 modules exercised");
 Ok(())
}
