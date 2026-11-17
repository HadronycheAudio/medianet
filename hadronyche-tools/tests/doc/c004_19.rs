//! Ticket 004-19 — sync request and reply layouts. Port of
//! `checks/004-19.sh` (deleted with this port; `doccheck-equiv 004-19`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_19`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion, work item and DoD), on one denial per assertion of the
//! script, on denials and acceptance probes aimed at the places where
//! Python and Rust read text differently, and on the red-phase tree. Every
//! denial and probe is also run against the old script by the ignored test
//! `old_script_agrees`.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-19.sh`): `cargo run --bin wire-check` exits 0
//! and reports "checks clean"; `cargo test -p hadronyche-tools --test
//! wire_tables` exits 0, prints a result line, and its last one reads
//! exactly 12 passed, 0 failed, 0 ignored; `python3
//! doc/kickoff/check-design.py doc/design` exits 0 with "OK" as its last
//! line (script lines 1322-1378).
//!
//! Header and code: the toml size findings (max_size not an int, fields not
//! contiguous, the sum, the pad field) are reported under AC1 (lines
//! 1180-1218) and the toml DSCP names under W4; the port follows the code.
//! The placeholder branch fails every criterion but AC6 (lines 667-684).

use hadronyche_tools::doccheck::checks::c004_19::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Request and reply have identical byte size": the sums of the
/// tables tile and agree, the stated size equals them, the request pads
/// with an explicit zero field, and wire.toml's max_size and fields agree.
#[test]
fn ac1_identical_byte_size() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The reply carries the echoed send timestamp (64 bits), the epoch
/// (16 bits, C-10) and the holder module id (64 bits, C-11); both carry a
/// 16-bit domain id with 0 = unconfigured."
#[test]
fn ac2_the_reply_fields() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "Stateless; the reply echoes the request's send timestamp; no
/// per-request field" (a keyword detector, as the script's header admits).
#[test]
fn ac3_stateless() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "The authentication-reservation decision, [decision] with a
/// falsifier, and the tables carrying (or not) the reservation."
#[test]
fn ac4_the_authentication_decision() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "At least 6 invalid forms; exact reasons; every named reason a
/// member." The closed enumeration has exactly 13 members
/// (`REJECTION_PIN`).
#[test]
fn ac5_invalid_forms() {
    assert_criterion(&report(), "AC5");
}

/// AC6 (document part) — the sync messages of wire.toml.
#[test]
fn ac6_wire_toml() {
    assert_criterion(&report(), "AC6");
}

/// W1, W2, W4, W5 — header rows, timestamps, behaviour without fields, the
/// receive-timestamp note slot.
#[test]
fn work_items() {
    for w in ["W1", "W2", "W4", "W5"] {
        assert_criterion(&report(), w);
    }
}

