//! Capture geometry: points, sizes, rectangles, and a multi-monitor virtual
//! desktop model with logical/physical (HiDPI) coordinate mapping.
//!
//! This powers frozen-screen area selection: the native shell grabs a still
//! frame of every display, and this module maps the user drag onto the correct
//! display and converts it to physical pixels on that display.

use serde::{Deserialize, Serialize};

/// A 2D point in logical (device-independent) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Width and height pair.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub w: f64,
    pub h: f64,
}

impl Size {
    pub fn new(w: f64, h: f64) -> Self {
        Self { w, h }
    }
    pub fn area(&self) -> f64 {
        self.w * self.h
    }
}

/// An axis-aligned rectangle whose x and y mark the top-left origin.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Build a rect from two arbitrary corners, handling any drag direction.
    pub fn from_corners(a: Point, b: Point) -> Self {
        Self {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            w: (a.x - b.x).abs(),
            h: (a.y - b.y).abs(),
        }
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn area(&self) -> f64 {
        self.w * self.h
    }
    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }
    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Normalize so width and height are non-negative.
    pub fn normalized(&self) -> Self {
        let x = if self.w < 0.0 {
            self.x + self.w
        } else {
            self.x
        };
        let y = if self.h < 0.0 {
            self.y + self.h
        } else {
            self.y
        };
        Self {
            x,
            y,
            w: self.w.abs(),
            h: self.h.abs(),
        }
    }

    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.right() && p.y >= self.y && p.y < self.bottom()
    }

    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Intersection of two rects, or None when they do not overlap.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right > x && bottom > y {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    /// Smallest rect that contains both inputs.
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }

    /// Clamp this rect so it lies within bounds; empty rect at the bounds origin if disjoint.
    pub fn clamp_to(&self, bounds: &Rect) -> Rect {
        self.normalized()
            .intersection(bounds)
            .unwrap_or(Rect::new(bounds.x, bounds.y, 0.0, 0.0))
    }

    /// Round outward to integer pixel boundaries.
    pub fn round_out(&self) -> Rect {
        let x = self.x.floor();
        let y = self.y.floor();
        Rect::new(x, y, self.right().ceil() - x, self.bottom().ceil() - y)
    }
}

/// A single physical display in the virtual desktop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Display {
    pub id: u32,
    /// Logical bounds within the virtual desktop; the origin may be negative.
    pub bounds: Rect,
    /// HiDPI scale factor where 1.0 is 96dpi and 2.0 is Retina.
    pub scale_factor: f64,
    pub is_primary: bool,
}

impl Display {
    pub fn new(id: u32, bounds: Rect, scale_factor: f64, is_primary: bool) -> Self {
        Self {
            id,
            bounds,
            scale_factor,
            is_primary,
        }
    }

    /// Physical pixel size of this display.
    pub fn physical_size(&self) -> Size {
        Size::new(
            self.bounds.w * self.scale_factor,
            self.bounds.h * self.scale_factor,
        )
    }
}

/// The whole multi-monitor desktop as a single coordinate space.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualDesktop {
    pub displays: Vec<Display>,
}

impl VirtualDesktop {
    pub fn new(displays: Vec<Display>) -> Self {
        Self { displays }
    }

    /// Logical bounding rect over all displays.
    pub fn bounds(&self) -> Rect {
        let mut iter = self.displays.iter();
        let first = match iter.next() {
            Some(d) => d.bounds,
            None => return Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        iter.fold(first, |acc, d| acc.union(&d.bounds))
    }

    pub fn primary(&self) -> Option<&Display> {
        self.displays
            .iter()
            .find(|d| d.is_primary)
            .or_else(|| self.displays.first())
    }

    /// Display whose logical bounds contain the point.
    pub fn display_at_point(&self, p: Point) -> Option<&Display> {
        self.displays.iter().find(|d| d.bounds.contains_point(p))
    }

    /// Display that overlaps the rect the most; used to pick the capture target.
    pub fn dominant_display(&self, r: &Rect) -> Option<&Display> {
        self.displays
            .iter()
            .filter_map(|d| d.bounds.intersection(r).map(|i| (d, i.area())))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(d, _)| d)
    }
}

/// A capture selection resolved to a concrete display and physical pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub display_id: u32,
    /// Logical rect clamped to the chosen display.
    pub logical: Rect,
    /// Physical pixel rect relative to the display origin.
    pub physical: Rect,
    pub scale_factor: f64,
}

