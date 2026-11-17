//! Ticket 004-10 — media measurement extension layout. Port of
//! `checks/004-10.sh` (deleted with this port; `doccheck-equiv 004-10` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_10`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on the denial table below:
//! one row per assertion of the script, plus rows aimed at the Python
//! semantics its code touches. Every row is also run against the old script
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-10.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! stays 12 passed, 0 failed, 0 ignored (AC5); the design checker prints OK
//! (DoD).
//!
//! Header and code disagree: the header says the outcomes are members of
//! "the closed 13-name enumeration"; the code replaces the 13 names with
//! those found in the Rejection reasons tables when at least 10 are found
//! (004-10:605-606). The port follows the code
//! (`ten_found_names_shrink_the_enumeration`).
//!
//! The script crashes (red, no FAIL line) when an extension row has no
//! offset digit and wire.toml carries all three fields: `min()` of nothing
//! or `None - int` at 004-10:1078-1081 (`py_missing_offset_crashes`).

use hadronyche_tools::doccheck::checks::c004_10::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- criteria

/// AC1 — "Three fields (hop count, accumulated residence time, accumulated
/// declared delay), each with a width, a unit and a saturation rule; the
/// declared delay in whole samples (C-105); the forwarding update rule; the
/// open item on the widths with its owner and falsifier."
#[test]
fn ac1_three_fields_with_width_unit_and_saturation() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The extension's byte size is stated and fits the C-137 header
/// budget." Script: a number equal to the widths, contiguous fields, the
/// arithmetic 100 − 38 = 62 and 62 − 49 = 13, and the checksum remainder
/// 13 − size (004-13), or the fallback omission when it does not fit.
#[test]
fn ac2_byte_size_and_the_c137_arithmetic() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "At least 3 invalid forms with outcomes." Script: flag set with
/// the bytes missing (TooShort or BadLength), hop count beyond the maximum
/// (BadLength), reserved bits set (ReservedNonZero); no reason outside the
/// enumeration in the units or the prose.
#[test]
fn ac3_three_invalid_forms_with_member_outcomes() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "No field identifies a receiver or destination (rule 8)."
#[test]
fn ac4_no_receiver_or_destination_field() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "The machine-readable table passes the 004-02 checker." Script
/// (document part): the three fields in wire.toml (media or section form)
/// agreeing with wire.md, the max_size invariant, members-only reasons with
/// BadLength and ReservedNonZero.
#[test]
fn ac5_wire_toml_carries_the_extension() {
    assert_criterion(&report(), "AC5");
}

/// DoD — every block of the section carries an evidence tag (tables and
/// code blocks inherit).
#[test]
fn dod_every_block_carries_an_evidence_tag() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (3a847e2, "failing checks
/// for the measurement extension layout") fails as the script did.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "3a847e27c573bf77fcf9923d46d2bc062ad9aa7f",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "DoD"],
    );
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC;
    setup_wire_md_missing,
    setup_no_section,
    setup_placeholder,
    setup_no_field_table,
    ac1_no_delay_row,
    ac1_width_not_whole_bytes,
    ac1_type_not_unsigned,
    ac1_width_disagrees_with_type,
    ac1_no_saturation_rule,
    ac1_hop_no_maximum,
    ac1_hop_above_64,
    ac1_hop_does_not_fit,
    ac1_residence_unit_untagged,
    ac1_delay_not_whole_samples,
    ac1_no_forwarding_rule,
    ac1_no_open_item,
    ac1_falsifier_not_the_tickets,
    ac2_size_not_stated,
    ac2_size_not_the_widths,
    ac2_not_contiguous,
    ac2_no_c137_arithmetic,
    ac2_no_checksum_remainder,
    ac2_fit_not_said,
    ac2_too_big_without_fallback,
    ac3_a_form_missing,
    ac3_a_form_without_a_member,
    ac3_a_form_under_the_wrong_reason,
    ac3_flag_form_under_the_wrong_reason,
    ac3_a_reason_outside_the_enumeration,
    ac3_fewer_than_three,
    ac3_headerless_validity_run_is_dropped,
    ac4_no_rule_8_statement,
    ac5_wire_toml_missing,
    ac5_wire_toml_unreadable,
    ac5_no_container,
    ac5_no_extension_fields,
    ac5_a_field_missing,
    ac5_width_not_an_integer,
    ac5_width_not_whole_bytes,
    ac5_offset_disagrees,
    ac5_width_disagrees,
    ac5_type_disagrees,
    ac5_total_disagrees_with_the_stated_size,
    ac5_max_size_not_an_integer,
    ac5_media_max_size_too_small,
    ac5_max_size_above_the_cap,
    ac5_no_reasons,
    ac5_a_reason_not_a_member,
    ac5_reserved_non_zero_missing,
    dod_an_untagged_paragraph,
    py_superscript_is_a_word_character,
    py_a_combining_mark_is_not_a_word_character,
    py_unit_separator_is_whitespace_to_lstrip,
    py_line_separator_starts_a_line,
    py_form_feed_starts_a_line,
    py_next_line_starts_a_line,
    py_bare_carriage_return_is_a_newline,
    py_dotless_i_matches_i_under_re_i,
    py_int_is_unbounded,
    py_missing_offset_crashes,
    py_separator_without_dash_is_no_table,
    toml_float_width_is_not_an_integer,
    toml_duplicate_key_last_wins,
    toml_message_a_number_crashes,
    toml_unhashable_reason_crashes,
    ac2_no_offset,
    ac5_section_form_passes,
    ac5_section_form_without_section_of,
    ac5_section_form_max_size_not_the_bytes,
    ac5_section_form_media_too_small,
    ac5_section_form_media_above_the_cap,
    toml_bare_word_section_of,
    ten_found_names_shrink_the_enumeration,
    nine_found_names_keep_the_thirteen,
);
