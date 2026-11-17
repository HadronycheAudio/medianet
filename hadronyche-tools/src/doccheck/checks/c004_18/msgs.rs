//! AC1, AC2 and AC4 per admission message: the admission block, header
//! rows, tiling, maximum size, type code, the C-115 values, the refusal
//! reason field (script lines 849-925).

use std::collections::BTreeMap;

use crate::doccheck::wp7::md::{find_sub, key_of, parse_tables, strip_fences};
use crate::doccheck::wp7::common::{NUM, to_int};
use crate::doccheck::wp7::py;
use super::{Ctx, DISCOVERY, Msg};
use crate::doccheck::Report;

/// The admission block of the hello's type-code table, or (0, -1).
fn block(ctx: &Ctx, report: &mut Report) -> (i128, i128) {
    let mut out = None;
    for (_, text) in find_sub(&ctx.hello_subs, r"type\s+codes") {
        for t in parse_tables(text) {
            for row in &t.rows {
                if row.len() > 1 && py::has_i("admission", &row[1]) {
                    let nums: Vec<i128> = py::finditer(r"\d+", &row[0], false).iter().filter_map(|m| py::int(m.all())).collect();
                    if let (Some(a), Some(b)) = (nums.first(), nums.last()) {
                        out = Some((*a, *b));
                    }
                }
            }
        }
    }
    out.unwrap_or_else(|| {
        report.fail("AC1", "the Hello section's Control message type codes table does not give the admission block");
        (0, -1)
    })
}

fn one(m: &Msg, ctx: &Ctx, blk: (i128, i128), codes: &mut BTreeMap<i128, String>, report: &mut Report) -> i128 {
    let (t, fl) = (&m.title, &m.fields);
    let keys: Vec<Option<&str>> = fl.iter().take(7).map(|f| key_of(&f.meaning)).collect();
    if fl.len() < 7 || keys != ctx.hdr_keys {
        report.fail("AC1", format!("'{t}': its table does not start with the common control header rows in the hello's order {:?}", ctx.hdr_keys));
    } else {
        for (a, b) in fl.iter().take(7).zip(&ctx.hdr) {
            if (a.off, a.bits, &a.ty) != (b.off, b.bits, &b.ty) {
                report.fail("AC1", format!("'{t}': header row '{}' differs from the common control header in offset, width or type", py::head(&b.meaning, 30)));
            }
        }
    }
    let mut cur: i128 = 0;
    for f in fl {
        if f.off != cur {
            report.fail("AC1", format!("'{t}': '{}' at byte {}, expected {cur} (rows must tile)", py::head(&f.meaning, 40), f.off));
        }
        if f.size_bits().rem_euclid(8) != 0 {
            report.fail("AC1", format!("'{t}': '{}' is not a whole number of bytes", py::head(&f.meaning, 40)));
        }
        cur = f.off.saturating_add(f.size_bits().div_euclid(8));
    }
    let stated = py::search(&format!(r"maximum\s+size[^.;\n]*?({NUM})\s*bytes"), &strip_fences(&m.text), true).map(|h| to_int(h.g(1)));
    match stated {
        None => report.fail("AC4", format!("'{t}': no sentence states its maximum size in bytes ('maximum size ... N bytes')")),
        Some(s) if s != Some(cur) => report.fail("AC4", format!("'{t}': states a maximum size of {} bytes; its table sums to {cur} (counted rows at their maximum counts)", s.unwrap_or_default())),
        Some(_) => {}
    }
    if ctx.bound != 0 && cur > ctx.bound {
        report.fail("AC4", format!("'{t}': its maximum size {cur} bytes is above the {}-byte control bound (C-17)", ctx.bound));
    }
    let row = fl.iter().find(|f| key_of(&f.meaning) == Some("type"));
    match row.and_then(|f| py::search(r"\d+", &f.valid, false)).and_then(|h| py::int(h.all())) {
        None => report.fail("AC1", format!("'{t}': the Message type row's valid range does not state the message's type code")),
        Some(code) => {
            if !(blk.0 <= code && code <= blk.1) || Some(code) == ctx.hello_type {
                report.fail("AC1", format!("'{t}': type code {code} is outside the admission block ({} to {}) the Hello section gives, or is the hello's code", blk.0, blk.1));
            }
            if let Some(other) = codes.get(&code) {
                report.fail("AC1", format!("'{t}' and '{other}' share type code {code}; the codes must be distinct"));
            }
            codes.insert(code, t.clone());
        }
    }
    if fl.len() >= 7 {
        let zero = |v: &str| py::has(r"\b0\b", v) && py::has_i(r"only|zero|=\s*0", v);
        if !zero(&fl[4].valid) {
            report.fail("AC2", format!("'{t}': the authentication scheme row does not fix the v1 value 0 (C-115)"));
        }
        if !zero(&fl[6].valid) {
            report.fail("AC2", format!("'{t}': the authentication tag row does not fix the v1 value of 128 zero bits (C-115)"));
        }
        if !py::has_i(r"\blive\b|liveness", &fl[5].meaning) {
            report.fail("AC2", format!("'{t}': the message counter row does not say it is live (C-115)"));
        }
    }
    cur
}

fn refusal_fields(ctx: &Ctx, report: &mut Report) {
    let rows: Vec<(&Msg, &super::Field)> = ctx.msgs.iter().flat_map(|m| m.fields.iter().map(move |f| (m, f))).filter(|(_, f)| py::has_i(r"refusal\s+reason", &f.meaning)).collect();
    if rows.is_empty() {
        report.fail("AC1", "no admission message carries a refusal reason field");
    }
    for (m, f) in rows {
        if f.bits != 8 {
            report.fail("AC1", format!("'{}': the refusal reason is {} bits; a u8 value of the enumeration", m.title, f.bits));
        }
        for n in DISCOVERY {
            if py::has(&format!(r"\b{n}\b"), &f.valid) && !py::has_i(r"\b(not|never|except|discover\w*|raised)\b", &f.valid) {
                report.fail("AC2", format!("'{}': the refusal reason's valid range names {n}, which is never carried by an admission refusal", m.title));
            }
        }
    }
}

/// AC1, AC2 and AC4 per message; records each message's total.
pub(crate) fn per_message(ctx: &mut Ctx, report: &mut Report) {
    let blk = block(ctx, report);
    let mut codes = BTreeMap::new();
    let totals: Vec<i128> = ctx.msgs.iter().map(|m| one(m, ctx, blk, &mut codes, report)).collect();
    for (m, t) in ctx.msgs.iter_mut().zip(totals) {
        m.total = t;
    }
    if !ctx.msgs.is_empty() && ctx.ad_para(&["C-115", "scheme|tag|reservation"]).is_none() {
        report.fail("AC2", "no sentence in the Admission messages section cites C-115 for the v1 values of the reservation");
    }
    refusal_fields(ctx, report);
}
