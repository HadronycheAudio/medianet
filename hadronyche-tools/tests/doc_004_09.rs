//! Ticket 004-09 — in-band announcement layouts. Port of `checks/004-09.sh`
//! (deleted with this port; `doccheck-equiv 004-09` reads it from git
//! history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_09`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion, the amendment's DEFECT detector and the DoD) and on the denial
//! list `c004_09::denials`: one denial per assertion of the script and per
//! branch of the DEFECT detector, plus rows aimed at the Python semantics
//! its code touches. `doccheck-equiv 004-09` runs every row against the old
//! script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-09.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC6); `cargo test -p hadronyche-tools --test wire_tables`
//! stays 12 passed, 0 failed, 0 ignored (AC6); the design checker prints OK
//! (DoD). The script never ran checks/004-01.sh.
//!
//! Header and code disagree: the header says the reasons are members of
//! "the closed 13-name enumeration"; the code replaces the 13 names with
//! those found in the Rejection reasons tables when at least 10 are found
//! (004-09:645-646). The port follows the code
//! (`ten_found_names_shrink_the_enumeration`). The validity units include
//! the field tables' rows (004-09:663-670), unlike 004-06, 004-08 and
//! 004-10.

use hadronyche_tools::doccheck::checks::c004_09::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- criteria

/// AC1 — "The section defines the container and the four types, each with a
/// type code, a field table and a byte size; each announcement names a future
/// position of 64 bits." Script: the 004-06 presence flag (bit 5), a
/// container table with count, type and length, skip by length tied to the
/// same-major rule, a 64-bit position row, "future", and per type a code
/// (a code-table row or "type code N"), a field table and a size.
#[test]
fn ac1_container_and_the_four_types() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The maximum number of announcements per packet and the maximum
/// announcement bytes are stated as numbers." Script: both numbers, the
/// C-137 arithmetic (62 = 100 − 38), the maximum bytes within 62 less the
/// stated extension and checksum shares, and count × per-announcement bytes
/// (from the container table) within the maximum.
#[test]
fn ac2_maximum_count_and_bytes() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "At least 6 invalid forms with outcomes, covering the ticket's list."
/// Script: unknown type skipped by length (no member named), length overrun,
/// position not in the future and count above maximum under BadLength,
/// duplicate type with a member, gap slice with a member or a "cannot occur",
/// zero count under BadLength; no reason outside the enumeration in the
/// units or the prose blocks.
#[test]
fn ac3_six_invalid_forms() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "C-43 and C-44 restated as sender rules." Script: C-43 with 64,
/// the 32 ms arithmetic and "sender"; C-44 with 16, every member, until the
/// position passes, and "sender".
#[test]
fn ac4_c43_and_c44_as_sender_rules() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "The proposed re-alignment and transport-reference-change payloads
/// are open items with epic 005 as confirming owner and the ticket's
/// falsifier; the epoch-switch payload states whether it names the new
/// session (C-19)."
#[test]
fn ac5_open_items_and_the_epoch_switch() {
    assert_criterion(&report(), "AC5");
}

/// AC6 — "The machine-readable side." Script (document part): announcement
/// fields (media packet fields at offset 304 or more, or an "announcement"
/// message) with type, length, a 64-bit u64 position and a count whose
/// maximum is the stated one; the carrying messages list BadLength and only
/// members, with an integer max_size (the media packet's above 38).
#[test]
fn ac6_wire_toml_carries_the_announcements() {
    assert_criterion(&report(), "AC6");
}

/// The 004-09 amendment (DEFECT): the lead (C-43) binds the sender's FIRST
/// issue; a re-issue (C-44) inside the lead is valid; a receiver rejects only
/// a position not in the future. No row, field-table cell, sentence, list
/// item or wire.toml string makes the lead a validity condition; the
/// statements are present; and (`defect R6`) the Sender obligations text
/// states the stop rule and puts no announcement in the slice at the named
/// position.
#[test]
fn defect_the_lead_is_the_senders_first_issue_obligation() {
    assert_criterion(&report(), "defect");
}

