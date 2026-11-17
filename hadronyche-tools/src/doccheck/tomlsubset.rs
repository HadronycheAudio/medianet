//! The 004 family's hand-written TOML reader, ported faithfully
//! ([`crate::doccheck::policy::TomlReader::Subset004_18`]).
//!
//! The scripts 004-03, 05, 06, 08-13, 16-20 and 24-29 carry the same small
//! TOML reader (`_scan_statements`, `_split_dotted`, `_ValueParser`,
//! `parse_toml`) because their Python 3.10 had no `tomllib`. The corpus
//! generator runs all twenty copies; they agree on every snippet (004-03
//! and 004-05 differ only in formatting). It is not TOML: it
//! rejects valid TOML (`"""…"""` and `'''…'''` strings parse as an empty
//! string followed by trailing text) and accepts invalid TOML (`040`, a
//! duplicate key — the last wins — an unquoted `C-116`, which reads as the
//! string "C-116"; a date reads as a string; a number too big for 64 bits
//! stays an int). A port reads `wire.toml` through this port of it, so the
//! script's verdict on any such input is reproduced, not excused.
//!
//! Each function mirrors its Python original line for line; the corpus
//! `hadronyche-tools/data/toml_corpus.txt` (generated from the Python reader
//! by `hadronyche-tools/tools/gen_toml_corpus.py`) is the proof, checked by
//! the tests below. Python's 4 300-digit limit on `int()` from a decimal
//! string (a longer token stays a string) is reproduced. Known limit: a `\uD800`-style escape gives Python a lone
//! surrogate, which a Rust string cannot hold; it reads as U+FFFD.

use super::pycompat::PyStr;
use super::pyvalue::{Dict, PyValue};
use super::text::rx;

/// The reader's failure: the Python exception type and its message.
pub type Error = String;

fn err(kind: &str, msg: &str) -> Error {
    format!("{kind}: {msg}")
}

fn starts_at(src: &[char], i: usize, pat: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    src.len() >= i + p.len() && src[i..i + p.len()] == p[..]
}

fn strip(s: &str) -> String {
    s.py_strip().to_string()
}

/// `_scan_statements`: the logical statements (lines joined inside
/// brackets, comments dropped outside strings).
fn scan_statements(src: &[char]) -> Result<Vec<String>, Error> {
    let (mut stmts, mut buf) = (Vec::new(), String::new());
    let (mut i, n, mut depth) = (0, src.len(), 0usize);
    let mut in_str: Option<String> = None;
    while i < n {
        let c = src[i];
        if let Some(q) = in_str.clone() {
            if (q == "\"" || q == "\"\"\"") && c == '\\' {
                buf.extend(&src[i..(i + 2).min(n)]);
                i += 2;
                continue;
            }
            if starts_at(src, i, &q) {
                buf.push_str(&q);
                i += q.chars().count();
                in_str = None;
                continue;
            }
            buf.push(c);
            i += 1;
            continue;
        }
        if c == '#' {
            match src[i..].iter().position(|&x| x == '\n') {
                None => break,
                Some(k) => {
                    i += k;
                    continue;
                }
            }
        }
        if c == '"' || c == '\'' {
            let triple: String = std::iter::repeat_n(c, 3).collect();
            if starts_at(src, i, &triple) {
                buf.push_str(&triple);
                in_str = Some(triple);
                i += 3;
                continue;
            }
            in_str = Some(c.to_string());
            buf.push(c);
            i += 1;
            continue;
        }
        if c == '[' || c == '{' {
            depth += 1;
        } else if c == ']' || c == '}' {
            depth = depth.saturating_sub(1);
        }
        if c == '\n' {
            if depth == 0 {
                let s = strip(&buf);
                if !s.is_empty() {
                    stmts.push(s);
                }
                buf.clear();
            } else {
                buf.push(' ');
            }
            i += 1;
            continue;
        }
        buf.push(c);
        i += 1;
    }
    let s = strip(&buf);
    if !s.is_empty() {
        stmts.push(s);
    }
    if in_str.is_some() {
        return Err(err("TomlError", "unterminated string"));
    }
    Ok(stmts)
}

