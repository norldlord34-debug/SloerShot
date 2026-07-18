#nullable enable
using System;
namespace SloerShot.Services;

// A user-defined external program run on a capture. {input} in Arguments is replaced by the file path.
public sealed class ExternalAction
{
 public string Id { get; set; } = Guid.NewGuid().ToString("N");
 public string Name { get; set; } = "Action";
 public string Program { get; set; } = "";
 public string Arguments { get; set; } = "\"{input}\"";
}
