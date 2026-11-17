//! Ticket 004-20 — receiver report and one-way probe layouts. Port of
//! `checks/004-20.sh` (deleted with this port; `doccheck-equiv 004-20`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_20`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion, consistency label and DoD), on one denial per assertion of
//! the script, on denials and acceptance probes aimed at the places where
//! Python and Rust read text differently, and on the red-phase tree. Every
//! denial and probe is also run against the old script by the ignored test
//! `old_script_agrees`.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-20.sh`): `cargo run --bin wire-check` exits 0
//! and reports "checks clean"; `cargo test -p hadronyche-tools --test
//! wire_tables` exits 0, prints a result line, and its last one reads
//! exactly 12 passed, 0 failed, 0 ignored; `python3
//! doc/kickoff/check-design.py doc/design` exits 0 with "OK" as its last
//! line (script lines 1495-1539).
//!
//! The labels `C-40`, `C-41`, `C-13` and `C-100`: the harness reads no
//! criterion from a label with a hyphen (`FAIL C-40:`), so the old script's
//! rejection of those denials is compared by verdict only.
//!
//! Header and code: the tiling of every field table is reported under AC1
//! (the admission tables' under AC3, line 1069), the header rows under W1,
//! a missing toml report or probe message under AC1 (lines 1343-1345), the
//! forward-media sentence under AC1 (line 1457); the port follows the code.
//! The tiling rule restarts every row at the running position, whether or
//! not the row was out of place (line 811 and 815), and the port does too.

use hadronyche_tools::doccheck::checks::c004_20::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Exactly two v1 measurement kinds, the report unicast only, no
/// query message (C-47)", the field tables tiling, the toml report and
/// probe present, no per-receiver state in forward media.
#[test]
fn ac1_two_kinds_unicast_no_query() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The report's maximum size with 32 members, stated and computed;
/// member count maximum 32 (C-08)."
#[test]
fn ac2_the_maximum_size() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "Every content item has a field": bundle identity, the eleven
/// C-159 counters with widths and wrap rules, drops, observed DSCP (6
/// bits), current A with width, unit and range, C-108, interrupted events,
/// demoted rounds, the A bound to 004-18's downstream A.
#[test]
fn ac3_every_content_item() {
    assert_criterion(&report(), "AC3");
    assert_criterion(&report(), "W1");
}

/// AC4 — "The probe's quantities and the authentication decision, tagged
/// [decision] with falsifiers; identifiers, timestamps, sizes."
#[test]
fn ac4_the_probe() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "At least 6 invalid forms covering the ticket's list; exact
/// reasons; no invented reason." The closed enumeration has exactly 13
/// members (`REJECTION_PIN`).
#[test]
fn ac5_invalid_forms() {
    assert_criterion(&report(), "AC5");
}

/// AC6 (document part) — the measurement messages of wire.toml.
#[test]
fn ac6_wire_toml() {
    assert_criterion(&report(), "AC6");
}

/// Consistency — C-40, C-41, C-13 and C-100 restated, no rate field.
#[test]
fn consistency() {
    for c in ["C-40", "C-41", "C-13", "C-100"] {
        assert_criterion(&report(), c);
    }
}

