//! The line-based TOML readers of 004-14 and 004-15, ported faithfully
//! ([`crate::doccheck::policy::TomlReader::Line004_14`] and
//! [`crate::doccheck::policy::TomlReader::Line004_15`]).
//!
//! Both scripts read `wire.toml` one line at a time (`strip_toml_comment`,
//! `parse_value`, `parse_toml`) into `(messages, others)`: the
//! `[[message]]` tables (each starts as `{"name": None, "fields": []}`,
//! its `[[message.field]]` tables appended to `"fields"`), and every other
//! table by name (a dict for `[name]`, a list of dicts for `[[name]]`). It
//! is not TOML: a scalar is `true`/`false`, else `int(t, 0)`, else
//! `int(t, 10)` (so `040` is 40), else the text as written (`1.5`, `C-116`,
//! `'x'`); an array is every string or integer token the regex finds in the
//! rest of the line (`[1.5]` is `[1, 5]`); a line without `=` is ignored.
//!
//! The two copies differ, so each is its own variant ([`Variant`]):
//!
//! | | 004-14 | 004-15 |
//! |---|---|---|
//! | a string value | `\"` and `\\` unescaped | kept as written |
//! | `[[message.field]]` before any `[[message]]` | `ValueError` | `IndexError` |
//! | `[[x]]` after `[x]` | `ValueError` | `AttributeError` (uncaught: the script crashes) |
//!
//! Each script catches some exception types (004-14: `ValueError`,
//! `IndexError`; 004-15: also `KeyError`) and prints `FAIL setup`; any other
//! is a traceback. [`Error::kind`] names the type, so a port tells the two
//! apart ([`Error::caught_by`]).
//!
//! Python's `int()` refuses a decimal string longer than 4 300 digits
//! (`sys.int_info.default_max_str_digits`, Python 3.10.7 and later): a
//! scalar that long reads as the text as written, an array token that long
//! is a `ValueError`. Both are reproduced.
//!
//! The proof is the corpus `hadronyche-tools/data/toml_corpus.txt`,
//! generated from each script's own reader by
//! `hadronyche-tools/tools/gen_toml_corpus.py` with Python 3.10 and 3.13.

use super::pycompat::{PyStr, splitlines};
use super::text::rx;
use super::tomlsubset::{MAX_STR_DIGITS, py_int_base};

/// Which script's copy of the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// `checks/004-14.sh`.
    V004_14,
    /// `checks/004-15.sh`.
    V004_15,
}

/// A Python value the reader produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Python's `None`: a message's default `"name"`, or a string value
    /// whose closing quote is missing.
    None,
    Bool(bool),
    Int(i128),
    /// An int beyond `i128`, as its decimal digits.
    BigInt(String),
    Str(String),
    List(Vec<Value>),
    Dict(Table),
}

/// A Python dict in insertion order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table(pub Vec<(String, Value)>);

impl Table {
    /// `d.get(key)`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// `d[key] = value` (an existing key keeps its place).
    pub fn insert(&mut self, key: String, value: Value) {
        match self.get_mut(&key) {
            Some(slot) => *slot = value,
            None => self.0.push((key, value)),
        }
    }

    /// The keys, in order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }
}

