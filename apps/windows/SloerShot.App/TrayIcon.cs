#nullable enable
using System;
using System.Windows.Input;
using H.NotifyIcon;
using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace SloerShot;

// System-tray presence (CleanShot/Snagit style): quick-capture menu + run in background.
public sealed class TrayIcon : IDisposable
{
 private TaskbarIcon? _icon;

 public void Setup(Action onShow, Action<string> onCapture, Action onSettings, Action onQuit)
 {
 var menu = new MenuFlyout();
 void AddItem(string text, Action act) { var mi = new MenuFlyoutItem { Text = text }; mi.Click += (_, _) => act(); menu.Items.Add(mi); }
 AddItem("Capture Area", () => onCapture("area"));
 AddItem("Capture Window", () => onCapture("window"));
 AddItem("Capture Fullscreen", () => onCapture("full"));
 AddItem("Scrolling Capture", () => onCapture("scroll"));
 menu.Items.Add(new MenuFlyoutSeparator());
 AddItem("Open SloerShot", onShow);
 AddItem("Settings", onSettings);
 menu.Items.Add(new MenuFlyoutSeparator());
 AddItem("Quit SloerShot", onQuit);

 _icon = new TaskbarIcon
 {
 ToolTipText = "SloerShot - click to open",
 IconSource = new GeneratedIconSource
 {
 Text = "S",
 FontSize = 28,
 Foreground = new SolidColorBrush(Colors.White),
 Background = new SolidColorBrush(Color.FromArgb(255, 0x3D, 0x7E, 0xFF)),
 CornerRadius = new CornerRadius(6),
 },
 ContextMenuMode = ContextMenuMode.SecondWindow,
 ContextFlyout = menu,
 NoLeftClickDelay = true,
 LeftClickCommand = new RelayCommand(onShow),
 };
 _icon.ForceCreate();
 }

 public void Dispose() { try { _icon?.Dispose(); } catch { } _icon = null; }
}

internal sealed class RelayCommand : ICommand
{
 private readonly Action _action;
 public RelayCommand(Action action) { _action = action; }
 public event EventHandler? CanExecuteChanged { add { } remove { } }
 public bool CanExecute(object? parameter) => true;
 public void Execute(object? parameter) => _action();
}
