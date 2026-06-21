import CoreGraphics
import ScreenCaptureKit

/// Screen capture via ScreenCaptureKit (macOS 14+). Area capture works on a frozen
/// still: capture the whole display once, let the overlay select, then crop - so menus
/// and hover states stay put while selecting.
@available(macOS 14.0, *)
enum Capture {
 enum CaptureError: Error { case noDisplay }

 static func shareableContent() async throws -> SCShareableContent {
 try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
 }

 static func primaryDisplay() async throws -> SCDisplay {
 let content = try await shareableContent()
 guard let display = content.displays.first else { throw CaptureError.noDisplay }
 return display
 }

 /// Full-resolution still of a display: the frozen frame the selection overlay uses.
 static func captureDisplay(_ display: SCDisplay) async throws -> CGImage {
 let filter = SCContentFilter(display: display, excludingWindows: [])
 let config = SCStreamConfiguration()
 config.width = display.width
 config.height = display.height
 config.showsCursor = false
 return try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config)
 }

 /// Capture a single window independent of what is in front of it.
 static func captureWindow(_ window: SCWindow) async throws -> CGImage {
 let filter = SCContentFilter(desktopIndependentWindow: window)
 let config = SCStreamConfiguration()
 config.width = Int(window.frame.width)
 config.height = Int(window.frame.height)
 config.showsCursor = false
 return try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config)
 }

 /// Crop a frozen capture to a selection rect in image pixel space.
 static func crop(_ image: CGImage, to rect: CGRect) -> CGImage? {
 image.cropping(to: rect.integral)
 }

 /// Capture the whole primary display (fullscreen mode).
 static func captureFullscreen() async throws -> CGImage {
 try await captureDisplay(try await primaryDisplay())
 }

 /// Capture the primary display, then crop to `selection` (area mode).
 static func captureArea(_ selection: CGRect) async throws -> CGImage? {
 let full = try await captureFullscreen()
 return crop(full, to: selection)
 }
}
