import AppKit

// Floating screenshot pinned above all windows (CleanShot Floating Screenshots), over the
// tested pin core. A borderless, always-on-top NSPanel that shows the captured image.
final class PinPanel: NSPanel {
 private let imageView = NSImageView()

 init(image: NSImage, at origin: CGPoint) {
 super.init(contentRect: NSRect(origin: origin, size: image.size),
 styleMask: [.nonactivatingPanel, .borderless],
 backing: .buffered, defer: false)
 self.level = .floating
 self.isFloatingPanel = true
 self.hasShadow = true
 self.backgroundColor = .clear
 imageView.image = image
 imageView.imageScaling = .scaleProportionallyUpOrDown
 self.contentView = imageView
 }

 // CleanShot pin opacity slider.
 func setOpacity(_ value: Double) { self.alphaValue = CGFloat(max(0.1, min(1.0, value))) }

 // Precise on-screen positioning with arrow keys.
 func nudge(dx: CGFloat, dy: CGFloat) { setFrameOrigin(NSPoint(x: frame.origin.x + dx, y: frame.origin.y + dy)) }

 // Lock mode: click-through so apps underneath stay interactive.
 func setLocked(_ locked: Bool) { self.ignoresMouseEvents = locked }
}
