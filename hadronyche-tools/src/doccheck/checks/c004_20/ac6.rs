//! AC6 (document part): the measurement messages of wire.toml, read with
//! the scripts' subset reader (script lines 1323-1431; a missing toml
//! report or probe message is reported under AC1, as the script does).

use std::collections::BTreeSet;

use super::{Ctx, Field};
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::has_i;
use crate::doccheck::tomlsubset;
use crate::doccheck::wire_toml::WIRE_TOML;

const PRIMITIVE: [&str; 14] = ["uint", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "q1.31", "pad"];

/// `str(d.get(k))` (`None` when absent).
fn str_of(d: &Dict, k: &str) -> String {
    d.get(k).map_or_else(|| "None".to_string(), PyValue::py_str)
}

/// `str(d.get(k, ""))`.
fn text_of(d: &Dict, k: &str) -> String {
    d.get(k).map_or_else(String::new, PyValue::py_str)
}

/// `x == want` for an optional int (`None == None` is true in Python).
fn eq_opt(v: Option<&PyValue>, want: Option<i128>) -> bool {
    match (v, want) {
        (None, None) => true,
        (Some(v), Some(w)) => v.eq_int(w),
        _ => false,
    }
}

/// `x in a_set`: hashing a list or a dict raises `TypeError`.
fn hashable(v: &PyValue) -> Flow<()> {
    match v {
        PyValue::List(_) | PyValue::Dict(_) => crash(format!("TypeError: unhashable type: '{}'", v.type_name())),
        _ => Ok(()),
    }
}

/// The items of `(m.get("field") or [])`: a list's items, a string's
/// characters, a dict's keys; a non-iterable is the script's `TypeError`.
fn field_items(m: &Dict) -> Flow<Vec<PyValue>> {
    match m.get("field").filter(|v| v.truthy()) {
        None => Ok(Vec::new()),
        Some(PyValue::List(l)) => Ok(l.clone()),
        Some(v) => match v.py_iter_str() {
            Ok(s) => Ok(s.into_iter().map(PyValue::Str).collect()),
            Err(e) => crash(format!("TypeError: {e}")),
        },
    }
}

/// `[f for f in (m.get("field") or []) if isinstance(f, dict)]`.
fn field_dicts(m: &Dict) -> Flow<Vec<Dict>> {
    Ok(field_items(m)?.iter().filter_map(PyValue::as_dict).cloned().collect())
}

/// `check_tile(name, tm, dscp_want)`.
fn check_tile(name: &str, tm: &Dict, want: Option<i128>, ctx: &Ctx, all: &mut Vec<PyValue>, report: &mut Report) -> Flow<()> {
    if !tm.get("layer").is_some_and(|v| v.eq_str("measurement")) {
        report.fail("AC6", format!("{name} in wire.toml must have layer = \"measurement\""));
    }
    if !eq_opt(tm.get("dscp"), want) {
        report.fail("AC6", format!("{name} in wire.toml must have dscp = {} (C-13)", want.map_or("None".into(), |w| w.to_string())));
    }
    let reasons = match tm.get("rejection_reasons") {
        Some(PyValue::List(l)) => l.clone(),
        _ => {
            report.fail("AC6", format!("{name} in wire.toml has no rejection_reasons list"));
            Vec::new()
        }
    };
    for r in reasons {
        hashable(&r)?;
        if !r.as_str().is_some_and(|s| ctx.members.contains(s)) {
            report.fail("AC6", format!("{name} names '{}', which is not a member of the closed enumeration", r.py_str()));
        }
        if !all.iter().any(|x| x.py_eq(&r)) {
            all.push(r);
        }
    }
    Ok(())
}

fn tiles_match(name: &str, tfields: &[Dict], md: &[Field], report: &mut Report) {
    if tfields.len() != md.len() {
        report.fail("AC6", format!("{name}: the markdown table has {} fields, wire.toml {}", md.len(), tfields.len()));
        return;
    }
    for (tf, mf) in tfields.iter().zip(md) {
        let same = eq_opt(tf.get("offset"), Some(mf.off)) && eq_opt(tf.get("width"), Some(mf.w));
        if !same {
            report.fail("AC6", format!("{name}: markdown field '{}' at bit {} ({} bits) does not match wire.toml field '{}'", mf.label, mf.off, mf.w, str_of(tf, "name")));
            return;
        }
        if PRIMITIVE.contains(&mf.ty.as_str()) && mf.ty != text_of(tf, "type").to_lowercase() {
            report.fail("AC6", format!("{name}: markdown type '{}' of '{}' does not match wire.toml type '{}'", mf.ty, mf.label, str_of(tf, "type")));
            return;
        }
    }
}

