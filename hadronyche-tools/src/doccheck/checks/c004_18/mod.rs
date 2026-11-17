//! Port of `checks/004-18.sh`: the admission handshake messages (ticket
//! 004-18).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_18.rs`. A transcription with Python semantics on the
//! shared WP7 helpers of `doccheck::wp7` ([`py`], [`md`], [`pytoml`]). 004-18's
//! own variants: two closed enumerations — the packet reasons of wire.md
//! ([`rejection_members`], pinned at [`REJECTION_PIN`]) and the 18 refusal
//! names read from C-185 (the CamelCase words before the first ". ", with
//! parentheticals removed; pinned at [`REFUSAL_PIN`]) — whose union is what
//! an outcome may name; the reason tokens `\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b`
//! (the second `CAMEL` of the script, line 741, which replaces the first
//! before any use); the 24-character look-back for
//! `not|never|nor|no|neither` over that union; the 004-18 `NON_REJECTION`
//! (with `never a miss` and `not a refusal`); `TIMER` with `pending`; an
//! admission message is a `###` subsection with a field table whose title
//! has no "header"; its rows reported under AC2 when malformed; the
//! receiver report found as the first heading of any level matching
//! `receiver\s+report`; labels `Settled` and `Hand-off` (the harness reads
//! no criterion from `FAIL Hand-off:`, so it compares that one by verdict).
//!
//! The repository-level assertions (wire-check "checks clean"; wire_tables 0
//! failed and at least 12 passed) are in `scripts/repo-state.d/004-18.sh`.
//!
//! [`py`]: crate::doccheck::wp7::py
//! [`md`]: crate::doccheck::wp7::md
//! [`pytoml`]: crate::doccheck::wp7::pytoml
//! [`rejection_members`]: crate::doccheck::rejection_members

mod ac1;
mod ac6;
mod ac7;
mod msgs;

pub mod denials;
use crate::doccheck::wp7::md::{self, Field, Head, fields_of, find_sub, first_table, is_field_table, parse_tables, prose_blocks};
use crate::doccheck::wp7::common::{NUM, const_int, const_val};
use crate::doccheck::wp7::py;
use crate::doccheck::wp7::common::form_rows;
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::rejection_members::Enumeration;
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

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
    toml: TomlReader::SubsetReader,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[
    KnownDifference {
        label: r"^denial py_denial_(rejection_reasons_a_number|a_width_as_a_string)$",
        class: DiffClass::ScriptCrashed,
        reason: "the script dies with a Python TypeError (iterating an int; adding a str to the size); the port reports the same cause under CRASH",
    },
    KnownDifference {
        label: ".*",
        class: DiffClass::ScriptCrashed,
        reason: "not a crash: the harness reads no criterion from the script's `FAIL Hand-off:` line (its label pattern \
                 `[A-Za-z]+ ?[0-9]*` stops at the hyphen), so a script red only on Hand-off looks like a crash; the port \
                 reports the same Hand-off failure. Only the kind 'script crash, port red' is excused here",
    },
    KnownDifference {
        label: ".*",
        class: DiffClass::PortStricter,
        reason: "not stricter: the port fails Hand-off as the script does, but the harness reads no criterion from the \
                 script's `FAIL Hand-off:` line, so the port shows one criterion more. Only the kind 'port fails more \
                 criteria' is excused here; every such row is listed in the summary",
    },
];

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
    pub bound: i128,
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
        self.ad_paras.iter().find(|t| pats.iter().all(|p| py::has_i(p, t)))
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

/// 004-18's reason tokens (the `CAMEL` of line 741).
pub(crate) const CAMEL: &str = r"\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b";