/// `_split_dotted`.
fn split_dotted(key: &str) -> Result<Vec<String>, Error> {
    let (mut parts, mut cur) = (Vec::new(), String::new());
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
            parts.push(strip(&cur));
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    parts.push(strip(&cur));
    let mut out = Vec::new();
    for mut p in parts {
        let cs: Vec<char> = p.chars().collect();
        if cs.len() >= 2 && cs[0] == cs[cs.len() - 1] && (cs[0] == '"' || cs[0] == '\'') {
            p = cs[1..cs.len() - 1].iter().collect();
        }
        if p.is_empty() {
            return Err(err("TomlError", "empty key component"));
        }
        out.push(p);
    }
    Ok(out)
}

/// `_ValueParser`.
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

    fn parse(&mut self) -> Result<PyValue, Error> {
        self.skip_ws();
        match self.peek() {
            None => Err(err("TomlError", "missing value")),
            Some('"') => self.basic_string(),
            Some('\'') => self.literal_string(),
            Some('[') => self.array(),
            Some('{') => self.inline_table(),
            Some(_) => self.bare(),
        }
    }

    fn hex_escape(&mut self, width: usize) -> Result<char, Error> {
        let end = (self.i + width).min(self.t.len());
        let s: String = self.t[self.i..end].iter().collect();
        self.i += width;
        let v = py_int_base(&s, 16).ok_or_else(|| {
            err(
                "ValueError",
                &format!("invalid literal for int() with base 16: {s:?}"),
            )
        })?;
        match v.parse::<u32>() {
            Ok(x) if x <= 0x10FFFF => Ok(char::from_u32(x).unwrap_or('\u{FFFD}')),
            _ => Err(err("ValueError", "chr() arg not in range(0x110000)")),
        }
    }

    fn basic_string(&mut self) -> Result<PyValue, Error> {
        self.i += 1;
        let mut out = String::new();
        while self.i < self.t.len() {
            let c = self.t[self.i];
            self.i += 1;
            if c == '"' {
                return Ok(PyValue::Str(out));
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
                other => out.push(other),
            }
        }
        Err(err("TomlError", "unterminated basic string"))
    }

    fn literal_string(&mut self) -> Result<PyValue, Error> {
        self.i += 1;
        let Some(k) = self.t[self.i..].iter().position(|&c| c == '\'') else {
            return Err(err("TomlError", "unterminated literal string"));
        };
        let s: String = self.t[self.i..self.i + k].iter().collect();
        self.i += k + 1;
        Ok(PyValue::Str(s))
    }

    fn array(&mut self) -> Result<PyValue, Error> {
        self.i += 1;
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(']') => {
                    self.i += 1;
                    return Ok(PyValue::List(out));
                }
                None => return Err(err("TomlError", "unterminated array")),
                Some(_) => {}
            }
            out.push(self.parse()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {
                    self.i += 1;
                    return Ok(PyValue::List(out));
                }
                _ => return Err(err("TomlError", "expected ',' or ']' in array")),
            }
        }
    }

    fn inline_key(&mut self) -> Result<String, Error> {
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
            return Err(err("TomlError", "expected '=' in inline table"));
        }
        let key: String = self.t[start..self.i].iter().collect();
        self.i += 1;
        Ok(strip(&key))
    }

    fn inline_table(&mut self) -> Result<PyValue, Error> {
        self.i += 1;
        let mut out = PyValue::Dict(Dict::default());
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.i += 1;
                return Ok(out);
            }
            let key = self.inline_key()?;
            let value = self.parse()?;
            assign(&mut out, &split_dotted(&key)?, value)?;
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(out);
                }
                _ => return Err(err("TomlError", "expected ',' or '}' in inline table")),
            }
        }
    }

    fn bare(&mut self) -> Result<PyValue, Error> {
        let start = self.i;
        while self.i < self.t.len() && !",]}\n".contains(self.t[self.i]) {
            self.i += 1;
        }
        let tok = strip(&self.t[start..self.i].iter().collect::<String>());
        if tok.is_empty() {
            return Err(err("TomlError", "empty value"));
        }
        Ok(bare_value(&tok))
    }
}

