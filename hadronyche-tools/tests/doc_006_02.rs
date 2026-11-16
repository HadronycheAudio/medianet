//! Ticket 006-02 — domain line and position map. Port of `checks/006-02.sh`
//! (deleted with this port; `doccheck-equiv 006-02` reads it from git
//! history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c006_02`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion) and on denial fixtures: the real documents with one mutation
//! applied in memory, each also run against the old script by the ignored
//! test `old_script_agrees_on_the_denials` (see `wp9_support`).
//!
//! The script has no repository-level assertion. Its header says the
//! checker "tries both readings" of a row (relative to the origin or
//! absolute); the code does exactly that and the port follows it.
//!
//! Assertions no document can reach (the port keeps them): "constant
//! self-check failed" (the constants are fixed; the unit test
//! `the_constants_hold` of the module runs the same loops), and "ticket
//! example p=N should be M ns" (a row that passes the formula always has the
//! ticket's value among its offsets).
//!
//! Python semantics with denials below: the heading is matched exactly and
//! case-sensitively and without regard to code fences
//! (`heading_is_case_sensitive`, `a_fenced_heading_is_found_first`); an
//! identity needs the same variable on both sides
//! (`ac2_denial_identity_with_another_variable`); a non-whole decimal in a
//! cell is a `Fraction` (`ac5_denial_a_decimal_value`).

