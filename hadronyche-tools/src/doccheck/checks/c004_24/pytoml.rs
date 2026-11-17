//! The TOML subset reader the 004-16 to 004-29 scripts carry (Python 3.10,
//! no `tomllib`), ported line for line: `_scan_statements`,
//! `_split_dotted`, `_ValueParser` and `parse_toml`.
//!
//! It differs from a TOML parser in both directions, and the ports must
//! read wire.toml as their scripts did: it rejects multi-line strings
//! (`"""…"""` scans as one statement and then fails as trailing text),
//! which the `toml` crate accepts — reading with the crate would make a port
//! weaker; it accepts a bare word as a string (`width = abc`), a duplicate
//! key (the later wins), `1_2_`, unknown escapes, trailing commas in inline
//! tables, which the crate rejects. Every Python exception of the reader
//! (`TomlError`, `ValueError`, `AttributeError`, `IndexError`) is an `Err`:
//! the scripts catch `Exception` around it and report the file as not
//! parsing.
//!
//! Values map to `toml::Value`: `str` to String, `int` to Integer (an `int`
//! beyond 64 bits to Float, never equal to a stated integer), `float`,
//! `bool`, `list` to Array, `dict` to Table.

use toml::{Table, Value};

use crate::doccheck::text::full;

type R<T> = Result<T, String>;

