import AppKit
import ImageIO
import SwiftUI
import UniformTypeIdentifiers

// Persistent capture history: every capture is written to Application Support/SloerShot/History
// and listed in a browsable window with thumbnails, search, and quick actions.
struct HistoryItem: Identifiable, Hashable {
let url: URL
let date: Date
var id: URL { url }
}

@MainActor
final class CaptureHistory: ObservableObject {
static let shared = CaptureHistory()
@Published private(set) var items: [HistoryItem] = []
let dir: URL
private var thumbCache: [URL: NSImage] = [:]
init() {
let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first ?? FileManager.default.temporaryDirectory
dir = base.appendingPathComponent("SloerShot/History", isDirectory: true)
try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
load()
}
func load() {
let fm = FileManager.default
let urls = (try? fm.contentsOfDirectory(at: dir, includingPropertiesForKeys: [.contentModificationDateKey])) ?? []
items = urls.filter { $0.pathExtension.lowercased() == "png" }.map { url in
let d = (try? url.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate) ?? Date.distantPast
return HistoryItem(url: url, date: d)
}.sorted { $0.date > $1.date }
}
@discardableResult
func add(_ image: CGImage) -> URL? {
let url = dir.appendingPathComponent("SloerShot-\(Int(Date().timeIntervalSince1970)).png")
guard writePNG(image, to: url) else { return nil }
items.insert(HistoryItem(url: url, date: Date()), at: 0)
return url
}
func delete(_ item: HistoryItem) {
try? FileManager.default.removeItem(at: item.url)
thumbCache[item.url] = nil
items.removeAll { $0.id == item.id }
}
func clearAll() {
for it in items { try? FileManager.default.removeItem(at: it.url) }
thumbCache.removeAll()
items.removeAll()
}
func thumbnail(_ url: URL, max: Int = 260) -> NSImage? {
if let c = thumbCache[url] { return c }
guard let src = CGImageSourceCreateWithURL(url as CFURL, nil) else { return nil }
let opts: [CFString: Any] = [kCGImageSourceCreateThumbnailFromImageAlways: true, kCGImageSourceThumbnailMaxPixelSize: max, kCGImageSourceCreateThumbnailWithTransform: true]
guard let cg = CGImageSourceCreateThumbnailAtIndex(src, 0, opts as CFDictionary) else { return nil }
let img = NSImage(cgImage: cg, size: NSSize(width: cg.width, height: cg.height))
thumbCache[url] = img
return img
}
}


struct HistoryView: View {
@ObservedObject var model: AppModel
@ObservedObject var history = CaptureHistory.shared
@Environment(\.openWindow) private var openWindow
@State private var query = ""
private var filtered: [HistoryItem] {
if query.isEmpty { return history.items }
return history.items.filter { $0.url.lastPathComponent.localizedCaseInsensitiveContains(query) }
}
var body: some View {
VStack(spacing: 0) {
HStack {
Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
TextField("Search captures", text: $query).textFieldStyle(.plain)
Spacer()
Text("\(history.items.count) items").font(.caption).foregroundStyle(.secondary)
Button("Refresh") { history.load() }
Button("Clear All") { history.clearAll() }
}.padding(10)
Divider()
if filtered.isEmpty {
Spacer(); Text("No captures yet").foregroundStyle(.secondary); Spacer()
} else {
ScrollView {
LazyVGrid(columns: [GridItem(.adaptive(minimum: 210), spacing: 14)], spacing: 14) {
ForEach(filtered) { item in cell(item) }
}.padding(14)
}
}
}
.frame(minWidth: 640, minHeight: 460)
.onAppear { history.load() }
}
private func cell(_ item: HistoryItem) -> some View {
VStack(alignment: .leading, spacing: 6) {
ZStack {
RoundedRectangle(cornerRadius: 8).fill(Color(nsColor: .windowBackgroundColor))
if let thumb = history.thumbnail(item.url) { Image(nsImage: thumb).resizable().scaledToFit().padding(6) }
}
.frame(height: 150)
.overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.gray.opacity(0.25), lineWidth: 1))
Text(item.date.formatted(date: .abbreviated, time: .shortened)).font(.caption).foregroundStyle(.secondary)
HStack(spacing: 12) {
Button { open(item) } label: { Image(systemName: "square.and.pencil") }.help("Open in editor")
Button { copyItem(item) } label: { Image(systemName: "doc.on.doc") }.help("Copy")
Button { PinStore.pin(item.url) } label: { Image(systemName: "pin") }.help("Pin to screen")
Button { cloud(item) } label: { Image(systemName: "icloud.and.arrow.up") }.help("Upload to Cloud")
Button { NSWorkspace.shared.activateFileViewerSelecting([item.url]) } label: { Image(systemName: "folder") }.help("Reveal in Finder")
Spacer()
Button { history.delete(item) } label: { Image(systemName: "trash") }.help("Delete")
}.buttonStyle(.borderless)
}
}
private func open(_ item: HistoryItem) {
guard let img = loadCGImage(item.url) else { return }
model.openEditor(with: img)
openWindow(id: "editor")
}
private func cloud(_ item: HistoryItem) {
let server = UserDefaults.standard.string(forKey: "ss.serverUrl") ?? ""
guard !server.isEmpty else { Toast.show("Configure a Cloud server in Settings first"); return }
Toast.show("Uploading...")
Task { @MainActor in
if let link = await CloudClient(baseURL: server).uploadImage(fileURL: item.url) {
let pb = NSPasteboard.general; pb.clearContents(); pb.setString(link, forType: .string)
Toast.show("Link copied: " + link)
} else { Toast.show("Upload failed") }
}
}
private func copyItem(_ item: HistoryItem) {
guard let img = NSImage(contentsOf: item.url) else { return }
let pb = NSPasteboard.general; pb.clearContents(); pb.writeObjects([img])
}
}
