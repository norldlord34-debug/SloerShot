#nullable enable
using System;
using SloerShot.Interop;
using Microsoft.UI;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.UI;
using WinShapes = Microsoft.UI.Xaml.Shapes;

namespace SloerShot.Editor;

/// <summary>
/// Interactive annotation surface. Forwards pointer events to the shared core
/// editor (via the C ABI handle) and renders the core render_json as XAML shapes.
/// All editing behavior lives in the Rust core; this control is a thin view.
/// </summary>
public sealed class AnnotationCanvas : Canvas
{
    private IntPtr _editor = IntPtr.Zero;
    private bool _drawing;

    /// Display-to-image scale (image coords = display coords / RenderScale).
    public double RenderScale { get; set; } = 1.0;

    public AnnotationCanvas()
    {
        PointerPressed += OnPressed;
        PointerMoved += OnMoved;
        PointerReleased += OnReleased;
        Unloaded += (_, _) => Free();
    }

    /// Start a fresh editor document of the given pixel size.
    public void NewDocument(uint width, uint height)
    {
        Free();
        _editor = ShotCore.EditorNew(width, height);
        Refresh();
    }

    public void SetTool(uint tool)
    {
        if (_editor != IntPtr.Zero) ShotCore.EditorSetTool(_editor, tool);
    }

    public void Undo() { if (_editor != IntPtr.Zero) { ShotCore.EditorUndo(_editor); Refresh(); } }
    public void Redo() { if (_editor != IntPtr.Zero) { ShotCore.EditorRedo(_editor); Refresh(); } }
    public void DeleteSelected() { if (_editor != IntPtr.Zero) { ShotCore.EditorDeleteSelected(_editor); Refresh(); } }
    public string? DocumentJson() => _editor == IntPtr.Zero ? null : ShotCore.EditorDocumentJson(_editor);

    private void Free()
    {
        if (_editor != IntPtr.Zero) { ShotCore.EditorFree(_editor); _editor = IntPtr.Zero; }
    }

    private Point ToImage(PointerRoutedEventArgs e)
    {
        var p = e.GetCurrentPoint(this).Position;
        var s = RenderScale <= 0 ? 1.0 : RenderScale;
        return new Point(p.X / s, p.Y / s);
    }

    private void OnPressed(object sender, PointerRoutedEventArgs e)
    {
        if (_editor == IntPtr.Zero) return;
        var p = ToImage(e);
        ShotCore.EditorPointerDown(_editor, p.X, p.Y);
        _drawing = true;
        CapturePointer(e.Pointer);
        Refresh();
    }

