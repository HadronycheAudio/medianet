//! Port of `checks/004-16.sh`: the hello message layout and the
//! 32-member size bound (ticket 004-16).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_16.rs`. The port is a transcription of the script with
//! Python semantics, because its rules are its own variants (inventory
//! section 4): the 004-16/17/18 Markdown helpers ([`md`]: `section_span`
//! at any level or an exact one, tables only with a separator line, `###`
//! subsections ending at the next heading of level 3 or higher, prose
//! blocks split by blank lines, headings and table/prose changes); the
//! closed enumeration by [`rejection_members`] pinned at
//! [`REJECTION_PIN`]; the reason tokens `\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b`;
//! the 24-character negation look-back for `not|never|nor|no|neither`; the
//! 004-16 `NON_REJECTION` pattern; at least ten invalid forms in the first
//! table of an `invalid|validity` subsection with a form and an outcome
//! column; the evidence rule (five `[A-Za-z]{2,}` words outside code spans;
//! a table needs a tag somewhere in its subsection); open items as
//! paragraphs starting `open item(s)`; wire.toml read with the scripts'
//! subset reader (`pytoml`).
//!
//! The Markdown helpers, the Python semantics and the TOML subset reader
//! are the shared WP7 module [`crate::doccheck::wp7`] (004-17 to 004-20 use
//! them too; no port imports another port).
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` is
//! exactly 12 passed, 0 failed, 0 ignored) are in
//! `scripts/repo-state.d/004-16.sh`.
//!
//! [`rejection_members`]: crate::doccheck::wp7::rejection_members

pub(crate) mod ac1;
pub(crate) mod ac2;
mod ac3;
pub(crate) mod ac4;
pub(crate) mod ac5;
pub(crate) mod ac6;
pub(crate) mod dod;

pub mod denials;
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::wp7::rejection_members::Enumeration;
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};
use crate::doccheck::wp7::common::{NUM, const_int, to_int};
use crate::doccheck::wp7::md::{self, Field, Head};
use crate::doccheck::wp7::py;

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The number of members the closed rejection enumeration must have
/// (`len(MEMBERS) != 13`, script line 386).
pub const REJECTION_PIN: usize = 13;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-16",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
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
            heading: Some("^Sync request and reply$"),
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
    keywords: &["fits", "falsified", "rejection", "counted", "ignored", "stale", "proposal", "reuse"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"^denial py_denial_(rejection_reasons_a_number|a_counted_width_as_a_string|a_sync_header_row_of_three_cells)$",
    class: DiffClass::ScriptCrashed,
    reason: "the script dies with a Python exception (TypeError iterating an int, TypeError adding a str to the size, \
             IndexError on a sync row of three cells); the port reports the same cause under CRASH",
}];

/// The constants the script reads (None where unreadable).
#[derive(Debug, Default)]
pub(crate) struct Konst {
    pub bound: i128,
    pub members_max: Option<i128>,
    pub streams_max: Option<i128>,
    pub dscp_control: Option<i128>,
    pub module_bits: Option<i128>,
    pub nonce_bits: Option<i128>,
    pub domain_bits: Option<i128>,
    pub epoch_bits: Option<i128>,
    pub scheme_bits: Option<i128>,
    pub counter_bits: Option<i128>,
    pub tag_bits: Option<i128>,
    pub peers_max: Option<i128>,
    pub hello_period_s: Option<i128>,
}

fn konst(text: &str, report: &mut Report) -> Konst {
    let c = |cid: &str, p: &str| const_int(text, cid, p);
    let mut k = Konst {
        bound: 0,
        members_max: c("C-08", r"at\s+most\s+(\d+)"),
        streams_max: c("C-09", r"at\s+most\s+(\d+)"),
        dscp_control: c("C-13", r"control\s+CS5\s*\((\d+)\)"),
        module_bits: c("C-11", r"(\d+)\s*/\s*\d+\s*/\s*\d+\s*bits"),
        nonce_bits: c("C-11", r"\d+\s*/\s*(\d+)\s*/\s*\d+\s*bits"),
        domain_bits: c("C-11", r"\d+\s*/\s*\d+\s*/\s*(\d+)\s*bits"),
        epoch_bits: c("C-10", r"\d+\s*/\s*\d+\s*/\s*(\d+)\s*bits"),
        scheme_bits: c("C-12", r"scheme\s+(\d+)\s+bits"),
        counter_bits: c("C-12", r"counter\s+(\d+)\s+bits"),
        tag_bits: c("C-12", r"tag\s+(\d+)\s+bits"),
        peers_max: c("C-34", r"at\s+most\s+(\d+)"),
        hello_period_s: c("C-30", r"(\d+)\s*s\b"),
    };
    let bound = c("C-17", &format!(r"at\s+most\s+({NUM})\s*bytes"));
    let named = [
        ("bound", bound),
        ("members_max", k.members_max),
        ("streams_max", k.streams_max),
        ("dscp_control", k.dscp_control),
        ("module_bits", k.module_bits),
        ("nonce_bits", k.nonce_bits),
        ("domain_bits", k.domain_bits),
        ("epoch_bits", k.epoch_bits),
        ("scheme_bits", k.scheme_bits),
        ("counter_bits", k.counter_bits),
        ("tag_bits", k.tag_bits),
        ("peers_max", k.peers_max),
        ("hello_period_s", k.hello_period_s),
    ];
    for (key, v) in named {
        if v.is_none() {
            report.fail("DoD", format!("constant for {key} could not be read from {CONSTANTS_MD}"));
        }
    }
    k.bound = bound.unwrap_or(0);
    k
}

/// The documents as the script reads them, and what its early parts
/// compute for the later ones.
pub(crate) struct Ctx {
    pub k: Konst,
    pub lines: Vec<String>,
    pub heads: Vec<Head>,
    pub members: Enumeration,
    /// The Hello section's lines, text, fence-blanked text, subsections and
    /// prose paragraphs.
    pub hello_lines: Vec<String>,
    pub plain: String,
    pub subs: Vec<(String, String)>,
    pub paras: Vec<String>,
    pub toml_text: Option<String>,
}

impl Ctx {
    /// `section_span(pattern, level)`.
    pub fn span(&self, pattern: &str, level: Option<usize>) -> Option<(usize, usize)> {
        md::section_span(self.lines.len(), &self.heads, pattern, level)
    }

    /// The lines of a span joined.
    pub fn text(&self, span: (usize, usize)) -> String {
        self.lines[span.0.min(span.1)..span.1].join("\n")
    }

    /// `para_with(*pats)`: the first prose paragraph matching every pattern
    /// (any case).
    pub fn para_with(&self, pats: &[&str]) -> Option<&String> {
        self.paras.iter().find(|t| pats.iter().all(|p| py::has_i(p, t)))
    }
}

/// What the layout part computes for the later parts.
#[derive(Debug, Default)]
pub(crate) struct Layout {
    pub hdr_sub: Option<String>,
    pub hdr: Vec<Field>,
    pub hdr_keys: Vec<Option<&'static str>>,
    pub lay: Vec<Field>,
    pub h_bytes: i128,
}

fn context(docs: &Docs, report: &mut Report) -> Ctx {
    let consts = docs.read(CONSTANTS_MD).map(|t| py::universal(&t)).unwrap_or_default();
    let k = konst(&consts, report);
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
        k,
        lines,
        heads,
        members,
        hello_lines: Vec::new(),
        plain: String::new(),
        subs: Vec::new(),
        paras: Vec::new(),
        toml_text: docs.read(WIRE_TOML).map(|t| py::universal(&t)),
    };
    match ctx.span(r"^hello$", Some(2)) {
        None => report.fail("AC1", "wire.md has no '## Hello' section"),
        Some((a, b)) => ctx.hello_lines = ctx.lines[a.min(b)..b].to_vec(),
    }
    let hello = ctx.hello_lines.join("\n");
    ctx.plain = md::strip_fences(&hello);
    ctx.subs = md::subsections(&ctx.hello_lines);
    ctx.paras = md::prose_blocks(&hello).into_iter().filter(|(k, _)| *k == 'p').map(|(_, t)| t).collect();
    ctx
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let ctx = context(docs, &mut report);
    let lay = ac5::layout(&ctx, &mut report);
    let rows = ac1::rows(&ctx, &lay, &mut report);
    ac2::sizes(&ctx, &lay, &rows, &mut report);
    ac3::alarms_and_role(&ctx, &lay.lay, &mut report);
    let reasons = ac4::forms(&ctx, &mut report);
    ac3::work3_and_behaviour(&ctx, &mut report);
    dod::tags(&ctx, &mut report);
    ac6::toml(&ctx, &lay, &rows, &reasons, &mut report);
    report
}
