#nullable enable
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
namespace SloerShot.Interop;
/// <summary>Global hotkeys via RegisterHotKey, delivered through a WndProc subclass (comctl32).</summary>
public sealed class HotkeyService
{
public const uint ModAlt = 0x1;
public const uint ModControl = 0x2;
public const uint ModShift = 0x4;
public const uint ModWin = 0x8;
public const uint ModNoRepeat = 0x4000;
private const uint WmHotkey = 0x0312;
private const uint SubclassId = 0xB0B0;
public event Action<int>? HotkeyPressed;
private IntPtr _hwnd = IntPtr.Zero;
private bool _subclassed;
private readonly List<int> _ids = new();
private SubclassProc? _proc;
private delegate IntPtr SubclassProc(IntPtr hWnd, uint uMsg, UIntPtr wParam, IntPtr lParam, UIntPtr uIdSubclass, UIntPtr dwRefData);
[DllImport("user32.dll", SetLastError = true)]
private static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);
[DllImport("user32.dll", SetLastError = true)]
private static extern bool UnregisterHotKey(IntPtr hWnd, int id);
[DllImport("comctl32.dll", SetLastError = true)]
private static extern bool SetWindowSubclass(IntPtr hWnd, SubclassProc pfnSubclass, UIntPtr uIdSubclass, UIntPtr dwRefData);
[DllImport("comctl32.dll")]
private static extern IntPtr DefSubclassProc(IntPtr hWnd, uint uMsg, UIntPtr wParam, IntPtr lParam);
[DllImport("comctl32.dll")]
private static extern bool RemoveWindowSubclass(IntPtr hWnd, SubclassProc pfnSubclass, UIntPtr uIdSubclass);
public void Attach(IntPtr hwnd)
{
_hwnd = hwnd;
if (_subclassed) return;
_proc = new SubclassProc(WndProc);
_subclassed = SetWindowSubclass(hwnd, _proc, new UIntPtr(SubclassId), UIntPtr.Zero);
}
private IntPtr WndProc(IntPtr hWnd, uint uMsg, UIntPtr wParam, IntPtr lParam, UIntPtr uIdSubclass, UIntPtr dwRefData)
{
if (uMsg == WmHotkey)
{
int id = (int)wParam.ToUInt32();
HotkeyPressed?.Invoke(id);
}
return DefSubclassProc(hWnd, uMsg, wParam, lParam);
}
public bool Register(int id, uint mods, uint vk)
{
if (_hwnd == IntPtr.Zero) return false;
UnregisterHotKey(_hwnd, id);
bool ok = RegisterHotKey(_hwnd, id, mods | ModNoRepeat, vk);
if (ok && !_ids.Contains(id)) _ids.Add(id);
return ok;
}
public void Unregister(int id)
{
if (_hwnd != IntPtr.Zero) UnregisterHotKey(_hwnd, id);
_ids.Remove(id);
}
public void Detach()
{
if (_hwnd == IntPtr.Zero) return;
foreach (var id in _ids.ToArray()) UnregisterHotKey(_hwnd, id);
_ids.Clear();
if (_subclassed && _proc != null) { RemoveWindowSubclass(_hwnd, _proc, new UIntPtr(SubclassId)); _subclassed = false; }
}
}
