import AppKit
import SwiftUI

/// Orchestrates capture and hands the result to the editor. Capture methods are async
/// and return whether an editor is ready, so the caller can open the editor window.
@MainActor
final class AppModel: ObservableObject {
 @Published var editor: EditorModel?
 @Published var lastError: String?
@Published var captureDelaySeconds: Int = 0
var lastImage: CGImage?
var pins: [PinPanel] = []
var recorder: RecordingEngine?
@Published var isRecording = false
var editorOpener: (() -> Void)?
 var pendingWorkflowUpload = false
 init() { WorkflowStore.shared.attach(self) }

 @discardableResult
 func captureFullscreen() async -> Bool {
 guard #available(macOS 14.0, *) else { lastError = "Requires macOS 14+"; return false }
 do {
 await applyDelay()
let image = try await Capture.captureFullscreen()
 showQAO(image)
 return true
 } catch {
 lastError = String(describing: error)
 return false
 }
 }

 @discardableResult
 func captureArea() async -> Bool {
 guard #available(macOS 14.0, *) else { lastError = "Requires macOS 14+"; return false }
 do {
 await applyDelay()
let frozen = try await Capture.captureFullscreen()
 guard let selection = await SelectionOverlay.present(image: frozen) else { return false }
 guard let cropped = Capture.crop(frozen, to: selection) else { return false }
 showQAO(cropped)
 return true
 } catch {
 lastError = String(describing: error)
 return false
 }
 }

 func runWorkflowMode(_ mode: String, autoUpload: Bool) {
switch mode {
case "record": if isRecording { stopRecording() } else { startRecording() }
case "window": pendingWorkflowUpload = autoUpload; Task { await captureWindow() }
case "full": pendingWorkflowUpload = autoUpload; Task { await captureFullscreen() }
default: pendingWorkflowUpload = autoUpload; Task { await captureArea() }
}
}
func uploadLast() {
guard let img = lastImage else { lastError = "Nothing to upload"; return }
guard let dest = DestinationStore.shared.active else { Toast.show("No upload destination"); return }
guard let fileURL = UploaderEngine.writeTempPNG(img) else { Toast.show("Encode failed"); return }
let cfg = DestinationStore.shared.resolveConfig(dest)
Toast.show("Uploading to " + dest.name + "...")
Task {
let outcome = await UploaderEngine.upload(configJson: cfg, fileURL: fileURL)
guard outcome.success else { Toast.show("Upload failed: " + outcome.error); return }
await self.afterUpload(url: outcome.url)
}
}
func afterUpload(url initial: String) async {
let d = UserDefaults.standard
var link = initial
let shortener = d.string(forKey: "ss.urlShortener") ?? "none"
var scfg: String? = nil
if shortener == "isgd" { scfg = BuiltInShorteners.isgd } else if shortener == "tinyurl" { scfg = BuiltInShorteners.tinyurl }
if let s = scfg {
Toast.show("Shortening...")
if let short = await UploaderEngine.shorten(configJson: s, longURL: initial) { link = short }
}
if d.bool(forKey: "ss.afterUploadCopy") { NSPasteboard.general.clearContents(); NSPasteboard.general.setString(link, forType: .string) }
Toast.show("Uploaded - " + link)
if d.bool(forKey: "ss.afterUploadOpen"), let u = URL(string: link) { NSWorkspace.shared.open(u) }
if d.bool(forKey: "ss.afterUploadQr") { showQR(for: link) }
}
func showQR(for text: String) {
let out = FileManager.default.temporaryDirectory.appendingPathComponent("sloershot-qr-" + UUID().uuidString + ".png")
if ShotCore.qrEncodePng(text: text, scale: 8, quiet: 4, outPath: out.path) == 0 { NSWorkspace.shared.open(out) }
}
func openEditor(with image: CGImage) {
 editor = EditorModel(background: image, width: UInt32(image.width), height: UInt32(image.height))
lastImage = image
 }
}

@main
struct SloerShotApp: App {
 @StateObject private var model = AppModel()
 init() {
 UserDefaults.standard.register(defaults: [
 "ss.qaoPosition": "Bottom Left",
 "ss.pinRounded": true,
 "ss.pinShadow": true,
 "ss.pinBorder": true,
 "ss.scShowOverlay": true,
 "ss.scCopy": false,
 "ss.scSave": false,
 "ss.recShowCursor": true,
 "ss.recFPS": 60,
 "ss.recSystemAudio": false,
 "ss.recHighlightClicks": true,
 "ss.recShowKeystrokes": false,
 "ss.recCamera": false,
 "ss.scrollFrames": 8,
 "ss.recCameraCorner": "Bottom Left",
 "ss.recCameraSize": 0.18,
 "ss.qaoAutoClose": false,
 "ss.qaoInterval": 30,
 "ss.exportLocation": "Desktop",
"ss.afterUploadCopy": true,
"ss.afterUploadOpen": false,
"ss.afterUploadQr": false,
"ss.urlShortener": "none",
"ss.afterCaptureUpload": false
 ])
 }

