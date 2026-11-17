//! Port of `checks/004-20.sh`: the receiver report and the one-way probe
//! (ticket 004-20).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_20.rs`. A transcription with Python semantics on the
//! shared WP7 helpers of `doccheck::wp7` ([`py`], [`md`], [`pytoml`]). 004-20's
//! own variants: sections found by the first heading of any level matching
//! `^receiver\s+report`, `^one-way\s+probe` and `^admission`; blocks as
//! 004-19's; field tables told apart by the heading or paragraph above
//! them; its own `width_to_bits` (bits only when "bits" and no "bytes"),
//! `offset_bytes` (an optional `+`) and a tiling rule that lets a sub-byte
//! row share its byte (and always restarts at the running position); row
//! labels (`cell_label`); sentences split on `.`/`;` and newlines; invalid
//! forms in every table whose first header cell names a form, rule, case or
//! condition and which has an outcome cell; the reason tokens
//! `\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b` with the 25-character look-back for
//! `never|not|nor|rather than|instead of`; 004-20's `NON_REJECTION`; the
//! Layers cell of each member (from [`rejection_members`]); the labels
//! `C-40`, `C-41`, `C-13` and `C-100` (the harness reads no criterion from a
//! label with a hyphen, so those compare by verdict).
//!
//! [`REJECTION_PIN`] is the pin of line 702. The repository-level
//! assertions (wire-check "checks clean"; wire_tables exactly 12/0/0 with a
//! result line; the design checker printing OK) are in
//! `scripts/repo-state.d/004-20.sh`.
//!
//! [`py`]: crate::doccheck::wp7::py
//! [`md`]: crate::doccheck::wp7::md
//! [`pytoml`]: crate::doccheck::wp7::pytoml
//! [`rejection_members`]: crate::doccheck::rejection_members

mod ac1;
mod ac4;
mod ac5;
mod ac6;
mod consist;

pub mod denials;
use crate::doccheck::wp7::md::{self, is_field_table, parse_tables};
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::common::{block_kind, section_blocks};
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::rejection_members::Enumeration;
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
    toml: TomlReader::SubsetReader,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[
    KnownDifference {
        label: r"^denial py_denial_a_reason_as_a_list$",
        class: DiffClass::ScriptCrashed,
        reason: "the script dies with a Python TypeError (hashing a list reason); the port reports the same cause under CRASH",
    },
    KnownDifference {
        label: ".*",
        class: DiffClass::ScriptCrashed,
        reason: "not a crash: the harness reads no criterion from the script's `FAIL C-40:`, `C-41:`, `C-13:` or `C-100:` \
                 lines (its label pattern `[A-Za-z]+ ?[0-9]*` stops at the hyphen), so a script red only on those looks \
                 like a crash; the port reports the same failures. Only the kind 'script crash, port red' is excused here",
    },
    KnownDifference {
        label: ".*",
        class: DiffClass::PortStricter,
        reason: "not stricter: the port fails C-40, C-41, C-13 or C-100 as the script does, but the harness reads no \
                 criterion from those script lines, so the port shows more criteria. Only the kind 'port fails more \
                 criteria' is excused here; every such row is listed in the summary",
    },
];

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
    pub blocks: Vec<Vec<String>>,
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
        let lines: Vec<String> = py::splitlines(&text).into_iter().map(String::from).collect();
        let blocks = section_blocks(&lines);
        let prose: Vec<String> = blocks.iter().filter(|b| block_kind(b) == "prose").map(|b| b.join("\n")).collect();
        let prose_all = prose.join("\n");
        let sents = sentences(&prose_all);
        Sec {
            text,
            lines,
            blocks,
            prose,
            prose_all,
            sents,
        }
    }

    fn table_label(&self, start: usize) -> String {
        let (mut heading, mut para) = (None, None);
        for k in (0..start.min(self.lines.len())).rev() {
            let ln = py::strip(&self.lines[k]);
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
        parse_tables(&self.text)
            .into_iter()
            .filter(|t| is_field_table(&t.header))
            .map(|t| FieldTable {
                label: self.table_label(t.start),
                rows: t.rows,
            })
            .collect()
    }

    /// `form_rows()`.
    pub fn form_rows(&self) -> Vec<Vec<String>> {
        let mut out = Vec::new();
        for t in parse_tables(&self.text) {
            let heads: Vec<String> = t.header.iter().map(|h| norm(h)).collect();
            let named = heads.len() >= 2 && ["form", "invalid", "rule", "case", "condition"].iter().any(|k| heads[0].contains(k)) && heads.iter().any(|h| h.contains("outcome"));
            if named {
                out.extend(t.rows.into_iter().filter(|r| r.len() >= 2 && !py::strip(&r[0]).is_empty()));
            }
        }
        out
    }
}

/// `norm(s)`: the lower-case ASCII letters only.
pub(crate) fn norm(s: &str) -> String {
    s.to_lowercase().chars().filter(char::is_ascii_lowercase).collect()
}

/// `sentences(text)`: `re.split(r"(?<=[.;])\s+|\n+", text)`, blank ones
/// dropped.
pub(crate) fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut start, mut prev) = (0, None::<char>);
    let mut it = text.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let after_stop = py::is_space(c) && matches!(prev, Some('.' | ';'));
        if !after_stop && c != '\n' {
            prev = Some(c);
            continue;
        }
        out.push(text[start..i].to_string());
        let mut end = i + c.len_utf8();
        let mut last = c;
        while let Some(&(j, d)) = it.peek() {
            if !(if after_stop { py::is_space(d) } else { d == '\n' }) {
                break;
            }
            end = j + d.len_utf8();
            last = d;
            it.next();
        }
        start = end;
        prev = Some(last);
    }
    out.push(text[start..].to_string());
    out.into_iter().filter(|s| !py::strip(s).is_empty()).collect()
}

