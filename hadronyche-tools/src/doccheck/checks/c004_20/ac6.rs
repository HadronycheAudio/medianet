//! AC6 (document part): the measurement messages of wire.toml, read with
//! the scripts' subset reader (script lines 1323-1431; a missing toml
//! report or probe message is reported under AC1, as the script does).

use std::collections::BTreeSet;

use crate::doccheck::wp7::common::{read, sortable, str_of};
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::pytoml::{self, PyDict, PyVal};
use super::{Ctx, Field};
use crate::doccheck::Report;

const PRIMITIVE: [&str; 14] = ["uint", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "q1.31", "pad"];

/// `x == want` for an optional int (`None == None` is true in Python).
fn eq_opt(v: Option<&PyVal>, want: Option<i128>) -> bool {
    match (v, want) {
        (None, None) => true,
        (Some(v), Some(w)) => v.eq_int(w),
        _ => false,
    }
}

/// `check_tile(name, tm, dscp_want)`; `None` after a crash.
fn check_tile(name: &str, tm: &PyDict, want: Option<i128>, ctx: &Ctx, reasons_out: &mut Vec<PyVal>, report: &mut Report) -> Option<()> {
    if !tm.get("layer").is_some_and(|v| v.as_str() == Some("measurement")) {
        report.fail("AC6", format!("{name} in wire.toml must have layer = \"measurement\""));
    }
    if !eq_opt(tm.get("dscp"), want) {
        report.fail("AC6", format!("{name} in wire.toml must have dscp = {} (C-13)", want.map_or("None".into(), |w| w.to_string())));
    }
    let reasons = match tm.get("rejection_reasons") {
        Some(PyVal::List(l)) => l.clone(),
        _ => {
            report.fail("AC6", format!("{name} in wire.toml has no rejection_reasons list"));
            Vec::new()
        }
    };
    for r in reasons {
        if !r.hashable() {
            report.fail("CRASH", "TypeError: unhashable rejection reason");
            return None;
        }
        if !r.as_str().is_some_and(|s| ctx.members.contains(s)) {
            report.fail("AC6", format!("{name} names '{}', which is not a member of the closed enumeration", r.str()));
        }
        if !reasons_out.iter().any(|x| x.eq(&r)) {
            reasons_out.push(r);
        }
    }
    Some(())
}

fn tiles_match(name: &str, tfields: &[&PyDict], md: &[Field], report: &mut Report) {
    if tfields.len() != md.len() {
        report.fail("AC6", format!("{name}: the markdown table has {} fields, wire.toml {}", md.len(), tfields.len()));
        return;
    }
    for (tf, mf) in tfields.iter().zip(md) {
        let same = tf.get("offset").is_some_and(|v| v.eq_int(mf.off)) && tf.get("width").is_some_and(|v| v.eq_int(mf.w));
        if !same {
            report.fail("AC6", format!("{name}: markdown field '{}' at bit {} ({} bits) does not match wire.toml field '{}'", mf.label, mf.off, mf.w, str_of(tf, "name")));
            return;
        }
        if PRIMITIVE.contains(&mf.ty.as_str()) && mf.ty != tf.get("type").map_or(String::new(), PyVal::str).to_lowercase() {
            report.fail("AC6", format!("{name}: markdown type '{}' of '{}' does not match wire.toml type '{}'", mf.ty, mf.label, str_of(tf, "type")));
            return;
        }
    }
}

/// The dicts of `(m.get("field") or [])`; `None` where it is not iterable.
fn field_dicts(m: &PyDict) -> Option<Vec<PyDict>> {
    let v = m.get("field").filter(|v| v.truthy()).cloned().unwrap_or(PyVal::List(Vec::new()));
    Some(v.iter_items()?.iter().filter_map(PyVal::as_dict).cloned().collect())
}

