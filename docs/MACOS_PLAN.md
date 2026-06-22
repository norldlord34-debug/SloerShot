# SloerShot for macOS - CleanShot X Parity Mega-Plan

Goal: a native SwiftUI/AppKit macOS app that matches or beats CleanShot X in UI/UX, structure and features.
Shared tested Rust core (shotcore) over the C ABI does the heavy logic; Swift owns the native experience.
Verification: CI macOS job runs swift build (no Mac here to run the GUI). Status legend: [have] [partial] [new].

## Phase 0 - Foundation
- [partial] Menu-bar app (MenuBarExtra) + editor window. Add a proper AppKit AppDelegate for panels/hotkeys.
- [new] SettingsStore (persisted JSON in Application Support) mirroring CleanShot prefs; bind all UI to it.
- [new] After-Capture pipeline: per-type actions (screenshot/recording): Show QAO, Copy, Save, Upload+link, Open Annotate, Open Video Editor.
- [new] CaptureResult model (image/url/kind/size/retina scale) flowing through QAO -> editor -> save/share.
- [new] Global hotkeys via a real registrar (Carbon RegisterEventHotKey) so shortcuts work app-wide, user-rebindable.

## Phase 1 - Quick Access Overlay (QAO) [the signature CleanShot UX]
- [new] Floating NSPanel after every capture (position per settings: Left/corners), with thumbnail.
- [new] Actions: Copy, Save, Pin, Annotate (edit), Upload to Cloud, Close (icons + Copy/Save buttons as in images).
- [new] Drag me: drag the file into any app (NSFilePromiseProvider / pasteboard).
- [new] Right-click context menu: Open Annotation Tool, Pin to Screen, Rotate Left/Right, Upload to Cloud, Quick Look, Show in Finder, Open With, Open in Mail, Temporarily Hide, Close.
- [new] Quick Look (space), swipe-to-dismiss, auto-close timer, multi-display, restore recently closed.
- [new] Alt-modifier variants (copy without closing, advanced upload).

## Phase 2 - All-In-One bar
- [new] Single hotkey shows a bar: Area, Fullscreen, Window, Scrolling, Timer, OCR, Recording + live dimensions + lock aspect + remembers last selection.

## Phase 3 - Advanced capture
- [have] Area (frozen) + Fullscreen + Window (frontmost) + aspect presets.
- [new] Crosshair mode + Magnifier loupe with live x/y coordinates and pixel grid (Cmd toggles).
- [partial] Self-timer countdown (have overlay) - wire interval 3/5/10/30 from settings.
- [new] Window snap: hover-highlight windows, Tab toggles snap, F fullscreen, click to capture exact window.
- [new] Boundary snap: selection edges snap to strong color edges (core edges/lines); Option bypass.
- [new] Resolution + aspect presets before/after (1:1, 4:3, 3:2, 16:9, 9:16, 5:4); editable W/H.
- [new] Capture Previous Area; multi-monitor stitched selection; auto-crop notch.

## Phase 4 - Annotate parity
- [have] Tools: arrow, rectangle, ellipse, line, freehand, text, counter, highlighter, redact + single-key shortcuts + effects + backdrop + copy + set-text + pin.
- [new] Filled rectangle; Arrow 4 styles incl. curved (core curvedArrowPath); Blur (secure+smooth) vs Pixelate (randomized); Spotlight object; Smart Highlighter (core smartHighlight + OCR word detect).
- [new] Color picker popover with custom colors + opacity slider + saved swatches; stroke width; font size; 7 text styles + rounded style + fonts; bold/italic/underline/alignment.
- [new] Crop tool with aspect + edge snapping + rule-of-thirds; Resize tool (change resolution); Rotate/Flip; canvas zoom; lock canvas; expand canvas; combine multiple images (drag-drop, position).
- [new] .cleanshot project format (save/reopen editable) via core document JSON; Save as (PNG/JPG/WebP/HEIC); shadow toggle; sRGB convert; checkerboard transparency.
- [new] Toolbar layout to match CleanShot (crop, shapes, text A, pixelate, spotlight, counter, pencil, background, color swatch, font pt, A-style, Save as, Done) + bottom bar (pin, Drag me, copy/share/cloud).

## Phase 5 - Background tool (social-post beautify)
- [partial] Beautify presets via core beautify. Build the full side panel from the screenshot:
- [new] Gradients grid (10), Wallpapers (bundled + add own +), Blurred presets, Plain color grid.
- [new] Sliders: Padding, Inset, Shadow, Corners; Alignment 9-grid; Ratio dropdown (Auto/1:1/...); Auto-balance (core autoBalance); manual numeric entry; live preview; zoom %; save as preset.

## Phase 6 - Screen recording
- [have] SCK -> AVAssetWriter H.264 MP4 (area/full). Expand:
- [new] Recording area selector + controls bar (mic toggle, system audio, camera, highlight clicks, show keystrokes, dimensions, Record GIF / Record Video).
- [new] System audio + microphone capture & merge; per-track volume; mono option.
- [new] Camera overlay (circular/square/rect, position/size, fullscreen) via AVCaptureSession.
- [new] Highlight clicks (color/size/style/animation) + keystrokes overlay (position/size/dark-light/all keys) composited per frame (core recordCameraRect/recordKeystrokeRect).
- [new] GIF recording + quality/fps/size; countdown; DND while recording; menu-bar timer; pause/resume/restart/discard; controls position; crash recovery.

## Phase 7 - Video editor
- [new] Trim (timeline), change quality, change resolution, audio mono/mute/volume, playback (AVPlayer), estimated size, Trim Only / Trim & Convert (core videoeditOutput).

