import AppKit
import SwiftUI

/// Presents a full-screen selection overlay over a frozen capture and returns the
/// chosen rect in image pixel space (or nil if cancelled / too small).
@available(macOS 14.0, *)
enum SelectionOverlay {
 @MainActor
 static func present(image: CGImage) async -> CGRect? {
 await withCheckedContinuation { cont in
 let controller = OverlayController(image: image) { rect in
 cont.resume(returning: rect)
 }
 controller.show()
 }
 }
}

/// Owns the borderless overlay window for one selection.
@MainActor
final class OverlayController {
 private var window: NSWindow?
 private let image: CGImage
 private let completion: (CGRect?) -> Void
 private var finished = false

 init(image: CGImage, completion: @escaping (CGRect?) -> Void) {
 self.image = image
 self.completion = completion
 }

 func show() {
 guard let screen = NSScreen.main else { finish(nil); return }
 let frame = screen.frame
 let scaleX = CGFloat(image.width) / frame.width
 let scaleY = CGFloat(image.height) / frame.height
 let win = NSWindow(contentRect: frame, styleMask: [.borderless], backing: .buffered, defer: false)
 win.level = .screenSaver
 win.isOpaque = false
 win.backgroundColor = .clear
 let view = SelectionView(image: image, viewSize: frame.size, scaleX: scaleX, scaleY: scaleY) { [weak self] rect in
 self?.finish(rect)
 }
 win.contentView = NSHostingView(rootView: view)
 win.makeKeyAndOrderFront(nil)
 NSApp.activate(ignoringOtherApps: true)
 window = win
 }

 private func finish(_ rect: CGRect?) {
 guard !finished else { return }
 finished = true
 window?.orderOut(nil)
 window = nil
 completion(rect)
 }
}

/// Dims the frozen frame and lets the user drag a selection rectangle.
struct SelectionView: View {
 let image: CGImage
 let viewSize: CGSize
 let scaleX: CGFloat
 let scaleY: CGFloat
 let onComplete: (CGRect?) -> Void
 @State private var aspect: Double = 0
@State private var hover: CGPoint?
@State private var startPt: CGPoint?
 @State private var endPt: CGPoint?

 private var selection: CGRect? {
 guard let s = startPt, let e = endPt else { return nil }
 let x = min(s.x, e.x), y = min(s.y, e.y)
var w = abs(e.x - s.x), h = abs(e.y - s.y)
if aspect > 0 { h = w / aspect }
return CGRect(x: x, y: y, width: w, height: h)
 }

 private var cursorOverlay: some View {
ZStack {
if let p = endPt ?? hover {
Path { path in
path.move(to: CGPoint(x: p.x, y: 0)); path.addLine(to: CGPoint(x: p.x, y: viewSize.height))
path.move(to: CGPoint(x: 0, y: p.y)); path.addLine(to: CGPoint(x: viewSize.width, y: p.y))
}.stroke(Color.white.opacity(0.45), lineWidth: 1)
loupe(at: p)
}
}
.frame(width: viewSize.width, height: viewSize.height)
.allowsHitTesting(false)
}
private func loupe(at p: CGPoint) -> some View {
let size: CGFloat = 120
let srcPx: CGFloat = 15
let ixc = p.x * scaleX, iyc = p.y * scaleY
let rect = CGRect(x: ixc - srcPx / 2, y: iyc - srcPx / 2, width: srcPx, height: srcPx).integral
let cropped = image.cropping(to: rect)
var lx = p.x + 72, ly = p.y + 72
if lx + size / 2 > viewSize.width { lx = p.x - 72 }
if ly + size / 2 > viewSize.height { ly = p.y - 72 }
return VStack(spacing: 4) {
Group {
if let c = cropped {
Image(decorative: c, scale: 1, orientation: .up).resizable().interpolation(.none).frame(width: size, height: size)
} else {
Color.black.frame(width: size, height: size)
}
}
.clipShape(RoundedRectangle(cornerRadius: 10))
.overlay(RoundedRectangle(cornerRadius: 10).stroke(Color.white, lineWidth: 2))
.overlay(Rectangle().stroke(Color.white.opacity(0.7), lineWidth: 1).frame(width: size / srcPx, height: size / srcPx))
Text("\(Int(ixc)), \(Int(iyc))").font(.system(size: 11, design: .monospaced)).foregroundColor(.white).padding(.horizontal, 6).padding(.vertical, 2).background(Color.black.opacity(0.7)).cornerRadius(4)
}
.position(x: lx, y: ly)
}
private var aspectBar: some View {
HStack(spacing: 8) {
aspectButton("Free", 0)
aspectButton("1:1", 1)
aspectButton("4:3", 4.0 / 3.0)
aspectButton("16:9", 16.0 / 9.0)
aspectButton("9:16", 9.0 / 16.0)
}
.padding(8)
.background(Color.black.opacity(0.6))
.cornerRadius(8)
.padding(.top, 22)
}
private func aspectButton(_ label: String, _ value: Double) -> some View {
Button(label) { aspect = value }
.buttonStyle(.plain)
.padding(.horizontal, 8)
.padding(.vertical, 4)
.background(aspect == value ? Color.accentColor : Color.white.opacity(0.15))
.foregroundColor(.white)
.cornerRadius(5)
}
var body: some View {
 Image(decorative: image, scale: 1, orientation: .up)
 .resizable()
 .frame(width: viewSize.width, height: viewSize.height)
 .overlay(Color.black.opacity(0.3))
 .overlay(alignment: .topLeading) {
 if let r = selection {
 Rectangle()
 .strokeBorder(Color.white, lineWidth: 2)
 .frame(width: r.width, height: r.height)
 .offset(x: r.minX, y: r.minY)
 }
 }
 .overlay(alignment: .top) { aspectBar }
.overlay { cursorOverlay }
.onContinuousHover(coordinateSpace: .local) { phase in
switch phase {
case .active(let loc): hover = loc
case .ended: hover = nil
}
}
.contentShape(Rectangle())
 .gesture(
 DragGesture(minimumDistance: 0)
 .onChanged { v in
 if startPt == nil { startPt = v.startLocation }
 endPt = v.location
 }
 .onEnded { _ in
 guard let r = selection, r.width > 3, r.height > 3 else {
 onComplete(nil)
 return
 }
 let imgRect = CGRect(x: r.minX * scaleX, y: r.minY * scaleY,
 width: r.width * scaleX, height: r.height * scaleY)
 onComplete(imgRect)
 }
 )
 }
}
