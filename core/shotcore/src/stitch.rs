//! Scroll-capture stitching.
//!
//! Joins a sequence of overlapping vertical scroll frames into one tall image.
//! Consecutive frames are aligned by finding the vertical overlap that best
//! matches the bottom of the upper frame against the top of the lower frame,
//! then the duplicated region is dropped.

use image::RgbaImage;

/// Maximum mean per-channel difference (0 to 255) for rows to count as matching.
const MATCH_THRESHOLD: f64 = 8.0;

/// Detect how many bottom rows of `top` overlap the top rows of `bottom`.
/// Returns 0 when no confident overlap is found.
pub fn detect_overlap(top: &RgbaImage, bottom: &RgbaImage, max_overlap: u32) -> u32 {
    let w = top.width().min(bottom.width());
    if w == 0 {
        return 0;
    }
    let max_o = max_overlap.min(top.height()).min(bottom.height());
    let mut best_o = 0u32;
    let mut best_score = f64::INFINITY;
    for o in 1..=max_o {
        let mut sum = 0u64;
        for r in 0..o {
            let ty = top.height() - o + r;
            for x in 0..w {
                let tp = top.get_pixel(x, ty).0;
                let bp = bottom.get_pixel(x, r).0;
                sum += (tp[0] as i64 - bp[0] as i64).unsigned_abs();
                sum += (tp[1] as i64 - bp[1] as i64).unsigned_abs();
                sum += (tp[2] as i64 - bp[2] as i64).unsigned_abs();
            }
        }
        let score = sum as f64 / (o as f64 * w as f64 * 3.0);
        if score < best_score {
            best_score = score;
            best_o = o;
        }
    }
    if best_score <= MATCH_THRESHOLD {
        best_o
    } else {
        0
    }
}

/// Stitch overlapping vertical frames into a single tall image.
pub fn stitch_vertical(frames: &[RgbaImage], max_overlap: u32) -> Option<RgbaImage> {
    let first = frames.first()?;
    let w = first.width();
    let mut offsets = Vec::with_capacity(frames.len());
    offsets.push(0u32);
    let mut total_h = first.height();
    for i in 1..frames.len() {
        let o = detect_overlap(&frames[i - 1], &frames[i], max_overlap);
        let place = offsets[i - 1] + frames[i - 1].height() - o;
        offsets.push(place);
        total_h = total_h.max(place + frames[i].height());
    }
    let mut out = RgbaImage::new(w, total_h);
    for (i, f) in frames.iter().enumerate() {
        let oy = offsets[i];
        let fw = w.min(f.width());
        for y in 0..f.height() {
            for x in 0..fw {
                out.put_pixel(x, oy + y, *f.get_pixel(x, y));
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn striped(width: u32, rows: &[u8]) -> RgbaImage {
        let mut img = RgbaImage::new(width, rows.len() as u32);
        for (y, &v) in rows.iter().enumerate() {
            for x in 0..width {
                img.put_pixel(x, y as u32, Rgba([v, v, v, 255]));
            }
        }
        img
    }

    #[test]
    fn detect_overlap_finds_known_region() {
        let top = striped(4, &[10, 20, 30, 40]);
        let bottom = striped(4, &[30, 40, 50, 60]);
        assert_eq!(detect_overlap(&top, &bottom, 4), 2);
    }

    #[test]
    fn detect_overlap_zero_when_disjoint() {
        let top = striped(4, &[0, 0, 0, 0]);
        let bottom = striped(4, &[200, 200, 200, 200]);
        assert_eq!(detect_overlap(&top, &bottom, 4), 0);
    }

    #[test]
    fn stitch_reconstructs_source() {
        let source = striped(4, &[10, 20, 30, 40, 50, 60]);
        let top = striped(4, &[10, 20, 30, 40]);
        let bottom = striped(4, &[30, 40, 50, 60]);
        let out = stitch_vertical(&[top, bottom], 4).unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 6);
        assert_eq!(out, source);
    }

    #[test]
    fn single_frame_is_unchanged() {
        let f = striped(3, &[1, 2, 3]);
        let out = stitch_vertical(std::slice::from_ref(&f), 4).unwrap();
        assert_eq!(out, f);
    }

    #[test]
    fn empty_returns_none() {
        let frames: Vec<RgbaImage> = Vec::new();
        assert!(stitch_vertical(&frames, 4).is_none());
    }
}
