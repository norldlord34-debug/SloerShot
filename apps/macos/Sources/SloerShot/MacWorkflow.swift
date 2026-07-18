import Foundation
import Combine
import Carbon.HIToolbox

struct MacWorkflow: Codable, Identifiable {
 var id = UUID()
 var name = "Workflow"
 var mode = "area"
 var key = "4"
 var useCmd = true
 var useShift = true
 var useOpt = false
 var useCtrl = false
 var autoUpload = false
}

final class WorkflowStore: ObservableObject {
 static let shared = WorkflowStore()
 @Published var workflows: [MacWorkflow] = []
 private weak var model: AppModel?

 init() { load() }

 func load() {
 if let data = UserDefaults.standard.data(forKey: "ss.workflows"), let arr = try? JSONDecoder().decode([MacWorkflow].self, from: data) { workflows = arr }
 }
 func save() {
 if let data = try? JSONEncoder().encode(workflows) { UserDefaults.standard.set(data, forKey: "ss.workflows") }
 registerAll()
 }
 func attach(_ m: AppModel) { model = m; registerAll() }
 func add(_ w: MacWorkflow) { workflows.append(w); save() }
 func remove(_ w: MacWorkflow) { workflows.removeAll { $0.id == w.id }; save() }
 func update(_ w: MacWorkflow) { if let i = workflows.firstIndex(where: { $0.id == w.id }) { workflows[i] = w; save() } }

 func registerAll() {
 GlobalHotkeys.shared.unregisterAll()
 guard let model = model else { return }
 for w in workflows {
 guard let code = GlobalHotkeys.keyMap[w.key] else { continue }
 var mods: UInt32 = 0
 if w.useCmd { mods |= UInt32(cmdKey) }
 if w.useShift { mods |= UInt32(shiftKey) }
 if w.useOpt { mods |= UInt32(optionKey) }
 if w.useCtrl { mods |= UInt32(controlKey) }
 let mode = w.mode
 let up = w.autoUpload
 GlobalHotkeys.shared.register(keyCode: code, modifiers: mods) { [weak model] in Task { @MainActor in model?.runWorkflowMode(mode, autoUpload: up) } }
 }
 }
}
