//! C ABI surface for the native shells.
//!
//! The rich model crosses the boundary as JSON strings plus integer status
//! codes, keeping the ABI tiny and identical for the Windows (C#) and macOS
//! (Swift) hosts. Strings returned by this module are heap-allocated and must
//! be released with `shotcore_string_free`.

use crate::geometry::{resolve_selection, Rect, VirtualDesktop};
use crate::history::HistoryStore;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Success.
pub const OK: c_int = 0;
/// A required pointer argument was null or unreadable.
pub const ERR_ARG: c_int = -1;
/// A JSON argument failed to parse.
pub const ERR_JSON: c_int = -3;
/// An image could not be read, decoded, or written.
pub const ERR_IMAGE: c_int = -4;
/// The license signature or key was invalid.
pub const ERR_LICENSE: c_int = -5;
/// The license is well-formed but expired.
pub const ERR_EXPIRED: c_int = -6;

unsafe fn cstr_to_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok().map(|s| s.to_owned())
}

fn string_to_cstr(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn load_font(path: &str) -> Option<ab_glyph::FontVec> {
    let bytes = std::fs::read(path).ok()?;
    ab_glyph::FontVec::try_from_vec(bytes).ok()
}

/// Core version string. Free with `shotcore_string_free`.
#[no_mangle]
pub extern "C" fn shotcore_version() -> *mut c_char {
    string_to_cstr(env!("CARGO_PKG_VERSION").to_string())
}

/// Free a string previously returned by this library.
#[no_mangle]
pub extern "C" fn shotcore_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

/// Create a new empty document and return it as JSON. Free with `shotcore_string_free`.
#[no_mangle]
pub extern "C" fn shotcore_document_new(width: u32, height: u32) -> *mut c_char {
    let doc = crate::model::Document::new(width, height);
    match crate::sidecar::to_json(&doc) {
        Ok(json) => string_to_cstr(json),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Flatten: load `image_path`, apply the annotation document `doc_json`, and
/// write the composited PNG to `out_path`. `font_path` may be null. Returns OK or an error code.
#[no_mangle]
pub extern "C" fn shotcore_export_png(
    image_path: *const c_char,
    doc_json: *const c_char,
    out_path: *const c_char,
    font_path: *const c_char,
) -> c_int {
    let (image_path, doc_json, out_path) = unsafe {
        match (
            cstr_to_string(image_path),
            cstr_to_string(doc_json),
            cstr_to_string(out_path),
        ) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return ERR_ARG,
        }
    };
    let font_path = unsafe { cstr_to_string(font_path) };
    let doc = match crate::sidecar::from_json(&doc_json) {
        Ok(d) => d,
        Err(_) => return ERR_JSON,
    };
    let base = match image::open(&image_path) {
        Ok(i) => i.to_rgba8(),
        Err(_) => return ERR_IMAGE,
    };
    let font = font_path.and_then(|p| load_font(&p));
    match crate::export::export_to_path(&base, &doc, font.as_ref(), &out_path) {
        Ok(()) => OK,
        Err(_) => ERR_IMAGE,
    }
}

/// Beautify: load `in_path`, wrap it per `options_json` (a BeautifyOptions), and write to `out_path`.
#[no_mangle]
pub extern "C" fn shotcore_beautify_png(
    in_path: *const c_char,
    out_path: *const c_char,
    options_json: *const c_char,
) -> c_int {
    let (in_path, out_path, options_json) = unsafe {
        match (
            cstr_to_string(in_path),
            cstr_to_string(out_path),
            cstr_to_string(options_json),
        ) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return ERR_ARG,
        }
    };
    let opts: crate::beautify::BeautifyOptions = match serde_json::from_str(&options_json) {
        Ok(o) => o,
        Err(_) => return ERR_JSON,
    };
    let base = match image::open(&in_path) {
        Ok(i) => i.to_rgba8(),
        Err(_) => return ERR_IMAGE,
    };
    let out = crate::beautify::beautify(&base, &opts);
    match out.save(&out_path) {
        Ok(()) => OK,
        Err(_) => ERR_IMAGE,
    }
}

/// Resolve a drag selection against a VirtualDesktop JSON; returns CaptureRegion JSON or null.
#[no_mangle]
pub extern "C" fn shotcore_resolve_selection(
    desktop_json: *const c_char,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> *mut c_char {
    let desktop_json = match unsafe { cstr_to_string(desktop_json) } {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let desktop: VirtualDesktop = match serde_json::from_str(&desktop_json) {
        Ok(d) => d,
        Err(_) => return std::ptr::null_mut(),
    };
    match resolve_selection(&desktop, Rect::new(x, y, w, h)) {
        Some(region) => match serde_json::to_string(&region) {
            Ok(j) => string_to_cstr(j),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

/// Search a HistoryStore JSON; returns a JSON array of matching entries (or null).
#[no_mangle]
pub extern "C" fn shotcore_history_search(
    store_json: *const c_char,
    query: *const c_char,
) -> *mut c_char {
    let (store_json, query) = match unsafe { (cstr_to_string(store_json), cstr_to_string(query)) } {
        (Some(a), Some(b)) => (a, b),
        _ => return std::ptr::null_mut(),
    };
    let store: HistoryStore = match serde_json::from_str(&store_json) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let matches = store.search(&query);
    match serde_json::to_string(&matches) {
        Ok(j) => string_to_cstr(j),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Sample one RGBA8 pixel from a buffer; returns a Color as JSON or null.
/// # Safety: `rgba` must point to at least `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_palette_eyedrop(rgba: *const u8, len: usize, width: u32, height: u32, x: u32, y: u32) -> *mut c_char {
 if rgba.is_null() {
 return std::ptr::null_mut();
 }
 let buf = std::slice::from_raw_parts(rgba, len);
 match crate::palette::eyedrop(buf, width, height, x, y) {
 Some(c) => match serde_json::to_string(&c) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 },
 None => std::ptr::null_mut(),
 }
}

/// Compute VideoEdit output as JSON {duration_ms,width,height,gain,channels}.
#[no_mangle]
pub extern "C" fn shotcore_videoedit_output(edit_json: *const c_char, source_ms: u64, w: u32, h: u32, channels: u32) -> *mut c_char {
 let s = match unsafe { cstr_to_string(edit_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let edit: crate::videoedit::VideoEdit = match serde_json::from_str(&s) {
 Ok(e) => e,
 Err(_) => return std::ptr::null_mut(),
 };
 let (ow, oh) = edit.output_dimensions(w, h);
 let out = serde_json::json!({
 "duration_ms": edit.output_duration_ms(source_ms),
 "width": ow,
 "height": oh,
 "gain": edit.effective_gain(),
 "channels": edit.output_channels(channels as u16),
 });
 string_to_cstr(out.to_string())
}

/// SHA-256 hex of a share password. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_share_hash_password(plain: *const c_char) -> *mut c_char {
 match unsafe { cstr_to_string(plain) } {
 Some(s) => string_to_cstr(crate::share::hash_password(&s)),
 None => std::ptr::null_mut(),
 }
}

/// Auto-balance padding insets [left,top,right,bottom] as a JSON array.
#[no_mangle]
pub extern "C" fn shotcore_auto_balance(cw: f64, ch: f64, w: f64, h: f64) -> *mut c_char {
 let (l, t, r, b) = crate::beautify::auto_balance(cw, ch, w, h);
 match serde_json::to_string(&[l, t, r, b]) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Vertical-stack layout for image sizes (JSON array of [w,h]) with a gap.
#[no_mangle]
pub extern "C" fn shotcore_combine_stack_vertical(sizes_json: *const c_char, gap: u32) -> *mut c_char {
 let s = match unsafe { cstr_to_string(sizes_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let sizes: Vec<(u32, u32)> = match serde_json::from_str(&s) {
 Ok(v) => v,
 Err(_) => return std::ptr::null_mut(),
 };
 match serde_json::to_string(&crate::combine::stack_vertical(&sizes, gap)) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Sample a curved arrow shaft; returns a JSON array of points.
#[no_mangle]
pub extern "C" fn shotcore_curved_arrow_path(fx: f64, fy: f64, tx: f64, ty: f64, bow: f64, segments: u32) -> *mut c_char {
 let path = crate::smooth::curved_arrow_path(crate::geometry::Point::new(fx, fy), crate::geometry::Point::new(tx, ty), bow, segments);
 match serde_json::to_string(&path) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Snap a highlight rect (x,y,w,h) to overlapping OCR word boxes. Rect JSON or null.
#[no_mangle]
pub extern "C" fn shotcore_smart_highlight(ocr_json: *const c_char, x: f64, y: f64, w: f64, h: f64) -> *mut c_char {
 let s = match unsafe { cstr_to_string(ocr_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let ocr: crate::ocr::OcrResult = match serde_json::from_str(&s) {
 Ok(o) => o,
 Err(_) => return std::ptr::null_mut(),
 };
 match crate::smarthl::snap_highlight(&ocr, Rect::new(x, y, w, h)) {
 Some(r) => match serde_json::to_string(&r) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 },
 None => std::ptr::null_mut(),
 }
}

/// Parse a sloershot:// URL into a SchemeCommand JSON, or null.
#[no_mangle]
pub extern "C" fn shotcore_urlscheme_parse(url: *const c_char) -> *mut c_char {
 let url = match unsafe { cstr_to_string(url) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 match crate::urlscheme::parse(&url) {
 Some(cmd) => match serde_json::to_string(&cmd) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 },
 None => std::ptr::null_mut(),
 }
}

/// Page slice rects for a tall capture as a JSON array.
#[no_mangle]
pub extern "C" fn shotcore_print_page_slices(content_w: u32, content_h: u32, page_h: u32, overlap: u32) -> *mut c_char {
 let slices = crate::print::page_slices(content_w, content_h, page_h, overlap);
 match serde_json::to_string(&slices) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Downscale a Retina PNG to 1x by scale_factor. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn shotcore_scale_to_1x_png(in_path: *const c_char, out_path: *const c_char, scale_factor: f32) -> c_int {
 let (ip, op) = match unsafe { (cstr_to_string(in_path), cstr_to_string(out_path)) } {
 (Some(a), Some(b)) => (a, b),
 _ => return -1,
 };
 let img = match image::open(&ip) {
 Ok(i) => i.to_rgba8(),
 Err(_) => return -1,
 };
 let out = crate::imgexport::scale_to_1x(&img, scale_factor);
 match out.save(&op) {
 Ok(_) => 0,
 Err(_) => -1,
 }
}

/// WCAG contrast between two hex colors. JSON {ratio,normal,large} or null.
#[no_mangle]
pub extern "C" fn shotcore_contrast(hex_a: *const c_char, hex_b: *const c_char) -> *mut c_char {
 let (a, b) = match unsafe { (cstr_to_string(hex_a), cstr_to_string(hex_b)) } {
 (Some(a), Some(b)) => (a, b),
 _ => return std::ptr::null_mut(),
 };
 let (ca, cb) = match (crate::model::Color::from_hex(&a), crate::model::Color::from_hex(&b)) {
 (Some(x), Some(y)) => (x, y),
 _ => return std::ptr::null_mut(),
 };
 let ratio = crate::contrast::contrast_ratio(ca, cb);
 let out = serde_json::json!({
 "ratio": ratio,
 "normal": format!("{:?}", crate::contrast::rate(ratio, false)),
 "large": format!("{:?}", crate::contrast::rate(ratio, true)),
 });
 string_to_cstr(out.to_string())
}

/// Render a Guide JSON to Markdown.
#[no_mangle]
pub extern "C" fn shotcore_guide_markdown(guide_json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(guide_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 match serde_json::from_str::<crate::guide::Guide>(&s) {
 Ok(g) => string_to_cstr(g.to_markdown()),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Render a Guide JSON to HTML.
#[no_mangle]
pub extern "C" fn shotcore_guide_html(guide_json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(guide_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 match serde_json::from_str::<crate::guide::Guide>(&s) {
 Ok(g) => string_to_cstr(g.to_html()),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Estimated [w,h] of a CodeCard JSON given monospace cell size.
#[no_mangle]
pub extern "C" fn shotcore_codeshot_size(card_json: *const c_char, char_w: f32, line_h: f32) -> *mut c_char {
 let s = match unsafe { cstr_to_string(card_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let card: crate::codeshot::CodeCard = match serde_json::from_str(&s) {
 Ok(c) => c,
 Err(_) => return std::ptr::null_mut(),
 };
 let (w, h) = card.estimated_size(char_w, line_h);
 match serde_json::to_string(&[w, h]) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Measure between two points. JSON {distance,dx,dy,angle}.
#[no_mangle]
pub extern "C" fn shotcore_measure(ax: f64, ay: f64, bx: f64, by: f64) -> *mut c_char {
 let a = crate::geometry::Point::new(ax, ay);
 let b = crate::geometry::Point::new(bx, by);
 let out = serde_json::json!({
 "distance": crate::measure::distance(a, b),
 "dx": b.x - a.x,
 "dy": b.y - a.y,
 "angle": crate::measure::angle_deg(a, b),
 });
 string_to_cstr(out.to_string())
}

/// Auto-zoom keyframes for clicks JSON. Returns a JSON array of keyframes.
#[no_mangle]
pub extern "C" fn shotcore_auto_zoom(clicks_json: *const c_char, scale: f64, ease_ms: u64, hold_ms: u64) -> *mut c_char {
 let s = match unsafe { cstr_to_string(clicks_json) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let clicks: Vec<crate::zoompan::ClickEvent> = match serde_json::from_str(&s) {
 Ok(c) => c,
 Err(_) => return std::ptr::null_mut(),
 };
 match serde_json::to_string(&crate::zoompan::auto_zoom_keyframes(&clicks, scale, ease_ms, hold_ms)) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Detect a table from OCR JSON and return CSV.
#[no_mangle]
pub extern "C" fn shotcore_table_csv(ocr_json: *const c_char, col_tol: f64) -> *mut c_char {
 let s = match unsafe { cstr_to_string(ocr_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let ocr: crate::ocr::OcrResult = match serde_json::from_str(&s) { Ok(o) => o, Err(_) => return std::ptr::null_mut() };
 string_to_cstr(crate::table::to_csv(&crate::table::detect_table(&ocr, col_tol)))
}

/// Detect a table from OCR JSON and return a Markdown table.
#[no_mangle]
pub extern "C" fn shotcore_table_markdown(ocr_json: *const c_char, col_tol: f64) -> *mut c_char {
 let s = match unsafe { cstr_to_string(ocr_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let ocr: crate::ocr::OcrResult = match serde_json::from_str(&s) { Ok(o) => o, Err(_) => return std::ptr::null_mut() };
 string_to_cstr(crate::table::to_markdown(&crate::table::detect_table(&ocr, col_tol)))
}

/// Convert transcript segments JSON to SRT.
#[no_mangle]
pub extern "C" fn shotcore_captions_srt(segments_json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(segments_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let segs: Vec<crate::captions::Segment> = match serde_json::from_str(&s) { Ok(v) => v, Err(_) => return std::ptr::null_mut() };
 string_to_cstr(crate::captions::to_srt(&segs))
}

/// Convert transcript segments JSON to WebVTT.
#[no_mangle]
pub extern "C" fn shotcore_captions_vtt(segments_json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(segments_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let segs: Vec<crate::captions::Segment> = match serde_json::from_str(&s) { Ok(v) => v, Err(_) => return std::ptr::null_mut() };
 string_to_cstr(crate::captions::to_vtt(&segs))
}

/// Extract up to max tags from text; returns a JSON array of strings.
#[no_mangle]
pub extern "C" fn shotcore_autotag(text: *const c_char, max: u32) -> *mut c_char {
 let t = match unsafe { cstr_to_string(text) } { Some(s) => s, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::autotag::extract_tags(&t, max as usize)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Render an annotation Document JSON to SVG markup.
#[no_mangle]
pub extern "C" fn shotcore_svg(doc_json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(doc_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 match serde_json::from_str::<crate::model::Document>(&s) { Ok(d) => string_to_cstr(crate::svgexport::to_svg(&d)), Err(_) => std::ptr::null_mut() }
}

/// Align rects JSON to a shared edge. edge: 0 Left,1 HCenter,2 Right,3 Top,4 VCenter,5 Bottom.
#[no_mangle]
pub extern "C" fn shotcore_align(rects_json: *const c_char, edge: u32) -> *mut c_char {
 let s = match unsafe { cstr_to_string(rects_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let rects: Vec<Rect> = match serde_json::from_str(&s) { Ok(v) => v, Err(_) => return std::ptr::null_mut() };
 use crate::align::Edge;
 let e = match edge { 1 => Edge::HCenter, 2 => Edge::Right, 3 => Edge::Top, 4 => Edge::VCenter, 5 => Edge::Bottom, _ => Edge::Left };
 match serde_json::to_string(&crate::align::align(&rects, e)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Mockup outer size + content origin. frame: 0 Browser,1 Window,2 Phone,3 Laptop.
#[no_mangle]
pub extern "C" fn shotcore_mockup_size(frame: u32, content_w: u32, content_h: u32) -> *mut c_char {
 use crate::mockup::{Frame, Mockup};
 let f = match frame { 1 => Frame::Window, 2 => Frame::Phone, 3 => Frame::Laptop, _ => Frame::Browser };
 let m = Mockup { frame: f, content_w, content_h };
 let (ow, oh) = m.outer_size();
 let (ox, oy) = m.content_origin();
 string_to_cstr(serde_json::json!({ "outer": [ow, oh], "origin": [ox, oy] }).to_string())
}

/// Shape-mask a PNG. shape: 0 Ellipse, 1 RoundedRect(radius). Returns 0 or -1.
#[no_mangle]
pub extern "C" fn shotcore_mask_png(in_path: *const c_char, out_path: *const c_char, shape: u32, radius: f32) -> c_int {
 let (ip, op) = match unsafe { (cstr_to_string(in_path), cstr_to_string(out_path)) } { (Some(a), Some(b)) => (a, b), _ => return -1 };
 let img = match image::open(&ip) { Ok(i) => i.to_rgba8(), Err(_) => return -1 };
 let m = if shape == 1 { crate::mask::MaskShape::RoundedRect { radius } } else { crate::mask::MaskShape::Ellipse };
 match crate::mask::apply_mask(&img, m).save(&op) { Ok(_) => 0, Err(_) => -1 }
}

/// Snap a search rect to the tight object bounds in an RGBA8 buffer. Returns Rect JSON.
/// # Safety: `rgba` must point to at least `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_snap_object(rgba: *const u8, len: usize, width: u32, height: u32, x: f64, y: f64, w: f64, h: f64, tolerance: u8) -> *mut c_char {
 if rgba.is_null() {
 return std::ptr::null_mut();
 }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) {
 Some(i) => i,
 None => return std::ptr::null_mut(),
 };
 let r = crate::objdetect::snap_to_object(&img, Rect::new(x, y, w, h), tolerance);
 match serde_json::to_string(&r) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Apply a crop aspect ratio to a rect. ratio: 0 Free, 1 Square, 2 4:3, 3 3:2,
/// 4 16:9, 5 5:4, 6 9:16. Returns the constrained rect as JSON. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_crop_constrain(x: f64, y: f64, w: f64, h: f64, ratio: u32) -> *mut c_char {
 use crate::crop::{constrain, AspectRatio};
 let ar = match ratio {
 1 => AspectRatio::Square,
 2 => AspectRatio::R4x3,
 3 => AspectRatio::R3x2,
 4 => AspectRatio::R16x9,
 5 => AspectRatio::R5x4,
 6 => AspectRatio::R9x16,
 _ => AspectRatio::Free,
 };
 let r = constrain(Rect::new(x, y, w, h), ar);
 match serde_json::to_string(&r) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Classify a decoded barcode/QR payload string. Returns BarcodePayload JSON. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_classify_payload(raw: *const c_char) -> *mut c_char {
 let raw = match unsafe { cstr_to_string(raw) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let payload = crate::recognize::BarcodePayload::new(raw);
 match serde_json::to_string(&payload) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Extract http(s)/www links from recognized text. Returns a JSON array of strings. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_extract_links(text: *const c_char) -> *mut c_char {
 let text = match unsafe { cstr_to_string(text) } {
 Some(s) => s,
 None => return std::ptr::null_mut(),
 };
 let links = crate::recognize::extract_links(&text);
 match serde_json::to_string(&links) {
 Ok(j) => string_to_cstr(j),
 Err(_) => std::ptr::null_mut(),
 }
}

/// Verify a license token against a hex public key at unix time `now`. Returns OK or an error code.
#[no_mangle]
pub extern "C" fn shotcore_license_verify(
    public_key_hex: *const c_char,
    token: *const c_char,
    now: i64,
) -> c_int {
    let (public_key_hex, token) = unsafe {
        match (cstr_to_string(public_key_hex), cstr_to_string(token)) {
            (Some(a), Some(b)) => (a, b),
            _ => return ERR_ARG,
        }
    };
    let vk = match crate::license::verifying_key_from_hex(&public_key_hex) {
        Ok(v) => v,
        Err(_) => return ERR_LICENSE,
    };
    match crate::license::verify(&vk, &token, now) {
        Ok(_) => OK,
        Err(crate::license::LicenseError::Expired) => ERR_EXPIRED,
        Err(_) => ERR_LICENSE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    unsafe fn take(p: *mut c_char) -> String {
        assert!(!p.is_null());
        let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
        shotcore_string_free(p);
        s
    }

    fn cs(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn version_is_returned() {
        let v = unsafe { take(shotcore_version()) };
        assert!(!v.is_empty());
    }

    #[test]
    fn document_new_returns_json() {
        let j = unsafe { take(shotcore_document_new(640, 480)) };
        assert!(j.contains("schema_version"));
        assert!(j.contains("640"));
    }

    #[test]
    fn resolve_selection_ffi() {
        let desktop = r#"{"displays":[{"id":0,"bounds":{"x":0.0,"y":0.0,"w":1920.0,"h":1080.0},"scale_factor":2.0,"is_primary":true}]}"#;
        let dj = cs(desktop);
        let p = shotcore_resolve_selection(dj.as_ptr(), 100.0, 50.0, 200.0, 100.0);
        let out = unsafe { take(p) };
        assert!(out.contains("physical"));
        assert!(out.contains("display_id"));
    }

    #[test]
    fn license_verify_ffi() {
        use crate::license::{generate_keypair, issue, public_key_hex, Entitlement, Plan};
        let (sk, vk) = generate_keypair();
        let ent = Entitlement {
            subject: "u".into(),
            plan: Plan::Pro,
            issued_at: 0,
            expires_at: 100,
            features: vec![],
        };
        let token = issue(&sk, &ent).unwrap();
        let pk = cs(&public_key_hex(&vk));
        let tok = cs(&token);
        assert_eq!(shotcore_license_verify(pk.as_ptr(), tok.as_ptr(), 50), OK);
        assert_eq!(
            shotcore_license_verify(pk.as_ptr(), tok.as_ptr(), 200),
            ERR_EXPIRED
        );
    }

    #[test]
    fn export_png_ffi() {
        use image::{Rgba, RgbaImage};
        let dir = tempfile::tempdir().unwrap();
        let in_path = dir.path().join("in.png");
        let out_path = dir.path().join("out.png");
        RgbaImage::from_pixel(8, 8, Rgba([1, 2, 3, 255]))
            .save(&in_path)
            .unwrap();
        let inp = cs(in_path.to_str().unwrap());
        let outp = cs(out_path.to_str().unwrap());
        let docj = unsafe { take(shotcore_document_new(8, 8)) };
        let dj = cs(&docj);
        let rc = shotcore_export_png(inp.as_ptr(), dj.as_ptr(), outp.as_ptr(), std::ptr::null());
        assert_eq!(rc, OK);
        assert!(out_path.exists());
    }

    #[test]
    fn beautify_png_ffi() {
        use image::{Rgba, RgbaImage};
        let dir = tempfile::tempdir().unwrap();
        let in_path = dir.path().join("in.png");
        let out_path = dir.path().join("out.png");
        RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]))
            .save(&in_path)
            .unwrap();
        let opts = r#"{"background":{"Solid":{"r":0,"g":0,"b":255,"a":255}},"padding":5,"corner_radius":0.0,"shadow":null}"#;
        let inp = cs(in_path.to_str().unwrap());
        let outp = cs(out_path.to_str().unwrap());
        let oj = cs(opts);
        let rc = shotcore_beautify_png(inp.as_ptr(), outp.as_ptr(), oj.as_ptr());
        assert_eq!(rc, OK);
        let loaded = image::open(&out_path).unwrap();
        assert_eq!(loaded.width(), 20);
    }

    #[test]
    fn null_args_are_rejected() {
        let rc = shotcore_export_png(
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        assert_eq!(rc, ERR_ARG);
    }
}

// ---- Smart auto-redaction ----

fn tool_from_code(code: u32) -> crate::editor::Tool {
    use crate::editor::Tool;
    match code {
        1 => Tool::Arrow,
        2 => Tool::Rectangle,
        3 => Tool::Ellipse,
        4 => Tool::Line,
        5 => Tool::Freehand,
        6 => Tool::Text,
        7 => Tool::Counter,
        8 => Tool::Highlighter,
        9 => Tool::Redact,
        _ => Tool::Select,
    }
}

/// Detect sensitive data in `ocr_json` and return `doc_json` with Redact
/// annotations added over the matching word boxes. style: 0 = blur, 1 = pixelate.
#[no_mangle]
pub extern "C" fn shotcore_auto_redact_into(
    doc_json: *const c_char,
    ocr_json: *const c_char,
    style: u32,
    strength: u32,
) -> *mut c_char {
    let (doc_json, ocr_json) = match unsafe { (cstr_to_string(doc_json), cstr_to_string(ocr_json)) }
    {
        (Some(a), Some(b)) => (a, b),
        _ => return std::ptr::null_mut(),
    };
    let mut doc: crate::model::Document = match serde_json::from_str(&doc_json) {
        Ok(d) => d,
        Err(_) => return std::ptr::null_mut(),
    };
    let ocr: crate::ocr::OcrResult = match serde_json::from_str(&ocr_json) {
        Ok(o) => o,
        Err(_) => return std::ptr::null_mut(),
    };
    let rs = if style == 1 {
        crate::model::RedactStyle::Pixelate
    } else {
        crate::model::RedactStyle::Blur
    };
    for a in crate::detect::auto_redact(&ocr, &[], rs, strength.max(2)) {
        doc.add(a);
    }
    match serde_json::to_string(&doc) {
        Ok(j) => string_to_cstr(j),
        Err(_) => std::ptr::null_mut(),
    }
}

// ---- Stateful editor controller over an opaque handle ----

/// Create an editor over a blank document. Free with `shotcore_editor_free`.
#[no_mangle]
pub extern "C" fn shotcore_editor_new(width: u32, height: u32) -> *mut crate::editor::Editor {
    Box::into_raw(Box::new(crate::editor::Editor::new(width, height)))
}

/// Free an editor handle created by `shotcore_editor_new`.
#[no_mangle]
pub extern "C" fn shotcore_editor_free(ptr: *mut crate::editor::Editor) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Set the active tool. Codes: 0 Select, 1 Arrow, 2 Rectangle, 3 Ellipse, 4 Line,
/// 5 Freehand, 6 Text, 7 Counter, 8 Highlighter, 9 Redact.
#[no_mangle]
pub extern "C" fn shotcore_editor_set_tool(ptr: *mut crate::editor::Editor, tool: u32) {
    if ptr.is_null() {
        return;
    }
    unsafe { &mut *ptr }.set_tool(tool_from_code(tool));
}

#[no_mangle]
pub extern "C" fn shotcore_editor_pointer_down(ptr: *mut crate::editor::Editor, x: f64, y: f64) {
    if ptr.is_null() {
        return;
    }
    unsafe { &mut *ptr }.pointer_down(crate::geometry::Point::new(x, y));
}

#[no_mangle]
pub extern "C" fn shotcore_editor_pointer_drag(ptr: *mut crate::editor::Editor, x: f64, y: f64) {
    if ptr.is_null() {
        return;
    }
    unsafe { &mut *ptr }.pointer_drag(crate::geometry::Point::new(x, y));
}

#[no_mangle]
pub extern "C" fn shotcore_editor_pointer_up(ptr: *mut crate::editor::Editor, x: f64, y: f64) {
    if ptr.is_null() {
        return;
    }
    unsafe { &mut *ptr }.pointer_up(crate::geometry::Point::new(x, y));
}

#[no_mangle]
pub extern "C" fn shotcore_editor_undo(ptr: *mut crate::editor::Editor) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    if unsafe { &mut *ptr }.undo() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn shotcore_editor_redo(ptr: *mut crate::editor::Editor) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    if unsafe { &mut *ptr }.redo() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn shotcore_editor_delete_selected(ptr: *mut crate::editor::Editor) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    if unsafe { &mut *ptr }.delete_selected() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn shotcore_editor_bring_to_front(ptr: *mut crate::editor::Editor) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    if unsafe { &mut *ptr }.bring_selection_to_front() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn shotcore_editor_send_to_back(ptr: *mut crate::editor::Editor) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    if unsafe { &mut *ptr }.send_selection_to_back() {
        1
    } else {
        0
    }
}

/// Set the text of the selected Text annotation. Returns 1 on success.
#[no_mangle]
pub extern "C" fn shotcore_editor_set_selected_text(
    ptr: *mut crate::editor::Editor,
    text: *const c_char,
) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    let text = match unsafe { cstr_to_string(text) } {
        Some(t) => t,
        None => return 0,
    };
    if unsafe { &mut *ptr }.set_selected_text(text) {
        1
    } else {
        0
    }
}

/// The document to draw (committed plus any live draft/transform), as JSON.
#[no_mangle]
pub extern "C" fn shotcore_editor_render_json(ptr: *mut crate::editor::Editor) -> *mut c_char {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let doc = unsafe { &*ptr }.render();
    match serde_json::to_string(&doc) {
        Ok(j) => string_to_cstr(j),
        Err(_) => std::ptr::null_mut(),
    }
}

/// The committed document (no preview), as JSON.
#[no_mangle]
pub extern "C" fn shotcore_editor_document_json(ptr: *mut crate::editor::Editor) -> *mut c_char {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    match serde_json::to_string(unsafe { &*ptr }.document()) {
        Ok(j) => string_to_cstr(j),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn shotcore_editor_can_undo(ptr: *mut crate::editor::Editor) -> c_int {
    if !ptr.is_null() && unsafe { &*ptr }.can_undo() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn shotcore_editor_can_redo(ptr: *mut crate::editor::Editor) -> c_int {
    if !ptr.is_null() && unsafe { &*ptr }.can_redo() {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod editor_ffi_tests {
    use super::*;
    use std::ffi::CString;

    unsafe fn take_str(p: *mut c_char) -> String {
        assert!(!p.is_null());
        let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
        shotcore_string_free(p);
        s
    }

    #[test]
    fn editor_handle_draw_commit_undo() {
        let ed = shotcore_editor_new(400, 300);
        assert!(!ed.is_null());
        shotcore_editor_set_tool(ed, 2);
        shotcore_editor_pointer_down(ed, 10.0, 10.0);
        shotcore_editor_pointer_drag(ed, 110.0, 60.0);
        let render = unsafe { take_str(shotcore_editor_render_json(ed)) };
        assert!(render.contains("Rectangle"));
        let committed = unsafe { take_str(shotcore_editor_document_json(ed)) };
        assert!(committed.contains("\"annotations\":[]"));
        assert_eq!(shotcore_editor_can_undo(ed), 0);
        shotcore_editor_pointer_up(ed, 110.0, 60.0);
        assert_eq!(shotcore_editor_can_undo(ed), 1);
        let committed2 = unsafe { take_str(shotcore_editor_document_json(ed)) };
        assert!(committed2.contains("Rectangle"));
        assert_eq!(shotcore_editor_undo(ed), 1);
        let committed3 = unsafe { take_str(shotcore_editor_document_json(ed)) };
        assert!(committed3.contains("\"annotations\":[]"));
        shotcore_editor_free(ed);
    }

    #[test]
    fn null_editor_is_safe() {
        assert_eq!(shotcore_editor_can_undo(std::ptr::null_mut()), 0);
        shotcore_editor_pointer_down(std::ptr::null_mut(), 1.0, 1.0);
        shotcore_editor_free(std::ptr::null_mut());
    }

    #[test]
    fn auto_redact_into_adds_redactions() {
        let doc = unsafe { take_str(shotcore_document_new(200, 100)) };
        let ocr = r#"{"lines":[{"text":"alice@example.com","bbox":{"x":0.0,"y":0.0,"w":120.0,"h":10.0},"words":[{"text":"alice@example.com","bbox":{"x":0.0,"y":0.0,"w":120.0,"h":10.0},"confidence":0.9}]}]}"#;
        let dj = CString::new(doc).unwrap();
        let oj = CString::new(ocr).unwrap();
        let out = unsafe { take_str(shotcore_auto_redact_into(dj.as_ptr(), oj.as_ptr(), 1, 12)) };
        assert!(out.contains("Redact"));
    }
}

// ---- Image effects (fx) ----

/// Apply an image effect described by `op_json` to `in_path`, writing `out_path`.
/// op_json examples: {"op":"grayscale"}, {"op":"rotate","deg":90}, {"op":"flip","axis":"h"},
/// {"op":"crop","x":0,"y":0,"w":100,"h":80}, {"op":"resize","w":640,"h":480},
/// {"op":"scale","factor":0.5}, {"op":"blur","sigma":4}, {"op":"vignette","strength":0.7},
/// {"op":"brightness","delta":40}, {"op":"contrast","factor":1.3}, {"op":"invert"},
/// {"op":"sepia"}, {"op":"border","thickness":16,"color":{"r":255,"g":255,"b":255,"a":255}},
/// {"op":"spotlight","x":40,"y":40,"w":300,"h":200,"dim":0.6}, {"op":"jpeg","quality":85}.
#[no_mangle]
pub extern "C" fn shotcore_fx_apply(
    in_path: *const c_char,
    out_path: *const c_char,
    op_json: *const c_char,
) -> c_int {
    let (in_path, out_path, op_json) = unsafe {
        match (
            cstr_to_string(in_path),
            cstr_to_string(out_path),
            cstr_to_string(op_json),
        ) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return ERR_ARG,
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&op_json) {
        Ok(x) => x,
        Err(_) => return ERR_JSON,
    };
    let op = match v.get("op").and_then(|o| o.as_str()) {
        Some(s) => s.to_string(),
        None => return ERR_JSON,
    };
    let base = match image::open(&in_path) {
        Ok(i) => i.to_rgba8(),
        Err(_) => return ERR_IMAGE,
    };
    let num = |key: &str, d: f64| v.get(key).and_then(|x| x.as_f64()).unwrap_or(d);
    let rect =
        || crate::geometry::Rect::new(num("x", 0.0), num("y", 0.0), num("w", 0.0), num("h", 0.0));
    let out = match op.as_str() {
        "grayscale" => crate::fx::grayscale(&base),
        "sepia" => crate::fx::sepia(&base),
        "invert" => crate::fx::invert(&base),
        "blur" => crate::fx::blur(&base, num("sigma", 4.0) as f32),
        "vignette" => crate::fx::vignette(&base, num("strength", 0.6) as f32),
        "brightness" => crate::fx::adjust_brightness(&base, num("delta", 0.0) as i32),
        "contrast" => crate::fx::adjust_contrast(&base, num("factor", 1.0) as f32),
        "flip" => {
            if v.get("axis").and_then(|a| a.as_str()) == Some("v") {
                crate::fx::flip_vertical(&base)
            } else {
                crate::fx::flip_horizontal(&base)
            }
        }
        "rotate" => match num("deg", 90.0) as i32 {
            180 => crate::fx::rotate180(&base),
            270 => crate::fx::rotate270(&base),
            _ => crate::fx::rotate90(&base),
        },
        "resize" => crate::fx::resize(
            &base,
            num("w", base.width() as f64) as u32,
            num("h", base.height() as f64) as u32,
        ),
        "scale" => crate::fx::scale(&base, num("factor", 1.0) as f32),
        "crop" => crate::fx::crop(&base, &rect()),
        "spotlight" => crate::fx::spotlight(&base, &rect(), num("dim", 0.6) as f32),
        "border" => {
            let c = v.get("color");
            let ch = |k: &str, d: u64| {
                c.and_then(|o| o.get(k))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(d) as u8
            };
            let color =
                crate::model::Color::rgba(ch("r", 255), ch("g", 255), ch("b", 255), ch("a", 255));
            crate::fx::add_border(&base, num("thickness", 12.0) as u32, color)
        }
        "jpeg" => {
            let bytes = match crate::fx::to_jpeg_bytes(&base, num("quality", 85.0) as u8) {
                Ok(b) => b,
                Err(_) => return ERR_IMAGE,
            };
            return match std::fs::write(&out_path, bytes) {
                Ok(_) => OK,
                Err(_) => ERR_IMAGE,
            };
        }
        _ => return ERR_JSON,
    };
    match out.save(&out_path) {
        Ok(_) => OK,
        Err(_) => ERR_IMAGE,
    }
}

#[cfg(test)]
mod fx_ffi_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn fx_apply_grayscale_and_crop() {
        use image::{Rgba, RgbaImage};
        let dir = tempfile::tempdir().unwrap();
        let inp = dir.path().join("in.png");
        let outp = dir.path().join("out.png");
        RgbaImage::from_pixel(12, 12, Rgba([200, 100, 50, 255]))
            .save(&inp)
            .unwrap();
        let i = CString::new(inp.to_str().unwrap()).unwrap();
        let o = CString::new(outp.to_str().unwrap()).unwrap();
        let gray = CString::new("{\"op\":\"grayscale\"}").unwrap();
        assert_eq!(shotcore_fx_apply(i.as_ptr(), o.as_ptr(), gray.as_ptr()), OK);
        let g = image::open(&outp).unwrap().to_rgba8();
        let p = g.get_pixel(0, 0).0;
        assert_eq!(p[0], p[1]);
        assert_eq!(p[1], p[2]);
        let crop = CString::new("{\"op\":\"crop\",\"x\":2,\"y\":2,\"w\":5,\"h\":4}").unwrap();
        assert_eq!(shotcore_fx_apply(i.as_ptr(), o.as_ptr(), crop.as_ptr()), OK);
        assert_eq!(image::open(&outp).unwrap().to_rgba8().dimensions(), (5, 4));
        let bad = CString::new("{\"op\":\"nope\"}").unwrap();
        assert_eq!(
            shotcore_fx_apply(i.as_ptr(), o.as_ptr(), bad.as_ptr()),
            ERR_JSON
        );
    }
}

#[cfg(test)]
mod ffi_ext_tests {
 use super::*;
 use std::ffi::CString;

 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }

 #[test]
 fn crop_constrain_square_over_ffi() {
 let j = unsafe { take(shotcore_crop_constrain(0.0, 0.0, 200.0, 100.0, 1)) };
 assert!(j.contains("\"w\":100.0"));
 assert!(j.contains("\"h\":100.0"));
 }

 #[test]
 fn classify_payload_url_over_ffi() {
 let raw = CString::new("https://sloershot.app").unwrap();
 let j = unsafe { take(shotcore_classify_payload(raw.as_ptr())) };
 assert!(j.contains("Url"));
 }

 #[test]
 fn extract_links_over_ffi() {
 let text = CString::new("visit https://a.com now").unwrap();
 let j = unsafe { take(shotcore_extract_links(text.as_ptr())) };
 assert!(j.contains("https://a.com"));
 }
}

#[cfg(test)]
mod ffi_ext2_tests {
 use super::*;
 use std::ffi::CString;

 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }

 #[test]
 fn auto_balance_over_ffi() {
 let j = unsafe { take(shotcore_auto_balance(100.0, 100.0, 200.0, 300.0)) };
 assert!(j.contains("50") && j.contains("100"));
 }

 #[test]
 fn share_hash_over_ffi() {
 let p = CString::new("pw").unwrap();
 let j = unsafe { take(shotcore_share_hash_password(p.as_ptr())) };
 assert_eq!(j.len(), 64);
 }

 #[test]
 fn videoedit_output_over_ffi() {
 let edit = CString::new("{\"trim_start_ms\":0,\"trim_end_ms\":0,\"scale\":0.5,\"mute\":false,\"mono\":false,\"volume\":1.0}").unwrap();
 let j = unsafe { take(shotcore_videoedit_output(edit.as_ptr(), 10000, 1920, 1080, 2)) };
 assert!(j.contains("\"width\":960"));
 assert!(j.contains("\"duration_ms\":10000"));
 }

 #[test]
 fn combine_layout_over_ffi() {
 let sizes = CString::new("[[100,40],[60,30]]").unwrap();
 let j = unsafe { take(shotcore_combine_stack_vertical(sizes.as_ptr(), 10)) };
 assert!(j.contains("\"canvas_w\":100"));
 }

 #[test]
 fn curved_arrow_over_ffi() {
 let j = unsafe { take(shotcore_curved_arrow_path(0.0, 0.0, 10.0, 0.0, 0.5, 4)) };
 assert_eq!(j.matches("\"x\"").count(), 5);
 }

 #[test]
 fn eyedrop_over_ffi() {
 let buf: [u8; 8] = [255, 0, 0, 255, 0, 255, 0, 255];
 let j = unsafe { take(shotcore_palette_eyedrop(buf.as_ptr(), buf.len(), 2, 1, 1, 0)) };
 assert!(j.contains("\"g\":255"));
 }

 #[test]
 fn smart_highlight_over_ffi() {
 let ocr = CString::new("{\"lines\":[{\"text\":\"Hi\",\"bbox\":{\"x\":10.0,\"y\":10.0,\"w\":40.0,\"h\":12.0},\"words\":[{\"text\":\"Hi\",\"bbox\":{\"x\":10.0,\"y\":10.0,\"w\":40.0,\"h\":12.0},\"confidence\":0.9}]}]}").unwrap();
 let j = unsafe { take(shotcore_smart_highlight(ocr.as_ptr(), 12.0, 12.0, 5.0, 4.0)) };
 assert!(j.contains("\"x\":10"));
 }
}

#[cfg(test)]
mod ffi_ext3_tests {
 use super::*;
 use std::ffi::CString;

 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }

 #[test]
 fn urlscheme_parse_over_ffi() {
 let u = CString::new("sloershot://capture-area?x=10&width=300").unwrap();
 let j = unsafe { take(shotcore_urlscheme_parse(u.as_ptr())) };
 assert!(j.contains("capture-area"));
 assert!(j.contains("300"));
 }

 #[test]
 fn print_slices_over_ffi() {
 let j = unsafe { take(shotcore_print_page_slices(100, 250, 100, 20)) };
 assert_eq!(j.matches("\"h\"").count(), 3);
 }

 #[test]
 fn scale_to_1x_png_over_ffi() {
 let dir = tempfile::tempdir().unwrap();
 let inp = dir.path().join("in.png");
 let outp = dir.path().join("out.png");
 image::RgbaImage::new(40, 20).save(&inp).unwrap();
 let i = CString::new(inp.to_str().unwrap()).unwrap();
 let o = CString::new(outp.to_str().unwrap()).unwrap();
 assert_eq!(shotcore_scale_to_1x_png(i.as_ptr(), o.as_ptr(), 2.0), 0);
 let out = image::open(&outp).unwrap();
 assert_eq!((out.width(), out.height()), (20, 10));
 }
}

#[cfg(test)]
mod ffi_ext4_tests {
 use super::*;
 use std::ffi::CString;

 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }

 #[test]
 fn contrast_over_ffi() {
 let a = CString::new("#000000").unwrap();
 let b = CString::new("#FFFFFF").unwrap();
 let j = unsafe { take(shotcore_contrast(a.as_ptr(), b.as_ptr())) };
 assert!(j.contains("AAA"));
 }

 #[test]
 fn guide_markdown_over_ffi() {
 let g = CString::new("{\"title\":\"How\",\"steps\":[{\"index\":1,\"image_path\":\"a.png\",\"caption\":\"Open\"}]}").unwrap();
 let j = unsafe { take(shotcore_guide_markdown(g.as_ptr())) };
 assert!(j.contains("# How"));
 assert!(j.contains("1. Open"));
 }

 #[test]
 fn codeshot_size_over_ffi() {
 let c = CString::new("{\"code\":\"abcdef\",\"language\":\"rust\",\"theme\":\"Dark\",\"controls\":\"None\",\"line_numbers\":false,\"padding\":32,\"font_size\":14.0,\"tab_width\":4}").unwrap();
 let j = unsafe { take(shotcore_codeshot_size(c.as_ptr(), 8.0, 18.0)) };
 assert!(j.contains("112"));
 }

 #[test]
 fn measure_over_ffi() {
 let j = unsafe { take(shotcore_measure(0.0, 0.0, 3.0, 4.0)) };
 assert!(j.contains("\"distance\":5"));
 }

 #[test]
 fn auto_zoom_over_ffi() {
 let clicks = CString::new("[{\"t_ms\":0,\"x\":10.0,\"y\":20.0}]").unwrap();
 let j = unsafe { take(shotcore_auto_zoom(clicks.as_ptr(), 2.0, 200, 500)) };
 assert_eq!(j.matches("\"scale\"").count(), 4);
 }
}

#[cfg(test)]
mod ffi_ext5_tests {
 use super::*;
 use std::ffi::CString;

 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }

 #[test]
 fn table_csv_over_ffi() {
 use crate::geometry::Rect;
 use crate::ocr::{OcrLine, OcrResult, OcrWord};
 let mk = |t: &str, x: f64| OcrWord { text: t.to_string(), bbox: Rect::new(x, 0.0, 30.0, 10.0), confidence: 1.0 };
 let ocr = OcrResult::new(vec![
 OcrLine::from_words(vec![mk("Name", 0.0), mk("Age", 100.0)]),
 OcrLine::from_words(vec![mk("Alice", 2.0), mk("30", 101.0)]),
 ]);
 let j = CString::new(serde_json::to_string(&ocr).unwrap()).unwrap();
 let csv = unsafe { take(shotcore_table_csv(j.as_ptr(), 10.0)) };
 assert!(csv.contains("Name,Age"));
 }

 #[test]
 fn captions_srt_over_ffi() {
 let segs = CString::new("[{\"start_ms\":1000,\"end_ms\":2000,\"text\":\"Hi\"}]").unwrap();
 let srt = unsafe { take(shotcore_captions_srt(segs.as_ptr())) };
 assert!(srt.contains("00:00:01,000 --> 00:00:02,000"));
 }

 #[test]
 fn autotag_over_ffi() {
 let t = CString::new("invoice invoice total").unwrap();
 let j = unsafe { take(shotcore_autotag(t.as_ptr(), 3)) };
 assert!(j.contains("invoice"));
 }

 #[test]
 fn svg_over_ffi() {
 let doc = crate::model::Document::new(100, 50);
 let dj = CString::new(serde_json::to_string(&doc).unwrap()).unwrap();
 let svg = unsafe { take(shotcore_svg(dj.as_ptr())) };
 assert!(svg.contains("<svg"));
 }

 #[test]
 fn align_over_ffi() {
 use crate::geometry::Rect;
 let rects = vec![Rect::new(10.0, 0.0, 20.0, 10.0), Rect::new(40.0, 0.0, 30.0, 10.0)];
 let rj = CString::new(serde_json::to_string(&rects).unwrap()).unwrap();
 let j = unsafe { take(shotcore_align(rj.as_ptr(), 0)) };
 assert!(j.contains("\"x\":10.0"));
 }

 #[test]
 fn mockup_over_ffi() {
 let j = unsafe { take(shotcore_mockup_size(0, 800, 600)) };
 assert!(j.contains("\"outer\":[802,641]"));
 }

 #[test]
 fn mask_png_over_ffi() {
 let dir = tempfile::tempdir().unwrap();
 let inp = dir.path().join("in.png");
 let outp = dir.path().join("out.png");
 image::RgbaImage::from_pixel(10, 10, image::Rgba([255, 255, 255, 255])).save(&inp).unwrap();
 let i = CString::new(inp.to_str().unwrap()).unwrap();
 let o = CString::new(outp.to_str().unwrap()).unwrap();
 assert_eq!(shotcore_mask_png(i.as_ptr(), o.as_ptr(), 0, 0.0), 0);
 let out = image::open(&outp).unwrap().to_rgba8();
 assert_eq!(out.get_pixel(0, 0).0[3], 0);
 }
}


#[cfg(test)]
mod ffi_ext6_tests {
 use super::*;

 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }

 #[test]
 fn snap_object_over_ffi() {
 let mut buf = vec![255u8; 4 * 4 * 4];
 let idx = (1 * 4 + 1) * 4;
 buf[idx] = 0; buf[idx + 1] = 0; buf[idx + 2] = 0; buf[idx + 3] = 255;
 let j = unsafe { take(shotcore_snap_object(buf.as_ptr(), buf.len(), 4, 4, 0.0, 0.0, 4.0, 4.0, 10)) };
 assert!(j.contains("\"x\":1.0"));
 assert!(j.contains("\"w\":1.0"));
 }
}
/// Encode text into a QR code. Returns JSON {version, size, modules:[strings of 0/1]} or null.
/// Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_qr_encode(text: *const c_char) -> *mut c_char {
 let t = match unsafe { cstr_to_string(text) } { Some(s) => s, None => return std::ptr::null_mut() };
 match crate::qrcode::encode(&t) {
 Ok((version, grid)) => {
 let modules: Vec<String> = grid
 .iter()
 .map(|row| row.iter().map(|&d| if d { 49u8 as char } else { 48u8 as char }).collect())
 .collect();
 let v = serde_json::json!({"version": version, "size": grid.len(), "modules": modules});
 string_to_cstr(v.to_string())
 }
 Err(_) => std::ptr::null_mut(),
 }
}

/// Decode the first QR symbol in an RGBA8 buffer. Returns JSON {text, kind} or null.
/// # Safety: `rgba` must point to at least `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_qr_decode(rgba: *const u8, len: usize, width: u32, height: u32) -> *mut c_char {
 if rgba.is_null() {
 return std::ptr::null_mut();
 }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) {
 Some(i) => i,
 None => return std::ptr::null_mut(),
 };
 match crate::qrcode::decode(&img) {
 Ok(text) => {
 let kind = crate::recognize::classify(&text);
 let v = serde_json::json!({"text": text, "kind": format!("{:?}", kind)});
 string_to_cstr(v.to_string())
 }
 Err(_) => std::ptr::null_mut(),
 }
}

#[cfg(test)]
mod qr_ffi_tests {
 use super::*;
 use std::ffi::{CStr, CString};
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn qr_encode_and_decode_over_ffi() {
 let text = CString::new("https://sloer.sh/x").unwrap();
 let enc = unsafe { take(shotcore_qr_encode(text.as_ptr())) };
 assert!(enc.contains("\"version\""));
 assert!(enc.contains("\"modules\""));
 let (_, grid) = crate::qrcode::encode("https://sloer.sh/x").unwrap();
 let img = crate::qrcode::render(&grid, 4, 4);
 let raw = img.as_raw();
 let j = unsafe { take(shotcore_qr_decode(raw.as_ptr(), raw.len(), img.width(), img.height())) };
 assert!(j.contains("sloer.sh"));
 assert!(j.contains("Url"));
 }
}

/// Recording: elapsed-time label (m:ss) for the menu bar. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_record_elapsed(ms: u64) -> *mut c_char {
 string_to_cstr(crate::recordcompose::format_elapsed(ms))
}

/// Recording: number of frames captured for a target fps over a duration (ms).
#[no_mangle]
pub extern "C" fn shotcore_record_frame_count(fps: u32, duration_ms: u64) -> u64 {
 crate::recordcompose::frame_count(fps, duration_ms) as u64
}

/// Recording: webcam overlay rect for a frame. cam_json is a CameraOverlay; returns Rect JSON.
#[no_mangle]
pub extern "C" fn shotcore_record_camera_rect(cam_json: *const c_char, width: u32, height: u32) -> *mut c_char {
 let s = match unsafe { cstr_to_string(cam_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let cam: crate::record::CameraOverlay = match serde_json::from_str(&s) { Ok(c) => c, Err(_) => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::recordcompose::camera_rect(&cam, width, height)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Recording: keystroke HUD bar rect for a frame. disp_json is a KeystrokeDisplay.
#[no_mangle]
pub extern "C" fn shotcore_record_keystroke_rect(disp_json: *const c_char, width: u32, height: u32, text_len: u32) -> *mut c_char {
 let s = match unsafe { cstr_to_string(disp_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let disp: crate::record::KeystrokeDisplay = match serde_json::from_str(&s) { Ok(d) => d, Err(_) => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::recordcompose::keystroke_bar_rect(&disp, width, height, text_len as usize)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

#[cfg(test)]
mod record_ffi_tests {
 use super::*;
 use std::ffi::{CStr, CString};
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn record_ffi_basics() {
 assert_eq!(unsafe { take(shotcore_record_elapsed(65_000)) }, "1:05");
 assert_eq!(shotcore_record_frame_count(30, 1000), 30);
 let cam = CString::new("{\"shape\":\"Circle\",\"position\":\"BottomLeft\",\"size_frac\":0.18,\"fullscreen\":false,\"mirrored\":true}").unwrap();
 let r = unsafe { take(shotcore_record_camera_rect(cam.as_ptr(), 1920, 1080)) };
 assert!(r.contains("\"x\""));
 }
}

/// Crosshair guide lines through (cx,cy) on a w x h frame. Returns JSON [horizRect, vertRect].
#[no_mangle]
pub extern "C" fn shotcore_crosshair_lines(cx: f64, cy: f64, w: f64, h: f64) -> *mut c_char {
 let (hz, vt) = crate::magnifier::crosshair_lines(cx, cy, w, h);
 match serde_json::to_string(&[hz, vt]) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Hex color (#RRGGBB) of the pixel at (x,y) in an RGBA8 buffer, or null.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_pixel_hex(rgba: *const u8, len: usize, width: u32, height: u32, x: u32, y: u32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match crate::magnifier::pixel_hex(&img, x, y) { Some(s) => string_to_cstr(s), None => std::ptr::null_mut() }
}

/// Average hex color of a region in an RGBA8 buffer (eyedropper / loupe center read).
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_region_average_hex(rgba: *const u8, len: usize, width: u32, height: u32, x: f64, y: f64, w: f64, h: f64) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 string_to_cstr(crate::magnifier::region_average_hex(&img, Rect::new(x, y, w, h)))
}

#[cfg(test)]
mod magnifier_ffi_tests {
 use super::*;
 use std::ffi::CStr;
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn crosshair_and_pixel_over_ffi() {
 let j = unsafe { take(shotcore_crosshair_lines(10.0, 20.0, 100.0, 80.0)) };
 assert!(j.contains("\"y\":20") && j.contains("\"x\":10"));
 let buf = vec![255u8; 2 * 2 * 4];
 let hex = unsafe { take(shotcore_pixel_hex(buf.as_ptr(), buf.len(), 2, 2, 0, 0)) };
 assert_eq!(hex, "#FFFFFF");
 }
}

/// OCR: recognized text whose word centers fall in a region, in reading order. ocr_json
/// is an OcrResult. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_ocr_text_in_region(ocr_json: *const c_char, x: f64, y: f64, w: f64, h: f64) -> *mut c_char {
 let s = match unsafe { cstr_to_string(ocr_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let ocr: crate::ocr::OcrResult = match serde_json::from_str(&s) { Ok(o) => o, Err(_) => return std::ptr::null_mut() };
 string_to_cstr(crate::ocrflow::text_in_region(&ocr, Rect::new(x, y, w, h)))
}

/// OCR: the whole result as a single clipboard line. ocr_json is an OcrResult.
#[no_mangle]
pub extern "C" fn shotcore_ocr_single_line(ocr_json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(ocr_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 let ocr: crate::ocr::OcrResult = match serde_json::from_str(&s) { Ok(o) => o, Err(_) => return std::ptr::null_mut() };
 string_to_cstr(crate::ocrflow::to_single_line(&ocr))
}

/// OCR: number of recognized words whose center falls in a region. -1 on bad input.
#[no_mangle]
pub extern "C" fn shotcore_ocr_word_count_region(ocr_json: *const c_char, x: f64, y: f64, w: f64, h: f64) -> i64 {
 let s = match unsafe { cstr_to_string(ocr_json) } { Some(s) => s, None => return -1 };
 let ocr: crate::ocr::OcrResult = match serde_json::from_str(&s) { Ok(o) => o, Err(_) => return -1 };
 crate::ocrflow::word_count_in_region(&ocr, Rect::new(x, y, w, h)) as i64
}

#[cfg(test)]
mod ocrflow_ffi_tests {
 use super::*;
 use std::ffi::{CStr, CString};
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn ocr_region_over_ffi() {
 let ocr = "{\"lines\":[{\"text\":\"Hi there\",\"bbox\":{\"x\":0.0,\"y\":0.0,\"w\":80.0,\"h\":16.0},\"words\":[{\"text\":\"Hi\",\"bbox\":{\"x\":0.0,\"y\":0.0,\"w\":20.0,\"h\":16.0},\"confidence\":0.9},{\"text\":\"there\",\"bbox\":{\"x\":30.0,\"y\":0.0,\"w\":50.0,\"h\":16.0},\"confidence\":0.9}]}]}";
 let j = CString::new(ocr).unwrap();
 let line = unsafe { take(shotcore_ocr_single_line(j.as_ptr())) };
 assert_eq!(line, "Hi there");
 assert_eq!(shotcore_ocr_word_count_region(j.as_ptr(), 0.0, 0.0, 200.0, 50.0), 2);
 }
}

/// Cloud: build a POST /v1/share request body. password may be null (open link);
/// expires_at/max_views negative means unset. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_share_request(password: *const c_char, expires_at: i64, max_views: i64) -> *mut c_char {
 let pw = unsafe { cstr_to_string(password) };
 let req = crate::cloud::ShareRequest {
 password: pw,
 expires_at: if expires_at < 0 { None } else { Some(expires_at) },
 max_views: if max_views < 0 { None } else { Some(max_views as u32) },
 };
 string_to_cstr(req.to_json())
}

/// Cloud: build the absolute share link from a base URL and the backend response JSON.
#[no_mangle]
pub extern "C" fn shotcore_share_link(base_url: *const c_char, response_json: *const c_char) -> *mut c_char {
 let base = match unsafe { cstr_to_string(base_url) } { Some(s) => s, None => return std::ptr::null_mut() };
 let resp = match unsafe { cstr_to_string(response_json) } { Some(s) => s, None => return std::ptr::null_mut() };
 match crate::cloud::share_link(&base, &resp) { Some(s) => string_to_cstr(s), None => std::ptr::null_mut() }
}

#[cfg(test)]
mod cloud_ffi_tests {
 use super::*;
 use std::ffi::{CStr, CString};
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn share_link_and_request_over_ffi() {
 let base = CString::new("https://x.io").unwrap();
 let resp = CString::new("{\"id\":\"k\",\"url\":\"/s/k\"}").unwrap();
 let link = unsafe { take(shotcore_share_link(base.as_ptr(), resp.as_ptr())) };
 assert_eq!(link, "https://x.io/s/k");
 let req = unsafe { take(shotcore_share_request(std::ptr::null(), -1, 5)) };
 assert!(req.contains("\"max_views\":5"));
 }
}

/// Default application settings as JSON. Free with shotcore_string_free.
#[no_mangle]
pub extern "C" fn shotcore_settings_default() -> *mut c_char {
 string_to_cstr(crate::settings::Settings::default().to_json())
}

/// Normalize/validate a settings JSON (clamps save format + recording). Returns JSON or null.
#[no_mangle]
pub extern "C" fn shotcore_settings_normalize(json: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(json) } { Some(s) => s, None => return std::ptr::null_mut() };
 match crate::settings::Settings::from_json(&s) { Some(set) => string_to_cstr(set.normalized().to_json()), None => std::ptr::null_mut() }
}

/// Default Quick Access Overlay state as JSON.
#[no_mangle]
pub extern "C" fn shotcore_overlay_default() -> *mut c_char {
 string_to_cstr(serde_json::to_string(&crate::overlay::QuickAccessOverlay::default()).unwrap_or_default())
}

/// Whether the overlay should auto-close. overlay_json is a QuickAccessOverlay. 1 yes, 0 no, -1 bad.
#[no_mangle]
pub extern "C" fn shotcore_overlay_should_close(overlay_json: *const c_char, now: i64, last_interaction: i64) -> i32 {
 let s = match unsafe { cstr_to_string(overlay_json) } { Some(s) => s, None => return -1 };
 let o: crate::overlay::QuickAccessOverlay = match serde_json::from_str(&s) { Ok(o) => o, Err(_) => return -1 };
 if o.should_auto_close(now, last_interaction) { 1 } else { 0 }
}

#[cfg(test)]
mod settings_ffi_tests {
 use super::*;
 use std::ffi::CStr;
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn settings_and_overlay_defaults_over_ffi() {
 let def = unsafe { take(shotcore_settings_default()) };
 assert!(def.contains("\"save_format\":\"png\""));
 let ov = unsafe { take(shotcore_overlay_default()) };
 assert!(ov.contains("auto_close_secs"));
 }
}

/// Perceptual average hash of an RGBA8 buffer (history dedupe / find-similar). 0 on bad input.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_ahash(rgba: *const u8, len: usize, width: u32, height: u32) -> u64 {
 if rgba.is_null() { return 0; }
 let buf = std::slice::from_raw_parts(rgba, len);
 match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => crate::phash::ahash(&i), None => 0 }
}

/// Perceptual difference hash of an RGBA8 buffer. 0 on bad input.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_dhash(rgba: *const u8, len: usize, width: u32, height: u32) -> u64 {
 if rgba.is_null() { return 0; }
 let buf = std::slice::from_raw_parts(rgba, len);
 match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => crate::phash::dhash(&i), None => 0 }
}

/// Hamming distance between two perceptual hashes (0..=64).
#[no_mangle]
pub extern "C" fn shotcore_hamming(a: u64, b: u64) -> u32 {
 crate::phash::hamming(a, b)
}

#[cfg(test)]
mod phash_ffi_tests {
 use super::*;
 #[test]
 fn phash_over_ffi() {
 let buf = vec![255u8; 16 * 16 * 4];
 let h1 = unsafe { shotcore_ahash(buf.as_ptr(), buf.len(), 16, 16) };
 let h2 = unsafe { shotcore_ahash(buf.as_ptr(), buf.len(), 16, 16) };
 assert_eq!(shotcore_hamming(h1, h2), 0);
 let d = unsafe { shotcore_dhash(buf.as_ptr(), buf.len(), 16, 16) };
 assert_eq!(shotcore_hamming(d, d), 0);
 }
}

/// Extract up to k dominant colors from an RGBA8 buffer (median-cut). Returns JSON [Color,...].
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_dominant_colors(rgba: *const u8, len: usize, width: u32, height: u32, k: u32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 let cols = crate::palettegen::dominant_colors(&img, k as usize);
 match serde_json::to_string(&cols) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

#[cfg(test)]
mod palettegen_ffi_tests {
 use super::*;
 use std::ffi::CStr;
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn dominant_colors_over_ffi() {
 let buf = vec![128u8; 8 * 8 * 4];
 let j = unsafe { take(shotcore_dominant_colors(buf.as_ptr(), buf.len(), 8, 8, 3)) };
 assert!(j.contains("\"r\""));
 }
}

/// Image diff between two RGBA8 buffers. Returns DiffResult JSON or null.
/// # Safety: both pointers must be readable for their given lengths.
#[no_mangle]
pub unsafe extern "C" fn shotcore_image_diff(a: *const u8, a_len: usize, aw: u32, ah: u32, b: *const u8, b_len: usize, bw: u32, bh: u32, tol: u8) -> *mut c_char {
 if a.is_null() || b.is_null() { return std::ptr::null_mut(); }
 let abuf = std::slice::from_raw_parts(a, a_len);
 let bbuf = std::slice::from_raw_parts(b, b_len);
 let ai = match image::RgbaImage::from_raw(aw, ah, abuf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 let bi = match image::RgbaImage::from_raw(bw, bh, bbuf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::imagediff::diff(&ai, &bi, tol)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Strong-edge pixel count (Sobel) for an RGBA8 buffer. 0 on bad input.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_edge_count(rgba: *const u8, len: usize, width: u32, height: u32, threshold: u8) -> u64 {
 if rgba.is_null() { return 0; }
 let buf = std::slice::from_raw_parts(rgba, len);
 match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => crate::edges::edge_count(&crate::edges::sobel(&i), threshold), None => 0 }
}

/// Bounding boxes of connected foreground regions in an RGBA8 buffer. Returns JSON [Rect,...].
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_segment_regions(rgba: *const u8, len: usize, width: u32, height: u32, tol: u8, min_area: u32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::segment::regions(&img, tol, min_area)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

#[cfg(test)]
mod vision_ffi_tests {
 use super::*;
 use std::ffi::CStr;
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn diff_edges_segment_over_ffi() {
 let a = vec![255u8; 8 * 8 * 4];
 let mut b = a.clone();
 let idx = (2 * 8 + 2) * 4;
 b[idx] = 0; b[idx + 1] = 0; b[idx + 2] = 0;
 let j = unsafe { take(shotcore_image_diff(a.as_ptr(), a.len(), 8, 8, b.as_ptr(), b.len(), 8, 8, 10)) };
 assert!(j.contains("\"changed_pixels\":1"));
 let ec = unsafe { shotcore_edge_count(b.as_ptr(), b.len(), 8, 8, 30) };
 assert!(ec > 0);
 let regions = unsafe { take(shotcore_segment_regions(b.as_ptr(), b.len(), 8, 8, 50, 1)) };
 assert!(regions.contains("\"x\":2"));
 }
}

/// Otsu adaptive threshold (0..=255) for an RGBA8 buffer. -1 on bad input.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_otsu_threshold(rgba: *const u8, len: usize, width: u32, height: u32) -> i32 {
 if rgba.is_null() { return -1; }
 let buf = std::slice::from_raw_parts(rgba, len);
 match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => crate::analyze::otsu_threshold(&i) as i32, None => -1 }
}

/// Horizontal guide-line Y positions (rows of high edge density). Returns JSON [u32].
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_horizontal_lines(rgba: *const u8, len: usize, width: u32, height: u32, frac: f32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::analyze::horizontal_lines(&img, frac)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Vertical guide-line X positions (columns of high edge density). Returns JSON [u32].
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_vertical_lines(rgba: *const u8, len: usize, width: u32, height: u32, frac: f32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::analyze::vertical_lines(&img, frac)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Encode 12 or 13 ASCII digits into an EAN-13 module pattern. Returns JSON {pattern,modules} or null.
#[no_mangle]
pub extern "C" fn shotcore_ean13_encode(digits: *const c_char) -> *mut c_char {
 let s = match unsafe { cstr_to_string(digits) } { Some(s) => s, None => return std::ptr::null_mut() };
 let mut ds = Vec::new();
 for ch in s.chars() {
 match ch.to_digit(10) { Some(d) => ds.push(d as u8), None => return std::ptr::null_mut() }
 }
 match crate::ean13::encode(&ds) {
 Ok(bits) => {
 let pattern: String = bits.iter().map(|&b| if b { 49u8 as char } else { 48u8 as char }).collect();
 let v = serde_json::json!({"pattern": pattern, "modules": bits.len()});
 string_to_cstr(v.to_string())
 }
 Err(_) => std::ptr::null_mut(),
 }
}

/// Decode an EAN-13 barcode from an RGBA8 buffer. Returns the 13-digit string or null.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_ean13_decode(rgba: *const u8, len: usize, width: u32, height: u32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match crate::ean13::decode(&img) { Ok(s) => string_to_cstr(s), Err(_) => std::ptr::null_mut() }
}

#[cfg(test)]
mod analyze_ean_ffi_tests {
 use super::*;
 use std::ffi::{CStr, CString};
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn otsu_and_ean_over_ffi() {
 let white = vec![255u8; 8 * 8 * 4];
 let t = unsafe { shotcore_otsu_threshold(white.as_ptr(), white.len(), 8, 8) };
 assert!(t >= 0);
 let enc = CString::new("590123412345").unwrap();
 let j = unsafe { take(shotcore_ean13_encode(enc.as_ptr())) };
 assert!(j.contains("\"modules\":95"));
 let bits = crate::ean13::encode(&[5, 9, 0, 1, 2, 3, 4, 1, 2, 3, 4, 5]).unwrap();
 let img = crate::ean13::render(&bits, 3, 40, 10);
 let raw = img.as_raw();
 let dec = unsafe { take(shotcore_ean13_decode(raw.as_ptr(), raw.len(), img.width(), img.height())) };
 assert_eq!(dec, "5901234123457");
 }
}

/// Hough dominant lines from an RGBA8 buffer. Returns JSON [HoughLine] or null.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_hough_lines(rgba: *const u8, len: usize, width: u32, height: u32, edge_threshold: u8, max_lines: u32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::hough::detect_lines(&img, edge_threshold, max_lines as usize)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

/// Dominant Hough line angle (degrees, 0..180) for an RGBA8 buffer. -1.0 if no strong edges.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_hough_dominant_angle(rgba: *const u8, len: usize, width: u32, height: u32, edge_threshold: u8) -> f64 {
 if rgba.is_null() { return -1.0; }
 let buf = std::slice::from_raw_parts(rgba, len);
 match image::RgbaImage::from_raw(width, height, buf.to_vec()) {
 Some(i) => crate::hough::dominant_angle(&i, edge_threshold).unwrap_or(-1.0),
 None => -1.0,
 }
}

/// Skew correction angle (degrees) to straighten a tilted RGBA8 capture. 0.0 on bad input.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_skew_angle(rgba: *const u8, len: usize, width: u32, height: u32) -> f64 {
 if rgba.is_null() { return 0.0; }
 let buf = std::slice::from_raw_parts(rgba, len);
 match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => crate::deskew::estimate_skew_degrees(&i), None => 0.0 }
}

/// Harris corner points from an RGBA8 buffer. Returns JSON [Point] or null.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_corners(rgba: *const u8, len: usize, width: u32, height: u32, k: f64, rel_threshold: f64, min_distance: u32) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match serde_json::to_string(&crate::corners::detect(&img, k, rel_threshold, min_distance)) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() }
}

#[cfg(test)]
mod vision2_ffi_tests {
 use super::*;
 use std::ffi::CStr;
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn hough_skew_corners_over_ffi() {
 let w = 40u32;
 let h = 40u32;
 let mut buf = vec![255u8; (w * h * 4) as usize];
 for x in 0..w {
 let i = ((20 * w + x) * 4) as usize;
 buf[i] = 0; buf[i + 1] = 0; buf[i + 2] = 0;
 }
 let lines = unsafe { take(shotcore_hough_lines(buf.as_ptr(), buf.len(), w, h, 80, 4)) };
 assert!(lines.starts_with("["));
 let ang = unsafe { shotcore_hough_dominant_angle(buf.as_ptr(), buf.len(), w, h, 80) };
 assert!(ang >= 0.0);
 let skew = unsafe { shotcore_skew_angle(buf.as_ptr(), buf.len(), w, h) };
 assert!(skew.abs() <= 15.0);
 let corners = unsafe { take(shotcore_corners(buf.as_ptr(), buf.len(), w, h, 0.04, 0.2, 5)) };
 assert!(corners.starts_with("["));
 }
}

/// Perspective-unwarp a quadrilateral (corners_json = 4 Points TL,TR,BR,BL) from in_path into
/// an out_w x out_h image at out_path. 0 ok, negative on error.
#[no_mangle]
pub extern "C" fn shotcore_perspective_unwarp(in_path: *const c_char, out_path: *const c_char, corners_json: *const c_char, out_w: u32, out_h: u32) -> c_int {
 let ip = match unsafe { cstr_to_string(in_path) } { Some(s) => s, None => return ERR_ARG };
 let op = match unsafe { cstr_to_string(out_path) } { Some(s) => s, None => return ERR_ARG };
 let cj = match unsafe { cstr_to_string(corners_json) } { Some(s) => s, None => return ERR_ARG };
 let pts: Vec<crate::geometry::Point> = match serde_json::from_str(&cj) { Ok(v) => v, Err(_) => return ERR_JSON };
 if pts.len() != 4 { return ERR_ARG; }
 let corners = [pts[0], pts[1], pts[2], pts[3]];
 let img = match image::open(&ip) { Ok(i) => i.to_rgba8(), Err(_) => return ERR_IMAGE };
 match crate::perspective::unwarp(&img, corners, out_w, out_h).save(&op) { Ok(_) => OK, Err(_) => ERR_IMAGE }
}

/// Unsharp-mask sharpen in_path -> out_path. 0 ok, negative on error.
#[no_mangle]
pub extern "C" fn shotcore_unsharp(in_path: *const c_char, out_path: *const c_char, radius: u32, amount: f32) -> c_int {
 let ip = match unsafe { cstr_to_string(in_path) } { Some(s) => s, None => return ERR_ARG };
 let op = match unsafe { cstr_to_string(out_path) } { Some(s) => s, None => return ERR_ARG };
 let img = match image::open(&ip) { Ok(i) => i.to_rgba8(), Err(_) => return ERR_IMAGE };
 match crate::sharpen::unsharp(&img, radius, amount).save(&op) { Ok(_) => OK, Err(_) => ERR_IMAGE }
}

#[cfg(test)]
mod perspective_sharpen_ffi_tests {
 use super::*;
 use std::ffi::CString;
 #[test]
 fn unwarp_and_unsharp_over_ffi() {
 let dir = tempfile::tempdir().unwrap();
 let inp = dir.path().join("in.png");
 image::RgbaImage::from_pixel(20, 20, image::Rgba([120, 120, 120, 255])).save(&inp).unwrap();
 let cin = CString::new(inp.to_str().unwrap()).unwrap();
 let outp = dir.path().join("out.png");
 let cout = CString::new(outp.to_str().unwrap()).unwrap();
 assert_eq!(shotcore_unsharp(cin.as_ptr(), cout.as_ptr(), 2, 1.0), OK);
 assert!(outp.exists());
 let corners = CString::new("[{\"x\":0.0,\"y\":0.0},{\"x\":20.0,\"y\":0.0},{\"x\":20.0,\"y\":20.0},{\"x\":0.0,\"y\":20.0}]").unwrap();
 let outp2 = dir.path().join("out2.png");
 let cout2 = CString::new(outp2.to_str().unwrap()).unwrap();
 assert_eq!(shotcore_perspective_unwarp(cin.as_ptr(), cout2.as_ptr(), corners.as_ptr(), 20, 20), OK);
 assert!(outp2.exists());
 }
}

/// Auto-detect the document quadrilateral in an RGBA8 buffer. Returns JSON [Point;4] or null.
/// # Safety: rgba must point to at least len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn shotcore_detect_document(rgba: *const u8, len: usize, width: u32, height: u32, tol: u8) -> *mut c_char {
 if rgba.is_null() { return std::ptr::null_mut(); }
 let buf = std::slice::from_raw_parts(rgba, len);
 let img = match image::RgbaImage::from_raw(width, height, buf.to_vec()) { Some(i) => i, None => return std::ptr::null_mut() };
 match crate::docdetect::detect_document(&img, tol) {
 Some(q) => match serde_json::to_string(&q) { Ok(j) => string_to_cstr(j), Err(_) => std::ptr::null_mut() },
 None => std::ptr::null_mut(),
 }
}

/// Gray-world white balance in_path -> out_path. 0 ok, negative on error.
#[no_mangle]
pub extern "C" fn shotcore_white_balance(in_path: *const c_char, out_path: *const c_char) -> c_int {
 let ip = match unsafe { cstr_to_string(in_path) } { Some(s) => s, None => return ERR_ARG };
 let op = match unsafe { cstr_to_string(out_path) } { Some(s) => s, None => return ERR_ARG };
 let img = match image::open(&ip) { Ok(i) => i.to_rgba8(), Err(_) => return ERR_IMAGE };
 match crate::whitebalance::gray_world(&img).save(&op) { Ok(_) => OK, Err(_) => ERR_IMAGE }
}

/// Per-channel auto-contrast in_path -> out_path. 0 ok, negative on error.
#[no_mangle]
pub extern "C" fn shotcore_auto_color(in_path: *const c_char, out_path: *const c_char) -> c_int {
 let ip = match unsafe { cstr_to_string(in_path) } { Some(s) => s, None => return ERR_ARG };
 let op = match unsafe { cstr_to_string(out_path) } { Some(s) => s, None => return ERR_ARG };
 let img = match image::open(&ip) { Ok(i) => i.to_rgba8(), Err(_) => return ERR_IMAGE };
 match crate::whitebalance::auto_contrast(&img).save(&op) { Ok(_) => OK, Err(_) => ERR_IMAGE }
}

/// Export images (in_paths_json = JSON array of file paths) to a multi-page PDF at out_path.
#[no_mangle]
pub extern "C" fn shotcore_images_to_pdf(in_paths_json: *const c_char, out_path: *const c_char, quality: u32) -> c_int {
 let pj = match unsafe { cstr_to_string(in_paths_json) } { Some(s) => s, None => return ERR_ARG };
 let op = match unsafe { cstr_to_string(out_path) } { Some(s) => s, None => return ERR_ARG };
 let paths: Vec<String> = match serde_json::from_str(&pj) { Ok(v) => v, Err(_) => return ERR_JSON };
 let mut imgs = Vec::new();
 for path in &paths {
 match image::open(path) { Ok(i) => imgs.push(i.to_rgba8()), Err(_) => return ERR_IMAGE }
 }
 let bytes = crate::pdf::images_to_pdf(&imgs, quality as u8);
 match std::fs::write(&op, bytes) { Ok(_) => OK, Err(_) => ERR_IMAGE }
}

#[cfg(test)]
mod doc_wb_pdf_ffi_tests {
 use super::*;
 use std::ffi::{CStr, CString};
 unsafe fn take(p: *mut c_char) -> String {
 assert!(!p.is_null());
 let s = CStr::from_ptr(p).to_str().unwrap().to_owned();
 shotcore_string_free(p);
 s
 }
 #[test]
 fn detect_wb_pdf_over_ffi() {
 let dir = tempfile::tempdir().unwrap();
 let mut buf = vec![0u8; 40 * 40 * 4];
 for i in 0..40 * 40 { buf[i * 4 + 3] = 255; }
 for y in 10..30usize { for x in 10..30usize { let i = (y * 40 + x) * 4; buf[i] = 255; buf[i + 1] = 255; buf[i + 2] = 255; } }
 let j = unsafe { take(shotcore_detect_document(buf.as_ptr(), buf.len(), 40, 40, 60)) };
 assert!(j.contains("\"x\""));
 let inp = dir.path().join("in.png");
 image::RgbaImage::from_pixel(16, 16, image::Rgba([200, 100, 100, 255])).save(&inp).unwrap();
 let cin = CString::new(inp.to_str().unwrap()).unwrap();
 let wbout = dir.path().join("wb.png");
 let cwb = CString::new(wbout.to_str().unwrap()).unwrap();
 assert_eq!(shotcore_white_balance(cin.as_ptr(), cwb.as_ptr()), OK);
 assert!(wbout.exists());
 let paths = CString::new(format!("[{:?}]", inp.to_str().unwrap())).unwrap();
 let pdfout = dir.path().join("out.pdf");
 let cpdf = CString::new(pdfout.to_str().unwrap()).unwrap();
 assert_eq!(shotcore_images_to_pdf(paths.as_ptr(), cpdf.as_ptr(), 80), OK);
 assert!(pdfout.exists());
 }
}
