import SwiftUI

// Interactive crop overlay over the editor canvas (same image-pixel coordinate space).
// Drag to define the crop region; shows a dimmed mask + rule-of-thirds guides.
struct CropOverlay: View {
let imageSize: CGSize
@Binding var rect: CGRect
@State private var origin: CGPoint?
var body: some View {
ZStack {
Path { p in
p.addRect(CGRect(origin: .zero, size: imageSize))
if isValid { p.addRect(rect) }
}
.fill(Color.black.opacity(0.45), style: FillStyle(eoFill: true))
if isValid {
Path { p in p.addRect(rect) }.stroke(Color.white, lineWidth: 1.5)
Path { p in
let x1 = rect.minX + rect.width / 3, x2 = rect.minX + rect.width * 2 / 3
let y1 = rect.minY + rect.height / 3, y2 = rect.minY + rect.height * 2 / 3
p.move(to: CGPoint(x: x1, y: rect.minY)); p.addLine(to: CGPoint(x: x1, y: rect.maxY))
p.move(to: CGPoint(x: x2, y: rect.minY)); p.addLine(to: CGPoint(x: x2, y: rect.maxY))
p.move(to: CGPoint(x: rect.minX, y: y1)); p.addLine(to: CGPoint(x: rect.maxX, y: y1))
p.move(to: CGPoint(x: rect.minX, y: y2)); p.addLine(to: CGPoint(x: rect.maxX, y: y2))
}.stroke(Color.white.opacity(0.6), lineWidth: 0.75)
}
}
.frame(width: imageSize.width, height: imageSize.height)
.contentShape(Rectangle())
.gesture(
DragGesture(minimumDistance: 0)
.onChanged { v in
let s = origin ?? v.startLocation
origin = s
let x = min(s.x, v.location.x), y = min(s.y, v.location.y)
let w = abs(v.location.x - s.x), h = abs(v.location.y - s.y)
rect = CGRect(x: x, y: y, width: w, height: h).intersection(CGRect(origin: .zero, size: imageSize))
}
.onEnded { _ in origin = nil }
)
}
private var isValid: Bool { !rect.isNull && rect.width > 1 && rect.height > 1 }
}
