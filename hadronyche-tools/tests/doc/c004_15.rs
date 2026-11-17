//! Ticket 004-15 — recompute capacity figures from the final layout. Port of
//! `checks/004-15.sh` (deleted with this port; `doccheck-equiv 004-15` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_15`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory. Every denial is also run
//! against the old script by the harness (`doccheck-equiv 004-15
//! --denials-only`).
//!
//! The script has no repository-level assertion (it reads documents only),
//! so nothing of it is in `scripts/repo-state-checks.sh`.
//!
//! Two parts of the script cannot be driven to a failure by a document:
//! the toml reader's uncaught error for `[[x]]` after `[x]` (the port
//! reports `CRASH`), and the `FAIL AC 3` for a constants table that is
//! updated while the section still says the constants stand is reachable
//! only together with the other exceed-branch failures.

use hadronyche_tools::doccheck::checks::c004_15::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::assert_criterion;

/// AC1 — "`wire.md` states the new wire size per default packet and bit rate
/// per stream with the arithmetic shown."
///
/// Script: every equation of the subsection evaluates; the default packet's
/// on-wire size is UDP payload + 66 (the 004-14 default total); the four
/// Ethernet parts and their total 38 stand beside their names; the rate per
/// stream (wire x 8 x 2 000), per channel (/ 6) and per source for 24, 48 and
/// 96 channels with C-22's figures; every row of the wire-size table
/// recomputes.
#[test]
fn ac1_the_wire_size_and_the_bit_rates_with_their_arithmetic() {
    assert_criterion(&report_of(&SPEC), "AC1");
}

/// AC2 — "The difference from 1 320 bytes / 21.1 Mbit/s is stated in absolute
/// and percent, and the per-source rates are compared with C-22."
///
/// Script: one unit with C-20, the absolute byte difference and the percent
/// (within 0.1 point); one with C-21 (21.1), the rate difference and its
/// percent; every difference and percent against C-22 evaluates; the share
/// against C-21 x streams is the per-stream share; a per-channel figure is a
/// computed one.
#[test]
fn ac2_the_differences_in_absolute_and_percent() {
    assert_criterion(&report_of(&SPEC), "AC2");
}

/// AC3 — "If > 5 %, the constants table is updated (or an open item with
/// owner and falsifier is recorded)."
///
/// Script: a sentence states the 5 % rule with a direction that matches the
/// computed difference; within 5 %, a sentence says no constant is
/// recomputed; over it, C-20, C-21, C-22, C-24, C-25, C-26, C-90, C-92 and
/// C-118 are named and the table is updated or an open item with owner and
/// falsifier names C-20.
#[test]
fn ac3_the_five_percent_rule() {
    assert_criterion(&report_of(&SPEC), "AC3");
}

/// AC4 — "`wire.md` states the controller-stream wire cost of C-219 as a
/// formula in controller bytes per slice, at 2 000 packets/s, with the 66
/// bytes of overhead shown."
///
/// Script: the 28 + 38 and the 66, a symbolic formula, 2 000 packets/s, a
/// bit-rate formula, the header term as the cap; worked examples
/// `H + c + 28 + 38 = N` and `N x 8 x 2 000 = R` that evaluate, with a controller
/// example.
#[test]
fn ac4_the_controller_stream_wire_cost() {
    assert_criterion(&report_of(&SPEC), "AC4");
}

/// AC5 (the script's) — optional toml keys that name a wire size, rate or
/// overhead agree with the computed value; an audio declared rate is the
/// cap form (cap + audio + 66 bytes). The ticket has four criteria; the
/// script's AC 5 is its own addition.
#[test]
fn ac5_toml_keys_and_the_declared_rate() {
    assert_criterion(&report_of(&SPEC), "AC5");
}

/// DoD — "Every new factual claim carries an evidence tag" and "No new open
/// item without an owner and a falsifier": prose of 5 or more
/// `[A-Za-z0-9]+` words carries a tag, open items name an owner and a
/// falsifier, and every Ethernet framing unit carries `[memory]` (with a
/// checking task) or `[standard]` (with a clause) (C-132).
#[test]
fn dod_evidence_tags_open_items_and_the_framing_tags() {
    assert_criterion(&report_of(&SPEC), "DoD");
}

/// DoD — "Design documents updated where behaviour differs": the document
/// side is setup (the files read, the constants and toml tables found).
#[test]
fn dod_the_inputs_read() {
    let r = report_of(&SPEC);
    assert!(r.of("setup").is_empty(), "{}", r.render());
}

// ---------------------------------------------------------------- denials

/// The tree at the commit that added the script fails the criterion the
/// script failed: AC1 (its subsection did not exist).
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(&SPEC, "d3e5ea6b1ef0ca0f96089a8c068c7afb34d3ed38", &["AC1"]);
}

