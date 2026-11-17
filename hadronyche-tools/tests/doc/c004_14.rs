//! Ticket 004-14 — media packet size budget at 1 472 and 1 280 bytes. Port
//! of `checks/004-14.sh` (deleted with this port; `doccheck-equiv 004-14`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_14`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory. Every denial is also run
//! against the old script by the ignored test
//! `old_script_agrees_on_the_denials` (see `wp9_support`).
//!
//! Not here, because it runs a tool (it is in `scripts/repo-state-checks.sh`,
//! under 004-14): when wire.toml carries no `[budget]` table, `cargo run
//! --bin wire-check` must pass and print the default and fallback totals.
//! The script skipped this run under its `WIRE_MD`/`WIRE_TOML` overrides.
//!
//! Two assertions of the script cannot be reached by any document: the
//! `wire.toml has no [[message]] named ...` failure of `msg()` is printed
//! only with the list of failures, but every message it looks up feeds a
//! part the script requires next, so the exit "does not declare what the
//! budget needs" that follows discards it; and the `a 1 472 column has the
//! wrong bound` failure tests a bound the script has just set. The port
//! reproduces both (the first is dropped by the same exit).

use hadronyche_tools::doccheck::checks::c004_14::SPEC;
use hadronyche_tools::doccheck::mutate::{replace_all, within_section};
use hadronyche_tools::doccheck::testing::{red_phase, repo, report_of};
use hadronyche_tools::doccheck::{assert_criterion, assert_fails};

/// AC1 — "The default-packet budget shows 6 channels plus all mandatory
/// headers fit in 1 472 bytes and states the spare bytes as a number (epic
/// AC-2)."
///
/// Script: a budget table (a Total row, a Spare row and an audio row; every
/// numeric column is one case) whose default column holds the mandatory
/// parts only, each part equal to the size the landed toml declares for it
/// (audio 6 × 48 × 4 from C-04, C-02, C-03), total at most 1 472, spare =
/// 1 472 minus total; the section cites C-100 and states the 1 472, 1 280
/// and 1 252 byte figures.
#[test]
fn ac1_the_default_packet_fits_and_the_spare_is_a_number() {
    assert_criterion(&report_of(&SPEC), "AC1");
}

/// AC2 — "The worst case (all optional parts at maximum) is also shown with
/// its spare or deficit."
///
/// Script: a worst case with the announcements at maximum and the extension,
/// and one with announcements at maximum and no extension; announcement
/// rows sum to the count byte plus 12 per announcement, the extension is 0
/// or the toml's size.
#[test]
fn ac2_the_worst_case_is_shown() {
    assert_criterion(&report_of(&SPEC), "AC2");
}

/// AC3 — "The 1 280-byte fallback budget uses 1 252 bytes of UDP payload,
/// shows 6 channels fit with headers and checksum at most 100 bytes
/// (C-137), and lists the optional parts omitted there."
///
/// Script: a fallback column at bound 1 252 with non-audio bytes at most
/// the cap of C-137; the section cites C-06 and C-137; an omission rule
/// naming announcements, the measurement extension and controller
/// messages with a priority (or a sentence on the fallback saying nothing
/// is omitted); a fallback that omits an announcement or the extension must
/// say it amends or scopes the landed sender obligations.
#[test]
fn ac3_the_fallback_budget_and_the_omission_rule() {
    assert_criterion(&report_of(&SPEC), "AC3");
}

/// AC4 — "The open sub-question is answered with a verdict against its
/// falsifier."
///
/// Script: a paragraph containing "verdict"; with the mandatory headers and
/// checksum (46 bytes from the tables) within the cap it says C-137 holds
/// and not falsified; above it, falsified with "5 channels" and C-04 at 5
/// in the constants table.
#[test]
fn ac4_the_verdict_against_the_falsifier() {
    assert_criterion(&report_of(&SPEC), "AC4");
}

/// AC5 — "Totals in `wire.md` match the sums from the machine-readable
/// tables (checked by the 004-02 checker or by hand with the arithmetic
/// shown)."
///
/// Script: every total is the sum of its rows and every component equals
/// the toml-computed size; the toml `[budget]` keys are bound to their case
/// by name tokens and equal the computed figures; wire.toml's media
/// packet, checksum, count and extension sizes agree. (The `wire-check` run
/// for a toml without a budget table is in repo-state-checks.sh.)
#[test]
fn ac5_the_totals_match_the_machine_readable_tables() {
    assert_criterion(&report_of(&SPEC), "AC5");
}

