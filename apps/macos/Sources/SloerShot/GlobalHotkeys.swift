import Carbon.HIToolbox
import Foundation

// Global (system-wide) hotkeys via Carbon RegisterEventHotKey. Works without Accessibility permission.
final class GlobalHotkeys {
 static let shared = GlobalHotkeys()
 private var refs: [EventHotKeyRef?] = []
 private var handlers: [UInt32: () -> Void] = [:]
 private var installed = false
 private var nextId: UInt32 = 1

 func unregisterAll() {
 for r in refs { if let r = r { UnregisterEventHotKey(r) } }
 refs.removeAll()
 handlers.removeAll()
 nextId = 1
 }

 @discardableResult
 func register(keyCode: UInt32, modifiers: UInt32, action: @escaping () -> Void) -> Bool {
 installHandlerIfNeeded()
 let id = nextId; nextId += 1
 handlers[id] = action
 var ref: EventHotKeyRef?
 let hotID = EventHotKeyID(signature: OSType(0x53534854), id: id)
 let status = RegisterEventHotKey(keyCode, modifiers, hotID, GetApplicationEventTarget(), 0, &ref)
 if status == noErr { refs.append(ref); return true }
 handlers[id] = nil
 return false
 }

 private func installHandlerIfNeeded() {
 guard !installed else { return }
 installed = true
 var spec = EventTypeSpec(eventClass: OSType(kEventClassKeyboard), eventKind: UInt32(kEventHotKeyPressed))
 InstallEventHandler(GetApplicationEventTarget(), { (_, event, userData) -> OSStatus in
 guard let event = event, let userData = userData else { return noErr }
 var hkID = EventHotKeyID()
 GetEventParameter(event, EventParamName(kEventParamDirectObject), EventParamType(typeEventHotKeyID), nil, MemoryLayout<EventHotKeyID>.size, nil, &hkID)
 let mgr = Unmanaged<GlobalHotkeys>.fromOpaque(userData).takeUnretainedValue()
 let id = hkID.id
 DispatchQueue.main.async { mgr.handlers[id]?() }
 return noErr
 }, 1, &spec, Unmanaged.passUnretained(self).toOpaque(), nil)
 }

 // Carbon virtual key codes for a curated set of keys.
 static let keyMap: [String: UInt32] = [
 "A": 0x00, "B": 0x0B, "C": 0x08, "D": 0x02, "E": 0x0E, "F": 0x03, "G": 0x05, "H": 0x04, "I": 0x22,
 "J": 0x26, "K": 0x28, "L": 0x25, "M": 0x2E, "N": 0x2D, "O": 0x1F, "P": 0x23, "Q": 0x0C, "R": 0x0F,
 "S": 0x01, "T": 0x11, "U": 0x20, "V": 0x09, "W": 0x0D, "X": 0x07, "Y": 0x10, "Z": 0x06,
 "0": 0x1D, "1": 0x12, "2": 0x13, "3": 0x14, "4": 0x15, "5": 0x17, "6": 0x16, "7": 0x1A, "8": 0x1C, "9": 0x19,
 "F1": 0x7A, "F2": 0x78, "F3": 0x63, "F4": 0x76, "F5": 0x60, "F6": 0x61, "F7": 0x62, "F8": 0x64,
 "F9": 0x65, "F10": 0x6D, "F11": 0x67, "F12": 0x6F
 ]
 static let keyNames: [String] = [
 "A","B","C","D","E","F","G","H","I","J","K","L","M","N","O","P","Q","R","S","T","U","V","W","X","Y","Z",
 "0","1","2","3","4","5","6","7","8","9","F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12"
 ]
}
