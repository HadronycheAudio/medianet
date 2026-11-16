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
//! failed. They are in `scripts/repo-state-checks.sh`, under 004-22.
//!
//! Declared limits, carried from the script header (verified by review
//! only, not by this check): references RESOLVE — a reference names a real
//! heading of wire.md and a real field of wire.toml; the check does not
//! verify that the named section really implements the rule or constant, so
//! a row that points at a real but wrong section passes. The completeness of
//! the review record (every difference of epic 004) is not verified; the
//! record must claim only what it found.

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_22::{WIRE_MD, run};
use hadronyche_tools::doccheck::mutate::{
    delete_row, rename_heading, replace_all, replace_in_line, strip_tags,
};
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion, assert_fails};

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

/// The check on wire.md with `edit` applied.
fn wire(edit: impl FnOnce(&str) -> String) -> Report {
    run(&base().mutate(WIRE_MD, edit))
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
    assert_criterion(report(), "AC1");
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
    assert_criterion(report(), "AC2");
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
    assert_criterion(report(), "AC3");
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
    assert_criterion(report(), "AC4");
}

/// DoD — "`wire.md` reviewed against the architecture specification section
/// on protocol and packetisation, with the traceability table attached":
/// the same assertions as AC4 (the architecture heading named, Work 3, the
/// table attached); the script checks it there.
#[test]
fn dod_reviewed_against_the_architecture_with_the_table_attached() {
    let r = report();
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
    assert_none(report(), "DoD", "untagged");
}

/// DoD — "No new open item without an owner and a falsifier." Script: every
/// paragraph of the three scopes that mentions "open item" names an owner
/// and a falsifier.
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(report(), "DoD", "open item");
}

/// DoD (script header): "no `main` ref" in the three scopes.
#[test]
fn dod_no_main_ref() {
    assert_none(report(), "DoD", "main ref");
}

/// Everything else the port reports (the inputs, `WIRE`).
#[test]
fn the_inputs_are_readable() {
    assert_criterion(report(), "WIRE");
}

// ---------------------------------------------------------------- AC1 denials

#[test]
fn ac1_denial_a_missing_rule_row() {
    assert_fails(
        &wire(|t| delete_row(t, r"^\| Rule 7 \|")),
        "AC1",
        "no row for rule(s) [7]",
    );
}

#[test]
fn ac1_denial_a_rule_with_two_rows() {
    let r = wire(|t| replace_in_line(t, r"^\| Rule 8 \|", r"Rule 8", "Rule 7"));
    assert_fails(&r, "AC1", "rule(s) [7] have more than one row");
}

#[test]
fn ac1_denial_an_empty_mapping() {
    let r = wire(|t| replace_in_line(t, r"^\| Rule 6 \|", r"^.*$", "| Rule 6 | - | | | |"));
    assert_fails(&r, "AC1", "rule 6: the mapping cell is empty");
}

#[test]
fn ac1_denial_a_mapping_that_resolves_to_no_heading() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| Rule 6 \|",
            r"^.*$",
            "| Rule 6 | checksum | none | (No such heading) | implemented |",
        )
    });
    assert_fails(
        &r,
        "AC1",
        "rule 6: no reference in its mapping cell resolves",
    );
}

#[test]
fn ac1_denial_a_dangling_anchor() {
    let r = wire(|t| replace_in_line(t, r"^\| Rule 1 \|", r"\|\s*$", " [x](#no-such-heading) |"));
    assert_fails(
        &r,
        "AC1",
        "names an anchor that is not a heading of doc/design/spec/wire.md",
    );
}

#[test]
fn ac1_denial_a_link_to_a_missing_file() {
    let r = wire(|t| replace_in_line(t, r"^\| Rule 1 \|", r"\|\s*$", " [x](../no-such-file.md) |"));
    assert_fails(&r, "AC1", "names a file that does not exist");
}

#[test]
fn ac1_denial_a_field_that_is_not_in_wire_toml() {
    let r = wire(|t| replace_in_line(t, r"^\| Rule 1 \|", r"\|\s*$", " `no_such_field` |"));
    assert_fails(
        &r,
        "AC1",
        "cites `no_such_field`, which is not a field of wire.toml",
    );
}

#[test]
fn ac1_denial_no_traceability_section() {
    let r = wire(|t| rename_heading(t, "^Traceability table$", "Mapping table"));
    assert_fails(&r, "AC1", "has no Traceability section");
}

// ---------------------------------------------------------------- AC2 denials

#[test]
fn ac2_denial_a_constant_named_only_outside_its_constant_cell() {
    let r = wire(|t| delete_row(t, r"^\| C-105 \|"));
    assert_fails(
        &r,
        "AC2",
        "constant C-105 of the ticket's list is not named in the CONSTANT cell",
    );
}

#[test]
fn ac2_denial_a_constant_row_whose_mapping_resolves_nowhere() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| C-140 \|",
            r"\(Manifest, E and the cost bound\)",
            "(Nowhere)",
        )
    });
    assert_fails(
        &r,
        "AC2",
        "constant C-140: no row naming it in the constants table has a mapping cell that resolves",
    );
}

