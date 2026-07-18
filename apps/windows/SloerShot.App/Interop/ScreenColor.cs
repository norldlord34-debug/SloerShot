#nullable enable
using System;
using System.Runtime.InteropServices;
namespace SloerShot.Interop;

// Live screen sampling (eyedropper / ruler) via GDI GetPixel + GetCursorPos.
public static class ScreenColor
{
 [StructLayout(LayoutKind.Sequential)]
 public struct POINT { public int X; public int Y; }
 [DllImport("user32.dll")] private static extern bool GetCursorPos(out POINT p);
 [DllImport("user32.dll")] private static extern IntPtr GetDC(IntPtr hWnd);
 [DllImport("user32.dll")] private static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);
 [DllImport("gdi32.dll")] private static extern uint GetPixel(IntPtr hdc, int x, int y);

 public static POINT Cursor() { GetCursorPos(out var p); return p; }

 public static (byte r, byte g, byte b) PixelAt(int x, int y)
 {
 var hdc = GetDC(IntPtr.Zero);
 uint c = GetPixel(hdc, x, y);
 ReleaseDC(IntPtr.Zero, hdc);
 return ((byte)(c & 0xFF), (byte)((c >> 8) & 0xFF), (byte)((c >> 16) & 0xFF));
 }
}
