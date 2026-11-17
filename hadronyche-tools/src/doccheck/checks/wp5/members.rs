//! The 13-name enumeration as the four WP5 scripts compute it, and the
//! section lookup they share.
//!
//! **Header and code disagree** in all four scripts: their headers say "the
//! closed 13-name enumeration"; the code starts from the 13 names and
//! replaces them with the names found in the Rejection reasons tables when
//! at least 10 are found (004-06:650-653, 004-08:783-784, 004-09:645-646,
//! 004-10:605-606). The ports follow the code.

use super::md::{parse_tables, section_of};
use super::py::{py_strip, si};

/// The 13 names the script starts from (004-06:624-628).
pub(crate) const DEFAULT_MEMBERS: [&str; 13] = [
    "TooShort",
    "BadMagic",
    "VersionMismatch",
    "ForeignDomain",
    "UnconfiguredDomain",
    "UnknownStream",
    "UnadmittedSource",
    "BadLength",
    "BadChecksum",
    "BadPacketIndex",
    "WrongEpoch",
    "OversizeControl",
    "ReservedNonZero",
];

/// The section of the first heading whose title matches `pattern` (`re.I`).
pub(crate) fn find_section(lines: &[&str], heads: &[(usize, usize, String)], pattern: &str) -> Option<String> {
    let k = heads.iter().position(|h| si(pattern, &h.2))?;
    Some(section_of(lines, heads, k))
}

/// `MEMBERS`: the 13 names, replaced by those found in the Rejection reasons
/// tables (cells that are one capitalised name among the 13) when at least
/// 10 are found.
pub(crate) fn members(lines: &[&str], heads: &[(usize, usize, String)]) -> Vec<String> {
    let mut out: Vec<String> = DEFAULT_MEMBERS.iter().map(|s| s.to_string()).collect();
    let Some(rej) = find_section(lines, heads, r"rejection[-\s]+reasons?") else {
        return out;
    };
    let mut found = std::collections::BTreeSet::new();
    for (_, _, rows) in parse_tables(&rej) {
        for cell in rows.iter().flatten() {
            if let Some(c) = crate::doccheck::text::rx(r"^[`*_\s]*([A-Z][A-Za-z0-9]+)[`*_\s]*$").captures(py_strip(cell))
                && DEFAULT_MEMBERS.contains(&&c[1])
            {
                found.insert(c[1].to_string());
            }
        }
    }
    if found.len() >= 10 {
        out = found.into_iter().collect();
    }
    out
}
