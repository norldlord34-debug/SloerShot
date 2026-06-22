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
var key: KeyEquivalent {
switch self {
case .arrow: return "a"
case .rectangle: return "r"
case .ellipse: return "o"
case .line: return "l"
case .freehand: return "p"
case .text: return "t"
case .counter: return "n"
case .highlighter: return "h"
case .redact: return "b"
}
}
}

/// Drives the shared core editor and republishes its render output for the canvas.
@MainActor
final class EditorModel: ObservableObject {
 private(set) var handle: EditorHandle
 @Published private(set) var background: CGImage?
 @Published private(set) var imageSize: CGSize
 @Published var specs: [ShapeSpec] = []
 @Published var tool: UInt32 = Tool.arrow.rawValue
 @Published var canUndo = false
 @Published var canRedo = false
 @Published var showBackgroundPanel = false
 @Published var backgroundPreview: NSImage?
@Published var strokeColor = Color.red
@Published var fillEnabled = false
@Published var fillColor = Color(red: 0.0, green: 0.48, blue: 1.0)
@Published var strokeWidth = 4.0
@Published var styleOpacity = 1.0
@Published var arrowStyle = "Straight"
@Published var filledShapes = false
@Published var textStyle = "Plain"
@Published var smartHighlighter = false
@Published var pencilSmooth = true

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

