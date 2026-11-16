//! Ticket 004-27 — path-MTU probe pair: the probe request and reply, their
//! exemption and their validity rules. Port of `checks/004-27.sh` (deleted
//! with this port; `doccheck-equiv 004-27` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_27`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory.
//!
//! Not here, because they run tools or inspect the repository (they are in
//! `scripts/repo-state-checks.sh`, under 004-27): `cargo run --bin
//! wire-check` reports "checks clean"; `cargo test -p hadronyche-tools
//! --test wire_tables` exits 0 with 0 failed and at least 12 passed (the
//! count that stood before this ticket); and the checker reads the working
//! tree only and runs no other check (the script grepped its own text for
//! a repository ref naming the main branch and for a sibling check; the
//! relocated assertion greps this port's sources for the same ref pattern
//! and for a spawned process, the only way a Rust check could run another
//! check or read a ref).
//!
//! Numbers are read at run time, as in the script: C-06 (the fallback MTU),
//! C-17 (the control bound), C-13 (CS5), the hello's common header and its
//! type-code block, the admission request's intended packet size row, the
//! refusal enumeration, and the Rejection reasons with their Layers cells.

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_27::{WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{
    delete_row, delete_section, delete_unit, rename_heading, replace_all, replace_in_line,
    strip_tags, within_section,
};
use hadronyche_tools::doccheck::ticket::CONSTANTS_MD;
use hadronyche_tools::doccheck::wire_toml::WIRE_TOML;
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion, assert_fails};

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

fn wire(edit: impl FnOnce(&str) -> String) -> Report {
    run(&base().mutate(WIRE_MD, edit))
}

/// The check on wire.md with `edit` applied to the probe section only.
fn probe(edit: impl FnOnce(&str) -> String) -> Report {
    wire(|t| within_section(t, "^Path-MTU probe pair$", edit))
}

fn toml(edit: impl FnOnce(&str) -> String) -> Report {
    run(&base().mutate(WIRE_TOML, edit))
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
    assert_criterion(report(), "AC1");
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
    assert_criterion(report(), "AC2");
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
    assert_criterion(report(), "AC3");
}

/// AC4 — "Passes the 004-02 checker with the exemption attribute."
///
/// Script (document part): wire.toml parses; each probe message: layer
/// control, dscp 40 (C-13), a max_size above the bound for the request and
/// within it for the reply (no padding, no counted field), fields tiling
/// from bit 0 to max_size, rejection reasons that are members of the closed
/// enumeration; the Markdown and toml rows agree row for row; the refusal
/// enumeration of wire.toml equals wire.md's 18. The checker run itself is
/// in scripts/repo-state-checks.sh.
#[test]
fn ac4_wire_toml_agrees_with_the_tables_and_carries_the_exemption() {
    assert_criterion(report(), "AC4");
}

/// DoD — "Every new factual claim carries an evidence tag": every prose
/// paragraph of 5+ words (policy `PROSE_004_27`) and every `###` subsection
/// of the probe section carries a tag.
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(report(), "DoD", "evidence tag");
}

/// DoD — "No new open item without an owner and a falsifier": every "Open
/// item" paragraph names an owner (a ticket or an epic) and a falsifier.
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(report(), "DoD", "open item");
}

/// DoD — "Tests green in CI, unprivileged" and "The machine-readable tables
/// parse and round-trip": the document side is AC4's parse and agreement;
/// the tool side (wire-check, wire_tables) is in repo-state-checks.sh.
#[test]
fn dod_the_tables_parse() {
    assert_none(report(), "AC4", "does not parse");
}

// ---------------------------------------------------------------- AC1 denials

#[test]
fn ac1_denial_no_probe_section() {
    let r = wire(|t| rename_heading(t, "^Path-MTU probe pair$", "Probe pair"));
    assert_fails(&r, "AC1", "has a '## Path-MTU probe pair' section");
}

#[test]
fn ac1_denial_the_placeholder() {
    let r = wire(|t| replace_in_line(t, "^The path-MTU probe pair is how", "^", "To be filled. "));
    assert_fails(&r, "AC1", "not the placeholder");
}

#[test]
fn ac1_denial_a_header_row_missing() {
    let r = wire(|t| delete_row(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme"));
    assert_fails(&r, "AC1", "seven common control header rows");
}

#[test]
fn ac1_denial_rows_that_do_not_tile() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| 48 \| 4 \| u32 \| Request id: the request id of the admission",
            r"^\| 48",
            "| 49",
        )
    });
    assert_fails(&r, "AC1", "tile from byte 0");
}

