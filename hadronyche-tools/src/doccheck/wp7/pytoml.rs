//! The TOML subset reader of the 004 scripts (`_scan_statements`,
//! `_split_dotted`, `_ValueParser`, `parse_toml`), transcribed with Python
//! value semantics.
//!
//! Nineteen scripts carry this reader byte for byte (004-03, 06, 08-13,
//! 16-20, 24-29; inventory section 2); it exists because the scripts ran on
//! Python 3.10, without `tomllib`. It is not TOML: duplicate keys overwrite,
//! a bare value that is not a number or a boolean is kept as its text, `"""`
//! strings do not parse, `0x28` and `40.0` are numbers equal to 40, and a
//! table header over a value raises a Python exception instead of an error.
//! A port that judges `wire.toml` the way its script did must read it this
//! way, so the values are `PyVal`, with Python's equality, truth and
//! `str()`.
//!
//! `parse` returns `Err(Fail::Error)` where the script's reader raises its
//! `TomlError` (the script reports "does not parse") and `Err(Fail::Crash)`
//! where Python raises anything else (the script dies with a traceback).

use super::py::{self, HUGE, int, strip};
use crate::doccheck::rejection_members::digit_value;

/// A Python value read from the TOML text.
#[derive(Debug, Clone)]
pub(crate) enum PyVal {
    Bool(bool),
    Int(i128),
    Float(f64),
    Str(String),
    List(Vec<PyVal>),
    Dict(PyDict),
}

/// A Python dict with string keys, in insertion order.
#[derive(Debug, Clone, Default)]
pub(crate) struct PyDict(pub Vec<(String, PyVal)>);

impl PyDict {
    /// `d.get(key)`.
    pub fn get(&self, key: &str) -> Option<&PyVal> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut PyVal> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// `key in d`.
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// `d[key] = value` (an existing key keeps its place).
    fn set(&mut self, key: &str, value: PyVal) {
        match self.get_mut(key) {
            Some(v) => *v = value,
            None => self.0.push((key.to_string(), value)),
        }
    }

    /// `d.setdefault(key, default)`.
    fn setdefault(&mut self, key: &str, default: PyVal) -> &mut PyVal {
        if !self.contains(key) {
            self.0.push((key.to_string(), default));
        }
        self.get_mut(key).expect("just inserted")
    }
}

impl PyVal {
    /// Python truth (`bool(v)`).
    pub fn truthy(&self) -> bool {
        match self {
            PyVal::Bool(b) => *b,
            PyVal::Int(i) => *i != 0,
            PyVal::Float(f) => *f != 0.0,
            PyVal::Str(s) => !s.is_empty(),
            PyVal::List(l) => !l.is_empty(),
            PyVal::Dict(d) => !d.0.is_empty(),
        }
    }

