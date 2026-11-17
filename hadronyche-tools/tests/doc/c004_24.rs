//! Ticket 004-24 — regime stamp message: its sections, its version and its
//! fetch request. Port of `checks/004-24.sh` (deleted with this port;
//! `doccheck-equiv 004-24` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_24`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script. `denials_agree_with_the_script` (ignored by
//! default: it needs Python and the script from git history) runs every
//! denial fixture through the harness sandbox against the old script and
//! requires the script to reject it too.
//!
//! Not here, because they run tools or inspect the repository (they are in
//! `scripts/repo-state-checks.sh`, under 004-24): `cargo run --bin
//! wire-check` reports "checks clean" (AC5); `cargo test -p hadronyche-tools
//! --test wire_tables` exits 0 with 0 failed and at least 12 passed (AC5,
//! the count that stood before this ticket); and the checker reads the
//! working tree only and runs no other check (DoD: the script grepped its
//! own text for a ref naming the main branch and for `checks/[0-9]`; the
//! relocated assertion greps this port's sources for the same ref pattern
//! and for a spawned process).
//!
//! Header and code: the header's AC4 says a row whose outcome is "not a
//! rejection", "ignored" or "valid" and names no Reject reason is not
//! counted; the code's pattern is `not a rejection|never a rejection|
//! \bignored\b|\bvalid\b` (it also takes "never a rejection"), and the port
//! follows the code. The header lists the toml rejection reasons under AC5;
//! the code reports them under AC4, as the port does. The header's AC1 says
//! "every section type"; the code checks the stamp section table and the
//! request (the section types are 004-25's and 004-28's), as the port does.

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_24::{SPEC, WIRE_MD, run};
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

/// wire.md with `edit` applied to the Regime stamp section only.
fn stamp(edit: impl FnOnce(&str) -> String) -> Docs {
    wire(|t| within_section(t, "^Regime stamp$", edit))
}

/// wire.md with `edit` applied to the `###` section `heading` only.
fn sub(heading: &'static str, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(|t| within_section(t, heading, edit))
}

fn toml(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(WIRE_TOML, edit)
}

fn consts(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(CONSTANTS_MD, edit)
}

/// The head of the section message of wire.toml.
const SEC_HEAD: &str = "name = \"regime stamp section\"\nlayer = \"control\"\ndscp = 40\nmax_size = 1252\nsection_of = \"regime stamp\"\n";
/// The head of the request message of wire.toml.
const REQ_HEAD: &str =
    "name = \"regime stamp request\"\nlayer = \"control\"\ndscp = 40\nmax_size = 40\n";

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

/// AC1 — "Every section type and the request have a maximum size ≤ 1 252
/// bytes (C-17, epic AC-3)."
///
/// Script: the section and the request tile their bytes, each at most the
/// control bound (C-17, C-18); the common control header is the landed hello
/// header; the stamp version is 32 bits and compared as the hello's version
/// is; the section carries index, count, type, length and body; both message
/// types lie in the block the hello gives 004-24; the section placeholder is
/// gone; the constants and the hello's widths read.
#[test]
fn ac1_the_section_and_the_request_are_within_the_control_bound() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The maximum stamp size and per-peer storage are stated as
/// numbers."
///
/// Script: the maximum section count, the maximum stamp size and the
/// per-peer storage are stated as numbers: the count matches its valid
/// range, the stamp size is the count times the section maximum (at most the
/// count times the control bound), the storage figures are multiples of the
/// stamp size, every product is exact, and totals over the peers are the
/// per-peer figure times C-217 or C-34 with the reason stated.
#[test]
fn ac2_the_maximum_stamp_size_and_storage_are_stated_as_numbers() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The multicast-on-change, 10 s refresh and unicast fetch of C-18
/// appear as sender and receiver rules."
///
/// Script: multicast on any regime change, the refresh period of C-18, the
/// version bump, the unicast fetch of a request, skipping unknown section
/// types by their length (004-04), the complete-stamp test, discarding an
/// incomplete stamp when a newer version arrives; a sender part and a
/// receiver part; the request carries the module id; the return path is
/// tagged [decision]; the review findings (the first stamp, a new instance
/// nonce, fetch only when stale, the fetch-load open item naming 011-23 and
/// 005-07).
#[test]
fn ac3_the_c18_rules_appear_as_sender_and_receiver_rules() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "At least 7 invalid forms with outcomes."
///
/// Script: at least seven invalid forms with outcomes (a row whose outcome
/// is not a rejection and names no Reject reason is not counted); the seven
/// categories; each outcome names only members whose Layers include
/// control; OversizeControl and BadLength where required; disagreeing
/// sections name exactly one reason; one stated outcome for an identical
/// repeat; the enumeration has 13 members; the toml reasons are members and
/// cover the table's.
#[test]
fn ac4_at_least_seven_invalid_forms_with_outcomes() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "Passes the 004-02 checker."
///
/// Script (document part): wire.toml parses; a message with section_of and
/// one without carry the Markdown tables row for row; each is layer control
/// at dscp 40 (C-13), tiles to a max_size at most the bound, carries no
/// exemption, and agrees in size. The checker run is in
/// scripts/repo-state-checks.sh.
#[test]
fn ac5_wire_toml_carries_the_stamp_messages() {
    assert_criterion(report(), "AC5");
}

