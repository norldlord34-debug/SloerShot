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
// The no-secret hosts work out of the box; templates need the user to fill a key/account.
public static class BuiltInDestinations
{
 public const string ServerToken = "%SERVER%";
 public const string ImgurClientToken = "%IMGUR_CLIENT_ID%";

 // Editable templates (loaded into the Add-custom box for the user to fill in).
 public const string PastebinTemplate = "{\"Name\":\"Pastebin\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://pastebin.com/api/api_post.php\",\"Body\":\"FormURLEncoded\",\"Arguments\":{\"api_dev_key\":\"YOUR_PASTEBIN_KEY\",\"api_option\":\"paste\",\"api_paste_code\":\"{input}\"},\"URL\":\"{response}\"}";
 public const string BearerTemplate = "{\"Name\":\"My API\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://api.example.com/upload\",\"Headers\":{\"Authorization\":\"Bearer YOUR_TOKEN\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:url}\"}";
 public const string FtpTemplate = "{\"Name\":\"My FTP\",\"RequestMethod\":\"PUT\",\"RequestURL\":\"ftp://user:pass@ftp.example.com/public_html/{filename}\",\"Body\":\"Binary\",\"URL\":\"https://example.com/{filename}\"}";

 private static UploadDestination B(string id, string name, string cfg) => new UploadDestination { Id = id, Name = name, BuiltIn = true, ConfigJson = cfg };

 public static List<UploadDestination> Seed()
 {
 return new List<UploadDestination>
 {
 B("builtin-sloershot", "SloerShot Backend", "{\"Name\":\"SloerShot Backend\",\"RequestMethod\":\"POST\",\"RequestURL\":\"%SERVER%/v1/upload\",\"Body\":\"Binary\",\"URL\":\"{json:url}\"}"),
 B("builtin-imgur", "Imgur (anonymous)", "{\"Name\":\"Imgur\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://api.imgur.com/3/image\",\"Headers\":{\"Authorization\":\"Client-ID %IMGUR_CLIENT_ID%\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"image\",\"URL\":\"{json:data.link}\",\"DeletionURL\":\"https://imgur.com/delete/{json:data.deletehash}\"}"),
 B("builtin-catbox", "catbox.moe", "{\"Name\":\"catbox.moe\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://catbox.moe/user/api.php\",\"Body\":\"MultipartFormData\",\"Arguments\":{\"reqtype\":\"fileupload\"},\"FileFormName\":\"fileToUpload\",\"URL\":\"{response}\"}"),
 B("builtin-litterbox", "Litterbox (1h)", "{\"Name\":\"Litterbox\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://litterbox.catbox.moe/resources/internals/api.php\",\"Body\":\"MultipartFormData\",\"Arguments\":{\"reqtype\":\"fileupload\",\"time\":\"1h\"},\"FileFormName\":\"fileToUpload\",\"URL\":\"{response}\"}"),
 B("builtin-0x0", "0x0.st", "{\"Name\":\"0x0.st\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://0x0.st\",\"Headers\":{\"User-Agent\":\"SloerShot/1.0\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{response}\"}"),
 B("builtin-transfersh", "transfer.sh", "{\"Name\":\"transfer.sh\",\"RequestMethod\":\"PUT\",\"RequestURL\":\"https://transfer.sh/{filename}\",\"Body\":\"Binary\",\"URL\":\"{response}\"}"),
 B("builtin-tmpfiles", "tmpfiles.org", "{\"Name\":\"tmpfiles.org\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://tmpfiles.org/api/v1/upload\",\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:data.url}\"}"),
 B("builtin-fileio", "file.io", "{\"Name\":\"file.io\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://file.io\",\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:link}\"}"),
 B("builtin-pasters", "paste.rs (text)", "{\"Name\":\"paste.rs\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://paste.rs/\",\"Body\":\"Binary\",\"URL\":\"{response}\"}"),
 };
 }
}

// URL shorteners modeled as ShareX custom uploaders (GET with {input} = the long URL).
public static class BuiltInShorteners
{
 public const string Isgd = "{\"RequestMethod\":\"GET\",\"RequestURL\":\"https://is.gd/create.php\",\"Parameters\":{\"format\":\"simple\",\"url\":\"{input}\"}}";
 public const string TinyUrl = "{\"RequestMethod\":\"GET\",\"RequestURL\":\"https://tinyurl.com/api-create.php\",\"Parameters\":{\"url\":\"{input}\"}}";
}
