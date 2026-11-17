//! The landed text 004-26 reads, never rewrites: the control bound (C-17),
//! the Hello's common control header, the manifest envelope of the
//! Admission messages, the Reject and refusal enumerations; and, at the
//! end, the landed admission tables' headers unchanged.

use super::{NUM, Wire, field_tables, model_rows, pipe_rows, subs, tile, to_int};
use crate::doccheck::Report;
use crate::doccheck::family004::find_sub;
use crate::doccheck::rejection_members::{Enumeration, REFUSAL_004_26, REJECTION_004_26};
use crate::doccheck::text::rx;
use crate::doccheck::ticket::Constants;

/// (offset, bits, type) of a header row.
pub(crate) type Sig = Vec<(i128, i128, String)>;

/// What the later criteria read.
pub(crate) struct Landed {
    /// `BOUND or 0`.
    pub bound: i128,
    pub hello: Option<Sig>,
    pub section_max: Option<i128>,
    pub length_max: Option<i128>,
    pub manifest_max: Option<i128>,
    pub reject: Enumeration,
    pub refusal: Enumeration,
}

impl Landed {
    /// `ALL_REASONS`.
    pub fn all_reasons(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.reject.names().chain(self.refusal.names()).collect();
        v.sort();
        v.dedup();
        v
    }
}

fn hello(wire: &Wire) -> Option<Sig> {
    let h = wire.section_lines(r"^Hello$", 2)?;
    let s = subs(&h);
    let sub = s.iter().find(|(t, _)| rx(r"(?i)common control header").is_match(t))?;
    let (rows, _) = model_rows(&pipe_rows(&sub.1));
    Some(rows.iter().take(7).map(|r| (r.off, r.bits, r.ty.clone())).collect())
}

fn envelope(wire: &Wire, bound: i128, report: &mut Report) -> (Option<i128>, Option<i128>, Option<i128>) {
    let (mut section_max, mut length_max, mut manifest_max) = (None, None, None);
    let mut env_bytes = None;
    let body = wire.section_lines(r"^Admission messages$", 2).and_then(|adm| {
        subs(&adm).into_iter().rfind(|(t, _)| rx(r"(?i)manifest section").is_match(t)).map(|(_, b)| b)
    });
    if let Some(t) = body.as_deref().and_then(|b| field_tables(b).into_iter().next()) {
        let (rows, _) = model_rows(&super::cells(&t));
        env_bytes = Some(tile(&rows).1.div_euclid(8));
        let range = |v: &str| rx(&format!("1 to ({NUM})")).captures(v).and_then(|c| to_int(&c[1]));
        for r in &rows {
            let m = r.meaning.to_lowercase();
            if m.starts_with("section count") {
                section_max = range(&r.valid);
            }
            if m.starts_with("manifest length") {
                length_max = range(&r.valid);
            }
            if m.starts_with("manifest bytes") {
                manifest_max = r.maxc;
            }
        }
    }
    let read = section_max.is_some() && length_max.is_some() && manifest_max.is_some();
    report.check("AC3", read, format!("the landed manifest section reads: section count max {section_max:?}, manifest length max {length_max:?}, manifest bytes max {manifest_max:?}"));
    if let (Some(m), Some(l)) = (manifest_max, length_max) {
        report.check("AC3", m == l, "the landed envelope's manifest bytes maximum equals its manifest length maximum");
        report.check("AC3", env_bytes == Some(bound), format!("the landed manifest section computes to {env_bytes:?} bytes at its maximum, the control bound {bound} (C-17)"));
    }
    (section_max, length_max, manifest_max)
}

/// Reads the landed text.
pub(crate) fn landed(consts: &str, wire: &Wire, report: &mut Report) -> Landed {
    let c = Constants::parse(consts);
    let bound = rx(&format!(r"(?i)at\s+most\s+({NUM})\s*bytes")).captures(c.value("C-17")).and_then(|m| to_int(&m[1]));
    report.check("AC3", bound.is_some(), format!("the control bound (C-17) reads as {bound:?} bytes from doc/design/02-constants.md"));
    let hello = hello(wire);
    report.check("DoD", hello.as_ref().is_some_and(|h| h.len() == 7), "the Hello common control header is readable (seven rows)");
    // `BOUND = BOUND or 0`
    let bound = bound.unwrap_or_default();
    let (section_max, length_max, manifest_max) = envelope(wire, bound, report);
    let reject = Enumeration::parse(&wire.raw, REJECTION_004_26);
    let refusal = Enumeration::parse(&wire.raw, REFUSAL_004_26);
    report.check("DoD", reject.has_exactly(13) && refusal.has_exactly(18), format!("the landed enumerations read as {} Reject names and {} refusal names (13 and 18)", reject.count(), refusal.count()));
    Landed {
        bound,
        hello,
        section_max,
        length_max,
        manifest_max,
        reject,
        refusal,
    }
}

/// The landed admission tables' first seven rows equal the Hello's.
pub(crate) fn unchanged(wire: &Wire, landed: &Landed, report: &mut Report) {
    let (Some(hello), Some(adm)) = (&landed.hello, wire.section_lines(r"^Admission messages$", 2)) else { return };
    let s = subs(&adm);
    for want in ["Admission request", "Admission accept", "Admission refusal", "Manifest section"] {
        let Some((_, body)) = find_sub(&s, want).first().copied() else {
            report.fail("DoD", format!("landed '{want}' table is present"));
            continue;
        };
        let Some(t) = field_tables(body).into_iter().next() else {
            report.fail("DoD", format!("landed '{want}' has a field table"));
            continue;
        };
        let (rows, _) = model_rows(&super::cells(&t));
        let head: Sig = rows.iter().take(7).map(|r| (r.off, r.bits, r.ty.clone())).collect();
        report.check("DoD", &head == hello, format!("landed '{want}' header rows equal the Hello common control header"));
    }
}
