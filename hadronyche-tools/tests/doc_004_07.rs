//! Ticket 004-07 — timeline epoch wrap analysis (C-10, 16 bits). Port of
//! `checks/004-07.sh` (deleted with this port; `doccheck-equiv 004-07` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_07`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, each also run against the
//! old script by the ignored test `old_script_agrees_on_the_denials` (see
//! `wp9_support`; the harness gives the script its override variables as
//! `./` paths, so it skips its repository-level runs).
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-07.sh`): the design checker
//! (`doc/kickoff/check-design.py doc/design`) prints OK, and
//! `checks/004-01.sh` still passes.
//!
//! The script sets `CONST_MD` and never reads it. Header and code differ in
//! one place: the header says the stated time-to-wrap must agree "30 %";
//! the code accepts a duration within 30 % of 65 536 x interval anywhere in
//! a 700-before/900-after character window of the marker, and the port
//! follows the code. One assertion no document can reach: "the analysis
//! claims falsified but the plausible wrap is at or above a year" (its branch
//! runs only when the verdict does not claim falsified).
//!
//! Python semantics with denials below: the analysis heading is found
//! without regard to fences (`a_fenced_heading_is_found_first`); windows are
//! in characters, not bytes (`window_in_characters`).

#[macro_use]
mod wp9_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_07::{WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{replace_all, replace_in_line, within_section};
use hadronyche_tools::doccheck::text::rx;
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion};

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

/// wire.md with `edit` applied to the analysis section only.
fn sec(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(WIRE_MD, |t| {
        within_section(t, "^Timeline epoch width and wrap$", edit)
    })
}

/// `edit` applied to every line of `t` that starts with `prefix`.
fn para(t: &str, prefix: &str, edit: impl Fn(&str) -> String) -> String {
    let mut hit = false;
    let out: Vec<String> = t
        .split('\n')
        .map(|l| {
            if l.starts_with(prefix) {
                hit = true;
                edit(l)
            } else {
                l.to_string()
            }
        })
        .collect();
    assert!(hit, "no line starts with {prefix:?}");
    out.join("\n")
}

fn re_all(t: &str, pattern: &str, to: &str) -> String {
    rx(pattern).replace_all(t, to).into_owned()
}

/// A small synthetic wire.md whose analysis passes every assertion; the
/// denials that the real section cannot isolate start from it.
const SYN: &str = "# Doc\n\n### Timeline epoch width and wrap\n\nThe epoch is 65 536 values (C-10), sources C-113, C-213, C-136, C-32, C-33, C-37 `[decision]`. Pathological ceiling: one bump every 2 s `[decision]`; 65 536 × 2 s = 131 072 s, about 1.5 days. Plausible worst-case operating rate: one bump every 10 minutes `[memory]`; 65 536 × 10 minutes = 39 321 600 s, about 1.25 years. Verdict against the 1 year falsifier: C-10 is not falsified at the plausible rate `[decision]`; the pathological ceiling is far under a year and is answered here. A receiver decides current versus stale by comparing epochs by equality; at wrap the same value recurs.\n\n## Next\n";

fn syn(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.with(WIRE_MD, edit(SYN))
}

/// The synthetic section passes (and the old script agrees: the ignored test
/// below runs it too).
#[test]
fn the_synthetic_section_passes() {
    assert!(run(&syn(base(), |t| t.to_string())).passed());
}

/// The plausible rate made one bump a minute: the wrap is under a year.
fn fast(t: &str) -> String {
    replace_all(t, "every 10 minutes", "every 1 minute")
}

/// AC1 — "The analysis states both rates (pathological ceiling and plausible
/// worst case), their sources, the time to wrap 65 536 values at each, and a
/// verdict against the 1-year falsifier of C-10."
#[test]
fn ac1_both_rates_their_wrap_times_and_a_consistent_verdict() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "A comparison rule for epochs (current versus stale), including its
/// behaviour at wrap, is specified."
#[test]
fn ac2_the_comparison_rule_at_wrap() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "If falsified, the fix, the constants-table flag and an open item
/// with an owner and a falsifier are recorded." (vacuous when not falsified;
/// any open item the analysis raises still needs both.)
#[test]
fn ac3_the_falsification_record_when_falsified() {
    assert_criterion(report(), "AC3");
}

/// DoD — "Every new factual claim carries an evidence tag" and "No new open
/// item without an owner and a falsifier": at least three tags in the
/// section, a `[decision]` or `[memory]` tag beside each rate, a tag beside
/// the verdict, owned open items.
#[test]
fn dod_evidence_tags_and_open_items() {
    assert_criterion(report(), "DoD");
}

denials! {
    denial_wire_md_missing => ("AC1", "missing doc/design/spec/wire.md", |b: &Docs| b.without(WIRE_MD));
    denial_no_analysis_section => ("AC1", "neither an epoch wrap/bump section nor the Media packet header section", |b: &Docs| b.mutate(WIRE_MD, |t: &str| re_all(t, r"(?im)^(#{1,6}\s+).*(?:media packet header|epoch).*$", "${1}Plain")));
    denial_the_placeholder => ("AC1", "still a placeholder or empty", |b| sec(b, |t| format!("{t}\n\nTo be written.\n")));
    a_fenced_heading_is_found_first => ("AC1", "never states the 65 536", |b: &Docs| b.mutate(WIRE_MD, |t: &str| format!("```\n## Epoch wrap\n```\n\nShort.\n\n{t}")));
    ac1_denial_no_65536 => ("AC1", "never states the 65 536", |b| sec(b, |t| re_all(t, r"65[\d,   ]*536|2\^16|2\*\*16", "N")));
    ac1_denial_a_source_not_cited => ("AC1", "missing source citation(s): C-213", |b| sec(b, |t| replace_all(t, "C-213", "C-2l3")));
    ac1_denial_no_pathological_rate => ("AC1", "no pathological rate stated", |b| sec(b, |t| re_all(t, r"(?i)patholog\w*|ceiling", "bound")));
    ac1_denial_the_ceiling_without_a_matching_wrap => ("AC1", "the pathological rate", |b| syn(b, |t| replace_all(t, "131 072 s, about 1.5 days", "9 s")));
    ac1_denial_the_plausible_rate_without_a_matching_wrap => ("AC1", "the plausible rate", |b| sec(b, |t| replace_all(&replace_all(&replace_all(t, "39 321 600 s", "100 s"), "455 days", "5 days"), "1.25 years", "5 years")));
    ac1_denial_a_ceiling_faster_than_the_election_timers => ("AC1", "faster than the election timers allow", |b| sec(b, |t| replace_all(t, "1.9 s", "1.5 s")));
    ac1_denial_a_plausible_rate_faster_than_the_ceiling => ("AC1", "internal inconsistency", |b| syn(b, |t| replace_all(t, "every 10 minutes", "every 1 s")));
    ac1_denial_no_one_year_threshold => ("AC1", "1-year falsifier threshold is never named", |b| sec(b, |t| re_all(t, r"(?i)1[\s ]?year|one\s+year|a\s+year|365\s*days", "x")));
    ac1_denial_no_verdict => ("AC1", "no decidable verdict", |b| sec(b, |t| re_all(t, r"(?i)falsified|C-10\s+(?:holds|stands|survives)", "x")));
    ac1_denial_a_contradictory_verdict => ("AC1", "the verdict is contradictory", |b| sec(b, |t| format!("{t}\n\nC-10 is falsified `[decision]`.\n")));
    ac1_denial_falsified_although_the_wrap_exceeds_a_year => ("AC1", "yet the analysis says falsified", |b| syn(b, |t| replace_all(t, "C-10 is not falsified", "C-10 is falsified")));
    ac1_denial_not_falsified_although_the_wrap_is_under_a_year => ("AC1", "yet the analysis says not falsified", |b| sec(b, fast));
    ac1_denial_the_verdict_never_addresses_the_ceiling => ("AC1", "never addresses the ceiling", |b| sec(b, |t| para(t, "**Verdict against the falsifier.**", |l| re_all(l, r"(?i)patholog\w*|ceiling", "bound"))));
    window_in_characters => ("AC1", "never addresses the ceiling", |b| sec(b, |t| para(&para(t, "**Verdict against the falsifier.**", |l| re_all(l, r"(?i)patholog\w*|ceiling", "bound")), "**Open item.**", |l| format!("{} {}", "\u{2014}".repeat(420), l))));
    ac2_denial_no_comparison_rule => ("AC2", "no comparison rule for 'current' vs 'stale' epochs", |b: &Docs| b.mutate(WIRE_MD, |t: &str| re_all(t, "(?i)stale", "old")));
    ac3_denial_a_wrap_under_a_year_not_declared_falsified => ("AC3", "does not declare C-10 falsified", |b| sec(b, fast));
    ac3_denial_falsified_without_a_replacement => ("AC3", "no proposed replacement width", |b| syn(b, |t| fast(&replace_all(t, "C-10 is not falsified", "C-10 is falsified"))));
    ac3_denial_falsified_without_the_constants_table_flag => ("AC3", "not flagged for the constants table", |b| sec(b, |t| re_all(&fast(&replace_all(t, "C-10 is not falsified at the plausible rate", "C-10 is falsified at the plausible rate")), r"(?i)constants table", "table")));
    ac3_denial_falsified_without_an_open_item_owner => ("AC3", "no open item with an owner and a falsifier", |b| sec(b, |t| re_all(&fast(&replace_all(t, "C-10 is not falsified at the plausible rate", "C-10 is falsified at the plausible rate")), r"(?i)owner", "keeper")));
    dod_denial_an_open_item_without_an_owner => ("DoD", "raises an open item without an owner and a falsifier", |b| sec(b, |t| replace_in_line(t, "^The timeline epoch is 16 bits", "^", "Open item: later. ")));
    dod_denial_fewer_than_three_tags => ("DoD", "evidence tag(s); at least 3 are required", |b| sec(b, |t| re_all(t, r"\[(?:decision|memory|measured|code|standard|kernel-doc)\]", "")));
    dod_denial_the_ceiling_without_a_rate_tag => ("DoD", "the pathological rate carries no [decision]/[memory] tag", |b| sec(b, |t| re_all(&re_all(t, r"\[memory\]", "[code]"), r"\[decision\]", "[code]")));
    dod_denial_the_verdict_without_a_tag => ("DoD", "the verdict carries no evidence tag", |b| sec(b, |t| re_all(t, TAGS, "")));
}

const TAGS: &str = r"\[(?:decision|memory|measured|code|standard|kernel-doc)\]";

/// Every denial is rejected by the old script, except the ones listed here.
const SCRIPT_ACCEPTS: &[&str] = &[];

/// `cargo test -p hadronyche-tools --test doc_004_07 -- --ignored`.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees_on_the_denials() {
    let all = all_denials();
    assert_eq!(wp9_support::script_accepts("004-07", &all), SCRIPT_ACCEPTS);
    let synthetic = [wp9_support::Denial {
        name: "synthetic",
        criterion: "",
        needle: "",
        docs: syn(base(), |t| t.to_string()),
    }];
    assert_eq!(
        wp9_support::script_accepts("004-07", &synthetic),
        ["synthetic"],
        "the old script must accept the synthetic section"
    );
}

/// The tree at the commit that added the script fails it (skipped when the
/// commit is not in the clone).
#[test]
fn red_phase_tree_fails() {
    let commit = "abf15d66e03cc6e92f45b52c7d9c9ecfb2033213";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    assert!(!r.passed(), "{}", r.render());
}
