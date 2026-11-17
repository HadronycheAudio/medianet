//! Ticket 004-07 — timeline epoch wrap analysis (C-10, 16 bits). Port of
//! `checks/004-07.sh` (deleted with this port; `doccheck-equiv 004-07` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_07`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, each also run against the
//! old script by the harness (`doccheck-equiv 004-07 --denials-only`; it
//! gives the script its override variables as `./` paths, so it skips its
//! repository-level runs).
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

use hadronyche_tools::doccheck::assert_criterion;
use hadronyche_tools::doccheck::checks::c004_07::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};

/// AC1 — "The analysis states both rates (pathological ceiling and plausible
/// worst case), their sources, the time to wrap 65 536 values at each, and a
/// verdict against the 1-year falsifier of C-10."
#[test]
fn ac1_both_rates_their_wrap_times_and_a_consistent_verdict() {
    assert_criterion(&report_of(&SPEC), "AC1");
}

/// AC2 — "A comparison rule for epochs (current versus stale), including its
/// behaviour at wrap, is specified."
#[test]
fn ac2_the_comparison_rule_at_wrap() {
    assert_criterion(&report_of(&SPEC), "AC2");
}

/// AC3 — "If falsified, the fix, the constants-table flag and an open item
/// with an owner and a falsifier are recorded." (vacuous when not falsified;
/// any open item the analysis raises still needs both.)
#[test]
fn ac3_the_falsification_record_when_falsified() {
    assert_criterion(&report_of(&SPEC), "AC3");
}

/// DoD — "Every new factual claim carries an evidence tag" and "No new open
/// item without an owner and a falsifier": at least three tags in the
/// section, a `[decision]` or `[memory]` tag beside each rate, a tag beside
/// the verdict, owned open items.
#[test]
fn dod_evidence_tags_and_open_items() {
    assert_criterion(&report_of(&SPEC), "DoD");
}

/// The tree at the commit that added the script fails it.
#[test]
fn red_phase_tree_fails() {
    red_phase(
        &SPEC,
        "abf15d66e03cc6e92f45b52c7d9c9ecfb2033213",
        &["AC1", "AC2"],
    );
}

hadronyche_tools::denial_tests!(
    SPEC;
    an_interval_of_0,
    an_interval_of_007,
    an_interval_of_plus7,
    an_interval_of_1_2,
    an_interval_of_2e63,
    an_interval_of_20_digits,
    an_interval_of_45_digits,
    an_interval_of_1e13,
    an_interval_of_1e36,
    an_interval_of_superscript,
    an_interval_of_micro,
    denial_wire_md_missing,
    denial_no_analysis_section,
    denial_the_placeholder,
    a_fenced_heading_is_found_first,
    ac1_denial_no_65536,
    ac1_denial_a_source_not_cited,
    ac1_denial_no_pathological_rate,
    ac1_denial_the_ceiling_without_a_matching_wrap,
    ac1_denial_the_plausible_rate_without_a_matching_wrap,
    ac1_denial_a_ceiling_faster_than_the_election_timers,
    ac1_denial_a_plausible_rate_faster_than_the_ceiling,
    ac1_denial_no_one_year_threshold,
    ac1_denial_no_verdict,
    ac1_denial_a_contradictory_verdict,
    ac1_denial_falsified_although_the_wrap_exceeds_a_year,
    ac1_denial_not_falsified_although_the_wrap_is_under_a_year,
    ac1_denial_the_verdict_never_addresses_the_ceiling,
    window_in_characters,
    ac2_denial_no_comparison_rule,
    ac3_denial_a_wrap_under_a_year_not_declared_falsified,
    ac3_denial_falsified_without_a_replacement,
    ac3_denial_falsified_without_the_constants_table_flag,
    ac3_denial_falsified_without_an_open_item_owner,
    dod_denial_an_open_item_without_an_owner,
    dod_denial_fewer_than_three_tags,
    dod_denial_the_ceiling_without_a_rate_tag,
    dod_denial_the_verdict_without_a_tag,
    a_huge_plausible_interval_passes,
    a_400_digit_interval,
    a_leading_dot_interval,
    a_comma_in_an_interval_passes,
    a_superscript_after_an_interval,
    a_stated_wrap_exactly_on_the_tolerance,
    a_stated_wrap_just_inside_the_tolerance,
    a_stated_wrap_just_outside_the_tolerance,
    crlf_line_ends_pass,
    crlf_line_ends_in_the_synthetic_section_pass,
);
