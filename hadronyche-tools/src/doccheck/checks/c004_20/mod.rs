//! Port of `checks/004-20.sh`: the receiver report and the one-way probe
//! (ticket 004-20).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_20.rs`. A transcription with Python semantics on the
//! library's named variants ([`family004`]: `heading_index`, `section_span`,
//! `tables`, `is_field_table`, `const_line`, `width_to_bits` under
//! `WidthRule::FirstNumber004_20`, `norm_name` under `NameNorm::Letters004_20`;
//! `section_blocks`, `block_role`, `is_placeholder`; `FormRows::FirstCell004_20`;
//! `SentenceSplit::NEWLINES_004_20`; `REJECTION_004_16` with each member's
//! Layers cell; `negated_at` with `LOOKBACK_004_19`, `NON_REJECTION_004_20`;
//! the DoD under `TagPolicy::BLOCKS_004_19`, `TableTagRule::AfterProse004_19`
//! and `OpenItemPolicy::WordLeading`; [`crate::doccheck::tomlsubset`]). A
//! Python exception stops the check (`Stop::Crash`); an integer beyond `i128`
//! is red by rule (`INT_LIMIT`). The harness's default reader reads the
//! labels `C-40`, `C-41`, `C-13` and `C-100`. 004-20's own variants, private
//! to this port: sections found by the first heading of any level matching
//! `^receiver\s+report`, `^one-way\s+probe` and `^admission`; blocks as
//! 004-19's; field tables told apart by the heading or paragraph above
//! them; its own `width_to_bits` (bits only when "bits" and no "bytes"),
//! `offset_bytes` (an optional `+`) and a tiling rule that lets a sub-byte
//! row share its byte (and always restarts at the running position); row
//! labels (`cell_label`); `stated_bytes` and `max_int` over digit groups.
//!
//! [`REJECTION_PIN`] is the pin of line 702. The repository-level
//! assertions (wire-check "checks clean"; wire_tables exactly 12/0/0 with a
//! result line; the design checker printing OK) are in
//! `scripts/repo-state.d/004-20.sh`.

mod ac1;
mod ac4;
mod ac5;
mod ac6;
mod consist;

