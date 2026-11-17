//! AC4: at least ten invalid forms with outcomes, the exact reasons of the
//! ticket's list, member reasons or argued non-rejections, no invented
//! reason (script lines 1156-1265).

use crate::doccheck::wp7::common::named_reasons;
use std::collections::BTreeSet;

use crate::doccheck::wp7::md::parse_tables;
use super::{Ctx, py};
use crate::doccheck::Report;
use crate::doccheck::wp7::rejection_members::Enumeration;

/// 004-16's `NON_REJECTION` (it differs in each script).
pub(crate) const NON_REJECTION: &str = r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|cannot\s+occur|ignored";

/// 004-16's reason tokens (`CAMEL`).
pub(crate) const CAMEL: &str = r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b";

/// The (form, outcome) rows of the first invalid-form table.
fn form_rows(ctx: &Ctx, report: &mut Report) -> Vec<(String, String)> {
    let mut table = None;
    for (_, x) in ctx.subs.iter().filter(|(t, _)| py::has_i("invalid|validity", t)) {
        table = parse_tables(x).into_iter().find(|tb| {
            let h: Vec<String> = tb.header.iter().map(|c| c.to_lowercase()).collect();
            h.iter().any(|c| c.contains("form") || c.contains("invalid")) && h.iter().any(|c| c.contains("outcome"))
        });
        if table.is_some() {
            break;
        }
    }
    let mut rows = Vec::new();
    match table {
        None => report.fail("AC4", "no invalid-form table with 'Form' and 'Outcome' columns in the Hello section"),
        Some(t) => {
            let fi = t.header.iter().position(|c| py::has_i("form|invalid", c)).unwrap_or(0);
            let oi = t.header.iter().position(|c| py::has_i("outcome", c)).unwrap_or(0);
            for r in &t.rows {
                if r.len() > fi.max(oi) {
                    rows.push((r[fi].clone(), r[oi].clone()));
                }
            }
        }
    }
    if rows.len() < 10 {
        report.fail("AC4", format!("{} invalid form(s) with outcomes; at least 10 are required", rows.len()));
    }
    rows
}

/// `cat_of(form)`.
pub(crate) fn cat_of(form: &str) -> Option<&'static str> {
    let f = form.to_lowercase();
    let has = |p: &str| py::has(p, &f);
    if has(r"too\s+short|shorter|fewer\s+than|less\s+than\s+\d+\s*bytes") {
        Some("short")
    } else if f.contains("magic") {
        Some("magic")
    } else if f.contains("version") {
        Some("version")
    } else if has(r"collision|same\s+module\s+id") {
        Some("collision")
    } else if has(r"domain\s*(id\s*)?0\b|unconfigured") {
        Some("domain0")
    } else if has(r"foreign|different\s+non-?zero") {
        Some("foreign")
    } else if has(r"oversiz|larger\s+than|above\s+1") {
        Some("oversize")
    } else if has(r"(count|member|bundle|stream)[^|]*(above|exceed|more\s+than|beyond)") {
        Some("counts")
    } else if has(r"length|size") {
        Some("length")
    } else if has("counter") {
        Some("counter")
    } else if has(r"\bname\b|utf|encoding") {
        Some("name")
    } else if has(r"reserved|non-?zero|scheme|\btag\b") {
        Some("reserved")
    } else {
        None
    }
}

fn exact(cat: &str) -> Option<&'static str> {
    Some(match cat {
        "short" => "TooShort",
        "magic" => "BadMagic",
        "version" => "VersionMismatch",
        "domain0" => "UnconfiguredDomain",
        "foreign" => "ForeignDomain",
        "oversize" => "OversizeControl",
        "counts" | "length" => "BadLength",
        "reserved" => "ReservedNonZero",
        _ => return None,
    })
}

fn one_row(form: &str, outcome: &str, members: &Enumeration, seen: &mut BTreeSet<&'static str>, report: &mut Report) {
    for h in py::finditer(CAMEL, outcome, false) {
        let name = h.all();
        if !members.contains(name) && !py::has(&format!(r"\b(not|never|nor)\b[^|;.]*{name}"), outcome) {
            report.fail("AC4", format!("the outcome '{}' names '{name}', which is not in the closed enumeration", py::head(outcome, 60)));
        }
    }
    let cat = cat_of(form);
    let argued = || !named_reasons(outcome, members).is_empty() || py::has_i(NON_REJECTION, outcome);
    if let Some(reason) = cat.and_then(exact) {
        seen.insert(cat.unwrap_or_default());
        if !py::has(&format!(r"\b{reason}\b"), outcome) {
            report.fail("AC4", format!("the form '{}' must be counted under {reason}: {}", py::head(form, 50), py::head(outcome, 70)));
        }
        if cat == Some("domain0") && named_reasons(outcome, members).iter().any(|n| n == "ForeignDomain") {
            report.fail("AC4", "domain 0 is mapped to ForeignDomain; it is ignored and counted as UnconfiguredDomain (C-173)");
        }
        if cat == Some("version") && !py::has_i(r"\bevent\b", outcome) {
            report.fail("AC4", "the version mismatch row does not say that it raises a named event");
        }
    } else if !argued() {
        report.fail("AC4", format!("the form '{}' names no member of the closed enumeration and argues no non-rejection", py::head(form, 50)));
    }
    if matches!(cat, Some("collision" | "counter" | "name")) && !argued() {
        report.fail("AC4", format!("the form '{}' needs a member reason or an argued non-rejection", py::head(form, 50)));
    }
}

/// AC4; returns the reasons the rows name (`EXACT_REASONS`).
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) -> BTreeSet<String> {
    let rows = form_rows(ctx, report);
    let mut seen = BTreeSet::new();
    for (form, outcome) in &rows {
        one_row(form, outcome, &ctx.members, &mut seen, report);
    }
    for cat in ["short", "magic", "version", "domain0", "foreign", "oversize", "counts", "length", "reserved"] {
        if !seen.contains(cat) {
            report.fail("AC4", format!("no invalid-form row covers the '{cat}' case of the ticket's list"));
        }
    }
    let any_cat = |c: &str| rows.iter().any(|(f, _)| cat_of(f) == Some(c));
    if !any_cat("counter") {
        report.fail("AC4", "no invalid-form row covers a liveness counter going backwards for the same instance nonce");
    }
    if !any_cat("name") {
        report.fail("AC4", "no invalid-form row covers a name that is not valid in the chosen encoding");
    }
    let collision_para = ctx.para_with(&["collision", r"not\s+a\s+parse\s+rejection"]).is_some();
    if !any_cat("collision") && !collision_para {
        report.fail("AC4", "the module-id collision (C-46) is not stated as a non-rejection");
    }
    if !collision_para {
        report.fail("AC4", "no sentence says that a module-id collision (same module id, different instance nonce, C-46) is not a parse rejection (state-machine behaviour, epic 005)");
    }
    rows.iter().flat_map(|(_, o)| named_reasons(o, &ctx.members)).collect()
}
