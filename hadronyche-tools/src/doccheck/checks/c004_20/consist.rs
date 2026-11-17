//! Consistency (C-40, C-41 as behaviour, no rate field, rule 8, the DSCP
//! values, IPv4 unicast on the reverse port) and the DoD (script lines
//! 1437-1490).

use crate::doccheck::wp7::md::has_evidence;
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::common::block_kind;
use super::{Ctx, sentences};
use crate::doccheck::Report;

const NEG_WORD: &str = r"\b(?:no|not|never|without|cannot|nor)\b";

fn report_section(ctx: &Ctx, report: &mut Report) {
    let p = &ctx.rep.prose_all;
    let k = &ctx.k;
    if let Some(ms) = k.report_ms
        && !py::has_i(&format!(r"\b{ms}\s*(?:ms|milliseconds)\b"), p)
    {
        report.fail("C-40", format!("the report section does not restate one report per bundle every {ms} ms (C-40)"));
    }
    if !py::has_i(r"\bbundle\b", p) {
        report.fail("C-40", "the report section does not say the report is per bundle (C-40)");
    }
    if let Some(s) = k.timeout_s
        && !py::has_i(&format!(r"\b{s}\s*(?:s|seconds?)\b"), p)
    {
        report.fail("C-41", format!("the report section does not restate the producer's {s} s timeout without receiver reports (C-41)"));
    }
    if let Some(d) = k.dscp_report
        && !py::has(&format!(r"CS5\s*\({d}\)"), p)
    {
        report.fail("C-13", format!("the report section does not restate its DSCP as CS5 ({d}) (C-13)"));
    }
    for f in ctx.rep_fix.iter().chain(&ctx.rep_mem) {
        let tokens: Vec<String> = f.label.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| !t.is_empty()).map(str::to_lowercase).collect();
        let named = tokens.iter().any(|t| ["rate", "interval", "period", "frequency"].contains(&t.as_str()));
        if named || (py::has_i(r"per\s+second", &f.meaning) && !py::has_i(NEG_WORD, &f.meaning)) {
            report.fail("C-40", format!("the report table names a rate or interval field ('{}'); C-40 is behaviour, not a field", f.label));
        }
    }
    for s in sentences(&ctx.rep.prose_all).iter().chain(&sentences(&ctx.prb.prose_all)) {
        if py::has_i(r"(forward|media)\s+(?:media\s+)?(?:packets?|headers?)", s) && py::has_i(r"receiver|per[- ]receiver", s) && !py::has_i(NEG_WORD, s) {
            report.fail("AC1", format!("a sentence says forward media packets carry receiver state, which rule 8 forbids: {}", py::head(s, 90)));
        }
    }
}

fn dod(ctx: &Ctx, report: &mut Report) {
    for (name, sec, place) in [("Receiver report", &ctx.rep, ctx.rep_place), ("One-way probe", &ctx.prb, ctx.prb_place)] {
        if place {
            continue;
        }
        let mut last_tagged = true;
        for b in &sec.blocks {
            let kind = block_kind(b);
            let text = b.join("\n");
            if kind == "heading" || kind == "code" {
                continue;
            }
            if kind == "table" {
                if !(has_evidence(&text) || last_tagged) {
                    report.fail("DoD", format!("a table of the {name} section carries no evidence tag and follows an untagged paragraph"));
                }
                continue;
            }
            let words = crate::doccheck::wp7::common::words(&text);
            let tagged = has_evidence(&text);
            if words >= 5 && !tagged {
                report.fail("DoD", format!("a paragraph of the {name} section carries no evidence tag: {}", py::head(py::splitlines(&text).first().copied().unwrap_or(""), 90)));
            }
            last_tagged = tagged || words < 5;
        }
        for b in &sec.prose {
            if py::matches(r"\W*open\s+items?\b", py::strip(b), true).is_some() && (!py::has_i(r"\bowner\b", b) || !py::has_i("falsifier", b)) {
                report.fail("DoD", format!("an open item of the {name} section names no owner or no falsifier: {}", py::head(py::splitlines(b).first().copied().unwrap_or(""), 90)));
            }
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
        && !py::has(&format!(r"EF\s*\({d}\)"), &ctx.prb.prose_all)
    {
        report.fail("C-13", format!("the probe section does not restate its DSCP as EF ({d}) (C-13)"));
    }
    if !(ctx.rep_place && ctx.prb_place) {
        let both = format!("{}\n{}", ctx.rep.prose_all, ctx.prb.prose_all);
        if !py::has("IPv4", &both) || !py::has_i("unicast", &both) || !py::has_i("reverse", &both) {
            report.fail("C-100", "the sections do not restate IPv4 unicast on the reverse port (C-100)");
        }
    }
    dod(ctx, report);
}
