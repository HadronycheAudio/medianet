//! Consistency (C-40, C-41 as behaviour, no rate field, rule 8, the DSCP
//! values, IPv4 unicast on the reverse port) and the DoD (script lines
//! 1437-1490).

use super::{Ctx, sentences};
use crate::doccheck::Report;
use crate::doccheck::evidence::{TableTagRule, TagPolicy, untagged, untagged_tables};
use crate::doccheck::items::{OpenItemPolicy, unowned_items};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::{has, has_i, head, rx};

const NEG_WORD: &str = r"\b(?:no|not|never|without|cannot|nor)\b";

fn report_section(ctx: &Ctx, report: &mut Report) {
    let p = &ctx.rep.prose_all;
    let k = &ctx.k;
    if let Some(ms) = k.report_ms
        && !has_i(&format!(r"\b{ms}\s*(?:ms|milliseconds)\b"), p)
    {
        report.fail("C-40", format!("the report section does not restate one report per bundle every {ms} ms (C-40)"));
    }
    if !has_i(r"\bbundle\b", p) {
        report.fail("C-40", "the report section does not say the report is per bundle (C-40)");
    }
    if let Some(s) = k.timeout_s
        && !has_i(&format!(r"\b{s}\s*(?:s|seconds?)\b"), p)
    {
        report.fail("C-41", format!("the report section does not restate the producer's {s} s timeout without receiver reports (C-41)"));
    }
    if let Some(d) = k.dscp_report
        && !has(&format!(r"CS5\s*\({d}\)"), p)
    {
        report.fail("C-13", format!("the report section does not restate its DSCP as CS5 ({d}) (C-13)"));
    }
    for f in ctx.rep_fix.iter().chain(&ctx.rep_mem) {
        let tokens: Vec<String> = rx("[^A-Za-z0-9]+").split(&f.label).filter(|t| !t.is_empty()).map(str::to_lowercase).collect();
        let named = tokens.iter().any(|t| ["rate", "interval", "period", "frequency"].contains(&t.as_str()));
        if named || (has_i(r"per\s+second", &f.meaning) && !has_i(NEG_WORD, &f.meaning)) {
            report.fail("C-40", format!("the report table names a rate or interval field ('{}'); C-40 is behaviour, not a field", f.label));
        }
    }
    for s in sentences(&ctx.rep.prose_all).iter().chain(&sentences(&ctx.prb.prose_all)) {
        if has_i(r"(forward|media)\s+(?:media\s+)?(?:packets?|headers?)", s) && has_i(r"receiver|per[- ]receiver", s) && !has_i(NEG_WORD, s) {
            report.fail("AC1", format!("a sentence says forward media packets carry receiver state, which rule 8 forbids: {}", head(s, 90)));
        }
    }
}

fn dod(ctx: &Ctx, report: &mut Report) {
    let first_line = |text: &str| head(splitlines(text).first().copied().unwrap_or(""), 90);
    for (name, sec, place) in [("Receiver report", &ctx.rep, ctx.rep_place), ("One-way probe", &ctx.prb, ctx.prb_place)] {
        if place {
            continue;
        }
        for _ in untagged_tables(&sec.text, TableTagRule::AfterProse004_19) {
            report.fail("DoD", format!("a table of the {name} section carries no evidence tag and follows an untagged paragraph"));
        }
        for u in untagged(&sec.text, &TagPolicy::BLOCKS_004_19) {
            report.fail("DoD", format!("a paragraph of the {name} section carries no evidence tag: {}", first_line(&u.text)));
        }
        for item in unowned_items(&sec.text, OpenItemPolicy::WordLeading) {
            report.fail("DoD", format!("an open item of the {name} section names no owner or no falsifier: {}", first_line(&item.text)));
        }
    }
}

/// Consistency and DoD.
pub(crate) fn consistency_and_dod(ctx: &Ctx, report: &mut Report) {
    if !ctx.rep_place {
        report_section(ctx, report);
    }
    if !ctx.prb_place
        && let Some(d) = ctx.k.dscp_probe
        && !has(&format!(r"EF\s*\({d}\)"), &ctx.prb.prose_all)
    {
        report.fail("C-13", format!("the probe section does not restate its DSCP as EF ({d}) (C-13)"));
    }
    if !(ctx.rep_place && ctx.prb_place) {
        let both = format!("{}\n{}", ctx.rep.prose_all, ctx.prb.prose_all);
        if !has("IPv4", &both) || !has_i("unicast", &both) || !has_i("reverse", &both) {
            report.fail("C-100", "the sections do not restate IPv4 unicast on the reverse port (C-100)");
        }
    }
    dod(ctx, report);
}
