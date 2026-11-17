//! AC1 (every Context item is a field), AC3 (the downstream A), AC5 (no
//! timer), the settled rulings, the C1 and C6 review findings and the
//! hand-off (script lines 927-1072).

use crate::doccheck::wp7::common::md_name;
use crate::doccheck::wp7::common::is_open_item;
use crate::doccheck::wp7::md::{Field, fields_of, is_field_table, parse_tables, section_span};
use crate::doccheck::wp7::py;
use super::{Ctx, Msg};
use crate::doccheck::Report;

/// 004-18's `TIMER` (with `pending`).
pub(crate) const TIMER: &str = r"timer|interval|timeout|time[\s-]*out|listen|window|silence|period|expir|deadline|backoff|delay|pending|\bms\b|seconds?\b";

fn rows_of(ctx: &Ctx, pred: impl Fn(&Field) -> bool) -> Vec<(&Msg, &Field)> {
    ctx.msgs.iter().flat_map(|m| m.fields.iter().map(move |f| (m, f))).filter(|(_, f)| pred(f)).collect()
}

fn both(f: &Field) -> String {
    format!("{} {}", f.meaning, f.valid)
}

fn rates(ctx: &Ctx, report: &mut Report) {
    let need: i128 = (1472 + 66) * 8 * 2000;
    let need_bits = 128 - i128::from(need.leading_zeros());
    let bps = rows_of(ctx, |f| py::has_i(r"bit\s*/\s*s|bits?\s+per\s+second", &both(f)) && py::has_i("declared|rate", &f.meaning));
    if bps.is_empty() {
        report.fail("AC1", "no admission field carries a declared rate in bit/s (Context: declared rates)");
    }
    for (m, f) in bps {
        if f.bits < need_bits {
            report.fail("AC1", format!("'{}': the declared rate in bit/s is {} bits; the C-219 figure ({need} bit/s) needs {need_bits}", m.title, f.bits));
        }
    }
    let pps = rows_of(ctx, |f| py::has_i(r"packets?\s*/\s*s|packets?\s+per\s+second", &both(f)));
    if pps.is_empty() {
        report.fail("AC1", "no admission field carries a declared rate in packets/s");
    }
    for (m, f) in pps {
        if f.bits < 16 {
            report.fail("AC1", format!("'{}': the packets/s rate is {} bits; at least 16 are needed for 2 000 packets/s", m.title, f.bits));
        }
    }
    let slice = rows_of(ctx, |f| py::has_i(r"slice\s+rate", &f.meaning));
    if slice.is_empty() {
        report.fail("AC1", "no admission field carries a slice rate (Context: declared rates)");
    }
    for (m, f) in slice {
        if f.bits < 16 {
            report.fail("AC1", format!("'{}': the slice rate is {} bits; at least 16 are needed for 2 000 slices/s (C-02)", m.title, f.bits));
        }
    }
    if ctx.ad_para(&["C-219"]).is_none() {
        report.fail("AC1", "no sentence in the Admission messages section cites C-219 for the declared rates");
    }
}

fn items(ctx: &Ctx, report: &mut Report) {
    if rows_of(ctx, |f| py::has_i("member", &f.meaning) && f.maxc == Some(32)).is_empty() {
        report.fail("AC1", "no admission field lists bundle members with a maximum count of 32 (C-08, C-209)");
    }
    if rows_of(ctx, |f| py::has_i("reverse", &f.meaning) && py::has_i("port", &f.meaning) && f.bits == 16).is_empty() {
        report.fail("AC1", "no admission field carries the reverse measurement port as a 16-bit port");
    }
    let addr = rows_of(ctx, |f| py::has_i("address", &f.meaning));
    if addr.is_empty() {
        report.fail("AC1", "no admission field carries a pinned address");
    }
    for (m, f) in addr {
        if f.bits != 32 {
            report.fail("AC1", format!("'{}': the pinned address is {} bits; an IPv4 address is 4 bytes (C-100)", m.title, f.bits));
        }
    }
    let p = &ctx.ad_plain;
    if !(py::has("IPv4", p) && p.contains("C-100")) {
        report.fail("Behaviour", "unicast over IPv4 (C-100) is not restated in the Admission messages section");
    }
    if !(py::has(r"CS5\s*\(40\)", p) && p.contains("C-13")) {
        report.fail("Behaviour", "DSCP CS5 (40) with C-13 is not restated in the Admission messages section");
    }
    if rows_of(ctx, |f| py::has_i("manifest", &both(f)) && py::has("004-26", &both(f))).is_empty() {
        report.fail("AC1", "no admission field refers to the manifest laid out in 004-26");
    }
    if rows_of(ctx, |f| py::has_i("probe", &both(f)) && py::has("004-27", &both(f))).is_empty() {
        report.fail("AC1", "no admission field refers to the path-MTU probe pair laid out in 004-27");
    }
}

