//! Ticket 004-22 — traceability table and review against the architecture
//! specification. Port of `checks/004-22.sh` (deleted with this port; it is
//! in git history and `doccheck-equiv 004-22` still reads it from there).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_22`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory.
//!
//! The script's two repository-level assertions are not here (they run
//! tools): `cargo run --bin wire-check` exits 0, and `cargo test -p
//! hadronyche-tools --test wire_tables` reports at least 12 passed and 0
//! failed. They are in `scripts/repo-state.d/004-22.sh`.
//!
//! Declared limits, carried from the script header (verified by review
//! only, not by this check): references RESOLVE — a reference names a real
//! heading of wire.md and a real field of wire.toml; the check does not
//! verify that the named section really implements the rule or constant, so
//! a row that points at a real but wrong section passes. The completeness of
//! the review record (every difference of epic 004) is not verified; the
//! record must claim only what it found.

use hadronyche_tools::doccheck::checks::c004_22::SPEC;
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

/// AC1 — "All 22 numbered rules have at least one mapped field or validity
/// rule (epic AC-1)."
///
/// Script: the Traceability section has a table with one row per rule 1..22
/// (first cell "Rule N" or "N"; each number once). Each row's mapping cell
/// (every cell after the first) is non-empty and holds at least one
/// reference that resolves: a heading of wire.md (plain name, "Section,
/// Subsection", or a markdown link to an anchor). Every markdown anchor in
/// the section resolves. A backticked snake_case field name must be a field
/// of wire.toml.
#[test]
fn ac1_every_rule_maps_to_a_field_or_validity_rule() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "All listed constants and rulings are mapped, each wire-visible one
/// to a field with width and unit (epic AC-8)."
///
/// Script: every constant of the ticket's "Constants traced:" list (ranges
/// expanded, read from the ticket) is named in the CONSTANT cell (the first
/// cell) of a row of the constants table, grouped rows listing every id they
/// cover; a mention in a rule row or in another row does not count, and the
/// row's mapping must resolve. A constant the ticket marks wire-visible (the
/// ids after "wire-visible rulings") needs such a row that states, IN THE
/// ROW, each named backticked wire.toml field with its width and a unit,
/// written as "`name` (N bits, unit)" or "`a`, `b` (N bits each, unit)" (N
/// equal to the wire.toml width, bytes converted to bits), or the unit as
/// "unit: none (reason)". Every backticked field of such a row needs a
/// width. A row that says a constant is NOT wire-visible gives a reason of
/// at least three words.
#[test]
fn ac2_every_listed_constant_is_mapped_wire_visible_ones_with_width_and_unit() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "The no-per-receiver-field audit covers every media field and finds
/// none (epic AC-5)."
///
/// Script: a section headed "audit" lists every field of the media-layer
/// messages of wire.toml (the media packet with its announcement and slice
/// checksum sections, the content-message header, the four content messages
/// and the sparse keyed entry), each named (backticked) in the FIELD column
/// (the column whose header is "field", else the first) of a row; a mention
/// in another row's reason does not count. Each row has a verdict. A row
/// with a destination-specific or per-receiver verdict STOPS the ticket: it
/// fails with the stop message, and its field must also appear in the
/// review record's discrepancies. The audit states that no
/// destination-specific or per-receiver field exists.
#[test]
fn ac3_the_audit_covers_every_media_field_and_finds_none() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "A review record exists listing reviewer, date, discrepancies and
/// resolutions, with the traceability table attached."
///
/// Script: a review record (a wire.md section headed "Review record", or a
/// file under REPORTS_DIR linked from the Traceability section) states the
/// reviewer, a date, the protocol and packetisation heading of
/// 01-architecture.md (it must exist), a discrepancy list whose every row
/// has a resolution (or an explicit "no discrepancy"), a resolution that
/// changes another design document names 004-23 or an owner, and that the
/// traceability table is attached (a link to the Traceability anchor or the
/// table itself). Work 3: every subsection 3.N of the architecture's
/// protocol section is either named by a discrepancy row/item or stated to
/// have no difference; the known contradictions (the copy-index
/// "retransmitting" wording against § 3.7, the "verified by the receiving
/// peer" marking of § 3.5 when the architecture says it, and the
/// residence_time sentences that contradict each other) must each have a
/// discrepancy row for the design owner. The record must not claim to hold
/// every known difference, and must not say the ticket allows its
/// placement. The record's links resolve (the script's DoD line; its code
/// reports them under AC4, and the port follows the code).
#[test]
fn ac4_a_review_record_lists_reviewer_date_discrepancies_and_resolutions_with_the_table_attached() {
    assert_criterion(&report(), "AC4");
}

