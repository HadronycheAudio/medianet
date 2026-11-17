//! Ticket 004-18 — admission handshake messages. Port of
//! `checks/004-18.sh` (deleted with this port; `doccheck-equiv 004-18`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_18`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion, DoD item and the script's other labels), on one denial per
//! assertion of the script, on denials and acceptance probes aimed at the
//! places where Python and Rust read text differently, and on the red-phase
//! tree. Every denial and probe is also run against the old script by the
//! ignored test `old_script_agrees`.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-18.sh`): `cargo run --bin wire-check` exits 0
//! and reports "checks clean"; `cargo test -p hadronyche-tools --test
//! wire_tables` exits 0 with its last result line at 0 failed and at least
//! 12 passed (script lines 1270-1294).
//!
//! Unreachable and kept: `fields_of`'s "the table does not have the five
//! columns" (line 249). The `Hand-off` label: the harness reads no
//! criterion from `FAIL Hand-off:` (its pattern stops at the hyphen), so the
//! old script's rejection of that denial is compared by verdict only.
//!
//! Header and code: the hello's missing election epoch row is reported
//! under AC3 with the 004-17 message (line 422, copied from 004-17), and the
//! malformed rows of the admission tables under AC2 (line 841), the
//! receiver report's under AC1 (line 1004); the open item and the toml
//! timer names under DoD; the request's drop-before-refusal order and the
//! ForeignDomain citation under AC6. The port follows the code.