/// AC6 — "The controller allowance inside the cap for a single-stream
/// source (C-181) is stated as a number."
///
/// Script: the allowance table at both path sizes (allowance = bound -
/// audio - counted headers - checksum, or the cap less counted headers and
/// checksum), the upper-bound sentence, C-181's second stream, C-205's
/// audio member, and a sentence giving the sparse entries that fit beside
/// a transport state.
#[test]
fn ac6_the_controller_allowance_is_a_number() {
    assert_criterion(&report_of(&SPEC), "AC6");
}

/// DoD — "Every new factual claim carries an evidence tag" and "No new open
/// item without an owner and a falsifier": every unit of 5 or more
/// `[A-Za-z0-9]+` words of the section carries a tag; an "Open item" unit
/// names an owner and a falsifier.
#[test]
fn dod_evidence_tags_and_open_items() {
    assert_criterion(&report_of(&SPEC), "DoD");
}

/// Pointers — "Design documents updated where behaviour differs": no open
/// item owned by 004-14 and no "But see ... Budget" pointer remains, no
/// unresolved pointer to 004-14 (Budget) outside the section, the header-cap
/// open item is closed, the landed sentences the budget contradicts are
/// removed or cite the section, no provisional cap line is left.
#[test]
fn pointers_the_landed_sections_point_at_the_budget() {
    assert_criterion(&report_of(&SPEC), "pointer");
}

/// DoD — "The machine-readable tables parse and round-trip": the document
/// side is the toml reader and the checks of AC5; the tool side is in
/// repo-state-checks.sh.
#[test]
fn dod_the_machine_readable_tables_parse() {
    let r = report_of(&SPEC);
    assert!(r.of("setup").is_empty(), "{}", r.render());
}

// ---------------------------------------------------------------- denials

/// "A falsified verdict must say C-04 falls to 5 channels" needs the
/// constants (a header cap below the mandatory 46 bytes) and wire.md (no
/// "5 channels") edited together; a denial edits one file.
#[test]
fn ac4_denial_a_falsified_verdict_without_five_channels() {
    let docs = repo()
        .mutate("doc/design/02-constants.md", |t| {
            replace_all(
                t,
                "at most 100 bytes for all headers",
                "at most 40 bytes for all headers",
            )
        })
        .mutate("doc/design/spec/wire.md", |t| {
            within_section(t, "^Size budget$", |s| {
                hadronyche_tools::doccheck::text::rx(r"(?i)5 channels|five channels")
                    .replace_all(s, "fewer channels")
            })
        });
    assert_fails(
        &SPEC.evaluate(&docs),
        "AC4",
        "must say C-04 falls to 5 channels",
    );
}

/// The tree at the commit that added the script fails the criteria the
/// script failed.
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "4ae89126ca507ec558c1453671efba975e9398bc",
        &["AC1", "AC2", "AC3", "AC4", "AC6", "DoD", "pointer"],
    );
}

