import SwiftUI

// Native multi-tab Settings window (CleanShot-style). Persisted via @AppStorage (UserDefaults).
struct SettingsView: View {
@AppStorage("ss.startAtLogin") private var startAtLogin = false
@AppStorage("ss.playSounds") private var playSounds = true
@AppStorage("ss.shutterSound") private var shutterSound = "Classic"
@AppStorage("ss.showMenuBarIcon") private var showMenuBarIcon = true
@AppStorage("ss.exportLocation") private var exportLocation = "Desktop"
@AppStorage("ss.hideDesktopIcons") private var hideDesktopIcons = false
@AppStorage("ss.scShowOverlay") private var scShowOverlay = true
@AppStorage("ss.scCopy") private var scCopy = false
@AppStorage("ss.scSave") private var scSave = true
@AppStorage("ss.scUpload") private var scUpload = false
@AppStorage("ss.scAnnotate") private var scAnnotate = false
@AppStorage("ss.wallpaperMode") private var wallpaperMode = "Desktop"
@AppStorage("ss.windowShot") private var windowShot = "With wallpaper"
@AppStorage("ss.windowPadding") private var windowPadding = 0.5
@AppStorage("ss.captureWindowShadow") private var captureWindowShadow = true
@AppStorage("ss.qaoPosition") private var qaoPosition = "Bottom Left"
@AppStorage("ss.qaoActiveDisplay") private var qaoActiveDisplay = true
@AppStorage("ss.qaoSize") private var qaoSize = 0.5
@AppStorage("ss.qaoAutoClose") private var qaoAutoClose = false
@AppStorage("ss.qaoInterval") private var qaoInterval = 30
@AppStorage("ss.qaoCloseAfterDrag") private var qaoCloseAfterDrag = true
@AppStorage("ss.qaoCloseAfterUpload") private var qaoCloseAfterUpload = true
@AppStorage("ss.recShowControls") private var recShowControls = true
@AppStorage("ss.recMenuBarTime") private var recMenuBarTime = true
@AppStorage("ss.recRetina1x") private var recRetina1x = false
@AppStorage("ss.recDND") private var recDND = false
@AppStorage("ss.recShowCursor") private var recShowCursor = true
@AppStorage("ss.recSystemAudio") private var recSystemAudio = false
@AppStorage("ss.recHighlightClicks") private var recHighlightClicks = true
@AppStorage("ss.recShowKeystrokes") private var recShowKeystrokes = false
@AppStorage("ss.recCamera") private var recCamera = false
@AppStorage("ss.recRememberArea") private var recRememberArea = false
@AppStorage("ss.recFPS") private var recFPS = 60
@AppStorage("ss.recOpenEditor") private var recOpenEditor = false
@AppStorage("ss.gifFPS") private var gifFPS = 15
@AppStorage("ss.gifQuality") private var gifQuality = 0.7
@AppStorage("ss.fileFormat") private var fileFormat = "PNG"
@AppStorage("ss.shotRetina1x") private var shotRetina1x = false
@AppStorage("ss.shotBorder") private var shotBorder = false
@AppStorage("ss.freeze") private var freeze = false
@AppStorage("ss.crosshairMode") private var crosshairMode = "Disabled"
@AppStorage("ss.showMagnifier") private var showMagnifier = false
@AppStorage("ss.selfTimer") private var selfTimer = 5
@AppStorage("ss.scrollFrames") private var scrollFrames = 8
@AppStorage("ss.showCursor") private var showCursor = true
@AppStorage("ss.inverseArrow") private var inverseArrow = false
@AppStorage("ss.smoothPencil") private var smoothPencil = true
@AppStorage("ss.objectShadow") private var objectShadow = true
@AppStorage("ss.autoExpand") private var autoExpand = false
@AppStorage("ss.colorNames") private var colorNames = false
@AppStorage("ss.annotateOnTop") private var annotateOnTop = false
@AppStorage("ss.dockIcon") private var dockIcon = true
@AppStorage("ss.serverUrl") private var serverUrl = ""
@AppStorage("ss.cloudQuality") private var cloudQuality = "Optimized for sharing"
@AppStorage("ss.cloudCopyType") private var cloudCopyType = "Link"
@AppStorage("ss.fileNameFormat") private var fileNameFormat = "SloerShot %y-%m-%d at %H.%M.%S"
@AppStorage("ss.retina2xSuffix") private var retina2xSuffix = true
@AppStorage("ss.copyMode") private var copyMode = "File & Image"
@AppStorage("ss.pinRounded") private var pinRounded = true
@AppStorage("ss.pinShadow") private var pinShadow = true
@AppStorage("ss.pinBorder") private var pinBorder = true
@AppStorage("ss.ocrKeepLineBreaks") private var ocrKeepLineBreaks = true
var body: some View {
TabView {
generalTab.tabItem { Label("General", systemImage: "gearshape") }
wallpaperTab.tabItem { Label("Wallpaper", systemImage: "photo") }
shortcutsTab.tabItem { Label("Shortcuts", systemImage: "command") }
quickAccessTab.tabItem { Label("Quick Access", systemImage: "rectangle.on.rectangle") }
recordingTab.tabItem { Label("Recording", systemImage: "video") }
screenshotsTab.tabItem { Label("Screenshots", systemImage: "camera") }
annotateTab.tabItem { Label("Annotate", systemImage: "pencil.tip") }
cloudTab.tabItem { Label("Cloud", systemImage: "cloud") }
advancedTab.tabItem { Label("Advanced", systemImage: "wrench.and.screwdriver") }
}
.frame(width: 600, height: 520)
}
private var generalTab: some View {
Form {
Toggle("Start at login", isOn: $startAtLogin)
Toggle("Play sounds", isOn: $playSounds)
Picker("Shutter sound", selection: $shutterSound) { Text("Classic").tag("Classic"); Text("Modern").tag("Modern"); Text("Silent").tag("Silent") }
Toggle("Show menu bar icon", isOn: $showMenuBarIcon)
Picker("Export location", selection: $exportLocation) { Text("Desktop").tag("Desktop"); Text("Pictures").tag("Pictures"); Text("Downloads").tag("Downloads") }
Toggle("Hide desktop icons while capturing", isOn: $hideDesktopIcons)
Section("After screenshot") {
Toggle("Show Quick Access Overlay", isOn: $scShowOverlay)
Toggle("Copy to clipboard", isOn: $scCopy)
Toggle("Save", isOn: $scSave)
Toggle("Upload to Cloud and copy link", isOn: $scUpload)
Toggle("Open Annotate tool", isOn: $scAnnotate)
}
}
.formStyle(.grouped)
}
private var wallpaperTab: some View {
Form {
Picker("Background", selection: $wallpaperMode) { Text("Desktop wallpaper").tag("Desktop"); Text("Custom wallpaper").tag("Custom"); Text("Plain color").tag("Plain") }
Picker("Window screenshot", selection: $windowShot) { Text("With wallpaper").tag("With wallpaper"); Text("Transparent").tag("Transparent") }
VStack(alignment: .leading) { Text("Padding"); Slider(value: $windowPadding) }
Toggle("Capture window shadow", isOn: $captureWindowShadow)
}
.formStyle(.grouped)
}
private var shortcutsTab: some View {
Form {
Section("Screenshots") {
LabeledContent("Capture Area", value: "Cmd Shift 4")
LabeledContent("Capture Fullscreen", value: "Cmd Shift 9")
LabeledContent("Capture Window", value: "Cmd Shift 5")
LabeledContent("Scrolling Capture", value: "Cmd Shift 6")
LabeledContent("Capture Text (OCR)", value: "Cmd Shift T")
LabeledContent("Pick Color", value: "Cmd Shift C")
}
Section("Other") {
LabeledContent("Pin Last Capture", value: "Cmd Shift P")
LabeledContent("Start/Stop Recording", value: "Cmd Shift 2")
}
Text("Rebindable global hotkeys are coming in a later update.").font(.caption).foregroundStyle(.secondary)
}
.formStyle(.grouped)
}
private var quickAccessTab: some View {
Form {
Picker("Position on screen", selection: $qaoPosition) { Text("Bottom Left").tag("Bottom Left"); Text("Bottom Right").tag("Bottom Right"); Text("Top Left").tag("Top Left"); Text("Top Right").tag("Top Right") }
Toggle("Show on active display", isOn: $qaoActiveDisplay)
VStack(alignment: .leading) { Text("Overlay size"); Slider(value: $qaoSize) }
Toggle("Auto-close", isOn: $qaoAutoClose)
Picker("Auto-close interval", selection: $qaoInterval) { Text("10 seconds").tag(10); Text("30 seconds").tag(30); Text("60 seconds").tag(60) }.disabled(!qaoAutoClose)
Toggle("Close after dragging", isOn: $qaoCloseAfterDrag)
Toggle("Close after uploading", isOn: $qaoCloseAfterUpload)
}
.formStyle(.grouped)
}
private var recordingTab: some View {
Form {
Section("General") {
Toggle("Show controls while recording", isOn: $recShowControls)
Toggle("Display recording time in menu bar", isOn: $recMenuBarTime)
Toggle("Scale Retina videos to 1x", isOn: $recRetina1x)
Toggle("Enable Do Not Disturb while recording", isOn: $recDND)
Toggle("Show cursor", isOn: $recShowCursor)
Toggle("Capture system audio", isOn: $recSystemAudio)
Toggle("Highlight clicks", isOn: $recHighlightClicks)
Toggle("Show keystrokes", isOn: $recShowKeystrokes)
Toggle("Show camera (webcam) overlay", isOn: $recCamera)
Toggle("Remember last recording area", isOn: $recRememberArea)
}
Section("Video") {
Picker("Video FPS", selection: $recFPS) { Text("24").tag(24); Text("30").tag(30); Text("60").tag(60) }
Toggle("Open Video Editor after recording", isOn: $recOpenEditor)
}
Section("GIF") {
Picker("GIF FPS", selection: $gifFPS) { Text("10").tag(10); Text("15").tag(15); Text("30").tag(30) }
VStack(alignment: .leading) { Text("GIF quality"); Slider(value: $gifQuality) }
}
}
.formStyle(.grouped)
}
private var screenshotsTab: some View {
Form {
Picker("File format", selection: $fileFormat) { Text("PNG").tag("PNG"); Text("JPG").tag("JPG"); Text("WebP").tag("WebP"); Text("HEIC").tag("HEIC") }
Toggle("Scale Retina screenshots to 1x", isOn: $shotRetina1x)
Toggle("Add 1px border to all screenshots", isOn: $shotBorder)
Toggle("Freeze screen when taking a screenshot", isOn: $freeze)
Picker("Crosshair mode", selection: $crosshairMode) { Text("Disabled").tag("Disabled"); Text("Always").tag("Always"); Text("With Command key").tag("Command") }
Toggle("Show magnifier", isOn: $showMagnifier)
Picker("Self-Timer interval", selection: $selfTimer) { Text("None").tag(0); Text("3 seconds").tag(3); Text("5 seconds").tag(5); Text("10 seconds").tag(10) }
Toggle("Show cursor on screenshots", isOn: $showCursor)
Picker("Scrolling capture frames", selection: $scrollFrames) { Text("4").tag(4); Text("6").tag(6); Text("8").tag(8); Text("12").tag(12) }
}
.formStyle(.grouped)
}
private var annotateTab: some View {
Form {
Toggle("Inverse arrow direction", isOn: $inverseArrow)
Toggle("Smooth pencil drawing", isOn: $smoothPencil)
Toggle("Draw shadow on objects", isOn: $objectShadow)
Toggle("Automatically expand canvas", isOn: $autoExpand)
Toggle("Show color names", isOn: $colorNames)
Toggle("Annotate window always on top", isOn: $annotateOnTop)
Toggle("Show Dock icon", isOn: $dockIcon)
}
.formStyle(.grouped)
}
private var cloudTab: some View {
Form {
TextField("Cloud server URL", text: $serverUrl, prompt: Text("https://your-server"))
Picker("Screenshot quality", selection: $cloudQuality) { Text("Optimized for sharing").tag("Optimized for sharing"); Text("Full quality").tag("Full quality") }
Picker("Copy to clipboard", selection: $cloudCopyType) { Text("Link").tag("Link"); Text("Direct download link").tag("Direct") }
Text("Upload captures from the Quick Access Overlay to get a shareable link.").font(.caption).foregroundStyle(.secondary)
}
.formStyle(.grouped)
}
private var advancedTab: some View {
Form {
TextField("File name format", text: $fileNameFormat)
Text("Tokens: %y %m %d %H %M %S %w %p %t %a %r").font(.caption).foregroundStyle(.secondary)
Toggle("Add @2x suffix to Retina screenshots", isOn: $retina2xSuffix)
Picker("Copy to clipboard", selection: $copyMode) { Text("File & Image").tag("File & Image"); Text("Image only").tag("Image only"); Text("File only").tag("File only") }
Section("Pinned screenshots") {
Toggle("Rounded corners", isOn: $pinRounded)
Toggle("Shadow", isOn: $pinShadow)
Toggle("Border", isOn: $pinBorder)
}
Toggle("OCR: keep line breaks", isOn: $ocrKeepLineBreaks)
}
.formStyle(.grouped)
}
}
