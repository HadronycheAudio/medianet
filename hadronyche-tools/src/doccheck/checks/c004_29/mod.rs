//! Port of `checks/004-29.sh`: the commissioning window announcement and
//! close messages (ticket 004-29).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_29.rs`. Built on the library: wire.md read with U+2212
//! turned into `-` and digit groups joined; `span_of` at an exact level
//! ([`Ctx::span_of`]); the tables of the 004 family ([`tables`],
//! [`first_field_table`]); the constants by `const_row` ([`const_int`]
//! under [`ConstInt::Exact004_28`]); the Rejection reasons
//! ([`REJECTION_004_25`]), required to have 13 members and the seven
//! control reasons it names, not pinned by value
//! ([`Enumeration::has_control_reasons`] with [`CONTROL_REASONS_004_29`]);
//! reason tokens ([`CAMEL_DIGITS`](crate::doccheck::rejection_members::reasons::CAMEL_DIGITS));
//! the non-rejection pattern without `\bvalid\b`
//! ([`NON_REJECTION_004_29`](crate::doccheck::rejection_members::reasons::NON_REJECTION_004_29));
//! `sentences_of` ([`SentenceSplit::PARAGRAPHS_004_29`](crate::doccheck::sentences::SentenceSplit::PARAGRAPHS_004_29));
//! the untagged-claim rule per prose block of the section
//! ([`TagPolicy::PROSE_004_29`]); open items as prose blocks that start with
//! `Open item` after optional `**` ([`OpenItemPolicy::Leading`]); every
//! `int()` through [`parse_int`] (a value beyond `i128` is `INT_LIMIT`);
//! every exception the script does not catch a
//! [`crate::doccheck::report::Stop::Crash`].
//!
//! Private to this port (the library has no equivalent): the six control
//! violations and their reasons (`GENERIC`, [`forms`]); tiling without the
//! list count (`pos = off + bits // 8`, [`msgs`]); the clock terms of
//! `doc/design/spec/timeline.md`, read at run time ([`clock`]); the
//! Reserved fields table (the last table headed "Field…", "…location…");
//! the strict TOML item helpers ([`toml_side`]).
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed, the count at the merged text) are in
//! `scripts/repo-state.d/004-29.sh`; "the checker names no ref containing
//! main and runs no other check" is the library-wide rule.
//!
//! [`TagPolicy::PROSE_004_29`]: crate::doccheck::evidence::TagPolicy::PROSE_004_29
//! [`OpenItemPolicy::Leading`]: crate::doccheck::items::OpenItemPolicy::Leading
//! [`Enumeration::has_control_reasons`]: crate::doccheck::rejection_members::Enumeration::has_control_reasons

mod clock;
mod denials;
mod dod;
mod forms;
mod msgs;
mod rules;
mod toml_side;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::family004::{ConstInt, Ctx, Field, const_int, fields_of, find_sub, first_field_table, subsections, tables};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::{LineSplit, TomlReader};
use crate::doccheck::pycompat::{PyStr, parse_int, universal_newlines};
use crate::doccheck::rejection_members::{CONTROL_REASONS_004_29, Enumeration, REJECTION_004_25};
use crate::doccheck::report::Flow;
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
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
};

/// How the script splits its texts (`splitlines()`).
const LINES: LineSplit = LineSplit::PySplitlines;

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The merged text the section reads.
pub(crate) struct Merged29 {
    pub ctrl_bound: Option<i128>,
    pub dscp: Option<i128>,
    pub cch_rows: Vec<(i128, i128)>,
    pub block: Option<(i128, i128)>,
    pub cw: Vec<Field>,
    pub w_stamp: Option<i128>,
    pub e_stamp: Option<i128>,
    pub rej: Enumeration,
    pub res_rows: Vec<Row>,
}

impl Merged29 {
    /// `name in REJ_VALUE`.
    pub fn is_member(&self, name: &str) -> bool {
        self.rej.contains(name)
    }

    /// `name in CONTROL_MEMBERS`.
    pub fn is_control(&self, name: &str) -> bool {
        self.rej.control_members().contains(&name)
    }
}

/// `RES_ROWS`: the rows of the last table of `wire` headed "Field…",
/// "…location…" (the Reserved fields table).
fn reserved_rows(wire: &str) -> Vec<Row> {
    tables(wire)
        .into_iter()
        .rfind(|t| {
            let h = t.head();
            h.first().is_some_and(|x| x.to_lowercase().starts_with("field")) && h.len() > 1 && h[1].to_lowercase().contains("location")
        })
        .map(|t| t.rows)
        .unwrap_or_default()
}

/// The section and its three subsections.
pub(crate) struct Section {
    pub text: String,
    pub ann: String,
    pub clo: String,
    pub rul: String,
}

/// The header rows as (offset, bits) and the type-code block.
type HelloRead = (Vec<(i128, i128)>, Option<(i128, i128)>);

