//! Ticket 002-02 — reading notes on `audio.rs`: `analyse` and
//! `analyse_voices`. Port of `checks/002-02.sh` (deleted with this port;
//! `doccheck-equiv 002-02` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c002_02`; these tests
//! run it on the repository's documents (one `#[test]` per criterion the
//! script groups its assertions under) and on denial fixtures: the real
//! report with one edit applied in memory.
//!
//! The script is bash. Its grep, sed, tr, cut and bash semantics come from
//! `doccheck::family002`, proven against the real tools; the test
//! `every_pattern_the_port_uses_is_proven` holds the port to the patterns
//! the tables prove. The script prints unlabelled `FAIL: message` lines;
//! a criterion is the banner its `bad` call sits under (`Helpers`,
//! `Locate`, `AC 1`.. `AC 5`, `DoD`), as the harness reads them.
//!
//! Header and code: the header says the sub-section ends "at the next
//! heading of the same or higher level"; the code measures a level as the
//! length of the leading run of `#` AND spaces (`${line%%[! #]*}`), so
//! `### x` is level 4 and `###\tx` level 3. The header says the verdict is
//! "one sentence"; the code also requires at least 6 words. The header
//! says a fact's keywords are on the statement line "or split over that
//! line and the immediately next one"; the code takes the next line of the
//! FILE, past the end of the sub-section. The DoD row is searched to the
//! end of the file, not the Open sub-questions section. The port follows
//! the code (PORTING.md ruling 8).
//!
//! No repository-level assertion: nothing moved to `scripts/repo-state.d/`.

use hadronyche_tools::doccheck::checks::c002_02::SPEC;
use hadronyche_tools::doccheck::family002::unproven_patterns;
use hadronyche_tools::doccheck::testing::{check_denial, red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_clean, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

/// Structure (script banners "Helpers" and "Locate"): the report exists;
/// section (d) is found, then a section (e) heading after it; under (d), a
/// heading naming `audio.rs` and `analyse`/`analysis`.
#[test]
fn structure_the_report_section_d_and_the_audio_subsection_exist() {
    assert_criterion(&report(), "Helpers");
    assert_criterion(&report(), "Locate");
}

/// AC1 — "The notes fit on one page and cite audio.rs line numbers."
/// Script: at most 45 non-empty lines; at least 4 distinct `audio.rs:N`
/// pointers (`grep -oE 'audio\.rs:[0-9]+' | sort -u`; GNU grep's `-o` reads
/// `[0-9]` by collation, so `audio.rs:6²` counts whole, `audio.rs:²` not).
#[test]
fn ac1_the_notes_are_compact_and_cite_at_least_four_pointers() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The voice comparison is answered bit-level yes or no; a no
/// names the tolerance." Script: a line reading exactly
/// `Voice comparison bit-level: yes|no` after the bullet, quote and `**`
/// stripping of `normalise` and lowercasing; for "no", one of
/// `toleran|threshold|epsilon|eps\b` (case-insensitive) in the sub-section.
#[test]
fn ac2_the_bit_level_question_is_answered_exactly() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The five loss-handling facts are confirmed or corrected against
/// the code." Script: per fact, the first line matching its anchor whose
/// block (it and the next line) holds every needle; the block carries
/// `confirmed`/`corrected` as a word and `[code]`.
#[test]
fn ac3_the_five_facts_are_confirmed_or_corrected_with_code() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "Every exclusion rule is listed with its line and its reason."
/// Script: every list entry (`-`, `*`, `+`, `>`, `N.`, `N)` or a table
/// row) naming an exclusion category, unless marked confirmed/corrected,
/// carries an `audio.rs:N` pointer and a reason word; each of the five
/// categories has such a complete entry.
#[test]
fn ac4_every_exclusion_entry_has_a_pointer_and_a_reason() {
    assert_criterion(&report(), "AC4");
}

/// AC5 — "A one-sentence verdict statement." Script: a `Verdict
/// statement:` line with text; after removing pointers, decimals,
/// `e.g.`/`i.e.` and evidence tags, at most one `.`, `!` or `?` and at
/// least 6 words holding a letter or digit.
#[test]
fn ac5_the_verdict_statement_is_one_sentence() {
    assert_criterion(&report(), "AC5");
}

/// DoD — "The open sub-question is marked answered." Script: from the Open
/// sub-questions heading on, the first line naming the bit-level voice
/// comparison is a table row containing `answered` and a standalone
/// `yes` or `no`.
#[test]
fn dod_the_open_subquestion_is_marked_answered() {
    assert_criterion(&report(), "DoD");
}

/// The whole check passes on the working tree.
#[test]
fn the_check_passes_on_the_working_tree() {
    assert_clean(&report());
}

/// Every grep and sed pattern the port gave `family002`, on the working
/// tree and on every denial, is one the committed tables prove against the
/// real tools (`data/family002/patterns.tsv`).
#[test]
fn every_pattern_the_port_uses_is_proven() {
    let _ = report();
    for d in SPEC.denials {
        check_denial(&SPEC, d.name);
    }
    let unproven = unproven_patterns();
    assert!(
        unproven.is_empty(),
        "patterns no table proves: {unproven:?}"
    );
}

/// The tree at the commit that added the script fails as the script did:
/// the audio.rs sub-section does not exist yet (`Locate`).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "36340218a01728f7c0009cfe116d089346c64e5b",
        &["Locate"],
    );
}