/// `_scan_statements`.
fn scan_statements(src: &[char]) -> R<Vec<String>> {
    let (mut stmts, mut buf) = (Vec::new(), String::new());
    let (mut i, n) = (0, src.len());
    let mut in_str: Option<Vec<char>> = None;
    let mut depth = 0usize;
    let starts = |at: usize, pat: &[char]| src.get(at..at + pat.len()) == Some(pat);
    while i < n {
        let c = src[i];
        if let Some(q) = &in_str {
            if (q == &['"'] || q == &['"', '"', '"']) && c == '\\' {
                buf.extend(&src[i..(i + 2).min(n)]);
                i += 2;
                continue;
            }
            if starts(i, q) {
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
            let triple = [c, c, c];
            if starts(i, &triple) {
                in_str = Some(triple.to_vec());
                buf.extend(triple);
                i += 3;
            } else {
                in_str = Some(vec![c]);
                buf.push(c);
                i += 1;
            }
            continue;
        }
        if c == '[' || c == '{' {
            depth += 1;
        } else if c == ']' || c == '}' {
            depth = depth.saturating_sub(1);
        }
        if c == '\n' {
            if depth == 0 {
                let s = py_strip(&buf);
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
    let s = py_strip(&buf);
    if !s.is_empty() {
        stmts.push(s);
    }
    if in_str.is_some() {
        return Err("unterminated string".into());
    }
    Ok(stmts)
}

/// Python's `str.strip()` (its whitespace includes U+001C to U+001F).
fn py_strip(s: &str) -> String {
    s.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)).to_string()
}

/// `_split_dotted`.
fn split_dotted(key: &str) -> R<Vec<String>> {
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
            parts.push(py_strip(&cur));
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    parts.push(py_strip(&cur));
    let mut out = Vec::new();
    for p in parts {
        let ch: Vec<char> = p.chars().collect();
        let p = if ch.len() >= 2 && ch[0] == ch[ch.len() - 1] && (ch[0] == '"' || ch[0] == '\'') {
            ch[1..ch.len() - 1].iter().collect()
        } else {
            p
        };
        if p.is_empty() {
            return Err("empty key component".into());
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

    fn parse(&mut self) -> R<Value> {
        self.skip_ws();
        match self.peek() {
            None => Err("missing value".into()),
            Some('"') => self.basic_string().map(Value::String),
            Some('\'') => self.literal_string().map(Value::String),
            Some('[') => self.array(),
            Some('{') => self.inline_table(),
            Some(_) => self.bare(),
        }
    }

    fn hex(&mut self, len: usize) -> R<char> {
        let s: String = self.t.get(self.i..(self.i + len).min(self.t.len())).unwrap_or(&[]).iter().collect();
        let v = u32::from_str_radix(s.trim(), 16).map_err(|e| format!("ValueError: {e}"))?;
        self.i += len;
        // Python builds a lone surrogate; Rust cannot, and no check reads it.
        Ok(char::from_u32(v).unwrap_or('\u{fffd}'))
    }

    fn basic_string(&mut self) -> R<String> {
        self.i += 1;
        let mut out = String::new();
        while self.i < self.t.len() {
            let c = self.t[self.i];
            self.i += 1;
            if c == '"' {
                return Ok(out);
            }
            if c != '\\' {
                out.push(c);
                continue;
            }
            let Some(e) = self.peek() else { break };
            self.i += 1;
            match e {
                'u' => out.push(self.hex(4)?),
                'U' => out.push(self.hex(8)?),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                other => out.push(other),
            }
        }
        Err("unterminated basic string".into())
    }

    fn literal_string(&mut self) -> R<String> {
        self.i += 1;
        let j = self.t[self.i..].iter().position(|&c| c == '\'').ok_or("unterminated literal string")?;
        let s = self.t[self.i..self.i + j].iter().collect();
        self.i += j + 1;
        Ok(s)
    }

    fn array(&mut self) -> R<Value> {
        self.i += 1;
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(']') => {
                    self.i += 1;
                    return Ok(Value::Array(out));
                }
                None => return Err("unterminated array".into()),
                _ => {}
            }
            out.push(self.parse()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {
                    self.i += 1;
                    return Ok(Value::Array(out));
                }
                _ => return Err("expected ',' or ']' in array".into()),
            }
        }
    }

    fn inline_table(&mut self) -> R<Value> {
        self.i += 1;
        let mut out = Table::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.i += 1;
                return Ok(Value::Table(out));
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
                return Err("expected '=' in inline table".into());
            }
            let key: String = self.t[start..self.i].iter().collect();
            self.i += 1;
            let value = self.parse()?;
            set_path(&mut out, &split_dotted(&py_strip(&key))?, value)?;
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(Value::Table(out));
                }
                _ => return Err("expected ',' or '}' in inline table".into()),
            }
        }
    }

    fn bare(&mut self) -> R<Value> {
        let start = self.i;
        while self.i < self.t.len() && !",]}\n".contains(self.t[self.i]) {
            self.i += 1;
        }
        let tok = py_strip(&self.t[start..self.i].iter().collect::<String>());
        if tok.is_empty() {
            return Err("empty value".into());
        }
        Ok(bare_value(&tok))
    }
}

/// A bare token as the reader reads it: a bool, an int (0x, 0o, 0b or
/// decimal), a float, or else the token itself as a string.
fn bare_value(tok: &str) -> Value {
    let low = tok.to_lowercase();
    if low == "true" {
        return Value::Boolean(true);
    }
    if low == "false" {
        return Value::Boolean(false);
    }
    let t = tok.replace('_', "");
    let tl = t.to_lowercase();
    let radix = [("0x", 16), ("0o", 8), ("0b", 2)].into_iter().find(|(p, _)| tl.starts_with(p));
    let int = |s: &str, r: u32| -> Option<Value> {
        let v = i128::from_str_radix(s.trim(), r).ok()?;
        Some(i64::try_from(v).map_or(Value::Float(v as f64), Value::Integer))
    };
    let parsed = match radix {
        Some((_, r)) => int(&t[2..], r),
        None if full(r"[+-]?\d+", &t) => int(&t, 10).or(Some(Value::Float(f64::INFINITY))),
        None => py_float(&t).map(Value::Float),
    };
    parsed.unwrap_or_else(|| Value::String(tok.to_string()))
}

