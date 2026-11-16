//! Ticket 006-01 — timeline spec skeleton and constants table. Port of
//! `checks/006-01.sh` (deleted with this port; `doccheck-equiv 006-01` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c006_01`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion) and on denial fixtures: the real documents with one mutation
//! applied in memory, each also run against the old script by the ignored
//! test `old_script_agrees_on_the_denials` (see `wp9_support`).
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

#[macro_use]
mod wp9_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c006_01::{SPEC_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{replace_all, replace_in_line};
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion};

const CONSTANTS: &str = "doc/design/02-constants.md";

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

fn spec(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(SPEC_MD, edit)
}

fn consts(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(CONSTANTS, edit)
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// AC1 — "`doc/design/spec/timeline.md` exists and its first section after
/// the introduction is the constants table with exactly the 47 rows above,
/// whose values match `doc/design/02-constants.md`."
#[test]
fn ac1_the_constants_table_comes_first_and_matches() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "Every section listed in Work step 1 is present as a heading."
/// (and each placeholder names the ticket that fills it, the DoD item).
#[test]
fn ac2_every_section_is_present_and_names_its_ticket() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The retired POC behaviour note carries the `[code]` tag and cites
/// `module.rs:1630-1634`."
#[test]
fn ac3_the_retired_poc_note() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "The file renders as Markdown without errors."
#[test]
fn ac4_the_markdown_parses() {
    assert_criterion(report(), "AC4");
}

/// The micro sign and the Greek mu are the same under NFKC: a value written
/// with the other one is still equal, as the script reads it.
#[test]
fn nfkc_equal_values_are_accepted() {
    let d = spec(base(), |t| replace_all(t, "\u{b5}", "\u{3bc}"));
    assert_criterion(&run(&d), "AC1");
}

denials! {
    denial_timeline_missing => ("AC1", "does not exist", |b: &Docs| b.without(SPEC_MD));
    denial_constants_missing => ("AC1", "does not exist; cannot verify values", |b: &Docs| b.without(CONSTANTS));
    ac1_denial_the_source_lacks_an_id => ("AC1", "02-constants.md lacks ids required by the ticket", |b| consts(b, |t| replace_in_line(t, r"^\| C-212 \|", "C-212", "C-2l2")));
    ac1_denial_no_constants_table => ("AC1", "no constants table found", |b| spec(b, |t| replace_in_line(t, r"^\| ID \| Constant", "ID", "Key")));
    ac1_denial_no_status_column => ("AC1", "constants table has no status column", |b| spec(b, |t| replace_in_line(t, r"^\| ID \| Constant", "Status", "Mood")));
    ac1_denial_a_row_with_an_unexpected_id => ("AC1", "unexpected row id", |b| spec(b, |t| replace_in_line(t, r"^\| C-212 \|", "$", "\n| X-1 | a | b | decided |")));
    ac1_denial_a_duplicate_row => ("AC1", "duplicate row C-01", |b| spec(b, |t| replace_in_line(t, r"^\| C-212 \|", "$", "\n| C-01 | a | b | decided |")));
    ac1_denial_a_missing_id => ("AC1", "constants table is missing ids: C-212", |b| spec(b, |t| replace_in_line(t, r"^\| C-212 \|", "C-212", "C-2l2")));
    ac1_denial_an_extra_id => ("AC1", "ids not in the ticket's 47 rows: C-03", |b| spec(b, |t| replace_in_line(t, r"^\| C-212 \|", "$", "\n| C-03 | a | b | decided |")));
    ac1_denial_a_value_that_differs => ("AC1", "value mismatch against 02-constants.md", |b| spec(b, |t| replace_in_line(t, r"^\| C-01 \|", "96 000 Hz", "96 001 Hz")));
    ac1_denial_a_status_that_differs => ("AC1", "status mismatch against 02-constants.md", |b| spec(b, |t| replace_in_line(t, r"^\| C-209 \|", r"\| decided \|$", "| inherited |")));
    ac1_denial_c61_not_pending_simulation => ("AC1", "C-61 must be marked pending simulation", |b| spec(&consts(b, |t| replace_in_line(t, r"^\| C-61 \|", "pending-simulation", "decided")), |t| replace_in_line(t, r"^\| C-61 \|", "pending-simulation", "decided")));
    ac1_denial_the_table_is_not_an_introduced_section => ("AC1", "not an introduced section", |b| spec(b, |t| replace_in_line(t, "^## Constants used by this document", "^## ", "# ")));
    ac1_denial_another_section_comes_first => ("AC1", "not the first section after the introduction", |b| spec(b, |t| replace_in_line(t, "^## Constants used by this document", "^", "## Extras\n\nText.\n\n")));
    ac1_denial_the_table_is_under_a_level_3_heading => ("AC1", "not under a level-2 heading", |b| spec(b, |t| replace_in_line(t, "^## Constants used by this document", "^## ", "### ")));
    ac1_denial_no_sentence_says_the_constants_table_wins => ("AC1", "no sentence states that the constants table wins", |b| spec(b, |t| replace_in_line(t, "^Every later section refers", "the constants table wins", "the constants table decides")));
    ac2_denial_a_section_missing => ("AC2", "missing section heading: Epoch width and wrap", |b| spec(b, |t| replace_in_line(t, "^## Epoch width and wrap", "Epoch width and wrap", "Width")));
    ac2_denial_a_placeholder_without_its_ticket => ("AC2", "does not name the ticket id that fills it", |b| spec(b, |t| replace_in_line(t, "^To be written in 006-03", "006-03", "a later ticket")));
    ac2_denial_a_range_that_stops_short => ("AC2", "(missing 006-14", |b| spec(b, |t| replace_in_line(t, r"^To be written in 006-13\.\.15", r"006-13\.\.15", "006-13")));
    ac2_denial_a_stemmed_heading_still_matches_but_a_synonym_does_not => ("AC2", "missing section heading: Discontinuity event", |b| spec(b, |t| replace_in_line(t, "^## Discontinuity event", "event", "occurrence")));
    ac3_denial_no_retired_note => ("AC3", "no 'Retired POC behaviour' note", |b| spec(b, |t| replace_all(t, "(?i)retired POC behaviour", "Old clock")));
    ac3_denial_no_code_tag => ("AC3", "does not carry [code]", |b| spec(b, |t| replace_in_line(t, r"^`\[code\]` In the POC", r"`\[code\]` ", "")));
    ac3_denial_no_citation => ("AC3", "does not cite module.rs:1630-1634", |b| spec(b, |t| replace_in_line(t, r"^`\[code\]` In the POC", "1630-1634", "1630-1635")));
    ac3_denial_a_note_that_is_not_a_heading_still_needs_the_tag => ("AC3", "does not carry [code]", |b| spec(b, |t| replace_in_line(&replace_in_line(t, "^## Retired POC behaviour", "^## Retired POC behaviour", "**Retired POC behaviour**"), r"^`\[code\]` In the POC", r"`\[code\]` ", "")));
    ac4_denial_an_unclosed_fence => ("AC4", "unclosed code fence", |b| spec(b, |t| append(t, "```\ncode")));
    fence_closer_with_an_info_string => ("AC4", "unclosed code fence", |b| spec(b, |t| append(t, "```\ncode\n```rust")));
    fence_closed_by_the_other_character => ("AC4", "unclosed code fence", |b| spec(b, |t| append(t, "```\ncode\n~~~")));
    ac4_denial_an_indented_heading => ("AC4", "heading is indented", |b| spec(b, |t| append(t, "  ## Indented")));
    ac4_denial_a_row_wider_than_its_header => ("AC4", "table row has 5 columns; its table header has 4", |b| spec(b, |t| replace_in_line(t, r"^\| C-209 \|", r"\| decided \|$", "| decided | extra |")));
    ac4_denial_a_separator_narrower_than_the_header => ("AC4", "table row has 3 columns; its table header has 4", |b| spec(b, |t| replace_in_line(t, r"^\|---\|---\|---\|---\|$", r"\|---\|---\|---\|---\|", "|---|---|---|")));
}

/// Every denial is rejected by the old script, except the ones listed here.
const SCRIPT_ACCEPTS: &[&str] = &[];

/// `cargo test -p hadronyche-tools --test doc_006_01 -- --ignored`.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees_on_the_denials() {
    let all = all_denials();
    assert_eq!(wp9_support::script_accepts("006-01", &all), SCRIPT_ACCEPTS);
}

/// The tree at the commit that added the script fails the criteria the
/// script failed (skipped when the commit is not in the clone).
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    let commit = "532d9ca1bcafb128119a2d0ffed3f1af6752761d";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    assert!(!r.passed(), "{}", r.render());
}
