//! Shared annotation-editor controller.
//!
//! A platform-independent interaction state machine that both native canvases
//! (WinUI 3 and SwiftUI) drive identically: tool selection, drag-to-draw, click-
//! to-place text and counters, select/move/resize via handles, delete, reorder,
//! and undo/redo. It composes the model, the undo engine, hit-testing, and the
//! transforms so the two apps share one source of truth for editing behavior.

use crate::geometry::{Point, Rect};
use crate::hit::{set_bounds, shape_bounds, topmost_at, translate};
use crate::model::{Annotation, Document, RedactStyle, ShapeKind, ShapeStyle};
use crate::undo::History;
use uuid::Uuid;

const HANDLE_TOL: f64 = 8.0;
const MIN_DRAG: f64 = 3.0;

/// The active editing tool. Select manipulates existing shapes; the rest create them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Arrow,
    Rectangle,
    Ellipse,
    Line,
    Freehand,
    Text,
    Counter,
    Highlighter,
    Redact,
}

/// The eight resize handles around a selection bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

enum Interaction {
    None,
    Drawing {
        draft: Annotation,
        start: Point,
    },
    Moving {
        id: Uuid,
        orig: ShapeKind,
        start: Point,
        preview: ShapeKind,
    },
    Resizing {
        id: Uuid,
        orig_bounds: Rect,
        orig: ShapeKind,
        handle: Handle,
        preview: ShapeKind,
    },
}

/// Stateful editor over a document with full undo/redo.
pub struct Editor {
    history: History,
    tool: Tool,
    style: ShapeStyle,
    selection: Option<Uuid>,
    interaction: Interaction,
}

impl Editor {
    pub fn new(width: u32, height: u32) -> Self {
        Self::with_document(Document::new(width, height))
    }

    pub fn with_document(doc: Document) -> Self {
        Self {
            history: History::new(doc),
            tool: Tool::Select,
            style: ShapeStyle::default(),
            selection: None,
            interaction: Interaction::None,
        }
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.interaction = Interaction::None;
        if tool != Tool::Select {
            self.selection = None;
        }
    }

    pub fn style(&self) -> ShapeStyle {
        self.style
    }

    /// Set the active style, also applying it to the current selection (undoable).
    pub fn set_style(&mut self, style: ShapeStyle) {
        self.style = style;
        if let Some(id) = self.selection {
            self.history.edit(|d| {
                if let Some(a) = d.get_mut(id) {
                    a.style = style;
                }
            });
        }
    }

    pub fn selection(&self) -> Option<Uuid> {
        self.selection
    }

    pub fn selected(&self) -> Option<&Annotation> {
        self.selection.and_then(|id| self.history.current().get(id))
    }

    /// The committed document, without any in-progress preview.
    pub fn document(&self) -> &Document {
        self.history.current()
    }

    /// The document to display: committed content plus any live draft or transform.
    pub fn render(&self) -> Document {
        let mut doc = self.history.current().clone();
        match &self.interaction {
            Interaction::None => {}
            Interaction::Drawing { draft, .. } => {
                let mut d = draft.clone();
                d.z = i32::MAX;
                doc.annotations.push(d);
            }
            Interaction::Moving { id, preview, .. } | Interaction::Resizing { id, preview, .. } => {
                if let Some(a) = doc.get_mut(*id) {
                    a.kind = preview.clone();
                }
            }
        }
        doc
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        self.interaction = Interaction::None;
        let ok = self.history.undo();
        self.ensure_selection_valid();
        ok
    }

    pub fn redo(&mut self) -> bool {
        self.interaction = Interaction::None;
        let ok = self.history.redo();
        self.ensure_selection_valid();
        ok
    }

    fn ensure_selection_valid(&mut self) {
        if let Some(id) = self.selection {
            if self.history.current().get(id).is_none() {
                self.selection = None;
            }
        }
    }

    /// Delete the selected annotation (undoable). Returns true if something was removed.
    pub fn delete_selected(&mut self) -> bool {
        if let Some(id) = self.selection {
            self.history.edit(|d| {
                d.remove(id);
            });
            self.selection = None;
            true
        } else {
            false
        }
    }