/// The value of a bare token, as `_ValueParser.bare` reads it: a bool, an
/// int (hex, octal, binary, decimal), a float, or else the token itself as
/// a string (a `ValueError` from `int()`/`float()` returns the token).
/// Python's default limit on the digits `int()` converts from a decimal
/// string (`sys.int_info.default_max_str_digits`).
pub(crate) const MAX_STR_DIGITS: usize = 4300;

fn bare_value(tok: &str) -> PyValue {
    let low = tok.to_lowercase();
    if low == "true" {
        return PyValue::Bool(true);
    }
    if low == "false" {
        return PyValue::Bool(false);
    }
    let t = tok.replace('_', "");
    let tl = t.to_lowercase();
    let based = [("0x", 16), ("0o", 8), ("0b", 2)]
        .iter()
        .find(|(p, _)| tl.starts_with(p));
    if let Some((_, base)) = based {
        return py_int_base(&t, *base)
            .map_or_else(|| PyValue::Str(tok.to_string()), PyValue::from_big);
    }
    if rx(r"^[+-]?\d+$").is_match(&t) {
        // Python 3.10.7+ refuses a decimal string over 4 300 digits
        // (`ValueError`, so the token stays a string)
        if t.trim_start_matches(['+', '-']).chars().count() > MAX_STR_DIGITS {
            return PyValue::Str(tok.to_string());
        }
        return py_int_base(&t, 10)
            .map_or_else(|| PyValue::Str(tok.to_string()), PyValue::from_big);
    }
    py_float(&t).map_or_else(|| PyValue::Str(tok.to_string()), PyValue::Float)
}

/// An int from Python's `int(s, base)`: an optional sign, the base's prefix
/// (`0x`, `0o`, `0b`; optional), digits with single underscores; `None` for
/// a `ValueError`. The value as a decimal string (no size limit).
pub(crate) fn py_int_base(s: &str, base: u32) -> Option<String> {
    let t = s.py_strip();
    let (neg, rest) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let lower = rest.to_lowercase();
    let prefix = match base {
        16 => "0x",
        8 => "0o",
        2 => "0b",
        _ => "",
    };
    let digits = if !prefix.is_empty() && lower.starts_with(prefix) {
        rest[2..].strip_prefix('_').unwrap_or(&rest[2..])
    } else {
        rest
    };
    if digits.is_empty()
        || digits.starts_with('_')
        || digits.ends_with('_')
        || digits.contains("__")
    {
        return None;
    }
    let mut limbs: Vec<u32> = vec![0]; // base 1e9, little endian
    for c in digits.chars().filter(|c| *c != '_') {
        let d = c.to_digit(base)?;
        let mut carry = u64::from(d);
        for l in &mut limbs {
            let v = u64::from(*l) * u64::from(base) + carry;
            *l = (v % 1_000_000_000) as u32;
            carry = v / 1_000_000_000;
        }
        while carry > 0 {
            limbs.push((carry % 1_000_000_000) as u32);
            carry /= 1_000_000_000;
        }
    }
    let mut out = limbs.last().copied().unwrap_or_default().to_string();
    for l in limbs.iter().rev().skip(1) {
        out.push_str(&format!("{l:09}"));
    }
    let is_zero = out.chars().all(|c| c == '0');
    Some(if neg && !is_zero {
        format!("-{out}")
    } else {
        out
    })
}

