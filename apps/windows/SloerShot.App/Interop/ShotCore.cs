// P/Invoke bindings to the shotcore shared library (C ABI: core/shotcore/src/ffi.rs).
// The native shotcore.dll must sit next to the executable (see the .csproj copy step).
#nullable enable
using System;
using System.Runtime.InteropServices;

namespace SloerShot.Interop;

/// <summary>Thin managed wrapper over the shotcore C ABI.</summary>
public static class ShotCore
{
    private const string Lib = "shotcore";

    public const int Ok = 0;
    public const int ErrArg = -1;
    public const int ErrJson = -3;
    public const int ErrImage = -4;
    public const int ErrLicense = -5;
    public const int ErrExpired = -6;

    [DllImport(Lib, EntryPoint = "shotcore_version")]
    private static extern IntPtr VersionRaw();

    [DllImport(Lib, EntryPoint = "shotcore_string_free")]
    private static extern void StringFree(IntPtr ptr);

    [DllImport(Lib, EntryPoint = "shotcore_document_new")]
    private static extern IntPtr DocumentNewRaw(uint width, uint height);

    [DllImport(Lib, EntryPoint = "shotcore_export_png", CharSet = CharSet.Ansi)]
    private static extern int ExportPngRaw(string imagePath, string docJson, string outPath, string? fontPath);

    [DllImport(Lib, EntryPoint = "shotcore_beautify_png", CharSet = CharSet.Ansi)]
    private static extern int BeautifyPngRaw(string inPath, string outPath, string optionsJson);
 [DllImport(Lib, EntryPoint = "shotcore_beautify_framed_png", CharSet = CharSet.Ansi)]
 private static extern int BeautifyFramedRaw(string inPath, string outPath, string optionsJson);

    [DllImport(Lib, EntryPoint = "shotcore_resolve_selection", CharSet = CharSet.Ansi)]
    private static extern IntPtr ResolveSelectionRaw(string desktopJson, double x, double y, double w, double h);

    [DllImport(Lib, EntryPoint = "shotcore_history_search", CharSet = CharSet.Ansi)]
    private static extern IntPtr HistorySearchRaw(string storeJson, string query);

    [DllImport(Lib, EntryPoint = "shotcore_license_verify", CharSet = CharSet.Ansi)]
    private static extern int LicenseVerifyRaw(string publicKeyHex, string token, long now);