pub mod denials;
use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::blocks::{FormRows, block_role, form_rows, is_placeholder, section_blocks};
use crate::doccheck::equiv::script::DEFAULT_READER;
use crate::doccheck::family004::{self, NameNorm, SpanRule, WidthRule, is_field_table};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{PyStr, parse_int, splitlines, universal_newlines};
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::rejection_members::{Enumeration, REJECTION_004_16};
use crate::doccheck::report::Flow;
use crate::doccheck::sentences::{SentenceSplit, split};
use crate::doccheck::text::{has, has_i, head, rx};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The number of members the closed rejection enumeration must have
/// (`len(MEMBERS) != 13`, script line 702).
pub const REJECTION_PIN: usize = 13;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-20",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Receiver report$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^One-way probe$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Rejection reasons$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Admission messages$"),
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
    keywords: &["unicast", "multicast", "query", "wraps", "saturates", "events", "monotonic", "same"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// Python's `a + b` on ints. Beyond `i128` the value is recorded
/// (`INT_LIMIT`: the verdict is red by rule) and 0 stands in for it.
pub(crate) fn int_add(a: i128, b: i128) -> i128 {
    a.checked_add(b).unwrap_or_else(|| {
        record_int_limit(&format!("{a} + {b}"));
        0
    })
}

/// Python's `a * b` on ints; beyond `i128` as [`int_add`].
pub(crate) fn int_mul(a: i128, b: i128) -> i128 {
    a.checked_mul(b).unwrap_or_else(|| {
        record_int_limit(&format!("{a} * {b}"));
        0
    })
}

/// The constants the script reads.
#[derive(Debug, Default)]
pub(crate) struct Konst {
    pub members_max: Option<i128>,
    pub dscp_report: Option<i128>,
    pub dscp_probe: Option<i128>,
    pub report_ms: Option<i128>,
    pub timeout_s: Option<i128>,
    pub scheme_bits: Option<i128>,
    pub counter_bits: Option<i128>,
    pub tag_bits: Option<i128>,
}

/// One section (`class Sec`).
#[derive(Debug, Default)]
pub(crate) struct Sec {
    pub text: String,
    pub lines: Vec<String>,
    pub prose: Vec<String>,
    pub prose_all: String,
    pub sents: Vec<String>,
}

/// A field table and its label.
#[derive(Debug, Clone)]
pub(crate) struct FieldTable {
    pub label: String,
    pub rows: Vec<Vec<String>>,
}

impl Sec {
    fn new(text: Option<&str>) -> Sec {
        let text = text.unwrap_or("").to_string();
        let lines: Vec<String> = splitlines(&text).into_iter().map(String::from).collect();
        let prose: Vec<String> = section_blocks(&lines).iter().filter(|b| block_role(b).name() == "prose").map(|b| b.join("\n")).collect();
        let prose_all = prose.join("\n");
        let sents = sentences(&prose_all);
        Sec {
            text,
            lines,
            prose,
            prose_all,
            sents,
        }
    }

    fn table_label(&self, start: usize) -> String {
        let (mut heading, mut para) = (None, None);
        for k in (0..start.min(self.lines.len())).rev() {
            let ln = self.lines[k].py_strip();
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
        heading.or(para).unwrap_or_default()
    }

    /// `field_tables()`.
    pub fn field_tables(&self) -> Vec<FieldTable> {
        family004::tables(&self.text)
            .into_iter()
            .filter(|t| is_field_table(t.head()))
            .map(|t| FieldTable {
                label: self.table_label(t.line),
                rows: t.rows.into_iter().map(|r| r.cells).collect(),
            })
            .collect()
    }

    /// `form_rows()`.
    pub fn form_rows(&self) -> Vec<Vec<String>> {
        form_rows(&self.text, FormRows::FirstCell004_20)
    }
}

/// `norm(s)`: the lower-case ASCII letters only.
pub(crate) fn norm(s: &str) -> String {
    family004::norm_name(s, NameNorm::Letters004_20)
}

/// `sentences(text)`: `re.split(r"(?<=[.;])\s+|\n+", text)`, blank ones
/// dropped.
pub(crate) fn sentences(text: &str) -> Vec<String> {
    split(text, SentenceSplit::NEWLINES_004_20)
}

/// `stated_bytes(sentence)`.
pub(crate) fn stated_bytes(s: &str) -> Vec<i128> {
    // beyond i128: INT_LIMIT (recorded), left out
    rx("(\\d[\\d ,\u{a0}\u{202f}]*\\d|\\d)\\s*bytes?\\b").captures_iter(s).filter_map(|h| parse_int(&strip_groups(&h[1])).ok()).collect()
}

fn strip_groups(s: &str) -> String {
    s.chars().filter(|c| !" ,\u{a0}\u{202f}".contains(*c)).collect()
}

/// `max_int(text)`.
pub(crate) fn max_int(text: &str) -> Option<i128> {
    // beyond i128: INT_LIMIT (recorded), left out
    rx("\\d[\\d ,\u{a0}\u{202f}]*\\d|\\d").find_iter(text).filter_map(|h| parse_int(&strip_groups(h.as_str())).ok()).max()
}

/// `width_to_bits(cell)` (`WidthRule::FirstNumber004_20`); beyond `i128`
/// `INT_LIMIT` (recorded by the library) and no width.
pub(crate) fn width_to_bits(cell: &str) -> Option<i128> {
    family004::width_to_bits(cell, WidthRule::FirstNumber004_20).ok().flatten()
}

/// `cell_label(meaning)`.
pub(crate) fn cell_label(meaning: &str) -> String {
    let s: String = meaning.chars().filter(|c| !"*_`".contains(*c)).collect();
    let s = s.py_strip();
    if head(s, 80).contains(':') {
        return (s.split(':').next().unwrap_or("")).py_strip().to_string();
    }
    s.py_split().take(4).collect::<Vec<_>>().join(" ")
}

/// One tiled row.
#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub off: i128,
    pub w: i128,
    pub label: String,
    pub lab: String,
    pub ty: String,
    pub meaning: String,
    pub valid: String,
    pub text: String,
}

/// `layout(rows, name, ac)`: (fields, bits).
pub(crate) fn layout(rows: &[Vec<String>], name: &str, ac: &str, report: &mut Report) -> (Vec<Field>, i128) {
    let mut fields = Vec::new();
    let mut pos: i128 = 0;
    for row in rows {
        if row.len() < 5 {
            continue;
        }
        // `offset_bytes`; beyond i128: INT_LIMIT (recorded), the row left out
        let Some(off) = rx(r"^\s*\+?\s*(\d+)\s*\z").captures(&row[0]).and_then(|m| parse_int(&m[1]).ok()) else { continue };
        let Some(w) = width_to_bits(&row[1]) else {
            report.fail(ac, format!("{name}: the width of '{}' is not stated", cell_label(&row[3])));
            continue;
        };
        let declared = int_mul(off, 8);
        let shares = w < 8 && declared.div_euclid(8) == pos.div_euclid(8) && pos.rem_euclid(8) + w <= 8;
        if declared != pos && !shares {
            report.fail(ac, format!("{name}: the row at byte {off} ('{}') does not start where the previous row ends (bit {pos}): a gap or an overlap", cell_label(&row[3])));
        }
        let label = cell_label(&row[3]);
        fields.push(Field {
            off: pos,
            w,
            lab: norm(&label),
            label,
            ty: row[2].chars().filter(|c| !"`*_ ".contains(*c)).collect::<String>().to_lowercase(),
            meaning: row[3].clone(),
            valid: row[4].clone(),
            text: row.join(" | "),
        });
        pos = int_add(pos, w);
    }
    if pos.rem_euclid(8) != 0 {
        report.fail(ac, format!("{name}: the fields sum to {pos} bits, not whole bytes"));
    }
    (fields, pos)
}

/// What the script reads.
pub(crate) struct Ctx {
    pub k: Konst,
    pub members: Enumeration,
    pub rep: Sec,
    pub prb: Sec,
    pub adm: Sec,
    pub rep_place: bool,
    pub prb_place: bool,
    pub adm_place: bool,
    pub rep_ok: bool,
    pub rep_fix: Vec<Field>,
    pub rep_mem: Vec<Field>,
    pub rep_fix_bits: Option<i128>,
    pub rep_mem_bits: Option<i128>,
    pub prb_ok: bool,
    pub req_f: Vec<Field>,
    pub rep_f: Vec<Field>,
    pub req_bits: Option<i128>,
    pub rpl_bits: Option<i128>,
    pub toml_text: Option<String>,
}

impl Ctx {
    /// `MAX_REP`.
    pub fn max_rep(&self) -> Option<i128> {
        let (f, m) = (self.rep_fix_bits?, self.rep_mem_bits?);
        if !self.rep_ok || f.rem_euclid(8) != 0 || m.rem_euclid(8) != 0 {
            return None;
        }
        let mm = self.k.members_max.filter(|&x| x != 0)?;
        Some(int_add(f, int_mul(mm, m)).div_euclid(8))
    }

    /// `FIX_BYTES`, `MEM_BYTES`.
    pub fn bytes(&self, bits: Option<i128>) -> Option<i128> {
        bits.filter(|b| self.rep_ok && b.rem_euclid(8) == 0).map(|b| b.div_euclid(8))
    }

    /// The Layers cell of a member, or "".
    pub fn layers(&self, name: &str) -> String {
        self.members.get(name).map_or_else(String::new, |m| m.layers().to_string())
    }
}

fn konst(docs: &Docs, report: &mut Report) -> Konst {
    let text = docs.read(CONSTANTS_MD).map(|t| universal_newlines(&t)).unwrap_or_default();
    let row = |cid: &str| family004::const_line(&text, cid);
    // beyond i128: INT_LIMIT (recorded)
    let fi = |p: &str, cid: &str| rx(&format!("(?i){p}")).captures(&row(cid)).and_then(|m| parse_int(&m[1]).ok());
    let k = Konst {
        members_max: fi(r"at\s+most\s+(\d+)", "C-08"),
        dscp_report: fi(r"control\s+CS5\s*\((\d+)\)", "C-13"),
        dscp_probe: fi(r"EF\s*\((\d+)\)", "C-13"),
        report_ms: fi(r"every\s+(\d+)\s*ms", "C-40"),
        timeout_s: fi(r"(\d+)\s*s\s+without", "C-41"),
        scheme_bits: fi(r"scheme\s+(\d+)\s+bits", "C-12"),
        counter_bits: fi(r"counter\s+(\d+)\s+bits", "C-12"),
        tag_bits: fi(r"tag\s+(\d+)\s+bits", "C-12"),
    };
    let named = [
        ("members_max", k.members_max),
        ("dscp_report", k.dscp_report),
        ("dscp_probe", k.dscp_probe),
        ("report_ms", k.report_ms),
        ("timeout_s", k.timeout_s),
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

fn report_tables(ctx: &mut Ctx, report: &mut Report) {
    if ctx.rep_place {
        return;
    }
    let tabs = ctx.rep.field_tables();
    let (member, fixed): (Vec<&FieldTable>, Vec<&FieldTable>) = tabs.iter().partition(|t| has_i(r"per[- ]member|each member|member block", &t.label));
    if fixed.len() != 1 || member.len() != 1 {
        report.fail("AC1", "the receiver report needs one fixed-part field table and one per-member field table (name them by their heading, e.g. '### Fixed part' and '### Per-member part')");
        return;
    }
    let (f, fb) = layout(&fixed[0].rows, "receiver report fixed part", "AC1", report);
    let (m, mb) = layout(&member[0].rows, "receiver report per-member part", "AC1", report);
    (ctx.rep_fix, ctx.rep_fix_bits, ctx.rep_mem, ctx.rep_mem_bits) = (f, Some(fb), m, Some(mb));
    ctx.rep_ok = true;
}

fn probe_tables(ctx: &mut Ctx, report: &mut Report) {
    if ctx.prb_place {
        return;
    }
    let tabs = ctx.prb.field_tables();
    let (mut req, mut rep) = (Vec::new(), Vec::new());
    for t in &tabs {
        let lab = t.label.to_lowercase();
        let (has_req, has_rep) = (lab.contains("request"), has(r"\brepl(y|ies)\b", &lab));
        if has_rep && !has_req {
            rep.push(t);
        } else if has_req && !has_rep {
            req.push(t);
        }
    }
    if tabs.len() != 2 || req.len() != 1 || rep.len() != 1 {
        report.fail("AC1", "the one-way probe needs exactly one request field table and one reply field table (name them by their heading, e.g. '### Request' and '### Reply')");
        return;
    }
    let (a, ab) = layout(&req[0].rows, "probe request", "AC1", report);
    let (b, bb) = layout(&rep[0].rows, "probe reply", "AC1", report);
    (ctx.req_f, ctx.req_bits, ctx.rep_f, ctx.rpl_bits) = (a, Some(ab), b, Some(bb));
    ctx.prb_ok = true;
}

fn context(docs: &Docs, report: &mut Report) -> Ctx {
    let k = konst(docs, report);
    let wire = match docs.read(WIRE_MD) {
        Some(t) => universal_newlines(&t),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let lines: Vec<&str> = splitlines(&wire);
    let heads = family004::heading_index(&lines);
    let members = Enumeration::parse(&wire, REJECTION_004_16);
    if !members.section_found {
        report.fail("AC5", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read");
    } else if !members.has_exactly(REJECTION_PIN) {
        report.fail("AC5", format!("the closed enumeration parsed from wire.md has {} names, not {REJECTION_PIN}", members.count()));
    }
    let span = |p: &str| family004::section_span(lines.len(), &heads, p, None, SpanRule::AfterHeading004_16).map(|(a, b)| lines[a.min(b)..b].join("\n"));
    let (rep_t, prb_t, adm_t) = (span(r"^receiver\s+report"), span(r"^one-way\s+probe"), span("^admission"));
    let (rep_place, prb_place) = (is_placeholder(rep_t.as_deref()), is_placeholder(prb_t.as_deref()));
    if rep_place {
        for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"] {
            report.fail(ac, "the Receiver report section is missing or still the 004-20 placeholder");
        }
    }
    if prb_place {
        for ac in ["AC1", "AC4", "AC5", "AC6", "DoD"] {
            report.fail(ac, "the One-way probe section is missing or still the 004-20 placeholder");
        }
    }
    let mut ctx = Ctx {
        k,
        members,
        rep: Sec::new(rep_t.as_deref()),
        prb: Sec::new(prb_t.as_deref()),
        adm: Sec::new(adm_t.as_deref()),
        rep_place,
        prb_place,
        adm_place: is_placeholder(adm_t.as_deref()),
        rep_ok: false,
        rep_fix: Vec::new(),
        rep_mem: Vec::new(),
        rep_fix_bits: None,
        rep_mem_bits: None,
        prb_ok: false,
        req_f: Vec::new(),
        rep_f: Vec::new(),
        req_bits: None,
        rpl_bits: None,
        toml_text: docs.read(WIRE_TOML).map(|t| universal_newlines(&t)),
    };
    report_tables(&mut ctx, report);
    probe_tables(&mut ctx, report);
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
    ac1::ac1_to_ac3(&ctx, report);
    ac4::probe(&ctx, report);
    let (form_rows, named) = ac5::forms(&ctx, report);
    ac6::toml(&ctx, &form_rows, &named, report)?;
    consist::consistency_and_dod(&ctx, report);
    Ok(())
}
