import AppKit
import CoreGraphics
import ScreenCaptureKit

// Vertical stitch of scrolling-capture frames using per-row signature overlap detection.
enum ScrollStitch {
static func rowSig(_ img: CGImage) -> [Int]? {
let w = img.width, h = img.height
guard w > 0, h > 0, let cs = CGColorSpace(name: CGColorSpace.sRGB) else { return nil }
guard let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0, space: cs, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { return nil }
ctx.draw(img, in: CGRect(x: 0, y: 0, width: w, height: h))
guard let data = ctx.data else { return nil }
let stride = ctx.bytesPerRow
let ptr = data.bindMemory(to: UInt8.self, capacity: stride * h)
var sig = [Int](repeating: 0, count: h)
let step = max(4, (w / 64) * 4)
let lim = w * 4
for y in 0..<h {
var s = 0
let ro = y * stride
var x = 0
while x + 2 < lim { s += Int(ptr[ro + x]) + Int(ptr[ro + x + 1]) + Int(ptr[ro + x + 2]); x += step }
sig[y] = s
}
return sig
}
static func rgba(_ img: CGImage) -> [UInt8]? {
let w = img.width, h = img.height
guard w > 0, h > 0, let cs = CGColorSpace(name: CGColorSpace.sRGB),
let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w * 4, space: cs, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { return nil }
ctx.draw(img, in: CGRect(x: 0, y: 0, width: w, height: h))
guard let data = ctx.data else { return nil }
let ptr = data.bindMemory(to: UInt8.self, capacity: w * 4 * h)
return Array(UnsafeBufferPointer(start: ptr, count: w * 4 * h))
}
static func bestOverlap(_ a: [Int], _ b: [Int]) -> Int {
let ha = a.count, hb = b.count
let maxOv = min(ha, hb) - 1
if maxOv < 8 { return 0 }
var best = Double.greatestFiniteMagnitude
var bestK = 0
var k = 8
while k <= maxOv {
var diff = 0.0
var cnt = 0
let stepj = max(1, k / 200)
var j = 0
while j < k { diff += abs(Double(a[ha - k + j] - b[j])); cnt += 1; j += stepj }
let avg = cnt > 0 ? diff / Double(cnt) : Double.greatestFiniteMagnitude
if avg < best { best = avg; bestK = k }
k += 1
}
return bestK
}
static func stitch(_ images: [CGImage]) -> CGImage? {
guard let first = images.first else { return nil }
let w = first.width
var sigs: [[Int]] = []
for im in images { guard let s = rowSig(im) else { return nil }; sigs.append(s) }
var ks = [Int](repeating: 0, count: images.count)
var totalH = first.height
if images.count > 1 {
for i in 1..<images.count { ks[i] = bestOverlap(sigs[i - 1], sigs[i]); totalH += max(0, images[i].height - ks[i]) }
}
guard let cs = CGColorSpace(name: CGColorSpace.sRGB) else { return nil }
guard let ctx = CGContext(data: nil, width: w, height: max(1, totalH), bitsPerComponent: 8, bytesPerRow: 0, space: cs, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else { return nil }
ctx.draw(first, in: CGRect(x: 0, y: totalH - first.height, width: w, height: first.height))
var yTop = first.height
if images.count > 1 {
for i in 1..<images.count {
let k = ks[i]
let hh = images[i].height - k
if hh <= 0 { continue }
guard let cropped = images[i].cropping(to: CGRect(x: 0, y: k, width: images[i].width, height: hh)) else { continue }
ctx.draw(cropped, in: CGRect(x: 0, y: totalH - yTop - hh, width: w, height: hh))
yTop += hh
}
}
return ctx.makeImage()
}
}

@available(macOS 14.0, *)
extension AppModel {
@discardableResult
func scrollCapture() async -> Bool {
do {
let content = try await Capture.shareableContent()
let mine = NSRunningApplication.current.processIdentifier
let candidates = content.windows.filter { $0.isOnScreen && $0.owningApplication?.processID != mine && $0.frame.width > 80 && $0.frame.height > 80 }.sorted { ($0.frame.width * $0.frame.height) > ($1.frame.width * $1.frame.height) }
guard let win = candidates.first else { lastError = "No window to scroll-capture"; return false }
let target = max(2, UserDefaults.standard.integer(forKey: "ss.scrollFrames"))
Toast.show("Scroll the window slowly - capturing up to \(target) frames")
var frames: [CGImage] = []
var lastHash: UInt64 = 0
var attempts = 0
while frames.count < target && attempts < target * 4 {
attempts += 1
let img = try await Capture.captureWindow(win)
if let bytes = ScrollStitch.rgba(img) {
let hsh = ShotCore.dHash(rgba: bytes, width: UInt32(img.width), height: UInt32(img.height))
if frames.isEmpty || ShotCore.hamming(lastHash, hsh) >= 3 {
frames.append(img)
lastHash = hsh
Toast.show("Captured \(frames.count)/\(target)")
}
} else {
frames.append(img)
}
try? await Task.sleep(nanoseconds: 700_000_000)
}
guard let stitched = ScrollStitch.stitch(frames) else { lastError = "Stitch failed"; return false }
showQAO(stitched)
Toast.show("Scrolling capture stitched")
return true
} catch {
lastError = String(describing: error)
return false
}
}
}
