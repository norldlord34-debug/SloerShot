//! Auto-tagging: extract candidate tags from recognized text by frequency, skipping
//! short words and common stopwords. Deterministic (alphabetical tie-break).
use std::collections::BTreeMap;

const STOPWORDS: &[&str] = &[
 "the", "and", "for", "that", "this", "with", "you", "your", "are", "was", "but", "not",
 "from", "have", "has", "its", "it", "of", "to", "in", "is", "on", "or", "an", "as", "at",
 "be", "by", "we", "all", "can", "will", "they", "their", "them", "our",
];

/// Return up to `max` lowercase tags ordered by frequency then alphabetically.
pub fn extract_tags(text: &str, max: usize) -> Vec<String> {
 let mut counts: BTreeMap<String, usize> = BTreeMap::new();
 for tok in text.split(|c: char| !c.is_alphanumeric()) {
 if tok.is_empty() {
 continue;
 }
 let w = tok.to_lowercase();
 if w.chars().count() < 3 || STOPWORDS.contains(&w.as_str()) {
 continue;
 }
 *counts.entry(w).or_insert(0) += 1;
 }
 let mut v: Vec<(String, usize)> = counts.into_iter().collect();
 v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
 v.into_iter().take(max).map(|(w, _)| w).collect()
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn frequency_then_alpha() {
 let tags = extract_tags("Invoice invoice total Total due now", 2);
 assert_eq!(tags, vec!["invoice".to_string(), "total".to_string()]);
 }

 #[test]
 fn skips_stopwords_and_short() {
 let tags = extract_tags("the cat is on a mat", 5);
 assert_eq!(tags, vec!["cat".to_string(), "mat".to_string()]);
 }

 #[test]
 fn respects_max() {
 let tags = extract_tags("alpha beta gamma delta", 2);
 assert_eq!(tags.len(), 2);
 }
}
