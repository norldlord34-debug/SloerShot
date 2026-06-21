//! Caption export: turn timed transcript segments into SRT or WebVTT subtitle files
//! for screen recordings.
use serde::{Deserialize, Serialize};

/// A timed transcript segment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
 pub start_ms: u64,
 pub end_ms: u64,
 pub text: String,
}

fn fmt_ts(ms: u64, sep: &str) -> String {
 let h = ms / 3_600_000;
 let m = (ms / 60_000) % 60;
 let s = (ms / 1000) % 60;
 let millis = ms % 1000;
 format!("{:02}:{:02}:{:02}{}{:03}", h, m, s, sep, millis)
}

/// Export segments as SRT (comma decimal separator, 1-based indices).
pub fn to_srt(segs: &[Segment]) -> String {
 let mut out = String::new();
 for (i, seg) in segs.iter().enumerate() {
 out.push_str(&format!(
 "{}\n{} --> {}\n{}\n\n",
 i + 1,
 fmt_ts(seg.start_ms, ","),
 fmt_ts(seg.end_ms, ","),
 seg.text
 ));
 }
 out
}

/// Export segments as WebVTT (dot decimal separator, WEBVTT header).
pub fn to_vtt(segs: &[Segment]) -> String {
 let mut out = String::from("WEBVTT\n\n");
 for seg in segs {
 out.push_str(&format!(
 "{} --> {}\n{}\n\n",
 fmt_ts(seg.start_ms, "."),
 fmt_ts(seg.end_ms, "."),
 seg.text
 ));
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;

 fn seg() -> Vec<Segment> {
 vec![Segment { start_ms: 1000, end_ms: 2000, text: "Hello".to_string() }]
 }

 #[test]
 fn srt_format() {
 assert_eq!(to_srt(&seg()), "1\n00:00:01,000 --> 00:00:02,000\nHello\n\n");
 }

 #[test]
 fn vtt_format() {
 let v = to_vtt(&seg());
 assert!(v.starts_with("WEBVTT\n\n"));
 assert!(v.contains("00:00:01.000 --> 00:00:02.000"));
 }

 #[test]
 fn timestamp_hours_and_millis() {
 assert_eq!(fmt_ts(3_661_234, ","), "01:01:01,234");
 }
}
