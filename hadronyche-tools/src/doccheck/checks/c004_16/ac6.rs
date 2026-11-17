//! AC6 (document part): the hello of wire.toml, read with the scripts'
//! subset reader, against the Markdown layout (script lines 1328-1407).

use std::collections::BTreeSet;

use super::ac1::Rows;
use super::{Ctx, Layout};
use crate::doccheck::Report;
use crate::doccheck::family004::{Field, NameNorm, md_name, norm_name};
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{has, starts};
use crate::doccheck::tomlsubset;
use crate::doccheck::wire_toml::WIRE_TOML;

const PRIM: [&str; 10] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"];

/// `str(d.get(k))` (`None` when absent).
fn str_of(d: &Dict, k: &str) -> String {
    d.get(k).map_or_else(|| "None".to_string(), PyValue::py_str)
}

/// `d.get(k) != n` for an int `n`. `n` is the script's own int; beyond
/// `i128` it is recorded by the caller, never compared here.
fn ne_int(d: &Dict, k: &str, n: i128) -> bool {
    !d.get(k).is_some_and(|v| v.eq_int(n))
}

/// `d.get(k) != "s"`.
fn ne_str(d: &Dict, k: &str, s: &str) -> bool {
    !d.get(k).is_some_and(|v| v.eq_str(s))
}

/// `n * 8` on the script's own ints (beyond `i128`: `INT_LIMIT`).
fn times8(n: i128) -> Option<i128> {
    Some(super::int_mul(n, 8))
}

/// A number of the size sum: an int, or a float once a float took part.
#[derive(Debug, Clone, Copy)]
enum Sum {
    Int(i128),
    Float(f64),
}

impl Sum {
    /// A TOML value as a number of the sum; `None` for a non-number (the
    /// script's `TypeError`); a big integer is recorded by `py_int`.
    fn of(v: &PyValue) -> Option<Sum> {
        match v {
            PyValue::Float(f) => Some(Sum::Float(*f)),
            v if v.is_int() => Some(Sum::Int(v.py_int().ok()?)),
            _ => None,
        }
    }

    fn f(self) -> f64 {
        match self {
            #[allow(clippy::cast_precision_loss)]
            Sum::Int(i) => i as f64,
            Sum::Float(f) => f,
        }
    }

    fn op(self, other: Sum, int: fn(i128, i128) -> i128, float: fn(f64, f64) -> f64) -> Sum {
        match (self, other) {
            (Sum::Int(a), Sum::Int(b)) => Sum::Int(int(a, b)),
            (a, b) => Sum::Float(float(a.f(), b.f())),
        }
    }

    /// `total_bits != n` for an int `n`.
    fn ne_int(self, n: i128) -> bool {
        match self {
            Sum::Int(i) => i != n,
            // exact, as Python compares a float with an int
            #[allow(clippy::cast_possible_truncation)]
            Sum::Float(f) => !(f.is_finite() && f.fract() == 0.0 && f.abs() < 1.0e38 && f as i128 == n),
        }
    }
}

/// `x.get(k, 0)` as a number of the sum, or the `TypeError` the script's
/// `*` or `+=` raises.
fn operand(d: &Dict, k: &str, name: &str) -> Flow<Option<Sum>> {
    match d.get(k) {
        None => Ok(Some(Sum::Int(0))),
        Some(v) if v.is_int() && v.py_int().is_err() => Ok(None),
        Some(v) => match Sum::of(v) {
            Some(s) => Ok(Some(s)),
            None => crash(format!("TypeError: the toml hello field {name} adds a {} to the size", v.type_name())),
        },
    }
}

/// The field rows (`T.get("field") or []`): a list's items, a string's
/// characters, a dict's keys; anything without `len()` is a `TypeError`.
fn items(t: &Dict) -> Flow<Vec<PyValue>> {
    match t.get("field").filter(|v| v.truthy()) {
        None => Ok(Vec::new()),
        Some(PyValue::List(l)) => Ok(l.clone()),
        Some(v @ (PyValue::Str(_) | PyValue::Dict(_))) => match v.py_iter_str() {
            Ok(s) => Ok(s.into_iter().map(PyValue::Str).collect()),
            Err(e) => crash(format!("TypeError: {e}")),
        },
        Some(v) => crash(format!("TypeError: object of type '{}' has no len()", v.type_name())),
    }
}