#[test]
fn ac1_denial_a_maximum_size_that_differs_from_the_rows() {
    let r = wire(|t| {
        replace_in_line(
            t,
            "^The reply's maximum size is 54 bytes",
            "54 bytes",
            "56 bytes",
        )
    });
    assert_fails(&r, "AC1", "maximum size equal to the sum");
}

#[test]
fn ac1_denial_a_type_code_outside_the_block() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| 6 \| 1 \| u8 \| Message type: 33",
            r"33 only",
            "36 only",
        )
    });
    assert_fails(&r, "AC1", "inside the block the hello gives the probe pair");
}

#[test]
fn ac1_denial_the_block_not_assigned_to_this_ticket() {
    let r = wire(|t| {
        replace_in_line(
            t,
            r"^\| 32 to 35 \| path-MTU probe pair",
            "004-27",
            "004-99",
        )
    });
    assert_fails(&r, "AC1", "block assigned to 004-27");
}

#[test]
fn ac1_denial_no_padding_row_in_the_request() {
    let r = wire(|t| delete_row(t, r"^\| 54 \| 1 × max 65453 \| pad"));
    assert_fails(&r, "AC1", "zero-valued padding row");
}

#[test]
fn ac1_denial_no_echoed_size_in_the_reply() {
    let r = wire(|t| delete_row(t, r"^\| 52 \| 2 \| u16 \| Echoed intended packet size"));
    assert_fails(&r, "AC1", "echoes the intended packet size");
}

#[test]
fn ac1_denial_no_matching_subsection() {
    let r = wire(|t| delete_section(t, "^Matching a reply to its probe$"));
    assert_fails(
        &r,
        "AC1",
        "a matching subsection says how a reply identifies its probe",
    );
}

#[test]
fn ac1_denial_no_df_rule() {
    let r = wire(|t| replace_all(t, r"(?i)rule 11", "the rule"));
    assert_fails(&r, "AC1", "the DF requirement is stated");
}

#[test]
fn ac1_denial_no_owner_ruling_on_the_acknowledgement() {
    let r = wire(|t| replace_all(t, r"small acknowledgement", "short message"));
    assert_fails(&r, "AC1", "the owner's ruling");
}

#[test]
fn ac1_denial_a_padded_reply() {
    let r = wire(|t| {
        replace_in_line(
            t,
            "^The probe reply is the consumer side's answer",
            "$",
            " The reply is padded.",
        )
    });
    assert_fails(&r, "AC1", "does not say the reply is padded");
}

#[test]
fn ac1_denial_the_fallback_sequence_out_of_order() {
    let r = wire(|t| replace_all(t, "PathMtuTooSmall", "PathMtuFailed"));
    assert_fails(&r, "AC1", "the fallback sequence is stated in order");
}

#[test]
fn ac1_denial_the_fallback_arithmetic_broken() {
    let r = run(&base().mutate(CONSTANTS_MD, |t| {
        replace_in_line(t, r"^\| C-06 \|", "1 280", "1 300")
    }));
    assert_fails(&r, "AC1", "the fallback arithmetic holds");
}

#[test]
fn ac1_denial_the_admission_request_intended_size_changed() {
    let r = wire(|t| {
        within_section(t, "^Admission request$", |s| {
            replace_all(
                s,
                r"\| u16 \| Intended packet size",
                "| u16 | Declared packet size",
            )
        })
    });
    assert_fails(
        &r,
        "AC1",
        "the admission request still carries the intended packet size",
    );
}

#[test]
fn ac1_denial_the_toml_type_code_outside_the_block() {
    let r = toml(|t| replace_all(t, r#"valid = "33 only""#, r#"valid = "36 only""#));
    assert_fails(&r, "AC1", "carries a message type inside the probe block");
}

// ---------------------------------------------------------------- AC2 denials

#[test]
fn ac2_denial_the_reply_also_exempt() {
    let r = toml(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe reply\"\n",
            "name = \"path-MTU probe reply\"\nexempt_from_control_bound = \"C-116\"\n",
        )
    });
    assert_fails(
        &r,
        "AC2",
        "exactly one message of wire.toml carries the exemption",
    );
}

#[test]
fn ac2_denial_an_exemption_citing_another_constant() {
    let r = wire(|t| {
        replace_in_line(
            t,
            "^In wire.toml only the probe request carries",
            "$",
            " It is also exempt under C-18.",
        )
    });
    assert_fails(
        &r,
        "AC2",
        "every sentence of the section that says exempt cites C-116",
    );
}

// ---------------------------------------------------------------- AC3 denials

#[test]
fn ac3_denial_fewer_than_four_forms() {
    let r = probe(|t| delete_section(t, "^Validity rules$"));
    assert_fails(&r, "AC3", "at least 4 invalid forms");
}

