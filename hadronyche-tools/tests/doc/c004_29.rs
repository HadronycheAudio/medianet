//! Ticket 004-29 — the commissioning window announcement and close
//! messages. Port of `checks/004-29.sh` (deleted with this port;
//! `doccheck-equiv 004-29` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_29`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, listed in `c004_29/denials.rs` with the exact
//! criteria the old script fails on them (`CRASH` where it stops with a
//! Python exception); `doccheck-equiv 004-29` runs every denial against the
//! old script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-29.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed, the count at the merged
//! text (AC5). The script's DoD "names no ref containing main and runs no
//! other check" (it grepped its own text) is now the library-wide rule
//! (`hadronyche-tools/clippy.toml` and `doccheck::testing::source_rules`).
//!
//! Header and code: the header's AC4 says the control violations "are not
//! ignored"; the code's non-rejection pattern is `not a rejection|never a
//! rejection|\bignored\b|no reason` (without 004-24's `\bvalid\b`), and an
//! outcome "ignore and count under UnconfiguredDomain" is not matched by
//! `\bignored\b`; the port follows the code. The placeholder of the section
//! is reported under DoD, as the code does (the header lists it under the
//! author contract).

use hadronyche_tools::doccheck::checks::c004_29::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Announcement and close each have a type code, a field table and a
/// maximum size ≤ 1 252 bytes (epic AC-3)." Script: one subsection each;
/// a field table starting with the Hello's common control header, tiling,
/// its size stated and within the bound, a Message type code in this
/// ticket's block, distinct; the window identifier, the tool identity, the
/// close's single identifier.
#[test]
fn ac1_both_messages_have_a_code_a_table_and_a_size() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The expiry time has a width, a unit and a clock (epic AC-8,
/// C-153)." Script: widths equal to the merged stamp's, a unit, one form,
/// the clock named from timeline.md's terms (network time), and for a
/// relative form its origin, its repeat rule and the merged stamp updated.
#[test]
fn ac2_the_expiry_has_a_width_a_unit_and_a_clock() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The open, expiry and explicit-close transitions of C-153 appear
/// as receiver rules." Script: three different numbered rules; the second
/// announcement's outcome; reviews F2 and F7.
#[test]
fn ac3_the_three_transitions_are_receiver_rules() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least 6 invalid forms with outcomes across the two messages."
/// Script: the control violations name their merged reason and are not
/// ignored; the three cases of the ticket; review F3.
#[test]
fn ac4_at_least_six_invalid_forms_with_outcomes() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "Passes the 004-02 checker." Script (document part): one message
/// per table in wire.toml, control at DSCP 40 with the table's size, a
/// non-empty list of control reasons, fields row for row.
#[test]
fn ac5_wire_toml_agrees_with_the_tables() {
    assert_criterion(&report(), "AC5");
}

/// DoD — evidence tags (policy `PROSE_004_29`), open items with owner and
/// falsifier (`OpenItemPolicy::Leading`), merged tables unchanged, reserved
/// fields recorded, Open item 11 decided, review F1.
#[test]
fn dod_tags_open_items_merged_tables_and_open_item_11() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(
    SPEC;
    denial_wire_md_missing,
    denial_wire_toml_missing,
    ac1_denial_the_control_bound_constant,
    ac1_denial_no_control_bound_where_the_script_stops,
    ac2_denial_the_control_dscp_constant,
    dod_denial_the_hello_header_changed,
    dod_denial_the_code_column_changed,
    ac1_denial_no_code_block,
    ac2_denial_the_stamp_expiry_renamed,
    dod_denial_the_enumeration_short,
    dod_denial_the_placeholder,
    ac1_denial_no_announcement_subsection,
    ac1_denial_no_close_subsection,
    ac3_denial_no_receiver_rules_subsection,
    ac1_denial_no_announcement_table,
    ac1_denial_a_malformed_row,
    ac1_denial_the_header_rows_changed,
    ac1_denial_rows_that_do_not_tile,
    ac1_denial_the_size_not_stated,
    ac1_denial_a_code_outside_the_block,
    ac1_denial_two_messages_one_code,
    ac1_denial_no_window_identifier_row,
    ac1_denial_no_tool_identity_row,
    ac1_denial_the_identifier_not_stable,
    ac2_denial_a_window_identifier_off_the_stamp_width,
    ac2_denial_an_expiry_off_the_stamp_width,
    ac2_denial_no_clock,
    ac2_denial_the_expiry_without_a_unit,
    ac2_denial_the_expiry_in_two_forms,
    ac2_denial_the_form_not_in_a_tagged_sentence,
    ac2_denial_a_relative_expiry_without_its_origin,
    ac2_denial_a_relative_expiry_without_the_repeat_rule,
    ac2_denial_a_relative_expiry_copied_verbatim,
    dod_denial_a_relative_expiry_without_the_status,
    ac1_denial_no_close_identifier,
    ac2_denial_the_close_identifier_off_the_stamp_width,
    ac1_denial_the_close_with_a_second_field,
    ac4_denial_fewer_than_six_forms,
    ac4_denial_a_reason_outside_the_control_layer,
    ac4_denial_a_violation_without_its_reason,
    ac4_denial_a_violation_marked_ignored,
    ac4_denial_the_version_mismatch_without_its_event,
    ac4_denial_a_form_without_an_outcome,
    ac4_denial_no_expiry_out_of_range_form,
    ac4_denial_no_expiry_past_form,
    ac4_denial_no_unknown_close_form,
    ac3_denial_no_second_announcement_outcome,
    ac3_denial_no_open_rule,
    ac3_denial_no_expiry_rule,
    ac3_denial_no_explicit_close_rule,
    ac3_denial_two_transitions_in_one_rule,
    dod_denial_an_untagged_subsection,
    dod_denial_an_untagged_paragraph,
    dod_denial_an_open_item_without_an_owner,
    dod_denial_a_reserved_field_not_recorded,
    dod_denial_open_item_11_still_open,
    ac4_denial_no_outcome_after_an_explicit_close,
    dod_denial_a_reopening_without_an_open_item,
    ac3_denial_two_windows_not_stated,
    ac3_denial_a_lost_close_not_stated,
    dod_denial_a_bare_liveness_denial,
    dod_denial_a_section_type_without_the_cap,
    ac5_denial_wire_toml_does_not_parse,
    ac5_denial_a_multi_line_string_the_reader_rejects,
    ac5_denial_no_announcement_message,
    ac5_denial_a_max_size_off_the_table,
    ac5_denial_no_reasons,
    ac5_denial_fields_that_disagree,
    ac1_denial_a_heading_after_a_python_line_break,
    ac1_denial_a_word_character_only_python_knows,
    ac4_denial_a_validity_table_without_separator,
    accepted_a_separator_row_only_the_scripts_accept,
    num_c17_ten_to_19,
    num_c17_ten_to_20,
    num_c17_ten_to_36,
    num_c17_beyond_i128,
    num_c17_leading_zero,
    num_c17_leading_plus,
    num_c17_superscript_after,
    num_c17_superscript_before,
    num_c17_underscore,
    num_enum_big_value,
    num_enum_beyond_i128,
    num_enum_leading_zeros,
    num_enum_empty_value,
    num_enum_superscript,
    num_enum_duplicate_row,
    num_enum_duplicate_row_same_value,
    num_enum_missing_layers_cell,
    num_toml_dscp_underscore,
    num_toml_dscp_string,
    num_toml_dscp_padded_signed_string,
    num_toml_dscp_bad_string,
    num_toml_dscp_superscript_string,
    num_toml_dscp_big,
    num_toml_dscp_beyond_i128,
    num_toml_dscp_float,
    num_toml_dscp_plus,
    num_toml_max_size_float,
    num_toml_max_size_string,
    num_toml_max_size_bool,
    num_toml_max_size_bad_string,
    num_tool_empty_width_cell,
    num_tool_row_cut_to_two_cells,
    num_size_superscript_before,
    num_size_superscript_after,
    num_size_leading_zero,
    num_size_big,
    num_size_beyond_i128,
    num_type_code_leading_zero,
    num_type_code_big,
    num_type_code_beyond_i128,
    num_type_code_superscript,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (0243573, "failing checks
/// for the commissioning window messages") fails the criteria the script
/// failed there. A missing commit fails the test
/// (`DOCCHECK_ALLOW_SHALLOW=1` skips it visibly).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(&SPEC, "0243573af47644bdbf800423bebe9d6a52980d79", &["AC1", "AC3", "AC4", "AC5", "DoD"]);
}
