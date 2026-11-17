//! Port of `checks/004-17.sh`: the election message layouts (ticket
//! 004-17).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_17.rs`. Like the 004-16 port it is a transcription
//! with Python semantics, on the shared WP7 helpers of `doccheck::wp7` ([`py`],
//! [`md`], [`pytoml`]). 004-17's own variants: an election message is a
//! `###` subsection of "Election messages" holding a field table; the
//! hello's header and layout rows are read under the label `H`; the
//! election block of the type-code table defaults to 2 to 15; invalid forms
//! are the shared tables plus each message's own (at least 6 per message),
//! whose header has a cell containing "form" or "invalid" and one
//! containing "outcome", columns chosen by the same lowercase test; the
//! categories of `classify` (with `epoch`, `eligibility`, `timestate`); the
//! reason tokens `\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b` negated by
//! `not|never|nor|no` anywhere before them in the clause; "count under X"
//! names; the 24-character negation look-back for `named_reasons`; the
//! open item owned by 005 (`owner[^.;]*?\b005\b`); durations of C-32, C-33
//! and C-37 restated with `(?<![\d.])`; no field named like a timer; toml
//! messages matched by name with a trailing "message(s)" dropped.
//!
//! [`REJECTION_PIN`] is the 13-member pin of line 373. The repository-level
//! assertions (wire-check "checks clean"; wire_tables exactly 12/0/0) are
//! in `scripts/repo-state.d/004-17.sh`.
//!
//! [`py`]: crate::doccheck::wp7::py
//! [`md`]: crate::doccheck::wp7::md
//! [`pytoml`]: crate::doccheck::wp7::pytoml

mod ac1;
mod ac3;
mod ac4;
mod ac5;
pub(crate) mod ac6;

pub mod denials;
use crate::doccheck::wp7::common::form_rows;
use crate::doccheck::wp7::md::{self, Field, Head, fields_of, find_sub, first_table, is_field_table, parse_tables, prose_blocks};
use crate::doccheck::wp7::common::{NUM, const_int, const_val};
use crate::doccheck::wp7::py;
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::wp7::rejection_members::Enumeration;
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

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
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"^denial py_denial_(rejection_reasons_a_number|a_width_as_a_string)$",
    class: DiffClass::ScriptCrashed,
    reason: "the script dies with a Python TypeError (iterating an int; adding a str to the size); the port reports the same cause under CRASH",
}];

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
    pub bound: i128,
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
        self.el_paras.iter().find(|t| pats.iter().all(|p| py::has_i(p, t)))
    }
}

/// `const_dur(cid)`.
fn const_dur(text: &str, cid: &str) -> Option<(String, String)> {
    let v = const_val(text, cid);
    py::search(r"(\d+(?:\.\d+)?)\s*(ms|s)\b", &v, false).map(|m| (m.g(1).to_string(), m.g(2).to_string()))
}

fn constants(docs: &Docs, report: &mut Report) -> (i128, Option<i128>, Option<i128>, [Dur; 3]) {
    let text = docs.read(CONSTANTS_MD).map(|t| py::universal(&t)).unwrap_or_default();
    let bound = const_int(&text, "C-17", &format!(r"at\s+most\s+({NUM})\s*bytes"));
    let dscp = const_int(&text, "C-13", r"control\s+CS5\s*\((\d+)\)");
    let peers = const_int(&text, "C-34", r"at\s+most\s+(\d+)");
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
    (bound.unwrap_or(0), dscp, peers, dur)
}

fn span(lines: &[String], heads: &[Head], pattern: &str, level: Option<usize>) -> Vec<String> {
    md::section_span(lines.len(), heads, pattern, level).map_or_else(Vec::new, |(a, b)| lines[a.min(b)..b].to_vec())
}

