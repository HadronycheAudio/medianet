//! AC6 (document part): the hello of wire.toml, read with the scripts'
//! subset reader, against the Markdown layout (script lines 1328-1407).

use crate::doccheck::wp7::common::{Num, get_or, md_name, mul, ne, ne_str, norm_name, read, set_of, sortable, str_of};
use std::collections::BTreeSet;

use super::ac1::Rows;
use crate::doccheck::wp7::md::Field;
use crate::doccheck::wp7::pytoml::{self, PyDict, PyVal};
use super::{Ctx, Layout, py};
use crate::doccheck::Report;

const PRIM: [&str; 10] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"];

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