    // Marshal a returned UTF-8 char* into a managed string, then free the native buffer.
    private static string? TakeString(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero) return null;
        try { return Marshal.PtrToStringUTF8(ptr); }
        finally { StringFree(ptr); }
    }

    public static string Version() => TakeString(VersionRaw()) ?? "unknown";

    public static string? NewDocument(uint width, uint height) => TakeString(DocumentNewRaw(width, height));

    public static int Export(string imagePath, string docJson, string outPath, string? fontPath = null)
        => ExportPngRaw(imagePath, docJson, outPath, fontPath);

    public static int Beautify(string inPath, string outPath, string optionsJson)
        => BeautifyPngRaw(inPath, outPath, optionsJson);
 public static int BeautifyFramed(string inPath, string outPath, string optionsJson)
 => BeautifyFramedRaw(inPath, outPath, optionsJson);

    public static string? ResolveSelection(string desktopJson, double x, double y, double w, double h)
        => TakeString(ResolveSelectionRaw(desktopJson, x, y, w, h));

    public static string? HistorySearch(string storeJson, string query)
        => TakeString(HistorySearchRaw(storeJson, query));

    public static int VerifyLicense(string publicKeyHex, string token, long now)
        => LicenseVerifyRaw(publicKeyHex, token, now);

    // ---- Smart auto-redaction ----
    [DllImport(Lib, EntryPoint = "shotcore_auto_redact_into", CharSet = CharSet.Ansi)]
    private static extern IntPtr AutoRedactIntoRaw(string docJson, string ocrJson, uint style, uint strength);
    public static string? AutoRedactInto(string docJson, string ocrJson, uint style, uint strength)
        => TakeString(AutoRedactIntoRaw(docJson, ocrJson, style, strength));

    // ---- Effects (fx): apply a JSON-described op via the shared core ----
    [DllImport(Lib, EntryPoint = "shotcore_fx_apply", CharSet = CharSet.Ansi)]
    private static extern int FxApplyRaw(string inPath, string outPath, string opJson);
    /// <summary>Apply a JSON-described fx op (grayscale/sepia/invert/blur/vignette/brightness/contrast/flip/rotate/resize/scale/crop/spotlight/border/jpeg). Returns a status code.</summary>
    public static int FxApply(string inPath, string outPath, string opJson)
        => FxApplyRaw(inPath, outPath, opJson);

    // ---- Stateful editor handle (drive the shared editor from the canvas) ----
    [DllImport(Lib, EntryPoint = "shotcore_editor_new")]
    public static extern IntPtr EditorNew(uint width, uint height);
    [DllImport(Lib, EntryPoint = "shotcore_editor_free")]
    public static extern void EditorFree(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_set_tool")]
    public static extern void EditorSetTool(IntPtr ed, uint tool);
    [DllImport(Lib, EntryPoint = "shotcore_editor_pointer_down")]
    public static extern void EditorPointerDown(IntPtr ed, double x, double y);
    [DllImport(Lib, EntryPoint = "shotcore_editor_pointer_drag")]
    public static extern void EditorPointerDrag(IntPtr ed, double x, double y);
    [DllImport(Lib, EntryPoint = "shotcore_editor_pointer_up")]
    public static extern void EditorPointerUp(IntPtr ed, double x, double y);
    [DllImport(Lib, EntryPoint = "shotcore_editor_undo")]
    public static extern int EditorUndo(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_redo")]
    public static extern int EditorRedo(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_delete_selected")]
    public static extern int EditorDeleteSelected(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_bring_to_front")]
    public static extern int EditorBringToFront(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_send_to_back")]
    public static extern int EditorSendToBack(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_set_selected_text", CharSet = CharSet.Ansi)]
    public static extern int EditorSetSelectedText(IntPtr ed, string text);
    [DllImport(Lib, EntryPoint = "shotcore_editor_render_json")]
    private static extern IntPtr EditorRenderJsonRaw(IntPtr ed);
    public static string? EditorRenderJson(IntPtr ed) => TakeString(EditorRenderJsonRaw(ed));
    [DllImport(Lib, EntryPoint = "shotcore_editor_document_json")]
    private static extern IntPtr EditorDocumentJsonRaw(IntPtr ed);
    public static string? EditorDocumentJson(IntPtr ed) => TakeString(EditorDocumentJsonRaw(ed));
    [DllImport(Lib, EntryPoint = "shotcore_editor_can_undo")]
    public static extern int EditorCanUndo(IntPtr ed);
    [DllImport(Lib, EntryPoint = "shotcore_editor_can_redo")]
    public static extern int EditorCanRedo(IntPtr ed);
 // --- Recording engine (overlay geometry + scheduling via the tested core) ---
 [DllImport(Lib, EntryPoint = "shotcore_record_elapsed")]
 private static extern IntPtr RecordElapsedRaw(ulong ms);
 public static string? RecordElapsed(ulong ms) => TakeString(RecordElapsedRaw(ms));
 [DllImport(Lib, EntryPoint = "shotcore_record_frame_count")]
 public static extern ulong RecordFrameCount(uint fps, ulong durationMs);
 [DllImport(Lib, EntryPoint = "shotcore_record_camera_rect", CharSet = CharSet.Ansi)]
 private static extern IntPtr RecordCameraRectRaw(string camJson, uint width, uint height);
 public static string? RecordCameraRect(string camJson, uint width, uint height) => TakeString(RecordCameraRectRaw(camJson, width, height));
 [DllImport(Lib, EntryPoint = "shotcore_record_keystroke_rect", CharSet = CharSet.Ansi)]
 private static extern IntPtr RecordKeystrokeRectRaw(string dispJson, uint width, uint height, uint textLen);
 public static string? RecordKeystrokeRect(string dispJson, uint width, uint height, uint textLen) => TakeString(RecordKeystrokeRectRaw(dispJson, width, height, textLen));
 // --- QR generate + scan (pure-Rust core) ---
 [DllImport(Lib, EntryPoint = "shotcore_qr_encode", CharSet = CharSet.Ansi)]
 private static extern IntPtr QrEncodeRaw(string text);
 public static string? QrEncode(string text) => TakeString(QrEncodeRaw(text));
 [DllImport(Lib, EntryPoint = "shotcore_qr_encode_png", CharSet = CharSet.Ansi)]
 public static extern int QrEncodePng(string text, uint scale, uint quiet, string outPath);
 [DllImport(Lib, EntryPoint = "shotcore_qr_decode")]
 private static extern IntPtr QrDecodeRaw(byte[] rgba, UIntPtr len, uint width, uint height);
 public static string? QrDecode(byte[] rgba, uint width, uint height) => TakeString(QrDecodeRaw(rgba, (UIntPtr)rgba.Length, width, height));
 // --- PixelSnap snap-to-object ---
 [DllImport(Lib, EntryPoint = "shotcore_snap_object")]
 private static extern IntPtr SnapObjectRaw(byte[] rgba, UIntPtr len, uint width, uint height, double x, double y, double w, double h, byte tolerance);
 public static string? SnapObject(byte[] rgba, uint width, uint height, double x, double y, double w, double h, byte tolerance) => TakeString(SnapObjectRaw(rgba, (UIntPtr)rgba.Length, width, height, x, y, w, h, tolerance));
 // --- Crosshair + magnifier + eyedropper (advanced capture) ---
 [DllImport(Lib, EntryPoint = "shotcore_crosshair_lines")]
 private static extern IntPtr CrosshairLinesRaw(double cx, double cy, double w, double h);
 public static string? CrosshairLines(double cx, double cy, double w, double h) => TakeString(CrosshairLinesRaw(cx, cy, w, h));
 [DllImport(Lib, EntryPoint = "shotcore_pixel_hex")]
 private static extern IntPtr PixelHexRaw(byte[] rgba, UIntPtr len, uint width, uint height, uint x, uint y);
 public static string? PixelHex(byte[] rgba, uint width, uint height, uint x, uint y) => TakeString(PixelHexRaw(rgba, (UIntPtr)rgba.Length, width, height, x, y));
 [DllImport(Lib, EntryPoint = "shotcore_region_average_hex")]
 private static extern IntPtr RegionAverageHexRaw(byte[] rgba, UIntPtr len, uint width, uint height, double x, double y, double w, double h);
 public static string? RegionAverageHex(byte[] rgba, uint width, uint height, double x, double y, double w, double h) => TakeString(RegionAverageHexRaw(rgba, (UIntPtr)rgba.Length, width, height, x, y, w, h));
 // --- OCR capture flow (text recognition copy) ---
 [DllImport(Lib, EntryPoint = "shotcore_ocr_text_in_region", CharSet = CharSet.Ansi)]
 private static extern IntPtr OcrTextInRegionRaw(string ocrJson, double x, double y, double w, double h);
 public static string? OcrTextInRegion(string ocrJson, double x, double y, double w, double h) => TakeString(OcrTextInRegionRaw(ocrJson, x, y, w, h));
 [DllImport(Lib, EntryPoint = "shotcore_ocr_single_line", CharSet = CharSet.Ansi)]
 private static extern IntPtr OcrSingleLineRaw(string ocrJson);
 public static string? OcrSingleLine(string ocrJson) => TakeString(OcrSingleLineRaw(ocrJson));
 [DllImport(Lib, EntryPoint = "shotcore_ocr_word_count_region", CharSet = CharSet.Ansi)]
 public static extern long OcrWordCountRegion(string ocrJson, double x, double y, double w, double h);
 // --- Cloud share (CleanShot Cloud) ---
 [DllImport(Lib, EntryPoint = "shotcore_share_request", CharSet = CharSet.Ansi)]
 private static extern IntPtr ShareRequestRaw(string? password, long expiresAt, long maxViews);
 public static string? ShareRequestBody(string? password, long expiresAt, long maxViews) => TakeString(ShareRequestRaw(password, expiresAt, maxViews));
 [DllImport(Lib, EntryPoint = "shotcore_share_link", CharSet = CharSet.Ansi)]
 private static extern IntPtr ShareLinkRaw(string baseUrl, string responseJson);
 public static string? ShareLink(string baseUrl, string responseJson) => TakeString(ShareLinkRaw(baseUrl, responseJson));
 // --- Settings + Quick Access Overlay ---
 [DllImport(Lib, EntryPoint = "shotcore_settings_default")]
 private static extern IntPtr SettingsDefaultRaw();
 public static string? SettingsDefault() => TakeString(SettingsDefaultRaw());
 [DllImport(Lib, EntryPoint = "shotcore_settings_normalize", CharSet = CharSet.Ansi)]
 private static extern IntPtr SettingsNormalizeRaw(string json);
 public static string? SettingsNormalize(string json) => TakeString(SettingsNormalizeRaw(json));
 [DllImport(Lib, EntryPoint = "shotcore_overlay_default")]
 private static extern IntPtr OverlayDefaultRaw();
 public static string? OverlayDefault() => TakeString(OverlayDefaultRaw());
 [DllImport(Lib, EntryPoint = "shotcore_overlay_should_close", CharSet = CharSet.Ansi)]
 public static extern int OverlayShouldClose(string overlayJson, long now, long lastInteraction);
 // --- Perceptual hashing (history dedupe / find-similar) ---
 [DllImport(Lib, EntryPoint = "shotcore_ahash")]
 private static extern ulong AHashRaw(byte[] rgba, UIntPtr len, uint width, uint height);
 public static ulong AHash(byte[] rgba, uint width, uint height) => AHashRaw(rgba, (UIntPtr)rgba.Length, width, height);
 [DllImport(Lib, EntryPoint = "shotcore_dhash")]
 private static extern ulong DHashRaw(byte[] rgba, UIntPtr len, uint width, uint height);
 public static ulong DHash(byte[] rgba, uint width, uint height) => DHashRaw(rgba, (UIntPtr)rgba.Length, width, height);
 [DllImport(Lib, EntryPoint = "shotcore_hamming")]
 public static extern uint Hamming(ulong a, ulong b);
 // --- Dominant color extraction (Background tool palette) ---
 [DllImport(Lib, EntryPoint = "shotcore_dominant_colors")]
 private static extern IntPtr DominantColorsRaw(byte[] rgba, UIntPtr len, uint width, uint height, uint k);
 public static string? DominantColors(byte[] rgba, uint width, uint height, uint k) => TakeString(DominantColorsRaw(rgba, (UIntPtr)rgba.Length, width, height, k));
 // --- Vision: image diff, edge count, region segmentation ---
 [DllImport(Lib, EntryPoint = "shotcore_image_diff")]
 private static extern IntPtr ImageDiffRaw(byte[] a, UIntPtr aLen, uint aw, uint ah, byte[] b, UIntPtr bLen, uint bw, uint bh, byte tol);
 public static string? ImageDiff(byte[] a, uint aw, uint ah, byte[] b, uint bw, uint bh, byte tol) => TakeString(ImageDiffRaw(a, (UIntPtr)a.Length, aw, ah, b, (UIntPtr)b.Length, bw, bh, tol));
 [DllImport(Lib, EntryPoint = "shotcore_edge_count")]
 private static extern ulong EdgeCountRaw(byte[] rgba, UIntPtr len, uint width, uint height, byte threshold);
 public static ulong EdgeCount(byte[] rgba, uint width, uint height, byte threshold) => EdgeCountRaw(rgba, (UIntPtr)rgba.Length, width, height, threshold);
 [DllImport(Lib, EntryPoint = "shotcore_segment_regions")]
 private static extern IntPtr SegmentRegionsRaw(byte[] rgba, UIntPtr len, uint width, uint height, byte tol, uint minArea);
 public static string? SegmentRegions(byte[] rgba, uint width, uint height, byte tol, uint minArea) => TakeString(SegmentRegionsRaw(rgba, (UIntPtr)rgba.Length, width, height, tol, minArea));
 // --- Adaptive threshold + guide-line detection + EAN-13 barcode ---
 [DllImport(Lib, EntryPoint = "shotcore_otsu_threshold")]
 private static extern int OtsuThresholdRaw(byte[] rgba, UIntPtr len, uint width, uint height);
 public static int OtsuThreshold(byte[] rgba, uint width, uint height) => OtsuThresholdRaw(rgba, (UIntPtr)rgba.Length, width, height);
 [DllImport(Lib, EntryPoint = "shotcore_horizontal_lines")]
 private static extern IntPtr HorizontalLinesRaw(byte[] rgba, UIntPtr len, uint width, uint height, float frac);
 public static string? HorizontalLines(byte[] rgba, uint width, uint height, float frac) => TakeString(HorizontalLinesRaw(rgba, (UIntPtr)rgba.Length, width, height, frac));
 [DllImport(Lib, EntryPoint = "shotcore_vertical_lines")]
 private static extern IntPtr VerticalLinesRaw(byte[] rgba, UIntPtr len, uint width, uint height, float frac);
 public static string? VerticalLines(byte[] rgba, uint width, uint height, float frac) => TakeString(VerticalLinesRaw(rgba, (UIntPtr)rgba.Length, width, height, frac));
 [DllImport(Lib, EntryPoint = "shotcore_ean13_encode", CharSet = CharSet.Ansi)]
 private static extern IntPtr Ean13EncodeRaw(string digits);
 public static string? Ean13Encode(string digits) => TakeString(Ean13EncodeRaw(digits));
 [DllImport(Lib, EntryPoint = "shotcore_ean13_decode")]
 private static extern IntPtr Ean13DecodeRaw(byte[] rgba, UIntPtr len, uint width, uint height);
 public static string? Ean13Decode(byte[] rgba, uint width, uint height) => TakeString(Ean13DecodeRaw(rgba, (UIntPtr)rgba.Length, width, height));
 // --- Hough lines + auto-deskew + Harris corners ---
 [DllImport(Lib, EntryPoint = "shotcore_hough_lines")]
 private static extern IntPtr HoughLinesRaw(byte[] rgba, UIntPtr len, uint width, uint height, byte edgeThreshold, uint maxLines);
 public static string? HoughLines(byte[] rgba, uint width, uint height, byte edgeThreshold, uint maxLines) => TakeString(HoughLinesRaw(rgba, (UIntPtr)rgba.Length, width, height, edgeThreshold, maxLines));
 [DllImport(Lib, EntryPoint = "shotcore_hough_dominant_angle")]
 private static extern double HoughDominantAngleRaw(byte[] rgba, UIntPtr len, uint width, uint height, byte edgeThreshold);
 public static double HoughDominantAngle(byte[] rgba, uint width, uint height, byte edgeThreshold) => HoughDominantAngleRaw(rgba, (UIntPtr)rgba.Length, width, height, edgeThreshold);
 [DllImport(Lib, EntryPoint = "shotcore_skew_angle")]
 private static extern double SkewAngleRaw(byte[] rgba, UIntPtr len, uint width, uint height);
 public static double SkewAngle(byte[] rgba, uint width, uint height) => SkewAngleRaw(rgba, (UIntPtr)rgba.Length, width, height);
 [DllImport(Lib, EntryPoint = "shotcore_corners")]
 private static extern IntPtr CornersRaw(byte[] rgba, UIntPtr len, uint width, uint height, double k, double relThreshold, uint minDistance);
 public static string? Corners(byte[] rgba, uint width, uint height, double k, double relThreshold, uint minDistance) => TakeString(CornersRaw(rgba, (UIntPtr)rgba.Length, width, height, k, relThreshold, minDistance));
 // --- Perspective unwarp (document flatten) + unsharp sharpen (path-based) ---
 [DllImport(Lib, EntryPoint = "shotcore_perspective_unwarp", CharSet = CharSet.Ansi)]
 public static extern int PerspectiveUnwarp(string inPath, string outPath, string cornersJson, uint outW, uint outH);
 [DllImport(Lib, EntryPoint = "shotcore_unsharp", CharSet = CharSet.Ansi)]
 public static extern int Unsharp(string inPath, string outPath, uint radius, float amount);
 // --- Auto document detection + white balance/auto-color + multi-page PDF ---
 [DllImport(Lib, EntryPoint = "shotcore_detect_document")]
 private static extern IntPtr DetectDocumentRaw(byte[] rgba, UIntPtr len, uint width, uint height, byte tol);
 public static string? DetectDocument(byte[] rgba, uint width, uint height, byte tol) => TakeString(DetectDocumentRaw(rgba, (UIntPtr)rgba.Length, width, height, tol));
 [DllImport(Lib, EntryPoint = "shotcore_white_balance", CharSet = CharSet.Ansi)]
 public static extern int WhiteBalance(string inPath, string outPath);
 [DllImport(Lib, EntryPoint = "shotcore_auto_color", CharSet = CharSet.Ansi)]
 public static extern int AutoColor(string inPath, string outPath);
 [DllImport(Lib, EntryPoint = "shotcore_images_to_pdf", CharSet = CharSet.Ansi)]
 public static extern int ImagesToPdf(string inPathsJson, string outPath, uint quality);

[DllImport(Lib, EntryPoint = "shotcore_editor_set_stroke_color")]
public static extern void EditorSetStrokeColor(IntPtr ed, byte r, byte g, byte b, byte a);
[DllImport(Lib, EntryPoint = "shotcore_editor_set_stroke_width")]
public static extern void EditorSetStrokeWidth(IntPtr ed, double width);

 [DllImport(Lib, EntryPoint = "shotcore_editor_set_style_json", CharSet = CharSet.Ansi)]
 public static extern int EditorSetStyleJson(IntPtr ed, string json);

[DllImport(Lib, EntryPoint = "shotcore_deskew", CharSet = CharSet.Ansi)]
public static extern int Deskew(string inPath, string outPath);

 [DllImport(Lib, EntryPoint = "shotcore_encode_gif_dir", CharSet = CharSet.Ansi)]
 public static extern int EncodeGifDir(string dirPath, uint fps, uint maxWidth, string outPath);

 [DllImport(Lib, EntryPoint = "shotcore_extract_links", CharSet = CharSet.Ansi)]
 private static extern IntPtr ExtractLinksRaw(string text);
 public static string? ExtractLinks(string text) => TakeString(ExtractLinksRaw(text));
 [DllImport(Lib, EntryPoint = "shotcore_table_csv", CharSet = CharSet.Ansi)]
 private static extern IntPtr TableCsvRaw(string ocrJson, double colTolerance);
 public static string? TableCsv(string ocrJson, double colTolerance) => TakeString(TableCsvRaw(ocrJson, colTolerance));
 [DllImport(Lib, EntryPoint = "shotcore_table_markdown", CharSet = CharSet.Ansi)]
 private static extern IntPtr TableMarkdownRaw(string ocrJson, double colTolerance);
 public static string? TableMarkdown(string ocrJson, double colTolerance) => TakeString(TableMarkdownRaw(ocrJson, colTolerance));
 [DllImport(Lib, EntryPoint = "shotcore_combine_stack_vertical", CharSet = CharSet.Ansi)]
 private static extern IntPtr CombineStackVerticalRaw(string sizesJson, uint gap);
 public static string? CombineStackVertical(string sizesJson, uint gap) => TakeString(CombineStackVerticalRaw(sizesJson, gap));

 // ShareX-style custom uploader (engine lives in the Rust core: custom_uploader.rs).
 [DllImport(Lib, EntryPoint = "shotcore_custom_uploader_build_plan", CharSet = CharSet.Ansi)]
 private static extern IntPtr CustomUploaderBuildPlanRaw(string configJson, string input, string filename);
 public static string? CustomUploaderBuildPlan(string configJson, string input, string filename) => TakeString(CustomUploaderBuildPlanRaw(configJson, input, filename));
 [DllImport(Lib, EntryPoint = "shotcore_custom_uploader_resolve_response", CharSet = CharSet.Ansi)]
 private static extern IntPtr CustomUploaderResolveResponseRaw(string configJson, string response, string headersJson, string input, string filename);
 public static string? CustomUploaderResolveResponse(string configJson, string response, string headersJson, string input, string filename) => TakeString(CustomUploaderResolveResponseRaw(configJson, response, headersJson, input, filename));
}
