//! Ticket 004-24 — regime stamp message: its sections, its version and its
//! fetch request. Port of `checks/004-24.sh` (deleted with this port;
//! `doccheck-equiv 004-24` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_24`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, listed in `c004_24/denials.rs` with the exact
//! criteria the old script fails on them (`CRASH` where it stops with a
//! Python exception); `doccheck-equiv 004-24` runs every denial against the
//! old script as well.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-24.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed (AC5, the count that stood
//! before this ticket). The script's DoD "the checker reads the working tree
//! only and runs no other check" (it grepped its own text for a ref naming
//! the main branch and for `checks/[0-9]`) is now the library-wide rule
//! (`hadronyche-tools/clippy.toml` and `doccheck::testing::source_rules`).
//!
//! Header and code: the header's AC4 says a row whose outcome is "not a
//! rejection", "ignored" or "valid" and names no Reject reason is not
//! counted; the code's pattern is `not a rejection|never a rejection|
//! \bignored\b|\bvalid\b` (it also takes "never a rejection"), and the port
//! follows the code. The header lists the toml rejection reasons under AC5;
//! the code reports them under AC4, as the port does. The header's AC1 says
//! "every section type"; the code checks the stamp section table and the
//! request (the section types are 004-25's and 004-28's), as the port does.

use hadronyche_tools::doccheck::checks::c004_24::SPEC;
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

/// AC1 — "Every section type and the request have a maximum size ≤ 1 252
/// bytes (C-17, epic AC-3)."
///
/// Script: the section and the request tile their bytes, each at most the
/// control bound (C-17, C-18); the common control header is the landed hello
/// header; the stamp version is 32 bits and compared as the hello's version
/// is; the section carries index, count, type, length and body; both message
/// types lie in the block the hello gives 004-24; the section placeholder is
/// gone; the constants and the hello's widths read.
#[test]
fn ac1_the_section_and_the_request_are_within_the_control_bound() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The maximum stamp size and per-peer storage are stated as
/// numbers."
///
/// Script: the maximum section count, the maximum stamp size and the
/// per-peer storage are stated as numbers: the count matches its valid
/// range, the stamp size is the count times the section maximum (at most the
/// count times the control bound), the storage figures are multiples of the
/// stamp size, every product is exact, and totals over the peers are the
/// per-peer figure times C-217 or C-34 with the reason stated.
#[test]
fn ac2_the_maximum_stamp_size_and_storage_are_stated_as_numbers() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The multicast-on-change, 10 s refresh and unicast fetch of C-18
/// appear as sender and receiver rules."
///
/// Script: multicast on any regime change, the refresh period of C-18, the
/// version bump, the unicast fetch of a request, skipping unknown section
/// types by their length (004-04), the complete-stamp test, discarding an
/// incomplete stamp when a newer version arrives; a sender part and a
/// receiver part; the request carries the module id; the return path is
/// tagged [decision]; the review findings (the first stamp, a new instance
/// nonce, fetch only when stale, the fetch-load open item naming 011-23 and
/// 005-07).
#[test]
fn ac3_the_c18_rules_appear_as_sender_and_receiver_rules() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "At least 7 invalid forms with outcomes."
///
/// Script: at least seven invalid forms with outcomes (a row whose outcome
/// is not a rejection and names no Reject reason is not counted); the seven
/// categories; each outcome names only members whose Layers include
/// control; OversizeControl and BadLength where required; disagreeing
/// sections name exactly one reason; one stated outcome for an identical
/// repeat; the enumeration has 13 members; the toml reasons are members and
/// cover the table's.
#[test]
fn ac4_at_least_seven_invalid_forms_with_outcomes() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "Passes the 004-02 checker."
///
/// Script (document part): wire.toml parses; a message with section_of and
/// one without carry the Markdown tables row for row; each is layer control
/// at dscp 40 (C-13), tiles to a max_size at most the bound, carries no
/// exemption, and agrees in size. The checker run is in
/// scripts/repo-state-checks.sh.
#[test]
fn ac5_wire_toml_carries_the_stamp_messages() {
    assert_criterion(&report(), "AC5");
}

/// DoD — "Every new factual claim carries an evidence tag": every prose line
/// of 5+ words (policy `LINES_004_24`) and every `###` subsection of the
/// section carries a tag.
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(&report(), "DoD", "evidence tag");
}

/// DoD — "No new open item without an owner and a falsifier": every block
/// that mentions an open item (policy `MENTION_BLOCKS_004_24`) names an
/// owner and a falsifier.
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(&report(), "DoD", "Open item");
}

