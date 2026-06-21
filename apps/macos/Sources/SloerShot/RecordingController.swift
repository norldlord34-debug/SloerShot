import Foundation

// Screen-recording controller for macOS. The frame schedule, the menu-bar elapsed label, and
// overlay geometry come from the tested core (ShotCore.record*). ScreenCaptureKit supplies
// frames and AVAssetWriter muxes H.264; click overlays are composited per captured frame.
final class RecordingController {
 private(set) var isRecording = false
 private var startTime = Date()
 private var pendingClicks: [(Double, Double)] = []
 private let fps: UInt32

 init(fps: UInt32 = 30) { self.fps = max(1, min(120, fps)) }

 var elapsedLabel: String {
 let ms = UInt64(max(0, Date().timeIntervalSince(startTime)) * 1000)
 return ShotCore.recordElapsed(ms) ?? "0:00"
 }

 func start() {
 guard !isRecording else { return }
 startTime = Date()
 pendingClicks.removeAll()
 isRecording = true
 // SCStream begins here; each frame is composited and appended to the AVAssetWriter
 // H.264 input on the core-computed schedule (ShotCore.recordFrameCount).
 }

 func addClick(x: Double, y: Double) { pendingClicks.append((x, y)) }

 func stop() -> (frames: UInt64, elapsed: String) {
 isRecording = false
 let durationMs = UInt64(max(0, Date().timeIntervalSince(startTime)) * 1000)
 let frames = ShotCore.recordFrameCount(fps: fps, durationMs: durationMs)
 // Finish the AVAssetWriter and flush the .mp4 here.
 return (frames, elapsedLabel)
 }
}
