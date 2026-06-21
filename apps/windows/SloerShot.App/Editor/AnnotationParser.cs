#nullable enable
using System.Collections.Generic;
using System.Text.Json;

namespace SloerShot.Editor;

public enum ShapeType { Line, Arrow, Rectangle, Ellipse, Freehand, Text, Counter, Highlighter, Redact }

public readonly record struct ColorSpec(byte R, byte G, byte B, byte A);

/// A flat, UI-agnostic description of one shape to draw. Produced by parsing the
/// core render_json; the WinUI canvas turns these into XAML shapes.
public sealed class ShapeSpec
{
    public ShapeType Type;
    public double X, Y, W, H;
    public double X1, Y1, X2, Y2;
    public List<(double X, double Y)> Points = new();
    public string Text = "";
    public double FontSize;
    public uint Number;
    public double CornerRadius;
    public ColorSpec Stroke;
    public ColorSpec? Fill;
    public double StrokeWidth = 1.0;
    public double Opacity = 1.0;
    public int Z;
}

/// Parses the core editor/document render_json into an ordered list of shape specs.
public static class AnnotationParser
{
    public static List<ShapeSpec> Parse(string? json)
    {
        var list = new List<ShapeSpec>();
        if (string.IsNullOrEmpty(json)) return list;
        using var doc = JsonDocument.Parse(json);
        if (!doc.RootElement.TryGetProperty("annotations", out var anns) || anns.ValueKind != JsonValueKind.Array)
            return list;

        foreach (var ann in anns.EnumerateArray())
        {
            if (ann.TryGetProperty("hidden", out var hid) && hid.ValueKind == JsonValueKind.True)
                continue;
            var spec = new ShapeSpec();
            if (ann.TryGetProperty("z", out var z)) spec.Z = z.GetInt32();
            if (ann.TryGetProperty("style", out var style))
            {
                spec.Stroke = ReadColor(style.GetProperty("stroke"));
                if (style.TryGetProperty("fill", out var fill) && fill.ValueKind == JsonValueKind.Object)
                    spec.Fill = ReadColor(fill);
                if (style.TryGetProperty("stroke_width", out var sw)) spec.StrokeWidth = sw.GetDouble();
                if (style.TryGetProperty("opacity", out var op)) spec.Opacity = op.GetDouble();
            }
            if (!ann.TryGetProperty("kind", out var kind) || kind.ValueKind != JsonValueKind.Object)
                continue;
            string? variant = null;
            JsonElement data = default;
            foreach (var prop in kind.EnumerateObject()) { variant = prop.Name; data = prop.Value; break; }
            if (variant == null) continue;

            switch (variant)
            {
                case "Rectangle":
                    ReadRect(data.GetProperty("rect"), spec);
                    spec.Type = ShapeType.Rectangle;
                    if (data.TryGetProperty("corner_radius", out var cr)) spec.CornerRadius = cr.GetDouble();
                    break;
                case "Ellipse":
                    ReadRect(data.GetProperty("rect"), spec);
                    spec.Type = ShapeType.Ellipse;
                    break;
                case "Highlighter":
                    ReadRect(data.GetProperty("rect"), spec);
                    spec.Type = ShapeType.Highlighter;
                    break;
                case "Redact":
                    ReadRect(data.GetProperty("rect"), spec);
                    spec.Type = ShapeType.Redact;
                    break;
                case "Line":
                    ReadSeg(data, spec);
                    spec.Type = ShapeType.Line;
                    break;
                case "Arrow":
                    ReadSeg(data, spec);
                    spec.Type = ShapeType.Arrow;
                    break;
                case "Freehand":
                    foreach (var p in data.GetProperty("points").EnumerateArray())
                        spec.Points.Add((p.GetProperty("x").GetDouble(), p.GetProperty("y").GetDouble()));
                    spec.Type = ShapeType.Freehand;
                    break;
                case "Text":
                    {
                        var pos = data.GetProperty("position");
                        spec.X = pos.GetProperty("x").GetDouble();
                        spec.Y = pos.GetProperty("y").GetDouble();
                        spec.Text = data.GetProperty("content").GetString() ?? "";
                        spec.FontSize = data.GetProperty("font_size").GetDouble();
                        spec.Type = ShapeType.Text;
                    }
                    break;
                case "Counter":
                    {
                        var ctr = data.GetProperty("center");
                        spec.X = ctr.GetProperty("x").GetDouble();
                        spec.Y = ctr.GetProperty("y").GetDouble();
                        spec.H = data.GetProperty("radius").GetDouble();
                        spec.Number = data.GetProperty("number").GetUInt32();
                        spec.Type = ShapeType.Counter;
                    }
                    break;
                default:
                    continue;
                }
            list.Add(spec);
        }
        list.Sort((a, b) => a.Z.CompareTo(b.Z));
        return list;
    }

    private static ColorSpec ReadColor(JsonElement c) =>
        new ColorSpec(c.GetProperty("r").GetByte(), c.GetProperty("g").GetByte(), c.GetProperty("b").GetByte(), c.GetProperty("a").GetByte());

    private static void ReadRect(JsonElement rect, ShapeSpec spec)
    {
        spec.X = rect.GetProperty("x").GetDouble();
        spec.Y = rect.GetProperty("y").GetDouble();
        spec.W = rect.GetProperty("w").GetDouble();
        spec.H = rect.GetProperty("h").GetDouble();
    }

    private static void ReadSeg(JsonElement data, ShapeSpec spec)
    {
        var from = data.GetProperty("from");
        var to = data.GetProperty("to");
        spec.X1 = from.GetProperty("x").GetDouble();
        spec.Y1 = from.GetProperty("y").GetDouble();
        spec.X2 = to.GetProperty("x").GetDouble();
        spec.Y2 = to.GetProperty("y").GetDouble();
    }
}