use hadronyche_tools::doccheck::checks::c004_18::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Every Context item is a field: declared rates (bit/s wide enough
/// for C-219, packets/s, slice rate), the member list (max 32), the reverse
/// port (16 bits), pinned IPv4 addresses (32 bits), the manifest (004-26)
/// and probe pair (004-27) references, the downstream A (C-142); codes
/// distinct inside the admission block; common header rows; a u8 refusal
/// reason." The hello's rows read under `H` parse.
#[test]
fn ac1_every_context_item_is_a_field() {
    assert_criterion(&report(), "AC1");
    assert_criterion(&report(), "H");
}

/// AC2 — "The refusal enumeration is exactly the 18 names of C-185 with
/// unique values (0..17 in C-185's order); VersionMismatch, NameTaken and
/// ModuleCap are raised at discovery, never carried by a refusal." Also the
/// C-115 reservation values per message.
#[test]
fn ac2_the_refusal_enumeration() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The downstream A has a unit (nanoseconds) and the width of the
/// consumer's current A in the receiver report."
#[test]
fn ac3_the_downstream_a() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "Every message's maximum size equals its table sum and is at most
/// C-17's bound." The closed enumeration has exactly 13 members
/// (`REJECTION_PIN`).
#[test]
fn ac4_maximum_sizes() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "No admission field is a timeout or a Pending timer; C-45 is
/// cited."
#[test]
fn ac5_no_timer_and_c45() {
    assert_criterion(&report(), "AC5");
}

/// AC6 — "At least 8 invalid forms, at least 6 per message; the ticket's
/// categories; exact reasons (BadMemberList for member forms); no
/// discovery-raised refusal; no reason outside the two enumerations."
#[test]
fn ac6_invalid_forms() {
    assert_criterion(&report(), "AC6");
}

/// AC7 (document part) — each admission message and the refusal
/// enumeration in wire.toml agree with wire.md.
#[test]
fn ac7_wire_toml_agrees() {
    assert_criterion(&report(), "AC7");
}

/// The settled rulings cited (C-17, C-103, C-142, C-185, C-209, C-219), the
/// behaviour restated, and the hand-off to 012-01.
#[test]
fn settled_behaviour_and_hand_off() {
    assert_criterion(&report(), "Settled");
    assert_criterion(&report(), "Behaviour");
    assert_criterion(&report(), "Hand-off");
}

/// DoD — tags, open items (owner 004 with 005, the falsifier), constants
/// readable, no toml field named like a timer.
#[test]
fn dod_tags_and_open_items() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC; ac2_denial_an_admission_row_of_four_cells, ac2_denial_an_admission_offset_not_a_number, ac2_denial_an_admission_width_not_a_count, h_denial_a_hello_offset_not_a_number, ac1_denial_a_receiver_report_offset_not_a_number, dod_denial_a_constant_unreadable, behaviour_denial_a_duration_unreadable, denial_wire_md_missing, ac4_denial_no_rejection_reasons_section, ac4_denial_the_enumeration_not_thirteen, ac1_denial_no_hello_section, ac3_denial_no_hello_election_epoch, ac2_denial_c185_not_eighteen, ac2_denial_c185_without_a_discovery_reason, ac1_denial_no_admission_section, ac2_denial_a_refusal_listed_twice, ac2_denial_no_refusal_enumeration, ac2_denial_refusal_names_differ_from_c185, ac2_denial_refusal_values_not_unique, ac2_denial_a_refusal_value_outside_a_u8, ac2_denial_a_discovery_reason_not_marked, ac2_denial_another_reason_marked_discovery, ac2_denial_no_discovery_sentence, ac2_denial_no_c185_sentence, ac2_denial_refusal_values_out_of_c185_order, ac1_denial_fewer_than_two_messages, ac1_denial_no_admission_block, ac1_denial_not_starting_with_the_header, ac1_denial_a_header_row_of_another_type, ac1_denial_rows_that_do_not_tile, ac1_denial_a_row_of_part_of_a_byte, ac4_denial_no_maximum_size_sentence, ac4_denial_a_maximum_size_that_differs, ac4_denial_a_message_above_the_bound, ac1_denial_no_type_code, ac1_denial_a_code_outside_the_block, ac1_denial_a_shared_code, ac2_denial_a_scheme_not_zero, ac2_denial_a_tag_not_zero, ac2_denial_a_counter_not_live, ac2_denial_no_c115_sentence, ac1_denial_no_refusal_reason_field, ac1_denial_a_refusal_reason_of_two_bytes, ac2_denial_a_refusal_field_naming_a_discovery_reason, ac1_denial_no_bit_rate, ac1_denial_a_bit_rate_too_narrow, ac1_denial_no_packet_rate, ac1_denial_a_packet_rate_too_narrow, ac1_denial_no_slice_rate, ac1_denial_a_slice_rate_too_narrow, ac1_denial_no_c219_sentence, ac1_denial_no_member_list_of_32, ac1_denial_no_reverse_port, ac1_denial_no_address, ac1_denial_an_address_of_two_bytes, behaviour_denial_no_ipv4, behaviour_denial_no_cs5, ac1_denial_no_manifest_reference, ac1_denial_no_probe_reference, ac1_denial_no_downstream_a, ac1_denial_no_receiver_report_section, ac1_denial_no_current_a_row, ac1_denial_a_downstream_a_without_unit, ac1_denial_a_downstream_a_of_another_width, ac1_denial_a_downstream_a_without_width, ac5_denial_a_timer_field, ac5_denial_c45_not_cited, settled_denial_c103_not_cited, ac6_denial_no_request_message, ac6_denial_a_refusal_before_a_drop, ac6_denial_no_order_hand_off, ac6_denial_no_foreign_domain_citation, hand_off_denial_no_012_01, ac6_denial_counted_under_a_non_member, ac6_denial_an_invented_reason, ac6_denial_an_exact_reason_missing, ac6_denial_a_member_form_without_bad_member_list, ac6_denial_a_discovery_reason_refused, ac6_denial_a_form_naming_no_reason, ac6_denial_fewer_than_six_forms, ac6_denial_fewer_than_eight_forms, ac6_denial_no_oversize_row, dod_denial_no_owned_open_item, dod_denial_an_untagged_paragraph, dod_denial_an_open_item_without_owner, dod_denial_a_table_in_an_untagged_subsection, ac7_denial_wire_toml_missing, ac7_denial_wire_toml_does_not_parse, ac7_denial_no_request_message, ac7_denial_the_layer, ac7_denial_the_dscp, ac7_denial_the_max_size, ac7_denial_a_field_missing, ac7_denial_a_field_misnamed, ac7_denial_a_field_offset, ac7_denial_a_field_width, ac7_denial_a_max_count, ac7_denial_a_field_type, dod_denial_a_toml_field_named_like_a_timer, ac7_denial_fields_not_summing_to_max_size, ac7_denial_a_named_reason_not_listed, ac7_denial_a_listed_reason_not_a_member, ac7_denial_the_toml_refusals_differ, ac7_denial_the_toml_refusals_out_of_order, py_denial_a_reason_glued_to_a_superscript, py_denial_an_open_item_after_a_unit_separator, py_denial_a_line_separator_inside_the_validity_table, py_denial_a_form_feed_inside_the_validity_table, py_denial_a_refusal_value_beyond_64_bits, py_denial_an_offset_in_arabic_indic_digits, py_denial_a_citation_followed_by_a_long_s, py_denial_max_size_as_a_string, py_denial_rejection_reasons_as_a_string, py_denial_a_refusal_value_as_a_string, py_denial_a_triple_quoted_string, py_denial_a_duplicate_dscp, py_denial_rejection_reasons_a_number, py_denial_a_width_as_a_string, py_denial_a_separator_with_a_letter, py_denial_an_open_item_with_a_dotless_i, py_accepts_an_offset_in_arabic_indic_digits, py_accepts_a_negation_within_24_characters_of_wide_ones, py_accepts_a_float_dscp, py_accepts_a_hex_dscp, py_accepts_a_refusal_value_true, py_accepts_a_separator_with_inner_colons, py_accepts_a_unit_separator_in_a_cell, py_accepts_a_combining_accent_after_a,
    num_denial_max_size_of_10_19,
    num_denial_max_size_of_10_40,
    num_denial_max_size_just_beyond_i128,
    num_denial_max_size_with_leading_zeros,
    num_denial_max_size_in_hex,
    num_denial_max_size_as_a_float,
    num_denial_max_size_with_an_underscore,
    num_denial_max_size_missing,
    num_denial_a_width_of_10_40,
    num_denial_a_width_as_a_float,
    num_denial_a_width_as_nan,
    num_denial_a_width_as_an_empty_string,
    num_denial_a_width_as_a_string,
    num_denial_a_width_as_true,
    num_denial_field_a_number,
    num_denial_rejection_reasons_mixed_types,
    num_denial_rejection_reasons_with_a_list,
    num_denial_duplicate_rejection_reasons,
    num_denial_a_toml_refusal_value_of_10_40,
    num_denial_a_toml_refusal_value_of_10_19,
    num_denial_a_toml_refusal_value_as_true,
    num_denial_a_toml_refusal_value_as_a_float,
    num_denial_a_toml_refusal_value_as_a_string,
    num_denial_a_toml_refusal_value_with_leading_zeros,
    num_denial_a_duplicate_toml_refusal,
    num_denial_a_refusal_value_of_10_40,
    num_denial_a_refusal_value_of_10_19,
    num_denial_a_refusal_value_of_256,
    num_denial_a_refusal_value_with_leading_zeros,
    num_denial_a_refusal_value_with_a_plus,
    num_denial_a_refusal_value_glued_to_a_superscript,
    num_denial_an_empty_refusal_value,
    num_denial_a_duplicate_refusal_row,
    num_denial_two_refusals_on_one_value,
    num_denial_the_admission_block_of_10_40,
    num_denial_the_admission_block_with_leading_zeros,
    num_denial_a_duplicate_admission_block,
    num_denial_a_type_code_of_10_19,
    num_denial_a_type_code_of_10_40,
    num_denial_a_stated_size_of_10_40,
    num_denial_a_stated_size_in_groups,
    num_denial_a_stated_size_glued_to_a_superscript,
    num_denial_an_offset_of_10_40,
    num_denial_the_bound_of_10_40,
    num_denial_the_bound_of_10_19,
    num_denial_a_refusal_value_of_2_63,
    num_denial_a_refusal_value_of_45_digits,
    num_denial_a_refusal_value_with_an_underscore,
    num_denial_a_refusal_value_after_a_micro_sign,
    num_denial_a_refusal_value_of_zero,
    num_denial_max_size_of_45_digits,
    num_denial_a_toml_refusal_value_of_2_63,
    lb_denial_lookback_at_exactly_24,
    lb_denial_lookback_at_25,
    lb_denial_lookback_with_a_word_character_at_its_edge,
    tbl_denial_a_blank_line_inside_the_validity_table,
    tbl_denial_an_escaped_pipe_in_a_form_cell,
    tbl_denial_a_ragged_validity_row,
    tbl_denial_a_duplicate_validity_row,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (7221624, "failing checks
/// for the admission handshake messages") fails what the script failed.
/// A missing commit fails the test (see `testing::red_phase`).
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "72216247160b39f0653fa155e18893fa9d8ac7a3",
        &[
            "AC1",
            "AC2",
            "AC5",
            "AC6",
            "AC7",
            "Behaviour",
            "DoD",
            "Hand-off",
            "Settled",
        ],
    );
}
