//! The merged text the section types must not change (shared by 004-25 and
//! 004-28): the stamp section's header rows, its 1-to-64 section count, a
//! block of the merged Section type codes, the Rejection reasons pinned by
//! name and value ([`REJ_FIXED`]), and the Reserved fields table.

use std::collections::BTreeMap;

use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::landed::rejection_members;
use crate::doccheck::checks::c004_24::py::tables;
use crate::doccheck::checks::c004_27::{Ctx, width_of};
use crate::doccheck::tables::Row;
use crate::doccheck::text::{full, rx};

/// The stamp section's header rows as (offset, width) in bytes.
pub(crate) const HDR_STAMP: [(i64, i64); 14] = [
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

/// The thirteen members of the closed Rejection reasons enumeration, pinned
/// by name and value (004-25's and 004-28's `REJ_FIXED`).
pub(crate) const REJ_FIXED: [(&str, i64); 13] = [
    ("TooShort", 0),
    ("BadMagic", 1),
    ("VersionMismatch", 2),
    ("ForeignDomain", 3),
    ("UnconfiguredDomain", 4),
    ("UnknownStream", 5),
    ("UnadmittedSource", 6),
    ("BadLength", 7),
    ("BadChecksum", 8),
    ("BadPacketIndex", 9),
    ("WrongEpoch", 10),
    ("OversizeControl", 11),
    ("ReservedNonZero", 12),
];

/// The merged text as the later criteria read it.
pub(crate) struct Merged {
    pub section_max: Option<i64>,
    pub rej_value: BTreeMap<String, i64>,
    pub rej_layers: BTreeMap<String, String>,
    pub res_rows: Vec<Row>,
}

impl Merged {
    /// `CONTROL_MEMBERS`: the members whose Layers cell names control.
    pub fn is_control(&self, name: &str) -> bool {
        self.rej_layers.get(name).is_some_and(|l| l.contains("control"))
    }
}

/// `sorted(REJ_VALUE.items(), key=value) == REJ_FIXED` (and non-empty):
/// with distinct pinned values this is equality of the two sets of pairs.
pub(crate) fn rejections_pinned(values: &BTreeMap<String, i64>) -> bool {
    values.len() == REJ_FIXED.len() && REJ_FIXED.iter().all(|(n, v)| values.get(*n) == Some(v))
}

fn header_unchanged(stamp: &str, report: &mut Report) -> Result<Option<i64>, Stop> {
    let sec_t = tables(stamp).into_iter().find(|t| t.rows.iter().any(|r| r.cells.len() > 3 && r.cell(3).starts_with("Section length")));
    let rows: Vec<Row> = sec_t.map(|t| t.rows).unwrap_or_default();
    let sig_ok = rows.len() >= 15
        && rows.iter().zip(HDR_STAMP).all(|(r, (off, b))| {
            let c0 = r.cell(0).trim();
            full(r"\d+", c0) && c0.parse::<i64>().ok() == Some(off) && width_of(r.cell(1)).map(|w| w.0) == Some(b * 8)
        });
    report.check("DoD", sig_ok, "the merged stamp section header rows (bytes 0 to 57) are unchanged");
    let secmax = rows.iter().find(|r| r.cells.len() > 3 && r.cell(3).starts_with("Section count"));
    // Python reads `_secmax[0][4]`, which raises on a four-cell row.
    let section_max = match secmax {
        Some(r) if r.cells.len() < 5 => return Err(Stop("IndexError: the Section count row has no fifth cell".into())),
        Some(r) => rx(r"1 to (\d+)").captures(r.cell(4)).and_then(|c| c[1].parse().ok()),
        None => None,
    };
    report.check("AC1", section_max == Some(64), format!("the merged stamp allows 1 to 64 sections per stamp (read {section_max:?})"));
    Ok(section_max)
}

/// The block the merged Section type codes give `ticket` (the last row of
/// any table headed "Codes…" whose third cell names it).
pub(crate) fn code_block(wire: &str, ticket: &str) -> Option<(i64, i64)> {
    let mut block = None;
    for t in tables(wire) {
        if !t.head().first().is_some_and(|h| h.to_lowercase().starts_with("codes")) {
            continue;
        }
        for r in &t.rows {
            if r.cells.len() >= 3 && r.cell(2).contains(ticket)
                && let Some(c) = rx(r"(\d+)\s+to\s+(\d+)").captures(r.cell(0))
                    && let (Ok(a), Ok(b)) = (c[1].parse(), c[2].parse()) {
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
        .into_iter().rfind(|t| {
            let h = t.head();
            h.first().is_some_and(|x| x.to_lowercase().starts_with("field")) && h.len() > 1 && h[1].to_lowercase().contains("location")
        })
        .map(|t| t.rows)
        .unwrap_or_default()
}

/// Reads and checks the merged text (`wire` is the whole normalised
/// wire.md, `stamp` the Regime stamp section).
/// `before_rejections` runs between the section count and the Rejection
/// reasons, where the script checks its own code block.
pub(crate) fn merged(ctx: &Ctx, wire: &str, stamp: &str, report: &mut Report, before_rejections: impl FnOnce(&mut Report)) -> Result<Merged, Stop> {
    let section_max = header_unchanged(stamp, report)?;
    before_rejections(report);
    let (rej_value, rej_layers) = rejection_members(ctx, r"^rejection\s+reasons$");
    report.check("DoD", rejections_pinned(&rej_value), "the merged Rejection reasons enumeration is unchanged (13 members, names and values)");
    let res_rows = reserved_rows(wire);
    report.check("DoD", res_rows.iter().any(|r| r.cell(0).to_lowercase().contains("section type codes")), "the merged Reserved fields table still records the section type codes row");
    Ok(Merged {
        section_max,
        rej_value,
        rej_layers,
        res_rows,
    })
}
