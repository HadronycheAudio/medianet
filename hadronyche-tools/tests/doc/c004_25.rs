//! Ticket 004-25 — the regime stamp section types the constants table
//! names. Port of `checks/004-25.sh` (deleted with this port;
//! `doccheck-equiv 004-25` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_25`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, listed in `c004_25/denials.rs` with the exact
//! criteria the old script fails on them (`CRASH` where it stops with a
//! Python exception); `doccheck-equiv 004-25` runs every denial against the
//! old script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-25.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC6); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed (AC6). The script's DoD
//! "names no ref containing main and runs no other check" (it grepped its
//! own text) is now the library-wide rule (`hadronyche-tools/clippy.toml`
//! and `doccheck::testing::source_rules`).
//!
//! Header and code: the header's "Declared limits" say the capability
//! vocabulary of review C2 "is the list in review.py"; no review.py exists
//! in the repository and the code holds the list inline (`CAP_VOCAB`); the
//! port follows the code. The header lists the merged-text and review
//! assertions under DoD, as the code reports them.

use hadronyche_tools::doccheck::checks::c004_25::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

/// Panics when a failure of `criterion` contains `needle`.
fn assert_none(r: &Report, criterion: &str, needle: &str) {
    let hits: Vec<String> = r
        .of(criterion)
        .iter()
        .filter(|f| f.message.contains(needle))
        .map(|f| f.to_string())
        .collect();
    assert!(hits.is_empty(), "{}", hits.join("\n"));
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Seven section types exist, one per item in Context, each with a
/// type code and a maximum size ≤ 1 252 bytes."
///
/// Script: seven types, one subsection each; each type's field table and
/// its entry tables (depth-first) tile, each entry table sums to its list
/// row's width, the section at its maxima is within 1 252 bytes and is
/// stated; ⌈M / F⌉ = N statements; a type code inside the block of 004-25,
/// all distinct; the Context values; every product and sum exact; the
/// 64-section bound fits or is reported with Open item 1; the merged
/// section count and code block.
#[test]
fn ac1_seven_section_types_with_codes_and_sizes() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The capability-state enumeration has exactly the four values of
/// C-129; the timestamp-source enumeration has exactly the four values of
/// C-131."
#[test]
fn ac2_the_two_enumerations_have_exactly_four_values() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The health section has the eleven C-159 counters plus the
/// observed DSCP." Script: each counter its own row (a maximum matching),
/// each with a wrap or saturation rule and a unit; the observed DSCP.
#[test]
fn ac3_the_health_section_has_the_eleven_counters_and_the_dscp() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "The capability section includes control isolation (C-161) and
/// the injection section includes the panic hook (C-145)."
#[test]
fn ac4_control_isolation_and_the_panic_hook() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "At least 5 invalid forms with outcomes." Script: at least five
/// rows in the tables of invalid forms other than the merged one, each
/// naming control reasons (or silent only for a repeat, an unknown code or
/// an older version), ReservedNonZero and BadLength where required, and the
/// four categories.
#[test]
fn ac5_at_least_five_invalid_forms_with_outcomes() {
    assert_criterion(&report(), "AC5");
}

/// AC6 — "Passes the 004-02 checker." Script (document part): wire.toml
/// parses; the merged section message is unchanged; one regime stamp
/// message per type, control at DSCP 40 with the table's max_size, control
/// reasons only, fields agreeing row for row; one entry message per entry
/// table, bit for bit. The checker run is in scripts/repo-state-checks.sh.
#[test]
fn ac6_wire_toml_agrees_with_the_tables() {
    assert_criterion(&report(), "AC6");
}

/// DoD — "Every new factual claim carries an evidence tag": each type
/// subsection and each of its prose paragraphs of 5+ words (policy
/// `PROSE_004_29`).
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(&report(), "DoD", "evidence tag");
}

/// DoD — "No new open item without an owner and a falsifier" (policy
/// `MENTION_BLOCKS_004_24`).
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(&report(), "DoD", "Open item paragraph");
}

