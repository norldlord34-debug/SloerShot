import AppKit
import AVFoundation

// Webcam picture-in-picture overlay shown during a recording (CleanShot camera overlay).
// A floating, click-through, top-level circular window the recording captures. Size comes
// from the tested core (record_camera_rect). Requires camera permission at runtime.
@MainActor
final class CameraOverlay: NSObject {
static let shared = CameraOverlay()
private var window: NSWindow?
private var session: AVCaptureSession?

func start() {
guard window == nil, let scr = NSScreen.main else { return }
guard let device = AVCaptureDevice.default(for: .video) else { return }
let session = AVCaptureSession()
session.sessionPreset = .high
guard let input = try? AVCaptureDeviceInput(device: device), session.canAddInput(input) else { return }
session.addInput(input)
var side: CGFloat = 200
let camJson = "{\"shape\":\"Circle\",\"position\":\"BottomLeft\",\"size_frac\":0.18,\"fullscreen\":false,\"mirrored\":true}"
if let rj = ShotCore.recordCameraRect(cameraJson: camJson, width: UInt32(scr.frame.width), height: UInt32(scr.frame.height)),
let data = rj.data(using: .utf8),
let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
let w = obj["w"] as? Double { side = CGFloat(w) }
let margin: CGFloat = 28
let origin = NSPoint(x: scr.frame.minX + margin, y: scr.frame.minY + margin)
let win = NSWindow(contentRect: NSRect(origin: origin, size: NSSize(width: side, height: side)), styleMask: [.borderless], backing: .buffered, defer: false)
win.isOpaque = false
win.backgroundColor = .clear
win.level = .screenSaver
win.ignoresMouseEvents = true
win.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
let container = NSView(frame: NSRect(x: 0, y: 0, width: side, height: side))
container.wantsLayer = true
container.layer?.cornerRadius = side / 2
container.layer?.masksToBounds = true
container.layer?.borderWidth = 3
container.layer?.borderColor = NSColor.white.cgColor
let preview = AVCaptureVideoPreviewLayer(session: session)
preview.frame = container.bounds
preview.videoGravity = .resizeAspectFill
if let conn = preview.connection, conn.isVideoMirroringSupported { conn.automaticallyAdjustsVideoMirroring = false; conn.isVideoMirrored = true }
container.layer?.addSublayer(preview)
win.contentView = container
win.orderFrontRegardless()
self.session = session
self.window = win
let s = session
DispatchQueue.global(qos: .userInitiated).async { s.startRunning() }
}
func stop() {
session?.stopRunning()
session = nil
window?.orderOut(nil)
window = nil
}
}