/// Resolve a raw drag selection (logical virtual-desktop coordinates) into a
/// concrete capture region: pick the dominant display, clamp to it, and convert
/// to physical pixels relative to that display origin.
pub fn resolve_selection(desktop: &VirtualDesktop, selection: Rect) -> Option<CaptureRegion> {
    let sel = selection.normalized();
    if sel.is_empty() {
        return None;
    }
    let display = desktop.dominant_display(&sel)?;
    let clamped = sel.clamp_to(&display.bounds).round_out();
    if clamped.is_empty() {
        return None;
    }
    let rel_x = clamped.x - display.bounds.x;
    let rel_y = clamped.y - display.bounds.y;
    let s = display.scale_factor;
    let physical = Rect::new(rel_x * s, rel_y * s, clamped.w * s, clamped.h * s).round_out();
    Some(CaptureRegion {
        display_id: display.id,
        logical: clamped,
        physical,
        scale_factor: s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn rect_from_corners_handles_any_direction() {
        let r = Rect::from_corners(Point::new(30.0, 40.0), Point::new(10.0, 10.0));
        assert!(approx(r.x, 10.0) && approx(r.y, 10.0));
        assert!(approx(r.w, 20.0) && approx(r.h, 30.0));
    }

    #[test]
    fn intersection_and_union() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        assert_eq!(a.intersection(&b).unwrap(), Rect::new(5.0, 5.0, 5.0, 5.0));
        assert_eq!(a.union(&b), Rect::new(0.0, 0.0, 15.0, 15.0));
        assert!(a.intersection(&Rect::new(100.0, 100.0, 1.0, 1.0)).is_none());
    }

    #[test]
    fn contains_point_is_half_open() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains_point(Point::new(0.0, 0.0)));
        assert!(r.contains_point(Point::new(9.999, 9.999)));
        assert!(!r.contains_point(Point::new(10.0, 5.0)));
    }

    #[test]
    fn desktop_bounds_with_negative_origin() {
        let d0 = Display::new(0, Rect::new(0.0, 0.0, 1920.0, 1080.0), 1.0, true);
        let d1 = Display::new(1, Rect::new(-1280.0, 0.0, 1280.0, 720.0), 1.0, false);
        let vd = VirtualDesktop::new(vec![d0, d1]);
        assert_eq!(vd.bounds(), Rect::new(-1280.0, 0.0, 3200.0, 1080.0));
    }

    #[test]
    fn display_lookup_and_primary() {
        let d0 = Display::new(0, Rect::new(0.0, 0.0, 1920.0, 1080.0), 1.0, true);
        let d1 = Display::new(1, Rect::new(-1280.0, 0.0, 1280.0, 720.0), 2.0, false);
        let vd = VirtualDesktop::new(vec![d0, d1]);
        assert_eq!(vd.display_at_point(Point::new(-10.0, 10.0)).unwrap().id, 1);
        assert_eq!(vd.display_at_point(Point::new(10.0, 10.0)).unwrap().id, 0);
        assert!(vd.display_at_point(Point::new(5000.0, 5000.0)).is_none());
        assert_eq!(vd.primary().unwrap().id, 0);
    }

    #[test]
    fn resolve_selection_scales_to_physical() {
        let d = Display::new(7, Rect::new(0.0, 0.0, 1440.0, 900.0), 2.0, true);
        let vd = VirtualDesktop::new(vec![d]);
        let region = resolve_selection(&vd, Rect::new(100.0, 50.0, 200.0, 100.0)).unwrap();
        assert_eq!(region.display_id, 7);
        assert_eq!(region.logical, Rect::new(100.0, 50.0, 200.0, 100.0));
        assert_eq!(region.physical, Rect::new(200.0, 100.0, 400.0, 200.0));
        assert!(approx(region.scale_factor, 2.0));
    }

    #[test]
    fn resolve_selection_relative_to_secondary_origin() {
        let d0 = Display::new(0, Rect::new(0.0, 0.0, 1920.0, 1080.0), 1.0, true);
        let d1 = Display::new(1, Rect::new(1920.0, 0.0, 1280.0, 720.0), 2.0, false);
        let vd = VirtualDesktop::new(vec![d0, d1]);
        let region = resolve_selection(&vd, Rect::new(2020.0, 100.0, 100.0, 50.0)).unwrap();
        assert_eq!(region.display_id, 1);
        assert_eq!(region.physical, Rect::new(200.0, 200.0, 200.0, 100.0));
    }

    #[test]
    fn resolve_selection_clamps_and_picks_dominant() {
        let d0 = Display::new(0, Rect::new(0.0, 0.0, 100.0, 100.0), 1.0, true);
        let d1 = Display::new(1, Rect::new(100.0, 0.0, 100.0, 100.0), 1.0, false);
        let vd = VirtualDesktop::new(vec![d0, d1]);
        let region = resolve_selection(&vd, Rect::new(70.0, 10.0, 90.0, 20.0)).unwrap();
        assert_eq!(region.display_id, 1);
        assert_eq!(region.logical, Rect::new(100.0, 10.0, 60.0, 20.0));
    }

    #[test]
    fn empty_selection_is_rejected() {
        let d = Display::new(0, Rect::new(0.0, 0.0, 100.0, 100.0), 1.0, true);
        let vd = VirtualDesktop::new(vec![d]);
        assert!(resolve_selection(&vd, Rect::new(10.0, 10.0, 0.0, 0.0)).is_none());
    }
}

impl Rect {
    /// Expand the rect by `d` on every side (use a negative `d` to shrink).
    pub fn inflate(&self, d: f64) -> Rect {
        Rect::new(self.x - d, self.y - d, self.w + 2.0 * d, self.h + 2.0 * d)
    }

    /// Inclusive point test where points on the border count as inside.
    pub fn contains_point_inclusive(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }
}
