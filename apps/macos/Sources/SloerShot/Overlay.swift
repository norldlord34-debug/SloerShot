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
 @State private var startPt: CGPoint?
 @State private var endPt: CGPoint?

 private var selection: CGRect? {
 guard let s = startPt, let e = endPt else { return nil }
 return CGRect(x: min(s.x, e.x), y: min(s.y, e.y), width: abs(e.x - s.x), height: abs(e.y - s.y))
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
