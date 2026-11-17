//! AC6 (document part): the hello of wire.toml, read with the scripts'
//! subset reader, against the Markdown layout (script lines 1328-1407).

use std::collections::BTreeSet;

use super::ac1::{Rows, md_name, norm_name};
use super::md::Field;
use super::pytoml::{self, Fail, PyDict, PyVal};
use super::{Ctx, Layout, py};
use crate::doccheck::Report;
use crate::doccheck::wire_toml::WIRE_TOML;

const PRIM: [&str; 10] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"];

/// A running Python sum: an int until a float joins it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Num {
    I(i128),
    F(f64),
}

impl Num {
    /// `self + v` for a numeric value; `None` where Python raises
    /// `TypeError` (a str, list or dict joins the sum).
    pub fn add(self, v: &PyVal) -> Option<Num> {
        Some(match (self, v) {
            (Num::I(a), PyVal::Int(_) | PyVal::Bool(_)) => Num::I(a.saturating_add(v.as_int()?)),
            (Num::I(a), PyVal::Float(f)) => Num::F(a as f64 + f),
            (Num::F(a), _) => Num::F(a + v.number()?),
            _ => return None,
        })
    }

    /// Python `self != n`.
    pub fn ne_int(self, n: i128) -> bool {
        match self {
            Num::I(a) => a != n,
            Num::F(a) => a != n as f64,
        }
    }
}

/// `a * b` for two values; `None` unless both are numbers (a str or list
/// times an int is a str or list, which the sum then rejects).
pub(crate) fn mul(a: &PyVal, b: &PyVal) -> Option<PyVal> {
    match (a.as_int(), b.as_int()) {
        (Some(x), Some(y)) => Some(PyVal::Int(x.saturating_mul(y))),
        _ => Some(PyVal::Float(a.number()? * b.number()?)),
    }
}

/// `d.get(key, default)` with an int default.
pub(crate) fn get_or(d: &PyDict, key: &str, default: i128) -> PyVal {
    d.get(key).cloned().unwrap_or(PyVal::Int(default))
}

/// `d.get(key)` compared with `!=` against an int (a missing key is `None`,
/// unequal to every int).
pub(crate) fn ne(d: &PyDict, key: &str, n: i128) -> bool {
    !d.get(key).is_some_and(|v| v.eq_int(n))
}

/// `d.get(key) != text` (None is unequal).
pub(crate) fn ne_str(d: &PyDict, key: &str, text: &str) -> bool {
    !d.get(key).is_some_and(|v| v.as_str() == Some(text))
}

/// `str(d.get(key, ""))`.
pub(crate) fn str_of(d: &PyDict, key: &str) -> String {
    d.get(key).map_or(String::new(), PyVal::str)
}

/// Reads wire.toml; `None` after reporting a missing file, an error or a
/// crash.
pub(crate) fn read(text: Option<&str>, ac: &str, report: &mut Report) -> Option<PyDict> {
    let Some(text) = text else {
        report.fail(ac, format!("{WIRE_TOML} does not exist"));
        return None;
    };
    match pytoml::parse(text) {
        Ok(d) => Some(d),
        Err(Fail::Error(e)) => {
            report.fail(ac, format!("{WIRE_TOML} does not parse: {e}"));
            None
        }
        Err(Fail::Crash(e)) => {
            report.fail("CRASH", format!("the script's TOML reader raises: {e}"));
            None
        }
    }
}

