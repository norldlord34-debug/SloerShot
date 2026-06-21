using System;
using System.Net.Http;
using System.Text;
using System.Threading.Tasks;
using SloerShot.Interop;

namespace SloerShot;

// CleanShot-Cloud-style client: uploads a capture to the SloerShot backend /v1/share and
// returns a shareable link. The request body and the final link are built by the tested
// core (ShotCore.ShareRequestBody / ShotCore.ShareLink); this performs the HTTP call.
public sealed class CloudClient
{
 private readonly HttpClient _http = new();
 private readonly string _baseUrl;

 public CloudClient(string baseUrl)
 {
 _baseUrl = baseUrl.EndsWith("/") ? baseUrl.Substring(0, baseUrl.Length - 1) : baseUrl;
 }

 public async Task<string?> CreateShareLinkAsync(string? password, long expiresAt, long maxViews)
 {
 var body = ShotCore.ShareRequestBody(password, expiresAt, maxViews) ?? "{}";
 using var content = new StringContent(body, Encoding.UTF8, "application/json");
 var resp = await _http.PostAsync(_baseUrl + "/v1/share", content);
 if (!resp.IsSuccessStatusCode) return null;
 var json = await resp.Content.ReadAsStringAsync();
 return ShotCore.ShareLink(_baseUrl, json);
 }
}
