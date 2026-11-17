//! Port of `checks/004-17.sh`: the election message layouts (ticket
//! 004-17).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_17.rs`. Like the 004-16 port it is a transcription
//! with Python semantics on the library's named variants ([`family004`],
//! `REJECTION_004_16`, `LOOKBACK_004_16`, `FormRows::AnyCell004_17`, the DoD
//! of 004-16, [`crate::doccheck::tomlsubset`]). 004-17's own variants, private
//! to this port: an election message is a `###` subsection of "Election
//! messages" holding a field table; the hello's header and layout rows are
//! read under the label `H`; the election block of the type-code table
//! defaults to 2 to 15; at least 6 invalid forms per message (the shared
//! tables plus its own); the categories of `classify` (with `epoch`,
//! `eligibility`, `timestate`); the reason tokens negated by
//! `not|never|nor|no` anywhere before them in the clause; "count under X"
//! names; the open item owned by 005 (`owner[^.;]*?\b005\b`); durations of
//! C-32, C-33 and C-37 restated with `(?<![\d.])`; no field named like a
//! timer; toml messages matched by name with a trailing "message(s)"
//! dropped. A Python exception stops the check (`Stop::Crash`); an integer
//! beyond `i128` is red by rule (`INT_LIMIT`).
//!
//! [`REJECTION_PIN`] is the 13-member pin of line 373. The repository-level
//! assertions (wire-check "checks clean"; wire_tables exactly 12/0/0) are
//! in `scripts/repo-state.d/004-17.sh`.

mod ac1;
mod ac3;
mod ac4;
mod ac5;
pub(crate) mod ac6;

pub mod denials;
use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::blocks::{FormRows, form_rows};
use crate::doccheck::equiv::script::DEFAULT_READER;
use crate::doccheck::evidence::{TableTagRule, TagPolicy, untagged, untagged_tables};
use crate::doccheck::family004::{self, ConstInt, Field, Head, IntRule, NUM_004_16, SpanRule, fields_of, find_sub, first_field_table, tables};
use crate::doccheck::items::{OpenItemPolicy, unowned_items};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{IntError, parse_int, splitlines, universal_newlines};
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::rejection_members::{Enumeration, REJECTION_004_16};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has_i, head, rx, starts};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::units::{BlockKind, prose_blocks, strip_fences};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// `NUM` (004-16's digit groups).
pub(crate) const NUM: &str = NUM_004_16;

/// `to_int(s)`; a Python `ValueError` is a crash, beyond `i128` is
/// `INT_LIMIT` (recorded by the library).
pub(crate) fn to_int(s: &str) -> Flow<Option<i128>> {
    match family004::to_int(s, IntRule::Spaces004_16) {
        Ok(v) => Ok(Some(v)),
        Err(IntError::Overflow(_)) => Ok(None),
        Err(e) => crash(format!("ValueError: {e}")),
    }
}

/// `const_int(cid, pattern)`; a Python `ValueError` is a crash.
fn const_int(text: &str, cid: &str, pattern: &str) -> Flow<Option<i128>> {
    match family004::const_int(text, cid, pattern, ConstInt::AnyCase004_16) {
        Ok(v) => Ok(v),
        Err(IntError::Overflow(_)) => Ok(None),
        Err(e) => crash(format!("ValueError: {e}")),
    }
}

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

/// `size_bits(f)`: `f["bits"] * (f["maxc"] if f["maxc"] else 1)`.
pub(crate) fn size_bits(f: &Field) -> i128 {
    int_mul(f.bits, f.maxc.filter(|&m| m != 0).unwrap_or(1))
}

