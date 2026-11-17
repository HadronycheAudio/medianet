//! AC7 (document part): each admission message and the refusal enumeration
//! in wire.toml, read with the scripts' subset reader (script lines
//! 1191-1268).

use std::collections::BTreeSet;

use crate::doccheck::wp7::common::{md_name, norm_name};
use crate::doccheck::wp7::common::{Num, mul, ne, ne_str, read, set_of, sortable, str_of};
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::pytoml::{self, PyDict, PyVal};
use crate::doccheck::wp7::common::tname;
use super::ac1::TIMER;
use super::ac6::named_in;
use super::{Ctx, Msg};
use crate::doccheck::Report;

const PRIM: [&str; 10] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"];

fn or(v: Option<&PyVal>, default: i128) -> PyVal {
    v.filter(|v| v.truthy()).cloned().unwrap_or(PyVal::Int(default))
}

fn fields(m: &Msg, t: &PyDict, report: &mut Report) -> Option<Num> {
    let name = str_of(t, "name");
    let raw = t.get("field").filter(|v| v.truthy()).cloned().unwrap_or(PyVal::List(Vec::new()));
    let Some(items) = raw.iter_items() else {
        report.fail("CRASH", format!("TypeError: the field value of '{name}' is not iterable"));
        return None;
    };
    let tf: Vec<PyDict> = items.iter().filter_map(PyVal::as_dict).cloned().collect();
    if tf.len() != m.fields.len() {
        report.fail("AC7", format!("toml message '{name}' has {} fields; the markdown table has {} rows", tf.len(), m.fields.len()));
    }
    let mut total = Num::I(0);
    for (mf, tfl) in m.fields.iter().zip(&tf) {
        let nm = md_name(&mf.meaning);
        let tn = str_of(tfl, "name");
        if norm_name(nm) != norm_name(&tn) {
            report.fail("AC7", format!("toml field '{tn}' is not named after the markdown row '{nm}'"));
        }
        if ne(tfl, "offset", mf.off.saturating_mul(8)) {
            report.fail("AC7", format!("toml field {tn}: offset {} bits, markdown byte {}", str_of(tfl, "offset"), mf.off));
        }
        if ne(tfl, "width", mf.bits) {
            report.fail("AC7", format!("toml field {tn}: width {} bits; markdown {} bits", str_of(tfl, "width"), mf.bits));
        }
        if let Some(maxc) = mf.maxc
            && ne(tfl, "max_count", maxc)
        {
            report.fail("AC7", format!("toml field {tn}: max_count {}; markdown says max {maxc}", str_of(tfl, "max_count")));
        }
        if PRIM.contains(&mf.ty.as_str()) && ne_str(tfl, "type", &mf.ty) {
            report.fail("AC7", format!("toml field {tn}: type {}; markdown {}", str_of(tfl, "type"), mf.ty));
        }
        if py::has_i(TIMER, &tn) {
            report.fail("DoD", format!("toml field '{tn}' is named like a timer"));
        }
        match mul(&or(tfl.get("width"), 0), &or(tfl.get("max_count"), 1)).and_then(|v| total.add(&v)) {
            Some(x) => total = x,
            None => {
                report.fail("CRASH", format!("TypeError: toml field {tn} adds a non-number to the size"));
                return None;
            }
        }
    }
    Some(total)
}