fn fields(t: &PyDict, lay: &[Field], report: &mut Report) -> Option<(Vec<PyVal>, Num)> {
    let tf_val = t.get("field").filter(|v| v.truthy()).cloned().unwrap_or(PyVal::List(Vec::new()));
    let (Some(n), Some(tf)) = (tf_val.len(), tf_val.iter_items()) else {
        report.fail("CRASH", "TypeError: the hello's field value has no len()");
        return None;
    };
    if n != lay.len() {
        report.fail("AC6", format!("the toml hello has {n} fields; the markdown layout has {} rows: they must correspond one to one", lay.len()));
    }
    // `[f.get("name") for f in tf if isinstance(f, dict)]` (None when absent)
    let names: Vec<Option<PyVal>> = tf.iter().filter_map(PyVal::as_dict).map(|f| f.get("name").cloned()).collect();
    let mut total = Num::I(0);
    for (i, (mf, tfl)) in lay.iter().zip(&tf).enumerate() {
        let Some(tfl) = tfl.as_dict() else { continue };
        let tname = str_of(tfl, "name");
        if norm_name(md_name(&mf.meaning)) != norm_name(&tname) {
            report.fail("AC6", format!("toml hello field '{tname}' is not named after the markdown row '{}' (names are compared lower case, spaces and underscores equivalent)", md_name(&mf.meaning)));
        }
        if ne(tfl, "offset", mf.off.saturating_mul(8)) {
            report.fail("AC6", format!("toml hello field {tname}: offset {} bits, markdown byte {} (= {} bits)", str_of(tfl, "offset"), mf.off, mf.off * 8));
        }
        let add = if let Some(maxc) = mf.maxc {
            if ne(tfl, "width", mf.bits) {
                report.fail("AC6", format!("toml hello field {tname}: width {} bits per element; markdown element is {} bits", str_of(tfl, "width"), mf.bits));
            }
            if ne(tfl, "max_count", maxc) {
                report.fail("AC6", format!("toml hello field {tname}: max_count {}; markdown says max {maxc}", str_of(tfl, "max_count")));
            }
            let count = tfl.get("count");
            let earlier = &names[..i.min(names.len())];
            if !count.is_some_and(|c| c.truthy() && earlier.iter().flatten().any(|n| n.eq(c))) {
                report.fail("AC6", format!("toml hello field {tname}: its count must name an earlier field"));
            }
            mul(&get_or(tfl, "width", 0), &get_or(tfl, "max_count", 0))
        } else {
            if ne(tfl, "width", mf.bits) {
                report.fail("AC6", format!("toml hello field {tname}: width {} bits; markdown {}", str_of(tfl, "width"), mf.bits));
            }
            if PRIM.contains(&mf.ty.as_str()) && ne_str(tfl, "type", &mf.ty) {
                report.fail("AC6", format!("toml hello field {tname}: type {}; markdown {}", str_of(tfl, "type"), mf.ty));
            }
            Some(get_or(tfl, "width", 0))
        };
        match add.and_then(|v| total.add(&v)) {
            Some(t) => total = t,
            None => {
                report.fail("CRASH", format!("TypeError: the toml hello field {tname} adds a non-number to the size"));
                return None;
            }
        }
    }
    Some((tf, total))
}

fn nested(ctx: &Ctx, tf: &[PyVal], lay: &[Field], rows: &Rows, report: &mut Report) {
    let p_max = rows.port_row.and_then(|i| lay[i].maxc);
    for (i, tfl) in tf.iter().enumerate().take(lay.len()) {
        let Some(tfl) = tfl.as_dict() else { continue };
        let txt = format!("{}{}", str_of(tfl, "valid"), str_of(tfl, "meaning"));
        if rows.port_row == Some(i) {
            for v in [p_max, rows.t_ct].into_iter().flatten() {
                if !py::has(&format!(r"\b{v}\b"), &txt) {
                    report.fail("AC6", format!("the toml port field does not state the maximum {v} of its nested count"));
                }
            }
        }
        if rows.bun_row == Some(i)
            && let Some(mm) = ctx.k.members_max
            && !py::has(&format!(r"\b{mm}\b"), &txt)
        {
            report.fail("AC6", format!("the toml bundle field does not state the {mm}-member maximum (C-08)"));
        }
    }
}

/// `set(v or [])`: the distinct items, or the TypeError Python raises (a
/// value that is not iterable, or an unhashable item).
pub(crate) fn set_of(v: Option<&PyVal>) -> Result<Vec<PyVal>, &'static str> {
    let raw = v.filter(|v| v.truthy()).cloned().unwrap_or(PyVal::List(Vec::new()));
    let Some(items) = raw.iter_items() else {
        return Err("TypeError: the value is not iterable");
    };
    if items.iter().any(|v| !v.hashable()) {
        return Err("TypeError: unhashable item");
    }
    let mut have: Vec<PyVal> = Vec::new();
    for v in items {
        if !have.iter().any(|h| h.eq(&v)) {
            have.push(v);
        }
    }
    Ok(have)
}