/// DoD — "Regime stamp fields touched are complete and documented": the
/// merged header rows, the Rejection reasons (13 names and values), the
/// Reserved fields rows; the review findings C1 to C4.
#[test]
fn dod_merged_text_unchanged_and_review_findings() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(
    SPEC;
    denial_wire_md_missing,
    denial_wire_toml_missing,
    ac1_denial_the_control_bound_constant,
    ac3_denial_c159_not_eleven,
    ac2_denial_c129_not_four,
    ac2_denial_c131_not_four,
    ac1_denial_the_placeholder,
    dod_denial_the_merged_header_changed,
    ac1_denial_the_section_count_not_64,
    ac1_denial_no_code_block,
    dod_denial_a_rejection_value_changed,
    dod_denial_the_reserved_codes_row_removed,
    ac1_denial_two_subsections_name_one_type,
    ac1_denial_a_type_missing,
    ac1_denial_a_malformed_type_row,
    ac1_denial_a_type_without_field_table,
    ac1_denial_an_unused_field_table,
    ac1_denial_an_offset_neither_0_nor_58,
    ac1_denial_rows_that_do_not_tile,
    ac1_denial_an_entry_table_off_its_width,
    ac1_denial_a_type_over_the_bound,
    ac1_denial_no_section_count_statement,
    ac1_denial_a_wrong_section_count_statement,
    ac1_denial_the_size_not_stated,
    ac1_denial_a_type_code_outside_the_block,
    ac1_denial_two_types_share_a_code,
    ac1_denial_an_inexact_product,
    ac1_denial_an_inexact_sum,
    ac1_denial_the_excess_not_reported,
    ac1_denial_no_stream_in_the_multi_packet_section,
    ac1_denial_no_expiry_time,
    ac1_denial_no_time_per_operation,
    ac1_denial_no_epoch_in_the_injections_section,
    ac4_denial_no_panic_hook,
    ac4_denial_no_control_isolation,
    ac1_denial_the_timestamp_source_without_a_state,
    ac2_denial_a_fifth_capability_state,
    ac2_denial_a_timestamp_source_renamed,
    ac3_denial_a_counter_without_its_row,
    ac3_denial_a_counter_without_its_wrap_rule,
    ac3_denial_a_counter_without_its_unit,
    ac3_denial_no_observed_dscp,
    ac5_denial_fewer_than_five_forms,
    ac5_denial_a_reason_outside_the_control_layer,
    ac5_denial_a_silent_form,
    ac5_denial_reserved_bits_without_reserved_non_zero,
    ac5_denial_a_length_form_without_bad_length,
    ac5_denial_a_count_form_without_bad_length,
    ac5_denial_no_length_form,
    dod_denial_reserved_fields_not_recorded,
    dod_denial_an_untagged_subsection,
    dod_denial_an_untagged_paragraph,
    dod_denial_an_open_item_without_an_owner,
    ac6_denial_wire_toml_does_not_parse,
    ac6_denial_the_merged_message_changed,
    ac6_denial_no_message_for_a_type,
    ac6_denial_a_max_size_off_the_table,
    ac6_denial_a_reason_outside_the_control_layer,
    ac6_denial_fields_that_disagree,
    ac6_denial_an_entry_message_missing,
    ac6_denial_an_entry_message_that_disagrees,
    dod_denial_sender_rule_7_not_quoted,
    dod_denial_the_health_maximum_off_the_constants,
    dod_denial_the_product_without_its_constants,
    dod_denial_the_report_without_the_total,
    dod_denial_the_report_without_the_design_owner,
    dod_denial_the_report_without_rule_7,
    dod_denial_the_report_without_no_stamp,
    dod_denial_the_report_without_the_threshold,
    dod_denial_the_report_untagged,
    dod_denial_a_capability_named_without_its_row,
    dod_denial_the_stated_capability_count_wrong,
    dod_denial_a_manifest_item_handed_back,
    dod_denial_no_admission_marker,
    ac6_denial_a_multi_line_string_the_reader_rejects,
    ac1_denial_a_heading_after_a_python_line_break,
    dod_denial_a_word_character_only_python_knows,
    ac2_denial_a_state_table_without_separator,
    accepted_a_separator_row_only_the_scripts_accept,
    ac1_denial_no_control_bound_where_the_script_stops,
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
    num_toml_reasons_unhashable,
    num_toml_reasons_an_integer,
    num_calibration_empty_width_cell,
    num_calibration_row_cut_to_two_cells,
    num_sum_exponent_exact,
    num_sum_exponent_wrong,
    num_sum_leading_zero_syntax_error,
    num_sum_beyond_i128_exact,
    num_sum_negative_exponent,
    num_sum_big_exact,
    num_sum_result_beyond_i128,
    num_sum_superscript,
    num_section_count_leading_zero,
    num_section_count_big,
    num_section_count_beyond_i128,
    num_section_count_superscript,
    chk_c17_zero,
    chk_c17_double_zero_seven,
    chk_c17_two_to_63,
    chk_c17_forty_five_digits,
    chk_c17_micro_before,
    chk_c17_micro_after,
    chk_enum_swap,
    chk_enum_tie,
    chk_enum_twelve_members,
    chk_enum_fourteen_members,
    chk_enum_value_two_to_63,
    chk_enum_value_micro,
    chk_enum_escaped_pipe,
    chk_calibration_row_escaped_pipe,
    chk_calibration_row_blank_line_inside,
    chk_calibration_row_ragged_row,
    chk_calibration_row_fenced_row,
    chk_calibration_row_offset_double_zero_seven,
    chk_calibration_row_offset_plus,
    chk_calibration_row_offset_forty_five_digits,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (e4670d4, "failing checks
/// for the regime stamp section types") fails AC1 and AC5, as the script
/// did there (the section types were not laid out; the other criteria's
/// assertions are guarded by the types being found). A missing commit
/// fails the test (`DOCCHECK_ALLOW_SHALLOW=1` skips it visibly).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(&SPEC, "e4670d418ae6eb4d0995f95690f412be8b0951c2", &["AC1", "AC5"]);
}
