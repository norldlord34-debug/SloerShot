# Builds the SloerShot Windows installer end to end.
#   1. builds the Rust core (release) so shotcore.dll is fresh,
#   2. builds the WinUI 3 app in Release,
#   3. runs Inno Setup's ISCC compiler on SloerShot.iss.
#
# Usage (from the repo root):
#   pwsh packaging\windows\build-installer.ps1
#   pwsh packaging\windows\build-installer.ps1 -Version 0.6.0
#
# Requires: Rust toolchain, .NET 8 SDK, and Inno Setup 6+ (ISCC.exe on PATH or in the
# default install location). Code signing is NOT performed here - see packaging/SIGNING.md.

param(
    [string]$Version = "0.6.0",
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Write-Host "Repo root: $repo"

Write-Host "== Building the Rust core (release) =="
cargo build -p shotcore --release --manifest-path (Join-Path $repo "Cargo.toml")

Write-Host "== Building the WinUI 3 app ($Configuration) =="
$csproj = Join-Path $repo "apps\windows\SloerShot.App\SloerShot.App.csproj"
dotnet build $csproj -c $Configuration

$publish = Join-Path $repo "apps\windows\SloerShot.App\bin\x64\$Configuration\net8.0-windows10.0.19041.0\win-x64"
if (-not (Test-Path (Join-Path $publish "SloerShot.App.exe"))) {
    throw "Build output not found at $publish"
}

Write-Host "== Locating Inno Setup (ISCC.exe) =="
$isccCmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
$iscc = if ($isccCmd) { $isccCmd.Source } else { $null }
if (-not $iscc) {
    foreach ($c in @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe")) {
        if (Test-Path $c) { $iscc = $c; break }
    }
}
if (-not $iscc) {
    throw "ISCC.exe not found. Install Inno Setup 6+ from https://jrsoftware.org/isdl.php"
}
Write-Host "Using ISCC: $iscc"

$iss = Join-Path $PSScriptRoot "SloerShot.iss"
& $iscc "/DAppVersion=$Version" "/DSourceDir=$publish" $iss

$out = Join-Path $repo "dist\installer\SloerShot-$Version-Setup-x64.exe"
if (Test-Path $out) {
    Write-Host "`nInstaller built: $out"
} else {
    Write-Warning "ISCC finished but the expected installer was not found at $out"
}
