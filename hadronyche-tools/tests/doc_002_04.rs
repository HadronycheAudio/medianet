//! Ticket 002-04 — reading notes on `sender.rs`, `stats.rs` and `tui.rs`.
//! Port of `checks/002-04.sh` (deleted with this port;
//! `doccheck-equiv 002-04` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c002_04`; these tests
//! run it on the repository's documents (one `#[test]` per criterion the
//! script groups its assertions under) and on denial fixtures. The bash
//! semantics come from `doccheck::family002`, proven against the real
//! tools. Criteria are the banners of the script's `bad` calls:
//! `Helpers`, `Locate`, `AC 1`..`AC 3`, `Context fact`, `DoD`,
//! `Section marker`.
//!
//! Header and code (the port follows the code, PORTING.md ruling 8): the
//! header asks for "exactly one heading whose text names sender.rs ...
//! three distinct per-file headings"; the code's "the same heading" failure
//! cannot fire, since a per-file heading names exactly one file (kept in
//! the port, no denial possible). The header says a hop's block is "one
//! line or that line plus the next"; the code clips the next line at the
//! sub-section's end, and takes the FIRST pointer line that satisfies
//! every requirement.
//!
//! No repository-level assertion: nothing moved to `scripts/repo-state.d/`.

use hadronyche_tools::doccheck::checks::c002_04::SPEC;
use hadronyche_tools::doccheck::family002::unproven_patterns;
use hadronyche_tools::doccheck::testing::{check_denial, red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_clean, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

/// Structure (banners "Helpers" and "Locate"): the report; section (d) and
/// an (e) after it; one heading per file naming only that file (umbrella
/// headings skipped), deeper than (d), the three sub-sections disjoint.
#[test]
fn structure_three_disjoint_per_file_subsections_under_d() {
    assert_criterion(&report(), "Helpers");
    assert_criterion(&report(), "Locate");
}

/// AC1 — "About half a page per file, with file:line pointers to that
/// file." Script: at most 30 non-empty lines, at least 3 distinct pointers,
/// at least 2 to its own file.
#[test]
fn ac1_each_subsection_is_compact_and_cites_its_own_file() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The three hops are described: field, width, unit, [code]."
/// Script: per hop, the first pointer line (sender.rs:435, stats.rs:324,
/// tui.rs:107 to 114) whose block holds every required word.
#[test]
fn ac2_the_three_hops_are_described() {
    assert_criterion(&report(), "AC2");
}

/// Context fact (input to 002-08) — the stats.rs note says the stored loss
/// is the latest record's, pooled across return streams, with a pointer
/// and a tag.
#[test]
fn context_fact_the_loss_value_is_the_latest_records_across_streams() {
    assert_criterion(&report(), "Context fact");
}

/// AC3 — "TX pacing lateness: where it is computed and its unit." Script:
/// a line naming it whose block has a sender.rs pointer, a time unit and a
/// tag.
#[test]
fn ac3_tx_pacing_lateness_has_its_site_and_unit() {
    assert_criterion(&report(), "AC3");
}

/// DoD — pointer lines tagged (Tag6) on the line or the next; open items
/// owned.
#[test]
fn dod_pointer_lines_are_tagged_and_open_items_owned() {
    assert_criterion(&report(), "DoD");
}

/// Section marker — exactly one `verdict:` line in section (d), pending or
/// a final form.
#[test]
fn section_marker_one_verdict_line_in_an_allowed_form() {
    assert_criterion(&report(), "Section marker");
}

/// The whole check passes on the working tree.
#[test]
fn the_check_passes_on_the_working_tree() {
    assert_clean(&report());
}

/// Every grep and sed pattern the port used, on the working tree and every
/// denial, is one the committed tables prove.
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
/// the three sub-sections do not exist yet (`Locate`).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "741de631309b54841f69019e4f901c9051302000",
        &["Locate"],
    );
}

// One test per entry of the denial list (`src/doccheck/checks/c002_04/denials.rs`),
// the same list the harness runs against the old script.
hadronyche_tools::denial_tests!(
    SPEC;
    helpers_denial_report_missing,
    locate_denial_no_section_d,
    locate_denial_no_section_e,
    locate_denial_no_sender_heading,
    locate_denial_no_stats_heading,
    locate_denial_no_tui_heading,
    locate_semantics_an_umbrella_heading_is_no_subsection,
    locate_denial_two_sender_headings,
    locate_denial_two_stats_headings,
    locate_denial_two_tui_headings,
    locate_semantics_a_fenced_heading_counts,
    locate_denial_a_subsection_not_deeper_than_d,
    locate_denial_a_subsection_inside_another,
    locate_denial_a_subsection_inside_a_later_one,
    locate_semantics_a_tab_after_the_hashes,
    ac1_denial_too_long,
    ac1_denial_fewer_than_three_pointers,
    ac1_denial_fewer_than_two_own_pointers,
    ac2_denial_sender_hop_not_mentioned,
    ac2_denial_sender_hop_lacks_its_width,
    ac2_denial_stats_hop_lacks_the_tag,
    ac2_denial_tui_hop_lacks_the_branch,
    ac2_semantics_a_superscript_in_the_tui_range,
    ac2_semantics_the_words_fold_case,
    context_denial_no_streams_line,
    context_denial_no_pooled_marker,
    ac3_denial_no_tx_line,
    ac3_denial_tx_line_without_a_unit,
    ac3_semantics_underscore_ms_is_no_unit,
    dod_denial_a_pointer_line_without_a_tag,
    dod_semantics_standard_is_a_tag_here,
    dod_denial_an_open_item_without_owner,
    section_marker_denial_no_verdict_line,
    section_marker_denial_two_verdict_lines,
    section_marker_denial_a_verdict_in_no_allowed_form,
    semantics_crlf_line_ends,
);
