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
/// <summary>Fullscreen CleanShot-style eyedropper over a frozen screenshot with an 8x magnifier loupe and live hex/RGB readout. Click to pick, Esc to cancel.</summary>
public sealed class ColorPickerOverlay
{
private readonly Window _win = new();
private readonly Grid _root = new();
private readonly Canvas _layer = new();
private readonly Image _frozen = new();
private readonly WinShapes.Line _crossV = new();
private readonly WinShapes.Line _crossH = new();
private readonly Border _hint = new();
private readonly TextBlock _hintText = new();
private TaskCompletionSource<string?>? _tcs;
private bool _done;
private int _imgW;
private int _imgH;
private System.Drawing.Bitmap? _sample;
private readonly Border _loupe = new();
private readonly Canvas _loupeInner = new();
private readonly Image _loupeImg = new();
private readonly TranslateTransform _loupeTx = new();
private readonly WinShapes.Line _loupeCV = new();
private readonly WinShapes.Line _loupeCH = new();
private readonly Border _readout = new();
private readonly StackPanel _readoutPanel = new();
private readonly Border _swatch = new();
private readonly TextBlock _hexText = new();
private readonly TextBlock _rgbText = new();
private string _currentHex = "#000000";
private const double LoupeSize = 132;
private const double LoupeZoom = 8;
public ColorPickerOverlay()
{
_frozen.Stretch = Stretch.Fill;
_crossV.Stroke = new SolidColorBrush(Color.FromArgb(150, 61, 126, 255));
_crossV.StrokeThickness = 1;
_crossH.Stroke = new SolidColorBrush(Color.FromArgb(150, 61, 126, 255));
_crossH.StrokeThickness = 1;
_hint.Background = new SolidColorBrush(Color.FromArgb(205, 20, 20, 24));
_hint.CornerRadius = new CornerRadius(8);
_hint.Padding = new Thickness(12, 6, 12, 6);
_hint.HorizontalAlignment = HorizontalAlignment.Center;
_hint.VerticalAlignment = VerticalAlignment.Top;
_hint.Margin = new Thickness(0, 24, 0, 0);
_hintText.Text = "Move over any pixel, click to copy its color. Esc to cancel.";
_hintText.Foreground = new SolidColorBrush(Colors.White);
_hintText.FontSize = 13;
_hint.Child = _hintText;
_layer.Background = new SolidColorBrush(Color.FromArgb(1, 0, 0, 0));
_layer.Children.Add(_crossV);
_layer.Children.Add(_crossH);
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
_swatch.Width = 34; _swatch.Height = 34; _swatch.CornerRadius = new CornerRadius(5);
_swatch.BorderBrush = new SolidColorBrush(Color.FromArgb(200, 255, 255, 255)); _swatch.BorderThickness = new Thickness(1);
_hexText.Foreground = new SolidColorBrush(Colors.White); _hexText.FontSize = 14; _hexText.FontFamily = new FontFamily("Consolas");
_rgbText.Foreground = new SolidColorBrush(Color.FromArgb(210, 255, 255, 255)); _rgbText.FontSize = 11;
var textCol = new StackPanel { Spacing = 2 }; textCol.Children.Add(_hexText); textCol.Children.Add(_rgbText);
_readoutPanel.Orientation = Orientation.Horizontal; _readoutPanel.Spacing = 10;
_readoutPanel.Children.Add(_swatch); _readoutPanel.Children.Add(textCol);
_readout.Background = new SolidColorBrush(Color.FromArgb(225, 20, 20, 24));
_readout.CornerRadius = new CornerRadius(8);
_readout.Padding = new Thickness(10, 8, 12, 8);
_readout.Child = _readoutPanel;
_readout.Visibility = Visibility.Collapsed;
_layer.Children.Add(_readout);
_root.Children.Add(_frozen);
_root.Children.Add(_layer);
_root.Children.Add(_hint);
_layer.PointerMoved += OnMove;
_layer.PointerPressed += OnDown;
_win.Content = _root;
_root.Loaded += (s, e) => InitSizes();
_root.SizeChanged += (s, e) => InitSizes();
_win.Closed += (s, e) => Finish(null);
var esc = new KeyboardAccelerator { Key = Windows.System.VirtualKey.Escape };
esc.Invoked += (s, a) => { a.Handled = true; Finish(null); };
_root.KeyboardAccelerators.Add(esc);
}
public Task<string?> PickAsync(string frozenPath, int vx, int vy, int vw, int vh, int imgW, int imgH)
{
_imgW = imgW; _imgH = imgH;
_tcs = new TaskCompletionSource<string?>();
try { var bmp = new BitmapImage(); bmp.UriSource = new Uri(frozenPath); _frozen.Source = bmp; _loupeImg.Source = bmp; } catch { }
try { _sample = new System.Drawing.Bitmap(frozenPath); } catch { _sample = null; }
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
SampleAt(p);
_readout.Visibility = Visibility.Visible;
Canvas.SetLeft(_readout, lx); Canvas.SetTop(_readout, ly + LoupeSize + 8);
}
private void SampleAt(Point p)
{
if (_sample == null) return;
double sx = _root.ActualWidth > 0 ? _imgW / _root.ActualWidth : 1.0;
double sy = _root.ActualHeight > 0 ? _imgH / _root.ActualHeight : 1.0;
int ix = Math.Clamp((int)Math.Round(p.X * sx), 0, Math.Max(0, _imgW - 1));
int iy = Math.Clamp((int)Math.Round(p.Y * sy), 0, Math.Max(0, _imgH - 1));
try
{
var c = _sample.GetPixel(ix, iy);
_currentHex = "#" + c.R.ToString("X2") + c.G.ToString("X2") + c.B.ToString("X2");
_swatch.Background = new SolidColorBrush(Color.FromArgb(255, c.R, c.G, c.B));
_hexText.Text = _currentHex;
_rgbText.Text = "RGB " + c.R + ", " + c.G + ", " + c.B + "  |  " + ix + ", " + iy;
}
catch { }
}
private void OnDown(object sender, PointerRoutedEventArgs e)
{
SampleAt(e.GetCurrentPoint(_layer).Position);
Finish(_currentHex);
}
private void Finish(string? result)
{
if (_done) return;
_done = true;
var tcs = _tcs;
try { _sample?.Dispose(); } catch { }
_sample = null;
try { _win.Close(); } catch { }
tcs?.TrySetResult(result);
}
}
