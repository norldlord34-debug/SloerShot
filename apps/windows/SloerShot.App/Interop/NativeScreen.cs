#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.Runtime.InteropServices;
using System.Text;

namespace SloerShot.Interop;

/// <summary>
/// Win32 multi-monitor enumeration. Produces the VirtualDesktop JSON consumed by
/// ShotCore.ResolveSelection (shape matches shotcore::geometry::VirtualDesktop).
/// </summary>
public static class NativeScreen
{
    [StructLayout(LayoutKind.Sequential)]
    private struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    [StructLayout(LayoutKind.Sequential)]
    private struct MONITORINFO { public int CbSize; public RECT RcMonitor; public RECT RcWork; public int DwFlags; }

    private delegate bool MonitorEnumProc(IntPtr hMonitor, IntPtr hdc, ref RECT lprc, IntPtr data);

    [DllImport("user32.dll")]
    private static extern bool EnumDisplayMonitors(IntPtr hdc, IntPtr clip, MonitorEnumProc proc, IntPtr data);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern bool GetMonitorInfo(IntPtr hMonitor, ref MONITORINFO info);

    [DllImport("Shcore.dll")]
    private static extern int GetDpiForMonitor(IntPtr hMonitor, int dpiType, out uint dpiX, out uint dpiY);

    private const int MonitorinfofPrimary = 1;
    private const int MdtEffectiveDpi = 0;

    /// <summary>Build the VirtualDesktop JSON for every monitor.</summary>
    public static string DescribeDesktopJson()
    {
        var mons = new List<(int X, int Y, int W, int H, double Scale, bool Primary)>();
        MonitorEnumProc cb = (IntPtr h, IntPtr hdc, ref RECT lprc, IntPtr data) =>
        {
            var mi = new MONITORINFO { CbSize = Marshal.SizeOf<MONITORINFO>() };
            if (GetMonitorInfo(h, ref mi))
            {
                double scale = 1.0;
                if (GetDpiForMonitor(h, MdtEffectiveDpi, out uint dpiX, out _) == 0 && dpiX > 0)
                    scale = dpiX / 96.0;
                var r = mi.RcMonitor;
                bool primary = (mi.DwFlags & MonitorinfofPrimary) != 0;
                mons.Add((r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top, scale, primary));
            }
            return true;
        };
        EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, cb, IntPtr.Zero);
        GC.KeepAlive(cb);

        var sb = new StringBuilder();
        sb.Append("{\"displays\":[");
        for (int i = 0; i < mons.Count; i++)
        {
            var m = mons[i];
            if (i > 0) sb.Append(",");
            sb.Append("{\"id\":").Append(i)
            .Append(",\"bounds\":{\"x\":").Append(m.X)
            .Append(",\"y\":").Append(m.Y)
            .Append(",\"w\":").Append(m.W)
            .Append(",\"h\":").Append(m.H)
            .Append("},\"scale_factor\":").Append(m.Scale.ToString(CultureInfo.InvariantCulture))
            .Append(",\"is_primary\":").Append(m.Primary ? "true" : "false")
            .Append("}");
        }
        sb.Append("]}");
        return sb.ToString();
    }
}
