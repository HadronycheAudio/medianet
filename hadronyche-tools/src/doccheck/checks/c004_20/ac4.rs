//! AC4: the probe's measured quantities, the authentication decision for
//! both messages, the probe identifiers, timestamps and sizes (script lines
//! 1088-1218).

use super::{Ctx, Field, Sec, stated_bytes};
use crate::doccheck::Report;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{has_i, head, rx};

fn decision_blocks(sec: &Sec) -> Vec<(usize, &String)> {
    sec.prose.iter().enumerate().filter(|(_, b)| has_i(r"\[decision\]", b)).collect()
}

/// `falsifier_near(sec, idx)`: the text after "falsifier" in the block and
/// its neighbours.
fn falsifier_near(sec: &Sec, idx: usize) -> Option<String> {
    let window = sec.prose[idx.max(1) - 1..(idx + 2).min(sec.prose.len())].join("\n");
    rx(r"(?i)(?s)falsifier\s*[:\-—]?\s*(.*)").captures(&window).map(|m| m.get(1).map_or("", |m| m.as_str()).to_string())
}

fn words(s: &str) -> usize {
    s.py_split().count()
}

fn quantities(ctx: &Ctx, report: &mut Report) {
    let hit = decision_blocks(&ctx.prb).into_iter().find(|(_, b)| has_i(r"round[\s-]+trip|one-way", b) && has_i("delay", b) && has_i("timestamp", b));
    match hit {
        None => report.fail("AC4", "no [decision]-tagged probe paragraph states which delays its timestamps yield (one-way in each direction under network time, round trip, or both)"),
        Some((i, _)) => {
            let fal = falsifier_near(&ctx.prb, i);
            if !fal.is_some_and(|f| words(&f) >= 4 && has_i("delay", &f)) {
                report.fail("AC4", "the probe-quantities decision has no falsifier naming the one-way delay a consumer could need and the timestamps cannot yield");
            }
        }
    }
}

fn decision_of(text: &str) -> Option<&'static str> {
    for s in rx(r"(?<=[.;])\s+").split(text) {
        if !has_i("authentication|reservation", s) {
            continue;
        }
        if has_i(r"\b(?:not|no|without|never)\b|n't", s) {
            return Some("not carried");
        }
        if has_i(r"\b(?:carr(?:y|ies|ied)|includ\w*|bears?|has)\b", s) {
            return Some("carried");
        }
    }
    None
}

fn carried(ctx: &Ctx, report: &mut Report) {
    let k = &ctx.k;
    for (name, fields) in [("report fixed part", &ctx.rep_fix), ("probe request", &ctx.req_f), ("probe reply", &ctx.rep_f)] {
        let sch = fields.iter().any(|f| has_i("authentication|scheme", &f.label) && Some(f.w) == k.scheme_bits && has_i(r"\b0\b|none|zero", &f.valid));
        let cnt = fields.iter().any(|f| has_i("counter", &f.label) && Some(f.w) == k.counter_bits && has_i("live", &format!("{} {}", f.meaning, f.valid)));
        let tag = fields.iter().any(|f| has_i(r"\btag\b", &f.label) && Some(f.w) == k.tag_bits && has_i(r"zero|\b0\b", &format!("{} {}", f.valid, f.meaning)));
        if !sch || !cnt || !tag {
            report.fail("AC4", format!("the decision carries the reservation but the {name} table lacks the C-12 widths with the C-115 values (scheme 0, live counter, zero tag)"));
        }
    }
}

fn not_carried(ctx: &Ctx, report: &mut Report) {
    for (name, fields) in [("report fixed part", &ctx.rep_fix), ("report per-member part", &ctx.rep_mem), ("probe request", &ctx.req_f), ("probe reply", &ctx.rep_f)] {
        for f in fields.iter() {
            if has_i("authentication|scheme|reservation", &f.label) || has_i(r"\btag\b|message counter", &f.label) {
                report.fail("AC4", format!("the decision says the messages do not carry the reservation, but the {name} table has the field '{}'", f.label));
            }
        }
    }
}

