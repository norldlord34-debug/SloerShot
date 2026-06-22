import AppKit

// Floating recording HUD (CleanShot-style): red dot + live elapsed time + Stop button,
// shown while a screen recording is in progress. Time formatting comes from the tested core.
@MainActor
final class RecordingHUD: NSObject {
static let shared = RecordingHUD()
private var panel: NSPanel?
private var timer: Timer?
private var startTime = Date()
private var onStop: (() -> Void)?
private weak var timeLabel: NSTextField?

func show(onStop: @escaping () -> Void) {
self.onStop = onStop
startTime = Date()
build()
}
func hide() {
timer?.invalidate(); timer = nil
panel?.orderOut(nil); panel = nil
}
private func build() {
panel?.orderOut(nil)
let width: CGFloat = 210, height: CGFloat = 44
let content = NSView(frame: NSRect(x: 0, y: 0, width: width, height: height))
content.wantsLayer = true
content.layer?.backgroundColor = NSColor(calibratedWhite: 0.11, alpha: 0.97).cgColor
content.layer?.cornerRadius = 12
let dot = NSView(frame: NSRect(x: 14, y: height / 2 - 5, width: 10, height: 10))
dot.wantsLayer = true
dot.layer?.backgroundColor = NSColor.systemRed.cgColor
dot.layer?.cornerRadius = 5
content.addSubview(dot)
let lbl = NSTextField(labelWithString: "0:00")
lbl.font = NSFont.monospacedDigitSystemFont(ofSize: 14, weight: .medium)
lbl.textColor = .white
lbl.frame = NSRect(x: 34, y: height / 2 - 10, width: 90, height: 20)
content.addSubview(lbl)
timeLabel = lbl
let stop = NSButton(title: "Stop", target: self, action: #selector(stopTapped))
stop.bezelStyle = .rounded
stop.keyEquivalent = "\u{1b}"
stop.frame = NSRect(x: width - 72, y: 8, width: 60, height: 28)
content.addSubview(stop)
let p = NSPanel(contentRect: content.frame, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
p.isFloatingPanel = true
p.level = .floating
p.backgroundColor = .clear
p.isOpaque = false
p.hasShadow = true
p.contentView = content
if let scr = NSScreen.main { let f = scr.visibleFrame; p.setFrameOrigin(NSPoint(x: f.midX - width / 2, y: f.maxY - height - 12)) }
p.orderFrontRegardless()
panel = p
let t = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in Task { @MainActor in self?.tick() } }
RunLoop.main.add(t, forMode: .common)
timer = t
}
private func tick() {
let ms = UInt64(max(0, Date().timeIntervalSince(startTime)) * 1000)
timeLabel?.stringValue = ShotCore.recordElapsed(ms) ?? "0:00"
}
@objc private func stopTapped() { let s = onStop; hide(); s?() }
}
