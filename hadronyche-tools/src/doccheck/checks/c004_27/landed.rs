//! The landed sections the probe pair reads from: the constants (C-06,
//! C-13, C-17), the hello's common control header and type-code block, the
//! admission request's intended packet size, the refusal and rejection
//! enumerations, and wire.toml.

use std::collections::BTreeMap;

use toml::Table as TomlTable;

use super::{Ctx, Field, fields_of, find_sub, first_field_table, subsections, tables};
use crate::doccheck::Report;
use crate::doccheck::text::{first_int, full, has_i, ints, rx};
use crate::doccheck::wire_toml::{WireToml, int, text};

/// What the later criteria read.
pub(crate) struct Landed {
    pub fallback: Option<i64>,
    pub bound: Option<i64>,
    pub dscp: Option<i64>,
    /// (offset, bits, type) of the hello's first seven header rows.
    pub hdr_sig: HeaderSig,
    /// The type-code block of the probe pair (inclusive).
    pub block: Option<(i64, i64)>,
    pub req_intended: Vec<Field>,
    pub refusals: BTreeMap<String, i64>,
    pub members: BTreeMap<String, i64>,
    pub layers: BTreeMap<String, String>,
    pub tmsgs: Vec<TomlTable>,
}

impl Landed {
    /// True when `code` is in the probe block.
    pub fn in_block(&self, code: Option<i64>) -> bool {
        matches!((self.block, code), (Some((a, b)), Some(c)) if a <= c && c <= b)
    }
}

const IPV4_UDP_HEADERS: i64 = 28;

fn constants(ctx: &Ctx, report: &mut Report) -> (Option<i64>, Option<i64>, Option<i64>) {
    let fallback = first_int(ctx.consts.value("C-06"));
    let grab = |id: &str, pat: &str| rx(pat).captures(ctx.consts.value(id)).and_then(|c| c[1].parse().ok());
    let bound: Option<i64> = grab("C-17", r"at most (\d+) bytes");
    let dscp: Option<i64> = grab("C-13", r"control CS5 \((\d+)\)");
    let ok = matches!((fallback, bound), (Some(f), Some(b)) if f != 0 && b != 0 && f - IPV4_UDP_HEADERS == b);
    report.check("AC1", ok, format!("the fallback arithmetic holds: C-06's {fallback:?} bytes less {IPV4_UDP_HEADERS} bytes of IPv4 and UDP headers is C-17's {bound:?} bytes of UDP payload"));
    (fallback, bound, dscp)
}

/// (offset, bits, type) of each header row.
type HeaderSig = Vec<(i64, i64, String)>;

fn hello(ctx: &Ctx, report: &mut Report) -> (HeaderSig, Option<(i64, i64)>) {
    let hsubs = subsections(&ctx.body(ctx.span_of(r"^hello$", 2)));
    let hdr_table = find_sub(&hsubs, r"common\s+control\s+header").first().and_then(|(_, x)| first_field_table(x));
    let hdr = fields_of(hdr_table.as_ref(), "AC1", "the hello's common control header", report);
    let bytes: i64 = hdr.iter().map(Field::size_bits).sum::<i64>() / 8;
    let tiles = hdr
        .iter()
        .enumerate()
        .all(|(i, f)| f.off == hdr[..i].iter().map(Field::size_bits).sum::<i64>() / 8);
    report.check("AC1", hdr.len() == 7 && bytes == 32 && tiles, format!("the hello's common control header is seven rows tiling 32 bytes (found {} rows, {bytes} bytes)", hdr.len()));
    let sig = hdr.iter().take(7).map(|f| (f.off, f.bits, f.ty.clone())).collect();
    let mut block = None;
    let mut assigned = String::new();
    if let Some((_, codes)) = find_sub(&hsubs, r"control\s+message\s+type\s+codes").first() {
        for t in tables(codes) {
            for r in &t.rows {
                if r.cells.len() >= 3 && has_i("probe", r.cell(1)) {
                    let n = ints(r.cell(0));
                    if let (Some(a), Some(b)) = (n.first(), n.last()) {
                        block = Some((*a, *b));
                        assigned = r.cell(2).to_string();
                    }
                }
            }
        }
    }
    report.check("AC1", block.is_some() && assigned.contains("004-27"), format!("the hello's type-code table gives the path-MTU probe pair a block assigned to 004-27 ({block:?})"));
    (sig, block)
}

