//! AC2 of 004-08: the slice header's field table — position 64 bits and the
//! five rule-2 fields, offsets from byte 16 tiling contiguously, the valid
//! ranges of frames, packet index and packet count.

use std::collections::HashMap;

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::big::Big;
use super::wp5::md::{bit_positions, is_field_table, parse_int_token, parse_tables, width_to_bits};
use super::wp5::py::{head, py_int, s, si};
use crate::doccheck::text::rx;

/// One field row: (cells, text, offset bytes, width bits, stated bits).
#[derive(Debug, Clone)]
pub(crate) struct Row {
    pub cells: Vec<String>,
    pub text: String,
    pub off: Big,
    pub bits: Big,
    pub stated: Option<(i64, i64)>,
}

/// The rows of the first field table of 4+ rows that all read.
pub(crate) type Fields = Vec<Row>;

/// The field table of the section.
pub(crate) fn field_table(section: &str) -> Option<Fields> {
    for (_, header, rows) in parse_tables(section) {
        if !is_field_table(&header) || rows.len() < 4 {
            continue;
        }
        let mut out = Vec::new();
        for row in &rows {
            let (Some(off), Some(bits)) = (parse_int_token(&row[0]), row.get(1).and_then(|c| width_to_bits(c))) else {
                out.clear();
                break;
            };
            let text = row.join(" | ");
            out.push(Row { cells: row.clone(), stated: bit_positions(&text), text, off, bits });
        }
        if !out.is_empty() {
            return Some(out);
        }
    }
    None
}

/// The required fields, each claimed once; returns label → cells.
fn required(rows: &[Row], report: &mut Report) -> HashMap<&'static str, Vec<String>> {
    let req: [(&str, &str, Option<i64>); 5] = [
        ("position", r"\bposition\b", Some(64)),
        ("frames", r"\bframes?\b", None),
        ("packet index", r"packet\s+index", None),
        ("packet count", r"packet\s+count", None),
        ("send timestamp", r"send[\s_-]*timestamp", None),
    ];
    let mut claimed = vec![false; rows.len()];
    let mut by_label = HashMap::new();
    for (label, pattern, need) in req {
        let Some(k) = (0..rows.len()).find(|&k| !claimed[k] && si(pattern, &rows[k].text)) else {
            report.fail("AC2", format!("the slice header table has no {label} field (rule 2)"));
            continue;
        };
        claimed[k] = true;
        by_label.insert(label, rows[k].cells.clone());
        if let Some(n) = need
            && rows[k].bits != Big::from(n)
        {
            report.fail("AC2", format!("{label}: the table gives {} bits; rule 2 fixes position at 64 bits", rows[k].bits));
        }
    }
    by_label
}

/// The tiling from bit 128; returns the end cursor.
fn tiling(rows: &[Row], report: &mut Report) -> Big {
    let eight = Big::from(8);
    let mut ivs = Vec::new();
    let mut unstated = false;
    for r in rows {
        let base = r.off.mul(&eight);
        let start = if r.bits < eight {
            match r.stated {
                Some((lo, hi)) => {
                    if r.bits != Big::from(hi - lo + 1) {
                        report.fail("AC2", format!("sub-byte field states bits {hi}..{lo} ({} bits) but its width is {} bits: {}", hi - lo + 1, r.bits, head(&r.text, 90)));
                    }
                    base.add(&Big::from(lo))
                }
                None => {
                    unstated = true;
                    base
                }
            }
        } else {
            if r.bits.divmod_small(8).1 != 0 {
                report.fail("AC2", format!("a field of {} bits is neither whole bytes nor stated bits: {}", r.bits, head(&r.text, 90)));
            }
            base
        };
        ivs.push((start.clone(), start.add(&r.bits), r.text.clone(), r.bits < eight && r.stated.is_none()));
    }
    ivs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)).then_with(|| a.3.cmp(&b.3)));
    let mut cursor = Big::from(128);
    for (start, end, text, approx) in &ivs {
        if *start > cursor {
            let gap = start.sub(&cursor);
            if !(gap < eight && unstated) {
                report.fail("AC2", format!("undeclared gap of {gap} bits before {}; the tiling must be contiguous", head(text, 60)));
            }
        } else if *start < cursor && !approx {
            report.fail("AC2", format!("overlapping fields at bit {start}: {}", head(text, 90)));
        }
        if *end > cursor {
            cursor = end.clone();
        }
    }
    if cursor.divmod_small(8).1 != 0 && !unstated {
        report.fail("AC2", format!("the fields end at bit {cursor}, not a whole byte boundary"));
    }
    cursor
}

/// The valid-range cells of frames, packet index and packet count.
fn ranges(by_label: &HashMap<&str, Vec<String>>, report: &mut Report) {
    if let Some(cell) = by_label.get("frames").and_then(|c| c.get(4)) {
        let lower = s(r"(?:^|[^\d])1\s*(?:to|–|—|\.\.\.?|through|-)\s*\d|≥\s*1\b|>=\s*1\b|at\s+least\s+1\b", cell);
        let upper = rx(r"(\d[\d _]*)").captures_iter(cell).any(|c| py_int(&c[1].replace([' ', '_'], "")).expect("digits") > Big::from(1))
            || s(r"at\s+most\s+\d+", cell);
        if !lower {
            report.fail("AC2", format!("the frames field's valid range does not state the lower bound 1 (frames >= 1): {cell:?}"));
        }
        if !upper {
            report.fail("AC2", format!("the frames field's valid range states no upper bound (a fixed maximum): {cell:?}"));
        }
    }
    if let Some(cell) = by_label.get("packet index").and_then(|c| c.get(4))
        && !si(r"packet\s+count|count\s*[−–-]\s*1|index\s*<|<\s*(?:the\s+)?packet|below\s+(?:the\s+)?packet\s+count", cell)
    {
        report.fail("AC2", format!("the packet index's valid range does not bound it by the packet count (packet index < packet count): {cell:?}"));
    }
    if let Some(cell) = by_label.get("packet count").and_then(|c| c.get(4))
        && (!s(r"\b1\b", cell) || !si("default", cell))
    {
        report.fail("AC2", format!("the packet count's valid range does not state the default regime's value 1: {cell:?}"));
    }
}

/// AC2; returns `slice_bytes` when the table was checked.
pub(crate) fn ac2(ctx: &Ctx, report: &mut Report) -> Option<Big> {
    let rows = ctx.field.as_ref().filter(|_| ctx.open)?;
    let by_label = required(rows, report);
    let first = rows.iter().map(|r| r.off.clone()).min().expect("4+ rows");
    if first != Big::from(16) {
        report.fail("AC2", format!("the slice header table starts at byte {first}; the packet header occupies bytes 0 to 15 (004-06), so the slice-header offsets continue at byte 16"));
    }
    let cursor = tiling(rows, report);
    ranges(&by_label, report);
    let (q, _) = cursor.sub(&Big::from(128)).add(&Big::from(7)).divmod_small(8);
    Some(q)
}
