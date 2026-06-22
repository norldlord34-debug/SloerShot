#nullable enable
using System;
using System.Threading.Tasks;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Foundation;
using Windows.UI;
using WinShapes = Microsoft.UI.Xaml.Shapes;
namespace SloerShot.Capture;
public readonly record struct PixelRect(int X, int Y, int W, int H);
/// <summary>Fullscreen CleanShot-style region picker over a frozen screenshot.</summary>
public sealed class RegionOverlay
{
private readonly Window _win = new();
private readonly Grid _root = new();
private readonly Canvas _layer = new();
private readonly Image _frozen = new();
private readonly WinShapes.Path _dim = new();
private readonly GeometryGroup _dimGeo = new() { FillRule = FillRule.EvenOdd };
private readonly RectangleGeometry _outerGeo = new();
private readonly RectangleGeometry _holeGeo = new();
private readonly WinShapes.Rectangle _selBorder = new();
private readonly WinShapes.Line _crossV = new();
private readonly WinShapes.Line _crossH = new();
private readonly Border _label = new();
private readonly TextBlock _labelText = new();
private readonly Border _hint = new();
private readonly TextBlock _hintText = new();
private TaskCompletionSource<PixelRect?>? _tcs;
private bool _dragging;
private bool _done;
private Point _start;
private int _imgW;
private int _imgH;
private readonly Border _loupe = new();
private readonly Canvas _loupeInner = new();
private readonly Image _loupeImg = new();
private readonly TranslateTransform _loupeTx = new();
private readonly WinShapes.Line _loupeCV = new();
private readonly WinShapes.Line _loupeCH = new();
private const double LoupeSize = 132;
private const double LoupeZoom = 8;
public RegionOverlay()
{
_frozen.Stretch = Stretch.Fill;
_dim.Fill = new SolidColorBrush(Color.FromArgb(140, 0, 0, 0));
_dimGeo.Children.Add(_outerGeo);
_dimGeo.Children.Add(_holeGeo);
_dim.Data = _dimGeo;
_selBorder.Stroke = new SolidColorBrush(Color.FromArgb(255, 61, 126, 255));
_selBorder.StrokeThickness = 1.5;
_selBorder.Visibility = Visibility.Collapsed;
_crossV.Stroke = new SolidColorBrush(Color.FromArgb(110, 255, 255, 255));
_crossV.StrokeThickness = 1;
_crossH.Stroke = new SolidColorBrush(Color.FromArgb(110, 255, 255, 255));
_crossH.StrokeThickness = 1;
_label.Background = new SolidColorBrush(Color.FromArgb(220, 20, 20, 24));
_label.CornerRadius = new CornerRadius(5);
_label.Padding = new Thickness(8, 3, 8, 3);
_label.Visibility = Visibility.Collapsed;
_labelText.Foreground = new SolidColorBrush(Colors.White);
_labelText.FontSize = 12;
_label.Child = _labelText;
_hint.Background = new SolidColorBrush(Color.FromArgb(205, 20, 20, 24));
_hint.CornerRadius = new CornerRadius(8);
_hint.Padding = new Thickness(12, 6, 12, 6);
_hint.HorizontalAlignment = HorizontalAlignment.Center;
_hint.VerticalAlignment = VerticalAlignment.Top;
_hint.Margin = new Thickness(0, 24, 0, 0);
_hintText.Text = "Drag to select a region. Esc to cancel.";
_hintText.Foreground = new SolidColorBrush(Colors.White);
_hintText.FontSize = 13;
_hint.Child = _hintText;
_layer.Background = new SolidColorBrush(Color.FromArgb(1, 0, 0, 0));
_layer.Children.Add(_dim);
_layer.Children.Add(_crossV);
_layer.Children.Add(_crossH);
_layer.Children.Add(_selBorder);
_layer.Children.Add(_label);
_loupeImg.Stretch = Stretch.Fill;
_loupeImg.RenderTransform = _loupeTx;
_loupeInner.Width = LoupeSize; _loupeInner.Height = LoupeSize;
_loupeInner.Clip = new RectangleGeometry { Rect = new Rect(0, 0, LoupeSize, LoupeSize) };
_loupeInner.Children.Add(_loupeImg);
_loupeCV.Stroke = new SolidColorBrush(Color.FromArgb(200, 61, 126, 255)); _loupeCV.StrokeThickness = 1; _loupeCV.X1 = LoupeSize / 2; _loupeCV.X2 = LoupeSize / 2; _loupeCV.Y1 = 0; _loupeCV.Y2 = LoupeSize;
_loupeCH.Stroke = new SolidColorBrush(Color.FromArgb(200, 61, 126, 255)); _loupeCH.StrokeThickness = 1; _loupeCH.Y1 = LoupeSize / 2; _loupeCH.Y2 = LoupeSize / 2; _loupeCH.X1 = 0; _loupeCH.X2 = LoupeSize;
_loupeInner.Children.Add(_loupeCV); _loupeInner.Children.Add(_loupeCH);
_loupe.Width = LoupeSize; _loupe.Height = LoupeSize;
_loupe.CornerRadius = new CornerRadius(8);
_loupe.BorderBrush = new SolidColorBrush(Color.FromArgb(230, 255, 255, 255));
_loupe.BorderThickness = new Thickness(2);
_loupe.Child = _loupeInner;
_loupe.Visibility = Visibility.Collapsed;
_layer.Children.Add(_loupe);
_root.Children.Add(_frozen);
_root.Children.Add(_layer);
_root.Children.Add(_hint);
_win.Content = _root;
_layer.PointerPressed += OnDown;
_layer.PointerMoved += OnMove;
_layer.PointerReleased += OnUp;
_root.Loaded += (s, e) => InitSizes();
_root.SizeChanged += (s, e) => InitSizes();
_win.Closed += (s, e) => Finish(null);
var esc = new KeyboardAccelerator { Key = Windows.System.VirtualKey.Escape };
esc.Invoked += (s, a) => { a.Handled = true; Finish(null); };
_root.KeyboardAccelerators.Add(esc);
}
public Task<PixelRect?> PickAsync(string frozenPath, int vx, int vy, int vw, int vh, int imgW, int imgH)
{
_imgW = imgW; _imgH = imgH;
_tcs = new TaskCompletionSource<PixelRect?>();
try { var bmp = new BitmapImage(); bmp.UriSource = new Uri(frozenPath); _frozen.Source = bmp; _loupeImg.Source = bmp; } catch { }
var appWin = _win.AppWindow;
if (appWin.Presenter is OverlappedPresenter p) { p.SetBorderAndTitleBar(false, false); p.IsAlwaysOnTop = true; p.IsResizable = false; p.IsMaximizable = false; p.IsMinimizable = false; }
try { appWin.MoveAndResize(new Windows.Graphics.RectInt32(vx, vy, vw, vh)); } catch { }
_win.Activate();
return _tcs.Task;
}
private void InitSizes()
{
double w = _root.ActualWidth, h = _root.ActualHeight;
if (w < 1 || h < 1) return;
_outerGeo.Rect = new Rect(0, 0, w, h);
_crossV.Y1 = 0; _crossV.Y2 = h;
_crossH.X1 = 0; _crossH.X2 = w;
_loupeImg.Width = w * LoupeZoom; _loupeImg.Height = h * LoupeZoom;
}
private void OnDown(object sender, PointerRoutedEventArgs e)
{
_start = e.GetCurrentPoint(_layer).Position;
_dragging = true;
_layer.CapturePointer(e.Pointer);
_hint.Visibility = Visibility.Collapsed;
_selBorder.Visibility = Visibility.Visible;
_label.Visibility = Visibility.Visible;
UpdateSelection(_start);
}
private void OnMove(object sender, PointerRoutedEventArgs e)
{
var p = e.GetCurrentPoint(_layer).Position;
_crossV.X1 = p.X; _crossV.X2 = p.X;
_crossH.Y1 = p.Y; _crossH.Y2 = p.Y;
_loupe.Visibility = Visibility.Visible;
_loupeTx.X = LoupeSize / 2 - p.X * LoupeZoom;
_loupeTx.Y = LoupeSize / 2 - p.Y * LoupeZoom;
double lx = p.X + 18, ly = p.Y + 18;
if (lx + LoupeSize > _root.ActualWidth) lx = p.X - 18 - LoupeSize;
if (ly + LoupeSize > _root.ActualHeight) ly = p.Y - 18 - LoupeSize;
Canvas.SetLeft(_loupe, lx); Canvas.SetTop(_loupe, ly);
if (_dragging) UpdateSelection(p);
}
private void UpdateSelection(Point p)
{
double x = Math.Min(p.X, _start.X), y = Math.Min(p.Y, _start.Y);
double w = Math.Abs(p.X - _start.X), h = Math.Abs(p.Y - _start.Y);
_holeGeo.Rect = new Rect(x, y, w, h);
Canvas.SetLeft(_selBorder, x); Canvas.SetTop(_selBorder, y);
_selBorder.Width = w; _selBorder.Height = h;
double sx = _root.ActualWidth > 0 ? _imgW / _root.ActualWidth : 1.0;
double sy = _root.ActualHeight > 0 ? _imgH / _root.ActualHeight : 1.0;
int iw = (int)Math.Round(w * sx), ih = (int)Math.Round(h * sy);
_labelText.Text = iw + " x " + ih;
double ly = y - 26; if (ly < 4) ly = y + 6;
Canvas.SetLeft(_label, x); Canvas.SetTop(_label, ly);
}
private void OnUp(object sender, PointerRoutedEventArgs e)
{
if (!_dragging) return;
_dragging = false;
_layer.ReleasePointerCapture(e.Pointer);
var rect = _holeGeo.Rect;
double sx = _root.ActualWidth > 0 ? _imgW / _root.ActualWidth : 1.0;
double sy = _root.ActualHeight > 0 ? _imgH / _root.ActualHeight : 1.0;
int ix = (int)Math.Round(rect.X * sx);
int iy = (int)Math.Round(rect.Y * sy);
int iw = (int)Math.Round(rect.Width * sx);
int ih = (int)Math.Round(rect.Height * sy);
ix = Math.Clamp(ix, 0, Math.Max(0, _imgW - 1));
iy = Math.Clamp(iy, 0, Math.Max(0, _imgH - 1));
iw = Math.Clamp(iw, 1, _imgW - ix);
ih = Math.Clamp(ih, 1, _imgH - iy);
if (iw < 4 || ih < 4) { Finish(null); return; }
Finish(new PixelRect(ix, iy, iw, ih));
}
private void Finish(PixelRect? result)
{
if (_done) return;
_done = true;
var tcs = _tcs;
try { _win.Close(); } catch { }
tcs?.TrySetResult(result);
}
}
