//! The DEFECT detector of 004-09 (the amendment): the lead (C-43) is a
//! sender obligation on the FIRST issue of a change; a re-issue (C-44)
//! inside the lead is valid; a receiver rejects only a position that is not
//! in the future. Keyword detectors over the section's table rows and cells,
//! prose sentences, list items and wire.toml field strings, each branch as
//! the script wrote it (004-09:1482-1770). Text is shadowed and `İ`/`ı`
//! folded before matching; spans are byte offsets in that text.

use super::toml_side::AnnField;
use super::{Ctx, is_table_block, prose_sentences, split_sentences};
use crate::doccheck::Report;
use super::wp5::md::is_field_table;
use super::wp5::py::{fold_i, head, py_strip, shadow};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

const LEAD: &str = r"(?i)\blead|C-43|closer|nearer|inside|within|less\s+than\s+a\s+lead";
const EXEMPT: &str = r"(?i)\bfirst\b|re-?issu|C-44";
const NEG: &str = r"(?i)\b(?:not|never|no|nor|cannot|neither|none)\b|n't";
const REJECT: &str = r"(?i)BadLength|\bdrops?\b|\breject|\bdisagree|\binvalid|\bmalformed|\bcount\s+under";
const LEAD_FORM: &str = r"(?i)\blead\b|C-43|\bcloser\b|\bnearer\b|64\s+slices|within\s+the\s+lead|inside\s+the\s+lead|less\s+than\s+a\s+lead";
const EVERY_1: &str = r"(?i)\b(?:lead|C-43)\b[^.;]{0,80}\bbinds?\b[^.;]{0,40}\b(?:every|each|all|any)\s+(?:re-?)?issues?\b";
const EVERY_2: &str = r"(?i)\b(?:every|each|all|any)\s+(?:re-?)?issues?\b[^.;]{0,40}\b(?:bind|binds|bound|applies|apply|governs?)\b[^.;]{0,40}\b(?:lead|C-43)\b";
const R6_STOP: &str = r"(?i)\b(?:no|nothing|none)\b[^.;]{0,80}\b(?:rides?|carr(?:y|ies|ied)|re-?issued?|sent|issued|goes?\s+out|travels?)\b[^.;]{0,80}\bslices?\b[^.;]{0,60}\b(?:at\s+or\s+after|at\s+or\s+beyond|on\s+or\s+after)\b";
const R6_REACH: &str = r"(?i)\bslice\s+position\s+reach(?:es)?\b|\breach(?:es)?\s+(?:it|the\s+named\s+position|the\s+position)\b";
const IN_SLICE_AT_P: &str = r"(?i)\bslices?\b[^.;]{0,40}?\b(?:whose\s+slice\s+position\s+(?:is|equals)|at|equal\s+to|equals)\s+(?:the\s+)?(?:named\s+position|position\s+it\s+names)\b";
const LEAD_SPAN: &str = r"(?:(?:inside|within)\s+(?:the|a|one|its)\s+lead\b|(?:less\s+than\s+(?:64\s+slices|(?:one|a|the)\s+lead)|closer\s+than\s+(?:the|a|one)\s+lead)\b)";
const REJECT_VERBS: [&str; 10] = [
    r"(?i)\breject\w*",
    r"(?i)\bdrop\w*",
    r"(?i)\bdiscard\w*",
    r"(?i)\bcount(?:s|ed|ing)?\s+(?:\S+\s+){0,3}?under\b",
    r"\bBadLength\b",
    r"(?i)\bpolic\w*",
    r"(?i)\benforc\w*",
    r"(?i)\bcheck(?:s|ed|ing)?\b",
    r"(?i)\bverif\w*",
    r"(?i)\brequir(?:e|es|ed|ing)\b",
];
const LEAD_PHRASES: [&str; 4] = [r"(?i)\blead\b", r"(?i)\bcloser\s+than\b", r"(?i)\bless\s+than\s+(?:one\s+lead|a\s+lead|the\s+lead|64\s+slices)\b", r"\bC-43\b"];

fn has(p: &str, t: &str) -> bool {
    rx(p).is_match(t)
}

/// `clause_around(s, a, b)`: the text between `,`, `;`, `:` around a..b.
fn clause_around(s: &str, a: usize, b: usize) -> &str {
    let lo = [',', ';', ':'].iter().filter_map(|&c| s[..a].rfind(c)).max().map_or(0, |i| i + 1);
    let hi = [',', ';', ':'].iter().filter_map(|&c| s[b..].find(c).map(|i| i + b)).min().unwrap_or(s.len());
    if lo <= hi { &s[lo..hi] } else { "" }
}

/// Some match of `pattern` whose clause holds no negation.
fn unnegated(pattern: &str, s: &str) -> bool {
    rx(pattern).find_iter(s).any(|m| !has(NEG, clause_around(s, m.start(), m.end())))
}

/// `lead_as_rejection(seg, cell)` on folded, shadowed text.
fn lead_as_rejection(seg: &str, cell: bool) -> bool {
    if !has(LEAD, seg) || has(EXEMPT, seg) || has(NEG, seg) {
        return false;
    }
    if has(REJECT, seg) {
        return true;
    }
    let (valid, at_least) = (has(r"(?i)\bvalid\b", seg), has(r"(?i)\bat\s+least\b", seg));
    if cell { valid || at_least } else { valid && at_least }
}

/// `every_issue_violation(s)`.
fn every_issue(s: &str) -> bool {
    [EVERY_1, EVERY_2].iter().any(|p| {
        rx(p).find_iter(s).any(|m| match rx(r"(?i)\b(?:every|each|all|any)\b").find(m.as_str()) {
            None => true,
            Some(q) => {
                let qa = m.start() + q.start();
                !has(NEG, clause_around(s, qa, qa + 1))
            }
        })
    })
}