fn report_message(ctx: &Ctx, t: &Dict, all: &mut Vec<PyValue>, report: &mut Report) -> Flow<()> {
    check_tile("receiver report", t, ctx.k.dscp_report, ctx, all, report)?;
    let tf = field_dicts(t)?;
    if tf.len() != ctx.rep_fix.len() + 1 {
        report.fail("AC6", format!("receiver report in wire.toml: {} fields, expected the {} fixed fields plus one counted member field", tf.len(), ctx.rep_fix.len()));
    } else {
        tiles_match("receiver report", &tf[..tf.len() - 1], &ctx.rep_fix, report);
        let el = &tf[tf.len() - 1];
        if !eq_opt(el.get("width"), ctx.rep_mem_bits) || !eq_opt(el.get("max_count"), ctx.k.members_max) {
            report.fail("AC6", "receiver report in wire.toml: the counted member field must have the per-member width per element and the C-08 max_count");
        }
        let idx = ctx.rep_fix.iter().position(|f| f.lab.contains("member") && f.lab.contains("count"));
        let named = idx.is_some_and(|i| match (el.get("count"), tf[i].get("name")) {
            (None, None) => true,
            (Some(a), Some(b)) => a.py_eq(b),
            _ => false,
        });
        if !named {
            report.fail("AC6", "receiver report in wire.toml: the counted member field's count must name the member count field");
        }
    }
    if let Some(max) = ctx.max_rep()
        && !t.get("max_size").is_some_and(|v| v.eq_int(max))
    {
        report.fail("AC6", format!("receiver report in wire.toml: max_size {} differs from the markdown maximum {max}", str_of(t, "max_size")));
    }
    Ok(())
}

fn probes(ctx: &Ctx, probe_t: &[&Dict], all: &mut Vec<PyValue>, report: &mut Report) -> Flow<()> {
    let by = |p: &str| probe_t.iter().copied().find(|m| has_i(p, &text_of(m, "name")));
    let (mut t_req, t_rep) = (by("request"), by("repl"));
    if probe_t.len() == 1 && t_req.is_none() && t_rep.is_none() {
        t_req = Some(probe_t[0]);
    }
    for (tm, md, bits, label) in [(t_req, &ctx.req_f, ctx.req_bits, "probe request"), (t_rep, &ctx.rep_f, ctx.rpl_bits, "probe reply")] {
        if tm.is_none() && label == "probe reply" && probe_t.len() == 1 {
            continue;
        }
        let Some(tm) = tm.filter(|_| !ctx.prb_place && ctx.prb_ok) else {
            if tm.is_none() && !ctx.prb_place && ctx.prb_ok {
                report.fail("AC6", format!("wire.toml declares no [[message]] for the {label}"));
            }
            continue;
        };
        if label == "probe reply" && md.is_empty() {
            continue;
        }
        check_tile(label, tm, ctx.k.dscp_probe, ctx, all, report)?;
        let tf = field_dicts(tm)?;
        tiles_match(label, &tf, md, report);
        if let Some(b) = bits
            && !tm.get("max_size").is_some_and(|v| v.eq_int(b.div_euclid(8)))
        {
            report.fail("AC6", format!("{label} in wire.toml: max_size {} differs from the markdown size {}", str_of(tm, "max_size"), b.div_euclid(8)));
        }
    }
    Ok(())
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, form_rows: &[Vec<String>], named: &BTreeSet<String>, report: &mut Report) -> Flow<()> {
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
    let rep_t: Vec<&Dict> = dicts.iter().copied().filter(|m| has_i(r"receiver[\s_-]*report", &text_of(m, "name"))).collect();
    // `next(m for m in REP_T if any(... "count" in f ...))`: lazy, so the
    // field values are read up to the first owner
    let mut owner = None;
    for m in &rep_t {
        if field_items(m)?.iter().any(|f| f.as_dict().is_some_and(|d| d.contains_key("count"))) {
            owner = Some(*m);
            break;
        }
    }
    let t_rep = owner.or_else(|| rep_t.first().copied());
    let probe_t: Vec<&Dict> = dicts.iter().copied().filter(|m| has_i("probe", &text_of(m, "name"))).collect();
    if t_rep.is_none() {
        report.fail("AC1", "wire.toml declares no [[message]] named 'receiver report'");
    }
    if probe_t.is_empty() {
        report.fail("AC1", "wire.toml declares no [[message]] for the one-way probe");
    }
    let mut all: Vec<PyValue> = Vec::new();
    if !ctx.rep_place
        && ctx.rep_ok
        && let Some(t) = t_rep
    {
        report_message(ctx, t, &mut all, report)?;
    }
    probes(ctx, &probe_t, &mut all, report)?;
    if form_rows.is_empty() {
        return Ok(());
    }
    for n in named {
        if !all.iter().any(|r| r.eq_str(n)) {
            report.fail("AC6", format!("the measurement messages' rejection_reasons in wire.toml miss '{n}', which an invalid-form row names"));
        }
    }
    // `sorted(ALL_TOML_REASONS)`: a str beside a non-str raises TypeError
    let strs = all.iter().filter(|r| r.as_str().is_some()).count();
    if strs != 0 && strs != all.len() {
        return crash("TypeError: '<' not supported between instances of 'str' and a non-str rejection reason");
    }
    for r in &all {
        if let Some(s) = r.as_str()
            && ctx.members.contains(s)
            && !ctx.layers(s).contains("measurement")
        {
            report.fail("AC6", format!("the enumeration row for '{s}' does not list the measurement layer in its Layers cell, but a measurement message can produce it"));
        }
    }
    Ok(())
}