 func replaceImage(_ img: CGImage) {
guard let h = EditorHandle(width: UInt32(img.width), height: UInt32(img.height)) else { return }
handle = h
background = img
imageSize = CGSize(width: Double(img.width), height: Double(img.height))
handle.setTool(tool)
refresh()
}
func setText(_ text: String) { _ = handle.setSelectedText(text); refresh() }
func styleJSON() -> String {
let s = ssColorRGBA(strokeColor)
let stroke = "{\"r\":\(s.0),\"g\":\(s.1),\"b\":\(s.2),\"a\":\(s.3)}"
var fill = "null"
if fillEnabled { let f = ssColorRGBA(fillColor); fill = "{\"r\":\(f.0),\"g\":\(f.1),\"b\":\(f.2),\"a\":\(f.3)}" }
return "{\"stroke\":\(stroke),\"fill\":\(fill),\"stroke_width\":\(strokeWidth),\"opacity\":\(styleOpacity),\"arrow_style\":\"\(arrowStyle)\",\"filled\":\(filledShapes),\"text_style\":\"\(textStyle)\",\"highlighter_smart\":\(smartHighlighter),\"pencil_smooth\":\(pencilSmooth)}"
}
func applyStyle() { _ = handle.setStyleJson(styleJSON()); refresh() }
func applyFx(_ opJson: String) -> Bool {
guard let bg = background else { return false }
let dir = FileManager.default.temporaryDirectory
let tin = dir.appendingPathComponent("ss-fx-in.png")
let tout = dir.appendingPathComponent("ss-fx-out.png")
guard writePNG(bg, to: tin) else { return false }
guard ShotCore.fxApply(inPath: tin.path, outPath: tout.path, opJson: opJson) == 0 else { return false }
guard let img = loadCGImage(tout) else { return false }
replaceImage(img)
return true
}
func beautify(from src: CGImage, json: String) -> Bool {
let dir = FileManager.default.temporaryDirectory
let tin = dir.appendingPathComponent("ss-bg-in.png")
let tout = dir.appendingPathComponent("ss-bg-out.png")
guard writePNG(src, to: tin) else { return false }
guard ShotCore.beautify(inPath: tin.path, outPath: tout.path, optionsJson: json) == 0 else { return false }
guard let img = loadCGImage(tout) else { return false }
replaceImage(img)
return true
}
func applyBeautify(_ json: String) -> Bool {
guard let bg = background else { return false }
return beautify(from: bg, json: json)
}
/// Bake current annotations into a flattened CGImage to feed the background tool.
func flattenedImage() -> CGImage? {
guard let url = flattenedURL() else { return nil }
return loadCGImage(url)
}
/// Non-destructive preview: beautify a source image to an NSImage without mutating the editor.
func beautifyPreview(source: CGImage, json: String) -> NSImage? {
let dir = FileManager.default.temporaryDirectory
let tin = dir.appendingPathComponent("ss-bgprev-in.png")
let tout = dir.appendingPathComponent("ss-bgprev-out.png")
guard writePNG(source, to: tin) else { return nil }
guard ShotCore.beautify(inPath: tin.path, outPath: tout.path, optionsJson: json) == 0 else { return nil }
return NSImage(contentsOf: tout)
}
/// Commit the background tool: bake annotations, then wrap in the chosen background.
@discardableResult func commitBackground(json: String) -> Bool {
guard let src = flattenedImage() else { return false }
return beautify(from: src, json: json)
}
func applyEffect(_ key: String) { if let j = EditorModel.effectJson(key) { _ = applyFx(j) } }
func applyBackdrop(_ key: String) { _ = applyBeautify(EditorModel.beautifyJson(key)) }
func flattenedURL() -> URL? {
guard let bg = background else { return nil }
let dir = FileManager.default.temporaryDirectory
let tin = dir.appendingPathComponent("ss-flat-in.png")
let tout = dir.appendingPathComponent("ss-flat-out.png")
guard writePNG(bg, to: tin) else { return nil }
if let doc = documentJSON(), ShotCore.export(imagePath: tin.path, docJson: doc, outPath: tout.path) == 0 { return tout }
return tin
}
func copyToClipboard() -> Bool {
guard let url = flattenedURL(), let img = NSImage(contentsOf: url) else { return false }
let pb = NSPasteboard.general
pb.clearContents()
pb.writeObjects([img])
return true
}
static func effectJson(_ key: String) -> String? {
switch key {
case "grayscale": return "{\"op\":\"grayscale\"}"
case "sepia": return "{\"op\":\"sepia\"}"
case "invert": return "{\"op\":\"invert\"}"
case "blur": return "{\"op\":\"blur\",\"sigma\":4.0}"
case "vignette": return "{\"op\":\"vignette\",\"strength\":0.6}"
case "brighten": return "{\"op\":\"brightness\",\"delta\":30}"
case "contrast": return "{\"op\":\"contrast\",\"factor\":1.3}"
case "rotate": return "{\"op\":\"rotate\",\"deg\":90}"
case "flip": return "{\"op\":\"flip\",\"axis\":\"h\"}"
case "border": return "{\"op\":\"border\",\"thickness\":16,\"color\":{\"r\":255,\"g\":255,\"b\":255,\"a\":255}}"
default: return nil
}
}
static func beautifyJson(_ key: String) -> String {
let shadow = "{\"color\":{\"r\":0,\"g\":0,\"b\":0,\"a\":255},\"blur\":24.0,\"dx\":0.0,\"dy\":16.0,\"opacity\":0.35}"
switch key {
case "ocean": return "{\"background\":{\"Preset\":\"Ocean\"},\"padding\":64,\"corner_radius\":16.0,\"shadow\":" + shadow + "}"
case "sunset": return "{\"background\":{\"Preset\":\"Sunset\"},\"padding\":64,\"corner_radius\":16.0,\"shadow\":" + shadow + "}"
case "white": return "{\"background\":{\"Solid\":{\"r\":255,\"g\":255,\"b\":255,\"a\":255}},\"padding\":48,\"corner_radius\":12.0,\"shadow\":" + shadow + "}"
case "dark": return "{\"background\":{\"Solid\":{\"r\":24,\"g\":24,\"b\":28,\"a\":255}},\"padding\":48,\"corner_radius\":12.0,\"shadow\":" + shadow + "}"
case "tight": return "{\"background\":{\"Preset\":\"Graphite\"},\"padding\":24,\"corner_radius\":10.0,\"shadow\":null}"
default: return "{\"background\":{\"Preset\":\"Indigo\"},\"padding\":64,\"corner_radius\":16.0,\"shadow\":" + shadow + "}"
}
}
func documentJSON() -> String? { handle.documentJson() }
}

struct EditorView: View {
 @StateObject var model: EditorModel
 @State private var status = ""
 @State private var showStyle = false

