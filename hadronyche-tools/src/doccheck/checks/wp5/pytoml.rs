//! The scripts' own TOML subset reader (`_scan_statements`, `_split_dotted`,
//! `_ValueParser`, `parse_toml`), identical in 19 scripts of the 004 family
//! (004-03, 06, 08 to 13, 16 to 20, 24 to 29), reproduced statement for
//! statement with the Python values it builds. It is not TOML: it accepts
//! duplicate keys (the last wins), reads `46.0`, `true`, `0x2e` and bare
//! words its own way (an unparsable bare value is kept as a string), and
//! rejects `"""` strings it cannot close. Python containers keep their
//! dynamic typing: assigning into a non-table, `nxt[-1]` on an empty list and
//! a bad `\u` escape raise an exception the script does not catch, which
//! made the script crash (red, no FAIL line): [`Fail::Crash`].

use super::big::Big;
use super::py::{float_repr, py_digit, py_float, py_int, py_int_base, py_strip, str_repr};

/// A Python value built by the reader.
#[derive(Clone, Debug)]
pub(crate) enum Value {
    Int(Big),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Dict(Dict),
}

/// A Python `dict` (insertion order; assigning an existing key keeps its
/// place).
#[derive(Clone, Debug, Default)]
pub(crate) struct Dict(pub Vec<(String, Value)>);

impl Dict {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// `d[key] = value`.
    pub fn set(&mut self, key: &str, value: Value) {
        match self.get_mut(key) {
            Some(slot) => *slot = value,
            None => self.0.push((key.to_string(), value)),
        }
    }

    /// `d.setdefault(key, default)`.
    fn setdefault(&mut self, key: &str, default: Value) -> &mut Value {
        if self.get(key).is_none() {
            self.0.push((key.to_string(), default));
        }
        self.get_mut(key).expect("just set")
    }
}

/// Why reading failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Fail {
    /// `TomlError`: the script reports "could not be read by this check".
    Error(String),
    /// Any other exception: the script crashed.
    Crash(String),
}

type R<T> = Result<T, Fail>;

fn err<T>(m: impl Into<String>) -> R<T> {
    Err(Fail::Error(m.into()))
}

fn crash<T>(m: impl Into<String>) -> R<T> {
    Err(Fail::Crash(m.into()))
}

impl Value {
    /// Python truthiness.
    pub fn truthy(&self) -> bool {
        match self {
            Value::Int(b) => !b.is_zero(),
            Value::Float(f) => *f != 0.0,
            Value::Bool(b) => *b,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Dict(d) => !d.0.is_empty(),
        }
    }

    /// `isinstance(v, int)`: an int or a bool, as an integer.
    pub fn int_like(&self) -> Option<Big> {
        match self {
            Value::Int(b) => Some(b.clone()),
            Value::Bool(b) => Some(Big::from(i64::from(*b))),
            _ => None,
        }
    }

    /// `isinstance(v, int) and not isinstance(v, bool)`.
    pub fn strict_int(&self) -> Option<&Big> {
        match self {
            Value::Int(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&Dict> {
        match self {
            Value::Dict(d) => Some(d),
            _ => None,
        }
    }

    /// `str(v)`.
    pub fn py_str(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            other => other.repr(),
        }
    }

    /// `repr(v)`.
    pub fn repr(&self) -> String {
        match self {
            Value::Int(b) => b.to_string(),
            Value::Float(f) => float_repr(*f),
            Value::Bool(true) => "True".into(),
            Value::Bool(false) => "False".into(),
            Value::Str(s) => str_repr(s),
            Value::List(l) => format!("[{}]", l.iter().map(Value::repr).collect::<Vec<_>>().join(", ")),
            Value::Dict(d) => format!(
                "{{{}}}",
                d.0.iter().map(|(k, v)| format!("{}: {}", str_repr(k), v.repr())).collect::<Vec<_>>().join(", ")
            ),
        }
    }

    /// `v == other` with Python's numeric tower (`46 == 46.0 == ...`,
    /// `True == 1`).
    pub fn py_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::List(a), Value::List(b)) => a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.py_eq(y)),
            (Value::Dict(a), Value::Dict(b)) => {
                a.0.len() == b.0.len() && a.0.iter().all(|(k, v)| b.get(k).is_some_and(|w| v.py_eq(w)))
            }
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Float(f), n) | (n, Value::Float(f)) => match n.int_like() {
                Some(i) => float_eq_int(*f, &i),
                None => false,
            },
            (a, b) => match (a.int_like(), b.int_like()) {
                (Some(x), Some(y)) => x == y,
                _ => false,
            },
        }
    }

    /// `v == n` for an integer `n`.
    pub fn eq_int(&self, n: i64) -> bool {
        self.py_eq(&Value::Int(Big::from(n)))
    }

    /// Hashable (a member test against a set of strings): lists and dicts
    /// raise `TypeError: unhashable type`.
    pub fn hashable(&self) -> bool {
        !matches!(self, Value::List(_) | Value::Dict(_))
    }

    /// `for x in (v or [])`: the items, or a crash for a number.
    pub fn iter_or_empty(v: Option<&Value>) -> R<Vec<Value>> {
        match v {
            None => Ok(Vec::new()),
            Some(v) if !v.truthy() => Ok(Vec::new()),
            Some(Value::List(l)) => Ok(l.clone()),
            Some(Value::Dict(d)) => Ok(d.0.iter().map(|(k, _)| Value::Str(k.clone())).collect()),
            Some(Value::Str(s)) => Ok(s.chars().map(|c| Value::Str(c.to_string())).collect()),
            Some(other) => crash(format!("TypeError: '{}' object is not iterable", other.type_name())),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
        }
    }
}

