#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using SloerShot.Interop;
namespace SloerShot.Services;

// Executes a ShareX-compatible custom uploader: the Rust core builds the request plan and
// resolves the response links; this class performs the actual HTTP call (multipart/form/json/xml/binary).
public sealed class UploaderEngine
{
 private static readonly HttpClient _http = new HttpClient { Timeout = TimeSpan.FromSeconds(60) };

 public async Task<UploadOutcome> UploadFileAsync(string configJson, string filePath)
 {
 if (!File.Exists(filePath)) return UploadOutcome.Fail("File not found");
 string fileName = Path.GetFileName(filePath);
 byte[] fileBytes;
 try { fileBytes = await File.ReadAllBytesAsync(filePath); }
 catch (Exception ex) { return UploadOutcome.Fail("Read failed: " + ex.Message); }

 string? planJson = ShotCore.CustomUploaderBuildPlan(configJson, fileName, fileName);
 if (string.IsNullOrEmpty(planJson)) return UploadOutcome.Fail("Invalid uploader config JSON");
 RequestPlan? plan;
 try { plan = JsonSerializer.Deserialize<RequestPlan>(planJson); } catch { plan = null; }
 if (plan == null || string.IsNullOrWhiteSpace(plan.Url)) return UploadOutcome.Fail("Could not build request");

 using var req = new HttpRequestMessage(new HttpMethod(string.IsNullOrEmpty(plan.Method) ? "POST" : plan.Method), plan.Url);
 string mime = GuessMime(fileName);
 switch (plan.Body)
 {
 case "MultipartFormData":
 {
 var mp = new MultipartFormDataContent();
 foreach (var a in plan.Arguments) mp.Add(new StringContent(a.Value ?? ""), a.Key);
 var fc = new ByteArrayContent(fileBytes);
 fc.Headers.ContentType = new MediaTypeHeaderValue(mime);
 mp.Add(fc, string.IsNullOrEmpty(plan.FileFormName) ? "file" : plan.FileFormName, fileName);
 req.Content = mp;
 break;
 }
 case "FormURLEncoded":
 req.Content = new FormUrlEncodedContent(plan.Arguments);
 break;
 case "JSON":
 req.Content = new StringContent(plan.Data ?? "", Encoding.UTF8, "application/json");
 break;
 case "XML":
 req.Content = new StringContent(plan.Data ?? "", Encoding.UTF8, "application/xml");
 break;
 case "Binary":
 {
 var bc = new ByteArrayContent(fileBytes);
 bc.Headers.ContentType = new MediaTypeHeaderValue(mime);
 req.Content = bc;
 break;
 }
 default:
 break;
 }

 foreach (var h in plan.Headers)
 {
 if (!req.Headers.TryAddWithoutValidation(h.Key, h.Value) && req.Content != null)
 req.Content.Headers.TryAddWithoutValidation(h.Key, h.Value);
 }

 HttpResponseMessage resp;
 try { resp = await _http.SendAsync(req); }
 catch (Exception ex) { return UploadOutcome.Fail("Network error: " + ex.Message); }

 string respBody = await resp.Content.ReadAsStringAsync();
 if (!resp.IsSuccessStatusCode)
 return UploadOutcome.Fail("HTTP " + (int)resp.StatusCode + ": " + Trunc(respBody, 300));

 var headers = new Dictionary<string, string>();
 foreach (var h in resp.Headers) headers[h.Key] = string.Join(",", h.Value);
 foreach (var h in resp.Content.Headers) headers[h.Key] = string.Join(",", h.Value);
 string headersJson = JsonSerializer.Serialize(headers);

 string? linksJson = ShotCore.CustomUploaderResolveResponse(configJson, respBody, headersJson, fileName, fileName);
 ResponseLinks? links = null;
 try { if (!string.IsNullOrEmpty(linksJson)) links = JsonSerializer.Deserialize<ResponseLinks>(linksJson); } catch { }
 string url = links?.Url ?? "";
 if (string.IsNullOrWhiteSpace(url)) url = respBody.Trim();
 if (string.IsNullOrWhiteSpace(url)) return UploadOutcome.Fail("No URL in response");
 return UploadOutcome.Ok(url, links?.ThumbnailUrl ?? "", links?.DeletionUrl ?? "");
 }

