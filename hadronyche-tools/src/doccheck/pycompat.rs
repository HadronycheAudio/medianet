//! Python compatibility: the semantics the scripts ran on.
//!
//! Equivalence includes the Python the scripts were written in (lead's
//! ruling R1). Wherever a script relied on a Python behaviour that Rust's
//! defaults do not share, the library uses the helper here, by name:
//!
//! * regex classes — [`translate`]: Python's `\w` is `str.isalnum() or "_"`
//!   (letters `L*`, every numeric `N*` such as `²`, `_`), Rust's is
//!   Alphabetic + marks + `Nd` + connector punctuation; Python's `\s` also
//!   holds `\x1c`–`\x1f`; `\b`/`\B` use Python's word class; `$` (without
//!   `(?m)`) also matches before a trailing newline.
//!   [`crate::doccheck::text::rx`] translates every pattern, so all checks
//!   get Python's semantics. Constructs with no faithful translation
//!   ([`unsupported`]: `\Z`, `{,n}`, back-references, look-behind) are
//!   refused: the pattern does not build, which fails the port's pattern
//!   test and, at check time, turns the check red (`REGEX_LIMIT`), never a
//!   panic.
//! * line boundaries — [`splitlines`] (`str.splitlines()`: `\n`, `\r`,
//!   `\r\n`, VT, FF, FS, GS, RS, NEL, U+2028, U+2029) and
//!   [`universal_newlines`] (what `open()` in text mode does on read:
//!   `\r\n` and `\r` become `\n`);
//! * paths — [`normpath`] (`os.path.normpath`: a leading `..` is kept, never
//!   clamped at the root);
//! * integers — [`parse_int`] (`int(str)`, arbitrary size in Python; here
//!   checked, reporting [`IntError::Overflow`] beyond `i128` instead of
//!   wrapping or panicking);
//! * TOML values as Python sees them: [`super::pyvalue::PyValue`], read by
//!   the scripts' own readers ([`super::tomlsubset`], [`super::tomllib`]).

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
    translate_with(pattern, PY_BOUNDARY, PY_NOT_BOUNDARY, PY_DOLLAR)
}

/// The pattern with `\b`/`\B` and Python's `$` removed: it matches
/// wherever the translated pattern does (and more), so it is a sound
/// prefilter for it.
pub fn relaxed(pattern: &str) -> String {
    translate_with(pattern, "", "", "").0
}

/// Python's `$` without `re.M`: the end of the text, or before a newline
/// that ends it.
const PY_DOLLAR: &str = r"(?=\n?\z)";

/// True when the pattern sets `(?m)` in its leading flag group (then `$`
/// is the end of a line in both engines).
fn multiline(pattern: &str) -> bool {
    pattern.strip_prefix("(?").is_some_and(|rest| {
        rest.chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .any(|c| c == 'm')
    })
}

/// Why `pattern` uses a Python construct the library refuses (no faithful
/// translation), or `None`: `\Z`, `{,n}`, a back-reference (`\1`,
/// `(?P=name)`) or a look-behind (`(?<=`, `(?<!`). Python accepts them;
/// a port that needs one must rewrite its pattern.
pub fn unsupported(pattern: &str) -> Option<String> {
    let c: Vec<char> = pattern.chars().collect();
    let mut in_class = false;
    let mut i = 0;
    let refuse = |what: &str| {
        Some(format!(
            "{what} is not supported by the library (PORTING.md, \"Regex constructs\")"
        ))
    };
    while i < c.len() {
        match c[i] {
            '\\' => {
                match c.get(i + 1) {
                    Some('Z') if !in_class => return refuse(r"`\Z`"),
                    Some('1'..='9') if !in_class => return refuse("a back-reference"),
                    _ => {}
                }
                i += 2;
                continue;
            }
            '[' if !in_class => {
                in_class = true;
                if c.get(i + 1) == Some(&'^') {
                    i += 1;
                }
                if c.get(i + 1) == Some(&']') {
                    i += 1;
                }
            }
            ']' if in_class => in_class = false,
            '(' if !in_class => {
                let next: String = c[i + 1..].iter().take(3).collect();
                if next.starts_with("?<=") || next.starts_with("?<!") {
                    return refuse("a look-behind");
                }
                if next.starts_with("?P=") {
                    return refuse("a named back-reference");
                }
            }
            '{' if !in_class && c.get(i + 1) == Some(&',') => return refuse("`{,n}`"),
            _ => {}
        }
        i += 1;
    }
    None
}

fn translate_with(
    pattern: &str,
    boundary: &str,
    not_boundary: &str,
    dollar: &str,
) -> (String, bool) {
    let mut out = String::with_capacity(pattern.len() + 16);
    let mut chars = pattern.chars().peekable();
    let mut in_class = false;
    let mut fancy = false;
    let py_dollar = !multiline(pattern);
    while let Some(c) = chars.next() {
        if c == '$' && !in_class && py_dollar {
            fancy = true;
            out.push_str(dollar);
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dollar_and_refused_constructs_follow_python() {
        use crate::doccheck::text::{full, has, rx};
        // Python: re.search("a$", "a\n") matches; fullmatch does not
        assert!(has("a$", "a\n") && has("a$", "a") && !has("a$", "a\n\n"));
        assert!(!has("a$", "ab") && has("(?m)a$", "a\nb"));
        assert!(!full("a", "a\n") && full("a", "a"));
        for (p, why) in [
            (r"a\Z", r"`\Z`"),
            ("a{,2}b", "`{,n}`"),
            (r"(a)\1", "back-reference"),
            ("(?P<x>a)(?P=x)", "back-reference"),
            (r"(?<=\d)x", "look-behind"),
            (r"(?<!\d)x", "look-behind"),
        ] {
            let msg = unsupported(p).unwrap_or_default();
            assert!(msg.contains(why), "{p}: {msg}");
            assert!(rx(p).compile_error().is_some(), "{p} does not build");
        }
        for p in [r"[\Z]", r"a{1,2}", r"\\1", r"[(?<=]", r"(?:x)", r"\{,"] {
            assert_eq!(unsupported(p), None, "{p}");
        }
    }

    #[test]
    fn python_word_class() {
        assert!(is_word('²') && is_word('a') && is_word('_') && is_word('٣'));
        assert!(!is_word('\u{301}') && !is_word('-'));
        let re = regex::Regex::new(&translate(r"[^\w\- ]").0).unwrap();
        assert!(!re.is_match("²"), "² is a Python word character");
        assert!(re.is_match("\u{301}"), "a combining mark is not");
        let w = crate::doccheck::text::rx(r"^\w+$");
        assert!(w.is_match("x²"));
        let s = crate::doccheck::text::rx(r"^\s$");
        assert!(s.is_match("\u{1c}") && s.is_match(" "));
        assert_eq!(translate(r"a\\w[]x]").0, r"a\\w[\]x]");
        assert_eq!(translate(r"\d\.\(").0, r"\d\.\(");
        assert!(translate(r"\bx").1 && !translate(r"x").1);
        assert!(translate(r"x$").1 && !translate(r"(?m)x$").1 && !translate(r"[$]").1);
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
    fn python_strip_and_split() {
        assert_eq!("\u{1c} a \u{1f}".py_strip(), "a");
        assert_eq!(
            "a\u{1d}b c".py_split().collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }
}
