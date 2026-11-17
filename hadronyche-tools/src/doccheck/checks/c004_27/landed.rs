//! The landed sections the probe pair reads from: the constants (C-06,
//! C-13, C-17), the hello's common control header and type-code block, the
//! admission request's intended packet size, the refusal and rejection
//! enumerations, and wire.toml (read as the script's subset reader and
//! Python saw it: [`TomlReader::Subset004_18`](crate::doccheck::policy::TomlReader)).

use std::collections::BTreeMap;

use crate::doccheck::family004::{Ctx, Field, fields_of, find_sub, first_field_table, subsections, tables};
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::Report;
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::text::{first_int, full, has_i, ints, rx};

/// What the later criteria read.
pub(crate) struct Landed {
    pub fallback: Option<i128>,
    pub bound: Option<i128>,
    pub dscp: Option<i128>,
    /// (offset, bits, type) of the hello's first seven header rows.
    pub hdr_sig: HeaderSig,
    /// The type-code block of the probe pair (inclusive).
    pub block: Option<(i128, i128)>,
    pub req_intended: Vec<Field>,
    pub refusals: BTreeMap<String, i128>,
    pub members: BTreeMap<String, i128>,
    pub layers: BTreeMap<String, String>,
    pub tmsgs: Vec<Dict>,
}

impl Landed {
    /// True when `code` is in the probe block.
    pub fn in_block(&self, code: Option<i128>) -> bool {
        matches!((self.block, code), (Some((a, b)), Some(c)) if a <= c && c <= b)
    }
}

/// (offset, bits, type) of each header row.
type HeaderSig = Vec<(i128, i128, String)>;

const IPV4_UDP_HEADERS: i128 = 28;

/// A number read from a document, or a reported failure when it is beyond
/// what the check can compare (Python would have kept it).
fn checked<T>(r: Result<Option<T>, crate::doccheck::pycompat::IntError>, ac: &str, what: &str, report: &mut Report) -> Option<T> {
    r.unwrap_or_else(|e| {
        report.fail(ac, format!("{what}: {e}"));
        None
    })
}

fn constants(ctx: &Ctx, report: &mut Report) -> (Option<i128>, Option<i128>, Option<i128>) {
    let fallback = checked(first_int(ctx.consts.value("C-06")), "AC1", "C-06", report);
    let grab = |id: &str, pat: &str, report: &mut Report| {
        let found = rx(pat).captures(ctx.consts.value(id)).map(|c| c[1].to_string());
        checked(found.map(|n| parse_int(&n)).transpose(), "AC1", id, report)
    };
    let bound = grab("C-17", r"at most (\d+) bytes", report);
    let dscp = grab("C-13", r"control CS5 \((\d+)\)", report);
    let ok = matches!((fallback, bound), (Some(f), Some(b)) if f != 0 && b != 0 && f - IPV4_UDP_HEADERS == b);
    report.check("AC1", ok, format!("the fallback arithmetic holds: C-06's {fallback:?} bytes less {IPV4_UDP_HEADERS} bytes of IPv4 and UDP headers is C-17's {bound:?} bytes of UDP payload"));
    (fallback, bound, dscp)
}

fn header(hsubs: &[(String, String)], report: &mut Report) -> HeaderSig {
    let hdr_table = find_sub(hsubs, r"common\s+control\s+header").first().and_then(|(_, x)| first_field_table(x));
    let hdr = fields_of(hdr_table.as_ref(), "AC1", "the hello's common control header", report);
    let sum = |fs: &[Field]| fs.iter().fold(0i128, |a, f| a.saturating_add(f.size_bits()));
    let bytes = sum(&hdr) / 8;
    let tiles = hdr.iter().enumerate().all(|(i, f)| f.off == sum(&hdr[..i]) / 8);
    report.check("AC1", hdr.len() == 7 && bytes == 32 && tiles, format!("the hello's common control header is seven rows tiling 32 bytes (found {} rows, {bytes} bytes)", hdr.len()));
    hdr.iter().take(7).map(|f| (f.off, f.bits, f.ty.clone())).collect()
}

