//! AC5: invalid forms with outcomes across the two sections (script lines
//! 1223-1315).

use std::collections::{BTreeMap, BTreeSet};

use crate::doccheck::wp7::py;
use super::Ctx;
use crate::doccheck::Report;

const SHORT_FORM: &str = r"too\s+short|shorter\s+than|fewer\s+than|less\s+than|under\s+\d+\s+(?:bytes?|octets?)";

/// 004-20's `NON_REJECTION`.
pub(crate) const NON_REJECTION: &str = r"not\s+a\s+rejection|never\s+a\s+rejection|cannot\s+occur|ignored|is\s+not\s+counted|not\s+counted";

const NEGATED: &str = r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$";

/// `category(form)`.
fn category(form: &str) -> Option<&'static str> {
    let h = |p: &str| py::has_i(p, form);
    Some(if h("dscp|codepoint") {
        "dscp"
    } else if h(SHORT_FORM) {
        "short"
    } else if h(r"\bversion\b") {
        "version"
    } else if h("foreign") {
        "foreign"
    } else if h("member") && h(r"above|more\s+than|exceed|beyond|greater") {
        "members"
    } else if h(r"\blength\b|\bsize\b|inconsistent|disagree") {
        "length"
    } else if h("reply|match|echo") {
        "reply"
    } else if h("bundle") {
        "bundle"
    } else if py::has(r"\bA\b", form) {
        "a"
    } else {
        return None;
    })
}

fn exact(cat: &str) -> Option<&'static str> {
    Some(match cat {
        "version" => "VersionMismatch",
        "foreign" => "ForeignDomain",
        "short" => "TooShort",
        "members" | "length" => "BadLength",
        _ => return None,
    })
}

fn names_member(ctx: &Ctx, outcome: &str) -> bool {
    ctx.members.names().any(|n| py::has(&format!(r"\b{n}\b"), outcome))
}

fn rows_check(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        let (form, outcome) = (&r[0], r[1..].join(" | "));
        let argued = names_member(ctx, &outcome) || py::has_i(NON_REJECTION, &outcome);
        let Some(cat) = category(form) else {
            if !argued {
                report.fail("AC5", format!("the invalid-form row '{}' is outside the ticket's list and its outcome names no member of the closed enumeration nor argues a non-rejection: {}", py::head(form, 60), py::head(&outcome, 90)));
            }
            continue;
        };
        *seen.entry(cat).or_default() += 1;
        if let Some(reason) = exact(cat)
            && !py::has(&format!(r"\b{reason}\b"), &outcome)
        {
            report.fail("AC5", format!("the invalid form '{}' must be counted under {reason} (the ticket names it): {}", py::head(form, 60), py::head(&outcome, 90)));
        }
        if matches!(cat, "bundle" | "a" | "reply") && !argued {
            report.fail("AC5", format!("the invalid form '{}' needs a member reason or an argued non-rejection: {}", py::head(form, 60), py::head(&outcome, 90)));
        }
        if cat == "dscp" && !(py::has_i(NON_REJECTION, &outcome) || py::has("ReservedNonZero", &outcome)) {
            report.fail("AC5", format!("the DSCP-above-63 form must say it cannot occur (6-bit field) or name the padding reason ReservedNonZero: {}", py::head(&outcome, 90)));
        }
    }
    let wanted = [
        ("version", "bad version"),
        ("foreign", "foreign domain"),
        ("short", "too short"),
        ("members", "member count above 32"),
        ("length", "length inconsistent with member count"),
        ("bundle", "unknown bundle"),
        ("dscp", "DSCP above 63"),
        ("a", "A out of its valid range"),
        ("reply", "reply without matching fields"),
    ];
    for (cat, what) in wanted {
        if !seen.contains_key(cat) {
            report.fail("AC5", format!("no invalid form for {what} (the ticket's list)"));
        }
    }
}

/// The member names of the rows, not negated (`NAMED`), and the invented
/// ones reported.
fn tokens(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) -> BTreeSet<String> {
    let mut named = BTreeSet::new();
    for r in rows {
        let outcome = r[1..].join(" | ");
        for h in py::finditer(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b", &outcome, false) {
            let at = py::char_index(&outcome, h.start);
            let before = py::slice(&outcome, at.saturating_sub(25), at);
            let negated = py::has_i(NEGATED, before);
            if ctx.members.contains(h.all()) {
                if !negated {
                    named.insert(h.all().to_string());
                }
            } else if !negated {
                report.fail("AC5", format!("the outcome names '{}', which is not in the closed enumeration", h.all()));
            }
        }
    }
    named
}

/// AC5; returns `FORM_ROWS` and `NAMED`.
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) -> (Vec<Vec<String>>, BTreeSet<String>) {
    let mut rows = Vec::new();
    if !ctx.rep_place {
        rows.extend(ctx.rep.form_rows());
    }
    if !ctx.prb_place {
        rows.extend(ctx.prb.form_rows());
    }
    if ctx.rep_place && ctx.prb_place {
        return (rows, BTreeSet::new());
    }
    if rows.len() < 6 {
        report.fail("AC5", format!("only {} invalid form(s) with outcomes across the two messages; at least 6 required", rows.len()));
    }
    rows_check(&rows, ctx, report);
    let named = tokens(&rows, ctx, report);
    for h in py::finditer(r"\bunder\s+`?([A-Z][A-Za-z]+)`?", &format!("{}\n{}", ctx.rep.text, ctx.prb.text), false) {
        if !ctx.members.contains(h.g(1)) {
            report.fail("AC5", format!("the section counts a rejection under '{}', not a member of the closed enumeration", h.g(1)));
        }
    }
    (rows, named)
}