## Phase 8 - Scrolling capture
- [partial] Vertical stitch (have). Add: live preview panel, auto-scroll, horizontal scrolling, Vision-based alignment, too-long warning, multi-page print (core printPageSlices).

## Phase 9 - OCR / Capture Text
- [have] Vision OCR -> clipboard (quick OCR). Add: auto language detect, keep-line-breaks toggle, paragraph formatting, QR/barcode read (core qrDecode/ean13), link detection (core extractLinks), table to CSV/Markdown (core tableCSV/tableMarkdown), with/without line breaks shortcuts.

## Phase 10 - Floating screenshots (Pin)
- [have] PinPanel (drag, opacity, menu). Add: lock mode (click-through) toggle, arrow-key nudge, two-finger opacity, close-all, hide/show all, middle-click close, rounded/shadow/border prefs, update after crop.

## Phase 11 - Capture history
- [new] Up to 1 month of captures; tabs All/Screenshots/Videos/GIFs; restore, delete, filter; recent menu; double-click opens Annotate (core historySearch).

## Phase 12 - Cloud
- [partial] CloudClient share link. Add: upload screenshots/recordings + copy link, password + expiry, tags, name/tags-before-upload, custom domain/branding, team management, self-destruct, recently-uploaded menu.

## Phase 13 - Settings window (10 tabs, native)
- [new] General (start at login, sounds + shutter, menu-bar icon, export location, hide desktop icons, after-capture matrix).
- [new] Wallpaper (desktop/custom/plain; window screenshot with-wallpaper vs transparent; padding; capture window shadow).
- [new] Shortcuts (rebindable: general + screenshots + recording + annotate; restore defaults; system defaults).
- [new] Quick Access (position, multi-display, overlay size, auto-close action/interval, drag-close, cloud-close).
- [new] Recording (General/Video/GIF tabs: controls, menu-bar time, retina 1x, DND, cursor, highlight clicks, keystrokes, remember area; FPS, audio config, open editor after; GIF fps/quality/size).
- [new] Screenshots (format PNG/JPG/WebP/HEIC, retina 1x, 1px border, freeze, crosshair mode + magnifier, self-timer interval, cursor).
- [new] Annotate (inverse arrow, smooth pencil, shadow on objects, auto-expand canvas, color names, always-on-top, dock icon).
- [new] Cloud (account, plan, screenshot quality, copy-to-clipboard type, recently uploaded, name+tags).
- [new] Advanced (file-name token editor %y %m %d %H %M %S %w %p %t %a %r + UTC, copy-to-clipboard mode, pinned rounded/shadow/border, OCR language + keep line breaks, reset warnings).

## Phase 14 - Integrations / extras (optionals at max)
- [new] URL Scheme API (cleanshot://) for all capture modes + params (core parseURLScheme).
- [new] Finder Share extension + Services (Open in Annotate, Pin).
- [new] Open With CleanShot for images/videos (load into QAO).
- [new] PixelSnap-style measure overlay (core measure/snapObject).
- [new] Push notifications on cloud comments/views; Raycast/AI hooks; presenter overlay.

## Build order (this initiative)
P1 QAO -> P3 crosshair/magnifier -> P13 Settings window -> P5 Background panel -> P4 Annotate parity -> P6/7 Recording+editor -> P11 history -> P2 All-In-One -> P9 OCR extras -> P8 scrolling -> P10 pin extras -> P12 cloud -> P14 integrations.
Each phase: implement in Swift, push, confirm CI macOS swift build green, then next.


## Progress log (live)
Shipped + CI-green (swift build on macOS) in order:
- Phase 1: Quick Access Overlay + crosshair/magnifier selection.
- Phase 13: native Settings window (9 tabs, @AppStorage).
- Phase 5: Background tool side panel (gradients/color/transparent, padding/corners/shadow, live preview).
- Phase 5+: aspect-ratio framing + 9-point alignment (core beautify_framed FFI).
- Phase 4a: color/style popover + core set_style_json FFI (stroke/fill/opacity/width/arrow style/text style/smart highlighter).
- Phase 4b: Save As PNG/JPG/WebP/HEIC + expanded Effects (rotate/flip/sharpen/white balance/auto color/auto-straighten).
- Phase 11: Capture History (on-disk store + thumbnail grid window, search, open/copy/pin/cloud/reveal/delete).
- Phase 2: All-In-One floating capture bar (Cmd+Shift+A).
- Phase 12: real Cloud image hosting (backend /v1/upload + /f/:name, verified end-to-end) wired into QAO + History.
- Phase 10: Settings wired to behavior (pin rounded/shadow/border, QAO position, after-screenshot copy/save/show).
- Phase 9: Capture Text window (review/edit OCR, copy, save .txt, links, table to CSV/Markdown).
- Phase 6: floating recording HUD (live elapsed time + Stop).
Remaining/next: Phase 6 deep (mic/camera/clicks/keystrokes), Phase 7 video editor, Phase 4 crop/resize/combine, Phase 8 scroll polish, Phase 14 integrations.


### Continued (batch 2)
- Phase 4: interactive crop tool (drag + dimmed mask + rule-of-thirds) and resize (25-200%) via core fx.
- Open Image... menu entry (load external image into the editor).
- Phase 6 deep: recording reads cursor/fps from settings; opt-in system audio (AAC track); highlight clicks (fading-ring overlay); show keystrokes (opt-in on-screen badge). All overlays are captured by the recording; nothing logged/stored.
- Phase 7: export a recording to animated GIF (AVAssetImageGenerator + ImageIO).
Note: macOS AV features are CI-compile-verified only (no Mac for runtime testing); the click/keystroke overlays are ephemeral and never log or transmit input.
