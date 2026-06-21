import CShotCore
import Foundation

/// Take ownership of a heap C string returned by the core, copy it into a Swift
/// String, and free it. Returns nil when the core returned null.
fileprivate func takeString(_ ptr: UnsafeMutablePointer<CChar>?) -> String? {
 guard let ptr = ptr else { return nil }
 defer { shotcore_string_free(ptr) }
 return String(cString: ptr)
}

// MARK: - Strongly-typed option codes (match the C ABI integer arguments).

enum CropRatio: UInt32 { case free = 0, square = 1, r4x3 = 2, r3x2 = 3, r16x9 = 4, r5x4 = 5, r9x16 = 6 }
enum AlignEdge: UInt32 { case left = 0, hCenter = 1, right = 2, top = 3, vCenter = 4, bottom = 5 }
enum MockupFrame: UInt32 { case browser = 0, window = 1, phone = 2, laptop = 3 }
enum MaskShapeKind: UInt32 { case ellipse = 0, roundedRect = 1 }
enum RedactKind: UInt32 { case blur = 0, pixelate = 1 }

// MARK: - Remaining stateless C ABI wrappers. Each returns JSON (or plain text) the
// caller decodes, mirroring core/shotcore/src/ffi.rs. See shotcore.h for contracts.
extension ShotCore {
 static func historySearch(storeJson: String, query: String) -> String? {
 takeString(shotcore_history_search(storeJson, query))
 }

 static func cropConstrain(x: Double, y: Double, w: Double, h: Double, ratio: CropRatio) -> String? {
 takeString(shotcore_crop_constrain(x, y, w, h, ratio.rawValue))
 }

 static func classifyPayload(_ raw: String) -> String? {
 takeString(shotcore_classify_payload(raw))
 }

 static func extractLinks(from text: String) -> String? {
 takeString(shotcore_extract_links(text))
 }

 /// Sample one RGBA8 pixel from a buffer; returns a Color JSON.
 static func eyedrop(rgba: [UInt8], width: UInt32, height: UInt32, x: UInt32, y: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in
 takeString(shotcore_palette_eyedrop(buf.baseAddress, UInt(buf.count), width, height, x, y))
 }
 }

 static func videoeditOutput(editJson: String, sourceMs: UInt64, w: UInt32, h: UInt32, channels: UInt32) -> String? {
 takeString(shotcore_videoedit_output(editJson, sourceMs, w, h, channels))
 }

 static func hashSharePassword(_ plain: String) -> String? {
 takeString(shotcore_share_hash_password(plain))
 }

 static func autoBalance(contentW: Double, contentH: Double, canvasW: Double, canvasH: Double) -> String? {
 takeString(shotcore_auto_balance(contentW, contentH, canvasW, canvasH))
 }

 static func combineStackVertical(sizesJson: String, gap: UInt32) -> String? {
 takeString(shotcore_combine_stack_vertical(sizesJson, gap))
 }

 static func curvedArrowPath(from: CGPoint, to: CGPoint, bow: Double, segments: UInt32) -> String? {
 takeString(shotcore_curved_arrow_path(Double(from.x), Double(from.y), Double(to.x), Double(to.y), bow, segments))
 }

 static func smartHighlight(ocrJson: String, x: Double, y: Double, w: Double, h: Double) -> String? {
 takeString(shotcore_smart_highlight(ocrJson, x, y, w, h))
 }

 static func parseURLScheme(_ url: String) -> String? {
 takeString(shotcore_urlscheme_parse(url))
 }

 static func printPageSlices(contentW: UInt32, contentH: UInt32, pageH: UInt32, overlap: UInt32) -> String? {
 takeString(shotcore_print_page_slices(contentW, contentH, pageH, overlap))
 }

 static func scaleTo1xPng(inPath: String, outPath: String, scaleFactor: Float) -> Int32 {
 shotcore_scale_to_1x_png(inPath, outPath, scaleFactor)
 }

 static func contrast(hexA: String, hexB: String) -> String? {
 takeString(shotcore_contrast(hexA, hexB))
 }

 static func guideMarkdown(guideJson: String) -> String? {
 takeString(shotcore_guide_markdown(guideJson))
 }

 static func guideHTML(guideJson: String) -> String? {
 takeString(shotcore_guide_html(guideJson))
 }

 static func codeshotSize(cardJson: String, charW: Float, lineH: Float) -> String? {
 takeString(shotcore_codeshot_size(cardJson, charW, lineH))
 }

 static func measure(from: CGPoint, to: CGPoint) -> String? {
 takeString(shotcore_measure(Double(from.x), Double(from.y), Double(to.x), Double(to.y)))
 }

 static func autoZoom(clicksJson: String, scale: Double, easeMs: UInt64, holdMs: UInt64) -> String? {
 takeString(shotcore_auto_zoom(clicksJson, scale, easeMs, holdMs))
 }

 static func tableCSV(ocrJson: String, colTolerance: Double) -> String? {
 takeString(shotcore_table_csv(ocrJson, colTolerance))
 }

 static func tableMarkdown(ocrJson: String, colTolerance: Double) -> String? {
 takeString(shotcore_table_markdown(ocrJson, colTolerance))
 }

 static func captionsSRT(segmentsJson: String) -> String? {
 takeString(shotcore_captions_srt(segmentsJson))
 }

 static func captionsVTT(segmentsJson: String) -> String? {
 takeString(shotcore_captions_vtt(segmentsJson))
 }

 static func autoTags(text: String, max: UInt32) -> String? {
 takeString(shotcore_autotag(text, max))
 }

 static func svg(docJson: String) -> String? {
 takeString(shotcore_svg(docJson))
 }

 static func align(rectsJson: String, edge: AlignEdge) -> String? {
 takeString(shotcore_align(rectsJson, edge.rawValue))
 }

 static func mockupSize(frame: MockupFrame, contentW: UInt32, contentH: UInt32) -> String? {
 takeString(shotcore_mockup_size(frame.rawValue, contentW, contentH))
 }

 static func maskPng(inPath: String, outPath: String, shape: MaskShapeKind, radius: Float) -> Int32 {
 shotcore_mask_png(inPath, outPath, shape.rawValue, radius)
 }
}

