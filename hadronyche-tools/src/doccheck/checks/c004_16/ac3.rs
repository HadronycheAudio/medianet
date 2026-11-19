//! AC3 (the alarm field and the transport role), Work 3 (the regime stamp
//! version number), the DoD's regime-stamp sentence and the behaviour
//! restated as fact (script lines 1082-1154 and 1267-1296).

use super::Ctx;
use super::ac1::row_for;
use crate::doccheck::Report;
use crate::doccheck::family004::{Field, find_sub, tables};
use crate::doccheck::pycompat::parse_int;
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::units::clean_start;

const ALARMS: [(&str, &str); 6] = [
    ("refusal", "refus"),
    ("interrupted", "interrupt"),
    ("dead", r"\bdead\b"),
    ("demoted", "demot"),
    ("cannot-meet-A", r"cannot[\s_-]*meet"),
    ("control-degraded", r"control[\s_-]*degrad"),
];

/// An alarm's value: `1 << int(bit)`, `int(hex, 16)` or `int(value)`. A
/// value beyond `i128` is recorded (`INT_LIMIT`) and the check goes on
/// with 0 (its verdict is red by rule).
fn encoding(rest: &str) -> Option<i128> {
    let big = |what: &str| {
        record_int_limit(what);
        0
    };
    if let Some(m) = rx(r"(?i)bit\s+(\d+)").captures(rest) {
        let t = m.get(1).map_or("", |m| m.as_str());
        return Some(match parse_int(t) {
            Ok(n) if (0..127).contains(&n) => 1_i128 << n,
            _ => big(&format!("1 << {t}")),
        });
    }
    if let Some(m) = rx(r"(?i)0x([0-9a-f]+)").captures(rest) {
        let t = m.get(1).map_or("", |m| m.as_str());
        return Some(i128::from_str_radix(t, 16).unwrap_or_else(|_| big(&format!("0x{t}"))));
    }
    rx(r"(?i)(?:value\s+)?\b(\d+)\b")
        .captures(rest)
        .map(|m| parse_int(m.get(1).map_or("", |m| m.as_str())).unwrap_or_else(|e| big(&e.to_string())))
}

/// `int.bit_length()`.
fn bit_length(v: i128) -> i128 {
    i128::from(128 - v.unsigned_abs().leading_zeros())
}

fn alarm_table(ctx: &Ctx, lay: &[Field], report: &mut Report) {
    let subs: Vec<&(String, String)> = ctx.subs.iter().filter(|(t, _)| has_i("alarm", t)).collect();
    if subs.is_empty() {
        report.fail("AC3", "no '### Alarm field' subsection with the encoding of the six connection alarms");
        return;
    }
    let Some(at) = subs.iter().find_map(|(_, x)| tables(x).into_iter().find(|t| t.rows.len() >= 6)) else {
        report.fail("AC3", "the alarm subsection has no table with one row per alarm");
        return;
    };
    let mut values: Vec<(&str, i128)> = Vec::new();
    for (name, pat) in ALARMS {
        let Some(hit) = at.rows.iter().map(|r| &r.cells).find(|r| !r.is_empty() && has_i(pat, &r[0])) else {
            report.fail("AC3", format!("the alarm table has no row for the alarm '{name}' (C-173)"));
            continue;
        };
        let rest = hit.get(1).map_or("", String::as_str);
        match encoding(rest) {
            Some(v) => values.push((name, v)),
            None => report.fail("AC3", format!("the alarm '{name}' has no bit or value")),
        }
    }
    let mut distinct: Vec<i128> = values.iter().map(|(_, v)| *v).collect();
    distinct.sort();
    distinct.dedup();
    if distinct.len() != values.len() {
        report.fail("AC3", format!("the alarm encodings are not distinct: {values:?}"));
    }
    if values.iter().any(|(_, v)| *v == 0) {
        report.fail("AC3", "an alarm is encoded as zero, which means no alarm");
    }
    if let (Some(i), Some(max)) = (row_for(lay, "alarm", None), distinct.last())
        && bit_length(*max) > lay[i].bits
    {
        report.fail("AC3", "an alarm value does not fit the alarm field's width");
    }
}

