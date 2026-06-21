# CLAUDE.md

Guidance for AI coding agents working in this repository.

## What this is
SloerShot is a cross-platform (Windows + macOS) screenshot and annotation tool in the spirit
of CleanShot X / PixelSnap. The design splits cleanly into a shared, tested logic core and
thin native shells.

## Architecture
- `core/shotcore` (Rust): the brain. All platform-independent logic - capture geometry, the
 non-destructive annotation model, export/compose, beautify, OCR result model + flow,
 history, licensing, video/GIF, recognition (QR encode/decode + EAN-13), and a computer-
 vision suite (edges, Otsu, segmentation, image diff, perceptual hash, palette, Hough,
 deskew, corners, perspective unwarp, document detection, white balance, sharpen). Exposed
 over a C ABI in `core/shotcore/src/ffi.rs`. 60+ modules, 340+ unit tests.
- `apps/cli` (`shotcli`, Rust): a console demo/proof that exercises the core end to end
 (`demo`, `showcase`, `parity`, `verify-license`). The runnable proof on any machine.
- `apps/windows` (C#, WinUI 3): the Windows GUI. P/Invokes the core via `Interop/ShotCore.cs`.
- `apps/macos` (Swift, SwiftUI): the macOS GUI. Calls the core via `Sources/CShotCore` +
 `Sources/SloerShot/ShotCoreExtras.swift`.
- `backend` (Node/Express): auth, Stripe webhook, and ed25519 entitlement issuance + share
 links. The core verifies entitlements offline.

## Build & test
```
cargo test -p shotcore # run the core unit tests (the source of truth)
cargo run -p shotcli -- parity # exercise the newest modules end to end
cargo build -p shotcore --release # produce the native core (target/release/shotcore.dll)
npm test --prefix backend # backend unit + HTTP e2e tests
dotnet build apps/windows/SloerShot.App/SloerShot.App.csproj -c Debug # Windows GUI (needs .NET SDK)
swift build # macOS GUI, run inside apps/macos (needs a Mac + Xcode)
```

## Conventions when adding a core feature
1. Write the logic as a pure module in `core/shotcore/src/<name>.rs` with unit tests; register
 it with `pub mod <name>;` in `lib.rs`.
2. Keep it additive - do not break existing exhaustive matches or public types.
3. Expose it over the C ABI in `ffi.rs` as `#[no_mangle] pub extern "C" fn shotcore_...`,
 returning JSON strings (freed by `shotcore_string_free`) or status codes.
4. Declare the new function in BOTH headers, kept byte-identical:
 `core/shotcore/include/shotcore.h` and `apps/macos/Sources/CShotCore/shotcore.h`.
5. Add the binding to `apps/windows/.../Interop/ShotCore.cs` (P/Invoke) and
 `apps/macos/.../ShotCoreExtras.swift` (Swift wrapper).
6. Run `cargo test -p shotcore` - it must stay green and warning-free.

## Constraints / notes
- The C ABI is the only boundary: rich data crosses as JSON, plus integer status codes.
- Release builds use thin LTO; on low-memory machines build with
 `--config profile.release.lto=false` to avoid OOM.
- Never commit secrets: `backend/keys/*.pem` (the ed25519 private key) is git-ignored.
- CI (`.github/workflows/ci.yml`) builds + tests the core, backend, macOS, and Windows.