    pub fn bring_selection_to_front(&mut self) -> bool {
        if let Some(id) = self.selection {
            self.history.edit(|d| {
                d.bring_to_front(id);
            });
            true
        } else {
            false
        }
    }

    pub fn send_selection_to_back(&mut self) -> bool {
        if let Some(id) = self.selection {
            self.history.edit(|d| {
                d.send_to_back(id);
            });
            true
        } else {
            false
        }
    }

    /// Set the text of the selected Text annotation (used after click-to-place).
    pub fn set_selected_text(&mut self, text: impl Into<String>) -> bool {
        let text = text.into();
        let Some(id) = self.selection else {
            return false;
        };
        let mut done = false;
        self.history.edit(|d| {
            if let Some(a) = d.get_mut(id) {
                if let ShapeKind::Text { content, .. } = &mut a.kind {
                    *content = text;
                    done = true;
                }
            }
        });
        done
    }

    /// Bounding box of the current selection in image coordinates.
    pub fn selection_bounds(&self) -> Option<Rect> {
        self.selected().map(|a| shape_bounds(&a.kind))
    }

    /// The eight resize handles of the current selection, if any.
    pub fn selection_handles(&self) -> Option<Vec<(Handle, Point)>> {
        self.selection_bounds().map(|b| handle_points(&b).to_vec())
    }
}

impl Editor {
    /// Begin an interaction at `p` (mouse/touch down).
    pub fn pointer_down(&mut self, p: Point) {
        match self.tool {
            Tool::Select => self.begin_select_or_transform(p),
            Tool::Text => self.place_text(p),
            Tool::Counter => self.place_counter(p),
            _ => {
                if let Some(draft) = new_draft(self.tool, p, self.style) {
                    self.interaction = Interaction::Drawing { draft, start: p };
                }
            }
        }
    }

    /// Update the in-progress interaction as the pointer moves.
    pub fn pointer_drag(&mut self, p: Point) {
        let tool = self.tool;
        match &mut self.interaction {
            Interaction::Drawing { draft, start } => {
                update_draft_kind(tool, &mut draft.kind, *start, p)
            }
            Interaction::Moving {
                orig,
                start,
                preview,
                ..
            } => {
                *preview = translate(orig, p.x - start.x, p.y - start.y);
            }
            Interaction::Resizing {
                orig,
                orig_bounds,
                handle,
                preview,
                ..
            } => {
                *preview = set_bounds(orig, resize_bounds(*orig_bounds, *handle, p));
            }
            Interaction::None => {}
        }
    }

    /// Finish the interaction at `p` (mouse/touch up), committing one undo step.
    pub fn pointer_up(&mut self, p: Point) {
        let tool = self.tool;
        match std::mem::replace(&mut self.interaction, Interaction::None) {
            Interaction::None => {}
            Interaction::Drawing { mut draft, start } => {
                update_draft_kind(tool, &mut draft.kind, start, p);
                if keepable(&draft.kind) {
                    let id = draft.id;
                    self.history.edit(move |d| {
                        d.add(draft);
                    });
                    self.selection = Some(id);
                }
            }
            Interaction::Moving {
                id, orig, preview, ..
            } => {
                if preview != orig {
                    self.history.edit(move |d| {
                        if let Some(a) = d.get_mut(id) {
                            a.kind = preview;
                        }
                    });
                }
            }
            Interaction::Resizing {
                id, orig, preview, ..
            } => {
                if preview != orig {
                    self.history.edit(move |d| {
                        if let Some(a) = d.get_mut(id) {
                            a.kind = preview;
                        }
                    });
                }
            }
        }
    }

