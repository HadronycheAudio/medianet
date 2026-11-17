//! Ticket 004-26 — the manifest encoding in the admission handshake. Port
//! of `checks/004-26.sh` (deleted with this port; `doccheck-equiv 004-26`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_26`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, listed in `c004_26/denials.rs` with the exact
//! criteria the old script fails on them (`CRASH` where it stops with a
//! Python exception); `doccheck-equiv 004-26` runs every denial against the
//! old script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-26.sh`): `cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` exits
//! 0 with 0 failed and at least 12 passed. The script's DoD "names no ref
//! containing main and runs no other check" (it grepped its own text) is
//! now the library-wide rule (`hadronyche-tools/clippy.toml` and
//! `doccheck::testing::source_rules`).
//!
//! Assertions no denial can fail (the script's own selection makes them
//! hold, so they are carried by the port and not denied): "the {none,
//! unknown} sentinel N fits the B-bit E field" and "the {none, unknown}
//! sentinel N lies outside its stated E range" — `encoding_of` only takes a
//! sentinel that fits its field and lies outside the row's range; "the
//! maximum manifest size, N bytes, fits the control bound" is a
//! `check(True)`.
//!
//! Header and code: the header's AC5 says "every outcome names a member"
//! of the Reject or refusal enumeration; the code skips the identical
//! repeat row (its outcome names no reason, checked separately), as the
//! port does. The header's DoD says the landed tables are checked "in the
//! way 004-18's check verifies them"; the code compares the first seven
//! rows of four admission tables with the Hello's common control header,
//! as the port does.

use hadronyche_tools::doccheck::checks::c004_26::SPEC;
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

/// AC1 — "Every item in Context has a field with width and unit."
/// Script: ports, content types, the channel map, L in whole samples, the
/// join-delay count and each join delay with its frequency flag, the dense
/// initial values, the conversion delay of a device-facing port with its
/// unit, the cost bound in operations per slice, required and declared E
/// with the unit of E; the tables model and tile; review points 4, T3 (a)
/// and T3 (f).
#[test]
fn ac1_every_context_item_has_a_field_with_width_and_unit() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "'None' for required E and 'unknown' for declared E have distinct
/// encodings that no valid E value can take." Script: a sentinel outside
/// the stated E range or a flag field, per encoding; distinct; review T3
/// (b), the prose and wire.toml state the table's codes.
#[test]
fn ac2_none_and_unknown_have_distinct_encodings() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The maximum manifest size is stated in bytes and fits 1 252
/// bytes, or a falsified constant is reported (epic AC-3)." Script: the
/// constants and the landed envelope read; the computed size of the table
/// no counted row references is stated; the sections it needs fit the
/// landed count; above the bound, the report paragraph affirms the
/// falsification of AC-3 and 012-08, states the size, the excess, the
/// design owner and the split (review T1).
#[test]
fn ac3_the_maximum_manifest_size_is_stated_and_reported() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "The join-delay bound is recorded as an open item with owner and
/// falsifier." Script: an open item on the join bound, owned by epic 004,
/// saying the second clause of the falsifier is already met.
#[test]
fn ac4_the_join_delay_bound_is_an_open_item() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "At least 5 invalid forms with outcomes." Script: five rows, each
/// naming a member of the Reject or refusal enumeration, the five
/// categories of Work item 3, the repeat and disagreement rows.
#[test]
fn ac5_at_least_five_invalid_forms_with_outcomes() {
    assert_criterion(&report(), "AC5");
}

/// AC6 — "Passes the 004-02 checker." Script (document part): wire.toml
/// parses; each field table within the bound has an agreeing message; the
/// fixed part is declared row for row (review T2). The checker run is in
/// scripts/repo-state.d/004-26.sh.
#[test]
fn ac6_wire_toml_agrees_with_the_tables() {
    assert_criterion(&report(), "AC6");
}

/// DoD — "Every new factual claim carries an evidence tag" (policy
/// `PROSE_004_26`, and every subsection with a table) and "No new open item
/// without an owner and a falsifier"; the landed tables unchanged; the
/// review points labelled DoD.
#[test]
fn dod_tags_open_items_landed_tables_and_review_points() {
    assert_criterion(&report(), "DoD");
}

