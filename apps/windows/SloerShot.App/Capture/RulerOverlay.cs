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
/// <summary>Fullscreen CleanShot-style ruler over a frozen screenshot: drag to measure distance/angle in image pixels, with an 8x magnifier loupe. Esc to close.</summary>
public sealed class RulerOverlay
{
private readonly Window _win = new();
private readonly Grid _root = new();
private readonly Canvas _layer = new();
private readonly Image _frozen = new();
private readonly WinShapes.Line _measure = new();
private readonly WinShapes.Ellipse _dotA = new();
private readonly WinShapes.Ellipse _dotB = new();
private readonly WinShapes.Line _crossV = new();
private readonly WinShapes.Line _crossH = new();
private readonly Border _label = new();
private readonly TextBlock _labelText = new();
private readonly Border _hint = new();
private readonly TextBlock _hintText = new();
private TaskCompletionSource<bool>? _tcs;
private bool _done;
private bool _dragging;
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
public RulerOverlay()
{
_frozen.Stretch = Stretch.Fill;
_measure.Stroke = new SolidColorBrush(Color.FromArgb(255, 61, 126, 255));
_measure.StrokeThickness = 2;
_measure.Visibility = Visibility.Collapsed;
_dotA.Width = 8; _dotA.Height = 8; _dotA.Fill = new SolidColorBrush(Color.FromArgb(255, 255, 255, 255)); _dotA.Stroke = new SolidColorBrush(Color.FromArgb(255, 61, 126, 255)); _dotA.StrokeThickness = 1.5; _dotA.Visibility = Visibility.Collapsed;
_dotB.Width = 8; _dotB.Height = 8; _dotB.Fill = new SolidColorBrush(Color.FromArgb(255, 255, 255, 255)); _dotB.Stroke = new SolidColorBrush(Color.FromArgb(255, 61, 126, 255)); _dotB.StrokeThickness = 1.5; _dotB.Visibility = Visibility.Collapsed;
_crossV.Stroke = new SolidColorBrush(Color.FromArgb(110, 255, 255, 255));
_crossV.StrokeThickness = 1;
_crossH.Stroke = new SolidColorBrush(Color.FromArgb(110, 255, 255, 255));
_crossH.StrokeThickness = 1;
_label.Background = new SolidColorBrush(Color.FromArgb(225, 20, 20, 24));
_label.CornerRadius = new CornerRadius(5);
_label.Padding = new Thickness(8, 4, 8, 4);
_label.Visibility = Visibility.Collapsed;
_labelText.Foreground = new SolidColorBrush(Colors.White);
_labelText.FontSize = 12;
_labelText.FontFamily = new FontFamily("Consolas");
_label.Child = _labelText;
_hint.Background = new SolidColorBrush(Color.FromArgb(205, 20, 20, 24));
_hint.CornerRadius = new CornerRadius(8);
_hint.Padding = new Thickness(12, 6, 12, 6);
_hint.HorizontalAlignment = HorizontalAlignment.Center;
_hint.VerticalAlignment = VerticalAlignment.Top;
_hint.Margin = new Thickness(0, 24, 0, 0);
_hintText.Text = "Drag to measure distance and angle. Esc to close.";
_hintText.Foreground = new SolidColorBrush(Colors.White);
_hintText.FontSize = 13;
_hint.Child = _hintText;
_layer.Background = new SolidColorBrush(Color.FromArgb(1, 0, 0, 0));
_layer.Children.Add(_crossV);
_layer.Children.Add(_crossH);
_layer.Children.Add(_measure);
_layer.Children.Add(_dotA);
_layer.Children.Add(_dotB);
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
_layer.PointerPressed += OnDown;
_layer.PointerMoved += OnMove;
_layer.PointerReleased += OnUp;
_win.Content = _root;
_root.Loaded += (s, e) => InitSizes();
_root.SizeChanged += (s, e) => InitSizes();
_win.Closed += (s, e) => Finish();
var esc = new KeyboardAccelerator { Key = Windows.System.VirtualKey.Escape };
esc.Invoked += (s, a) => { a.Handled = true; Finish(); };
_root.KeyboardAccelerators.Add(esc);
}
public Task<bool> RunAsync(string frozenPath, int vx, int vy, int vw, int vh, int imgW, int imgH)
{
_imgW = imgW; _imgH = imgH;
_tcs = new TaskCompletionSource<bool>();
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
_measure.Visibility = Visibility.Visible;
_dotA.Visibility = Visibility.Visible;
_dotB.Visibility = Visibility.Visible;
_label.Visibility = Visibility.Visible;
_measure.X1 = _start.X; _measure.Y1 = _start.Y;
Canvas.SetLeft(_dotA, _start.X - 4); Canvas.SetTop(_dotA, _start.Y - 4);
UpdateMeasure(_start);
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
if (_dragging) UpdateMeasure(p);
}
private void UpdateMeasure(Point p)
{
_measure.X2 = p.X; _measure.Y2 = p.Y;
Canvas.SetLeft(_dotB, p.X - 4); Canvas.SetTop(_dotB, p.Y - 4);
double sx = _root.ActualWidth > 0 ? _imgW / _root.ActualWidth : 1.0;
double sy = _root.ActualHeight > 0 ? _imgH / _root.ActualHeight : 1.0;
double dx = (p.X - _start.X) * sx;
double dy = (p.Y - _start.Y) * sy;
double dist = Math.Sqrt(dx * dx + dy * dy);
double ang = Math.Atan2(dy, dx) * 180.0 / Math.PI;
_labelText.Text = "dx " + ((int)Math.Round(dx)) + "  dy " + ((int)Math.Round(dy)) + "  dist " + dist.ToString("0.0") + "  ang " + ang.ToString("0.0");
double mx = (_start.X + p.X) / 2 + 10;
double my = (_start.Y + p.Y) / 2 + 10;
if (my < 4) my = 4;
Canvas.SetLeft(_label, mx); Canvas.SetTop(_label, my);
}
private void OnUp(object sender, PointerRoutedEventArgs e)
{
if (!_dragging) return;
_dragging = false;
_layer.ReleasePointerCapture(e.Pointer);
}
private void Finish()
{
if (_done) return;
_done = true;
var tcs = _tcs;
try { _win.Close(); } catch { }
tcs?.TrySetResult(true);
}
}
