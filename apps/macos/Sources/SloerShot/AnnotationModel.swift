import CoreGraphics
import Foundation

/// The nine annotation shapes the canvas can draw.
enum ShapeType {
 case line, arrow, rectangle, ellipse, freehand, text, counter, highlighter, redact
}

/// 8-bit RGBA color decoded from the core render JSON.
struct ColorSpec: Equatable {
 var r: UInt8 = 0
 var g: UInt8 = 0
 var b: UInt8 = 0
 var a: UInt8 = 255
}

/// A flat, UI-agnostic description of one shape to draw. Produced by parsing the
/// core render_json; the SwiftUI canvas turns these into paths. Mirrors the Windows
/// AnnotationParser so both platforms render identically.
struct ShapeSpec {
 var type: ShapeType
 var x = 0.0, y = 0.0, w = 0.0, h = 0.0
 var x1 = 0.0, y1 = 0.0, x2 = 0.0, y2 = 0.0
 var points: [CGPoint] = []
 var text = ""
 var fontSize = 0.0
 var number: UInt32 = 0
 var cornerRadius = 0.0
 var stroke = ColorSpec()
 var fill: ColorSpec? = nil
 var strokeWidth = 1.0
 var opacity = 1.0
 var z = 0
}

/// Parses the core editor/document render_json into an ordered list of shape specs.
enum AnnotationParser {
 static func parse(_ json: String?) -> [ShapeSpec] {
 guard let json = json,
 let data = json.data(using: .utf8),
 let root = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
 let anns = root["annotations"] as? [[String: Any]]
 else { return [] }

 var specs: [ShapeSpec] = []
 for ann in anns {
 if (ann["hidden"] as? Bool) == true { continue }
 guard let kind = ann["kind"] as? [String: Any],
 let (variant, data) = kind.first
 else { continue }
 let payload = data as? [String: Any] ?? [:]

 var spec: ShapeSpec
 switch variant {
 case "Rectangle":
 spec = ShapeSpec(type: .rectangle)
 readRect(payload["rect"], into: &spec)
 spec.cornerRadius = dbl(payload["corner_radius"])
 case "Ellipse":
 spec = ShapeSpec(type: .ellipse)
 readRect(payload["rect"], into: &spec)
 case "Highlighter":
 spec = ShapeSpec(type: .highlighter)
 readRect(payload["rect"], into: &spec)
 case "Redact":
 spec = ShapeSpec(type: .redact)
 readRect(payload["rect"], into: &spec)
 case "Line":
 spec = ShapeSpec(type: .line)
 readSegment(payload, into: &spec)
 case "Arrow":
 spec = ShapeSpec(type: .arrow)
 readSegment(payload, into: &spec)
 case "Freehand":
 spec = ShapeSpec(type: .freehand)
 if let pts = payload["points"] as? [[String: Any]] {
 spec.points = pts.map { CGPoint(x: dbl($0["x"]), y: dbl($0["y"])) }
 }
 case "Text":
 spec = ShapeSpec(type: .text)
 if let pos = payload["position"] as? [String: Any] {
 spec.x = dbl(pos["x"]); spec.y = dbl(pos["y"])
 }
 spec.text = payload["content"] as? String ?? ""
 spec.fontSize = dbl(payload["font_size"])
 case "Counter":
 spec = ShapeSpec(type: .counter)
 if let c = payload["center"] as? [String: Any] {
 spec.x = dbl(c["x"]); spec.y = dbl(c["y"])
 }
 spec.h = dbl(payload["radius"])
 spec.number = UInt32(dbl(payload["number"]))
 default:
 continue
 }

 spec.z = Int(dbl(ann["z"]))
 if let style = ann["style"] as? [String: Any] {
 spec.stroke = readColor(style["stroke"]) ?? spec.stroke
 spec.fill = readColor(style["fill"])
 if let sw = style["stroke_width"] { spec.strokeWidth = dbl(sw) }
 if let op = style["opacity"] { spec.opacity = dbl(op) }
 }
 specs.append(spec)
 }
 specs.sort { $0.z < $1.z }
 return specs
 }

 private static func dbl(_ v: Any?) -> Double {
 if let d = v as? Double { return d }
 if let n = v as? NSNumber { return n.doubleValue }
 return 0
 }

 private static func readColor(_ v: Any?) -> ColorSpec? {
 guard let c = v as? [String: Any] else { return nil }
 return ColorSpec(
 r: UInt8(clamping: Int(dbl(c["r"]))),
 g: UInt8(clamping: Int(dbl(c["g"]))),
 b: UInt8(clamping: Int(dbl(c["b"]))),
 a: UInt8(clamping: Int(dbl(c["a"])))
 )
 }

 private static func readRect(_ v: Any?, into spec: inout ShapeSpec) {
 guard let r = v as? [String: Any] else { return }
 spec.x = dbl(r["x"]); spec.y = dbl(r["y"]); spec.w = dbl(r["w"]); spec.h = dbl(r["h"])
 }

 private static func readSegment(_ v: [String: Any], into spec: inout ShapeSpec) {
 if let f = v["from"] as? [String: Any] { spec.x1 = dbl(f["x"]); spec.y1 = dbl(f["y"]) }
 if let t = v["to"] as? [String: Any] { spec.x2 = dbl(t["x"]); spec.y2 = dbl(t["y"]) }
 }
}