fn float_eq_int(f: f64, i: &Big) -> bool {
    if !f.is_finite() || f.fract() != 0.0 {
        return false;
    }
    let digits = format!("{:.0}", f.abs());
    let v = Big::from_digits(&digits).map(|b| if f < 0.0 { b.neg() } else { b });
    v.as_ref() == Some(i)
}

/// `_scan_statements(src)`.
fn scan_statements(src: &[char]) -> R<Vec<String>> {
    let mut stmts = Vec::new();
    let mut buf = String::new();
    let mut i = 0;
    let n = src.len();
    let mut in_str: Option<Vec<char>> = None;
    let mut depth = 0usize;
    let starts = |i: usize, pat: &[char]| src.len() >= i + pat.len() && &src[i..i + pat.len()] == pat;
    while i < n {
        let c = src[i];
        if let Some(q) = &in_str {
            if q[0] == '"' && c == '\\' {
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
                None => break,
                Some(j) => i += j,
            }
            continue;
        }
        if c == '"' || c == '\'' {
            let q = if starts(i, &[c, c, c]) { vec![c, c, c] } else { vec![c] };
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
                let s = py_strip(&buf);
                if !s.is_empty() {
                    stmts.push(s.to_string());
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
        stmts.push(s.to_string());
    }
    if in_str.is_some() {
        return err("unterminated string");
    }
    Ok(stmts)
}

/// `_split_dotted(key)`.
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
            parts.push(py_strip(&cur).to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    parts.push(py_strip(&cur).to_string());
    let mut out = Vec::new();
    for p in parts {
        let ch: Vec<char> = p.chars().collect();
        let p = if ch.len() >= 2 && ch[0] == ch[ch.len() - 1] && (ch[0] == '"' || ch[0] == '\'') {
            ch[1..ch.len() - 1].iter().collect()
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

/// `_ValueParser`.
struct Parser {
    t: Vec<char>,
    i: usize,
}

impl Parser {
    fn skip_ws(&mut self) {
        while self.i < self.t.len() && matches!(self.t[self.i], ' ' | '\t' | '\r' | '\n') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.t.get(self.i).copied()
    }

    fn parse(&mut self) -> R<Value> {
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

    /// `chr(int(self.t[i:i+k], 16))`: a bad escape is a crash.
    fn escape(&mut self, k: usize) -> R<char> {
        let end = (self.i + k).min(self.t.len());
        let hex: String = self.t[self.i..end].iter().collect();
        self.i += k;
        let Some(v) = py_int_base(&hex, 16) else {
            return crash(format!("ValueError: invalid literal for int() with base 16: {hex:?}"));
        };
        match v.to_i64() {
            Some(n) if (0..=0x10FFFF).contains(&n) => {
                Ok(char::from_u32(n as u32).unwrap_or('\u{FFFD}'))
            }
            _ => crash("ValueError: chr() arg not in range(0x110000)"),
        }
    }

    fn basic_string(&mut self) -> R<Value> {
        self.i += 1;
        let mut out = String::new();
        while self.i < self.t.len() {
            let c = self.t[self.i];
            self.i += 1;
            if c == '"' {
                return Ok(Value::Str(out));
            }
            if c != '\\' {
                out.push(c);
                continue;
            }
            if self.i >= self.t.len() {
                break;
            }
            let e = self.t[self.i];
            self.i += 1;
            let ch = match e {
                'u' => self.escape(4)?,
                'U' => self.escape(8)?,
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                'b' => '\u{8}',
                'f' => '\u{c}',
                other => other,
            };
            out.push(ch);
        }
        err("unterminated basic string")
    }

    fn literal_string(&mut self) -> R<Value> {
        self.i += 1;
        let Some(j) = self.t[self.i..].iter().position(|&c| c == '\'') else {
            return err("unterminated literal string");
        };
        let s: String = self.t[self.i..self.i + j].iter().collect();
        self.i += j + 1;
        Ok(Value::Str(s))
    }

    fn array(&mut self) -> R<Value> {
        self.i += 1;
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(']') => {
                    self.i += 1;
                    return Ok(Value::List(out));
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
                    return Ok(Value::List(out));
                }
                _ => return err("expected ',' or ']' in array"),
            }
        }
    }

    fn inline_table(&mut self) -> R<Value> {
        self.i += 1;
        let mut out = Value::Dict(Dict::default());
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.i += 1;
                return Ok(out);
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
            let parts = split_dotted(py_strip(&key))?;
            assign_dotted(&mut out, &parts, value)?;
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(out);
                }
                _ => return err("expected ',' or '}' in inline table"),
            }
        }
    }

    fn bare(&mut self) -> R<Value> {
        let start = self.i;
        while self.i < self.t.len() && !matches!(self.t[self.i], ',' | ']' | '}' | '\n') {
            self.i += 1;
        }
        let raw: String = self.t[start..self.i].iter().collect();
        let tok = py_strip(&raw).to_string();
        if tok.is_empty() {
            return err("empty value");
        }
        let low = tok.to_lowercase();
        if low == "true" {
            return Ok(Value::Bool(true));
        }
        if low == "false" {
            return Ok(Value::Bool(false));
        }
        let t = tok.replace('_', "");
        let tl = t.to_lowercase();
        let parsed = if tl.starts_with("0x") {
            py_int_base(&t, 16).map(Value::Int)
        } else if tl.starts_with("0o") {
            py_int_base(&t, 8).map(Value::Int)
        } else if tl.starts_with("0b") {
            py_int_base(&t, 2).map(Value::Int)
        } else if is_signed_digits(&t) {
            py_int(&t).map(Value::Int)
        } else {
            py_float(&t).map(Value::Float)
        };
        Ok(parsed.unwrap_or(Value::Str(tok)))
    }
}

/// `re.fullmatch(r"[+-]?\d+", t)`.
fn is_signed_digits(t: &str) -> bool {
    let body = t.strip_prefix(['+', '-']).unwrap_or(t);
    !body.is_empty() && body.chars().all(py_digit)
}

/// `target = out; for p in parts[:-1]: target = target.setdefault(p, {});
/// target[parts[-1]] = value` (no list handling: a list or scalar on the way
/// is a crash).
fn assign_dotted(root: &mut Value, parts: &[String], value: Value) -> R<()> {
    let mut target = root;
    for p in &parts[..parts.len() - 1] {
        let Value::Dict(d) = target else {
            return crash(format!("AttributeError: '{}' object has no attribute 'setdefault'", target.type_name()));
        };
        target = d.setdefault(p, Value::Dict(Dict::default()));
    }
    match target {
        Value::Dict(d) => {
            d.set(&parts[parts.len() - 1], value);
            Ok(())
        }
        other => crash(format!("TypeError: '{}' object does not support item assignment", other.type_name())),
    }
}

/// A step from the root to the current table.
#[derive(Clone, Debug)]
enum Step {
    Key(String),
    Last,
}

fn resolve<'a>(root: &'a mut Value, path: &[Step]) -> &'a mut Value {
    let mut node = root;
    for step in path {
        node = match (step, node) {
            (Step::Key(k), Value::Dict(d)) => d.get_mut(k).expect("the path was built through it"),
            (Step::Last, Value::List(l)) => l.last_mut().expect("the path was built through it"),
            _ => unreachable!("the path was built through tables and lists"),
        };
    }
    node
}

