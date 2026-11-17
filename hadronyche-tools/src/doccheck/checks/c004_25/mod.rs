//! Port of `checks/004-25.sh`: the regime stamp section types the
//! constants table names (ticket 004-25).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_25.rs`; the denial list is `denials.rs`. The script's
//! variants, named where used:
//!
//! * wire.md read as `open()` does, U+2212 turned into `-`, digit groups
//!   joined, then `splitlines()` ([`LineSplit::PySplitlines`]);
//! * the 004 family's helpers ([`crate::doccheck::family004`]);
//! * the Rejection reasons by [`REJECTION_004_25`] (anchored), pinned by
//!   name and value ([`REJ_FIXED`]); reason tokens [`CAMEL_DIGITS`]; the
//!   non-rejection pattern [`NON_REJECTION_004_25`];
//! * the arithmetic tree of the section types ([`tree`], private: the
//!   library has none; 004-28 has the same functions) and Python's `eval`
//!   of a stated sum ([`crate::doccheck::pynum::eval_arith`]: a
//!   `SyntaxError` there is a crash, the script has no `try`);
//! * sentences by [`SentenceSplit::BLOCKS_004_25`];
//! * the untagged-claim rule per prose block of each type subsection
//!   ([`TagPolicy::PROSE_004_29`]); open items
//!   ([`OpenItemPolicy::MentionBlocks`]);
//! * wire.toml through the script's subset reader
//!   ([`TomlReader::Subset004_18`]), every exception of the reader caught
//!   as an AC6 failure; the uncaught exceptions of the values' use, of a
//!   missing C-17 bound, of a four-cell Section count row, of a health type
//!   without entries, of `int()` of a superscript digit, are crashes.
//!
//! Its repository-level assertions are in `scripts/repo-state.d/004-25.sh`.
//!
//! [`LineSplit::PySplitlines`]: crate::doccheck::policy::LineSplit::PySplitlines
//! [`REJECTION_004_25`]: crate::doccheck::rejection_members::REJECTION_004_25
//! [`REJ_FIXED`]: crate::doccheck::rejection_members::REJ_FIXED
//! [`CAMEL_DIGITS`]: crate::doccheck::rejection_members::reasons::CAMEL_DIGITS
//! [`NON_REJECTION_004_25`]: crate::doccheck::rejection_members::reasons::NON_REJECTION_004_25
//! [`SentenceSplit::BLOCKS_004_25`]: crate::doccheck::sentences::SentenceSplit::BLOCKS_004_25
//! [`TagPolicy::PROSE_004_29`]: crate::doccheck::evidence::TagPolicy::PROSE_004_29
//! [`OpenItemPolicy::MentionBlocks`]: crate::doccheck::items::OpenItemPolicy::MentionBlocks
//! [`TomlReader::Subset004_18`]: crate::doccheck::policy::TomlReader::Subset004_18

