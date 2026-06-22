import SwiftUI

// CleanShot-style color and style popover for the editor. Drives the core shape style
// (stroke, fill, opacity, width, arrow style, filled, text style) live via set_style_json.
struct StylePopover: View {
@ObservedObject var model: EditorModel
private let swatches: [Color] = [.red, .orange, .yellow, .green, .blue, .purple, .pink, .black, .white, Color(white: 0.5), Color(red: 0.0, green: 0.48, blue: 1.0), Color(red: 0.13, green: 0.77, blue: 0.37)]
private let arrowStyles = ["Straight", "Curved", "DoubleHeaded", "Thin"]
private let textStyles = ["Plain", "Outline", "Filled", "Rounded", "Bubble", "Highlight", "Shadow"]
var body: some View {
VStack(alignment: .leading, spacing: 10) {
Text("Color").font(.caption).foregroundStyle(.secondary)
LazyVGrid(columns: Array(repeating: GridItem(.fixed(26), spacing: 6), count: 6), spacing: 6) {
ForEach(0..<swatches.count, id: \.self) { i in
Circle().fill(swatches[i]).frame(width: 24, height: 24).overlay(Circle().stroke(Color.primary.opacity(0.3), lineWidth: 1)).onTapGesture { model.strokeColor = swatches[i]; model.applyStyle() }
}
}
ColorPicker("Custom color", selection: $model.strokeColor).onChange(of: model.strokeColor) { model.applyStyle() }
HStack { Text("Opacity").font(.caption).frame(width: 56, alignment: .leading); Slider(value: $model.styleOpacity, in: 0.1...1.0) { e in if !e { model.applyStyle() } } }
HStack { Text("Width").font(.caption).frame(width: 56, alignment: .leading); Slider(value: $model.strokeWidth, in: 1...40) { e in if !e { model.applyStyle() } }; Text("\(Int(model.strokeWidth))").font(.caption).foregroundStyle(.secondary) }
Divider()
Toggle("Fill shape", isOn: $model.fillEnabled).onChange(of: model.fillEnabled) { model.applyStyle() }
if model.fillEnabled { ColorPicker("Fill color", selection: $model.fillColor).onChange(of: model.fillColor) { model.applyStyle() } }
Toggle("Filled rectangle/ellipse", isOn: $model.filledShapes).onChange(of: model.filledShapes) { model.applyStyle() }
Divider()
Picker("Arrow", selection: $model.arrowStyle) { ForEach(arrowStyles, id: \.self) { Text($0).tag($0) } }.onChange(of: model.arrowStyle) { model.applyStyle() }
Picker("Text style", selection: $model.textStyle) { ForEach(textStyles, id: \.self) { Text($0).tag($0) } }.onChange(of: model.textStyle) { model.applyStyle() }
Toggle("Smart highlighter (snap to words)", isOn: $model.smartHighlighter).onChange(of: model.smartHighlighter) { model.applyStyle() }
Toggle("Smooth pencil", isOn: $model.pencilSmooth).onChange(of: model.pencilSmooth) { model.applyStyle() }
}
.padding(14).frame(width: 280)
}
}