fn report_message(ctx: &Ctx, t: &PyDict, reasons: &mut Vec<PyVal>, report: &mut Report) -> Option<()> {
    check_tile("receiver report", t, ctx.k.dscp_report, ctx, reasons, report)?;
    let Some(tf) = field_dicts(t) else {
        report.fail("CRASH", "TypeError: the receiver report's field value is not iterable");
        return None;
    };
    let mm = ctx.k.members_max;
    if tf.len() != ctx.rep_fix.len() + 1 {
        report.fail("AC6", format!("receiver report in wire.toml: {} fields, expected the {} fixed fields plus one counted member field", tf.len(), ctx.rep_fix.len()));
    } else {
        let refs: Vec<&PyDict> = tf[..tf.len() - 1].iter().collect();
        tiles_match("receiver report", &refs, &ctx.rep_fix, report);
        let el = &tf[tf.len() - 1];
        if !eq_opt(el.get("width"), ctx.rep_mem_bits) || !eq_opt(el.get("max_count"), mm) {
            report.fail("AC6", "receiver report in wire.toml: the counted member field must have the per-member width per element and the C-08 max_count");
        }
        let idx = ctx.rep_fix.iter().position(|f| f.lab.contains("member") && f.lab.contains("count"));
        let named = idx.is_some_and(|i| match (el.get("count"), tf[i].get("name")) {
            (None, None) => true,
            (Some(a), Some(b)) => a.eq(b),
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
    Some(())
}

fn probes(ctx: &Ctx, probe_t: &[&PyDict], reasons: &mut Vec<PyVal>, report: &mut Report) -> Option<()> {
    let by = |p: &str| probe_t.iter().copied().find(|m| py::has_i(p, &str_of(m, "name")));
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
        check_tile(label, tm, ctx.k.dscp_probe, ctx, reasons, report)?;
        let Some(tf) = field_dicts(tm) else {
            report.fail("CRASH", format!("TypeError: the {label}'s field value is not iterable"));
            return None;
        };
        tiles_match(label, &tf.iter().collect::<Vec<_>>(), md, report);
        if let Some(b) = bits
            && !tm.get("max_size").is_some_and(|v| v.eq_int(b.div_euclid(8)))
        {
            report.fail("AC6", format!("{label} in wire.toml: max_size {} differs from the markdown size {}", str_of(tm, "max_size"), b.div_euclid(8)));
        }
    }
    Some(())
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, form_rows: &[Vec<String>], named: &BTreeSet<String>, report: &mut Report) {
    let root = read(ctx.toml_text.as_deref(), "AC6", report);
    let msgs: &[PyVal] = root.as_ref().map_or(&[], |r| pytoml::list_of(r, "message"));
    let dicts: Vec<&PyDict> = msgs.iter().filter_map(PyVal::as_dict).collect();
    let rep_t: Vec<&PyDict> = dicts.iter().copied().filter(|m| py::has_i(r"receiver[\s_-]*report", &str_of(m, "name"))).collect();
    let mut owner = None;
    for m in &rep_t {
        let Some(fs) = field_dicts(m) else {
            report.fail("CRASH", "TypeError: a receiver report's field value is not iterable");
            return;
        };
        if fs.iter().any(|f| f.contains("count")) {
            owner = Some(*m);
            break;
        }
    }
    let t_rep = owner.or_else(|| rep_t.first().copied());
    let probe_t: Vec<&PyDict> = dicts.iter().copied().filter(|m| py::has_i("probe", &str_of(m, "name"))).collect();
    if t_rep.is_none() {
        report.fail("AC1", "wire.toml declares no [[message]] named 'receiver report'");
    }
    if probe_t.is_empty() {
        report.fail("AC1", "wire.toml declares no [[message]] for the one-way probe");
    }
    let mut reasons: Vec<PyVal> = Vec::new();
    if !ctx.rep_place
        && ctx.rep_ok
        && let Some(t) = t_rep
        && report_message(ctx, t, &mut reasons, report).is_none()
    {
        return;
    }
    if probes(ctx, &probe_t, &mut reasons, report).is_none() || form_rows.is_empty() {
        return;
    }
    for n in named {
        if !reasons.iter().any(|r| r.as_str() == Some(n.as_str())) {
            report.fail("AC6", format!("the measurement messages' rejection_reasons in wire.toml miss '{n}', which an invalid-form row names"));
        }
    }
    let refs: Vec<&PyVal> = reasons.iter().collect();
    if !sortable(&refs) {
        report.fail("CRASH", "TypeError: sorted() of str and non-str rejection reasons");
        return;
    }
    for r in &reasons {
        if let Some(s) = r.as_str()
            && ctx.members.contains(s)
            && !ctx.layers(s).contains("measurement")
        {
            report.fail("AC6", format!("the enumeration row for '{s}' does not list the measurement layer in its Layers cell, but a measurement message can produce it"));
        }
    }
}
