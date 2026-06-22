#nullable enable
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
namespace SloerShot.Capture;
/// <summary>Vertical stitch of scrolling-capture frames using per-row signature overlap detection.</summary>
public static class ScrollingStitch
{
public static bool StitchFiles(List<string> paths, string outPath)
{
var frames = new List<Bitmap>();
try
{
foreach (var p in paths) { try { frames.Add(new Bitmap(p)); } catch { } }
if (frames.Count == 0) return false;
int w = frames[0].Width;
var sigs = new List<long[]>();
foreach (var f in frames) sigs.Add(RowSig(f));
var ks = new int[frames.Count];
int totalH = frames[0].Height;
for (int i = 1; i < frames.Count; i++) { ks[i] = BestOverlap(sigs[i - 1], sigs[i]); totalH += Math.Max(0, frames[i].Height - ks[i]); }
using var outBmp = new Bitmap(w, Math.Max(1, totalH), PixelFormat.Format32bppArgb);
using (var g = Graphics.FromImage(outBmp))
{
g.DrawImage(frames[0], new Rectangle(0, 0, w, frames[0].Height), new Rectangle(0, 0, frames[0].Width, frames[0].Height), GraphicsUnit.Pixel);
int y = frames[0].Height;
for (int i = 1; i < frames.Count; i++)
{
int k = ks[i]; int hh = frames[i].Height - k;
if (hh <= 0) continue;
g.DrawImage(frames[i], new Rectangle(0, y, w, hh), new Rectangle(0, k, frames[i].Width, hh), GraphicsUnit.Pixel);
y += hh;
}
}
outBmp.Save(outPath, ImageFormat.Png);
return true;
}
catch { return false; }
finally { foreach (var f in frames) f.Dispose(); }
}
private static long[] RowSig(Bitmap bmp)
{
int w = bmp.Width, h = bmp.Height;
var data = bmp.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
int stride = data.Stride;
var buf = new byte[stride * h];
Marshal.Copy(data.Scan0, buf, 0, buf.Length);
bmp.UnlockBits(data);
var sig = new long[h];
int step = Math.Max(4, (w / 64) * 4);
for (int yy = 0; yy < h; yy++)
{
long s = 0; int rowOff = yy * stride;
for (int x = 0; x + 3 < stride; x += step) { int o = rowOff + x; s += buf[o] + buf[o + 1] + buf[o + 2]; }
sig[yy] = s;
}
return sig;
}
private static int BestOverlap(long[] a, long[] b)
{
int ha = a.Length, hb = b.Length;
int maxOv = Math.Min(ha, hb) - 1;
if (maxOv < 8) return 0;
double best = double.MaxValue; int bestK = 0;
for (int k = 8; k <= maxOv; k++)
{
double diff = 0; int cnt = 0;
int stepj = Math.Max(1, k / 200);
for (int j = 0; j < k; j += stepj) { diff += Math.Abs(a[ha - k + j] - b[j]); cnt++; }
double avg = cnt > 0 ? diff / cnt : double.MaxValue;
if (avg < best) { best = avg; bestK = k; }
}
return bestK;
}
}
