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
_hotkey.Unregister(1); _hotkey.Unregister(2); _hotkey.Unregister(3); _hotkey.Unregister(4); _hotkey.Unregister(5);
if (_settings.HotkeyEnabled)
{
_hotkey.Register(1, _settings.HotkeyModifiers, _settings.HotkeyVk);
const uint cs = HotkeyService.ModControl | HotkeyService.ModShift;
_hotkey.Register(2, cs, 0x34);
_hotkey.Register(3, cs, 0x35);
_hotkey.Register(4, cs, 0x36);
_hotkey.Register(5, cs, 0x32);
}
}
UpdateHotkeyHint();
}
private void OnHotkey(int id)
{
var dq = DispatcherQueue;
Action act = id switch
{
2 => () => DoCapture("area"),
3 => () => DoCapture("window"),
4 => () => DoCapture("full"),
5 => () => OnToggleRecording(this, new RoutedEventArgs()),
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
var dp = new DataPackage(); dp.SetText(outcome.Url); Clipboard.SetContent(dp);
StatusText.Text = "Uploaded to " + dest.Name + " - link copied: " + outcome.Url;
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
var imgurBox = new TextBox { Text = _settings.ImgurClientId, PlaceholderText = "Imgur Client ID (needed for Imgur)", MinWidth = 470 };
var panel = new StackPanel { Spacing = 8, MinWidth = 480 };
panel.Children.Add(new TextBlock { Text = "ShareX-compatible upload destinations. Use %SERVER% for your backend URL.", TextWrapping = TextWrapping.Wrap });
panel.Children.Add(list);
panel.Children.Add(btnRow);
panel.Children.Add(addBox);
panel.Children.Add(addBtn);
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
panel.Children.Add(new CommunityToolkit.WinUI.Controls.SettingsCard { Header = "Share server URL", Description = "Backend base URL for cloud share links", HeaderIcon = new FontIcon { Glyph = "\uE753" }, Content = serverBox });
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
