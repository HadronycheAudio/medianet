//! AC6 (document part): the sync request and reply of wire.toml, read with
//! the scripts' subset reader, against the Markdown tables (script lines
//! 1121-1235; its toml size findings are reported under AC1 and its DSCP
//! names under W4, as the script does).

use std::collections::BTreeSet;

use super::{Ctx, Row, int_add, int_mul};
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has, rx};
use crate::doccheck::tomlsubset;
use crate::doccheck::wire_toml::WIRE_TOML;

const PRIMITIVE: [&str; 14] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "q1.31", "uint", "pad"];

/// `name_tokens(text)`: split on non-alphanumerics and camelCase.
pub(crate) fn name_tokens(text: &str) -> Vec<String> {
    let spaced = rx("([a-z0-9])([A-Z])").replace_all(text, "$1 $2");
    let spaced = rx("([A-Z]+)([A-Z][a-z])").replace_all(&spaced, "$1 $2");
    rx("[^A-Za-z0-9]+").split(&spaced).filter(|t| !t.is_empty()).map(str::to_lowercase).collect()
}

/// `is_dscp_or_rate(text)`.
pub(crate) fn is_dscp_or_rate(text: &str) -> bool {
    name_tokens(text).iter().any(|t| ["dscp", "rate", "codepoint"].contains(&t.as_str()))
}

/// `str(d.get(k))` (`None` when absent).
fn str_of(d: &Dict, k: &str) -> String {
    d.get(k).map_or_else(|| "None".to_string(), PyValue::py_str)
}

/// A Python number as `(int, float)` for comparisons: `None` for a
/// non-number. A big integer is recorded (`INT_LIMIT`) by `py_int` and
/// compares by its sign.
fn number(v: &PyValue) -> Option<f64> {
    match v {
        PyValue::Float(f) => Some(*f),
        PyValue::BigInt(s) => {
            let _ = v.py_int();
            Some(if s.starts_with('-') { f64::NEG_INFINITY } else { f64::INFINITY })
        }
        v if v.is_int() => v.py_int().ok().map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let f = i as f64;
            f
        }),
        _ => None,
    }
}

/// Python's `a < b`; `Err` is the script's `TypeError`.
fn py_lt(a: &PyValue, b: &PyValue) -> Result<bool, String> {
    if let (Some(x), Some(y)) = (number(a), number(b)) {
        // two ints compare exactly
        if let (Ok(i), Ok(j)) = (a.py_int(), b.py_int())
            && a.is_int()
            && b.is_int()
        {
            return Ok(i < j);
        }
        return Ok(x < y);
    }
    match (a, b) {
        (PyValue::Str(x), PyValue::Str(y)) => Ok(x < y),
        (PyValue::List(x), PyValue::List(y)) => {
            for (p, q) in x.iter().zip(y) {
                if !p.py_eq(q) {
                    return py_lt(p, q);
                }
            }
            Ok(x.len() < y.len())
        }
        _ => Err(format!("'<' not supported between instances of '{}' and '{}'", a.type_name(), b.type_name())),
    }
}

/// `sorted(items, key)`: CPython 3.10's sort for fewer than 64 items (one
/// run, then binary insertion), with Python's `<`; `Err` is the
/// `TypeError` of the first comparison that raises.
fn py_sorted<T: Clone>(items: &[T], key: impl Fn(&T) -> PyValue) -> Result<Vec<T>, String> {
    let mut v: Vec<(PyValue, T)> = items.iter().map(|x| (key(x), x.clone())).collect();
    let n = v.len();
    if n < 2 {
        return Ok(v.into_iter().map(|(_, x)| x).collect());
    }
    // count_run
    let mut run = 2;
    if py_lt(&v[1].0, &v[0].0)? {
        while run < n && py_lt(&v[run].0, &v[run - 1].0)? {
            run += 1;
        }
        v[..run].reverse();
    } else {
        while run < n && !py_lt(&v[run].0, &v[run - 1].0)? {
            run += 1;
        }
    }
    // binarysort(lo, hi, start = run)
    for start in run..n {
        let (mut l, mut r) = (0, start);
        while l < r {
            let p = l + (r - l) / 2;
            if py_lt(&v[start].0, &v[p].0)? {
                r = p;
            } else {
                l = p + 1;
            }
        }
        let pivot = v.remove(start);
        v.insert(l, pivot);
    }
    Ok(v.into_iter().map(|(_, x)| x).collect())
}

