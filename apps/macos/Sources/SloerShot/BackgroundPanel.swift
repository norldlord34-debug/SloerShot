import AppKit
import SwiftUI

// CleanShot-style Background tool side panel. Drives the tested core beautify pass with a live, non-destructive preview.
private struct SSGradPreset: Identifiable { let id: String; let a: Color; let b: Color }

struct BackgroundPanel: View {
@ObservedObject var model: EditorModel
@State private var source: CGImage?
@State private var bgType = 0
@State private var preset = "Indigo"
@State private var useCustomGradient = false
@State private var gradStart = Color(red: 0.39, green: 0.40, blue: 0.95)
@State private var gradEnd = Color(red: 0.66, green: 0.33, blue: 0.97)
@State private var angle = 135.0
@State private var solid = Color(white: 1.0)
@State private var padding = 64.0
@State private var corner = 16.0
@State private var shadowOn = true
@State private var shadowOpacity = 0.35
@State private var shadowBlur = 24.0
private let presets: [SSGradPreset] = [
SSGradPreset(id: "Indigo", a: Color(red: 0.39, green: 0.40, blue: 0.95), b: Color(red: 0.66, green: 0.33, blue: 0.97)),
SSGradPreset(id: "Sunset", a: Color(red: 0.98, green: 0.45, blue: 0.09), b: Color(red: 0.93, green: 0.28, blue: 0.60)),
SSGradPreset(id: "Ocean", a: Color(red: 0.05, green: 0.65, blue: 0.91), b: Color(red: 0.13, green: 0.83, blue: 0.93)),
SSGradPreset(id: "Forest", a: Color(red: 0.13, green: 0.77, blue: 0.37), b: Color(red: 0.06, green: 0.73, blue: 0.51)),
SSGradPreset(id: "Candy", a: Color(red: 0.93, green: 0.28, blue: 0.60), b: Color(red: 0.98, green: 0.75, blue: 0.14)),
SSGradPreset(id: "Midnight", a: Color(red: 0.06, green: 0.09, blue: 0.16), b: Color(red: 0.20, green: 0.25, blue: 0.33)),
SSGradPreset(id: "Graphite", a: Color(red: 0.15, green: 0.15, blue: 0.15), b: Color(red: 0.32, green: 0.32, blue: 0.32)),
]
private let colorPresets: [Color] = [Color(white: 1.0), Color(white: 0.0), Color(white: 0.95), Color(white: 0.2), Color(red: 0.92, green: 0.94, blue: 0.98), Color(red: 0.99, green: 0.95, blue: 0.90), Color(red: 0.24, green: 0.49, blue: 1.0), Color(red: 0.10, green: 0.10, blue: 0.12)]

var body: some View {
ScrollView {
VStack(alignment: .leading, spacing: 14) {
HStack { Text("Background").font(.headline); Spacer(); Button { close() } label: { Image(systemName: "xmark.circle.fill") }.buttonStyle(.borderless).help("Close") }
Picker("", selection: $bgType) { Text("Gradient").tag(0); Text("Color").tag(1); Text("None").tag(2) }.pickerStyle(.segmented).onChange(of: bgType) { regenerate() }
if bgType == 0 { gradientSection } else if bgType == 1 { colorSection } else { Text("Transparent background, no fill.").font(.caption).foregroundStyle(.secondary) }
Divider()
sliderRow("Padding", value: $padding, range: 0...260)
sliderRow("Corners", value: $corner, range: 0...64)
Toggle("Shadow", isOn: $shadowOn).onChange(of: shadowOn) { regenerate() }
if shadowOn {
sliderRow("Shadow blur", value: $shadowBlur, range: 0...80)
sliderRow("Shadow opacity", value: Binding(get: { shadowOpacity * 100 }, set: { shadowOpacity = $0 / 100 }), range: 0...100)
}
Divider()
HStack { Button("Reset") { resetControls() }; Spacer(); Button("Apply Background") { commit() }.buttonStyle(.borderedProminent) }
}
.padding(14)
}
.onAppear { source = model.flattenedImage(); regenerate() }
.onDisappear { model.backgroundPreview = nil }
}

private var gradientSection: some View {
VStack(alignment: .leading, spacing: 8) {
LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 4), spacing: 8) {
ForEach(presets) { p in
RoundedRectangle(cornerRadius: 8).fill(LinearGradient(colors: [p.a, p.b], startPoint: .topLeading, endPoint: .bottomTrailing)).frame(height: 40).overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.accentColor, lineWidth: (!useCustomGradient && preset == p.id) ? 3 : 0)).onTapGesture { preset = p.id; useCustomGradient = false; regenerate() }
}
}
Toggle("Custom gradient", isOn: $useCustomGradient).onChange(of: useCustomGradient) { regenerate() }
if useCustomGradient {
HStack { ColorPicker("From", selection: $gradStart).onChange(of: gradStart) { regenerate() }; ColorPicker("To", selection: $gradEnd).onChange(of: gradEnd) { regenerate() } }
sliderRow("Angle", value: $angle, range: 0...360)
}
}
}