extension ShotCore {
 /// Snap a search rect to the tight bounds of the object inside an RGBA8 buffer
 /// (PixelSnap-style). Returns a Rect JSON.
 static func snapObject(rgba: [UInt8], width: UInt32, height: UInt32, search: CGRect, tolerance: UInt8) -> String? {
 rgba.withUnsafeBufferPointer { buf in
 takeString(shotcore_snap_object(buf.baseAddress, UInt(buf.count), width, height,
 Double(search.minX), Double(search.minY),
 Double(search.width), Double(search.height), tolerance))
 }
 }
}

extension ShotCore {
 /// Encode text into a QR module grid. Returns JSON {version, size, modules:[strings of 0/1]}.
 /// Fully on-device, pure Rust - no native QR dependency.
 static func qrEncode(_ text: String) -> String? {
 takeString(shotcore_qr_encode(text))
 }
 /// Decode the first QR symbol in an RGBA8 buffer. Returns JSON {text, kind}.
 static func qrDecode(rgba: [UInt8], width: UInt32, height: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in
 takeString(shotcore_qr_decode(buf.baseAddress, UInt(buf.count), width, height))
 }
 }
}

extension ShotCore {
 // Recording engine: overlay geometry + scheduling from the tested core.
 static func recordElapsed(_ ms: UInt64) -> String? {
 takeString(shotcore_record_elapsed(ms))
 }
 static func recordFrameCount(fps: UInt32, durationMs: UInt64) -> UInt64 {
 shotcore_record_frame_count(fps, durationMs)
 }
 static func recordCameraRect(cameraJson: String, width: UInt32, height: UInt32) -> String? {
 takeString(shotcore_record_camera_rect(cameraJson, width, height))
 }
 static func recordKeystrokeRect(displayJson: String, width: UInt32, height: UInt32, textLen: UInt32) -> String? {
 takeString(shotcore_record_keystroke_rect(displayJson, width, height, textLen))
 }
}

