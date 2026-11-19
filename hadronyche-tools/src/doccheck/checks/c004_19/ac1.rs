//! AC1 to AC4 of the sync section: identical sizes stated and padded, the
//! reply's echo, epoch, holder and domain rows, statelessness, and the
//! authentication-reservation decision (script lines 838-1010).

use std::collections::BTreeSet;

use super::{Ctx, Row};
use crate::doccheck::Report;
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::text::{has_i, head, rx};

/// `re.split(r"(?<=[.;:])\s+|\n+", text)`.
pub(crate) fn split_size_sentences(text: &str) -> Vec<&str> {
    rx(r"(?<=[.;:])\s+|\n+").split(text).collect()
}

fn ac1(ctx: &Ctx, report: &mut Report) {
    let (Some(rq), Some(rp)) = (ctx.req_size, ctx.rep_size) else { return };
    if rq != rp {
        report.fail("AC1", format!("request ({rq} bytes) and reply ({rp} bytes) do not have the same byte size, by the sums of their field tables"));
    }
    let mut stated = BTreeSet::new();
    for s in split_size_sentences(&ctx.prose_all) {
        if !has_i("size|long|length", s) || !has_i("request|reply|both|each|fixed|every|sync", s) {
            continue;
        }
        for h in rx("(\\d[\\d ,_\u{a0}\u{202f}]*\\d|\\d)\\s*-?\\s*bytes?\\b").captures_iter(s) {
            let digits: String = h.get(1).map_or("", |m| m.as_str()).chars().filter(|c| !" ,_\u{a0}\u{202f}".contains(*c)).collect();
            stated.extend(parse_int(&digits).ok());
        }
    }
    if stated.is_empty() {
        report.fail("AC1", "the section states no byte size for the request and reply as a number (a sentence with 'size' or 'long', the number and 'bytes')");
    } else if stated != BTreeSet::from([rq]) {
        report.fail("AC1", format!("the stated byte size {stated:?} does not equal the sum of the field widths of each table, {rq} bytes"));
    }
    let pads: Vec<&Row> = ctx.req_f.iter().filter(|f| f.w > 0 && (f.ty == "pad" || has_i(r"\bpad|reserved", &format!("{} {}", f.meaning, f.valid)))).collect();
    if pads.is_empty() {
        report.fail("AC1", "the request table has no explicit padding or reserved field that reaches the fixed size");
    } else if !pads.iter().any(|f| has_i(r"\b0\b|zero", &format!("{} {}", f.valid, f.meaning))) {
        report.fail("AC1", "the request padding has no required value of zero in its valid range");
    }
}

/// Python's `f"{x}"` of an optional int.
pub(crate) fn opt(v: Option<i128>) -> String {
    v.map_or("None".to_string(), |v| v.to_string())
}

fn rows<'a>(fields: &'a [Row], pat: &str) -> Vec<&'a Row> {
    fields.iter().filter(|f| has_i(pat, &f.meaning)).collect()
}

fn ac2(ctx: &Ctx, report: &mut Report) {
    if ctx.rep_size.is_none() {
        return;
    }
    let k = &ctx.k;
    let echo: Vec<&Row> = rows(&ctx.rep_f, r"send\s+timestamp").into_iter().filter(|f| has_i("echo", &f.text)).collect();
    if echo.is_empty() {
        report.fail("AC2", "the reply table has no row that echoes the request's send timestamp");
    } else if !echo.iter().any(|f| f.w == 64) {
        report.fail("AC2", "the echoed send timestamp must be 64 bits, the width of the request's send timestamp");
    }
    let epoch = rows(&ctx.rep_f, "epoch");
    if epoch.is_empty() {
        report.fail("AC2", "the reply table has no timeline epoch row");
    } else if !epoch.iter().any(|f| Some(f.w) == k.epoch_bits) {
        report.fail("AC2", format!("the reply's timeline epoch must be {} bits (C-10)", opt(k.epoch_bits)));
    }
    let modid = rows(&ctx.rep_f, r"module\s+id");
    if modid.is_empty() {
        report.fail("AC2", "the reply table has no holder module id row");
    } else if !modid.iter().any(|f| Some(f.w) == k.module_bits) {
        report.fail("AC2", format!("the holder module id must be {} bits (C-11)", opt(k.module_bits)));
    }
    for (name, tab) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
        let dom = rows(tab, "domain");
        if dom.is_empty() {
            report.fail("AC2", format!("the {name} table has no domain id row"));
        } else if !dom.iter().any(|f| Some(f.w) == k.domain_bits) {
            report.fail("AC2", format!("the {name} domain id must be {} bits (C-11)", opt(k.domain_bits)));
        } else if !dom.iter().any(|f| has_i(r"unconfigured|0\s*=|zero", &f.text)) {
            report.fail("AC2", format!("the {name} domain id row does not state that 0 is the unconfigured domain"));
        }
    }
}

