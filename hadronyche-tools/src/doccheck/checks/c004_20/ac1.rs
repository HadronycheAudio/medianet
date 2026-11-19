//! AC1 (two kinds, unicast only, no query), AC2 (the report's maximum size
//! and member count), AC3 (every content item, W1 header rows, the A
//! binding to 004-18) (script lines 891-1082).

use super::{Ctx, Field, int_mul, layout, max_int, stated_bytes};
use crate::doccheck::Report;
use crate::doccheck::family004::{is_field_table, tables};
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{has, has_i, head, rx};

/// 004-20's `UNIT`.
pub(crate) const UNIT: &str = r"\b(ns|nanoseconds?|samples?|ms|milliseconds?|µs|us|microseconds?)\b";
const WRAP: &str = r"\b(wraps?|modulo|saturates?)\b";
const RECLASS: &str = r"decreas|not\s+monotonic|non-?monotonic|unambiguous|[−-]\s*1\b|minus\s+one";

fn find_lab<'a>(fields: &'a [Field], key: &str) -> Vec<&'a Field> {
    fields.iter().filter(|f| f.lab.contains(key)).collect()
}

fn ac1(ctx: &Ctx, report: &mut Report) {
    let p = &ctx.rep.prose_all;
    if !has_i(r"\bunicast\b", p) || !has_i(r"\b(?:not|never|no)\b[^.;]{0,20}\bmulticast\b|\bunicast\s+only\b", p) {
        report.fail("AC1", "the receiver report section does not say the report is unicast only, not multicast (C-47)");
    }
    if !has_i(r"\b(?:no|not|never|without)\b[^.;]{0,40}\bquer(?:y|ies)\b", p) {
        report.fail("AC1", "the receiver report section does not state that there is no query message (C-47)");
    }
}

fn ac2(ctx: &Ctx, report: &mut Report) {
    let Some(max) = ctx.max_rep() else { return };
    let (fix, mem) = (ctx.bytes(ctx.rep_fix_bits), ctx.bytes(ctx.rep_mem_bits));
    let mm = ctx.k.members_max;
    let all32 = match (mem, mm) {
        (Some(m), Some(k)) if m != 0 => Some(int_mul(k, m)),
        _ => None,
    };
    let allowed = [Some(max), fix, mem, all32];
    let size_sents: Vec<&String> = ctx.rep.sents.iter().filter(|s| has_i("maximum|at most|largest|size|long", s) && has(r"\b32\b", s)).collect();
    if !size_sents.iter().any(|s| stated_bytes(s).contains(&max)) {
        report.fail("AC2", format!("the receiver report's maximum size with 32 members is not stated in bytes as fixed + 32 x per-member = {max} bytes"));
    }
    for s in &size_sents {
        if let Some(n) = stated_bytes(s).into_iter().find(|n| !allowed.contains(&Some(*n))) {
            report.fail("AC2", format!("a size sentence states {n} bytes, which is not the computed maximum {max} (or its parts): {}", head(s, 90)));
        }
    }
    match ctx.rep_fix.iter().find(|f| f.lab.contains("member") && f.lab.contains("count")) {
        None => report.fail("AC2", "the fixed part has no member count field"),
        Some(c) => {
            let vmax = max_int(&c.valid);
            if vmax != mm {
                report.fail("AC2", format!("the member count's valid range has maximum {vmax:?}, not {mm:?} (C-08)"));
            }
        }
    }
}

const COUNTERS: [(&str, &str); 11] = [
    ("received", "received"),
    ("late", "late"),
    ("missing", "missing"),
    ("markedgap", "marked gap"),
    ("otherepoch", "other-epoch"),
    ("tooearly", "too-early"),
    ("duplicate", "duplicates"),
    ("incompletereassembl", "incomplete reassemblies"),
    ("demoted", "demoted rounds"),
    ("interrupted", "interrupted events"),
    ("dead", "dead"),
];

