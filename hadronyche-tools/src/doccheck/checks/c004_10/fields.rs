//! The extension's three field rows and AC1 of 004-10: widths, types,
//! saturation, the hop maximum, the residence unit, the declared delay in
//! whole samples, the forwarding rule, the open item and its falsifier.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::big::Big;
use super::wp5::md::{is_field_table, parse_tables, width_to_bits};
use super::wp5::py::{fold_i, py_int, s, si};
use crate::doccheck::text::rx;

/// (key, Meaning pattern, label).
pub(crate) const KEYS: [(&str, &str, &str); 3] = [
    ("hop", r"\bhop\b", "hop count"),
    ("residence", "residence", "accumulated residence time"),
    ("delay", r"\bdelay\b", "accumulated declared delay"),
];

/// One extension field row.
#[derive(Debug, Clone)]
pub(crate) struct Ext {
    pub cells: Vec<String>,
    pub text: String,
    /// The first `\d+` of the Offset cell.
    pub off: Option<Big>,
    pub bits: Option<Big>,
    /// The Type cell, stripped and lower-cased.
    pub ty: String,
    pub valid: String,
}

/// The rows of every field table of the section, each giving the first key
/// not yet found whose pattern its Meaning cell matches; and whether a field
/// table was found.
pub(crate) fn extension_rows(section: &str) -> ([Option<Ext>; 3], bool) {
    let mut ext: [Option<Ext>; 3] = [None, None, None];
    let mut found = false;
    for (_, header, rows) in parse_tables(section) {
        if !is_field_table(&header) {
            continue;
        }
        found = true;
        for row in rows.iter().filter(|r| r.len() >= 4) {
            let Some(k) = (0..3).find(|&k| ext[k].is_none() && si(KEYS[k].1, &row[3])) else {
                continue;
            };
            ext[k] = Some(Ext {
                cells: row.clone(),
                text: row.join(" | "),
                off: rx(r"\d+").find(&row[0]).map(|m| py_int(m.as_str()).expect("digits")),
                bits: width_to_bits(&row[1]),
                ty: super::wp5::py::py_strip(&row[2]).to_lowercase(),
                valid: row.get(4).cloned().unwrap_or_default(),
            });
        }
    }
    (ext, found)
}

/// `[A-Za-z]` under `re.I`: ASCII letters and `İ`, `ı`, `ſ`, `K`.
fn letter_i(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, '\u{130}' | '\u{131}' | '\u{17f}' | '\u{212a}')
}

/// `re.search(UNIT_RE, t, re.I)`: a unit not preceded and not followed by a
/// letter (`(?<![A-Za-z])(ns|nanoseconds?|...)(?![A-Za-z])`). Every start is
/// tried; an alternative ending in `s?` may also give back its `s` before
/// the look-ahead, as Python's backtracking does.
pub(crate) fn has_unit(t: &str) -> bool {
    const ALTS: [&str; 8] = [
        "ns",
        "nanoseconds?",
        "µs",
        "us",
        "microseconds?",
        "ms",
        "milliseconds?",
        r"samples?\s+at\s+(?:the\s+)?(?:domain\s+rate|96)",
    ];
    let folded = fold_i(t);
    for (start, _) in folded.char_indices() {
        if folded[..start].chars().next_back().is_some_and(letter_i) {
            continue;
        }
        for alt in ALTS {
            let Some(m) = rx(&format!("(?i)^(?:{alt})")).find(&folded[start..]) else { continue };
            let mut ends = vec![m.end()];
            if alt.ends_with("s?") && m.as_str().ends_with(['s', 'S']) {
                ends.push(m.end() - 1);
            }
            if ends.iter().any(|&e| !folded[start + e..].chars().next().is_some_and(letter_i)) {
                return true;
            }
        }
    }
    false
}

