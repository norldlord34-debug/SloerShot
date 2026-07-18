import SwiftUI
import AppKit

struct DestinationsView: View {
 @ObservedObject private var store = DestinationStore.shared
 @AppStorage("ss.imgurClientId") private var imgurClientId = ""
 @AppStorage("ss.afterUploadCopy") private var afterUploadCopy = true
 @AppStorage("ss.afterUploadOpen") private var afterUploadOpen = false
 @AppStorage("ss.afterUploadQr") private var afterUploadQr = false
 @AppStorage("ss.urlShortener") private var urlShortener = "none"
 @AppStorage("ss.afterCaptureUpload") private var afterCaptureUpload = false
 @State private var pasteJson = ""
 var body: some View {
 Form {
 Section("Active destination") {
 Picker("Upload to", selection: Binding(get: { store.activeId }, set: { store.setActive($0) })) {
 ForEach(store.destinations) { d in Text(d.name).tag(d.id) }
 }
 }
 Section("Destinations") {
 ForEach(store.destinations) { d in
 HStack {
 Text(d.name)
 if d.builtIn { Text("built-in").font(.caption).foregroundStyle(.secondary) }
 Spacer()
 if !d.builtIn { Button("Remove") { store.remove(d) } }
 }
 }
 }
 Section("Add custom (paste ShareX JSON or .sxcu)") {
 TextEditor(text: $pasteJson).frame(height: 80).font(.system(.caption, design: .monospaced))
 HStack {
 Button("Add") { addPasted() }
 Button("Import .sxcu...") { importSxcu() }
 Button("Pastebin") { pasteJson = BuiltInDestinations.pastebinTemplate }
 Button("Bearer") { pasteJson = BuiltInDestinations.bearerTemplate }
 }
 }
 Section("After upload") {
 Toggle("Copy URL to clipboard", isOn: $afterUploadCopy)
 Toggle("Open URL in browser", isOn: $afterUploadOpen)
 Toggle("Show QR of URL", isOn: $afterUploadQr)
 Picker("Shorten URL", selection: $urlShortener) { Text("None").tag("none"); Text("is.gd").tag("isgd"); Text("TinyURL").tag("tinyurl") }
 }
 Section("After capture") { Toggle("Auto-upload new captures", isOn: $afterCaptureUpload) }
 Section("Imgur") { TextField("Imgur Client ID", text: $imgurClientId) }
 Text("Use %SERVER% for your backend URL (set it in the Cloud tab).").font(.caption).foregroundStyle(.secondary)
 }
 .formStyle(.grouped)
 .padding()
 .frame(width: 520, height: 470)
 }
 private func addPasted() {
 let t = pasteJson.trimmingCharacters(in: .whitespacesAndNewlines)
 guard !t.isEmpty else { return }
 store.add(UploadDestination(id: UUID().uuidString, name: extractName(t) ?? "Custom uploader", configJson: t, builtIn: false))
 pasteJson = ""
 }
 private func importSxcu() {
 let panel = NSOpenPanel()
 panel.allowsMultipleSelection = false
 if panel.runModal() == .OK, let url = panel.url, let text = try? String(contentsOf: url, encoding: .utf8) {
 store.add(UploadDestination(id: UUID().uuidString, name: extractName(text) ?? url.deletingPathExtension().lastPathComponent, configJson: text, builtIn: false))
 }
 }
 private func extractName(_ json: String) -> String? {
 guard let data = json.data(using: .utf8), let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any], let n = obj["Name"] as? String, !n.isEmpty else { return nil }
 return n
 }
}