fn hello(ctx: &Ctx, report: &mut Report) -> HelloRead {
    let hs = subsections(&ctx.body(ctx.span_of(r"^hello$", 2)));
    let hit = find_sub(&hs, "common control header");
    let t = if hit.len() == 1 { first_field_table(&hit[0].1) } else { None };
    let cch = if t.is_some() { fields_of(t.as_ref(), "AC1", "the Hello common control header", report) } else { Vec::new() };
    let rows: Vec<(i128, i128)> = cch.iter().map(|f| (f.off, f.bits)).collect();
    report.check("DoD", rows == [(0, 16), (2, 16), (4, 16), (6, 8), (7, 8), (8, 64), (16, 128)], format!("the merged common control header of the Hello section is unchanged (read {rows:?})"));
    let wire = ctx.md.lines.join("\n");
    let codes = tables(&wire).into_iter().rfind(|t| t.head().first().is_some_and(|h| h.to_lowercase().starts_with("codes")) && t.rows.iter().any(|r| r.cells.len() > 1 && has_i("hello", r.cell(1))));
    let first: Vec<String> = codes.as_ref().map(|t| t.rows.iter().map(|r| r.cell(0).py_strip().to_string()).collect()).unwrap_or_default();
    report.check("DoD", first == ["0", "1", "2 to 15", "16 to 31", "32 to 35", "36 to 47", "48 to 51", "52 to 255"], format!("the merged control message type codes keep their code column (read {first:?})"));
    let mut block = None;
    for r in codes.iter().flat_map(|t| t.rows.iter()) {
        if r.cells.len() >= 3 && r.cell(2).contains("004-29")
            && let Some(c) = rx(r"(\d+)\s+to\s+(\d+)").captures(r.cell(0))
                && let (Ok(a), Ok(b)) = (parse_int(&c[1]), parse_int(&c[2])) {
                    block = Some((a, b));
                }
    }
    report.check("AC1", block.is_some(), format!("the control message type codes give this ticket a block (read {block:?})"));
    (rows, block)
}

fn merged(ctx: &Ctx, const_text: &str, report: &mut Report) -> Merged29 {
    let int = |id: &str, pat: &str| const_int(const_text, id, pat, ConstInt::Exact004_28).ok().flatten();
    let ctrl_bound = int("C-17", r"at most (\d+) bytes");
    let dscp = int("C-13", r"control CS5 \((\d+)\)");
    report.check("AC1", ctrl_bound == Some(1252), format!("the constants table gives the control bound (C-17) as 1 252 bytes (read {ctrl_bound:?})"));
    report.check("AC2", dscp == Some(40), format!("the constants table gives control CS5 as 40 (C-13) (read {dscp:?})"));
    let (cch_rows, block) = hello(ctx, report);
    let ss = subsections(&ctx.body(ctx.span_of(r"^regime\s+stamp$", 2)));
    let hit = find_sub(&ss, "commissioning window section");
    let t = if hit.len() == 1 { first_field_table(&hit[0].1) } else { None };
    let cw = if t.is_some() { fields_of(t.as_ref(), "AC2", "the merged commissioning window section", report) } else { Vec::new() };
    let win: Vec<i128> = cw.iter().filter(|f| starts("(?i)window identifier", &f.meaning)).map(|f| f.bits).collect();
    let exp: Vec<i128> = cw.iter().filter(|f| starts("(?i)expiry time", &f.meaning)).map(|f| f.bits).collect();
    report.check("AC2", win.len() == 1 && exp.len() == 1, format!("the merged commissioning window section has one window identifier row and one expiry row (read {win:?}, {exp:?})"));
    let wire = ctx.md.lines.join("\n");
    let rej = Enumeration::parse(&wire, REJECTION_004_25);
    let m = Merged29 { ctrl_bound, dscp, cch_rows, block, w_stamp: win.first().copied(), e_stamp: exp.first().copied(), cw, rej, res_rows: reserved_rows(&wire) };
    report.check("DoD", m.rej.has_control_reasons(13, &CONTROL_REASONS_004_29), format!("the merged Rejection reasons enumeration keeps its 13 members, with the control reasons (read {})", m.rej.count()));
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
        report.fail("AC5", format!("{WIRE_TOML} does not exist"));
    }
    let constants = docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default();
    let const_text = LINES.normalize(&join_digit_groups(&universal_newlines(&constants)));
    let timeline = universal_newlines(&docs.read(TIMELINE_MD).map(|t| t.to_string()).unwrap_or_default());
    let ctx = Ctx { md: Markdown::parse(&wire), consts: Constants::parse(&const_text) };
    let m = merged(&ctx, &const_text, &mut report);
    let s = section(&ctx, &mut report);
    let flow = checks(&ctx, &s, &m, &timeline, toml_text.as_deref(), &mut report);
    report.finish(flow)
}

fn checks(ctx: &Ctx, s: &Section, m: &Merged29, timeline: &str, toml_text: Option<&str>, report: &mut Report) -> Flow<()> {
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
    toml_side::ac5(toml_text, s, m, ann.as_ref(), clo.as_ref(), report)
}
