; Inno Setup script for SloerShot (Windows, WinUI 3 / .NET 8).
; Builds a single-file installer around the self-contained win-x64 publish folder.
;
; Prerequisites:
;   - Inno Setup 6+  (https://jrsoftware.org/isdl.php)  -> provides the ISCC.exe compiler.
;   - A Release build of the app (see packaging/windows/build-installer.ps1).
;
; Usage (from the repo root, after building):
;   ISCC packaging\windows\SloerShot.iss
;   ; override version or source folder:
;   ISCC /DAppVersion=0.6.0 /DSourceDir="apps\windows\SloerShot.App\bin\x64\Release\net8.0-windows10.0.19041.0\win-x64" packaging\windows\SloerShot.iss
;
; Code signing (optional, requires a certificate - see packaging/SIGNING.md):
;   ISCC /DSign=1 packaging\windows\SloerShot.iss
;   ; and define a "SignTool" in the Inno Setup IDE:  Tools > Configure Sign Tools...
;   ; named "signtool" with a command like:
;   ;   signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a $f

#ifndef AppVersion
  #define AppVersion "0.6.0"
#endif

#ifndef SourceDir
  #define SourceDir "..\..\apps\windows\SloerShot.App\bin\x64\Release\net8.0-windows10.0.19041.0\win-x64"
#endif

#define AppName "SloerShot"
#define AppPublisher "SloerShot"
#define AppExe "SloerShot.App.exe"
#define AppUrl "https://github.com/norldlord34-debug/SloerShot"

[Setup]
AppId={{B9F1B2A0-6E2B-4C1E-9E3D-5C0A7F2E9A10}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppUrl}
AppSupportURL={#AppUrl}/issues
AppUpdatesURL={#AppUrl}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayName={#AppName} {#AppVersion}
UninstallDisplayIcon={app}\{#AppExe}
OutputDir=..\..\dist\installer
OutputBaseFilename=SloerShot-{#AppVersion}-Setup-x64
Compression=lzma2/max
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
#ifdef Sign
SignTool=signtool
SignedUninstaller=yes
#endif

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Start {#AppName} when I sign in to Windows"; GroupDescription: "Startup:"; Flags: unchecked

[Files]
; Ship the entire self-contained publish folder (app + shotcore.dll + Windows App SDK runtime + WinUI assets).
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; Optional auto-start at sign-in (per-user Run key), tied to the "autostart" task.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#AppName}"; ValueData: """{app}\{#AppExe}"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