    fn begin_select_or_transform(&mut self, p: Point) {
        if let Some(id) = self.selection {
            if let Some(b) = self.selection_bounds() {
                if let Some(handle) = handle_at(&b, p) {
                    if let Some(a) = self.history.current().get(id) {
                        self.interaction = Interaction::Resizing {
                            id,
                            orig_bounds: b,
                            orig: a.kind.clone(),
                            handle,
                            preview: a.kind.clone(),
                        };
                        return;
                    }
                }
            }
        }
        match topmost_at(self.history.current(), p, HANDLE_TOL) {
            Some(id) => {
                self.selection = Some(id);
                if let Some(a) = self.history.current().get(id) {
                    self.interaction = Interaction::Moving {
                        id,
                        orig: a.kind.clone(),
                        start: p,
                        preview: a.kind.clone(),
                    };
                }
            }
            None => {
                self.selection = None;
                self.interaction = Interaction::None;
            }
        }
    }

    fn place_text(&mut self, p: Point) {
        let style = self.style;
        let ann = Annotation::new(ShapeKind::Text {
            position: p,
            content: String::from("Text"),
            font_size: 28.0,
        })
        .with_style(style);
        let id = ann.id;
        self.history.edit(move |d| {
            d.add(ann);
        });
        self.selection = Some(id);
        self.tool = Tool::Select;
        self.interaction = Interaction::None;
    }

    fn place_counter(&mut self, p: Point) {
        let style = self.style;
        let mut id = Uuid::nil();
        self.history.edit(|d| {
            let n = d.next_counter_number();
            let ann = Annotation::new(ShapeKind::Counter {
                center: p,
                radius: 18.0,
                number: n,
            })
            .with_style(style);
            id = ann.id;
            d.add(ann);
        });
        self.selection = Some(id);
        self.interaction = Interaction::None;
    }
}

fn new_draft(tool: Tool, p: Point, style: ShapeStyle) -> Option<Annotation> {
    let kind = match tool {
        Tool::Arrow => ShapeKind::Arrow { from: p, to: p },
        Tool::Line => ShapeKind::Line { from: p, to: p },
        Tool::Rectangle => ShapeKind::Rectangle {
            rect: Rect::new(p.x, p.y, 0.0, 0.0),
            corner_radius: 0.0,
        },
        Tool::Ellipse => ShapeKind::Ellipse {
            rect: Rect::new(p.x, p.y, 0.0, 0.0),
        },
        Tool::Highlighter => ShapeKind::Highlighter {
            rect: Rect::new(p.x, p.y, 0.0, 0.0),
        },
        Tool::Redact => ShapeKind::Redact {
            rect: Rect::new(p.x, p.y, 0.0, 0.0),
            style: RedactStyle::Blur,
            strength: 12,
        },
        Tool::Freehand => ShapeKind::Freehand { points: vec![p] },
        _ => return None,
    };
    Some(Annotation::new(kind).with_style(style))
}

fn update_draft_kind(tool: Tool, kind: &mut ShapeKind, start: Point, p: Point) {
    match tool {
        Tool::Arrow => *kind = ShapeKind::Arrow { from: start, to: p },
        Tool::Line => *kind = ShapeKind::Line { from: start, to: p },
        Tool::Rectangle => {
            let cr = if let ShapeKind::Rectangle { corner_radius, .. } = kind {
                *corner_radius
            } else {
                0.0
            };
            *kind = ShapeKind::Rectangle {
                rect: Rect::from_corners(start, p),
                corner_radius: cr,
            };
        }
        Tool::Ellipse => {
            *kind = ShapeKind::Ellipse {
                rect: Rect::from_corners(start, p),
            }
        }
        Tool::Highlighter => {
            *kind = ShapeKind::Highlighter {
                rect: Rect::from_corners(start, p),
            }
        }
        Tool::Redact => {
            let (style, strength) = if let ShapeKind::Redact {
                style, strength, ..
            } = kind
            {
                (*style, *strength)
            } else {
                (RedactStyle::Blur, 12)
            };
            *kind = ShapeKind::Redact {
                rect: Rect::from_corners(start, p),
                style,
                strength,
            };
        }
        Tool::Freehand => {
            if let ShapeKind::Freehand { points } = kind {
                points.push(p);
            }
        }
        _ => {}
    }
}

