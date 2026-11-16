//! The script's number readers. Its regular expressions use lookarounds
//! (`(?<![\w.\-])`, `(?!\w)`), which the regex crate does not have: each is
//! a scan that applies the guards the way Python's `finditer` does, trying
//! every start position from the left.

use crate::doccheck::text::rx;

/// How a run ends before its report (see c004_14).
pub(crate) use crate::doccheck::checks::c004_14::util::Stop;

pub(crate) type R<T> = Result<T, Stop>;

/// An uncaught exception.
pub(crate) fn crash<T>(why: &str) -> R<T> {
    Err(Stop::Crash(why.to_string()))
}

/// The failures the script collects (`fails`).
#[derive(Debug, Default)]
pub(crate) struct Fails(pub Vec<(&'static str, String)>);

impl Fails {
    pub fn add(&mut self, ac: &'static str, msg: impl Into<String>) {
        self.0.push((ac, msg.into()));
    }
}

/// `\w`.
pub(crate) fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `nrm`: the thin and no-break spaces become spaces, whitespace squashed,
/// trimmed.
pub(crate) fn nrm(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if matches!(c, '\u{a0}' | '\u{2009}' | '\u{202f}') { ' ' } else { c })
        .collect();
    rx(r"\s+").replace_all(&t, " ").trim().to_string()
}

/// `to_int`: spaces removed; a non-number raises `ValueError`.
pub(crate) fn to_int(s: &str) -> R<i64> {
    s.replace(' ', "").parse().or_else(|_| crash("int() of a non-number"))
}

fn digits_end(chars: &[char], from: usize) -> usize {
    let mut j = from;
    while j < chars.len() && chars[j].is_ascii_digit() {
        j += 1;
    }
    j
}

fn prev_ok(chars: &[char], i: usize, bad: impl Fn(char) -> bool) -> bool {
    i == 0 || !bad(chars[i - 1])
}

/// `INT_RE`: `(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)`.
pub(crate) fn int_matches(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let tail_ok = |e: usize| {
        let next = chars.get(e).copied();
        let dot_digit = next == Some('.') && chars.get(e + 1).is_some_and(char::is_ascii_digit);
        !next.is_some_and(is_word) && !dot_digit
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() || !prev_ok(&chars, i, |c| is_word(c) || c == '.' || c == '-') {
            i += 1;
            continue;
        }
        let run = digits_end(&chars, i);
        let mut end = None;
        if run - i <= 3 {
            let mut ends = Vec::new();
            let mut e = run;
            while chars.get(e) == Some(&' ') && digits_end(&chars, e + 1) - (e + 1) >= 3 {
                e += 4;
                ends.push(e);
            }
            end = ends.iter().rev().copied().find(|&e| tail_ok(e));
        }
        if end.is_none() && tail_ok(run) {
            end = Some(run);
        }
        match end {
            Some(e) => {
                out.push(chars[i..e].iter().collect());
                i = e;
            }
            None => i += 1,
        }
    }
    out
}

/// A number with its decimals: `(value, number of decimals)`.
pub(crate) type Dec = (f64, usize);

fn dec_of(int: &str, frac: Option<&str>) -> Dec {
    let text = match frac {
        Some(f) => format!("{int}.{f}"),
        None => int.to_string(),
    };
    (text.parse().unwrap_or(0.0), frac.map_or(0, str::len))
}

/// Scans `re` (no lookbehind) and keeps the matches whose previous
/// character `bad` does not reject, like `(?<![...])` at the start.
pub(crate) fn guarded<'t>(
    re: &'static regex::Regex,
    text: &'t str,
    bad: impl Fn(char) -> bool,
) -> Vec<regex::Captures<'t>> {
    let mut out = Vec::new();
    let mut at = 0;
    while at <= text.len() {
        let Some(c) = re.captures_at(text, at) else { break };
        let m = c.get(0).map_or(0..0, |m| m.range());
        let before = text[..m.start].chars().next_back();
        if before.is_some_and(&bad) {
            at = m.start + text[m.start..].chars().next().map_or(1, char::len_utf8);
        } else {
            at = m.end.max(m.start + 1);
            out.push(c);
        }
    }
    out
}

/// `floats(s)`: `FLOAT_RE` over `nrm(s)`.
pub(crate) fn floats(s: &str) -> Vec<Dec> {
    let n = nrm(s);
    let re = rx(r"(\d+)\.(\d+)");
    let mut out = Vec::new();
    for c in guarded(re, &n, |c| is_word(c) || c == '.' || c == '-') {
        let end = c.get(0).map_or(0, |m| m.end());
        if n[end..].chars().next().is_some_and(is_word) {
            continue;
        }
        out.push(dec_of(&c[1], Some(&c[2])));
    }
    out
}

/// `pcts(s)`: `PCT_RE` over `nrm(s)`.
pub(crate) fn pcts(s: &str) -> Vec<Dec> {
    let re = rx(r"(\d+)(?:\.(\d+))?\s*%");
    guarded(re, &nrm(s), |c| is_word(c) || c == '.')
        .iter()
        .map(|c| dec_of(&c[1], c.get(2).map(|m| m.as_str())))
        .collect()
}

/// `MBIT_RE` over `text` as given.
pub(crate) fn mbit_list(text: &str) -> Vec<Dec> {
    let re = rx(r"(\d+)(?:\.(\d+))?\s*Mbit/s");
    guarded(re, text, |c| is_word(c) || c == '.')
        .iter()
        .map(|c| dec_of(&c[1], c.get(2).map(|m| m.as_str())))
        .collect()
}

/// `ints(s)`: `INT_RE` over `nrm(s)`.
pub(crate) fn ints(s: &str) -> R<Vec<i64>> {
    int_matches(&nrm(s)).iter().map(|m| to_int(m)).collect()
}

/// `tol_of(decimals)`.
pub(crate) fn tol_of(decimals: usize) -> f64 {
    if decimals > 0 {
        0.5 * 10f64.powi(-(decimals as i32))
    } else {
        0.5
    }
}

/// `close(a, b, tol)`.
pub(crate) fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol + 1e-9
}

/// A decimal number as `float("a.b")` reads it.
pub(crate) fn dec_value(int: &str, frac: Option<&str>) -> Dec {
    dec_of(int, frac)
}

/// `re.search(r"(?<![\w.])<literal>(?![\d])", text)` for a literal number
/// text.
pub(crate) fn has_number(text: &str, literal: &str) -> bool {
    let re = rx(&regex::escape(literal));
    guarded(re, text, |c| is_word(c) || c == '.').iter().any(|c| {
        let end = c.get(0).map_or(0, |m| m.end());
        !text[end..].chars().next().is_some_and(|c| c.is_numeric())
    })
}

/// A window of `text` in characters: `text[lo - before : hi + after]`, `lo` and `hi` byte offsets.
pub(crate) fn window(text: &str, lo: usize, hi: usize, before: usize, after: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let cl = text[..lo].chars().count().saturating_sub(before);
    let ch = (text[..hi].chars().count() + after).min(chars.len());
    chars[cl..ch].iter().collect()
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
