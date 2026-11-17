//! Port of `checks/004-25.sh`: the regime stamp section types the
//! constants table names (ticket 004-25).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_25.rs`. The script's variants, named where used:
//! wire.md read with U+2212 turned into `-` and digit groups joined; the
//! constants table with digit groups joined; headings found by `re.search`
//! at an exact level (004-27's `span_of`), the Rejection reasons by the
//! anchored `^rejection\s+reasons$`; the tables of the 004-16 to 004-29
//! family ([`crate::doccheck::checks::c004_24::py::tables`]); the
//! arithmetic tree of the section types ([`tree`], shared with 004-28); the
//! Rejection reasons pinned by name and value ([`merged::REJ_FIXED`],
//! shared with 004-28); reason tokens `[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+`; the
//! non-rejection pattern with "no reason" ([`validity::NONREJ`]); the
//! untagged-claim rule per prose block of each type subsection
//! ([`TagPolicy::PROSE_004_29`]); open items as any prose or table block of
//! the Regime stamp section that mentions "open item"
//! ([`MENTION_BLOCKS_004_24`], the same rule as 004-24's).
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed; the checker names no ref containing main
//! and runs no other check) are in `scripts/repo-state-checks.sh`.
//!
//! [`TagPolicy::PROSE_004_29`]: crate::doccheck::evidence::TagPolicy::PROSE_004_29
//! [`MENTION_BLOCKS_004_24`]: crate::doccheck::checks::c004_24::dod::MENTION_BLOCKS_004_24

mod content;
pub(crate) mod merged;
pub(crate) mod review;
pub(crate) mod tree;
pub(crate) mod toml_side;
pub(crate) mod types;
pub(crate) mod validity;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::checks::c004_24::dod::{MENTION_BLOCKS_004_24, block_items};
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::{py_text, stopped};
use crate::doccheck::checks::c004_27::{Ctx, subsections};
use crate::doccheck::evidence::{TagPolicy, has_tag, untagged};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::text::{has_i, head, join_digit_groups, rx};
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::units::{Block, prose_blocks};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

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
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The numbers and lists the constants table gives.
pub(crate) struct Consts {
    pub ctrl_bound: Option<i64>,
    pub c159: Vec<String>,
    pub dscp: Option<i64>,
    pub cap_states: Vec<String>,
    pub ts_sources: Vec<String>,
    pub c09_max: Option<i64>,
    pub peers: Option<i64>,
}

/// The Regime stamp section.
pub(crate) struct Stamp {
    pub text: String,
    pub subs: Vec<(String, String)>,
    pub prose: Vec<Block>,
}

fn grab(c: &Constants, id: &str, pat: &str) -> Option<i64> {
    rx(pat).captures(c.value(id)).and_then(|m| m[1].parse().ok())
}

fn consts(c: &Constants, report: &mut Report) -> Consts {
    let ctrl_bound = grab(c, "C-17", r"at most (\d+) bytes");
    let c159: Vec<String> = c.value("C-159").split(',').map(str::trim).filter(|x| !x.is_empty()).map(String::from).collect();
    let cap_states: Vec<String> = rx(r"four: ([^.]*)\.").captures(c.value("C-129")).map(|m| m[1].split(',').map(|x| x.trim().to_lowercase()).collect()).unwrap_or_default();
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
    let (n, unowned) = block_items(&st.text, &MENTION_BLOCKS_004_24);
    report.check("DoD", unowned.is_empty(), format!("every Open item paragraph of the regime stamp section names an owner and a falsifier ({n} found); without: {unowned:?}"));
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let raw = match docs.read(WIRE_MD) {
        Some(t) => t.to_string(),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let wire = join_digit_groups(&py_text(&raw.replace('\u{2212}', "-")));
    let toml_text = docs.read(WIRE_TOML).map(|t| t.to_string());
    if toml_text.is_none() {
        report.fail("AC6", format!("{WIRE_TOML} does not exist"));
    }
    let constants = Constants::parse(&join_digit_groups(&py_text(&docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default())));
    let consts = consts(&constants, &mut report);
    let ctx = Ctx {
        md: Markdown::parse(&wire),
        consts: constants,
        toml_text,
    };
    let span = ctx.span_of(r"^regime\s+stamp$", 2);
    let text = ctx.body(span);
    report.check("AC1", span.is_some() && !text.to_lowercase().contains("to be filled"), "wire.md has a Regime stamp section with its placeholder replaced");
    let st = Stamp {
        subs: subsections(&text),
        prose: prose_blocks(&text),
        text,
    };
    if let Err(Stop(e)) = checks(&ctx, &st, &wire, &consts, &mut report) {
        stopped(&mut report, "AC1", &e);
    }
    report
}

fn checks(ctx: &Ctx, st: &Stamp, wire: &str, consts: &Consts, report: &mut Report) -> Result<(), Stop> {
    let block = merged::code_block(wire, "004-25");
    let m = merged::merged(ctx, wire, &st.text, report, |r| {
        r.check("AC1", block.is_some(), format!("the merged section type codes give 004-25 a block (read {block:?})"));
    })?;
    let (types, s) = types::section_types(st, consts.ctrl_bound, block, m.section_max, report)?;
    content::context(&types, report);
    content::ac2(&types, consts, report);
    content::ac3(&types, consts, report)?;
    validity::ac5(&st.text, &m, report);
    dod(st, &types, &m, report);
    toml_side::ac6(ctx.toml_text.as_deref(), &types, &m, consts.dscp, report)?;
    review::c1(st, &types, s, m.section_max, consts, report);
    review::c2(st, &types, report);
    review::c3(ctx, st, report);
    review::c4(&types, report)
}
