import AppKit
import SwiftUI
import UniformTypeIdentifiers

// Capture Text results window: review recognized text, copy/save, and (via the tested core)
// extract links or convert detected tables to CSV / Markdown.
@MainActor
enum OcrPanel {
static var window: NSWindow?
static func show(text: String, ocrJson: String?) {
let host = NSHostingView(rootView: OcrResultView(text: text, ocrJson: ocrJson))
let win = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 540, height: 440), styleMask: [.titled, .closable, .resizable], backing: .buffered, defer: false)
win.title = "Captured Text"
win.contentView = host
win.center()
win.isReleasedWhenClosed = false
win.makeKeyAndOrderFront(nil)
NSApp.activate(ignoringOtherApps: true)
window = win
}
}

struct OcrResultView: View {
@State var text: String
let ocrJson: String?
@State private var status = ""
var body: some View {
VStack(alignment: .leading, spacing: 8) {
Text("Captured Text").font(.headline)
TextEditor(text: $text).font(.system(.body, design: .monospaced)).overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.gray.opacity(0.3), lineWidth: 1))
HStack {
Button("Copy") { copyString(text); status = "copied" }
Button("Save .txt") { saveTxt() }
Button("Links") { if let j = ShotCore.extractLinks(from: text) { copyString(j); status = "links copied" } else { status = "no links" } }
if let j = ocrJson {
Button("Table to CSV") { if let c = ShotCore.tableCSV(ocrJson: j, colTolerance: 20) { copyString(c); status = "CSV copied" } }
Button("Table to Markdown") { if let m = ShotCore.tableMarkdown(ocrJson: j, colTolerance: 20) { copyString(m); status = "Markdown copied" } }
}
Spacer()
Text(status).font(.caption).foregroundStyle(.secondary)
}
}
.padding(14)
.frame(minWidth: 480, minHeight: 360)
}
private func copyString(_ s: String) { let pb = NSPasteboard.general; pb.clearContents(); pb.setString(s, forType: .string) }
private func saveTxt() {
let sp = NSSavePanel()
sp.allowedContentTypes = [UTType.plainText]
sp.nameFieldStringValue = "Captured.txt"
if sp.runModal() == .OK, let url = sp.url { try? text.data(using: .utf8)?.write(to: url); status = "saved " + url.lastPathComponent }
}
}
