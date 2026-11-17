//! Ticket 004-28 — the regime stamp sections added after the split: the
//! observed DSCP of peers and the alignment group. Port of
//! `checks/004-28.sh` (deleted with this port; `doccheck-equiv 004-28`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_28`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, listed in `c004_28/denials.rs` with the exact
//! criteria the old script fails on them (`CRASH` where it stops with a
//! Python exception); `doccheck-equiv 004-28` runs every denial against the
//! old script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-28.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed, the count at the merged
//! text (AC5). The script's DoD "names no ref containing main and runs no
//! other check" (it grepped its own text) is now the library-wide rule
//! (`hadronyche-tools/clippy.toml` and `doccheck::testing::source_rules`).
//!
//! Not denied: the budget's "fits with its arithmetic stated" branch is not
//! reachable from the current documents without rewriting the 004-25 and
//! 004-28 tables (the types need 904 sections, the overflow branch is the
//! live one); the "budget cannot be computed" assertion is denied with a
//! new type missing.
//!
//! Header and code: the header's AC4 lists "the outcomes Work 3 fixes";
//! the code checks the five categories and, for the DSCP category, only
//! when a DSCP field is wider than six bits (`DSCP_WIDE_FLAG`), as the port
//! does. The header's AC5 says "wire_tables 0 failed, at least the merged
//! count"; the code pins 12, kept in the repository-state file.