/// DoD — "`wire.md` reviewed against the architecture specification section
/// on protocol and packetisation, with the traceability table attached":
/// the same assertions as AC4 (the architecture heading named, Work 3, the
/// table attached); the script checks it there.
#[test]
fn dod_reviewed_against_the_architecture_with_the_table_attached() {
    let r = &report();
    assert_none(r, "AC4", "architecture");
    assert_none(r, "AC4", "attached");
    assert_none(r, "AC4", "Work 3");
}

/// DoD — "Every new factual claim carries an evidence tag." Script: every
/// paragraph of 5+ words in the Traceability section, the audit and the
/// review record carries an evidence tag (policy `UNITS_004_22`: the six
/// tags in lower case; a list item is its own paragraph; words are `\S+`
/// after links are reduced to their text).
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(&report(), "DoD", "untagged");
}

/// DoD — "No new open item without an owner and a falsifier." Script: every
/// paragraph of the three scopes that mentions "open item" names an owner
/// and a falsifier.
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(&report(), "DoD", "open item");
}

/// DoD (script header): "no `main` ref" in the three scopes.
#[test]
fn dod_no_main_ref() {
    assert_none(&report(), "DoD", "main ref");
}

/// Everything else the port reports (the inputs, `WIRE`).
#[test]
fn the_inputs_are_readable() {
    assert_criterion(&report(), "WIRE");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script fails every criterion, as
/// the script did. A missing commit FAILS unless `DOCCHECK_ALLOW_SHALLOW=1`
/// (then a visible SKIPPED line); a run prints a visible EXECUTED line.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "38e890034cfa6d210853a3fc71db4e74459fec71",
        &["AC1", "AC2", "AC3", "AC4", "DoD"],
    );
}

// ---------------------------------------------------------------- denials

