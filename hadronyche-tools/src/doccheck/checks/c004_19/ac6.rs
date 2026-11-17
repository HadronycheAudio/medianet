//! AC6 (document part): the sync request and reply of wire.toml, read with
//! the scripts' subset reader, against the Markdown tables (script lines
//! 1121-1235; its toml size findings are reported under AC1 and its DSCP
//! names under W4, as the script does).

use std::collections::BTreeSet;

use crate::doccheck::wp7::common::{read, str_of};
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::pytoml::{self, PyDict, PyVal};
use super::{Ctx, Row};
use crate::doccheck::Report;

const PRIMITIVE: [&str; 14] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "q1.31", "uint", "pad"];

/// `name_tokens(text)`: split on non-alphanumerics and camelCase.
pub(crate) fn name_tokens(text: &str) -> Vec<String> {
    let spaced = crate::doccheck::text::rx_raw("([a-z0-9])([A-Z])").replace_all(text, "$1 $2");
    let spaced = crate::doccheck::text::rx_raw("([A-Z]+)([A-Z][a-z])").replace_all(&spaced, "$1 $2");
    spaced.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| !t.is_empty()).map(str::to_lowercase).collect()
}

/// `is_dscp_or_rate(text)`.
pub(crate) fn is_dscp_or_rate(text: &str) -> bool {
    name_tokens(text).iter().any(|t| ["dscp", "rate", "codepoint"].contains(&t.as_str()))
}

/// Python `a + b` for two values; `None` where Python raises.
fn add(a: &PyVal, b: &PyVal) -> Option<PyVal> {
    match (a, b) {
        (PyVal::Str(x), PyVal::Str(y)) => Some(PyVal::Str(format!("{x}{y}"))),
        (PyVal::List(x), PyVal::List(y)) => Some(PyVal::List(x.iter().chain(y).cloned().collect())),
        _ => match (a.as_int(), b.as_int()) {
            (Some(x), Some(y)) => Some(PyVal::Int(x.saturating_add(y))),
            _ => Some(PyVal::Float(a.number()? + b.number()?)),
        },
    }
}

fn or0(v: Option<&PyVal>) -> PyVal {
    v.filter(|v| v.truthy()).cloned().unwrap_or(PyVal::Int(0))
}

/// The sort key class of `x.get("offset", 0)`: numbers, str, or other.
fn key_class(v: &PyVal) -> u8 {
    match v {
        PyVal::Str(_) => 1,
        PyVal::List(_) | PyVal::Dict(_) => 2,
        _ => 0,
    }
}

/// `sorted(tfields, key=offset)`; `None` where the keys cannot be compared.
fn sorted_fields(tfields: &[PyVal]) -> Option<Vec<PyVal>> {
    let key = |f: &PyVal| f.as_dict().and_then(|d| d.get("offset").cloned()).unwrap_or(PyVal::Int(0));
    let classes: BTreeSet<u8> = tfields.iter().map(|f| key_class(&key(f))).collect();
    if tfields.len() > 1 && (classes.len() > 1 || classes.contains(&2)) {
        return None;
    }
    let mut out = tfields.to_vec();
    out.sort_by(|a, b| match (key(a), key(b)) {
        (PyVal::Str(x), PyVal::Str(y)) => x.cmp(&y),
        (x, y) => x.number().unwrap_or(0.0).partial_cmp(&y.number().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal),
    });
    Some(out)
}