    /// The number of a bool, int or float (`True` is 1).
    pub fn number(&self) -> Option<f64> {
        match self {
            PyVal::Bool(b) => Some(f64::from(u8::from(*b))),
            PyVal::Int(i) => Some(*i as f64),
            PyVal::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// `isinstance(v, int)` (a bool is an int) and its value.
    pub fn as_int(&self) -> Option<i128> {
        match self {
            PyVal::Bool(b) => Some(i128::from(*b)),
            PyVal::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// The text of a `str` value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PyVal::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The dict of a dict value.
    pub fn as_dict(&self) -> Option<&PyDict> {
        match self {
            PyVal::Dict(d) => Some(d),
            _ => None,
        }
    }

    /// Python `==`.
    pub fn eq(&self, other: &PyVal) -> bool {
        match (self, other) {
            (PyVal::Str(a), PyVal::Str(b)) => a == b,
            (PyVal::List(a), PyVal::List(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq(y))
            }
            (PyVal::Dict(a), PyVal::Dict(b)) => {
                a.0.len() == b.0.len() && a.0.iter().all(|(k, v)| b.get(k).is_some_and(|w| v.eq(w)))
            }
            (PyVal::Int(a), PyVal::Int(b)) => a == b,
            _ => match (self.as_int(), other.as_int()) {
                (Some(a), Some(b)) => a == b,
                _ => matches!((self.number(), other.number()), (Some(a), Some(b)) if a == b),
            },
        }
    }

    /// Python `v == n` for an integer `n`.
    pub fn eq_int(&self, n: i128) -> bool {
        self.eq(&PyVal::Int(n))
    }

    /// Python `str(v)`.
    pub fn str(&self) -> String {
        match self {
            PyVal::Str(s) => s.clone(),
            _ => self.repr(),
        }
    }

    /// Python `repr(v)`.
    pub fn repr(&self) -> String {
        match self {
            PyVal::Bool(true) => "True".into(),
            PyVal::Bool(false) => "False".into(),
            PyVal::Int(i) => i.to_string(),
            PyVal::Float(f) => float_repr(*f),
            PyVal::Str(s) => str_repr(s),
            PyVal::List(l) => format!(
                "[{}]",
                l.iter().map(PyVal::repr).collect::<Vec<_>>().join(", ")
            ),
            PyVal::Dict(d) => format!(
                "{{{}}}",
                d.0.iter()
                    .map(|(k, v)| format!("{}: {}", str_repr(k), v.repr()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    /// `for x in v` over a list, the characters of a str, or the keys of a
    /// dict; `None` where Python raises `TypeError` (not iterable).
    pub fn iter_items(&self) -> Option<Vec<PyVal>> {
        match self {
            PyVal::List(l) => Some(l.clone()),
            PyVal::Str(s) => Some(s.chars().map(|c| PyVal::Str(c.to_string())).collect()),
            PyVal::Dict(d) => Some(d.0.iter().map(|(k, _)| PyVal::Str(k.clone())).collect()),
            _ => None,
        }
    }

    /// `len(v)`; `None` where Python raises `TypeError`.
    pub fn len(&self) -> Option<usize> {
        match self {
            PyVal::List(l) => Some(l.len()),
            PyVal::Str(s) => Some(py::len(s)),
            PyVal::Dict(d) => Some(d.0.len()),
            _ => None,
        }
    }

    /// True when the value can be a set element (`hash(v)` does not raise).
    pub fn hashable(&self) -> bool {
        !matches!(self, PyVal::List(_) | PyVal::Dict(_))
    }
}

/// Python's `repr` of a str.
pub(crate) fn str_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c != ' ' && py::has(r"^[\p{C}\p{Z}]$", c.encode_utf8(&mut [0; 4])) => {
                let n = c as u32;
                out.push_str(&if n < 0x100 {
                    format!("\\x{n:02x}")
                } else if n < 0x10000 {
                    format!("\\u{n:04x}")
                } else {
                    format!("\\U{n:08x}")
                });
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Python's `repr` of a float (shortest round trip; scientific below 1e-4
/// and from 1e16).
pub(crate) fn float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let sci = format!("{f:e}");
    let (mant, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(char::is_ascii_digit).collect();
    let sign = if neg { "-" } else { "" };
    if (-4..16).contains(&exp) {
        let point = exp + 1;
        let body = if point <= 0 {
            format!("0.{}{digits}", "0".repeat((-point) as usize))
        } else if (point as usize) >= digits.len() {
            format!("{digits}{}.0", "0".repeat(point as usize - digits.len()))
        } else {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        };
        format!("{sign}{body}")
    } else {
        let m = if digits.len() > 1 {
            format!("{}.{}", &digits[..1], &digits[1..])
        } else {
            digits
        };
        format!(
            "{sign}{m}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    }
}

/// Why the reader stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Fail {
    /// The reader's own `TomlError`.
    Error(String),
    /// Any other Python exception: the script dies with a traceback.
    Crash(String),
}

type R<T> = Result<T, Fail>;

fn err<T>(m: &str) -> R<T> {
    Err(Fail::Error(m.to_string()))
}

fn crash<T>(m: &str) -> R<T> {
    Err(Fail::Crash(m.to_string()))
}

fn starts_at(src: &[char], i: usize, pat: &[char]) -> bool {
    src.len() >= i + pat.len() && &src[i..i + pat.len()] == pat
}

/// `_scan_statements`.
fn scan_statements(src: &[char]) -> R<Vec<String>> {
    let (mut stmts, mut buf) = (Vec::new(), String::new());
    let (mut i, n) = (0, src.len());
    let mut in_str: Option<Vec<char>> = None;
    let mut depth = 0usize;
    let push = |buf: &mut String, stmts: &mut Vec<String>| {
        let s = strip(buf).to_string();
        if !s.is_empty() {
            stmts.push(s);
        }
        buf.clear();
    };
    while i < n {
        let c = src[i];
        if let Some(q) = &in_str {
            if (q.len() == 1 && q[0] == '"' || q == &['"'; 3]) && c == '\\' {
                buf.extend(&src[i..(i + 2).min(n)]);
                i += 2;
                continue;
            }
            if starts_at(src, i, q) {
                buf.extend(q.iter());
                i += q.len();
                in_str = None;
                continue;
            }
            buf.push(c);
            i += 1;
            continue;
        }
        if c == '#' {
            match src[i..].iter().position(|&x| x == '\n') {
                Some(j) => i += j,
                None => break,
            }
            continue;
        }
        if c == '"' || c == '\'' {
            let q = if starts_at(src, i, &[c; 3]) {
                vec![c; 3]
            } else {
                vec![c]
            };
            buf.extend(q.iter());
            i += q.len();
            in_str = Some(q);
            continue;
        }
        if c == '[' || c == '{' {
            depth += 1;
        } else if c == ']' || c == '}' {
            depth = depth.saturating_sub(1);
        }
        if c == '\n' {
            if depth == 0 {
                push(&mut buf, &mut stmts);
            } else {
                buf.push(' ');
            }
            i += 1;
            continue;
        }
        buf.push(c);
        i += 1;
    }
    push(&mut buf, &mut stmts);
    if in_str.is_some() {
        return err("unterminated string");
    }
    Ok(stmts)
}

/// `_split_dotted`.
fn split_dotted(key: &str) -> R<Vec<String>> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in key.chars() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            cur.push(c);
        } else if c == '"' || c == '\'' {
            quote = Some(c);
            cur.push(c);
        } else if c == '.' {
            parts.push(strip(&cur).to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    parts.push(strip(&cur).to_string());
    let mut out = Vec::new();
    for p in parts {
        let cs: Vec<char> = p.chars().collect();
        let p = if cs.len() >= 2 && cs[0] == cs[cs.len() - 1] && (cs[0] == '"' || cs[0] == '\'') {
            cs[1..cs.len() - 1].iter().collect()
        } else {
            p
        };
        if p.is_empty() {
            return err("empty key component");
        }
        out.push(p);
    }
    Ok(out)
}

/// Python `int(s, base)`: whitespace, a sign, the base prefix for base 16,
/// 8 or 2, single underscores between digits, any Unicode decimal digit.
fn int_base(s: &str, base: u32) -> Option<i128> {
    let t = strip(s);
    let (neg, mut t) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let prefix = match base {
        16 => "0x",
        8 => "0o",
        2 => "0b",
        _ => "",
    };
    if !prefix.is_empty() && t.len() >= 2 && t[..2].eq_ignore_ascii_case(prefix) {
        t = t[2..].strip_prefix('_').unwrap_or(&t[2..]);
    }
    if t.is_empty() || t.starts_with('_') || t.ends_with('_') || t.contains("__") {
        return None;
    }
    let mut v: i128 = 0;
    for c in t.chars().filter(|&c| c != '_') {
        let d = match digit_value(c) {
            Some(d) => u32::from(d),
            None => c.to_digit(36)?,
        };
        if d >= base {
            return None;
        }
        v = v
            .saturating_mul(i128::from(base))
            .saturating_add(i128::from(d))
            .min(HUGE);
    }
    Some(if neg { -v } else { v })
}

/// Python `float(s)`.
fn py_float(s: &str) -> Option<f64> {
    let t: String = strip(s)
        .chars()
        .map(|c| digit_value(c).map_or(c, |d| char::from(b'0' + d)))
        .collect();
    let low = t.to_ascii_lowercase();
    let body = low.trim_start_matches(['+', '-']);
    if low.len() - body.len() > 1 {
        return None;
    }
    if matches!(body, "inf" | "infinity" | "nan") {
        return t.parse::<f64>().ok().or(Some(if body == "nan" {
            f64::NAN
        } else if low.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }));
    }
    if !py::has(
        r"^[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$",
        &t,
    ) {
        return None;
    }
    t.parse::<f64>().ok()
}

struct ValueParser {
    t: Vec<char>,
    i: usize,
}

impl ValueParser {
    fn skip_ws(&mut self) {
        while self.i < self.t.len() && " \t\r\n".contains(self.t[self.i]) {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.t.get(self.i).copied()
    }

    fn parse(&mut self) -> R<PyVal> {
        self.skip_ws();
        match self.peek() {
            None => err("missing value"),
            Some('"') => self.basic_string(),
            Some('\'') => self.literal_string(),
            Some('[') => self.array(),
            Some('{') => self.inline_table(),
            Some(_) => self.bare(),
        }
    }

    fn hex_escape(&mut self, width: usize) -> R<char> {
        let end = (self.i + width).min(self.t.len());
        let digits: String = self.t[self.i..end].iter().collect();
        self.i += width;
        let Some(n) = int_base(&digits, 16) else {
            return crash("ValueError: invalid literal for int() with base 16");
        };
        if !(0..=0x10_FFFF).contains(&n) {
            return crash("ValueError: chr() arg not in range(0x110000)");
        }
        Ok(char::from_u32(n as u32).unwrap_or('\u{fffd}'))
    }

    fn basic_string(&mut self) -> R<PyVal> {
        self.i += 1;
        let mut out = String::new();
        while self.i < self.t.len() {
            let c = self.t[self.i];
            self.i += 1;
            if c == '"' {
                return Ok(PyVal::Str(out));
            }
            if c != '\\' {
                out.push(c);
                continue;
            }
            let Some(&e) = self.t.get(self.i) else { break };
            self.i += 1;
            match e {
                'u' => out.push(self.hex_escape(4)?),
                'U' => out.push(self.hex_escape(8)?),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                e => out.push(e),
            }
        }
        err("unterminated basic string")
    }

    fn literal_string(&mut self) -> R<PyVal> {
        self.i += 1;
        let Some(j) = self.t[self.i..].iter().position(|&c| c == '\'') else {
            return err("unterminated literal string");
        };
        let s: String = self.t[self.i..self.i + j].iter().collect();
        self.i += j + 1;
        Ok(PyVal::Str(s))
    }

    fn array(&mut self) -> R<PyVal> {
        self.i += 1;
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(']') => {
                    self.i += 1;
                    return Ok(PyVal::List(out));
                }
                None => return err("unterminated array"),
                _ => {}
            }
            out.push(self.parse()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {
                    self.i += 1;
                    return Ok(PyVal::List(out));
                }
                _ => return err("expected ',' or ']' in array"),
            }
        }
    }

    fn inline_table(&mut self) -> R<PyVal> {
        self.i += 1;
        let mut out = PyDict::default();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.i += 1;
                return Ok(PyVal::Dict(out));
            }
            let start = self.i;
            let mut quote: Option<char> = None;
            while self.i < self.t.len() {
                let c = self.t[self.i];
                if let Some(q) = quote {
                    if c == q {
                        quote = None;
                    }
                } else if c == '"' || c == '\'' {
                    quote = Some(c);
                } else if c == '=' {
                    break;
                }
                self.i += 1;
            }
            if self.peek() != Some('=') {
                return err("expected '=' in inline table");
            }
            let key: String = self.t[start..self.i].iter().collect();
            self.i += 1;
            let value = self.parse()?;
            let parts = split_dotted(strip(&key))?;
            assign(&mut out, &parts, value)?;
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(PyVal::Dict(out));
                }
                _ => return err("expected ',' or '}' in inline table"),
            }
        }
    }

    fn bare(&mut self) -> R<PyVal> {
        let start = self.i;
        while self.i < self.t.len() && !",]}\n".contains(self.t[self.i]) {
            self.i += 1;
        }
        let raw: String = self.t[start..self.i].iter().collect();
        let tok = strip(&raw).to_string();
        if tok.is_empty() {
            return err("empty value");
        }
        let low = tok.to_lowercase();
        if low == "true" {
            return Ok(PyVal::Bool(true));
        }
        if low == "false" {
            return Ok(PyVal::Bool(false));
        }
        let t = tok.replace('_', "");
        let tl = t.to_lowercase();
        let parsed = if tl.starts_with("0x") {
            int_base(&t, 16).map(PyVal::Int)
        } else if tl.starts_with("0o") {
            int_base(&t, 8).map(PyVal::Int)
        } else if tl.starts_with("0b") {
            int_base(&t, 2).map(PyVal::Int)
        } else if py::fullmatch(r"[+-]?\d+", &t, false).is_some() {
            int(&t).map(PyVal::Int)
        } else {
            py_float(&t).map(PyVal::Float)
        };
        Ok(parsed.unwrap_or(PyVal::Str(tok)))
    }
}

/// `target = target.setdefault(p, {})` for every part but the last, then
/// `target[last] = value` (no list step: a list there raises).
fn assign(root: &mut PyDict, parts: &[String], value: PyVal) -> R<()> {
    let mut target = root;
    for p in &parts[..parts.len() - 1] {
        target = match target.setdefault(p, PyVal::Dict(PyDict::default())) {
            PyVal::Dict(d) => d,
            _ => return crash("AttributeError: setdefault on a non-dict"),
        };
    }
    target.set(&parts[parts.len() - 1], value);
    Ok(())
}

/// One step from a dict to the current table.
#[derive(Debug, Clone)]
enum Step {
    Key(String),
    Last,
}

/// The table at `path` from the root (`current` of `parse_toml`); a value
/// that is not a dict there is where Python raises on the next key.
fn walk<'a>(root: &'a mut PyDict, path: &[Step]) -> R<&'a mut PyDict> {
    let mut d = root;
    let mut k = 0;
    while k < path.len() {
        let Step::Key(key) = &path[k] else {
            return crash("a path step without a key");
        };
        let Some(mut v) = d.get_mut(key) else {
            return crash("a walked key is gone");
        };
        k += 1;
        if matches!(path.get(k), Some(Step::Last)) {
            k += 1;
            v = match v {
                PyVal::List(l) => match l.last_mut() {
                    Some(x) => x,
                    None => return crash("IndexError: list index out of range"),
                },
                _ => return crash("TypeError: not a list"),
            };
        }
        d = match v {
            PyVal::Dict(x) => x,
            _ => return crash("TypeError/AttributeError: the table is not a dict"),
        };
    }
    Ok(d)
}

/// The header navigation of `parse_toml`: from the root, `setdefault(p,
/// {})` per part, stepping into the last element of a list. Returns the
/// path of the table reached (which Python does not check to be a dict
/// until a key is assigned in it).
fn navigate(root: &mut PyDict, parts: &[String]) -> R<Vec<Step>> {
    let mut path: Vec<Step> = Vec::new();
    for p in parts {
        let target = walk(root, &path)?;
        let nxt = target.setdefault(p, PyVal::Dict(PyDict::default()));
        path.push(Step::Key(p.clone()));
        if let PyVal::List(l) = nxt {
            if l.is_empty() {
                return crash("IndexError: list index out of range");
            }
            path.push(Step::Last);
        }
    }
    Ok(path)
}

/// `parse_toml(src)`: the root dict.
pub(crate) fn parse(src: &str) -> Result<PyDict, Fail> {
    let chars: Vec<char> = src.chars().collect();
    let mut root = PyDict::default();
    let mut current: Vec<Step> = Vec::new();
    for stmt in scan_statements(&chars)? {
        if stmt.starts_with("[[") && stmt.ends_with("]]") && stmt.len() >= 4 {
            let path = split_dotted(&stmt[2..stmt.len() - 2])?;
            let mut nav = navigate(&mut root, &path[..path.len() - 1])?;
            let target = walk(&mut root, &nav)?;
            let last = &path[path.len() - 1];
            match target.setdefault(last, PyVal::List(Vec::new())) {
                PyVal::List(l) => l.push(PyVal::Dict(PyDict::default())),
                _ => return err(&format!("[[{}]] is not an array of tables", path.join("."))),
            }
            nav.push(Step::Key(last.clone()));
            nav.push(Step::Last);
            current = nav;
            continue;
        }
        if stmt.starts_with('[') && stmt.ends_with(']') && stmt.len() >= 2 {
            let path = split_dotted(&stmt[1..stmt.len() - 1])?;
            current = navigate(&mut root, &path)?;
            continue;
        }
        let Some(eq) = key_end(&stmt) else {
            return err(&format!(
                "not a key/value statement: {}",
                py::head(&stmt, 60)
            ));
        };
        let key = strip(&stmt[..eq]).to_string();
        let mut parser = ValueParser {
            t: stmt[eq + 1..].chars().collect(),
            i: 0,
        };
        let value = parser.parse()?;
        parser.skip_ws();
        if parser.i != parser.t.len() {
            return err("trailing text after value");
        }
        let parts = split_dotted(&key)?;
        let target = walk(&mut root, &current)?;
        assign(target, &parts, value)?;
    }
    Ok(root)
}

/// The byte index of the first `=` outside quotes.
fn key_end(stmt: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (k, c) in stmt.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c == '=' {
            return Some(k);
        }
    }
    None
}

/// `[m for m in toml.get("message", [])]` when it is a list, else empty.
pub(crate) fn list_of<'a>(root: &'a PyDict, key: &str) -> &'a [PyVal] {
    match root.get(key) {
        Some(PyVal::List(l)) => l,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(d: &'a PyDict, k: &str) -> &'a PyVal {
        d.get(k).expect("key")
    }

    #[test]
    fn reads_the_subset_with_python_types() {
        let src = "# c\nname = \"x\" # tail\n[[message]]\nname = 'hello'\ndscp = 40.0\nmax = 0x28\nodd = forty\nflag = TRUE\nlist = [\n  \"a\",\n  1,\n]\n[[message.field]]\nname = \"f\"\n[[message]]\nname = \"b\"\n";
        let root = parse(src).expect("parses");
        assert_eq!(get(&root, "name").as_str(), Some("x"));
        let msgs = list_of(&root, "message");
        assert_eq!(msgs.len(), 2);
        let m = msgs[0].as_dict().expect("dict");
        assert!(get(m, "dscp").eq_int(40));
        assert!(get(m, "max").eq_int(40));
        assert_eq!(get(m, "odd").as_str(), Some("forty"));
        assert!(get(m, "flag").eq_int(1));
        assert_eq!(get(m, "list").repr(), "['a', 1]");
        assert_eq!(list_of(m, "field").len(), 1);
        assert_eq!(get(m, "dscp").str(), "40.0");
    }

    #[test]
    fn errors_and_crashes_are_python_s() {
        assert!(matches!(parse("a = \"\"\"x\"\"\""), Err(Fail::Error(_))));
        assert!(matches!(parse("a = \"x"), Err(Fail::Error(_))));
        assert!(matches!(parse("a = 1\n[a]\nb = 2"), Err(Fail::Crash(_))));
        assert!(matches!(parse("a = \"\\uZZZZ\""), Err(Fail::Crash(_))));
        assert!(parse("[[a]]\n[[a]]\nx = 1\n[a.b]\ny=2").is_ok());
        let root = parse("a = 1\na = 2\n").expect("duplicate keys overwrite");
        assert!(get(&root, "a").eq_int(2));
        assert!(matches!(parse("= 1"), Err(Fail::Error(_))));
    }

    #[test]
    fn python_reprs() {
        assert_eq!(float_repr(40.0), "40.0");
        assert_eq!(float_repr(1e16), "1e+16");
        assert_eq!(float_repr(1.5e-5), "1.5e-05");
        assert_eq!(float_repr(0.0001), "0.0001");
        assert_eq!(float_repr(123.25), "123.25");
        assert_eq!(str_repr("it's"), "\"it's\"");
        assert_eq!(str_repr("a\u{1}"), "'a\\x01'");
        assert_eq!(py_float(" 1e3 "), Some(1000.0));
        assert_eq!(py_float("1.e3"), Some(1000.0));
        assert_eq!(py_float("abc"), None);
        assert_eq!(int_base("0x1F", 16), Some(31));
        assert_eq!(int_base("0x", 16), None);
    }
}
