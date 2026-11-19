//! AC5: invalid forms with outcomes across the two sections (script lines
//! 1223-1315).

use std::collections::{BTreeMap, BTreeSet};

use super::Ctx;
use crate::doccheck::Report;
use crate::doccheck::rejection_members::reasons::{CAMEL_LETTERS, LOOKBACK_004_19, NON_REJECTION_004_20, negated_at};
use crate::doccheck::text::{has, has_i, head, rx};

const SHORT_FORM: &str = r"too\s+short|shorter\s+than|fewer\s+than|less\s+than|under\s+\d+\s+(?:bytes?|octets?)";

/// `category(form)`.
fn category(form: &str) -> Option<&'static str> {
    let h = |p: &str| has_i(p, form);
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
    } else if has(r"\bA\b", form) {
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
    ctx.members.names().any(|n| has(&format!(r"\b{n}\b"), outcome))
}

fn rows_check(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        let (form, outcome) = (&r[0], r[1..].join(" | "));
        let argued = names_member(ctx, &outcome) || NON_REJECTION_004_20.is_match(&outcome);
        let Some(cat) = category(form) else {
            if !argued {
                report.fail("AC5", format!("the invalid-form row '{}' is outside the ticket's list and its outcome names no member of the closed enumeration nor argues a non-rejection: {}", head(form, 60), head(&outcome, 90)));
            }
            continue;
        };
        *seen.entry(cat).or_default() += 1;
        if let Some(reason) = exact(cat)
            && !has(&format!(r"\b{reason}\b"), &outcome)
        {
            report.fail("AC5", format!("the invalid form '{}' must be counted under {reason} (the ticket names it): {}", head(form, 60), head(&outcome, 90)));
        }
        if matches!(cat, "bundle" | "a" | "reply") && !argued {
            report.fail("AC5", format!("the invalid form '{}' needs a member reason or an argued non-rejection: {}", head(form, 60), head(&outcome, 90)));
        }
        if cat == "dscp" && !(NON_REJECTION_004_20.is_match(&outcome) || has("ReservedNonZero", &outcome)) {
            report.fail("AC5", format!("the DSCP-above-63 form must say it cannot occur (6-bit field) or name the padding reason ReservedNonZero: {}", head(&outcome, 90)));
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
        for h in rx(CAMEL_LETTERS).find_iter(&outcome) {
            let negated = negated_at(&outcome, outcome[..h.start()].chars().count(), LOOKBACK_004_19);
            if ctx.members.contains(h.as_str()) {
                if !negated {
                    named.insert(h.as_str().to_string());
                }
            } else if !negated {
                report.fail("AC5", format!("the outcome names '{}', which is not in the closed enumeration", h.as_str()));
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
    for h in rx(r"\bunder\s+`?([A-Z][A-Za-z]+)`?").captures_iter(&format!("{}\n{}", ctx.rep.text, ctx.prb.text)) {
        if !ctx.members.contains(h.get(1).map_or("", |m| m.as_str())) {
            report.fail("AC5", format!("the section counts a rejection under '{}', not a member of the closed enumeration", h.get(1).map_or("", |m| m.as_str())));
        }
    }
    (rows, named)
}
