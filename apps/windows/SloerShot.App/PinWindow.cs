using System;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Graphics;

namespace SloerShot;

// Floating screenshot pinned above all windows (CleanShot Floating Screenshots). The pin
// geometry/opacity/lock state lives in the tested shotcore pin module; this is the thin
// native always-on-top window that shows the image and applies nudge/opacity/lock.
public sealed class PinWindow : Window
{
 private readonly Image _image = new();
 private readonly OverlappedPresenter _presenter;

 public PinWindow(string imagePath, int x, int y)
 {
 _presenter = OverlappedPresenter.CreateForToolWindow();
 _presenter.IsAlwaysOnTop = true;
 _presenter.IsResizable = true;
 _presenter.SetBorderAndTitleBar(false, false);
 AppWindow.SetPresenter(_presenter);
 _image.Source = new BitmapImage(new Uri(imagePath));
 Content = _image;
 AppWindow.Move(new PointInt32(x, y));
 }

 // Adjust opacity (CleanShot pin opacity slider).
 public void SetOpacity(double opacity) => _image.Opacity = Math.Clamp(opacity, 0.1, 1.0);

 // Precise on-screen positioning with arrow keys.
 public void Nudge(int dx, int dy)
 {
 var p = AppWindow.Position;
 AppWindow.Move(new PointInt32(p.X + dx, p.Y + dy));
 }

 // Lock mode: click-through so you can interact with apps beneath the pin.
 public void SetLocked(bool locked)
 {
 _presenter.IsAlwaysOnTop = true;
 _image.IsHitTestVisible = !locked;
 }
}