// One test per entry of the check's denial list
// (`src/doccheck/checks/c004_22/denials.rs`), the same list the harness runs
// against the old script (`doccheck-equiv 004-22 --denials-only`).
hadronyche_tools::denial_tests!(
    SPEC;
    ac1_denial_a_missing_rule_row,
    ac1_denial_a_rule_with_two_rows,
    ac1_denial_an_empty_mapping,
    ac1_denial_a_mapping_that_resolves_to_no_heading,
    ac1_denial_a_dangling_anchor,
    ac1_denial_a_link_to_a_missing_file,
    ac1_denial_a_field_that_is_not_in_wire_toml,
    ac1_denial_no_traceability_section,
    ac2_denial_a_constant_named_only_outside_its_constant_cell,
    ac2_denial_a_constant_row_whose_mapping_resolves_nowhere,
    ac2_denial_a_width_that_differs_from_wire_toml,
    ac2_denial_a_width_without_a_unit,
    ac2_denial_a_field_named_without_its_width,
    ac2_denial_not_wire_visible_without_a_reason,
    ac2_denial_the_ticket_has_no_constants_list,
    ac3_denial_a_media_field_missing_from_the_field_column,
    ac3_denial_a_per_receiver_verdict_stops_the_ticket,
    ac3_denial_a_row_without_a_verdict,
    ac3_denial_no_conclusion,
    ac3_denial_no_audit_section,
    ac4_denial_no_review_record,
    ac4_denial_no_reviewer,
    ac4_denial_no_date,
    ac4_denial_the_architecture_heading_is_not_named,
    ac4_denial_a_discrepancy_without_a_resolution,
    ac4_denial_a_resolution_changing_another_document_without_an_owner,
    ac4_denial_the_table_is_not_attached,
    ac4_denial_a_dangling_link_in_the_record,
    ac4_denial_work_3_a_subsection_neither_named_nor_cleared,
    ac4_denial_the_copy_index_contradiction_is_not_recorded,
    ac4_denial_the_verified_marking_is_not_recorded,
    ac4_denial_the_residence_contradiction_is_not_recorded,
    ac4_denial_the_record_claims_every_known_difference,
    ac4_denial_the_record_says_the_ticket_allows_its_placement,
    dod_denial_an_untagged_paragraph,
    dod_denial_a_tag_in_the_wrong_case_does_not_count,
    dod_denial_an_open_item_without_a_falsifier,
    dod_denial_a_main_ref,
    denial_wire_md_missing,
    review_1a_a_link_climbing_above_the_repository,
    review_1b_a_superscript_in_the_traceability_heading,
    review_r001_ac1_delete_the_rule_7_row,
    review_r002_ac1_duplicate_rule_20_rule_21_renumbered,
    review_r003_ac1_break_one_anchor_copy_index_c_07_c_7,
    review_r004_ac1_link_escaping_the_repo_root_doc_design_spec_wire_md,
    review_r005_ac1_cite_a_non_field_snake_case_name,
    review_r006_ac2_change_one_width_frames_16_8_bits,
    review_r007_ac2_drop_one_unit_structural_latency,
    review_r008_ac2_drop_one_constant_row_id_c_122_c_922,
    review_r009_ac3_one_verdict_per_receiver_magic,
    review_r010_ac3_one_verdict_emptied_version,
    review_r011_ac3_conclusion_negated,
    review_r012_ac3_one_field_dropped_from_the_audit_bar_origin,
    review_r013_ac4_date_removed,
    review_r014_ac4_reviewer_tbd,
    review_r015_ac4_resolution_column_renamed,
    review_r016_ac4_retransmi_wording_dropped_from_r_14,
    review_r017_ac4_over_claim_every_known_difference,
    review_r018_ac4_placement_claim_004_22_allows,
    review_r019_dod_strip_one_tag_purpose,
    review_r020_dod_tag_case_decision,
    review_r021_dod_main_ref,
    review_r022_dod_open_item_without_falsifier,
    review_r023_dod_paragraph_split_by_a_fence_3_3_words,
    review_r024_slug_superscript_in_the_traceability_heading,
    review_r025_slug_duplicate_slug_interplay_heading_traceability_table_1_b,
    review_r045_crlf_line_ends_in_wire_md,
    review_r049_r2_b_before_a_combining_mark_main_branch_u_0301,
    review_r050_r2_b_before_superscript_main_branch_python_no_boundary,
    review_r051_r2_s_with_x1f_separators_untagged_5_word_paragraph,
    review_r052_r2_s_in_rule_re_rule_x1c7,
    review_r053_r2_strip_duplicate_rule_7_row_prefixed_by_x1c,
    review_r054_r2_universal_newlines_lone_r_starts_other_inside_the_record,
    review_r055_r2_u_2028_is_not_a_line_break_for_split_n,
    review_r056_r2_re_i_with_dotted_capital_i_open_i_tem_without_owner,
    review_r057_r2_re_i_dotted_i_on_a_required_word_revi_ewer_line,
    review_r058_r2_d_non_ascii_rule,
    review_r059_r2_d_non_ascii_constant_id_c,
    review_r060_r2_normpath_absolute_link_doc_design_spec_wire_md,
    review_r061_r2_normpath_link_to_a_directory,
    review_r062_r2_normpath_a_inside_the_path,
    review_r063_r2_tablerules_keep_headerless_duplicate_rule_run,
    review_r064_r2_tablerules_cellwise_separator_on_the_constants_table,
    review_r065_r2_slug_combining_mark_in_heading_python_drops_it,
    review_r066_r2_tomllib_width_as_float_16_0_frames,
    review_r067_r2_tomllib_duplicate_key_invalid_toml,
    review_r100_r2_d_non_ascii_duplicate_extra_row_rule,
);
