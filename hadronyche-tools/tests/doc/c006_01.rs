//! Ticket 006-01 — timeline spec skeleton and constants table. Port of
//! `checks/006-01.sh` (deleted with this port; `doccheck-equiv 006-01` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c006_01`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion) and on denial fixtures: the real documents with one mutation
//! applied in memory, each also run against the old script by the
//! harness (`doccheck-equiv 006-01 --denials-only`).
//!
//! The script has no repository-level assertion. Its header says it
//! validates "the ticket's acceptance criteria"; its code is what the port
//! follows. Two assertions are unreachable: "is empty" (a section's text
//! always includes its heading) and "has N rows; expected exactly 47" (no
//! missing and no extra id leaves exactly 47 distinct ids).
//!
//! Semantics the script has of its own, each with a denial below: a fence is
//! closed only by the same character, at least as long, with nothing after
//! it (`fence_closer_with_an_info_string`, `fence_closed_by_the_other_character`);
//! a heading is matched by stemmed tokens at level 2 or deeper; rows are
//! split by walking characters; values are compared after NFKC.

use hadronyche_tools::doccheck::assert_criterion;
use hadronyche_tools::doccheck::checks::c006_01::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};

/// AC1 — "`doc/design/spec/timeline.md` exists and its first section after
/// the introduction is the constants table with exactly the 47 rows above,
/// whose values match `doc/design/02-constants.md`."
#[test]
fn ac1_the_constants_table_comes_first_and_matches() {
    assert_criterion(&report_of(&SPEC), "AC1");
}

/// AC2 — "Every section listed in Work step 1 is present as a heading."
/// (and each placeholder names the ticket that fills it, the DoD item).
#[test]
fn ac2_every_section_is_present_and_names_its_ticket() {
    assert_criterion(&report_of(&SPEC), "AC2");
}

/// AC3 — "The retired POC behaviour note carries the `[code]` tag and cites
/// `module.rs:1630-1634`."
#[test]
fn ac3_the_retired_poc_note() {
    assert_criterion(&report_of(&SPEC), "AC3");
}

/// AC4 — "The file renders as Markdown without errors."
#[test]
fn ac4_the_markdown_parses() {
    assert_criterion(&report_of(&SPEC), "AC4");
}

/// The tree at the commit that added the script fails the criteria the
/// script failed.
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    red_phase(
        &SPEC,
        "532d9ca1bcafb128119a2d0ffed3f1af6752761d",
        &["AC1", "AC2", "AC3", "AC4"],
    );
}

hadronyche_tools::denial_tests!(
    SPEC;
    denial_timeline_missing,
    denial_constants_missing,
    ac1_denial_the_source_lacks_an_id,
    ac1_denial_no_constants_table,
    ac1_denial_no_status_column,
    ac1_denial_a_row_with_an_unexpected_id,
    ac1_denial_a_duplicate_row,
    ac1_denial_a_missing_id,
    ac1_denial_an_extra_id,
    ac1_denial_a_value_that_differs,
    ac1_denial_a_status_that_differs,
    ac1_denial_c61_not_pending_simulation,
    ac1_denial_the_table_is_not_an_introduced_section,
    ac1_denial_another_section_comes_first,
    ac1_denial_the_table_is_under_a_level_3_heading,
    ac1_denial_no_sentence_says_the_constants_table_wins,
    ac2_denial_a_section_missing,
    ac2_denial_a_placeholder_without_its_ticket,
    ac2_denial_a_range_that_stops_short,
    ac2_denial_a_stemmed_heading_still_matches_but_a_synonym_does_not,
    ac3_denial_no_retired_note,
    ac3_denial_no_code_tag,
    ac3_denial_no_citation,
    ac3_denial_a_note_that_is_not_a_heading_still_needs_the_tag,
    ac4_denial_an_unclosed_fence,
    fence_closer_with_an_info_string,
    fence_closed_by_the_other_character,
    ac4_denial_an_indented_heading,
    ac4_denial_a_row_wider_than_its_header,
    ac4_denial_a_separator_narrower_than_the_header,
    ac2_denial_a_superscript_in_a_heading,
    a_superscript_minus_in_a_heading_passes,
    greek_mu_for_the_micro_sign_is_outside_the_repertoire,
    a_leading_tab_before_a_row_passes,
    a_trailing_tab_after_a_row_passes,
    crlf_line_ends_pass,
    a_range_of_exactly_thirty_passes,
    a_range_of_thirty_one_fails,
    a_range_with_an_en_dash_passes,
    a_range_with_to_passes,
    a_reversed_range_fails,
);