/// The fields, compared row by row; returns them and their bit sum (`None`
/// once a number beyond `i128` made the verdict red by rule).
fn fields(t: &Dict, lay: &[Field], report: &mut Report) -> Flow<(Vec<PyValue>, Option<Sum>)> {
    let tf = items(t)?;
    if tf.len() != lay.len() {
        report.fail("AC6", format!("the toml hello has {} fields; the markdown layout has {} rows: they must correspond one to one", tf.len(), lay.len()));
    }
    // `[f.get("name") for f in tf if isinstance(f, dict)]` (None when absent)
    let names: Vec<Option<PyValue>> = tf.iter().filter_map(PyValue::as_dict).map(|f| f.get("name").cloned()).collect();
    let mut total = Some(Sum::Int(0));
    for (i, (mf, tfl)) in lay.iter().zip(&tf).enumerate() {
        let Some(tfl) = tfl.as_dict() else { continue };
        let tname = str_of(tfl, "name");
        let field_name = tfl.get("name").map_or_else(String::new, PyValue::py_str);
        if norm_name(md_name(&mf.meaning), NameNorm::Spaced004_16) != norm_name(&field_name, NameNorm::Spaced004_16) {
            report.fail("AC6", format!("toml hello field '{tname}' is not named after the markdown row '{}' (names are compared lower case, spaces and underscores equivalent)", md_name(&mf.meaning)));
        }
        if times8(mf.off).is_some_and(|o| ne_int(tfl, "offset", o)) {
            report.fail("AC6", format!("toml hello field {tname}: offset {} bits, markdown byte {}", str_of(tfl, "offset"), mf.off));
        }
        let add = if let Some(maxc) = mf.maxc {
            if ne_int(tfl, "width", mf.bits) {
                report.fail("AC6", format!("toml hello field {tname}: width {} bits per element; markdown element is {} bits", str_of(tfl, "width"), mf.bits));
            }
            if ne_int(tfl, "max_count", maxc) {
                report.fail("AC6", format!("toml hello field {tname}: max_count {}; markdown says max {maxc}", str_of(tfl, "max_count")));
            }
            let count = tfl.get("count");
            let earlier = &names[..i.min(names.len())];
            // `x in list` is `any(x is e or x == e)`; `None` (an absent name)
            // is never a truthy count.
            if !count.is_some_and(|c| c.truthy() && earlier.iter().flatten().any(|n| n.py_eq(c))) {
                report.fail("AC6", format!("toml hello field {tname}: its count must name an earlier field"));
            }
            match (operand(tfl, "width", &tname)?, operand(tfl, "max_count", &tname)?) {
                (Some(w), Some(m)) => Some(w.op(m, super::int_mul, |a, b| a * b)),
                _ => None,
            }
        } else {
            if ne_int(tfl, "width", mf.bits) {
                report.fail("AC6", format!("toml hello field {tname}: width {} bits; markdown {}", str_of(tfl, "width"), mf.bits));
            }
            if PRIM.contains(&mf.ty.as_str()) && ne_str(tfl, "type", &mf.ty) {
                report.fail("AC6", format!("toml hello field {tname}: type {}; markdown {}", str_of(tfl, "type"), mf.ty));
            }
            operand(tfl, "width", &tname)?
        };
        total = match (total, add) {
            (Some(t), Some(a)) => Some(t.op(a, super::int_add, |a, b| a + b)),
            _ => None,
        };
    }
    Ok((tf, total))
}

fn nested(ctx: &Ctx, tf: &[PyValue], lay: &[Field], rows: &Rows, report: &mut Report) {
    let p_max = rows.port_row.and_then(|i| lay[i].maxc);
    for (i, tfl) in tf.iter().enumerate().take(lay.len()) {
        let Some(tfl) = tfl.as_dict() else { continue };
        let text_of = |k: &str| tfl.get(k).map_or_else(String::new, PyValue::py_str);
        let txt = format!("{}{}", text_of("valid"), text_of("meaning"));
        if rows.port_row == Some(i) {
            for v in [p_max, rows.t_ct].into_iter().flatten() {
                if !has(&format!(r"\b{v}\b"), &txt) {
                    report.fail("AC6", format!("the toml port field does not state the maximum {v} of its nested count"));
                }
            }
        }
        if rows.bun_row == Some(i)
            && let Some(mm) = ctx.k.members_max
            && !has(&format!(r"\b{mm}\b"), &txt)
        {
            report.fail("AC6", format!("the toml bundle field does not state the {mm}-member maximum (C-08)"));
        }
    }
}

