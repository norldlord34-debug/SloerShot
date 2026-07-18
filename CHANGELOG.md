# Changelog

All notable changes to SloerShot are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Every release is built by
GitHub Actions with all four CI jobs green (Rust core, Node backend, macOS app, Windows app).

## [0.6.0] - Windows + macOS parity polish

Brings the macOS app up to full parity with Windows, polishes the Windows UX, and
adds fullscreen capture overlays. Everything runs over the shared Rust core (382 unit
tests + 1 integration test).

### Added
- macOS: Swift upload engine (URLSession) using the shared core's custom-uploader
  build-plan / resolve-response, with the same built-in destinations as Windows and a
  destinations manager in Settings.
- macOS: after-capture / after-upload pipeline - auto copy, auto upload, open URL, QR
  of the URL, and is.gd / TinyURL link shortening.
- macOS: Effects studio with live preview + presets, and the expanded effects menu.
- macOS: configurable Workflows with global hotkeys (Carbon `RegisterEventHotKey`) and
  a workflow editor.
- Windows: fullscreen color picker and ruler overlays over a frozen screenshot, each
  with an 8x magnifier loupe (replacing the previous dialog-based tools).
- Windows: optional Light / Dark / System theme, and accessibility names
  (`AutomationProperties`) on icon buttons.
- CI: an end-to-end upload-engine smoke test against the local backend
  (`backend/scripts/test-upload-http.js`) covering binary upload, byte-for-byte
  round-trip, content-type handling, and error cases.

### Changed
- Windows: the Tools menu is grouped into submenus with per-item icons, and Settings is
  reorganized into labelled sections.
- Documentation refreshed to reflect that both native shells are full apps built in CI.

## [0.5.0] - 2026-07-18 - ShareX parity, complete

### Added
- Upload destinations with no account needed: catbox.moe, Litterbox, 0x0.st,
  transfer.sh, tmpfiles.org, file.io, paste.rs (plus the SloerShot backend and anonymous Imgur).
- FTP/FTPS upload and Pastebin / Bearer-API / FTP templates.
- 11 new core effects: RGB split, glow, outline, drop shadow, reflection, polaroid,
  slice glitch, torn/wave edges, replace-color, selective color, image watermark.
- Effects studio: tune parameters with a live preview and save/load presets.
- Configurable workflows, each with its own capture mode, global hotkey, and auto-copy/upload.
- Single instance: a second launch forwards its command-line args to the running app.
- Tools: screen color picker, screen ruler, external actions (run a program on a capture), thumbnailer.
- macOS menu-bar tools over the shared core: file hashes, QR from clipboard, folder index, image split.

## [0.4.0] - 2026-06-23 - ShareX parity

### Added
- ShareX-compatible custom uploader engine (any HTTP API): Multipart / Form / JSON /
  XML / Binary bodies and full ShareX response syntax (`{json:path}`, `{regex:pat|n}`,
  `{xml:tag}`, `{header:name}`, `{base64:..}`, `{input}`, `{filename}`, `{random:a|b}`).
- Built-in destinations (SloerShot backend, anonymous Imgur), `.sxcu` import, and a destination manager.
- After-capture / after-upload workflows: copy, save, annotate, auto-upload, copy URL,
  open in browser, QR of the link, and shorten (is.gd / TinyURL / custom).
- Core effects: pixelate, gamma, posterize, black & white, solarize, colorize, hue
  rotate, saturation, emboss, edge detect, sharpen.
- Tools: folder indexer (HTML/text/JSON), hash checker (MD5/SHA-1/SHA-256/SHA-512/CRC32),
  image splitter, text/paste uploader, QR code generator.
- Global hotkeys (area / window / fullscreen / record / upload) and a command-line interface.

## [0.3.0] - 2026-06-22 - macOS CleanShot-parity batch

### Added
- macOS Quick Access Overlay after every capture, All-In-One capture bar, native
  multi-tab Settings, full editor (crop/resize/style popover/effects/background), Capture
  History window, Capture Text (OCR) window, and screen recording with a floating HUD.
- Crosshair + magnifier selection and a visible countdown overlay for delayed captures.
- Backend: real Cloud image hosting (`POST /v1/upload` + `GET /f/:name`), wired into the
  overlay and history upload.

## [0.2.0] - 2026-06-22

### Added
- Windows: CleanShot-style region capture overlay (dim mask, crosshair, magnifier loupe,
  live size), window capture, scrolling capture, Settings panel, global + in-app
  shortcuts, pin-to-screen, Mica backdrop, OCR copy-text, annotation tools, beautify
  presets, export to PDF and Save As, eyedropper, flatten/scan-QR/deskew, and a captures gallery.

## [0.1.0] - 2026-06-21

### Added
- First tagged build: the shared Rust core (341 tests + 1 integration test), the WinUI 3
  Windows app, and the SwiftPM macOS app, all built by GitHub Actions.

[0.6.0]: https://github.com/norldlord34-debug/SloerShot/releases/tag/v0.6.0
[0.5.0]: https://github.com/norldlord34-debug/SloerShot/releases/tag/v0.5.0
[0.4.0]: https://github.com/norldlord34-debug/SloerShot/releases/tag/v0.4.0
[0.3.0]: https://github.com/norldlord34-debug/SloerShot/releases/tag/v0.3.0
[0.2.0]: https://github.com/norldlord34-debug/SloerShot/releases/tag/v0.2.0
[0.1.0]: https://github.com/norldlord34-debug/SloerShot/releases/tag/v0.1.0
