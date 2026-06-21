import Foundation
import Vision
import CoreGraphics

// Real on-device OCR via Apple Vision. Produces the OcrResult JSON that shotcore consumes,
// so all region / reading-order / clipboard formatting runs in the tested core
// (ShotCore.ocrTextInRegion / ocrSingleLine). Vision only supplies recognized lines + boxes.
enum OcrService {
 static func recognizeJSON(cgImage: CGImage, width: Int, height: Int) -> String? {
 let request = VNRecognizeTextRequest()
 request.recognitionLevel = .accurate
 let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
 do {
 try handler.perform([request])
 } catch {
 return nil
 }
 guard let observations = request.results else { return nil }
 let w = Double(width)
 let h = Double(height)
 var lines: [[String: Any]] = []
 for obs in observations {
 guard let candidate = obs.topCandidates(1).first else { continue }
 // Vision boxes are normalized, origin bottom-left; convert to top-left pixels.
 let bb = obs.boundingBox
 let bbox: [String: Any] = ["x": bb.minX * w, "y": (1.0 - bb.maxY) * h, "w": bb.width * w, "h": bb.height * h]
 let word: [String: Any] = ["text": candidate.string, "bbox": bbox, "confidence": Double(candidate.confidence)]
 lines.append(["text": candidate.string, "bbox": bbox, "words": [word]])
 }
 guard let data = try? JSONSerialization.data(withJSONObject: ["lines": lines]) else { return nil }
 return String(data: data, encoding: .utf8)
 }
}
