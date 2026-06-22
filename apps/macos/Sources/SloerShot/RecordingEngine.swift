import AppKit
import AVFoundation
import CoreMedia
import ScreenCaptureKit

// Screen recording: ScreenCaptureKit frames muxed to H.264 MP4 via AVAssetWriter.
final class RecordingEngine: NSObject, SCStreamOutput, SCStreamDelegate {
private var stream: SCStream?
private var writer: AVAssetWriter?
private var videoInput: AVAssetWriterInput?
private let queue = DispatchQueue(label: "sloershot.recording")
private(set) var outputURL: URL?
private(set) var isRecording = false
func start() async throws {
let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
guard let display = content.displays.first else { throw NSError(domain: "SloerShot.Recording", code: 1) }
let filter = SCContentFilter(display: display, excludingWindows: [])
let config = SCStreamConfiguration()
config.width = display.width
config.height = display.height
config.minimumFrameInterval = CMTime(value: 1, timescale: 30)
config.pixelFormat = kCVPixelFormatType_32BGRA
config.showsCursor = true
let movies = FileManager.default.urls(for: .moviesDirectory, in: .userDomainMask).first ?? FileManager.default.temporaryDirectory
let folder = movies.appendingPathComponent("SloerShot")
try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
let url = folder.appendingPathComponent("recording-\(Int(Date().timeIntervalSince1970)).mp4")
outputURL = url
let w = try AVAssetWriter(url: url, fileType: .mp4)
let settings: [String: Any] = [AVVideoCodecKey: AVVideoCodecType.h264, AVVideoWidthKey: display.width, AVVideoHeightKey: display.height]
let input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
input.expectsMediaDataInRealTime = true
if w.canAdd(input) { w.add(input) }
writer = w
videoInput = input
let s = SCStream(filter: filter, configuration: config, delegate: self)
try s.addStreamOutput(self, type: .screen, sampleHandlerQueue: queue)
stream = s
try await s.startCapture()
isRecording = true
}
func stop() async {
isRecording = false
if let s = stream { try? await s.stopCapture() }
stream = nil
videoInput?.markAsFinished()
if let w = writer {
await withCheckedContinuation { (cont: CheckedContinuation<Void, Never>) in
w.finishWriting { cont.resume() }
}
}
writer = nil
videoInput = nil
}
func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
guard type == .screen, sampleBuffer.isValid else { return }
guard let arr = CMSampleBufferGetSampleAttachmentsArray(sampleBuffer, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
let attachments = arr.first,
let statusRaw = attachments[.status] as? Int,
let status = SCFrameStatus(rawValue: statusRaw), status == .complete else { return }
guard let writer = writer, let input = videoInput else { return }
if writer.status == .unknown {
writer.startWriting()
writer.startSession(atSourceTime: CMSampleBufferGetPresentationTimeStamp(sampleBuffer))
}
if writer.status == .writing, input.isReadyForMoreMediaData {
input.append(sampleBuffer)
}
}
func stream(_ stream: SCStream, didStopWithError error: Error) {
isRecording = false
}
}

extension AppModel {
func startRecording() {
guard recorder == nil else { return }
let eng = RecordingEngine()
recorder = eng
Task {
do {
try await eng.start()
isRecording = true
Toast.show("Recording started")
RecordingHUD.shared.show { [weak self] in self?.stopRecording() }
} catch {
lastError = String(describing: error)
recorder = nil
Toast.show("Recording failed (grant Screen Recording permission)")
}
}
}
func stopRecording() {
guard let eng = recorder else { return }
RecordingHUD.shared.hide()
Task {
await eng.stop()
isRecording = false
recorder = nil
if let url = eng.outputURL { Toast.show("Saved " + url.lastPathComponent) }
}
}
}
