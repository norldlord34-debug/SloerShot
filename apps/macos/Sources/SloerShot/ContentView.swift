import SwiftUI

/// Shows the active editor, or a welcome placeholder until something is captured.
struct EditorHost: View {
 @ObservedObject var model: AppModel

 var body: some View {
 Group {
 if let editor = model.editor {
 EditorView(model: editor)
 } else {
 VStack(spacing: 12) {
 Image(systemName: "camera.viewfinder")
 .font(.system(size: 48))
 .foregroundStyle(.secondary)
 Text("SloerShot").font(.largeTitle).bold()
 Text("shotcore \(ShotCore.version())").foregroundStyle(.secondary)
 Text("Capture from the menu bar (Cmd-Shift-4 area, Cmd-Shift-9 fullscreen), then annotate here.")
 .multilineTextAlignment(.center)
 .frame(maxWidth: 360)
 if let e = model.lastError {
 Text(e).foregroundStyle(.red).font(.callout)
 }
 }
 .padding(40)
 .frame(minWidth: 520, minHeight: 360)
 }
 }
 }
}
