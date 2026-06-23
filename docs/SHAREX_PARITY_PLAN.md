# SloerShot x ShareX - parity mega plan

Bring ShareX-class power to SloerShot Windows over the shared tested Rust core.
Logic-heavy pieces live in the core (Rust, unit-tested, reused by all shells); the WinUI
shell does UI + the actual network/file I/O. Each phase builds locally (dotnet 0/0) and CI-green.

## ShareX areas mapped
- Custom Uploader (the crown jewel): JSON config -> any HTTP API. Body None/Multipart/Form/JSON/XML/Binary;
 response syntax {response} {json:path} {regex:pat|n} {xml:xpath} {header:name} {base64:..} {input} {filename} {random:a|b}.
- Upload destinations: image/file/text hosts, URL shorteners, URL sharing services. We ship the custom-uploader
 engine (unlimited) + built-ins that need no secret: SloerShot backend, Imgur (anon), generic HTTP PUT/POST, is.gd/TinyURL.
- After-capture tasks (workflow): copy image, copy file, save file, save to clipboard, annotate, upload, copy URL, etc.
- After-upload tasks: copy URL, open URL, QR of URL, shorten URL, copy to clipboard.
- Image effects (ImageEffectsLib): Filters (Blur/GaussianBlur/Pixelate/EdgeDetect/Emboss/Sharpen/Outline/Glow/MeanRemoval/Smooth/ColorDepth/RGBSplit/TornEdge/WaveEdge/Slice/Reflection/Shadow/MatrixConvolution),
 Adjustments (Brightness/Contrast/Gamma/Hue/Saturation/Grayscale/Sepia/BlackWhite/Inverse/Alpha/Colorize/Polaroid/SelectiveColor/ReplaceColor/MatrixColor),
 Manipulations (Resize/Scale/Crop/AutoCrop/Canvas/Flip/Rotate/Skew/RoundedCorners/ForceProportions),
 Drawings (DrawBackground/Image/Border/Checkerboard/Text/Particles).
- Folder indexer (IndexerLib): index a folder tree -> HTML / text / JSON / XML.
- Tools: color picker, ruler, QR (encode/decode), hash check (CRC32/MD5/SHA), image combiner/splitter/thumbnailer, video converter.
- Workflows + global hotkeys (per task), command-line arguments, history, OCR, scrolling capture, screen recording.

## Phases (build order)
- P1 Custom Uploader engine in core: config model, request-plan builder, response syntax parser; Rust tests + FFI.
- P2 Upload destinations: C# executor (Multipart/Form/JSON/Binary via HttpClient) + built-ins (backend, Imgur anon, generic HTTP); destinations manager UI + import a ShareX .sxcu config.
- P3 After-upload tasks + URL shorteners (is.gd, TinyURL, custom) + QR-of-URL.
- P4 After-capture task workflow (configurable pipeline) wired into the capture flow + settings.
- P5 Expanded image effects in core (the ShareX filter/adjustment set) + an Effects gallery in the editor.
- P6 Folder indexer in core (HTML/text/JSON) + a Tools window.
- P7 More tools: hash checker, image splitter, text/paste uploader (+ custom text uploader).
- P8 Workflows/hotkeys expansion + command-line args + settings polish; release.

## Honesty / scope
- We do NOT hardcode 80+ proprietary OAuth uploaders; the custom-uploader engine + .sxcu import covers them generically,
 plus no-secret built-ins. Users paste any ShareX custom uploader and it works.
- Network/file actions are build+logic verified; the core parsers are unit-tested. The WinUI app is built locally (dotnet)
 and on CI, but not driven interactively here.

## Status: shipped (P1-P8 complete, CI-green)
- P1 core custom_uploader.rs (config parse + request plan + response syntax) + FFI; unit-tested.
- P2 UploaderEngine (Multipart/Form/JSON/XML/Binary) + built-ins (SloerShot backend, Imgur anon) + destinations manager (.sxcu import / paste JSON).
- P3 after-upload pipeline (shorten/copy/open/QR) + is.gd/TinyURL + shotcore_qr_encode_png.
- P4 after-capture pipeline (copy/save/annotate/upload) wired into FinishCapture + toast Upload button.
- P5 core effects: pixelate, gamma, posterize, black_white, solarize, colorize, hue, saturation, emboss, edge, sharpen (via shotcore_fx_apply) + Effects menu.
- P6 core indexer (HTML/text/JSON) + shotcore_index_folder + Tools > Index folder.
- P7 tools: hashing.rs (MD5/SHA1/SHA256/SHA512/CRC32) + fx::split_grid + Tools (hash checker, split image, upload text, generate QR).
- P8 command-line args + Ctrl+Shift+U upload hotkey + settings polish.

### Command-line arguments
- `SloerShot.exe --capture area|window|full` (alias `-c`) capture on launch.
- `SloerShot.exe --record` (alias `-r`) toggle screen recording.
- `SloerShot.exe --upload <file>` (alias `-u`) upload a file to the active destination and copy the link.
- `SloerShot.exe <image-file>` open an image in the editor.

### Global hotkeys
- Ctrl+Shift+4 area, Ctrl+Shift+5 window, Ctrl+Shift+6 fullscreen, Ctrl+Shift+2 record, Ctrl+Shift+U upload last capture (plus the configurable primary hotkey).