/// Python's `float()` on the forms it accepts.
fn py_float(t: &str) -> Option<f64> {
    let s = t.trim().to_lowercase();
    let body = s.trim_start_matches(['+', '-']);
    if ["inf", "infinity", "nan"].contains(&body) {
        let neg = s.starts_with('-');
        let v = if body == "nan" { f64::NAN } else { f64::INFINITY };
        return Some(if neg { -v } else { v });
    }
    if !full(r"[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?", &s) {
        return None;
    }
    s.parse().ok()
}

/// `target.setdefault(p, {})` down `parts[..-1]`, then `target[last] = value`.
fn set_path(root: &mut Table, parts: &[String], value: Value) -> R<()> {
    let mut target = root;
    for p in &parts[..parts.len() - 1] {
        let next = target.entry(p.clone()).or_insert_with(|| Value::Table(Table::new()));
        target = match next {
            Value::Table(t) => t,
            _ => return Err("AttributeError: setdefault on a value".into()),
        };
    }
    target.insert(parts[parts.len() - 1].clone(), value);
    Ok(())
}

/// Walks `path` from the root as `[...]` / `[[...]]` do: `setdefault(p, {})`,
/// descending into the last element of an array.
fn walk<'a>(root: &'a mut Table, path: &[String]) -> R<&'a mut Table> {
    let mut target = root;
    for p in path {
        let next = target.entry(p.clone()).or_insert_with(|| Value::Table(Table::new()));
        target = match next {
            Value::Table(t) => t,
            Value::Array(a) => match a.last_mut() {
                Some(Value::Table(t)) => t,
                Some(_) => return Err("AttributeError: not a table".into()),
                None => return Err("IndexError: empty array".into()),
            },
            _ => return Err("AttributeError: setdefault on a value".into()),
        };
    }
    Ok(target)
}

/// Where the next key/value statements go: the root, or a path whose
/// array-of-tables steps take their last element.
#[derive(Clone)]
struct Cursor(Vec<String>);

/// `parse_toml`.
pub(crate) fn parse_toml(src: &str) -> R<Table> {
    let chars: Vec<char> = src.chars().collect();
    let mut root = Table::new();
    let mut current = Cursor(Vec::new());
    for stmt in scan_statements(&chars)? {
        if stmt.starts_with("[[") && stmt.ends_with("]]") && stmt.chars().count() >= 4 {
            let inner: String = stmt.chars().skip(2).take(stmt.chars().count() - 4).collect();
            let path = split_dotted(&inner)?;
            let (last, head) = path.split_last().expect("split_dotted returns a part");
            let target = walk(&mut root, head)?;
            let arr = target.entry(last.clone()).or_insert_with(|| Value::Array(Vec::new()));
            let Value::Array(a) = arr else {
                return Err(format!("[[{}]] is not an array of tables", path.join(".")));
            };
            a.push(Value::Table(Table::new()));
            current = Cursor(path);
            continue;
        }
        if stmt.starts_with('[') && stmt.ends_with(']') && stmt.chars().count() >= 2 {
            let inner: String = stmt.chars().skip(1).take(stmt.chars().count() - 2).collect();
            let path = split_dotted(&inner)?;
            walk(&mut root, &path)?;
            current = Cursor(path);
            continue;
        }
        key_value(&mut root, &current, &stmt)?;
    }
    Ok(root)
}

