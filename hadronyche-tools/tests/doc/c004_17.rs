//! Ticket 004-17 — election message layouts. Port of `checks/004-17.sh`
//! (deleted with this port; `doccheck-equiv 004-17` reads it from git
//! history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_17`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and DoD item), on one denial per assertion of the script, on
//! denials and acceptance probes aimed at the places where Python and Rust
//! read text differently, and on the red-phase tree. Every denial and probe
//! is also run against the old script by the ignored test
//! `old_script_agrees`.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-17.sh`): `cargo run --bin wire-check` exits 0
//! and reports "checks clean"; `cargo test -p hadronyche-tools --test
//! wire_tables` exits 0 with its last result line exactly 12 passed, 0
//! failed, 0 ignored (script lines 1133-1154).
//!
//! Unreachable and kept: `fields_of`'s "the table does not have the five
//! columns" (line 254): every caller passes a table chosen by
//! `is_field_table`. The hello's header and layout rows are reported under
//! the label `H` (lines 412-417), as the script prints them.
//!
//! Header and code: the header's AC 4 asks for "at least 6 invalid forms"
//! per message; the code counts the shared rows plus the message's own
//! (line 699), and the port follows the code.

use hadronyche_tools::doccheck::checks::c004_17::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Each message has a type code inside the block the Hello section
/// gives election messages, distinct; the common header rows in the hello's
/// order and widths; the rows tile; the stated maximum size equals the sum
/// and is within C-17." The hello's header and layout, read under `H`,
/// parse.
#[test]
fn ac1_type_codes_headers_and_sizes() {
    assert_criterion(&report(), "AC1");
    assert_criterion(&report(), "H");
}

/// AC2 — "The scheme and tag rows fix the v1 value 0 and the counter row is
/// live (C-115); a sentence cites C-115."
#[test]
fn ac2_the_v1_reservation_values() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "Module id and nonce (64 bits), election epoch at the hello's
/// width; across the set a time state (three values with C-162, or four
/// with a mapping, C-213), holder eligibility bit 0, believed holder,
/// winner's timeline epoch kept or bumped (C-213, C-113), which message
/// carries which field."
#[test]
fn ac3_the_election_fields() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least 6 invalid forms per message (shared plus own); the
/// ticket's list covered; exact reasons; no invented reason." The closed
/// enumeration has exactly 13 members (`REJECTION_PIN`).
#[test]
fn ac4_invalid_forms_with_member_reasons() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "An 'Open item' names 005 as owner with the falsifier; a sentence
/// says the messages reuse or confirm the hello's epoch width and
/// time-state encoding."
#[test]
fn ac5_the_open_item_and_the_hand_off() {
    assert_criterion(&report(), "AC5");
}

/// AC6 (document part) — each election message declared in wire.toml,
/// read with the scripts' subset reader.
#[test]
fn ac6_wire_toml_declares_each_message() {
    assert_criterion(&report(), "AC6");
}

/// Behaviour — TTL 1 and unicast over IPv4 (C-100), CS5 (40) (C-13), the
/// C-32, C-33 and C-37 durations restated; no field is a timer.
#[test]
fn behaviour_restated_and_no_timer_field() {
    assert_criterion(&report(), "Behaviour");
}

/// DoD — tags, open items with owner and falsifier, constants readable.
#[test]
fn dod_tags_and_open_items() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- a two-file denial

