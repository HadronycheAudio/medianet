//! Ticket 004-06 — media packet header layout. Port of `checks/004-06.sh`
//! (deleted with this port; `doccheck-equiv 004-06` reads it from git
//! history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_06`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on the denial table below:
//! one row per assertion of the script, plus rows aimed at each Python
//! semantics the script's code touches. The list is `c004_06::denials`;
//! `doccheck-equiv 004-06` runs every row against the old script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-06.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC6); `cargo test -p hadronyche-tools --test wire_tables`
//! stays 12 passed, 0 failed, 0 ignored (AC6); the design checker prints OK
//! (DoD); the 004-01 check still passes (DoD).
//!
//! Header and code disagree: the header says the forms' reasons are members
//! of "the closed 13-name enumeration"; the code replaces the 13 names with
//! the ones found in the Rejection reasons tables when at least 10 are found
//! (004-06:650-653), so the enumeration may shrink to 10. The port follows
//! the code (`ten_found_names_shrink_the_enumeration`: wire.md lists 10 of the
//! names, and the code's set no longer equals wire.toml's 13).
//!
//! Unreachable in the script and kept in the port: "the invalid form gap
//! flag / invalid flags carries neither a member reason nor a named event"
//! (004-06:976): a form without a member reason already failed and
//! `continue`d at 004-06:957.

use hadronyche_tools::doccheck::checks::c004_06::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

// ---------------------------------------------------------------- criteria

/// AC1 — "The header table contains all rule-1 fields with the
/// C-10/C-11/C-07 widths: domain 16, stream 16, session 32, epoch 16, copy
/// index 2." Script: magic 16 bits at offset 0 and version 16 bits at
/// offset 2 (004-04), domain, stream number, session, timeline epoch, copy
/// index and a flags field of at least a byte naming gap and degraded; the
/// values 0x4d4e and 0x0100 stated; wire.toml's `[versioning]` agrees.
#[test]
fn ac1_every_rule_1_field_with_its_width() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The header contains no module id, no destination address, no
/// per-receiver acknowledgement field (checked field by field in a short
/// note)." Script: each kind named with a negation, a field-by-field
/// phrase, and no table field destination-specific.
#[test]
fn ac2_no_module_id_destination_or_acknowledgement() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "Header size in bytes is stated and every field offset is
/// given." Script: the offsets tile the header bit by bit (sub-byte fields
/// at their stated bits), and the total in bytes is stated in the prose.
#[test]
fn ac3_size_stated_and_offsets_contiguous() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least 10 invalid forms are listed, each with a rejection
/// reason or named event." Script: the eleven forms the ticket names, each
/// with a member reason (VersionMismatch also with its event), at least 10
/// with a member, and no reason outside the enumeration (see the module
/// comment for which enumeration).
#[test]
fn ac4_ten_invalid_forms_with_member_reasons() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "The session field's meaning cites C-19 and the degraded flag's
/// meaning cites C-122."
#[test]
fn ac5_session_cites_c19_and_degraded_cites_c122() {
    assert_criterion(&report(), "AC5");
}

/// AC6 — "The machine-readable table passes the 004-02 checker." Script
/// (document part): the media packet `[[message]]` in wire.toml (read with
/// the scripts' own reader), layer media, dscp 46 (C-13), max_size an
/// integer covering the header, members-only reasons, its fields agreeing
/// with wire.md; the `[[reject]]` names equal wire.md's enumeration.
#[test]
fn ac6_wire_toml_agrees_with_the_table() {
    assert_criterion(&report(), "AC6");
}

/// Work step 4 — "the don't-fragment requirement (rule 11) is a sender
/// obligation on the IPv4 DF bit (C-100), outside the header."
#[test]
fn df_the_dont_fragment_obligation() {
    assert_criterion(&report(), "DF");
}

/// DoD — every block of the section carries an evidence tag; a table or a
/// code block may inherit the tag of the block before it.
#[test]
fn dod_every_block_carries_an_evidence_tag() {
    assert_criterion(&report(), "DoD");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (c6a5d71, "checks for the
/// media packet header layout") fails AC1 to AC6 and DF, as the script did
/// (its DoD block needs the section, which was the placeholder).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "c6a5d71cf56bc685e11e1275b191966af4d3ef2e",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DF"],
    );
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(SPEC;
    setup_wire_md_missing,
    setup_wire_md_missing_dod,
    setup_no_section,
    setup_placeholder,
    setup_empty_section,
    setup_no_field_table,
    setup_a_row_without_a_width_drops_the_table,
    ac1_no_copy_index_field,
    ac1_domain_width,
    ac1_magic_offset,
    ac1_version_offset,
    ac1_flags_below_a_byte,
    ac1_no_magic_value,
    ac1_no_version_value,
    ac1_gap_not_named,
    ac1_degraded_not_named,
    ac1_versioning_magic,
    ac1_versioning_version,
    ac2_module_id_not_named,
    ac2_destination_not_named,
    ac2_acknowledgement_not_named,
    ac2_not_field_by_field,
    ac2_a_destination_specific_field,
    ac3_stated_bits_disagree_with_the_width,
    ac3_bits_not_whole_bytes,
    ac3_a_gap,
    ac3_an_overlap,
    ac3_ends_mid_byte,
    ac3_size_not_stated,
    ac3_no_size_at_all,
    ac4_a_form_missing,
    ac4_a_form_without_a_member,
    ac4_version_mismatch_without_its_event,
    ac4_a_reason_outside_the_enumeration,
    ac4_fewer_than_ten_forms_with_a_member,
    ac4_headerless_validity_run_is_dropped,
    ac5_session_without_c19,
    ac5_degraded_without_c122,
    ac6_wire_toml_missing,
    ac6_wire_toml_unreadable_triple_quotes,
    ac6_reject_enumeration_disagrees,
    ac6_no_media_message,
    ac6_layer,
    ac6_dscp,
    ac6_max_size_a_string,
    ac6_max_size_below_the_header,
    ac6_no_reasons,
    ac6_a_reason_not_a_member,
    ac6_no_field_agrees,
    ac6_offset_not_an_integer,
    ac6_sub_byte_offset_outside_its_byte,
    ac6_offset_mismatch,
    ac6_width_mismatch,
    df_not_stated,
    df_no_ipv4,
    df_not_a_sender_obligation,
    df_not_outside_the_header,
    df_no_c100,
    dod_an_untagged_paragraph,
    dod_a_table_after_an_untagged_paragraph,
    py_superscript_is_a_word_character,
    py_a_combining_mark_is_not_a_word_character,
    py_unit_separator_is_whitespace_to_lstrip,
    py_line_separator_starts_a_line,
    py_form_feed_starts_a_line,
    py_next_line_starts_a_line,
    py_bare_carriage_return_is_a_newline,
    py_dotless_i_matches_i_under_re_i,
    py_dotted_capital_i_matches_i_under_re_i,
    py_int_is_unbounded,
    py_int_reads_arabic_indic_digits,
    py_separator_with_colons_is_a_table,
    py_separator_without_dash_is_no_table,
    toml_float_equals_int,
    toml_hex_int,
    toml_bool_is_an_int,
    toml_bare_word_is_a_string,
    toml_duplicate_key_last_wins,
    toml_duplicate_key_same_value,
    toml_reasons_a_string,
    toml_unhashable_reason_crashes,
    toml_bad_unicode_escape_crashes,
    ten_found_names_shrink_the_enumeration,
    nine_found_names_keep_the_thirteen,
);
