//! Port of `checks/004-16.sh`: the hello message layout and the
//! 32-member size bound (ticket 004-16).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_16.rs`. The port is a transcription of the script with
//! Python semantics, on the library's named variants: the 004 family's
//! Markdown helpers ([`family004`]: `section_span` under
//! `SpanRule::AfterHeading004_16`, field tables, `###` subsections); the
//! closed enumeration ([`Enumeration`] under `REJECTION_004_16`) pinned at
//! [`REJECTION_PIN`]; the reason tokens `CAMEL_LETTERS` with the 24-character
//! negation look-back `LOOKBACK_004_16`; `NON_REJECTION_004_16`; the DoD
//! under `TagPolicy::ALPHA_004_16`, `TableTagRule::SectionTagged004_16` and
//! `OpenItemPolicy::LeadingClean`; wire.toml read with the scripts' subset
//! reader ([`crate::doccheck::tomlsubset`]). A Python exception stops the
//! check ([`crate::doccheck::report::Stop::Crash`]); an integer beyond
//! `i128` is red by rule (`INT_LIMIT`).
//!
//! Private to this port (no library item has the rule): the invalid-form
//! table of AC4 (the first table of an `invalid|validity` subsection with a
//! form and an outcome header), `cat_of` and the exact-reason map, the
//! verdict classifier and its quote stripping (AC2), the alarm encodings
//! (AC3) and the TOML size sum (AC6, Python `int`/`float` arithmetic).
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` is
//! exactly 12 passed, 0 failed, 0 ignored) are in
//! `scripts/repo-state.d/004-16.sh`.

pub(crate) mod ac1;
pub(crate) mod ac2;
mod ac3;
pub(crate) mod ac4;
pub(crate) mod ac5;
pub(crate) mod ac6;
pub(crate) mod dod;

pub mod denials;
use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::equiv::script::DEFAULT_READER;
use crate::doccheck::family004::{self, ConstInt, Field, Head, IntRule, NUM_004_16, SpanRule};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{IntError, splitlines, universal_newlines};
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::rejection_members::{Enumeration, REJECTION_004_16};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::has_i;
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::units::{BlockKind, prose_blocks, strip_fences};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// `NUM` (004-16's digit groups).
pub(crate) const NUM: &str = NUM_004_16;

/// `to_int(s)`: digit groups joined; a Python `ValueError` stops the check
/// as the uncaught exception it is.
pub(crate) fn to_int(s: &str) -> Flow<Option<i128>> {
    match family004::to_int(s, IntRule::Spaces004_16) {
        Ok(v) => Ok(Some(v)),
        Err(IntError::Overflow(_)) => Ok(None),
        Err(e) => crash(format!("ValueError: {e}")),
    }
}

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
    fail_reader: DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

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

/// `const_int(cid, pattern)`; a Python `ValueError` is a crash.
fn const_int(text: &str, cid: &str, pattern: &str) -> Flow<Option<i128>> {
    match family004::const_int(text, cid, pattern, ConstInt::AnyCase004_16) {
        Ok(v) => Ok(v),
        Err(IntError::Overflow(_)) => Ok(None),
        Err(e) => crash(format!("ValueError: {e}")),
    }
}

fn konst(text: &str, report: &mut Report) -> Flow<Konst> {
    let c = |cid: &str, p: &str| const_int(text, cid, p);
    let mut k = Konst {
        bound: 0,
        members_max: c("C-08", r"at\s+most\s+(\d+)")?,
        streams_max: c("C-09", r"at\s+most\s+(\d+)")?,
        dscp_control: c("C-13", r"control\s+CS5\s*\((\d+)\)")?,
        module_bits: c("C-11", r"(\d+)\s*/\s*\d+\s*/\s*\d+\s*bits")?,
        nonce_bits: c("C-11", r"\d+\s*/\s*(\d+)\s*/\s*\d+\s*bits")?,
        domain_bits: c("C-11", r"\d+\s*/\s*\d+\s*/\s*(\d+)\s*bits")?,
        epoch_bits: c("C-10", r"\d+\s*/\s*\d+\s*/\s*(\d+)\s*bits")?,
        scheme_bits: c("C-12", r"scheme\s+(\d+)\s+bits")?,
        counter_bits: c("C-12", r"counter\s+(\d+)\s+bits")?,
        tag_bits: c("C-12", r"tag\s+(\d+)\s+bits")?,
        peers_max: c("C-34", r"at\s+most\s+(\d+)")?,
        hello_period_s: c("C-30", r"(\d+)\s*s\b")?,
    };
    let bound = c("C-17", &format!(r"at\s+most\s+({NUM})\s*bytes"))?;
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
    k.bound = bound.unwrap_or_default();
    Ok(k)
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
        family004::section_span(self.lines.len(), &self.heads, pattern, level, SpanRule::AfterHeading004_16)
    }

    /// The lines of a span joined.
    pub fn text(&self, span: (usize, usize)) -> String {
        self.lines[span.0.min(span.1)..span.1].join("\n")
    }

    /// `para_with(*pats)`: the first prose paragraph matching every pattern
    /// (any case).
    pub fn para_with(&self, pats: &[&str]) -> Option<&String> {
        self.paras.iter().find(|t| pats.iter().all(|p| has_i(p, t)))
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

fn context(docs: &Docs, report: &mut Report) -> Flow<Ctx> {
    let consts = docs.read(CONSTANTS_MD).map(|t| universal_newlines(&t)).unwrap_or_default();
    let k = konst(&consts, report)?;
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
        k,
        lines,
        heads,
        members,
        hello_lines: Vec::new(),
        plain: String::new(),
        subs: Vec::new(),
        paras: Vec::new(),
        toml_text: docs.read(WIRE_TOML).map(|t| universal_newlines(&t)),
    };
    match ctx.span(r"^hello$", Some(2)) {
        None => report.fail("AC1", "wire.md has no '## Hello' section"),
        Some((a, b)) => ctx.hello_lines = ctx.lines[a.min(b)..b].to_vec(),
    }
    let hello = ctx.hello_lines.join("\n");
    ctx.plain = strip_fences(&hello);
    ctx.subs = family004::subsections(&hello);
    ctx.paras = prose_blocks(&hello).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text).collect();
    Ok(ctx)
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let flow = body(docs, &mut report);
    report.finish(flow)
}

/// The script, in its order; a Python exception stops it.
fn body(docs: &Docs, report: &mut Report) -> Flow<()> {
    let ctx = context(docs, report)?;
    let lay = ac5::layout(&ctx, report)?;
    let rows = ac1::rows(&ctx, &lay, report);
    ac2::sizes(&ctx, &lay, &rows, report)?;
    ac3::alarms_and_role(&ctx, &lay.lay, report);
    let reasons = ac4::forms(&ctx, report)?;
    ac3::work3_and_behaviour(&ctx, report);
    dod::tags(&ctx, report);
    ac6::toml(&ctx, &lay, &rows, &reasons, report)
}
