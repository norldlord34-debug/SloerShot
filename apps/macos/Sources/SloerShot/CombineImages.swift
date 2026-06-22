import AppKit

// Combine several images into one canvas, stacked vertically and centered, using the
// tested core layout (combine_stack_vertical). Composited with NSImage for correct orientation.
enum CombineImages {
static func combine(urls: [URL], gap: Int = 16) -> CGImage? {
let images = urls.compactMap { NSImage(contentsOf: $0) }
guard !images.isEmpty else { return nil }
let sizes: [(Int, Int)] = images.map { img in
if let r = img.representations.first { return (r.pixelsWide, r.pixelsHigh) }
return (Int(img.size.width), Int(img.size.height))
}
let sizesJson = "[" + sizes.map { "[\($0.0),\($0.1)]" }.joined(separator: ",") + "]"
guard let lj = ShotCore.combineStackVertical(sizesJson: sizesJson, gap: UInt32(max(0, gap))),
let data = lj.data(using: .utf8),
let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
let cw = obj["canvas_w"] as? Int, let ch = obj["canvas_h"] as? Int, cw > 0, ch > 0,
let placements = obj["placements"] as? [[String: Any]],
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: cw, pixelsHigh: ch, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0) else { return nil }
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
for p in placements {
guard let idx = p["image"] as? Int, idx < images.count, let x = p["x"] as? Int, let y = p["y"] as? Int else { continue }
let (iw, ih) = sizes[idx]
images[idx].draw(in: NSRect(x: x, y: ch - y - ih, width: iw, height: ih))
}
NSGraphicsContext.restoreGraphicsState()
return rep.cgImage
}
}
