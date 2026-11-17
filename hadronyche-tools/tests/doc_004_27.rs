//! Ticket 004-27 — path-MTU probe pair: the probe request and reply, their
//! exemption and their validity rules. Port of `checks/004-27.sh` (deleted
//! with this port; `doccheck-equiv 004-27` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_27`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-27.sh`): `cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` exits
//! 0 with 0 failed and at least 12 passed (the count that stood before this
//! ticket). The script's DoD "the checker reads the working tree only and
//! runs no other check" (it grepped its own text for a ref naming the main
//! branch and for a sibling check) is now the library-wide test
//! `doccheck::checks::tests::checks_read_only_their_docs`, which fails if
//! any check or test file uses a process, a file, the environment, git or
//! any document source other than the `Docs` it is given.
//!
//! Numbers are read at run time, as in the script: C-06 (the fallback MTU),
//! C-17 (the control bound), C-13 (CS5), the hello's common header and its
//! type-code block, the admission request's intended packet size row, the
//! refusal enumeration, and the Rejection reasons with their Layers cells.

use hadronyche_tools::doccheck::checks::c004_27::SPEC;
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

/// AC1 — "The probe pair is specified with sizes, DF requirement and
/// matching rule, and the fallback sequence ends in `PathMtuTooSmall` when
/// the 1 280-byte probe fails (C-173)."
///
/// Script: the message tables (the seven common control header rows in the
/// hello's order and widths, a Message type code in the block the hello
/// gives the probe pair and assigns to 004-27, the rows tiling from byte 0,
/// a "maximum size ... N bytes" sentence equal to the sum, a zero-valued
/// padding row in the request only, the request's intended packet size at
/// the admission request's width and an identifier, the reply's request id
/// and echoed size) and the rule sentences (the size rule, DF with rule 11,
/// IPv4 only with C-100, CS5 (40) with C-13, the owner's [decision] ruling
/// on the small acknowledgement, no "padded" reply, the no-reply outcome,
/// the fallback sequence in order ending in PathMtuTooSmall, C-06 and C-173
/// cited); the toml code field inside the block; the fallback arithmetic
/// C-06 − 28 = C-17.
#[test]
fn ac1_the_probe_pair_is_specified_with_sizes_df_matching_and_the_fallback_sequence() {
    assert_criterion(&report(), "AC1");
}

/// AC2 — "The exemption from C-17 cites C-116 and is the only one in the
/// control layer."
///
/// Script: the exemption cites C-116 and is carried by exactly the probe
/// request in wire.toml (the header says "the probe messages"; the code
/// requires the request alone and no exemption on the reply, and the port
/// follows the code); every sentence of the section that says "exempt"
/// cites C-116 only (C-17 is allowed as the bound it exempts from).
#[test]
fn ac2_the_exemption_cites_c116_and_is_the_only_one() {
    assert_criterion(&report(), "AC2");
}

/// AC3 — "At least 4 invalid forms with outcomes."
///
/// Script: at least 4 invalid forms with outcomes; the probe-size, non-zero
/// padding, reply matching and generic control rows each have their
/// outcome; every outcome names an enumerated reason, and every packet
/// reason named lists the control layer; the reply length rule (a row or
/// the shared rule cited); the Rejection reasons enumeration has 13
/// members.
#[test]
fn ac3_at_least_four_invalid_forms_with_outcomes() {
    assert_criterion(&report(), "AC3");
}

/// AC4 — "Passes the 004-02 checker with the exemption attribute."
///
/// Script (document part): wire.toml parses; each probe message: layer
/// control, dscp 40 (C-13), a max_size above the bound for the request and
/// within it for the reply (no padding, no counted field), fields tiling
/// from bit 0 to max_size, rejection reasons that are members of the closed
/// enumeration; the Markdown and toml rows agree row for row; the refusal
/// enumeration of wire.toml equals wire.md's 18. The checker run itself is
/// in scripts/repo-state.d/004-27.sh.
#[test]
fn ac4_wire_toml_agrees_with_the_tables_and_carries_the_exemption() {
    assert_criterion(&report(), "AC4");
}