/// Python's `a + b`; `Err` is the script's `TypeError`.
fn py_add(a: &PyValue, b: &PyValue) -> Result<PyValue, String> {
    match (a, b) {
        (PyValue::Str(x), PyValue::Str(y)) => Ok(PyValue::Str(format!("{x}{y}"))),
        (PyValue::List(x), PyValue::List(y)) => Ok(PyValue::List(x.iter().chain(y).cloned().collect())),
        (x, y) if x.is_int() && y.is_int() => match (x.py_int(), y.py_int()) {
            (Ok(i), Ok(j)) => Ok(PyValue::Int(int_add(i, j))),
            // beyond i128: INT_LIMIT (recorded by py_int)
            _ => Ok(PyValue::Int(0)),
        },
        (x, y) => match (number(x), number(y)) {
            (Some(i), Some(j)) => Ok(PyValue::Float(i + j)),
            _ => Err(format!("unsupported operand type(s) for +: '{}' and '{}'", x.type_name(), y.type_name())),
        },
    }
}

/// `v or 0`.
fn or0(v: Option<&PyValue>) -> PyValue {
    v.filter(|v| v.truthy()).cloned().unwrap_or(PyValue::Int(0))
}

/// `a != n` for an int `n` (`None` is never equal).
fn ne_int(a: Option<&PyValue>, n: i128) -> bool {
    !a.is_some_and(|v| v.eq_int(n))
}

/// One toml message; returns its reasons.
fn message(name: &str, tm: &Dict, fields_md: &[Row], size_md: Option<i128>, is_req: bool, ctx: &Ctx, report: &mut Report) -> Flow<Vec<PyValue>> {
    if !tm.get("layer").is_some_and(|v| v.eq_str("sync")) {
        report.fail("AC6", format!("{name} in wire.toml must have layer = \"sync\""));
    }
    let dscp_ok = match (tm.get("dscp"), ctx.k.dscp) {
        (None, None) => true,
        (Some(v), Some(d)) => v.eq_int(d),
        _ => false,
    };
    if !dscp_ok {
        report.fail("AC6", format!("{name} in wire.toml must have dscp = {} (C-13, sync CS6)", super::ac1::opt(ctx.k.dscp)));
    }
    // `isinstance(ms, int)`: a bool is one; a big integer is recorded
    // (INT_LIMIT) by py_int and the message is not compared further
    let Some(ms) = tm.get("max_size").filter(|v| v.is_int()) else {
        report.fail("AC1", format!("{name} in wire.toml has no integer max_size"));
        return Ok(Vec::new());
    };
    let Ok(ms) = ms.py_int() else { return Ok(Vec::new()) };
    let tfields: Vec<PyValue> = match tm.get("field") {
        Some(PyValue::List(l)) => l.clone(),
        _ => Vec::new(),
    };
    let key = |x: &PyValue| x.as_dict().map_or(PyValue::Int(0), |d| d.get("offset").cloned().unwrap_or(PyValue::Int(0)));
    let sorted = match py_sorted(&tfields, key) {
        Ok(s) => s,
        Err(e) => return crash(format!("TypeError: {e} (sorting the fields of {name} by offset)")),
    };
    let mut pos = PyValue::Int(0);
    let mut tlist: Vec<(Option<PyValue>, Option<PyValue>, String)> = Vec::new();
    for f in sorted.iter().filter_map(PyValue::as_dict) {
        let (o, w) = (f.get("offset").cloned(), f.get("width").cloned());
        tlist.push((o.clone(), w.clone(), f.get("type").map_or_else(String::new, PyValue::py_str).to_lowercase()));
        if !o.as_ref().is_some_and(|o| o.py_eq(&pos)) {
            report.fail("AC1", format!("{name} in wire.toml: field '{}' is not contiguous (bit {} expected)", str_of(f, "name"), pos.py_str()));
        }
        pos = match py_add(&or0(o.as_ref()), &or0(w.as_ref())) {
            Ok(p) => p,
            Err(e) => return crash(format!("TypeError: {e} (offset plus width of a field of {name})")),
        };
    }
    if !pos.eq_int(int_mul(ms, 8)) {
        report.fail("AC1", format!("{name} in wire.toml: fields sum to {} bits, max_size is {ms} bytes", pos.py_str()));
    }
    if let Some(s) = size_md
        && ms != s
    {
        report.fail("AC1", format!("{name}: wire.toml max_size {ms} differs from the markdown size {s}"));
    }
    let reasons: Vec<PyValue> = match tm.get("rejection_reasons") {
        Some(PyValue::List(l)) => l.clone(),
        _ => Vec::new(),
    };
    // `ALL_TOML_REASONS.update(...)` hashes every item
    if let Some(bad) = reasons.iter().find(|r| matches!(r, PyValue::List(_) | PyValue::Dict(_))) {
        return crash(format!("TypeError: unhashable type: '{}'", bad.type_name()));
    }
    for r in &reasons {
        if !r.as_str().is_some_and(|s| ctx.members.contains(s)) {
            report.fail("AC6", format!("{name} names '{}', which is not a member of the closed enumeration", r.py_str()));
        }
    }
    tiling(name, fields_md, &tlist, report);
    if is_req {
        pad(name, &tfields, report)?;
    }
    for f in tfields.iter().filter_map(PyValue::as_dict) {
        let n = f.get("name").map_or_else(String::new, PyValue::py_str);
        if is_dscp_or_rate(&n) {
            report.fail("W4", format!("{name} in wire.toml names a DSCP or rate field ('{}'); C-152 and C-204 keep both out of sync messages", str_of(f, "name")));
        }
    }
    Ok(reasons)
}