#[test]
fn ac2_denial_a_width_that_differs_from_wire_toml() {
    let r = wire(|t| replace_in_line(t, r"^\| C-198 \|", r"32 bits", "16 bits"));
    assert_fails(
        &r,
        "AC2",
        "`current_a` is stated at 16 bits but wire.toml gives",
    );
    assert_fails(
        &r,
        "AC2",
        "wire-visible constant C-198 has no row stating its field",
    );
}

#[test]
fn ac2_denial_a_width_without_a_unit() {
    let r = wire(|t| replace_in_line(t, r"^\| C-140 \|", r"32 bits, ns;", "32 bits;"));
    assert_fails(&r, "AC2", "no unit in the row for declared_e");
}

#[test]
fn ac2_denial_a_field_named_without_its_width() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| C-198 \|",
            r"\| implemented \|",
            "| implemented, see `declared_e` |",
        )
    });
    assert_fails(
        &r,
        "AC2",
        "`declared_e` is named in the row without its width",
    );
}

#[test]
fn ac2_denial_not_wire_visible_without_a_reason() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| C-17 \|",
            r"^.*$",
            "| C-17 | bound | Field-table format | not wire-visible |",
        )
    });
    assert_fails(
        &r,
        "AC2",
        "a row says C-17 is not wire-visible without a reason of at least three words",
    );
}

#[test]
fn ac2_denial_the_ticket_has_no_constants_list() {
    let docs = base();
    let ticket = docs.glob("doc/design/tickets/004", "004-22-")[0].clone();
    let r = run(&docs.mutate(&ticket, |t| t.replace("Constants traced:", "Constants:")));
    assert_fails(&r, "AC2", "the ticket has no 'Constants traced:' sentence");
}

// ---------------------------------------------------------------- AC3 denials

#[test]
fn ac3_denial_a_media_field_missing_from_the_field_column() {
    let r = wire(|t| delete_row(t, r"^\| `magic` \| media packet \|"));
    assert_fails(&r, "AC3", "names media field(s): magic");
}

#[test]
fn ac3_denial_a_per_receiver_verdict_stops_the_ticket() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| `magic` \| media packet \|",
            r"\| none \|",
            "| per-receiver |",
        )
    });
    assert_fails(
        &r,
        "AC3",
        "STOP: the audit finds a destination-specific or per-receiver media field (magic)",
    );
    assert_fails(
        &r,
        "AC3",
        "per-receiver field magic is not recorded as a discrepancy",
    );
}

#[test]
fn ac3_denial_a_row_without_a_verdict() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| `magic` \| media packet \|",
            r"\| none \|",
            "| maybe |",
        )
    });
    assert_fails(&r, "AC3", "the audit row for magic states no verdict");
}

#[test]
fn ac3_denial_no_conclusion() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^Conclusion: the media layer",
            r"no destination-specific and no per-receiver field",
            "only shared fields",
        )
    });
    assert_fails(&r, "AC3", "the audit's conclusion does not state");
}

#[test]
fn ac3_denial_no_audit_section() {
    let r = wire(|t| rename_heading(t, "^Per-receiver field audit$", "Per-receiver fields"));
    assert_fails(&r, "AC3", "has no section headed 'audit'");
}

// ---------------------------------------------------------------- AC4 denials

#[test]
fn ac4_denial_no_review_record() {
    let r = wire(|t| rename_heading(t, "^Review record$", "Record"));
    assert_fails(&r, "AC4", "no review record");
}

#[test]
fn ac4_denial_no_reviewer() {
    let r = wire(|t| replace_all(t, "(?i)reviewer", "checker"));
    assert_fails(&r, "AC4", "names no reviewer");
}

#[test]
fn ac4_denial_no_date() {
    let r = wire(|t| replace_in_line(t, r"^\*\*Date:\*\*", "2026-11-16", "soon"));
    assert_fails(&r, "AC4", "states no date");
}

#[test]
fn ac4_denial_the_architecture_heading_is_not_named() {
    let r = wire(|t| replace_all(t, "(?i)protocol and packetisation", "wire layout section"));
    assert_fails(
        &r,
        "AC4",
        "names no architecture heading on protocol and packetisation",
    );
}

#[test]
fn ac4_denial_a_discrepancy_without_a_resolution() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| R-1 \|",
            r"^.*$",
            "| R-1 | 3.1 Slices and packets | a | b | c | | | |",
        )
    });
    assert_fails(&r, "AC4", "a discrepancy has no resolution");
}

#[test]
fn ac4_denial_a_resolution_changing_another_document_without_an_owner() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| R-1 \|",
            r"^.*$",
            "| R-1 | 3.1 Slices and packets | a | b | c | edit 01-architecture.md | | |",
        )
    });
    assert_fails(
        &r,
        "AC4",
        "changes a design document other than wire.md without naming 004-23 or an owner",
    );
}