fn counters(ctx: &Ctx, report: &mut Report) {
    let all: Vec<Field> = ctx.rep_fix.iter().chain(&ctx.rep_mem).cloned().collect();
    match find_lab(&ctx.rep_fix, "bundle").first() {
        None => report.fail("AC3", "the fixed part has no bundle identity field"),
        Some(b) if !(b.w == 64 && has_i(r"module\s+id", &b.meaning)) => {
            report.fail("AC3", format!("the bundle identity must be the bundle source's module id, 64 bits (C-11): {}", head(&b.meaning, 80)));
        }
        Some(_) => {}
    }
    let cover_all = ctx.rep.sents.iter().any(|s| has_i(r"\bcounters?\b", s) && has_i(WRAP, s));
    for (key, shown) in COUNTERS {
        let Some(r0) = find_lab(&ctx.rep_mem, key).first().copied() else {
            report.fail("AC3", format!("the per-member table has no field for the C-159 counter '{shown}'"));
            continue;
        };
        if r0.w <= 0 {
            report.fail("AC3", format!("the counter '{shown}' has no width"));
        }
        let both = format!("{} {}", r0.meaning, r0.valid);
        let is_flag = key == "dead" && has_i(r"\bflag\b", &both);
        if !(cover_all || is_flag || has_i(WRAP, &both)) {
            report.fail("AC3", format!("the counter '{shown}' states no wrap-or-saturate rule (wrap, modulo or saturate)"));
        }
    }
    for (key, shown) in [("socket", "socket drop"), ("interface", "interface drop")] {
        match all.iter().find(|f| f.lab.contains(key) && f.lab.contains("drop")) {
            None => report.fail("AC3", format!("the report has no {shown} counter field")),
            Some(r) if r.w <= 0 || !(cover_all || has_i(WRAP, &format!("{} {}", r.meaning, r.valid))) => {
                report.fail("AC3", format!("the {shown} counter needs a width and a wrap-or-saturate rule"));
            }
            Some(_) => {}
        }
    }
    match all.iter().find(|f| f.lab.contains("dscp")) {
        None => report.fail("AC3", "the report has no observed DSCP field"),
        Some(d) => {
            if d.w != 6 {
                report.fail("AC3", format!("the observed DSCP must be exactly 6 bits, not {} (the DSCP is 0 to 63)", d.w));
            }
            if max_int(&d.valid) != Some(63) {
                report.fail("AC3", format!("the observed DSCP's valid range must end at 63: {}", head(&d.valid, 60)));
            }
        }
    }
}

/// The current A row (`A_ROW`), checked.
fn a_row(ctx: &Ctx, report: &mut Report) -> Option<Field> {
    let all: Vec<&Field> = ctx.rep_fix.iter().chain(&ctx.rep_mem).collect();
    let Some(a) = all.into_iter().find(|f| has(r"\bA\b", &f.label) || f.meaning.contains("C-198")) else {
        report.fail("AC3", "the report has no field for the consumer's current A (C-198)");
        return None;
    };
    if a.w <= 0 {
        report.fail("AC3", "the current A has no stated width");
    }
    if !has_i(UNIT, &format!("{} {}", a.meaning, a.valid)) {
        report.fail("AC3", "the current A has no stated unit (ns, samples or ms)");
    }
    if !has(r"\d", &a.valid) {
        report.fail("AC3", "the current A has no stated valid range");
    }
    Some(a.clone())
}

