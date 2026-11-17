//! Character windows: the scripts' context windows around a match
//! (`SUB_JOINED[max(0, m.start() - 80):m.end() + 80]`).
//!
//! The scripts' look-behinds and look-aheads need no helper: write the
//! script's pattern as is in [`super::text::rx`], which runs look-arounds
//! on the backtracking engine with Python's classes (`pycompat`). The
//! proof is `data/lookaround_corpus.txt` (`tools/gen_markdown_corpus.py`):
//! every complete look-around pattern of `checks/*.sh` at b7c71c5, run by
//! Python 3.10 and 3.13 over the lines of the design documents and edge
//! strings; `text::rx` gives the same matches, spans and groups on every
//! one (`mdproof`).

/// `text[max(0, start - before) : end + after]` in characters, `start` and
/// `end` byte offsets of a match (the scripts' context windows:
/// `SUB_JOINED[max(0, m.start() - 80):m.end() + 80]`). Python clamps both
/// ends; so does this.
pub fn window(text: &str, start: usize, end: usize, before: usize, after: usize) -> &str {
    let lo_char = text[..start].chars().count().saturating_sub(before);
    let lo = text
        .char_indices()
        .nth(lo_char)
        .map_or(text.len(), |(b, _)| b);
    let hi = text[end..]
        .char_indices()
        .nth(after)
        .map_or(text.len(), |(b, _)| end + b);
    &text[lo..hi]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doccheck::text::rx;

    #[test]
    fn look_behind_and_ahead_follow_python() {
        let int = rx(r"(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)");
        let got: Vec<&str> = int
            .find_iter("1 280 a1 2.5 -7 65 507x 12 345")
            .map(|m| m.as_str())
            .collect();
        // Python 3.10/3.13: ["1 280", "65", "12 345"] ("65 507x" backtracks to "65")
        assert_eq!(got, vec!["1 280", "65", "12 345"]);
        let split: Vec<&str> = rx(r"(?<=[.;])\s+").split("a. b; c d.").collect();
        assert_eq!(split, vec!["a.", "b;", "c d."]);
        assert!(rx(r"(?<!not )\bfits\b").is_match("it fits"));
        assert!(!rx(r"(?<!not )\bfits\b").is_match("it does not fits"));
    }

    #[test]
    fn windows_clamp_in_characters() {
        let t = "µµµ 1280 µµ";
        let s = t.find("1280").expect("in the text");
        assert_eq!(window(t, s, s + 4, 2, 2), "µ 1280 µ");
        assert_eq!(window(t, s, s + 4, 80, 80), t);
        assert_eq!(window(t, 0, 0, 3, 0), "");
    }
}
