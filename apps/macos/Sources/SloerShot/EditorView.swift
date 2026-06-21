import AppKit
import ImageIO
import SwiftUI
import UniformTypeIdentifiers

/// The nine annotation tools. Raw values match shotcore_editor_set_tool codes.
enum Tool: UInt32, CaseIterable {
 case arrow = 1, rectangle = 2, ellipse = 3, line = 4, freehand = 5
 case text = 6, counter = 7, highlighter = 8, redact = 9

 var symbol: String {
 switch self {
 case .arrow: return "arrow.up.right"
 case .rectangle: return "rectangle"
 case .ellipse: return "circle"
 case .line: return "line.diagonal"
 case .freehand: return "scribble"
 case .text: return "textformat"
 case .counter: return "number.circle"
 case .highlighter: return "highlighter"
 case .redact: return "eye.slash"
 }
 }

 var help: String { String(describing: self).capitalized }
}

/// Drives the shared core editor and republishes its render output for the canvas.
@MainActor
final class EditorModel: ObservableObject {
 let handle: EditorHandle
 let background: CGImage?
 let imageSize: CGSize
 @Published var specs: [ShapeSpec] = []
 @Published var tool: UInt32 = Tool.arrow.rawValue
 @Published var canUndo = false
 @Published var canRedo = false

 init?(background: CGImage?, width: UInt32, height: UInt32) {
 guard let h = EditorHandle(width: width, height: height) else { return nil }
 self.handle = h
 self.background = background
 self.imageSize = CGSize(width: Double(width), height: Double(height))
 handle.setTool(tool)
 refresh()
 }

 func setTool(_ t: UInt32) { tool = t; handle.setTool(t) }
 func pointerDown(_ p: CGPoint) { handle.pointerDown(p.x, p.y); refresh() }
 func pointerDrag(_ p: CGPoint) { handle.pointerDrag(p.x, p.y); refresh() }
 func pointerUp(_ p: CGPoint) { handle.pointerUp(p.x, p.y); refresh() }
 func undo() { handle.undo(); refresh() }
 func redo() { handle.redo(); refresh() }
 func deleteSelected() { handle.deleteSelected(); refresh() }

 func refresh() {
 specs = AnnotationParser.parse(handle.renderJson())
 canUndo = handle.canUndo()
 canRedo = handle.canRedo()
 }

 func documentJSON() -> String? { handle.documentJson() }
}

struct EditorView: View {
 @StateObject var model: EditorModel
 @State private var status = ""

 var body: some View {
 VStack(spacing: 0) {
 toolbar
 Divider()
 ScrollView([.horizontal, .vertical]) {
 AnnotationCanvas(
 background: model.background,
 specs: model.specs,
 imageSize: model.imageSize,
 onPointerDown: { model.pointerDown($0) },
 onPointerDrag: { model.pointerDrag($0) },
 onPointerUp: { model.pointerUp($0) }
 )
 .padding(16)
 }
 }
 .frame(minWidth: 640, minHeight: 480)
 }

 private var toolbar: some View {
 HStack(spacing: 6) {
 ForEach(Tool.allCases, id: \.self) { t in
 Button { model.setTool(t.rawValue) } label: {
 Image(systemName: t.symbol).frame(width: 22, height: 22)
 }
 .buttonStyle(.borderless)
 .help(t.help)
 .background(model.tool == t.rawValue ? Color.accentColor.opacity(0.25) : Color.clear)
 .cornerRadius(4)
 }
 Divider().frame(height: 20)
 Button { model.undo() } label: { Image(systemName: "arrow.uturn.backward") }
 .disabled(!model.canUndo)
 Button { model.redo() } label: { Image(systemName: "arrow.uturn.forward") }
 .disabled(!model.canRedo)
 Spacer()
 Button("Export PNG") { export() }
 Text(status).foregroundStyle(.secondary).lineLimit(1)
 }
 .padding(8)
 }

 /// Flatten through the core: write the captured image to a temp PNG, then call
 /// shotcore_export_png with the live document JSON to a user-chosen destination.
 private func export() {
 guard let bg = model.background, let docJSON = model.documentJSON() else {
 status = "nothing to export"
 return
 }
 let panel = NSSavePanel()
 panel.allowedContentTypes = [.png]
 panel.nameFieldStringValue = "SloerShot.png"
 guard panel.runModal() == .OK, let outURL = panel.url else { return }

 let tmp = FileManager.default.temporaryDirectory.appendingPathComponent("sloershot-src.png")
 guard writePNG(bg, to: tmp) else { status = "temp write failed"; return }
 let rc = ShotCore.export(imagePath: tmp.path, docJson: docJSON, outPath: outURL.path)
 status = rc == 0 ? "exported \(outURL.lastPathComponent)" : "export failed (\(rc))"
 }
}

/// Write a CGImage to a PNG file via ImageIO.
func writePNG(_ image: CGImage, to url: URL) -> Bool {
 guard let dest = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) else {
 return false
 }
 CGImageDestinationAddImage(dest, image, nil)
 return CGImageDestinationFinalize(dest)
}
