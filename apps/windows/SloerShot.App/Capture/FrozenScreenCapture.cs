#nullable enable
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

namespace SloerShot.Capture;

/// <summary>
/// Frozen-screen capture. Grabs a still bitmap of the whole virtual desktop the
/// instant it is called (GDI BitBlt via Graphics.CopyFromScreen), so the user
/// selects on a still frame instead of chasing moving UI. The native shell shows
/// the still; selection geometry is resolved through ShotCore.ResolveSelection
/// together with NativeScreen.DescribeDesktopJson.
/// </summary>
public static class FrozenScreenCapture
{
    public readonly record struct CaptureResult(string Path, int Width, int Height);

    [DllImport("user32.dll")]
    private static extern int GetSystemMetrics(int index);

    private const int SmXVirtualScreen = 76;
    private const int SmYVirtualScreen = 77;
    private const int SmCxVirtualScreen = 78;
    private const int SmCyVirtualScreen = 79;

    /// <summary>Freeze the whole virtual desktop into a PNG at the given path.</summary>
    public static CaptureResult CaptureVirtualScreen(string outPath)
    {
        int x = GetSystemMetrics(SmXVirtualScreen);
        int y = GetSystemMetrics(SmYVirtualScreen);
        int w = GetSystemMetrics(SmCxVirtualScreen);
        int h = GetSystemMetrics(SmCyVirtualScreen);
        if (w <= 0) w = 1;
        if (h <= 0) h = 1;

        using var bmp = new Bitmap(w, h, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(bmp))
        {
            g.CopyFromScreen(x, y, 0, 0, new Size(w, h), CopyPixelOperation.SourceCopy);
        }
        bmp.Save(outPath, ImageFormat.Png);
        return new CaptureResult(outPath, w, h);
    }
}
