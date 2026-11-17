//! Python behaviours the regime-stamp family of scripts (004-24, 004-25,
//! 004-26, 004-28, 004-29) leans on and the library does not offer in the
//! exact form: their `parse_tables` separator test, `re.split` on a
//! look-behind, look-behind number patterns, Python truthiness and `int()`
//! of a TOML value.
//!
//! These are private to the WP8 ports (004-25, 004-26, 004-28 and 004-29
//! import them from here); each is a library candidate, proposed separately
//! under PORTING.md "Adding a library helper".

use toml::Value;

use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::RowSplit;
use crate::doccheck::tables::{Table, split_row};
use crate::doccheck::text::rx;

/// The scripts' `parse_tables` (the 004-16 to 004-29 family): a run of lines
/// starting with `|` outside fences is a table only when its second line
/// matches `^\s*\|?[\s:|-]+\|?\s*$` and contains a `-`; the first line is the
/// header, the rest after the separator are rows. Runs without such a
/// second line are not tables at all.
///
/// The library's separator test (`:?-+:?` per cell) accepts a subset of
/// these lines; this re-checks the runs it leaves headerless.
pub(crate) fn tables(text: &str) -> Vec<Table> {
    let md = Markdown::parse(text);
    let sep = rx(r"^\s*\|?[\s:|-]+\|?\s*$");
    md.tables_with(md.all(), RowSplit::Escaped)
        .into_iter()
        .filter_map(|t| {
            if t.header.is_some() {
                return Some(t);
            }
            let second = t.rows.get(1)?;
            let line = &md.lines[second.line];
            if !(sep.is_match(line) && line.contains('-')) {
                return None;
            }
            Some(Table {
                line: t.line,
                header: Some(split_row(&md.lines[t.rows[0].line])),
                rows: t.rows[2..].to_vec(),
            })
        })
        .collect()
}

/// `first_table(text, pred)`.
pub(crate) fn first_table(text: &str, pred: impl Fn(&[String]) -> bool) -> Option<Table> {
    tables(text).into_iter().find(|t| pred(t.head()))
}

/// The text as the scripts' Python sees it, rewritten so that the `regex`
/// crate classifies every character as Python's `re` does:
///
/// * lines are cut where `str.splitlines()` cuts them (`\r\n`, `\r`,
///   U+000B, U+000C, U+001C to U+001E, U+0085, U+2028, U+2029), and joined
///   with `\n` — the scripts read `WIRE.splitlines()` and join sections back
///   with `\n`, while the library's `Markdown` cuts on `\n` only;
/// * U+001F, whitespace to Python (`\s`, `str.split()`, `strip()`) but not
///   to the crate, becomes a space;
/// * a character that is a word character (`\w`, hence `\b`) to Python but
///   not to the crate (category No: `²`, `½`) becomes `ǂ` (U+01C2, a letter
///   to both), and one that is a word character to the crate but not to
///   Python (combining marks, connector punctuation other than `_`, other
///   alphabetic symbols such as `Ⓐ`) becomes `¦` (U+00A6, a symbol to both).
///
/// No pattern of the ports names these characters, so only the
/// classification changes. The digit-group join must run after this:
/// Python's `(?<=\d)` is Nd only, the library's is any numeric character.
pub(crate) fn py_text(text: &str) -> String {
    py_chars(text, true)
}

/// [`py_text`] without the line-boundary rewrite: for a script that also
/// runs regular expressions over the raw file text (where Python's `[^\n]`
/// matches U+2028), with the word and white-space classes made to agree.
pub(crate) fn py_chars(text: &str, lines: bool) -> String {
    let py_word = rx(r"^[\p{L}\p{N}_]$");
    let rs_word = rx(r"^\w$");
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' if lines => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push('\n');
            }
            '\u{0b}' | '\u{0c}' | '\u{1c}' | '\u{1d}' | '\u{1e}' | '\u{85}' | '\u{2028}' | '\u{2029}' if lines => out.push('\n'),
            '\u{1f}' => out.push(' '),
            c if c.is_ascii() => out.push(c),
            c => {
                let mut b = [0u8; 4];
                let s: &str = c.encode_utf8(&mut b);
                match (py_word.is_match(s), rs_word.is_match(s)) {
                    (true, false) => out.push('\u{1c2}'),
                    (false, true) => out.push('\u{a6}'),
                    _ => out.push(c),
                }
            }
        }
    }
    out
}

/// Records that the script stops here with a Python exception: no FAIL
/// line is printed for the exception itself, and every later check is
/// skipped. The run must still be red, so the note is filed under the first
/// criterion that already failed (the script's set of failed criteria is
/// then unchanged), or under `fallback` when nothing failed before.
pub(crate) fn stopped(report: &mut crate::doccheck::Report, fallback: &str, why: &str) {
    let crit = report.failures.first().map_or(fallback.to_string(), |f| f.criterion.clone());
    report.fail(&crit, format!("the script stops here with a Python exception ({why}); no later check runs"));
}

