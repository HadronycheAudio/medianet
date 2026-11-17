//! AC1 and AC2, per election message: the common header rows, tiling, the
//! stated maximum size, the type code in the election block, the v1
//! reservation values (script lines 496-547).

use std::collections::BTreeMap;

use super::{Ctx, Msg, NUM, int_add, size_bits, to_int};
use crate::doccheck::Report;
use crate::doccheck::family004::{Field, key_of};
use crate::doccheck::pycompat::parse_int;
use crate::doccheck::report::Flow;
use crate::doccheck::text::{has, has_i, head, rx};
use crate::doccheck::units::strip_fences;

fn header(m: &Msg, ctx: &Ctx, report: &mut Report) {
    let (t, fl) = (&m.title, &m.fields);
    let keys: Vec<Option<&str>> = fl.iter().take(7).map(|f| key_of(&f.meaning)).collect();
    if fl.len() < 7 || keys != ctx.hdr_keys {
        report.fail("AC1", format!("'{t}': its table does not start with the common control header rows in the hello's order {:?}", ctx.hdr_keys));
        return;
    }
    for (a, b) in fl.iter().take(7).zip(&ctx.hdr) {
        if (a.off, a.bits, &a.ty) != (b.off, b.bits, &b.ty) {
            report.fail("AC1", format!("'{t}': header row '{}' differs from the common control header in offset, width or type", head(&b.meaning, 30)));
        }
    }
}

fn tiling(m: &Msg, report: &mut Report) -> i128 {
    let mut cur: i128 = 0;
    for f in &m.fields {
        if f.off != cur {
            report.fail("AC1", format!("'{}': '{}' at byte {}, expected {cur} (rows must tile)", m.title, head(&f.meaning, 40), f.off));
        }
        if size_bits(f).rem_euclid(8) != 0 {
            report.fail("AC1", format!("'{}': '{}' is not a whole number of bytes", m.title, head(&f.meaning, 40)));
        }
        cur = int_add(f.off, size_bits(f).div_euclid(8));
    }
    cur
}

fn code(m: &Msg, ctx: &Ctx, seen: &mut BTreeMap<i128, String>, report: &mut Report) {
    let t = &m.title;
    let row = m.fields.iter().find(|f| key_of(&f.meaning) == Some("type"));
    // beyond i128: INT_LIMIT (recorded), and the check goes on without it
    let code = row.and_then(|f| rx(r"\d+").find(&f.valid)).and_then(|h| parse_int(h.as_str()).ok());
    let Some(code) = code else {
        report.fail("AC1", format!("'{t}': the Message type row's valid range does not state the message's type code"));
        return;
    };
    let (lo, hi) = ctx.elec;
    if !(lo <= code && code <= hi) || Some(code) == ctx.hello_type {
        report.fail("AC1", format!("'{t}': type code {code} is outside the block the Hello section gives election messages ({lo} to {hi}), or is the hello's code"));
    }
    if let Some(other) = seen.get(&code) {
        report.fail("AC1", format!("'{t}' and '{other}' share type code {code}; the codes must be distinct"));
    }
    seen.insert(code, t.clone());
}

fn reservation(fl: &[Field], t: &str, report: &mut Report) {
    if fl.len() < 7 {
        return;
    }
    let zero = |v: &str| has(r"\b0\b", v) && has_i(r"only|zero|=\s*0", v);
    if !zero(&fl[4].valid) {
        report.fail("AC2", format!("'{t}': the authentication scheme row does not fix the v1 value 0 (C-115)"));
    }
    if !zero(&fl[6].valid) {
        report.fail("AC2", format!("'{t}': the authentication tag row does not fix the v1 value of 128 zero bits (C-115)"));
    }
    if !has_i(r"\blive\b|liveness", &fl[5].meaning) {
        report.fail("AC2", format!("'{t}': the message counter row does not say it is live (C-115)"));
    }
}

/// AC1 and AC2 for every message; records each message's total.
pub(crate) fn messages(ctx: &mut Ctx, report: &mut Report) -> Flow<()> {
    let mut seen = BTreeMap::new();
    let mut totals = Vec::new();
    for m in &ctx.msgs {
        let t = &m.title;
        header(m, ctx, report);
        let cur = tiling(m, report);
        totals.push(cur);
        let plain = strip_fences(&m.text);
        let caps = rx(&format!(r"(?i)maximum\s+size[^.;\n]*?({NUM})\s*bytes")).captures(&plain);
        match caps {
            None => report.fail("AC1", format!("'{t}': no sentence states its maximum size in bytes ('maximum size ... N bytes')")),
            Some(h) => {
                // None: beyond i128 (INT_LIMIT, red by rule)
                if let Some(s) = to_int(h.get(1).map_or("", |m| m.as_str()))?
                    && s != cur
                {
                    report.fail("AC1", format!("'{t}': states a maximum size of {s} bytes; its table sums to {cur}"));
                }
            }
        }
        if let Some(b) = ctx.bound.filter(|&b| b != 0)
            && cur > b
        {
            report.fail("AC1", format!("'{t}': its maximum size {cur} bytes is above the {b}-byte control bound (C-17)"));
        }
        code(m, ctx, &mut seen, report);
        reservation(&m.fields, t, report);
    }
    for (m, total) in ctx.msgs.iter_mut().zip(totals) {
        m.total = total;
    }
    if !ctx.msgs.is_empty() && ctx.el_para(&["C-115", "scheme|tag|reservation"]).is_none() {
        report.fail("AC2", "no sentence in the Election messages section cites C-115 for the v1 values of the reservation");
    }
    Ok(())
}
