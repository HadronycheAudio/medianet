//! Ticket 002-03 — reading notes on `transport.rs` and `control.rs`.
//! Port of `checks/002-03.sh` (deleted with this port;
//! `doccheck-equiv 002-03` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c002_03`; these tests
//! run it on the repository's documents (one `#[test]` per criterion the
//! script groups its assertions under) and on denial fixtures.
//!
//! The script is bash and awk (mawk, byte-based); the semantics come from
//! `doccheck::family002`, proven against the real tools, and the script's
//! own awk program is proven whole against mawk by
//! `unit_offenders_is_the_scripts_awk_program`. Criteria are the banners
//! of the script's `bad` calls: `Helpers`, `Locate`, `AC 1`..`AC 4`,
//! `DoD`, `Section marker`.
//!
//! Header and code (the port follows the code, PORTING.md ruling 8):
//! - the header lists the open-item words "open item", "open question",
//!   "TODO", "TBD"; the code also matches `unresolved` (inventory 5);
//! - the header says a quantity cue includes "a known dimensioned constant
//!   such as STATE_LEN, ROOM, CYCLE_SECS, ... *_NS"; the awk program tests
//!   the cue on the LOWERCASED line, so those upper-case alternatives never
//!   fire there (a line whose only cue is such a name is counted by the
//!   case-insensitive `grep -Eqi` before it, and then never inspected);
//! - the header calls `×` an arithmetic operator; the awk program tests
//!   one BYTE, so `÷` (same first byte) is one too;
//! - "a unit must follow the number within 24 characters": 24 bytes.
//!
//! No repository-level assertion: nothing moved to `scripts/repo-state.d/`.

use hadronyche_tools::doccheck::checks::c002_03::SPEC;
use hadronyche_tools::doccheck::checks::c002_03::units::unit_offenders;
use hadronyche_tools::doccheck::family002::{check_snippet, unproven_patterns};
use hadronyche_tools::doccheck::testing::{check_denial, red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_clean, assert_criterion};

fn report() -> Report {
    report_of(&SPEC)
}

/// Structure (banners "Helpers" and "Locate"): the report exists; section
/// (d) and an (e) heading after it; exactly one heading under (d) naming
/// transport.rs and one naming control.rs, two different headings, deeper
/// than (d).
#[test]
fn structure_the_two_subsections_exist_once_each_under_d() {
    assert_criterion(&report(), "Helpers");
    assert_criterion(&report(), "Locate");
}

/// AC1 — "About half a page per file, with file:line pointers." Script: per
/// sub-section at most 30 non-empty lines, at least 3 distinct pointers
/// (`[A-Za-z0-9_./-]+\.rs:[0-9]+`), at least 2 to its own file.
#[test]
fn ac1_each_subsection_is_compact_and_cites_its_own_file() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The four facts of the Context are answered or marked not in
/// these files." Script: per fact, an anchor line whose block (it and the
/// next line, clipped at the sub-section) has a pointer and the fact's
/// keywords, or a not-in-files marker, and an evidence tag.
#[test]
fn ac2_the_four_facts_are_answered_or_marked() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "Every quoted constant has a value and a unit." Script: on every
/// line with a digit and a quantity cue, the awk program finds no number
/// without a unit within 24 bytes (pointers, ids, Q-formats, time codes,
/// factors and kind-code assignments exempt).
#[test]
fn ac3_every_quoted_number_has_a_unit() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "C-30 to C-33 are matched against the code or marked not in these
/// files; a difference is flagged for 002-13." Script: per id, a line
/// naming it (or a range holding it) whose block is a tagged not-in-files
/// marker or a tagged match/differ with a pointer; a "differs" block names
/// 002-13.
#[test]
fn ac4_c30_to_c33_are_accounted_for() {
    assert_criterion(&report(), "AC4");
}

