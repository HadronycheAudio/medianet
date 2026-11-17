//! AC6 (document part): each election message declared in wire.toml, read
//! with the scripts' subset reader (script lines 1066-1131).

use std::collections::BTreeSet;

use super::ac5::TIMER;
use super::{Ctx, Msg, int_add, int_mul};
use crate::doccheck::Report;
use crate::doccheck::family004::{md_name, NameNorm, norm_name, tname};
use crate::doccheck::pynum::PyNum;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::rejection_members::reasons::{LOOKBACK_004_16, named_reasons};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::has_i;
use crate::doccheck::tomlsubset;
use crate::doccheck::wire_toml::WIRE_TOML;

const PRIM: [&str; 10] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"];

/// `str(d.get(k))` (`None` when absent).
fn str_of(d: &Dict, k: &str) -> String {
    d.get(k).map_or_else(|| "None".to_string(), PyValue::py_str)
}

/// `str(d.get(k, ""))`.
fn text_of(d: &Dict, k: &str) -> String {
    d.get(k).map_or_else(String::new, PyValue::py_str)
}

/// `d.get(k) != n` for an int `n`.
fn ne_int(d: &Dict, k: &str, n: i128) -> bool {
    !d.get(k).is_some_and(|v| v.eq_int(n))
}

/// `d.get(k) != "s"`.
fn ne_str(d: &Dict, k: &str, s: &str) -> bool {
    !d.get(k).is_some_and(|v| v.eq_str(s))
}

/// `(d.get(k) or default)` as a number; `Ok(None)` for a big integer
/// (recorded: `INT_LIMIT`); a truthy non-number makes the script's `*` or
/// `+=` raise `TypeError`.
fn operand(d: &Dict, k: &str, default: i128, name: &str) -> Flow<Option<PyNum>> {
    match d.get(k).filter(|v| v.truthy()) {
        None => Ok(Some(PyNum::Int(default))),
        Some(PyValue::Float(f)) => Ok(Some(PyNum::Float(*f))),
        Some(v) if v.is_int() => Ok(v.py_int().ok().map(PyNum::Int)),
        Some(v) => crash(format!("TypeError: toml field {name} adds a {} to the size", v.type_name())),
    }
}

/// Python's `a op b` on two numbers (an int result beyond `i128` is
/// `INT_LIMIT`).
fn op(a: PyNum, b: PyNum, int: fn(i128, i128) -> i128, float: fn(f64, f64) -> f64) -> PyNum {
    match (a, b) {
        (PyNum::Int(x), PyNum::Int(y)) => PyNum::Int(int(x, y)),
        (x, y) => PyNum::Float(float(x.as_f64(), y.as_f64())),
    }
}

/// `n == m` exactly, as Python compares an int with a float.
fn num_eq_int(n: PyNum, m: i128) -> bool {
    match n {
        PyNum::Int(i) => i == m,
        #[allow(clippy::cast_possible_truncation)]
        PyNum::Float(f) => f.is_finite() && f.fract() == 0.0 && f.abs() < 1.0e38 && f as i128 == m,
    }
}

/// The field items (`[x for x in (T.get("field") or []) if isinstance(x,
/// dict)]`): a non-iterable is the script's `TypeError`.
fn field_dicts(t: &Dict) -> Flow<Vec<&Dict>> {
    match t.get("field").filter(|v| v.truthy()) {
        None => Ok(Vec::new()),
        Some(v) => v.py_dicts().or_else(|e| crash(format!("TypeError: {e}"))),
    }
}

fn fields(m: &Msg, t: &Dict, report: &mut Report) -> Flow<Option<PyNum>> {
    let name = str_of(t, "name");
    let tf = field_dicts(t)?;
    if tf.len() != m.fields.len() {
        report.fail("AC6", format!("toml message '{name}' has {} fields; the markdown table has {} rows", tf.len(), m.fields.len()));
    }
    let mut total = Some(PyNum::Int(0));
    for (mf, tfl) in m.fields.iter().zip(&tf) {
        let nm = md_name(&mf.meaning);
        let tn = str_of(tfl, "name");
        if norm_name(nm, NameNorm::Spaced004_16) != norm_name(&text_of(tfl, "name"), NameNorm::Spaced004_16) {
            report.fail("AC6", format!("toml field '{tn}' is not named after the markdown row '{nm}' (names are compared lower case, spaces and underscores equivalent)"));
        }
        if ne_int(tfl, "offset", int_mul(mf.off, 8)) {
            report.fail("AC6", format!("toml field {tn}: offset {} bits, markdown byte {}", str_of(tfl, "offset"), mf.off));
        }
        if ne_int(tfl, "width", mf.bits) {
            report.fail("AC6", format!("toml field {tn}: width {} bits; markdown {} bits", str_of(tfl, "width"), mf.bits));
        }
        if let Some(maxc) = mf.maxc
            && ne_int(tfl, "max_count", maxc)
        {
            report.fail("AC6", format!("toml field {tn}: max_count {}; markdown says max {maxc}", str_of(tfl, "max_count")));
        }
        if PRIM.contains(&mf.ty.as_str()) && ne_str(tfl, "type", &mf.ty) {
            report.fail("AC6", format!("toml field {tn}: type {}; markdown {}", str_of(tfl, "type"), mf.ty));
        }
        if has_i(TIMER, &text_of(tfl, "name")) {
            report.fail("Behaviour", format!("toml field '{tn}' is named like a timer"));
        }
        let w = operand(tfl, "width", 0, &tn)?;
        let c = operand(tfl, "max_count", 1, &tn)?;
        total = match (total, w, c) {
            (Some(t), Some(w), Some(c)) => Some(op(t, op(w, c, int_mul, |a, b| a * b), int_add, |a, b| a + b)),
            _ => None,
        };
    }
    Ok(total)
}

