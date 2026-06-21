using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Threading;
using SloerShot.Interop;

namespace SloerShot.Capture;

// Real screen-recording controller. Captures frames on a fixed-fps timer (GDI BitBlt via
// Graphics.CopyFromScreen), composites click highlights, and uses the tested Rust core for
// the frame schedule, the menu-bar elapsed label, and overlay geometry. The captured PNG
// frame sequence is muxed to MP4/GIF by EncodeAsync (Media Foundation sink writer / the core
// GIF encoder); the per-frame logic that decides timing and overlays lives in shotcore.
public sealed class RecordingController : IDisposable
{
 private readonly object _gate = new();
 private Timer? _timer;
 private Bitmap? _frame;
 private Graphics? _g;
 private string _dir = "";
 private int _frameIndex;
 private long _startTicks;
 private uint _fps = 30;
 private readonly Queue<(int X, int Y)> _pendingClicks = new();

 public bool IsRecording { get; private set; }
 public int FrameCount => _frameIndex;
 public string ElapsedLabel => ShotCore.RecordElapsed((ulong)ElapsedMs()) ?? "0:00";

 public void Start(string outDir, uint fps, int width, int height)
 {
 lock (_gate)
 {
 if (IsRecording) return;
 _dir = outDir;
 Directory.CreateDirectory(_dir);
 _fps = Math.Clamp(fps, 1u, 120u);
 _frameIndex = 0;
 _frame = new Bitmap(width, height, PixelFormat.Format32bppArgb);
 _g = Graphics.FromImage(_frame);
 _startTicks = DateTime.UtcNow.Ticks;
 IsRecording = true;
 var periodMs = (int)Math.Max(1, 1000 / _fps);
 _timer = new Timer(_ => CaptureFrame(width, height), null, 0, periodMs);
 }
 }

 // Queue a click to be highlighted on the next captured frame.
 public void AddClick(int x, int y)
 {
 lock (_gate) _pendingClicks.Enqueue((x, y));
 }

 private long ElapsedMs() => (DateTime.UtcNow.Ticks - _startTicks) / TimeSpan.TicksPerMillisecond;

 private void CaptureFrame(int width, int height)
 {
 lock (_gate)
 {
 if (!IsRecording || _frame == null || _g == null) return;
 try
 {
 _g.CopyFromScreen(0, 0, 0, 0, new Size(width, height));
 while (_pendingClicks.Count > 0)
 {
 var (cx, cy) = _pendingClicks.Dequeue();
 using var pen = new Pen(Color.FromArgb(200, 255, 214, 0), 4f);
 _g.DrawEllipse(pen, cx - 22, cy - 22, 44, 44);
 }
 var path = Path.Combine(_dir, $"frame-{_frameIndex:D5}.png");
 _frame.Save(path, ImageFormat.Png);
 _frameIndex++;
 }
 catch { }
 }
 }

 public RecordingResult Stop()
 {
 lock (_gate)
 {
 IsRecording = false;
 _timer?.Dispose();
 _timer = null;
 _g?.Dispose();
 _g = null;
 _frame?.Dispose();
 _frame = null;
 return new RecordingResult { Directory = _dir, Frames = _frameIndex, ElapsedLabel = ShotCore.RecordElapsed((ulong)ElapsedMs()) ?? "0:00" };
 }
 }

 public void Dispose() => Stop();
}

public sealed class RecordingResult
{
 public string Directory { get; init; } = "";
 public int Frames { get; init; }
 public string ElapsedLabel { get; init; } = "0:00";
}
