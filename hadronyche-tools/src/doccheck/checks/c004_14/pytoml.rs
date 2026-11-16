//! The script's own TOML reader for the subset `wire.toml` uses: a line
//! reader that strips comments, takes `[[name]]` and `[name]` headers and
//! `key = value` lines, and reads values with Python's `int(text, 0)`.
//!
//! The port reproduces it instead of using the crate's TOML parser: the
//! reader tolerates files a real parser rejects (multi-line arrays are read
//! line by line) and its verdict is what has been passing.

use crate::doccheck::text::rx;

/// A value as the script's `parse_value` returns it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum V {
    /// A quoted string (the script's `None` when it does not close).
    Str(Option<String>),
    /// A bracketed value: its quoted and integer items.
    List(Vec<V>),
    Bool(bool),
    Int(i64),
    /// Anything else, as written.
    Raw(String),
}

impl V {
    /// The value as Python arithmetic sees it: integers and booleans.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            V::Int(n) => Some(*n),
            V::Bool(b) => Some(i64::from(*b)),
            _ => None,
        }
    }
}

/// A TOML table: keys in the order written, the last assignment winning.
#[derive(Debug, Clone, Default)]
pub(crate) struct Dict(pub Vec<(String, V)>);

impl Dict {
    pub fn get(&self, key: &str) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn set(&mut self, key: &str, value: V) {
        match self.0.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = value,
            None => self.0.push((key.to_string(), value)),
        }
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(V::Str(Some(s))) => Some(s),
            _ => None,
        }
    }
}

/// A `[[message]]`: its keys and its `[[message.field]]` tables.
#[derive(Debug, Clone, Default)]
pub(crate) struct Message {
    pub keys: Dict,
    pub fields: Vec<Dict>,
}

/// A table other than a message: one `[x]` or the entries of `[[x]]`.
#[derive(Debug, Clone)]
pub(crate) enum Other {
    One(Dict),
    Many(Vec<Dict>),
}

/// The two readers of the family differ in two places: 004-14 unescapes the
/// strings it reads and rejects `[[x]]` after `[x]` with a `ValueError`
/// (a parse failure); 004-15 keeps strings as written and raises an
/// uncaught `AttributeError` there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flavor {
    Budget,
    Capacity,
}

/// Why the reader stops: a caught `ValueError`, or an uncaught exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bad {
    Value,
    Crash,
}

/// Where `key = value` lines land.
enum Target {
    None,
    Message(usize),
    Field(usize),
    One(usize),
    Entry(usize),
}

/// What the reader returns.
#[derive(Debug, Clone, Default)]
pub(crate) struct Parsed {
    pub messages: Vec<Message>,
    pub others: Vec<(String, Other)>,
}

/// Python `int(text, 0)` then `int(text, 10)`; `None` when both raise.
pub(crate) fn py_int(text: &str, base0: bool) -> Option<i64> {
    let t = text.trim();
    let (neg, body) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let lower = body.to_ascii_lowercase();
    let (radix, digits) = if base0 && lower.len() > 1 && lower.starts_with('0') {
        match lower.as_bytes()[1] {
            b'x' => (16, &lower[2..]),
            b'o' => (8, &lower[2..]),
            b'b' => (2, &lower[2..]),
            _ if lower.bytes().all(|c| c == b'0') => (10, lower.as_str()),
            _ => return None,
        }
    } else {
        (10, lower.as_str())
    };
    if digits.is_empty() || !digits.bytes().all(|c| char::from(c).is_digit(radix)) {
        return None;
    }
    let n = i64::from_str_radix(digits, radix).ok()?;
    Some(if neg { -n } else { n })
}

fn strip_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_str = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if c == '\\' {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
                continue;
            }
            if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(c);
        } else if c == '#' {
            break;
        } else {
            out.push(c);
        }
    }
    out
}