impl Value {
    /// Python's `type(v).__name__`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "NoneType",
            Value::Bool(_) => "bool",
            Value::Int(_) | Value::BigInt(_) => "int",
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
        }
    }

    /// The string, for a `Str`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The int, for an `Int` (a `bool` is not one here: callers that rely
    /// on `isinstance(True, int)` test `Bool` themselves).
    pub fn as_int(&self) -> Option<i128> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// The table, for a `Dict`.
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Dict(t) => Some(t),
            _ => None,
        }
    }

    /// The canonical form of the corpus (`gen_toml_corpus.py`'s `canon`,
    /// dict keys in insertion order).
    pub fn canonical(&self) -> String {
        match self {
            Value::None => "n".into(),
            Value::Bool(b) => if *b { "true" } else { "false" }.into(),
            Value::Int(n) => format!("i{n}"),
            Value::BigInt(s) => format!("i{s}"),
            Value::Str(s) => canon_str(s),
            Value::List(xs) => format!(
                "[{}]",
                xs.iter()
                    .map(Value::canonical)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Value::Dict(t) => format!(
                "{{{}}}",
                t.0.iter()
                    .map(|(k, v)| format!("{}:{}", canon_str(k), v.canonical()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

fn canon_str(s: &str) -> String {
    let mut out = String::from("s\"");
    for c in s.chars() {
        if (' '..='~').contains(&c) && c != '"' && c != '\\' {
            out.push(c);
        } else {
            out.push_str(&format!("\\u{{{:x}}}", u32::from(c)));
        }
    }
    out.push('"');
    out
}

/// What the reader returns: the `[[message]]` tables and the other tables.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Doc {
    pub messages: Vec<Table>,
    pub others: Table,
}

impl Doc {
    /// The canonical form of the corpus: `[messages, others]`.
    pub fn canonical(&self) -> String {
        Value::List(vec![
            Value::List(self.messages.iter().cloned().map(Value::Dict).collect()),
            Value::Dict(self.others.clone()),
        ])
        .canonical()
    }
}

/// The Python exception a reader raised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// `ValueError`, `IndexError` or `AttributeError`.
    pub kind: &'static str,
    pub message: String,
}

impl Error {
    fn new(kind: &'static str, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }

    /// True when the script's `except` around `parse_toml` catches it (it
    /// then prints `FAIL setup: wire.toml does not parse: <message>`);
    /// false for an uncaught exception (a traceback).
    pub fn caught_by(&self, variant: Variant) -> bool {
        match variant {
            Variant::V004_14 => matches!(self.kind, "ValueError" | "IndexError"),
            Variant::V004_15 => matches!(self.kind, "ValueError" | "IndexError" | "KeyError"),
        }
    }
}

/// `strip_toml_comment(line)`: everything before a `#` outside a
/// double-quoted string (a backslash in a string takes the next character).
pub fn strip_toml_comment(line: &str) -> String {
    let c: Vec<char> = line.chars().collect();
    let (mut out, mut in_str, mut i) = (String::new(), false, 0);
    while i < c.len() {
        let ch = c[i];
        if in_str {
            out.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                out.push(c[i + 1]);
                i += 2;
                continue;
            }
            if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
            out.push(ch);
        } else if ch == '#' {
            break;
        } else {
            out.push(ch);
        }
        i += 1;
    }
    out
}

fn int_value(digits: String) -> Value {
    match super::pyvalue::PyValue::from_big(digits) {
        super::pyvalue::PyValue::Int(n) => Value::Int(n),
        super::pyvalue::PyValue::BigInt(s) => Value::BigInt(s),
        other => Value::Str(other.py_str()),
    }
}

/// Python's `int(t, 0)`, or `None` for its `ValueError`.
fn py_int0(t: &str) -> Option<Value> {
    let s = t.py_strip();
    let body = s
        .strip_prefix('-')
        .or_else(|| s.strip_prefix('+'))
        .unwrap_or(s);
    let lower = body.to_ascii_lowercase();
    let base = match lower.get(..2) {
        Some("0x") => 16,
        Some("0o") => 8,
        Some("0b") => 2,
        _ => 10,
    };
    if base == 10 {
        // base 0: no leading zero unless the number is zero
        if body.len() > 1 && body.starts_with('0') && !body.chars().all(|c| c == '0') {
            return None;
        }
        if body.chars().count() > MAX_STR_DIGITS {
            return None;
        }
    }
    py_int_base(s, base).map(int_value)
}

/// Python's `int(t, 10)`, or `None` for its `ValueError`.
fn py_int10(t: &str) -> Option<Value> {
    let s = t.py_strip();
    let body = s
        .strip_prefix('-')
        .or_else(|| s.strip_prefix('+'))
        .unwrap_or(s);
    if body.chars().count() > MAX_STR_DIGITS {
        return None;
    }
    py_int_base(s, 10).map(int_value)
}

fn py_repr_str(s: &str) -> String {
    if s.contains('\'') && !s.contains('"') {
        format!("\"{s}\"")
    } else {
        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

/// `parse_value(s)` of `variant`.
pub fn parse_value(s: &str, variant: Variant) -> Result<Value, Error> {
    let s = s.py_strip();
    if s.starts_with('"') {
        let Some(c) = rx(r#"^"((?:[^"\\]|\\.)*)""#).captures(s) else {
            return Ok(Value::None);
        };
        let raw = c[1].to_string();
        return Ok(Value::Str(match variant {
            Variant::V004_14 => raw.replace("\\\"", "\"").replace("\\\\", "\\"),
            Variant::V004_15 => raw,
        }));
    }
    if s.starts_with('[') {
        let mut items = Vec::new();
        for c in rx(r#""((?:[^"\\]|\\.)*)"|(-?0x[0-9a-fA-F_]+|-?\d[\d_]*)"#).captures_iter(s) {
            if let Some(m) = c.get(1) {
                items.push(Value::Str(m.as_str().to_string()));
            } else if let Some(m) = c.get(2) {
                let t = m.as_str().replace('_', "");
                let v = py_int0(&t).ok_or_else(|| {
                    Error::new(
                        "ValueError",
                        format!("invalid literal for int() with base 0: {}", py_repr_str(&t)),
                    )
                })?;
                items.push(v);
            }
        }
        return Ok(Value::List(items));
    }
    if s == "true" || s == "false" {
        return Ok(Value::Bool(s == "true"));
    }
    let t = s.replace('_', "");
    Ok(py_int0(&t)
        .or_else(|| py_int10(&t))
        .unwrap_or_else(|| Value::Str(s.to_string())))
}

/// Where the next `key = value` lines go.
enum Target {
    None,
    Message(usize),
    Field(usize, usize),
    Other(String),
    OtherArray(String, usize),
}

fn not_found() -> Error {
    Error::new("ValueError", "substring not found")
}

/// The characters of `line` from `from` to the first `pat`.
fn header_name(line: &str, from: usize, pat: &str) -> Result<String, Error> {
    let at = line.find(pat).ok_or_else(not_found)?;
    let start = line.char_indices().nth(from).map_or(line.len(), |(i, _)| i);
    Ok(line.get(start..at).unwrap_or("").py_strip().to_string())
}

fn table_of<'a>(doc: &'a mut Doc, target: &Target) -> Option<&'a mut Table> {
    match target {
        Target::None => None,
        Target::Message(i) => doc.messages.get_mut(*i),
        Target::Field(i, j) => match doc.messages.get_mut(*i)?.get_mut("fields")? {
            Value::List(xs) => match xs.get_mut(*j)? {
                Value::Dict(t) => Some(t),
                _ => None,
            },
            _ => None,
        },
        Target::Other(name) => match doc.others.get_mut(name)? {
            Value::Dict(t) => Some(t),
            _ => None,
        },
        Target::OtherArray(name, k) => match doc.others.get_mut(name)? {
            Value::List(xs) => match xs.get_mut(*k)? {
                Value::Dict(t) => Some(t),
                _ => None,
            },
            _ => None,
        },
    }
}

fn no_append(v: &Value) -> Error {
    Error::new(
        "AttributeError",
        format!("'{}' object has no attribute 'append'", v.type_name()),
    )
}

fn field_header(doc: &mut Doc, variant: Variant) -> Result<Target, Error> {
    let Some(i) = doc.messages.len().checked_sub(1) else {
        return Err(match variant {
            Variant::V004_14 => {
                Error::new("ValueError", "a [[message.field]] precedes any [[message]]")
            }
            Variant::V004_15 => Error::new("IndexError", "list index out of range"),
        });
    };
    match doc.messages[i].get_mut("fields") {
        Some(Value::List(xs)) => {
            xs.push(Value::Dict(Table::default()));
            Ok(Target::Field(i, xs.len() - 1))
        }
        Some(other) => Err(no_append(other)),
        None => Err(Error::new("KeyError", "'fields'")),
    }
}

fn array_header(doc: &mut Doc, name: String, variant: Variant) -> Result<Target, Error> {
    if doc.others.get(&name).is_none() {
        doc.others.insert(name.clone(), Value::List(Vec::new()));
    }
    match doc.others.get_mut(&name) {
        Some(Value::List(xs)) => {
            xs.push(Value::Dict(Table::default()));
            let k = xs.len() - 1;
            Ok(Target::OtherArray(name, k))
        }
        Some(other) => Err(match variant {
            Variant::V004_14 => Error::new(
                "ValueError",
                format!("table {name} is both an array and a table"),
            ),
            Variant::V004_15 => no_append(other),
        }),
        None => Err(Error::new("KeyError", name)),
    }
}

/// `parse_toml(src)` of `variant`: `(messages, others)`, or the exception
/// the script's reader raised.
pub fn parse(src: &str, variant: Variant) -> Result<Doc, Error> {
    let mut doc = Doc::default();
    let mut target = Target::None;
    for raw in splitlines(src) {
        let stripped = strip_toml_comment(raw);
        let line = stripped.py_strip();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("[[") {
            let name = header_name(line, 2, "]]")?;
            target = match name.as_str() {
                "message" => {
                    let mut t = Table::default();
                    t.insert("name".into(), Value::None);
                    t.insert("fields".into(), Value::List(Vec::new()));
                    doc.messages.push(t);
                    Target::Message(doc.messages.len() - 1)
                }
                "message.field" => field_header(&mut doc, variant)?,
                _ => array_header(&mut doc, name, variant)?,
            };
            continue;
        }
        if line.starts_with('[') {
            let name = header_name(line, 1, "]")?;
            doc.others
                .insert(name.clone(), Value::Dict(Table::default()));
            target = Target::Other(name);
            continue;
        }
        // `"=" in line and target is not None`: no value is read before
        // the first table header
        if let (Some((key, value)), false) = (line.split_once('='), matches!(target, Target::None))
        {
            let value = parse_value(value, variant)?;
            if let Some(t) = table_of(&mut doc, &target) {
                t.insert(key.py_strip().to_string(), value);
            }
        }
    }
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_variants_equal_their_python_readers_on_the_corpus() {
        let recs = crate::doccheck::tomllib::corpus::records();
        assert!(recs.len() >= 130);
        let mut bad = Vec::new();
        for r in &recs {
            for (key, variant) in [("line14", Variant::V004_14), ("line15", Variant::V004_15)] {
                let want = r.want(key);
                let got = match parse(&r.src, variant) {
                    Ok(d) => d.canonical(),
                    Err(e) => format!("ERR {}", e.kind),
                };
                if got != want {
                    bad.push(format!("{} ({key}): Python {want}, port {got}", r.name));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "the line readers differ:\n{}",
            bad.join("\n")
        );
    }

    #[test]
    fn the_scripts_quirks() {
        let v = |s| parse_value(s, Variant::V004_14).unwrap().canonical();
        assert_eq!(v("040"), "i40");
        assert_eq!(v("0x_1f"), "i31");
        assert_eq!(v("1.5"), "s\"1.5\"");
        assert_eq!(v("[1.5, \"a\"]"), "[i1,i5,s\"a\"]");
        assert_eq!(v("\"a\\\"b\""), "s\"a\\u{22}b\"");
        assert_eq!(
            parse_value("\"a\\\"b\"", Variant::V004_15)
                .unwrap()
                .canonical(),
            "s\"a\\u{5c}\\u{22}b\""
        );
        assert_eq!(v("\"open"), "n");
        let e = parse_value("[040]", Variant::V004_14).unwrap_err();
        assert_eq!(e.kind, "ValueError");
        assert_eq!(e.message, "invalid literal for int() with base 0: '040'");
    }

    #[test]
    fn messages_fields_and_others() {
        let src = "[[message]]\nname = \"a\"\n[[message.field]]\nwidth = 8 # c\n[meta]\nx = 1\n[[reject]]\nname = \"B\"\n";
        let d = parse(src, Variant::V004_14).unwrap();
        assert_eq!(d.messages.len(), 1);
        assert_eq!(
            d.canonical(),
            "[[{s\"name\":s\"a\",s\"fields\":[{s\"width\":i8}]}],{s\"meta\":{s\"x\":i1},s\"reject\":[{s\"name\":s\"B\"}]}]"
        );
        let early = parse("[[message.field]]\n", Variant::V004_14).unwrap_err();
        assert!(early.caught_by(Variant::V004_14));
        let early = parse("[[message.field]]\n", Variant::V004_15).unwrap_err();
        assert_eq!(early.kind, "IndexError");
        let both = parse("[a]\n[[a]]\n", Variant::V004_15).unwrap_err();
        assert_eq!(both.kind, "AttributeError");
        assert!(!both.caught_by(Variant::V004_15), "004-15 crashes");
        assert_eq!(
            parse("[a]\n[[a]]\n", Variant::V004_14).unwrap_err().kind,
            "ValueError"
        );
    }
}