#[macro_use]
mod wp9_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c006_02::{SPEC_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{replace_all, replace_in_line};
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

fn spec(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(SPEC_MD, edit)
}

/// The section only: `edit` applied to the text between the heading and the
/// next `## `, so an earlier use of the same words cannot be hit.
fn section(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    spec(b, |t| {
        let a = t.find("## Domain line and position map").expect("heading");
        let z = t[a + 3..].find("\n## ").map_or(t.len(), |i| a + 3 + i);
        format!("{}{}{}", &t[..a], edit(&t[a..z]), &t[z..])
    })
}

fn drop_lines(t: &str, pattern: &str) -> String {
    rx(&format!(r"(?m)^(?:{pattern}).*\n"))
        .replace_all(t, "")
        .into_owned()
}

/// AC1 — "`position_at` and `time_of` are given as formulas with floor and
/// ceil and a 128-bit intermediate."
#[test]
fn ac1_the_formulas_with_floor_ceil_and_128_bit() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The inverse identities and the exactness of slice boundaries (48
/// samples = 500 000 ns) are stated with their proof."
#[test]
fn ac2_the_inverse_identities_and_exact_boundaries() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The rounding rule off a slice boundary (Q-1) is stated, with at
/// least three off-boundary examples."
#[test]
fn ac3_the_rounding_rule_off_a_boundary() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "Behaviour before the origin (Q-2) is specified and tagged
/// `[decision]`."
#[test]
fn ac4_behaviour_before_the_origin() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "A worked-example table of at least six rows, both directions, a
/// slice boundary, every row recomputed independently."
#[test]
fn ac5_the_worked_examples_recompute() {
    assert_criterion(report(), "AC5");
}

/// AC6 — "96 kHz is stated as the only rate with a position space, citing
/// C-01, C-80 and C-134."
#[test]
fn ac6_the_only_rate_and_its_citations() {
    assert_criterion(report(), "AC6");
}

denials! {
    denial_timeline_missing => ("AC1", "missing doc/design/spec/timeline.md", |b: &Docs| b.without(SPEC_MD));
    denial_no_heading => ("AC1", "has no '## Domain line and position map' heading", |b| spec(b, |t| replace_in_line(t, "^## Domain line and position map$", "position map", "position maps")));
    heading_is_case_sensitive => ("AC1", "has no '## Domain line and position map' heading", |b| spec(b, |t| replace_in_line(t, "^## Domain line and position map$", "Domain line", "Domain Line")));
    a_fenced_heading_is_found_first => ("AC1", "no 'position_at(...) = ...' formula", |b| spec(b, |t| format!("```\n## Domain line and position map\n```\n{t}")));
    denial_the_placeholder => ("AC1", "still the 006-02 placeholder", |b| spec(b, |t| replace_in_line(t, "^## Domain line and position map$", "$", "\n\nTo be written in 006-02.")));
    ac1_denial_no_position_formula => ("AC1", "no 'position_at(...) = ...' formula", |b| section(b, |t| replace_all(t, "position_at", "pos_at")));
    ac1_denial_no_time_formula => ("AC1", "no 'time_of(...) = ...' formula", |b| section(b, |t| replace_all(t, "time_of", "tm_of")));
    ac1_denial_no_floor => ("AC1", "position_at does not state floor", |b| section(b, |t| replace_all(t, "(?i)floor\\w*|round(?:ed|ing)?\\s+down", "x")));
    ac1_denial_no_ceil => ("AC1", "time_of does not state ceil", |b| section(b, |t| replace_all(t, "(?i)ceil\\w*|round(?:ed|ing)?\\s+up", "x")));
    ac1_denial_no_128_bit => ("AC1", "does not name a 128-bit", |b| section(b, |t| replace_all(t, "128", "256")));
    ac2_denial_identity_with_another_variable => ("AC2", "does not state position_at(time_of(p)) == p", |b| section(b, |t| replace_all(t, r"position_at\(time_of\(p\)\) == p", "position_at(time_of(p)) == q")));
    ac2_denial_no_second_identity => ("AC2", "does not state time_of(position_at(t)) <= t", |b| section(b, |t| replace_all(t, r"time_of\(position_at\(t\)\) <= t", "time_of(position_at(t)) <= u")));
    ac2_denial_no_500000 => ("AC2", "does not give 48 samples = 500 000 ns", |b| section(b, |t| replace_all(t, r"500 000", "600 000")));
    ac2_denial_no_exactness_argument => ("AC2", "no argument that 48-sample slice boundaries are exact", |b| section(b, |t| replace_all(t, r"(?i)exact\w*|without rounding|whole nanosecond|no rounding", "x")));
    ac2_denial_exactness_without_boundaries => ("AC2", "exactness argument does not mention slice boundaries", |b| section(b, |t| replace_all(t, r"(?i)boundar\w*", "edge")));
    ac2_denial_no_12_125000_reduction => ("AC2", "does not show the 12 / 125 000 reduction", |b| section(b, |t| replace_all(t, r"125 000", "125 001")));
    ac2_denial_no_31250_3_reduction => ("AC2", "does not show the 31 250 / 3 reduction", |b| section(b, |t| replace_all(t, r"31 250", "31 251")));
    ac3_denial_no_off_boundary_rule => ("AC3", "does not state the rounding rule for positions off a slice boundary", |b| section(b, |t| replace_all(t, r"(?i)off[\s-]*(?:a\s+)?(?:slice\s+)?boundar\w*|inside\s+(?:a\s+)?slice|within\s+(?:a\s+)?slice|not\s+on\s+a\s+slice\s+boundar\w*", "x")));
    ac3_denial_fewer_than_three_off_boundary_rows => ("AC3", "fewer than three off-boundary example rows", |b| section(b, |t| drop_lines(t, r"\| time_of \| p = (?:1|2|3|4|49) \||\| position_at \| origin \+ (?:15 000|31 249) ns")));
    ac4_denial_before_the_origin_without_a_decision_tag => ("AC4", "not specified and tagged [decision]", |b| section(b, |t| replace_all(t, r"\[decision\]", "[choice]")));
    ac4_denial_before_the_origin_without_a_result => ("AC4", "does not say what is returned", |b: &Docs| b.with(SPEC_MD, "## Domain line and position map\n\nA negative time is a `[decision]` panic.\n"));
    ac5_denial_a_decimal_value => ("AC5", "is not an integer", |b| section(b, |t| replace_in_line(t, r"^\| position_at \| origin \+ 510 417 ns", "$", "\n| time_of | p = 5 | origin + 52 083.5 ns |")));
    ac5_denial_a_row_with_one_number => ("AC5", "fewer than two numeric values", |b| section(b, |t| replace_in_line(t, r"^\| position_at \| origin \+ 510 417 ns", "$", "\n| time_of | p | origin |")));
    ac5_denial_a_row_that_fails_the_formula => ("AC5", "row fails the section's own formula", |b| section(b, |t| replace_all(t, r"origin \+ 10 417 ns", "origin + 10 418 ns")));
    ac5_denial_fewer_than_six_rows => ("AC5", "0 verified row(s); at least 6 are required", |b| section(b, |t| replace_all(t, r"\|---\|---\|---\|", "|-|-|-|")));
    ac5_denial_one_direction_only => ("AC5", "does not give both directions", |b| section(b, |t| drop_lines(t, r"\| position_at \|")));
    ac5_denial_no_slice_boundary_row => ("AC5", "no slice-boundary example row", |b| section(b, |t| drop_lines(t, r"\| time_of \| p = (?:48|96) \||\| position_at \| origin \+ 500 000 ns")));
    ac6_denial_not_the_only_rate => ("AC6", "does not state that 96 kHz is the only rate", |b| section(b, |t| replace_all(t, r"(?i)\bonly\b|no other|single|sole", "x")));
    ac6_denial_a_citation_missing => ("AC6", "missing citation(s): C-80", |b| section(b, |t| replace_all(t, "C-80", "C-8O")));
}

/// Every denial is rejected by the old script, except the ones listed here.
const SCRIPT_ACCEPTS: &[&str] = &[];

/// `cargo test -p hadronyche-tools --test doc_006_02 -- --ignored`.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees_on_the_denials() {
    let all = all_denials();
    assert_eq!(wp9_support::script_accepts("006-02", &all), SCRIPT_ACCEPTS);
}

/// The tree at the commit that added the script is the placeholder: the
/// script (and the port) fail AC1 to AC6 there (skipped when the commit is
/// not in the clone).
#[test]
fn red_phase_tree_fails() {
    let commit = "74dd39a0b4e4d493c8909052dc183db5e55289d1";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    assert!(!r.passed(), "{}", r.render());
}