/// DoD — "Evidence tags; open items owned." Script: every pointer line has a
/// tag on it or the next line; every open item names an owner and a
/// falsifier.
#[test]
fn dod_pointer_lines_are_tagged_and_open_items_owned() {
    assert_criterion(&report(), "DoD");
}

/// Section marker — exactly one `verdict:` line in section (d): pending, or
/// a final form (`mechanism:` or `explanation:`).
#[test]
fn section_marker_one_verdict_line_in_an_allowed_form() {
    assert_criterion(&report(), "Section marker");
}

/// The whole check passes on the working tree.
#[test]
fn the_check_passes_on_the_working_tree() {
    assert_clean(&report());
}

/// The port's `unit_offenders` prints what the script's awk program printed
/// under mawk, on every corpus input (`data/family002/corpus.tsv`, snippet
/// `002-03 unit_offenders`).
#[test]
fn unit_offenders_is_the_scripts_awk_program() {
    let n = check_snippet("002-03 unit_offenders", |line| {
        unit_offenders(line)
            .into_iter()
            .flat_map(|mut o| {
                o.push(b'\n');
                o
            })
            .collect()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(n > 1000, "{n} inputs compared");
}

/// Every grep, sed and awk pattern the port used, on the working tree and
/// every denial, is one the committed tables prove.
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
/// the two sub-sections do not exist yet (`Locate`).
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "b2852626f80c43db42e3e42f9c9e54cdec4b562c",
        &["Locate"],
    );
}

// One test per entry of the denial list (`src/doccheck/checks/c002_03/denials.rs`),
// the same list the harness runs against the old script.
hadronyche_tools::denial_tests!(
    SPEC;
    helpers_denial_report_missing,
    locate_denial_no_section_d,
    locate_denial_no_section_e,
    locate_denial_no_transport_heading,
    locate_denial_no_control_heading,
    locate_denial_two_transport_headings,
    locate_denial_two_control_headings,
    locate_denial_one_heading_for_both_files,
    locate_denial_a_subsection_not_deeper_than_d,
    locate_semantics_a_fenced_heading_counts,
    locate_semantics_a_tab_after_the_hashes,
    ac1_denial_transport_too_long,
    ac1_denial_fewer_than_three_pointers,
    ac1_denial_fewer_than_two_own_pointers,
    ac1_semantics_superscript_pointers_after_an_ascii_one,
    ac2_denial_fact_answered_without_a_tag,
    ac2_denial_fact_not_stated,
    ac2_denial_fact_marked_not_in_files_without_a_tag,
    ac2_semantics_fact_marked_not_in_files_with_a_tag,
    ac3_denial_a_number_without_a_unit,
    ac3_semantics_a_unit_within_24_bytes,
    ac3_semantics_multiplication_and_division_signs_are_factors,
    ac3_semantics_an_upper_case_cue_alone_is_not_checked_by_awk,
    ac3_semantics_an_assignment_to_a_kind_code,
    ac3_semantics_an_assignment_to_a_dimensioned_name,
    ac4_denial_differs_not_flagged,
    ac4_semantics_differs_flagged_for_002_13,
    ac4_denial_not_in_files_without_a_tag,
    ac4_denial_matches_without_a_pointer,
    ac4_denial_matches_without_a_tag,
    ac4_denial_not_accounted_for,
    ac4_semantics_a_range_with_a_leading_zero,
    ac4_semantics_a_range_beyond_64_bits,
    ac4_semantics_a_range_written_with_dots,
    dod_denial_a_pointer_line_without_a_tag,
    dod_denial_an_open_item_without_owner_and_falsifier,
    dod_semantics_a_negated_open_item,
    dod_semantics_todo_inside_a_word,
    dod_semantics_the_owner_on_the_next_line,
    dod_semantics_the_next_line_is_clipped_at_the_end,
    section_marker_denial_no_verdict_line,
    section_marker_denial_two_verdict_lines,
    section_marker_denial_a_verdict_in_no_allowed_form,
    section_marker_semantics_pending_is_accepted,
    semantics_crlf_line_ends,
);
