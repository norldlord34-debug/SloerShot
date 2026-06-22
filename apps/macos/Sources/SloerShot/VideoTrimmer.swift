import AppKit
import AVKit
import AVFoundation
import SwiftUI
import UniformTypeIdentifiers

// Minimal video trim editor: load a recording, set start/end, export the trimmed range
// via AVAssetExportSession. Opened from the "Trim Last Recording" menu action.
@MainActor
enum VideoTrimmer {
static var window: NSWindow?
static func show(url: URL) {
let host = NSHostingView(rootView: TrimView(url: url))
let win = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 740, height: 540), styleMask: [.titled, .closable, .resizable], backing: .buffered, defer: false)
win.title = "Trim Recording"
win.contentView = host
win.center()
win.isReleasedWhenClosed = false
win.makeKeyAndOrderFront(nil)
NSApp.activate(ignoringOtherApps: true)
window = win
}
}

struct TrimView: View {
let url: URL
@State private var player = AVPlayer()
@State private var duration: Double = 0
@State private var startT: Double = 0
@State private var endT: Double = 0
@State private var status = ""
var body: some View {
VStack(spacing: 10) {
VideoPlayer(player: player).frame(minHeight: 320)
VStack(alignment: .leading, spacing: 6) {
HStack { Text("Start").frame(width: 44, alignment: .leading); Slider(value: $startT, in: 0...max(0.1, duration)) { e in if !e { seek(startT) } }; Text(fmt(startT)).font(.caption).frame(width: 56) }
HStack { Text("End").frame(width: 44, alignment: .leading); Slider(value: $endT, in: 0...max(0.1, duration)) { e in if !e { seek(endT) } }; Text(fmt(endT)).font(.caption).frame(width: 56) }
}
HStack {
Button("Play range") { seek(startT); player.play() }
Spacer()
Text(status).font(.caption).foregroundStyle(.secondary)
Button("Export Trimmed") { export() }.buttonStyle(.borderedProminent)
}
}
.padding(14)
.frame(minWidth: 640, minHeight: 460)
.onAppear { load() }
}
private func load() {
player.replaceCurrentItem(with: AVPlayerItem(url: url))
Task {
let asset = AVURLAsset(url: url)
if let d = try? await asset.load(.duration) {
let secs = CMTimeGetSeconds(d)
duration = secs
endT = secs
}
}
}
private func seek(_ t: Double) { player.seek(to: CMTime(seconds: t, preferredTimescale: 600), toleranceBefore: .zero, toleranceAfter: .zero) }
private func fmt(_ t: Double) -> String { String(format: "%.1fs", t) }
private func export() {
guard endT > startT else { status = "End must be after start"; return }
let sp = NSSavePanel()
sp.allowedContentTypes = [.mpeg4Movie]
sp.nameFieldStringValue = "SloerShot-trimmed.mp4"
guard sp.runModal() == .OK, let dest = sp.url else { return }
let asset = AVURLAsset(url: url)
guard let session = AVAssetExportSession(asset: asset, presetName: AVAssetExportPresetHighestQuality) else { status = "Cannot export"; return }
session.outputURL = dest
session.outputFileType = .mp4
session.timeRange = CMTimeRange(start: CMTime(seconds: startT, preferredTimescale: 600), end: CMTime(seconds: endT, preferredTimescale: 600))
status = "Exporting..."
Task {
await withCheckedContinuation { (cont: CheckedContinuation<Void, Never>) in session.exportAsynchronously { cont.resume() } }
status = session.status == .completed ? "Saved " + dest.lastPathComponent : "Export failed"
}
}
}
