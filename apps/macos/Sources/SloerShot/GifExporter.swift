import AVFoundation
import AppKit
import ImageIO
import UniformTypeIdentifiers

// Export a recorded MP4 to an animated GIF by sampling frames with AVAssetImageGenerator
// and writing them via ImageIO. Used by the "Export Recording to GIF" menu action.
enum GifExporter {
static func latestRecording() -> URL? {
let movies = FileManager.default.urls(for: .moviesDirectory, in: .userDomainMask).first ?? FileManager.default.temporaryDirectory
let folder = movies.appendingPathComponent("SloerShot")
let urls = (try? FileManager.default.contentsOfDirectory(at: folder, includingPropertiesForKeys: [.contentModificationDateKey])) ?? []
return urls.filter { $0.pathExtension.lowercased() == "mp4" }.sorted { a, b in
let da = (try? a.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate) ?? .distantPast
let db = (try? b.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate) ?? .distantPast
return da > db
}.first
}

static func export(from src: URL, to dest: URL, fps: Int = 12, maxWidth: CGFloat = 640) async -> Bool {
let asset = AVURLAsset(url: src)
guard let duration = try? await asset.load(.duration) else { return false }
let total = CMTimeGetSeconds(duration)
guard total > 0 else { return false }
let frameCount = min(450, max(1, Int(total * Double(fps))))
let gen = AVAssetImageGenerator(asset: asset)
gen.appliesPreferredTrackTransform = true
gen.requestedTimeToleranceBefore = .zero
gen.requestedTimeToleranceAfter = .zero
gen.maximumSize = CGSize(width: maxWidth, height: 4000)
guard let out = CGImageDestinationCreateWithURL(dest as CFURL, UTType.gif.identifier as CFString, frameCount, nil) else { return false }
let gifProps = [kCGImagePropertyGIFDictionary as String: [kCGImagePropertyGIFLoopCount as String: 0]]
CGImageDestinationSetProperties(out, gifProps as CFDictionary)
let frameProps = [kCGImagePropertyGIFDictionary as String: [kCGImagePropertyGIFDelayTime as String: 1.0 / Double(fps)]]
for i in 0..<frameCount {
let t = CMTime(seconds: Double(i) / Double(fps), preferredTimescale: 600)
guard let cg = try? await frame(gen, at: t) else { continue }
CGImageDestinationAddImage(out, cg, frameProps as CFDictionary)
}
return CGImageDestinationFinalize(out)
}

private static func frame(_ gen: AVAssetImageGenerator, at time: CMTime) async throws -> CGImage {
try await gen.image(at: time).image
}
}