/// DoD — "The machine-readable tables parse and round-trip": the document
/// side is AC5's parse and agreement; the tool side (wire-check,
/// wire_tables) is in repo-state-checks.sh.
#[test]
fn dod_the_tables_parse() {
    assert_none(&report(), "AC5", "does not parse");
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(
    SPEC;
    denial_wire_md_missing,
    denial_wire_toml_missing,
    ac1_denial_constants_unread,
    ac1_denial_a_malformed_hello_header_row,
    ac1_denial_a_hello_header_row_missing,
    ac1_denial_the_hello_width_unread,
    ac1_denial_the_block_not_assigned,
    ac1_denial_no_stamp_section,
    ac1_denial_the_placeholder,
    ac1_denial_a_malformed_stamp_row,
    ac1_denial_no_section_table,
    ac3_denial_no_request_table,
    ac1_denial_the_section_header_rows_changed,
    ac1_denial_the_version_width_differs_from_the_hello,
    ac1_denial_no_version_comparison,
    ac1_denial_no_section_length_row,
    ac1_denial_section_rows_that_do_not_tile,
    ac1_denial_the_section_maximum_not_stated,
    ac1_denial_the_request_header_rows_changed,
    ac3_denial_the_requested_module_id_narrowed,
    ac1_denial_request_rows_that_do_not_tile,
    ac1_denial_the_request_maximum_not_stated,
    ac1_denial_a_type_code_outside_the_block,
    ac2_denial_no_maximum_section_count,
    ac2_denial_the_section_count_not_stated,
    ac2_denial_the_peer_constants,
    ac2_denial_the_stamp_size_not_stated,
    ac2_denial_a_storage_figure_not_a_multiple,
    ac2_denial_an_inexact_product,
    ac2_denial_a_total_without_its_reason,
    ac3_denial_no_multicast_on_change,
    ac3_denial_no_refresh_period,
    ac3_denial_no_unicast_request,
    ac3_denial_no_version_bump,
    ac3_denial_no_unknown_type_skip,
    ac3_denial_no_incomplete_discard,
    ac3_denial_no_completeness_test,
    ac3_denial_no_sender_part,
    ac3_denial_the_return_path_not_a_decision,
    ac3_denial_no_first_stamp_rule,
    ac3_denial_no_new_instance_rule,
    ac3_denial_no_fetch_only_when_stale,
    ac3_denial_the_fetch_load_open_item_without_its_owners,
    ac4_denial_the_enumeration_not_thirteen,
    ac4_denial_no_validity_table,
    ac4_denial_fewer_than_seven_forms,
    ac4_denial_no_oversize_row,
    ac4_denial_an_outcome_naming_no_member,
    ac4_denial_oversize_not_oversize_control,
    ac4_denial_disagreeing_sections_name_two_reasons,
    ac4_denial_an_identical_repeat_named_a_rejection,
    ac4_denial_a_toml_reason_outside_the_control_layer,
    ac4_denial_a_table_reason_missing_from_wire_toml,
    ac5_denial_wire_toml_does_not_parse,
    ac5_denial_no_section_message,
    ac5_denial_no_request_message,
    ac5_denial_a_parent_that_is_not_a_name,
    ac5_denial_a_max_size_off_the_tiling,
    ac5_denial_the_section_message_not_the_table,
    ac5_denial_the_wrong_dscp,
    ac5_denial_an_exemption,
    ac5_denial_the_request_size_differs,
    dod_denial_an_untagged_line,
    dod_denial_an_untagged_subsection,
    dod_denial_an_open_item_without_an_owner,
    ac5_denial_a_multi_line_string_the_reader_rejects,
    ac4_denial_a_heading_after_a_python_line_break,
    ac1_denial_a_word_character_only_python_knows,
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
    num_toml_reasons_unhashable,
    num_toml_reasons_an_integer,
    num_request_empty_width_cell,
    num_request_row_cut_to_two_cells,
    num_figure_superscript_before,
    num_figure_superscript_after,
    num_figure_leading_zero,
    num_figure_big,
    num_figure_beyond_i128,
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
    chk_request_row_escaped_pipe,
    chk_request_row_blank_line_inside,
    chk_request_row_ragged_row,
    chk_request_row_fenced_row,
    chk_request_row_offset_double_zero_seven,
    chk_request_row_offset_plus,
    chk_request_row_offset_forty_five_digits,
);

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (045f432, "failing checks
/// for the regime stamp message and request") fails every criterion, as
/// the script did. A missing commit fails the test
/// (`DOCCHECK_ALLOW_SHALLOW=1` skips it visibly).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(&SPEC, "045f4321a06f998d5161406b56d75c9cc7ec662c", &["AC1", "AC2", "AC3", "AC4", "AC5", "DoD"]);
}
