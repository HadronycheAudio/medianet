//! AC5: at least 6 invalid forms, exact reasons by the form cell, the size
//! rows by their outcome, every named reason a member (script lines
//! 1016-1113).

use std::collections::BTreeMap;

use crate::doccheck::wp7::py;
use super::Ctx;
use crate::doccheck::Report;

const SHORT_FORM: &str = r"\btoo\s+short\b|\bshorter\s+than\b|\bfewer\s+than\b|\bless\s+than\b|\bunder\s+\d+\s+(?:bytes?|octets?)\b|\bsmaller\s+than\b";
const FIXED_WORD: &str = r"fixed|not\s+equal|equal\s+to|other\s+than";
const SIZE_FORM: &str = r"\bsize\b|\bbytes?\b|\boctets?\b|\blength\b";
const CATS: [(&str, &str, bool); 6] = [
    ("ReservedNonZero", r"\bpad|reserved", true),
    ("BadMagic", "magic", true),
    ("VersionMismatch", "version", true),
    ("ForeignDomain", "foreign", true),
    ("WrongEpoch", "epoch", true),
    ("holder", "holder", false),
];

/// The rows of the first table whose header has a form/invalid cell and an
/// outcome cell (letters only, lower case).
fn form_table(ctx: &Ctx) -> Option<Vec<Vec<String>>> {
    ctx.tables
        .iter()
        .find(|t| {
            let heads: Vec<String> = t.header.iter().map(|h| h.to_lowercase().chars().filter(char::is_ascii_lowercase).collect()).collect();
            heads.iter().any(|h| h.contains("form") || h.contains("invalid")) && heads.iter().any(|h| h.contains("outcome"))
        })
        .map(|t| t.rows.clone())
}

fn reason_tokens(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) {
    for r in rows {
        let outcome = r[1..].join(" | ");
        for h in py::finditer(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b", &outcome, false) {
            let at = py::char_index(&outcome, h.start);
            let before = py::slice(&outcome, at.saturating_sub(25), at);
            if !ctx.members.contains(h.all()) && !py::has_i(r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$", before) {
                report.fail("AC5", format!("the outcome names '{}', which is not in the closed enumeration", h.all()));
            }
        }
    }
}

fn categories(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) {
    let mut seen: BTreeMap<&str, String> = BTreeMap::new();
    let (mut short, mut fixed) = (Vec::new(), Vec::new());
    for r in rows {
        let (form, outcome) = (&r[0], r[1..].join(" | "));
        let Some(cat) = CATS.iter().find(|c| py::has_i(c.1, form)) else {
            if py::has_i(SHORT_FORM, form) && !py::has_i(FIXED_WORD, form) {
                short.push((form, outcome));
            } else if py::has_i(SIZE_FORM, form) {
                fixed.push((form, outcome));
            }
            continue;
        };
        if seen.contains_key(cat.0) {
            continue;
        }
        seen.insert(cat.0, outcome.clone());
        if cat.2 {
            if !py::has(&format!(r"\b{}\b", cat.0), &outcome) {
                report.fail("AC5", format!("the invalid form '{}' must be counted under {} (the ticket names it): {}", py::head(form, 60), cat.0, py::head(&outcome, 90)));
            }
        } else if !ctx.members.names().any(|n| py::has(&format!(r"\b{n}\b"), &outcome)) {
            report.fail("AC5", format!("the invalid form '{}' has no member reason in its outcome", py::head(form, 60)));
        }
    }
    for cat in CATS {
        if cat.0 != "holder" && !seen.contains_key(cat.0) {
            report.fail("AC5", format!("no invalid form for {} (ticket list: size, bad magic, bad version, foreign domain, request padding non-zero, reply from a non-holder, wrong epoch)", cat.0));
        }
        if cat.0 == "holder" && !seen.contains_key("holder") {
            report.fail("AC5", "no invalid form for a reply from a non-holder");
        }
    }
    for (form, outcome) in &short {
        if !py::has(r"\bTooShort\b", outcome) || py::has(r"\bBadLength\b", outcome) {
            report.fail("AC5", format!("the invalid form '{}' is shorter than the minimum that carries the magic and the version: it must be counted under TooShort: {}", py::head(form, 60), py::head(outcome, 90)));
        }
    }
    for (form, outcome) in &fixed {
        if py::has(r"\bTooShort\b", outcome) {
            report.fail("AC5", format!("the invalid form '{}' is a size that is not the fixed size, but it is mapped to TooShort; it must be counted under BadLength (the ticket names it)", py::head(form, 60)));
        }
    }
    if !fixed.iter().any(|(_, o)| py::has(r"\bBadLength\b", o) && !py::has(r"\bTooShort\b", o)) {
        report.fail("AC5", "no invalid form for a size that is not the fixed size counted under BadLength (the ticket names it)");
    }
}

/// AC5; returns the form table's rows (for AC6's exact reasons).
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) -> Option<Vec<Vec<String>>> {
    let table = form_table(ctx);
    if ctx.placeholder {
        return table;
    }
    match &table {
        None => report.fail("AC5", "no table of invalid forms with an outcome column"),
        Some(all) => {
            let rows: Vec<Vec<String>> = all.iter().filter(|r| r.len() >= 2 && !py::strip(&r[0]).is_empty() && !py::strip(&r[1..].join(" ")).is_empty()).cloned().collect();
            if rows.len() < 6 {
                report.fail("AC5", format!("only {} invalid form(s) with outcomes; at least 6 are required", rows.len()));
            }
            categories(&rows, ctx, report);
            reason_tokens(&rows, ctx, report);
        }
    }
    for h in py::finditer(r"\bunder\s+`?([A-Z][A-Za-z]+)`?", &ctx.section, false) {
        if !ctx.members.contains(h.g(1)) {
            report.fail("AC5", format!("the section counts a rejection under '{}', which is not a member of the closed enumeration", h.g(1)));
        }
    }
    table
}
