//! The merged text the section types must not change: the stamp section's
//! header rows, its 1-to-64 section count, the block of the merged Section
//! type codes, the Rejection reasons pinned by name and value (the
//! library's [`REJ_FIXED`] and [`Enumeration::matches_fixed`]), and the
//! Reserved fields table. 004-25's own checks, built on the library.

use crate::doccheck::Report;
use crate::doccheck::family004::{tables, width_of};
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::rejection_members::{Enumeration, REJ_FIXED, REJECTION_004_25};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::tables::Row;
use crate::doccheck::text::{full, rx};

/// The stamp section's header rows as (offset, width) in bytes.
const HDR_STAMP: [(i128, i128); 14] = [
    (0, 2),
    (2, 2),
    (4, 2),
    (6, 1),
    (7, 1),
    (8, 8),
    (16, 16),
    (32, 8),
    (40, 8),
    (48, 4),
    (52, 1),
    (53, 1),
    (54, 2),
    (56, 2),
];

/// The merged text as the later criteria read it.
pub(crate) struct Merged {
    pub section_max: Option<i128>,
    pub rej: Enumeration,
    pub res_rows: Vec<Row>,
}

impl Merged {
    /// `name in REJ_VALUE`.
    pub fn is_member(&self, name: &str) -> bool {
        self.rej.contains(name)
    }

    /// `name in CONTROL_MEMBERS`.
    pub fn is_control(&self, name: &str) -> bool {
        self.rej.control_members().contains(&name)
    }
}

fn header_unchanged(stamp: &str, report: &mut Report) -> Flow<Option<i128>> {
    let sec_t = tables(stamp).into_iter().find(|t| t.rows.iter().any(|r| r.cells.len() > 3 && r.cell(3).starts_with("Section length")));
    let rows: Vec<Row> = sec_t.map(|t| t.rows).unwrap_or_default();
    let sig_ok = rows.len() >= 15
        && rows.iter().zip(HDR_STAMP).all(|(r, (off, b))| {
            let c0 = r.cell(0).py_strip();
            full(r"\d+", c0) && parse_int(r.cell(0)).ok() == Some(off) && matches!(width_of(r.cell(1)), Ok(Some((bits, _))) if bits == b * 8)
        });
    report.check("DoD", sig_ok, "the merged stamp section header rows (bytes 0 to 57) are unchanged");
    let secmax = rows.iter().find(|r| r.cells.len() > 3 && r.cell(3).starts_with("Section count"));
    let section_max = match secmax {
        // `_secmax[0][4]` on a four-cell row
        Some(r) if r.cells.len() < 5 => return crash("IndexError: list index out of range"),
        Some(r) => rx(r"1 to (\d+)").captures(r.cell(4)).and_then(|c| parse_int(&c[1]).ok()),
        None => None,
    };
    report.check("AC1", section_max == Some(64), format!("the merged stamp allows 1 to 64 sections per stamp (read {section_max:?})"));
    Ok(section_max)
}

/// The block the merged Section type codes give `ticket` (the last row of
/// any table headed "Codes…" whose third cell names it).
pub(crate) fn code_block(wire: &str, ticket: &str) -> Option<(i128, i128)> {
    let mut block = None;
    for t in tables(wire) {
        if !t.head().first().is_some_and(|h| h.to_lowercase().starts_with("codes")) {
            continue;
        }
        for r in t.rows.iter().filter(|r| r.cells.len() >= 3 && r.cell(2).contains(ticket)) {
            if let Some(c) = rx(r"(\d+)\s+to\s+(\d+)").captures(r.cell(0))
                && let (Ok(a), Ok(b)) = (parse_int(&c[1]), parse_int(&c[2]))
            {
                block = Some((a, b));
            }
        }
    }
    block
}

/// The rows of the last table of `wire` headed "Field…", "…location…" (the
/// Reserved fields table).
pub(crate) fn reserved_rows(wire: &str) -> Vec<Row> {
    tables(wire)
        .into_iter()
        .rfind(|t| {
            let h = t.head();
            h.first().is_some_and(|x| x.to_lowercase().starts_with("field")) && h.len() > 1 && h[1].to_lowercase().contains("location")
        })
        .map(|t| t.rows)
        .unwrap_or_default()
}

/// Reads and checks the merged text (`wire` is the whole normalised
/// wire.md, `stamp` the Regime stamp section); `between` runs where the
/// script checks its code block, between the section count and the
/// Rejection reasons.
pub(crate) fn merged(wire: &str, stamp: &str, report: &mut Report, between: impl FnOnce(&mut Report)) -> Flow<Merged> {
    let section_max = header_unchanged(stamp, report)?;
    between(report);
    let rej = Enumeration::parse(wire, REJECTION_004_25);
    report.check("DoD", rej.matches_fixed(&REJ_FIXED), "the merged Rejection reasons enumeration is unchanged (13 members, names and values)");
    let res_rows = reserved_rows(wire);
    report.check("DoD", res_rows.iter().any(|r| r.cell(0).to_lowercase().contains("section type codes")), "the merged Reserved fields table still records the section type codes row");
    Ok(Merged { section_max, rej, res_rows })
}
