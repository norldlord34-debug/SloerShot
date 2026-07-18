import AppKit
import Foundation
import UniformTypeIdentifiers

// macOS Tools UI for the ShareX-parity FFIs (hashes, QR, folder index, image split).
enum MacTools {
 static func showHashes() {
 let panel = NSOpenPanel()
 panel.allowsMultipleSelection = false
 guard panel.runModal() == .OK, let url = panel.url else { return }
 guard let json = ShotCore.fileHashes(path: url.path) else { Toast.show("Hashing failed"); return }
 let alert = NSAlert()
 alert.messageText = "Hashes: " + url.lastPathComponent
 alert.informativeText = prettyHashes(json)
 alert.addButton(withTitle: "Copy")
 alert.addButton(withTitle: "Close")
 if alert.runModal() == .alertFirstButtonReturn {
 NSPasteboard.general.clearContents()
 NSPasteboard.general.setString(json, forType: .string)
 }
 }
 private static func prettyHashes(_ json: String) -> String {
 guard let data = json.data(using: .utf8),
 let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return json }
 var lines: [String] = []
 for key in ["md5", "sha1", "sha256", "sha512", "crc32"] {
 if let v = obj[key] as? String { lines.append(key.uppercased() + ": " + v) }
 }
 return lines.joined(separator: "\n")
 }
 static func qrFromClipboard() {
 guard let text = NSPasteboard.general.string(forType: .string), !text.isEmpty else { Toast.show("Clipboard has no text"); return }
 let sp = NSSavePanel()
 sp.allowedContentTypes = [.png]
 sp.nameFieldStringValue = "qr.png"
 guard sp.runModal() == .OK, let dest = sp.url else { return }
 let rc = ShotCore.qrEncodePng(text: text, scale: 8, quiet: 4, outPath: dest.path)
 Toast.show(rc == 0 ? "QR saved: " + dest.lastPathComponent : "QR failed")
 }
 static func indexFolder() {
 let panel = NSOpenPanel()
 panel.canChooseDirectories = true
 panel.canChooseFiles = false
 guard panel.runModal() == .OK, let dir = panel.url else { return }
 let sp = NSSavePanel()
 sp.allowedContentTypes = [.html]
 sp.nameFieldStringValue = "index.html"
 guard sp.runModal() == .OK, let dest = sp.url else { return }
 let rc = ShotCore.indexFolder(rootPath: dir.path, format: "html", outPath: dest.path)
 if rc == 0 { NSWorkspace.shared.open(dest); Toast.show("Folder indexed") } else { Toast.show("Index failed") }
 }
 static func splitImage() {
 let panel = NSOpenPanel()
 panel.allowedContentTypes = [.png, .jpeg, .image]
 guard panel.runModal() == .OK, let url = panel.url else { return }
 let alert = NSAlert()
 alert.messageText = "Split image"
 alert.informativeText = "Rows and columns (e.g. 2,2):"
 let field = NSTextField(string: "2,2")
 field.frame = NSRect(x: 0, y: 0, width: 200, height: 24)
 alert.accessoryView = field
 alert.addButton(withTitle: "Split")
 alert.addButton(withTitle: "Cancel")
 guard alert.runModal() == .alertFirstButtonReturn else { return }
 let parts = field.stringValue.split(separator: Character(","))
 let rows = parts.count > 0 ? (UInt32(parts[0].trimmingCharacters(in: .whitespaces)) ?? 2) : 2
 let cols = parts.count > 1 ? (UInt32(parts[1].trimmingCharacters(in: .whitespaces)) ?? 2) : 2
 let out = NSOpenPanel()
 out.canChooseDirectories = true
 out.canChooseFiles = false
 out.prompt = "Choose output folder"
 guard out.runModal() == .OK, let outDir = out.url else { return }
 let rc = ShotCore.splitImage(inPath: url.path, rows: rows, cols: cols, outDir: outDir.path, base: "tile")
 Toast.show(rc == 0 ? "Split into tiles" : "Split failed")
 }
}