fn keepable(kind: &ShapeKind) -> bool {
    match kind {
        ShapeKind::Arrow { from, to } | ShapeKind::Line { from, to } => {
            ((from.x - to.x).powi(2) + (from.y - to.y).powi(2)).sqrt() >= MIN_DRAG
        }
        ShapeKind::Freehand { points } => points.len() >= 2,
        ShapeKind::Rectangle { rect, .. }
        | ShapeKind::Ellipse { rect }
        | ShapeKind::Highlighter { rect }
        | ShapeKind::Redact { rect, .. } => rect.w.abs() >= MIN_DRAG && rect.h.abs() >= MIN_DRAG,
        _ => true,
    }
}

fn handle_points(b: &Rect) -> [(Handle, Point); 8] {
    let (l, r, t, bot) = (b.x, b.right(), b.y, b.bottom());
    let cx = b.x + b.w / 2.0;
    let cy = b.y + b.h / 2.0;
    [
        (Handle::TopLeft, Point::new(l, t)),
        (Handle::Top, Point::new(cx, t)),
        (Handle::TopRight, Point::new(r, t)),
        (Handle::Right, Point::new(r, cy)),
        (Handle::BottomRight, Point::new(r, bot)),
        (Handle::Bottom, Point::new(cx, bot)),
        (Handle::BottomLeft, Point::new(l, bot)),
        (Handle::Left, Point::new(l, cy)),
    ]
}

fn handle_at(b: &Rect, p: Point) -> Option<Handle> {
    handle_points(b).into_iter().find_map(|(h, hp)| {
        if (hp.x - p.x).abs() <= HANDLE_TOL && (hp.y - p.y).abs() <= HANDLE_TOL {
            Some(h)
        } else {
            None
        }
    })
}