fn key_value(root: &mut Table, current: &Cursor, stmt: &str) -> R<()> {
    let ch: Vec<char> = stmt.chars().collect();
    let mut quote: Option<char> = None;
    let mut eq = None;
    for (k, &c) in ch.iter().enumerate() {
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
    let eq = eq.ok_or_else(|| format!("not a key/value statement: {:?}", ch.iter().take(60).collect::<String>()))?;
    let key = py_strip(&ch[..eq].iter().collect::<String>());
    let mut parser = ValueParser { t: ch[eq + 1..].to_vec(), i: 0 };
    let value = parser.parse()?;
    parser.skip_ws();
    if parser.i != parser.t.len() {
        return Err("trailing text after value".into());
    }
    let parts = split_dotted(&key)?;
    let target = walk(root, &current.0)?;
    set_path(target, &parts, value)
}

/// `TOML.get(key, [])` iterated as Python iterates it: a list's elements, a
/// string's characters, a table's keys; `Err` for a value Python cannot
/// iterate (an int, a float, a bool).
pub(crate) fn py_iter(v: Option<&Value>) -> Result<Vec<Value>, String> {
    match v {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => Ok(a.clone()),
        Some(Value::String(s)) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        Some(Value::Table(t)) => Ok(t.keys().map(|k| Value::String(k.clone())).collect()),
        Some(other) => Err(format!("TypeError: {other} is not iterable")),
    }
}

/// The dicts of `TOML.get(key, [])` (`[m for m in … if isinstance(m, dict)]`).
pub(crate) fn dicts(v: Option<&Value>) -> Vec<Table> {
    py_iter(v).unwrap_or_default().into_iter().filter_map(|x| if let Value::Table(t) = x { Some(t) } else { None }).collect()
}

/// The tables of `t.get(key, [])` when every element must be one (the
/// script calls `.get` on each element, which raises on anything else).
pub(crate) fn tables_strict(v: Option<&Value>) -> Result<Vec<Table>, String> {
    py_iter(v)?
        .into_iter()
        .map(|x| match x {
            Value::Table(t) => Ok(t),
            other => Err(format!("AttributeError: {other} has no get")),
        })
        .collect()
}

/// The elements of `t.get(key, [])` as hashable values printed as Python
/// prints them (`set(…)`, `r in dict`); `Err` for an unhashable element.
pub(crate) fn names(v: Option<&Value>) -> Result<Vec<String>, String> {
    py_iter(v)?
        .into_iter()
        .map(|x| match x {
            Value::Array(_) | Value::Table(_) => Err("TypeError: unhashable".to_string()),
            other => Ok(crate::doccheck::wire_toml::display(&other)),
        })
        .collect()
}

/// Python's `v == n` for an integer `n` (an int, a float or a bool).
pub(crate) fn eq_int(v: Option<&Value>, n: Option<i64>) -> bool {
    match (v, n) {
        (None, None) => true,
        (Some(Value::Integer(i)), Some(n)) => *i == n,
        (Some(Value::Float(f)), Some(n)) => *f == n as f64,
        (Some(Value::Boolean(b)), Some(n)) => i64::from(*b) == n,
        _ => false,
    }
}

/// `isinstance(v, int)` (a bool is an int in Python).
pub(crate) fn as_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Integer(i)) => Some(*i),
        Some(Value::Boolean(b)) => Some(i64::from(*b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_subset_reader() {
        let t = parse_toml("a = 1\n[[message]]\nname = \"m\" # c\nx = [1,\n 2]\n[[message.field]]\nw = 0x10\nb = abc\n").unwrap();
        let m = t["message"].as_array().unwrap()[0].as_table().unwrap();
        assert_eq!(m["name"].as_str(), Some("m"));
        assert_eq!(m["x"].as_array().unwrap().len(), 2);
        let f = m["field"].as_array().unwrap()[0].as_table().unwrap();
        assert_eq!(f["w"].as_integer(), Some(16));
        assert_eq!(f["b"].as_str(), Some("abc"), "a bare word is a string");
        assert!(parse_toml("a = \"\"\"x\"\"\"\n").is_err(), "multi-line strings are not read");
        assert!(parse_toml("a = \"x\n").is_err());
        assert!(parse_toml("a = 1\na = 2\n").is_ok(), "a duplicate key: the later wins");
        assert!(parse_toml("[[a]]\n[[a.b]]\nx = 1\n").is_ok());
        assert!(parse_toml("a = 1\n[a.b]\n").is_err());
        assert_eq!(py_iter(Some(&Value::String("ab".into()))).unwrap().len(), 2);
        assert!(py_iter(Some(&Value::Integer(1))).is_err());
    }
}