fn authentication(ctx: &Ctx, report: &mut Report) {
    let mut hit = None;
    for sec in [&ctx.rep, &ctx.prb] {
        if let Some((i, b)) = decision_blocks(sec).into_iter().find(|(_, b)| has_i("authentication|reservation", b)) {
            hit = Some((sec, i, b.clone()));
            break;
        }
    }
    let Some((sec, idx, text)) = hit else {
        report.fail("AC4", "no [decision]-tagged paragraph states the authentication-reservation choice");
        return;
    };
    if !((has_i(r"\breport", &text) && has_i(r"\bprobe", &text)) || has_i(r"\bboth\b", &text)) {
        report.fail("AC4", "the authentication decision does not say it covers both the report and the probe");
    }
    let decision = decision_of(&text);
    if decision.is_none() {
        report.fail("AC4", "the authentication decision does not say whether the messages carry the reservation");
    }
    if !falsifier_near(sec, idx).is_some_and(|f| words(&f) >= 4) {
        report.fail("AC4", "the authentication decision has no falsifier (the word 'falsifier' followed by the condition that would overturn it)");
    }
    match decision {
        Some("carried") => carried(ctx, report),
        Some(_) => not_carried(ctx, report),
        None => {}
    }
}

fn identifiers_and_sizes(ctx: &Ctx, report: &mut Report) {
    let ident: Vec<&Field> = ctx.req_f.iter().filter(|f| has_i("identif", &format!("{} {}", f.label, f.meaning))).collect();
    let echo: Vec<&Field> = ctx.rep_f.iter().filter(|f| has_i(r"\becho", &f.meaning)).collect();
    if ident.is_empty() {
        report.fail("AC4", "the probe request has no identifier field closing the round trip");
    }
    if echo.is_empty() || (!ident.is_empty() && !echo.iter().any(|e| e.w == ident[0].w)) {
        report.fail("AC4", "the probe reply does not echo the request's identifier at the same width (the reply without matching fields is an invalid form)");
    }
    for (name, fields) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
        if !fields.iter().any(|f| has_i("timestamp", &format!("{} {}", f.label, f.meaning)) && f.w == 64) {
            report.fail("AC4", format!("the probe {name} has no 64-bit timestamp field"));
        }
    }
    let p = &ctx.prb.prose_all;
    if !has_i(r"slice[\s-]header", p) || !has_i("same|consistent|different|differs", p) {
        report.fail("AC4", "the probe does not state its timestamps are consistent with (or differ from) the slice-header send timestamp (004-08) and the sync timestamps (004-19)");
    }
    if !has_i(r"nanosecond|\bns\b", p) || !has_i("monotonic", p) {
        report.fail("AC4", "the probe timestamps' unit (nanoseconds) and clock (monotonic) are not stated");
    }
    let whole = |b: Option<i128>| b.filter(|b| b.rem_euclid(8) == 0).map(|b| b.div_euclid(8));
    let (Some(rq), Some(rp)) = (whole(ctx.req_bits), whole(ctx.rpl_bits)) else {
        report.fail("AC4", "the probe request or reply has no whole-byte size");
        return;
    };
    let size_sents: Vec<&String> = ctx.prb.sents.iter().filter(|s| has_i("size|long|bytes", s) && !stated_bytes(s).is_empty()).collect();
    let all: Vec<i128> = size_sents.iter().flat_map(|s| stated_bytes(s)).collect();
    if !all.contains(&rq) || !all.contains(&rp) {
        report.fail("AC4", format!("the probe sizes are not stated as the sums of their tables (request {rq} bytes, reply {rp} bytes)"));
    }
    for s in &size_sents {
        if let Some(n) = stated_bytes(s).into_iter().find(|n| *n != rq && *n != rp) {
            report.fail("AC4", format!("a probe size sentence states {n} bytes, not a table sum: {}", head(s, 90)));
        }
    }
}

/// AC4, when the probe section and both its tables exist.
pub(crate) fn probe(ctx: &Ctx, report: &mut Report) {
    if ctx.prb_place || !ctx.prb_ok {
        return;
    }
    quantities(ctx, report);
    authentication(ctx, report);
    identifiers_and_sizes(ctx, report);
}
