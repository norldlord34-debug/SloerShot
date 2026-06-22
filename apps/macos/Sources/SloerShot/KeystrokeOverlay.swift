import AppKit

// Visualizes the users own keystrokes as a transient on-screen badge during a recording
// (CleanShot "Show keystrokes"; cf. the FOSS KeyCastr). Opt-in, ephemeral: shows the keys
// being pressed so they appear in the recording. Nothing is logged, stored, or transmitted.
@MainActor
final class KeystrokeOverlay {
static let shared = KeystrokeOverlay()
private var window: NSWindow?
private var monitor: Any?
private let label = NSTextField(labelWithString: "")
private var clearTask: DispatchWorkItem?

func start() {
guard window == nil, let scr = NSScreen.main else { return }
let w: CGFloat = 460, h: CGFloat = 58
let win = NSWindow(contentRect: NSRect(x: scr.frame.midX - w / 2, y: scr.frame.minY + 90, width: w, height: h), styleMask: [.borderless], backing: .buffered, defer: false)
win.isOpaque = false
win.backgroundColor = .clear
win.level = .screenSaver
win.ignoresMouseEvents = true
win.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
let bg = NSView(frame: NSRect(x: 0, y: 0, width: w, height: h))
bg.wantsLayer = true
bg.layer?.backgroundColor = NSColor(calibratedWhite: 0.08, alpha: 0.85).cgColor
bg.layer?.cornerRadius = 12
label.frame = bg.bounds.insetBy(dx: 16, dy: 0)
label.font = NSFont.systemFont(ofSize: 24, weight: .semibold)
label.textColor = .white
label.alignment = .center
label.lineBreakMode = .byTruncatingHead
label.autoresizingMask = [.width, .height]
bg.addSubview(label)
win.contentView = bg
win.alphaValue = 0
window = win
monitor = NSEvent.addGlobalMonitorForEvents(matching: [.keyDown]) { [weak self] event in
let s = KeystrokeOverlay.describe(event)
guard let self, !s.isEmpty else { return }
Task { @MainActor in self.push(s) }
}
}
func stop() {
if let m = monitor { NSEvent.removeMonitor(m); monitor = nil }
clearTask?.cancel(); clearTask = nil
window?.orderOut(nil); window = nil
label.stringValue = ""
}
private func push(_ s: String) {
guard let win = window else { return }
let cur = label.stringValue.count > 36 ? String(label.stringValue.suffix(28)) : label.stringValue
label.stringValue = cur + s + " "
win.orderFrontRegardless()
win.animator().alphaValue = 1
clearTask?.cancel()
let task = DispatchWorkItem { [weak self] in self?.label.stringValue = ""; self?.window?.animator().alphaValue = 0 }
clearTask = task
DispatchQueue.main.asyncAfter(deadline: .now() + 1.6, execute: task)
}
static func describe(_ e: NSEvent) -> String {
var mods = ""
let f = e.modifierFlags
if f.contains(.command) { mods += "\u{2318}" }
if f.contains(.option) { mods += "\u{2325}" }
if f.contains(.control) { mods += "\u{2303}" }
if f.contains(.shift) { mods += "\u{21E7}" }
let chars = e.charactersIgnoringModifiers ?? ""
let key: String
switch e.keyCode {
case 36: key = "\u{21A9}"
case 48: key = "\u{21E5}"
case 49: key = "space"
case 51: key = "\u{232B}"
case 53: key = "esc"
case 123: key = "\u{2190}"
case 124: key = "\u{2192}"
case 125: key = "\u{2193}"
case 126: key = "\u{2191}"
default: key = chars.isEmpty ? "" : chars.uppercased()
}
return mods + key
}
}
