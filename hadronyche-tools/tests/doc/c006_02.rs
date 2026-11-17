//! Ticket 006-02 — domain line and position map. Port of `checks/006-02.sh`
//! (deleted with this port; `doccheck-equiv 006-02` reads it from git
//! history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c006_02`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion) and on denial fixtures: the real documents with one mutation
//! applied in memory, each also run against the old script by the ignored
//! harness (`doccheck-equiv 006-02 --denials-only`).
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

use hadronyche_tools::doccheck::assert_criterion;
use hadronyche_tools::doccheck::checks::c006_02::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};

/// AC1 — "`position_at` and `time_of` are given as formulas with floor and
/// ceil and a 128-bit intermediate."
#[test]
fn ac1_the_formulas_with_floor_ceil_and_128_bit() {
    assert_criterion(&report_of(&SPEC), "AC1");
}

/// AC2 — "The inverse identities and the exactness of slice boundaries (48
/// samples = 500 000 ns) are stated with their proof."
#[test]
fn ac2_the_inverse_identities_and_exact_boundaries() {
    assert_criterion(&report_of(&SPEC), "AC2");
}

/// AC3 — "The rounding rule off a slice boundary (Q-1) is stated, with at
/// least three off-boundary examples."
#[test]
fn ac3_the_rounding_rule_off_a_boundary() {
    assert_criterion(&report_of(&SPEC), "AC3");
}

/// AC4 — "Behaviour before the origin (Q-2) is specified and tagged
/// `[decision]`."
#[test]
fn ac4_behaviour_before_the_origin() {
    assert_criterion(&report_of(&SPEC), "AC4");
}

/// AC5 — "A worked-example table of at least six rows, both directions, a
/// slice boundary, every row recomputed independently."
#[test]
fn ac5_the_worked_examples_recompute() {
    assert_criterion(&report_of(&SPEC), "AC5");
}

/// AC6 — "96 kHz is stated as the only rate with a position space, citing
/// C-01, C-80 and C-134."
#[test]
fn ac6_the_only_rate_and_its_citations() {
    assert_criterion(&report_of(&SPEC), "AC6");
}

/// The tree at the commit that added the script is the placeholder: the
/// script (and the port) fail AC1 to AC6 there.
#[test]
fn red_phase_tree_fails() {
    red_phase(
        &SPEC,
        "74dd39a0b4e4d493c8909052dc183db5e55289d1",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "AC6"],
    );
}

hadronyche_tools::denial_tests!(
    SPEC;
    a_position_of_0,
    a_position_of_007,
    a_position_of_plus7,
    a_position_of_1_2,
    a_position_of_2e63,
    a_position_of_20_digits,
    a_position_of_45_digits,
    a_position_of_1e13,
    a_position_of_1e36,
    a_position_of_superscript,
    a_position_of_micro,
    denial_timeline_missing,
    denial_no_heading,
    heading_is_case_sensitive,
    a_fenced_heading_is_found_first,
    denial_the_placeholder,
    ac1_denial_no_position_formula,
    ac1_denial_no_time_formula,
    ac1_denial_no_floor,
    ac1_denial_no_ceil,
    ac1_denial_no_128_bit,
    ac2_denial_identity_with_another_variable,
    ac2_denial_no_second_identity,
    ac2_denial_no_500000,
    ac2_denial_no_exactness_argument,
    ac2_denial_exactness_without_boundaries,
    ac2_denial_no_12_125000_reduction,
    ac2_denial_no_31250_3_reduction,
    ac3_denial_no_off_boundary_rule,
    ac3_denial_fewer_than_three_off_boundary_rows,
    ac4_denial_before_the_origin_without_a_decision_tag,
    ac4_denial_before_the_origin_without_a_result,
    ac5_denial_a_decimal_value,
    ac5_denial_a_row_with_one_number,
    ac5_denial_a_row_that_fails_the_formula,
    ac5_denial_fewer_than_six_rows,
    ac5_denial_one_direction_only,
    ac5_denial_no_slice_boundary_row,
    ac6_denial_not_the_only_rate,
    ac6_denial_a_citation_missing,
    a_row_with_1e19_positions_passes,
    a_row_with_1e36_positions_is_beyond_i128,
    a_row_with_1e40_positions_is_beyond_i128,
    a_position_written_1e3,
    a_position_written_dot_5,
    a_position_with_a_plus_sign_passes,
    a_position_with_a_superscript,
    a_position_with_an_underscore_separator,
    a_negative_position,
    a_whole_decimal_position_passes,
    a_comma_grouped_position,
    another_declared_origin,
    a_declared_origin_beyond_i128,
    crlf_line_ends_pass,
);