fn one(ctx: &Ctx, m: &Msg, t: &PyDict, report: &mut Report) -> bool {
    let name = str_of(t, "name");
    if ne_str(t, "layer", "control") {
        report.fail("AC7", format!("toml message '{name}' has layer '{}', not 'control'", str_of(t, "layer")));
    }
    if let Some(d) = ctx.dscp_control
        && ne(t, "dscp", d)
    {
        report.fail("AC7", format!("toml message '{name}' has dscp {}; C-13 fixes CS5 ({d})", str_of(t, "dscp")));
    }
    if ne(t, "max_size", m.total) {
        report.fail("AC7", format!("toml message '{name}' has max_size {}; the markdown table sums to {}", str_of(t, "max_size"), m.total));
    }
    let Some(total) = fields(m, t, report) else { return false };
    if let Some(s) = t.get("max_size").and_then(PyVal::as_int)
        && total.ne_int(s.saturating_mul(8))
    {
        report.fail("AC7", format!("the toml fields of '{name}' compute a size other than max_size {s}"));
    }
    let need: BTreeSet<String> = ctx
        .shared
        .iter()
        .chain(&m.forms)
        .flat_map(|(_, o)| named_in(o, ctx))
        .filter(|w| ctx.members.contains(w))
        .collect();
    let have = match set_of(t.get("rejection_reasons")) {
        Ok(h) => h,
        Err(e) => {
            report.fail("CRASH", format!("{e} (rejection_reasons of '{name}')"));
            return false;
        }
    };
    for r in &need {
        if !have.iter().any(|h| h.as_str() == Some(r.as_str())) {
            report.fail("AC7", format!("toml message '{name}' rejection_reasons does not list '{r}', which its invalid-form rows name"));
        }
    }
    let extra: Vec<&PyVal> = have.iter().filter(|h| !h.as_str().is_some_and(|s| ctx.members.contains(s))).collect();
    if !sortable(&extra) {
        report.fail("CRASH", "TypeError: sorted() of str and non-str rejection reasons");
        return false;
    }
    for r in extra {
        report.fail("AC7", format!("toml message '{name}' rejection_reasons names '{}', not a member of the closed Reject enumeration (refusal reasons are not packet rejections)", r.str()));
    }
    true
}

/// The refusal enumeration of wire.toml: every list at the top level but
/// `message`, `reject` and `reserved`, its dicts with a str name and an int
/// (or bool) value; a later name overwrites.
fn toml_refusals(root: &PyDict) -> Vec<(String, PyVal)> {
    let mut out: Vec<(String, PyVal)> = Vec::new();
    for (key, val) in &root.0 {
        let PyVal::List(items) = val else { continue };
        if ["message", "reject", "reserved"].contains(&key.as_str()) {
            continue;
        }
        for d in items.iter().filter_map(PyVal::as_dict) {
            let (Some(n), Some(v)) = (d.get("name").and_then(PyVal::as_str), d.get("value")) else { continue };
            if v.as_int().is_none() {
                continue;
            }
            match out.iter_mut().find(|(k, _)| k == n) {
                Some(e) => e.1 = v.clone(),
                None => out.push((n.to_string(), v.clone())),
            }
        }
    }
    out
}

fn refusals(ctx: &Ctx, root: Option<&PyDict>, report: &mut Report) {
    let tref = root.map(toml_refusals).unwrap_or_default();
    let get = |n: &str| tref.iter().find(|(k, _)| k == n).map(|(_, v)| v);
    let equal = tref.len() == ctx.refs.len() && ctx.refs.iter().all(|(n, v, _)| get(n).is_some_and(|x| x.eq_int(*v)));
    if !ctx.refs.is_empty() && !equal {
        report.fail("AC7", format!("wire.toml declares no refusal enumeration equal to the one in wire.md's Admission messages section (toml {}, wire.md {})", tref.len(), ctx.refs.len()));
    }
    for (i, n) in ctx.refusal_set.iter().enumerate() {
        if !get(n).is_some_and(|v| v.eq_int(i as i128)) {
            report.fail("AC7", format!("wire.toml gives refusal reason {n} the value {}; C-185's order gives {i}", get(n).map_or("None".to_string(), PyVal::repr)));
        }
    }
}

/// AC7 (document part).
pub(crate) fn toml(ctx: &Ctx, report: &mut Report) {
    let root = read(ctx.toml_text.as_deref(), "AC7", report);
    if root.is_none() && report.of("CRASH").iter().any(|f| f.message.contains("TOML reader")) {
        return;
    }
    let msgs: &[PyVal] = root.as_ref().map_or(&[], |r| pytoml::list_of(r, "message"));
    for m in &ctx.msgs {
        let want = tname(&m.title);
        let Some(t) = msgs.iter().filter_map(PyVal::as_dict).find(|x| tname(&str_of(x, "name")) == want) else {
            report.fail("AC7", format!("wire.toml declares no [[message]] for the admission message '{}'", m.title));
            continue;
        };
        if !one(ctx, m, t, report) {
            return;
        }
    }
    refusals(ctx, root.as_ref(), report);
}
