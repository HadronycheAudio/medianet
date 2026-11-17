//! Python values, as a script's TOML reader built them.
//!
//! The scripts read `wire.toml` into Python dicts, lists, strings, ints,
//! floats and bools (and, through `tomllib`, dates). The ports read the file
//! with a faithful port of their script's reader into [`PyValue`], and query
//! it with Python's semantics: `int()` ([`PyValue::py_int`]), `isinstance(x,
//! int)`, `==` across int, float and bool, iteration (a string iterates its
//! characters), `str()`.
//!
//! [`canonical`] is the form the TOML corpus records for each value, written
//! the same way by `hadronyche-tools/tools/gen_toml_corpus.py` from the
//! Python readers.

use super::pycompat::{IntError, parse_int};

/// A Python value.
#[derive(Debug, Clone, PartialEq)]
pub enum PyValue {
    Str(String),
    /// An int that fits `i128`.
    Int(i128),
    /// A bigger int, in decimal (Python has no limit; a check reports it).
    BigInt(String),
    Float(f64),
    Bool(bool),
    /// A `tomllib` date, time or datetime (the checks only test its type).
    Date(String),
    List(Vec<PyValue>),
    Dict(Dict),
}

/// A Python dict with string keys, in insertion order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dict(pub Vec<(String, PyValue)>);

