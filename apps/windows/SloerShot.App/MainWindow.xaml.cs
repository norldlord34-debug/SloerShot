using System;
using System.Collections.ObjectModel;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using SloerShot.Capture;
using SloerShot.Interop;
using SloerShot.Services;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Microsoft.UI.Windowing;
using Windows.Foundation;
using Windows.System;
using Windows.Storage;
using Windows.Storage.Streams;
using Windows.ApplicationModel.DataTransfer;
namespace SloerShot;
public sealed class CaptureItem
{
public string Path { get; init; } = "";
public string Name { get; init; } = "";
public string Dims { get; init; } = "";
public BitmapImage? Thumb { get; init; }
}
public sealed partial class MainWindow : Window
{
private readonly AppSettings _settings;
private readonly UploaderEngine _uploader = new();
private string _lastUploadDeletionUrl = "";
private string _lastUploadUrl = "";
private Workflow? _pendingWorkflow;
private HotkeyService? _hotkey;
private DispatcherTimer? _toastTimer;
private readonly List<PinWindow> _pins = new();
private string? _lastCapturePath;
private int _fxCounter;
private double _renderScale = 1.0;
private int _imgW;
private int _imgH;
private readonly ObservableCollection<CaptureItem> _captures = new();
private readonly List<ToggleButton> _toolToggles = new();
private string? _activeFxTool;
private bool _selecting;
private Point _selStart;
private Microsoft.UI.Xaml.Shapes.Rectangle? _marquee;
private bool _suppressSelection;
private TrayIcon? _tray;
private bool _reallyQuit;
private RecordingController? _recorder;
private DispatcherTimer? _recTimer;
private DateTime _recStart;
private (byte r, byte g, byte b) _styleColor = (0xE5, 0x39, 0x35);
private bool _fillEnabled;
private double _styleWidth = 4;
private double _styleOpacity = 1.0;
private string _arrowStyle = "Straight";
private string _textStyle = "Plain";
private bool _smartHighlighter;
private string _bgType = "gradient";
private string _bgPreset = "Indigo";
private (byte r, byte g, byte b) _bgColor = (255, 255, 255);
public MainWindow()
{
this.InitializeComponent();
_settings = AppSettings.Load();
CoreVersionText.Text = $"core {ShotCore.Version()}";
CapturesList.ItemsSource = _captures;
CapturesGrid.ItemsSource = _captures;
_toolToggles.AddRange(new[] { ToolSelect, ToolRect, ToolEllipse, ToolArrow, ToolLine, ToolText, ToolCounter, ToolMarker, ToolRedact, ToolCrop, ToolSpotlight, ToolPick });
ToolSelect.IsChecked = true;
LoadCapturesFolder();
UpdateEmptyState();
try { this.AppWindow.Resize(new Windows.Graphics.SizeInt32(1180, 760)); } catch { }
try
{
if (Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported())
{
this.SystemBackdrop = new MicaBackdrop();
Root.Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent);
}
}
catch { }
try { this.ExtendsContentIntoTitleBar = true; this.SetTitleBar(TitleBar); } catch { }
InitHotkey();
this.Closed += OnWindowClosed;
try { this.AppWindow.Closing += OnAppWindowClosing; } catch { }
SetupTray();
HandleCommandLine();
}
private void InitHotkey()
{
try
{
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
_hotkey = new HotkeyService();
_hotkey.HotkeyPressed += OnHotkey;
_hotkey.Attach(hwnd);
RegisterCaptureHotkey();
}
catch { UpdateHotkeyHint(); }
}
private void RegisterCaptureHotkey()
{
if (_hotkey != null)
{
_hotkey.Unregister(1); _hotkey.Unregister(2); _hotkey.Unregister(3); _hotkey.Unregister(4); _hotkey.Unregister(5); _hotkey.Unregister(6);
for (int wid = 100; wid < 150; wid++) _hotkey.Unregister(wid);
if (_settings.HotkeyEnabled)
{
_hotkey.Register(1, _settings.HotkeyModifiers, _settings.HotkeyVk);
const uint cs = HotkeyService.ModControl | HotkeyService.ModShift;
_hotkey.Register(2, cs, 0x34);
_hotkey.Register(3, cs, 0x35);
_hotkey.Register(4, cs, 0x36);
_hotkey.Register(5, cs, 0x32);
_hotkey.Register(6, cs, 0x55);
for (int wi = 0; wi < _settings.Workflows.Count && wi < 50; wi++) { var wf = _settings.Workflows[wi]; if (wf.Enabled && wf.HotkeyVk != 0) _hotkey.Register(100 + wi, wf.HotkeyModifiers, wf.HotkeyVk); }
}
}
UpdateHotkeyHint();
}
private void OnHotkey(int id)
{
var dq = DispatcherQueue;
if (id >= 100) { int wi = id - 100; if (dq != null) dq.TryEnqueue(() => RunWorkflow(wi)); else RunWorkflow(wi); return; }
Action act = id switch
{
2 => () => DoCapture("area"),
3 => () => DoCapture("window"),
4 => () => DoCapture("full"),
5 => () => OnToggleRecording(this, new RoutedEventArgs()),
6 => () => _ = UploadToActiveAsync(),
_ => () => DoCapture(_settings.DefaultMode),
};
if (dq != null) dq.TryEnqueue(() => act()); else act();
}
private void UpdateHotkeyHint()
{
try { HotkeyHintText.Text = _settings.HotkeyEnabled ? ("Tip: " + DescribeHotkey() + " captures from anywhere.") : "Global hotkey is off (enable it in Settings)."; } catch { }
}
private string DescribeHotkey()
{
var parts = new List<string>();
if ((_settings.HotkeyModifiers & HotkeyService.ModControl) != 0) parts.Add("Ctrl");
if ((_settings.HotkeyModifiers & HotkeyService.ModShift) != 0) parts.Add("Shift");
if ((_settings.HotkeyModifiers & HotkeyService.ModAlt) != 0) parts.Add("Alt");
if ((_settings.HotkeyModifiers & HotkeyService.ModWin) != 0) parts.Add("Win");
string keyName = (_settings.HotkeyVk >= 0x41 && _settings.HotkeyVk <= 0x5A) ? ((char)_settings.HotkeyVk).ToString() : ("VK" + _settings.HotkeyVk);
parts.Add(keyName);
return string.Join("+", parts);
}
private string CapturesFolder()
{
Directory.CreateDirectory(_settings.SaveFolder);
return _settings.SaveFolder;
}
private void LoadCapturesFolder()
{
try
{
var files = new DirectoryInfo(CapturesFolder()).GetFiles("*.png").OrderByDescending(f => f.LastWriteTimeUtc).Take(60);
_captures.Clear();
foreach (var f in files) _captures.Add(MakeItem(f.FullName));
CaptureCountText.Text = _captures.Count.ToString();
}
catch { }
}
private static CaptureItem MakeItem(string path)
{
int w = 0, h = 0;
try { using var img = System.Drawing.Image.FromFile(path); w = img.Width; h = img.Height; } catch { }
var bmp = new BitmapImage { DecodePixelWidth = 132 };
try { bmp.UriSource = new Uri(path); } catch { }
return new CaptureItem { Path = path, Name = System.IO.Path.GetFileName(path), Dims = w > 0 ? $"{w} x {h}" : "", Thumb = bmp };
}
private void OnCaptureOption(object sender, RoutedEventArgs e)
{
var tag = (sender as FrameworkElement)?.Tag as string ?? "full";
DoCapture(tag);
}
private async void DoCapture(string mode)
{
try
{
if (mode == "scroll") { DoScrollingCapture(); return; }
bool hide = _settings.HideWindowDuringCapture || mode == "area" || mode == "window";
if (hide) { try { this.AppWindow.Hide(); } catch { } }
if (_settings.CaptureDelaySeconds > 0) { try { await new CountdownOverlay().RunAsync(_settings.CaptureDelaySeconds); } catch { } await Task.Delay(120); }
else if (hide) { await Task.Delay(240); }
(int X, int Y, int W, int H)? winRect = null;
if (mode == "window") winRect = FrozenScreenCapture.ForegroundWindowRectRelative();
Directory.CreateDirectory(_settings.SaveFolder);
var stamp = DateTime.Now.ToString("yyyyMMdd-HHmmss");
var frozenPath = System.IO.Path.Combine(_settings.SaveFolder, $"capture-{stamp}.png");
var result = FrozenScreenCapture.CaptureVirtualScreen(frozenPath);
if (mode == "area")
{
var vr = FrozenScreenCapture.VirtualScreenRect();
var overlay = new RegionOverlay();
var pick = await overlay.PickAsync(frozenPath, vr.X, vr.Y, vr.W, vr.H, result.Width, result.Height);
if (hide) { try { this.AppWindow.Show(); this.Activate(); } catch { } }
if (pick == null) { try { File.Delete(frozenPath); } catch { } StatusText.Text = "Capture cancelled."; return; }
var r = pick.Value;
var areaPath = System.IO.Path.Combine(_settings.SaveFolder, $"area-{stamp}.png");
var cj = $"{{\"op\":\"crop\",\"x\":{r.X},\"y\":{r.Y},\"w\":{r.W},\"h\":{r.H}}}";
if (ShotCore.FxApply(frozenPath, areaPath, cj) != 0) { StatusText.Text = "Crop failed."; return; }
try { File.Delete(frozenPath); } catch { }
FinishCapture(areaPath, r.W, r.H, "Captured area");
return;
}
if (hide) { try { this.AppWindow.Show(); this.Activate(); } catch { } }
if (mode == "window" && winRect != null)
{
var r = winRect.Value;
var winPath = System.IO.Path.Combine(_settings.SaveFolder, $"window-{stamp}.png");
var cj = $"{{\"op\":\"crop\",\"x\":{r.X},\"y\":{r.Y},\"w\":{r.W},\"h\":{r.H}}}";
if (ShotCore.FxApply(frozenPath, winPath, cj) == 0) { try { File.Delete(frozenPath); } catch { } FinishCapture(winPath, r.W, r.H, "Captured window"); return; }
}
FinishCapture(result.Path, result.Width, result.Height, $"Captured {result.Width} x {result.Height}");
}
catch (Exception ex) { try { this.AppWindow.Show(); } catch { } StatusText.Text = "Capture failed: " + ex.Message; }
}
private async void DoScrollingCapture()
{
try
{
var info = new ContentDialog { Title = "Scrolling capture", Content = "I will take 6 shots over about 8 seconds. Put the target window in front and scroll down steadily after you click Start.", PrimaryButtonText = "Start", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await info.ShowAsync() != ContentDialogResult.Primary) { StatusText.Text = "Scrolling capture cancelled."; return; }
this.AppWindow.Hide();
await Task.Delay(450);
var win = FrozenScreenCapture.ForegroundWindowRectRelative();
Directory.CreateDirectory(_settings.SaveFolder);
var stamp = DateTime.Now.ToString("yyyyMMdd-HHmmss");
var frames = new List<string>();
int count = 6;
for (int i = 0; i < count; i++)
{
var full = System.IO.Path.Combine(_settings.SaveFolder, $"scrollf-{stamp}-{i}.png");
var res = FrozenScreenCapture.CaptureVirtualScreen(full);
string framePath = full;
if (win != null)
{
var r = win.Value;
var cropPath = System.IO.Path.Combine(_settings.SaveFolder, $"scrollc-{stamp}-{i}.png");
var cj = $"{{\"op\":\"crop\",\"x\":{r.X},\"y\":{r.Y},\"w\":{r.W},\"h\":{r.H}}}";
if (ShotCore.FxApply(full, cropPath, cj) == 0) { try { File.Delete(full); } catch { } framePath = cropPath; }
}
frames.Add(framePath);
if (i < count - 1) await Task.Delay(1300);
}
this.AppWindow.Show(); this.Activate();
var outPath = System.IO.Path.Combine(_settings.SaveFolder, $"scrolling-{stamp}.png");
bool ok = ScrollingStitch.StitchFiles(frames, outPath);
foreach (var f in frames) { try { File.Delete(f); } catch { } }
if (!ok) { StatusText.Text = "Scrolling stitch failed."; return; }
int sw = 0, sh = 0; try { using var im = System.Drawing.Image.FromFile(outPath); sw = im.Width; sh = im.Height; } catch { }
FinishCapture(outPath, sw, sh, "Scrolling capture stitched");
}
catch (Exception ex) { try { this.AppWindow.Show(); } catch { } StatusText.Text = "Scrolling capture failed: " + ex.Message; }
}
private async void OnToggleRecording(object sender, RoutedEventArgs e)
{
 if (_recorder != null && _recorder.IsRecording) { await StopRecordingAsync(); }
 else { StartRecording(); }
}
private void StartRecording()
{
 try
 {
 var (vx, vy, vw, vh) = FrozenScreenCapture.VirtualScreenRect();
 var recDir = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "SloerShot-rec-" + DateTime.UtcNow.Ticks);
 _recorder = new RecordingController();
 _recorder.Start(recDir, 10, vw, vh);
 _recStart = DateTime.UtcNow;
 try { RecordMenuItem.Text = "Stop Recording"; } catch { }
 StatusText.Text = "Recording... 00:00";
 _recTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
 _recTimer.Tick += (_, _) => { var s = (int)(DateTime.UtcNow - _recStart).TotalSeconds; StatusText.Text = $"Recording {s / 60:00}:{s % 60:00} - use Capture > Stop Recording"; };
 _recTimer.Start();
 }
 catch (Exception ex) { StatusText.Text = "Recording failed: " + ex.Message; _recorder = null; }
}
private async System.Threading.Tasks.Task StopRecordingAsync()
{
 _recTimer?.Stop(); _recTimer = null;
 var rec = _recorder; _recorder = null;
 try { RecordMenuItem.Text = "Start Recording"; } catch { }
 if (rec == null) return;
 var result = rec.Stop();
 StatusText.Text = "Encoding GIF...";
 var outPath = System.IO.Path.Combine(CapturesFolder(), "recording-" + DateTime.Now.ToString("yyyyMMdd-HHmmss") + ".gif");
 int rc = await System.Threading.Tasks.Task.Run(() => ShotCore.EncodeGifDir(result.Directory, 10, 960, outPath));
 try { System.IO.Directory.Delete(result.Directory, true); } catch { }
 if (rc == 0 && System.IO.File.Exists(outPath))
 {
 StatusText.Text = "Saved recording " + System.IO.Path.GetFileName(outPath) + " (path copied)";
 try { var dp = new DataPackage(); dp.SetText(outPath); Clipboard.SetContent(dp); } catch { }
 }
 else { StatusText.Text = "GIF encode failed (" + rc + ")"; }
}
private async void OnCombineImages(object sender, RoutedEventArgs e)
{
 try
 {
 var picker = new Windows.Storage.Pickers.FileOpenPicker();
 picker.SuggestedStartLocation = Windows.Storage.Pickers.PickerLocationId.PicturesLibrary;
 picker.FileTypeFilter.Add(".png"); picker.FileTypeFilter.Add(".jpg"); picker.FileTypeFilter.Add(".jpeg");
 var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
 WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
 var files = await picker.PickMultipleFilesAsync();
 if (files == null || files.Count == 0) { StatusText.Text = "Combine cancelled."; return; }
 var bmps = new List<System.Drawing.Bitmap>();
 foreach (var ff in files) { try { bmps.Add(new System.Drawing.Bitmap(ff.Path)); } catch { } }
 if (bmps.Count == 0) { StatusText.Text = "No images loaded."; return; }
 int n = bmps.Count;
 var sizesJson = "[" + string.Join(",", bmps.Select(b => "[" + b.Width + "," + b.Height + "]")) + "]";
 var layoutJson = ShotCore.CombineStackVertical(sizesJson, 16);
 if (string.IsNullOrEmpty(layoutJson)) { foreach (var b in bmps) b.Dispose(); StatusText.Text = "Combine failed."; return; }
 using var docj = System.Text.Json.JsonDocument.Parse(layoutJson);
 int cw = docj.RootElement.GetProperty("canvas_w").GetInt32();
 int ch = docj.RootElement.GetProperty("canvas_h").GetInt32();
 if (cw <= 0 || ch <= 0) { foreach (var b in bmps) b.Dispose(); StatusText.Text = "Combine failed."; return; }
 var outPath = System.IO.Path.Combine(CapturesFolder(), "combined-" + DateTime.Now.ToString("yyyyMMdd-HHmmss") + ".png");
 using (var canvas = new System.Drawing.Bitmap(cw, ch, System.Drawing.Imaging.PixelFormat.Format32bppArgb))
 using (var g = System.Drawing.Graphics.FromImage(canvas))
 {
 g.Clear(System.Drawing.Color.Transparent);
 foreach (var pl in docj.RootElement.GetProperty("placements").EnumerateArray())
 {
 int idx = pl.GetProperty("image").GetInt32();
 int x = pl.GetProperty("x").GetInt32();
 int y = pl.GetProperty("y").GetInt32();
 if (idx >= 0 && idx < bmps.Count) g.DrawImage(bmps[idx], x, y, bmps[idx].Width, bmps[idx].Height);
 }
 canvas.Save(outPath, System.Drawing.Imaging.ImageFormat.Png);
 }
 foreach (var b in bmps) b.Dispose();
 FinishCapture(outPath, cw, ch, "Combined " + n + " images");
 }
 catch (Exception ex) { StatusText.Text = "Combine failed: " + ex.Message; }
}
private async void OnToastDragStarting(UIElement sender, DragStartingEventArgs args)
{
 if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) return;
 var def = args.GetDeferral();
 try { var f = await StorageFile.GetFileFromPathAsync(_lastCapturePath); args.Data.SetStorageItems(new[] { f }); args.Data.RequestedOperation = DataPackageOperation.Copy; }
 catch { }
 finally { def.Complete(); }
}
private void FinishCapture(string path, int w, int h, string status)
{
_suppressSelection = true;
_captures.Insert(0, MakeItem(path));
CaptureCountText.Text = _captures.Count.ToString();
CapturesList.SelectedIndex = 0;
_suppressSelection = false;
LoadImage(path);
StatusText.Text = status + ".";
if (_settings.AutoCopyToClipboard) _ = CopyImageFileAsync(path);
if (_settings.AfterCaptureUpload) _ = UploadToActiveAsync();
if (_pendingWorkflow != null) { var wf = _pendingWorkflow; _pendingWorkflow = null; if (wf.AutoCopy && !_settings.AutoCopyToClipboard) _ = CopyImageFileAsync(path); if (wf.AutoUpload && !_settings.AfterCaptureUpload) _ = UploadToActiveAsync(); }
ShowCaptureToast(path, w, h);
}
private async Task CopyImageFileAsync(string path)
{
try { var file = await StorageFile.GetFileFromPathAsync(path); var dp = new DataPackage(); dp.SetBitmap(RandomAccessStreamReference.CreateFromFile(file)); Clipboard.SetContent(dp); } catch { }
}
private void ShowCaptureToast(string path, int w, int h)
{
try { var bmp = new BitmapImage { DecodePixelWidth = 108 }; bmp.UriSource = new Uri(path); ToastThumb.Source = bmp; } catch { }
ToastText.Text = $"Captured {w} x {h}";
CaptureToast.Visibility = Visibility.Visible;
_toastTimer ??= CreateToastTimer();
_toastTimer.Stop();
_toastTimer.Start();
}
private DispatcherTimer CreateToastTimer()
{
var t = new DispatcherTimer { Interval = TimeSpan.FromSeconds(6) };
t.Tick += (s, e) => { CaptureToast.Visibility = Visibility.Collapsed; t.Stop(); };
return t;
}
private void OnToastDismiss(object sender, RoutedEventArgs e) { CaptureToast.Visibility = Visibility.Collapsed; _toastTimer?.Stop(); }
private async void OnOcr(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null) { StatusText.Text = "Capture something first."; return; }
if (!OcrService.IsAvailable) { StatusText.Text = "OCR engine not available on this system."; return; }
var file = await StorageFile.GetFileFromPathAsync(_lastCapturePath);
using var stream = await file.OpenAsync(FileAccessMode.Read);
var decoder = await Windows.Graphics.Imaging.BitmapDecoder.CreateAsync(stream);
using var sw = await decoder.GetSoftwareBitmapAsync();
using var conv = Windows.Graphics.Imaging.SoftwareBitmap.Convert(sw, Windows.Graphics.Imaging.BitmapPixelFormat.Bgra8, Windows.Graphics.Imaging.BitmapAlphaMode.Premultiplied);
var text = await OcrService.RecognizeTextAsync(conv);
if (string.IsNullOrWhiteSpace(text)) { StatusText.Text = "No text found in this capture."; return; }
var json = await OcrService.RecognizeJsonAsync(conv);
await ShowOcrDialogAsync(text!, json);
}
catch (Exception ex) { StatusText.Text = "OCR failed: " + ex.Message; }
}
private async System.Threading.Tasks.Task ShowOcrDialogAsync(string text, string? ocrJson)
{
 var box = new TextBox { Text = text, AcceptsReturn = true, TextWrapping = TextWrapping.Wrap, Height = 300, Width = 500, FontFamily = new FontFamily("Consolas") };
 var status = new TextBlock { Foreground = new SolidColorBrush(Microsoft.UI.Colors.Gray), FontSize = 12 };
 var actions = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 };
 void Copy(string s) { var dp = new DataPackage(); dp.SetText(s); Clipboard.SetContent(dp); }
 void AddBtn(string label, Action act) { var b = new Button { Content = label }; b.Click += (_, _) => act(); actions.Children.Add(b); }
 AddBtn("Copy", () => { Copy(box.Text); status.Text = "Copied."; });
 AddBtn("Save .txt", () => { try { var sp = System.IO.Path.Combine(CapturesFolder(), "text-" + DateTime.Now.ToString("yyyyMMdd-HHmmss") + ".txt"); System.IO.File.WriteAllText(sp, box.Text); status.Text = "Saved " + System.IO.Path.GetFileName(sp); } catch (Exception ex) { status.Text = ex.Message; } });
 AddBtn("Links", () => { var j = ShotCore.ExtractLinks(box.Text); if (!string.IsNullOrEmpty(j)) { Copy(j!); status.Text = "Links copied."; } else { status.Text = "No links found."; } });
 if (!string.IsNullOrEmpty(ocrJson))
 {
 AddBtn("Table CSV", () => { var c = ShotCore.TableCsv(ocrJson!, 20); if (!string.IsNullOrEmpty(c)) { Copy(c!); status.Text = "CSV copied."; } });
 AddBtn("Table MD", () => { var m = ShotCore.TableMarkdown(ocrJson!, 20); if (!string.IsNullOrEmpty(m)) { Copy(m!); status.Text = "Markdown copied."; } });
 }
 var panel = new StackPanel { Spacing = 10 };
 panel.Children.Add(box); panel.Children.Add(actions); panel.Children.Add(status);
 var dlg = new ContentDialog { Title = "Captured Text", Content = panel, CloseButtonText = "Close", XamlRoot = this.Content.XamlRoot };
 StatusText.Text = "OCR ready (" + text.Length + " chars).";
 await dlg.ShowAsync();
}
private void OnPin(object sender, RoutedEventArgs e)
{
if (_lastCapturePath == null) { StatusText.Text = "Nothing to pin."; return; }
try { var pin = new PinWindow(_lastCapturePath, 140, 140); _pins.Add(pin); pin.Closed += (s, a) => _pins.Remove(pin); pin.Activate(); StatusText.Text = "Pinned to screen."; }
catch (Exception ex) { StatusText.Text = "Pin failed: " + ex.Message; }
}
private void OnOpenFolder(object sender, RoutedEventArgs e)
{
try { Directory.CreateDirectory(_settings.SaveFolder); System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = _settings.SaveFolder, UseShellExecute = true }); }
catch (Exception ex) { StatusText.Text = "Open folder failed: " + ex.Message; }
}
private static CaptureItem? ItemFrom(object sender) => (sender as FrameworkElement)?.DataContext as CaptureItem;
private void OnItemReveal(object sender, RoutedEventArgs e)
{
var it = ItemFrom(sender); if (it == null) return;
try { System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = "explorer.exe", Arguments = $"/select,\"{it.Path}\"", UseShellExecute = true }); } catch { }
}
private async void OnItemCopy(object sender, RoutedEventArgs e) { var it = ItemFrom(sender); if (it != null) { await CopyImageFileAsync(it.Path); StatusText.Text = "Copied image."; } }
private void OnItemCopyPath(object sender, RoutedEventArgs e) { var it = ItemFrom(sender); if (it == null) return; var dp = new DataPackage(); dp.SetText(it.Path); Clipboard.SetContent(dp); StatusText.Text = "Copied path."; }
private void OnItemDelete(object sender, RoutedEventArgs e)
{
var it = ItemFrom(sender); if (it == null) return;
try { if (File.Exists(it.Path)) File.Delete(it.Path); } catch { }
_captures.Remove(it);
CaptureCountText.Text = _captures.Count.ToString();
if (it.Path == _lastCapturePath) { _lastCapturePath = null; UpdateEmptyState(); }
StatusText.Text = "Deleted.";
}
private void OnCaptureSelected(object sender, SelectionChangedEventArgs e)
{
if (_suppressSelection) return;
if ((sender as Selector)?.SelectedItem is CaptureItem item && File.Exists(item.Path)) LoadImage(item.Path);
}
private double FitScale()
{
if (_imgW <= 0 || _imgH <= 0) return 1.0;
double availW = StageScroller.ViewportWidth > 10 ? StageScroller.ViewportWidth - 60 : 900;
double availH = StageScroller.ViewportHeight > 10 ? StageScroller.ViewportHeight - 120 : 560;
double scale = Math.Min(Math.Min(availW / _imgW, availH / _imgH), 1.0);
if (scale <= 0 || double.IsNaN(scale) || double.IsInfinity(scale)) scale = 1.0;
return scale;
}
private void ApplyScale(double scale)
{
scale = Math.Clamp(scale, 0.05, 8.0);
_renderScale = scale;
double dw = _imgW * scale, dh = _imgH * scale;
EditorCanvas.Width = dw; EditorCanvas.Height = dh; EditorCanvas.RenderScale = scale; EditorCanvas.Refresh();
SelectionLayer.Width = dw; SelectionLayer.Height = dh;
DimsText.Text = $"{_imgW} x {_imgH} - {(int)Math.Round(scale * 100)}%";
}
private void OnZoomIn(object sender, RoutedEventArgs e) { if (_lastCapturePath != null) ApplyScale(_renderScale * 1.2); }
private void OnZoomOut(object sender, RoutedEventArgs e) { if (_lastCapturePath != null) ApplyScale(_renderScale / 1.2); }
private void OnZoomFit(object sender, RoutedEventArgs e) { if (_lastCapturePath != null) ApplyScale(FitScale()); }
private void LoadImage(string path)
{
try
{
int w, h;
using (var img = System.Drawing.Image.FromFile(path)) { w = img.Width; h = img.Height; }
_imgW = w; _imgH = h; _lastCapturePath = path;
var bmp = new BitmapImage();
bmp.UriSource = new Uri(path);
EditorCanvas.Background = new ImageBrush { ImageSource = bmp, Stretch = Stretch.Fill };
EditorCanvas.NewDocument((uint)w, (uint)h);
ApplyScale(FitScale());
ClearMarquee();
UpdateEmptyState();
}
catch (Exception ex) { StatusText.Text = "Load failed: " + ex.Message; }
}
private void UpdateEmptyState()
{
bool has = _lastCapturePath != null;
EmptyState.Visibility = has ? Visibility.Collapsed : Visibility.Visible;
Toolbar.Visibility = has ? Visibility.Visible : Visibility.Collapsed;
}
private void SyncToggles(ToggleButton active)
{
foreach (var t in _toolToggles) if (!ReferenceEquals(t, active)) t.IsChecked = false;
}
private void OnToolToggle(object sender, RoutedEventArgs e)
{
if (sender is not ToggleButton tb) return;
if (tb.IsChecked != true) { tb.IsChecked = true; return; }
SyncToggles(tb);
ActivateTool(tb.Tag as string ?? "0");
}
private void ActivateTool(string tag)
{
if (tag.StartsWith("fx:"))
{
_activeFxTool = tag.Substring(3);
SelectionLayer.IsHitTestVisible = true;
StatusText.Text = _activeFxTool == "crop" ? "Drag a region to crop." : (_activeFxTool == "pick" ? "Click a pixel to copy its color." : "Drag a region to spotlight.");
}
else
{
_activeFxTool = null;
SelectionLayer.IsHitTestVisible = false;
ClearMarquee();
if (uint.TryParse(tag, out var tool)) EditorCanvas.SetTool(tool);
}
}
private void OnUndo(object sender, RoutedEventArgs e) => EditorCanvas.Undo();
private void OnRedo(object sender, RoutedEventArgs e) => EditorCanvas.Redo();
private void OnDelete(object sender, RoutedEventArgs e) => EditorCanvas.DeleteSelected();
private void OnSelectionPressed(object sender, PointerRoutedEventArgs e)
{
if (_activeFxTool == null) return;
if (_activeFxTool == "pick") { PickColorAt(e.GetCurrentPoint(SelectionLayer).Position); return; }
_selStart = e.GetCurrentPoint(SelectionLayer).Position;
_selecting = true;
SelectionLayer.CapturePointer(e.Pointer);
ClearMarquee();
_marquee = new Microsoft.UI.Xaml.Shapes.Rectangle { Stroke = Rgba(255, 61, 126, 255), StrokeThickness = 1.5, StrokeDashArray = new DoubleCollection { 4, 3 }, Fill = Rgba(64, 61, 126, 255) };
Canvas.SetLeft(_marquee, _selStart.X);
Canvas.SetTop(_marquee, _selStart.Y);
SelectionLayer.Children.Add(_marquee);
}
private void OnSelectionMoved(object sender, PointerRoutedEventArgs e)
{
if (!_selecting || _marquee == null) return;
var p = e.GetCurrentPoint(SelectionLayer).Position;
double x = Math.Min(p.X, _selStart.X), y = Math.Min(p.Y, _selStart.Y);
double w = Math.Abs(p.X - _selStart.X), h = Math.Abs(p.Y - _selStart.Y);
Canvas.SetLeft(_marquee, x); Canvas.SetTop(_marquee, y);
_marquee.Width = w; _marquee.Height = h;
}
private void OnSelectionReleased(object sender, PointerRoutedEventArgs e)
{
if (!_selecting || _marquee == null) { _selecting = false; return; }
_selecting = false;
SelectionLayer.ReleasePointerCapture(e.Pointer);
double dx = Canvas.GetLeft(_marquee), dy = Canvas.GetTop(_marquee), dw = _marquee.Width, dh = _marquee.Height;
var op = _activeFxTool;
ClearMarquee();
if (dw < 6 || dh < 6 || op == null) { StatusText.Text = "Selection too small - drag a larger region."; return; }
var s = _renderScale <= 0 ? 1.0 : _renderScale;
int ix = (int)Math.Round(dx / s), iy = (int)Math.Round(dy / s);
int iw = (int)Math.Round(dw / s), ih = (int)Math.Round(dh / s);
ix = Math.Clamp(ix, 0, Math.Max(0, _imgW - 1));
iy = Math.Clamp(iy, 0, Math.Max(0, _imgH - 1));
iw = Math.Clamp(iw, 1, _imgW - ix);
ih = Math.Clamp(ih, 1, _imgH - iy);
string json = op == "crop" ? $"{{\"op\":\"crop\",\"x\":{ix},\"y\":{iy},\"w\":{iw},\"h\":{ih}}}" : $"{{\"op\":\"spotlight\",\"x\":{ix},\"y\":{iy},\"w\":{iw},\"h\":{ih},\"dim\":0.6}}";
ApplyFx(json, op);
}
private void ClearMarquee()
{
if (_marquee != null) { SelectionLayer.Children.Remove(_marquee); _marquee = null; }
}
private static SolidColorBrush Rgba(byte a, byte r, byte g, byte b) => new(new Windows.UI.Color { A = a, R = r, G = g, B = b });
private static byte[]? LoadRgba(string path, out int w, out int h)
{
w = 0; h = 0;
try
{
using var bmp = new System.Drawing.Bitmap(path);
w = bmp.Width; h = bmp.Height;
var data = bmp.LockBits(new System.Drawing.Rectangle(0, 0, w, h), System.Drawing.Imaging.ImageLockMode.ReadOnly, System.Drawing.Imaging.PixelFormat.Format32bppArgb);
int stride = data.Stride;
var buf = new byte[stride * h];
System.Runtime.InteropServices.Marshal.Copy(data.Scan0, buf, 0, buf.Length);
bmp.UnlockBits(data);
var rgba = new byte[w * h * 4];
for (int y = 0; y < h; y++)
{
int ro = y * stride; int wo = y * w * 4;
for (int x = 0; x < w; x++)
{
int o = ro + x * 4; int d = wo + x * 4;
rgba[d] = buf[o + 2]; rgba[d + 1] = buf[o + 1]; rgba[d + 2] = buf[o]; rgba[d + 3] = buf[o + 3];
}
}
return rgba;
}
catch { return null; }
}
private void FlattenDocument()
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
var rgba = LoadRgba(_lastCapturePath, out int w, out int h);
if (rgba == null) { StatusText.Text = "Could not read image."; return; }
var corners = ShotCore.DetectDocument(rgba, (uint)w, (uint)h, 24);
if (string.IsNullOrEmpty(corners)) { StatusText.Text = "No document detected."; return; }
var dir = System.IO.Path.GetDirectoryName(_lastCapturePath) ?? CapturesFolder();
var outPath = System.IO.Path.Combine(dir, $"flat-{DateTime.Now:yyyyMMdd-HHmmss}-{++_fxCounter}.png");
if (ShotCore.PerspectiveUnwarp(_lastCapturePath, outPath, corners, (uint)w, (uint)h) != 0) { StatusText.Text = "Flatten failed."; return; }
_suppressSelection = true; _captures.Insert(0, MakeItem(outPath)); CaptureCountText.Text = _captures.Count.ToString(); CapturesList.SelectedIndex = 0; _suppressSelection = false;
LoadImage(outPath); StatusText.Text = "Document flattened.";
}
catch (Exception ex) { StatusText.Text = "Flatten failed: " + ex.Message; }
}
private void ScanCode()
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
var rgba = LoadRgba(_lastCapturePath, out int w, out int h);
if (rgba == null) { StatusText.Text = "Could not read image."; return; }
var text = ShotCore.QrDecode(rgba, (uint)w, (uint)h);
if (string.IsNullOrEmpty(text)) text = ShotCore.Ean13Decode(rgba, (uint)w, (uint)h);
if (string.IsNullOrEmpty(text)) { StatusText.Text = "No QR or barcode found."; return; }
var dp = new DataPackage(); dp.SetText(text); Clipboard.SetContent(dp);
StatusText.Text = "Decoded: " + text + " (copied).";
}
catch (Exception ex) { StatusText.Text = "Scan failed: " + ex.Message; }
}
private void OnToggleView(object sender, RoutedEventArgs e)
{
bool grid = ViewToggle.IsChecked == true;
CapturesGrid.Visibility = grid ? Visibility.Visible : Visibility.Collapsed;
CapturesList.Visibility = grid ? Visibility.Collapsed : Visibility.Visible;
}
private async void OnSetText(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null) { StatusText.Text = "Capture something first."; return; }
var box = new TextBox { AcceptsReturn = true, PlaceholderText = "Type annotation text", MinWidth = 320, Height = 90, TextWrapping = TextWrapping.Wrap };
var dialog = new ContentDialog { Title = "Set annotation text", Content = box, PrimaryButtonText = "Apply", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await dialog.ShowAsync() == ContentDialogResult.Primary) { EditorCanvas.SetSelectedText(box.Text ?? ""); StatusText.Text = "Text applied (select a text annotation first)."; }
}
catch (Exception ex) { StatusText.Text = "Set text failed: " + ex.Message; }
}
private async void OnUploadShare(object sender, RoutedEventArgs e)
{
await UploadToActiveAsync();
}
private async Task UploadToActiveAsync()
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
var dest = ActiveDestination();
if (dest == null) { StatusText.Text = "No upload destination configured."; return; }
if (dest.Id == "builtin-sloershot" && string.IsNullOrWhiteSpace(_settings.ServerUrl)) { StatusText.Text = "Set a share server URL in Settings first."; return; }
if (dest.Id == "builtin-imgur" && string.IsNullOrWhiteSpace(_settings.ImgurClientId)) { StatusText.Text = "Set your Imgur Client ID in Manage destinations first."; return; }
var cfg = _settings.ResolveDestinationConfig(dest);
StatusText.Text = "Uploading to " + dest.Name + "...";
var outcome = await _uploader.UploadFileAsync(cfg, _lastCapturePath);
if (!outcome.Success) { StatusText.Text = "Upload failed: " + outcome.Error; return; }
_lastUploadDeletionUrl = outcome.DeletionUrl ?? "";
var finalUrl = outcome.Url;
if (!string.IsNullOrEmpty(_settings.UrlShortener) && _settings.UrlShortener != "none")
{
var scfg = _settings.ShortenerConfig();
if (!string.IsNullOrWhiteSpace(scfg))
{
StatusText.Text = "Shortening link...";
var sres = await _uploader.ShortenUrlAsync(scfg, outcome.Url);
if (sres.Success && !string.IsNullOrWhiteSpace(sres.Url)) finalUrl = sres.Url;
}
}
_lastUploadUrl = finalUrl;
if (_settings.AfterUploadCopyUrl) { var dp = new DataPackage(); dp.SetText(finalUrl); Clipboard.SetContent(dp); }
StatusText.Text = "Uploaded to " + dest.Name + " - " + finalUrl;
if (_settings.AfterUploadOpenUrl) { try { System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = finalUrl, UseShellExecute = true }); } catch { } }
if (_settings.AfterUploadShowQr) { await ShowQrDialogAsync(finalUrl); }
}
catch (Exception ex) { StatusText.Text = "Share failed: " + ex.Message; }
}
private UploadDestination? ActiveDestination()
{
var list = _settings.Destinations;
if (list == null || list.Count == 0) return null;
foreach (var d in list) { if (d.Id == _settings.ActiveDestinationId) return d; }
return list[0];
}
private void OnUploadFlyoutOpening(object sender, object e)
{
try
{
var mf = UploadFlyout;
if (mf == null) return;
mf.Items.Clear();
foreach (var d in _settings.Destinations)
{
var captured = d;
var item = new MenuFlyoutItem { Text = d.Name + (d.Id == _settings.ActiveDestinationId ? " (active)" : "") };
item.Click += async (s, ev) => { _settings.ActiveDestinationId = captured.Id; _settings.Save(); await UploadToActiveAsync(); };
mf.Items.Add(item);
}
mf.Items.Add(new MenuFlyoutSeparator());
var manage = new MenuFlyoutItem { Text = "Manage destinations..." };
manage.Click += async (s, ev) => { await OnManageDestinationsAsync(); };
mf.Items.Add(manage);
if (!string.IsNullOrEmpty(_lastUploadUrl))
{
mf.Items.Add(new MenuFlyoutSeparator());
var qr = new MenuFlyoutItem { Text = "QR of last link" };
qr.Click += async (s, ev) => { await ShowQrDialogAsync(_lastUploadUrl); };
mf.Items.Add(qr);
var openLink = new MenuFlyoutItem { Text = "Open last link" };
openLink.Click += (s, ev) => { try { System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = _lastUploadUrl, UseShellExecute = true }); } catch { } };
mf.Items.Add(openLink);
}
}
catch { }
}
private async Task OnManageDestinationsAsync()
{
var list = new ListView { SelectionMode = ListViewSelectionMode.Single, MaxHeight = 220, MinWidth = 470 };
void Refresh()
{
list.Items.Clear();
foreach (var d in _settings.Destinations)
{
var label = d.Name + (d.Id == _settings.ActiveDestinationId ? " (active)" : "") + (d.BuiltIn ? " [built-in]" : "");
list.Items.Add(new ListViewItem { Content = label, Tag = d.Id });
}
}
Refresh();
string? Selected() => (list.SelectedItem as ListViewItem)?.Tag as string;
var setActive = new Button { Content = "Set active" };
setActive.Click += (s, e) => { var id = Selected(); if (id != null) { _settings.ActiveDestinationId = id; Refresh(); } };
var import = new Button { Content = "Import .sxcu..." };
import.Click += async (s, e) => { var d = await ImportSxcuAsync(); if (d != null) { _settings.Destinations.Add(d); _settings.ActiveDestinationId = d.Id; Refresh(); } };
var remove = new Button { Content = "Remove" };
remove.Click += (s, e) => { var id = Selected(); if (id != null) { var d = _settings.Destinations.Find(x => x.Id == id); if (d != null && !d.BuiltIn) { _settings.Destinations.Remove(d); if (_settings.ActiveDestinationId == id && _settings.Destinations.Count > 0) _settings.ActiveDestinationId = _settings.Destinations[0].Id; Refresh(); } } };
var btnRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 };
btnRow.Children.Add(setActive); btnRow.Children.Add(import); btnRow.Children.Add(remove);
var addBox = new TextBox { AcceptsReturn = true, Height = 96, TextWrapping = TextWrapping.Wrap, PlaceholderText = "Paste a ShareX custom uploader JSON, then click Add custom" };
var addBtn = new Button { Content = "Add custom" };
addBtn.Click += (s, e) => { var tx = addBox.Text ?? ""; if (!string.IsNullOrWhiteSpace(tx)) { var d = new UploadDestination { Name = ExtractName(tx, "Custom uploader"), ConfigJson = tx, BuiltIn = false }; _settings.Destinations.Add(d); addBox.Text = ""; Refresh(); } };
var tplCombo = new ComboBox { MinWidth = 210 };
tplCombo.Items.Add("Pastebin (needs key)"); tplCombo.Items.Add("Bearer API (needs token)"); tplCombo.Items.Add("FTP / FTPS");
tplCombo.SelectedIndex = 0;
var tplBtn = new Button { Content = "Load template" };
tplBtn.Click += (s, e) => { addBox.Text = tplCombo.SelectedIndex == 1 ? BuiltInDestinations.BearerTemplate : (tplCombo.SelectedIndex == 2 ? BuiltInDestinations.FtpTemplate : BuiltInDestinations.PastebinTemplate); };
var tplRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 };
tplRow.Children.Add(tplCombo); tplRow.Children.Add(tplBtn);
var imgurBox = new TextBox { Text = _settings.ImgurClientId, PlaceholderText = "Imgur Client ID (needed for Imgur)", MinWidth = 470 };
var panel = new StackPanel { Spacing = 8, MinWidth = 480 };
panel.Children.Add(new TextBlock { Text = "ShareX-compatible upload destinations. Use %SERVER% for your backend URL.", TextWrapping = TextWrapping.Wrap });
panel.Children.Add(list);
panel.Children.Add(btnRow);
panel.Children.Add(addBox);
panel.Children.Add(addBtn);
panel.Children.Add(tplRow);
panel.Children.Add(new TextBlock { Text = "Imgur Client ID", FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
panel.Children.Add(imgurBox);
var scroll = new ScrollViewer { Content = panel, VerticalScrollBarVisibility = ScrollBarVisibility.Auto, MaxHeight = 520 };
var dialog = new ContentDialog { Title = "Upload Destinations", PrimaryButtonText = "Done", Content = scroll, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
await dialog.ShowAsync();
_settings.ImgurClientId = imgurBox.Text ?? "";
_settings.Fixup();
_settings.Save();
StatusText.Text = "Destinations updated.";
}
private async Task<UploadDestination?> ImportSxcuAsync()
{
try
{
var picker = new Windows.Storage.Pickers.FileOpenPicker();
picker.FileTypeFilter.Add(".sxcu");
picker.FileTypeFilter.Add(".json");
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var file = await picker.PickSingleFileAsync();
if (file == null) return null;
var text = await Windows.Storage.FileIO.ReadTextAsync(file);
return new UploadDestination { Name = ExtractName(text, file.Name), ConfigJson = text, BuiltIn = false };
}
catch { return null; }
}
private static string ExtractName(string json, string fallback)
{
try { using var doc = System.Text.Json.JsonDocument.Parse(json); if (doc.RootElement.TryGetProperty("Name", out var n)) { var sv = n.GetString(); if (!string.IsNullOrWhiteSpace(sv)) return sv; } } catch { }
return System.IO.Path.GetFileNameWithoutExtension(fallback);
}
private async Task ShowQrDialogAsync(string text)
{
try
{
var tmp = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "sloershot-qr-" + Guid.NewGuid().ToString("N") + ".png");
int rc = await Task.Run(() => ShotCore.QrEncodePng(text, 8, 4, tmp));
if (rc != 0 || !File.Exists(tmp)) { StatusText.Text = "QR generation failed."; return; }
var bmp = new BitmapImage();
using (var fs = File.OpenRead(tmp)) { await bmp.SetSourceAsync(fs.AsRandomAccessStream()); }
var img = new Image { Source = bmp, Width = 280, Height = 280 };
var linkText = new TextBlock { Text = text, TextWrapping = TextWrapping.Wrap, MaxWidth = 300, HorizontalAlignment = HorizontalAlignment.Center };
var panel = new StackPanel { Spacing = 10, HorizontalAlignment = HorizontalAlignment.Center };
panel.Children.Add(img);
panel.Children.Add(linkText);
var dlg = new ContentDialog { Title = "QR code", Content = panel, PrimaryButtonText = "Save PNG...", SecondaryButtonText = "Copy link", CloseButtonText = "Close", XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
var r = await dlg.ShowAsync();
if (r == ContentDialogResult.Primary)
{
var picker = new Windows.Storage.Pickers.FileSavePicker();
picker.SuggestedFileName = "qr";
picker.FileTypeChoices.Add("PNG", new List<string> { ".png" });
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var destFile = await picker.PickSaveFileAsync();
if (destFile != null) { try { File.Copy(tmp, destFile.Path, true); StatusText.Text = "QR saved."; } catch { } }
}
else if (r == ContentDialogResult.Secondary)
{
var dp = new DataPackage(); dp.SetText(text); Clipboard.SetContent(dp); StatusText.Text = "Link copied.";
}
try { File.Delete(tmp); } catch { }
}
catch (Exception ex) { StatusText.Text = "QR error: " + ex.Message; }
}
private async void OnIndexFolder(object sender, RoutedEventArgs e)
{
try
{
var folder = await PickFolderAsync();
if (folder == null) return;
var combo = new ComboBox { MinWidth = 160 };
combo.Items.Add("HTML"); combo.Items.Add("Text"); combo.Items.Add("JSON");
combo.SelectedIndex = 0;
var panel = new StackPanel { Spacing = 8 };
panel.Children.Add(new TextBlock { Text = "Folder: " + folder, TextWrapping = TextWrapping.Wrap, MaxWidth = 380 });
panel.Children.Add(combo);
var dlg = new ContentDialog { Title = "Folder Indexer", Content = panel, PrimaryButtonText = "Create", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await dlg.ShowAsync() != ContentDialogResult.Primary) return;
var idx = combo.SelectedIndex;
var fmt = idx == 1 ? "text" : (idx == 2 ? "json" : "html");
var ext = idx == 1 ? ".txt" : (idx == 2 ? ".json" : ".html");
var picker = new Windows.Storage.Pickers.FileSavePicker();
picker.SuggestedFileName = "index";
picker.FileTypeChoices.Add(fmt.ToUpperInvariant(), new List<string> { ext });
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var outFile = await picker.PickSaveFileAsync();
if (outFile == null) return;
StatusText.Text = "Indexing folder...";
int rc = await Task.Run(() => ShotCore.IndexFolder(folder, fmt, outFile.Path));
if (rc != 0) { StatusText.Text = "Indexing failed (" + rc + ")."; return; }
StatusText.Text = "Folder indexed: " + outFile.Path;
try { System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = outFile.Path, UseShellExecute = true }); } catch { }
}
catch (Exception ex) { StatusText.Text = "Indexer error: " + ex.Message; }
}
private async void OnHashCheck(object sender, RoutedEventArgs e)
{
try
{
var picker = new Windows.Storage.Pickers.FileOpenPicker();
picker.FileTypeFilter.Add("*");
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var file = await picker.PickSingleFileAsync();
if (file == null) return;
StatusText.Text = "Hashing...";
var json = await Task.Run(() => ShotCore.FileHashes(file.Path));
if (string.IsNullOrEmpty(json)) { StatusText.Text = "Hashing failed."; return; }
using var doc = System.Text.Json.JsonDocument.Parse(json);
var root = doc.RootElement;
var panel = new StackPanel { Spacing = 6, MinWidth = 470 };
panel.Children.Add(new TextBlock { Text = file.Name, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, TextWrapping = TextWrapping.Wrap });
foreach (var algo in new[] { "md5", "sha1", "sha256", "sha512", "crc32" })
{
if (!root.TryGetProperty(algo, out var hv)) continue;
var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
row.Children.Add(new TextBlock { Text = algo.ToUpperInvariant(), Width = 60, VerticalAlignment = VerticalAlignment.Center });
row.Children.Add(new TextBox { Text = hv.GetString() ?? "", IsReadOnly = true, MinWidth = 360, FontFamily = new Microsoft.UI.Xaml.Media.FontFamily("Consolas") });
panel.Children.Add(row);
}
var dlg = new ContentDialog { Title = "File hashes", Content = new ScrollViewer { Content = panel, MaxHeight = 420 }, CloseButtonText = "Close", XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
await dlg.ShowAsync();
StatusText.Text = "Hashes computed.";
}
catch (Exception ex) { StatusText.Text = "Hash error: " + ex.Message; }
}
private async void OnSplitImage(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture or open an image first."; return; }
var rowsBox = new NumberBox { Header = "Rows", Value = 2, Minimum = 1, Maximum = 20, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Inline };
var colsBox = new NumberBox { Header = "Columns", Value = 2, Minimum = 1, Maximum = 20, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Inline };
var panel = new StackPanel { Spacing = 10, MinWidth = 240 };
panel.Children.Add(rowsBox); panel.Children.Add(colsBox);
var dlg = new ContentDialog { Title = "Split image", Content = panel, PrimaryButtonText = "Split", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await dlg.ShowAsync() != ContentDialogResult.Primary) return;
uint rows = (uint)Math.Max(1, (int)rowsBox.Value);
uint cols = (uint)Math.Max(1, (int)colsBox.Value);
var outDir = System.IO.Path.Combine(CapturesFolder(), "split-" + DateTime.Now.ToString("yyyyMMdd-HHmmss"));
Directory.CreateDirectory(outDir);
StatusText.Text = "Splitting...";
int rc = await Task.Run(() => ShotCore.SplitImage(_lastCapturePath, rows, cols, outDir, "tile"));
if (rc != 0) { StatusText.Text = "Split failed (" + rc + ")."; return; }
StatusText.Text = "Split into " + (rows * cols) + " tiles.";
try { System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = outDir, UseShellExecute = true }); } catch { }
}
catch (Exception ex) { StatusText.Text = "Split error: " + ex.Message; }
}
private async void OnUploadText(object sender, RoutedEventArgs e)
{
try
{
var box = new TextBox { AcceptsReturn = true, Height = 180, MinWidth = 420, TextWrapping = TextWrapping.Wrap, PlaceholderText = "Type or paste text to upload to the active destination" };
var dlg = new ContentDialog { Title = "Upload text", Content = box, PrimaryButtonText = "Upload", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await dlg.ShowAsync() != ContentDialogResult.Primary) return;
var text = box.Text ?? "";
if (string.IsNullOrWhiteSpace(text)) return;
var dest = ActiveDestination();
if (dest == null) { StatusText.Text = "No upload destination configured."; return; }
if (dest.Id == "builtin-sloershot" && string.IsNullOrWhiteSpace(_settings.ServerUrl)) { StatusText.Text = "Set a share server URL in Settings first."; return; }
var tmp = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "paste-" + DateTime.Now.ToString("yyyyMMdd-HHmmss") + ".txt");
await File.WriteAllTextAsync(tmp, text);
var cfg = _settings.ResolveDestinationConfig(dest);
StatusText.Text = "Uploading text...";
var outcome = await _uploader.UploadFileAsync(cfg, tmp);
try { File.Delete(tmp); } catch { }
if (!outcome.Success) { StatusText.Text = "Text upload failed: " + outcome.Error; return; }
_lastUploadUrl = outcome.Url;
var dp = new DataPackage(); dp.SetText(outcome.Url); Clipboard.SetContent(dp);
StatusText.Text = "Text uploaded - link copied: " + outcome.Url;
}
catch (Exception ex) { StatusText.Text = "Text upload error: " + ex.Message; }
}
private async void OnQrTool(object sender, RoutedEventArgs e)
{
var box = new TextBox { AcceptsReturn = true, Height = 90, MinWidth = 360, TextWrapping = TextWrapping.Wrap, PlaceholderText = "Text or URL to encode", Text = _lastUploadUrl ?? "" };
var dlg = new ContentDialog { Title = "Generate QR", Content = box, PrimaryButtonText = "Generate", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await dlg.ShowAsync() != ContentDialogResult.Primary) return;
var text = box.Text ?? "";
if (!string.IsNullOrWhiteSpace(text)) await ShowQrDialogAsync(text);
}
private void HandleCommandLine()
{
try
{
var args = Environment.GetCommandLineArgs();
if (args.Length >= 2) ProcessCliArgs(args, 1);
SetupSingleInstanceListener();
}
catch { }
}
internal static string ForwardFilePath()
{
var d = System.IO.Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "SloerShot");
try { Directory.CreateDirectory(d); } catch { }
return System.IO.Path.Combine(d, "cli-forward.txt");
}
private void SetupSingleInstanceListener()
{
try
{
var ev = new System.Threading.EventWaitHandle(false, System.Threading.EventResetMode.AutoReset, "SloerShot-Activate-B0B0");
var t = new System.Threading.Thread(() =>
{
while (true)
{
try { ev.WaitOne(); } catch { break; }
string[] fwd;
try { var fp = ForwardFilePath(); fwd = File.Exists(fp) ? File.ReadAllLines(fp) : System.Array.Empty<string>(); } catch { fwd = System.Array.Empty<string>(); }
if (fwd.Length > 0) ProcessCliArgs(fwd, 0);
else { var dq = DispatcherQueue; if (dq != null) dq.TryEnqueue(() => ShowFromTray()); }
}
});
t.IsBackground = true;
t.Start();
}
catch { }
}
private void ProcessCliArgs(string[] args, int startIndex)
{
try
{
var dq = DispatcherQueue;
if (dq == null) return;
dq.TryEnqueue(async () =>
{
try
{
ShowFromTray();
for (int i = startIndex; i < args.Length; i++)
{
var a = args[i];
var lower = a.ToLowerInvariant();
if (lower == "--capture" || lower == "-c") { var mode = (i + 1 < args.Length) ? args[++i].ToLowerInvariant() : "area"; DoCapture(mode); }
else if (lower == "--record" || lower == "-r") { OnToggleRecording(this, new RoutedEventArgs()); }
else if (lower == "--upload" || lower == "-u") { if (i + 1 < args.Length) { await UploadFilePathAsync(args[++i]); } }
else if (File.Exists(a)) { OpenImagePath(a); }
}
}
catch { }
});
}
catch { }
}
private void OpenImagePath(string path)
{
var ext = System.IO.Path.GetExtension(path).ToLowerInvariant();
if (ext != ".png" && ext != ".jpg" && ext != ".jpeg" && ext != ".bmp" && ext != ".gif" && ext != ".webp") return;
_suppressSelection = true; _captures.Insert(0, MakeItem(path)); CaptureCountText.Text = _captures.Count.ToString(); CapturesList.SelectedIndex = 0; _suppressSelection = false;
LoadImage(path);
}
private async Task UploadFilePathAsync(string path)
{
try
{
if (!File.Exists(path)) { StatusText.Text = "File not found: " + path; return; }
var dest = ActiveDestination();
if (dest == null) { StatusText.Text = "No upload destination configured."; return; }
if (dest.Id == "builtin-sloershot" && string.IsNullOrWhiteSpace(_settings.ServerUrl)) { StatusText.Text = "Set a share server URL in Settings first."; return; }
var cfg = _settings.ResolveDestinationConfig(dest);
StatusText.Text = "Uploading " + System.IO.Path.GetFileName(path) + "...";
var outcome = await _uploader.UploadFileAsync(cfg, path);
if (!outcome.Success) { StatusText.Text = "Upload failed: " + outcome.Error; return; }
_lastUploadUrl = outcome.Url;
var dp = new DataPackage(); dp.SetText(outcome.Url); Clipboard.SetContent(dp);
StatusText.Text = "Uploaded - link copied: " + outcome.Url;
}
catch (Exception ex) { StatusText.Text = "Upload error: " + ex.Message; }
}
private async void OnWatermarkImage(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture or open an image first."; return; }
var picker = new Windows.Storage.Pickers.FileOpenPicker();
picker.FileTypeFilter.Add(".png"); picker.FileTypeFilter.Add(".jpg"); picker.FileTypeFilter.Add(".jpeg");
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var mark = await picker.PickSingleFileAsync();
if (mark == null) return;
var mp = mark.Path.Replace("\\", "/");
var op = "{\"op\":\"watermark_image\",\"mark_path\":\"" + mp + "\",\"corner\":3,\"opacity\":0.75,\"margin\":16}";
ApplyFx(op, "Watermark");
}
catch (Exception ex) { StatusText.Text = "Watermark error: " + ex.Message; }
}
private async void OnEffectsStudio(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture or open an image first."; return; }
var previewBase = _lastCapturePath;
var previewOut = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "fxstudio-out-" + Guid.NewGuid().ToString("N") + ".png");
var combo = new ComboBox { MinWidth = 220 };
foreach (var sp in Services.EffectSpec.All) combo.Items.Add(sp.Display);
combo.SelectedIndex = 0;
var s1 = new Slider { Width = 240 }; var s2 = new Slider { Width = 240 }; var s3 = new Slider { Width = 240 };
var l1 = new TextBlock(); var l2 = new TextBlock(); var l3 = new TextBlock();
var img = new Image { Width = 300, Height = 225, Stretch = Stretch.Uniform };
Services.EffectSpec Cur() => Services.EffectSpec.All[combo.SelectedIndex < 0 ? 0 : combo.SelectedIndex];
void Cfg(Slider sl, TextBlock lb, string name, double mn, double mx, double dv)
{
if (string.IsNullOrEmpty(name)) { sl.Visibility = Visibility.Collapsed; lb.Visibility = Visibility.Collapsed; }
else { sl.Visibility = Visibility.Visible; lb.Visibility = Visibility.Visible; lb.Text = name; sl.Minimum = mn; sl.Maximum = mx; sl.StepFrequency = (mx - mn) / 100.0; sl.Value = dv; }
}
void ConfigSliders() { var sp = Cur(); Cfg(s1, l1, sp.P1Name, sp.P1Min, sp.P1Max, sp.P1Def); Cfg(s2, l2, sp.P2Name, sp.P2Min, sp.P2Max, sp.P2Def); Cfg(s3, l3, sp.P3Name, sp.P3Min, sp.P3Max, sp.P3Def); }
bool rendering = false;
async void Render()
{
if (rendering) return;
rendering = true;
try
{
var op = Cur().BuildOp(s1.Value, s2.Value, s3.Value);
var rc = await Task.Run(() => ShotCore.FxApply(previewBase, previewOut, op));
if (rc == 0 && File.Exists(previewOut)) { var bmp = new BitmapImage(); using (var fs = File.OpenRead(previewOut)) { await bmp.SetSourceAsync(fs.AsRandomAccessStream()); } img.Source = bmp; }
}
finally { rendering = false; }
}
combo.SelectionChanged += (s, e2) => { ConfigSliders(); Render(); };
s1.ValueChanged += (s, e2) => Render();
s2.ValueChanged += (s, e2) => Render();
s3.ValueChanged += (s, e2) => Render();
var presetCombo = new ComboBox { MinWidth = 180 };
void RefreshPresets() { presetCombo.Items.Clear(); foreach (var p in _settings.EffectPresets) presetCombo.Items.Add(p.Name); }
RefreshPresets();
var nameBox = new TextBox { PlaceholderText = "Preset name", MinWidth = 140 };
var saveP = new Button { Content = "Save preset" };
saveP.Click += (s, e2) => { var nm = string.IsNullOrWhiteSpace(nameBox.Text) ? Cur().Display : nameBox.Text; _settings.EffectPresets.RemoveAll(p => p.Name == nm); _settings.EffectPresets.Add(new Services.EffectPreset { Name = nm, Key = Cur().Key, P1 = s1.Value, P2 = s2.Value, P3 = s3.Value }); _settings.Save(); RefreshPresets(); StatusText.Text = "Preset saved."; };
var loadP = new Button { Content = "Load" };
loadP.Click += (s, e2) => { if (presetCombo.SelectedIndex < 0 || presetCombo.SelectedIndex >= _settings.EffectPresets.Count) return; var p = _settings.EffectPresets[presetCombo.SelectedIndex]; var idx = Services.EffectSpec.All.FindIndex(x => x.Key == p.Key); if (idx >= 0) { combo.SelectedIndex = idx; ConfigSliders(); s1.Value = p.P1; s2.Value = p.P2; s3.Value = p.P3; Render(); } };
ConfigSliders(); Render();
var left = new StackPanel { Spacing = 6, MinWidth = 260 };
left.Children.Add(combo); left.Children.Add(l1); left.Children.Add(s1); left.Children.Add(l2); left.Children.Add(s2); left.Children.Add(l3); left.Children.Add(s3);
var presetRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 }; presetRow.Children.Add(presetCombo); presetRow.Children.Add(loadP); left.Children.Add(presetRow);
var saveRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 }; saveRow.Children.Add(nameBox); saveRow.Children.Add(saveP); left.Children.Add(saveRow);
var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 16 }; row.Children.Add(left); row.Children.Add(new Border { Child = img, Width = 300, Height = 225 });
var dlg = new ContentDialog { Title = "Effects studio", Content = row, PrimaryButtonText = "Apply", CloseButtonText = "Close", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
var r = await dlg.ShowAsync();
if (r == ContentDialogResult.Primary) { ApplyFx(Cur().BuildOp(s1.Value, s2.Value, s3.Value), Cur().Display); }
try { File.Delete(previewOut); } catch { }
}
catch (Exception ex) { StatusText.Text = "Effects studio error: " + ex.Message; }
}
private void RunWorkflow(int index)
{
if (index < 0 || index >= _settings.Workflows.Count) return;
var wf = _settings.Workflows[index];
if (wf.Mode == "record") { OnToggleRecording(this, new RoutedEventArgs()); return; }
_pendingWorkflow = wf;
DoCapture(wf.Mode);
}
private async void OnWorkflows(object sender, RoutedEventArgs e)
{
var keyNames = new List<string> { "None" };
var keyVks = new List<uint> { 0 };
for (uint vk = 0x41; vk <= 0x5A; vk++) { keyNames.Add(((char)vk).ToString()); keyVks.Add(vk); }
for (uint vk = 0x30; vk <= 0x39; vk++) { keyNames.Add(((char)vk).ToString()); keyVks.Add(vk); }
for (int fi = 1; fi <= 12; fi++) { keyNames.Add("F" + fi); keyVks.Add((uint)(0x6F + fi)); }
string ModeFromIndex(int idx) => idx == 1 ? "window" : (idx == 2 ? "full" : (idx == 3 ? "record" : "area"));
int IndexFromMode(string mm) => mm == "window" ? 1 : (mm == "full" ? 2 : (mm == "record" ? 3 : 0));
var list = new ListView { SelectionMode = ListViewSelectionMode.Single, MaxHeight = 170, MinWidth = 460 };
var nameBox = new TextBox { PlaceholderText = "Name", MinWidth = 200 };
var modeCombo = new ComboBox { MinWidth = 150 };
modeCombo.Items.Add("Area"); modeCombo.Items.Add("Window"); modeCombo.Items.Add("Fullscreen"); modeCombo.Items.Add("Record"); modeCombo.SelectedIndex = 0;
var cCtrl = new CheckBox { Content = "Ctrl", IsChecked = true }; var cShift = new CheckBox { Content = "Shift", IsChecked = true }; var cAlt = new CheckBox { Content = "Alt" }; var cWin = new CheckBox { Content = "Win" };
var keyCombo = new ComboBox { MinWidth = 90 }; foreach (var kn in keyNames) keyCombo.Items.Add(kn); keyCombo.SelectedIndex = 0;
var tCopy = new CheckBox { Content = "Auto-copy" }; var tUpload = new CheckBox { Content = "Auto-upload" };
string Hk(Workflow w) { if (w.HotkeyVk == 0) return "(no hotkey)"; var parts = new List<string>(); if ((w.HotkeyModifiers & HotkeyService.ModControl) != 0) parts.Add("Ctrl"); if ((w.HotkeyModifiers & HotkeyService.ModShift) != 0) parts.Add("Shift"); if ((w.HotkeyModifiers & HotkeyService.ModAlt) != 0) parts.Add("Alt"); if ((w.HotkeyModifiers & HotkeyService.ModWin) != 0) parts.Add("Win"); var ki = keyVks.IndexOf(w.HotkeyVk); parts.Add(ki >= 0 ? keyNames[ki] : ("VK" + w.HotkeyVk)); return string.Join("+", parts); }
void Refresh() { list.Items.Clear(); foreach (var w in _settings.Workflows) list.Items.Add(new ListViewItem { Content = w.Name + " [" + w.Mode + "] " + Hk(w), Tag = w.Id }); }
void WriteEditor(Workflow w) { nameBox.Text = w.Name; modeCombo.SelectedIndex = IndexFromMode(w.Mode); cCtrl.IsChecked = (w.HotkeyModifiers & HotkeyService.ModControl) != 0; cShift.IsChecked = (w.HotkeyModifiers & HotkeyService.ModShift) != 0; cAlt.IsChecked = (w.HotkeyModifiers & HotkeyService.ModAlt) != 0; cWin.IsChecked = (w.HotkeyModifiers & HotkeyService.ModWin) != 0; var ki = keyVks.IndexOf(w.HotkeyVk); keyCombo.SelectedIndex = ki >= 0 ? ki : 0; tCopy.IsChecked = w.AutoCopy; tUpload.IsChecked = w.AutoUpload; }
void ReadEditor(Workflow w) { w.Name = string.IsNullOrWhiteSpace(nameBox.Text) ? "Workflow" : nameBox.Text; w.Mode = ModeFromIndex(modeCombo.SelectedIndex); uint mods = 0; if (cCtrl.IsChecked == true) mods |= HotkeyService.ModControl; if (cShift.IsChecked == true) mods |= HotkeyService.ModShift; if (cAlt.IsChecked == true) mods |= HotkeyService.ModAlt; if (cWin.IsChecked == true) mods |= HotkeyService.ModWin; w.HotkeyModifiers = mods; var ki = keyCombo.SelectedIndex; w.HotkeyVk = (ki >= 0 && ki < keyVks.Count) ? keyVks[ki] : 0; w.AutoCopy = tCopy.IsChecked == true; w.AutoUpload = tUpload.IsChecked == true; }
Workflow? Selected() { var id = (list.SelectedItem as ListViewItem)?.Tag as string; return id == null ? null : _settings.Workflows.Find(w => w.Id == id); }
list.SelectionChanged += (s, e2) => { var w = Selected(); if (w != null) WriteEditor(w); };
var addBtn = new Button { Content = "Add new" }; addBtn.Click += (s, e2) => { var w = new Workflow(); ReadEditor(w); _settings.Workflows.Add(w); Refresh(); };
var updBtn = new Button { Content = "Update" }; updBtn.Click += (s, e2) => { var w = Selected(); if (w != null) { ReadEditor(w); Refresh(); } };
var remBtn = new Button { Content = "Remove" }; remBtn.Click += (s, e2) => { var w = Selected(); if (w != null) { _settings.Workflows.Remove(w); Refresh(); } };
Refresh();
var modRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 }; modRow.Children.Add(cCtrl); modRow.Children.Add(cShift); modRow.Children.Add(cAlt); modRow.Children.Add(cWin); modRow.Children.Add(keyCombo);
var togRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12 }; togRow.Children.Add(tCopy); togRow.Children.Add(tUpload);
var btnRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 }; btnRow.Children.Add(addBtn); btnRow.Children.Add(updBtn); btnRow.Children.Add(remBtn);
var panel = new StackPanel { Spacing = 8, MinWidth = 470 };
panel.Children.Add(new TextBlock { Text = "Workflows run a capture in a mode with a global hotkey and optional auto-copy/upload.", TextWrapping = TextWrapping.Wrap });
panel.Children.Add(list); panel.Children.Add(nameBox); panel.Children.Add(modeCombo); panel.Children.Add(modRow); panel.Children.Add(togRow); panel.Children.Add(btnRow);
var dlg = new ContentDialog { Title = "Workflows", Content = new ScrollViewer { Content = panel, MaxHeight = 520 }, PrimaryButtonText = "Done", XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
await dlg.ShowAsync();
_settings.Fixup(); _settings.Save(); RegisterCaptureHotkey(); StatusText.Text = "Workflows updated.";
}
private async void OnColorPicker(object sender, RoutedEventArgs e)
{
var swatch = new Border { Width = 80, Height = 80, CornerRadius = new CornerRadius(8), BorderThickness = new Thickness(1), BorderBrush = new SolidColorBrush(Microsoft.UI.Colors.Gray) };
var hexText = new TextBox { IsReadOnly = true, MinWidth = 160, FontFamily = new Microsoft.UI.Xaml.Media.FontFamily("Consolas") };
var rgbText = new TextBlock();
string currentHex = "#000000";
var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(60) };
timer.Tick += (s, e2) =>
{
try
{
var p = ScreenColor.Cursor();
var (r, g, b) = ScreenColor.PixelAt(p.X, p.Y);
swatch.Background = new SolidColorBrush(Microsoft.UI.ColorHelper.FromArgb(255, r, g, b));
currentHex = "#" + r.ToString("X2") + g.ToString("X2") + b.ToString("X2");
hexText.Text = currentHex;
rgbText.Text = "RGB " + r + ", " + g + ", " + b + " at " + p.X + ", " + p.Y;
}
catch { }
};
timer.Start();
var panel = new StackPanel { Spacing = 10 };
panel.Children.Add(new TextBlock { Text = "Move the mouse over any pixel on screen; the color updates live.", TextWrapping = TextWrapping.Wrap, MaxWidth = 320 });
var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12 };
row.Children.Add(swatch);
var col2 = new StackPanel { Spacing = 6 }; col2.Children.Add(hexText); col2.Children.Add(rgbText); row.Children.Add(col2);
panel.Children.Add(row);
var dlg = new ContentDialog { Title = "Screen color picker", Content = panel, PrimaryButtonText = "Copy hex", CloseButtonText = "Close", XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
var res = await dlg.ShowAsync();
timer.Stop();
if (res == ContentDialogResult.Primary) { var dp = new DataPackage(); dp.SetText(currentHex); Clipboard.SetContent(dp); StatusText.Text = "Color copied: " + currentHex; }
}
private async void OnScreenRuler(object sender, RoutedEventArgs e)
{
var posText = new TextBlock();
var deltaText = new TextBlock();
int ax = 0; int ay = 0; bool hasAnchor = false;
var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(60) };
timer.Tick += (s, e2) =>
{
var p = ScreenColor.Cursor();
posText.Text = "Cursor: " + p.X + ", " + p.Y;
if (hasAnchor) { int dx = p.X - ax; int dy = p.Y - ay; double dist = Math.Sqrt(dx * (double)dx + dy * (double)dy); double ang = Math.Atan2(dy, dx) * 180.0 / Math.PI; deltaText.Text = "dx=" + dx + " dy=" + dy + " dist=" + dist.ToString("0.0") + " angle=" + ang.ToString("0.0"); }
};
timer.Start();
var anchorBtn = new Button { Content = "Set anchor at cursor" };
anchorBtn.Click += (s, e2) => { var p = ScreenColor.Cursor(); ax = p.X; ay = p.Y; hasAnchor = true; };
var panel = new StackPanel { Spacing = 10, MinWidth = 340 };
panel.Children.Add(new TextBlock { Text = "Live cursor position. Set an anchor, then move to measure distance.", TextWrapping = TextWrapping.Wrap });
panel.Children.Add(posText); panel.Children.Add(deltaText); panel.Children.Add(anchorBtn);
var dlg = new ContentDialog { Title = "Screen ruler", Content = panel, CloseButtonText = "Close", XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
await dlg.ShowAsync();
timer.Stop();
}
private async void OnThumbnail(object sender, RoutedEventArgs e)
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture or open an image first."; return; }
var box = new NumberBox { Header = "Max dimension (px)", Value = 320, Minimum = 16, Maximum = 4096, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Inline };
var dlg = new ContentDialog { Title = "Create thumbnail", Content = box, PrimaryButtonText = "Create", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
if (await dlg.ShowAsync() != ContentDialogResult.Primary) return;
int maxDim = Math.Max(16, (int)box.Value);
int w = _imgW; int h = _imgH;
if (w <= 0 || h <= 0) { w = maxDim; h = maxDim; }
double scale = Math.Min(1.0, (double)maxDim / Math.Max(w, h));
int nw = Math.Max(1, (int)(w * scale)); int nh = Math.Max(1, (int)(h * scale));
ApplyFx("{\"op\":\"resize\",\"w\":" + nw + ",\"h\":" + nh + "}", "Thumbnail");
}
private void RunAction(Services.ExternalAction a)
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
try
{
var args = (a.Arguments ?? "").Replace("{input}", _lastCapturePath);
System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo { FileName = a.Program, Arguments = args, UseShellExecute = true });
StatusText.Text = "Ran action: " + a.Name;
}
catch (Exception ex) { StatusText.Text = "Action failed: " + ex.Message; }
}
private async void OnActions(object sender, RoutedEventArgs e)
{
var list = new ListView { SelectionMode = ListViewSelectionMode.Single, MaxHeight = 160, MinWidth = 460 };
var nameBox = new TextBox { PlaceholderText = "Name", MinWidth = 200 };
var progBox = new TextBox { PlaceholderText = "Program (e.g. mspaint.exe)", MinWidth = 300 };
var argsBox = new TextBox { PlaceholderText = "Arguments, use {input} for the file", Text = "\"{input}\"", MinWidth = 300 };
var browse = new Button { Content = "Browse" };
browse.Click += async (s, e2) => { try { var pk = new Windows.Storage.Pickers.FileOpenPicker(); pk.FileTypeFilter.Add(".exe"); pk.FileTypeFilter.Add("*"); var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this); WinRT.Interop.InitializeWithWindow.Initialize(pk, hwnd); var fpk = await pk.PickSingleFileAsync(); if (fpk != null) progBox.Text = fpk.Path; } catch { } };
void Refresh() { list.Items.Clear(); foreach (var a in _settings.ExternalActions) list.Items.Add(new ListViewItem { Content = a.Name + " -> " + a.Program, Tag = a.Id }); }
Services.ExternalAction? Selected() { var id = (list.SelectedItem as ListViewItem)?.Tag as string; return id == null ? null : _settings.ExternalActions.Find(a => a.Id == id); }
void WriteEditor(Services.ExternalAction a) { nameBox.Text = a.Name; progBox.Text = a.Program; argsBox.Text = a.Arguments; }
void ReadEditor(Services.ExternalAction a) { a.Name = string.IsNullOrWhiteSpace(nameBox.Text) ? "Action" : nameBox.Text; a.Program = progBox.Text ?? ""; a.Arguments = argsBox.Text ?? ""; }
list.SelectionChanged += (s, e2) => { var a = Selected(); if (a != null) WriteEditor(a); };
var addBtn = new Button { Content = "Add" }; addBtn.Click += (s, e2) => { var a = new Services.ExternalAction(); ReadEditor(a); _settings.ExternalActions.Add(a); Refresh(); };
var updBtn = new Button { Content = "Update" }; updBtn.Click += (s, e2) => { var a = Selected(); if (a != null) { ReadEditor(a); Refresh(); } };
var remBtn = new Button { Content = "Remove" }; remBtn.Click += (s, e2) => { var a = Selected(); if (a != null) { _settings.ExternalActions.Remove(a); Refresh(); } };
var runBtn = new Button { Content = "Run on current" }; runBtn.Click += (s, e2) => { var a = Selected(); if (a != null) RunAction(a); };
Refresh();
var progRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 }; progRow.Children.Add(progBox); progRow.Children.Add(browse);
var btnRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 }; btnRow.Children.Add(addBtn); btnRow.Children.Add(updBtn); btnRow.Children.Add(remBtn); btnRow.Children.Add(runBtn);
var panel = new StackPanel { Spacing = 8, MinWidth = 470 };
panel.Children.Add(new TextBlock { Text = "External programs run on the current capture. Use {input} for the file path.", TextWrapping = TextWrapping.Wrap });
panel.Children.Add(list); panel.Children.Add(nameBox); panel.Children.Add(progRow); panel.Children.Add(argsBox); panel.Children.Add(btnRow);
var dlg = new ContentDialog { Title = "External actions", Content = new ScrollViewer { Content = panel, MaxHeight = 520 }, PrimaryButtonText = "Done", XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
await dlg.ShowAsync();
_settings.Save(); StatusText.Text = "Actions updated.";
}
private void OnBgPreset(object sender, RoutedEventArgs e) { var tg = (sender as FrameworkElement)?.Tag as string; if (tg != null) { _bgPreset = tg; _bgType = "gradient"; if (BgTypeCombo != null) BgTypeCombo.SelectedIndex = 0; StatusText.Text = "Gradient: " + tg; } }
private void OnBgColorSwatch(object sender, RoutedEventArgs e) { var hex = (sender as FrameworkElement)?.Tag as string; if (hex != null && hex.Length >= 6) { try { _bgColor = (Convert.ToByte(hex.Substring(0, 2), 16), Convert.ToByte(hex.Substring(2, 2), 16), Convert.ToByte(hex.Substring(4, 2), 16)); _bgType = "color"; if (BgTypeCombo != null) BgTypeCombo.SelectedIndex = 1; } catch { } } }
private void OnBgTypeChanged(object sender, SelectionChangedEventArgs e) { if ((sender as ComboBox)?.SelectedItem is ComboBoxItem it && it.Content is string sv) _bgType = sv.ToLowerInvariant(); }
private async void OnApplyBackground(object sender, RoutedEventArgs e)
{
 if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
 var json = BuildFramedJson();
 var dir = System.IO.Path.GetDirectoryName(_lastCapturePath) ?? CapturesFolder();
 var outPath = System.IO.Path.Combine(dir, $"bg-{DateTime.Now:yyyyMMdd-HHmmss}-{++_fxCounter}.png");
 StatusText.Text = "Applying background...";
 int rc = await System.Threading.Tasks.Task.Run(() => ShotCore.BeautifyFramed(_lastCapturePath, outPath, json));
 if (rc != 0 || !File.Exists(outPath)) { StatusText.Text = "Background failed (" + rc + ")."; return; }
 _suppressSelection = true; _captures.Insert(0, MakeItem(outPath)); CaptureCountText.Text = _captures.Count.ToString(); CapturesList.SelectedIndex = 0; _suppressSelection = false;
 LoadImage(outPath); StatusText.Text = "Background applied.";
}
private string BuildFramedJson()
{
 var ci = System.Globalization.CultureInfo.InvariantCulture;
 string bg;
 if (_bgType == "color") bg = "{\"Solid\":{\"r\":" + _bgColor.r + ",\"g\":" + _bgColor.g + ",\"b\":" + _bgColor.b + ",\"a\":255}}";
 else if (_bgType == "transparent") bg = "{\"Solid\":{\"r\":0,\"g\":0,\"b\":0,\"a\":0}}";
 else bg = "{\"Preset\":\"" + _bgPreset + "\"}";
 int padding = (int)(BgPadding?.Value ?? 64);
 double corners = BgCorners?.Value ?? 16;
 bool shadowOn = BgShadow?.IsChecked == true;
 string shadow = shadowOn ? "{\"color\":{\"r\":0,\"g\":0,\"b\":0,\"a\":255},\"blur\":24.0,\"dx\":0.0,\"dy\":16.0,\"opacity\":0.35}" : "null";
 string beautify = "{\"background\":" + bg + ",\"padding\":" + padding + ",\"corner_radius\":" + corners.ToString(ci) + ",\"shadow\":" + shadow + "}";
 var ratio = RatioFromIndex(BgRatio?.SelectedIndex ?? 0);
 int align = BgAlign?.SelectedIndex ?? 4;
 return "{\"beautify\":" + beautify + ",\"aspect_w\":" + ratio.w + ",\"aspect_h\":" + ratio.h + ",\"align\":" + align + "}";
}
private static (int w, int h) RatioFromIndex(int i) => i switch { 1 => (1, 1), 2 => (4, 3), 3 => (3, 2), 4 => (16, 9), 5 => (9, 16), 6 => (3, 4), _ => (0, 0) };
private void OnBackdrop(object sender, RoutedEventArgs e)
{
var key = (sender as FrameworkElement)?.Tag as string;
if (key == null) return;
var json = BuildBeautifyJson(key);
if (json == null) return;
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
var dir = System.IO.Path.GetDirectoryName(_lastCapturePath) ?? CapturesFolder();
var outPath = System.IO.Path.Combine(dir, $"bg-{DateTime.Now:yyyyMMdd-HHmmss}-{++_fxCounter}.png");
if (ShotCore.Beautify(_lastCapturePath, outPath, json) != 0) { StatusText.Text = "Backdrop failed."; return; }
_suppressSelection = true; _captures.Insert(0, MakeItem(outPath)); CaptureCountText.Text = _captures.Count.ToString(); CapturesList.SelectedIndex = 0; _suppressSelection = false;
LoadImage(outPath); StatusText.Text = "Backdrop applied.";
}
catch (Exception ex) { StatusText.Text = "Backdrop failed: " + ex.Message; }
}
private static string? BuildBeautifyJson(string key)
{
string shadow = "{\"color\":{\"r\":0,\"g\":0,\"b\":0,\"a\":255},\"blur\":24.0,\"dx\":0.0,\"dy\":16.0,\"opacity\":0.35}";
switch (key)
{
case "indigo": return "{\"background\":{\"Preset\":\"Indigo\"},\"padding\":64,\"corner_radius\":16.0,\"shadow\":" + shadow + "}";
case "ocean": return "{\"background\":{\"Preset\":\"Ocean\"},\"padding\":64,\"corner_radius\":16.0,\"shadow\":" + shadow + "}";
case "sunset": return "{\"background\":{\"Preset\":\"Sunset\"},\"padding\":64,\"corner_radius\":16.0,\"shadow\":" + shadow + "}";
case "white": return "{\"background\":{\"Solid\":{\"r\":255,\"g\":255,\"b\":255,\"a\":255}},\"padding\":48,\"corner_radius\":12.0,\"shadow\":" + shadow + "}";
case "dark": return "{\"background\":{\"Solid\":{\"r\":24,\"g\":24,\"b\":28,\"a\":255}},\"padding\":48,\"corner_radius\":12.0,\"shadow\":" + shadow + "}";
case "tight": return "{\"background\":{\"Preset\":\"Graphite\"},\"padding\":24,\"corner_radius\":10.0,\"shadow\":null}";
default: return null;
}
}
private void OnExportPdf(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null) { StatusText.Text = "Capture something first."; return; }
var outPdf = System.IO.Path.Combine(_settings.SaveFolder, $"export-{DateTime.Now:yyyyMMdd-HHmmss}.pdf");
var json = System.Text.Json.JsonSerializer.Serialize(new[] { _lastCapturePath });
if (ShotCore.ImagesToPdf(json, outPdf, 90) != 0) { StatusText.Text = "PDF export failed."; return; }
StatusText.Text = "Exported " + System.IO.Path.GetFileName(outPdf);
if (_settings.OpenFolderAfterSave) OnOpenFolder(sender, e);
}
catch (Exception ex) { StatusText.Text = "PDF export failed: " + ex.Message; }
}
private void OnExportAllPdf(object sender, RoutedEventArgs e)
{
try
{
if (_captures.Count == 0) { StatusText.Text = "No captures to export."; return; }
var paths = _captures.Select(it => it.Path).Where(File.Exists).ToArray();
if (paths.Length == 0) { StatusText.Text = "No captures to export."; return; }
var outPdf = System.IO.Path.Combine(_settings.SaveFolder, $"album-{DateTime.Now:yyyyMMdd-HHmmss}.pdf");
var json = System.Text.Json.JsonSerializer.Serialize(paths);
if (ShotCore.ImagesToPdf(json, outPdf, 90) != 0) { StatusText.Text = "PDF export failed."; return; }
StatusText.Text = "Exported " + paths.Length + " pages to " + System.IO.Path.GetFileName(outPdf);
}
catch (Exception ex) { StatusText.Text = "PDF export failed: " + ex.Message; }
}
private async void OnSaveAs(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null) { StatusText.Text = "Capture something first."; return; }
var json = EditorCanvas.DocumentJson();
var picker = new Windows.Storage.Pickers.FileSavePicker();
picker.SuggestedStartLocation = Windows.Storage.Pickers.PickerLocationId.PicturesLibrary;
picker.FileTypeChoices.Add("PNG image", new List<string> { ".png" });
picker.FileTypeChoices.Add("JPEG image", new List<string> { ".jpg" });
picker.FileTypeChoices.Add("HEIC image", new List<string> { ".heic" });
picker.FileTypeChoices.Add("TIFF image", new List<string> { ".tiff" });
picker.FileTypeChoices.Add("BMP image", new List<string> { ".bmp" });
picker.SuggestedFileName = "SloerShot-" + DateTime.Now.ToString("yyyyMMdd-HHmmss");
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var file = await picker.PickSaveFileAsync();
if (file == null) { StatusText.Text = "Save cancelled."; return; }
var tmpPng = System.IO.Path.ChangeExtension(_lastCapturePath, null) + "-saveas.png";
string src = _lastCapturePath;
if (json != null && ShotCore.Export(_lastCapturePath, json, tmpPng, null) == 0) src = tmpPng;
var ext = file.FileType.ToLowerInvariant();
if (ext == ".jpg") { using (var img = System.Drawing.Image.FromFile(src)) { SaveJpeg(img, file.Path, _settings.JpegQuality); } }
else if (ext == ".heic") { await EncodeWinRTAsync(src, file, Windows.Graphics.Imaging.BitmapEncoder.HeifEncoderId); }
else if (ext == ".tiff") { await EncodeWinRTAsync(src, file, Windows.Graphics.Imaging.BitmapEncoder.TiffEncoderId); }
else if (ext == ".bmp") { await EncodeWinRTAsync(src, file, Windows.Graphics.Imaging.BitmapEncoder.BmpEncoderId); }
else { System.IO.File.Copy(src, file.Path, true); }
try { if (src == tmpPng) File.Delete(tmpPng); } catch { }
StatusText.Text = "Saved to " + file.Path;
}
catch (Exception ex) { StatusText.Text = "Save As failed: " + ex.Message; }
}
private void PickColorAt(Point disp)
{
try
{
if (_lastCapturePath == null) return;
var s = _renderScale <= 0 ? 1.0 : _renderScale;
int ix = Math.Clamp((int)Math.Round(disp.X / s), 0, Math.Max(0, _imgW - 1));
int iy = Math.Clamp((int)Math.Round(disp.Y / s), 0, Math.Max(0, _imgH - 1));
using var bmp = new System.Drawing.Bitmap(_lastCapturePath);
var col = bmp.GetPixel(ix, iy);
string hex = string.Format("#{0:X2}{1:X2}{2:X2}", col.R, col.G, col.B);
var dp = new DataPackage(); dp.SetText(hex); Clipboard.SetContent(dp);
StatusText.Text = "Picked " + hex + " (copied).";
}
catch (Exception ex) { StatusText.Text = "Pick failed: " + ex.Message; }
}
private void ApplyStyle()
{
 var ci = System.Globalization.CultureInfo.InvariantCulture;
 string stroke = "{\"r\":" + _styleColor.r + ",\"g\":" + _styleColor.g + ",\"b\":" + _styleColor.b + ",\"a\":255}";
 string fill = _fillEnabled ? stroke : "null";
 string json = "{\"stroke\":" + stroke + ",\"fill\":" + fill
 + ",\"stroke_width\":" + _styleWidth.ToString(ci)
 + ",\"opacity\":" + _styleOpacity.ToString(ci)
 + ",\"arrow_style\":\"" + _arrowStyle + "\""
 + ",\"filled\":" + (_fillEnabled ? "true" : "false")
 + ",\"text_style\":\"" + _textStyle + "\""
 + ",\"highlighter_smart\":" + (_smartHighlighter ? "true" : "false")
 + ",\"pencil_smooth\":true}";
 EditorCanvas.SetStyleJson(json);
}
private void OnOpacityChanged(object sender, RangeBaseValueChangedEventArgs e) { _styleOpacity = e.NewValue; ApplyStyle(); }
private void OnToggleFill(object sender, RoutedEventArgs e) { _fillEnabled = (sender as CheckBox)?.IsChecked == true; ApplyStyle(); StatusText.Text = _fillEnabled ? "Fill on." : "Fill off."; }
private void OnArrowStyle(object sender, SelectionChangedEventArgs e) { if ((sender as ComboBox)?.SelectedItem is ComboBoxItem it && it.Content is string sv) { _arrowStyle = sv; ApplyStyle(); } }
private void OnTextStyle(object sender, SelectionChangedEventArgs e) { if ((sender as ComboBox)?.SelectedItem is ComboBoxItem it && it.Content is string sv) { _textStyle = sv; ApplyStyle(); } }
private void OnSmartHighlighter(object sender, RoutedEventArgs e) { _smartHighlighter = (sender as CheckBox)?.IsChecked == true; ApplyStyle(); }
private void OnPickColor(object sender, RoutedEventArgs e)
{
var hex = (sender as FrameworkElement)?.Tag as string;
if (hex == null || hex.Length < 6) return;
try { byte r = Convert.ToByte(hex.Substring(0, 2), 16); byte g = Convert.ToByte(hex.Substring(2, 2), 16); byte b = Convert.ToByte(hex.Substring(4, 2), 16); _styleColor = (r, g, b); ApplyStyle(); StatusText.Text = "Annotation color set."; } catch { }
}
private void OnPickStroke(object sender, RoutedEventArgs e)
{
var t = (sender as FrameworkElement)?.Tag as string;
if (t != null && double.TryParse(t, out var wv)) { _styleWidth = wv; ApplyStyle(); StatusText.Text = "Annotation thickness set."; }
}
private void OnEffectMenu(object sender, RoutedEventArgs e)
{
var key = (sender as FrameworkElement)?.Tag as string;
if (key == null) return;
if (key == "whitebalance" || key == "autocolor" || key == "sharpen" || key == "deskew") { ApplyImageFunc(key); return; }
if (key == "flatten") { FlattenDocument(); return; }
if (key == "scan") { ScanCode(); return; }
var json = BuildEffectOp(key);
if (json != null) ApplyFx(json, key);
}
private static string? BuildEffectOp(string key)
{
switch (key)
{
case "grayscale": return "{\"op\":\"grayscale\"}";
case "sepia": return "{\"op\":\"sepia\"}";
case "invert": return "{\"op\":\"invert\"}";
case "blur": return "{\"op\":\"blur\",\"sigma\":4.0}";
case "vignette": return "{\"op\":\"vignette\",\"strength\":0.6}";
case "brighten": return "{\"op\":\"brightness\",\"delta\":30}";
case "contrast": return "{\"op\":\"contrast\",\"factor\":1.3}";
case "rotate": return "{\"op\":\"rotate\",\"deg\":90}";
case "flip": return "{\"op\":\"flip\",\"axis\":\"h\"}";
case "border": return "{\"op\":\"border\",\"thickness\":16,\"color\":{\"r\":255,\"g\":255,\"b\":255,\"a\":255}}";
case "pixelate": return "{\"op\":\"pixelate\",\"block\":12}";
case "emboss": return "{\"op\":\"emboss\"}";
case "edge": return "{\"op\":\"edge\"}";
case "posterize": return "{\"op\":\"posterize\",\"levels\":4}";
case "bw": return "{\"op\":\"black_white\",\"threshold\":128}";
case "solarize": return "{\"op\":\"solarize\",\"threshold\":128}";
case "colorize": return "{\"op\":\"colorize\",\"color\":{\"r\":220,\"g\":40,\"b\":40}}";
case "gamma_up": return "{\"op\":\"gamma\",\"gamma\":1.5}";
case "gamma_down": return "{\"op\":\"gamma\",\"gamma\":0.7}";
case "hue": return "{\"op\":\"hue\",\"degrees\":90}";
case "saturate": return "{\"op\":\"saturation\",\"factor\":1.6}";
case "desaturate": return "{\"op\":\"saturation\",\"factor\":0.4}";
case "rgb_split": return "{\"op\":\"rgb_split\",\"offset\":3}";
case "selective_color": return "{\"op\":\"selective_color\",\"hue\":0,\"range\":30}";
case "glow": return "{\"op\":\"glow\",\"sigma\":6,\"intensity\":0.6}";
case "slice": return "{\"op\":\"slice\",\"slices\":8,\"max_shift\":12}";
case "torn_edge": return "{\"op\":\"torn_edge\",\"depth\":12}";
case "wave_edge": return "{\"op\":\"wave_edge\",\"amp\":10,\"period\":20}";
case "reflection": return "{\"op\":\"reflection\",\"frac\":0.4,\"opacity\":0.5}";
case "shadow": return "{\"op\":\"shadow\",\"dx\":10,\"dy\":10,\"sigma\":8,\"color\":{\"r\":0,\"g\":0,\"b\":0}}";
case "polaroid": return "{\"op\":\"polaroid\",\"border\":16,\"bottom\":56}";
case "outline": return "{\"op\":\"outline\",\"thickness\":2,\"color\":{\"r\":255,\"g\":80,\"b\":0}}";
case "replace_white": return "{\"op\":\"replace_color\",\"from\":{\"r\":255,\"g\":255,\"b\":255},\"to\":{\"r\":0,\"g\":0,\"b\":0},\"tol\":40}";
default: return null;
}
}
private void ApplyFx(string opJson, string label)
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
var dir = System.IO.Path.GetDirectoryName(_lastCapturePath) ?? CapturesFolder();
var outPath = System.IO.Path.Combine(dir, $"fx-{DateTime.Now:yyyyMMdd-HHmmss}-{++_fxCounter}.png");
var rc = ShotCore.FxApply(_lastCapturePath, outPath, opJson);
if (rc != 0) { StatusText.Text = $"Effect {label} failed (code {rc})."; return; }
_suppressSelection = true;
_captures.Insert(0, MakeItem(outPath));
CaptureCountText.Text = _captures.Count.ToString();
CapturesList.SelectedIndex = 0;
_suppressSelection = false;
LoadImage(outPath);
StatusText.Text = $"Applied {label}.";
}
catch (Exception ex) { StatusText.Text = "Effect failed: " + ex.Message; }
}
private void ApplyImageFunc(string key)
{
try
{
if (_lastCapturePath == null || !File.Exists(_lastCapturePath)) { StatusText.Text = "Capture something first."; return; }
var dir = System.IO.Path.GetDirectoryName(_lastCapturePath) ?? CapturesFolder();
var outPath = System.IO.Path.Combine(dir, $"fx-{DateTime.Now:yyyyMMdd-HHmmss}-{++_fxCounter}.png");
int rc = key == "whitebalance" ? ShotCore.WhiteBalance(_lastCapturePath, outPath) : key == "autocolor" ? ShotCore.AutoColor(_lastCapturePath, outPath) : key == "deskew" ? ShotCore.Deskew(_lastCapturePath, outPath) : ShotCore.Unsharp(_lastCapturePath, outPath, 2, 1.2f);
if (rc != 0) { StatusText.Text = $"Effect {key} failed (code {rc})."; return; }
_suppressSelection = true;
_captures.Insert(0, MakeItem(outPath));
CaptureCountText.Text = _captures.Count.ToString();
CapturesList.SelectedIndex = 0;
_suppressSelection = false;
LoadImage(outPath);
StatusText.Text = $"Applied {key}.";
}
catch (Exception ex) { StatusText.Text = "Effect failed: " + ex.Message; }
}
private void OnSave(object sender, RoutedEventArgs e)
{
try
{
var json = EditorCanvas.DocumentJson();
if (_lastCapturePath == null || json == null) { StatusText.Text = "Capture something first."; return; }
var basePath = System.IO.Path.ChangeExtension(_lastCapturePath, null) + "-annotated";
var pngPath = basePath + ".png";
var rc = ShotCore.Export(_lastCapturePath, json, pngPath, null);
if (rc != 0) { StatusText.Text = $"Export failed (code {rc})."; return; }
string finalPath = pngPath;
if (_settings.Format == "jpg")
{
try { var jpgPath = basePath + ".jpg"; using (var img = System.Drawing.Image.FromFile(pngPath)) { SaveJpeg(img, jpgPath, _settings.JpegQuality); } File.Delete(pngPath); finalPath = jpgPath; } catch { }
}
StatusText.Text = $"Saved {System.IO.Path.GetFileName(finalPath)}";
if (_settings.OpenFolderAfterSave) OnOpenFolder(sender, e);
}
catch (Exception ex) { StatusText.Text = "Save failed: " + ex.Message; }
}
private static async System.Threading.Tasks.Task EncodeWinRTAsync(string srcPath, StorageFile destFile, Guid encoderId)
{
 var srcFile = await StorageFile.GetFileFromPathAsync(srcPath);
 using var inStream = await srcFile.OpenAsync(FileAccessMode.Read);
 var decoder = await Windows.Graphics.Imaging.BitmapDecoder.CreateAsync(inStream);
 using var swb = await decoder.GetSoftwareBitmapAsync();
 using var conv = Windows.Graphics.Imaging.SoftwareBitmap.Convert(swb, Windows.Graphics.Imaging.BitmapPixelFormat.Bgra8, Windows.Graphics.Imaging.BitmapAlphaMode.Premultiplied);
 using var outStream = await destFile.OpenAsync(FileAccessMode.ReadWrite);
 outStream.Size = 0;
 var encoder = await Windows.Graphics.Imaging.BitmapEncoder.CreateAsync(encoderId, outStream);
 encoder.SetSoftwareBitmap(conv);
 await encoder.FlushAsync();
}
private static void SaveJpeg(System.Drawing.Image img, string path, int quality)
{
var enc = System.Drawing.Imaging.ImageCodecInfo.GetImageEncoders().FirstOrDefault(c => c.FormatID == System.Drawing.Imaging.ImageFormat.Jpeg.Guid);
var ep = new System.Drawing.Imaging.EncoderParameters(1);
ep.Param[0] = new System.Drawing.Imaging.EncoderParameter(System.Drawing.Imaging.Encoder.Quality, (long)quality);
if (enc != null) img.Save(path, enc, ep); else img.Save(path, System.Drawing.Imaging.ImageFormat.Jpeg);
}
private async void OnCopy(object sender, RoutedEventArgs e)
{
try
{
if (_lastCapturePath == null) { StatusText.Text = "Nothing to copy."; return; }
string path = _lastCapturePath;
var json = EditorCanvas.DocumentJson();
if (json != null) { var flat = System.IO.Path.ChangeExtension(_lastCapturePath, null) + "-annotated.png"; if (ShotCore.Export(_lastCapturePath, json, flat, null) == 0) path = flat; }
await CopyImageFileAsync(path);
StatusText.Text = "Copied to clipboard.";
}
catch (Exception ex) { StatusText.Text = "Copy failed: " + ex.Message; }
}
private void OnAccelerator(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
{
var key = sender.Key;
var mods = sender.Modifiers;
bool ctrl = mods.HasFlag(VirtualKeyModifiers.Control);
bool shift = mods.HasFlag(VirtualKeyModifiers.Shift);
if (ctrl && !shift)
{
if (key == VirtualKey.N) { DoCapture("area"); args.Handled = true; return; }
if (key == VirtualKey.C) { OnCopy(sender, new RoutedEventArgs()); args.Handled = true; return; }
if (key == VirtualKey.S) { OnSave(sender, new RoutedEventArgs()); args.Handled = true; return; }
if (key == VirtualKey.Z) { EditorCanvas.Undo(); args.Handled = true; return; }
if (key == VirtualKey.Y) { EditorCanvas.Redo(); args.Handled = true; return; }
}
if (ctrl && shift)
{
if (key == VirtualKey.F) { DoCapture("full"); args.Handled = true; return; }
if (key == VirtualKey.W) { DoCapture("window"); args.Handled = true; return; }
}
if (mods == VirtualKeyModifiers.None)
{
if (key == VirtualKey.Delete) { EditorCanvas.DeleteSelected(); args.Handled = true; return; }
var focused = FocusManager.GetFocusedElement(Content?.XamlRoot) as FrameworkElement;
if (focused is TextBox || focused is PasswordBox) return;
ToggleButton? tool = null;
if (key == VirtualKey.V) tool = ToolSelect;
else if (key == VirtualKey.R) tool = ToolRect;
else if (key == VirtualKey.O) tool = ToolEllipse;
else if (key == VirtualKey.A) tool = ToolArrow;
else if (key == VirtualKey.L) tool = ToolLine;
else if (key == VirtualKey.T) tool = ToolText;
else if (key == VirtualKey.H) tool = ToolMarker;
else if (key == VirtualKey.X) tool = ToolRedact;
if (tool != null && _lastCapturePath != null) { tool.IsChecked = true; SyncToggles(tool); ActivateTool(tool.Tag as string ?? "0"); args.Handled = true; }
}
}
private async void OnOpenSettings(object sender, RoutedEventArgs e)
{
var panel = new StackPanel { Spacing = 4, MinWidth = 460 };
string chosenFolder = _settings.SaveFolder;
var folderCard = new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Save folder", Description = _settings.SaveFolder, HeaderIcon = new FontIcon { Glyph = "\uE838" } };
var browse = new Button { Content = "Browse" };
browse.Click += async (s, e2) => { var f = await PickFolderAsync(); if (f != null) { chosenFolder = f; folderCard.Description = f; } };
folderCard.Content = browse;
panel.Children.Add(new TextBlock { Text = "Capture and save", FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, Margin = new Thickness(2, 2, 0, 2) });
panel.Children.Add(folderCard);
var delayCombo = new ComboBox { MinWidth = 140 };
delayCombo.Items.Add("None"); delayCombo.Items.Add("3 seconds"); delayCombo.Items.Add("5 seconds");
delayCombo.SelectedIndex = _settings.CaptureDelaySeconds >= 5 ? 2 : (_settings.CaptureDelaySeconds >= 3 ? 1 : 0);
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Capture delay", Description = "Wait before grabbing the screen", HeaderIcon = new FontIcon { Glyph = "\uE916" }, Content = delayCombo });
var fmtCombo = new ComboBox { MinWidth = 140 };
fmtCombo.Items.Add("PNG"); fmtCombo.Items.Add("JPG");
fmtCombo.SelectedIndex = _settings.Format == "jpg" ? 1 : 0;
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Save format", HeaderIcon = new FontIcon { Glyph = "\uEB9F" }, Content = fmtCombo });
var hideToggle = new ToggleSwitch { IsOn = _settings.HideWindowDuringCapture };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Hide window during capture", Description = "Avoid capturing SloerShot itself", HeaderIcon = new FontIcon { Glyph = "\uE7B3" }, Content = hideToggle });
var copyToggle = new ToggleSwitch { IsOn = _settings.AutoCopyToClipboard };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Copy to clipboard after capture", HeaderIcon = new FontIcon { Glyph = "\uE8C8" }, Content = copyToggle });
var openToggle = new ToggleSwitch { IsOn = _settings.OpenFolderAfterSave };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Open folder after saving", HeaderIcon = new FontIcon { Glyph = "\uE838" }, Content = openToggle });
var hkToggle = new ToggleSwitch { IsOn = _settings.HotkeyEnabled };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Global capture hotkey", Description = DescribeHotkey(), HeaderIcon = new FontIcon { Glyph = "\uE765" }, Content = hkToggle });
var serverBox = new TextBox { Text = _settings.ServerUrl, PlaceholderText = "https://your-server", MinWidth = 240 };
panel.Children.Add(new TextBlock { Text = "Sharing and uploads", FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, Margin = new Thickness(2, 12, 0, 2) });
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Share server URL", Description = "Backend base URL for cloud share links", HeaderIcon = new FontIcon { Glyph = "\uE753" }, Content = serverBox });
var destBtn = new Button { Content = "Manage..." };
destBtn.Click += async (s, e2) => { await OnManageDestinationsAsync(); };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Upload destinations", Description = "Imgur, custom .sxcu, SloerShot backend", HeaderIcon = new FontIcon { Glyph = "\uE898" }, Content = destBtn });
var afterCapUploadToggle = new ToggleSwitch { IsOn = _settings.AfterCaptureUpload };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Upload after capture", Description = "Automatically upload every new capture to the active destination", HeaderIcon = new FontIcon { Glyph = "\uEB9F" }, Content = afterCapUploadToggle });
var copyUrlToggle = new ToggleSwitch { IsOn = _settings.AfterUploadCopyUrl };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Copy URL after upload", HeaderIcon = new FontIcon { Glyph = "\uE8C8" }, Content = copyUrlToggle });
var openUrlToggle = new ToggleSwitch { IsOn = _settings.AfterUploadOpenUrl };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Open URL after upload", HeaderIcon = new FontIcon { Glyph = "\uE774" }, Content = openUrlToggle });
var qrToggle = new ToggleSwitch { IsOn = _settings.AfterUploadShowQr };
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Show QR after upload", HeaderIcon = new FontIcon { Glyph = "\uE8A4" }, Content = qrToggle });
var shortenCombo = new ComboBox { MinWidth = 160 };
shortenCombo.Items.Add("None"); shortenCombo.Items.Add("is.gd"); shortenCombo.Items.Add("TinyURL");
shortenCombo.SelectedIndex = _settings.UrlShortener == "isgd" ? 1 : (_settings.UrlShortener == "tinyurl" ? 2 : 0);
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "URL shortener", Description = "Shorten the link after upload", HeaderIcon = new FontIcon { Glyph = "\uE71B" }, Content = shortenCombo });
var scroll = new ScrollViewer { Content = panel, VerticalScrollBarVisibility = ScrollBarVisibility.Auto, MaxHeight = 520 };
var dialog = new ContentDialog { Title = "Settings", PrimaryButtonText = "Save", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, Content = scroll, XamlRoot = Content.XamlRoot, RequestedTheme = ElementTheme.Dark };
var res = await dialog.ShowAsync();
if (res == ContentDialogResult.Primary)
{
_settings.SaveFolder = string.IsNullOrWhiteSpace(chosenFolder) ? _settings.SaveFolder : chosenFolder;
_settings.CaptureDelaySeconds = delayCombo.SelectedIndex == 2 ? 5 : (delayCombo.SelectedIndex == 1 ? 3 : 0);
_settings.Format = fmtCombo.SelectedIndex == 1 ? "jpg" : "png";
_settings.HideWindowDuringCapture = hideToggle.IsOn;
_settings.AutoCopyToClipboard = copyToggle.IsOn;
_settings.OpenFolderAfterSave = openToggle.IsOn;
_settings.HotkeyEnabled = hkToggle.IsOn;
_settings.ServerUrl = serverBox.Text ?? "";
_settings.AfterUploadCopyUrl = copyUrlToggle.IsOn;
_settings.AfterUploadOpenUrl = openUrlToggle.IsOn;
_settings.AfterUploadShowQr = qrToggle.IsOn;
_settings.AfterCaptureUpload = afterCapUploadToggle.IsOn;
_settings.UrlShortener = shortenCombo.SelectedIndex == 1 ? "isgd" : (shortenCombo.SelectedIndex == 2 ? "tinyurl" : "none");
_settings.Fixup();
_settings.Save();
RegisterCaptureHotkey();
StatusText.Text = "Settings saved.";
}
}
private async Task<string?> PickFolderAsync()
{
try
{
var picker = new Windows.Storage.Pickers.FolderPicker();
picker.FileTypeFilter.Add("*");
var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
var folder = await picker.PickSingleFolderAsync();
return folder?.Path;
}
catch { return null; }
}
private void SetupTray()
{
 try
 {
 _tray = new TrayIcon();
 _tray.Setup(
 onShow: ShowFromTray,
 onCapture: mode => { var dq = DispatcherQueue; if (dq != null) dq.TryEnqueue(() => DoCapture(mode)); else DoCapture(mode); },
 onRecord: () => { var dq = DispatcherQueue; if (dq != null) dq.TryEnqueue(() => OnToggleRecording(this, new RoutedEventArgs())); },
 onSettings: () => { ShowFromTray(); var dq = DispatcherQueue; if (dq != null) dq.TryEnqueue(() => OnOpenSettings(this, new RoutedEventArgs())); },
 onQuit: () => { _reallyQuit = true; try { _tray?.Dispose(); } catch { } this.Close(); });
 }
 catch { }
}
private void ShowFromTray()
{
 try { this.AppWindow.Show(); } catch { }
 try { this.Activate(); } catch { }
}
private void OnAppWindowClosing(Microsoft.UI.Windowing.AppWindow sender, Microsoft.UI.Windowing.AppWindowClosingEventArgs e)
{
 if (!_reallyQuit) { e.Cancel = true; try { sender.Hide(); } catch { } }
}
private void OnWindowClosed(object sender, WindowEventArgs args)
{
 try { _tray?.Dispose(); } catch { }
 try { _hotkey?.Detach(); } catch { }
 try { _settings.Save(); } catch { }
}
}
