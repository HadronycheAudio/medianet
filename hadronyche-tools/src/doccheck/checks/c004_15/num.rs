//! The script's number readers, written as the script writes them
//! (`INT_RE`, `FLOAT_RE`, `PCT_RE`, `MBIT_RE` with their look-arounds, run
//! by `text::rx`), `tol_of` and `close`.

use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::pynum::{PyNum, py_float};
use crate::doccheck::report::Report;
use crate::doccheck::text::rx;

const INT_RE: &str = r"(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)";
const FLOAT_RE: &str = r"(?<![\w.\-])(\d+)\.(\d+)(?!\w)";
const PCT_RE: &str = r"(?<![\w.])(\d+)(?:\.(\d+))?\s*%";
const MBIT_RE: &str = r"(?<![\w.])(\d+)(?:\.(\d+))?\s*Mbit/s";

/// `nrm`: the no-break, thin and narrow spaces as spaces, whitespace
/// squashed, stripped.
pub(crate) fn nrm(s: &str) -> String {
    let t = rx(r"[   ]").replace_all(s, " ");
    rx(r"\s+").replace_all(&t, " ").py_strip().to_string()
}

/// `to_int(s) = int(s.replace(" ", ""))`; a number beyond `i128` is
/// recorded (`INT_LIMIT`) and reads `None`.
pub(crate) fn to_int(s: &str) -> Option<i128> {
    parse_int(&s.replace(' ', "")).ok()
}

/// A decimal with the number of its digits after the point.
pub(crate) type Dec = (f64, usize);

fn dec_of(int: &str, frac: Option<&str>) -> Option<Dec> {
    let text = match frac {
        Some(f) => format!("{int}.{f}"),
        None => int.to_string(),
    };
    Some((py_float(&text)?, frac.map_or(0, |f| f.chars().count())))
}

/// `ints(s)`: `INT_RE` over `nrm(s)`.
pub(crate) fn ints(s: &str) -> Vec<i128> {
    rx(INT_RE)
        .captures_iter(&nrm(s))
        .filter_map(|m| to_int(&m[1]))
        .collect()
}

/// `floats(s)`: `FLOAT_RE` over `nrm(s)`.
pub(crate) fn floats(s: &str) -> Vec<Dec> {
    rx(FLOAT_RE)
        .captures_iter(&nrm(s))
        .filter_map(|m| dec_of(&m[1], m.get(2).map(|x| x.as_str())))
        .collect()
}

/// `pcts(s)`: `PCT_RE` over `nrm(s)`.
pub(crate) fn pcts(s: &str) -> Vec<Dec> {
    rx(PCT_RE)
        .captures_iter(&nrm(s))
        .filter_map(|m| dec_of(&m[1], m.get(2).map(|x| x.as_str())))
        .collect()
}

/// `MBIT_RE` over `text` as given (`mbit_list` / `mbits`).
pub(crate) fn mbit_list(text: &str) -> Vec<Dec> {
    rx(MBIT_RE)
        .captures_iter(text)
        .filter_map(|m| dec_of(&m[1], m.get(2).map(|x| x.as_str())))
        .collect()
}

/// `float(a + "." + b)` of two digit runs; `None` for a `ValueError`.
pub(crate) fn dec_value(int: &str, frac: Option<&str>) -> Option<Dec> {
    dec_of(int, frac)
}

/// `float(n)` of an int (nearest double, as Python).
pub(crate) fn fl(n: i128) -> f64 {
    PyNum::Int(n).as_f64()
}

/// `tol_of(decimals)`.
pub(crate) fn tol_of(decimals: usize) -> f64 {
    if decimals > 0 {
        0.5 * 10f64.powf(-fl(i128::try_from(decimals).unwrap_or_default()))
    } else {
        0.5
    }
}

/// `close(a, b, tol)`.
pub(crate) fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol + 1e-9
}

/// `close(a, b, tol)` over Python numbers: two ints are compared exactly
/// (Python subtracts ints exactly), anything else as floats.
pub(crate) fn close_n(a: PyNum, b: PyNum, tol: f64) -> bool {
    match (a, b) {
        (PyNum::Int(x), PyNum::Int(y)) => {
            i128::try_from(x.abs_diff(y)).is_ok_and(|d| fl(d) <= tol + 1e-9)
        }
        _ => (a.as_f64() - b.as_f64()).abs() <= tol + 1e-9,
    }
}

/// `re.search(r"(?<![\w.])<literal>(?![\d])", text)` for a literal number
/// text.
pub(crate) fn has_number(text: &str, literal: &str) -> bool {
    rx(&format!(
        r"(?<![\w.]){}(?![\d])",
        literal.replace('.', r"\.")
    ))
    .is_match(text)
}

/// `re.split(r"(?<=[.;])\s+", text)`.
pub(crate) fn sentences(text: &str) -> Vec<String> {
    rx(r"(?<=[.;])\s+").split(text).map(String::from).collect()
}

/// The failures the script collects (`fails`), as a report.
pub(crate) type Fails = Report;
