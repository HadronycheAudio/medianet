//! The landed sections the regime stamp reads from: the constants (C-17,
//! C-13, C-18), the hello's common control header, its widths and its
//! type-code block, and the Rejection reasons with their Layers cells.

use std::collections::BTreeMap;

use super::py::{first_table, tables};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_27::{Ctx, Field, fields_of, find_sub, is_field_table, subsections, width_of};
use crate::doccheck::text::{full, has_i, ints, rx};

/// (offset, bits, type) of the hello's first seven header rows.
pub(crate) type HeaderSig = Vec<(i64, i64, String)>;

/// What the later criteria read.
pub(crate) struct Landed {
    pub ctrl_bound: Option<i64>,
    pub dscp: Option<i64>,
    pub refresh_s: Option<i64>,
    pub hdr_sig: HeaderSig,
    pub ver_bits: Option<i64>,
    pub module_bits: Option<i64>,
    /// The type-code block of 004-24, inclusive (Python's `range(a, b + 1)`).
    pub block: Option<(i64, i64)>,
    pub members: BTreeMap<String, i64>,
    pub layers: BTreeMap<String, String>,
}

impl Landed {
    /// `c in BLOCK`.
    pub fn in_block(&self, c: i64) -> bool {
        self.block.is_some_and(|(a, b)| a <= c && c <= b)
    }

    /// The Layers cell of a member, or `""`.
    pub fn layer(&self, name: &str) -> &str {
        self.layers.get(name).map_or("", String::as_str)
    }
}

fn grab(ctx: &Ctx, id: &str, pat: &str) -> Option<i64> {
    rx(pat).captures(ctx.consts.value(id)).and_then(|c| c[1].parse().ok())
}

fn hello_width(hello: &str, pattern: &str) -> Option<i64> {
    for t in tables(hello) {
        for r in &t.rows {
            if r.cells.len() >= 5 && full(r"\d+", r.cell(0).trim()) && has_i(pattern, r.cell(3))
                && let Some((bits, mc)) = width_of(r.cell(1))
                    && mc.is_none_or(|m| m == 0) {
                        return Some(bits);
                    }
        }
    }
    None
}

fn header(subs: &[(String, String)], report: &mut Report) -> HeaderSig {
    let hdr_t = find_sub(subs, r"common\s+control\s+header")
        .first()
        .and_then(|(_, x)| first_table(x, is_field_table));
    let hdr = fields_of(hdr_t.as_ref(), "AC1", "the hello's common control header", report);
    let total: i64 = hdr.iter().map(Field::size_bits).sum();
    let tiles = hdr.iter().enumerate().all(|(i, f)| f.off == hdr[..i].iter().map(Field::size_bits).sum::<i64>() / 8);
    report.check("AC1", hdr.len() == 7 && total == 256 && tiles, format!("the hello's common control header is seven rows tiling 32 bytes (found {} rows)", hdr.len()));
    hdr.iter().take(7).map(|f| (f.off, f.bits, f.ty.clone())).collect()
}

fn block(subs: &[(String, String)], report: &mut Report) -> Option<(i64, i64)> {
    let mut block = None;
    if let Some((_, codes)) = find_sub(subs, r"control\s+message\s+type\s+codes").first() {
        for t in tables(codes) {
            for r in &t.rows {
                if r.cells.len() >= 3 && r.cell(2).contains("004-24") {
                    let n = ints(r.cell(0));
                    if let (Some(a), Some(b)) = (n.first(), n.last()) {
                        block = Some((*a, *b));
                    }
                }
            }
        }
    }
    let codes = block.map(|(a, b)| format!(" (codes {a} to {b})")).unwrap_or_default();
    report.check("AC1", block.is_some(), format!("the hello's type-code table gives the regime stamp messages a block assigned to 004-24{codes}"));
    block
}

/// The rows of the Rejection reasons enumeration: a table row with at least
/// four cells, a numeric first cell and a `[A-Z][A-Za-z0-9]+` name; (value
/// by name, Layers cell by name). `pattern` names the section (level 2):
/// 004-24 and 004-27 use the unanchored `rejection[-\s]+reasons?`, 004-25,
/// 004-28 and 004-29 the anchored `^rejection\s+reasons$`.
pub(crate) fn rejection_members(ctx: &Ctx, pattern: &str) -> (BTreeMap<String, i64>, BTreeMap<String, String>) {
    let (mut members, mut layers) = (BTreeMap::new(), BTreeMap::new());
    for t in tables(&ctx.body(ctx.span_of(pattern, 2))) {
        for r in &t.rows {
            let (v, n) = (r.cell(0).trim(), r.cell(1).trim());
            if r.cells.len() >= 4 && full(r"\d+", v) && full(r"[A-Z][A-Za-z0-9]+", n) {
                members.insert(n.to_string(), v.parse().unwrap_or(0));
                layers.insert(n.to_string(), r.cell(3).to_string());
            }
        }
    }
    (members, layers)
}

fn rejection(ctx: &Ctx, report: &mut Report) -> (BTreeMap<String, i64>, BTreeMap<String, String>) {
    let (members, layers) = rejection_members(ctx, r"rejection[-\s]+reasons?");
    report.check("AC4", members.len() == 13, format!("the Rejection reasons enumeration of wire.md has 13 members (found {})", members.len()));
    (members, layers)
}

/// Reads the landed sections, checking what the script checks of them.
pub(crate) fn landed(ctx: &Ctx, report: &mut Report) -> Landed {
    let ctrl_bound = grab(ctx, "C-17", r"at most (\d+) bytes");
    let dscp = grab(ctx, "C-13", r"control CS5 \((\d+)\)");
    let refresh_s = grab(ctx, "C-18", r"every (\d+) s\b");
    let nz = |v: Option<i64>| v.is_some_and(|x| x != 0);
    report.check("AC1", nz(ctrl_bound) && nz(dscp) && nz(refresh_s), format!("the constants read: the control bound is {ctrl_bound:?} bytes (C-17), control DSCP {dscp:?} (C-13), the refresh period {refresh_s:?} s (C-18)"));
    let hello = ctx.body(ctx.span_of(r"^hello$", 2));
    let hsubs = subsections(&hello);
    let hdr_sig = header(&hsubs, report);
    let ver_bits = hello_width(&hello, r"regime\s+stamp\s+version\s+number");
    let module_bits = hello_width(&hello, r"module\s+id\s+\(C-11\)");
    report.check("AC1", ver_bits.is_some() && module_bits.is_some(), format!("the hello's layout gives the widths the stamp reuses: the stamp version number {ver_bits:?} bits, the module id {module_bits:?} bits (C-11)"));
    let block = block(&hsubs, report);
    let (members, layers) = rejection(ctx, report);
    Landed {
        ctrl_bound,
        dscp,
        refresh_s,
        hdr_sig,
        ver_bits,
        module_bits,
        block,
        members,
        layers,
    }
}
