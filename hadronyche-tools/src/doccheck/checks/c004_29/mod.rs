//! Port of `checks/004-29.sh`: the commissioning window announcement and
//! close messages (ticket 004-29).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_29.rs`. The script's variants, named where used:
//! wire.md read with U+2212 turned into `-` and digit groups joined;
//! `span_of` at an exact level; the tables of the 004-16 to 004-29 family;
//! the Rejection reasons anchored, required to have 13 members and the
//! seven control reasons it names (not pinned by value); reason tokens
//! `[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+`; the non-rejection pattern without
//! `\bvalid\b` ([`forms`]); the six control violations and their reasons
//! (`GENERIC`); tiling without the list count (`pos = off + bits // 8`);
//! the untagged-claim rule per prose block of the section
//! ([`TagPolicy::PROSE_004_29`]); open items as prose blocks that start with
//! `Open item` after optional `**` ([`OpenItemPolicy::Leading`]); the clock
//! terms of `doc/design/spec/timeline.md`, read at run time ([`clock`]).
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed, the count at the merged text; the checker
//! names no ref containing main and runs no other check) are in
//! `scripts/repo-state.d/004-29.sh`.
//!
//! [`TagPolicy::PROSE_004_29`]: crate::doccheck::evidence::TagPolicy::PROSE_004_29
//! [`OpenItemPolicy::Leading`]: crate::doccheck::items::OpenItemPolicy::Leading

mod clock;
mod dod;
mod forms;
mod msgs;
mod rules;
mod toml_side;

