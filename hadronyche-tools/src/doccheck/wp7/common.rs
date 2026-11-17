//! The helpers several WP7 ports share (004-16 to 004-20), each written
//! once with the semantics the scripts share: constants rows, row names,
//! sentence splits, member reasons with the 24-character look-back, the
//! Python value helpers over the subset TOML reader, prose words and open
//! items, invalid-form rows, the 004-19/004-20 blocks. A candidate for the
//! library (see the WP7 report); no port imports another port.

use super::md::{clean_start, parse_tables};
use super::py;
use super::pytoml::{self, Fail, PyDict, PyVal};
use crate::doccheck::Report;
use crate::doccheck::wp7::rejection_members::Enumeration;
use crate::doccheck::wire_toml::WIRE_TOML;

/// `NUM`: a number with optional digit groups (space or no-break space).
pub(crate) const NUM: &str = r"(?:\d{1,3}(?:[ \u{a0}]\d{3})+|\d+)";

/// `to_int`: digit groups joined.
pub(crate) fn to_int(s: &str) -> Option<i128> {
    py::int(&s.replace([' ', '\u{a0}'], ""))
}

/// `const_val(cid)`: the third cell of the row starting `| cid |`.
pub(crate) fn const_val(text: &str, cid: &str) -> String {
    let lead = format!("| {cid} |");
    for line in py::splitlines(text) {
        if line.starts_with(&lead) {
            let cells: Vec<&str> = line.split('|').collect();
            return cells
                .get(3)
                .map_or(String::new(), |c| py::strip(c).to_string());
        }
    }
    String::new()
}

/// `const_int(cid, pattern)` (any case).
pub(crate) fn const_int(text: &str, cid: &str, pattern: &str) -> Option<i128> {
    let v = const_val(text, cid);
    py::search(pattern, &v, true).and_then(|m| to_int(m.g(1)))
}

/// `md_name(meaning)`: the leading words of a Meaning cell.
pub(crate) fn md_name(meaning: &str) -> &str {
    py::matches(r"\s*([A-Za-z][A-Za-z ]*[A-Za-z]|[A-Za-z])", meaning, false)
        .map_or(meaning, |m| m.g(1))
}

/// `norm_name(text)`.
pub(crate) fn norm_name(text: &str) -> String {
    py::strip(&py::sub(
        r"[\s_]+",
        " ",
        &text.replace('-', " ").to_lowercase(),
        false,
    ))
    .to_string()
}

/// `re.split(r"(?<=[.;])\s+", text)`.
pub(crate) fn split_sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut prev: Option<char> = None;
    let mut it = text.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if py::is_space(c) && matches!(prev, Some('.' | ';')) {
            out.push(&text[start..i]);
            let mut end = i + c.len_utf8();
            while let Some(&(j, d)) = it.peek() {
                if !py::is_space(d) {
                    break;
                }
                end = j + d.len_utf8();
                it.next();
            }
            start = end;
            prev = text[..end].chars().next_back();
            continue;
        }
        prev = Some(c);
    }
    out.push(&text[start..]);
    out
}

/// `named_reasons(outcome)`: the member names of the outcome not negated
/// by `not|never|nor|no|neither` in the 24 characters before them.
pub(crate) fn named_reasons(outcome: &str, members: &Enumeration) -> Vec<String> {
    let mut found = Vec::new();
    for m in py::finditer(r"\b[A-Za-z]+\b", outcome, false) {
        let w = m.all();
        if !members.contains(w) {
            continue;
        }
        let at = py::char_index(outcome, m.start);
        let before = py::slice(outcome, at.saturating_sub(24), at);
        if py::has_i(r"\b(not|never|nor|no|neither)\b[^|;.]*$", before) {
            continue;
        }
        found.push(w.to_string());
    }
    found
}

/// A running Python sum: an int until a float joins it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Num {
    I(i128),
    F(f64),
}

impl Num {
    /// `self + v` for a numeric value; `None` where Python raises
    /// `TypeError` (a str, list or dict joins the sum).
    pub fn add(self, v: &PyVal) -> Option<Num> {
        Some(match (self, v) {
            (Num::I(a), PyVal::Int(_) | PyVal::Bool(_)) => Num::I(a.saturating_add(v.as_int()?)),
            (Num::I(a), PyVal::Float(f)) => Num::F(a as f64 + f),
            (Num::F(a), _) => Num::F(a + v.number()?),
            _ => return None,
        })
    }

    /// Python `self != n`.
    pub fn ne_int(self, n: i128) -> bool {
        match self {
            Num::I(a) => a != n,
            Num::F(a) => a != n as f64,
        }
    }
}

/// `a * b` for two values; `None` unless both are numbers (a str or list
/// times an int is a str or list, which the sum then rejects).
pub(crate) fn mul(a: &PyVal, b: &PyVal) -> Option<PyVal> {
    match (a.as_int(), b.as_int()) {
        (Some(x), Some(y)) => Some(PyVal::Int(x.saturating_mul(y))),
        _ => Some(PyVal::Float(a.number()? * b.number()?)),
    }
}

