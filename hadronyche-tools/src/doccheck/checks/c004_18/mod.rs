//! Port of `checks/004-18.sh`: the admission handshake messages (ticket
//! 004-18).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_18.rs`. A transcription with Python semantics on the
//! library's named variants ([`family004`], `REJECTION_004_16` pinned at
//! [`REJECTION_PIN`], `refusal_set` (C-185, pinned at [`REFUSAL_PIN`]),
//! `numbered_rows` under `RowRule::Loose`, `CAMEL_REFUSAL_004_18`,
//! `LOOKBACK_004_16`, `NON_REJECTION_004_18`, `FormRows::AnyCell004_17`, the
//! DoD of 004-16, [`crate::doccheck::tomlsubset`]). 004-18's own variants,
//! private to this port: the union of the packet reasons and the 18 refusal
//! names is what an outcome may name; `TIMER` with `pending`; an admission
//! message is a `###` subsection with a field table whose title has no
//! "header", its rows reported under AC2 when malformed; the receiver report
//! found as the first heading of any level matching `receiver\s+report`;
//! labels `Settled` and `Hand-off` (read by the harness's default reader,
//! hyphen included). A Python exception stops the check (`Stop::Crash`); an
//! integer beyond `i128` is red by rule (`INT_LIMIT`).
//!
//! The repository-level assertions (wire-check "checks clean"; wire_tables 0
//! failed and at least 12 passed) are in `scripts/repo-state.d/004-18.sh`.

mod ac1;
mod ac6;
mod ac7;
mod msgs;

pub mod denials;
use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::blocks::{FormRows, form_rows};
use crate::doccheck::equiv::script::DEFAULT_READER;
use crate::doccheck::evidence::{TableTagRule, TagPolicy, untagged, untagged_tables};
use crate::doccheck::family004::{self, ConstInt, Field, Head, IntRule, NUM_004_16, SpanRule, fields_of, find_sub, first_field_table};
use crate::doccheck::items::{OpenItemPolicy, unowned_items};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{IntError, parse_int, splitlines, universal_newlines};
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::rejection_members::{Enumeration, REJECTION_004_16, RowRule, numbered_rows, refusal_set};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has, has_i, head, rx, starts};
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

/// `section_span(pattern, level)` of the 004-16 family.
pub(crate) fn section_span(n: usize, heads: &[Head], pattern: &str, level: Option<usize>) -> Option<(usize, usize)> {
    family004::section_span(n, heads, pattern, level, SpanRule::AfterHeading004_16)
}

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The number of members the closed rejection enumeration must have
/// (`len(MEMBERS) != 13`, script line 368).
pub const REJECTION_PIN: usize = 13;

