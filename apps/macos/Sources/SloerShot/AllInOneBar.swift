import AppKit

// CleanShot-style All-In-One bar: one floating HUD to launch any capture mode.
@MainActor
final class AllInOneBar: NSObject {
static let shared = AllInOneBar()
private var panel: NSPanel?
private weak var model: AppModel?
private var openEditor: (() -> Void)?
private var openHistory: (() -> Void)?

func present(model: AppModel, openEditor: @escaping () -> Void, openHistory: @escaping () -> Void) {
self.model = model
self.openEditor = openEditor
self.openHistory = openHistory
build()
}

private func iconButton(_ symbol: String, _ label: String, _ sel: Selector) -> NSView {
let container = NSView(frame: NSRect(x: 0, y: 0, width: 64, height: 60))
let b = NSButton(frame: NSRect(x: 19, y: 26, width: 26, height: 26))
b.bezelStyle = .circular
b.image = NSImage(systemSymbolName: symbol, accessibilityDescription: label)
b.imagePosition = .imageOnly
b.contentTintColor = NSColor.white
b.target = self
b.action = sel
b.toolTip = label
container.addSubview(b)
let t = NSTextField(labelWithString: label)
t.font = NSFont.systemFont(ofSize: 9)
t.alignment = .center
t.textColor = NSColor.white
t.frame = NSRect(x: 0, y: 5, width: 64, height: 14)
container.addSubview(t)
return container
}

private func build() {
panel?.orderOut(nil)
var items: [(String, String, Selector)] = [
("camera.viewfinder", "Area", #selector(area)),
("macwindow", "Window", #selector(windowShot)),
("rectangle.inset.filled", "Fullscreen", #selector(fullscreen)),
("arrow.down.doc", "Scrolling", #selector(scrolling)),
("record.circle", "Record", #selector(record)),
("text.viewfinder", "OCR", #selector(ocr)),
("eyedropper", "Color", #selector(color)),
("pin", "Pin", #selector(pinLast)),
("clock.arrow.circlepath", "History", #selector(history)),
]
items.append(("xmark", "Close", #selector(closeBar)))
let cell: CGFloat = 64, pad: CGFloat = 14, h: CGFloat = 74
let width = pad * 2 + CGFloat(items.count) * cell
let content = NSView(frame: NSRect(x: 0, y: 0, width: width, height: h))
content.wantsLayer = true
content.layer?.backgroundColor = NSColor(calibratedWhite: 0.11, alpha: 0.97).cgColor
content.layer?.cornerRadius = 16
for (i, it) in items.enumerated() {
let v = iconButton(it.0, it.1, it.2)
v.frame = NSRect(x: pad + CGFloat(i) * cell, y: 7, width: cell, height: 60)
content.addSubview(v)
}
let p = NSPanel(contentRect: content.frame, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
p.isFloatingPanel = true
p.level = .floating
p.backgroundColor = .clear
p.isOpaque = false
p.hasShadow = true
p.contentView = content
if let scr = NSScreen.main {
p.setFrameOrigin(NSPoint(x: scr.frame.midX - width / 2, y: scr.frame.maxY - 170))
}
p.orderFrontRegardless()
panel = p
}

@objc private func closeBar() { panel?.orderOut(nil); panel = nil }
private func dismissThen(_ block: @escaping () -> Void) { closeBar(); block() }

@objc private func area() { model?.editorOpener = openEditor; let m = model; dismissThen { Task { await m?.captureArea() } } }
@objc private func windowShot() { model?.editorOpener = openEditor; let m = model; dismissThen { Task { await m?.captureWindow() } } }
@objc private func fullscreen() { model?.editorOpener = openEditor; let m = model; dismissThen { Task { await m?.captureFullscreen() } } }
@objc private func scrolling() { model?.editorOpener = openEditor; let m = model; dismissThen { Task { await m?.scrollCapture() } } }
@objc private func record() { let m = model; dismissThen { if m?.isRecording == true { m?.stopRecording() } else { m?.startRecording() } } }
@objc private func ocr() { let m = model; dismissThen { Task { await m?.quickOCR() } } }
@objc private func color() { let m = model; dismissThen { m?.pickColor() } }
@objc private func pinLast() { let m = model; dismissThen { m?.pinLast() } }
@objc private func history() { let open = openHistory; dismissThen { open?() } }
}