/// DoD — "The machine-readable tables parse and round-trip": the document
/// side is AC6's parse.
#[test]
fn dod_the_tables_parse() {
    assert_none(&report(), "AC6", "does not parse");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(
    SPEC;
    denial_wire_md_missing,
    denial_wire_toml_missing,
    ac3_denial_the_control_bound_unread,
    dod_denial_the_hello_header_short,
    ac3_denial_the_envelope_unread,
    ac3_denial_the_envelope_maxima_differ,
    ac3_denial_the_envelope_off_the_bound,
    dod_denial_an_enumeration_short,
    ac1_denial_no_manifest_section,
    ac1_denial_no_field_table,
    ac1_denial_a_malformed_row,
    ac1_denial_rows_that_do_not_tile,
    ac1_denial_no_content_type_field,
    ac1_denial_no_channel_map_field,
    ac1_denial_l_without_samples,
    ac1_denial_no_counted_join_row,
    ac1_denial_join_delays_without_samples,
    ac1_denial_no_frequency_flag,
    ac1_denial_no_initial_values,
    ac1_denial_no_conversion_delay_field,
    ac1_denial_the_conversion_delay_without_unit,
    ac1_denial_the_cost_bound_without_operations,
    ac1_denial_the_cost_bound_not_per_slice,
    ac1_denial_no_required_e,
    ac1_denial_no_declared_e,
    ac1_denial_no_unit_of_e,
    ac2_denial_a_code_inside_its_range,
    ac2_denial_no_none_encoding,
    ac2_denial_no_unknown_encoding,
    ac2_denial_a_flag_without_not_interpreted,
    ac2_denial_a_flag_without_its_distinct_sentence,
    ac2_denial_the_two_sentinels_equal,
    ac3_denial_the_maximum_not_stated,
    ac3_denial_more_sections_than_the_envelope_allows,
    ac3_denial_a_denied_falsification,
    ac3_denial_no_falsification_report,
    ac3_denial_ac3_not_affirmed,
    ac3_denial_012_08_not_affirmed,
    ac3_denial_the_report_without_the_excess,
    ac3_denial_the_report_without_the_design_owner,
    ac3_denial_the_report_without_the_split,
    ac4_denial_the_falsifier_clause_not_met,
    ac4_denial_no_join_open_item,
    ac4_denial_the_join_item_not_owned_by_004,
    ac5_denial_fewer_than_five_forms,
    ac5_denial_an_outcome_without_a_member,
    ac5_denial_no_count_above_form,
    ac5_denial_the_length_form_without_bad_length,
    ac5_denial_no_channel_map_form,
    ac5_denial_no_conversion_form,
    ac5_denial_no_reserved_form,
    ac5_denial_no_repeat_row,
    ac5_denial_the_repeat_a_rejection,
    ac5_denial_no_manifest_length_disagreement,
    ac5_denial_no_section_count_disagreement,
    ac5_denial_a_disagreement_naming_two_reasons,
    dod_denial_an_untagged_paragraph,
    dod_denial_an_untagged_subsection_with_a_table,
    dod_denial_an_open_item_without_an_owner,
    dod_denial_a_major_change_sentence_without_the_exception,
    dod_denial_a_measured_maximum,
    dod_denial_a_chosen_number_without_decision,
    dod_denial_a_reserved_field_not_recorded,
    dod_denial_a_wait_without_bad_member_list,
    dod_denial_a_landed_header_changed,
    dod_denial_a_landed_table_missing,
    dod_denial_a_landed_table_without_fields,
    ac6_denial_wire_toml_does_not_parse,
    ac6_denial_a_multi_line_string_the_reader_rejects,
    ac6_denial_no_owner_for_the_tooling_gap,
    ac6_denial_a_table_without_its_message,
    ac6_denial_a_message_with_the_wrong_size,
    ac6_denial_no_fixed_part_message,
    ac6_denial_the_fixed_part_of_the_wrong_size,
    ac1_denial_wire_toml_without_the_audio_bound,
    ac1_denial_an_l_that_saturates,
    ac1_denial_the_hop_count_unreadable,
    ac1_denial_no_audio_length_bound,
    ac2_denial_a_stray_prose_sentinel,
    ac2_denial_wire_toml_states_another_sentinel,
    ac4_denial_a_heading_after_a_python_line_break,
    ac2_denial_a_word_character_only_python_knows,
    ac5_denial_a_validity_table_without_separator,
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
    num_c17_hex,
    num_c17_hex_upper,
    num_c17_comma,
    num_c17_narrow_nbsp,
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
    num_required_e_empty_width_cell,
    num_required_e_row_cut_to_two_cells,
    num_manifest_size_hex,
    num_manifest_size_leading_zero,
    num_manifest_size_superscript,
    num_manifest_size_big,
    num_manifest_size_beyond_i128,
    num_manifest_size_comma,
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
    chk_required_e_row_escaped_pipe,
    chk_required_e_row_blank_line_inside,
    chk_required_e_row_ragged_row,
    chk_required_e_row_fenced_row,
    chk_required_e_row_offset_double_zero_seven,
    chk_required_e_row_offset_plus,
    chk_required_e_row_offset_forty_five_digits,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (9b170b2, "failing checks
/// for the manifest encoding") fails every criterion, as the script did.
/// A missing commit fails the test
/// (`DOCCHECK_ALLOW_SHALLOW=1` skips it visibly).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(&SPEC, "9b170b2731f399aba01666587ff1c6995038686f", &["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"]);
}