hadronyche_tools::denial_tests!(
    SPEC;
    a_c20_of_0,
    a_c20_of_007,
    a_c20_of_plus7,
    a_c20_of_1_2,
    a_c20_of_2e63,
    a_c20_of_20_digits,
    a_c20_of_45_digits,
    a_c20_of_1e13,
    a_c20_of_1e19,
    a_c20_of_1e36,
    a_c20_of_superscript,
    a_c20_of_micro,
    a_payload_of_1e13,
    a_payload_of_1e19,
    a_payload_of_1e36,
    a_payload_of_45_digits,
    ac3_denial_over_five_percent_with_a_constant_not_named,
    denial_wire_md_missing,
    denial_wire_toml_missing,
    denial_constants_missing,
    denial_a_constant_row_missing,
    denial_c22_does_not_parse,
    denial_wire_toml_does_not_parse,
    denial_the_toml_lacks_the_layout_tables,
    ac1_denial_a_budget_default_total_that_differs,
    ac1_denial_no_wire_size_heading,
    ac1_denial_an_equation_that_does_not_hold,
    ac1_denial_the_default_wire_size_stated_wrongly,
    ac1_denial_no_arithmetic_for_the_default_wire_size,
    ac1_denial_the_ethernet_header_not_beside_its_name,
    ac1_denial_the_fcs_not_beside_its_name,
    ac1_denial_the_preamble_not_beside_its_name,
    ac1_denial_the_gap_not_beside_its_name,
    ac1_denial_the_ethernet_total_not_stated,
    ac1_denial_no_rate_per_stream,
    ac1_denial_no_rate_per_channel,
    ac1_denial_no_unit_for_a_source,
    ac1_denial_a_wire_size_row_without_a_column,
    ac1_denial_a_wire_size_row_without_arithmetic,
    ac1_denial_a_wire_size_row_with_the_wrong_payload,
    ac1_denial_a_wire_size_row_not_payload_plus_66,
    ac1_denial_a_wire_size_row_without_a_rate,
    ac1_denial_a_wire_size_row_with_the_wrong_rate,
    ac1_denial_a_wire_size_row_with_the_wrong_bit_rate_equation,
    ac1_denial_a_wire_size_row_without_a_per_channel_rate,
    ac1_denial_a_wire_size_row_with_the_wrong_per_channel_rate,
    ac2_denial_no_byte_difference,
    ac2_denial_a_percent_beside_c20_that_is_not_the_share,
    ac2_denial_a_percent_beside_c21_that_is_not_the_share,
    ac2_denial_no_rate_difference,
    ac2_denial_a_c22_difference_that_is_not_computed,
    ac2_denial_a_c22_share_that_is_not_computed,
    ac2_denial_a_share_against_c21_times_streams_that_is_not_the_per_stream_share,
    ac2_denial_a_plain_share_against_c21_times_streams,
    ac2_denial_a_per_channel_figure_that_is_not_computed,
    ac3_denial_no_five_percent_sentence,
    ac3_denial_a_five_percent_sentence_without_a_direction,
    ac3_denial_a_direction_that_contradicts_the_computed_difference,
    ac3_denial_within_five_percent_without_the_constants_standing,
    ac3_denial_over_five_percent_but_the_section_says_the_constants_stand,
    ac3_denial_over_five_percent_with_neither_update_nor_open_item,
    ac4_denial_no_controller_cost_paragraph,
    ac4_denial_the_overhead_not_shown_as_28_plus_38,
    ac4_denial_no_symbolic_formula,
    ac4_denial_not_stated_at_2000_packets_a_second,
    ac4_denial_no_bit_rate_formula,
    ac4_denial_the_header_term_not_named_as_the_cap,
    ac4_denial_a_worked_example_whose_header_term_is_not_a_budget_figure,
    ac4_denial_no_worked_example,
    ac4_denial_no_controller_example,
    ac4_denial_no_bit_rate_example,
    ac4_denial_a_bit_rate_example_that_does_not_use_a_controller_total,
    ac5_denial_an_audio_declared_rate_the_layout_does_not_compute,
    ac5_denial_an_audio_declared_rate_that_is_not_the_cap_form,
    ac5_denial_an_audio_declared_rate_with_other_bytes,
    ac5_denial_an_overhead_key_that_differs,
    ac5_denial_an_ethernet_key_that_differs,
    ac5_denial_an_ipv4_header_key_that_differs,
    ac5_denial_a_udp_header_key_that_differs,
    ac5_denial_a_default_wire_key_that_differs,
    ac5_denial_a_maximum_wire_key_that_differs,
    ac5_denial_a_stream_rate_key_that_differs,
    dod_denial_an_untagged_paragraph,
    dod_denial_an_open_item_without_an_owner,
    dod_denial_a_framing_row_without_a_tag,
    dod_denial_a_memory_framing_row_without_a_checking_task,
    dod_denial_a_standard_framing_row_without_a_clause,
    dod_denial_a_standard_framing_sentence_without_a_clause,
    c21_with_a_superscript_after_its_number,
    a_difference_written_with_a_superscript_minus,
    a_percent_just_inside_the_tolerance,
    a_percent_just_outside_the_tolerance,
    an_equation_exactly_on_its_tolerance,
    an_equation_just_inside_its_tolerance,
    an_equation_just_outside_its_tolerance,
    an_equation_beyond_float_precision_fails,
    an_equation_beyond_float_precision_holds,
    an_equation_beyond_i128,
    a_result_with_a_zero_decimal_passes,
    a_result_with_a_decimal_part_fails,
    a_payload_with_an_underscore,
    a_c20_beyond_i128,
    a_c21_of_zero_crashes,
    a_c04_of_zero_crashes,
    a_toml_value_with_trailing_tabs_and_a_comment_passes,
    a_field_before_any_message,
    an_array_table_after_a_table_crashes,
    a_max_size_beyond_i128,
    fields_that_is_a_list_of_an_int_crashes,
    crlf_line_ends_pass,
);
