#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;
namespace SloerShot.Services;

public sealed class EffectPreset
{
 public string Name { get; set; } = "";
 public string Key { get; set; } = "";
 public double P1 { get; set; }
 public double P2 { get; set; }
 public double P3 { get; set; }
}

// Describes an effect and its adjustable parameters for the Effects studio (live preview + presets).
public sealed class EffectSpec
{
 public string Key = "";
 public string Display = "";
 public string P1Name = ""; public double P1Min; public double P1Max = 1; public double P1Def;
 public string P2Name = ""; public double P2Min; public double P2Max = 1; public double P2Def;
 public string P3Name = ""; public double P3Min; public double P3Max = 1; public double P3Def;
 public string Extra = "";

 private static string Fmt(double d) => d.ToString("0.###", CultureInfo.InvariantCulture);

 public string BuildOp(double p1, double p2, double p3)
 {
 var sb = new StringBuilder();
 sb.Append("{\"op\":\"").Append(Key).Append("\"");
 if (P1Name.Length > 0) sb.Append(",\"").Append(P1Name).Append("\":").Append(Fmt(p1));
 if (P2Name.Length > 0) sb.Append(",\"").Append(P2Name).Append("\":").Append(Fmt(p2));
 if (P3Name.Length > 0) sb.Append(",\"").Append(P3Name).Append("\":").Append(Fmt(p3));
 if (Extra.Length > 0) sb.Append(Extra);
 sb.Append("}");
 return sb.ToString();
 }

 public static readonly List<EffectSpec> All = new List<EffectSpec>
 {
 new EffectSpec { Key="blur", Display="Blur", P1Name="sigma", P1Min=0, P1Max=20, P1Def=4 },
 new EffectSpec { Key="pixelate", Display="Pixelate", P1Name="block", P1Min=2, P1Max=64, P1Def=12 },
 new EffectSpec { Key="gamma", Display="Gamma", P1Name="gamma", P1Min=0.2, P1Max=3, P1Def=1 },
 new EffectSpec { Key="hue", Display="Hue rotate", P1Name="degrees", P1Min=0, P1Max=360, P1Def=90 },
 new EffectSpec { Key="saturation", Display="Saturation", P1Name="factor", P1Min=0, P1Max=3, P1Def=1.5 },
 new EffectSpec { Key="brightness", Display="Brightness", P1Name="delta", P1Min=-100, P1Max=100, P1Def=25 },
 new EffectSpec { Key="contrast", Display="Contrast", P1Name="factor", P1Min=0, P1Max=3, P1Def=1.3 },
 new EffectSpec { Key="vignette", Display="Vignette", P1Name="strength", P1Min=0, P1Max=1, P1Def=0.6 },
 new EffectSpec { Key="posterize", Display="Posterize", P1Name="levels", P1Min=2, P1Max=16, P1Def=4 },
 new EffectSpec { Key="sharpen", Display="Sharpen", P1Name="amount", P1Min=0, P1Max=3, P1Def=1.2 },
 new EffectSpec { Key="glow", Display="Glow", P1Name="sigma", P1Min=0, P1Max=20, P1Def=6, P2Name="intensity", P2Min=0, P2Max=1, P2Def=0.6 },
 new EffectSpec { Key="rgb_split", Display="RGB split", P1Name="offset", P1Min=0, P1Max=20, P1Def=3 },
 new EffectSpec { Key="reflection", Display="Reflection", P1Name="frac", P1Min=0.1, P1Max=1, P1Def=0.4, P2Name="opacity", P2Min=0, P2Max=1, P2Def=0.5 },
 new EffectSpec { Key="polaroid", Display="Polaroid frame", P1Name="border", P1Min=4, P1Max=40, P1Def=16, P2Name="bottom", P2Min=8, P2Max=100, P2Def=56 },
 new EffectSpec { Key="outline", Display="Outline", P1Name="thickness", P1Min=1, P1Max=8, P1Def=2, Extra=",\"color\":{\"r\":255,\"g\":80,\"b\":0}" },
 new EffectSpec { Key="shadow", Display="Drop shadow", P1Name="dx", P1Min=-30, P1Max=30, P1Def=10, P2Name="dy", P2Min=-30, P2Max=30, P2Def=10, P3Name="sigma", P3Min=0, P3Max=20, P3Def=8, Extra=",\"color\":{\"r\":0,\"g\":0,\"b\":0}" },
 };
}
