import Foundation

// CleanShot-Cloud-style client: POST /v1/share and resolve the share link via the tested
// core (ShotCore.shareRequestBody / shareLink). This performs the HTTP call.
struct CloudClient {
 let baseURL: String

 func createShareLink(password: String?, expiresAt: Int64, maxViews: Int64) async -> String? {
 let base = baseURL.hasSuffix("/") ? String(baseURL.dropLast()) : baseURL
 guard let body = ShotCore.shareRequestBody(password: password, expiresAt: expiresAt, maxViews: maxViews),
 let url = URL(string: base + "/v1/share") else { return nil }
 var req = URLRequest(url: url)
 req.httpMethod = "POST"
 req.setValue("application/json", forHTTPHeaderField: "Content-Type")
 req.httpBody = body.data(using: .utf8)
 guard let (data, resp) = try? await URLSession.shared.data(for: req),
 let http = resp as? HTTPURLResponse, http.statusCode == 200,
 let json = String(data: data, encoding: .utf8) else { return nil }
 return ShotCore.shareLink(baseUrl: base, responseJson: json)
 }
}