/// `d.get(key, default)` with an int default.
pub(crate) fn get_or(d: &PyDict, key: &str, default: i128) -> PyVal {
    d.get(key).cloned().unwrap_or(PyVal::Int(default))
}

/// `d.get(key)` compared with `!=` against an int (a missing key is `None`,
/// unequal to every int).
pub(crate) fn ne(d: &PyDict, key: &str, n: i128) -> bool {
    !d.get(key).is_some_and(|v| v.eq_int(n))
}

/// `d.get(key) != text` (None is unequal).
pub(crate) fn ne_str(d: &PyDict, key: &str, text: &str) -> bool {
    !d.get(key).is_some_and(|v| v.as_str() == Some(text))
}

/// `str(d.get(key, ""))`.
pub(crate) fn str_of(d: &PyDict, key: &str) -> String {
    d.get(key).map_or(String::new(), PyVal::str)
}

/// Reads wire.toml; `None` after reporting a missing file, an error or a
/// crash.
pub(crate) fn read(text: Option<&str>, ac: &str, report: &mut Report) -> Option<PyDict> {
    let Some(text) = text else {
        report.fail(ac, format!("{WIRE_TOML} does not exist"));
        return None;
    };
    match pytoml::parse(text) {
        Ok(d) => Some(d),
        Err(Fail::Error(e)) => {
            report.fail(ac, format!("{WIRE_TOML} does not parse: {e}"));
            None
        }
        Err(Fail::Crash(e)) => {
            report.fail("CRASH", format!("the script's TOML reader raises: {e}"));
            None
        }
    }
}

/// `set(v or [])`: the distinct items, or the TypeError Python raises (a
/// value that is not iterable, or an unhashable item).
pub(crate) fn set_of(v: Option<&PyVal>) -> Result<Vec<PyVal>, &'static str> {
    let raw = v
        .filter(|v| v.truthy())
        .cloned()
        .unwrap_or(PyVal::List(Vec::new()));
    let Some(items) = raw.iter_items() else {
        return Err("TypeError: the value is not iterable");
    };
    if items.iter().any(|v| !v.hashable()) {
        return Err("TypeError: unhashable item");
    }
    let mut have: Vec<PyVal> = Vec::new();
    for v in items {
        if !have.iter().any(|h| h.eq(&v)) {
            have.push(v);
        }
    }
    Ok(have)
}

/// `sorted(values)` does not raise: no str beside a non-str.
pub(crate) fn sortable(values: &[&PyVal]) -> bool {
    let strs = values.iter().filter(|v| v.as_str().is_some()).count();
    values.len() < 2 || strs == 0 || strs == values.len()
}

/// The words of a paragraph as 004-16 to 004-18 count them: `[A-Za-z]{2,}`
/// outside code spans.
pub(crate) fn words(body: &str) -> usize {
    py::finditer("[A-Za-z]{2,}", &py::sub("`[^`]*`", "", body, false), false).len()
}

/// `re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), re.I)`.
pub(crate) fn is_open_item(body: &str) -> bool {
    py::matches(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), true).is_some()
}

/// `form_rows(text)`: (form, outcome) of every table whose header has a
/// "form" or "invalid" cell and an "outcome" cell (lowercase tests).
pub fn form_rows(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for t in parse_tables(text) {
        let h: Vec<String> = t.header.iter().map(|c| c.to_lowercase()).collect();
        let fi = h
            .iter()
            .position(|c| c.contains("form") || c.contains("invalid"));
        let oi = h.iter().position(|c| c.contains("outcome"));
        let (Some(fi), Some(oi)) = (fi, oi) else {
            continue;
        };
        for r in &t.rows {
            if r.len() > fi.max(oi) {
                out.push((r[fi].clone(), r[oi].clone()));
            }
        }
    }
    out
}

/// `tname(text)`: the normalised name without a trailing "message(s)".
pub fn tname(text: &str) -> String {
    py::sub(r"\s+messages?$", "", &norm_name(text), false)
}

/// `section_blocks(src_lines)`.
pub fn section_blocks(lines: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut in_fence = false;
    for ln in lines {
        if py::lstrip(ln).starts_with("```") {
            in_fence = !in_fence;
            cur.push(ln.clone());
            continue;
        }
        if in_fence {
            cur.push(ln.clone());
            continue;
        }
        if py::matches(r"#{1,6}\s", py::lstrip(ln), false).is_some() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(vec![ln.clone()]);
            continue;
        }
        if py::strip(ln).is_empty() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        cur.push(ln.clone());
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `block_kind(block)`.
pub fn block_kind(block: &[String]) -> &'static str {
    if block.len() == 1 && py::matches(r"#{1,6}\s", py::lstrip(&block[0]), false).is_some() {
        "heading"
    } else if block
        .first()
        .is_some_and(|l| py::lstrip(l).starts_with("```"))
    {
        "code"
    } else if block.iter().all(|l| py::lstrip(l).starts_with('|')) {
        "table"
    } else {
        "prose"
    }
}
