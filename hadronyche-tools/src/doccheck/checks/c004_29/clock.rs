//! The clock terms of `doc/design/spec/timeline.md` (`clock_terms`): the
//! phrases of up to three words ending in "time" or "clock", cut after the
//! last stop word, without the generic ones.

use crate::doccheck::text::rx;

const STOP: &[&str] = &[
    "a", "an", "the", "every", "of", "on", "its", "this", "one", "whose", "with", "from", "to", "at", "as", "is", "by", "each", "any", "and", "or", "in", "for", "no", "has", "keeps", "all", "such", "same", "that", "when", "be", "it", "only", "their", "which", "if", "but", "not", "into", "over", "under", "during", "after", "before", "does", "do", "must", "may", "can", "instant",
];

const EXCL: [&str; 15] = [
    "time",
    "clock",
    "origin time",
    "exact time",
    "usable time",
    "compile time",
    "published time",
    "representable time",
    "pre-origin time",
    "same time",
    "such time",
    "no time",
    "this time",
    "wall time",
    "gap time",
];

/// `clock_terms(text)`, sorted.
pub(crate) fn clock_terms(text: &str) -> Vec<String> {
    let t = text.to_lowercase().replace('`', "").replace("wall-clock", "wall clock");
    let mut out: Vec<String> = Vec::new();
    for c in rx(r"\b((?:[a-z][a-z']*\s+){0,3}?(?:time|clock))\b").captures_iter(&t) {
        let words: Vec<&str> = c[1].split_whitespace().collect();
        let cut = words[..words.len() - 1].iter().rposition(|w| STOP.contains(w));
        let words = match cut {
            Some(k) => &words[k + 1..],
            None => &words[..],
        };
        if words.len() > 3 || words.iter().any(|w| *w == "s" || w.contains('\'')) {
            continue;
        }
        let term = words.join(" ");
        if !term.is_empty() && !EXCL.contains(&term.as_str()) {
            out.push(term);
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terms() {
        let t = clock_terms("The network time of a module; its local clock and the wall-clock; compile time.");
        assert_eq!(t, vec!["local clock", "network time", "wall clock"]);
    }
}
