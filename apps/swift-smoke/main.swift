import CShotCore
import Foundation

func takeString(_ p: UnsafeMutablePointer<CChar>?) -> String? {
 guard let p = p else { return nil }
 defer { shotcore_string_free(p) }
 return String(cString: p)
}

var passed = 0
func check(_ name: String, _ cond: Bool) {
 if cond { passed += 1; print("ok - \(name)") }
 else { print("FAIL - \(name)"); exit(1) }
}

let version = takeString(shotcore_version()) ?? ""
check("version non-empty", !version.isEmpty)
print(" shotcore version: \(version)")

let doc = takeString(shotcore_document_new(800, 600)) ?? ""
check("document_new returns json", doc.contains("annotations"))

let crop = takeString(shotcore_crop_constrain(0, 0, 200, 100, 1)) ?? ""
check("crop square -> w 100", crop.contains("\"w\":100"))

let con = takeString(shotcore_contrast("#000000", "#FFFFFF")) ?? ""
check("contrast black/white AAA", con.contains("AAA"))

let srt = takeString(shotcore_captions_srt("[{\"start_ms\":1000,\"end_ms\":2000,\"text\":\"Hi\"}]")) ?? ""
check("captions srt timestamp", srt.contains("00:00:01,000 --> 00:00:02,000"))

let svg = takeString(shotcore_svg(doc)) ?? ""
check("svg export", svg.contains("<svg"))

let mock = takeString(shotcore_mockup_size(0, 800, 600)) ?? ""
check("browser mockup outer 802", mock.contains("802"))

let meas = takeString(shotcore_measure(0, 0, 3, 4)) ?? ""
check("measure distance 5", meas.contains("\"distance\":5"))

let tags = takeString(shotcore_autotag("invoice invoice total", 3)) ?? ""
check("autotag invoice", tags.contains("invoice"))

var buf = [UInt8](repeating: 255, count: 4 * 4 * 4)
let idx = (1 * 4 + 1) * 4
buf[idx] = 0; buf[idx + 1] = 0; buf[idx + 2] = 0; buf[idx + 3] = 255
let snap = buf.withUnsafeBufferPointer { p in
 takeString(shotcore_snap_object(p.baseAddress, UInt(p.count), 4, 4, 0, 0, 4, 4, 10))
} ?? ""
check("snap_object x 1", snap.contains("\"x\":1"))

let ed = shotcore_editor_new(100, 100)
shotcore_editor_set_tool(ed, 2)
shotcore_editor_pointer_down(ed, 10, 10)
shotcore_editor_pointer_up(ed, 60, 40)
let render = takeString(shotcore_editor_render_json(ed)) ?? ""
check("editor created a rectangle", render.contains("Rectangle"))
shotcore_editor_free(ed)

print("\nALL OK: \(passed) Swift<->Rust C ABI checks passed on \(version)")
