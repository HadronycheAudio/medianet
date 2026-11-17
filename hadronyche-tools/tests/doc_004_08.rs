//! Ticket 004-08 — slice header and send-timestamp format. Port of
//! `checks/004-08.sh` (deleted with this port; `doccheck-equiv 004-08` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_08`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion, work step and definition-of-done item) and on the denial table
//! below: one row per assertion of the script, plus rows aimed at the Python
//! semantics its code touches. Every row is also run against the old script
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-08.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! stays 12 passed, 0 failed, 0 ignored (AC5); the design checker prints OK
//! (DoD). The script never ran checks/004-01.sh.
//!
//! Header and code disagree: the header says the forms' reasons are members
//! of "the closed 13-name enumeration"; the code replaces the 13 names with
//! those found in the Rejection reasons tables when at least 10 are found
//! (004-08:783-784). The port follows the code
//! (`ten_found_names_shrink_the_enumeration`).

use hadronyche_tools::doccheck::checks::c004_08::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- criteria

/// AC1 — "The send timestamp has a stated width, unit, clock source and
/// wrap period, each tagged `[decision]`; the falsifier (a unit that cannot
/// resolve one sample period at 96 kHz, or a wrap inside one reassembly or
/// alignment span) is answered." Script: the four items in the send-timestamp
/// blocks and their neighbours, each with `[decision]` in its block or a
/// neighbour; the tick at most 10 416 2/3 ns; the wrap longer than 2 slice
/// periods (C-15), naming the reassembly and alignment spans and showing
/// the 96 000 Hz arithmetic.
#[test]
fn ac1_send_timestamp_width_unit_clock_and_wrap() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The slice header table contains position as 64 bits and all
/// rule-2 fields with offsets and valid ranges." Script: position 64 bits,
/// frames, packet index, packet count and send timestamp; offsets from byte
/// 16 tiling contiguously; frames 1 to a maximum; packet index below packet
/// count; packet count 1 in the default regime.
#[test]
fn ac2_slice_header_table() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The multi-packet regime is defined on the media side and the
/// regime stamp values it needs are handed to 004-25."
#[test]
fn ac3_multi_packet_regime_and_hand_off() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least 5 invalid forms with rejection reasons." Script: the
/// ticket's named forms (packet index >= count under BadPacketIndex, count
/// 0, frames 0 or above maximum, frames inconsistent with the payload under
/// BadLength, count inconsistent across a slice), each with a member, no
/// reason outside the enumeration, at least 5 with a member.
#[test]
fn ac4_five_invalid_forms_with_member_reasons() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "The machine-readable table passes the 004-02 checker." Script
/// (document part): wire.toml's slice-header fields agree with wire.md
/// offset by offset, max_size covers both headers, the media packet lists
/// BadPacketIndex.
#[test]
fn ac5_wire_toml_agrees_with_the_table() {
    assert_criterion(&report(), "AC5");
}

/// Work step 5 — the receive-timestamp note: C-131's four sources, C-129's
/// capability states, carried by 004-25's regime stamp, not the slice
/// header.
#[test]
fn ws5_receive_timestamp_note() {
    assert_criterion(&report(), "WS5");
}

/// DoD — every block of the section carries an evidence tag (a table or code
/// block inherits the previous block's), and "regime stamp fields touched
/// are complete and documented" is covered or marked not applicable (C-125).
#[test]
fn dod_tags_and_the_regime_stamp_line() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (af6782d, "failing checks
/// for the slice header and send timestamp") fails as the script did.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "af6782db01583fe2cfd10f1f45298178790f9a21",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "DoD", "WS5"],
    );
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC;
    setup_wire_md_missing,
    setup_no_section,
    setup_placeholder,
    setup_no_field_table,
    ac1_no_send_timestamp_region,
    ac1_no_width,
    ac1_unit_too_coarse,
    ac1_no_clock,
    ac1_no_wrap,
    ac1_wrap_inside_a_reassembly_span,
    ac1_no_reassembly_span,
    ac1_no_alignment_span,
    ac1_no_resolution_arithmetic,
    ac1_width_not_decision_tagged,
    ac1_unit_not_decision_tagged,
    ac1_clock_not_decision_tagged,
    ac1_wrap_not_decision_tagged,
    ac2_no_packet_index_field,
    ac2_position_not_64_bits,
    ac2_not_from_byte_16,
    ac2_stated_bits_disagree,
    ac2_not_whole_bytes,
    ac2_a_gap,
    ac2_an_overlap,
    ac2_ends_mid_byte,
    ac2_frames_no_lower_bound,
    ac2_frames_no_upper_bound,
    ac2_index_not_bounded_by_count,
    ac2_count_default_not_one,
    ac3_no_multi_packet_regime,
    ac3_hand_off_misses_frames_per_slice,
    ws5_no_note,
    ws5_a_source_missing,
    ws5_no_c129,
    ws5_no_regime_stamp_pointer,
    ws5_not_said_outside_the_slice_header,
    ac4_a_form_missing,
    ac4_index_form_not_under_bad_packet_index,
    ac4_a_form_without_a_member,
    ac4_a_reason_outside_the_enumeration,
    ac4_fewer_than_five,
    ac4_headerless_validity_run_is_dropped,
    ac5_wire_toml_missing,
    ac5_wire_toml_unreadable,
    ac5_no_carrying_message,
    ac5_no_slice_fields,
    ac5_no_field_agrees,
    ac5_width_not_an_integer,
    ac5_offset_mismatch,
    ac5_width_mismatch,
    ac5_max_size_a_string,
    ac5_max_size_a_float,
    ac5_max_size_too_small,
    ac5_no_bad_packet_index,
    dod_an_untagged_paragraph,
    dod_regime_stamp_line_not_covered,
    py_superscript_is_a_word_character,
    py_a_combining_mark_is_not_a_word_character,
    py_unit_separator_is_whitespace_to_lstrip,
    py_line_separator_starts_a_line,
    py_form_feed_starts_a_line,
    py_next_line_starts_a_line,
    py_bare_carriage_return_is_a_newline,
    py_dotless_i_matches_i_under_re_i,
    py_dotted_capital_i_matches_i_under_re_i,
    py_kelvin_sign_is_a_letter_to_the_unit_guard,
    py_greek_mu_folds_onto_micro,
    py_int_is_unbounded,
    py_int_unbounded_upper_bound,
    py_power_too_large_for_a_float_crashes,
    py_frames_too_large_for_a_float_crashes,
    py_separator_without_dash_is_no_table,
    toml_hex_offset,
    toml_duplicate_key_last_wins,
    toml_reasons_a_string,
    toml_message_a_number_crashes,
    ac1_no_unit,
    ac5_sub_byte_offset_outside_its_byte,
    ten_found_names_shrink_the_enumeration,
    nine_found_names_keep_the_thirteen,
);