impl Dict {
    pub fn get(&self, k: &str) -> Option<&PyValue> {
        self.0.iter().find(|(key, _)| key == k).map(|(_, v)| v)
    }
    pub fn get_mut(&mut self, k: &str) -> Option<&mut PyValue> {
        self.0.iter_mut().find(|(key, _)| key == k).map(|(_, v)| v)
    }
    pub fn contains_key(&self, k: &str) -> bool {
        self.get(k).is_some()
    }
    /// `d[k] = v`: an existing key keeps its position.
    pub fn insert(&mut self, k: &str, v: PyValue) {
        match self.get_mut(k) {
            Some(slot) => *slot = v,
            None => self.0.push((k.to_string(), v)),
        }
    }
    /// `d.setdefault(k, default)`.
    pub fn setdefault(&mut self, k: &str, default: PyValue) -> &mut PyValue {
        if !self.contains_key(k) {
            self.0.push((k.to_string(), default));
        }
        self.get_mut(k).expect("just inserted")
    }
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl PyValue {
    /// `bool(v)`: empty strings, lists and dicts, zero and `False` are
    /// false; a date and `nan` are true. A big integer is never zero.
    pub fn truthy(&self) -> bool {
        match self {
            PyValue::Str(s) => !s.is_empty(),
            PyValue::Int(n) => *n != 0,
            PyValue::BigInt(_) | PyValue::Date(_) => true,
            PyValue::Float(f) => *f != 0.0,
            PyValue::Bool(b) => *b,
            PyValue::List(l) => !l.is_empty(),
            PyValue::Dict(d) => !d.is_empty(),
        }
    }

    /// The Python type name (`str`, `int`, ...), for messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            PyValue::Str(_) => "str",
            PyValue::Int(_) | PyValue::BigInt(_) => "int",
            PyValue::Float(_) => "float",
            PyValue::Bool(_) => "bool",
            PyValue::Date(_) => "datetime",
            PyValue::List(_) => "list",
            PyValue::Dict(_) => "dict",
        }
    }

    pub fn as_dict(&self) -> Option<&Dict> {
        match self {
            PyValue::Dict(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PyValue::Str(s) => Some(s),
            _ => None,
        }
    }

    /// `int(v)`: an int as is (`Err` beyond i128), a bool as 0 or 1, a float
    /// truncated, a string parsed as Python does; anything else is the
    /// `TypeError` the script would crash with.
    pub fn py_int(&self) -> Result<i128, IntError> {
        match self {
            PyValue::Int(i) => Ok(*i),
            PyValue::BigInt(s) => {
                // beyond i128: red by rule (INT_LIMIT), never compared
                super::pynum::record_int_limit(s);
                Err(IntError::Overflow(s.clone()))
            }
            PyValue::Bool(b) => Ok(i128::from(*b)),
            PyValue::Float(f) if f.is_finite() && f.abs() < 1e38 => Ok(f.trunc() as i128),
            PyValue::Float(f) => Err(IntError::NotInt(f.to_string())),
            PyValue::Str(s) => parse_int(s),
            other => Err(IntError::NotInt(format!("a {}", other.type_name()))),
        }
    }

    /// `isinstance(v, int)` (a bool is an int).
    pub fn is_int(&self) -> bool {
        matches!(
            self,
            PyValue::Int(_) | PyValue::BigInt(_) | PyValue::Bool(_)
        )
    }

    fn num(&self) -> Option<f64> {
        match self {
            PyValue::Int(i) => Some(*i as f64),
            PyValue::Bool(b) => Some(f64::from(u8::from(*b))),
            PyValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// `v == n` for an int `n`.
    pub fn eq_int(&self, n: i128) -> bool {
        match self {
            PyValue::Int(i) => *i == n,
            PyValue::Bool(b) => i128::from(*b) == n,
            PyValue::Float(f) => *f == n as f64,
            _ => false,
        }
    }

    /// `v == "s"`.
    pub fn eq_str(&self, s: &str) -> bool {
        matches!(self, PyValue::Str(x) if x == s)
    }

    /// `a == b`.
    pub fn py_eq(&self, other: &PyValue) -> bool {
        match (self, other) {
            (PyValue::Str(a), PyValue::Str(b)) => a == b,
            (PyValue::BigInt(a), PyValue::BigInt(b)) => a == b,
            (PyValue::Date(a), PyValue::Date(b)) => a == b,
            (PyValue::List(a), PyValue::List(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.py_eq(y))
            }
            (PyValue::Dict(a), PyValue::Dict(b)) => {
                a.len() == b.len()
                    && a.0
                        .iter()
                        .all(|(k, v)| b.get(k).is_some_and(|w| v.py_eq(w)))
            }
            (a, b) => matches!((a.num(), b.num()), (Some(x), Some(y)) if x == y),
        }
    }

    /// `[x for x in v]` as `str()`s: a list's items, a string's characters,
    /// a dict's keys; `Err` (the script's `TypeError`) otherwise.
    pub fn py_iter_str(&self) -> Result<Vec<String>, String> {
        match self {
            PyValue::List(a) => Ok(a.iter().map(PyValue::py_str).collect()),
            PyValue::Str(s) => Ok(s.chars().map(String::from).collect()),
            PyValue::Dict(d) => Ok(d.keys().map(String::from).collect()),
            other => Err(format!("'{}' object is not iterable", other.type_name())),
        }
    }

    /// `[x for x in v if isinstance(x, dict)]`; `Err` when `v` is not
    /// iterable.
    pub fn py_dicts(&self) -> Result<Vec<&Dict>, String> {
        match self {
            PyValue::List(a) => Ok(a.iter().filter_map(PyValue::as_dict).collect()),
            PyValue::Str(_) | PyValue::Dict(_) => Ok(Vec::new()),
            other => Err(format!("'{}' object is not iterable", other.type_name())),
        }
    }

    /// `str(v)`.
    pub fn py_str(&self) -> String {
        match self {
            PyValue::Str(s) => s.clone(),
            other => other.py_repr(),
        }
    }

    /// `repr(v)`.
    pub fn py_repr(&self) -> String {
        match self {
            PyValue::Str(s) => repr_str(s),
            PyValue::Int(i) => i.to_string(),
            PyValue::BigInt(s) | PyValue::Date(s) => s.clone(),
            PyValue::Float(f) => repr_float(*f),
            PyValue::Bool(true) => "True".into(),
            PyValue::Bool(false) => "False".into(),
            PyValue::List(a) => format!(
                "[{}]",
                a.iter()
                    .map(PyValue::py_repr)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PyValue::Dict(d) => format!(
                "{{{}}}",
                d.0.iter()
                    .map(|(k, v)| format!("{}: {}", repr_str(k), v.py_repr()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// Python's `repr()` of a str, for the printable characters the checks meet
/// (quotes chosen as Python does; backslash, quote and controls escaped).
fn repr_str(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(q);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == q => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

/// Python's `repr()` of a float: the shortest round-trip digits, in
/// positional form for exponents -4 to 15, else `1e+16` style.
pub fn repr_float(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let sci = format!("{f:e}");
    let (mant, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or_default(); // Rust's own `{:e}` output
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(char::is_ascii_digit).collect();
    let body = if (-4..16).contains(&exp) {
        positional(&digits, exp)
    } else {
        let m = if digits.len() > 1 {
            format!("{}.{}", &digits[..1], &digits[1..])
        } else {
            digits.clone()
        };
        format!("{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs())
    };
    if neg { format!("-{body}") } else { body }
}

fn positional(digits: &str, exp: i32) -> String {
    let n = digits.len() as i32;
    if exp < 0 {
        format!("0.{}{}", "0".repeat((-exp - 1) as usize), digits)
    } else if exp + 1 >= n {
        format!("{}{}.0", digits, "0".repeat((exp + 1 - n) as usize))
    } else {
        let k = (exp + 1) as usize;
        format!("{}.{}", &digits[..k], &digits[k..])
    }
}

/// The corpus form of a value: `{k:v,...}` in insertion order (or sorted
/// keys with `sorted`), `[...]`, `s"..."` (non-printable-ASCII as `\u{hex}`),
/// `i<decimal>`, `f<bits in hex>`, `true`/`false`, `d` for a date.
pub fn canonical(v: &PyValue, sorted: bool) -> String {
    match v {
        PyValue::Str(s) => {
            let mut out = String::from("s\"");
            for c in s.chars() {
                if (' '..='~').contains(&c) && c != '"' && c != '\\' {
                    out.push(c);
                } else {
                    out.push_str(&format!("\\u{{{:x}}}", c as u32));
                }
            }
            out.push('"');
            out
        }
        PyValue::Int(i) => format!("i{i}"),
        PyValue::BigInt(s) => format!("i{s}"),
        PyValue::Float(f) => format!("f{:016x}", f.to_bits()),
        PyValue::Bool(b) => b.to_string(),
        PyValue::Date(_) => "d".into(),
        PyValue::List(a) => format!(
            "[{}]",
            a.iter()
                .map(|x| canonical(x, sorted))
                .collect::<Vec<_>>()
                .join(",")
        ),
        PyValue::Dict(d) => {
            let mut items: Vec<(String, String)> =
                d.0.iter()
                    .map(|(k, v)| (k.clone(), canonical(v, sorted)))
                    .collect();
            if sorted {
                items.sort();
            }
            let body: Vec<String> = items
                .into_iter()
                .map(|(k, v)| format!("{}:{v}", canonical(&PyValue::Str(k), sorted)))
                .collect();
            format!("{{{}}}", body.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_semantics() {
        assert!(PyValue::Float(40.0).eq_int(40) && PyValue::Bool(true).eq_int(1));
        assert!(!PyValue::Str("40".into()).eq_int(40));
        assert_eq!(
            PyValue::Str("Ab".into()).py_iter_str().unwrap(),
            vec!["A", "b"]
        );
        assert!(PyValue::Int(5).py_iter_str().is_err());
        assert!(PyValue::Bool(false).is_int() && !PyValue::Float(1.0).is_int());
        assert_eq!(PyValue::Str(" 12 ".into()).py_int(), Ok(12));
        assert!(PyValue::BigInt("9".repeat(50)).py_int().is_err());
        let mut d = Dict::default();
        d.insert("a", PyValue::Int(1));
        d.insert("b", PyValue::Int(2));
        d.insert("a", PyValue::Int(3));
        assert_eq!(d.keys().collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(PyValue::Dict(d).py_repr(), "{'a': 3, 'b': 2}");
        assert_eq!(
            PyValue::List(vec![PyValue::Str("x".into()), PyValue::Int(1)]).py_str(),
            "['x', 1]"
        );
    }

    #[test]
    fn floats_print_as_python() {
        for (f, s) in [
            (40.0, "40.0"),
            (1e16, "1e+16"),
            (1e15, "1000000000000000.0"),
            (0.0001, "0.0001"),
            (1e-5, "1e-05"),
            (65507.0, "65507.0"),
            (-2.5, "-2.5"),
            (0.1, "0.1"),
            (1.5e300, "1.5e+300"),
        ] {
            assert_eq!(repr_float(f), s);
        }
    }
}