/// DoD — "Every new factual claim carries an evidence tag": every prose line
/// of 5+ words (policy `LINES_004_24`) and every `###` subsection of the
/// section carries a tag.
#[test]
fn dod_every_new_claim_carries_an_evidence_tag() {
    assert_none(report(), "DoD", "evidence tag");
}

/// DoD — "No new open item without an owner and a falsifier": every block
/// that mentions an open item (policy `MENTION_BLOCKS_004_24`) names an
/// owner and a falsifier.
#[test]
fn dod_no_open_item_without_an_owner_and_a_falsifier() {
    assert_none(report(), "DoD", "Open item");
}

/// DoD — "The machine-readable tables parse and round-trip": the document
/// side is AC5's parse and agreement; the tool side (wire-check,
/// wire_tables) is in repo-state-checks.sh.
#[test]
fn dod_the_tables_parse() {
    assert_none(report(), "AC5", "does not parse");
}

// ---------------------------------------------------------------- denials

denials! {
    denial_wire_md_missing: ["AC1", "AC2", "AC3", "AC4", "AC5"], "AC1", "does not exist", base().without(WIRE_MD);
    denial_wire_toml_missing: ["AC4", "AC5"], "AC5", "does not exist", base().without(WIRE_TOML);
    ac1_denial_constants_unread: ["AC1", "AC3"], "AC1", "the constants read",
        consts(|t| replace_in_line(t, r"^\| C-18 \|", "every 10 s", "every ten s"));
    ac1_denial_a_malformed_hello_header_row: ["AC1"], "AC1", "is not a byte offset",
        sub("^Common control header$", |t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"^\| 7", "| seven"));
    ac1_denial_a_hello_header_row_missing: ["AC1"], "AC1", "seven rows tiling 32 bytes",
        sub("^Common control header$", |t| delete_row(t, r"^\| 8 \| 8 \| u64 \| Message counter"));
    ac1_denial_the_hello_width_unread: ["AC1"], "AC1", "the hello's layout gives the widths",
        wire(|t| replace_in_line(t, r"^\| 69 \| 4 \| u32 \| Regime stamp version number", r"Regime stamp version number \(C-18\).*\(Regime stamp version number\)", "Stamp generation (C-18)"));
    ac1_denial_the_block_not_assigned: ["AC1"], "AC1", "a block assigned to 004-24",
        wire(|t| replace_in_line(t, r"^\| 36 to 47 \|", "004-24", "004-99"));
    ac1_denial_no_stamp_section: ["AC1", "AC2", "AC3", "AC4", "AC5"], "AC1", "has a '## Regime stamp' section",
        wire(|t| rename_heading(t, "^Regime stamp$", "Stamp"));
    ac1_denial_the_placeholder: ["AC1"], "AC1", "not the placeholder",
        stamp(|t| replace_in_line(t, "^The regime stamp is a module's declaration", "^", "To be filled by 004-24. "));
    ac1_denial_a_malformed_stamp_row: ["AC1", "AC2", "AC5"], "AC1", "a regime stamp field table: the Width cell",
        stamp(|t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", r"^\| 53 \| 1 \|", "| 53 | one |"));
    ac1_denial_no_section_table: ["AC1", "AC2", "AC3", "AC4", "AC5"], "AC1", "no section table",
        stamp(|t| replace_all(t, r"(?i)section index", "section place"));
    ac3_denial_no_request_table: ["AC1", "AC3", "AC5"], "AC3", "no request table",
        stamp(|t| replace_all(t, r"(?i)module id", "module key"));
    ac1_denial_the_section_header_rows_changed: ["AC1"], "AC1", "the section table opens with the seven common control header rows",
        stamp(|t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"\| u8 \|", "| i8 |"));
    ac1_denial_the_version_width_differs_from_the_hello: ["AC1"], "AC1", "one stamp version number of the hello's width",
        wire(|t| replace_in_line(t, r"^\| 69 \| 4 \| u32 \| Regime stamp version number", r"\| 4 \| u32", "| 8 | u64"));
    ac1_denial_no_version_comparison: ["AC1"], "AC1", "the version comparison of the hello",
        stamp(|t| replace_all(&replace_all(t, r"(?i)hello|serial|modulo|half the range", "x"), r"2\^31", "x"));
    ac1_denial_no_section_length_row: ["AC1", "AC5"], "AC1", "a row for the section length",
        stamp(|t| delete_row(t, r"^\| 56 \| 2 \| u16 \| Section length"));
    ac1_denial_section_rows_that_do_not_tile: ["AC1", "AC5"], "AC1", "the section table tiles its bytes",
        stamp(|t| replace_in_line(t, r"^\| 52 \| 1 \| u8 \| Section index", r"^\| 52", "| 51"));
    ac1_denial_the_section_maximum_not_stated: ["AC1"], "AC1", "states the maximum size of a section as 1252 bytes",
        stamp(|t| replace_all(t, "1 252 bytes", "1 252 octets"));
    ac1_denial_the_request_header_rows_changed: ["AC1"], "AC1", "the stamp request opens with",
        sub("^Stamp request$", |t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"\| u8 \|", "| i8 |"));
    ac3_denial_the_requested_module_id_narrowed: ["AC1", "AC3", "AC5"], "AC3", "names the requested module by its",
        sub("^Stamp request$", |t| replace_in_line(t, r"^\| 32 \| 8 \| u64 \| Requested module id", r"\| 8 \| u64", "| 4 | u32"));
    ac1_denial_request_rows_that_do_not_tile: ["AC1", "AC5"], "AC1", "the request table tiles its bytes",
        sub("^Stamp request$", |t| replace_in_line(t, r"^\| 32 \| 8 \| u64 \| Requested module id", r"^\| 32", "| 33"));
    ac1_denial_the_request_maximum_not_stated: ["AC1"], "AC1", "maximum size of the request as 40 bytes",
        stamp(|t| replace_all(t, "40 bytes", "40 octets"));
    ac1_denial_a_type_code_outside_the_block: ["AC1"], "AC1", "the request message type is a code inside the block",
        sub("^Stamp request$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type", "37 only", "52 only"));
    ac2_denial_no_maximum_section_count: ["AC2"], "AC2", "states a maximum section count in its valid range",
        stamp(|t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "at least one"));
    ac2_denial_the_section_count_not_stated: ["AC2"], "AC2", "a sentence states the maximum section count",
        stamp(|t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 77"));
    ac2_denial_the_peer_constants: ["AC2"], "AC2", "the constants give",
        consts(|t| replace_in_line(t, r"^\| C-217 \|", "63 peers", "62 peers"));
    ac2_denial_the_stamp_size_not_stated: ["AC2"], "AC2", "states the maximum stamp size",
        stamp(|t| replace_all(t, "80 128 bytes", "80 128 octets"));
    ac2_denial_a_storage_figure_not_a_multiple: ["AC2"], "AC2", "every storage figure stated in bytes",
        stamp(|t| replace_all(t, "160 256 bytes of storage", "160 000 bytes of storage"));
    ac2_denial_an_inexact_product: ["AC2"], "AC2", "every product stated in the section is exact",
        stamp(|t| replace_all(t, "64 × 1 194 = 76 416", "64 × 1 194 = 76 417"));
    ac2_denial_a_total_without_its_reason: ["AC2"], "AC2", "every total over the peers",
        stamp(|t| replace_in_line(t, "^A receiver needs at most 160 256 bytes", r"\(C-217; at most 64 modules, C-34\)", "(at most 64 modules)"));
    ac3_denial_no_multicast_on_change: ["AC3"], "AC3", "multicast to the control group on any regime change",
        stamp(|t| replace_all(t, "(?i)multicast", "broadcast"));
    ac3_denial_no_refresh_period: ["AC3"], "AC3", "refreshed every",
        stamp(|t| replace_all(t, r"\b10 s\b", "ten s"));
    ac3_denial_no_unicast_request: ["AC3"], "AC3", "a request is unicast",
        stamp(|t| replace_all(t, "(?i)unicast", "direct"));
    ac3_denial_no_version_bump: ["AC1", "AC3", "AC4"], "AC3", "bumps the stamp version",
        stamp(|t| replace_all(t, "(?i)version", "edition"));
    ac3_denial_no_unknown_type_skip: ["AC3"], "AC3", "skips an unknown section type",
        stamp(|t| replace_all(t, "(?i)unknown|not know|unrecogni", "odd"));
    ac3_denial_no_incomplete_discard: ["AC3"], "AC3", "discards an incomplete stamp",
        stamp(|t| replace_all(t, "(?i)incomplete", "partial"));
    ac3_denial_no_completeness_test: ["AC3"], "AC3", "holds a complete stamp of one version",
        stamp(|t| replace_all(t, "(?i)complete", "whole"));
    ac3_denial_no_sender_part: ["AC3"], "AC3", "has a sender part and a receiver part",
        wire(|t| rename_heading(t, "^Sender rules$", "Origin rules"));
    ac3_denial_the_return_path_not_a_decision: ["AC3"], "AC3", "is tagged [decision]",
        stamp(|t| replace_all(t, r"\[decision\]", "[code]"));
    ac3_denial_no_first_stamp_rule: ["AC3"], "AC3", "for the first section of a peer",
        stamp(|t| replace_in_line(&replace_in_line(t, r"^3\. \*\*The first stamp", "starts the reassembly", "opens the reassembly"), "(?i)it starts at 0 when the entry first carries", "first carries", "carries"));
    ac3_denial_no_new_instance_rule: ["AC3"], "AC3", "a different instance nonce",
        stamp(|t| replace_in_line(t, r"^2\. \*\*A new instance", "A new instance", "Another instance"));
    ac3_denial_no_fetch_only_when_stale: ["AC3"], "AC3", "only when its held copy is stale",
        stamp(|t| replace_in_line(t, r"^9\. \*\*Fetch", "only when", "when"));
    ac3_denial_the_fetch_load_open_item_without_its_owners: ["AC3"], "AC3", "names 011-23 and 005-07",
        stamp(|t| replace_all(t, "011-23", "011-99"));
    ac4_denial_the_enumeration_not_thirteen: ["AC4"], "AC4", "has 13 members",
        wire(|t| delete_row(t, r"^\| 9 \| BadPacketIndex \|"));
    ac4_denial_no_validity_table: ["AC4"], "AC4", "has a Validity rules table",
        stamp(|t| replace_all(t, r"\| Invalid form \|", "| Form |"));
    ac4_denial_fewer_than_seven_forms: ["AC4"], "AC4", "at least 7 invalid forms",
        stamp(|t| replace_in_line(t, r"^\| foreign domain: a non-zero", "$", "\n"));
    ac4_denial_no_oversize_row: ["AC4"], "AC4", "the invalid forms include an oversize section",
        stamp(|t| delete_row(t, r"^\| oversize: a datagram larger"));
    ac4_denial_an_outcome_naming_no_member: ["AC4"], "AC4", "names only members of the Rejection enumeration",
        stamp(|t| replace_in_line(t, r"^\| bad magic", "BadMagic", "WrongMagic"));
    ac4_denial_oversize_not_oversize_control: ["AC4"], "AC4", "the oversize section (C-17) is outcome OversizeControl",
        stamp(|t| replace_in_line(t, r"^\| oversize: a datagram larger", "OversizeControl", "BadLength"));
    ac4_denial_disagreeing_sections_name_two_reasons: ["AC4"], "AC4", "every row whose sections disagree names exactly one",
        stamp(|t| replace_in_line(t, r"^\| sections of one stamp version disagree", r"\|$", "and ForeignDomain |"));
    ac4_denial_an_identical_repeat_named_a_rejection: ["AC4"], "AC4", "one stated outcome for an identical repeat",
        stamp(|t| replace_in_line(t, r"^\| identical repeat:", r"\|$", "and BadLength |"));
    ac4_denial_a_toml_reason_outside_the_control_layer: ["AC4"], "AC4", "rejection_reasons are members whose Layers include control",
        toml(|t| replace_all(t, &regex::escape(&format!("{SEC_HEAD}rejection_reasons = [")), &format!("{SEC_HEAD}rejection_reasons = [\"BadChecksum\", ")));
    ac4_denial_a_table_reason_missing_from_wire_toml: ["AC4"], "AC4", "every reason the stamp's validity table names",
        toml(|t| replace_all(t, r#", "OversizeControl""#, ""));
    ac5_denial_wire_toml_does_not_parse: ["AC4", "AC5"], "AC5", "does not parse",
        toml(|t| format!("{t}\n[[[broken"));
    ac5_denial_no_section_message: ["AC5"], "AC5", "a wire.toml message with section_of",
        toml(|t| replace_all(t, &regex::escape(SEC_HEAD), &SEC_HEAD.replace("section_of = \"regime stamp\"\n", "")));
    ac5_denial_no_request_message: ["AC3", "AC5"], "AC5", "a wire.toml message without section_of",
        sub("^Stamp request$", |t| replace_in_line(t, r"^\| 32 \| 8 \| u64 \| Requested module id", r"\| 8 \| u64", "| 4 × max 2 | u32"));
    ac5_denial_a_parent_that_is_not_a_name: ["AC5"], "AC5", "names its parent",
        toml(|t| replace_all(t, &regex::escape(SEC_HEAD), &SEC_HEAD.replace("\"regime stamp\"\n", "true\n")));
    ac5_denial_a_max_size_off_the_tiling: ["AC5"], "AC5", "tiles from bit 0 to max_size",
        toml(|t| replace_all(t, &regex::escape(SEC_HEAD), &SEC_HEAD.replace("1252", "1253")));
    ac5_denial_the_section_message_not_the_table: ["AC5"], "AC5", "a stamp section message of wire.toml is the markdown section table",
        toml(|t| replace_all(t, &regex::escape(SEC_HEAD), &SEC_HEAD.replace("1252", "1253")));
    ac5_denial_the_wrong_dscp: ["AC5"], "AC5", "is layer control at dscp",
        toml(|t| replace_all(t, &regex::escape(SEC_HEAD), &SEC_HEAD.replace("dscp = 40", "dscp = 46")));
    ac5_denial_an_exemption: ["AC5"], "AC5", "carries no exemption",
        toml(|t| replace_all(t, &regex::escape(SEC_HEAD), &format!("{SEC_HEAD}exempt_from_control_bound = \"C-116\"\n")));
    ac5_denial_the_request_size_differs: ["AC5"], "AC5", "agree row for row in offset, width and count, and in size",
        toml(|t| replace_all(t, &regex::escape(REQ_HEAD), &REQ_HEAD.replace("max_size = 40", "max_size = 41")));
    dod_denial_an_untagged_line: ["DoD"], "DoD", "every prose paragraph of 5 or more words in the regime stamp section",
        stamp(|t| strip_tags(t, "^The maximum section count is 64"));
    dod_denial_an_untagged_subsection: ["DoD"], "DoD", "every subsection of the regime stamp section carries an evidence tag",
        stamp(|t| strip_tags(t, "^wire.toml declares the stamp section as"));
    dod_denial_an_open_item_without_an_owner: ["DoD"], "DoD", "every Open item paragraph",
        stamp(|t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "Open item 99: the stamp's spare bits are unassigned `[decision]`.\n\n"));
    ac5_denial_a_multi_line_string_the_reader_rejects: ["AC4", "AC5"], "AC5", "does not parse",
        toml(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n"));
    ac4_denial_a_heading_after_a_python_line_break: ["AC3", "AC4", "DoD"], "AC4", "has a Validity rules table",
        stamp(|t| replace_in_line(t, "^Every malformed section or request is dropped", "^", "\u{2028}## Interlude\u{2028}"));
    ac1_denial_a_word_character_only_python_knows: ["AC1"], "AC1", "a row for the typed body",
        stamp(|t| replace_all(&replace_all(t, r"\bBody\b", "Body²"), r"\bbody\b", "body²"));
    ac4_denial_a_validity_table_without_separator: ["AC4"], "AC4", "has a Validity rules table",
        stamp(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n"));
    accepted_a_separator_row_only_the_scripts_accept: [], "", "",
        stamp(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n"));
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

/// The tree at the commit that added the script (045f432, "failing checks
/// for the regime stamp message and request") fails every criterion, as
/// the script did. Skipped when the commit is not in the clone.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "045f4321a06f998d5161406b56d75c9cc7ec662c";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "AC5", "DoD"],
        "{}",
        r.render()
    );
}
