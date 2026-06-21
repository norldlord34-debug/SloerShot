import SwiftUI

extension ColorSpec {
 /// SwiftUI color with the given extra opacity multiplier folded in.
 func color(_ mult: Double = 1.0) -> Color {
 Color(.sRGB, red: Double(r) / 255, green: Double(g) / 255, blue: Double(b) / 255,
 opacity: Double(a) / 255 * mult)
 }
}

/// Renders parsed shape specs over the captured image and forwards pointer drags to
/// the shared core editor. Coordinates are in image pixel space (the canvas is sized
/// to the image, so view points map 1:1).
struct AnnotationCanvas: View {
 let background: CGImage?
 let specs: [ShapeSpec]
 let imageSize: CGSize
 var onPointerDown: (CGPoint) -> Void = { _ in }
 var onPointerDrag: (CGPoint) -> Void = { _ in }
 var onPointerUp: (CGPoint) -> Void = { _ in }

 @State private var dragging = false

 var body: some View {
 Canvas { ctx, _ in
 if let bg = background {
 let img = Image(decorative: bg, scale: 1, orientation: .up)
 ctx.draw(img, in: CGRect(origin: .zero, size: imageSize))
 }
 for s in specs {
 draw(s, &ctx)
 }
 }
 .frame(width: imageSize.width, height: imageSize.height)
 .contentShape(Rectangle())
 .gesture(
 DragGesture(minimumDistance: 0)
 .onChanged { v in
 if dragging {
 onPointerDrag(v.location)
 } else {
 dragging = true
 onPointerDown(v.location)
 }
 }
 .onEnded { v in
 dragging = false
 onPointerUp(v.location)
 }
 )
 }

 private func draw(_ s: ShapeSpec, _ ctx: inout GraphicsContext) {
 let stroke = s.stroke.color(s.opacity)
 let lw = s.strokeWidth
 switch s.type {
 case .rectangle:
 let rect = CGRect(x: s.x, y: s.y, width: s.w, height: s.h)
 let path = Path(roundedRect: rect, cornerRadius: s.cornerRadius)
 if let fill = s.fill { ctx.fill(path, with: .color(fill.color(s.opacity))) }
 ctx.stroke(path, with: .color(stroke), lineWidth: lw)
 case .ellipse:
 let path = Path(ellipseIn: CGRect(x: s.x, y: s.y, width: s.w, height: s.h))
 if let fill = s.fill { ctx.fill(path, with: .color(fill.color(s.opacity))) }
 ctx.stroke(path, with: .color(stroke), lineWidth: lw)
 case .line:
 var path = Path()
 path.move(to: CGPoint(x: s.x1, y: s.y1))
 path.addLine(to: CGPoint(x: s.x2, y: s.y2))
 ctx.stroke(path, with: .color(stroke), lineWidth: lw)
 case .arrow:
 var path = Path()
 path.move(to: CGPoint(x: s.x1, y: s.y1))
 path.addLine(to: CGPoint(x: s.x2, y: s.y2))
 ctx.stroke(path, with: .color(stroke), lineWidth: lw)
 ctx.fill(arrowHead(from: CGPoint(x: s.x1, y: s.y1), to: CGPoint(x: s.x2, y: s.y2), width: lw), with: .color(stroke))
 case .freehand:
 guard let first = s.points.first else { return }
 var path = Path()
 path.move(to: first)
 for p in s.points.dropFirst() { path.addLine(to: p) }
 ctx.stroke(path, with: .color(stroke), lineWidth: lw)
 case .highlighter:
 let path = Path(CGRect(x: s.x, y: s.y, width: s.w, height: s.h))
 ctx.fill(path, with: .color(s.stroke.color(s.opacity * 0.4)))
 case .redact:
 let path = Path(CGRect(x: s.x, y: s.y, width: s.w, height: s.h))
 ctx.fill(path, with: .color(.black))
 case .text:
 let text = Text(s.text).font(.system(size: s.fontSize)).foregroundColor(stroke)
 ctx.draw(text, at: CGPoint(x: s.x, y: s.y), anchor: .topLeading)
 case .counter:
 let r = s.h
 let circle = Path(ellipseIn: CGRect(x: s.x - r, y: s.y - r, width: r * 2, height: r * 2))
 ctx.fill(circle, with: .color(stroke))
 let label = Text("\(s.number)").font(.system(size: r, weight: .bold)).foregroundColor(.white)
 ctx.draw(label, at: CGPoint(x: s.x, y: s.y), anchor: .center)
 }
 }

 /// Triangle for an arrow head pointing from `from` to `to`.
 private func arrowHead(from: CGPoint, to: CGPoint, width: Double) -> Path {
 let dx = to.x - from.x
 let dy = to.y - from.y
 let len = (dx * dx + dy * dy).squareRoot()
 var path = Path()
 guard len > 0 else { return path }
 let ux = dx / len, uy = dy / len
 let px = -uy, py = ux
 let size = max(width * 3, 8)
 let b1 = CGPoint(x: to.x - ux * size + px * size * 0.5, y: to.y - uy * size + py * size * 0.5)
 let b2 = CGPoint(x: to.x - ux * size - px * size * 0.5, y: to.y - uy * size - py * size * 0.5)
 path.move(to: to)
 path.addLine(to: b1)
 path.addLine(to: b2)
 path.closeSubpath()
 return path
 }
}