/// The landed Hello section: its header, layout, type code, election epoch
/// width and the election block of the type codes.
fn hello(ctx: &mut Ctx, lines: &[String], heads: &[Head], report: &mut Report) {
    if md::section_span(lines.len(), heads, r"^hello$", Some(2)).is_none() {
        report.fail("AC1", "wire.md has no '## Hello' section");
    }
    let subs = md::subsections(&span(lines, heads, r"^hello$", Some(2)));
    let hdr_t = find_sub(&subs, r"common\s+control\s+header").first().and_then(|(_, x)| first_table(x, is_field_table));
    ctx.hdr = fields_of(hdr_t.as_ref(), "H", "the hello's common control header", report);
    ctx.hdr_keys = ctx.hdr.iter().map(|f| md::key_of(&f.meaning)).collect();
    let lay_t = find_sub(&subs, "layout").first().and_then(|(_, x)| first_table(x, is_field_table));
    let lay = fields_of(lay_t.as_ref(), "H", "the hello layout", report);
    for f in &lay {
        if md::key_of(&f.meaning) == Some("type") && ctx.hello_type.is_none() {
            ctx.hello_type = py::search(r"\d+", &f.valid, false).and_then(|m| py::int(m.all()));
        }
        if py::matches(r"\s*election\s+epoch\b", &f.meaning, true).is_some() {
            ctx.hello_epoch_bits = Some(f.bits);
        }
    }
    if ctx.hello_epoch_bits.is_none() {
        report.fail("AC3", "the hello layout has no election epoch row to compare the election messages' width with");
    }
    let mut found = false;
    for (_, text) in find_sub(&subs, r"type\s+codes") {
        for t in parse_tables(text) {
            for row in &t.rows {
                if row.len() > 1 && py::has_i("election", &row[1]) {
                    let nums: Vec<i128> = py::finditer(r"\d+", &row[0], false).iter().filter_map(|m| py::int(m.all())).collect();
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
    if md::section_span(lines.len(), heads, r"^election\s+messages?$", Some(2)).is_none() {
        report.fail("AC1", "wire.md has no '## Election messages' section");
    }
    let el_lines = span(lines, heads, r"^election\s+messages?$", Some(2));
    let el = el_lines.join("\n");
    ctx.el_plain = md::strip_fences(&el);
    ctx.el_subs = md::subsections(&el_lines);
    ctx.el_top = el_lines.iter().take_while(|l| py::matches(r"###\s", l, false).is_none()).cloned().collect::<Vec<_>>().join("\n");
    ctx.el_paras = prose_blocks(&el).into_iter().filter(|(k, _)| *k == 'p').map(|(_, t)| t).collect();
    for (title, text) in &ctx.el_subs {
        match first_table(text, is_field_table) {
            Some(tb) => ctx.msgs.push(Msg {
                title: title.clone(),
                text: text.clone(),
                fields: fields_of(Some(&tb), "AC1", &format!("'{title}'"), report),
                forms: form_rows(text),
                total: 0,
            }),
            None => ctx.shared.extend(form_rows(text)),
        }
    }
    if ctx.msgs.is_empty() {
        report.fail("AC1", "the Election messages section has no election message: a level-3 subsection with a field table is required, at least one beyond the hello");
    }
}

fn context(docs: &Docs, report: &mut Report) -> Ctx {
    let (bound, dscp_control, peers_max, dur) = constants(docs, report);
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
        toml_text: docs.read(WIRE_TOML).map(|t| py::universal(&t)),
    };
    hello(&mut ctx, &lines, &heads, report);
    election(&mut ctx, &lines, &heads, report);
    ctx
}

/// The DoD tag and open-item rules over the section's top text and every
/// `###` subsection (as 004-16's).
fn dod(ctx: &Ctx, report: &mut Report) {
    use crate::doccheck::wp7::common::{is_open_item, words};
    let parts = std::iter::once(("(section)", ctx.el_top.as_str())).chain(ctx.el_subs.iter().map(|(t, x)| (t.as_str(), x.as_str())));
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
    ac1::messages(&mut ctx, &mut report);
    ac3::fields(&ctx, &mut report);
    ac4::forms(&ctx, &mut report);
    ac5::open_item_and_behaviour(&ctx, &mut report);
    dod(&ctx, &mut report);
    ac6::toml(&ctx, &mut report);
    report
}