impl PyValue {
    /// An int from its decimal string: `Int` when it fits i128, else
    /// `BigInt`.
    pub fn from_big(s: String) -> PyValue {
        s.parse::<i128>().map_or(PyValue::BigInt(s), PyValue::Int)
    }
}

/// Python's `float(s)` for ASCII text: decimal and exponent forms, `inf`,
/// `infinity`, `nan` (any case, optional sign); `None` for a `ValueError`.
fn py_float(s: &str) -> Option<f64> {
    let t = s.to_lowercase();
    let (neg, body) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(&t)),
    };
    let v = match body {
        "inf" | "infinity" => f64::INFINITY,
        "nan" => f64::NAN,
        _ if rx(r"^(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?$").is_match(body) => body.parse().ok()?,
        _ => return None,
    };
    Some(if neg { -v } else { v })
}

/// One step of the path to the table statements are written into.
#[derive(Clone)]
enum Step {
    Key(String),
    Last,
}

fn attr_err(v: &PyValue) -> Error {
    err(
        "AttributeError",
        &format!("'{}' object has no attribute 'setdefault'", v.type_name()),
    )
}

/// `target.setdefault(p, default)` on a value that must be a dict.
fn setdefault<'a>(
    target: &'a mut PyValue,
    p: &str,
    default: PyValue,
) -> Result<&'a mut PyValue, Error> {
    match target {
        PyValue::Dict(d) => Ok(d.setdefault(p, default)),
        other => Err(attr_err(other)),
    }
}

/// `target = target.setdefault(p, {})` for each step, as `parse_toml` walks
/// a header (a list is replaced by its last element).
fn walk<'a>(root: &'a mut PyValue, path: &[Step]) -> Result<&'a mut PyValue, Error> {
    let mut t = root;
    for s in path {
        t = match s {
            Step::Key(k) => setdefault(t, k, PyValue::Dict(Dict::default()))?,
            Step::Last => match t {
                PyValue::List(a) => a
                    .last_mut()
                    .ok_or_else(|| err("IndexError", "list index out of range"))?,
                other => return Err(attr_err(other)),
            },
        };
    }
    Ok(t)
}

/// `target = current; for p in parts[:-1]: target = target.setdefault(p,
/// {}); target[parts[-1]] = value`.
fn assign(current: &mut PyValue, parts: &[String], value: PyValue) -> Result<(), Error> {
    let (last, head) = parts
        .split_last()
        .ok_or_else(|| err("TomlError", "empty key"))?;
    let mut t = current;
    for p in head {
        t = setdefault(t, p, PyValue::Dict(Dict::default()))?;
    }
    match t {
        PyValue::Dict(d) => {
            d.insert(last, value);
            Ok(())
        }
        PyValue::List(_) => Err(err(
            "TypeError",
            "list indices must be integers or slices, not str",
        )),
        other => Err(err(
            "TypeError",
            &format!(
                "'{}' object does not support item assignment",
                other.type_name()
            ),
        )),
    }
}

/// The header path with `Last` after every component that is a list now.
fn header_path(root: &mut PyValue, parts: &[String]) -> Result<Vec<Step>, Error> {
    let mut path = Vec::new();
    for p in parts {
        path.push(Step::Key(p.clone()));
        if matches!(walk(root, &path)?, PyValue::List(_)) {
            path.push(Step::Last);
        }
    }
    Ok(path)
}

fn table_array(root: &mut PyValue, stmt: &str) -> Result<Vec<Step>, Error> {
    let inner: String = stmt
        .chars()
        .skip(2)
        .take(stmt.chars().count() - 4)
        .collect();
    let parts = split_dotted(&inner)?;
    let (last, head) = parts
        .split_last()
        .ok_or_else(|| err("TomlError", "empty key"))?;
    let mut path = header_path(root, head)?;
    let arr = setdefault(walk(root, &path)?, last, PyValue::List(Vec::new()))?;
    let PyValue::List(a) = arr else {
        return Err(err(
            "TomlError",
            &format!("[[{}]] is not an array of tables", parts.join(".")),
        ));
    };
    a.push(PyValue::Dict(Dict::default()));
    path.push(Step::Key(last.clone()));
    path.push(Step::Last);
    Ok(path)
}

