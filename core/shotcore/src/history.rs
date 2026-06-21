//! Searchable capture history index.
//!
//! A lightweight, pure-Rust store of capture metadata persisted as JSON. The
//! API is intentionally storage-agnostic so it can later be backed by SQLite
//! without changing call sites. Full-text search spans the file path, OCR
//! text, and tags.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// The kind of capture, for history filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureKind {
 Screenshot,
 Recording,
 Scrolling,
 Gif,
 Imported,
}
impl Default for CaptureKind {
 fn default() -> Self {
 CaptureKind::Screenshot
 }
}

/// Metadata for one capture in the history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub image_path: String,
    /// Unix timestamp in seconds when the capture was created.
    pub created_at: i64,
    pub width: u32,
    pub height: u32,
    pub ocr_text: Option<String>,
    pub tags: Vec<String>,
 /// What kind of capture this is.
 #[serde(default)]
 pub kind: CaptureKind,
}

impl HistoryEntry {
    pub fn new(image_path: impl Into<String>, created_at: i64, width: u32, height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            image_path: image_path.into(),
            created_at,
            width,
            height,
            ocr_text: None,
            tags: Vec::new(),
 kind: CaptureKind::default(),
        }
    }

    pub fn with_ocr_text(mut self, text: impl Into<String>) -> Self {
        self.ocr_text = Some(text.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_kind(mut self, kind: CaptureKind) -> Self {
 self.kind = kind;
 self
 }

 fn matches(&self, q: &str) -> bool {
        if self.image_path.to_lowercase().contains(q) {
            return true;
        }
        if let Some(t) = &self.ocr_text {
            if t.to_lowercase().contains(q) {
                return true;
            }
        }
        self.tags.iter().any(|tag| tag.to_lowercase().contains(q))
    }
}

/// An ordered collection of history entries with search and persistence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryStore {
    entries: Vec<HistoryEntry>,
}

impl HistoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Insert a new entry, or replace an existing one with the same id.
    pub fn upsert(&mut self, entry: HistoryEntry) {
        if let Some(slot) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *slot = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn get(&self, id: Uuid) -> Option<&HistoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn remove(&mut self, id: Uuid) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() != before
    }

    /// Most recent `n` entries, newest first.
    pub fn recent(&self, n: usize) -> Vec<&HistoryEntry> {
        let mut v: Vec<&HistoryEntry> = self.entries.iter().collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v.truncate(n);
        v
    }

    /// Case-insensitive search over path, OCR text, and tags, newest first.
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let mut v: Vec<&HistoryEntry> = self.entries.iter().filter(|e| e.matches(&q)).collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }

    /// All entries of a given kind, newest first.
 pub fn filter_kind(&self, kind: CaptureKind) -> Vec<&HistoryEntry> {
 let mut v: Vec<&HistoryEntry> = self.entries.iter().filter(|e| e.kind == kind).collect();
 v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
 v
 }

 /// Remove entries older than max_age_secs relative to now; returns count pruned.
 pub fn prune_older_than(&mut self, now: i64, max_age_secs: i64) -> usize {
 let cutoff = now - max_age_secs;
 let before = self.entries.len();
 self.entries.retain(|e| e.created_at >= cutoff);
 before - self.entries.len()
 }

 /// One month in seconds: the default capture-history retention window.
 pub const ONE_MONTH_SECS: i64 = 30 * 24 * 60 * 60;

 pub fn save(&self, path: impl AsRef<Path>) -> Result<(), HistoryError> {
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<HistoryStore, HistoryError> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, ts: i64) -> HistoryEntry {
        HistoryEntry::new(path, ts, 100, 100)
    }

    #[test]
    fn upsert_inserts_then_replaces() {
        let mut s = HistoryStore::new();
        let mut e = entry("a.png", 1);
        let id = e.id;
        s.upsert(e.clone());
        assert_eq!(s.len(), 1);
        e.width = 999;
        s.upsert(e);
        assert_eq!(s.len(), 1);
        assert_eq!(s.get(id).unwrap().width, 999);
    }

    #[test]
    fn remove_works() {
        let mut s = HistoryStore::new();
        let e = entry("a.png", 1);
        let id = e.id;
        s.upsert(e);
        assert!(s.remove(id));
        assert!(!s.remove(id));
        assert!(s.is_empty());
    }

    #[test]
    fn recent_is_newest_first_and_truncated() {
        let mut s = HistoryStore::new();
        s.upsert(entry("old.png", 10));
        s.upsert(entry("mid.png", 20));
        s.upsert(entry("new.png", 30));
        let r = s.recent(2);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].image_path, "new.png");
        assert_eq!(r[1].image_path, "mid.png");
    }

    #[test]
    fn search_spans_path_ocr_and_tags() {
        let mut s = HistoryStore::new();
        s.upsert(entry("invoice.png", 30).with_ocr_text("Total due 42"));
        s.upsert(entry("cat.png", 20).with_tags(vec!["Animal".to_string()]));
        s.upsert(entry("misc.png", 10));
        assert_eq!(s.search("invoice").len(), 1);
        assert_eq!(s.search("DUE").len(), 1);
        assert_eq!(s.search("animal").len(), 1);
        assert_eq!(s.search("nothing").len(), 0);
        assert_eq!(s.search("").len(), 0);
        let all = s.search(".png");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].image_path, "invoice.png");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut s = HistoryStore::new();
        s.upsert(entry("a.png", 1).with_ocr_text("hello"));
        s.save(&path).unwrap();
        let loaded = HistoryStore::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.recent(1)[0].ocr_text.as_deref(), Some("hello"));
    }
}

#[cfg(test)]
mod kind_tests {
 use super::*;

 #[test]
 fn kind_defaults_and_back_compat() {
 let e = HistoryEntry::new("a.png", 1, 10, 10);
 assert_eq!(e.kind, CaptureKind::Screenshot);
 let old = "{\"id\":\"00000000-0000-0000-0000-000000000000\",\"image_path\":\"a.png\",\"created_at\":1,\"width\":10,\"height\":10,\"ocr_text\":null,\"tags\":[]}";
 let parsed: HistoryEntry = serde_json::from_str(old).unwrap();
 assert_eq!(parsed.kind, CaptureKind::Screenshot);
 }

 #[test]
 fn filter_by_kind_newest_first() {
 let mut s = HistoryStore::new();
 s.upsert(HistoryEntry::new("a.png", 10, 10, 10).with_kind(CaptureKind::Screenshot));
 s.upsert(HistoryEntry::new("b.mp4", 20, 10, 10).with_kind(CaptureKind::Recording));
 s.upsert(HistoryEntry::new("c.mp4", 30, 10, 10).with_kind(CaptureKind::Recording));
 assert_eq!(s.filter_kind(CaptureKind::Recording).len(), 2);
 assert_eq!(s.filter_kind(CaptureKind::Screenshot).len(), 1);
 assert_eq!(s.filter_kind(CaptureKind::Recording)[0].image_path, "c.mp4");
 }

 #[test]
 fn prune_retention_window() {
 let mut s = HistoryStore::new();
 s.upsert(HistoryEntry::new("old.png", 100, 10, 10));
 s.upsert(HistoryEntry::new("new.png", 1000, 10, 10));
 assert_eq!(s.prune_older_than(1000, 500), 1);
 assert_eq!(s.len(), 1);
 assert_eq!(s.recent(1)[0].image_path, "new.png");
 }
}
