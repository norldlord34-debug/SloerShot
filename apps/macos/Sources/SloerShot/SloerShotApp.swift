import AppKit
import SwiftUI

/// Orchestrates capture and hands the result to the editor. Capture methods are async
/// and return whether an editor is ready, so the caller can open the editor window.
@MainActor
final class AppModel: ObservableObject {
 @Published var editor: EditorModel?
 @Published var lastError: String?

 @discardableResult
 func captureFullscreen() async -> Bool {
 guard #available(macOS 14.0, *) else { lastError = "Requires macOS 14+"; return false }
 do {
 let image = try await Capture.captureFullscreen()
 openEditor(with: image)
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
 let frozen = try await Capture.captureFullscreen()
 guard let selection = await SelectionOverlay.present(image: frozen) else { return false }
 guard let cropped = Capture.crop(frozen, to: selection) else { return false }
 openEditor(with: cropped)
 return true
 } catch {
 lastError = String(describing: error)
 return false
 }
 }

 func openEditor(with image: CGImage) {
 editor = EditorModel(background: image, width: UInt32(image.width), height: UInt32(image.height))
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
 }
}

/// The menu-bar dropdown. Owns the openWindow action so capture can surface the editor.
struct MenuContent: View {
 @ObservedObject var model: AppModel
 @Environment(\.openWindow) private var openWindow

 var body: some View {
 Button("Capture Area") {
 Task { if await model.captureArea() { openWindow(id: "editor") } }
 }
 .keyboardShortcut("4", modifiers: [.command, .shift])

 Button("Capture Fullscreen") {
 Task { if await model.captureFullscreen() { openWindow(id: "editor") } }
 }
 .keyboardShortcut("9", modifiers: [.command, .shift])

 Divider()
 Button("Open Editor") { openWindow(id: "editor") }
 Text("shotcore \(ShotCore.version())")
 Divider()
 Button("Quit SloerShot") { NSApplication.shared.terminate(nil) }
 .keyboardShortcut("q")
 }
}