fn hello(ctx: &Ctx, report: &mut Report) -> (HeaderSig, Option<(i128, i128)>) {
    let hsubs = subsections(&ctx.body(ctx.span_of(r"^hello$", 2)));
    let sig = header(&hsubs, report);
    let mut block = None;
    let mut assigned = String::new();
    if let Some((_, codes)) = find_sub(&hsubs, r"control\s+message\s+type\s+codes").first() {
        for t in tables(codes) {
            for r in &t.rows {
                if r.cells.len() >= 3 && has_i("probe", r.cell(1)) {
                    let n = checked(ints(r.cell(0)).map(Some), "AC1", "the probe block", report).unwrap_or_default();
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

/// Rows of an enumeration table: a numeric first cell and a CamelCase name.
fn enum_rows(text: &str, min_cells: usize, report: &mut Report) -> Vec<(String, i128, String)> {
    let mut out = Vec::new();
    for t in tables(text) {
        for r in &t.rows {
            let (v, n) = (r.cell(0).py_strip(), r.cell(1).py_strip());
            if r.cells.len() >= min_cells
                && full(r"\d+", v)
                && full(r"[A-Z][A-Za-z0-9]+", n)
                && let Some(v) = checked(parse_int(v).map(Some), "AC3", n, report)
            {
                out.push((n.to_string(), v, r.cell(3).to_string()));
            }
        }
    }
    out
}

fn admission(ctx: &Ctx, report: &mut Report) -> (Vec<Field>, BTreeMap<String, i128>) {
    let asubs = subsections(&ctx.body(ctx.span_of(r"^admission\s+messages$", 2)));
    let req_table = find_sub(&asubs, r"^admission\s+request$").first().and_then(|(_, x)| first_field_table(x));
    let req = fields_of(req_table.as_ref(), "AC1", "the admission request", report);
    let intended: Vec<Field> = req.into_iter().filter(|f| rx(r"(?i)^\s*intended\s+packet\s+size").is_match(&f.meaning)).collect();
    let ok = intended.len() == 1 && intended[0].bits == 16 && intended[0].valid.contains("65507");
    report.check("AC1", ok, "the admission request still carries the intended packet size as a u16 with a maximum of 65 507 (the size the probe pair takes)");
    let refusals = match find_sub(&asubs, r"refusal\s+reasons").first() {
        Some((_, rf)) => enum_rows(rf, 2, report).into_iter().map(|(n, v, _)| (n, v)).collect(),
        None => BTreeMap::new(),
    };
    (intended, refusals)
}

fn rejection(ctx: &Ctx, report: &mut Report) -> (BTreeMap<String, i128>, BTreeMap<String, String>) {
    let (mut members, mut layers) = (BTreeMap::new(), BTreeMap::new());
    for (n, v, l) in enum_rows(&ctx.body(ctx.span_of(r"rejection[-\s]+reasons?", 2)), 4, report) {
        members.insert(n.clone(), v);
        layers.insert(n, l);
    }
    report.check("AC3", members.len() == 13, format!("the Rejection reasons enumeration of wire.md has 13 members (found {})", members.len()));
    (members, layers)
}

/// `TREF == REF`: `{r["name"]: r["value"] for r in TOML.get("refusal", [])
/// if isinstance(r, dict) and "name" in r and "value" in r} == REF`, with
/// Python's `==`; `Err` where the script crashes (an unhashable name, a
/// "refusal" that cannot be iterated).
fn refusals_equal(toml: &Dict, md: &BTreeMap<String, i128>) -> Result<bool, String> {
    let entries = match toml.get("refusal") {
        Some(v) => v.py_dicts()?,
        None => Vec::new(),
    };
    let mut tref: Vec<(PyValue, PyValue)> = Vec::new();
    for r in entries {
        let (Some(name), Some(value)) = (r.get("name"), r.get("value")) else { continue };
        if matches!(name, PyValue::List(_) | PyValue::Dict(_)) {
            return Err(format!("TypeError: unhashable type: '{}'", name.type_name()));
        }
        match tref.iter_mut().find(|(k, _)| k.py_eq(name)) {
            Some(slot) => slot.1 = value.clone(),
            None => tref.push((name.clone(), value.clone())),
        }
    }
    Ok(tref.len() == md.len()
        && md.iter().all(|(k, v)| tref.iter().any(|(n, t)| n.eq_str(k) && t.eq_int(*v))))
}

/// `TMSGS = [m for m in TOML.get("message", []) if isinstance(m, dict)]`.
fn messages(toml: &Dict) -> Result<Vec<Dict>, String> {
    Ok(match toml.get("message") {
        Some(v) => v.py_dicts()?.into_iter().cloned().collect(),
        None => Vec::new(),
    })
}

fn toml_side(toml: &Dict, refusals: &BTreeMap<String, i128>, report: &mut Report) -> Vec<Dict> {
    let read = messages(toml).and_then(|m| refusals_equal(toml, refusals).map(|eq| (m, eq)));
    let (tmsgs, equal) = match read {
        Ok(x) => x,
        Err(e) => {
            report.crash(format!("wire.toml: {e}"));
            (Vec::new(), false)
        }
    };
    let ok = !refusals.is_empty() && refusals.len() == 18 && equal;
    report.check("AC4", ok, format!("the refusal enumeration of wire.toml equals the 18 names and values of wire.md's Refusal reasons (md {})", refusals.len()));
    tmsgs
}

/// Reads the landed sections, checking what the script checks of them.
pub(crate) fn landed(ctx: &Ctx, toml: &Dict, report: &mut Report) -> Landed {
    let (fallback, bound, dscp) = constants(ctx, report);
    let (hdr_sig, block) = hello(ctx, report);
    let (req_intended, refusals) = admission(ctx, report);
    let (members, layers) = rejection(ctx, report);
    let tmsgs = toml_side(toml, &refusals, report);
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
