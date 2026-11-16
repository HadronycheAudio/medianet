//! Python-flavoured helpers of the script: number reading, the failure
//! list, and the two ways the script stops early.

use crate::doccheck::text::rx;

/// How a run ends before its report: the script's traceback, or a printed
/// `FAIL` line followed by `sys.exit(1)` (which drops the failures gathered
/// so far).
#[derive(Debug)]
pub(crate) enum Stop {
    /// An uncaught Python exception (a `None` in arithmetic, a missing key).
    Crash(String),
    /// `FAIL <criterion>: <message>` and exit; nothing else is printed.
    Exit(&'static str, String),
}

pub(crate) type R<T> = Result<T, Stop>;

/// An uncaught exception.
pub(crate) fn crash<T>(why: &str) -> R<T> {
    Err(Stop::Crash(why.to_string()))
}

/// The failures the script collects and prints at the end.
#[derive(Debug, Default)]
pub(crate) struct Fails(pub Vec<(&'static str, String)>);

impl Fails {
    pub fn add(&mut self, ac: &'static str, msg: impl Into<String>) {
        self.0.push((ac, msg.into()));
    }
}

const NUM: &str = r"\d{1,3}(?:[ \u{a0}\u{202f}]\d{3})+|\d+";

/// `NUM_RE.search(text)` as the matched text.
pub(crate) fn num_match(text: &str) -> Option<&str> {
    rx(NUM).find(text).map(|m| m.as_str())
}

/// Every `NUM_RE` match.
pub(crate) fn num_matches(text: &str) -> Vec<&str> {
    rx(NUM).find_iter(text).map(|m| m.as_str()).collect()
}

/// `to_int`: digit-group separators and commas removed.
pub(crate) fn to_int(s: &str) -> i64 {
    s.chars()
        .filter(|c| !matches!(c, ' ' | '\u{a0}' | '\u{202f}' | ','))
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// `first_int`: the first number of `text` (digit groups joined).
pub(crate) fn first_int(text: Option<&str>) -> Option<i64> {
    num_match(text?).map(to_int)
}

/// `norm`: whitespace runs squashed to one space (not trimmed).
pub(crate) fn norm(s: &str) -> String {
    rx(r"\s+").replace_all(s, " ").into_owned()
}

/// `re.split(r"(?<=[.;])\s+", text)`.
pub(crate) fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for m in rx(r"[.;]\s+").find_iter(text) {
        out.push(text[start..m.start() + 1].to_string());
        start = m.end();
    }
    out.push(text[start..].to_string());
    out
}

/// `label_of`: markup characters removed, trimmed, lower case.
pub(crate) fn label_of(cell: &str) -> String {
    rx(r"[*_`]").replace_all(cell, "").trim().to_lowercase()
}

const DASHES: [&str; 9] = [
    "", "\u{2014}", "\u{2013}", "-", "none", "omitted", "absent", "n/a", "not sent",
];

/// True when a `-` that no word character precedes is followed by digits
/// (`(?<![\w])-\s*\d`), or a minus sign precedes digits, or `deficit`.
fn negative(s: &str) -> bool {
    if rx(r"(?i)\u{2212}\s*\d|deficit").is_match(s) {
        return true;
    }
    s.char_indices().any(|(i, c)| {
        c == '-'
            && !s[..i].chars().next_back().is_some_and(|p| rx(r"^\w$").is_match(&p.to_string()))
            && rx(r"^\s*\d").is_match(&s[i + 1..])
    })
}

/// `parse_num`: a table cell as a number; a missing cell reads 0.
pub(crate) fn parse_num(cell: Option<&str>) -> Option<i64> {
    let raw = cell.unwrap_or("").trim();
    let low = rx(r"[*_`]").replace_all(raw, "");
    if DASHES.contains(&low.trim().to_lowercase().as_str()) {
        return Some(0);
    }
    let mut s = raw.replace('*', "");
    if let Some((_, tail)) = s.clone().rsplit_once('=') {
        s = tail.to_string();
    }
    let s = rx(r"\([^)]*\)").replace_all(&s, "").into_owned();
    let value = to_int(num_match(&s)?);
    Some(if negative(&s) { -value } else { value })
}
