//! Ticket 004-16 — the hello message layout and the 32-member size bound.
//! Port of `checks/004-16.sh` (deleted with this port; `doccheck-equiv
//! 004-16` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_16`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and DoD item), on one denial per assertion of the script (the
//! real documents with one mutation applied in memory), on denials and
//! acceptance probes aimed at the places where Python and Rust read text
//! differently, and on the red-phase tree. Every denial and probe is also
//! run against the old script by the ignored test `old_script_agrees`.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-16.sh`): `cargo run --bin wire-check` exits 0
//! and reports "checks clean"; `cargo test -p hadronyche-tools --test
//! wire_tables` exits 0 and its last result line is exactly 12 passed, 0
//! failed, 0 ignored (script lines 1412-1433). The script skipped both under
//! its `WIRE_MD`/`WIRE_TOML` overrides.
//!
//! Unreachable and kept: `fields_of`'s "the table does not have the five
//! columns" (line 268): every caller passes a table already chosen by
//! `is_field_table`. The port reproduces it.
//!
//! Header and code: the header (lines 76-81) lists the exact reasons of the
//! ticket's list; the code also requires the `counter`, `name` and
//! `collision` rows and the collision sentence (lines 1254-1262), and
//! reports the falsified verdict's missing fallback under AC3 (line 777).
//! The port follows the code.

use hadronyche_tools::doccheck::checks::c004_16::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Every hello content has a row with its width; the capability bits
/// carry the holder-eligible bit; the port and content-type maxima; the
/// bundle group id, member list and channel map; the liveness counter is
/// the message counter; the regime stamp version number row." Script: rows
/// by Meaning prefix with the constants' widths (C-10, C-11), a counted UTF-8
/// name, 32 capability bits with a stated holder-eligible bit, a counted
/// port row whose width is its element's sum, the bundle and port element
/// subsections, the liveness-counter sentence.
#[test]
fn ac1_every_hello_content_has_a_row() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The maximum hello size equals the sum of the layout rows; the
/// 32-member size is computed; the verdict fits or is falsified with a
/// fallback." Script: the bundle row counted and equal to its element,
/// the stated bundles-per-hello maximum and its rule, the one-bundle figure,
/// "maximum hello size ... N bytes", the verdict paragraph classified with
/// quoted text removed and "not fits" ignored, each verdict's arithmetic.
#[test]
fn ac2_the_sizes_and_the_verdict() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The six alarms of C-173, one bit or value each, distinct; the
/// transport role is a proposal with epic 005 as the confirming owner and
/// its falsifier."
#[test]
fn ac3_the_alarms_and_the_transport_role() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least ten invalid forms; the exact reasons of the ticket's
/// list; member reasons or argued non-rejections; no reason invented." The
/// closed enumeration has exactly 13 members (`REJECTION_PIN`).
#[test]
fn ac4_ten_invalid_forms_with_member_reasons() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "The common control header is defined once, sums to its stated
/// size, the hello's table starts with it, and its order is the sync
/// header's."
#[test]
fn ac5_the_common_control_header() {
    assert_criterion(&report(), "AC5");
}

/// AC6 (document part) — "wire.toml hello: layer control, dscp 40, max_size
/// the Markdown sum within C-17, fields one to one, nested maxima,
/// rejection_reasons complete", read with the scripts' TOML subset reader.
#[test]
fn ac6_wire_toml_agrees_with_the_layout() {
    assert_criterion(&report(), "AC6");
}

/// Work 3 — the regime stamp version number's stale rule and its own
/// message (C-18).
#[test]
fn work3_the_regime_stamp_version_number() {
    assert_criterion(&report(), "Work3");
}

/// Behaviour restated as fact: 1 s (C-30), TTL 1 over IPv4 with unicast
/// (C-100), CS5 (40) (C-13), binary not the POC's text gossip, 64 peers
/// (C-34), with the constants' values.
#[test]
fn behaviour_restated_as_fact() {
    assert_criterion(&report(), "Behaviour");
}

