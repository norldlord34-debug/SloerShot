# SloerShot Windows - mega plan (CleanShot X / Snagit parity)

Goal: take the WinUI 3 app (C#) over the shared tested Rust core to full CleanShot X + Snagit
parity on Windows. Build order front-loads the defining Windows UX (tray + recording).
Each phase: build locally (dotnet build), then CI (cargo release + dotnet Release + .exe).

## Phase 0 - Foundation [have]
- WinUI 3 editor window (Mica, dark theme, brand accent, PerMonitorV2 DPI).
- Capture: area (RegionOverlay: dim/crosshair/magnifier/Esc), window, fullscreen, scrolling stitch.
- Annotation editor over the core (select/rect/ellipse/arrow/line/text/counter/marker/redact/crop/eyedropper/spotlight, undo/redo, set-text, delete).
- Style flyout (8 colors + thin/medium/thick); Effects menu; Backdrop presets; Export (Save as / PDF / all-to-PDF).
- Captures sidebar (list + grid), zoom, status bar, pin-to-screen, cloud share link, countdown overlay, settings, global hotkey.

## Phase 1 - System tray (defining Windows UX) [new]
- Tray icon (H.NotifyIcon.WinUI) with quick menu: Capture Area/Window/Fullscreen/Scroll, Record, OCR, Pick Color, Open SloerShot, Settings, Quit.
- Run in background / minimize to tray; left-click default capture; balloon on capture.

## Phase 2 - Screen recording end-to-end [partial: controller exists, not wired]
- Record button + region/fullscreen choice; recording HUD (live elapsed + Stop).
- Encode the PNG frame sequence to MP4 (Media Foundation SinkWriter via WIC/MF interop) and to animated GIF (WIC GIF encoder).
- Settings: fps, show cursor, highlight clicks; save into captures; menu/tray Start-Stop.

## Phase 3 - Rich annotation style [new]
- Bind shotcore_editor_set_style_json; style panel: fill on/off + color, opacity, stroke width slider, arrow styles (Straight/Curved/DoubleHeaded/Thin), 7 text styles, smart highlighter, filled shapes.

## Phase 4 - Background tool panel (social beautify) [partial: dropdown only]
- Side panel: gradient presets grid (7 core + 10 extra), solid color grid, transparent; padding/corners/shadow sliders; Ratio dropdown + 9-point alignment via shotcore_beautify_framed_png; live preview; apply/cancel.

## Phase 5 - Capture Text (OCR) window [partial: copy only]
- Window to review/edit recognized text; copy, save .txt, extract links, table to CSV / Markdown (core helpers).

## Phase 6 - Real cloud upload [partial: share link only]
- POST /v1/upload to the configured server; copy the hosted link; from toast / editor / sidebar context menu.

## Phase 7 - Combine images [new]
- Pick several images; stack via shotcore_combine_stack_vertical; composite (GDI) into one; open in editor.

## Phase 8 - Quick Access Overlay [partial: toast]
- Turn the post-capture toast into a floating, draggable overlay: drag-out file, pin, annotate, upload, copy/save, auto-close.

## Phase 9 - Export parity [partial]
- Save As WebP / HEIC via WIC; copy-as (file/image) options; Retina-like 1x scale option.

## Phase 10 - Hotkeys + All-In-One [partial]
- Per-mode configurable global hotkeys; All-In-One floating capture bar.

## Phase 11 - Polish
- Theming + animations, accessibility (narrator/keyboard), perf, About.

## Build order
P1 tray -> P2 recording -> P3 style -> P4 background panel -> P5 OCR window -> P6 cloud -> P7 combine -> P8 QAO -> P9 export -> P10 hotkeys/AIO -> P11 polish.

## Notes
- Local verification: dotnet 9 SDK present; dotnet build of SloerShot.App.csproj is the fast loop; copies target/release/shotcore.dll.
- New core FFI (if any) appended to ffi.rs, synced to both headers, dll rebuilt; the same core already powers macOS.
