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

 func openEditor(with image: CGImage) {
 editor = EditorModel(background: image, width: UInt32(image.width), height: UInt32(image.height))
lastImage = image
 }
}

@main
struct SloerShotApp: App {
 @StateObject private var model = AppModel()

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
 Button("Capture Area") {
 model.editorOpener = { openWindow(id: "editor") }
Task { await model.captureArea() }
 }
 .keyboardShortcut("4", modifiers: [.command, .shift])

 Button("Capture Fullscreen") {
 model.editorOpener = { openWindow(id: "editor") }
Task { await model.captureFullscreen() }
 }
 .keyboardShortcut("9", modifiers: [.command, .shift])
Button("Capture Window") {
model.editorOpener = { openWindow(id: "editor") }
Task { await model.captureWindow() }
}
.keyboardShortcut("5", modifiers: [.command, .shift])
Button("Scroll Capture (window)") {
model.editorOpener = { openWindow(id: "editor") }
Task { await model.scrollCapture() }
}
.keyboardShortcut("6", modifiers: [.command, .shift])
if model.isRecording {
Button("Stop Recording") { model.stopRecording() }
.keyboardShortcut("2", modifiers: [.command, .shift])
} else {
Button("Start Recording") { model.startRecording() }
.keyboardShortcut("2", modifiers: [.command, .shift])
}
Button("Quick OCR (copy text)") {
Task { await model.quickOCR() }
}
.keyboardShortcut("t", modifiers: [.command, .shift])
Button("Pick Color (hex)") {
model.pickColor()
}
.keyboardShortcut("c", modifiers: [.command, .shift])
Button("Pin Last Capture") {
model.pinLast()
}
.keyboardShortcut("p", modifiers: [.command, .shift])
Picker("Capture delay", selection: $model.captureDelaySeconds) {
Text("None").tag(0)
Text("3 seconds").tag(3)
Text("5 seconds").tag(5)
Text("10 seconds").tag(10)
}

 Divider()
 Button("Open Editor") { openWindow(id: "editor") }
Button("Capture History") { openWindow(id: "history") }
.keyboardShortcut("h", modifiers: [.command, .shift])
SettingsLink { Text("Settings...") }
.keyboardShortcut(",", modifiers: .command)
 Text("shotcore \(ShotCore.version())")
 Divider()
 Button("Quit SloerShot") { NSApplication.shared.terminate(nil) }
 .keyboardShortcut("q")
 }
}
