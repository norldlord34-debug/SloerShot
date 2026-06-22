import AppKit
import UniformTypeIdentifiers

// Draggable thumbnail that exports the capture file to any app (CleanShot Drag me).
final class DraggableImageView: NSImageView, NSDraggingSource {
var fileURL: URL?
override func mouseDragged(with event: NSEvent) {
guard let url = fileURL else { return }
let item = NSDraggingItem(pasteboardWriter: url as NSURL)
item.setDraggingFrame(self.bounds, contents: self.image)
beginDraggingSession(with: [item], event: event, source: self)
}
func draggingSession(_ session: NSDraggingSession, sourceOperationMaskFor context: NSDraggingContext) -> NSDragOperation { .copy }
}

// CleanShot-style Quick Access Overlay: floating panel shown after every capture.
final class QuickAccessOverlay: NSObject {
static let shared = QuickAccessOverlay()
private var panel: NSPanel?
private var image: CGImage?
private var nsImage: NSImage?
private var fileURL: URL?
private var autoCloseTimer: Timer?
var onAnnotate: ((CGImage) -> Void)?
var onUpload: ((URL) -> Void)?
func present(image: CGImage) {
self.image = image
self.nsImage = NSImage(cgImage: image, size: NSSize(width: image.width, height: image.height))
let url = FileManager.default.temporaryDirectory.appendingPathComponent("qao-\(Int(Date().timeIntervalSince1970)).png")
if writePNG(image, to: url) { self.fileURL = url }
build()
}
private func makeButton(_ title: String, _ sel: Selector) -> NSButton {
let b = NSButton(title: title, target: self, action: sel)
b.bezelStyle = .rounded
return b
}
private func makeIcon(_ symbol: String, _ sel: Selector, _ tip: String) -> NSButton {
let b = NSButton(frame: NSRect(x: 0, y: 0, width: 26, height: 26))
b.bezelStyle = .circular
b.image = NSImage(systemSymbolName: symbol, accessibilityDescription: tip)
b.target = self
b.action = sel
b.toolTip = tip
return b
}
private func build() {
panel?.orderOut(nil)
let pad: CGFloat = 12, barH: CGFloat = 44, thumbH: CGFloat = 150, width: CGFloat = 264
let totalH = pad * 2 + barH + thumbH
let content = NSView(frame: NSRect(x: 0, y: 0, width: width, height: totalH))
content.wantsLayer = true
content.layer?.backgroundColor = NSColor(calibratedWhite: 0.11, alpha: 0.97).cgColor
content.layer?.cornerRadius = 14
let thumb = DraggableImageView(frame: NSRect(x: pad, y: pad + barH, width: width - pad * 2, height: thumbH))
thumb.image = nsImage
thumb.imageScaling = .scaleProportionallyUpOrDown
thumb.wantsLayer = true
thumb.layer?.cornerRadius = 8
thumb.layer?.masksToBounds = true
thumb.fileURL = fileURL
thumb.menu = buildMenu()
content.addSubview(thumb)
let copyBtn = makeButton("Copy", #selector(copyAction))
let saveBtn = makeButton("Save", #selector(saveAction))
let bar = NSStackView(views: [copyBtn, saveBtn])
bar.orientation = .horizontal
bar.distribution = .fillEqually
bar.spacing = 10
bar.frame = NSRect(x: pad, y: pad, width: width - pad * 2, height: barH - 6)
content.addSubview(bar)
let topY = pad + barH + thumbH - 32
let botY = pad + barH + 6
let leftX = pad + 6, rightX = width - pad - 32
let pin = makeIcon("pin", #selector(pinAction), "Pin to screen"); pin.frame = NSRect(x: leftX, y: topY, width: 26, height: 26); content.addSubview(pin)
let close = makeIcon("xmark", #selector(closeAction), "Close"); close.frame = NSRect(x: rightX, y: topY, width: 26, height: 26); content.addSubview(close)
let edit = makeIcon("pencil", #selector(annotateAction), "Annotate"); edit.frame = NSRect(x: leftX, y: botY, width: 26, height: 26); content.addSubview(edit)
let cloud = makeIcon("icloud.and.arrow.up", #selector(uploadAction), "Upload to Cloud"); cloud.frame = NSRect(x: rightX, y: botY, width: 26, height: 26); content.addSubview(cloud)
let p = NSPanel(contentRect: content.frame, styleMask: [.nonactivatingPanel, .borderless], backing: .buffered, defer: false)
p.level = .floating
p.isFloatingPanel = true
p.hasShadow = true
p.backgroundColor = .clear
p.contentView = content
if let scr = NSScreen.main {
let f = scr.visibleFrame
let pos = UserDefaults.standard.string(forKey: "ss.qaoPosition") ?? "Bottom Left"
let pw = content.frame.width, ph = content.frame.height
let x = pos.contains("Right") ? f.maxX - pw - 24 : f.minX + 24
let y = pos.contains("Top") ? f.maxY - ph - 24 : f.minY + 24
p.setFrameOrigin(NSPoint(x: x, y: y))
}
p.orderFrontRegardless()
panel = p
if UserDefaults.standard.bool(forKey: "ss.qaoAutoClose") {
let interval = max(3, UserDefaults.standard.integer(forKey: "ss.qaoInterval"))
autoCloseTimer?.invalidate()
autoCloseTimer = Timer.scheduledTimer(withTimeInterval: TimeInterval(interval), repeats: false) { [weak self] _ in self?.closeAction() }
}
}
private func buildMenu() -> NSMenu {
let menu = NSMenu()
menu.addItem(NSMenuItem(title: "Open Annotation Tool", action: #selector(annotateAction), keyEquivalent: ""))
menu.addItem(NSMenuItem(title: "Pin to the Screen", action: #selector(pinAction), keyEquivalent: ""))
menu.addItem(NSMenuItem(title: "Rotate Left", action: #selector(rotateLeftAction), keyEquivalent: ""))
menu.addItem(NSMenuItem.separator())
menu.addItem(NSMenuItem(title: "Upload to Cloud", action: #selector(uploadAction), keyEquivalent: ""))
menu.addItem(NSMenuItem(title: "Share...", action: #selector(shareAction), keyEquivalent: ""))
menu.addItem(NSMenuItem(title: "Quick Look", action: #selector(quickLookAction), keyEquivalent: ""))
menu.addItem(NSMenuItem(title: "Show in Finder", action: #selector(revealAction), keyEquivalent: ""))
menu.addItem(NSMenuItem.separator())
menu.addItem(NSMenuItem(title: "Copy", action: #selector(copyAction), keyEquivalent: "c"))
menu.addItem(NSMenuItem(title: "Save", action: #selector(saveAction), keyEquivalent: "s"))
menu.addItem(NSMenuItem(title: "Close", action: #selector(closeAction), keyEquivalent: "w"))
for it in menu.items { it.target = self }
return menu
}
@objc private func copyAction() { guard let img = nsImage else { return }; let pb = NSPasteboard.general; pb.clearContents(); pb.writeObjects([img]); Toast.show("Copied to clipboard") }
@objc private func saveAction() {
guard let img = image else { return }
let sp = NSSavePanel()
sp.allowedContentTypes = [UTType.png]
sp.nameFieldStringValue = "SloerShot.png"
if sp.runModal() == .OK, let url = sp.url { _ = writePNG(img, to: url); Toast.show("Saved " + url.lastPathComponent) }
}
@objc private func pinAction() { PinStore.pin(fileURL); Toast.show("Pinned") }
@objc private func annotateAction() { if let img = image { onAnnotate?(img) }; closeAction() }
@objc private func uploadAction() { if let url = fileURL { onUpload?(url) } }
@objc private func shareAction() { if let url = fileURL { Task { @MainActor in ShareHelper.present(urls: [url]) } } }
@objc private func closeAction() { autoCloseTimer?.invalidate(); autoCloseTimer = nil; panel?.orderOut(nil); panel = nil }
@objc private func revealAction() { if let url = fileURL { NSWorkspace.shared.activateFileViewerSelecting([url]) } }
@objc private func quickLookAction() { if let url = fileURL { NSWorkspace.shared.open(url) } }
@objc private func rotateLeftAction() {
guard let url = fileURL else { return }
let out = url.deletingLastPathComponent().appendingPathComponent("qao-rot-\(Int(Date().timeIntervalSince1970)).png")
if ShotCore.fxApply(inPath: url.path, outPath: out.path, opJson: "{\"op\":\"rotate\",\"deg\":270}") == 0, let img = loadCGImage(out) { present(image: img) }
}
}

extension AppModel {
func showQAO(_ image: CGImage) {
lastImage = image
CaptureHistory.shared.add(image)
let qao = QuickAccessOverlay.shared
qao.onAnnotate = { [weak self] img in self?.openEditor(with: img); self?.editorOpener?() }
qao.onUpload = { url in
let server = UserDefaults.standard.string(forKey: "ss.serverUrl") ?? ""
guard !server.isEmpty else { Toast.show("Configure a Cloud server in Settings first"); return }
Toast.show("Uploading...")
Task { @MainActor in
if let link = await CloudClient(baseURL: server).uploadImage(fileURL: url) {
let pb = NSPasteboard.general; pb.clearContents(); pb.setString(link, forType: .string)
Toast.show("Link copied: " + link)
} else { Toast.show("Upload failed") }
}
}
let sd = UserDefaults.standard
if sd.bool(forKey: "ss.scCopy") { let pb = NSPasteboard.general; pb.clearContents(); pb.writeObjects([NSImage(cgImage: image, size: NSSize(width: image.width, height: image.height))]) }
if sd.bool(forKey: "ss.scSave") { saveCaptureToExportLocation(image) }
if sd.bool(forKey: "ss.scShowOverlay") { qao.present(image: image) }
}
}


/// Save a capture to the configured export location (Desktop/Pictures/Downloads).
func saveCaptureToExportLocation(_ image: CGImage) {
let loc = UserDefaults.standard.string(forKey: "ss.exportLocation") ?? "Desktop"
let dir: FileManager.SearchPathDirectory = loc == "Pictures" ? .picturesDirectory : (loc == "Downloads" ? .downloadsDirectory : .desktopDirectory)
let base = FileManager.default.urls(for: dir, in: .userDomainMask).first ?? FileManager.default.temporaryDirectory
let url = base.appendingPathComponent("SloerShot-\(Int(Date().timeIntervalSince1970)).png")
_ = writePNG(image, to: url)
}
