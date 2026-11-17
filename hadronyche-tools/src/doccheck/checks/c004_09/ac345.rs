//! AC3 (the invalid forms, unknown type skippable, members only), AC4 (C-43
//! and C-44 as sender rules) and AC5 (the open items owned by epic 005, the
//! falsifier, the epoch switch and C-19) of 004-09.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::reasons::{non_members, reasons_in};
use super::wp5::py::{head, s, si};

/// (label, pattern, what the outcome must be).
const FORMS: [(&str, &str, &str); 7] = [
    ("unknown type within the same major (skippable, not a rejection)", r"unknown\s+type", "SKIP"),
    ("length overrun", r"length[^.;\n]{0,60}(?:overrun|exceed|beyond|past|over\s+the)|overrun", "BadLength"),
    ("named position not in the future", r"position[^.;\n]{0,80}(?:not\s+(?:in\s+the\s+)?future|in\s+the\s+past|already\s+passed|past)", "BadLength"),
    ("duplicate type in one packet", "duplicate", "MEMBER"),
    ("announcement in a gap slice (if forbidden)", "gap", "GAP"),
    ("count above maximum", r"count[^.;\n]{0,60}(?:above|exceed|over|greater|more\s+than|larger)", "BadLength"),
    ("announcement-present flag with a zero count", r"zero[\s-]+count|flag[^.;\n]{0,80}\bzero\b", "BadLength"),
];

/// One form: Some(true) when it counts towards the 6.
fn form(ctx: &Ctx, label: &str, pattern: &str, need: &str, report: &mut Report) -> bool {
    let Some(k) = ctx.units.iter().position(|u| si(pattern, u)) else {
        report.fail("AC3", format!("no invalid form listed for: {label}"));
        return false;
    };
    let hit = &ctx.units[k];
    let window = ctx.units[k.saturating_sub(1)..(k + 2).min(ctx.units.len())].join("\n");
    let reasons = reasons_in(&ctx.members, hit);
    match need {
        "SKIP" => {
            if !si("skip", hit) {
                report.fail("AC3", format!("the unknown-type-within-same-major form does not state the skippable outcome (skip by length): {}", head(hit, 110)));
            } else if !reasons.is_empty() {
                report.fail("AC3", format!("the unknown-type-within-same-major form is given a rejection outcome ({}); within the same major an unknown type is SKIPPED by length, not rejected (004-04's minor-change rule): {}", reasons.join(", "), head(hit, 110)));
            }
            false
        }
        "GAP" => {
            let resolved = si(r"cannot|forbidden|impossible|never\s+carries|not\s+possible|no\s+announcement", &format!("{hit}\n{window}"));
            if reasons.is_empty() && !resolved {
                report.fail("AC3", format!("the gap-slice form states no outcome (a member reason, or an explicit statement that the case cannot occur): {}", head(hit, 110)));
            }
            !reasons.is_empty()
        }
        _ => {
            if need != "MEMBER" && !reasons.iter().any(|r| r == need) {
                report.fail("AC3", format!("the invalid form {label:?} must be rejected under {need}: {}", head(hit, 110)));
                return false;
            }
            if reasons.is_empty() {
                report.fail("AC3", format!("the invalid form {label:?} carries no member of the closed enumeration (expected one of {}): {}", ctx.members.join(", "), head(hit, 110)));
                return false;
            }
            true
        }
    }
}

/// AC3.
pub(crate) fn ac3(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let mut counted = 0;
    for (label, pattern, need) in FORMS {
        if form(ctx, label, pattern, need, report) {
            counted += 1;
        }
    }
    let scanned: Vec<String> = ctx.units.iter().chain(ctx.blks.iter()).cloned().collect();
    let outside = non_members(&ctx.members, &scanned);
    if !outside.is_empty() {
        report.fail("AC3", format!("the section names rejection reason(s) outside the closed enumeration: {}", outside.into_iter().collect::<Vec<_>>().join(", ")));
    }
    if counted < 6 {
        report.fail("AC3", format!("only {counted} invalid form(s) carry a member rejection reason or a stated resolution; at least 6 are required"));
    }
}

/// The window (block before, block, block after) of the first block holding
/// `cid` and the number `n`.
fn restatement(ctx: &Ctx, cid: &str, n: &str) -> Option<String> {
    let k = ctx.blks.iter().position(|b| s(cid, b) && s(&format!(r"\b{n}\b"), b))?;
    Some(ctx.blks[k.saturating_sub(1)..(k + 2).min(ctx.blks.len())].join("\n"))
}

/// AC4.
pub(crate) fn ac4(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    match restatement(ctx, "C-43", "64") {
        None => report.fail("AC4", "C-43 is not restated (the lead: 64 slices, flat)"),
        Some(w) => {
            if !s(r"\b32\b", &w) || !si("ms|millisecond", &w) {
                report.fail("AC4", "the C-43 restatement does not show the arithmetic (64 slices x 0.5 ms = 32 ms at the C-02 defaults)");
            }
            if !si("sender|senders|sender's", &w) {
                report.fail("AC4", "the C-43 restatement is not phrased as a sender rule (obligation of the sender)");
            }
        }
    }
    match restatement(ctx, "C-44", "16") {
        None => report.fail("AC4", "C-44 is not restated (re-issued every 16 slices until the position passes)"),
        Some(w) => {
            if !si("member", &w) {
                report.fail("AC4", "the C-44 restatement does not name the every-member rule (re-issued in every bundle member, C-08)");
            }
            if !si("until|passes", &w) {
                report.fail("AC4", "the C-44 restatement does not state the duration (until the named position passes)");
            }
            if !si("sender|senders|sender's", &w) {
                report.fail("AC4", "the C-44 restatement is not phrased as a sender rule (obligation of the sender)");
            }
        }
    }
}

/// AC5.
pub(crate) fn ac5(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let b = &ctx.blks;
    for (label, pattern) in [("re-alignment", r"re[\s_-]*alignment"), ("transport reference change", r"transport[\s_-]*reference")] {
        let ok = (0..b.len()).filter(|&k| si(pattern, &b[k])).any(|k| {
            let w = b[k.saturating_sub(1)..(k + 2).min(b.len())].join("\n");
            si(r"open\s+item", &format!("{}\n{w}", b[k])) && s("005", &w) && si("owner|confirm", &w)
        });
        if !ok {
            report.fail("AC5", format!("the proposed {label} payload is not recorded as an open item with epic 005 as the confirming owner"));
        }
    }
    if !si("falsif", &ctx.prose) {
        report.fail("AC5", "the open items state no falsifier");
    } else if !si(r"cannot\s+act[^.;\n]{0,80}payload|payload\s+alone", &ctx.prose) {
        report.fail("AC5", "the falsifier is not the ticket's (a receiver cannot act on the announcement from its payload alone)");
    }
    let epoch = (0..b.len()).filter(|&k| si(r"epoch[\s_-]*switch|epoch", &b[k])).any(|k| {
        let w = format!("{}\n{}", b[k], b.get(k + 1).map_or("", String::as_str));
        s("C-19", &w) && si("session", &w) && si(r"names?\s+the\s+new\s+session|does\s+not\s+name|also\s+bumps?|bumps?\s+the\s+session", &w)
    });
    if !epoch {
        report.fail("AC5", "the epoch-switch payload does not state whether it names the new session (C-19: a timeline epoch change also bumps the session of every stream)");
    }
}
