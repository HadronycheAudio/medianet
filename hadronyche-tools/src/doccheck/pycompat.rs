//! Python compatibility: the semantics the scripts ran on.
//!
//! Equivalence includes the Python the scripts were written in (lead's
//! ruling R1). Wherever a script relied on a Python behaviour that Rust's
//! defaults do not share, the library uses the helper here, by name:
//!
//! * regex classes — [`translate`]: Python's `\w` is `str.isalnum() or "_"`
//!   (letters `L*`, every numeric `N*` such as `²`, `_`), Rust's is
//!   Alphabetic + marks + `Nd` + connector punctuation; Python's `\s` also
//!   holds `\x1c`–`\x1f`. [`crate::doccheck::text::rx`] translates every
//!   pattern, so all checks get Python's classes. Known limit: `\b` keeps
//!   Rust's word definition (it differs from Python's only next to a
//!   non-decimal numeral such as `²` or a combining mark); a port that needs
//!   it exact writes the boundary as a class.
//! * line boundaries — [`splitlines`] (`str.splitlines()`: `\n`, `\r`,
//!   `\r\n`, VT, FF, FS, GS, RS, NEL, U+2028, U+2029) and
//!   [`universal_newlines`] (what `open()` in text mode does on read:
//!   `\r\n` and `\r` become `\n`);
//! * paths — [`normpath`] (`os.path.normpath`: a leading `..` is kept, never
//!   clamped at the root);
//! * integers — [`parse_int`] (`int(str)`, arbitrary size in Python; here
//!   checked, reporting [`IntError::Overflow`] beyond `i128` instead of
//!   wrapping or panicking);
//! * TOML values as Python sees them after the scripts' readers —
//!   [`py_int`], [`py_is_int`], [`py_eq_int`], [`py_iter`].

use toml::Value;

/// True when Python's `\w` matches `c` (`c.isalnum() or c == "_"`: a
/// letter `L*`, a numeral `N*`, or `_`).
pub fn is_word(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphanumeric() || c == '_';
    }
    crate::doccheck::text::rx_raw(r"^[\p{L}\p{N}_]$").is_match(c.encode_utf8(&mut [0; 4]))
}

/// True when Python's `\d` matches `c` (a Unicode decimal digit, `Nd`).
pub fn is_nd(c: char) -> bool {
    if c.is_ascii() || !c.is_numeric() {
        return c.is_ascii_digit();
    }
    crate::doccheck::text::rx_raw(r"^\p{Nd}$").is_match(c.encode_utf8(&mut [0; 4]))
}

const PY_W: &str = r"\p{L}\p{N}_";
const PY_S: &str = r"\s\x1c-\x1f";

/// Rewrites a Python pattern with Python's classes and boundaries: `\w`
/// `\W` `\s` `\S` inside and outside character classes, and `\b` `\B` as
/// look-arounds over Python's word class. Returns the pattern and whether
/// it needs look-around (`fancy-regex`). Everything else passes through
/// (`\d` is Unicode `Nd` in both).
pub fn translate(pattern: &str) -> (String, bool) {
    translate_with(pattern, PY_BOUNDARY, PY_NOT_BOUNDARY)
}

/// The pattern with `\b`/`\B` removed: it matches wherever the translated
/// pattern does (and more), so it is a sound prefilter for it.
pub fn relaxed(pattern: &str) -> String {
    translate_with(pattern, "", "").0
}

fn translate_with(pattern: &str, boundary: &str, not_boundary: &str) -> (String, bool) {
    let mut out = String::with_capacity(pattern.len() + 16);
    let mut chars = pattern.chars().peekable();
    let mut in_class = false;
    let mut fancy = false;
    while let Some(c) = chars.next() {
        if c == '\\' {
            let Some(n) = chars.next() else {
                out.push('\\');
                break;
            };
            if !in_class && (n == 'b' || n == 'B') {
                fancy = true;
                out.push_str(if n == 'b' { boundary } else { not_boundary });
                continue;
            }
            out.push_str(&escape_for(n, in_class));
            continue;
        }
        if c == '[' && !in_class {
            in_class = true;
            out.push(c);
            // a `]` right after `[` or `[^` is a literal in Python
            if chars.peek() == Some(&'^') {
                out.push(chars.next().unwrap_or('^'));
            }
            if chars.peek() == Some(&']') {
                chars.next();
                out.push_str(r"\]");
            }
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
        }
        out.push(c);
    }
    (out, fancy)
}