fn resize_bounds(orig: Rect, handle: Handle, p: Point) -> Rect {
    let mut x0 = orig.x;
    let mut y0 = orig.y;
    let mut x1 = orig.right();
    let mut y1 = orig.bottom();
    match handle {
        Handle::Left | Handle::TopLeft | Handle::BottomLeft => x0 = p.x,
        Handle::Right | Handle::TopRight | Handle::BottomRight => x1 = p.x,
        _ => {}
    }
    match handle {
        Handle::Top | Handle::TopLeft | Handle::TopRight => y0 = p.y,
        Handle::Bottom | Handle::BottomLeft | Handle::BottomRight => y1 = p.y,
        _ => {}
    }
    Rect::from_corners(Point::new(x0, y0), Point::new(x1, y1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Color;

    fn ed() -> Editor {
        Editor::new(800, 600)
    }

    #[test]
    fn draw_rectangle_commits_on_pointer_up() {
        let mut e = ed();
        e.set_tool(Tool::Rectangle);
        e.pointer_down(Point::new(10.0, 10.0));
        e.pointer_drag(Point::new(110.0, 60.0));
        assert_eq!(e.render().len(), 1);
        assert_eq!(e.document().len(), 0);
        e.pointer_up(Point::new(110.0, 60.0));
        assert_eq!(e.document().len(), 1);
        assert!(e.selection().is_some());
        assert!(e.can_undo());
    }

    #[test]
    fn tiny_drag_is_discarded() {
        let mut e = ed();
        e.set_tool(Tool::Rectangle);
        e.pointer_down(Point::new(10.0, 10.0));
        e.pointer_drag(Point::new(11.0, 11.0));
        e.pointer_up(Point::new(11.0, 11.0));
        assert_eq!(e.document().len(), 0);
    }

    #[test]
    fn select_and_move() {
        let mut e = ed();
        e.set_tool(Tool::Rectangle);
        e.set_style(ShapeStyle {
            fill: Some(Color::BLUE),
            ..ShapeStyle::default()
        });
        e.pointer_down(Point::new(100.0, 100.0));
        e.pointer_drag(Point::new(200.0, 200.0));
        e.pointer_up(Point::new(200.0, 200.0));
        let id = e.selection().unwrap();
        e.set_tool(Tool::Select);
        e.pointer_down(Point::new(150.0, 150.0));
        assert_eq!(e.selection(), Some(id));
        e.pointer_drag(Point::new(200.0, 150.0));
        e.pointer_up(Point::new(200.0, 150.0));
        let b = e.selection_bounds().unwrap();
        assert!((b.x - 150.0).abs() < 1e-6);
        assert!(e.can_undo());
        e.undo();
        let b2 = e.selection_bounds().unwrap();
        assert!((b2.x - 100.0).abs() < 1e-6);
    }

    #[test]
    fn resize_via_handle() {
        let mut e = ed();
        e.set_tool(Tool::Rectangle);
        e.set_style(ShapeStyle {
            fill: Some(Color::BLUE),
            ..ShapeStyle::default()
        });
        e.pointer_down(Point::new(0.0, 0.0));
        e.pointer_drag(Point::new(100.0, 100.0));
        e.pointer_up(Point::new(100.0, 100.0));
        e.set_tool(Tool::Select);
        e.pointer_down(Point::new(50.0, 50.0));
        e.pointer_up(Point::new(50.0, 50.0));
        assert!(e.selection().is_some());
        e.pointer_down(Point::new(100.0, 100.0));
        e.pointer_drag(Point::new(150.0, 120.0));
        e.pointer_up(Point::new(150.0, 120.0));
        let b = e.selection_bounds().unwrap();
        assert!((b.w - 150.0).abs() < 1e-6);
        assert!((b.h - 120.0).abs() < 1e-6);
    }

    #[test]
    fn place_text_and_set_content() {
        let mut e = ed();
        e.set_tool(Tool::Text);
        e.pointer_down(Point::new(40.0, 40.0));
        assert_eq!(e.document().len(), 1);
        assert!(e.selection().is_some());
        assert!(e.set_selected_text("Hello"));
        let a = e.selected().unwrap();
        if let ShapeKind::Text { content, .. } = &a.kind {
            assert_eq!(content, "Hello");
        } else {
            panic!("expected text");
        }
    }

    #[test]
    fn counters_increment() {
        let mut e = ed();
        e.set_tool(Tool::Counter);
        e.pointer_down(Point::new(10.0, 10.0));
        e.pointer_down(Point::new(60.0, 10.0));
        assert_eq!(e.document().len(), 2);
        let numbers: Vec<u32> = e
            .document()
            .annotations
            .iter()
            .filter_map(|a| {
                if let ShapeKind::Counter { number, .. } = &a.kind {
                    Some(*number)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(numbers, vec![1, 2]);
    }

    #[test]
    fn delete_and_undo() {
        let mut e = ed();
        e.set_tool(Tool::Counter);
        e.pointer_down(Point::new(10.0, 10.0));
        assert_eq!(e.document().len(), 1);
        e.set_tool(Tool::Select);
        e.pointer_down(Point::new(10.0, 10.0));
        e.pointer_up(Point::new(10.0, 10.0));
        assert!(e.selection().is_some());
        assert!(e.delete_selected());
        assert_eq!(e.document().len(), 0);
        e.undo();
        assert_eq!(e.document().len(), 1);
    }

    #[test]
    fn z_order_commands() {
        let mut e = ed();
        e.set_tool(Tool::Rectangle);
        e.set_style(ShapeStyle {
            fill: Some(Color::RED),
            ..ShapeStyle::default()
        });
        e.pointer_down(Point::new(0.0, 0.0));
        e.pointer_drag(Point::new(100.0, 100.0));
        e.pointer_up(Point::new(100.0, 100.0));
        let first = e.selection().unwrap();
        e.pointer_down(Point::new(0.0, 0.0));
        e.pointer_drag(Point::new(100.0, 100.0));
        e.pointer_up(Point::new(100.0, 100.0));
        let second = e.selection().unwrap();
        let order: Vec<_> = e.document().render_order().iter().map(|a| a.id).collect();
        assert_eq!(order, vec![first, second]);
        e.set_tool(Tool::Select);
        e.pointer_down(Point::new(50.0, 50.0));
        e.pointer_up(Point::new(50.0, 50.0));
        assert_eq!(e.selection(), Some(second));
        e.send_selection_to_back();
        let order2: Vec<_> = e.document().render_order().iter().map(|a| a.id).collect();
        assert_eq!(order2, vec![second, first]);
    }
}