use std::collections::BTreeMap;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::landed::rejection_members;
use crate::doccheck::checks::c004_24::py::{first_table, py_text, stopped, tables};
use crate::doccheck::checks::c004_25::merged::reserved_rows;
use crate::doccheck::checks::c004_27::{Ctx, Field, fields_of, find_sub, is_field_table, subsections};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::tables::Row;
use crate::doccheck::text::{has_i, join_digit_groups, rx, starts};
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The timeline, whose clock terms the check reads.
pub const TIMELINE_MD: &str = "doc/design/spec/timeline.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-29",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Commissioning window messages$") },
        Target { file: WIRE_MD, heading: Some("^Common control header$") },
        Target { file: WIRE_MD, heading: Some("^Control message type codes$") },
        Target { file: WIRE_MD, heading: Some("^Commissioning window section$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_MD, heading: Some("^Reserved fields$") },
        Target { file: WIRE_MD, heading: Some("^Open items$") },
        Target { file: WIRE_TOML, heading: None },
        Target { file: CONSTANTS_MD, heading: None },
        Target { file: TIMELINE_MD, heading: None },
    ],
    keywords: &["absolute", "relative", "explicit", "closes", "opens"],
    known_differences: KNOWN,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The merged text the section reads.
pub(crate) struct Merged29 {
    pub ctrl_bound: Option<i64>,
    pub dscp: Option<i64>,
    pub cch_rows: Vec<(i64, i64)>,
    pub block: Option<(i64, i64)>,
    pub cw: Vec<Field>,
    pub w_stamp: Option<i64>,
    pub e_stamp: Option<i64>,
    pub rej_value: BTreeMap<String, i64>,
    pub rej_layers: BTreeMap<String, String>,
    pub res_rows: Vec<Row>,
}

impl Merged29 {
    /// `CONTROL_MEMBERS`.
    pub fn is_control(&self, name: &str) -> bool {
        self.rej_layers.get(name).is_some_and(|l| l.contains("control"))
    }
}

/// The section and its three subsections.
pub(crate) struct Section {
    pub text: String,
    pub ann: String,
    pub clo: String,
    pub rul: String,
}

/// The header rows as (offset, bits) and the type-code block.
type HelloRead = (Vec<(i64, i64)>, Option<(i64, i64)>);

fn hello(ctx: &Ctx, report: &mut Report) -> HelloRead {
    let hs = subsections(&ctx.body(ctx.span_of(r"^hello$", 2)));
    let hit = find_sub(&hs, "common control header");
    let t = if hit.len() == 1 { first_table(&hit[0].1, is_field_table) } else { None };
    let cch = if t.is_some() { fields_of(t.as_ref(), "AC1", "the Hello common control header", report) } else { Vec::new() };
    let rows: Vec<(i64, i64)> = cch.iter().map(|f| (f.off, f.bits)).collect();
    report.check("DoD", rows == [(0, 16), (2, 16), (4, 16), (6, 8), (7, 8), (8, 64), (16, 128)], format!("the merged common control header of the Hello section is unchanged (read {rows:?})"));
    let wire = ctx.md.lines.join("\n");
    let codes = tables(&wire).into_iter().rfind(|t| t.head().first().is_some_and(|h| h.to_lowercase().starts_with("codes")) && t.rows.iter().any(|r| r.cells.len() > 1 && has_i("hello", r.cell(1))));
    let first: Vec<String> = codes.as_ref().map(|t| t.rows.iter().map(|r| r.cell(0).trim().to_string()).collect()).unwrap_or_default();
    report.check("DoD", first == ["0", "1", "2 to 15", "16 to 31", "32 to 35", "36 to 47", "48 to 51", "52 to 255"], format!("the merged control message type codes keep their code column (read {first:?})"));
    let mut block = None;
    for r in codes.iter().flat_map(|t| t.rows.iter()) {
        if r.cells.len() >= 3 && r.cell(2).contains("004-29")
            && let Some(c) = rx(r"(\d+)\s+to\s+(\d+)").captures(r.cell(0))
                && let (Ok(a), Ok(b)) = (c[1].parse(), c[2].parse()) {
                    block = Some((a, b));
                }
    }
    report.check("AC1", block.is_some(), format!("the control message type codes give this ticket a block (read {block:?})"));
    (rows, block)
}

fn merged(ctx: &Ctx, c: &Constants, report: &mut Report) -> Merged29 {
    let int = |id: &str, pat: &str| rx(pat).captures(c.value(id)).and_then(|m| m[1].parse::<i64>().ok());
    let ctrl_bound = int("C-17", r"at most (\d+) bytes");
    let dscp = int("C-13", r"control CS5 \((\d+)\)");
    report.check("AC1", ctrl_bound == Some(1252), format!("the constants table gives the control bound (C-17) as 1 252 bytes (read {ctrl_bound:?})"));
    report.check("AC2", dscp == Some(40), format!("the constants table gives control CS5 as 40 (C-13) (read {dscp:?})"));
    let (cch_rows, block) = hello(ctx, report);
    let ss = subsections(&ctx.body(ctx.span_of(r"^regime\s+stamp$", 2)));
    let hit = find_sub(&ss, "commissioning window section");
    let t = if hit.len() == 1 { first_table(&hit[0].1, is_field_table) } else { None };
    let cw = if t.is_some() { fields_of(t.as_ref(), "AC2", "the merged commissioning window section", report) } else { Vec::new() };
    let win: Vec<i64> = cw.iter().filter(|f| starts("(?i)window identifier", &f.meaning)).map(|f| f.bits).collect();
    let exp: Vec<i64> = cw.iter().filter(|f| starts("(?i)expiry time", &f.meaning)).map(|f| f.bits).collect();
    report.check("AC2", win.len() == 1 && exp.len() == 1, format!("the merged commissioning window section has one window identifier row and one expiry row (read {win:?}, {exp:?})"));
    let (rej_value, rej_layers) = rejection_members(ctx, r"^rejection\s+reasons$");
    let m = Merged29 { ctrl_bound, dscp, cch_rows, block, w_stamp: win.first().copied(), e_stamp: exp.first().copied(), cw, rej_value, rej_layers, res_rows: reserved_rows(&ctx.md.lines.join("\n")) };
    let needed = ["TooShort", "BadMagic", "VersionMismatch", "ForeignDomain", "OversizeControl", "ReservedNonZero", "BadLength"];
    report.check("DoD", m.rej_value.len() == 13 && needed.iter().all(|n| m.is_control(n)), format!("the merged Rejection reasons enumeration keeps its 13 members, with the control reasons (read {})", m.rej_value.len()));
    m
}

fn section(ctx: &Ctx, report: &mut Report) -> Section {
    let span = ctx.span_of(r"^commissioning\s+window\s+messages$", 2);
    let text = ctx.body(span);
    report.check("DoD", span.is_some() && !text.to_lowercase().contains("to be filled"), "wire.md has a Commissioning window messages section with its placeholder replaced");
    let subs = subsections(&text);
    let pick = |pat: &str| -> Vec<String> { find_sub(&subs, pat).into_iter().map(|(_, x)| x.clone()).collect() };
    let (ann, clo, rul) = (pick("announc"), pick(r"\bclose\b"), pick("receiver rule"));
    report.check("AC1", ann.len() == 1, format!("exactly one subsection names the announcement (found {})", ann.len()));
    report.check("AC1", clo.len() == 1, format!("exactly one subsection names the close (found {})", clo.len()));
    report.check("AC3", rul.len() == 1, format!("exactly one subsection holds the receiver rules (found {})", rul.len()));
    let one = |v: Vec<String>| if v.len() == 1 { v.into_iter().next().unwrap_or_default() } else { String::new() };
    Section { text, ann: one(ann), clo: one(clo), rul: one(rul) }
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
        report.fail("AC5", format!("{WIRE_TOML} does not exist"));
    }
    let constants = Constants::parse(&join_digit_groups(&py_text(&docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default())));
    let timeline = py_text(&docs.read(TIMELINE_MD).map(|t| t.to_string()).unwrap_or_default());
    let ctx = Ctx { md: Markdown::parse(&wire), consts: constants.clone(), toml_text };
    let m = merged(&ctx, &constants, &mut report);
    let s = section(&ctx, &mut report);
    if let Err(Stop(e)) = checks(&ctx, &s, &m, &timeline, &mut report) {
        stopped(&mut report, "AC1", &e);
    }
    report
}