/// Python's `\b`: between a Python word character and a non-word one (or
/// the start or end of the text).
const PY_BOUNDARY: &str =
    r"(?:(?<=[\p{L}\p{N}_])(?![\p{L}\p{N}_])|(?<![\p{L}\p{N}_])(?=[\p{L}\p{N}_]))";
/// Python's `\B`.
const PY_NOT_BOUNDARY: &str =
    r"(?:(?<=[\p{L}\p{N}_])(?=[\p{L}\p{N}_])|(?<![\p{L}\p{N}_])(?![\p{L}\p{N}_]))";

fn escape_for(n: char, in_class: bool) -> String {
    match (n, in_class) {
        ('w', false) => format!("[{PY_W}]"),
        ('w', true) => PY_W.to_string(),
        ('W', _) => format!("[^{PY_W}]"),
        ('s', false) => format!("[{PY_S}]"),
        ('s', true) => PY_S.to_string(),
        ('S', _) => format!("[^{PY_S}]"),
        (other, _) => format!("\\{other}"),
    }
}

/// Python's `str.splitlines()`: splits on every line boundary Python knows
/// and drops the terminators; a trailing terminator yields no empty line.
pub fn splitlines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let is_break = matches!(
            c,
            '\n' | '\r'
                | '\u{0b}'
                | '\u{0c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if !is_break {
            continue;
        }
        out.push(&text[start..i]);
        let mut next = i + c.len_utf8();
        if c == '\r' && matches!(chars.peek(), Some((_, '\n'))) {
            chars.next();
            next += 1;
        }
        start = next;
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

/// Python's `str.isspace()`: Unicode white space plus the separators
/// `\x1c`-`\x1f`, which Rust's `char::is_whitespace` leaves out.
pub fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python's `strip`, `lstrip`, `rstrip` and `split()` (white space as
/// [`is_space`]). Every port uses these instead of Rust's `trim` family.
pub trait PyStr {
    /// `str.strip()`.
    fn py_strip(&self) -> &str;
    /// `str.lstrip()`.
    fn py_lstrip(&self) -> &str;
    /// `str.rstrip()`.
    fn py_rstrip(&self) -> &str;
    /// `str.split()` (no argument).
    fn py_split(&self) -> std::vec::IntoIter<&str>;
}

impl PyStr for str {
    fn py_strip(&self) -> &str {
        self.trim_matches(is_space)
    }
    fn py_lstrip(&self) -> &str {
        self.trim_start_matches(is_space)
    }
    fn py_rstrip(&self) -> &str {
        self.trim_end_matches(is_space)
    }
    fn py_split(&self) -> std::vec::IntoIter<&str> {
        self.split(is_space)
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// What `open(path)` in text mode returns: `\r\n` and a lone `\r` read as
/// `\n` (universal newlines).
pub fn universal_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Python's `os.path.normpath` for `/`-separated paths: `.` dropped,
/// `a/..` collapsed, a leading `..` KEPT (a relative path may climb above
/// its start), a leading `/` kept, `""` becomes `"."`.
pub fn normpath(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|p| *p != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..");
                }
            }
            p => parts.push(p),
        }
    }
    let body = parts.join("/");
    match (absolute, body.is_empty()) {
        (true, _) => format!("/{body}"),
        (false, true) => ".".to_string(),
        (false, false) => body,
    }
}

/// Why an integer could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntError {
    /// Not an integer literal Python's `int()` accepts.
    NotInt(String),
    /// A valid integer beyond `i128`: Python would have carried on with it;
    /// a check reports it instead.
    Overflow(String),
}

impl std::fmt::Display for IntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntError::NotInt(s) => write!(f, "int() cannot read {s:?}"),
            IntError::Overflow(s) => {
                write!(f, "the integer {s} is beyond what the check can compare")
            }
        }
    }
}