/// DoD — tags, open items with owner and falsifier, constants readable.
#[test]
fn dod_tags_and_open_items() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC; dod_denial_a_constant_unreadable, denial_wire_md_missing, ac5_denial_no_rejection_reasons_section, ac5_denial_the_enumeration_not_thirteen, ac2_denial_no_receiver_report_section, ac4_denial_no_probe_section, ac1_denial_a_width_not_stated, ac1_denial_rows_that_do_not_tile, ac1_denial_fields_not_whole_bytes, ac1_denial_no_per_member_table, ac1_denial_no_probe_reply_table, ac1_denial_not_unicast_only, ac1_denial_no_query_statement, ac2_denial_maximum_not_stated, ac2_denial_a_size_sentence_with_another_number, ac2_denial_no_member_count, ac2_denial_a_member_count_maximum_not_32, ac3_denial_no_bundle_identity, ac3_denial_a_bundle_identity_not_a_module_id, ac3_denial_a_counter_missing, ac3_denial_a_counter_without_width, ac3_denial_no_wrap_rule, ac3_denial_no_socket_drop, ac3_denial_a_socket_drop_without_width, ac3_denial_no_observed_dscp, ac3_denial_a_dscp_of_five_bits, ac3_denial_a_dscp_range_not_63, ac3_denial_no_current_a, ac3_denial_a_without_width, ac3_denial_a_without_unit, ac3_denial_a_without_range, ac3_denial_no_reclassification_sentence, ac3_denial_no_interrupted_events_sentence, ac3_denial_no_demoted_rounds_sentence, ac3_denial_no_004_18_binding_sentence, w1_denial_no_magic, w1_denial_no_version, w1_denial_no_domain, w1_denial_no_message_type, ac3_denial_admission_placeholder_without_open_item, ac3_denial_no_downstream_a_in_admission, ac3_denial_a_downstream_a_of_another_width, ac4_denial_no_quantities_paragraph, ac4_denial_no_quantities_falsifier, ac4_denial_no_authentication_decision, ac4_denial_a_decision_not_covering_both, ac4_denial_a_decision_saying_neither, ac4_denial_a_decision_without_falsifier, ac4_denial_carried_without_the_fields, ac4_denial_not_carried_with_a_field, ac4_denial_no_probe_identifier, ac4_denial_no_echo, ac4_denial_no_request_timestamp, ac4_denial_no_slice_header_relation, ac4_denial_no_monotonic, ac4_denial_no_whole_byte_size, ac4_denial_sizes_not_stated, ac4_denial_a_size_sentence_with_another_number, ac5_denial_fewer_than_six_forms, ac5_denial_a_row_outside_the_list_without_a_reason, ac5_denial_an_exact_reason_missing, ac5_denial_an_unknown_bundle_without_a_reason, ac5_denial_dscp_without_its_argument, ac5_denial_no_a_row, ac5_denial_an_invented_reason, ac5_denial_counted_under_a_non_member, ac6_denial_wire_toml_missing, ac6_denial_wire_toml_does_not_parse, ac1_denial_no_toml_report, ac1_denial_no_toml_probe, ac6_denial_the_layer, ac6_denial_the_dscp, ac6_denial_no_reasons_list, ac6_denial_a_reason_not_a_member, ac6_denial_probe_field_count, ac6_denial_probe_field_width, ac6_denial_probe_field_type, ac6_denial_report_field_count, ac6_denial_member_max_count, ac6_denial_member_count_name, ac6_denial_report_max_size, ac6_denial_no_probe_reply_message, ac6_denial_probe_max_size, ac6_denial_a_named_reason_not_declared, ac6_denial_a_reason_without_the_measurement_layer, c40_denial_no_period, c40_denial_not_per_bundle, c41_denial_no_timeout, c13_denial_no_cs5, c40_denial_a_rate_field, ac1_denial_forward_media_with_receiver_state, c13_denial_no_ef, c100_denial_no_reverse_port, dod_denial_a_table_after_an_untagged_paragraph, dod_denial_an_untagged_paragraph, dod_denial_an_open_item_without_owner, py_denial_a_reason_glued_to_a_superscript, py_denial_a_reason_after_no, py_denial_a_line_separator_inside_the_validity_table, py_denial_a_form_feed_inside_the_validity_table, py_denial_an_offset_beyond_64_bits, py_denial_an_offset_in_arabic_indic_digits, py_denial_max_size_as_a_string, py_denial_a_triple_quoted_string, py_denial_a_duplicate_dscp, py_denial_a_reason_as_a_list, py_denial_a_separator_with_a_letter, py_denial_an_open_item_with_a_dotless_i, py_accepts_an_offset_in_arabic_indic_digits, py_accepts_a_unit_separator_in_a_duration, py_accepts_a_negation_within_25_characters_of_wide_ones, py_accepts_a_dotless_i, py_accepts_a_float_dscp, py_accepts_a_hex_dscp, py_accepts_a_separator_with_inner_colons, py_accepts_a_combining_accent_after_a,
    num_denial_max_size_of_10_19,
    num_denial_max_size_of_10_40,
    num_denial_max_size_as_a_float,
    num_denial_max_size_as_a_string,
    num_denial_max_size_with_leading_zeros,
    num_denial_max_size_with_an_underscore,
    num_denial_max_size_in_hex,
    num_denial_max_size_missing,
    num_denial_a_max_count_of_10_40,
    num_denial_a_max_count_as_true,
    num_denial_a_max_count_as_a_float,
    num_denial_a_count_as_a_number,
    num_denial_the_probe_max_size_of_10_19,
    num_denial_the_probe_dscp_as_a_float,
    num_denial_the_probe_dscp_as_true,
    num_denial_rejection_reasons_with_a_list,
    num_denial_rejection_reasons_mixed_types,
    num_denial_duplicate_rejection_reasons,
    num_denial_the_report_field_a_number,
    num_denial_an_offset_of_10_40,
    num_denial_an_offset_of_10_19,
    num_denial_an_offset_with_a_plus,
    num_denial_an_offset_with_leading_zeros,
    num_denial_an_empty_offset,
    num_denial_an_offset_glued_to_a_superscript,
    num_denial_a_width_of_10_38_bytes,
    num_denial_a_width_of_10_19,
    num_denial_a_width_glued_to_a_superscript,
    num_denial_a_width_of_zero,
    num_denial_a_duplicate_fixed_row,
    num_denial_a_member_count_maximum_of_10_40,
    num_denial_a_stated_maximum_in_groups,
    num_denial_a_stated_maximum_of_10_40,
    num_denial_a_stated_maximum_glued_to_a_superscript,
    num_denial_the_report_period_of_10_40,
    num_denial_the_report_period_with_leading_zeros,
    num_denial_the_timeout_of_10_19,
    num_denial_the_members_maximum_of_10_40,
    num_denial_the_ef_value_glued_to_a_superscript,
    num_denial_an_offset_of_2_63,
    num_denial_an_offset_of_45_digits,
    num_denial_an_offset_with_an_underscore,
    num_denial_a_width_after_a_micro_sign,
    num_denial_max_size_of_2_63,
    num_denial_the_report_period_of_45_digits,
    lb_denial_lookback_at_exactly_25,
    lb_denial_lookback_at_26,
    lb_denial_lookback_with_a_word_character_at_its_edge,
    tbl_denial_a_blank_line_inside_the_validity_table,
    tbl_denial_an_escaped_pipe_in_a_form_cell,
    tbl_denial_a_ragged_validity_row,
    tbl_denial_a_duplicate_validity_row,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (5958cd7, "failing checks
/// for the receiver report and one-way probe layouts") fails what the
/// script failed. A missing commit fails the test (see `testing::red_phase`).
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "5958cd7b73337e9b203636656beea4fda2ae8513",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"],
    );
}