fn key_value(root: &mut PyValue, current: &[Step], stmt: &str) -> Result<(), Error> {
    let chars: Vec<char> = stmt.chars().collect();
    let mut quote: Option<char> = None;
    let mut eq = None;
    for (k, &c) in chars.iter().enumerate() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c == '=' {
            eq = Some(k);
            break;
        }
    }
    let Some(eq) = eq else {
        return Err(err("TomlError", "not a key/value statement"));
    };
    let key = strip(&chars[..eq].iter().collect::<String>());
    let mut parser = ValueParser {
        t: chars[eq + 1..].to_vec(),
        i: 0,
    };
    let value = parser.parse()?;
    parser.skip_ws();
    if parser.i != parser.t.len() {
        return Err(err("TomlError", "trailing text after value"));
    }
    let parts = split_dotted(&key)?;
    assign(walk(root, current)?, &parts, value)
}

/// `parse_toml(src)`: the root dict, or the reader's exception.
pub fn parse(src: &str) -> Result<Dict, Error> {
    let chars: Vec<char> = src.chars().collect();
    let mut root = PyValue::Dict(Dict::default());
    let mut current: Vec<Step> = Vec::new();
    for stmt in scan_statements(&chars)? {
        if stmt.starts_with("[[") && stmt.ends_with("]]") && stmt.chars().count() >= 4 {
            current = table_array(&mut root, &stmt)?;
        } else if stmt.starts_with('[') && stmt.ends_with(']') && stmt.chars().count() >= 2 {
            let inner: String = stmt
                .chars()
                .skip(1)
                .take(stmt.chars().count() - 2)
                .collect();
            let parts = split_dotted(&inner)?;
            current = header_path(&mut root, &parts)?;
            walk(&mut root, &current)?;
        } else {
            key_value(&mut root, &current, &stmt)?;
        }
    }
    match root {
        PyValue::Dict(d) => Ok(d),
        _ => unreachable!("the root is a dict"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::pyvalue::canonical;
    use super::*;

    #[test]
    fn the_script_s_quirks() {
        let d = parse("a = 040\nb = C-116\nc = 1979-05-27\nd = 1\nd = 2\ne = 0x28\n").unwrap();
        assert_eq!(d.get("a"), Some(&PyValue::Int(40)));
        assert_eq!(d.get("b"), Some(&PyValue::Str("C-116".into())));
        assert_eq!(d.get("c"), Some(&PyValue::Str("1979-05-27".into())));
        assert_eq!(d.get("d"), Some(&PyValue::Int(2)));
        assert_eq!(d.get("e"), Some(&PyValue::Int(40)));
        assert!(parse("n = \"\"\"x\"\"\"\n").is_err());
        assert!(parse("n = '''x'''\n").is_err());
        let big = parse("n = 123456789012345678901234567890123456789012\n").unwrap();
        assert_eq!(
            canonical(big.get("n").unwrap(), false),
            "i123456789012345678901234567890123456789012"
        );
    }

    #[test]
    fn tables_and_arrays_of_tables() {
        let d = parse("[[m]]\nname = \"a\"\n[[m.f]]\nx = 1\n[[m]]\nname = \"b\"\n[t]\ny.z = 2\n")
            .unwrap();
        assert_eq!(
            canonical(&PyValue::Dict(d), false),
            r#"{s"m":[{s"name":s"a",s"f":[{s"x":i1}]},{s"name":s"b"}],s"t":{s"y":{s"z":i2}}}"#
        );
    }
}
