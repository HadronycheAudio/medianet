//! AC4: at least ten invalid forms with outcomes, the exact reasons of the
//! ticket's list, member reasons or argued non-rejections, no invented
//! reason (script lines 1156-1265).

use std::collections::BTreeSet;

use super::Ctx;
use crate::doccheck::Report;
use crate::doccheck::family004::tables;
use crate::doccheck::rejection_members::Enumeration;
use crate::doccheck::rejection_members::reasons::{CAMEL_LETTERS, LOOKBACK_004_16, NON_REJECTION_004_16, named_reasons};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has, has_i, head, rx};

fn named(outcome: &str, members: &Enumeration) -> Vec<String> {
    named_reasons(outcome, |w| members.contains(w), LOOKBACK_004_16)
}

/// The (form, outcome) rows of the first invalid-form table.
fn form_rows(ctx: &Ctx, report: &mut Report) -> Flow<Vec<(String, String)>> {
    let mut table = None;
    for (_, x) in ctx.subs.iter().filter(|(t, _)| has_i("invalid|validity", t)) {
        table = tables(x).into_iter().find(|tb| {
            let h: Vec<String> = tb.head().iter().map(|c| c.to_lowercase()).collect();
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
            // `next(...)` raises StopIteration when no header matches.
            let (Some(fi), Some(oi)) = (
                t.head().iter().position(|c| has_i("form|invalid", c)),
                t.head().iter().position(|c| has_i("outcome", c)),
            ) else {
                return crash("StopIteration: no invalid-form header cell matches form|invalid or outcome");
            };
            for r in t.rows.iter().map(|r| &r.cells) {
                if r.len() > fi.max(oi) {
                    rows.push((r[fi].clone(), r[oi].clone()));
                }
            }
        }
    }
    if rows.len() < 10 {
        report.fail("AC4", format!("{} invalid form(s) with outcomes; at least 10 are required", rows.len()));
    }
    Ok(rows)
}

/// `cat_of(form)`.
pub(crate) fn cat_of(form: &str) -> Option<&'static str> {
    let f = form.to_lowercase();
    let has = |p: &str| has(p, &f);
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
    for h in rx(CAMEL_LETTERS).find_iter(outcome) {
        let name = h.as_str();
        if !members.contains(name) && !has(&format!(r"\b(not|never|nor)\b[^|;.]*{name}"), outcome) {
            report.fail("AC4", format!("the outcome '{}' names '{name}', which is not in the closed enumeration", head(outcome, 60)));
        }
    }
    let cat = cat_of(form);
    let argued = || !named(outcome, members).is_empty() || NON_REJECTION_004_16.is_match(outcome);
    if let Some(reason) = cat.and_then(exact) {
        seen.insert(cat.unwrap_or_default());
        if !has(&format!(r"\b{reason}\b"), outcome) {
            report.fail("AC4", format!("the form '{}' must be counted under {reason}: {}", head(form, 50), head(outcome, 70)));
        }
        if cat == Some("domain0") && named(outcome, members).iter().any(|n| n == "ForeignDomain") {
            report.fail("AC4", "domain 0 is mapped to ForeignDomain; it is ignored and counted as UnconfiguredDomain (C-173)");
        }
        if cat == Some("version") && !has_i(r"\bevent\b", outcome) {
            report.fail("AC4", "the version mismatch row does not say that it raises a named event");
        }
    } else if !argued() {
        report.fail("AC4", format!("the form '{}' names no member of the closed enumeration and argues no non-rejection", head(form, 50)));
    }
    if matches!(cat, Some("collision" | "counter" | "name")) && !argued() {
        report.fail("AC4", format!("the form '{}' needs a member reason or an argued non-rejection", head(form, 50)));
    }
}

/// AC4; returns the reasons the rows name (`EXACT_REASONS`).
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) -> Flow<BTreeSet<String>> {
    let rows = form_rows(ctx, report)?;
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
    Ok(rows.iter().flat_map(|(_, o)| named(o, &ctx.members)).collect())
}