 var body: some View {
HStack(spacing: 0) {
VStack(spacing: 0) {
toolbar
Divider()
ZStack {
ScrollView([.horizontal, .vertical]) {
AnnotationCanvas(
background: model.background,
specs: model.specs,
imageSize: model.imageSize,
onPointerDown: { p in model.pointerDown(p) },
onPointerDrag: { p in model.pointerDrag(p) },
onPointerUp: { p in model.pointerUp(p) }
)
.padding(16)
}
if model.showBackgroundPanel, let prev = model.backgroundPreview {
Image(nsImage: prev).resizable().scaledToFit().padding(16).frame(maxWidth: .infinity, maxHeight: .infinity).background(Color(nsColor: .windowBackgroundColor))
}
}
}
if model.showBackgroundPanel {
Divider()
BackgroundPanel(model: model).frame(width: 290)
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
.keyboardShortcut(t.key, modifiers: [])
 .background(model.tool == t.rawValue ? Color.accentColor.opacity(0.25) : Color.clear)
 .cornerRadius(4)
 }
 Divider().frame(height: 20)
 Button { model.undo() } label: { Image(systemName: "arrow.uturn.backward") }
 .disabled(!model.canUndo)
 Button { model.redo() } label: { Image(systemName: "arrow.uturn.forward") }
 .disabled(!model.canRedo)
 Spacer()
 Menu("Effects") {
Button("Grayscale") { model.applyEffect("grayscale") }
Button("Sepia") { model.applyEffect("sepia") }
Button("Invert") { model.applyEffect("invert") }
Button("Blur") { model.applyEffect("blur") }
Button("Vignette") { model.applyEffect("vignette") }
Button("Brighten") { model.applyEffect("brighten") }
Button("Contrast") { model.applyEffect("contrast") }
Button("Rotate 90") { model.applyEffect("rotate") }
Button("Flip") { model.applyEffect("flip") }
Button("Border") { model.applyEffect("border") }
}
.frame(width: 86)
Menu("Backdrop") {
Button("Indigo") { model.applyBackdrop("indigo") }
Button("Ocean") { model.applyBackdrop("ocean") }
Button("Sunset") { model.applyBackdrop("sunset") }
Button("White") { model.applyBackdrop("white") }
Button("Dark") { model.applyBackdrop("dark") }
Button("Tight") { model.applyBackdrop("tight") }
}
.frame(width: 96)
Button { model.showBackgroundPanel.toggle() } label: { Image(systemName: "photo.on.rectangle.angled") }
.help("Background tool (gradients, padding, shadow)")
.background(model.showBackgroundPanel ? Color.accentColor.opacity(0.25) : Color.clear)
.cornerRadius(4)
Button { showStyle.toggle() } label: { RoundedRectangle(cornerRadius: 4).fill(model.strokeColor).frame(width: 22, height: 22).overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.primary.opacity(0.35), lineWidth: 1)) }
.buttonStyle(.borderless)
.help("Color and style")
.popover(isPresented: $showStyle, arrowEdge: .bottom) { StylePopover(model: model) }
Button { status = model.copyToClipboard() ? "copied" : "copy failed" } label: { Image(systemName: "doc.on.doc") }
.keyboardShortcut("c", modifiers: [.command])
.help("Copy to clipboard")
Button { if let txt = promptText() { model.setText(txt); status = "text set" } } label: { Image(systemName: "textformat") }
.help("Set text of selected annotation")
Button { PinStore.pin(model.flattenedURL()) } label: { Image(systemName: "pin") }
.help("Pin to screen")
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
func loadCGImage(_ url: URL) -> CGImage? {
guard let src = CGImageSourceCreateWithURL(url as CFURL, nil) else { return nil }
return CGImageSourceCreateImageAtIndex(src, 0, nil)
}
func promptText() -> String? {
let alert = NSAlert()
alert.messageText = "Set annotation text"
alert.informativeText = "Select a text annotation first, then apply."
alert.addButton(withTitle: "Apply")
alert.addButton(withTitle: "Cancel")
let field = NSTextField(frame: NSRect(x: 0, y: 0, width: 280, height: 24))
alert.accessoryView = field
return alert.runModal() == .alertFirstButtonReturn ? field.stringValue : nil
}
enum PinStore {
static var pins: [PinPanel] = []
static func pin(_ url: URL?) {
guard let url = url, let img = NSImage(contentsOf: url) else { return }
let maxDim: CGFloat = 600
if max(img.size.width, img.size.height) > maxDim { let s = maxDim / max(img.size.width, img.size.height); img.size = NSSize(width: img.size.width * s, height: img.size.height * s) }
var origin = NSPoint(x: 140, y: 140)
if let scr = NSScreen.main { origin = NSPoint(x: scr.frame.midX - img.size.width / 2, y: scr.frame.midY - img.size.height / 2) }
let panel = PinPanel(image: img, at: origin)
panel.orderFrontRegardless()
pins.append(panel)
}
}
func writePNG(_ image: CGImage, to url: URL) -> Bool {
 guard let dest = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) else {
 return false
 }
 CGImageDestinationAddImage(dest, image, nil)
 return CGImageDestinationFinalize(dest)
}


/// Convert a SwiftUI Color to sRGB 0-255 RGBA components for core style JSON.
func ssColorRGBA(_ c: Color) -> (Int, Int, Int, Int) {
let n = NSColor(c).usingColorSpace(.sRGB) ?? NSColor.black
return (Int((n.redComponent * 255).rounded()), Int((n.greenComponent * 255).rounded()), Int((n.blueComponent * 255).rounded()), Int((n.alphaComponent * 255).rounded()))
}