extension ShotCore {
 // Crosshair + magnifier + eyedropper readouts (advanced capture).
 static func crosshairLines(cx: Double, cy: Double, w: Double, h: Double) -> String? {
 takeString(shotcore_crosshair_lines(cx, cy, w, h))
 }
 static func pixelHex(rgba: [UInt8], width: UInt32, height: UInt32, x: UInt32, y: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_pixel_hex(buf.baseAddress, UInt(buf.count), width, height, x, y)) }
 }
 static func regionAverageHex(rgba: [UInt8], width: UInt32, height: UInt32, x: Double, y: Double, w: Double, h: Double) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_region_average_hex(buf.baseAddress, UInt(buf.count), width, height, x, y, w, h)) }
 }
}

extension ShotCore {
 // OCR capture flow: format a recognized OcrResult JSON for the clipboard.
 static func ocrTextInRegion(ocrJson: String, x: Double, y: Double, w: Double, h: Double) -> String? {
 takeString(shotcore_ocr_text_in_region(ocrJson, x, y, w, h))
 }
 static func ocrSingleLine(_ ocrJson: String) -> String? {
 takeString(shotcore_ocr_single_line(ocrJson))
 }
 static func ocrWordCountRegion(ocrJson: String, x: Double, y: Double, w: Double, h: Double) -> Int64 {
 shotcore_ocr_word_count_region(ocrJson, x, y, w, h)
 }
}

extension ShotCore {
 // Cloud share: build the request body and the absolute link via the tested core.
 static func shareRequestBody(password: String?, expiresAt: Int64, maxViews: Int64) -> String? {
 takeString(shotcore_share_request(password, expiresAt, maxViews))
 }
 static func shareLink(baseUrl: String, responseJson: String) -> String? {
 takeString(shotcore_share_link(baseUrl, responseJson))
 }
}

extension ShotCore {
 // App settings + Quick Access Overlay (tested core models).
 static func settingsDefault() -> String? { takeString(shotcore_settings_default()) }
 static func settingsNormalize(_ json: String) -> String? { takeString(shotcore_settings_normalize(json)) }
 static func overlayDefault() -> String? { takeString(shotcore_overlay_default()) }
 static func overlayShouldClose(overlayJson: String, now: Int64, lastInteraction: Int64) -> Int32 {
 shotcore_overlay_should_close(overlayJson, now, lastInteraction)
 }
}

extension ShotCore {
 // Perceptual hashing for near-duplicate detection in the capture history.
 static func aHash(rgba: [UInt8], width: UInt32, height: UInt32) -> UInt64 {
 rgba.withUnsafeBufferPointer { buf in shotcore_ahash(buf.baseAddress, UInt(buf.count), width, height) }
 }
 static func dHash(rgba: [UInt8], width: UInt32, height: UInt32) -> UInt64 {
 rgba.withUnsafeBufferPointer { buf in shotcore_dhash(buf.baseAddress, UInt(buf.count), width, height) }
 }
 static func hamming(_ a: UInt64, _ b: UInt64) -> UInt32 { shotcore_hamming(a, b) }
}

extension ShotCore {
 // Dominant colors (median-cut) for matching gradient backgrounds.
 static func dominantColors(rgba: [UInt8], width: UInt32, height: UInt32, k: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_dominant_colors(buf.baseAddress, UInt(buf.count), width, height, k)) }
 }
}

extension ShotCore {
 // Vision helpers: before/after diff, edge density, region segmentation.
 static func imageDiff(a: [UInt8], aw: UInt32, ah: UInt32, b: [UInt8], bw: UInt32, bh: UInt32, tol: UInt8) -> String? {
 a.withUnsafeBufferPointer { ap in b.withUnsafeBufferPointer { bp in
 takeString(shotcore_image_diff(ap.baseAddress, UInt(ap.count), aw, ah, bp.baseAddress, UInt(bp.count), bw, bh, tol))
 } }
 }
 static func edgeCount(rgba: [UInt8], width: UInt32, height: UInt32, threshold: UInt8) -> UInt64 {
 rgba.withUnsafeBufferPointer { buf in shotcore_edge_count(buf.baseAddress, UInt(buf.count), width, height, threshold) }
 }
 static func segmentRegions(rgba: [UInt8], width: UInt32, height: UInt32, tol: UInt8, minArea: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_segment_regions(buf.baseAddress, UInt(buf.count), width, height, tol, minArea)) }
 }
}