/// Ruling R6 of the amendment: the named position passes once the sender's
/// slice position reaches it; no announcement rides in a slice at or after
/// it. (The script printed these as `FAIL defect R6:`, a label the harness's
/// criterion pattern does not read: under the harness they show only in the
/// exit status.)
#[test]
fn defect_r6_the_stop_rule() {
    assert_criterion(&report(), "defect R6");
}

/// DoD — every block of the section carries an evidence tag (tables and code
/// blocks inherit).
#[test]
fn dod_every_block_carries_an_evidence_tag() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (00f02b4, "failing checks
/// for the in-band announcement layouts") fails as the script did.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "00f02b4676d7dbed1f4074821bbcfd8aed732729",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"],
    );
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC;
    setup_wire_md_missing,
    setup_no_section,
    setup_placeholder,
    ac1_presence_flag_not_tied,
    ac1_no_container_table,
    ac1_no_skip_by_length,
    ac1_skip_not_tied_to_versions,
    ac1_position_not_64_bits,
    ac1_future_never_said,
    ac1_a_type_never_named,
    ac1_a_type_without_a_code,
    ac1_a_type_without_a_field_table,
    ac1_no_byte_size,
    ac2_max_count_not_stated,
    ac2_max_bytes_not_stated,
    ac2_no_cap_arithmetic,
    ac2_max_bytes_above_the_remainder,
    ac2_numbers_not_self_consistent,
    ac3_unknown_type_not_skipped,
    ac3_unknown_type_rejected,
    ac3_a_form_missing,
    ac3_count_above_not_bad_length,
    ac3_a_form_without_a_member,
    ac3_gap_form_without_an_outcome,
    ac3_a_reason_outside_the_enumeration,
    ac3_fewer_than_six,
    ac3_headerless_validity_run_is_dropped,
    ac4_c43_not_restated,
    ac4_c43_no_arithmetic,
    ac4_c43_not_a_sender_rule,
    ac4_c44_not_a_sender_rule,
    ac4_c44_not_restated,
    ac4_c44_no_member_rule,
    ac4_c44_no_duration,
    ac5_realignment_open_item_without_005,
    ac5_transport_open_item_without_005,
    ac5_no_falsifier,
    ac5_falsifier_not_the_tickets,
    ac5_epoch_switch_session_unstated,
    ac6_wire_toml_missing,
    ac6_wire_toml_unreadable,
    ac6_no_announcement_fields,
    ac6_no_type_field,
    ac6_position_not_u64,
    ac6_count_without_a_maximum,
    ac6_count_maximum_disagrees,
    ac6_no_reasons,
    ac6_bad_length_missing,
    ac6_a_reason_not_a_member,
    ac6_max_size_not_an_integer,
    ac6_media_max_size_not_past_38,
    defect_a_field_table_cell,
    defect_a_validity_row_rejects_inside_the_lead,
    defect_a_table_row_invalidates_for_the_lead,
    defect_a_prose_sentence,
    defect_a_list_item,
    defect_a_wire_toml_string,
    defect_no_reissue_valid_statement,
    defect_lead_binds_every_issue,
    defect_a_receiver_polices_the_lead,
    defect_r6_a_slice_at_the_named_position,
    defect_r6_no_stop_rule,
    defect_first_issue_not_stated,
    dod_an_untagged_paragraph,
    py_superscript_after_a_type_code,
    py_a_combining_mark_is_not_a_word_character,
    py_superscript_is_a_word_character,
    py_unit_separator_is_whitespace_to_lstrip,
    py_line_separator_starts_a_list_item,
    py_form_feed_starts_a_list_item,
    py_next_line_starts_a_list_item,
    py_bare_carriage_return_starts_a_list_item,
    py_dotless_i_matches_i_under_re_i,
    py_int_is_unbounded,
    py_separator_without_dash_is_no_table,
    toml_float_max_count,
    toml_hex_max_count,
    toml_duplicate_key_last_wins,
    toml_message_a_number_crashes,
    toml_unhashable_reason_crashes,
    ten_found_names_shrink_the_enumeration,
    nine_found_names_keep_the_thirteen,
);