/// DoD — tags on every prose paragraph of 5+ words and on every table's
/// subsection, open items with owner and falsifier, the regime stamp fields
/// documented, the constants readable.
#[test]
fn dod_tags_open_items_and_regime_stamp() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC; dod_denial_a_constant_unreadable, denial_wire_md_missing, ac4_denial_no_rejection_reasons_section, ac4_denial_the_enumeration_not_thirteen, ac1_denial_no_hello_section, ac1_denial_a_layout_row_of_four_cells, ac1_denial_an_offset_not_a_number, ac1_denial_a_width_not_a_count, ac5_denial_no_common_header_subsection, ac5_denial_the_header_subsection_has_no_field_table, ac5_denial_a_header_row_of_no_known_field, ac5_denial_a_header_width_against_the_constants, ac5_denial_a_header_row_out_of_place, ac5_denial_the_header_not_32_bytes, ac5_denial_no_stated_header_size, ac5_denial_a_stated_header_size_that_differs, ac5_denial_no_reuse_sentence, ac5_denial_no_sync_section, ac5_denial_an_order_differing_from_sync_without_excuse, ac5_denial_an_excuse_that_does_not_name_the_field, ac5_denial_no_layout_subsection, ac5_denial_the_layout_subsection_has_no_field_table, ac5_denial_the_layout_not_starting_with_the_header, ac5_denial_a_layout_header_row_of_another_type, ac5_denial_layout_rows_that_do_not_tile, ac5_denial_a_row_of_part_of_a_byte, ac1_denial_no_transport_role_row, ac1_denial_a_module_id_of_the_wrong_width, ac1_denial_no_counted_name_row, ac1_denial_a_name_without_its_encoding, ac1_denial_capability_bits_not_32, ac1_denial_no_holder_eligible_bit, ac1_denial_no_counted_port_row, ac1_denial_the_counter_not_the_liveness_counter, ac1_denial_no_sentence_on_the_liveness_counter, ac1_denial_no_bundle_element_subsection, ac1_denial_no_channel_map_row, ac2_denial_bundle_element_rows_that_do_not_tile, ac2_denial_a_member_row_above_c08, ac2_denial_a_member_row_not_at_c08, ac2_denial_no_member_row_at_c08, ac1_denial_no_port_element_subsection, ac1_denial_no_counted_content_type_row, ac1_denial_a_port_row_unequal_to_its_element, ac2_denial_the_bundle_row_not_counted, ac2_denial_the_bundle_row_unequal_to_its_element, ac2_denial_no_one_byte_bundle_count, ac2_denial_no_stated_bundles_per_hello, ac2_denial_a_stated_maximum_unequal_to_the_row, ac2_denial_no_rule_for_more_bundles, ac2_denial_too_few_bundles_for_c09, ac2_denial_no_one_bundle_figure, ac2_denial_a_one_bundle_figure_that_differs, ac2_denial_no_maximum_hello_size, ac2_denial_no_verdict_paragraph, ac2_denial_a_verdict_stating_neither, ac2_denial_a_verdict_stating_both, ac2_denial_a_verdict_saying_c17_falsifier_holds, ac2_denial_fits_without_bundle_rows, ac2_denial_fits_above_the_bound, ac2_denial_fits_with_a_stated_maximum_that_differs, ac2_denial_falsified_with_bundle_rows, ac2_denial_falsified_without_an_all_bundles_maximum, ac2_denial_falsified_with_an_all_bundles_maximum_within_the_bound, ac2_denial_falsified_with_a_fallback_above_the_bound, ac2_denial_falsified_with_a_stated_maximum_that_differs, ac2_denial_falsified_without_the_all_bundles_size_stated, ac2_denial_falsified_citing_c17_against_its_falsifier, ac3_denial_falsified_without_a_fallback, ac3_denial_no_alarm_subsection, ac3_denial_an_alarm_table_of_five_rows, ac3_denial_an_alarm_without_a_row, ac3_denial_an_alarm_without_a_bit, ac3_denial_two_alarms_on_one_bit, ac3_denial_an_alarm_of_value_zero, ac3_denial_an_alarm_beyond_the_field, ac3_denial_no_transport_role_subsection, ac3_denial_a_role_table_of_one_row, ac3_denial_the_roles_not_a_proposal, ac3_denial_no_role_open_item, ac3_denial_a_role_open_item_without_owner, ac3_denial_a_role_open_item_without_its_falsifier, ac4_denial_no_invalid_form_table, ac4_denial_fewer_than_ten_forms, ac4_denial_an_invented_reason, ac4_denial_an_exact_reason_missing, ac4_denial_domain_zero_mapped_to_foreign_domain, ac4_denial_a_version_mismatch_without_its_event, ac4_denial_a_form_naming_no_reason, ac4_denial_a_collision_row_without_its_argument, ac4_denial_no_oversize_row, ac4_denial_no_counter_row, ac4_denial_no_name_row, ac4_denial_no_collision_row_nor_sentence, ac4_denial_no_collision_sentence, work3_denial_no_stale_rule, dod_denial_the_regime_stamp_fields_not_documented, work3_denial_c18_not_cited, behaviour_denial_ttl_missing, behaviour_denial_the_hello_period_of_c30, behaviour_denial_the_cs5_value_of_c13, behaviour_denial_the_peer_bound_of_c34, dod_denial_an_untagged_paragraph, dod_denial_an_open_item_without_owner, dod_denial_a_table_in_an_untagged_subsection, ac6_denial_wire_toml_missing, ac6_denial_wire_toml_does_not_parse, ac6_denial_a_hello_named_message_not_control, ac6_denial_no_hello_message, ac6_denial_the_hello_not_control, ac6_denial_the_hello_dscp, ac6_denial_the_hello_max_size, ac6_denial_the_hello_above_the_bound, ac6_denial_a_field_missing, ac6_denial_a_field_misnamed, ac6_denial_a_field_offset, ac6_denial_a_counted_width, ac6_denial_a_max_count, ac6_denial_a_count_naming_no_earlier_field, ac6_denial_a_width, ac6_denial_a_type, ac6_denial_fields_not_summing_to_max_size, ac6_denial_the_port_field_without_its_nested_maximum, ac6_denial_the_bundle_field_without_32, ac6_denial_a_named_reason_not_listed, ac6_denial_a_listed_reason_not_a_member, py_denial_a_reason_glued_to_a_superscript, py_denial_an_open_item_after_a_unit_separator, py_denial_a_line_separator_inside_the_validity_table, py_denial_a_form_feed_inside_the_validity_table, py_denial_an_offset_beyond_64_bits, py_denial_an_offset_in_arabic_indic_digits, py_denial_max_size_as_a_string, py_denial_rejection_reasons_as_a_string, py_denial_a_triple_quoted_string, py_denial_a_duplicate_dscp, py_denial_rejection_reasons_a_number, py_denial_a_counted_width_as_a_string, py_denial_a_sync_header_row_of_three_cells, py_denial_a_separator_with_a_letter, py_denial_a_headerless_alarm_table, py_denial_an_open_item_with_a_dotless_i, py_denial_a_verdict_of_not_fits, py_accepts_an_offset_in_arabic_indic_digits, py_accepts_a_combining_accent_after_an_alarm, py_accepts_a_dotless_i, py_accepts_a_negation_within_24_characters_of_wide_ones, py_accepts_a_float_dscp, py_accepts_a_hex_dscp, py_accepts_a_separator_with_inner_colons, py_accepts_a_unit_separator_in_a_cell);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (2e6ba78, "failing checks
/// for the hello layout and the 32-member size bound") fails what the
/// script failed: AC1 to AC6, Behaviour, DoD and Work3. Skipped when the
/// commit is not in the clone.
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "2e6ba7805f875587090453369dc0aebe092de7e3",
        &[
            "AC1",
            "AC2",
            "AC3",
            "AC4",
            "AC5",
            "AC6",
            "Behaviour",
            "DoD",
            "Work3",
        ],
    );
}