/// `sorted(values)` does not raise: no str beside a non-str.
pub(crate) fn sortable(values: &[&PyVal]) -> bool {
    let strs = values.iter().filter(|v| v.as_str().is_some()).count();
    values.len() < 2 || strs == 0 || strs == values.len()
}

fn reasons(ctx: &Ctx, t: &PyDict, exact: &BTreeSet<String>, report: &mut Report) {
    if exact.is_empty() {
        return;
    }
    let have = match set_of(t.get("rejection_reasons")) {
        Ok(h) => h,
        Err(e) => {
            report.fail("CRASH", format!("{e} (rejection_reasons is not iterable or holds a list)"));
            return;
        }
    };
    for r in exact {
        if !have.iter().any(|h| h.as_str() == Some(r.as_str())) {
            report.fail("AC6", format!("the toml hello rejection_reasons does not list '{r}', which the invalid-form rows name"));
        }
    }
    let extra: Vec<&PyVal> = have.iter().filter(|h| !h.as_str().is_some_and(|s| ctx.members.contains(s))).collect();
    if !sortable(&extra) {
        report.fail("CRASH", "TypeError: sorted() of str and non-str rejection reasons");
        return;
    }
    for r in extra {
        report.fail("AC6", format!("the toml hello rejection_reasons names '{}', which is not in the closed enumeration", r.str()));
    }
}

fn hello_message(ctx: &Ctx, t: &PyDict, lay: &Layout, rows: &Rows, exact: &BTreeSet<String>, report: &mut Report) {
    let k = &ctx.k;
    let h = lay.h_bytes;
    if ne_str(t, "layer", "control") {
        report.fail("AC6", format!("the toml hello has layer '{}', not 'control'", str_of(t, "layer")));
    }
    if let Some(d) = k.dscp_control
        && ne(t, "dscp", d)
    {
        report.fail("AC6", format!("the toml hello has dscp {}; C-13 fixes CS5 ({d})", str_of(t, "dscp")));
    }
    if ne(t, "max_size", h) {
        report.fail("AC6", format!("the toml hello has max_size {}; the markdown layout sums to {h}", str_of(t, "max_size")));
    }
    let max_size = t.get("max_size").and_then(PyVal::as_int);
    if k.bound != 0
        && let Some(m) = max_size
        && m > k.bound
    {
        report.fail("AC6", format!("the toml hello max_size {m} exceeds the {}-byte control bound (C-17)", k.bound));
    }
    let Some((tf, total)) = fields(t, &lay.lay, report) else { return };
    if let Some(m) = max_size
        && total.ne_int(m.saturating_mul(8))
    {
        report.fail("AC6", format!("the toml hello fields compute {} bits; max_size is {m}", match total { Num::I(i) => i.to_string(), Num::F(f) => pytoml::float_repr(f) }));
    }
    nested(ctx, &tf, &lay.lay, rows, report);
    reasons(ctx, t, exact, report);
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, lay: &Layout, rows: &Rows, exact: &BTreeSet<String>, report: &mut Report) {
    let root = read(ctx.toml_text.as_deref(), "AC6", report);
    let msgs: &[PyVal] = root.as_ref().map_or(&[], |r| pytoml::list_of(r, "message"));
    let dicts: Vec<&PyDict> = msgs.iter().filter_map(PyVal::as_dict).collect();
    for m in &dicts {
        if py::matches("hello", &str_of(m, "name"), true).is_some() && ne_str(m, "layer", "control") {
            report.fail("AC6", format!("toml message '{}' is a hello message but its layer is not 'control'", str_of(m, "name")));
        }
    }
    let hello = dicts.iter().find(|m| py::strip(&str_of(m, "name")).to_lowercase() == "hello");
    match hello {
        None => report.fail("AC6", "wire.toml declares no [[message]] named 'hello'"),
        Some(t) => hello_message(ctx, t, lay, rows, exact, report),
    }
}
