using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Threading.Tasks;
using Windows.Graphics.Imaging;
using Windows.Media.Ocr;

namespace SloerShot.Interop;

// Real on-device OCR via Windows.Media.Ocr. Produces the OcrResult JSON that shotcore
// consumes, so all region/reading-order/clipboard formatting runs in the tested core
// (ShotCore.OcrTextInRegion / OcrSingleLine). The OS engine only supplies words + boxes.
public static class OcrService
{
 public static bool IsAvailable => OcrEngine.AvailableRecognizerLanguages.Count > 0;

 public static async Task<string?> RecognizeJsonAsync(SoftwareBitmap bitmap)
 {
 var engine = OcrEngine.TryCreateFromUserProfileLanguages();
 if (engine == null) return null;
 var result = await engine.RecognizeAsync(bitmap);
 var lines = new List<object>();
 foreach (var line in result.Lines)
 {
 var words = new List<object>();
 double minX = double.MaxValue, minY = double.MaxValue, maxX = 0, maxY = 0;
 foreach (var word in line.Words)
 {
 var r = word.BoundingRect;
 words.Add(new { text = word.Text, bbox = new { x = r.X, y = r.Y, w = r.Width, h = r.Height }, confidence = 1.0 });
 minX = Math.Min(minX, r.X);
 minY = Math.Min(minY, r.Y);
 maxX = Math.Max(maxX, r.X + r.Width);
 maxY = Math.Max(maxY, r.Y + r.Height);
 }
 if (line.Words.Count == 0) { minX = 0; minY = 0; maxX = 0; maxY = 0; }
 lines.Add(new { text = line.Text, bbox = new { x = minX, y = minY, w = maxX - minX, h = maxY - minY }, words });
 }
 return JsonSerializer.Serialize(new { lines });
 }

 public static async Task<string?> RecognizeTextAsync(SoftwareBitmap bitmap)
 {
 var engine = OcrEngine.TryCreateFromUserProfileLanguages();
 if (engine == null) return null;
 var result = await engine.RecognizeAsync(bitmap);
 return result.Text;
 }
}
