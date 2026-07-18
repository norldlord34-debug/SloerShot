# Packaging and code signing

This document describes how SloerShot is packaged for distribution and how to sign the
Windows and macOS builds. Signing requires certificates that are **not** in this
repository - the scripts and steps below are ready to run once you have them.

> Status: the packaging scripts (Inno Setup installer for Windows) are implemented and
> the signing steps are documented. Actually signing a build requires a purchased
> Authenticode certificate (Windows) and an Apple Developer ID (macOS), so the signing
> commands below are prepared but not executed in CI.

## Windows

### 1. Build the installer

```
pwsh packaging/windows/build-installer.ps1 -Version 0.6.0
```

This builds the Rust core (release), builds the WinUI 3 app in Release, and runs
Inno Setup's `ISCC.exe` on `packaging/windows/SloerShot.iss`. The output is
`dist/installer/SloerShot-0.6.0-Setup-x64.exe`. The installer bundles the entire
self-contained `win-x64` folder: `SloerShot.App.exe`, `shotcore.dll`, the Windows App
SDK runtime, and the WinUI assets.

Prerequisites: Rust toolchain, .NET 8 SDK, and [Inno Setup 6+](https://jrsoftware.org/isdl.php).

### 2. Authenticode signing

You need a code-signing certificate from a public CA (DigiCert, Sectigo, GlobalSign,
etc.). EV certificates give immediate SmartScreen reputation; OV certificates build
reputation over time. The certificate lives either in a `.pfx` file or on a hardware
token / cloud HSM (required for EV).

Sign **both** the app executable and the finished installer, and always add an RFC-3161
timestamp so signatures stay valid after the certificate expires:

```
:: sign the app exe before packaging (optional, but recommended)
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
  /f MyCert.pfx /p <password> ^
  "apps\windows\SloerShot.App\bin\x64\Release\net8.0-windows10.0.19041.0\win-x64\SloerShot.App.exe"

:: sign the installer after packaging
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
  /f MyCert.pfx /p <password> ^
  "dist\installer\SloerShot-0.6.0-Setup-x64.exe"
```

To have Inno Setup sign the installer (and its uninstaller) automatically, configure a
Sign Tool in the Inno Setup IDE (`Tools > Configure Sign Tools...`) named `signtool`
with the command:

```
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a $f
```

then compile with signing enabled:

```
ISCC /DSign=1 packaging\windows\SloerShot.iss
```

(`/a` auto-selects the best certificate in the store; use `/f ... /p ...` for a `.pfx`,
or the token vendor's CSP options for an HSM/token.)

### 3. Verify

```
signtool verify /pa /v "dist\installer\SloerShot-0.6.0-Setup-x64.exe"
```

## macOS

The macOS app is built with `swift build -c release`. To distribute it outside the App
Store, sign it with a **Developer ID Application** certificate and notarize it with
Apple. You need an Apple Developer Program membership.

### 1. Bundle and sign

```
# codesign the binary (and any bundled dylibs) with a hardened runtime
codesign --force --options runtime --timestamp \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  path/to/SloerShot.app
```

If you ship a bare executable plus `libshotcore.dylib` (the current CI artifact), sign
the dylib first, then the executable, then zip them. A proper `.app` bundle is
preferred for notarization and Gatekeeper.

### 2. Notarize

```
# submit and wait for Apple's notary service
xcrun notarytool submit SloerShot.zip \
  --apple-id "you@example.com" \
  --team-id TEAMID \
  --password "app-specific-password" \
  --wait

# staple the ticket so the app validates offline
xcrun stapler staple path/to/SloerShot.app
```

Use an app-specific password (created at appleid.apple.com), or store credentials once
with `xcrun notarytool store-credentials`.

### 3. Verify

```
codesign --verify --deep --strict --verbose=2 path/to/SloerShot.app
spctl --assess --type execute --verbose path/to/SloerShot.app
```

## CI note

GitHub Actions builds unsigned artifacts (`sloershot-windows-exe`, `sloershot-macos`).
Signing is a separate, credential-gated step run on a machine that holds the
certificates. Never commit certificates, `.pfx` files, private keys, or notarization
passwords to the repository - inject them as CI secrets or keep them on the signing machine.