#[test]
fn ac3_denial_an_outcome_naming_no_member() {
    let r = probe(|t| replace_in_line(t, r"^\| bad magic", "BadMagic", "WrongMagic"));
    assert_fails(&r, "AC3", "not members: [\"WrongMagic\"]");
}

#[test]
fn ac3_denial_non_zero_padding_without_reserved_non_zero() {
    let r = probe(|t| replace_in_line(t, r"^\| padding non-zero", "ReservedNonZero", "BadLength"));
    assert_fails(
        &r,
        "AC3",
        "non-zero padding is dropped and counted under ReservedNonZero",
    );
}

#[test]
fn ac3_denial_no_unmatched_reply_row() {
    let r = probe(|t| delete_row(t, r"^\| a reply that matches no outstanding probe"));
    assert_fails(
        &r,
        "AC3",
        "a reply that matches no outstanding probe has an outcome",
    );
}

#[test]
fn ac3_denial_no_probe_size_row() {
    let r = probe(|t| {
        delete_row(
            t,
            r"^\| probe size different from its declared intended size",
        )
    });
    assert_fails(
        &r,
        "AC3",
        "a probe size different from its declared intended size",
    );
}

#[test]
fn ac3_denial_the_enumeration_not_thirteen() {
    let r = wire(|t| delete_row(t, r"^\| \d+ \| BadMagic \|"));
    assert_fails(&r, "AC3", "has 13 members");
}

// ---------------------------------------------------------------- AC4 denials

#[test]
fn ac4_denial_the_reply_above_the_bound() {
    let r = toml(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe reply\"\nlayer = \"control\"\ndscp = 40\nmax_size = 54",
            "name = \"path-MTU probe reply\"\nlayer = \"control\"\ndscp = 40\nmax_size = 1300",
        )
    });
    assert_fails(&r, "AC4", "declares a max_size at most the");
}

#[test]
fn ac4_denial_the_wrong_dscp() {
    let r = toml(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe request\"\nlayer = \"control\"\ndscp = 40",
            "name = \"path-MTU probe request\"\nlayer = \"control\"\ndscp = 46",
        )
    });
    assert_fails(&r, "AC4", "is layer control at dscp");
}

#[test]
fn ac4_denial_a_refusal_missing_from_wire_toml() {
    let r = toml(|t| replace_all(t, r#"name = "PathMtuTooSmall""#, r#"name = "PathTooSmall""#));
    assert_fails(&r, "AC4", "the refusal enumeration of wire.toml equals");
}

#[test]
fn ac4_denial_wire_toml_does_not_parse() {
    let r = toml(|t| format!("{t}\n[[[broken"));
    assert_fails(&r, "AC4", "does not parse");
}

#[test]
fn ac4_denial_no_probe_reply_in_wire_toml() {
    let r = toml(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe reply\"",
            "name = \"path-MTU answer\"",
        )
    });
    assert_fails(&r, "AC4", "wire.toml has no probe reply message");
}

// ---------------------------------------------------------------- DoD denials

#[test]
fn dod_denial_an_untagged_paragraph() {
    let r = probe(|t| strip_tags(t, r"^Don't-fragment \(rule 11\)"));
    assert_fails(
        &r,
        "DoD",
        "every prose paragraph of 5 or more words in the probe section carries an evidence tag",
    );
}

#[test]
fn dod_denial_an_untagged_subsection() {
    let r = wire(|t| {
        let t = delete_unit(t, "^A probe is identified by its request id");
        delete_unit(&t, "^The responder needs no match")
    });
    assert_fails(
        &r,
        "DoD",
        "every subsection of the probe section carries an evidence tag",
    );
}

#[test]
fn dod_denial_an_open_item_without_an_owning_ticket() {
    let r = probe(|t| {
        replace_in_line(
            t,
            r"^\*\*Open item 7",
            "$",
            "\n\n**Open item 8 — x.** Owner: the design owner. Falsifier: a run `[decision]`.",
        )
    });
    assert_fails(
        &r,
        "DoD",
        "every open item of the probe section names an owner and a falsifier",
    );
}

#[test]
fn denial_wire_md_missing() {
    let r = run(&base().without(WIRE_MD));
    assert_fails(&r, "AC1", "does not exist");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (a2a5ae1, "failing checks
/// for the path-MTU probe pair") fails every criterion, as the script did.
/// Skipped when the commit is not in the clone.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "a2a5ae1b27714ac69389d9de762ff782167f8c01";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "DoD"],
        "{}",
        r.render()
    );
}