/// `parse_value`: `Err` is the script's `ValueError` (an item that `int`
/// rejects).
fn parse_value(s: &str, flavor: Flavor) -> Result<V, Bad> {
    let s = s.trim();
    if s.starts_with('"') {
        let m = rx(r#"^"((?:[^"\\]|\\.)*)""#).captures(s);
        return Ok(V::Str(m.map(|c| match flavor {
            Flavor::Budget => c[1].replace("\\\"", "\"").replace("\\\\", "\\"),
            Flavor::Capacity => c[1].to_string(),
        })));
    }
    if s.starts_with('[') {
        let mut items = Vec::new();
        for m in rx(r#""((?:[^"\\]|\\.)*)"|(-?0x[0-9a-fA-F_]+|-?\d[\d_]*)"#).captures_iter(s) {
            match m.get(1) {
                Some(g) => items.push(V::Str(Some(g.as_str().to_string()))),
                None => items.push(V::Int(py_int(&m[2].replace('_', ""), true).ok_or(Bad::Value)?)),
            }
        }
        return Ok(V::List(items));
    }
    if s == "true" || s == "false" {
        return Ok(V::Bool(s == "true"));
    }
    let t = s.replace('_', "");
    Ok(py_int(&t, true)
        .or_else(|| py_int(&t, false))
        .map_or_else(|| V::Raw(s.to_string()), V::Int))
}

fn header_name(line: &str, open: usize, close: &str) -> Result<String, Bad> {
    let end = line.find(close).ok_or(Bad::Value)?;
    Ok(line.get(open..end).ok_or(Bad::Value)?.trim().to_string())
}

/// `parse_toml`: `Err` is the script's `ValueError`/`IndexError` ("wire.toml
/// does not parse").
pub(crate) fn parse(src: &str, flavor: Flavor) -> Result<Parsed, Bad> {
    let mut p = Parsed::default();
    let mut target = Target::None;
    for raw in crate::doccheck::text::splitlines(src) {
        let line = strip_comment(raw);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("[[") {
            let name = header_name(line, 2, "]]")?;
            target = open_array(&mut p, &name, flavor)?;
            continue;
        }
        if line.starts_with('[') {
            let name = header_name(line, 1, "]")?;
            let at = match p.others.iter().position(|(n, _)| *n == name) {
                Some(i) => {
                    p.others[i].1 = Other::One(Dict::default());
                    i
                }
                None => {
                    p.others.push((name, Other::One(Dict::default())));
                    p.others.len() - 1
                }
            };
            target = Target::One(at);
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = parse_value(value, flavor)?;
            if let Some(d) = dict_of(&mut p, &target) {
                d.set(key.trim(), value);
            }
        }
    }
    Ok(p)
}

fn open_array(p: &mut Parsed, name: &str, flavor: Flavor) -> Result<Target, Bad> {
    if name == "message" {
        p.messages.push(Message::default());
        return Ok(Target::Message(p.messages.len() - 1));
    }
    if name == "message.field" {
        let m = p.messages.last_mut().ok_or(Bad::Value)?;
        m.fields.push(Dict::default());
        return Ok(Target::Field(m.fields.len() - 1));
    }
    match p.others.iter().position(|(n, _)| n == name) {
        Some(i) => match &mut p.others[i].1 {
            Other::Many(v) => {
                v.push(Dict::default());
                Ok(Target::Entry(i))
            }
            Other::One(_) if flavor == Flavor::Budget => Err(Bad::Value),
            Other::One(_) => Err(Bad::Crash),
        },
        None => {
            p.others.push((name.to_string(), Other::Many(vec![Dict::default()])));
            Ok(Target::Entry(p.others.len() - 1))
        }
    }
}

fn dict_of<'a>(p: &'a mut Parsed, target: &Target) -> Option<&'a mut Dict> {
    match target {
        Target::None => None,
        Target::Message(i) => Some(&mut p.messages[*i].keys),
        Target::Field(i) => p.messages.last_mut().map(|m| &mut m.fields[*i]),
        Target::One(i) => match &mut p.others[*i].1 {
            Other::One(d) => Some(d),
            Other::Many(_) => None,
        },
        Target::Entry(i) => match &mut p.others[*i].1 {
            Other::Many(v) => v.last_mut(),
            Other::One(_) => None,
        },
    }
}