fn transport_role(ctx: &Ctx, report: &mut Report) {
    match find_sub(&ctx.subs, r"transport\s+role").first() {
        None => report.fail("AC3", "no '### Transport role' subsection with the proposed role values"),
        Some((_, x)) => {
            if tables(x).first().is_none_or(|t| t.rows.len() < 2) {
                report.fail("AC3", "the transport role subsection has no table with its values");
            }
            if !has_i("proposal", x) {
                report.fail("AC3", "the transport role values are not recorded as a proposal");
            }
        }
    }
    let open = ctx.paras.iter().find(|t| {
        rx(r"(?i)^(?:\s*\*{0,2}\s*open\s+items?\b)").is_match(&clean_start(t)) && has_i(r"transport\s+role", t)
    });
    let Some(open) = open else {
        report.fail("AC3", "no paragraph starting 'Open item' states the transport role open question");
        return;
    };
    if !has(r"\b005\b", open) || !has_i("owner", open) {
        report.fail("AC3", "the transport role open item does not name epic 005 as its confirming owner");
    }
    if !has_i(r"epic\s+005\s+needs\s+a\s+role\s+the\s+encoding\s+cannot\s+express", open) {
        report.fail("AC3", "the transport role open item lacks its falsifier 'epic 005 needs a role the encoding cannot express'");
    }
}

/// AC3: the alarm field and the transport role.
pub(crate) fn alarms_and_role(ctx: &Ctx, lay: &[Field], report: &mut Report) {
    alarm_table(ctx, lay, report);
    transport_role(ctx, report);
}

const BEHAVIOUR: [(&str, &[&str]); 5] = [
    ("hello every 1 s (C-30)", &[r"\b1\s*s\b", "C-30"]),
    ("multicast with TTL 1 and unicast over IPv4 (C-100)", &[r"TTL\s+1", "C-100", "unicast", "IPv4"]),
    ("DSCP CS5 (40) (C-13)", &[r"CS5\s*\(40\)", "C-13"]),
    ("binary, not the POC's text gossip ([code])", &["binary", r"\btext\b", r"\[code\]", "HADRO/1|POC"]),
    ("64-entry peer map (C-34)", &[r"\b64\b", "C-34"]),
];

/// Work 3, the regime-stamp DoD sentence and the behaviour restated.
pub(crate) fn work3_and_behaviour(ctx: &Ctx, report: &mut Report) {
    if ctx.para_with(&[r"regime\s+stamp", r"\bstale\b", r"\b(equal|equality|serial|wrap)"]).is_none() {
        report.fail("Work3", "no sentence gives the comparison rule of the regime stamp version number: a receiver treats its copy as stale by equality, serial comparison or wrap");
    }
    if ctx.para_with(&[r"regime\s+stamp", r"version\s+number|not\s+applicable"]).is_none() {
        report.fail("DoD", "no sentence says the regime stamp fields the hello touches are documented (version number) or not applicable");
    }
    if ctx.para_with(&["C-18", r"version\s+number", "own"]).is_none() {
        report.fail("Work3", "no sentence states that the stamp travels in its own message and the hello carries only its version number (C-18)");
    }
    let plain = &ctx.plain;
    for (name, pats) in BEHAVIOUR {
        if let Some(p) = pats.iter().find(|p| !has_i(p, plain)) {
            report.fail("Behaviour", format!("behaviour not restated as fact: {name} (missing '{p}')"));
        }
    }
    let k = &ctx.k;
    if let Some(v) = k.hello_period_s
        && !has(&format!(r"\b{v}\s*s\b"), plain)
    {
        report.fail("Behaviour", "the hello period is not restated with the value of C-30");
    }
    if let Some(v) = k.dscp_control
        && !has(&format!(r"\({v}\)"), plain)
    {
        report.fail("Behaviour", "CS5 is not stated with the value of C-13");
    }
    if let Some(v) = k.peers_max
        && !has(&format!(r"\b{v}\b"), plain)
    {
        report.fail("Behaviour", "the peer map bound is not restated with the value of C-34");
    }
}