/// `int(s)` for a string: surrounding white space, an optional sign, decimal
/// digits (any Unicode decimal digit, as Python), single `_` between digits.
pub fn parse_int(s: &str) -> Result<i128, IntError> {
    let t = s.py_strip();
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let bad = || IntError::NotInt(s.to_string());
    if digits.is_empty()
        || digits.starts_with('_')
        || digits.ends_with('_')
        || digits.contains("__")
    {
        return Err(bad());
    }
    let mut value: i128 = 0;
    for c in digits.chars().filter(|c| *c != '_') {
        let d = decimal_value(c).ok_or_else(bad)?;
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(i128::from(d)))
            .ok_or_else(|| IntError::Overflow(t.to_string()))?;
    }
    Ok(if neg { -value } else { value })
}

fn decimal_value(c: char) -> Option<u32> {
    if let Some(d) = c.to_digit(10) {
        return Some(d);
    }
    if !is_nd(c) {
        return None;
    }
    // Unicode decimal digits come in contiguous runs of ten, 0 to 9, some
    // runs adjacent: the value is the count of digits before it, mod 10
    let mut k = 0;
    let mut z = u32::from(c);
    while let Some(p) = z.checked_sub(1).and_then(char::from_u32) {
        if !is_nd(p) {
            break;
        }
        k += 1;
        z -= 1;
    }
    Some(k % 10)
}

/// `int(v)` for a TOML value read by a script: an integer as is, a boolean
/// as 0 or 1, a float truncated, a string parsed by [`parse_int`]; anything
/// else is the `TypeError` the script would have crashed with.
pub fn py_int(v: &Value) -> Result<i128, IntError> {
    match v {
        Value::Integer(i) => Ok(i128::from(*i)),
        Value::Boolean(b) => Ok(i128::from(*b)),
        Value::Float(f) if f.is_finite() && f.abs() < 1e38 => Ok(f.trunc() as i128),
        Value::String(s) => parse_int(s),
        other => Err(IntError::NotInt(other.to_string())),
    }
}

/// `isinstance(v, int)` (a boolean is an int in Python).
pub fn py_is_int(v: &Value) -> bool {
    matches!(v, Value::Integer(_) | Value::Boolean(_))
}

/// `v == n` for an integer `n` (an equal float or boolean compares equal).
pub fn py_eq_int(v: &Value, n: i128) -> bool {
    match v {
        Value::Integer(i) => i128::from(*i) == n,
        Value::Boolean(b) => i128::from(*b) == n,
        Value::Float(f) => *f == n as f64,
        _ => false,
    }
}

/// `for x in v`: an array's elements (as `str()`), a string's characters, a
/// table's keys; `Err` for what Python cannot iterate (the script's crash).
pub fn py_iter(v: &Value) -> Result<Vec<String>, String> {
    match v {
        Value::Array(a) => Ok(a.iter().map(crate::doccheck::wire_toml::display).collect()),
        Value::String(s) => Ok(s.chars().map(String::from).collect()),
        Value::Table(t) => Ok(t.keys().cloned().collect()),
        other => Err(format!("{} is not iterable", other.type_str())),
    }
}

/// `[x for x in v if isinstance(x, dict)]`: the tables Python finds when it
/// iterates `v` (an array's tables; nothing for a table, whose keys are
/// strings, or a string); `Err` when `v` is not iterable.
pub fn py_dicts(v: Option<&Value>) -> Result<Vec<&toml::Table>, String> {
    match v {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => Ok(a.iter().filter_map(Value::as_table).collect()),
        Some(Value::Table(_)) | Some(Value::String(_)) => Ok(Vec::new()),
        Some(other) => Err(format!("{} is not iterable", other.type_str())),
    }
}

/// `a == b` for two TOML values as Python compares what its reader built
/// (numbers by value across int, float and bool; strings exactly; arrays
/// element-wise).
pub fn py_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| py_eq(p, q))
        }
        (Value::Table(x), Value::Table(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| py_eq(v, w)))
        }
        (x, y) => match (py_num(x), py_num(y)) {
            (Some(p), Some(q)) => p == q,
            _ => false,
        },
    }
}

