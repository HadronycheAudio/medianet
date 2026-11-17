//! The reason tokens of the four WP5 scripts: a member named with
//! `\bNAME\b`, and the tokens `\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b` outside the
//! enumeration unless negated (`\b(?:no|not|never|without)\b[^.\n]{0,30}`
//! before the token, `re.I`, anywhere in the same unit).

use std::collections::BTreeSet;

use super::py::{s, si};
use crate::doccheck::text::rx;

/// The members named in `unit` (`\bNAME\b`, case-sensitive).
pub(crate) fn reasons_in(members: &[String], unit: &str) -> Vec<String> {
    members.iter().filter(|m| s(&format!(r"\b{m}\b"), unit)).cloned().collect()
}

/// The reason-looking tokens of the units outside the enumeration, unless a
/// unit negates them (`\b(?:no|not|never|without)\b[^.\n]{0,30}TOKEN`,
/// `re.I`, anywhere in the same unit).
pub(crate) fn non_members(members: &[String], units: &[String]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for unit in units {
        for c in rx(r"\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b").captures_iter(unit) {
            let token = &c[1];
            let negated = si(&format!(r"\b(?:no|not|never|without)\b[^.\n]{{0,30}}{}", regex::escape(token)), unit);
            if !members.iter().any(|m| m == token) && !negated {
                out.insert(token.to_string());
            }
        }
    }
    out
}
