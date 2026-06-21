import Foundation

// macOS scanning helpers built on the shared core: auto-flatten a tilted document (Harris
// corners -> 4-point perspective unwarp), scan QR / EAN-13 barcodes, and sharpen. All the
// vision math runs in the tested Rust core; this is the thin Swift orchestration.
struct ScanPoint { let x: Double; let y: Double }

enum Scanner {
 // Parse the [Point] JSON the core returns into points.
 static func parsePoints(_ json: String) -> [ScanPoint] {
 guard let data = json.data(using: .utf8),
 let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else { return [] }
 return arr.compactMap { dict in
 guard let x = dict["x"] as? Double, let y = dict["y"] as? Double else { return nil }
 return ScanPoint(x: x, y: y)
 }
 }

 // Pick the 4 document corners (TL,TR,BR,BL) from detected points using the x +/- y
 // extremes, returned as the corners JSON the unwarp FFI expects.
 static func documentQuadJSON(from points: [ScanPoint]) -> String? {
 guard points.count >= 4 else { return nil }
 let tl = points.min { $0.x + $0.y < $1.x + $1.y }!
 let br = points.max { $0.x + $0.y < $1.x + $1.y }!
 let tr = points.max { $0.x - $0.y < $1.x - $1.y }!
 let bl = points.min { $0.x - $0.y < $1.x - $1.y }!
 let quad = [tl, tr, br, bl].map { ["x": $0.x, "y": $0.y] }
 guard let data = try? JSONSerialization.data(withJSONObject: quad),
 let s = String(data: data, encoding: .utf8) else { return nil }
 return s
 }

 // Auto-flatten: detect corners in the RGBA buffer, pick the document quad, unwarp inPath.
 @discardableResult
 static func flatten(inPath: String, rgba: [UInt8], width: UInt32, height: UInt32,
 outW: UInt32, outH: UInt32, outPath: String) -> Bool {
 guard let cornersJson = ShotCore.corners(rgba: rgba, width: width, height: height,
 k: 0.04, relThreshold: 0.2, minDistance: 12),
 let quad = documentQuadJSON(from: parsePoints(cornersJson)) else { return false }
 return ShotCore.perspectiveUnwarp(inPath: inPath, outPath: outPath, cornersJson: quad,
 outW: outW, outH: outH) == 0
 }

 // Scan a barcode: QR first, then EAN-13. Returns the decoded payload or nil.
 static func scanBarcode(rgba: [UInt8], width: UInt32, height: UInt32) -> String? {
 if let qrJson = ShotCore.qrDecode(rgba: rgba, width: width, height: height),
 let data = qrJson.data(using: .utf8),
 let dict = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
 let text = dict["text"] as? String {
 return text
 }
 return ShotCore.ean13Decode(rgba: rgba, width: width, height: height)
 }

 @discardableResult
 static func sharpen(inPath: String, outPath: String, radius: UInt32 = 2, amount: Float = 1.0) -> Bool {
 ShotCore.unsharp(inPath: inPath, outPath: outPath, radius: radius, amount: amount) == 0
 }
}
