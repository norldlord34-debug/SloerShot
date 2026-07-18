#nullable enable
using System;
using System.IO;
using System.Collections.Generic;
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
public List<UploadDestination> Destinations { get; set; } = new();
public string ActiveDestinationId { get; set; } = "";
public string ImgurClientId { get; set; } = "";
public bool AfterUploadCopyUrl { get; set; } = true;
public bool AfterUploadOpenUrl { get; set; } = false;
public bool AfterUploadShowQr { get; set; } = false;
public bool AfterCaptureUpload { get; set; } = false;
public string UrlShortener { get; set; } = "none";
public string CustomShortenerConfig { get; set; } = "";
public List<EffectPreset> EffectPresets { get; set; } = new();
private void MergeBuiltInDestinations()
{
var seeded = BuiltInDestinations.Seed();
foreach (var s in seeded)
{
var existing = Destinations.Find(d => d.Id == s.Id);
if (existing == null) Destinations.Add(s);
else if (existing.BuiltIn) { existing.Name = s.Name; existing.ConfigJson = s.ConfigJson; }
}
}
public string ResolveDestinationConfig(UploadDestination d)
{
return ApplyTokens(d?.ConfigJson ?? "");
}
public string ShortenerConfig()
{
switch (UrlShortener)
{
case "isgd": return BuiltInShorteners.Isgd;
case "tinyurl": return BuiltInShorteners.TinyUrl;
case "custom": return ApplyTokens(CustomShortenerConfig ?? "");
default: return "";
}
}
private string ApplyTokens(string cfg)
{
var server = ServerUrl ?? "";
while (server.EndsWith("/")) server = server.Substring(0, server.Length - 1);
cfg = cfg.Replace(BuiltInDestinations.ServerToken, server);
cfg = cfg.Replace(BuiltInDestinations.ImgurClientToken, ImgurClientId ?? "");
return cfg;
}
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
if (Destinations == null) Destinations = new List<UploadDestination>();
if (EffectPresets == null) EffectPresets = new List<EffectPreset>();
MergeBuiltInDestinations();
if (string.IsNullOrWhiteSpace(ActiveDestinationId) || Destinations.TrueForAll(d => d.Id != ActiveDestinationId)) ActiveDestinationId = Destinations[0].Id;
return this;
}
}