/// Walks `path` from the root as the `[table]` branch does: `setdefault(p,
/// {})`, and the last entry of a list.
fn walk_table(root: &mut Value, path: &[String]) -> R<Vec<Step>> {
    let mut steps = Vec::new();
    for p in path {
        let node = resolve(root, &steps);
        let Value::Dict(d) = node else {
            return crash(format!("AttributeError: '{}' object has no attribute 'setdefault'", node.type_name()));
        };
        let nxt = d.setdefault(p, Value::Dict(Dict::default()));
        steps.push(Step::Key(p.clone()));
        if let Value::List(l) = nxt {
            if l.is_empty() {
                return crash("IndexError: list index out of range");
            }
            steps.push(Step::Last);
        }
    }
    Ok(steps)
}

/// `[[a.b]]`.
fn array_table(root: &mut Value, path: &[String]) -> R<Vec<Step>> {
    let mut steps = walk_table(root, &path[..path.len() - 1])?;
    let last = &path[path.len() - 1];
    let node = resolve(root, &steps);
    let Value::Dict(d) = node else {
        return crash(format!("AttributeError: '{}' object has no attribute 'setdefault'", node.type_name()));
    };
    let arr = d.setdefault(last, Value::List(Vec::new()));
    let Value::List(l) = arr else {
        return err(format!("[[{}]] is not an array of tables", path.join(".")));
    };
    l.push(Value::Dict(Dict::default()));
    steps.push(Step::Key(last.clone()));
    steps.push(Step::Last);
    Ok(steps)
}

