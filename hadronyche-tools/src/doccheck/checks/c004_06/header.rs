//! AC1, AC2, AC3 and AC5 of 004-06: the header's fields, the excluded
//! kinds, the size and tiling, the C-19 and C-122 citations.

use super::Ctx;
use super::wp5::big::Big;
use super::wp5::md::blocks;
use super::wp5::py::{char_slice, find_i, head, s, si};
use crate::doccheck::Report;
use crate::doccheck::text::rx;

/// AC1: every rule-1 field with its width, magic and version where 004-04
/// put them, a flags field naming gap and degraded.
pub(crate) fn ac1(ctx: &Ctx, report: &mut Report) {
    if !ctx.field_table_found {
        return;
    }
    let required: [(&str, &str, Option<i64>); 8] = [
        ("magic", r"\bmagic\b", Some(16)),
        ("version", r"\bversion\b", Some(16)),
        ("domain", r"\bdomain\b", Some(16)),
        ("stream number", r"stream\s*number|\bstream\b", Some(16)),
        ("session", r"\bsession\b", Some(32)),
        ("timeline epoch", r"\bepoch\b", Some(16)),
        ("copy index", r"copy[\s_-]*index", Some(2)),
        ("flags", r"\bflags?\b", None),
    ];
    let rows = &ctx.field_rows;
    let mut claimed = vec![false; rows.len()];
    for (label, pattern, need) in required {
        let Some(hit) = (0..rows.len()).find(|&k| !claimed[k] && si(pattern, &rows[k].text)) else {
            report.fail("AC1", format!("the header table has no {label} field"));
            continue;
        };
        claimed[hit] = true;
        let r = &rows[hit];
        if let Some(n) = need
            && r.bits != Big::from(n)
        {
            report.fail("AC1", format!("{label}: the table gives {} bits, the rule-1/C-10/C-11/C-07 width is {n} bits", r.bits));
        }
        if label == "magic" && !r.off.is_zero() {
            report.fail("AC1", format!("the magic sits at offset {}; offsets 0 and 2 are immovable (004-04)", r.off));
        }
        if label == "version" && r.off != Big::from(2) {
            report.fail("AC1", format!("the version sits at offset {}; offsets 0 and 2 are immovable (004-04)", r.off));
        }
        if label == "flags" && r.bits < Big::from(8) {
            report.fail("AC1", format!("the flags field is {} bits; a flags field is at least one byte", r.bits));
        }
    }
    if !si(r"0x4d4e|4d\s*4e", &ctx.section) {
        report.fail("AC1", "the section does not state the magic value 0x4d4e fixed by 004-04");
    }
    if !s(r"0x0100|\bv?1\.0\b", &ctx.section) {
        report.fail("AC1", "the section does not state the v1 version value 0x0100 (major 1 high byte, minor 0 low byte)");
    }
    let flags_text = (0..rows.len())
        .rev()
        .find(|&k| claimed[k] && si(r"\bflags?\b", &rows[k].text))
        .map_or("", |k| rows[k].text.as_str());
    for bit in ["gap", "degraded"] {
        let named = si(&format!(r"\b{bit}\b"), flags_text)
            || si(&format!(r"flag[^.\n]{{0,80}}\b{bit}\b|\b{bit}\b[^.\n]{{0,80}}flag"), &ctx.prose);
        if !named {
            report.fail("AC1", format!("the flags field does not name the {bit} bit"));
        }
    }
}

/// The negation words of AC2 (`NEGATION`, `re.I`).
const NEGATION: &str = r"\bno\b|\bnot\b|\bnever\b|\bwithout\b|\bexcluded?\b|\babsen(?:t|ce)\b|\bnone\b|\blacks?\b|\bomitted?\b|\bretired\b|\bcarries\s+neither";

/// The text around the first `re.I` match of `pattern` in `text`
/// (`before` characters before it, `after` after it).
fn around(pattern: &str, text: &str, before: usize, after: usize) -> Option<String> {
    let (a, b, _) = find_i(pattern, text)?;
    Some(char_slice(text, a.saturating_sub(before), b + after))
}

/// AC2: the note names each excluded kind with its absence, field by field;
/// no field of the table is destination-specific.
pub(crate) fn ac2(ctx: &Ctx, report: &mut Report) {
    if !ctx.open() {
        return;
    }
    let excluded = [
        ("module id", r"module\s*id"),
        ("destination address", r"destination|dest\b"),
        ("per-receiver acknowledgement", r"acknowledg|per[\s-]receiver|\back\b"),
    ];
    let prose_blocks = blocks(&ctx.prose);
    for (label, pattern) in excluded {
        let named = prose_blocks.iter().any(|block| match around(pattern, block, 120, 160) {
            Some(win) => si(NEGATION, &win) || si(NEGATION, block),
            None => false,
        });
        if !named {
            report.fail(
                "AC2",
                format!("no note states the header carries no {label} (rules 7 and 8); the excluded kinds must each be named with its absence"),
            );
        }
    }
    let walk = r"field\s+by\s+field|each\s+field|every\s+field|per\s+field|field\s+list|field\s+of\s+the\s+table|each\s+entry";
    if !si(walk, &ctx.prose) {
        report.fail(
            "AC2",
            "the excluded-kinds note is not field by field (the ticket asks for a field-by-field confirmation: 'each field', 'field by field', or a walk over the field list)",
        );
    }
    let specific = [r"\bmodule\s*id\b", r"destination", r"\back\b|acknowledg", r"receiver[\s-]specific|per[\s-]receiver\s+field"];
    for r in &ctx.field_rows {
        for pattern in specific {
            if let Some(win) = around(pattern, &r.text, 60, 80)
                && !si(NEGATION, &win)
            {
                report.fail(
                    "AC2",
                    format!("the header table carries a destination-specific field, contradicting the note: {}", head(&r.text, 110)),
                );
            }
        }
    }
}

