//! Share-link model for optional cloud sharing: expiry, password protection, and
//! self-destruct after a number of views. Hashing uses SHA-256 (never store plaintext).
//!
//! The backend owns storage and TLS; this model enforces the access rules identically
//! wherever it runs, and is the single source of truth the apps and server agree on.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hash a share password to lowercase hex SHA-256.
pub fn hash_password(plain: &str) -> String {
 let mut h = Sha256::new();
 h.update(plain.as_bytes());
 h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// A shareable link to an uploaded capture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLink {
 pub id: String,
 pub created_at: u64,
 /// Absolute expiry in unix seconds; None means no time limit.
 pub expires_at: Option<u64>,
 /// SHA-256 hex of the password; None means no password.
 pub password_hash: Option<String>,
 /// Self-destruct after this many successful views; None means unlimited.
 pub max_views: Option<u32>,
 pub views: u32,
}

/// Why a view was denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDenied {
 Expired,
 Exhausted,
 BadPassword,
}

impl ShareLink {
 /// A link with no restrictions.
 pub fn new(id: impl Into<String>, created_at: u64) -> Self {
 Self {
 id: id.into(),
 created_at,
 expires_at: None,
 password_hash: None,
 max_views: None,
 views: 0,
 }
 }

 pub fn with_expiry(mut self, expires_at: u64) -> Self {
 self.expires_at = Some(expires_at);
 self
 }

 pub fn with_password(mut self, plain: &str) -> Self {
 self.password_hash = Some(hash_password(plain));
 self
 }

 pub fn with_max_views(mut self, n: u32) -> Self {
 self.max_views = Some(n);
 self
 }

 pub fn is_expired(&self, now: u64) -> bool {
 matches!(self.expires_at, Some(e) if now >= e)
 }

 pub fn is_exhausted(&self) -> bool {
 matches!(self.max_views, Some(m) if self.views >= m)
 }

 /// Attempt a view. On success increments the view counter and returns Ok.
 pub fn try_view(&mut self, now: u64, password: Option<&str>) -> Result<(), AccessDenied> {
 if self.is_expired(now) {
 return Err(AccessDenied::Expired);
 }
 if self.is_exhausted() {
 return Err(AccessDenied::Exhausted);
 }
 if let Some(hash) = &self.password_hash {
 match password {
 Some(p) if &hash_password(p) == hash => {}
 _ => return Err(AccessDenied::BadPassword),
 }
 }
 self.views += 1;
 Ok(())
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn password_hash_is_stable_and_hidden() {
 let h = hash_password("hunter2");
 assert_eq!(h.len(), 64);
 assert_ne!(h, "hunter2");
 assert_eq!(h, hash_password("hunter2"));
 }

 #[test]
 fn open_link_allows_views() {
 let mut l = ShareLink::new("abc", 1000);
 assert!(l.try_view(2000, None).is_ok());
 assert_eq!(l.views, 1);
 }

 #[test]
 fn expiry_blocks() {
 let mut l = ShareLink::new("abc", 1000).with_expiry(1500);
 assert_eq!(l.try_view(2000, None), Err(AccessDenied::Expired));
 }

 #[test]
 fn password_required() {
 let mut l = ShareLink::new("abc", 1000).with_password("s3cret");
 assert_eq!(l.try_view(1100, None), Err(AccessDenied::BadPassword));
 assert_eq!(l.try_view(1100, Some("nope")), Err(AccessDenied::BadPassword));
 assert!(l.try_view(1100, Some("s3cret")).is_ok());
 }

 #[test]
 fn self_destruct_after_views() {
 let mut l = ShareLink::new("abc", 1000).with_max_views(2);
 assert!(l.try_view(1100, None).is_ok());
 assert!(l.try_view(1100, None).is_ok());
 assert_eq!(l.try_view(1100, None), Err(AccessDenied::Exhausted));
 }
}
