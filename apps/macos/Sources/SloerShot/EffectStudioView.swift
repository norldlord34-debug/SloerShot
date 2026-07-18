import SwiftUI
import AppKit

struct FxSpec {
 let key: String
 let display: String
 var p1 = ""; var p1min = 0.0; var p1max = 1.0; var p1def = 0.0
 var p2 = ""; var p2min = 0.0; var p2max = 1.0; var p2def = 0.0
 var p3 = ""; var p3min = 0.0; var p3max = 1.0; var p3def = 0.0
 var extra = ""
 private func f(_ d: Double) -> String { String(format: "%.3f", d) }
 func opJson(_ a: Double, _ b: Double, _ c: Double) -> String {
 var s = "{\"op\":\"" + key + "\""
 if !p1.isEmpty { s += ",\"" + p1 + "\":" + f(a) }
 if !p2.isEmpty { s += ",\"" + p2 + "\":" + f(b) }
 if !p3.isEmpty { s += ",\"" + p3 + "\":" + f(c) }
 s += extra + "}"
 return s
 }
}

struct FxPreset: Codable, Identifiable {
 var id = UUID()
 var name: String
 var key: String
 var v1: Double; var v2: Double; var v3: Double
}

struct EffectStudioView: View {
 @ObservedObject var model: EditorModel
 @Environment(\.dismiss) private var dismiss
 @State private var selected = 0
 @State private var v1 = 0.0
 @State private var v2 = 0.0
 @State private var v3 = 0.0
 @State private var preview: CGImage?
 @State private var presetName = ""
 @State private var presets: [FxPreset] = []

 private let specs: [FxSpec] = EffectStudioView.allSpecs()

 var body: some View {
 VStack(spacing: 12) {
 HStack(alignment: .top, spacing: 16) {
 VStack(alignment: .leading, spacing: 8) {
 Picker("Effect", selection: $selected) {
 ForEach(specs.indices, id: \.self) { i in Text(specs[i].display).tag(i) }
 }
 .onChange(of: selected) { _, _ in applyDefaults(); render() }
 paramSlider(specs[selected].p1, $v1, specs[selected].p1min, specs[selected].p1max)
 paramSlider(specs[selected].p2, $v2, specs[selected].p2min, specs[selected].p2max)
 paramSlider(specs[selected].p3, $v3, specs[selected].p3min, specs[selected].p3max)
 Divider()
 HStack { TextField("Preset name", text: $presetName); Button("Save") { savePreset() } }
 presetPicker
 }
 .frame(width: 260)
 previewBox
 }
 HStack {
 Spacer()
 Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
 Button("Apply") { _ = model.applyFx(specs[selected].opJson(v1, v2, v3)); dismiss() }.keyboardShortcut(.defaultAction)
 }
 }
 .padding()
 .frame(width: 640, height: 340)
 .onAppear { presets = EffectStudioView.loadPresets(); applyDefaults(); render() }
 }

 @ViewBuilder private func paramSlider(_ label: String, _ value: Binding<Double>, _ lo: Double, _ hi: Double) -> some View {
 if !label.isEmpty {
 Text(label).font(.caption)
 Slider(value: value, in: lo...hi).onChange(of: value.wrappedValue) { _, _ in render() }
 }
 }
 @ViewBuilder private var previewBox: some View {
 Group {
 if let p = preview {
 Image(nsImage: NSImage(cgImage: p, size: NSSize(width: p.width, height: p.height))).resizable().scaledToFit()
 } else {
 Color.gray.opacity(0.15)
 }
 }
 .frame(width: 320, height: 260)
 }
 @ViewBuilder private var presetPicker: some View {
 if !presets.isEmpty {
 Picker("Load preset", selection: Binding(get: { -1 }, set: { loadPreset($0) })) {
 Text("Choose...").tag(-1)
 ForEach(presets.indices, id: \.self) { i in Text(presets[i].name).tag(i) }
 }
 }
 }
 private func applyDefaults() {
 let sp = specs[selected]
 v1 = sp.p1def; v2 = sp.p2def; v3 = sp.p3def
 }
 private func render() { preview = model.renderFx(specs[selected].opJson(v1, v2, v3)) }
 private func savePreset() {
 let name = presetName.isEmpty ? specs[selected].display : presetName
 presets.removeAll { $0.name == name }
 presets.append(FxPreset(name: name, key: specs[selected].key, v1: v1, v2: v2, v3: v3))
 if let data = try? JSONEncoder().encode(presets) { UserDefaults.standard.set(data, forKey: "ss.fxPresets") }
 presetName = ""
 }
 private func loadPreset(_ i: Int) {
 guard i >= 0, i < presets.count else { return }
 let p = presets[i]
 if let idx = specs.firstIndex(where: { $0.key == p.key }) { selected = idx; v1 = p.v1; v2 = p.v2; v3 = p.v3; render() }
 }
 static func loadPresets() -> [FxPreset] {
 guard let data = UserDefaults.standard.data(forKey: "ss.fxPresets"), let arr = try? JSONDecoder().decode([FxPreset].self, from: data) else { return [] }
 return arr
 }
 static func allSpecs() -> [FxSpec] {
 return [
 FxSpec(key: "blur", display: "Blur", p1: "sigma", p1min: 0, p1max: 20, p1def: 4),
 FxSpec(key: "pixelate", display: "Pixelate", p1: "block", p1min: 2, p1max: 64, p1def: 12),
 FxSpec(key: "gamma", display: "Gamma", p1: "gamma", p1min: 0.2, p1max: 3, p1def: 1),
 FxSpec(key: "hue", display: "Hue rotate", p1: "degrees", p1min: 0, p1max: 360, p1def: 90),
 FxSpec(key: "saturation", display: "Saturation", p1: "factor", p1min: 0, p1max: 3, p1def: 1.5),
 FxSpec(key: "brightness", display: "Brightness", p1: "delta", p1min: -100, p1max: 100, p1def: 25),
 FxSpec(key: "contrast", display: "Contrast", p1: "factor", p1min: 0, p1max: 3, p1def: 1.3),
 FxSpec(key: "vignette", display: "Vignette", p1: "strength", p1min: 0, p1max: 1, p1def: 0.6),
 FxSpec(key: "posterize", display: "Posterize", p1: "levels", p1min: 2, p1max: 16, p1def: 4),
 FxSpec(key: "glow", display: "Glow", p1: "sigma", p1min: 0, p1max: 20, p1def: 6, p2: "intensity", p2min: 0, p2max: 1, p2def: 0.6),
 FxSpec(key: "rgb_split", display: "RGB split", p1: "offset", p1min: 0, p1max: 20, p1def: 3),
 FxSpec(key: "reflection", display: "Reflection", p1: "frac", p1min: 0.1, p1max: 1, p1def: 0.4, p2: "opacity", p2min: 0, p2max: 1, p2def: 0.5),
 FxSpec(key: "polaroid", display: "Polaroid", p1: "border", p1min: 4, p1max: 40, p1def: 16, p2: "bottom", p2min: 8, p2max: 100, p2def: 56),
 FxSpec(key: "outline", display: "Outline", p1: "thickness", p1min: 1, p1max: 8, p1def: 2, extra: ",\"color\":{\"r\":255,\"g\":80,\"b\":0}"),
 FxSpec(key: "shadow", display: "Drop shadow", p1: "dx", p1min: -30, p1max: 30, p1def: 10, p2: "dy", p2min: -30, p2max: 30, p2def: 10, p3: "sigma", p3min: 0, p3max: 20, p3def: 8, extra: ",\"color\":{\"r\":0,\"g\":0,\"b\":0}"),
 ]
 }
}
