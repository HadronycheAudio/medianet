//! Port of `checks/004-28.sh`: the regime stamp sections added after the
//! split — the observed DSCP of peers and the alignment group (ticket
//! 004-28).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_28.rs`. The script shares 004-25's conventions, and the
//! port reuses 004-25's port for them: wire.md read with U+2212 turned into
//! `-` and digit groups joined; `span_of` at an exact level; the
//! Rejection reasons anchored and pinned by name and value
//! ([`crate::doccheck::checks::c004_25::merged::REJ_FIXED`]); the
//! arithmetic tree ([`crate::doccheck::checks::c004_25::tree`]); its
//! `sentences`; the reason rules of an invalid form
//! ([`crate::doccheck::checks::c004_25::validity::row_rules`], with "no
//! reason" in the non-rejection pattern); the untagged-claim rule per prose
//! block of each new subsection ([`TagPolicy::PROSE_004_29`]); open items as
//! any block of the Regime stamp section that mentions "open item"
//! ([`MENTION_BLOCKS_004_24`]). Its own variants: a sum Python cannot
//! evaluate is "not exact" rather than a stop
//! ([`crate::doccheck::checks::c004_25::tree::bad_arithmetic_lenient`]);
//! `has_num2` ([`analyse::has_num2`]); the toml message of a new type must
//! name at least one reason.
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed, the count at the merged text; the checker
//! names no ref containing main and runs no other check) are in
//! `scripts/repo-state.d/004-28.sh`.
//!
//! [`TagPolicy::PROSE_004_29`]: crate::doccheck::evidence::TagPolicy::PROSE_004_29
//! [`MENTION_BLOCKS_004_24`]: crate::doccheck::checks::c004_24::dod::MENTION_BLOCKS_004_24