fn downstream_a(ctx: &Ctx, report: &mut Report) {
    let da = rows_of(ctx, |f| py::has_i("downstream", &f.meaning) && py::has(r"\bA\b", &f.meaning));
    if da.is_empty() {
        report.fail("AC1", "no admission field carries the downstream A of C-142");
    }
    let rr = section_span(ctx.lines.len(), &ctx.heads, r"receiver\s+report", None);
    let mut cur_a = None;
    match rr {
        None => report.fail("AC1", "wire.md has no receiver report section: the downstream A width cannot be tied to the consumer's current A"),
        Some((a, b)) => {
            for t in parse_tables(&ctx.lines[a.min(b)..b].join("\n")) {
                if !is_field_table(&t.header) {
                    continue;
                }
                for f in fields_of(Some(&t), "AC1", "receiver report", report) {
                    if py::has_i(r"current\s+a\b", &f.meaning) {
                        cur_a = Some(f.bits);
                    }
                }
            }
            if cur_a.is_none() {
                report.fail("AC1", "the receiver report has no current A row (004-20) to tie the downstream A width to");
            }
        }
    }
    for (m, f) in da {
        if !py::has_i(r"\bns\b|nanosec", &both(f)) {
            report.fail("AC1", format!("'{}': the downstream A has no unit (nanoseconds)", m.title));
        }
        if let Some(c) = cur_a
            && f.bits != c
        {
            report.fail("AC1", format!("'{}': the downstream A is {} bits; the consumer's current A (004-20) is {c}", m.title, f.bits));
        }
        if f.bits <= 0 {
            report.fail("AC1", format!("'{}': the downstream A has no width", m.title));
        }
    }
}

fn timers_and_rulings(ctx: &Ctx, report: &mut Report) {
    for m in &ctx.msgs {
        for f in &m.fields {
            let name = md_name(&f.meaning);
            if py::has_i(TIMER, name) {
                report.fail("AC5", format!("'{}': the field '{name}' is a timer; C-45 gives Pending no timeout", m.title));
            }
        }
    }
    let p = &ctx.ad_plain;
    if !p.contains("C-45") {
        report.fail("AC5", "C-45 (Pending has no timeout) is not cited in the Admission messages section");
    }
    for c in ["C-17", "C-103", "C-142", "C-185", "C-209", "C-219"] {
        if !p.contains(c) {
            report.fail("Settled", format!("{c} is not cited in the Admission messages section"));
        }
    }
}

fn order_and_citation(ctx: &Ctx, report: &mut Report) {
    match ctx.msgs.iter().find(|m| py::has_i("request", &m.title)) {
        None => report.fail("AC6", "no admission request message: its invalid-form table cannot be checked for drops before refusals"),
        Some(req) => {
            let kinds: Vec<&str> = req
                .forms
                .iter()
                .filter_map(|(_, o)| {
                    if py::has_i(r"\bdrop\b|\bignore\b", o) {
                        Some("drop")
                    } else if py::has_i("refus", o) {
                        Some("refusal")
                    } else {
                        None
                    }
                })
                .collect();
            let last_drop = kinds.iter().rposition(|k| *k == "drop");
            let first_ref = kinds.iter().position(|k| *k == "refusal");
            if let (Some(d), Some(r)) = (last_drop, first_ref)
                && d > r
            {
                report.fail("AC6", "the admission request's invalid-form table lists a refusal before a drop: every drop (decided from the datagram's bytes) must come before every refusal (receiver state, C-164)");
            }
        }
    }
    if ctx.ad_para(&["005-14", "order"]).is_none() {
        report.fail("AC6", "no sentence in the Admission messages section hands the order among refusals to 005-14");
    }
    // `open\s+item(?!s)` under re.I: the look-ahead refuses s, S and the
    // long s that re.I folds to s.
    let cites = |s: &str| py::finditer(r"open\s+item", s, true).iter().any(|h| !s[h.end..].chars().next().is_some_and(|c| matches!(c, 's' | 'S' | '\u{17f}')));
    let mut cited = false;
    for t in parse_tables(&ctx.ad_plain) {
        for row in &t.rows {
            if row.iter().any(|c| py::strip(c).trim_matches(|x| "`*_ ".contains(x)) == "ForeignDomain") && cites(&row.join(" | ")) {
                cited = true;
            }
        }
    }
    for p in &ctx.ad_paras {
        if py::has(r"\bForeignDomain\b", p) && cites(p) && !is_open_item(p) {
            cited = true;
        }
    }
    if !cited {
        report.fail("AC6", "the ForeignDomain refusal row (or a sentence naming it) does not cite an open item by number ('Open item N')");
    }
    if !py::has("012-01", &ctx.ad_plain) {
        report.fail("Hand-off", "the Admission messages section does not mention 012-01, which fixes RefuseReason's discriminants (0..=17 in the order of C-185)");
    }
}

/// AC1, AC3, AC5, Settled, the C1/C6 findings and the hand-off.
pub(crate) fn contents(ctx: &Ctx, report: &mut Report) {
    rates(ctx, report);
    items(ctx, report);
    downstream_a(ctx, report);
    timers_and_rulings(ctx, report);
    order_and_citation(ctx, report);
}
