#nullable enable
using System;
namespace SloerShot.Services;

// A configurable capture workflow with its own global hotkey and post-capture actions.
public sealed class Workflow
{
 public string Id { get; set; } = Guid.NewGuid().ToString("N");
 public string Name { get; set; } = "Workflow";
 public string Mode { get; set; } = "area";
 public uint HotkeyModifiers { get; set; } = 6;
 public uint HotkeyVk { get; set; }
 public bool AutoCopy { get; set; }
 public bool AutoUpload { get; set; }
 public bool Enabled { get; set; } = true;
}
