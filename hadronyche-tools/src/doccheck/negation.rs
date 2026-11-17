//! Negation look-backs: is a word negated by what precedes it?
//!
//! Several 004 scripts read a member name or reason token out of an outcome
//! cell and skip it when a negation precedes it, in a window of characters
//! before the match (Python slices by CHARACTER: `outcome[max(0, m.start() -
//! 24):m.start()]`), or anywhere earlier in the text. Each script's window,
//! word list and stop characters is one named [`Lookback`]:
//!
//! | Policy | Script | Python |
//! |---|---|---|
//! | [`Lookback::WINDOW_24_004_16`] | 004-16, 004-17 `named_reasons`; 004-18 `named_in` | `re.search(r"\b(not\|never\|nor\|no\|neither)\b[^\|;.]*$", outcome[max(0, start-24):start], re.I)` |
//! | [`Lookback::WINDOW_25_004_19`] | 004-19 (AC5, every CamelCase outcome token) | `re.search(r"\b(?:never\|not\|nor\|rather than\|instead of)\b[^\|.;]*$", outcome[max(0, start-25):start], re.I)` |
//! | [`Lookback::ANYWHERE_004_16`] | 004-16, 004-17, 004-18 (a CamelCase token not in the enumeration) | `re.search(r"\b(not\|never\|nor)\b[^\|;.]*" + name, outcome)`: case-sensitive, before ANY occurrence of the name |
//!
//! A negation reaches the end of the scope when no stop character follows
//! it, which is what `[^|;.]*$` says; the look-back is written as code (the
//! regex form costs a backtracking engine per word). Proven equal to the
//! scripts' regexes on `data/family004_corpus.txt` (`doccheck::proofs004`).

use super::pycompat::is_word;

/// Where the negation is looked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The `n` characters before the match.
    Window(usize),
    /// Anywhere before an occurrence of the name, in the whole text.
    Anywhere,
}

/// One script's negation look-back: a negation word (whole word, `\b…\b`)
/// with none of the `stops` characters between it and the end of the scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lookback {
    pub name: &'static str,
    pub scope: Scope,
    /// The negation words (a word may hold single spaces: "rather than").
    pub words: &'static [&'static str],
    /// The characters a negation may not reach across.
    pub stops: &'static str,
    /// `re.I`.
    pub any_case: bool,
}

impl Lookback {
    /// `\b(not|never|nor|no|neither)\b[^|;.]*$`, `re.I`, 24 characters.
    pub const WINDOW_24_004_16: Lookback = Lookback {
        name: "window-24-004-16",
        scope: Scope::Window(24),
        words: &["not", "never", "nor", "no", "neither"],
        stops: "|;.",
        any_case: true,
    };
    /// `\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$`, `re.I`, 25
    /// characters.
    pub const WINDOW_25_004_19: Lookback = Lookback {
        name: "window-25-004-19",
        scope: Scope::Window(25),
        words: &["never", "not", "nor", "rather than", "instead of"],
        stops: "|.;",
        any_case: true,
    };
    /// `\b(not|never|nor)\b[^|;.]*` + name, case-sensitive, anywhere.
    pub const ANYWHERE_004_16: Lookback = Lookback {
        name: "anywhere-004-16",
        scope: Scope::Anywhere,
        words: &["not", "never", "nor"],
        stops: "|;.",
        any_case: false,
    };
}

/// True when `seg` holds one of `rule.words` as a whole word. The words are
/// ASCII and `re.I` folds no repertoire character onto an ASCII letter, so
/// an ASCII case fold is Python's here (byte offsets are kept).
fn has_word(seg: &str, rule: &Lookback) -> bool {
    let hay = if rule.any_case {
        seg.to_ascii_lowercase()
    } else {
        seg.to_string()
    };
    let bounded = |at: usize, end: usize| {
        !seg[..at].chars().next_back().is_some_and(is_word)
            && !seg[end..].chars().next().is_some_and(is_word)
    };
    rule.words.iter().any(|w| {
        hay.match_indices(w)
            .any(|(at, m)| bounded(at, at + m.len()))
    })
}

/// The part of `seg` after its last stop character.
fn after_stops<'s>(seg: &'s str, rule: &Lookback) -> &'s str {
    seg.rfind(|c| rule.stops.contains(c))
        .map_or(seg, |k| &seg[k + 1..])
}

/// True when the word of `text` that starts at byte `start` (and reads
/// `word`) is negated under `rule`.
pub fn negated(text: &str, start: usize, word: &str, rule: &Lookback) -> bool {
    match rule.scope {
        Scope::Window(n) => {
            let at = text[..start].chars().count();
            let lo = text
                .char_indices()
                .nth(at.saturating_sub(n))
                .map_or(start, |(i, _)| i);
            has_word(after_stops(&text[lo..start], rule), rule)
        }
        // re.search(prefix + name): a name occurrence (as a substring)
        // with a negation before it, no stop between
        Scope::Anywhere => text
            .match_indices(word)
            .any(|(q, _)| has_word(after_stops(&text[..q], rule), rule)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_count_characters_and_stop_at_punctuation() {
        let t = "not × × × × × × × × BadMagic";
        let at = t.find("BadMagic").unwrap_or_default();
        assert!(negated(t, at, "BadMagic", &Lookback::WINDOW_24_004_16));
        let far = format!("not {} BadMagic", "x".repeat(30));
        let at = far.find("BadMagic").unwrap_or_default();
        assert!(!negated(&far, at, "BadMagic", &Lookback::WINDOW_24_004_16));
        let stop = "not counted; BadMagic";
        assert!(!negated(stop, 13, "BadMagic", &Lookback::WINDOW_24_004_16));
        let r = "rather than TooShort";
        assert!(negated(r, 12, "TooShort", &Lookback::WINDOW_25_004_19));
        assert!(!negated(r, 12, "TooShort", &Lookback::WINDOW_24_004_16));
        assert!(negated(
            "x | not even TooShort",
            0,
            "TooShort",
            &Lookback::ANYWHERE_004_16
        ));
        assert!(!negated(
            "Not TooShort",
            4,
            "TooShort",
            &Lookback::ANYWHERE_004_16
        ));
    }
}