#[test]
fn ac4_denial_the_table_is_not_attached() {
    let r = wire(|t| replace_all(t, r"\]\(#traceability-table\)", "](#review-record)"));
    assert_fails(&r, "AC4", "the traceability table is not attached");
}

#[test]
fn ac4_denial_a_dangling_link_in_the_record() {
    let r = wire(|t| replace_in_line(t, r"^\*\*Date:\*\*", r"$", " [x](#no-such-heading)"));
    assert_fails(
        &r,
        "AC4",
        "review record: link [x](#no-such-heading) names an anchor",
    );
}

#[test]
fn ac4_denial_work_3_a_subsection_neither_named_nor_cleared() {
    let r = wire(|t| delete_row(&delete_row(t, r"^\| R-9 \| 3\.4"), r"^\| 3\.4 \|"));
    assert_fails(&r, "AC4", "Work 3: the architecture subsection 3.4");
}

#[test]
fn ac4_denial_the_copy_index_contradiction_is_not_recorded() {
    let r = wire(|t| delete_row(t, r"^\| R-14 \| 3\.7"));
    assert_fails(&r, "AC4", "naming § 3.7 and 'retransmi'");
}

/// The script accepts any table row or list item of the record that names
/// 3.5 and "verif" without saying "no difference": the no-difference row
/// "| 3.5 | three classes (C-13) | ... how the marking is verified is R-10
/// and R-13" satisfies it as well, so the fixture removes all three rows.
#[test]
fn ac4_denial_the_verified_marking_is_not_recorded() {
    let r = wire(|t| {
        let t = delete_row(t, r"^\| R-10 \| 3\.5");
        let t = delete_row(&t, r"^\| R-13 \| 3\.5");
        delete_row(&t, r"^\| 3\.5 \| three classes")
    });
    assert_fails(&r, "AC4", "naming § 3.5 and 'verified'");
}

#[test]
fn ac4_denial_the_residence_contradiction_is_not_recorded() {
    let r = wire(|t| delete_row(t, r"^\| R-12 \| 3\.2"));
    assert_fails(
        &r,
        "AC4",
        "no discrepancy row naming residence_time as a contradiction",
    );
}

#[test]
fn ac4_denial_the_record_claims_every_known_difference() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\*\*Date:\*\*",
            r"$",
            " It holds every known difference.",
        )
    });
    assert_fails(
        &r,
        "AC4",
        "the review record claims to hold every known difference",
    );
}

#[test]
fn ac4_denial_the_record_says_the_ticket_allows_its_placement() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\*\*Date:\*\*",
            r"$",
            " Ticket 004-22 allows the record to be kept here.",
        )
    });
    assert_fails(
        &r,
        "AC4",
        "says ticket 004-22 allows or permits the record's placement",
    );
}

// ---------------------------------------------------------------- DoD denials

#[test]
fn dod_denial_an_untagged_paragraph() {
    let r = wire(|t| strip_tags(t, r"^\*\*Purpose:\*\*"));
    assert_fails(
        &r,
        "DoD",
        "an untagged paragraph of 5+ words in the review record",
    );
}

#[test]
fn dod_denial_a_tag_in_the_wrong_case_does_not_count() {
    let r = wire(|t| replace_in_line(t, r"^\*\*Purpose:\*\*", r"\[decision\]", "[Decision]"));
    assert_fails(
        &r,
        "DoD",
        "an untagged paragraph of 5+ words in the review record",
    );
}

#[test]
fn dod_denial_an_open_item_without_a_falsifier() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\*\*Date:\*\*",
            r"$",
            "\n\nOpen item: the owner is 004-23 and nothing else is said `[decision]`.",
        )
    });
    assert_fails(
        &r,
        "DoD",
        "an open item in the review record names no owner or no falsifier",
    );
}

#[test]
fn dod_denial_a_main_ref() {
    let r = wire(|t| replace_in_line(t, r"^\*\*Date:\*\*", r"$", " Compared with `main`."));
    assert_fails(&r, "DoD", "the review record text refers to a main ref");
}

#[test]
fn denial_wire_md_missing() {
    let r = run(&base().without(WIRE_MD));
    assert_fails(&r, "WIRE", "cannot read wire.md");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (38e8900, "failing checks
/// for the traceability table, the per-receiver audit and the review
/// record") fails every criterion, as the script did. Skipped when the
/// commit is not in the clone (a shallow checkout).
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "38e890034cfa6d210853a3fc71db4e74459fec71";
    let repo = hadronyche_tools::doccheck::docs::repo_root();
    let present = std::process::Command::new("git")
        .args([
            "--no-optional-locks",
            "cat-file",
            "-e",
            &format!("{commit}^{{commit}}"),
        ])
        .current_dir(&repo)
        .status()
        .is_ok_and(|s| s.success());
    if !present {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo, commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "DoD"],
        "{}",
        r.render()
    );
}