fn py_num(v: &Value) -> Option<f64> {
    match v {
        Value::Integer(i) => Some(*i as f64),
        Value::Boolean(b) => Some(f64::from(u8::from(*b))),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_word_class() {
        assert!(is_word('²') && is_word('a') && is_word('_') && is_word('٣'));
        assert!(!is_word('\u{301}') && !is_word('-'));
        let re = regex::Regex::new(&translate(r"[^\w\- ]").0).unwrap();
        assert!(!re.is_match("²"), "² is a Python word character");
        assert!(re.is_match("\u{301}"), "a combining mark is not");
        let w = regex::Regex::new(&translate(r"^\w+$").0).unwrap();
        assert!(w.is_match("x²"));
        let s = regex::Regex::new(&translate(r"^\s$").0).unwrap();
        assert!(s.is_match("\u{1c}") && s.is_match(" "));
        assert_eq!(translate(r"a\\w[]x]").0, r"a\\w[\]x]");
        assert_eq!(translate(r"\d\.\(").0, r"\d\.\(");
        assert!(translate(r"\bx").1 && !translate(r"x").1);
        let b = crate::doccheck::text::rx(r"branch\b");
        assert!(
            !b.is_match("branch²"),
            "² is a Python word character: no boundary"
        );
        assert!(
            b.is_match("branch\u{301}"),
            "a combining mark is not: a boundary"
        );
        assert!(b.is_match("branch.") && b.is_match("branch"));
        assert!(crate::doccheck::text::rx(r"a\Bb").is_match("ab"));
    }

    #[test]
    fn line_boundaries() {
        assert_eq!(
            splitlines("a\u{2028}b\u{0c}c\r\nd\re"),
            vec!["a", "b", "c", "d", "e"]
        );
        assert_eq!(splitlines("a\n"), vec!["a"]);
        assert_eq!(universal_newlines("a\r\nb\rc"), "a\nb\nc");
    }

    #[test]
    fn normpath_keeps_leading_dotdot() {
        assert_eq!(
            normpath("doc/design/spec/../../../../doc/x.md"),
            "../doc/x.md"
        );
        assert_eq!(normpath("a/./b/../c"), "a/c");
        assert_eq!(normpath("../../a"), "../../a");
        assert_eq!(normpath(""), ".");
        assert_eq!(normpath("/../a"), "/a");
    }

    #[test]
    fn checked_integers() {
        assert_eq!(parse_int(" 1_000 "), Ok(1000));
        assert_eq!(parse_int("-7"), Ok(-7));
        assert_eq!(parse_int("٣٤"), Ok(34));
        assert!(matches!(parse_int("1__0"), Err(IntError::NotInt(_))));
        assert!(matches!(parse_int("0x10"), Err(IntError::NotInt(_))));
        let big = "9".repeat(60);
        assert!(matches!(parse_int(&big), Err(IntError::Overflow(_))));
    }

    #[test]
    fn toml_values_as_python_reads_them() {
        assert_eq!(py_int(&Value::Float(40.0)), Ok(40));
        assert_eq!(py_int(&Value::String("12".into())), Ok(12));
        assert!(py_int(&Value::Array(vec![])).is_err());
        assert!(py_is_int(&Value::Boolean(true)) && !py_is_int(&Value::Float(1.0)));
        assert!(py_eq_int(&Value::Float(40.0), 40));
        assert_eq!(
            py_iter(&Value::String("Ab".into())).unwrap(),
            vec!["A", "b"]
        );
        assert!(py_iter(&Value::Integer(3)).is_err());
        assert!(py_eq(&Value::Float(3.0), &Value::Integer(3)));
        assert!(!py_eq(&Value::String("3".into()), &Value::Integer(3)));
        assert!(py_dicts(Some(&Value::Integer(1))).is_err());
        assert!(py_dicts(None).unwrap().is_empty());
        assert_eq!("\u{1c} a \u{1f}".py_strip(), "a");
        assert_eq!(
            "a\u{1d}b c".py_split().collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }
}
