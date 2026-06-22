import CShotCore
import Foundation

/// Swift wrapper over the shotcore C ABI (see Sources/CShotCore/shotcore.h).
enum ShotCore {
    static func version() -> String {
        guard let ptr = shotcore_version() else { return "unknown" }
        defer { shotcore_string_free(ptr) }
        return String(cString: ptr)
    }

    static func newDocument(width: UInt32, height: UInt32) -> String? {
        guard let ptr = shotcore_document_new(width, height) else { return nil }
        defer { shotcore_string_free(ptr) }
        return String(cString: ptr)
    }

    static func export(imagePath: String, docJson: String, outPath: String, fontPath: String? = nil) -> Int32 {
        shotcore_export_png(imagePath, docJson, outPath, fontPath)
    }

    static func beautify(inPath: String, outPath: String, optionsJson: String) -> Int32 {
 shotcore_beautify_png(inPath, outPath, optionsJson)
 }
 static func beautifyFramed(inPath: String, outPath: String, optionsJson: String) -> Int32 {
 shotcore_beautify_framed_png(inPath, outPath, optionsJson)
 }

    static func resolveSelection(desktopJson: String, x: Double, y: Double, w: Double, h: Double) -> String? {
        guard let ptr = shotcore_resolve_selection(desktopJson, x, y, w, h) else { return nil }
        defer { shotcore_string_free(ptr) }
        return String(cString: ptr)
    }

    static func verifyLicense(publicKeyHex: String, token: String, now: Int64) -> Int32 {
        shotcore_license_verify(publicKeyHex, token, now)
    }
}

extension ShotCore {
    /// Detect sensitive data in `ocrJson` and return `docJson` with Redact annotations added.
    /// style: 0 = blur, 1 = pixelate.
    static func autoRedactInto(docJson: String, ocrJson: String, style: UInt32, strength: UInt32) -> String? {
        guard let ptr = shotcore_auto_redact_into(docJson, ocrJson, style, strength) else { return nil }
        defer { shotcore_string_free(ptr) }
        return String(cString: ptr)
    }
}

/// RAII wrapper around the shared editor controller. The canvas forwards pointer
/// events here and renders `renderJson()`.
final class EditorHandle {
    private let ptr: OpaquePointer

    init?(width: UInt32, height: UInt32) {
        guard let p = shotcore_editor_new(width, height) else { return nil }
        ptr = p
    }

    deinit { shotcore_editor_free(ptr) }

    func setTool(_ tool: UInt32) { shotcore_editor_set_tool(ptr, tool) }
    func pointerDown(_ x: Double, _ y: Double) { shotcore_editor_pointer_down(ptr, x, y) }
    func pointerDrag(_ x: Double, _ y: Double) { shotcore_editor_pointer_drag(ptr, x, y) }
    func pointerUp(_ x: Double, _ y: Double) { shotcore_editor_pointer_up(ptr, x, y) }
    @discardableResult func undo() -> Bool { shotcore_editor_undo(ptr) != 0 }
    @discardableResult func redo() -> Bool { shotcore_editor_redo(ptr) != 0 }
    @discardableResult func deleteSelected() -> Bool { shotcore_editor_delete_selected(ptr) != 0 }
    @discardableResult func bringToFront() -> Bool { shotcore_editor_bring_to_front(ptr) != 0 }
    @discardableResult func sendToBack() -> Bool { shotcore_editor_send_to_back(ptr) != 0 }
    @discardableResult func setSelectedText(_ text: String) -> Bool { shotcore_editor_set_selected_text(ptr, text) != 0 }
 func setStrokeColor(r: UInt8, g: UInt8, b: UInt8, a: UInt8) { shotcore_editor_set_stroke_color(ptr, r, g, b, a) }
 func setStrokeWidth(_ w: Double) { shotcore_editor_set_stroke_width(ptr, w) }
 @discardableResult func setStyleJson(_ json: String) -> Bool { shotcore_editor_set_style_json(ptr, json) != 0 }
    func canUndo() -> Bool { shotcore_editor_can_undo(ptr) != 0 }
    func canRedo() -> Bool { shotcore_editor_can_redo(ptr) != 0 }
    func renderJson() -> String? {
        guard let p = shotcore_editor_render_json(ptr) else { return nil }
        defer { shotcore_string_free(p) }
        return String(cString: p)
    }
    func documentJson() -> String? {
        guard let p = shotcore_editor_document_json(ptr) else { return nil }
        defer { shotcore_string_free(p) }
        return String(cString: p)
    }
}

extension ShotCore {
    /// Apply a JSON-described fx op (grayscale/sepia/invert/blur/vignette/brightness/contrast/flip/rotate/resize/scale/crop/spotlight/border/jpeg) from inPath to outPath. Returns a status code.
    static func fxApply(inPath: String, outPath: String, opJson: String) -> Int32 {
        shotcore_fx_apply(inPath, outPath, opJson)
    }
}