/// DoD — "Every new factual claim carries an evidence tag": every prose
/// paragraph of 5+ words (policy `PROSE_004_27`) and every `###` subsection
/// of the probe section carries a tag.
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(&report(), "DoD", "evidence tag");
}

/// DoD — "No new open item without an owner and a falsifier": every "Open
/// item" paragraph names an owner (a ticket or an epic) and a falsifier.
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(&report(), "DoD", "open item");
}

/// DoD — "Tests green in CI, unprivileged" and "The machine-readable tables
/// parse and round-trip": the document side is AC4's parse and agreement;
/// the tool side (wire-check, wire_tables) is in scripts/repo-state.d/004-27.sh.
#[test]
fn dod_the_tables_parse() {
    assert_none(&report(), "AC4", "does not parse");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script fails every criterion, as
/// the script did. A missing commit FAILS unless `DOCCHECK_ALLOW_SHALLOW=1`
/// (then a visible SKIPPED line); a run prints a visible EXECUTED line.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "a2a5ae1b27714ac69389d9de762ff782167f8c01",
        &["AC1", "AC2", "AC3", "AC4", "DoD"],
    );
}

// ---------------------------------------------------------------- denials

// One test per entry of the check's denial list
// (`src/doccheck/checks/c004_27/denials.rs`), the same list the harness runs
// against the old script (`doccheck-equiv 004-27 --denials-only`).
hadronyche_tools::denial_tests!(SPEC;
    ac1_denial_no_probe_section,
    ac1_denial_the_placeholder,
    ac1_denial_a_header_row_missing,
    ac1_denial_rows_that_do_not_tile,
    ac1_denial_a_maximum_size_that_differs_from_the_rows,
    ac1_denial_a_type_code_outside_the_block,
    ac1_denial_the_block_not_assigned_to_this_ticket,
    ac1_denial_no_padding_row_in_the_request,
    ac1_denial_no_echoed_size_in_the_reply,
    ac1_denial_no_matching_subsection,
    ac1_denial_no_df_rule,
    ac1_denial_no_owner_ruling_on_the_acknowledgement,
    ac1_denial_a_padded_reply,
    ac1_denial_the_fallback_sequence_out_of_order,
    ac1_denial_the_fallback_arithmetic_broken,
    ac1_denial_the_admission_request_intended_size_changed,
    ac1_denial_the_toml_type_code_outside_the_block,
    ac2_denial_the_reply_also_exempt,
    ac2_denial_an_exemption_citing_another_constant,
    ac3_denial_fewer_than_four_forms,
    ac3_denial_an_outcome_naming_no_member,
    ac3_denial_non_zero_padding_without_reserved_non_zero,
    ac3_denial_no_unmatched_reply_row,
    ac3_denial_no_probe_size_row,
    ac3_denial_the_enumeration_not_thirteen,
    ac4_denial_the_reply_above_the_bound,
    ac4_denial_the_wrong_dscp,
    ac4_denial_a_refusal_missing_from_wire_toml,
    ac4_denial_wire_toml_does_not_parse,
    ac4_denial_no_probe_reply_in_wire_toml,
    dod_denial_an_untagged_paragraph,
    dod_denial_an_untagged_subsection,
    dod_denial_an_open_item_without_an_owning_ticket,
    denial_wire_md_missing,
    review_1c_a_line_separator_starts_an_untagged_subsection,
    review_1d_a_form_feed_starts_an_untagged_subsection,
    review_1e_rejection_reasons_as_a_string,
    review_1f_a_triple_quoted_string_the_subset_reader_rejects,
    review_stricter_dscp_written_as_a_float,
    review_2_a_separator_with_an_inner_colon,
);