extension ShotCore {
 // Adaptive threshold + guide detection + EAN-13 barcode.
 static func otsuThreshold(rgba: [UInt8], width: UInt32, height: UInt32) -> Int32 {
 rgba.withUnsafeBufferPointer { buf in shotcore_otsu_threshold(buf.baseAddress, UInt(buf.count), width, height) }
 }
 static func horizontalLines(rgba: [UInt8], width: UInt32, height: UInt32, frac: Float) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_horizontal_lines(buf.baseAddress, UInt(buf.count), width, height, frac)) }
 }
 static func verticalLines(rgba: [UInt8], width: UInt32, height: UInt32, frac: Float) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_vertical_lines(buf.baseAddress, UInt(buf.count), width, height, frac)) }
 }
 static func ean13Encode(_ digits: String) -> String? { takeString(shotcore_ean13_encode(digits)) }
 static func ean13Decode(rgba: [UInt8], width: UInt32, height: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_ean13_decode(buf.baseAddress, UInt(buf.count), width, height)) }
 }
}

extension ShotCore {
 // Hough lines + auto-deskew + Harris corners.
 static func houghLines(rgba: [UInt8], width: UInt32, height: UInt32, edgeThreshold: UInt8, maxLines: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_hough_lines(buf.baseAddress, UInt(buf.count), width, height, edgeThreshold, maxLines)) }
 }
 static func houghDominantAngle(rgba: [UInt8], width: UInt32, height: UInt32, edgeThreshold: UInt8) -> Double {
 rgba.withUnsafeBufferPointer { buf in shotcore_hough_dominant_angle(buf.baseAddress, UInt(buf.count), width, height, edgeThreshold) }
 }
 static func skewAngle(rgba: [UInt8], width: UInt32, height: UInt32) -> Double {
 rgba.withUnsafeBufferPointer { buf in shotcore_skew_angle(buf.baseAddress, UInt(buf.count), width, height) }
 }
 static func corners(rgba: [UInt8], width: UInt32, height: UInt32, k: Double, relThreshold: Double, minDistance: UInt32) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_corners(buf.baseAddress, UInt(buf.count), width, height, k, relThreshold, minDistance)) }
 }
}

extension ShotCore {
 // Perspective unwarp (document flatten) + unsharp sharpen (path-based).
 @discardableResult
 static func perspectiveUnwarp(inPath: String, outPath: String, cornersJson: String, outW: UInt32, outH: UInt32) -> Int32 {
 shotcore_perspective_unwarp(inPath, outPath, cornersJson, outW, outH)
 }
 @discardableResult
 static func unsharp(inPath: String, outPath: String, radius: UInt32, amount: Float) -> Int32 {
 shotcore_unsharp(inPath, outPath, radius, amount)
 }
}

extension ShotCore {
 // Auto document detection + white balance / auto-color + multi-page PDF.
 static func detectDocument(rgba: [UInt8], width: UInt32, height: UInt32, tol: UInt8) -> String? {
 rgba.withUnsafeBufferPointer { buf in takeString(shotcore_detect_document(buf.baseAddress, UInt(buf.count), width, height, tol)) }
 }
 @discardableResult
 static func whiteBalance(inPath: String, outPath: String) -> Int32 { shotcore_white_balance(inPath, outPath) }
 @discardableResult
 static func autoColor(inPath: String, outPath: String) -> Int32 { shotcore_auto_color(inPath, outPath) }
 @discardableResult
 static func imagesToPdf(inPathsJson: String, outPath: String, quality: UInt32) -> Int32 { shotcore_images_to_pdf(inPathsJson, outPath, quality) }
}
