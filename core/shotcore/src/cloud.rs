//! Cloud share client logic (CleanShot Cloud). Builds the POST /v1/share request body and
//! parses the response into an absolute shareable link. The native app performs the HTTP
//! call; request shaping + URL building + response parsing live here and are unit tested.
use serde::{Deserialize, Serialize};

/// Options for creating a share link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShareRequest {
 #[serde(skip_serializing_if = "Option::is_none")]
 pub password: Option<String>,
 #[serde(skip_serializing_if = "Option::is_none")]
 pub expires_at: Option<i64>,
 #[serde(skip_serializing_if = "Option::is_none")]
 pub max_views: Option<u32>,
}

impl ShareRequest {
 pub fn to_json(&self) -> String {
 serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"))
 }
}

/// The backend /v1/share response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShareResponse {
 pub id: String,
 #[serde(default)]
 pub url: String,
}

/// Build the absolute share link from a base URL and the backend response JSON.
pub fn share_link(base_url: &str, response_json: &str) -> Option<String> {
 let resp: ShareResponse = serde_json::from_str(response_json).ok()?;
 let path = if resp.url.is_empty() {
 format!("/s/{}", resp.id)
 } else {
 resp.url.clone()
 };
 let base = base_url.trim_end_matches("/");
 Some(format!("{}{}", base, path))
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn request_json_omits_none() {
 let req = ShareRequest { password: Some(String::from("pw")), expires_at: None, max_views: Some(3) };
 let j = req.to_json();
 assert!(j.contains("\"password\":\"pw\""));
 assert!(j.contains("\"max_views\":3"));
 assert!(!j.contains("expires_at"));
 }

 #[test]
 fn builds_link_from_response() {
 let with_url = share_link("https://sloer.sh/", "{\"id\":\"abc\",\"url\":\"/s/abc\"}").unwrap();
 assert_eq!(with_url, "https://sloer.sh/s/abc");
 let no_url = share_link("https://sloer.sh", "{\"id\":\"xyz\"}").unwrap();
 assert_eq!(no_url, "https://sloer.sh/s/xyz");
 assert!(share_link("https://x", "not json").is_none());
 }
}
