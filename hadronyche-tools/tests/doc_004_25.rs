//! Ticket 004-25 — the regime stamp section types the constants table
//! names. Port of `checks/004-25.sh` (deleted with this port;
//! `doccheck-equiv 004-25` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_25`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script. `denials_agree_with_the_script` (ignored by
//! default: it needs Python and the script from git history) runs every
//! denial fixture through the harness sandbox against the old script and
//! requires the script to reject it too.
//!
//! Not here, because they run tools or inspect the repository (they are in
//! `scripts/repo-state-checks.sh`, under 004-25): `cargo run --bin
//! wire-check` reports "checks clean" (AC6); `cargo test -p hadronyche-tools
//! --test wire_tables` exits 0 with 0 failed and at least 12 passed (AC6);
//! the checker names no ref containing main and runs no other check (DoD:
//! the script grepped its own text; the relocated assertion greps this
//! port's sources for the ref pattern and for a spawned process).
//!
//! Header and code: the header's "Declared limits" say the capability
//! vocabulary of review C2 "is the list in review.py"; no review.py exists
//! in the repository and the code holds the list inline (`CAP_VOCAB`); the
//! port follows the code. The header lists the merged-text and review
//! assertions under DoD, as the code reports them.

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_25::{SPEC, WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::equiv::script::{Sandbox, script_source};
use hadronyche_tools::doccheck::mutate::{
    delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section,
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

fn wire(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(WIRE_MD, edit)
}

/// wire.md with `edit` applied to the section of `heading` only.
fn sub(heading: &'static str, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(|t| within_section(t, heading, edit))
}

/// wire.md with `edit` applied to each of the sections of `headings`.
fn subs(headings: &[&'static str], edit: fn(&str) -> String) -> Docs {
    wire(|t| {
        headings
            .iter()
            .fold(t.to_string(), |t, h| within_section(&t, h, edit))
    })
}

fn stamp(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Regime stamp$", edit)
}

fn toml(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(WIRE_TOML, edit)
}

/// wire.toml with `from` (exact text) replaced by `to`.
fn toml_swap(from: &str, to: &str) -> Docs {
    toml(|t| replace_all(t, &regex::escape(from), to))
}

fn consts(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(CONSTANTS_MD, edit)
}

/// The three tables of invalid forms the check counts (004-25's and the
/// two of 004-28).
const VALIDITY_SUBS: [&str; 3] = [
    "^Validity rules of the section types$",
    "^Observed DSCP of peers section$",
    "^Alignment group section$",
];

const CAL_HEAD: &str = "name = \"regime stamp calibration factor section\"\nlayer = \"control\"\ndscp = 40\nmax_size = 66\nsection_of = \"regime stamp\"\nrejection_reasons = [";
const MULTI_ENTRY: &str =
    "name = \"regime stamp multi-packet entry\"\nlayer = \"control\"\ndscp = 40\nmax_size = 8";
const LEGACY: &str = "name = \"regime stamp section\"\nlayer = \"control\"\ndscp = 40\nmax_size = 1252\nsection_of = \"regime stamp\"\nrejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]\n\n[[message.field]]\nname = \"magic\"\noffset = 0\nwidth = 16";
const REPORT_LINE: &str = r"^\*\*Reported: the 64-section bound is exceeded at the maxima";

/// Every match of `pattern` on the first line matching `line` replaced.
fn every_in_line(text: &str, line: &str, pattern: &str, to: &str) -> String {
    let re = regex::Regex::new(line).expect("line pattern");
    let at = text
        .split('\n')
        .position(|l| re.is_match(l))
        .expect("a line matches");
    text.split('\n')
        .enumerate()
        .map(|(i, l)| {
            if i == at {
                l.replace(pattern, to)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

/// One denial: its name, the exact set of criteria both the port and the
/// old script fail on it (empty: both pass, a fixture that tells the
/// scripts' Python semantics from a near miss), the criterion and message
/// the port must report, and the mutated documents. Plain data, so the list
/// can move into the shared denial-list form.
struct Denial {
    name: &'static str,
    fails: &'static [&'static str],
    criterion: &'static str,
    needle: &'static str,
    docs: fn() -> Docs,
}

/// Checks one denial on the port: the exact failed set, and the targeted
/// message.
fn deny(d: &Denial) {
    let r = run(&(d.docs)());
    let got: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(got, d.fails, "{}: failed criteria\n{}", d.name, r.render());
    if !d.fails.is_empty() {
        assert_fails(&r, d.criterion, d.needle);
    }
}

/// Each entry becomes a `Denial` in `DENIALS` and a `#[test]` of it.
macro_rules! denials {
    ($($name:ident: [$($f:literal),*], $crit:literal, $needle:literal, $docs:expr;)*) => {
        const DENIALS: &[Denial] = &[$(Denial {
            name: stringify!($name),
            fails: &[$($f),*],
            criterion: $crit,
            needle: $needle,
            docs: || $docs,
        },)*];
        $(
            #[test]
            fn $name() {
                deny(DENIALS.iter().find(|d| d.name == stringify!($name)).expect("listed"));
            }
        )*
    };
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Seven section types exist, one per item in Context, each with a
/// type code and a maximum size ≤ 1 252 bytes."
///
/// Script: seven types, one subsection each; each type's field table and
/// its entry tables (depth-first) tile, each entry table sums to its list
/// row's width, the section at its maxima is within 1 252 bytes and is
/// stated; ⌈M / F⌉ = N statements; a type code inside the block of 004-25,
/// all distinct; the Context values; every product and sum exact; the
/// 64-section bound fits or is reported with Open item 1; the merged
/// section count and code block.
#[test]
fn ac1_seven_section_types_with_codes_and_sizes() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The capability-state enumeration has exactly the four values of
/// C-129; the timestamp-source enumeration has exactly the four values of
/// C-131."
#[test]
fn ac2_the_two_enumerations_have_exactly_four_values() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The health section has the eleven C-159 counters plus the
/// observed DSCP." Script: each counter its own row (a maximum matching),
/// each with a wrap or saturation rule and a unit; the observed DSCP.
#[test]
fn ac3_the_health_section_has_the_eleven_counters_and_the_dscp() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "The capability section includes control isolation (C-161) and
/// the injection section includes the panic hook (C-145)."
#[test]
fn ac4_control_isolation_and_the_panic_hook() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "At least 5 invalid forms with outcomes." Script: at least five
/// rows in the tables of invalid forms other than the merged one, each
/// naming control reasons (or silent only for a repeat, an unknown code or
/// an older version), ReservedNonZero and BadLength where required, and the
/// four categories.
#[test]
fn ac5_at_least_five_invalid_forms_with_outcomes() {
    assert_criterion(report(), "AC5");
}

/// AC6 — "Passes the 004-02 checker." Script (document part): wire.toml
/// parses; the merged section message is unchanged; one regime stamp
/// message per type, control at DSCP 40 with the table's max_size, control
/// reasons only, fields agreeing row for row; one entry message per entry
/// table, bit for bit. The checker run is in scripts/repo-state-checks.sh.
#[test]
fn ac6_wire_toml_agrees_with_the_tables() {
    assert_criterion(report(), "AC6");
}

/// DoD — "Every new factual claim carries an evidence tag": each type
/// subsection and each of its prose paragraphs of 5+ words (policy
/// `PROSE_004_29`).
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(report(), "DoD", "evidence tag");
}

/// DoD — "No new open item without an owner and a falsifier" (policy
/// `MENTION_BLOCKS_004_24`).
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(report(), "DoD", "Open item paragraph");
}

/// DoD — "Regime stamp fields touched are complete and documented": the
/// merged header rows, the Rejection reasons (13 names and values), the
/// Reserved fields rows; the review findings C1 to C4.
#[test]
fn dod_merged_text_unchanged_and_review_findings() {
    assert_criterion(report(), "DoD");
}

// ---------------------------------------------------------------- denials

denials! {
    denial_wire_md_missing: ["AC1", "AC5", "DoD"], "AC1", "does not exist", base().without(WIRE_MD);
    denial_wire_toml_missing: ["AC6"], "AC6", "does not exist", base().without(WIRE_TOML);
    ac1_denial_the_control_bound_constant: ["AC1"], "AC1", "gives the control bound C-17 as 1 252",
        consts(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 300"));
    ac3_denial_c159_not_eleven: ["AC3"], "AC3", "eleven C-159 counters, read",
        consts(|t| replace_in_line(t, r"^\| C-159 \|", ", dead", ""));
    ac2_denial_c129_not_four: ["AC2"], "AC2", "four C-129 capability states, read",
        consts(|t| replace_in_line(t, r"^\| C-129 \|", ", not requested\\.", "."));
    ac2_denial_c131_not_four: ["AC2"], "AC2", "four C-131 receive-timestamp sources",
        consts(|t| replace_in_line(t, r"^\| C-131 \|", r"interrupt \(stamped", "interrupt: stamped"));
    ac1_denial_the_placeholder: ["AC1"], "AC1", "placeholder replaced",
        stamp(|t| replace_in_line(t, "^The regime stamp is a module's declaration", "^", "To be filled. "));
    dod_denial_the_merged_header_changed: ["DoD"], "DoD", "header rows (bytes 0 to 57) are unchanged",
        stamp(|t| replace_in_line(t, r"^\| 52 \| 1 \| u8 \| Section index", r"^\| 52 \| 1 \|", "| 52 | 2 |"));
    ac1_denial_the_section_count_not_64: ["AC1", "DoD"], "AC1", "allows 1 to 64 sections",
        stamp(|t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 63"));
    ac1_denial_no_code_block: ["AC1"], "AC1", "give 004-25 a block",
        stamp(|t| replace_in_line(t, r"^\| 1 to 15 \|", r"\| 004-25 \|", "| 004-99 |"));
    dod_denial_a_rejection_value_changed: ["DoD"], "DoD", "Rejection reasons enumeration is unchanged",
        wire(|t| replace_in_line(t, r"^\| 9 \| BadPacketIndex \|", r"^\| 9", "| 13"));
    dod_denial_the_reserved_codes_row_removed: ["DoD"], "DoD", "still records the section type codes row",
        wire(|t| delete_row(t, r"^\| Section type codes not assigned in v1"));
    ac1_denial_two_subsections_name_one_type: ["AC1", "DoD"], "AC1", "names the calibration section type (found 2)",
        wire(|t| rename_heading(t, "^Injections section$", "Calibration and injections section"));
    ac1_denial_a_type_missing: ["AC1", "DoD"], "AC1", "seven section types are laid out",
        wire(|t| rename_heading(t, "^Commissioning window section$", "Window section"));
    ac1_denial_a_malformed_type_row: ["AC1", "DoD"], "AC1", "Calibration factor section: the Width cell",
        sub("^Calibration factor section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Calibration factor", r"^\| 0 \| 8 \|", "| 0 | eight |"));
    ac1_denial_a_type_without_field_table: ["AC1", "DoD"], "AC1", "commissioning section type has no field table",
        sub("^Commissioning window section$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"));
    ac1_denial_an_unused_field_table: ["AC1"], "AC1", "field table(s) that no list row of it names",
        sub("^Calibration factor section$", |t| replace_in_line(t, "^The maximum section size, which is also its fixed size, is 58 \\+ 8", "$", "\n\n| Offset | Width | Type | Meaning | Valid range |\n|---|---|---|---|---|\n| 0 | 1 | u8 | Spare | 0 only |"));
    ac1_denial_an_offset_neither_0_nor_58: ["AC1", "AC6"], "AC1", "starts at offset 4",
        sub("^Calibration factor section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Calibration factor", r"^\| 0 ", "| 4 "));
    ac1_denial_rows_that_do_not_tile: ["AC1", "AC6"], "AC1", "table does not tile",
        sub("^Commissioning window section$", |t| replace_in_line(t, r"^\| 8 \| 8 \| u64 \| Expiry time", r"^\| 8 ", "| 9 "));
    ac1_denial_an_entry_table_off_its_width: ["AC1", "AC6"], "AC1", "entry table sums to",
        sub("^Multi-packet regime section$", |t| replace_in_line(t, r"^\| 1 \| 8 × max 149", "8 × max", "9 × max"));
    ac1_denial_a_type_over_the_bound: ["AC1", "AC6", "DoD"], "AC1", "no more than the control bound",
        sub("^Multi-packet regime section$", |t| replace_in_line(t, r"^\| 1 \| 8 × max 149", "max 149", "max 160"));
    ac1_denial_no_section_count_statement: ["AC1", "DoD"], "AC1", "has no statement of the form",
        sub("^Multi-packet regime section$", |t| replace_all(t, "⌈256 / 149⌉", "256 over 149"));
    ac1_denial_a_wrong_section_count_statement: ["AC1"], "AC1", "the multi section count is wrong",
        sub("^Multi-packet regime section$", |t| replace_all(t, "⌈256 / 149⌉ = 2", "⌈256 / 149⌉ = 3"));
    ac1_denial_the_size_not_stated: ["AC1"], "AC1", "states its maximum section size as 74 bytes",
        sub("^Commissioning window section$", |t| replace_all(t, "58 \\+ 16 = 74 bytes", "58 + 16 = 74"));
    ac1_denial_a_type_code_outside_the_block: ["AC1"], "AC1", "calibration section type has a type code inside the block",
        sub("^Calibration factor section$", |t| replace_all(t, "Section type code 3 ", "Section type code 30 "));
    ac1_denial_two_types_share_a_code: ["AC1"], "AC1", "codes are distinct",
        sub("^Calibration factor section$", |t| replace_all(t, "Section type code 3 ", "Section type code 2 "));
    ac1_denial_an_inexact_product: ["AC1"], "AC1", "every product and sum stated in the regime stamp section is exact",
        sub("^Multi-packet regime section$", |t| replace_all(t, "149 × 8 = 1 192", "149 × 8 = 1 193"));
    ac1_denial_an_inexact_sum: ["AC1"], "AC1", "every product and sum stated in the regime stamp section is exact",
        sub("^Calibration factor section$", |t| replace_all(t, "58 \\+ 8 = 66", "58 + 8 = 67"));
    ac1_denial_the_excess_not_reported: ["AC1", "DoD"], "AC1", "or the text reports the excess and names Open item 1",
        stamp(|t| replace_all(t, "Open item 1", "Open item one"));
    ac1_denial_no_stream_in_the_multi_packet_section: ["AC1"], "AC1", "the multi-packet section carries the stream number",
        sub("^Multi-packet regime section$", |t| replace_all(t, "(?i)stream", "flow"));
    ac1_denial_no_expiry_time: ["AC1"], "AC1", "carries the expiry time",
        sub("^Commissioning window section$", |t| replace_all(t, "(?i)expir", "end"));
    ac1_denial_no_time_per_operation: ["AC1"], "AC1", "carries the calibration factor, time per counted operation",
        sub("^Calibration factor section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Calibration factor", "abstract operation on the sender's target, in femtoseconds per operation", "abstract op on the sender's target, in femtoseconds per op"));
    ac1_denial_no_epoch_in_the_injections_section: ["AC1"], "AC1", "a stream forced onto a timeline epoch",
        sub("^Injections section$", |t| replace_all(t, "(?i)epoch", "era"));
    ac4_denial_no_panic_hook: ["AC4"], "AC4", "the panic hook",
        sub("^Injections section$", |t| replace_all(t, "(?i)panic", "abort"));
    ac4_denial_no_control_isolation: ["AC4"], "AC4", "control-isolation capability",
        sub("^Capability states section$", |t| replace_all(t, "(?i)isolat", "separat"));
    ac1_denial_the_timestamp_source_without_a_state: ["AC1"], "AC1", "reports its source with a C-129 capability state",
        sub("^Receive-timestamp source section$", |t| replace_all(t, r"(?i)capabilit\w*\s+state|C-129", "x"));
    ac2_denial_a_fifth_capability_state: ["AC2"], "AC2", "exactly the four C-129 states",
        sub("^Capability states section$", |t| replace_in_line(t, r"^\| 4 \| not requested \|", "$", "\n| 5 | revoked | withdrawn |"));
    ac2_denial_a_timestamp_source_renamed: ["AC2"], "AC2", "exactly the four C-131 sources",
        sub("^Receive-timestamp source section$", |t| replace_in_line(t, r"^\| 4 \| interrupt \|", "interrupt", "irq"));
    ac3_denial_a_counter_without_its_row: ["AC3"], "AC3", "has its own row of the health section (matched 10)",
        sub("^Per-connection health section$", |t| replace_in_line(t, r"^\| 39 \| 4 \| u32 \| Too-early", "Too-early: packets that arrived too early", "Premature: packets that arrived prematurely"));
    ac3_denial_a_counter_without_its_wrap_rule: ["AC3"], "AC3", "the counter dead states its wrap",
        sub("^Per-connection health section$", |t| replace_in_line(t, r"^\| 59 \| 4 \| u32 \| Dead", r"cumulative, wraps modulo 2\^32 \| any 32-bit value \(wrap rule below\)", "cumulative | any 32-bit value"));
    ac3_denial_a_counter_without_its_unit: ["AC3"], "AC3", "the counter received states its unit",
        sub("^Per-connection health section$", |t| replace_in_line(t, r"^\| 19 \| 4 \| u32 \| Received", "slices accepted", "items accepted"));
    ac3_denial_no_observed_dscp: ["AC3"], "AC3", "carries the observed DSCP",
        sub("^Per-connection health section$", |t| replace_all(t, "(?i)observed DSCP", "seen DSCP"));
    ac5_denial_fewer_than_five_forms: ["AC5"], "AC5", "at least 5 invalid forms",
        subs(&VALIDITY_SUBS, |t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Form | Outcome |"));
    ac5_denial_a_reason_outside_the_control_layer: ["AC5"], "AC5", "names a reason outside the control layer",
        sub("^Validity rules of the section types$", |t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadChecksum"));
    ac5_denial_a_silent_form: ["AC5"], "AC5", "names no Reject reason",
        sub("^Validity rules of the section types$", |t| replace_in_line(t, r"^\| a capability states section that lacks", r"\| drop the section.*$", "| dropped silently |"));
    ac5_denial_reserved_bits_without_reserved_non_zero: ["AC5"], "AC5", "must name ReservedNonZero",
        sub("^Validity rules of the section types$", |t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadLength"));
    ac5_denial_a_length_form_without_bad_length: ["AC5"], "AC5", "a length form must name BadLength",
        sub("^Validity rules of the section types$", |t| replace_in_line(t, r"^\| section length inconsistent", r"\| drop the section.*$", "| drop the section and count under TooShort |"));
    ac5_denial_a_count_form_without_bad_length: ["AC5"], "AC5", "a count-above-maximum form must name BadLength",
        sub("^Validity rules of the section types$", |t| replace_in_line(t, r"^\| entry count above its maximum", r"\| drop the section.*$", "| drop the section and count under TooShort |"));
    ac5_denial_no_length_form: ["AC5"], "AC5", "an invalid form covers: length inconsistent",
        subs(&VALIDITY_SUBS, |t| replace_all(t, "section length inconsistent", "section size inconsistent"));
    dod_denial_reserved_fields_not_recorded: ["DoD"], "DoD", "is recorded in the Reserved fields table",
        sub("^Reserved fields$", |t| replace_all(t, "(?i)regime stamp", "stamp"));
    dod_denial_an_untagged_subsection: ["DoD"], "DoD", "the commissioning subsection carries an evidence tag",
        sub("^Commissioning window section$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""));
    dod_denial_an_untagged_paragraph: ["DoD"], "DoD", "every prose paragraph of 5 or more words in the calibration subsection",
        sub("^Calibration factor section$", |t| strip_tags(t, "^The unit agrees with the manifest's"));
    dod_denial_an_open_item_without_an_owner: ["DoD"], "DoD", "every Open item paragraph",
        stamp(|t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "Open item 99: the spare bits are unassigned `[decision]`.\n\n"));
    ac6_denial_wire_toml_does_not_parse: ["AC6"], "AC6", "does not parse",
        toml(|t| format!("{t}\n[[[broken"));
    ac6_denial_the_merged_message_changed: ["AC6"], "AC6", "the merged regime stamp section message of wire.toml is unchanged",
        toml_swap(LEGACY, &LEGACY.replace("width = 16", "width = 8"));
    ac6_denial_no_message_for_a_type: ["AC6"], "AC6", "exactly one regime stamp message for the calibration section type (found 0)",
        toml_swap("name = \"regime stamp calibration factor section\"", "name = \"regime stamp factor section\"");
    ac6_denial_a_max_size_off_the_table: ["AC6"], "AC6", "the calibration message of wire.toml is control",
        toml_swap(CAL_HEAD, &CAL_HEAD.replace("max_size = 66", "max_size = 67"));
    ac6_denial_a_reason_outside_the_control_layer: ["AC6"], "AC6", "the calibration message names only control reasons",
        toml_swap(CAL_HEAD, &format!("{CAL_HEAD}\"BadChecksum\", "));
    ac6_denial_fields_that_disagree: ["AC6"], "AC6", "the calibration fields of wire.toml agree",
        toml_swap("name = \"calibration_factor\"\noffset = 464\nwidth = 64", "name = \"calibration_factor\"\noffset = 464\nwidth = 32");
    ac6_denial_an_entry_message_missing: ["AC6"], "AC6", "one entry message per entry table of the multi section",
        toml_swap("name = \"regime stamp multi-packet entry\"", "name = \"regime stamp multi-packet item\"");
    ac6_denial_an_entry_message_that_disagrees: ["AC6"], "AC6", "the entry table of regime stamp multi-packet entry agrees",
        toml_swap(MULTI_ENTRY, &MULTI_ENTRY.replace("max_size = 8", "max_size = 9"));
    dod_denial_sender_rule_7_not_quoted: ["DoD"], "DoD", "the merged sender rule 7 is quoted",
        stamp(|t| replace_all(t, "a stamp that needs more cannot be sent", "a stamp that needs more is not sent"));
    dod_denial_the_health_maximum_off_the_constants: ["DoD"], "DoD", "the health maximum equals the constants' product",
        consts(|t| replace_in_line(t, r"^\| C-217 \|", "63 peers", "62 peers"));
    dod_denial_the_product_without_its_constants: ["DoD"], "DoD", "states the product with its constants C-09 and C-217",
        sub("^Per-connection health section$", |t| replace_all(t, "C-217", "the peer map"));
    dod_denial_the_report_without_the_total: ["DoD"], "DoD", "states the total 1799 sections",
        stamp(|t| replace_in_line(t, REPORT_LINE, "1 799 against 64, an excess of 1 799", "many against 64, an excess"));
    dod_denial_the_report_without_the_design_owner: ["DoD"], "DoD", "goes to the design owner",
        stamp(|t| replace_all(t, "design owner", "lead"));
    dod_denial_the_report_without_rule_7: ["DoD"], "DoD", "cites merged sender rule 7",
        stamp(|t| replace_in_line(t, REPORT_LINE, "a stamp that needs more cannot be sent", "a stamp that needs more is not sent"));
    dod_denial_the_report_without_no_stamp: ["DoD"], "DoD", "sends no stamp at all",
        stamp(|t| replace_in_line(t, REPORT_LINE, "sends no stamp at all", "sends nothing at all"));
    dod_denial_the_report_without_the_threshold: ["DoD"], "DoD", "gives the threshold in entries",
        stamp(|t| every_in_line(t, REPORT_LINE, "1 026", "many"));
    dod_denial_the_report_untagged: ["DoD"], "DoD", "the report paragraph carries an evidence tag",
        stamp(|t| strip_tags(t, REPORT_LINE));
    dod_denial_a_capability_named_without_its_row: ["DoD"], "DoD", "the capability \"idle states\" named with a ticket id",
        sub("^Capability states section$", |t| replace_in_line(&delete_row(t, r"^\| 3 \| CPU idle states"), "^Ids 1 to 4 are the capabilities", "^", "Idle states matter to 013-06 `[decision]`. "));
    dod_denial_the_stated_capability_count_wrong: ["DoD"], "DoD", "the stated count of 10 capabilities",
        stamp(|t| replace_all(t, "stamps nine capabilities", "stamps ten capabilities"));
    dod_denial_a_manifest_item_handed_back: ["DoD"], "DoD", "review C3",
        stamp(|t| replace_in_line(t, "^Ids 1 to 4 are the capabilities", "^", "Manifest, Open item 9 is left to its owner `[decision]`. "));
    dod_denial_no_admission_marker: ["DoD"], "DoD", "review C4",
        sub("^Per-connection health section$", |t| replace_all(&replace_all(t, "(?i)admission|request id|generation", "handshake"), "(?i)reset", "restart"));
    ac6_denial_a_multi_line_string_the_reader_rejects: ["AC6"], "AC6", "does not parse",
        toml(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n"));
    ac1_denial_a_heading_after_a_python_line_break: ["AC1", "AC5"], "AC1", "seven section types are laid out",
        stamp(|t| replace_in_line(t, "^This part lays out the seven section types", "^", "\u{2028}## Interlude\u{2028}"));
    dod_denial_a_word_character_only_python_knows: ["DoD"], "DoD", "the stated count of -1 capabilities",
        stamp(|t| replace_all(t, "stamps nine capabilities", "stamps nine² capabilities"));
    ac2_denial_a_state_table_without_separator: ["AC2"], "AC2", "exactly the four C-129 states",
        sub("^Capability states section$", |t| replace_all(t, r"\| Value \| State \| Meaning \|\n\|---\|---\|---\|\n", "| Value | State | Meaning |\n"));
    accepted_a_separator_row_only_the_scripts_accept: [], "", "",
        sub("^Capability states section$", |t| replace_all(t, r"\| Value \| State \| Meaning \|\n\|---\|---\|---\|\n", "| Value | State | Meaning |\n| - | :-: | - - |\n"));
    ac1_denial_no_control_bound_where_the_script_stops: ["AC1"], "AC1", "the script stops here with a Python exception",
        consts(|t| delete_row(t, r"^\| C-17 \|"));
}

/// Every denial fixture, run against the old script in the harness sandbox
/// (the documents exported from the same in-memory fixture): the script's
/// failed criteria must be exactly the denial's, as the port's are.
/// Ignored by default: it needs Python and the script (from the working
/// tree, or from git history once deleted).
#[test]
#[ignore = "runs the old script; needs python3 and git history"]
fn denials_agree_with_the_script() {
    let (text, from) =
        script_source(&repo_root(), SPEC.id).expect("the script, in the tree or its history");
    let control = Sandbox::new(base(), SPEC.id, &text, SPEC.script_env)
        .expect("sandbox")
        .run();
    assert!(
        control.passed,
        "the script must pass on the unmutated documents: {}",
        control.tail
    );
    let mut disagree = Vec::new();
    for d in DENIALS {
        let v = Sandbox::new(&(d.docs)(), SPEC.id, &text, SPEC.script_env)
            .expect("sandbox")
            .run();
        let got: Vec<&str> = v.criteria.iter().map(String::as_str).collect();
        let port: Vec<String> = run(&(d.docs)()).failed_criteria().into_iter().collect();
        println!(
            "DENIAL {} | port {:?} | script {:?} passed={}",
            d.name, port, got, v.passed
        );
        if got != d.fails || v.passed != d.fails.is_empty() {
            disagree.push(format!(
                "{}: the script fails {got:?} (passed {}), the denial says {:?} ({})",
                d.name, v.passed, d.fails, v.tail
            ));
        }
    }
    assert!(
        disagree.is_empty(),
        "script {from}:\n{}",
        disagree.join("\n")
    );
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (e4670d4, "failing checks
/// for the regime stamp section types") fails AC1 and AC5, as the script
/// did there (the section types were not laid out; the other criteria's
/// assertions are guarded by the types being found). Skipped when the
/// commit is not in the clone.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "e4670d418ae6eb4d0995f95690f412be8b0951c2";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(failed, ["AC1", "AC5"], "{}", r.render());
}
