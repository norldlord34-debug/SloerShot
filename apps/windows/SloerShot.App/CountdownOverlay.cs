#nullable enable
using System;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.Graphics;
using Windows.UI;
namespace SloerShot;
/// <summary>Brief always-on-top countdown shown before a delayed capture.</summary>
public sealed class CountdownOverlay
{
[DllImport("user32.dll")] private static extern int GetSystemMetrics(int index);
private readonly Window _win = new();
private readonly TextBlock _num = new();
public CountdownOverlay()
{
var border = new Border { Background = new SolidColorBrush(Color.FromArgb(214, 20, 20, 24)), CornerRadius = new CornerRadius(28) };
_num.FontSize = 84;
_num.FontWeight = Microsoft.UI.Text.FontWeights.SemiBold;
_num.Foreground = new SolidColorBrush(Colors.White);
_num.HorizontalAlignment = HorizontalAlignment.Center;
_num.VerticalAlignment = VerticalAlignment.Center;
border.Child = _num;
var grid = new Grid { Background = new SolidColorBrush(Colors.Transparent) };
grid.Children.Add(border);
_win.Content = grid;
if (_win.AppWindow.Presenter is OverlappedPresenter p) { p.SetBorderAndTitleBar(false, false); p.IsAlwaysOnTop = true; p.IsResizable = false; p.IsMaximizable = false; p.IsMinimizable = false; }
int sw = GetSystemMetrics(0), sh = GetSystemMetrics(1);
int size = 176;
try { _win.AppWindow.MoveAndResize(new RectInt32((sw - size) / 2, (sh - size) / 2, size, size)); } catch { }
}
public async Task RunAsync(int seconds)
{
try { _win.Activate(); } catch { }
for (int i = seconds; i > 0; i--) { _num.Text = i.ToString(); await Task.Delay(1000); }
try { _win.Close(); } catch { }
}
}