 private static string Trunc(string s, int n) => s.Length <= n ? s : s.Substring(0, n) + "...";

 private static string GuessMime(string name)
 {
 string e = Path.GetExtension(name).ToLowerInvariant();
 return e switch
 {
 ".png" => "image/png",
 ".jpg" or ".jpeg" => "image/jpeg",
 ".gif" => "image/gif",
 ".webp" => "image/webp",
 ".bmp" => "image/bmp",
 ".mp4" => "video/mp4",
 ".txt" => "text/plain",
 _ => "application/octet-stream",
 };
 }

 // Run a URL shortener (ShareX custom uploader with {input} = the long URL). No file payload.
 public async Task<UploadOutcome> ShortenUrlAsync(string configJson, string longUrl)
 {
 string? planJson = ShotCore.CustomUploaderBuildPlan(configJson, longUrl, "");
 if (string.IsNullOrEmpty(planJson)) return UploadOutcome.Fail("Invalid shortener config");
 RequestPlan? plan;
 try { plan = JsonSerializer.Deserialize<RequestPlan>(planJson); } catch { plan = null; }
 if (plan == null || string.IsNullOrWhiteSpace(plan.Url)) return UploadOutcome.Fail("Could not build request");
 using var req = new HttpRequestMessage(new HttpMethod(string.IsNullOrEmpty(plan.Method) ? "GET" : plan.Method), plan.Url);
 switch (plan.Body)
 {
 case "FormURLEncoded": req.Content = new FormUrlEncodedContent(plan.Arguments); break;
 case "JSON": req.Content = new StringContent(plan.Data ?? "", Encoding.UTF8, "application/json"); break;
 case "XML": req.Content = new StringContent(plan.Data ?? "", Encoding.UTF8, "application/xml"); break;
 default: break;
 }
 foreach (var h in plan.Headers)
 {
 if (!req.Headers.TryAddWithoutValidation(h.Key, h.Value) && req.Content != null)
 req.Content.Headers.TryAddWithoutValidation(h.Key, h.Value);
 }
 HttpResponseMessage resp;
 try { resp = await _http.SendAsync(req); }
 catch (Exception ex) { return UploadOutcome.Fail("Network error: " + ex.Message); }
 string respBody = await resp.Content.ReadAsStringAsync();
 if (!resp.IsSuccessStatusCode) return UploadOutcome.Fail("HTTP " + (int)resp.StatusCode + ": " + Trunc(respBody, 200));
 var headers = new Dictionary<string, string>();
 foreach (var h in resp.Headers) headers[h.Key] = string.Join(",", h.Value);
 foreach (var h in resp.Content.Headers) headers[h.Key] = string.Join(",", h.Value);
 string headersJson = JsonSerializer.Serialize(headers);
 string? linksJson = ShotCore.CustomUploaderResolveResponse(configJson, respBody, headersJson, longUrl, "");
 ResponseLinks? links = null;
 try { if (!string.IsNullOrEmpty(linksJson)) links = JsonSerializer.Deserialize<ResponseLinks>(linksJson); } catch { }
 string shortUrl = (links?.Url ?? "").Trim();
 if (string.IsNullOrWhiteSpace(shortUrl)) shortUrl = respBody.Trim();
 if (string.IsNullOrWhiteSpace(shortUrl) || !shortUrl.StartsWith("http")) return UploadOutcome.Fail("Shortener error: " + Trunc(respBody, 120));
 return UploadOutcome.Ok(shortUrl, "", "");
 }
}
