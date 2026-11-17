//! What the scripts do with the members in an outcome cell: reason tokens,
//! negation look-backs, the in-unit negation guard, and the per-script
//! "this outcome is not a rejection" patterns. Each variant is named by
//! the scripts that use it and proven on `data/rejection_corpus.txt`.

use super::super::text::{has, has_i, rx};

/// A negation look-back: a token is negated when the `chars` characters
/// before it (Python's `outcome[max(0, start - chars):start]`, in
/// characters) match `pattern` with `re.I`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lookback {
    pub chars: usize,
    pub pattern: &'static str,
}

/// 004-16's and 004-17's `named_reasons`, 004-18's `named_in`: 24
/// characters, `not|never|nor|no|neither`, no `|;.` after the word.
pub const LOOKBACK_004_16: Lookback = Lookback {
    chars: 24,
    pattern: r"\b(not|never|nor|no|neither)\b[^|;.]*$",
};

/// 004-19's and 004-20's inline guard: 25 characters,
/// `never|not|nor|rather than|instead of` (no `no`), no `|.;` after it.
pub const LOOKBACK_004_19: Lookback = Lookback {
    chars: 25,
    pattern: r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$",
};

/// True when the token starting at character `char_start` of `text` is
/// negated under `lb`.
pub fn negated_at(text: &str, char_start: usize, lb: Lookback) -> bool {
    let before: String = text
        .chars()
        .skip(char_start.saturating_sub(lb.chars))
        .take(char_start.min(lb.chars))
        .collect();
    has_i(lb.pattern, &before)
}

/// The tokens of `pattern` in `text` with their starting character index.
fn tokens<'h>(pattern: &str, text: &'h str) -> Vec<(usize, &'h str)> {
    rx(pattern)
        .find_iter(text)
        .map(|m| (text[..m.start()].chars().count(), m.as_str()))
        .collect()
}

/// `named_reasons(outcome)` (004-16, 004-17) and `named_in(outcome)`
/// (004-18, whose `is_member` is the union with the refusal names): every
/// `\b[A-Za-z]+\b` word that is a member and not negated under `lb`, in
/// order, repeats kept.
pub fn named_reasons(outcome: &str, is_member: impl Fn(&str) -> bool, lb: Lookback) -> Vec<String> {
    tokens(r"\b[A-Za-z]+\b", outcome)
        .into_iter()
        .filter(|(at, w)| is_member(w) && !negated_at(outcome, *at, lb))
        .map(|(_, w)| w.to_string())
        .collect()
}

/// `\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b`: 004-06/08/09/10's validity-unit
/// guard, 004-16's `CAMEL`, 004-19's outcome tokens, 004-20's `NAME_RE`.
pub const CAMEL_LETTERS: &str = r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b";
/// `\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b`: 004-25, 004-28, 004-29.
pub const CAMEL_DIGITS: &str = r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b";
/// `\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b`: 004-18's C-185 reading.
pub const CAMEL_REFUSAL_004_18: &str = r"\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b";

/// The tokens of `pattern` (one of the `CAMEL_*` constants) in `text`
/// negated under `lb`, in order (004-19, 004-20: a non-member token is
/// excused when negated; a member counts only when not).
pub fn negated_tokens(text: &str, pattern: &str, lb: Lookback) -> Vec<String> {
    tokens(pattern, text)
        .into_iter()
        .filter(|(at, _)| negated_at(text, *at, lb))
        .map(|(_, w)| w.to_string())
        .collect()
}

/// 004-06/08/09/10: the `CAMEL_LETTERS` tokens of `unit` that are not
/// members and that no `\b(?:no|not|never|without)\b[^.\n]{0,30}TOKEN`
/// negates anywhere in the same unit (`re.I`, so the token too matches in
/// any case).
pub fn unnegated_non_members(unit: &str, is_member: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (_, token) in tokens(r"\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b", unit) {
        let guard = format!(
            r"\b(?:no|not|never|without)\b[^.\n]{{0,30}}{}",
            regex::escape(token)
        );
        if !is_member(token) && !has_i(&guard, unit) && !out.iter().any(|t| t == token) {
            out.push(token.to_string());
        }
    }
    out
}

/// The per-script "this outcome is not a rejection" patterns, each with
/// `re.I` and searched (`re.search`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonRejection(pub &'static str);

/// 004-16, 004-17.
pub const NON_REJECTION_004_16: NonRejection = NonRejection(
    r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|cannot\s+occur|ignored",
);
/// 004-18 (adds `never a miss`, `not a refusal`).
pub const NON_REJECTION_004_18: NonRejection = NonRejection(
    r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|cannot\s+occur|ignored|never\s+a\s+miss|not\s+a\s+refusal",
);
/// 004-20 (no `parse`, adds a bare `not counted`).
pub const NON_REJECTION_004_20: NonRejection = NonRejection(
    r"not\s+a\s+rejection|never\s+a\s+rejection|cannot\s+occur|ignored|is\s+not\s+counted|not\s+counted",
);
/// 004-24 (`NONREJ_RE`: no `no reason`).
pub const NON_REJECTION_004_24: NonRejection =
    NonRejection(r"not a rejection|never a rejection|\bignored\b|\bvalid\b");
/// 004-25, 004-28 (`NONREJ_RE`).
pub const NON_REJECTION_004_25: NonRejection =
    NonRejection(r"not a rejection|never a rejection|\bignored\b|\bvalid\b|no reason");
/// 004-29 (`NONREJ_RE`: no `\bvalid\b`).
pub const NON_REJECTION_004_29: NonRejection =
    NonRejection(r"not a rejection|never a rejection|\bignored\b|no reason");

impl NonRejection {
    /// `NON_REJECTION.search(text)` as a boolean.
    pub fn is_match(self, text: &str) -> bool {
        has(&format!("(?i){}", self.0), text)
    }
}
