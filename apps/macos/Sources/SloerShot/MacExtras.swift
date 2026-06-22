import AppKit
import SwiftUI
import ScreenCaptureKit

// Transient HUD toast near the bottom of the main screen (OCR / color / save feedback).
enum Toast {
static func show(_ message: String) {
let label = NSTextField(labelWithString: message)
label.textColor = .white
label.font = .systemFont(ofSize: 13, weight: .medium)
label.alignment = .center
label.isBezeled = false
label.drawsBackground = false
label.sizeToFit()
let pad: CGFloat = 16
let w = min(560, label.frame.width + pad * 2)
let h = label.frame.height + pad
let container = NSView(frame: NSRect(x: 0, y: 0, width: w, height: h))
container.wantsLayer = true
container.layer?.backgroundColor = NSColor(calibratedWhite: 0.08, alpha: 0.95).cgColor
container.layer?.cornerRadius = 10
label.frame = NSRect(x: pad, y: (h - label.frame.height) / 2, width: w - pad * 2, height: label.frame.height)
container.addSubview(label)
let panel = NSPanel(contentRect: container.frame, styleMask: [.nonactivatingPanel, .borderless], backing: .buffered, defer: false)
panel.level = .floating
panel.isFloatingPanel = true
panel.backgroundColor = .clear
panel.hasShadow = true
panel.contentView = container
if let screen = NSScreen.main {
let f = screen.frame
panel.setFrameOrigin(NSPoint(x: f.midX - w / 2, y: f.minY + 90))
}
panel.orderFrontRegardless()
DispatchQueue.main.asyncAfter(deadline: .now() + 2.2) { panel.orderOut(nil) }
}
}

// Brief always-on-top countdown shown before a delayed capture.
@MainActor
final class CountdownOverlay {
private let window: NSWindow
private let label = NSTextField(labelWithString: "")
private let size: CGFloat = 180
init() {
let view = NSView(frame: NSRect(x: 0, y: 0, width: size, height: size))
view.wantsLayer = true
view.layer?.backgroundColor = NSColor(calibratedWhite: 0.07, alpha: 0.86).cgColor
view.layer?.cornerRadius = 30
label.frame = NSRect(x: 0, y: (size - 110) / 2, width: size, height: 110)
label.font = .systemFont(ofSize: 92, weight: .semibold)
label.textColor = .white
label.alignment = .center
label.isBezeled = false
label.drawsBackground = false
view.addSubview(label)
window = NSWindow(contentRect: view.frame, styleMask: [.borderless], backing: .buffered, defer: false)
window.level = .screenSaver
window.isOpaque = false
window.backgroundColor = .clear
window.contentView = view
if let screen = NSScreen.main {
let f = screen.frame
window.setFrameOrigin(NSPoint(x: f.midX - size / 2, y: f.midY - size / 2))
}
}
func run(seconds: Int) async {
window.orderFrontRegardless()
var i = seconds
while i > 0 {
label.stringValue = String(i)
try? await Task.sleep(nanoseconds: 1_000_000_000)
i -= 1
}
window.orderOut(nil)
}
}

// Native pixel color sampler (eyedropper); copies hex to the clipboard.
enum ColorPickerTool {
static func pick() {
let sampler = NSColorSampler()
sampler.show { (color: NSColor?) in
guard let color = color else { return }
let rgb = color.usingColorSpace(.sRGB) ?? color
let r = Int((rgb.redComponent * 255).rounded())
let g = Int((rgb.greenComponent * 255).rounded())
let b = Int((rgb.blueComponent * 255).rounded())
let hex = String(format: "#%02X%02X%02X", r, g, b)
let pb = NSPasteboard.general
pb.clearContents()
pb.setString(hex, forType: .string)
Toast.show("Picked " + hex)
}
}
}

@available(macOS 14.0, *)
extension AppModel {
func applyDelay() async {
let s = captureDelaySeconds
guard s > 0 else { return }
let overlay = CountdownOverlay()
await overlay.run(seconds: s)
try? await Task.sleep(nanoseconds: 150_000_000)
}
@discardableResult
func captureWindow() async -> Bool {
do {
await applyDelay()
let content = try await Capture.shareableContent()
let mine = NSRunningApplication.current.processIdentifier
let windows = content.windows.filter { $0.isOnScreen && $0.owningApplication?.processID != mine && $0.frame.width > 40 && $0.frame.height > 40 }
guard let win = windows.first else { lastError = "No capturable window found"; return false }
let image = try await Capture.captureWindow(win)
openEditor(with: image)
return true
} catch {
lastError = String(describing: error)
return false
}
}
@discardableResult
func quickOCR() async -> Bool {
do {
let frozen = try await Capture.captureFullscreen()
guard let sel = await SelectionOverlay.present(image: frozen) else { return false }
guard let cropped = Capture.crop(frozen, to: sel) else { return false }
guard let text = OcrService.recognizeText(cgImage: cropped, width: cropped.width, height: cropped.height), !text.isEmpty else {
Toast.show("No text found")
return false
}
let pb = NSPasteboard.general
pb.clearContents()
pb.setString(text, forType: .string)
Toast.show("Copied text (\(text.count) chars)")
return true
} catch {
lastError = String(describing: error)
return false
}
}
func pickColor() {
ColorPickerTool.pick()
}
func pinLast() {
guard let img = lastImage else { lastError = "Nothing to pin"; return }
let maxDim: CGFloat = 600
let scale = min(1.0, maxDim / CGFloat(max(img.width, img.height)))
let displaySize = NSSize(width: CGFloat(img.width) * scale, height: CGFloat(img.height) * scale)
let nsImage = NSImage(cgImage: img, size: displaySize)
var origin = NSPoint(x: 120, y: 120)
if let screen = NSScreen.main {
origin = NSPoint(x: screen.frame.midX - displaySize.width / 2, y: screen.frame.midY - displaySize.height / 2)
}
let panel = PinPanel(image: nsImage, at: origin)
panel.orderFrontRegardless()
pins.append(panel)
}
}