/// The header size: (bits, bytes) when the table was found.
pub(crate) type Size = (Big, Big);

/// The bit intervals of the field rows: (start, end, text, approximate),
/// sorted as Python sorts the tuples; and whether a sub-byte field leaves
/// its bits unstated.
fn intervals(ctx: &Ctx, report: &mut Report) -> (Vec<(Big, Big, String, bool)>, bool) {
    let eight = Big::from(8);
    let mut out = Vec::new();
    let mut unstated = false;
    for r in &ctx.field_rows {
        let base = r.off.mul(&eight);
        let start = if r.bits < eight {
            match r.stated {
                Some((lo, hi)) => {
                    if r.bits != Big::from(hi - lo + 1) {
                        report.fail(
                            "AC3",
                            format!("sub-byte field states bits {hi}..{lo} ({} bits) but its width is {} bits: {}", hi - lo + 1, r.bits, head(&r.text, 90)),
                        );
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
                report.fail("AC3", format!("a field of {} bits is neither whole bytes nor stated bits: {}", r.bits, head(&r.text, 90)));
            }
            base
        };
        let end = start.add(&r.bits);
        out.push((start, end, r.text.clone(), r.bits < eight && r.stated.is_none()));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)).then_with(|| a.3.cmp(&b.3)));
    (out, unstated)
}

/// The sizes the prose states: `N bytes` (`re.I`) and `N B`.
fn stated_sizes(prose: &str) -> Vec<Big> {
    let int = |m: &str| super::wp5::py::py_int(&m.replace([' ', '_'], "")).expect("digits");
    let folded = super::wp5::py::fold_i(prose);
    let mut out: Vec<Big> = rx(r"(?i)(\d[\d _]*)\s*bytes").captures_iter(&folded).map(|c| int(&c[1])).collect();
    out.extend(rx(r"(\d[\d _]*)\s*B\b").captures_iter(prose).map(|c| int(&c[1])));
    out
}

/// AC3: the offsets tile the header without gap or overlap, and its size in
/// bytes is stated.
pub(crate) fn ac3(ctx: &Ctx, report: &mut Report) -> Option<Size> {
    if !ctx.field_table_found {
        return None;
    }
    let (ivs, unstated) = intervals(ctx, report);
    let mut cursor = Big::default();
    for (start, end, text, approx) in &ivs {
        if *start > cursor {
            let gap = start.sub(&cursor);
            let small = gap < Big::from(8);
            if !(small && unstated) {
                report.fail("AC3", format!("undeclared gap of {gap} bits before {}; the offsets must be contiguous", head(text, 60)));
            }
        } else if *start < cursor && !approx {
            report.fail("AC3", format!("overlapping fields at bit {start}: {}", head(text, 90)));
        }
        if *end > cursor {
            cursor = end.clone();
        }
    }
    let (bytes_floor, rem) = cursor.divmod_small(8);
    let bytes = if rem == 0 { bytes_floor.clone() } else { bytes_floor.add(&Big::from(1)) };
    if rem != 0 && !unstated {
        report.fail("AC3", format!("the fields end at bit {cursor}, not a whole byte boundary"));
    }
    let sizes = stated_sizes(&ctx.prose);
    if rem == 0 {
        if !sizes.contains(&bytes) {
            report.fail(
                "AC3",
                format!("the table's fields total {bytes} bytes, but the section never states that size in bytes (AC 3: the header size in bytes is stated)"),
            );
        }
    } else if sizes.is_empty() {
        report.fail("AC3", "the section states no header size in bytes");
    }
    Some((cursor, bytes))
}

/// AC5: the session meaning cites C-19; the degraded flag cites C-122.
pub(crate) fn ac5(ctx: &Ctx, report: &mut Report) {
    if !(ctx.open() && ctx.field_table_found) {
        return;
    }
    let session = ctx.field_rows.iter().find(|r| si(r"\bsession\b", &r.text));
    if !session.is_some_and(|r| s("C-19", &r.text)) {
        report.fail("AC5", "the session field's meaning does not cite C-19");
    }
    let cited = ctx.field_rows.iter().any(|r| si("degraded", &r.text) && s("C-122", &r.text))
        || si(r"degraded[^.\n]{0,200}C-122|C-122[^.\n]{0,200}degraded", &ctx.prose);
    if !cited {
        report.fail("AC5", "the degraded flag's meaning does not cite C-122");
    }
}
