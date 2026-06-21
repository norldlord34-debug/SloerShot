using System;
using System.Collections.ObjectModel;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using SloerShot.Capture;
using SloerShot.Interop;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Foundation;
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
    
    public MainWindow()
    {
        this.InitializeComponent();
        CoreVersionText.Text = $"core {ShotCore.Version()}";
        CapturesList.ItemsSource = _captures;
        _toolToggles.AddRange(new[] { ToolSelect, ToolRect, ToolEllipse, ToolArrow, ToolLine, ToolText, ToolCounter, ToolMarker, ToolRedact, ToolCrop, ToolSpotlight });
        ToolSelect.IsChecked = true;
        LoadCapturesFolder();
        UpdateEmptyState();
    }
    
    private string CapturesFolder()
    {
        var folder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyPictures), "SloerShot");
        Directory.CreateDirectory(folder);
        return folder;
    }
    
    private void LoadCapturesFolder()
    {
        try
        {
            var files = new DirectoryInfo(CapturesFolder()).GetFiles("*.png").OrderByDescending(f => f.LastWriteTimeUtc).Take(40);
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
    
    private void DoCapture(string mode)
    {
        try
        {
            var outPath = Path.Combine(CapturesFolder(), $"capture-{DateTime.Now:yyyyMMdd-HHmmss}.png");
            var result = FrozenScreenCapture.CaptureVirtualScreen(outPath);
            _suppressSelection = true;
            _captures.Insert(0, MakeItem(result.Path));
            CaptureCountText.Text = _captures.Count.ToString();
            CapturesList.SelectedIndex = 0;
            _suppressSelection = false;
            LoadImage(result.Path);
            if (mode == "area")
            {
                ToolCrop.IsChecked = true;
                SyncToggles(ToolCrop);
                ActivateTool("fx:crop");
            }
            else
            {
                StatusText.Text = $"Captured {result.Width} x {result.Height}. Pick a tool to annotate.";
            }
        }
        catch (Exception ex) { StatusText.Text = "Capture failed: " + ex.Message; }
    }
    
    private void OnCaptureSelected(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressSelection) return;
        if (CapturesList.SelectedItem is CaptureItem item && File.Exists(item.Path)) LoadImage(item.Path);
    }
    
    private void LoadImage(string path)
    {
        try
        {
            int w, h;
            using (var img = System.Drawing.Image.FromFile(path)) { w = img.Width; h = img.Height; }
            _imgW = w; _imgH = h; _lastCapturePath = path;
            double availW = StageScroller.ViewportWidth > 10 ? StageScroller.ViewportWidth - 60 : 900;
            double availH = StageScroller.ViewportHeight > 10 ? StageScroller.ViewportHeight - 120 : 560;
            double scale = Math.Min(Math.Min(availW / w, availH / h), 1.0);
            if (scale <= 0 || double.IsNaN(scale) || double.IsInfinity(scale)) scale = 1.0;
            _renderScale = scale;
            double dw = w * scale, dh = h * scale;
            EditorCanvas.Width = dw; EditorCanvas.Height = dh; EditorCanvas.RenderScale = scale;
            SelectionLayer.Width = dw; SelectionLayer.Height = dh;
            var bmp = new BitmapImage();
            bmp.UriSource = new Uri(path);
            EditorCanvas.Background = new ImageBrush { ImageSource = bmp, Stretch = Stretch.Fill };
            EditorCanvas.NewDocument((uint)w, (uint)h);
            ClearMarquee();
            UpdateEmptyState();
            DimsText.Text = $"{w} x {h} - {(int)Math.Round(scale * 100)}%";
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
            StatusText.Text = _activeFxTool == "crop" ? "Drag a region to crop." : "Drag a region to spotlight.";
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
    
    private void OnEffectMenu(object sender, RoutedEventArgs e)
    {
        var key = (sender as FrameworkElement)?.Tag as string;
        if (key == null) return;
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
            var dir = Path.GetDirectoryName(_lastCapturePath) ?? CapturesFolder();
            var outPath = Path.Combine(dir, $"fx-{DateTime.Now:yyyyMMdd-HHmmss}-{++_fxCounter}.png");
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
    
    private void OnSave(object sender, RoutedEventArgs e)
    {
        try
        {
            var json = EditorCanvas.DocumentJson();
            if (_lastCapturePath == null || json == null) { StatusText.Text = "Capture something first."; return; }
            var outPath = Path.ChangeExtension(_lastCapturePath, null) + "-annotated.png";
            var rc = ShotCore.Export(_lastCapturePath, json, outPath, null);
            StatusText.Text = rc == 0 ? $"Saved {Path.GetFileName(outPath)}" : $"Export failed (code {rc}).";
        }
        catch (Exception ex) { StatusText.Text = "Save failed: " + ex.Message; }
    }
    
    private async void OnCopy(object sender, RoutedEventArgs e)
    {
        try
        {
            if (_lastCapturePath == null) { StatusText.Text = "Nothing to copy."; return; }
            string path = _lastCapturePath;
            var json = EditorCanvas.DocumentJson();
            if (json != null)
            {
                var flat = Path.ChangeExtension(_lastCapturePath, null) + "-annotated.png";
                if (ShotCore.Export(_lastCapturePath, json, flat, null) == 0) path = flat;
            }
            var file = await StorageFile.GetFileFromPathAsync(path);
            var dp = new DataPackage();
            dp.SetBitmap(RandomAccessStreamReference.CreateFromFile(file));
            Clipboard.SetContent(dp);
            StatusText.Text = "Copied to clipboard.";
        }
        catch (Exception ex) { StatusText.Text = "Copy failed: " + ex.Message; }
    }
}