/// `set(T.get("rejection_reasons") or [])`, then the two sorted differences.
fn reasons(ctx: &Ctx, t: &Dict, exact: &BTreeSet<String>, report: &mut Report) -> Flow<()> {
    if exact.is_empty() {
        return Ok(());
    }
    let have: Vec<PyValue> = match t.get("rejection_reasons").filter(|v| v.truthy()) {
        None => Vec::new(),
        Some(PyValue::List(l)) => {
            // `set(...)` hashes every item: a list or a dict is unhashable.
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
    for r in exact {
        if !have.iter().any(|h| h.eq_str(r)) {
            report.fail("AC6", format!("the toml hello rejection_reasons does not list '{r}', which the invalid-form rows name"));
        }
    }
    let extra: Vec<&PyValue> = have.iter().filter(|h| !h.as_str().is_some_and(|s| ctx.members.contains(s))).collect();
    // `sorted(have - set(MEMBERS))`: a str next to a non-str is a TypeError
    // (numbers compare among themselves).
    let strs = extra.iter().filter(|h| h.as_str().is_some()).count();
    if strs != 0 && strs != extra.len() {
        return crash("TypeError: '<' not supported between instances of 'str' and a non-str rejection reason");
    }
    for r in extra {
        report.fail("AC6", format!("the toml hello rejection_reasons names '{}', which is not in the closed enumeration", r.py_str()));
    }
    Ok(())
}

fn hello_message(ctx: &Ctx, t: &Dict, lay: &Layout, rows: &Rows, exact: &BTreeSet<String>, report: &mut Report) -> Flow<()> {
    let k = &ctx.k;
    let h = lay.h_bytes;
    if ne_str(t, "layer", "control") {
        report.fail("AC6", format!("the toml hello has layer '{}', not 'control'", str_of(t, "layer")));
    }
    if let Some(d) = k.dscp_control
        && ne_int(t, "dscp", d)
    {
        report.fail("AC6", format!("the toml hello has dscp {}; C-13 fixes CS5 ({d})", str_of(t, "dscp")));
    }
    if ne_int(t, "max_size", h) {
        report.fail("AC6", format!("the toml hello has max_size {}; the markdown layout sums to {h}", str_of(t, "max_size")));
    }
    // `isinstance(T.get("max_size"), int)`: a bool is an int; a big integer
    // is recorded (`INT_LIMIT`) by `py_int`.
    let max_size = t.get("max_size").filter(|v| v.is_int()).map(PyValue::py_int);
    let max_size = match max_size {
        Some(Ok(m)) => Some(m),
        Some(Err(_)) | None => None,
    };
    if k.bound != 0
        && let Some(m) = max_size
        && m > k.bound
    {
        report.fail("AC6", format!("the toml hello max_size {m} exceeds the {}-byte control bound (C-17)", k.bound));
    }
    let (tf, total) = fields(t, &lay.lay, report)?;
    if let (Some(m), Some(total)) = (max_size, total)
        && times8(m).is_some_and(|m8| total.ne_int(m8))
    {
        report.fail("AC6", format!("the toml hello fields compute {total:?} bits; max_size is {m}"));
    }
    nested(ctx, &tf, &lay.lay, rows, report);
    reasons(ctx, t, exact, report)
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, lay: &Layout, rows: &Rows, exact: &BTreeSet<String>, report: &mut Report) -> Flow<()> {
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
    let dicts: Vec<&Dict> = msgs.iter().filter_map(PyValue::as_dict).collect();
    let name_of = |m: &Dict| m.get("name").map_or_else(String::new, PyValue::py_str);
    for m in &dicts {
        if starts("(?i)hello", &name_of(m)) && ne_str(m, "layer", "control") {
            report.fail("AC6", format!("toml message '{}' is a hello message but its layer is not 'control'", str_of(m, "name")));
        }
    }
    let hello = dicts.iter().find(|m| name_of(m).py_strip().to_lowercase() == "hello");
    match hello {
        None => {
            report.fail("AC6", "wire.toml declares no [[message]] named 'hello'");
            Ok(())
        }
        Some(t) => hello_message(ctx, t, lay, rows, exact, report),
    }
}