mod content;
mod denials;
mod merged;
mod review;
mod toml_side;
mod tree;
mod types;
mod validity;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::evidence::{TagPolicy, has_tag, untagged};
use crate::doccheck::family004::{Ctx, subsections};
use crate::doccheck::items::{OpenItemPolicy, open_items, unowned_items};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::{LineSplit, TomlReader};
use crate::doccheck::pycompat::{PyStr, parse_int, universal_newlines};
use crate::doccheck::pyvalue::Dict;
use crate::doccheck::report::Flow;
use crate::doccheck::text::{has_i, head, join_digit_groups, rx};
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::tomlsubset;
use crate::doccheck::units::{Block, prose_blocks};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// How the script reads its documents.
const LINES: LineSplit = LineSplit::PySplitlines;

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-25",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Regime stamp$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Rejection reasons$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Reserved fields$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Manifest$"),
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
    keywords: &["exceed", "reset", "wrap", "control", "reserved"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The numbers and lists the constants table gives.
pub(crate) struct Consts {
    pub ctrl_bound: Option<i128>,
    pub c159: Vec<String>,
    pub dscp: Option<i128>,
    pub cap_states: Vec<String>,
    pub ts_sources: Vec<String>,
    pub c09_max: Option<i128>,
    pub peers: Option<i128>,
}

/// The Regime stamp section.
pub(crate) struct Stamp {
    pub text: String,
    pub subs: Vec<(String, String)>,
    pub prose: Vec<Block>,
}

/// `int(re.search(pat, const_row(id)).group(1))` when it matches.
fn grab(c: &Constants, id: &str, pat: &str) -> Option<i128> {
    rx(pat).captures(c.value(id)).and_then(|m| parse_int(&m[1]).ok())
}

fn consts(c: &Constants, report: &mut Report) -> Consts {
    let ctrl_bound = grab(c, "C-17", r"at most (\d+) bytes");
    let c159: Vec<String> = c.value("C-159").split(',').map(PyStr::py_strip).filter(|x| !x.is_empty()).map(String::from).collect();
    let cap_states: Vec<String> = rx(r"four: ([^.]*)\.").captures(c.value("C-129")).map(|m| m[1].split(',').map(|x| x.py_strip().to_lowercase()).collect()).unwrap_or_default();
    let ts_sources: Vec<String> = rx(r"([a-z][a-z-]*) \(").captures_iter(c.value("C-131")).map(|m| m[1].to_lowercase()).collect();
    report.check("AC1", ctrl_bound == Some(1252), format!("the constants table gives the control bound C-17 as 1 252 bytes (read {ctrl_bound:?})"));
    report.check("AC3", c159.len() == 11, format!("the constants table gives eleven C-159 counters, read at run time (found {}: {c159:?})", c159.len()));
    report.check("AC2", cap_states.len() == 4, format!("the constants table gives four C-129 capability states, read at run time (found {cap_states:?})"));
    report.check("AC2", ts_sources.len() == 4, format!("the constants table gives four C-131 receive-timestamp sources, read at run time (found {ts_sources:?})"));
    Consts {
        ctrl_bound,
        c159,
        dscp: grab(c, "C-13", r"control CS5 \((\d+)\)"),
        cap_states,
        ts_sources,
        c09_max: grab(c, "C-09", r"at most (\d+)"),
        peers: grab(c, "C-217", r"(\d+) peers"),
    }
}

fn dod(st: &Stamp, types: &[types::SType], m: &merged::Merged, report: &mut Report) {
    let need = types.iter().flat_map(|t| t.fl_all.iter()).any(|f| has_i(r"\breserved\b|\bunused\b|accept[- ]any", &format!("{} {}", f.meaning, f.valid)));
    let res = m.res_rows.iter().filter(|r| r.cells.len() > 1 && r.cell(1).to_lowercase().contains("regime stamp")).count();
    report.check("DoD", !need || res >= 2, "every reserved or accept-any field of the section types is recorded in the Reserved fields table");
    for t in types {
        report.check("DoD", has_tag(&t.text), format!("the {} subsection carries an evidence tag", t.key));
        let bad: Vec<String> = untagged(&t.text, &TagPolicy::PROSE_004_29).iter().map(|u| head(&u.text, 50)).collect();
        report.check("DoD", bad.is_empty(), format!("every prose paragraph of 5 or more words in the {} subsection carries an evidence tag; untagged: {bad:?}", t.key));
    }
    let n = open_items(&st.text, OpenItemPolicy::MentionBlocks).len();
    let unowned: Vec<String> = unowned_items(&st.text, OpenItemPolicy::MentionBlocks).iter().map(|i| head(&i.text, 50)).collect();
    report.check("DoD", unowned.is_empty(), format!("every Open item paragraph of the regime stamp section names an owner and a falsifier ({n} found); without: {unowned:?}"));
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    // norm(WIRE_RAW.replace("−", "-")), then splitlines()
    let wire = match docs.read(WIRE_MD) {
        Some(t) => LINES.normalize(&join_digit_groups(&universal_newlines(&t).replace('\u{2212}', "-"))),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let toml_text = docs.read(WIRE_TOML).map(|t| universal_newlines(&t));
    if toml_text.is_none() {
        report.fail("AC6", format!("{WIRE_TOML} does not exist"));
    }
    let constants = docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default();
    let constants = Constants::parse(&LINES.normalize(&join_digit_groups(&universal_newlines(&constants))));
    let consts = consts(&constants, &mut report);
    let ctx = Ctx { md: Markdown::parse(&wire), consts: constants };
    let span = ctx.span_of(r"^regime\s+stamp$", 2);
    let text = ctx.body(span);
    report.check("AC1", span.is_some() && !text.to_lowercase().contains("to be filled"), "wire.md has a Regime stamp section with its placeholder replaced");
    let st = Stamp { subs: subsections(&text), prose: prose_blocks(&text), text };
    let flow = checks(&ctx, &st, &wire, toml_text.as_deref(), &consts, &mut report);
    report.finish(flow)
}

fn checks(ctx: &Ctx, st: &Stamp, wire: &str, toml_text: Option<&str>, consts: &Consts, report: &mut Report) -> Flow<()> {
    let block = merged::code_block(wire, "004-25");
    let m = merged::merged(wire, &st.text, report, |r| {
        r.check("AC1", block.is_some(), format!("the merged section type codes give 004-25 a block (read {block:?})"));
    })?;
    let (types, s) = types::section_types(st, consts.ctrl_bound, block, m.section_max, report)?;
    content::context(&types, report);
    content::ac2(&types, consts, report);
    content::ac3(&types, consts, report)?;
    validity::ac5(&st.text, &m, report);
    dod(st, &types, &m, report);
    // TOML = parse_toml(TOML_TEXT), any exception caught as an AC6 failure
    let toml: Option<Dict> = match tomlsubset::parse(toml_text.unwrap_or("")) {
        Ok(t) => Some(t),
        Err(e) => {
            report.fail("AC6", format!("wire.toml does not parse: {e}"));
            None
        }
    };
    toml_side::ac6(toml.as_ref(), &types, &m, consts.dscp, report)?;
    review::c1(st, &types, s, m.section_max, consts, report)?;
    review::c2(st, &types, report)?;
    review::c3(ctx, st, report);
    review::c4(&types, report)
}
