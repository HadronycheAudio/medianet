//! Port of `checks/004-19.sh`: the sync request and reply layouts (ticket
//! 004-19).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_19.rs`. A transcription with Python semantics on the
//! library's named variants ([`family004`]: `heading_index`, `tables`,
//! `is_field_table`, `const_line`, `width_to_bits` under
//! `WidthRule::DigitGroups004_06`; `section_blocks` and `block_role`;
//! `REJECTION_004_16`; `negated_at` with `LOOKBACK_004_19`; the DoD under
//! `TagPolicy::BLOCKS_004_19`, `TableTagRule::AfterProse004_19` and
//! `OpenItemPolicy::Window3`; [`crate::doccheck::tomlsubset`]). A Python
//! exception stops the check (`Stop::Crash`); an integer beyond `i128` is
//! red by rule (`INT_LIMIT`). 004-19's own variants, private to this port: the section is the first heading of any level matching
//! `sync\s+requests?\b`, with a placeholder rule (`to be filled` and no
//! `|`); its blocks are runs of non-blank lines with fenced code kept inside
//! and each heading its own block; the request and reply tables are told
//! apart by the nearest heading or paragraph above them; widths are read by
//! `width_to_bits` (a counted width reads its element only); the stated
//! size is any number before "bytes" in a size sentence; the reason tokens
//! `\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b` with a 25-character look-back for
//! `never|not|nor|rather than|instead of` (no `no`); invalid forms in the
//! first table with "form"/"invalid" and "outcome" header cells, at least
//! 6; categories by the form cell; the toml messages matched by
//! `sync[ _-]request` / `sync[ _-]reply` in full; DSCP and rate names by
//! token; the DoD block rule (a table follows a tagged paragraph) and the
//! open-item window of three blocks.
//!
//! [`REJECTION_PIN`] is the pin of lines 649-650. The repository-level
//! assertions (wire-check "checks clean"; wire_tables exactly 12/0/0 with a
//! result line; the design checker printing OK) are in
//! `scripts/repo-state.d/004-19.sh`.

pub(crate) mod ac1;
mod ac5;
mod ac6;
mod works;