/// `set(T.get("rejection_reasons") or [])` and the two sorted differences.
fn reasons(ctx: &Ctx, m: &Msg, t: &Dict, report: &mut Report) -> Flow<()> {
    let name = str_of(t, "name");
    let need: BTreeSet<String> = ctx
        .shared
        .iter()
        .chain(&m.forms)
        .flat_map(|(_, o)| named_reasons(o, |w| ctx.members.contains(w), LOOKBACK_004_16))
        .collect();
    let have: Vec<PyValue> = match t.get("rejection_reasons").filter(|v| v.truthy()) {
        None => Vec::new(),
        Some(PyValue::List(l)) => {
            if let Some(bad) = l.iter().find(|v| matches!(v, PyValue::List(_) | PyValue::Dict(_))) {
                return crash(format!("TypeError: unhashable type: '{}'", bad.type_name()));
            }
            l.clone()
        }
        Some(v) => match v.py_iter_str() {
            Ok(s) => s.into_iter().map(PyValue::Str).collect(),
            Err(e) => return crash(format!("TypeError: {e}")),
        },
    };
    for r in &need {
        if !have.iter().any(|h| h.eq_str(r)) {
            report.fail("AC6", format!("toml message '{name}' rejection_reasons does not list '{r}', which its invalid-form rows name"));
        }
    }
    let extra: Vec<&PyValue> = have.iter().filter(|h| !h.as_str().is_some_and(|s| ctx.members.contains(s))).collect();
    // `sorted(...)` of a str next to a non-str raises TypeError
    let strs = extra.iter().filter(|h| h.as_str().is_some()).count();
    if strs != 0 && strs != extra.len() {
        return crash("TypeError: '<' not supported between instances of 'str' and a non-str rejection reason");
    }
    for r in extra {
        report.fail("AC6", format!("toml message '{name}' rejection_reasons names '{}', not in the closed enumeration", r.py_str()));
    }
    Ok(())
}

fn one(ctx: &Ctx, m: &Msg, t: &Dict, report: &mut Report) -> Flow<()> {
    let name = str_of(t, "name");
    if ne_str(t, "layer", "control") {
        report.fail("AC6", format!("toml message '{name}' has layer '{}', not 'control'", str_of(t, "layer")));
    }
    if let Some(d) = ctx.dscp_control
        && ne_int(t, "dscp", d)
    {
        report.fail("AC6", format!("toml message '{name}' has dscp {}; C-13 fixes CS5 ({d})", str_of(t, "dscp")));
    }
    if ne_int(t, "max_size", m.total) {
        report.fail("AC6", format!("toml message '{name}' has max_size {}; the markdown table sums to {}", str_of(t, "max_size"), m.total));
    }
    // `isinstance(T.get("max_size"), int)` (a bool is one; a big integer is
    // recorded by `py_int`)
    let max_size = t.get("max_size").filter(|v| v.is_int()).and_then(|v| v.py_int().ok());
    if let Some(b) = ctx.bound.filter(|&b| b != 0)
        && let Some(s) = max_size
        && s > b
    {
        report.fail("AC6", format!("toml message '{name}' max_size {s} exceeds the {b}-byte bound (C-17)"));
    }
    let total = fields(m, t, report)?;
    if let (Some(s), Some(total)) = (max_size, total)
        && !num_eq_int(total, int_mul(s, 8))
    {
        report.fail("AC6", format!("the toml fields of '{name}' compute a size other than max_size {s}"));
    }
    reasons(ctx, m, t, report)
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, report: &mut Report) -> Flow<()> {
    let root = match ctx.toml_text.as_deref() {
        None => {
            report.fail("AC6", format!("{WIRE_TOML} does not exist"));
            None
        }
        Some(text) => match tomlsubset::parse(text) {
            Ok(d) => Some(d),
            Err(e) if e.starts_with("TomlError:") => {
                report.fail("AC6", format!("{WIRE_TOML} does not parse: {e}"));
                None
            }
            Err(e) => return crash(e),
        },
    };
    let msgs: Vec<PyValue> = match root.as_ref().and_then(|r| r.get("message")) {
        Some(PyValue::List(l)) => l.clone(),
        _ => Vec::new(),
    };
    for m in &ctx.msgs {
        let want = tname(&m.title);
        let Some(t) = msgs.iter().filter_map(PyValue::as_dict).find(|x| tname(&text_of(x, "name")) == want) else {
            report.fail("AC6", format!("wire.toml declares no [[message]] for the election message '{}'", m.title));
            continue;
        };
        one(ctx, m, t, report)?;
    }
    Ok(())
}