/// One toml message; returns its reasons, or `None` after a crash.
fn message(name: &str, tm: &PyDict, fields_md: &[Row], size_md: Option<i128>, is_req: bool, ctx: &Ctx, report: &mut Report) -> Option<Vec<PyVal>> {
    if !tm.get("layer").is_some_and(|v| v.as_str() == Some("sync")) {
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
    let Some(ms) = tm.get("max_size").and_then(PyVal::as_int) else {
        report.fail("AC1", format!("{name} in wire.toml has no integer max_size"));
        return Some(Vec::new());
    };
    let tfields: Vec<PyVal> = match tm.get("field") {
        Some(PyVal::List(l)) => l.clone(),
        _ => Vec::new(),
    };
    let Some(sorted) = sorted_fields(&tfields) else {
        report.fail("CRASH", format!("TypeError: the offsets of {name} cannot be sorted"));
        return None;
    };
    let mut pos = PyVal::Int(0);
    let mut tlist: Vec<(Option<PyVal>, Option<PyVal>, String)> = Vec::new();
    for f in sorted.iter().filter_map(PyVal::as_dict) {
        let (o, w) = (f.get("offset").cloned(), f.get("width").cloned());
        tlist.push((o.clone(), w.clone(), f.get("type").map_or(String::new(), PyVal::str).to_lowercase()));
        if !o.as_ref().is_some_and(|o| o.eq(&pos)) {
            report.fail("AC1", format!("{name} in wire.toml: field '{}' is not contiguous (bit {} expected)", str_of(f, "name"), pos.str()));
        }
        let Some(next) = add(&or0(o.as_ref()), &or0(w.as_ref())) else {
            report.fail("CRASH", format!("TypeError: offset plus width of a field of {name}"));
            return None;
        };
        pos = next;
    }
    if !pos.eq_int(ms.saturating_mul(8)) {
        report.fail("AC1", format!("{name} in wire.toml: fields sum to {} bits, max_size is {ms} bytes", pos.str()));
    }
    if let Some(s) = size_md
        && ms != s
    {
        report.fail("AC1", format!("{name}: wire.toml max_size {ms} differs from the markdown size {s}"));
    }
    let reasons: Vec<PyVal> = match tm.get("rejection_reasons") {
        Some(PyVal::List(l)) => l.clone(),
        _ => Vec::new(),
    };
    if reasons.iter().any(|r| !r.hashable()) {
        report.fail("CRASH", "TypeError: unhashable rejection reason");
        return None;
    }
    for r in &reasons {
        if !r.as_str().is_some_and(|s| ctx.members.contains(s)) {
            report.fail("AC6", format!("{name} names '{}', which is not a member of the closed enumeration", r.str()));
        }
    }
    tiling(name, fields_md, &tlist, report);
    if is_req {
        pad(name, &tfields, report)?;
    }
    for f in tfields.iter().filter_map(PyVal::as_dict) {
        if is_dscp_or_rate(&str_of(f, "name")) {
            report.fail("W4", format!("{name} in wire.toml names a DSCP or rate field ('{}'); C-152 and C-204 keep both out of sync messages", str_of(f, "name")));
        }
    }
    Some(reasons)
}

fn tiling(name: &str, fields_md: &[Row], tlist: &[(Option<PyVal>, Option<PyVal>, String)], report: &mut Report) {
    if fields_md.is_empty() {
        return;
    }
    if fields_md.len() != tlist.len() {
        report.fail("AC6", format!("{name}: the markdown table has {} fields, wire.toml {}", fields_md.len(), tlist.len()));
        return;
    }
    for (m, (to, tw, tt)) in fields_md.iter().zip(tlist) {
        let same = to.as_ref().is_some_and(|v| v.eq_int(m.off)) && tw.as_ref().is_some_and(|v| v.eq_int(m.w));
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

/// The request's pad field: `type == "pad" and (width or 0) > 0`.
fn pad(name: &str, tfields: &[PyVal], report: &mut Report) -> Option<()> {
    let mut found = false;
    for f in tfields.iter().filter_map(PyVal::as_dict) {
        if !f.get("type").is_some_and(|t| t.as_str() == Some("pad")) {
            continue;
        }
        let w = or0(f.get("width"));
        match w.number() {
            Some(n) if n > 0.0 => {
                found = true;
                break;
            }
            Some(_) => {}
            None => {
                report.fail("CRASH", format!("TypeError: '>' between a {name} pad width and an int"));
                return None;
            }
        }
    }
    if !found {
        report.fail("AC1", "sync request in wire.toml has no pad field of width > 0 (the explicit padding)");
    }
    Some(())
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, form_table: Option<&Vec<Vec<String>>>, report: &mut Report) {
    let root = read(ctx.toml_text.as_deref(), "AC6", report);
    let msgs: &[PyVal] = root.as_ref().map_or(&[], |r| pytoml::list_of(r, "message"));
    let find = |p: &str| msgs.iter().filter_map(PyVal::as_dict).find(|m| py::fullmatch(p, &str_of(m, "name"), true).is_some());
    let (t_req, t_rep) = (find(r"sync[ _-]request"), find(r"sync[ _-]reply"));
    if t_req.is_none() {
        report.fail("AC6", "wire.toml declares no [[message]] named 'sync request'");
    }
    if t_rep.is_none() {
        report.fail("AC6", "wire.toml declares no [[message]] named 'sync reply'");
    }
    let mut all_reasons: Vec<PyVal> = Vec::new();
    for (name, tm, fields, size, is_req) in [("sync request", t_req, &ctx.req_f, ctx.req_size, true), ("sync reply", t_rep, &ctx.rep_f, ctx.rep_size, false)] {
        let Some(tm) = tm else { continue };
        match message(name, tm, fields, size, is_req, ctx, report) {
            Some(r) => all_reasons.extend(r),
            None => return,
        }
    }
    if ctx.placeholder || ctx.rep_f.is_empty() || ctx.req_f.is_empty() {
        return;
    }
    let mut exact = BTreeSet::new();
    for r in form_table.map_or(&[][..], |t| t.as_slice()) {
        let outcome = r.get(1..).map_or(String::new(), |c| c.join(" | "));
        for name in ctx.members.names() {
            let wanted = ["BadLength", "BadMagic", "VersionMismatch", "ForeignDomain", "ReservedNonZero", "WrongEpoch"];
            if wanted.contains(&name) && py::has(&format!(r"\b{name}\b"), &outcome) {
                exact.insert(name.to_string());
            }
        }
    }
    let missing: Vec<&String> = exact.iter().filter(|n| !all_reasons.iter().any(|r| r.as_str() == Some(n.as_str()))).collect();
    if !missing.is_empty() && t_req.is_some() && t_rep.is_some() {
        report.fail("AC6", format!("the sync messages' rejection_reasons in wire.toml miss: {}", missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
    }
}