const NEG: &str = r"\bsequence\b|\bseq\b|exchange\s+(?:id|number|identifier)|transaction|correlat|cookie|per[- ]request|request\s+(?:id|number|identifier|counter)|\bpending\b|\bhandle\b|\btoken\b";

fn ac3(ctx: &Ctx, report: &mut Report) {
    if !ctx.prose.iter().any(|b| has_i("stateless", b)) {
        report.fail("AC3", "the section does not state that the exchange is stateless");
    }
    if !ctx.prose.iter().any(|b| has_i(r"\becho", b)) {
        report.fail("AC3", "the section does not state that the reply echoes the request's own send timestamp");
    }
    for (name, tab) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
        for f in tab.iter() {
            if let Some(h) = rx(&format!("(?i){}", NEG)).captures(&format!("{} {}", f.meaning, f.valid)) {
                report.fail("AC3", format!("the {name} table has a field that only per-request holder state could fill ('{}'): {}", h.get(0).map_or("", |m| m.as_str()), head(&f.meaning, 90)));
            }
        }
    }
}

fn decision_of(para: &str) -> Option<&'static str> {
    for s in rx(r"(?<=[.;])\s+").split(para) {
        if !has_i("authentication|reservation", s) {
            continue;
        }
        if has_i(r"\b(?:not|no|without|never)\b|n't", s) {
            return Some("not carried");
        }
        if has_i(r"\b(?:carr(?:y|ies)|includ\w*|bears?|has)\b", s) {
            return Some("carried");
        }
    }
    None
}

fn carried(ctx: &Ctx, report: &mut Report) {
    let k = &ctx.k;
    for (name, fields) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
        let scheme = fields.iter().any(|f| Some(f.w) == k.scheme_bits && has_i("scheme|authentication", &f.meaning) && has_i(r"\b0\b|none|zero", &format!("{}{}", f.valid, f.meaning)));
        if !scheme {
            report.fail("AC4", format!("the {name} table carries the reservation but has no {}-bit scheme field with value 0 (C-115)", opt(k.scheme_bits)));
        }
        if !fields.iter().any(|f| Some(f.w) == k.counter_bits && has_i("counter", &f.meaning) && has_i("live", &f.text)) {
            report.fail("AC4", format!("the {name} table carries the reservation but has no {}-bit live message counter (C-115)", opt(k.counter_bits)));
        }
        let tag = fields.iter().any(|f| Some(f.w) == k.tag_bits && has_i(r"\btag\b", &f.meaning) && has_i(r"zero|\b0\b", &format!("{}{}", f.valid, f.meaning)));
        if !tag {
            report.fail("AC4", format!("the {name} table carries the reservation but has no {}-bit zero tag field (C-115)", opt(k.tag_bits)));
        }
    }
}

fn ac4(ctx: &Ctx, report: &mut Report) {
    let Some(i) = ctx.prose.iter().position(|b| has_i("authentication|reservation", b) && has_i(r"\[decision\]", b)) else {
        report.fail("AC4", "no [decision]-tagged paragraph states the authentication-reservation choice");
        return;
    };
    let decision = decision_of(&ctx.prose[i]);
    if decision.is_none() {
        report.fail("AC4", "the decision paragraph does not say whether sync messages carry the authentication reservation");
    }
    let window = ctx.prose[i.max(1) - 1..(i + 2).min(ctx.prose.len())].join("\n");
    let ok = rx(r"(?i)(?s)falsifier\s*[:\-—]?\s*(.*)").captures(&window).is_some_and(|m| m.get(1).map_or("", |m| m.as_str()).py_split().count() >= 4);
    if !ok {
        report.fail("AC4", "the authentication decision has no falsifier (the word 'falsifier' followed by the condition that would overturn it)");
    }
    match decision {
        Some("carried") => carried(ctx, report),
        Some(_) => {
            for (name, fields) in [("request", &ctx.req_f), ("reply", &ctx.rep_f)] {
                for f in fields.iter() {
                    if has_i(r"authentication|scheme|\btag\b|counter|reservation", &f.meaning) {
                        report.fail("AC4", format!("the decision says sync messages do not carry the reservation, but the {name} table has the field: {}", head(&f.meaning, 80)));
                    }
                }
            }
        }
        None => {}
    }
}

/// AC1 to AC4.
pub(crate) fn ac1_to_ac4(ctx: &Ctx, report: &mut Report) {
    if ctx.placeholder {
        return;
    }
    ac1(ctx, report);
    ac2(ctx, report);
    ac3(ctx, report);
    ac4(ctx, report);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_sentences_split_like_python() {
        assert_eq!(split_size_sentences("a: b\n\nc. d"), vec!["a:", "b", "c.", "d"]);
        assert_eq!(split_size_sentences("x.\n y"), vec!["x.", "y"]);
    }
}