mod align;
mod analyse;
mod budget;
mod dscp;
mod toml_side;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::checks::c004_24::dod::{MENTION_BLOCKS_004_24, block_items};
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::{py_text, stopped, tables};
use crate::doccheck::checks::c004_25::merged::{code_block, merged};
use crate::doccheck::checks::c004_25::review::sentences;
use crate::doccheck::checks::c004_25::validity::row_rules;
use crate::doccheck::checks::c004_25::Stamp;
use crate::doccheck::checks::c004_27::{Ctx, subsections};
use crate::doccheck::evidence::{TagPolicy, has_tag, untagged};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::text::{has, has_i, head, join_digit_groups, rx};
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::units::prose_blocks;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-28",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Observed DSCP of peers section$") },
        Target { file: WIRE_MD, heading: Some("^Alignment group section$") },
        Target { file: WIRE_MD, heading: Some("^Section count with the types of 004-28$") },
        Target { file: WIRE_MD, heading: Some("^Regime stamp$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_MD, heading: Some("^Reserved fields$") },
        Target { file: WIRE_TOML, heading: None },
        Target { file: CONSTANTS_MD, heading: None },
    ],
    keywords: &["exceed", "cap", "holder", "rolling", "reserved"],
    known_differences: KNOWN,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The numbers the constants give.
pub(crate) struct Consts {
    pub ctrl_bound: Option<i64>,
    pub max_peers: Option<i64>,
    pub dscp: Option<i64>,
    pub accept_hi: Option<i64>,
    pub window_s: Option<i64>,
}

fn consts(c: &Constants, report: &mut Report) -> Consts {
    let int = |id: &str, pat: &str, g: usize| rx(pat).captures(c.value(id)).and_then(|m| m[g].parse::<i64>().ok());
    let ctrl_bound = int("C-17", r"at most (\d+) bytes", 1);
    let modules = int("C-34", r"at most (\d+)", 1);
    let followers = int("C-114", r"holder has at most (\d+) real followers", 1);
    let max_peers = int("C-217", r"(\d+) peers", 1);
    let dscp = int("C-13", r"control CS5 \((\d+)\)", 1);
    let accept_hi = int("C-112", r"(\d+) to (\d+) per second", 2);
    let window_s = int("C-182", r"rolling (\d+) s window", 1);
    let peer_max = modules.map(|m| m - 1);
    report.check("AC1", ctrl_bound == Some(1252), format!("the constants table gives the control bound C-17 as 1 252 bytes (read {ctrl_bound:?})"));
    report.check("AC2", peer_max.is_some() && peer_max == max_peers && max_peers == followers, format!("the constants give 63 peers and 63 followers: 64 modules (C-34) less the module itself (C-217), and at most 63 real followers (C-114) (read {modules:?}, {max_peers:?}, {followers:?})"));
    report.check("AC2", dscp == Some(40), format!("the constants give control CS5 as 40 (C-13) (read {dscp:?})"));
    report.check("AC3", window_s == Some(60), format!("the constants give the lateness window as a rolling 60 s window (C-182) (read {window_s:?})"));
    report.check("AC2", accept_hi == Some(200), format!("the constants give the accepted sync request rate as 10 to 200 per second (C-112) (read {accept_hi:?})"));
    Consts { ctrl_bound, max_peers, dscp, accept_hi, window_s }
}

/// The invalid forms of the new subsections, by type.
fn forms(t: &budget::Types, m: &crate::doccheck::checks::c004_25::merged::Merged, wide: bool, report: &mut Report) -> Vec<(String, String)> {
    let mut rows: Vec<(&str, String, String)> = Vec::new();
    for (key, a) in &t.new {
        for tb in tables(&a.text) {
            let h = tb.head();
            if h.first().is_some_and(|x| x.to_lowercase().starts_with("invalid form")) && h.len() > 1 && h[1].to_lowercase().contains("required outcome") {
                rows.extend(tb.rows.iter().filter(|r| r.cells.len() >= 2).map(|r| (*key, r.cell(0).to_string(), r.cell(1).to_string())));
            }
        }
    }
    report.check("AC4", rows.len() >= 5, format!("at least 5 invalid forms with outcomes in the new subsections (found {})", rows.len()));
    for (_, f, o) in &rows {
        row_rules(f, o, m, "AC4", report);
    }
    let cats = [
        ("a DSCP value above 63", "DSCP"),
        ("a count above its maximum", "count|above|exceed|more than|maximum"),
        ("a length that disagrees", "length"),
        ("percentiles out of non-decreasing order", "non-decreasing|percentile|50th|99th"),
        ("reserved bits non-zero", "reserved|non-?zero"),
    ];
    for (cat, pat) in cats {
        if cat == "a DSCP value above 63" && !wide {
            continue;
        }
        report.check("AC4", rows.iter().any(|(_, f, _)| has_i(pat, f)), format!("an invalid form covers: {cat} (Work 3)"));
    }
    rows.into_iter().filter(|(k, _, _)| *k == "alignment").map(|(_, f, o)| (f, o)).collect()
}

fn dod(st: &Stamp, t: &budget::Types, m: &crate::doccheck::checks::c004_25::merged::Merged, report: &mut Report) {
    for (key, a) in &t.new {
        report.check("DoD", has_tag(&a.text), format!("the {key} subsection carries an evidence tag"));
        let bad: Vec<String> = untagged(&a.text, &TagPolicy::PROSE_004_29).iter().map(|u| head(&u.text, 50)).collect();
        report.check("DoD", bad.is_empty(), format!("every prose paragraph of 5 or more words in the {key} subsection carries an evidence tag; untagged: {bad:?}"));
    }
    let share: Vec<String> = sentences(&st.text).into_iter().filter(|x| x.contains("share") && x.contains("address") && x.contains("host")).collect();
    report.check("DoD", share.iter().all(|x| has("Admission|Receiver report", x)), "review C4: the sentence on two modules on one host sharing an address cites the merged wire text (Admission or Receiver report), not 01-architecture § 2.6 alone");
    let (n, unowned) = block_items(&st.text, &MENTION_BLOCKS_004_24);
    report.check("DoD", unowned.is_empty(), format!("every Open item paragraph of the regime stamp section names an owner and a falsifier ({n} found)"));
    let n_res_new = t.new.values().flat_map(|a| a.fl_all.iter()).filter(|f| has_i(r"\breserved\b", &f.meaning)).count();
    let res = m.res_rows.iter().filter(|r| r.cells.len() > 1 && r.cell(1).to_lowercase().contains("regime stamp")).count();
    report.check("DoD", res >= 4 + n_res_new, format!("every reserved field of the new types ({n_res_new} found) is recorded in the merged Reserved fields table, one row each (the table has {res} regime stamp rows, the merged four included)"));
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
    let c = consts(&constants, &mut report);
    let ctx = Ctx { md: Markdown::parse(&wire), consts: constants, toml_text };
    let span = ctx.span_of(r"^regime\s+stamp$", 2);
    let text = ctx.body(span);
    report.check("AC1", span.is_some() && !text.to_lowercase().contains("to be filled"), "wire.md has a Regime stamp section with its placeholder replaced");
    let st = Stamp { subs: subsections(&text), prose: prose_blocks(&text), text };
    if let Err(Stop(e)) = checks(&ctx, &st, &wire, &c, &mut report) {
        stopped(&mut report, "AC1", &e);
    }
    report
}

fn checks(ctx: &Ctx, st: &Stamp, wire: &str, c: &Consts, report: &mut Report) -> Result<(), Stop> {
    let m = merged(ctx, wire, &st.text, report, |_| {})?;
    let res = m.res_rows.iter().filter(|r| r.cells.len() > 1 && r.cell(1).to_lowercase().contains("regime stamp")).count();
    report.check("DoD", res >= 4, "the merged Reserved fields table keeps its four regime stamp rows (the new types add rows, none is removed)");
    let block = code_block(wire, "004-28");
    report.check("AC1", block.is_some(), format!("the merged section type codes give 004-28 a block (read {block:?})"));
    rest(ctx, st, &m, c, block, report).map_err(Stop)
}

fn rest(ctx: &Ctx, st: &Stamp, m: &crate::doccheck::checks::c004_25::merged::Merged, c: &Consts, block: Option<(i64, i64)>, report: &mut Report) -> Result<(), String> {
    let (merged_types, s_other, f_h) = budget::merged(st, report)?;
    let new = budget::new_types(st, report);
    let t = budget::Types { merged: merged_types, s_other, f_h, new };
    budget::per_type(&t, c.ctrl_bound, block, report)?;
    budget::budget(st, &t, m.section_max, report)?;
    let wide = dscp::ac2(t.new.get("dscp"), c, report);
    align::ac3(t.new.get("alignment"), t.health(), c, report);
    let align_rows = forms(&t, m, wide, report);
    align::c1(t.new.get("alignment"), &align_rows, report);
    dod(st, &t, m, report);
    toml_side::ac5(ctx.toml_text.as_deref(), &t, m, c.dscp, report)?;
    Ok(())
}
