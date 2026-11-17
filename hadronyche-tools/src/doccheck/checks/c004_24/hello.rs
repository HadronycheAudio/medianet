//! The landed sections the regime stamp reads from: the constants (C-17,
//! C-13, C-18), the hello's common control header, its widths and its
//! type-code block, and the Rejection reasons with their Layers cells.

use super::Input;
use crate::doccheck::Report;
use crate::doccheck::family004::{Ctx, Field, fields_of, find_sub, is_field_table, subsections, tables, width_of};
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::rejection_members::{Enumeration, REJECTION_004_24};
use crate::doccheck::report::Flow;
use crate::doccheck::text::{full, has_i, ints, rx};

/// (offset, bits, type) of the hello's first seven header rows.
pub(crate) type HeaderSig = Vec<(i128, i128, String)>;

/// What the later criteria read.
pub(crate) struct Landed {
    pub ctrl_bound: Option<i128>,
    pub dscp: Option<i128>,
    pub refresh_s: Option<i128>,
    pub hdr_sig: HeaderSig,
    pub ver_bits: Option<i128>,
    pub module_bits: Option<i128>,
    /// The type-code block of 004-24, inclusive (`range(a, b + 1)`).
    pub block: Option<(i128, i128)>,
    pub members: Enumeration,
}

impl Landed {
    /// `c in BLOCK`.
    pub fn in_block(&self, c: i128) -> bool {
        self.block.is_some_and(|(a, b)| a <= c && c <= b)
    }

    /// `name in MEMBERS`.
    pub fn is_member(&self, name: &str) -> bool {
        self.members.contains(name)
    }

    /// `MEMBER_LAYERS.get(name, "")`.
    pub fn layer(&self, name: &str) -> &str {
        self.members.get(name).map_or("", |m| m.layers())
    }
}

/// `int(re.search(pat, const_val(id)).group(1))`.
fn grab(ctx: &Ctx, id: &str, pat: &str) -> Option<i128> {
    rx(pat).captures(ctx.consts.value(id)).and_then(|c| parse_int(&c[1]).ok())
}

/// `hello_width`: the bits of the first fixed-width row of the hello whose
/// meaning matches.
fn hello_width(hello: &str, pattern: &str) -> Option<i128> {
    for t in tables(hello) {
        for r in &t.rows {
            if r.cells.len() >= 5 && full(r"\d+", r.cell(0).py_strip()) && has_i(pattern, r.cell(3))
                && let Ok(Some((bits, mc))) = width_of(r.cell(1))
                    && mc.is_none_or(|m| m == 0) {
                        return Some(bits);
                    }
        }
    }
    None
}

fn header(subs: &[(String, String)], report: &mut Report) -> HeaderSig {
    let t = find_sub(subs, r"common\s+control\s+header")
        .first()
        .and_then(|(_, x)| tables(x).into_iter().find(|t| is_field_table(t.head())));
    let hdr = fields_of(t.as_ref(), "AC1", "the hello's common control header", report);
    let total: i128 = hdr.iter().map(Field::size_bits).sum();
    let tiles = hdr
        .iter()
        .enumerate()
        .all(|(i, f)| f.off == hdr[..i].iter().map(Field::size_bits).sum::<i128>().div_euclid(8));
    report.check("AC1", hdr.len() == 7 && total == 256 && tiles, format!("the hello's common control header is seven rows tiling 32 bytes (found {} rows)", hdr.len()));
    hdr.iter().take(7).map(|f| (f.off, f.bits, f.ty.clone())).collect()
}

fn block(subs: &[(String, String)], report: &mut Report) -> Option<(i128, i128)> {
    let mut block = None;
    if let Some((_, codes)) = find_sub(subs, r"control\s+message\s+type\s+codes").first() {
        for t in tables(codes) {
            for r in &t.rows {
                if r.cells.len() >= 3 && r.cell(2).contains("004-24")
                    && let Ok(n) = ints(r.cell(0))
                        && let (Some(a), Some(b)) = (n.first(), n.last()) {
                            block = Some((*a, *b));
                        }
            }
        }
    }
    let codes = block.map(|(a, b)| format!(" (codes {a} to {b})")).unwrap_or_default();
    report.check("AC1", block.is_some(), format!("the hello's type-code table gives the regime stamp messages a block assigned to 004-24{codes}"));
    block
}

/// Reads the landed sections, checking what the script checks of them.
pub(crate) fn landed(input: &Input, report: &mut Report) -> Flow<Landed> {
    let ctx = &input.ctx;
    let ctrl_bound = grab(ctx, "C-17", r"at most (\d+) bytes");
    let dscp = grab(ctx, "C-13", r"control CS5 \((\d+)\)");
    let refresh_s = grab(ctx, "C-18", r"every (\d+) s\b");
    let nz = |v: Option<i128>| v.is_some_and(|x| x != 0);
    report.check("AC1", nz(ctrl_bound) && nz(dscp) && nz(refresh_s), format!("the constants read: the control bound is {ctrl_bound:?} bytes (C-17), control DSCP {dscp:?} (C-13), the refresh period {refresh_s:?} s (C-18)"));
    let hello = ctx.body(ctx.span_of(r"^hello$", 2));
    let hsubs = subsections(&hello);
    let hdr_sig = header(&hsubs, report);
    let ver_bits = hello_width(&hello, r"regime\s+stamp\s+version\s+number");
    let module_bits = hello_width(&hello, r"module\s+id\s+\(C-11\)");
    report.check("AC1", ver_bits.is_some() && module_bits.is_some(), format!("the hello's layout gives the widths the stamp reuses: the stamp version number {ver_bits:?} bits, the module id {module_bits:?} bits (C-11)"));
    let block = block(&hsubs, report);
    let members = Enumeration::parse(&input.wire, REJECTION_004_24);
    report.check("AC4", members.has_exactly(13), format!("the Rejection reasons enumeration of wire.md has 13 members (found {})", members.count()));
    Ok(Landed { ctrl_bound, dscp, refresh_s, hdr_sig, ver_bits, module_bits, block, members })
}
