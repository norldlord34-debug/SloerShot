#nullable enable
using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
namespace SloerShot.Services;

// A configurable upload destination. ConfigJson is a ShareX-compatible custom uploader config
// (parsed/executed by the Rust core via ShotCore.CustomUploaderBuildPlan/ResolveResponse).
public sealed class UploadDestination
{
 public string Id { get; set; } = Guid.NewGuid().ToString("N");
 public string Name { get; set; } = "";
 public string ConfigJson { get; set; } = "";
 public bool BuiltIn { get; set; }
}

// Mirrors the core RequestPlan JSON (ffi.rs / custom_uploader.rs).
public sealed class RequestPlan
{
 [JsonPropertyName("method")] public string Method { get; set; } = "POST";
 [JsonPropertyName("url")] public string Url { get; set; } = "";
 [JsonPropertyName("headers")] public Dictionary<string, string> Headers { get; set; } = new();
 [JsonPropertyName("body")] public string Body { get; set; } = "None";
 [JsonPropertyName("arguments")] public Dictionary<string, string> Arguments { get; set; } = new();
 [JsonPropertyName("file_form_name")] public string FileFormName { get; set; } = "";
 [JsonPropertyName("data")] public string Data { get; set; } = "";
}

// Mirrors the core ResponseResult JSON.
public sealed class ResponseLinks
{
 [JsonPropertyName("url")] public string Url { get; set; } = "";
 [JsonPropertyName("thumbnail_url")] public string ThumbnailUrl { get; set; } = "";
 [JsonPropertyName("deletion_url")] public string DeletionUrl { get; set; } = "";
}

public sealed class UploadOutcome
{
 public bool Success { get; set; }
 public string Url { get; set; } = "";
 public string ThumbnailUrl { get; set; } = "";
 public string DeletionUrl { get; set; } = "";
 public string Error { get; set; } = "";
 public static UploadOutcome Ok(string url, string thumb, string del) => new UploadOutcome { Success = true, Url = url, ThumbnailUrl = thumb, DeletionUrl = del };
 public static UploadOutcome Fail(string err) => new UploadOutcome { Success = false, Error = err };
}

// Factory for the built-in destinations. Tokens are substituted by AppSettings before upload.
public static class BuiltInDestinations
{
 public const string ServerToken = "%SERVER%";
 public const string ImgurClientToken = "%IMGUR_CLIENT_ID%";
 public static List<UploadDestination> Seed()
 {
 return new List<UploadDestination>
 {
 new UploadDestination { Id = "builtin-sloershot", Name = "SloerShot Backend", BuiltIn = true, ConfigJson = "{\"Name\":\"SloerShot Backend\",\"RequestMethod\":\"POST\",\"RequestURL\":\"%SERVER%/v1/upload\",\"Body\":\"Binary\",\"URL\":\"{json:url}\"}" },
 new UploadDestination { Id = "builtin-imgur", Name = "Imgur (anonymous)", BuiltIn = true, ConfigJson = "{\"Name\":\"Imgur\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://api.imgur.com/3/image\",\"Headers\":{\"Authorization\":\"Client-ID %IMGUR_CLIENT_ID%\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"image\",\"URL\":\"{json:data.link}\",\"DeletionURL\":\"https://imgur.com/delete/{json:data.deletehash}\"}" }
 };
 }
}

// URL shorteners modeled as ShareX custom uploaders (GET with {input} = the long URL).
public static class BuiltInShorteners
{
 public const string Isgd = "{\"RequestMethod\":\"GET\",\"RequestURL\":\"https://is.gd/create.php\",\"Parameters\":{\"format\":\"simple\",\"url\":\"{input}\"}}";
 public const string TinyUrl = "{\"RequestMethod\":\"GET\",\"RequestURL\":\"https://tinyurl.com/api-create.php\",\"Parameters\":{\"url\":\"{input}\"}}";
}
