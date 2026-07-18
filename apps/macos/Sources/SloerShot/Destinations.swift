import Foundation
import Combine

struct UploadDestination: Codable, Identifiable {
 var id: String
 var name: String
 var configJson: String
 var builtIn: Bool
}

enum BuiltInDestinations {
 static let serverToken = "%SERVER%"
 static let imgurClientToken = "%IMGUR_CLIENT_ID%"
 static let pastebinTemplate = "{\"Name\":\"Pastebin\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://pastebin.com/api/api_post.php\",\"Body\":\"FormURLEncoded\",\"Arguments\":{\"api_dev_key\":\"YOUR_PASTEBIN_KEY\",\"api_option\":\"paste\",\"api_paste_code\":\"{input}\"},\"URL\":\"{response}\"}"
 static let bearerTemplate = "{\"Name\":\"My API\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://api.example.com/upload\",\"Headers\":{\"Authorization\":\"Bearer YOUR_TOKEN\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:url}\"}"
 static func seed() -> [UploadDestination] {
 func b(_ id: String, _ name: String, _ cfg: String) -> UploadDestination { UploadDestination(id: id, name: name, configJson: cfg, builtIn: true) }
 return [
 b("builtin-sloershot", "SloerShot Backend", "{\"Name\":\"SloerShot Backend\",\"RequestMethod\":\"POST\",\"RequestURL\":\"%SERVER%/v1/upload\",\"Body\":\"Binary\",\"URL\":\"{json:url}\"}"),
 b("builtin-imgur", "Imgur (anonymous)", "{\"Name\":\"Imgur\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://api.imgur.com/3/image\",\"Headers\":{\"Authorization\":\"Client-ID %IMGUR_CLIENT_ID%\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"image\",\"URL\":\"{json:data.link}\",\"DeletionURL\":\"https://imgur.com/delete/{json:data.deletehash}\"}"),
 b("builtin-catbox", "catbox.moe", "{\"Name\":\"catbox.moe\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://catbox.moe/user/api.php\",\"Body\":\"MultipartFormData\",\"Arguments\":{\"reqtype\":\"fileupload\"},\"FileFormName\":\"fileToUpload\",\"URL\":\"{response}\"}"),
 b("builtin-litterbox", "Litterbox (1h)", "{\"Name\":\"Litterbox\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://litterbox.catbox.moe/resources/internals/api.php\",\"Body\":\"MultipartFormData\",\"Arguments\":{\"reqtype\":\"fileupload\",\"time\":\"1h\"},\"FileFormName\":\"fileToUpload\",\"URL\":\"{response}\"}"),
 b("builtin-0x0", "0x0.st", "{\"Name\":\"0x0.st\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://0x0.st\",\"Headers\":{\"User-Agent\":\"SloerShot/1.0\"},\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{response}\"}"),
 b("builtin-transfersh", "transfer.sh", "{\"Name\":\"transfer.sh\",\"RequestMethod\":\"PUT\",\"RequestURL\":\"https://transfer.sh/{filename}\",\"Body\":\"Binary\",\"URL\":\"{response}\"}"),
 b("builtin-tmpfiles", "tmpfiles.org", "{\"Name\":\"tmpfiles.org\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://tmpfiles.org/api/v1/upload\",\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:data.url}\"}"),
 b("builtin-fileio", "file.io", "{\"Name\":\"file.io\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://file.io\",\"Body\":\"MultipartFormData\",\"FileFormName\":\"file\",\"URL\":\"{json:link}\"}"),
 b("builtin-pasters", "paste.rs (text)", "{\"Name\":\"paste.rs\",\"RequestMethod\":\"POST\",\"RequestURL\":\"https://paste.rs/\",\"Body\":\"Binary\",\"URL\":\"{response}\"}"),
 ]
 }
}

final class DestinationStore: ObservableObject {
 static let shared = DestinationStore()
 @Published var destinations: [UploadDestination] = []
 @Published var activeId: String = ""

 init() { load() }

 func load() {
 let d = UserDefaults.standard
 if let data = d.data(forKey: "ss.destinations"), let arr = try? JSONDecoder().decode([UploadDestination].self, from: data) { destinations = arr }
 mergeBuiltIns()
 activeId = d.string(forKey: "ss.activeDestination") ?? ""
 if activeId.isEmpty || !destinations.contains(where: { $0.id == activeId }) { activeId = destinations.first?.id ?? "" }
 save()
 }
 private func mergeBuiltIns() {
 for s in BuiltInDestinations.seed() {
 if let idx = destinations.firstIndex(where: { $0.id == s.id }) {
 if destinations[idx].builtIn { destinations[idx].name = s.name; destinations[idx].configJson = s.configJson }
 } else { destinations.append(s) }
 }
 }
 func save() {
 let d = UserDefaults.standard
 if let data = try? JSONEncoder().encode(destinations) { d.set(data, forKey: "ss.destinations") }
 d.set(activeId, forKey: "ss.activeDestination")
 }
 var active: UploadDestination? { destinations.first(where: { $0.id == activeId }) ?? destinations.first }
 func resolveConfig(_ dest: UploadDestination) -> String {
 var cfg = dest.configJson
 var server = UserDefaults.standard.string(forKey: "ss.serverUrl") ?? ""
 while server.hasSuffix("/") { server = String(server.dropLast()) }
 cfg = cfg.replacingOccurrences(of: BuiltInDestinations.serverToken, with: server)
 let imgur = UserDefaults.standard.string(forKey: "ss.imgurClientId") ?? ""
 cfg = cfg.replacingOccurrences(of: BuiltInDestinations.imgurClientToken, with: imgur)
 return cfg
 }
 func add(_ dest: UploadDestination) { destinations.append(dest); save(); objectWillChange.send() }
 func remove(_ dest: UploadDestination) { guard !dest.builtIn else { return }; destinations.removeAll { $0.id == dest.id }; if activeId == dest.id { activeId = destinations.first?.id ?? "" }; save() }
 func setActive(_ id: String) { activeId = id; save() }
}

enum BuiltInShorteners {
 static let isgd = "{\"RequestMethod\":\"GET\",\"RequestURL\":\"https://is.gd/create.php\",\"Parameters\":{\"format\":\"simple\",\"url\":\"{input}\"}}"
 static let tinyurl = "{\"RequestMethod\":\"GET\",\"RequestURL\":\"https://tinyurl.com/api-create.php\",\"Parameters\":{\"url\":\"{input}\"}}"
}