/// The `=` of a key/value statement, outside quotes.
fn equals_at(stmt: &[char]) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (k, &c) in stmt.iter().enumerate() {
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

/// `parse_toml(src)`: the root table.
pub(crate) fn parse(src: &str) -> R<Dict> {
    let chars: Vec<char> = src.chars().collect();
    let mut root = Value::Dict(Dict::default());
    let mut current: Vec<Step> = Vec::new();
    for stmt in scan_statements(&chars)? {
        let sc: Vec<char> = stmt.chars().collect();
        let n = sc.len();
        if stmt.starts_with("[[") && stmt.ends_with("]]") && n >= 4 {
            let inner: String = sc[2..n - 2].iter().collect();
            current = array_table(&mut root, &split_dotted(&inner)?)?;
            continue;
        }
        if stmt.starts_with('[') && stmt.ends_with(']') {
            let inner: String = if n >= 2 { sc[1..n - 1].iter().collect() } else { String::new() };
            current = walk_table(&mut root, &split_dotted(&inner)?)?;
            continue;
        }
        let Some(eq) = equals_at(&sc) else {
            return err(format!("not a key/value statement: {}", str_repr(&sc.iter().take(60).collect::<String>())));
        };
        let key: String = sc[..eq].iter().collect();
        let mut parser = Parser { t: sc[eq + 1..].to_vec(), i: 0 };
        let value = parser.parse()?;
        parser.skip_ws();
        if parser.i != parser.t.len() {
            return err("trailing text after value");
        }
        let parts = split_dotted(py_strip(&key))?;
        assign_dotted(resolve(&mut root, &current), &parts, value)?;
    }
    match root {
        Value::Dict(d) => Ok(d),
        _ => unreachable!("the root is a table"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(d: &'a Dict, k: &str) -> &'a Value {
        d.get(k).unwrap_or_else(|| panic!("no {k}"))
    }

    #[test]
    fn reads_the_subset_as_python_did() {
        let d = parse("a = 46.0\nb = TRUE\nc = 0x4d_4e\nd = two words\ne = [1, 'x', {k = 2}]\n[[m]]\nname = \"p\" # c\n[[m]]\nname = 'q'\n[m.f]\nx = 1\na = 1\n").unwrap();
        assert!(get(&d, "a").eq_int(46));
        assert!(get(&d, "b").eq_int(1));
        assert!(get(&d, "c").eq_int(0x4d4e));
        assert_eq!(get(&d, "d").as_str(), Some("two words"));
        assert_eq!(get(&d, "e").repr(), "[1, 'x', {'k': 2}]");
        let Value::List(m) = get(&d, "m") else { panic!() };
        assert_eq!(m.len(), 2);
        assert_eq!(m[1].repr(), "{'name': 'q', 'f': {'x': 1, 'a': 1}}");
    }

    #[test]
    fn errors_and_crashes_as_python_raised_them() {
        assert!(matches!(parse("a = \"\"\"x\"\"\"\n"), Err(Fail::Error(_))));
        assert!(matches!(parse("a = 1\nb\n"), Err(Fail::Error(_))));
        assert!(matches!(parse("a = 1\n[[a]]\n"), Err(Fail::Error(_))));
        assert!(matches!(parse("a = \"x\"\n[a]\nb = 1\n"), Err(Fail::Crash(_))));
        assert!(parse("a = \"x\"\n[a]\n").is_ok());
        assert!(matches!(parse("a = []\n[a]\n"), Err(Fail::Crash(_))));
        assert!(matches!(parse("a = \"\\uZZZZ\"\n"), Err(Fail::Crash(_))));
        assert!(matches!(parse("a = \"x\" y\n"), Err(Fail::Error(_))));
        assert!(matches!(parse("a. = 1\n"), Err(Fail::Error(_))));
    }
}