// One test per entry of the denial list (`src/doccheck/checks/c002_02/denials.rs`),
// the same list the harness runs against the old script
// (`doccheck-equiv 002-02 --denials-only`).
hadronyche_tools::denial_tests!(
    SPEC;
    helpers_denial_report_missing,
    locate_denial_no_section_d,
    locate_denial_no_section_e,
    locate_semantics_an_e_heading_in_a_fence_before_d_counts,
    locate_denial_no_audio_subsection,
    locate_semantics_headings_fold_case,
    locate_semantics_a_fenced_heading_is_the_subsection,
    locate_semantics_a_tab_after_the_hashes_lowers_the_level,
    locate_semantics_trailing_spaces_on_the_heading,
    ac1_denial_too_long,
    ac1_denial_fewer_than_four_pointers,
    ac1_semantics_superscript_pointers_are_not_counted,
    ac2_denial_answer_no_without_tolerance,
    ac2_semantics_eps_before_a_superscript_is_a_word_end,
    ac2_semantics_eps_before_an_underscore_is_not,
    ac2_denial_answer_not_exact,
    ac2_semantics_trailing_spaces_after_the_bold_hide_its_end,
    ac2_semantics_bullet_bold_and_case_are_tolerated,
    ac2_semantics_a_carriage_return_after_the_bold_answer,
    ac3_denial_fact_1_not_stated,
    ac3_denial_fact_1_not_marked,
    ac3_denial_fact_2_not_stated,
    ac3_denial_fact_2_not_marked,
    ac3_denial_fact_3_not_stated,
    ac3_denial_fact_3_not_marked,
    ac3_denial_fact_4_not_stated,
    ac3_denial_fact_4_not_marked,
    ac3_denial_fact_5_not_stated,
    ac3_denial_fact_5_not_marked,
    ac3_semantics_the_next_line_carries_the_marker,
    ac3_semantics_mu_glued_to_the_marker_is_a_letter,
    ac3_semantics_mu_glued_to_the_marker_hides_it,
    ac3_semantics_a_dot_is_not_in_the_digit_group_bracket,
    ac4_denial_entry_without_reason,
    ac4_denial_entry_without_pointer,
    ac4_denial_a_category_without_a_complete_entry,
    ac4_denial_no_exclusion_entry,
    ac4_semantics_a_superscript_numbered_entry_is_no_entry,
    ac4_semantics_a_superscript_numbered_entry_with_a_hyphen_is_one,
    ac5_denial_no_verdict_statement,
    ac5_denial_two_sentences,
    ac5_semantics_decimals_pointers_and_eg_are_not_terminals,
    ac5_semantics_a_superscript_decimal_alone_keeps_its_period,
    ac5_semantics_a_superscript_decimal_after_an_ascii_one_is_removed,
    dod_denial_no_open_subquestions_section,
    dod_denial_no_row,
    dod_denial_not_a_table_row,
    dod_denial_answered_without_yes_or_no,
    dod_denial_not_answered,
    dod_semantics_the_row_as_an_unterminated_last_line_is_lost,
    dod_semantics_the_row_as_a_terminated_last_line_is_found,
    semantics_crlf_line_ends,
);