pub mod denials;
use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::blocks::{block_role, section_blocks};
use crate::doccheck::equiv::script::DEFAULT_READER;
use crate::doccheck::family004::{self, WidthRule, is_field_table};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{PyStr, parse_int, splitlines, universal_newlines};
use crate::doccheck::rejection_members::{Enumeration, REJECTION_004_16};
use crate::doccheck::report::Flow;
use crate::doccheck::tables::Table;
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// Python's `a + b` and `a * b` on ints: [`crate::doccheck::checked`] (R21;
/// `INT_LIMIT` on overflow).
pub(crate) use crate::doccheck::checked::{add as int_add, mul as int_mul};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The number of members the closed rejection enumeration must have
/// (`len(MEMBERS) != 13`, script lines 649-650).
pub const REJECTION_PIN: usize = 13;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-19",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Sync request and reply$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Rejection reasons$"),
        },
        Target {
            file: WIRE_TOML,
            heading: None,
        },
        Target {
            file: CONSTANTS_MD,
            heading: None,
        },
    ],
    keywords: &["stateless", "echo", "carries", "monotonic", "unicast", "zero", "rejection"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The constants the script reads.
#[derive(Debug, Default)]
pub(crate) struct Konst {
    pub dscp: Option<i128>,
    pub req_rate: Option<i128>,
    pub acc: Option<(i128, i128)>,
    pub epoch_bits: Option<i128>,
    pub module_bits: Option<i128>,
    pub domain_bits: Option<i128>,
    pub scheme_bits: Option<i128>,
    pub counter_bits: Option<i128>,
    pub tag_bits: Option<i128>,
}

/// One row of a field table, as `table_fields` reads it.
#[derive(Debug, Clone)]
pub(crate) struct Row {
    /// Offset in bits.
    pub off: i128,
    pub w: i128,
    pub ty: String,
    pub meaning: String,
    pub valid: String,
    pub text: String,
}

/// What the script reads.
pub(crate) struct Ctx {
    pub k: Konst,
    pub members: Enumeration,
    pub placeholder: bool,
    pub section: String,
    pub sec_lines: Vec<String>,
    pub blocks: Vec<Vec<String>>,
    pub prose: Vec<String>,
    pub prose_all: String,
    pub req_f: Vec<Row>,
    pub rep_f: Vec<Row>,
    pub req_size: Option<i128>,
    pub rep_size: Option<i128>,
    pub tables: Vec<Table>,
    pub toml_text: Option<String>,
}

/// `first_int(pattern, text)` (beyond `i128`: `INT_LIMIT`, recorded).
fn first_int(pattern: &str, text: &str) -> Option<i128> {
    rx(&format!("(?i){pattern}")).captures(text).and_then(|m| parse_int(&m[1]).ok())
}

fn konst(docs: &Docs, report: &mut Report) -> Konst {
    let text = docs.read(CONSTANTS_MD).map(|t| universal_newlines(&t)).unwrap_or_default();
    let row = |cid: &str| family004::const_line(&text, cid);
    let triple = |cid: &str| rx(r"(\d+)\s*/\s*(\d+)\s*/\s*(\d+)\s*bits").captures(&row(cid)).map(|m| (parse_int(&m[1]).ok(), parse_int(&m[3]).ok()));
    let k = Konst {
        dscp: first_int(r"CS6\s*\((\d+)\)", &row("C-13")),
        req_rate: first_int(r"(\d+)\s+per\s+second", &row("C-36")),
        acc: rx(r"(?i)(\d+)\s+to\s+(\d+)\s+per\s+second").captures(&row("C-112")).and_then(|m| Some((parse_int(&m[1]).ok()?, parse_int(&m[2]).ok()?))),
        epoch_bits: triple("C-10").and_then(|t| t.1),
        module_bits: triple("C-11").and_then(|t| t.0),
        domain_bits: triple("C-11").and_then(|t| t.1),
        scheme_bits: first_int(r"scheme\s+(\d+)\s+bits", &row("C-12")),
        counter_bits: first_int(r"counter\s+(\d+)\s+bits", &row("C-12")),
        tag_bits: first_int(r"tag\s+(\d+)\s+bits", &row("C-12")),
    };
    let named = [
        ("dscp", k.dscp),
        ("req_rate", k.req_rate),
        ("acc_lo", k.acc.map(|a| a.0)),
        ("acc_hi", k.acc.map(|a| a.1)),
        ("epoch_bits", k.epoch_bits),
        ("module_bits", k.module_bits),
        ("domain_bits", k.domain_bits),
        ("scheme_bits", k.scheme_bits),
        ("counter_bits", k.counter_bits),
        ("tag_bits", k.tag_bits),
    ];
    for (key, v) in named {
        if v.is_none() {
            report.fail("DoD", format!("constants row for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    k
}

/// `width_to_bits(cell)` (`WidthRule::DigitGroups004_06`); beyond `i128`
/// `INT_LIMIT` (recorded by the library) and no width.
pub(crate) fn width_to_bits(cell: &str) -> Option<i128> {
    family004::width_to_bits(cell, WidthRule::DigitGroups004_06).ok().flatten()
}

/// `classify(text)`: "request", "reply" or None.
fn classify(text: &str) -> Option<&'static str> {
    let t = text.to_lowercase();
    let (req, rep) = (t.contains("request"), has(r"\brepl(y|ies)\b", &t));
    match (req, rep) {
        (false, true) => Some("reply"),
        (true, false) => Some("request"),
        _ => None,
    }
}

/// `table_label(start)`: the nearest heading and paragraph line above.
fn table_label(lines: &[String], start: usize) -> (Option<String>, Option<String>) {
    let (mut heading, mut para) = (None, None);
    for k in (0..start.min(lines.len())).rev() {
        let ln = lines[k].py_strip();
        if ln.is_empty() {
            continue;
        }
        if ln.starts_with('#') {
            heading = Some(ln.to_string());
            break;
        }
        if ln.starts_with('|') {
            break;
        }
        if para.is_none() {
            para = Some(ln.to_string());
        }
    }
    (heading, para)
}

/// `table_fields(tab)`.
fn table_fields(t: Option<&Table>) -> Vec<Row> {
    let Some(t) = t else { return Vec::new() };
    let mut out = Vec::new();
    for row in t.rows.iter().map(|r| &r.cells) {
        if row.len() < 5 {
            continue;
        }
        // beyond i128: INT_LIMIT (recorded), the row left out
        let off = rx(r"^\s*(\d+)\s*$").captures(&row[0]).and_then(|m| parse_int(&m[1]).ok());
        let (Some(off), Some(w)) = (off, width_to_bits(&row[1])) else { continue };
        out.push(Row {
            off: int_mul(off, 8),
            w,
            ty: row[2].chars().filter(|c| !"`*_ ".contains(*c)).collect::<String>().to_lowercase(),
            meaning: row[3].clone(),
            valid: row[4].clone(),
            text: row.join(" | "),
        });
    }
    out
}

/// `size_of(fields, name)`.
fn size_of(fields: &[Row], name: &str, report: &mut Report) -> Option<i128> {
    let (mut pos, mut ok) = (0i128, true);
    for f in fields {
        if f.off != pos {
            report.fail("AC1", format!("{name} table: field at byte {} does not start where the previous field ends (byte {}); the fields must tile with no gap and no overlap", f.off.div_euclid(8), pos.div_euclid(8)));
            ok = false;
        }
        pos = int_add(f.off, f.w);
    }
    if pos.rem_euclid(8) != 0 {
        report.fail("AC1", format!("{name} table: the fields sum to {pos} bits, not whole bytes"));
    }
    ok.then(|| pos.div_euclid(8))
}

const ALL: [&str; 10] = ["AC1", "AC2", "AC3", "AC4", "AC5", "W1", "W2", "W4", "W5", "DoD"];

fn section(ctx: &mut Ctx, wire: &str, report: &mut Report) {
    let lines = splitlines(wire);
    let heads = family004::heading_index(&lines);
    let Some(k) = heads.iter().position(|h| has_i(r"sync\s+requests?\b", &h.2)) else {
        ctx.placeholder = true;
        for ac in ALL {
            report.fail(ac, "no 'Sync request' section heading in wire.md");
        }
        return;
    };
    let (start, level) = (heads[k].0 + 1, heads[k].1);
    let end = heads[k + 1..].iter().find(|h| h.1 <= level).map_or(lines.len(), |h| h.0);
    ctx.section = lines[start.min(end)..end].join("\n");
    ctx.sec_lines = splitlines(&ctx.section).into_iter().map(String::from).collect();
    if ctx.section.py_strip().is_empty() || (has_i(r"to\s+be\s+filled", &ctx.section) && !ctx.section.contains('|')) {
        ctx.placeholder = true;
        for ac in ALL {
            report.fail(ac, "the Sync request section is still the 004-19 placeholder");
        }
    }
}

fn tables(ctx: &mut Ctx, report: &mut Report) {
    ctx.tables = family004::tables(&ctx.section);
    let mut fts: Vec<(Option<&'static str>, &Table)> = Vec::new();
    for t in &ctx.tables {
        if !is_field_table(t.head()) {
            continue;
        }
        let (h, p) = table_label(&ctx.sec_lines, t.line);
        fts.push((classify(h.as_deref().unwrap_or("")).or_else(|| classify(p.as_deref().unwrap_or(""))), t));
    }
    if fts.len() == 2 && fts.iter().all(|t| t.0.is_none()) {
        fts[0].0 = Some("request");
        fts[1].0 = Some("reply");
    }
    let req = fts.iter().find(|t| t.0 == Some("request")).map(|t| t.1);
    let rep = fts.iter().find(|t| t.0 == Some("reply")).map(|t| t.1);
    if !ctx.placeholder && (req.is_none() || rep.is_none()) {
        report.fail("AC1", "the section needs a request field table and a reply field table (name them by their heading, e.g. '### Request' and '### Reply')");
    }
    let (rq, rp) = (table_fields(req), table_fields(rep));
    ctx.req_size = if rq.is_empty() { None } else { size_of(&rq, "request", report) };
    ctx.rep_size = if rp.is_empty() { None } else { size_of(&rp, "reply", report) };
    ctx.req_f = rq;
    ctx.rep_f = rp;
}

fn context(docs: &Docs, report: &mut Report) -> Ctx {
    let k = konst(docs, report);
    let wire = match docs.read(WIRE_MD) {
        Some(t) => universal_newlines(&t),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            for ac in ["AC2", "AC3", "AC4", "AC5", "DoD"] {
                report.fail(ac, format!("{WIRE_MD} does not exist"));
            }
            String::new()
        }
    };
    let members = Enumeration::parse(&wire, REJECTION_004_16);
    if !members.section_found {
        report.fail("AC5", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read");
    } else if !members.has_exactly(REJECTION_PIN) {
        report.fail("AC5", format!("the closed enumeration parsed from wire.md has {} names, not {REJECTION_PIN}", members.count()));
    }
    let mut ctx = Ctx {
        k,
        members,
        placeholder: false,
        section: String::new(),
        sec_lines: Vec::new(),
        blocks: Vec::new(),
        prose: Vec::new(),
        prose_all: String::new(),
        req_f: Vec::new(),
        rep_f: Vec::new(),
        req_size: None,
        rep_size: None,
        tables: Vec::new(),
        toml_text: docs.read(WIRE_TOML).map(|t| universal_newlines(&t)),
    };
    section(&mut ctx, &wire, report);
    ctx.blocks = section_blocks(&ctx.sec_lines);
    ctx.prose = ctx.blocks.iter().filter(|b| block_role(b).name() == "prose").map(|b| b.join("\n")).collect();
    ctx.prose_all = ctx.prose.join("\n");
    tables(&mut ctx, report);
    ctx
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let flow = body(docs, &mut report);
    report.finish(flow)
}

/// The script, in its order; a Python exception stops it.
fn body(docs: &Docs, report: &mut Report) -> Flow<()> {
    let ctx = context(docs, report);
    ac1::ac1_to_ac4(&ctx, report);
    let form_table = ac5::forms(&ctx, report);
    ac6::toml(&ctx, form_table.as_ref(), report)?;
    works::works_and_dod(&ctx, report);
    Ok(())
}