/// `[int(x) for x in re.findall(r"\d+", text)]` (beyond `i128`: `INT_LIMIT`,
/// left out).
fn ints(text: &str) -> Vec<i128> {
    rx(r"\d+").find_iter(text).filter_map(|m| parse_int(m.as_str()).ok()).collect()
}

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The number of members the closed rejection enumeration must have
/// (`len(MEMBERS) != 13`, script line 373).
pub const REJECTION_PIN: usize = 13;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-17",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Election messages$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Hello$"),
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
    keywords: &["compared", "unsigned", "kept", "bumped", "rejection", "ignored", "reuse", "carries"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// One election message: a `###` subsection with a field table.
#[derive(Debug, Clone)]
pub(crate) struct Msg {
    pub title: String,
    pub text: String,
    pub fields: Vec<Field>,
    pub forms: Vec<(String, String)>,
    pub total: i128,
}

/// (key, constant id, its (number, unit) when readable).
pub(crate) type Dur = (&'static str, &'static str, Option<(String, String)>);

/// What the script reads, as it reads it.
pub(crate) struct Ctx {
    /// `K["bound"]` (None when unreadable; falsy like 0).
    pub bound: Option<i128>,
    pub dscp_control: Option<i128>,
    pub peers_max: Option<i128>,
    /// (number, unit) of C-32, C-33, C-37.
    pub dur: [Dur; 3],
    pub members: Enumeration,
    pub hdr: Vec<Field>,
    pub hdr_keys: Vec<Option<&'static str>>,
    pub hello_type: Option<i128>,
    pub hello_epoch_bits: Option<i128>,
    pub elec: (i128, i128),
    pub el_plain: String,
    pub el_top: String,
    pub el_subs: Vec<(String, String)>,
    pub el_paras: Vec<String>,
    pub msgs: Vec<Msg>,
    pub shared: Vec<(String, String)>,
    pub toml_text: Option<String>,
}

impl Ctx {
    /// `el_para(*pats)`.
    pub fn el_para(&self, pats: &[&str]) -> Option<&String> {
        self.el_paras.iter().find(|t| pats.iter().all(|p| has_i(p, t)))
    }
}

/// `const_dur(cid)`.
fn const_dur(text: &str, cid: &str) -> Option<(String, String)> {
    let v = family004::const_cell(text, cid);
    rx(r"(\d+(?:\.\d+)?)\s*(ms|s)\b").captures(&v).map(|m| (m.get(1).map_or("", |m| m.as_str()).to_string(), m.get(2).map_or("", |m| m.as_str()).to_string()))
}

type Consts = (Option<i128>, Option<i128>, Option<i128>, [Dur; 3]);

fn constants(docs: &Docs, report: &mut Report) -> Flow<Consts> {
    let text = docs.read(CONSTANTS_MD).map(|t| universal_newlines(&t)).unwrap_or_default();
    let bound = const_int(&text, "C-17", &format!(r"at\s+most\s+({NUM})\s*bytes"))?;
    let dscp = const_int(&text, "C-13", r"control\s+CS5\s*\((\d+)\)")?;
    let peers = const_int(&text, "C-34", r"at\s+most\s+(\d+)")?;
    for (key, v) in [("bound", bound), ("dscp_control", dscp), ("peers_max", peers)] {
        if v.is_none() {
            report.fail("DoD", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    let dur = [
        ("lbc", "C-32", const_dur(&text, "C-32")),
        ("window", "C-33", const_dur(&text, "C-33")),
        ("silence", "C-37", const_dur(&text, "C-37")),
    ];
    for (key, _, v) in &dur {
        if v.is_none() {
            report.fail("Behaviour", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    Ok((bound, dscp, peers, dur))
}

fn span(lines: &[String], heads: &[Head], pattern: &str, level: Option<usize>) -> Vec<String> {
    family004::section_span(lines.len(), heads, pattern, level, SpanRule::AfterHeading004_16).map_or_else(Vec::new, |(a, b)| lines[a.min(b)..b].to_vec())
}

/// The landed Hello section: its header, layout, type code, election epoch
/// width and the election block of the type codes.
fn hello(ctx: &mut Ctx, lines: &[String], heads: &[Head], report: &mut Report) {
    if family004::section_span(lines.len(), heads, r"^hello$", Some(2), SpanRule::AfterHeading004_16).is_none() {
        report.fail("AC1", "wire.md has no '## Hello' section");
    }
    let subs = family004::subsections(&span(lines, heads, r"^hello$", Some(2)).join("\n"));
    let hdr_t = find_sub(&subs, r"common\s+control\s+header").first().and_then(|(_, x)| first_field_table(x));
    ctx.hdr = fields_of(hdr_t.as_ref(), "H", "the hello's common control header", report);
    ctx.hdr_keys = ctx.hdr.iter().map(|f| family004::key_of(&f.meaning)).collect();
    let lay_t = find_sub(&subs, "layout").first().and_then(|(_, x)| first_field_table(x));
    let lay = fields_of(lay_t.as_ref(), "H", "the hello layout", report);
    for f in &lay {
        if family004::key_of(&f.meaning) == Some("type") && ctx.hello_type.is_none() {
            ctx.hello_type = rx(r"\d+").find(&f.valid).and_then(|m| parse_int(m.as_str()).ok());
        }
        if rx(r"(?i)^(?:\s*election\s+epoch\b)").is_match(&f.meaning) {
            ctx.hello_epoch_bits = Some(f.bits);
        }
    }
    if ctx.hello_epoch_bits.is_none() {
        report.fail("AC3", "the hello layout has no election epoch row to compare the election messages' width with");
    }
    let mut found = false;
    for (_, text) in find_sub(&subs, r"type\s+codes") {
        for t in tables(text) {
            for row in t.rows.iter().map(|r| &r.cells) {
                if row.len() > 1 && has_i("election", &row[1]) {
                    let nums = ints(&row[0]);
                    if let (Some(a), Some(b)) = (nums.first(), nums.last()) {
                        ctx.elec = (*a, *b);
                        found = true;
                    }
                }
            }
        }
    }
    if !found {
        report.fail("AC1", "the Hello section's Control message type codes table does not give the block for election messages");
    }
}

fn election(ctx: &mut Ctx, lines: &[String], heads: &[Head], report: &mut Report) {
    if family004::section_span(lines.len(), heads, r"^election\s+messages?$", Some(2), SpanRule::AfterHeading004_16).is_none() {
        report.fail("AC1", "wire.md has no '## Election messages' section");
    }
    let el_lines = span(lines, heads, r"^election\s+messages?$", Some(2));
    let el = el_lines.join("\n");
    ctx.el_plain = strip_fences(&el);
    ctx.el_subs = family004::subsections(&el);
    ctx.el_top = el_lines.iter().take_while(|l| !starts(r"###\s", l)).cloned().collect::<Vec<_>>().join("\n");
    ctx.el_paras = prose_blocks(&el).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text).collect();
    let pairs = |text: &str| -> Vec<(String, String)> { form_rows(text, FormRows::AnyCell004_17).into_iter().map(|r| (r[0].clone(), r[1].clone())).collect() };
    for (title, text) in &ctx.el_subs {
        match first_field_table(text) {
            Some(tb) => ctx.msgs.push(Msg {
                title: title.clone(),
                text: text.clone(),
                fields: fields_of(Some(&tb), "AC1", &format!("'{title}'"), report),
                forms: pairs(text),
                total: 0,
            }),
            None => ctx.shared.extend(pairs(text)),
        }
    }
    if ctx.msgs.is_empty() {
        report.fail("AC1", "the Election messages section has no election message: a level-3 subsection with a field table is required, at least one beyond the hello");
    }
}

fn context(docs: &Docs, report: &mut Report) -> Flow<Ctx> {
    let (bound, dscp_control, peers_max, dur) = constants(docs, report)?;
    let wire = match docs.read(WIRE_MD) {
        Some(t) => universal_newlines(&t),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let lines: Vec<String> = splitlines(&wire).into_iter().map(String::from).collect();
    let heads = family004::heading_index(&lines);
    let members = Enumeration::parse(&wire, REJECTION_004_16);
    if !members.section_found {
        report.fail("AC4", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read");
    } else if !members.has_exactly(REJECTION_PIN) {
        report.fail("AC4", format!("the closed enumeration parsed from wire.md has {} names, not {REJECTION_PIN}", members.count()));
    }
    let mut ctx = Ctx {
        bound,
        dscp_control,
        peers_max,
        dur,
        members,
        hdr: Vec::new(),
        hdr_keys: Vec::new(),
        hello_type: None,
        hello_epoch_bits: None,
        elec: (2, 15),
        el_plain: String::new(),
        el_top: String::new(),
        el_subs: Vec::new(),
        el_paras: Vec::new(),
        msgs: Vec::new(),
        shared: Vec::new(),
        toml_text: docs.read(WIRE_TOML).map(|t| universal_newlines(&t)),
    };
    hello(&mut ctx, &lines, &heads, report);
    election(&mut ctx, &lines, &heads, report);
    Ok(ctx)
}

/// The DoD tag and open-item rules over the section's top text and every
/// `###` subsection (as 004-16's).
fn dod(ctx: &Ctx, report: &mut Report) {
    let first_line = |text: &str| head(splitlines(text).first().copied().unwrap_or(""), 80);
    let parts = std::iter::once(("(section)", ctx.el_top.as_str())).chain(ctx.el_subs.iter().map(|(t, x)| (t.as_str(), x.as_str())));
    for (title, text) in parts {
        for u in untagged(text, &TagPolicy::ALPHA_004_16) {
            report.fail("DoD", format!("a prose paragraph of '{title}' carries no evidence tag: {}", first_line(&u.text)));
        }
        for item in unowned_items(text, OpenItemPolicy::LeadingClean) {
            report.fail("DoD", format!("an open item in '{title}' names no owner or no falsifier: {}", first_line(&item.text)));
        }
        for _ in untagged_tables(text, TableTagRule::SectionTagged004_16) {
            report.fail("DoD", format!("a table of '{title}' follows no evidence-tagged paragraph"));
        }
    }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let flow = body(docs, &mut report);
    report.finish(flow)
}

/// The script, in its order; a Python exception stops it.
fn body(docs: &Docs, report: &mut Report) -> Flow<()> {
    let mut ctx = context(docs, report)?;
    ac1::messages(&mut ctx, report)?;
    ac3::fields(&ctx, report);
    ac4::forms(&ctx, report);
    ac5::open_item_and_behaviour(&ctx, report);
    dod(&ctx, report);
    ac6::toml(&ctx, report)
}
