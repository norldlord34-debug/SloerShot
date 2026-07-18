using Microsoft.UI.Xaml;
using System;
using System.IO;
using System.Threading;

namespace SloerShot;

public partial class App : Application
{
 private Window? _window;
 private static Mutex? _mutex;

 public App()
 {
 this.InitializeComponent();
 }

 protected override void OnLaunched(LaunchActivatedEventArgs args)
 {
 bool createdNew;
 try { _mutex = new Mutex(true, "SloerShot-Instance-B0B0", out createdNew); }
 catch { createdNew = true; }
 if (!createdNew)
 {
 try
 {
 var cli = Environment.GetCommandLineArgs();
 var fp = MainWindow.ForwardFilePath();
 if (cli.Length > 1)
 {
 var tail = new string[cli.Length - 1];
 Array.Copy(cli, 1, tail, 0, cli.Length - 1);
 File.WriteAllLines(fp, tail);
 }
 else if (File.Exists(fp)) File.Delete(fp);
 }
 catch { }
 try { using (var ev = EventWaitHandle.OpenExisting("SloerShot-Activate-B0B0")) ev.Set(); } catch { }
 Environment.Exit(0);
 return;
 }
 _window = new MainWindow();
 _window.Activate();
 }
}
