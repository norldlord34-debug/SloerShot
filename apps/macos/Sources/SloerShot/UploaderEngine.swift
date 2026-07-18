import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

struct UploadOutcome { var success: Bool; var url: String; var deletionURL: String; var error: String }

private struct RequestPlan: Codable {
 var method: String = "POST"
 var url: String = ""
 var headers: [String: String] = [:]
 var body: String = "None"
 var arguments: [String: String] = [:]
 var file_form_name: String = ""
 var data: String = ""
}
private struct ResponseLinks: Codable { var url = ""; var thumbnail_url = ""; var deletion_url = "" }

enum UploaderEngine {
 static func writeTempPNG(_ image: CGImage) -> URL? {
 let url = FileManager.default.temporaryDirectory.appendingPathComponent("sloershot-" + UUID().uuidString + ".png")
 guard let dest = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) else { return nil }
 CGImageDestinationAddImage(dest, image, nil)
 return CGImageDestinationFinalize(dest) ? url : nil
 }
 private static func fail(_ m: String) -> UploadOutcome { UploadOutcome(success: false, url: "", deletionURL: "", error: m) }
 private static func mime(_ name: String) -> String {
 switch (name as NSString).pathExtension.lowercased() {
 case "png": return "image/png"
 case "jpg", "jpeg": return "image/jpeg"
 case "gif": return "image/gif"
 case "webp": return "image/webp"
 case "txt": return "text/plain"
 default: return "application/octet-stream"
 }
 }
 private static func enc(_ s: String) -> String {
 var allowed = CharacterSet.alphanumerics
 allowed.insert(charactersIn: "-._~")
 return s.addingPercentEncoding(withAllowedCharacters: allowed) ?? s
 }
 private static func formEncode(_ args: [String: String]) -> Data {
 args.map { enc($0.key) + "=" + enc($0.value) }.joined(separator: "&").data(using: .utf8) ?? Data()
 }
 private static func multipart(_ args: [String: String], field: String, fileName: String, fileData: Data, mimeType: String, boundary: String) -> Data {
 var body = Data()
 let dash = "--" + boundary + "\r\n"
 for (k, v) in args {
 body.append(dash.data(using: .utf8)!)
 body.append("Content-Disposition: form-data; name=\"\(k)\"\r\n\r\n".data(using: .utf8)!)
 body.append((v + "\r\n").data(using: .utf8)!)
 }
 body.append(dash.data(using: .utf8)!)
 body.append("Content-Disposition: form-data; name=\"\(field)\"; filename=\"\(fileName)\"\r\n".data(using: .utf8)!)
 body.append("Content-Type: \(mimeType)\r\n\r\n".data(using: .utf8)!)
 body.append(fileData)
 body.append("\r\n".data(using: .utf8)!)
 body.append(("--" + boundary + "--\r\n").data(using: .utf8)!)
 return body
 }
 static func upload(configJson: String, fileURL: URL) async -> UploadOutcome {
 let fileName = fileURL.lastPathComponent
 guard let fileData = try? Data(contentsOf: fileURL) else { return fail("Cannot read file") }
 guard let planJson = ShotCore.customUploaderBuildPlan(configJson: configJson, input: fileName, filename: fileName),
 let plan = try? JSONDecoder().decode(RequestPlan.self, from: Data(planJson.utf8)) else { return fail("Invalid config") }
 if plan.url.hasPrefix("ftp") { return fail("FTP upload is not supported on macOS yet") }
 guard let url = URL(string: plan.url) else { return fail("Bad URL") }
 var req = URLRequest(url: url)
 req.httpMethod = plan.method.isEmpty ? "POST" : plan.method
 let mimeType = mime(fileName)
 switch plan.body {
 case "MultipartFormData":
 let boundary = "SloerShot-" + UUID().uuidString
 req.setValue("multipart/form-data; boundary=" + boundary, forHTTPHeaderField: "Content-Type")
 req.httpBody = multipart(plan.arguments, field: plan.file_form_name.isEmpty ? "file" : plan.file_form_name, fileName: fileName, fileData: fileData, mimeType: mimeType, boundary: boundary)
 case "FormURLEncoded":
 req.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
 req.httpBody = formEncode(plan.arguments)
 case "JSON":
 req.setValue("application/json", forHTTPHeaderField: "Content-Type")
 req.httpBody = Data(plan.data.utf8)
 case "XML":
 req.setValue("application/xml", forHTTPHeaderField: "Content-Type")
 req.httpBody = Data(plan.data.utf8)
 case "Binary":
 req.setValue(mimeType, forHTTPHeaderField: "Content-Type")
 req.httpBody = fileData
 default: break
 }
 for (k, v) in plan.headers { req.setValue(v, forHTTPHeaderField: k) }
 guard let (respData, resp) = try? await URLSession.shared.data(for: req) else { return fail("Network error") }
 let respBody = String(data: respData, encoding: .utf8) ?? ""
 if let http = resp as? HTTPURLResponse, http.statusCode >= 400 { return fail("HTTP \(http.statusCode)") }
 var headers: [String: String] = [:]
 if let http = resp as? HTTPURLResponse { for (k, v) in http.allHeaderFields { headers["\(k)"] = "\(v)" } }
 let headersJson = (try? JSONSerialization.data(withJSONObject: headers)).flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
 if let linksJson = ShotCore.customUploaderResolveResponse(configJson: configJson, response: respBody, headersJson: headersJson, input: fileName, filename: fileName),
 let links = try? JSONDecoder().decode(ResponseLinks.self, from: Data(linksJson.utf8)), !links.url.isEmpty {
 return UploadOutcome(success: true, url: links.url, deletionURL: links.deletion_url, error: "")
 }
 let trimmed = respBody.trimmingCharacters(in: .whitespacesAndNewlines)
 if trimmed.isEmpty { return fail("No URL in response") }
 return UploadOutcome(success: true, url: trimmed, deletionURL: "", error: "")
 }
 static func shorten(configJson: String, longURL: String) async -> String? {
 guard let planJson = ShotCore.customUploaderBuildPlan(configJson: configJson, input: longURL, filename: ""),
 let plan = try? JSONDecoder().decode(RequestPlan.self, from: Data(planJson.utf8)),
 let url = URL(string: plan.url) else { return nil }
 var req = URLRequest(url: url)
 req.httpMethod = plan.method.isEmpty ? "GET" : plan.method
 switch plan.body {
 case "FormURLEncoded": req.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type"); req.httpBody = formEncode(plan.arguments)
 case "JSON": req.setValue("application/json", forHTTPHeaderField: "Content-Type"); req.httpBody = Data(plan.data.utf8)
 default: break
 }
 for (k, v) in plan.headers { req.setValue(v, forHTTPHeaderField: k) }
 guard let (respData, resp) = try? await URLSession.shared.data(for: req), let http = resp as? HTTPURLResponse, http.statusCode < 400 else { return nil }
 let respBody = String(data: respData, encoding: .utf8) ?? ""
 if let linksJson = ShotCore.customUploaderResolveResponse(configJson: configJson, response: respBody, headersJson: "{}", input: longURL, filename: ""),
 let links = try? JSONDecoder().decode(ResponseLinks.self, from: Data(linksJson.utf8)), links.url.hasPrefix("http") { return links.url }
 let b = respBody.trimmingCharacters(in: .whitespacesAndNewlines)
 return b.hasPrefix("http") ? b : nil
 }
}