/// `re.split(r"(?<=[<chars>])\s+", text)`: splits at every whitespace run
/// that follows one of `chars`.
pub(crate) fn split_after(text: &str, chars: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for m in rx(r"\s+").find_iter(text) {
        let prev = text[..m.start()].chars().next_back();
        if m.start() > 0 && prev.is_some_and(|c| chars.contains(c)) {
            out.push(text[start..m.start()].to_string());
            start = m.end();
        }
    }
    out.push(text[start..].to_string());
    out
}

/// `[s.strip().lower() for s in re.split(r"(?<=[.;:])\s+", text) if
/// s.strip()]`.
pub(crate) fn sentences_lower(text: &str, chars: &str) -> Vec<String> {
    split_after(text, chars)
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// `re.findall(r"(?<![<not_before>])(\d+)<after>", text)` as integers:
/// every maximal digit run not preceded by a digit or a character of
/// `not_before` and followed by `after` (a regex).
pub(crate) fn nums_before(text: &str, not_before: &str, after: &str) -> Vec<i64> {
    let re = rx(&format!(r"(\d+)(?:{after})"));
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(c) = re.captures_at(text, at) {
        let g = c.get(1).expect("group 1");
        let prev = text[..g.start()].chars().next_back();
        let blocked = prev.is_some_and(|p| p.is_numeric() || not_before.contains(p));
        if blocked {
            // Python retries at every later position; a later start inside the
            // same digit run is preceded by a digit, so skip the run.
            let run_end = text[g.start()..]
                .char_indices()
                .find(|(_, ch)| !ch.is_numeric())
                .map_or(text.len(), |(i, _)| g.start() + i);
            at = run_end.max(g.start() + 1);
            continue;
        }
        if let Ok(n) = g.as_str().parse() {
            out.push(n);
        }
        at = c.get(0).expect("match").end();
    }
    out
}

/// `re.search(rf"(?<!\d){n}(?!\d)", text)`.
pub(crate) fn has_number(text: &str, n: i64) -> bool {
    rx(&format!(r"(?:^|\D){n}(?:\D|$)")).is_match(text)
}

/// Python truthiness of a TOML value.
pub(crate) fn truthy(v: Option<&Value>) -> bool {
    match v {
        None => false,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Integer(i)) => *i != 0,
        Some(Value::Float(f)) => *f != 0.0,
        Some(Value::Boolean(b)) => *b,
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Table(t)) => !t.is_empty(),
        Some(Value::Datetime(_)) => true,
    }
}

/// Python's `int(table.get(key, default))`: `Err` where Python raises.
pub(crate) fn py_int(t: &toml::Table, key: &str, default: i64) -> Result<i64, String> {
    match t.get(key) {
        None => Ok(default),
        Some(Value::Integer(i)) => Ok(*i),
        Some(Value::Boolean(b)) => Ok(i64::from(*b)),
        Some(Value::Float(f)) if f.is_finite() => Ok(f.trunc() as i64),
        Some(Value::String(s)) => s
            .trim()
            .replace('_', "")
            .parse()
            .map_err(|_| format!("int() of {key} = {s:?}")),
        Some(other) => Err(format!("int() of {key} = {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_splits_and_numbers() {
        assert_eq!(split_after("A b. C; d: e\nf.", ".;:"), vec!["A b.", "C;", "d:", "e\nf."]);
        assert_eq!(sentences_lower("A. B.  ", "."), vec!["a.", "b."]);
        assert_eq!(nums_before("x.5 bytes, 12 bytes and 3bytes", ".", r"\s*bytes"), vec![12, 3]);
        assert_eq!(nums_before("a1234 bytes", ".", r"\s*bytes"), vec![1234]);
        assert!(has_number("count 64.", 64));
        assert!(!has_number("640", 64));
        assert!(truthy(Some(&Value::String("x".into()))));
        assert!(!truthy(Some(&Value::String(String::new()))));
        assert_eq!(py_text("a\u{2028}b\r\nc\u{1f}d"), "a\nb\nc d");
        assert_eq!(py_text("x²"), "x\u{1c2}");
        assert_eq!(py_text("e\u{301}"), "e\u{a6}");
    }

    #[test]
    fn tables_follow_the_scripts_separator_rule() {
        let t = "| a | b |\n| - - | -- |\n| 1 | 2 |\n\n| x |\n| y |\n";
        let ts = tables(t);
        assert_eq!(ts.len(), 1, "a run without a separator is not a table");
        assert_eq!(ts[0].head(), ["a", "b"]);
        assert_eq!(ts[0].rows.len(), 1);
    }
}