fn admission(ctx: &Ctx, report: &mut Report) -> (Vec<Field>, BTreeMap<String, i64>) {
    let asubs = subsections(&ctx.body(ctx.span_of(r"^admission\s+messages$", 2)));
    let req_table = find_sub(&asubs, r"^admission\s+request$").first().and_then(|(_, x)| first_field_table(x));
    let req = fields_of(req_table.as_ref(), "AC1", "the admission request", report);
    let intended: Vec<Field> = req.into_iter().filter(|f| rx(r"(?i)^\s*intended\s+packet\s+size").is_match(&f.meaning)).collect();
    let ok = intended.len() == 1 && intended[0].bits == 16 && intended[0].valid.contains("65507");
    report.check("AC1", ok, "the admission request still carries the intended packet size as a u16 with a maximum of 65 507 (the size the probe pair takes)");
    let mut refusals = BTreeMap::new();
    if let Some((_, rf)) = find_sub(&asubs, r"refusal\s+reasons").first() {
        for t in tables(rf) {
            for r in &t.rows {
                let (v, n) = (r.cell(0).trim(), r.cell(1).trim());
                if r.cells.len() >= 2 && full(r"\d+", v) && full(r"[A-Z][A-Za-z0-9]+", n) {
                    refusals.insert(n.to_string(), v.parse().unwrap_or(0));
                }
            }
        }
    }
    (intended, refusals)
}

fn rejection(ctx: &Ctx, report: &mut Report) -> (BTreeMap<String, i64>, BTreeMap<String, String>) {
    let (mut members, mut layers) = (BTreeMap::new(), BTreeMap::new());
    for t in tables(&ctx.body(ctx.span_of(r"rejection[-\s]+reasons?", 2))) {
        for r in &t.rows {
            let (v, n) = (r.cell(0).trim(), r.cell(1).trim());
            if r.cells.len() >= 4 && full(r"\d+", v) && full(r"[A-Z][A-Za-z0-9]+", n) {
                members.insert(n.to_string(), v.parse().unwrap_or(0));
                layers.insert(n.to_string(), r.cell(3).to_string());
            }
        }
    }
    report.check("AC3", members.len() == 13, format!("the Rejection reasons enumeration of wire.md has 13 members (found {})", members.len()));
    (members, layers)
}

fn toml_side(ctx: &Ctx, refusals: &BTreeMap<String, i64>, report: &mut Report) -> Vec<TomlTable> {
    let parsed = match WireToml::parse(ctx.toml_text.as_deref().unwrap_or("")) {
        Ok(t) => Some(t),
        Err(e) => {
            report.fail("AC4", format!("wire.toml does not parse: {e}"));
            None
        }
    };
    let tmsgs: Vec<TomlTable> = parsed.as_ref().map(|t| t.messages().into_iter().cloned().collect()).unwrap_or_default();
    let trefs: BTreeMap<String, i64> = parsed
        .as_ref()
        .map(|t| {
            t.array("refusal")
                .into_iter()
                .filter(|r| r.contains_key("name") && r.contains_key("value"))
                .filter_map(|r| int(r, "value").map(|v| (text(r, "name").to_string(), v)))
                .collect()
        })
        .unwrap_or_default();
    let ok = !refusals.is_empty() && refusals.len() == 18 && &trefs == refusals;
    report.check("AC4", ok, format!("the refusal enumeration of wire.toml equals the 18 names and values of wire.md's Refusal reasons (md {}, toml {})", refusals.len(), trefs.len()));
    tmsgs
}

/// Reads the landed sections, checking what the script checks of them.
pub(crate) fn landed(ctx: &Ctx, report: &mut Report) -> Landed {
    let (fallback, bound, dscp) = constants(ctx, report);
    let (hdr_sig, block) = hello(ctx, report);
    let (req_intended, refusals) = admission(ctx, report);
    let (members, layers) = rejection(ctx, report);
    let tmsgs = toml_side(ctx, &refusals, report);
    Landed {
        fallback,
        bound,
        dscp,
        hdr_sig,
        block,
        req_intended,
        refusals,
        members,
        layers,
        tmsgs,
    }
}
