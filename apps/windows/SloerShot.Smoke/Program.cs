// Console smoke test: proves the Windows C# <-> Rust core binding (P/Invoke) and
// the native capture utilities work on this machine.
// Run: dotnet run --project apps/windows/SloerShot.Smoke [capture]
using SloerShot.Capture;
using SloerShot.Editor;
using SloerShot.Interop;

int failures = 0;
void Check(string label, bool ok)
{
    Console.WriteLine((ok ? "[ok] " : "[FAIL] ") + label);
    if (!ok) failures++;
}

var version = ShotCore.Version();
Console.WriteLine($"shotcore version: {version}");
Check("version is non-empty", !string.IsNullOrEmpty(version) && version != "unknown");

var doc = ShotCore.NewDocument(640, 480);
Check("document_new returns json", doc != null && doc.Contains("schema_version") && doc.Contains("640"));

var ed = ShotCore.EditorNew(800, 600);
Check("editor handle created", ed != IntPtr.Zero);
ShotCore.EditorSetTool(ed, 2);
ShotCore.EditorPointerDown(ed, 10, 10);
ShotCore.EditorPointerDrag(ed, 110, 60);
var preview = ShotCore.EditorRenderJson(ed);
Check("live preview shows the draft rectangle", preview != null && preview.Contains("Rectangle"));
var committedBefore = ShotCore.EditorDocumentJson(ed);
Check("committed doc empty before pointer-up", committedBefore != null && committedBefore.Contains("\"annotations\":[]"));
ShotCore.EditorPointerUp(ed, 110, 60);
Check("can undo after commit", ShotCore.EditorCanUndo(ed) == 1);
var committedAfter = ShotCore.EditorDocumentJson(ed);
Check("committed doc has the rectangle", committedAfter != null && committedAfter.Contains("Rectangle"));
Check("undo succeeds", ShotCore.EditorUndo(ed) == 1);
ShotCore.EditorFree(ed);

var docJson = ShotCore.NewDocument(200, 100)!;
var ocrJson = "{\"lines\":[{\"text\":\"alice@example.com\",\"bbox\":{\"x\":0,\"y\":0,\"w\":120,\"h\":10},\"words\":[{\"text\":\"alice@example.com\",\"bbox\":{\"x\":0,\"y\":0,\"w\":120,\"h\":10},\"confidence\":0.9}]}]}";
var redacted = ShotCore.AutoRedactInto(docJson, ocrJson, 1, 12);
Check("auto-redact added a Redact over the email", redacted != null && redacted.Contains("Redact"));

// Annotation parser probe: drive the editor to draw one of every tool, then parse render_json.
{
    var pe = ShotCore.EditorNew(400, 300);
    void Draw(uint tool, double x1, double y1, double x2, double y2)
    {
        ShotCore.EditorSetTool(pe, tool);
        ShotCore.EditorPointerDown(pe, x1, y1);
        ShotCore.EditorPointerDrag(pe, x2, y2);
        ShotCore.EditorPointerUp(pe, x2, y2);
    }
    void Place(uint tool, double x, double y)
    {
        ShotCore.EditorSetTool(pe, tool);
        ShotCore.EditorPointerDown(pe, x, y);
        ShotCore.EditorPointerUp(pe, x, y);
    }
    Draw(1, 10, 10, 50, 50);
    Draw(2, 10, 60, 60, 110);
    Draw(3, 70, 10, 120, 60);
    Draw(4, 130, 10, 180, 60);
    Draw(5, 10, 120, 40, 140);
    Place(6, 200, 10);
    Place(7, 220, 40);
    Draw(8, 10, 150, 100, 170);
    Draw(9, 120, 150, 200, 180);
    var rj = ShotCore.EditorDocumentJson(pe);
    var specs = AnnotationParser.Parse(rj);
    ShotCore.EditorFree(pe);
    Console.WriteLine($"parsed {specs.Count} shapes from editor output");
    var kinds = new[] { ShapeType.Arrow, ShapeType.Rectangle, ShapeType.Ellipse, ShapeType.Line, ShapeType.Freehand, ShapeType.Text, ShapeType.Counter, ShapeType.Highlighter, ShapeType.Redact };
    Check("parser found all 9 tool kinds", specs.Count == 9 && kinds.All(t => specs.Any(s => s.Type == t)));
    var rectSpec = specs.Find(s => s.Type == ShapeType.Rectangle);
    Check("rectangle spec has expected bounds", rectSpec != null && Math.Abs(rectSpec.W - 50) < 0.01 && Math.Abs(rectSpec.H - 50) < 0.01);
    Check("specs are sorted by z ascending", specs.Zip(specs.Skip(1), (a, b) => a.Z <= b.Z).All(ok => ok));
}

if (args.Contains("capture"))
{
    Console.WriteLine("--- native capture probe ---");
    var desktop = NativeScreen.DescribeDesktopJson();
    Console.WriteLine($"desktop: {desktop}");
    Check("desktop json has displays + scale_factor", desktop.Contains("displays") && desktop.Contains("scale_factor"));
    var region = ShotCore.ResolveSelection(desktop, 100, 100, 200, 150);
    Console.WriteLine($"resolve sample region: {region}");
    Check("core resolved a selection against the real desktop", !string.IsNullOrEmpty(region) && region!.Contains("physical"));
    var tmp = Path.Combine(Path.GetTempPath(), "sloershot-capture.png");
    var res = FrozenScreenCapture.CaptureVirtualScreen(tmp);
    Console.WriteLine($"captured {res.Width}x{res.Height} -> {res.Path}");
    Check("frozen-screen capture wrote a non-empty PNG", File.Exists(res.Path) && new FileInfo(res.Path).Length > 0);
}

Console.WriteLine(failures == 0 ? "ALL OK: Windows binding + capture verified." : $"{failures} CHECK(S) FAILED");
Environment.Exit(failures == 0 ? 0 : 1);
