//! Work items W1, W2, W4, W5 and the DoD of the sync section (script lines
//! 1238-1317).

use super::ac6::is_dscp_or_rate;
use super::{Ctx, Row};
use crate::doccheck::Report;
use crate::doccheck::evidence::{TableTagRule, TagPolicy, untagged, untagged_tables};
use crate::doccheck::items::{OpenItemPolicy, unowned_items};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::{has, has_i, head, rx};

const NEG_MENTION: &str = r"\b(?:no|not|without|never)\b[^|.;]{0,30}?\b(?:dscp|codepoint|rate)\b";

fn w1(ctx: &Ctx, report: &mut Report) {
    for (name, fields) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
        if fields.len() < 3 {
            report.fail("W1", format!("the {name} table has fewer than three header fields"));
            continue;
        }
        let (f0, f1) = (&fields[0], &fields[1]);
        if !(f0.off == 0 && f0.w == 16 && has_i("4d4e", &f0.text)) {
            report.fail("W1", format!("the {name} table does not start with the 2-byte magic 0x4d4e at offset 0"));
        }
        if !(f1.off == 16 && f1.w == 16 && has_i(r"0x0100|\b1\.0\b", &f1.text)) {
            report.fail("W1", format!("the {name} table's version is not the 2-byte v1 version 0x0100 at offset 2"));
        }
        if !fields.iter().any(|f| has_i(r"message\s+type|type\s+code|message\s+kind", &f.meaning)) {
            report.fail("W1", format!("the {name} table has no message type row (ticket Work 1)"));
        }
    }
}

fn w2_w4_w5(ctx: &Ctx, report: &mut Report) {
    let p = &ctx.prose_all;
    if !has_i(r"nanosecond|\bns\b", p) {
        report.fail("W2", "the sync timestamps' unit (nanoseconds) is not stated");
    }
    if !has_i("monotonic", p) {
        report.fail("W2", "the sync timestamps' clock (monotonic) is not stated");
    }
    if !has_i(r"slice[\s-]header", p) {
        report.fail("W2", "the section does not relate its timestamps to the slice-header send timestamp");
    } else if !has_i(r"same\s+(?:unit|clock|basis|as)|consistent\s+with|identical\s+to|explicitly\s+different|differs?\s+from|different\s+from", p) {
        report.fail("W2", "the timestamps are neither stated as consistent with nor explicitly different from the slice-header send timestamp");
    }
    let k = &ctx.k;
    if let Some(d) = k.dscp
        && !has(&format!(r"CS6\s*\(\s*{d}\s*\)"), p)
    {
        report.fail("W4", format!("the section does not restate the DSCP as CS6 ({d}) (C-13)"));
    }
    if !(has("IPv4", p) && has_i("unicast", p)) {
        report.fail("W4", "the section does not restate IPv4 unicast (C-100) as behaviour");
    }
    if let Some(r) = k.req_rate
        && !has_i(&format!(r"\b{r}\b[^.\n]{{0,40}}per\s+second"), p)
    {
        report.fail("W4", format!("the section does not restate {r} requests per second per follower (C-36)"));
    }
    if let Some((lo, hi)) = k.acc
        && !has_i(&format!(r"\b{lo}\b[^.\n]{{0,40}}\b{hi}\b[^.\n]{{0,40}}per\s+second"), p)
    {
        report.fail("W4", format!("the section does not restate the accepted {lo} to {hi} requests per second (C-112)"));
    }
    for (name, fields) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
        for f in fields.iter() {
            let cell = rx(&format!("(?i){}", NEG_MENTION)).replace_all_literal(&f.text, "");
            if is_dscp_or_rate(&cell) || has_i(r"per\s+second|per-second", &cell) {
                report.fail("W4", format!("the {name} table names a DSCP or a rate (C-152, C-204): {}", head(&f.meaning, 80)));
            }
        }
    }
    if !has_i(r"per[- ]OS", p) {
        report.fail("W5", "no note slot for per-OS receive-timestamp remarks (epic 001) in the section");
    }
    if !has("C-131", p) {
        report.fail("W5", "the section does not name C-131 for the receive-timestamp source");
    }
    if !has_i(r"regime\s+stamp", p) {
        report.fail("W5", "the section does not say the receive-timestamp source lives in the regime stamp");
    }
    let all: Vec<&Row> = ctx.rep_f.iter().chain(&ctx.req_f).collect();
    if let Some(f) = all.iter().find(|f| has_i(r"\b(?:kernel|userspace|mac-hardware|interrupt)\b", &f.meaning)) {
        report.fail("W5", format!("a sync table names a receive-timestamp source: {}", head(&f.meaning, 80)));
    }
}

fn dod(ctx: &Ctx, report: &mut Report) {
    let first_line = |text: &str| head(splitlines(text).first().copied().unwrap_or(""), 90);
    for _ in untagged_tables(&ctx.section, TableTagRule::AfterProse004_19) {
        report.fail("DoD", "a table of the Sync request section carries no evidence tag and follows an untagged paragraph");
    }
    for u in untagged(&ctx.section, &TagPolicy::BLOCKS_004_19) {
        report.fail("DoD", format!("a paragraph of the Sync request section carries no evidence tag: {}", first_line(&u.text)));
    }
    for item in unowned_items(&ctx.section, OpenItemPolicy::Window3) {
        report.fail("DoD", format!("an open item of the Sync request section names no owner or no falsifier: {}", first_line(&item.text)));
    }
}

/// W1, W2, W4, W5 and DoD, when the section and both tables exist.
pub(crate) fn works_and_dod(ctx: &Ctx, report: &mut Report) {
    if ctx.placeholder || ctx.req_f.is_empty() || ctx.rep_f.is_empty() {
        return;
    }
    w1(ctx, report);
    w2_w4_w5(ctx, report);
    dod(ctx, report);
}
