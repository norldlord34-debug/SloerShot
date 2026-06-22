#nullable enable
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
namespace SloerShot.Capture;
/// <summary>
/// Frozen-screen capture of the whole virtual desktop (GDI BitBlt). Also exposes
/// the virtual-screen rect and the foreground window rect (relative to the capture)
/// so the shell can offer region and window capture modes.
/// </summary>
public static class FrozenScreenCapture
{
public readonly record struct CaptureResult(string Path, int Width, int Height);
[DllImport("user32.dll")]
private static extern int GetSystemMetrics(int index);
[DllImport("user32.dll")]
private static extern IntPtr GetForegroundWindow();
[StructLayout(LayoutKind.Sequential)]
private struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
[DllImport("user32.dll")]
private static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
private const int SmXVirtualScreen = 76;
private const int SmYVirtualScreen = 77;
private const int SmCxVirtualScreen = 78;
private const int SmCyVirtualScreen = 79;
/// <summary>Physical-pixel rect of the whole virtual desktop.</summary>
public static (int X, int Y, int W, int H) VirtualScreenRect()
{
int x = GetSystemMetrics(SmXVirtualScreen);
int y = GetSystemMetrics(SmYVirtualScreen);
int w = GetSystemMetrics(SmCxVirtualScreen);
int h = GetSystemMetrics(SmCyVirtualScreen);
if (w <= 0) w = 1;
if (h <= 0) h = 1;
return (x, y, w, h);
}
/// <summary>Freeze the whole virtual desktop into a PNG at the given path.</summary>
public static CaptureResult CaptureVirtualScreen(string outPath)
{
var (x, y, w, h) = VirtualScreenRect();
using var bmp = new Bitmap(w, h, PixelFormat.Format32bppArgb);
using (var g = Graphics.FromImage(bmp))
{
g.CopyFromScreen(x, y, 0, 0, new Size(w, h), CopyPixelOperation.SourceCopy);
}
bmp.Save(outPath, ImageFormat.Png);
return new CaptureResult(outPath, w, h);
}
/// <summary>Foreground window rect mapped into capture-image pixel coordinates, clamped.</summary>
public static (int X, int Y, int W, int H)? ForegroundWindowRectRelative()
{
var hwnd = GetForegroundWindow();
if (hwnd == IntPtr.Zero) return null;
if (!GetWindowRect(hwnd, out var r)) return null;
var (vx, vy, vw, vh) = VirtualScreenRect();
int x = r.Left - vx;
int y = r.Top - vy;
int w = r.Right - r.Left;
int h = r.Bottom - r.Top;
if (w <= 0 || h <= 0) return null;
if (x < 0) { w += x; x = 0; }
if (y < 0) { h += y; y = 0; }
if (x + w > vw) w = vw - x;
if (y + h > vh) h = vh - y;
if (w <= 0 || h <= 0) return null;
return (x, y, w, h);
}
}
