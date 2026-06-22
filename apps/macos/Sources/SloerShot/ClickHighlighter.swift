import AppKit

// Visualizes the users own mouse clicks as a fading ring during a screen recording
// (CleanShot "Highlight clicks"). A transparent, click-through, top-level window the
// recording captures. Ephemeral only: nothing is logged, stored, or transmitted.
@MainActor
final class ClickHighlighter {
static let shared = ClickHighlighter()
private var window: NSWindow?
private var monitor: Any?
private let layerView = ClickLayerView()

func start() {
guard window == nil, let scr = NSScreen.main else { return }
let win = NSWindow(contentRect: scr.frame, styleMask: [.borderless], backing: .buffered, defer: false)
win.isOpaque = false
win.backgroundColor = .clear
win.level = .screenSaver
win.ignoresMouseEvents = true
win.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
layerView.frame = NSRect(origin: .zero, size: scr.frame.size)
win.contentView = layerView
win.orderFrontRegardless()
window = win
monitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] _ in
guard let self else { return }
Task { @MainActor in self.handleClick() }
}
}
func stop() {
if let m = monitor { NSEvent.removeMonitor(m); monitor = nil }
window?.orderOut(nil)
window = nil
}
private func handleClick() {
guard let scr = NSScreen.main else { return }
let p = NSEvent.mouseLocation
layerView.ripple(at: CGPoint(x: p.x - scr.frame.minX, y: p.y - scr.frame.minY))
}
}

final class ClickLayerView: NSView {
func ripple(at point: CGPoint) {
wantsLayer = true
let size: CGFloat = 56
let ring = CAShapeLayer()
ring.path = CGPath(ellipseIn: CGRect(x: point.x - size / 2, y: point.y - size / 2, width: size, height: size), transform: nil)
ring.fillColor = NSColor.systemYellow.withAlphaComponent(0.35).cgColor
ring.strokeColor = NSColor.systemYellow.cgColor
ring.lineWidth = 2.5
layer?.addSublayer(ring)
let fade = CABasicAnimation(keyPath: "opacity")
fade.fromValue = 0.9
fade.toValue = 0.0
fade.duration = 0.45
ring.add(fade, forKey: nil)
DispatchQueue.main.asyncAfter(deadline: .now() + 0.45) { ring.removeFromSuperlayer() }
}
}
