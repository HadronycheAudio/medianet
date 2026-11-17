//! AC6 (document part): each election message declared in wire.toml, read
//! with the scripts' subset reader (script lines 1066-1131).

use crate::doccheck::wp7::common::tname;
use std::collections::BTreeSet;

use crate::doccheck::wp7::common::{md_name, norm_name};
use crate::doccheck::wp7::common::named_reasons;
use crate::doccheck::wp7::common::{Num, mul, ne, ne_str, read, set_of, sortable, str_of};
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::pytoml::{self, PyDict, PyVal};
use super::ac5::TIMER;
use super::{Ctx, Msg};
use crate::doccheck::Report;

const PRIM: [&str; 10] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"];

/// `x or default` for a looked-up value.
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
        report.fail("AC6", format!("toml message '{name}' has {} fields; the markdown table has {} rows", tf.len(), m.fields.len()));
    }
    let mut total = Num::I(0);
    for (mf, tfl) in m.fields.iter().zip(&tf) {
        let nm = md_name(&mf.meaning);
        let tn = str_of(tfl, "name");
        if norm_name(nm) != norm_name(&tn) {
            report.fail("AC6", format!("toml field '{tn}' is not named after the markdown row '{nm}' (names are compared lower case, spaces and underscores equivalent)"));
        }
        if ne(tfl, "offset", mf.off.saturating_mul(8)) {
            report.fail("AC6", format!("toml field {tn}: offset {} bits, markdown byte {} (= {} bits)", str_of(tfl, "offset"), mf.off, mf.off * 8));
        }
        if ne(tfl, "width", mf.bits) {
            report.fail("AC6", format!("toml field {tn}: width {} bits; markdown {} bits", str_of(tfl, "width"), mf.bits));
        }
        if let Some(maxc) = mf.maxc
            && ne(tfl, "max_count", maxc)
        {
            report.fail("AC6", format!("toml field {tn}: max_count {}; markdown says max {maxc}", str_of(tfl, "max_count")));
        }
        if PRIM.contains(&mf.ty.as_str()) && ne_str(tfl, "type", &mf.ty) {
            report.fail("AC6", format!("toml field {tn}: type {}; markdown {}", str_of(tfl, "type"), mf.ty));
        }
        if py::has_i(TIMER, &tn) {
            report.fail("Behaviour", format!("toml field '{tn}' is named like a timer"));
        }
        match mul(&or(tfl.get("width"), 0), &or(tfl.get("max_count"), 1)).and_then(|v| total.add(&v)) {
            Some(t) => total = t,
            None => {
                report.fail("CRASH", format!("TypeError: toml field {tn} adds a non-number to the size"));
                return None;
            }
        }
    }
    Some(total)
}

fn reasons(ctx: &Ctx, m: &Msg, t: &PyDict, report: &mut Report) {
    let name = str_of(t, "name");
    let need: BTreeSet<String> = ctx.shared.iter().chain(&m.forms).flat_map(|(_, o)| named_reasons(o, &ctx.members)).collect();
    let have = match set_of(t.get("rejection_reasons")) {
        Ok(h) => h,
        Err(e) => {
            report.fail("CRASH", format!("{e} (rejection_reasons of '{name}')"));
            return;
        }
    };
    for r in &need {
        if !have.iter().any(|h| h.as_str() == Some(r.as_str())) {
            report.fail("AC6", format!("toml message '{name}' rejection_reasons does not list '{r}', which its invalid-form rows name"));
        }
    }
    let extra: Vec<&PyVal> = have.iter().filter(|h| !h.as_str().is_some_and(|s| ctx.members.contains(s))).collect();
    if !sortable(&extra) {
        report.fail("CRASH", "TypeError: sorted() of str and non-str rejection reasons");
        return;
    }
    for r in extra {
        report.fail("AC6", format!("toml message '{name}' rejection_reasons names '{}', not in the closed enumeration", r.str()));
    }
}

fn one(ctx: &Ctx, m: &Msg, t: &PyDict, report: &mut Report) -> bool {
    let name = str_of(t, "name");
    if ne_str(t, "layer", "control") {
        report.fail("AC6", format!("toml message '{name}' has layer '{}', not 'control'", str_of(t, "layer")));
    }
    if let Some(d) = ctx.dscp_control
        && ne(t, "dscp", d)
    {
        report.fail("AC6", format!("toml message '{name}' has dscp {}; C-13 fixes CS5 ({d})", str_of(t, "dscp")));
    }
    if ne(t, "max_size", m.total) {
        report.fail("AC6", format!("toml message '{name}' has max_size {}; the markdown table sums to {}", str_of(t, "max_size"), m.total));
    }
    let max_size = t.get("max_size").and_then(PyVal::as_int);
    if ctx.bound != 0
        && let Some(s) = max_size
        && s > ctx.bound
    {
        report.fail("AC6", format!("toml message '{name}' max_size {s} exceeds the {}-byte bound (C-17)", ctx.bound));
    }
    let Some(total) = fields(m, t, report) else { return false };
    if let Some(s) = max_size
        && total.ne_int(s.saturating_mul(8))
    {
        report.fail("AC6", format!("the toml fields of '{name}' compute a size other than max_size {s}"));
    }
    reasons(ctx, m, t, report);
    true
}

/// AC6 (document part).
pub(crate) fn toml(ctx: &Ctx, report: &mut Report) {
    let root = read(ctx.toml_text.as_deref(), "AC6", report);
    let msgs: &[PyVal] = root.as_ref().map_or(&[], |r| pytoml::list_of(r, "message"));
    for m in &ctx.msgs {
        let want = tname(&m.title);
        let Some(t) = msgs.iter().filter_map(PyVal::as_dict).find(|x| tname(&str_of(x, "name")) == want) else {
            report.fail("AC6", format!("wire.toml declares no [[message]] for the election message '{}'", m.title));
            continue;
        };
        if !one(ctx, m, t, report) {
            return;
        }
    }
}