fn sentences_of_ac3(ctx: &Ctx, report: &mut Report) {
    let s = &ctx.rep.sents;
    if !s.iter().any(|x| has_i("C-108|reclassif", x) && has_i(RECLASS, x)) {
        report.fail("AC3", "the report does not state how the C-108 reclassification (missing -1, late +1) is reflected");
    }
    if !s.iter().any(|x| has_i(r"\binterrupted\b", x) && has_i(r"\bevents?\b", x)) {
        report.fail("AC3", "the report does not say the interrupted counter counts interrupted EVENTS (C-102, C-38)");
    }
    if !s.iter().any(|x| has_i(r"\bdemoted\b", x) && has_i(r"\brounds?\b", x) && has_i(r"\bcounter", x)) {
        report.fail("AC3", "the report does not say Demoted is carried by the demoted-rounds counter (C-109, C-159)");
    }
    if !s.iter().any(|x| has("004-18", x) && has_i(r"\bsame\b|\bbind|\bequal|\bmatch", x)) {
        report.fail("AC3", "the report does not say its A has the width and unit of the downstream A of 004-18 (C-142)");
    }
}

fn w1(ctx: &Ctx, report: &mut Report) {
    for (name, fields) in [("receiver report fixed part", &ctx.rep_fix), ("probe request", &ctx.req_f), ("probe reply", &ctx.rep_f)] {
        if name != "receiver report fixed part" && ctx.prb_place {
            continue;
        }
        let short = fields.len() < 4;
        if short || !(has_i("magic", &fields[0].label) && fields[0].off == 0 && fields[0].w == 16 && has_i("4d4e", &fields[0].text)) {
            report.fail("W1", format!("the {name} does not start with the 2-byte magic 0x4d4e at offset 0"));
        }
        if short || !(has_i("version", &fields[1].label) && fields[1].off == 16 && fields[1].w == 16) {
            report.fail("W1", format!("the {name} has no 2-byte version field at offset 2"));
        }
        if short || !(has_i("domain", &fields[2].label) && fields[2].off == 32 && fields[2].w == 16) {
            report.fail("W1", format!("the {name} has no 2-byte domain id field at offset 4"));
        }
        if !fields.iter().any(|f| has_i(r"message\s+type", &format!("{} {}", f.label, f.meaning))) {
            report.fail("W1", format!("the {name} has no message type row"));
        }
    }
}

fn binding(ctx: &Ctx, a: Option<&Field>, report: &mut Report) {
    if ctx.adm_place {
        let ok = ctx.rep.prose.iter().any(|b| {
            rx(r"(?i)^(?:\W*open\s+items?\b)").is_match((b).py_strip()) && has("004-18", b) && has_i("owner", b) && has_i("falsifier", b) && has_i("width|unit", b)
        });
        if !ok {
            report.fail("AC3", "the A field is not bound to the downstream A of 004-18 (C-142): the Admission messages section is a placeholder, so an Open item must name 004-18 as owner, with a falsifier that says 004-18 lays out a different width or unit");
        }
        return;
    }
    let mut adm = Vec::new();
    for t in tables(&ctx.adm.text) {
        if is_field_table(t.head()) {
            let rows: Vec<Vec<String>> = t.rows.into_iter().map(|r| r.cells).collect();
            adm.extend(layout(&rows, "admission", "AC3", report).0);
        }
    }
    match adm.iter().find(|f| has(r"\bA\b", &f.label) || has_i("downstream", &f.meaning)) {
        None => report.fail("AC3", "the Admission messages section has no downstream A field to compare with (004-18)"),
        Some(d) => {
            if let Some(a) = a
                && (d.w != a.w || !has_i(UNIT, &format!("{} {}", d.meaning, d.valid)))
            {
                report.fail("AC3", format!("the current A ({} bits) does not have the width and unit of the downstream A of 004-18 ({} bits)", a.w, d.w));
            }
        }
    }
}

/// AC1 to AC3 and W1.
pub(crate) fn ac1_to_ac3(ctx: &Ctx, report: &mut Report) {
    if !ctx.rep_place {
        ac1(ctx, report);
    }
    if ctx.rep_ok {
        ac2(ctx, report);
        counters(ctx, report);
        let a = a_row(ctx, report);
        sentences_of_ac3(ctx, report);
        w1(ctx, report);
        binding(ctx, a.as_ref(), report);
    }
}