hadronyche_tools::denial_tests!(
    SPEC;
    denial_wire_md_missing,
    denial_wire_toml_missing,
    denial_constants_missing,
    denial_a_constant_unreadable,
    denial_fallback_mtu_disagrees_with_the_control_bound,
    denial_wire_toml_does_not_parse,
    ac1_denial_no_size_budget_section,
    ac1_denial_a_column_not_named,
    ac1_denial_a_row_with_no_number,
    ac1_denial_a_row_not_a_budget_part,
    ac1_denial_no_spare_row_in_a_column,
    ac1_denial_a_part_row_missing,
    ac1_denial_the_packet_header_differs_from_the_toml,
    ac1_denial_the_slice_header_differs_from_the_toml,
    ac1_denial_the_content_header_differs_from_the_toml,
    ac1_denial_the_audio_differs_from_c04_c02_c03,
    ac1_denial_the_checksum_differs_from_the_toml,
    ac1_denial_a_total_over_the_bound,
    ac1_denial_a_spare_that_is_not_the_bound_less_the_total,
    ac1_denial_the_default_column_carries_an_announcement,
    ac1_denial_the_default_total_is_not_mandatory_plus_audio,
    ac1_denial_no_budget_table,
    ac1_denial_no_default_column,
    ac1_denial_c100_not_cited,
    ac1_denial_a_byte_figure_missing,
    ac2_denial_announcement_rows_not_the_count_plus_twelve_each,
    ac2_denial_an_extension_that_is_not_the_toml_size,
    ac2_denial_a_worst_column_not_at_maximum,
    ac2_denial_no_worst_case_with_the_extension,
    ac2_denial_no_worst_case_without_the_extension,
    ac3_denial_no_fallback_column,
    ac3_denial_fallback_headers_above_the_cap,
    ac3_denial_c06_not_cited,
    ac3_denial_c137_not_cited,
    ac3_denial_no_omission_rule_and_nothing_stated,
    ac3_denial_the_omission_rule_names_no_announcements,
    ac3_denial_the_omission_rule_names_no_extension,
    ac3_denial_the_omission_rule_names_no_controller_messages,
    ac3_denial_the_omission_rule_states_no_priority,
    ac3_denial_a_fallback_that_omits_without_amending,
    ac4_denial_no_verdict,
    ac4_denial_a_falsified_verdict_when_the_headers_fit,
    ac4_denial_a_verdict_that_does_not_say_it_holds,
    ac4_denial_a_holding_verdict_when_the_headers_exceed_the_cap,
    ac4_denial_a_non_fit_verdict_must_report_falsified,
    ac4_denial_a_falsified_verdict_needs_c04_at_five,
    ac5_denial_the_toml_lacks_what_the_budget_needs,
    ac5_denial_the_checksum_max_size_differs_from_its_field,
    ac5_denial_the_audio_sample_differs_from_c03,
    ac5_denial_the_announcement_count_is_not_one_byte,
    ac5_denial_the_extension_does_not_start_where_the_container_ends,
    ac5_denial_the_media_packet_max_size_is_not_the_sum,
    ac5_denial_a_total_that_is_not_the_sum_of_its_rows,
    ac5_denial_no_budget_key_for_a_case,
    ac5_denial_a_budget_key_that_is_not_the_total,
    ac5_denial_a_budget_key_that_is_not_the_part_size,
    ac5_denial_an_allowance_key_with_no_row,
    ac5_denial_an_allowance_key_that_differs_from_its_row,
    ac5_denial_an_allowance_key_with_no_allowance_table,
    ac6_denial_no_allowance_table,
    ac6_denial_an_allowance_table_without_a_column,
    ac6_denial_a_row_naming_neither_path,
    ac6_denial_a_row_with_a_missing_number,
    ac6_denial_a_bound_that_is_neither_the_cap_nor_a_payload,
    ac6_denial_the_1472_path_with_the_fallback_payload,
    ac6_denial_the_fallback_with_the_default_payload,
    ac6_denial_an_audio_that_differs,
    ac6_denial_a_checksum_that_differs,
    ac6_denial_counted_headers_that_are_not_a_landed_sum,
    ac6_denial_an_allowance_that_is_not_the_bound_less_the_parts,
    ac6_denial_a_negative_allowance,
    ac6_denial_the_allowance_at_one_path_only,
    ac6_denial_no_upper_bound_sentence,
    ac6_denial_no_second_stream_sentence,
    ac6_denial_no_audio_member_sentence,
    ac6_denial_no_sparse_entries_sentence,
    dod_denial_an_untagged_paragraph,
    dod_denial_an_open_item_without_an_owner,
    pointer_denial_an_open_item_still_owned_by_004_14,
    pointer_denial_a_but_see_budget_pointer,
    pointer_denial_an_unresolved_pointer_to_004_14,
    pointer_denial_a_provisional_cap_line,
    pointer_denial_a_landed_sentence_not_cited,
    pointer_denial_the_header_cap_open_item_not_closed,
    a_string_max_size,
    a_float_max_size,
    a_max_size_with_an_underscore_passes,
    a_hex_max_size_passes,
    a_zero_padded_max_size_passes,
    a_max_size_of_1e19,
    a_max_size_beyond_i128,
    a_boolean_max_size,
    fields_that_is_an_int_then_a_field_crashes,
    fields_that_is_a_list_of_an_int_crashes,
    a_message_name_that_is_a_list_crashes,
    a_toml_value_with_trailing_tabs_and_a_comment_passes,
    a_field_after_a_comment_line_passes,
    a_zero_sparse_entry_size,
    a_total_of_1e13,
    a_total_of_1e19,
    a_total_beyond_i128,
    a_total_with_an_underscore,
    a_total_written_1e3,
    a_total_with_a_decimal_part,
    a_total_with_a_plus_sign,
    a_spare_with_a_minus_after_a_superscript,
    a_spare_with_a_spaced_minus,
    a_spare_with_a_deficit_word,
    crlf_line_ends_pass,
    a_constant_of_1e13,
);
