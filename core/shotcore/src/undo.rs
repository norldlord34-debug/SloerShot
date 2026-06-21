//! Snapshot-based undo and redo.
//!
//! The editor takes a full `Document` snapshot before each committed change.
//! This is simple and robust: every tool gets undo for free, matching the
//! snapshot-based-undo-for-everything promise. History depth is bounded so a
//! long editing session cannot grow without limit.

use crate::model::Document;

/// Default number of undo steps retained.
pub const DEFAULT_LIMIT: usize = 100;

/// A linear undo/redo timeline around a current document state.
pub struct History {
    past: Vec<Document>,
    present: Document,
    future: Vec<Document>,
    limit: usize,
}

impl History {
    /// Start a timeline at the given document with the default depth limit.
    pub fn new(initial: Document) -> Self {
        Self::with_limit(initial, DEFAULT_LIMIT)
    }

    /// Start a timeline with an explicit depth limit (at least 1).
    pub fn with_limit(initial: Document, limit: usize) -> Self {
        Self {
            past: Vec::new(),
            present: initial,
            future: Vec::new(),
            limit: limit.max(1),
        }
    }

    /// The current document state.
    pub fn current(&self) -> &Document {
        &self.present
    }

    /// Replace the current state with `next`, pushing the old state onto the
    /// undo stack and discarding any redo history.
    pub fn commit(&mut self, next: Document) {
        let prev = std::mem::replace(&mut self.present, next);
        self.past.push(prev);
        if self.past.len() > self.limit {
            self.past.remove(0);
        }
        self.future.clear();
    }

    /// Snapshot the current state, apply a mutation, and commit the result.
    pub fn edit<F: FnOnce(&mut Document)>(&mut self, f: F) {
        let mut next = self.present.clone();
        f(&mut next);
        self.commit(next);
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Step back one state. Returns false when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.past.pop() {
            Some(prev) => {
                let cur = std::mem::replace(&mut self.present, prev);
                self.future.push(cur);
                true
            }
            None => false,
        }
    }

    /// Step forward one state. Returns false when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        match self.future.pop() {
            Some(next) => {
                let cur = std::mem::replace(&mut self.present, next);
                self.past.push(cur);
                true
            }
            None => false,
        }
    }

    /// Discard all history and restart the timeline at `doc`.
    pub fn reset(&mut self, doc: Document) {
        self.past.clear();
        self.future.clear();
        self.present = doc;
    }

    pub fn undo_depth(&self) -> usize {
        self.past.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.future.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;
    use crate::model::{Annotation, ShapeKind};

    fn line() -> ShapeKind {
        ShapeKind::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        }
    }

    #[test]
    fn new_history_has_no_undo_or_redo() {
        let h = History::new(Document::new(10, 10));
        assert!(!h.can_undo());
        assert!(!h.can_redo());
        assert_eq!(h.current().len(), 0);
    }

    #[test]
    fn edit_then_undo_then_redo() {
        let mut h = History::new(Document::new(10, 10));
        h.edit(|d| {
            d.add(Annotation::new(line()));
        });
        assert_eq!(h.current().len(), 1);
        assert!(h.can_undo());
        assert!(h.undo());
        assert_eq!(h.current().len(), 0);
        assert!(h.can_redo());
        assert!(h.redo());
        assert_eq!(h.current().len(), 1);
    }

    #[test]
    fn commit_after_undo_clears_redo() {
        let mut h = History::new(Document::new(10, 10));
        h.edit(|d| {
            d.add(Annotation::new(line()));
        });
        h.undo();
        assert!(h.can_redo());
        h.edit(|d| {
            d.add(Annotation::new(line()));
        });
        assert!(!h.can_redo());
        assert_eq!(h.current().len(), 1);
    }

    #[test]
    fn undo_and_redo_on_empty_are_false() {
        let mut h = History::new(Document::new(10, 10));
        assert!(!h.undo());
        assert!(!h.redo());
    }

    #[test]
    fn depth_is_bounded() {
        let mut h = History::with_limit(Document::new(10, 10), 2);
        for _ in 0..5 {
            h.edit(|d| {
                d.add(Annotation::new(line()));
            });
        }
        assert_eq!(h.undo_depth(), 2);
        assert!(h.undo());
        assert!(h.undo());
        assert!(!h.undo());
    }

    #[test]
    fn reset_clears_history() {
        let mut h = History::new(Document::new(10, 10));
        h.edit(|d| {
            d.add(Annotation::new(line()));
        });
        h.reset(Document::new(20, 20));
        assert!(!h.can_undo());
        assert!(!h.can_redo());
        assert_eq!(h.current().image_width, 20);
    }
}