/// Width, type and saturation of one field.
fn field_rules(e: &Ext, label: &str, report: &mut Report) {
    let whole = e.bits.as_ref().is_some_and(|b| *b > Big::default() && b.divmod_small(8).1 == 0);
    if !whole {
        report.fail("AC1", format!("the {label} field's width is not a whole number of bytes: {:?}", e.cells[1]));
    }
    match rx(r"^u(\d+)$").captures(&e.ty) {
        None => report.fail("AC1", format!("the {label} field's type is not a fixed-width unsigned integer (u8, u16, u32, u64): {:?}", e.ty)),
        Some(c) => {
            if let Some(b) = &e.bits
                && py_int(&c[1]).expect("digits") != *b
            {
                report.fail("AC1", format!("the {label} field's width ({b} bits) disagrees with its type {}", e.ty));
            }
        }
    }
    if !si("saturat", &e.text) || !si("wrap", &e.text) {
        report.fail("AC1", format!("the {label} field states no saturation rule (saturate, never wrap) in its own row"));
    }
}

/// The hop count's maximum: at most 64 and within its width.
fn hop_rules(e: &Ext, report: &mut Report) {
    let nums: Vec<Big> = rx(r"\d+").find_iter(&e.valid).map(|m| py_int(m.as_str()).expect("digits")).collect();
    let Some(max) = nums.into_iter().max() else {
        report.fail("AC1", "the hop count's valid range states no maximum");
        return;
    };
    if max > Big::from(64) {
        report.fail("AC1", format!("the hop count's maximum ({max}) exceeds the 64 modules of C-34"));
    }
    if let Some(b) = e.bits.as_ref().and_then(Big::to_i64)
        && max.bit_length() as i64 > b
    {
        report.fail("AC1", format!("the hop count's maximum ({max}) does not fit its width"));
    }
}

fn texts_with<'a>(ctx: &'a Ctx, e: &'a Ext, pattern: &str) -> Vec<&'a str> {
    std::iter::once(e.text.as_str()).chain(ctx.prose_blks.iter().filter(|b| si(pattern, b)).map(String::as_str)).collect()
}

/// AC1.
pub(crate) fn ac1(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    for (k, (_, _, label)) in KEYS.iter().enumerate() {
        match &ctx.ext[k] {
            None => report.fail("AC1", format!("no field-table row for the {label} field (hop count, accumulated residence time, accumulated declared delay)")),
            Some(e) => field_rules(e, label, report),
        }
    }
    if let Some(e) = &ctx.ext[0] {
        hop_rules(e, report);
    }
    if let Some(e) = &ctx.ext[1]
        && !texts_with(ctx, e, "residence").iter().any(|t| has_unit(t) && si(r"\[decision\]", t))
    {
        report.fail("AC1", "the residence-time unit is not stated (ns or samples at the domain rate) and tagged [decision] in the same row or block");
    }
    if let Some(e) = &ctx.ext[2] {
        let ok = texts_with(ctx, e, "delay")
            .iter()
            .any(|t| si(r"\bsamples?\b", t) && si(r"96[\s\u{a0}_]?000|domain\s+rate", t) && s("C-105", t) && si(r"\bwhole\b|\binteger\b", t));
        if !ok {
            report.fail("AC1", "the accumulated declared delay is not stated as a whole-sample sum at the domain rate (samples, 96 000 Hz, whole, C-105)");
        }
    }
    let b = &ctx.prose_blks;
    let forwarding = b.iter().any(|b| {
        si("forward", b) && si("hop", b) && si("residence", b) && si("delay", b) && si("saturat", b) && si(r"increment|\badds?\b|\+\s*1", b)
    });
    if !forwarding {
        report.fail("AC1", "no forwarding update rule (each processor increments the hop count, adds its residence time and its declared delay, saturating, never wrapping)");
    }
    if !b.iter().any(|b| si(r"open\s+item", b) && si(r"epic\s+004", b) && si("width", b) && si("falsif", b)) {
        report.fail("AC1", "the open item on the widths is not recorded with epic 004 as owner and a falsifier");
    }
    let falsifier = b.iter().any(|b| {
        si(r"saturat[^.;\n]{0,160}?maximum\s+hop|maximum\s+hop[^.;\n]{0,160}?saturat", b) && s(r"largest\s+L\b", b) && si("manifest", b)
    });
    if !falsifier {
        report.fail("AC1", "the falsifier is not the ticket's (the declared-delay field saturates over the maximum hop count at the largest L a manifest admits)");
    }
}
