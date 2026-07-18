import SwiftUI

struct WorkflowsView: View {
 @ObservedObject private var store = WorkflowStore.shared
 @State private var draft = MacWorkflow()
 var body: some View {
 Form {
 Section("Workflows") {
 if store.workflows.isEmpty { Text("No workflows yet. Edit below and Add as new.").foregroundStyle(.secondary) }
 ForEach(store.workflows) { w in
 HStack {
 Text(w.name)
 Text(hotkeyLabel(w)).font(.caption).foregroundStyle(.secondary)
 Spacer()
 Button("Load") { draft = w }
 Button("Remove") { store.remove(w) }
 }
 }
 }
 Section("Edit workflow") {
 TextField("Name", text: $draft.name)
 Picker("Mode", selection: $draft.mode) { Text("Area").tag("area"); Text("Window").tag("window"); Text("Fullscreen").tag("full"); Text("Record").tag("record") }
 HStack { Toggle("Cmd", isOn: $draft.useCmd); Toggle("Shift", isOn: $draft.useShift); Toggle("Opt", isOn: $draft.useOpt); Toggle("Ctrl", isOn: $draft.useCtrl) }
 Picker("Key", selection: $draft.key) { ForEach(GlobalHotkeys.keyNames, id: \.self) { k in Text(k).tag(k) } }
 Toggle("Auto-upload after capture", isOn: $draft.autoUpload)
 HStack { Button("Add as new") { addNew() }; Button("Update selected") { store.update(draft) } }
 }
 Text("Global hotkeys work system-wide (Carbon). Pick a modifier + key unlikely to clash.").font(.caption).foregroundStyle(.secondary)
 }
 .formStyle(.grouped)
 .padding()
 .frame(width: 520, height: 470)
 }
 private func addNew() { var w = draft; w.id = UUID(); store.add(w) }
 private func hotkeyLabel(_ w: MacWorkflow) -> String {
 var parts: [String] = []
 if w.useCtrl { parts.append("Ctrl") }
 if w.useOpt { parts.append("Opt") }
 if w.useShift { parts.append("Shift") }
 if w.useCmd { parts.append("Cmd") }
 parts.append(w.key)
 return "[" + w.mode + "] " + parts.joined(separator: "+")
 }
}