private var colorSection: some View {
VStack(alignment: .leading, spacing: 8) {
LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 4), spacing: 8) {
ForEach(0..<colorPresets.count, id: \.self) { i in
RoundedRectangle(cornerRadius: 8).fill(colorPresets[i]).frame(height: 40).overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.gray.opacity(0.4), lineWidth: 1)).onTapGesture { solid = colorPresets[i]; regenerate() }
}
}
ColorPicker("Custom color", selection: $solid).onChange(of: solid) { regenerate() }
}
}

private func sliderRow(_ title: String, value: Binding<Double>, range: ClosedRange<Double>) -> some View {
VStack(alignment: .leading, spacing: 2) {
HStack { Text(title).font(.caption); Spacer(); Text("\(Int(value.wrappedValue))").font(.caption).foregroundStyle(.secondary) }
Slider(value: value, in: range) { editing in if !editing { regenerate() } }
}
}

private func regenerate() {
guard let src = source else { return }
model.backgroundPreview = model.beautifyPreview(source: src, json: optionsJSON())
}
private func commit() { model.commitBackground(json: optionsJSON()); close() }
private func close() { model.backgroundPreview = nil; model.showBackgroundPanel = false }
private func resetControls() { bgType = 0; preset = "Indigo"; useCustomGradient = false; padding = 64; corner = 16; shadowOn = true; shadowOpacity = 0.35; shadowBlur = 24; angle = 135; regenerate() }

private func rgba(_ c: Color) -> (Int, Int, Int, Int) {
let n = NSColor(c).usingColorSpace(.sRGB) ?? NSColor.black
return (Int((n.redComponent * 255).rounded()), Int((n.greenComponent * 255).rounded()), Int((n.blueComponent * 255).rounded()), Int((n.alphaComponent * 255).rounded()))
}
private func colorJSON(_ c: Color, alpha: Int? = nil) -> String {
let (r, g, b, a) = rgba(c)
return "{\"r\":\(r),\"g\":\(g),\"b\":\(b),\"a\":\(alpha ?? a)}"
}
private func backgroundJSON() -> String {
switch bgType {
case 1: return "{\"Solid\":\(colorJSON(solid))}"
case 2: return "{\"Solid\":{\"r\":0,\"g\":0,\"b\":0,\"a\":0}}"
default:
if useCustomGradient { return "{\"Gradient\":{\"start\":\(colorJSON(gradStart)),\"end\":\(colorJSON(gradEnd)),\"angle_deg\":\(angle)}}" }
return "{\"Preset\":\"\(preset)\"}"
}
}
private func optionsJSON() -> String {
let shadow = shadowOn ? "{\"color\":{\"r\":0,\"g\":0,\"b\":0,\"a\":255},\"blur\":\(shadowBlur),\"dx\":0.0,\"dy\":16.0,\"opacity\":\(shadowOpacity)}" : "null"
return "{\"background\":\(backgroundJSON()),\"padding\":\(Int(padding)),\"corner_radius\":\(corner),\"shadow\":\(shadow)}"
}
}