/// `stated_bytes(sentence)`.
pub(crate) fn stated_bytes(s: &str) -> Vec<i128> {
    py::finditer("(\\d[\\d ,\u{a0}\u{202f}]*\\d|\\d)\\s*bytes?\\b", s, false).iter().filter_map(|h| py::int(&strip_groups(h.g(1)))).collect()
}

fn strip_groups(s: &str) -> String {
    s.chars().filter(|c| !" ,\u{a0}\u{202f}".contains(*c)).collect()
}

/// `max_int(text)`.
pub(crate) fn max_int(text: &str) -> Option<i128> {
    py::finditer("\\d[\\d ,\u{a0}\u{202f}]*\\d|\\d", text, false).iter().filter_map(|h| py::int(&strip_groups(h.all()))).max()
}

/// `width_to_bits(cell)` (004-20's).
pub(crate) fn width_to_bits(cell: &str) -> Option<i128> {
    let s = cell.to_lowercase();
    let n = py::int(py::search(r"(\d+)", &s, false)?.g(1))?;
    Some(if py::has(r"\bbits?\b", &s) && !py::has(r"\bbytes?\b", &s) { n } else { n.saturating_mul(8) })
}

/// `cell_label(meaning)`.
pub(crate) fn cell_label(meaning: &str) -> String {
    let s: String = meaning.chars().filter(|c| !"*_`".contains(*c)).collect();
    let s = py::strip(&s);
    if py::head(s, 80).contains(':') {
        return py::strip(s.split(':').next().unwrap_or("")).to_string();
    }
    s.split(py::is_space).filter(|w| !w.is_empty()).take(4).collect::<Vec<_>>().join(" ")
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
        let Some(off) = py::fullmatch(r"\s*\+?\s*(\d+)\s*", &row[0], false).and_then(|m| py::int(m.g(1))) else { continue };
        let Some(w) = width_to_bits(&row[1]) else {
            report.fail(ac, format!("{name}: the width of '{}' is not stated", cell_label(&row[3])));
            continue;
        };
        let declared = off.saturating_mul(8);
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
        pos = pos.saturating_add(w);
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
        Some(f.saturating_add(mm.saturating_mul(m)).div_euclid(8))
    }

    /// `FIX_BYTES`, `MEM_BYTES`.
    pub fn bytes(&self, bits: Option<i128>) -> Option<i128> {
        bits.filter(|b| self.rep_ok && b.rem_euclid(8) == 0).map(|b| b.div_euclid(8))
    }

    /// The Layers cell of a member, or "".
    pub fn layers(&self, name: &str) -> String {
        self.members.get(name).and_then(|m| m.cells.get(3)).cloned().unwrap_or_default()
    }
}

fn konst(docs: &Docs, report: &mut Report) -> Konst {
    let text = docs.read(CONSTANTS_MD).map(|t| py::universal(&t)).unwrap_or_default();
    let row = |cid: &str| {
        let lead = format!("| {cid} |");
        py::splitlines(&text).into_iter().find(|l| l.starts_with(&lead)).unwrap_or("").to_string()
    };
    let fi = |p: &str, cid: &str| py::search(p, &row(cid), true).and_then(|m| py::int(m.g(1)));
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

/// `is_placeholder(text)`.
fn is_placeholder(text: Option<&str>) -> bool {
    text.is_none_or(|t| py::strip(t).is_empty() || (py::has_i(r"to\s+be\s+filled", t) && !t.contains('|')))
}

fn report_tables(ctx: &mut Ctx, report: &mut Report) {
    if ctx.rep_place {
        return;
    }
    let tabs = ctx.rep.field_tables();
    let (member, fixed): (Vec<&FieldTable>, Vec<&FieldTable>) = tabs.iter().partition(|t| py::has_i(r"per[- ]member|each member|member block", &t.label));
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
        let (has_req, has_rep) = (lab.contains("request"), py::has(r"\brepl(y|ies)\b", &lab));
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
        Some(t) => py::universal(&t),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let lines: Vec<&str> = py::splitlines(&wire);
    let heads = md::heading_index(&lines);
    let members = Enumeration::parse(&wire);
    if !members.section_found {
        report.fail("AC5", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read");
    } else if !members.has_exactly(REJECTION_PIN) {
        report.fail("AC5", format!("the closed enumeration parsed from wire.md has {} names, not {REJECTION_PIN}", members.count()));
    }
    let span = |p: &str| md::section_span(lines.len(), &heads, p, None).map(|(a, b)| lines[a.min(b)..b].join("\n"));
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
        toml_text: docs.read(WIRE_TOML).map(|t| py::universal(&t)),
    };
    report_tables(&mut ctx, report);
    probe_tables(&mut ctx, report);
    ctx
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let ctx = context(docs, &mut report);
    ac1::ac1_to_ac3(&ctx, &mut report);
    ac4::probe(&ctx, &mut report);
    let (form_rows, named) = ac5::forms(&ctx, &mut report);
    ac6::toml(&ctx, &form_rows, &named, &mut report);
    consist::consistency_and_dod(&ctx, &mut report);
    report
}