fn checks(ctx: &Ctx, s: &Section, m: &Merged29, timeline: &str, report: &mut Report) -> Result<(), Stop> {
    let ann = if s.ann.is_empty() { None } else { msgs::msg_analysis(&s.ann, "announcement", "AC1", m, report)? };
    let clo = if s.clo.is_empty() { None } else { msgs::msg_analysis(&s.clo, "close", "AC1", m, report)? };
    if let (Some(a), Some(c)) = (&ann, &clo) {
        report.check("AC1", a.code != c.code, format!("the announcement and the close have distinct type codes (found {:?} and {:?})", a.code, c.code));
    }
    let rel = ann.as_ref().is_some_and(|a| msgs::announcement(s, a, m, timeline, report));
    if let Some(c) = &clo {
        msgs::close(c, m, report);
    }
    let rows = forms::rows(s);
    forms::ac4(&rows, m, report);
    rules::second(s, &rows, report);
    rules::ac3(s, report);
    let new_fields: Vec<&Field> = ann.iter().chain(clo.iter()).flat_map(|x| x.fl.iter()).collect();
    let oi11_status = dod::dod(ctx, s, m, &new_fields, report);
    forms::f3(s, report);
    rules::f2_f7(s, report);
    dod::f1(s, report);
    if rel {
        report.check("DoD", oi11_status, "Open item 11 of the regime stamp carries a status sentence for the relative form (branch b)");
    }
    toml_side::ac5(ctx.toml_text.as_deref(), s, m, ann.as_ref(), clo.as_ref(), report).map_err(Stop)
}