/// DoD — tags on blocks, open items with owner and falsifier, constants
/// readable.
#[test]
fn dod_tags_and_open_items() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC; dod_denial_a_constant_unreadable, denial_wire_md_missing, ac5_denial_no_rejection_reasons_section, ac5_denial_the_enumeration_not_thirteen, ac1_denial_no_sync_section, ac1_denial_the_placeholder, ac1_denial_no_reply_table, ac1_denial_fields_that_do_not_tile, ac1_denial_fields_not_whole_bytes, ac1_denial_sizes_that_differ, ac1_denial_no_stated_size, ac1_denial_a_stated_size_that_differs, ac1_denial_no_padding_row, ac1_denial_padding_without_zero, ac2_denial_no_echo_row, ac2_denial_an_echo_not_64_bits, ac2_denial_no_epoch_row, ac2_denial_an_epoch_not_16_bits, ac2_denial_no_module_id_row, ac2_denial_a_module_id_not_64_bits, ac2_denial_no_domain_row, ac2_denial_a_domain_not_16_bits, ac2_denial_a_domain_without_unconfigured, ac3_denial_not_stateless, ac3_denial_no_echo_sentence, ac3_denial_a_per_request_field, ac4_denial_no_decision_paragraph, ac4_denial_no_carry_statement, ac4_denial_no_falsifier, ac4_denial_carried_without_a_zero_scheme, ac4_denial_carried_without_a_live_counter, ac4_denial_carried_without_a_zero_tag, ac4_denial_not_carried_with_its_fields, ac5_denial_no_form_table, ac5_denial_fewer_than_six_forms, ac5_denial_an_exact_reason_missing, ac5_denial_a_non_holder_without_a_member, ac5_denial_no_wrong_epoch_row, ac5_denial_no_non_holder_row, ac5_denial_a_short_form_not_too_short, ac5_denial_a_size_form_mapped_to_too_short, ac5_denial_no_bad_length_size_row, ac5_denial_an_invented_reason, ac5_denial_counted_under_a_non_member, ac6_denial_wire_toml_missing, ac6_denial_wire_toml_does_not_parse, ac6_denial_no_sync_request, ac6_denial_no_sync_reply, ac6_denial_the_layer, ac6_denial_the_dscp, ac1_denial_no_integer_max_size, ac1_denial_toml_fields_not_contiguous, ac1_denial_toml_fields_not_summing, ac1_denial_toml_size_against_markdown, ac6_denial_a_listed_reason_not_a_member, ac6_denial_a_field_count_that_differs, ac6_denial_a_field_width_that_differs, ac6_denial_a_field_type_that_differs, ac1_denial_no_toml_pad_field, w4_denial_a_toml_rate_field, ac6_denial_an_exact_reason_not_listed, w1_denial_fewer_than_three_rows, w1_denial_no_magic, w1_denial_no_version, w1_denial_no_message_type, w2_denial_no_unit, w2_denial_no_clock, w2_denial_no_slice_header, w2_denial_no_relation, w4_denial_no_cs6, w4_denial_no_ipv4_unicast, w4_denial_no_request_rate, w4_denial_no_accepted_rates, w4_denial_a_table_naming_a_rate, w5_denial_no_per_os_slot, w5_denial_no_c131, w5_denial_no_regime_stamp, w5_denial_a_table_naming_a_source, dod_denial_an_untagged_paragraph, dod_denial_a_table_after_an_untagged_paragraph, dod_denial_an_open_item_without_owner, py_denial_a_reason_glued_to_a_superscript, py_denial_a_reason_after_no, py_denial_a_line_separator_inside_the_validity_table, py_denial_a_form_feed_inside_the_validity_table, py_denial_an_offset_beyond_64_bits, py_denial_an_offset_in_arabic_indic_digits, py_denial_max_size_as_a_string, py_denial_a_triple_quoted_string, py_denial_a_duplicate_dscp, py_denial_an_offset_as_a_string, py_denial_a_reason_as_a_list, py_denial_a_separator_with_a_letter, py_denial_an_open_item_with_a_dotless_i, py_accepts_an_offset_in_arabic_indic_digits, py_accepts_a_negation_within_25_characters_of_wide_ones, py_accepts_a_dotless_i, py_accepts_a_float_dscp, py_accepts_a_hex_dscp, py_accepts_a_separator_with_inner_colons, py_accepts_a_unit_separator_in_a_cell, py_accepts_a_combining_accent_after_a_reason,
    num_denial_max_size_of_10_19,
    num_denial_max_size_of_10_40,
    num_denial_max_size_just_beyond_i128,
    num_denial_max_size_as_a_float,
    num_denial_max_size_as_true,
    num_denial_max_size_with_leading_zeros,
    num_denial_max_size_in_hex,
    num_denial_max_size_with_an_underscore,
    num_denial_max_size_missing,
    num_denial_an_offset_of_10_40,
    num_denial_an_offset_of_10_19,
    num_denial_an_offset_as_a_float,
    num_denial_an_offset_as_a_list,
    num_denial_an_offset_as_true,
    num_denial_an_offset_as_nan,
    num_denial_an_offset_missing,
    num_denial_a_duplicate_offset,
    num_denial_a_width_as_a_string,
    num_denial_a_width_of_10_40,
    num_denial_a_width_as_a_float,
    num_denial_a_pad_width_as_a_string,
    num_denial_a_pad_width_of_zero,
    num_denial_a_pad_width_negative,
    num_denial_a_pad_width_of_10_19,
    num_denial_the_dscp_as_a_float,
    num_denial_the_dscp_as_a_string,
    num_denial_rejection_reasons_with_a_list,
    num_denial_rejection_reasons_with_a_number,
    num_denial_an_md_offset_of_10_40,
    num_denial_an_md_offset_with_leading_zeros,
    num_denial_an_md_offset_with_a_plus,
    num_denial_an_empty_md_offset,
    num_denial_an_md_width_glued_to_a_superscript,
    num_denial_an_md_width_in_groups,
    num_denial_an_md_width_with_an_underscore,
    num_denial_an_md_width_of_10_36_bytes,
    num_denial_an_md_width_of_10_38_bytes,
    num_denial_a_duplicate_md_row,
    num_denial_a_stated_size_in_groups,
    num_denial_a_stated_size_of_10_40,
    num_denial_a_stated_size_glued_to_a_superscript,
    num_denial_the_cs6_value_with_leading_zeros,
    num_denial_the_request_rate_of_10_40,
    num_denial_the_accepted_rate_of_10_40,
    num_denial_the_epoch_width_of_10_40,
    num_denial_the_epoch_width_glued_to_a_superscript,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (20bb1e4, "failing checks
/// for the sync request and reply layouts") fails what the script failed.
/// A missing commit fails the test (see `testing::red_phase`).
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "20bb1e4b8970d291bfcd808b50ec17046cdc34b7",
        &[
            "AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD", "W1", "W2", "W4", "W5",
        ],
    );
}