fn tiling(name: &str, fields_md: &[Row], tlist: &[(Option<PyValue>, Option<PyValue>, String)], report: &mut Report) {
    if fields_md.is_empty() {
        return;
    }
    if fields_md.len() != tlist.len() {
        report.fail("AC6", format!("{name}: the markdown table has {} fields, wire.toml {}", fields_md.len(), tlist.len()));
        return;
    }
    for (m, (to, tw, tt)) in fields_md.iter().zip(tlist) {
        let same = !ne_int(to.as_ref(), m.off) && !ne_int(tw.as_ref(), m.w);
        if !same {
            report.fail("AC6", format!("{name}: markdown field at bit {} ({} bits) does not match wire.toml field", m.off, m.w));
            break;
        }
        if PRIMITIVE.contains(&m.ty.as_str()) && m.ty != *tt {
            report.fail("AC6", format!("{name}: markdown type '{}' at bit {} does not match wire.toml type '{tt}'", m.ty, m.off));
            break;
        }
    }
}

/// The request's pad field: `any(type == "pad" and (width or 0) > 0)`;
/// `>` of a non-number and 0 raises `TypeError`.
fn pad(name: &str, tfields: &[PyValue], report: &mut Report) -> Flow<()> {
    let mut found = false;
    for f in tfields.iter().filter_map(PyValue::as_dict) {
        if !f.get("type").is_some_and(|t| t.eq_str("pad")) {
            continue;
        }
        let w = or0(f.get("width"));
        match py_lt(&PyValue::Int(0), &w) {
            Ok(true) => {
                found = true;
                break;
            }
            Ok(false) => {}
            Err(e) => return crash(format!("TypeError: {e} (a {name} pad width against 0)")),
        }
    }
    if !found {
        report.fail("AC1", "sync request in wire.toml has no pad field of width > 0 (the explicit padding)");
    }
    Ok(())
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, form_table: Option<&Vec<Vec<String>>>, report: &mut Report) -> Flow<()> {
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
    let find = |p: &str| msgs.iter().filter_map(PyValue::as_dict).find(|m| rx(&format!("(?i)^(?:{p})\\z")).is_match(&m.get("name").map_or_else(String::new, PyValue::py_str)));
    let (t_req, t_rep) = (find(r"sync[ _-]request"), find(r"sync[ _-]reply"));
    if t_req.is_none() {
        report.fail("AC6", "wire.toml declares no [[message]] named 'sync request'");
    }
    if t_rep.is_none() {
        report.fail("AC6", "wire.toml declares no [[message]] named 'sync reply'");
    }
    let mut all_reasons: Vec<PyValue> = Vec::new();
    for (name, tm, fields, size, is_req) in [("sync request", t_req, &ctx.req_f, ctx.req_size, true), ("sync reply", t_rep, &ctx.rep_f, ctx.rep_size, false)] {
        let Some(tm) = tm else { continue };
        all_reasons.extend(message(name, tm, fields, size, is_req, ctx, report)?);
    }
    if ctx.placeholder || ctx.rep_f.is_empty() || ctx.req_f.is_empty() {
        return Ok(());
    }
    let mut exact = BTreeSet::new();
    for r in form_table.map_or(&[][..], |t| t.as_slice()) {
        let outcome = r.get(1..).map_or(String::new(), |c| c.join(" | "));
        for name in ctx.members.names() {
            let wanted = ["BadLength", "BadMagic", "VersionMismatch", "ForeignDomain", "ReservedNonZero", "WrongEpoch"];
            if wanted.contains(&name) && has(&format!(r"\b{name}\b"), &outcome) {
                exact.insert(name.to_string());
            }
        }
    }
    let missing: Vec<&String> = exact.iter().filter(|n| !all_reasons.iter().any(|r| r.eq_str(n))).collect();
    if !missing.is_empty() && t_req.is_some() && t_rep.is_some() {
        report.fail("AC6", format!("the sync messages' rejection_reasons in wire.toml miss: {}", missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
    }
    Ok(())
}