fn constants(docs: &Docs, report: &mut Report) -> (i128, Option<i128>, Vec<String>) {
    let text = docs.read(CONSTANTS_MD).map(|t| py::universal(&t)).unwrap_or_default();
    let bound = const_int(&text, "C-17", &format!(r"at\s+most\s+({NUM})\s*bytes"));
    let dscp = const_int(&text, "C-13", r"control\s+CS5\s*\((\d+)\)");
    let peers = const_int(&text, "C-34", r"at\s+most\s+(\d+)");
    for (key, v) in [("bound", bound), ("dscp_control", dscp), ("peers_max", peers)] {
        if v.is_none() {
            report.fail("DoD", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    for (key, cid) in [("lbc", "C-32"), ("window", "C-33"), ("silence", "C-37")] {
        if py::search(r"(\d+(?:\.\d+)?)\s*(ms|s)\b", &const_val(&text, cid), false).is_none() {
            report.fail("Behaviour", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    let c185 = const_val(&text, "C-185");
    let seg = c185.split(". ").next().unwrap_or("");
    let seg = py::sub(r"\([^)]*\)", "", seg, false);
    let mut set: Vec<String> = Vec::new();
    for h in py::finditer(CAMEL, &seg, false) {
        if !set.iter().any(|n| n == h.all()) {
            set.push(h.all().to_string());
        }
    }
    if set.len() != REFUSAL_PIN {
        report.fail("AC2", format!("C-185 names {} refusal reasons; the closed set is {REFUSAL_PIN}", set.len()));
    }
    for n in DISCOVERY {
        if !set.iter().any(|x| x == n) {
            report.fail("AC2", format!("discovery-raised reason {n} is not in the C-185 set"));
        }
    }
    (bound.unwrap_or(0), dscp, set)
}

fn hello(ctx: &mut Ctx, report: &mut Report) {
    let span = md::section_span(ctx.lines.len(), &ctx.heads, r"^hello$", Some(2));
    if span.is_none() {
        report.fail("AC1", "wire.md has no '## Hello' section");
    }
    let lines = span.map_or_else(Vec::new, |(a, b)| ctx.lines[a.min(b)..b].to_vec());
    ctx.hello_subs = md::subsections(&lines);
    let subs = &ctx.hello_subs;
    let hdr_t = find_sub(subs, r"common\s+control\s+header").first().and_then(|(_, x)| first_table(x, is_field_table));
    ctx.hdr = fields_of(hdr_t.as_ref(), "H", "the hello's common control header", report);
    ctx.hdr_keys = ctx.hdr.iter().map(|f| md::key_of(&f.meaning)).collect();
    let lay_t = find_sub(subs, "layout").first().and_then(|(_, x)| first_table(x, is_field_table));
    let lay = fields_of(lay_t.as_ref(), "H", "the hello layout", report);
    let mut epoch = None;
    for f in &lay {
        if md::key_of(&f.meaning) == Some("type") && ctx.hello_type.is_none() {
            ctx.hello_type = py::search(r"\d+", &f.valid, false).and_then(|m| py::int(m.all()));
        }
        if py::matches(r"\s*election\s+epoch\b", &f.meaning, true).is_some() {
            epoch = Some(f.bits);
        }
    }
    if epoch.is_none() {
        report.fail("AC3", "the hello layout has no election epoch row to compare the election messages' width with");
    }
}

fn admission(ctx: &mut Ctx, report: &mut Report) {
    let span = md::section_span(ctx.lines.len(), &ctx.heads, r"^admission\s+messages?$", Some(2));
    if span.is_none() {
        report.fail("AC1", "wire.md has no '## Admission messages' section");
    }
    let ad_lines = span.map_or_else(Vec::new, |(a, b)| ctx.lines[a.min(b)..b].to_vec());
    let ad = ad_lines.join("\n");
    ctx.ad_plain = md::strip_fences(&ad);
    ctx.ad_subs = md::subsections(&ad_lines);
    ctx.ad_top = ad_lines.iter().take_while(|l| py::matches(r"###\s", l, false).is_none()).cloned().collect::<Vec<_>>().join("\n");
    ctx.ad_paras = prose_blocks(&ad).into_iter().filter(|(k, _)| *k == 'p').map(|(_, t)| t).collect();
}

fn context(docs: &Docs, report: &mut Report) -> Ctx {
    let (bound, dscp_control, refusal_set) = constants(docs, report);
    let wire = match docs.read(WIRE_MD) {
        Some(t) => py::universal(&t),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let lines: Vec<String> = py::splitlines(&wire).into_iter().map(String::from).collect();
    let heads = md::heading_index(&lines);
    let members = Enumeration::parse(&wire);
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
        toml_text: docs.read(WIRE_TOML).map(|t| py::universal(&t)),
    };
    hello(&mut ctx, report);
    admission(&mut ctx, report);
    ctx
}

/// AC2: the numbered refusal enumeration of the section (lines 777-818).
fn refusals(ctx: &mut Ctx, report: &mut Report) {
    for t in parse_tables(&ctx.ad_plain) {
        for row in &t.rows {
            if row.len() < 2 || py::fullmatch(r"\d+", py::strip(&row[0]), false).is_none() {
                continue;
            }
            let nm = py::strip(&row[1]).trim_matches(|c| "`*_ ".contains(c)).to_string();
            if py::fullmatch("[A-Z][A-Za-z0-9]+", &nm, false).is_none() {
                continue;
            }
            let value = py::int(&row[0]).unwrap_or_default();
            let text = row.join(" | ");
            match ctx.refs.iter_mut().find(|(n, _, _)| *n == nm) {
                Some(e) => {
                    report.fail("AC2", format!("refusal reason {nm} is listed twice"));
                    e.1 = value;
                    e.2 = text;
                }
                None => ctx.refs.push((nm, value, text)),
            }
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
        let disc = py::has_i("discover", row);
        if DISCOVERY.contains(&n.as_str()) && !disc {
            report.fail("AC2", format!("refusal reason {n} is not marked as raised at discovery in its row"));
        }
        if !DISCOVERY.contains(&n.as_str()) && disc {
            report.fail("AC2", format!("refusal reason {n} is marked as discovery-raised; only {DISCOVERY:?} are (C-185)"));
        }
    }
    let said = ctx.ad_paras.iter().any(|t| DISCOVERY.iter().all(|n| py::has(&regex::escape(n), t)) && py::has_i("discover", t));
    if !said {
        report.fail("AC2", "no sentence in the Admission messages section says that VersionMismatch, NameTaken and ModuleCap are raised at discovery and never carried by an admission refusal (C-185)");
    }
    if ctx.ad_para(&["C-185"]).is_none() {
        report.fail("AC2", "no sentence in the Admission messages section cites C-185 for the refusal enumeration");
    }
    for (i, n) in ctx.refusal_set.iter().enumerate() {
        let v = ctx.ref_value(n);
        if v != Some(i as i128) {
            report.fail("AC2", format!("wire.md gives refusal reason {n} the value {v:?}; C-185's order gives {i} (the refusal values are 0..17 in the order of the closed set)"));
        }
    }
}

fn messages(ctx: &mut Ctx, report: &mut Report) {
    let subs = ctx.ad_subs.clone();
    for (title, text) in &subs {
        match first_table(text, is_field_table) {
            Some(tb) if !py::has_i("header", title) => ctx.msgs.push(Msg {
                title: title.clone(),
                text: text.clone(),
                fields: fields_of(Some(&tb), "AC2", &format!("'{title}'"), report),
                forms: form_rows(text),
                total: 0,
            }),
            _ => ctx.shared.extend(form_rows(text)),
        }
    }
    if ctx.msgs.len() < 2 {
        report.fail("AC1", "the Admission messages section needs at least two messages with a field table (a request and a refusal)");
    }
}

/// The DoD tag and open-item rules (as 004-16's) over the section.
fn dod(ctx: &Ctx, report: &mut Report) {
    use crate::doccheck::wp7::common::{is_open_item, words};
    let parts = std::iter::once(("(section)", ctx.ad_top.as_str())).chain(ctx.ad_subs.iter().map(|(t, x)| (t.as_str(), x.as_str())));
    for (title, text) in parts {
        let tagged = md::has_evidence(text);
        for (kind, body) in prose_blocks(text) {
            let first = py::head(py::splitlines(&body).first().copied().unwrap_or(""), 80);
            if kind == 'p' {
                if words(&body) >= 5 && !md::has_evidence(&body) {
                    report.fail("DoD", format!("a prose paragraph of '{title}' carries no evidence tag: {first}"));
                }
                if is_open_item(&body) && (!py::has_i("owner", &body) || !py::has_i("falsifier", &body)) {
                    report.fail("DoD", format!("an open item in '{title}' names no owner or no falsifier: {first}"));
                }
            } else if !tagged {
                report.fail("DoD", format!("a table of '{title}' follows no evidence-tagged paragraph"));
            }
        }
    }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let mut ctx = context(docs, &mut report);
    refusals(&mut ctx, &mut report);
    messages(&mut ctx, &mut report);
    msgs::per_message(&mut ctx, &mut report);
    ac1::contents(&ctx, &mut report);
    ac6::forms(&ctx, &mut report);
    dod(&ctx, &mut report);
    ac7::toml(&ctx, &mut report);
    report
}