/// The number of refusal reasons C-185 must name (line 746).
pub const REFUSAL_PIN: usize = 18;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-18",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Admission messages$"),
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
            file: WIRE_MD,
            heading: Some("^Receiver report"),
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
    keywords: &["discovery", "refused", "rejection", "ignored", "order", "drop", "carried"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The reasons raised at discovery, never carried by a refusal.
pub(crate) const DISCOVERY: [&str; 3] = ["VersionMismatch", "NameTaken", "ModuleCap"];

/// One admission message.
#[derive(Debug, Clone)]
pub(crate) struct Msg {
    pub title: String,
    pub text: String,
    pub fields: Vec<Field>,
    pub forms: Vec<(String, String)>,
    pub total: i128,
}

/// What the script reads.
pub(crate) struct Ctx {
    /// `K["bound"]` (None when unreadable; falsy like 0).
    pub bound: Option<i128>,
    pub dscp_control: Option<i128>,
    pub members: Enumeration,
    pub refusal_set: Vec<String>,
    pub lines: Vec<String>,
    pub heads: Vec<Head>,
    pub hello_subs: Vec<(String, String)>,
    pub hdr: Vec<Field>,
    pub hdr_keys: Vec<Option<&'static str>>,
    pub hello_type: Option<i128>,
    pub ad_plain: String,
    pub ad_top: String,
    pub ad_subs: Vec<(String, String)>,
    pub ad_paras: Vec<String>,
    pub msgs: Vec<Msg>,
    pub shared: Vec<(String, String)>,
    /// The numbered refusal rows: (name, value, row text), in order.
    pub refs: Vec<(String, i128, String)>,
    pub toml_text: Option<String>,
}

impl Ctx {
    /// `ad_para(*pats)`.
    pub fn ad_para(&self, pats: &[&str]) -> Option<&String> {
        self.ad_paras.iter().find(|t| pats.iter().all(|p| has_i(p, t)))
    }

    /// `name in UNION`.
    pub fn in_union(&self, name: &str) -> bool {
        self.members.contains(name) || self.refusal_set.iter().any(|n| n == name)
    }

    /// `REF.get(name)`.
    pub fn ref_value(&self, name: &str) -> Option<i128> {
        self.refs.iter().find(|(n, _, _)| n == name).map(|(_, v, _)| *v)
    }
}

fn constants(docs: &Docs, report: &mut Report) -> Flow<(Option<i128>, Option<i128>, Vec<String>)> {
    let text = docs.read(CONSTANTS_MD).map(|t| universal_newlines(&t)).unwrap_or_default();
    let bound = const_int(&text, "C-17", &format!(r"at\s+most\s+({NUM})\s*bytes"))?;
    let dscp = const_int(&text, "C-13", r"control\s+CS5\s*\((\d+)\)")?;
    let peers = const_int(&text, "C-34", r"at\s+most\s+(\d+)")?;
    for (key, v) in [("bound", bound), ("dscp_control", dscp), ("peers_max", peers)] {
        if v.is_none() {
            report.fail("DoD", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    for (key, cid) in [("lbc", "C-32"), ("window", "C-33"), ("silence", "C-37")] {
        if !rx(r"(\d+(?:\.\d+)?)\s*(ms|s)\b").is_match(&family004::const_cell(&text, cid)) {
            report.fail("Behaviour", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    let set = refusal_set(&family004::const_cell(&text, "C-185"));
    if set.len() != REFUSAL_PIN {
        report.fail("AC2", format!("C-185 names {} refusal reasons; the closed set is {REFUSAL_PIN}", set.len()));
    }
    for n in DISCOVERY {
        if !set.iter().any(|x| x == n) {
            report.fail("AC2", format!("discovery-raised reason {n} is not in the C-185 set"));
        }
    }
    Ok((bound, dscp, set))
}

fn hello(ctx: &mut Ctx, report: &mut Report) {
    let span = section_span(ctx.lines.len(), &ctx.heads, r"^hello$", Some(2));
    if span.is_none() {
        report.fail("AC1", "wire.md has no '## Hello' section");
    }
    let lines = span.map_or_else(Vec::new, |(a, b)| ctx.lines[a.min(b)..b].to_vec());
    ctx.hello_subs = family004::subsections(&lines.join("\n"));
    let subs = &ctx.hello_subs;
    let hdr_t = find_sub(subs, r"common\s+control\s+header").first().and_then(|(_, x)| first_field_table(x));
    ctx.hdr = fields_of(hdr_t.as_ref(), "H", "the hello's common control header", report);
    ctx.hdr_keys = ctx.hdr.iter().map(|f| family004::key_of(&f.meaning)).collect();
    let lay_t = find_sub(subs, "layout").first().and_then(|(_, x)| first_field_table(x));
    let lay = fields_of(lay_t.as_ref(), "H", "the hello layout", report);
    let mut epoch = None;
    for f in &lay {
        if family004::key_of(&f.meaning) == Some("type") && ctx.hello_type.is_none() {
            // beyond i128: INT_LIMIT (recorded)
            ctx.hello_type = rx(r"\d+").find(&f.valid).and_then(|m| parse_int(m.as_str()).ok());
        }
        if rx(r"(?i)^(?:\s*election\s+epoch\b)").is_match(&f.meaning) {
            epoch = Some(f.bits);
        }
    }
    if epoch.is_none() {
        report.fail("AC3", "the hello layout has no election epoch row to compare the election messages' width with");
    }
}

fn admission(ctx: &mut Ctx, report: &mut Report) {
    let span = section_span(ctx.lines.len(), &ctx.heads, r"^admission\s+messages?$", Some(2));
    if span.is_none() {
        report.fail("AC1", "wire.md has no '## Admission messages' section");
    }
    let ad_lines = span.map_or_else(Vec::new, |(a, b)| ctx.lines[a.min(b)..b].to_vec());
    let ad = ad_lines.join("\n");
    ctx.ad_plain = strip_fences(&ad);
    ctx.ad_subs = family004::subsections(&ad);
    ctx.ad_top = ad_lines.iter().take_while(|l| !starts(r"###\s", l)).cloned().collect::<Vec<_>>().join("\n");
    ctx.ad_paras = prose_blocks(&ad).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text).collect();
}

fn context(docs: &Docs, report: &mut Report) -> Flow<Ctx> {
    let (bound, dscp_control, refusal_set) = constants(docs, report)?;
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
        members,
        refusal_set,
        lines,
        heads,
        hello_subs: Vec::new(),
        hdr: Vec::new(),
        hdr_keys: Vec::new(),
        hello_type: None,
        ad_plain: String::new(),
        ad_top: String::new(),
        ad_subs: Vec::new(),
        ad_paras: Vec::new(),
        msgs: Vec::new(),
        shared: Vec::new(),
        refs: Vec::new(),
        toml_text: docs.read(WIRE_TOML).map(|t| universal_newlines(&t)),
    };
    hello(&mut ctx, report);
    admission(&mut ctx, report);
    Ok(ctx)
}

/// AC2: the numbered refusal enumeration of the section (lines 777-818).
fn refusals(ctx: &mut Ctx, report: &mut Report) {
    for m in numbered_rows(&ctx.ad_plain, RowRule::Loose) {
        // `int(row[0])`: beyond i128 it is INT_LIMIT (recorded) and the row
        // goes on with 0
        let value = m.int().unwrap_or_default();
        let text = m.cells.join(" | ");
        match ctx.refs.iter_mut().find(|(n, _, _)| *n == m.name) {
            Some(e) => {
                report.fail("AC2", format!("refusal reason {} is listed twice", m.name));
                e.1 = value;
                e.2 = text;
            }
            None => ctx.refs.push((m.name, value, text)),
        }
    }
    if ctx.refs.is_empty() {
        report.fail("AC2", "the Admission messages section has no numbered refusal enumeration (value, name rows)");
    }
    let names: std::collections::BTreeSet<&str> = ctx.refs.iter().map(|(n, _, _)| n.as_str()).collect();
    let want: std::collections::BTreeSet<&str> = ctx.refusal_set.iter().map(String::as_str).collect();
    if names != want {
        report.fail("AC2", format!("the refusal enumeration names differ from C-185: missing {:?}, extra {:?}", want.difference(&names).collect::<Vec<_>>(), names.difference(&want).collect::<Vec<_>>()));
    }
    let mut values: Vec<i128> = ctx.refs.iter().map(|(_, v, _)| *v).collect();
    values.sort();
    values.dedup();
    if values.len() != ctx.refs.len() {
        report.fail("AC2", "the refusal reason values are not unique");
    }
    for (n, v, _) in &ctx.refs {
        if !(0..=255).contains(v) {
            report.fail("AC2", format!("refusal reason {n} has value {v}, outside a u8"));
        }
    }
    for (n, _, row) in &ctx.refs {
        let disc = has_i("discover", row);
        if DISCOVERY.contains(&n.as_str()) && !disc {
            report.fail("AC2", format!("refusal reason {n} is not marked as raised at discovery in its row"));
        }
        if !DISCOVERY.contains(&n.as_str()) && disc {
            report.fail("AC2", format!("refusal reason {n} is marked as discovery-raised; only {DISCOVERY:?} are (C-185)"));
        }
    }
    let said = ctx.ad_paras.iter().any(|t| DISCOVERY.iter().all(|n| has(&regex::escape(n), t)) && has_i("discover", t));
    if !said {
        report.fail("AC2", "no sentence in the Admission messages section says that VersionMismatch, NameTaken and ModuleCap are raised at discovery and never carried by an admission refusal (C-185)");
    }
    if ctx.ad_para(&["C-185"]).is_none() {
        report.fail("AC2", "no sentence in the Admission messages section cites C-185 for the refusal enumeration");
    }
    for (i, n) in ctx.refusal_set.iter().enumerate() {
        let v = ctx.ref_value(n);
        if v != i128::try_from(i).ok() {
            report.fail("AC2", format!("wire.md gives refusal reason {n} the value {v:?}; C-185's order gives {i} (the refusal values are 0..17 in the order of the closed set)"));
        }
    }
}

fn messages(ctx: &mut Ctx, report: &mut Report) {
    let pairs = |text: &str| -> Vec<(String, String)> { form_rows(text, FormRows::AnyCell004_17).into_iter().map(|r| (r[0].clone(), r[1].clone())).collect() };
    let subs = ctx.ad_subs.clone();
    for (title, text) in &subs {
        match first_field_table(text) {
            Some(tb) if !has_i("header", title) => ctx.msgs.push(Msg {
                title: title.clone(),
                text: text.clone(),
                fields: fields_of(Some(&tb), "AC2", &format!("'{title}'"), report),
                forms: pairs(text),
                total: 0,
            }),
            _ => ctx.shared.extend(pairs(text)),
        }
    }
    if ctx.msgs.len() < 2 {
        report.fail("AC1", "the Admission messages section needs at least two messages with a field table (a request and a refusal)");
    }
}

/// The DoD tag and open-item rules (as 004-16's) over the section.
fn dod(ctx: &Ctx, report: &mut Report) {
    let first_line = |text: &str| head(splitlines(text).first().copied().unwrap_or(""), 80);
    let parts = std::iter::once(("(section)", ctx.ad_top.as_str())).chain(ctx.ad_subs.iter().map(|(t, x)| (t.as_str(), x.as_str())));
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
    refusals(&mut ctx, report);
    messages(&mut ctx, report);
    msgs::per_message(&mut ctx, report)?;
    ac1::contents(&ctx, report);
    ac6::forms(&ctx, report);
    dod(&ctx, report);
    ac7::toml(&ctx, report)
}