/// Behaviour — "C-34 is cited but its bound is not restated" (script line
/// 736) needs two files changed (the constant and a sentence citing it), so
/// it cannot be a single-file entry of the denial list. The old script
/// rejected the same two edits by `Behaviour` when run through the
/// harness's sandbox before the denial list existed (the earlier
/// `old_script_agrees` run of this port, all rows agreeing).
#[test]
fn behaviour_denial_c34_cited_without_its_bound() {
    use hadronyche_tools::doccheck::assert_fails;
    use hadronyche_tools::doccheck::checks::c004_17::WIRE_MD;
    use hadronyche_tools::doccheck::mutate::{replace_in_line, within_section};
    use hadronyche_tools::doccheck::testing::repo;
    use hadronyche_tools::doccheck::ticket::CONSTANTS_MD;
    let docs = repo()
        .mutate(CONSTANTS_MD, |t| {
            replace_in_line(t, r"^\| C-34 \|", "at most 64", "at most 97")
        })
        .mutate(WIRE_MD, |t| {
            within_section(t, "^Election messages$", |s| {
                format!(
                    "{}\n\nThe peer map holds C-34 entries at most `[decision]` here.\n\n",
                    s.trim_end()
                )
            })
        });
    assert_fails(
        &SPEC.evaluate(&docs),
        "Behaviour",
        "C-34 is cited but its bound is not restated",
    );
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC; dod_denial_a_constant_unreadable, behaviour_denial_a_duration_unreadable, denial_wire_md_missing, ac4_denial_no_rejection_reasons_section, ac4_denial_the_enumeration_not_thirteen, ac1_denial_no_hello_section, h_denial_a_hello_layout_row_of_four_cells, h_denial_a_hello_offset_not_a_number, h_denial_a_hello_width_not_a_count, ac3_denial_no_hello_election_epoch_row, ac1_denial_no_election_block, ac1_denial_no_election_section, ac1_denial_no_election_message, ac1_denial_a_row_of_four_cells, ac1_denial_an_offset_not_a_number, ac1_denial_a_width_not_a_count, ac1_denial_not_starting_with_the_header, ac1_denial_a_header_row_of_another_type, ac1_denial_rows_that_do_not_tile, ac1_denial_a_row_of_part_of_a_byte, ac1_denial_no_maximum_size_sentence, ac1_denial_a_maximum_size_that_differs, ac1_denial_a_message_above_the_bound, ac1_denial_no_type_code, ac1_denial_a_code_outside_the_block, ac1_denial_a_shared_code, ac2_denial_a_scheme_not_zero, ac2_denial_a_tag_not_zero, ac2_denial_a_counter_not_live, ac2_denial_no_c115_sentence, ac3_denial_no_module_id_of_64_bits, ac3_denial_no_nonce_of_64_bits, ac3_denial_no_election_epoch_row, ac3_denial_an_election_epoch_width, ac3_denial_no_time_state, ac3_denial_a_time_state_of_two_bytes, ac3_denial_a_time_state_without_its_values, ac3_denial_two_time_state_encodings, ac3_denial_no_time_state_sentence, ac3_denial_no_three_values_argument, ac3_denial_four_values_without_a_mapping, ac3_denial_no_holder_eligibility, ac3_denial_eligibility_without_bit_0, ac3_denial_eligibility_of_two_bytes, ac3_denial_no_believed_holder, ac3_denial_no_winner_timeline_epoch, ac3_denial_no_kept_or_bumped_sentence, ac3_denial_no_compared_unsigned_sentence, ac3_denial_no_carries_sentence, ac4_denial_counted_under_a_non_member, ac4_denial_an_invented_reason, ac4_denial_an_exact_reason_missing, ac4_denial_domain_zero_mapped_to_foreign_domain, ac4_denial_a_version_mismatch_without_its_event, ac4_denial_an_election_form_without_a_reason, ac4_denial_a_form_naming_no_reason, ac4_denial_fewer_than_six_forms, ac4_denial_no_invalid_form_table, ac4_denial_no_oversize_row, ac5_denial_no_owned_open_item, ac5_denial_no_hand_off_sentence, behaviour_denial_ttl_missing, behaviour_denial_cs5_missing, behaviour_denial_a_duration_not_restated, behaviour_denial_a_timer_field, dod_denial_an_untagged_paragraph, dod_denial_an_open_item_without_owner, dod_denial_a_table_in_an_untagged_subsection, ac6_denial_wire_toml_missing, ac6_denial_wire_toml_does_not_parse, ac6_denial_no_candidacy_message, ac6_denial_the_layer, ac6_denial_the_dscp, ac6_denial_the_max_size, ac6_denial_above_the_bound, ac6_denial_a_field_missing, ac6_denial_a_field_misnamed, ac6_denial_a_field_offset, ac6_denial_a_field_width, ac6_denial_a_field_type, behaviour_denial_a_toml_field_named_like_a_timer, ac6_denial_fields_not_summing_to_max_size, ac6_denial_a_named_reason_not_listed, ac6_denial_a_listed_reason_not_a_member, py_denial_a_reason_glued_to_a_superscript, py_denial_an_open_item_after_a_unit_separator, py_denial_a_line_separator_inside_the_validity_table, py_denial_a_form_feed_inside_the_validity_table, py_denial_an_offset_beyond_64_bits, py_denial_an_offset_in_arabic_indic_digits, py_denial_a_time_state_value_after_a_letter, py_denial_a_duration_inside_a_longer_number, py_denial_max_size_as_a_string, py_denial_rejection_reasons_as_a_string, py_denial_a_triple_quoted_string, py_denial_a_duplicate_dscp, py_denial_rejection_reasons_a_number, py_denial_a_width_as_a_string, py_denial_a_separator_with_a_letter, py_denial_an_open_item_with_a_dotless_i, py_accepts_an_offset_in_arabic_indic_digits, py_accepts_a_negation_within_24_characters_of_wide_ones, py_accepts_a_dotless_i, py_accepts_a_combining_accent_after_live, py_accepts_a_float_dscp, py_accepts_a_hex_dscp, py_accepts_a_separator_with_inner_colons, py_accepts_a_unit_separator_in_a_cell,
    num_denial_max_size_of_10_19,
    num_denial_max_size_of_10_36,
    num_denial_max_size_of_10_40,
    num_denial_max_size_just_beyond_i128,
    num_denial_max_size_with_an_underscore,
    num_denial_max_size_with_a_plus,
    num_denial_max_size_with_leading_zeros,
    num_denial_max_size_in_hex,
    num_denial_max_size_as_a_float,
    num_denial_max_size_as_true,
    num_denial_max_size_empty,
    num_denial_max_size_missing,
    num_denial_a_width_of_10_40,
    num_denial_a_width_of_10_19,
    num_denial_a_width_as_a_float,
    num_denial_a_width_as_inf,
    num_denial_a_width_as_nan,
    num_denial_a_width_of_zero_float,
    num_denial_a_width_as_an_empty_string,
    num_denial_a_width_as_true,
    num_denial_a_width_as_a_list,
    num_denial_a_max_count_of_zero,
    num_denial_field_a_number,
    num_denial_duplicate_rejection_reasons,
    num_denial_rejection_reasons_mixed_types,
    num_denial_rejection_reasons_with_a_list,
    num_denial_rejection_reasons_only_numbers,
    num_denial_the_bound_of_10_40,
    num_denial_the_bound_of_10_19,
    num_denial_the_cs5_value_with_leading_zeros,
    num_denial_a_duration_with_a_superscript,
    num_denial_the_election_block_of_10_40,
    num_denial_the_election_block_with_leading_zeros,
    num_denial_a_duplicate_election_block,
    num_denial_the_election_block_glued_to_a_superscript,
    num_denial_a_type_code_of_10_19,
    num_denial_a_type_code_of_10_40,
    num_denial_a_type_code_with_leading_zeros,
    num_denial_a_stated_size_of_10_40,
    num_denial_a_stated_size_of_10_19,
    num_denial_a_stated_size_in_groups,
    num_denial_a_stated_size_glued_to_a_superscript,
    num_denial_a_time_state_value_of_four_digits,
    num_denial_a_time_state_value_with_leading_zeros,
    num_denial_a_time_state_value_after_a_superscript,
    num_denial_an_offset_of_10_40,
    num_denial_an_empty_offset,
    num_denial_a_duplicate_layout_row,
    num_denial_a_rejection_code_of_10_40,
    num_denial_a_duplicate_rejection_member,
    num_denial_a_type_code_of_2_63,
    num_denial_a_type_code_of_45_digits,
    num_denial_a_type_code_of_zero,
    num_denial_a_type_code_with_an_underscore,
    num_denial_a_type_code_after_a_micro_sign,
    num_denial_max_size_of_2_63,
    num_denial_max_size_of_45_digits,
    num_denial_the_bound_of_45_digits,
    lb_denial_lookback_at_exactly_24,
    lb_denial_lookback_at_25,
    lb_denial_lookback_with_a_word_character_at_its_edge,
    tbl_denial_a_blank_line_inside_the_validity_table,
    tbl_denial_an_escaped_pipe_in_a_form_cell,
    tbl_denial_a_ragged_validity_row,
    tbl_denial_a_duplicate_validity_row,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (73ca91a, "failing checks
/// for the election message layouts") fails what the script failed.
/// A missing commit fails the test (see `testing::red_phase`).
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "73ca91a42d8b86056530e0a792b44544c84b3a90",
        &["AC1", "AC4", "AC5", "Behaviour", "DoD"],
    );
}
