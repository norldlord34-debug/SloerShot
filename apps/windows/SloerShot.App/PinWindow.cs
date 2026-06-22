#nullable enable
using System;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Graphics;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;
using Windows.Storage.Streams;
namespace SloerShot;
/// <summary>Floating always-on-top pinned screenshot (CleanShot Floating Screenshots): drag to move, context menu for copy/opacity/close.</summary>
public sealed class PinWindow : Window
{
[StructLayout(LayoutKind.Sequential)] private struct POINT { public int X; public int Y; }
[DllImport("user32.dll")] private static extern bool GetCursorPos(out POINT p);
private readonly Image _image = new();
private readonly OverlappedPresenter _presenter;
private readonly string _path;
private bool _locked;
private bool _dragging;
private int _offX;
private int _offY;
public PinWindow(string imagePath, int x, int y)
{
_path = imagePath;
_presenter = OverlappedPresenter.CreateForToolWindow();
_presenter.IsAlwaysOnTop = true;
_presenter.IsResizable = true;
_presenter.SetBorderAndTitleBar(false, false);
AppWindow.SetPresenter(_presenter);
_image.Source = new BitmapImage(new Uri(imagePath));
_image.Stretch = Stretch.Uniform;
_image.PointerPressed += OnPointerPressed;
_image.PointerMoved += OnPointerMoved;
_image.PointerReleased += OnPointerReleased;
_image.DoubleTapped += (s, e) => this.Close();
BuildMenu();
Content = _image;
int w = 480, h = 320;
try { using var img = System.Drawing.Image.FromFile(imagePath); double sc = Math.Min(1.0, Math.Min(720.0 / img.Width, 720.0 / img.Height)); w = Math.Max(80, (int)(img.Width * sc)); h = Math.Max(60, (int)(img.Height * sc)); } catch { }
try { AppWindow.MoveAndResize(new RectInt32(x, y, w, h)); } catch { }
}
private void BuildMenu()
{
var menu = new MenuFlyout();
var copy = new MenuFlyoutItem { Text = "Copy image" }; copy.Click += async (s, e) => await CopyAsync(); menu.Items.Add(copy);
var op100 = new MenuFlyoutItem { Text = "Opacity 100%" }; op100.Click += (s, e) => SetOpacity(1.0); menu.Items.Add(op100);
var op75 = new MenuFlyoutItem { Text = "Opacity 75%" }; op75.Click += (s, e) => SetOpacity(0.75); menu.Items.Add(op75);
var op50 = new MenuFlyoutItem { Text = "Opacity 50%" }; op50.Click += (s, e) => SetOpacity(0.5); menu.Items.Add(op50);
menu.Items.Add(new MenuFlyoutSeparator());
var close = new MenuFlyoutItem { Text = "Close" }; close.Click += (s, e) => this.Close(); menu.Items.Add(close);
_image.ContextFlyout = menu;
}
private void OnPointerPressed(object sender, PointerRoutedEventArgs e)
{
if (_locked) return;
if (GetCursorPos(out var c)) { var p = AppWindow.Position; _offX = c.X - p.X; _offY = c.Y - p.Y; _dragging = true; _image.CapturePointer(e.Pointer); }
}
private void OnPointerMoved(object sender, PointerRoutedEventArgs e)
{
if (!_dragging) return;
if (GetCursorPos(out var c)) AppWindow.Move(new PointInt32(c.X - _offX, c.Y - _offY));
}
private void OnPointerReleased(object sender, PointerRoutedEventArgs e)
{
_dragging = false;
_image.ReleasePointerCapture(e.Pointer);
}
private async Task CopyAsync()
{
try { var file = await StorageFile.GetFileFromPathAsync(_path); var dp = new DataPackage(); dp.SetBitmap(RandomAccessStreamReference.CreateFromFile(file)); Clipboard.SetContent(dp); } catch { }
}
public void SetOpacity(double opacity) => _image.Opacity = Math.Clamp(opacity, 0.1, 1.0);
public void Nudge(int dx, int dy) { var p = AppWindow.Position; AppWindow.Move(new PointInt32(p.X + dx, p.Y + dy)); }
public void SetLocked(bool locked) { _locked = locked; _presenter.IsAlwaysOnTop = true; }
}