 var body: some Scene {
 MenuBarExtra("SloerShot", systemImage: "camera.viewfinder") {
 MenuContent(model: model)
 }
 Window("SloerShot", id: "editor") {
EditorHost(model: model)
}
Settings {
SettingsView()
}
Window("Capture History", id: "history") {
HistoryView(model: model)
}
 }
}

/// The menu-bar dropdown. Owns the openWindow action so capture can surface the editor.
struct MenuContent: View {
 @ObservedObject var model: AppModel
 @Environment(\.openWindow) private var openWindow

 var body: some View {
Section("Capture") {
Button("All-In-One Bar") { model.editorOpener = { openWindow(id: "editor") }; AllInOneBar.shared.present(model: model, openEditor: { openWindow(id: "editor") }, openHistory: { openWindow(id: "history") }) }
.keyboardShortcut("a", modifiers: [.command, .shift])
Button("Capture Area") { model.editorOpener = { openWindow(id: "editor") }; Task { await model.captureArea() } }
.keyboardShortcut("4", modifiers: [.command, .shift])
Button("Capture Fullscreen") { model.editorOpener = { openWindow(id: "editor") }; Task { await model.captureFullscreen() } }
.keyboardShortcut("9", modifiers: [.command, .shift])
Button("Capture Window") { model.editorOpener = { openWindow(id: "editor") }; Task { await model.captureWindow() } }
.keyboardShortcut("5", modifiers: [.command, .shift])
Button("Scroll Capture (window)") { model.editorOpener = { openWindow(id: "editor") }; Task { await model.scrollCapture() } }
.keyboardShortcut("6", modifiers: [.command, .shift])
Picker("Capture delay", selection: $model.captureDelaySeconds) { Text("None").tag(0); Text("3 seconds").tag(3); Text("5 seconds").tag(5); Text("10 seconds").tag(10) }
}
Section("Recording") {
if model.isRecording {
Button("Stop Recording") { model.stopRecording() }.keyboardShortcut("2", modifiers: [.command, .shift])
} else {
Button("Start Recording") { model.startRecording() }.keyboardShortcut("2", modifiers: [.command, .shift])
}
Button("Trim Last Recording...") { guard let src = GifExporter.latestRecording() else { Toast.show("No recordings found"); return }; VideoTrimmer.show(url: src) }
Button("Export Recording to GIF...") { guard let src = GifExporter.latestRecording() else { Toast.show("No recordings found"); return }; let sp = NSSavePanel(); sp.allowedContentTypes = [.gif]; sp.nameFieldStringValue = "SloerShot.gif"; if sp.runModal() == .OK, let dest = sp.url { Toast.show("Exporting GIF..."); Task { let ok = await GifExporter.export(from: src, to: dest); Toast.show(ok ? "Saved " + dest.lastPathComponent : "GIF export failed") } } }
}
Section("Tools") {
Button("Quick OCR (copy text)") { Task { await model.quickOCR() } }.keyboardShortcut("t", modifiers: [.command, .shift])
Button("Capture Text to Window") { Task { await model.ocrToPanel() } }.keyboardShortcut("o", modifiers: [.command, .shift])
Button("Pick Color (hex)") { model.pickColor() }.keyboardShortcut("c", modifiers: [.command, .shift])
Button("Pin Last Capture") { model.pinLast() }.keyboardShortcut("p", modifiers: [.command, .shift])
Button("Upload Last Capture") { model.uploadLast() }.keyboardShortcut("u", modifiers: [.command, .shift])
Button("File Hashes...") { MacTools.showHashes() }
Button("QR from Clipboard...") { MacTools.qrFromClipboard() }
Button("Index Folder...") { MacTools.indexFolder() }
Button("Split Image...") { MacTools.splitImage() }
}
Section("Library") {
Button("Open Editor") { openWindow(id: "editor") }
Button("Combine Images...") { let panel = NSOpenPanel(); panel.allowedContentTypes = [.png, .jpeg, .image]; panel.allowsMultipleSelection = true; if panel.runModal() == .OK, !panel.urls.isEmpty, let img = CombineImages.combine(urls: panel.urls) { model.openEditor(with: img); openWindow(id: "editor") } }
Button("Open Image...") { let panel = NSOpenPanel(); panel.allowedContentTypes = [.png, .jpeg, .image]; panel.allowsMultipleSelection = false; if panel.runModal() == .OK, let url = panel.url, let img = loadCGImage(url) { model.openEditor(with: img); openWindow(id: "editor") } }
Button("Capture History") { openWindow(id: "history") }.keyboardShortcut("h", modifiers: [.command, .shift])
}
Divider()
SettingsLink { Text("Settings...") }.keyboardShortcut(",", modifiers: .command)
Text("shotcore \(ShotCore.version())")
Divider()
Button("Quit SloerShot") { NSApplication.shared.terminate(nil) }.keyboardShortcut("q")
}
}