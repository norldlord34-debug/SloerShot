#nullable enable
using System;
using System.IO;
using System.Text.Json;
namespace SloerShot.Services;
public sealed class AppSettings
{
public string SaveFolder { get; set; } = "";
public int CaptureDelaySeconds { get; set; } = 0;
public bool HideWindowDuringCapture { get; set; } = true;
public bool AutoCopyToClipboard { get; set; } = false;
public bool OpenFolderAfterSave { get; set; } = false;
public string Format { get; set; } = "png";
public int JpegQuality { get; set; } = 90;
public uint HotkeyModifiers { get; set; } = 6;
public uint HotkeyVk { get; set; } = 0x41;
public bool HotkeyEnabled { get; set; } = true;
public string DefaultMode { get; set; } = "area";
public bool DarkTheme { get; set; } = true;
public uint AccentArgb { get; set; } = 0xFF3D7EFF;
public string ServerUrl { get; set; } = "";
public static string DefaultPicturesFolder()
{
return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyPictures), "SloerShot");
}
private static string ConfigDir()
{
var d = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "SloerShot");
Directory.CreateDirectory(d);
return d;
}
private static string ConfigPath() => Path.Combine(ConfigDir(), "settings.json");
public static AppSettings Load()
{
try
{
var p = ConfigPath();
if (File.Exists(p))
{
var s = JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(p));
if (s != null) return s.Fixup();
}
}
catch { }
return new AppSettings().Fixup();
}
public void Save()
{
try
{
var opts = new JsonSerializerOptions { WriteIndented = true };
File.WriteAllText(ConfigPath(), JsonSerializer.Serialize(this, opts));
}
catch { }
}
public AppSettings Fixup()
{
if (string.IsNullOrWhiteSpace(SaveFolder)) SaveFolder = DefaultPicturesFolder();
try { Directory.CreateDirectory(SaveFolder); } catch { }
if (CaptureDelaySeconds < 0) CaptureDelaySeconds = 0;
if (CaptureDelaySeconds > 10) CaptureDelaySeconds = 10;
if (JpegQuality < 10) JpegQuality = 10;
if (JpegQuality > 100) JpegQuality = 100;
if (Format != "jpg") Format = "png";
return this;
}
}