/// `receiver_polices_lead(s)`.
fn receiver_polices(s: &str) -> bool {
    has(r"(?i)\breceivers?\b|\bconsumers?\b", s) && REJECT_VERBS.iter().any(|p| unnegated(p, s)) && LEAD_PHRASES.iter().any(|p| unnegated(p, s))
}

/// The table rows and cells.
fn tables(ctx: &Ctx, report: &mut Report) {
    for (_, header, rows) in &ctx.tables {
        for row in rows {
            let cells: Vec<String> = row.iter().map(|c| fold_i(c).into_owned()).collect();
            if is_field_table(header) {
                for (idx, cell) in cells.iter().enumerate() {
                    if lead_as_rejection(cell, idx == 4) {
                        report.fail("defect", format!("a field-table cell makes the lead a validity condition: {}", head(cell, 110)));
                    }
                }
                continue;
            }
            let joined = cells.join(" | ");
            if cells.len() >= 2 && has(REJECT, &cells[1..].join(" | ")) && unnegated(LEAD_FORM, &cells[0]) {
                report.fail("defect", format!("a Validity-rules row rejects an announcement for sitting inside the lead (no exemption): {}", head(&joined, 110)));
            }
            if lead_as_rejection(&joined, false) {
                report.fail("defect", format!("a table row rejects or invalidates a position for the lead: {}", head(&joined, 110)));
            }
        }
    }
}

/// The prose sentences, list items and wire.toml strings.
fn segments(ctx: &Ctx, fields: &[AnnField], report: &mut Report) {
    for b in ctx.blks.iter().filter(|b| !is_table_block(b)) {
        for sent in split_sentences(b) {
            if lead_as_rejection(&fold_i(&sent), false) {
                report.fail("defect", format!("prose makes the lead a validity condition: {}", head(py_strip(&sent), 110)));
            }
        }
    }
    for line in splitlines(&ctx.prose) {
        let st = py_strip(line);
        if (has(r"^[-*]\s+", st) || has(r"^\d+[.)]\s+", st)) && lead_as_rejection(&fold_i(st), false) {
            report.fail("defect", format!("a list item makes the lead a validity condition: {}", head(st, 110)));
        }
    }
    for (_, f) in fields {
        for key in ["valid", "meaning"] {
            if let Some(txt) = f.get(key).and_then(|v| v.as_str())
                && lead_as_rejection(&fold_i(&shadow(txt)), key == "valid")
            {
                report.fail("defect", format!("wire.toml field ({key}) makes the lead a validity condition: {}", head(txt, 110)));
            }
        }
    }
}

/// The sentence rules: a re-issue inside the lead is valid; the lead binds
/// the first issue only; no receiver polices it; R6.
fn sentences(ctx: &Ctx, report: &mut Report) {
    let all = prose_sentences(ctx, None);
    let reissue = [
        format!(r"(?i)re-?issu\w*[^.;]{{0,80}}?\b{LEAD_SPAN}[^.;]{{0,60}}?\b(?:valid|accepted)\b"),
        format!(r"(?i)re-?issu\w*[^.;]{{0,60}}?\b(?:valid|accepted)\b[^.;]{{0,60}}?{LEAD_SPAN}"),
    ];
    if !all.iter().any(|s| reissue.iter().any(|p| has(p, s))) {
        report.fail("defect", "no statement that a re-issue (C-44) inside the lead is valid (the sentence must name the lead itself); the receiver rejects only a position that is not in the future");
    }
    if let Some(s) = all.iter().find(|s| every_issue(s)) {
        report.fail("defect", format!("the lead is stated to bind every issue of a change (C-43 binds the sender's FIRST issue only): {}", head(s, 110)));
    }
    if let Some(s) = all.iter().find(|s| receiver_polices(s)) {
        report.fail("defect", format!("a receiver is stated to police or reject for the lead (receivers never do; they reject only a position that is not in the future): {}", head(s, 110)));
    }
    let sender = prose_sentences(ctx, Some(r"^\s*\**Sender obligations"));
    if let Some(s) = sender.iter().find(|s| unnegated(IN_SLICE_AT_P, s)) {
        report.fail("defect R6", format!("a sender sentence places an announcement in the slice at the named position (no announcement rides in a slice at or after it): {}", head(s, 110)));
    }
    let r6 = sender
        .iter()
        .filter(|s| has(r"(?i)sender|re-?issu|carr|ride", s))
        .any(|s| has(R6_STOP, s) || (has(R6_REACH, s) && has(r"(?i)\bpass(?:es|ed)?\b", s)));
    if !r6 {
        report.fail("defect R6", "the Sender obligations text does not state the stop rule at the named position: the position passes once the sender's slice position reaches it, so no announcement rides in a slice at or after the position it names");
    }
    let first = ctx.blks.iter().filter(|b| !is_table_block(b)).flat_map(|b| split_sentences(b)).any(|s| {
        let s = fold_i(&s);
        has(r"(?i)\bfirst\b", &s) && has(r"(?i)\blead|C-43|64\s+slices", &s) && has(r"(?i)sender", &s)
    });
    if !first {
        report.fail("defect", "the lead (C-43) is not stated as the sender's obligation on the FIRST issue");
    }
}

/// The DEFECT detector.
pub(crate) fn defect(ctx: &Ctx, fields: &[AnnField], report: &mut Report) {
    if !ctx.open {
        return;
    }
    tables(ctx, report);
    segments(ctx, fields, report);
    sentences(ctx, report);
}