use hadronyche_tools::doccheck::checks::c004_28::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Two section types exist, each with a type code and a maximum size
/// ≤ 1 252 bytes (epic AC-3)." Script: the constants and the merged text
/// read; the two types (one subsection each) with sizes within the bound,
/// stated, codes in 004-28's block and not 004-25's, ⌈M / F⌉ statements,
/// exact arithmetic; the merged types need 7 sections; the budget: the
/// complete overflow report and the cap condition (review C2).
#[test]
fn ac1_two_section_types_with_codes_and_sizes() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The DSCP section covers both C-152 observations (hellos, every
/// module; sync requests, holder only) and, in a holder's stamp, the
/// per-follower dropped-request counter of C-204 with a width."
#[test]
fn ac2_the_dscp_section_covers_both_observations_and_the_counter() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The alignment section carries exactly the four lateness figures
/// of C-160 (rolling 60 s window, C-182), the maximum since live (C-182)
/// and the reclaimable latency of C-157, each with a width and a unit (epic
/// AC-8)." Also review C1, the no-sample value of the maximum since live.
#[test]
fn ac3_the_alignment_section_carries_the_figures() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least 5 invalid forms with outcomes." Script: in the two new
/// subsections, each naming control reasons, ReservedNonZero and BadLength
/// where required, the categories of Work 3; no form rejects the since-live
/// no-sample value.
#[test]
fn ac4_at_least_five_invalid_forms_with_outcomes() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "Passes the 004-02 checker." Script (document part): wire.toml
/// parses; one regime stamp message per new type, control at DSCP 40 with
/// the table's max_size, a non-empty list of control reasons, fields
/// agreeing row for row; one entry message per entry table, bit for bit.
#[test]
fn ac5_wire_toml_agrees_with_the_tables() {
    assert_criterion(&report(), "AC5");
}

/// DoD — evidence tags (subsection and prose paragraphs, policy
/// `PROSE_004_29`), open items with owner and falsifier, the merged tables
/// unchanged and the new reserved fields recorded, review C4 and the
/// hand-off on sender rule 3.
#[test]
fn dod_tags_open_items_and_merged_tables() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(
    SPEC;
    denial_wire_md_missing,
    denial_wire_toml_missing,
    ac1_denial_the_control_bound_constant,
    ac2_denial_the_followers_constant,
    ac2_denial_the_control_dscp_constant,
    ac3_denial_the_window_constant,
    ac2_denial_the_rate_constant,
    ac1_denial_the_placeholder,
    dod_denial_the_merged_header_changed,
    ac1_denial_the_section_count_not_64,
    dod_denial_a_rejection_value_changed,
    dod_denial_the_reserved_codes_row_removed,
    dod_denial_the_stamp_reserved_rows_removed,
    ac1_denial_no_code_block,
    dod_denial_a_merged_type_missing,
    ac1_denial_the_merged_types_need_more_sections,
    dod_denial_the_merged_health_entry_changed,
    ac1_denial_no_dscp_subsection,
    ac2_denial_no_dscp_section,
    ac1_denial_no_usable_alignment_table,
    ac3_denial_no_alignment_section,
    ac1_denial_a_type_over_the_bound,
    ac1_denial_the_size_not_stated,
    ac1_denial_a_code_of_004_25,
    ac1_denial_no_count_statement,
    ac1_denial_a_wrong_count_statement,
    ac1_denial_an_inexact_product,
    ac1_denial_the_report_without_the_total,
    ac1_denial_the_report_without_the_design_owner,
    ac1_denial_the_report_without_rule_7,
    ac1_denial_the_report_without_no_stamp,
    ac1_denial_the_report_without_the_threshold,
    ac1_denial_the_report_without_the_cap,
    dod_denial_the_report_untagged,
    ac1_denial_no_cap_condition,
    ac1_denial_the_cap_condition_without_its_width,
    ac1_denial_the_cap_condition_without_open_item_17,
    ac1_denial_the_cap_condition_without_006_19,
    ac1_denial_the_own_limit_not_exact,
    ac1_denial_no_alternative_entry_size,
    ac1_denial_no_budget_without_both_types,
    ac2_denial_a_narrow_peer_id,
    ac2_denial_the_hello_dscp_without_63,
    ac2_denial_the_sync_dscp_not_holder_only,
    ac2_denial_the_counter_without_its_rate,
    ac2_denial_the_counter_without_a_width,
    ac2_denial_the_counter_without_a_unit,
    ac2_denial_the_counter_with_two_rules,
    ac2_denial_a_state_field_without_four_states,
    ac2_denial_a_not_observed_code_inside_0_to_63,
    ac2_denial_no_dscp_0_to_63,
    ac2_denial_no_entry_order,
    ac2_denial_no_constant_for_the_count,
    ac2_denial_the_count_not_the_peers,
    dod_denial_no_sender_rule_3,
    ac3_denial_a_key_off_the_health_width,
    ac3_denial_a_figure_renamed,
    ac3_denial_a_figure_without_a_unit,
    ac3_denial_no_rolling_window,
    ac3_denial_the_since_live_maximum_without_a_unit,
    ac3_denial_the_reclaimable_latency_without_a_unit,
    ac3_denial_the_reclaimable_latency_unbounded,
    ac3_denial_the_count_without_its_constant,
    ac3_denial_no_split,
    ac3_denial_no_alignment_order,
    ac3_denial_no_no_sample_value,
    ac3_denial_the_no_sample_value_excluded,
    ac3_denial_the_order_rule_silent_on_no_sample,
    ac4_denial_a_form_rejecting_the_no_sample_value,
    ac4_denial_fewer_than_five_forms,
    ac4_denial_a_reason_outside_the_control_layer,
    ac4_denial_a_silent_form,
    ac4_denial_reserved_bits_without_reserved_non_zero,
    ac4_denial_a_length_form_without_bad_length,
    ac4_denial_a_count_form_without_bad_length,
    ac4_denial_no_dscp_form,
    dod_denial_an_untagged_subsection,
    dod_denial_an_untagged_paragraph,
    dod_denial_a_shared_address_without_its_source,
    dod_denial_an_open_item_without_an_owner,
    dod_denial_a_new_reserved_field_not_recorded,
    ac5_denial_wire_toml_does_not_parse,
    ac5_denial_a_multi_line_string_the_reader_rejects,
    ac5_denial_no_message_for_a_type,
    ac5_denial_a_max_size_off_the_table,
    ac5_denial_no_reasons,
    ac5_denial_fields_that_disagree,
    ac5_denial_an_entry_message_missing,
    ac5_denial_an_entry_message_that_disagrees,
    ac2_denial_a_heading_after_a_python_line_break,
    ac2_denial_a_word_character_only_python_knows,
    ac4_denial_a_validity_table_without_separator,
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
    num_hello_dscp_empty_width_cell,
    num_hello_dscp_row_cut_to_two_cells,
    num_sum_exponent_exact,
    num_sum_exponent_wrong,
    num_sum_leading_zero_syntax_error,
    num_sum_beyond_i128_exact,
    num_sum_negative_exponent,
    num_sum_result_beyond_i128,
    num_sum_product_beyond_i128,
    num_sum_superscript,
    num_count_statement_leading_zero,
    num_count_statement_big,
    num_count_statement_beyond_i128,
    num_count_statement_superscript,
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
    chk_hello_dscp_row_escaped_pipe,
    chk_hello_dscp_row_blank_line_inside,
    chk_hello_dscp_row_ragged_row,
    chk_hello_dscp_row_fenced_row,
    chk_hello_dscp_row_offset_double_zero_seven,
    chk_hello_dscp_row_offset_plus,
    chk_hello_dscp_row_offset_forty_five_digits,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (26fb108, "failing checks
/// for the observed DSCP and alignment group sections") fails AC1 to AC5,
/// as the script did there. A missing commit fails the test
/// (`DOCCHECK_ALLOW_SHALLOW=1` skips it visibly).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(&SPEC, "26fb108f8927395c36333f5c1a007e86b42cbd76", &["AC1", "AC2", "AC3", "AC4", "AC5"]);
}