    private void OnMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_editor == IntPtr.Zero || !_drawing) return;
        var p = ToImage(e);
        ShotCore.EditorPointerDrag(_editor, p.X, p.Y);
        Refresh();
    }

    private void OnReleased(object sender, PointerRoutedEventArgs e)
    {
        if (_editor == IntPtr.Zero || !_drawing) return;
        var p = ToImage(e);
        ShotCore.EditorPointerUp(_editor, p.X, p.Y);
        _drawing = false;
        ReleasePointerCapture(e.Pointer);
        Refresh();
    }

    /// Rebuild the visual tree from the core render_json (committed + live preview).
    public void Refresh()
    {
        Children.Clear();
        if (_editor == IntPtr.Zero) return;
        var specs = AnnotationParser.Parse(ShotCore.EditorRenderJson(_editor));
        var s = RenderScale;
        foreach (var spec in specs)
            AddShape(spec, s);
    }

    private static SolidColorBrush Brush(ColorSpec c) => new(Color.FromArgb(c.A, c.R, c.G, c.B));

    private void AddShape(ShapeSpec spec, double s)
    {
        switch (spec.Type)
        {
            case ShapeType.Rectangle:
            case ShapeType.Highlighter:
            case ShapeType.Redact:
            {
                var r = new WinShapes.Rectangle { Width = spec.W * s, Height = spec.H * s, StrokeThickness = spec.StrokeWidth * s };
                if (spec.Type == ShapeType.Rectangle)
                {
                    r.Stroke = Brush(spec.Stroke);
                    if (spec.Fill is ColorSpec f) r.Fill = Brush(f);
                    if (spec.CornerRadius > 0) { r.RadiusX = spec.CornerRadius * s; r.RadiusY = spec.CornerRadius * s; }
                }
                else if (spec.Type == ShapeType.Highlighter)
                {
                    r.Fill = Brush(spec.Stroke);
                    r.Opacity = 0.4;
                }
                else
                {
                    r.Fill = new SolidColorBrush(Color.FromArgb(255, 40, 40, 40));
                }
                Canvas.SetLeft(r, spec.X * s);
                Canvas.SetTop(r, spec.Y * s);
                Children.Add(r);
                break;
            }
            case ShapeType.Ellipse:
            {
                var el = new WinShapes.Ellipse { Width = spec.W * s, Height = spec.H * s, Stroke = Brush(spec.Stroke), StrokeThickness = spec.StrokeWidth * s };
                if (spec.Fill is ColorSpec f) el.Fill = Brush(f);
                Canvas.SetLeft(el, spec.X * s);
                Canvas.SetTop(el, spec.Y * s);
                Children.Add(el);
                break;
            }
            case ShapeType.Line:
            case ShapeType.Arrow:
            {
                var line = new Line { X1 = spec.X1 * s, Y1 = spec.Y1 * s, X2 = spec.X2 * s, Y2 = spec.Y2 * s, Stroke = Brush(spec.Stroke), StrokeThickness = spec.StrokeWidth * s };
                Children.Add(line);
                if (spec.Type == ShapeType.Arrow) AddArrowHead(spec, s);
                break;
            }
            case ShapeType.Freehand:
            {
                var poly = new Polyline { Stroke = Brush(spec.Stroke), StrokeThickness = spec.StrokeWidth * s };
                foreach (var pt in spec.Points) poly.Points.Add(new Point(pt.X * s, pt.Y * s));
                Children.Add(poly);
                break;
            }
            case ShapeType.Text:
            {
                var tb = new TextBlock { Text = spec.Text, FontSize = spec.FontSize * s, Foreground = Brush(spec.Stroke) };
                Canvas.SetLeft(tb, spec.X * s);
                Canvas.SetTop(tb, spec.Y * s);
                Children.Add(tb);
                break;
            }
            case ShapeType.Counter:
            {
                double r = spec.H;
                var circle = new WinShapes.Ellipse { Width = 2 * r * s, Height = 2 * r * s, Fill = Brush(spec.Stroke) };
                Canvas.SetLeft(circle, (spec.X - r) * s);
                Canvas.SetTop(circle, (spec.Y - r) * s);
                Children.Add(circle);
                var num = new TextBlock { Text = spec.Number.ToString(), FontSize = r * s, Foreground = new SolidColorBrush(Colors.White) };
                Canvas.SetLeft(num, (spec.X - r * 0.3) * s);
                Canvas.SetTop(num, (spec.Y - r * 0.7) * s);
                Children.Add(num);
                break;
            }
        }
    }

    private void AddArrowHead(ShapeSpec spec, double s)
    {
        double dx = spec.X2 - spec.X1;
        double dy = spec.Y2 - spec.Y1;
        double len = Math.Sqrt(dx * dx + dy * dy);
        if (len < 1e-6) return;
        double ux = dx / len, uy = dy / len;
        double head = Math.Max(10.0, spec.StrokeWidth * 3.5);
        double a = 0.5;
        double ca = Math.Cos(a), sa = Math.Sin(a);
        var tip = new Point(spec.X2 * s, spec.Y2 * s);
        var left = new Point((spec.X2 - head * (ux * ca - uy * sa)) * s, (spec.Y2 - head * (uy * ca + ux * sa)) * s);
        var right = new Point((spec.X2 - head * (ux * ca + uy * sa)) * s, (spec.Y2 - head * (uy * ca - ux * sa)) * s);
        var poly = new Polygon { Fill = Brush(spec.Stroke) };
        poly.Points.Add(tip);
        poly.Points.Add(left);
        poly.Points.Add(right);
        Children.Add(poly);
    }

public void SetStrokeColor(byte r, byte g, byte b, byte a) { if (_editor != IntPtr.Zero) { ShotCore.EditorSetStrokeColor(_editor, r, g, b, a); Refresh(); } }
public void SetStrokeWidth(double width) { if (_editor != IntPtr.Zero) { ShotCore.EditorSetStrokeWidth(_editor, width); Refresh(); } }
public void SetStyleJson(string json) { if (_editor != IntPtr.Zero) { ShotCore.EditorSetStyleJson(_editor, json); Refresh(); } }

public void SetSelectedText(string text) { if (_editor != IntPtr.Zero) { ShotCore.EditorSetSelectedText(_editor, text); Refresh(); } }
}
