//! Ticket 004-16 — the hello message layout and the 32-member size bound.
//! Port of `checks/004-16.sh` (deleted with this port; `doccheck-equiv
//! 004-16` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_16`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and DoD item), on one denial per assertion of the script (the
//! real documents with one mutation applied in memory), on denials and
//! acceptance probes aimed at the places where Python and Rust read text
//! differently, and on the red-phase tree. Every denial and probe is also
//! run against the old script by the ignored test `old_script_agrees`.
//!
//! Not here, because they run tools (they are in
//! `scripts/repo-state.d/004-16.sh`): `cargo run --bin wire-check` exits 0
//! and reports "checks clean"; `cargo test -p hadronyche-tools --test
//! wire_tables` exits 0 and its last result line is exactly 12 passed, 0
//! failed, 0 ignored (script lines 1412-1433). The script skipped both under
//! its `WIRE_MD`/`WIRE_TOML` overrides.
//!
//! Unreachable and kept: `fields_of`'s "the table does not have the five
//! columns" (line 268): every caller passes a table already chosen by
//! `is_field_table`. The port reproduces it.
//!
//! Header and code: the header (lines 76-81) lists the exact reasons of the
//! ticket's list; the code also requires the `counter`, `name` and
//! `collision` rows and the collision sentence (lines 1254-1262), and
//! reports the falsified verdict's missing fallback under AC3 (line 777).
//! The port follows the code.

#[macro_use]
mod wp7_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_16::{WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{
    delete_row, delete_unit, rename_heading, replace_all, replace_in_line, strip_tags,
    within_section,
};
use hadronyche_tools::doccheck::ticket::CONSTANTS_MD;
use hadronyche_tools::doccheck::wire_toml::WIRE_TOML;
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion};

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

fn wire(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(WIRE_MD, edit)
}

/// wire.md with `edit` applied to the Hello section only.
fn hello(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(b, |t| within_section(t, "^Hello$", edit))
}

/// wire.md with `edit` applied to one `###` subsection of the Hello section.
fn sub(b: &Docs, title: &str, edit: impl FnOnce(&str) -> String) -> Docs {
    hello(b, |t| within_section(t, title, edit))
}

fn toml(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(WIRE_TOML, edit)
}

fn consts(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(CONSTANTS_MD, edit)
}

/// Replaces the first literal `from` (which must exist).
fn lit(t: &str, from: &str, to: &str) -> String {
    assert!(t.contains(from), "lit: {from:?} not found");
    t.replacen(from, to, 1)
}

/// Replaces the first `from` after the first `anchor`.
fn after(t: &str, anchor: &str, from: &str, to: &str) -> String {
    let at = t
        .find(anchor)
        .unwrap_or_else(|| panic!("after: no {anchor:?}"));
    let rel = t[at..]
        .find(from)
        .unwrap_or_else(|| panic!("after: no {from:?}"));
    let i = at + rel;
    format!("{}{}{}", &t[..i], to, &t[i + from.len()..])
}

/// Replaces `from` in the toml hello message (the first after its name).
fn hello_toml(b: &Docs, from: &str, to: &str) -> Docs {
    toml(b, |t| after(t, "name = \"hello\"\n", from, to))
}

/// Appends a paragraph to the end of the Hello section.
fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n\n", t.trim_end())
}

const VERDICT: &str = "**Verdict: the hello fits.**";

/// The verdict paragraph replaced by `para`.
fn verdict(t: &str, para: &str) -> String {
    let start = t.find(VERDICT).expect("the verdict paragraph");
    let end = t[start..].find("\n\n").map_or(t.len(), |k| start + k);
    format!("{}{para}{}", &t[..start], &t[end..])
}

const FALSIFIED: &str =
    "**Verdict: the hello is falsified.** C-17 is falsified: the hello is split `[decision]`.";

/// The layout without its bundle count and bundle rows.
fn no_bundle_rows(t: &str) -> String {
    let t = delete_row(t, r"^\| 75 \| 1 \| u8 \| Bundle count");
    delete_row(&t, r"^\| 465 \| 98 × max 8")
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Every hello content has a row with its width; the capability bits
/// carry the holder-eligible bit; the port and content-type maxima; the
/// bundle group id, member list and channel map; the liveness counter is
/// the message counter; the regime stamp version number row." Script: rows
/// by Meaning prefix with the constants' widths (C-10, C-11), a counted UTF-8
/// name, 32 capability bits with a stated holder-eligible bit, a counted
/// port row whose width is its element's sum, the bundle and port element
/// subsections, the liveness-counter sentence.
#[test]
fn ac1_every_hello_content_has_a_row() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The maximum hello size equals the sum of the layout rows; the
/// 32-member size is computed; the verdict fits or is falsified with a
/// fallback." Script: the bundle row counted and equal to its element,
/// the stated bundles-per-hello maximum and its rule, the one-bundle figure,
/// "maximum hello size ... N bytes", the verdict paragraph classified with
/// quoted text removed and "not fits" ignored, each verdict's arithmetic.
#[test]
fn ac2_the_sizes_and_the_verdict() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The six alarms of C-173, one bit or value each, distinct; the
/// transport role is a proposal with epic 005 as the confirming owner and
/// its falsifier."
#[test]
fn ac3_the_alarms_and_the_transport_role() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "At least ten invalid forms; the exact reasons of the ticket's
/// list; member reasons or argued non-rejections; no reason invented." The
/// closed enumeration has exactly 13 members (`REJECTION_PIN`).
#[test]
fn ac4_ten_invalid_forms_with_member_reasons() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "The common control header is defined once, sums to its stated
/// size, the hello's table starts with it, and its order is the sync
/// header's."
#[test]
fn ac5_the_common_control_header() {
    assert_criterion(report(), "AC5");
}

/// AC6 (document part) — "wire.toml hello: layer control, dscp 40, max_size
/// the Markdown sum within C-17, fields one to one, nested maxima,
/// rejection_reasons complete", read with the scripts' TOML subset reader.
#[test]
fn ac6_wire_toml_agrees_with_the_layout() {
    assert_criterion(report(), "AC6");
}

/// Work 3 — the regime stamp version number's stale rule and its own
/// message (C-18).
#[test]
fn work3_the_regime_stamp_version_number() {
    assert_criterion(report(), "Work3");
}

/// Behaviour restated as fact: 1 s (C-30), TTL 1 over IPv4 with unicast
/// (C-100), CS5 (40) (C-13), binary not the POC's text gossip, 64 peers
/// (C-34), with the constants' values.
#[test]
fn behaviour_restated_as_fact() {
    assert_criterion(report(), "Behaviour");
}

/// DoD — tags on every prose paragraph of 5+ words and on every table's
/// subsection, open items with owner and falsifier, the regime stamp fields
/// documented, the constants readable.
#[test]
fn dod_tags_open_items_and_regime_stamp() {
    assert_criterion(report(), "DoD");
}

// ---------------------------------------------------------------- denials

denials! {
    // ---- setup (lines 348, 357, 378, 387, 396)
    dod_denial_a_constant_unreadable => ("DoD", "constant for bound could not be read", |b| consts(b, |t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most twelve hundred bytes")));
    denial_wire_md_missing => ("AC1", "does not exist", |b: &Docs| b.without(WIRE_MD));
    ac4_denial_no_rejection_reasons_section => ("AC4", "no 'Rejection reasons' section", |b| wire(b, |t| rename_heading(t, "^Rejection reasons$", "Refusals")));
    ac4_denial_the_enumeration_not_thirteen => ("AC4", "has 12 names, not 13", |b| wire(b, |t| delete_row(t, r"^\| 9 \| BadPacketIndex \|")));
    ac1_denial_no_hello_section => ("AC1", "no '## Hello' section", |b| wire(b, |t| rename_heading(t, "^Hello$", "Greeting")));

    // ---- fields_of (lines 272, 275, 279)
    ac1_denial_a_layout_row_of_four_cells => ("AC1", "a row has fewer than five cells", |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |", "| 32 | 8 | u64 | Module id (C-11) of the sender |")));
    ac1_denial_an_offset_not_a_number => ("AC1", "is not a byte offset", |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| x32 | 8 | u64 | Module id")));
    ac1_denial_a_width_not_a_count => ("AC1", "is neither a byte count", |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | eight | u64 | Module id")));

    // ---- AC5 (lines 433-522)
    ac5_denial_no_common_header_subsection => ("AC5", "no '### Common control header' subsection", |b| hello(b, |t| rename_heading(t, "^Common control header$", "Shared header")));
    ac5_denial_the_header_subsection_has_no_field_table => ("AC5", "has no five-column field table", |b| sub(b, "^Common control header$", |t| lit(t, "| Offset (bytes) |", "| Position |")));
    ac5_denial_a_header_row_of_no_known_field => ("AC5", "must carry magic, version", |b| sub(b, "^Common control header$", |t| lit(t, "| Magic 0x4d4e", "| Marker 0x4d4e")));
    ac5_denial_a_header_width_against_the_constants => ("AC5", "common control header tag is 120 bits", |b| sub(b, "^Common control header$", |t| lit(t, "| 16 | 16 | u128 |", "| 16 | 15 | u128 |")));
    ac5_denial_a_header_row_out_of_place => ("AC5", "at byte 9, expected 8", |b| sub(b, "^Common control header$", |t| lit(t, "| 8 | 8 | u64 |", "| 9 | 8 | u64 |")));
    ac5_denial_the_header_not_32_bytes => ("AC5", "sums to 33 bytes", |b| sub(b, "^Common control header$", |t| lit(t, "| 16 | 16 | u128 |", "| 16 | 17 | u128 |")));
    ac5_denial_no_stated_header_size => ("AC5", "no sentence states the common control header's size", |b| hello(b, |t| lit(t, "the common control header, 32 bytes,", "the common control header (thirty-two octets),")));
    ac5_denial_a_stated_header_size_that_differs => ("AC5", "states 33 bytes", |b| hello(b, |t| lit(t, "the common control header, 32 bytes,", "the common control header, 33 bytes,")));
    ac5_denial_no_reuse_sentence => ("AC5", "reuse the common control header", |b| hello(b, |t| replace_all(t, "reuse", "adopt")));
    ac5_denial_no_sync_section => ("AC5", "no 'Sync request and reply' section", |b| wire(b, |t| rename_heading(t, "^Sync request and reply$", "Sync messages")));
    ac5_denial_an_order_differing_from_sync_without_excuse => ("AC5", "differs from the sync header order", |b| wire(b, |t| within_section(t, "^Sync request and reply$", |s| lit(s, "| Magic 0x4d4e", "| Marker 0x4d4e"))));
    ac5_denial_an_excuse_that_does_not_name_the_field => ("AC5", "does not name the differing field 'magic'", |b| {
        let b = wire(b, |t| within_section(t, "^Sync request and reply$", |s| lit(s, "| Magic 0x4d4e", "| Marker 0x4d4e")));
        sub(&b, "^Common control header$", |t| append(t, "The hello header differs from the sync header in its first row `[decision]`."))
    });
    ac5_denial_no_layout_subsection => ("AC5", "no '### Hello layout' subsection", |b| hello(b, |t| rename_heading(t, "^Hello layout$", "Hello fields")));
    ac5_denial_the_layout_subsection_has_no_field_table => ("AC5", "the hello layout subsection has no five-column field table", |b| sub(b, "^Hello layout$", |t| lit(t, "| Offset (bytes) |", "| Position |")));
    ac5_denial_the_layout_not_starting_with_the_header => ("AC5", "does not start with the common control header rows", |b| sub(b, "^Hello layout$", |t| lit(t, "| Magic 0x4d4e", "| Marker 0x4d4e")));
    ac5_denial_a_layout_header_row_of_another_type => ("AC5", "differs from the common control header in offset, width or type", |b| sub(b, "^Hello layout$", |t| lit(t, "| 7 | 1 | u8 | Authentication scheme", "| 7 | 1 | i8 | Authentication scheme")));
    ac5_denial_layout_rows_that_do_not_tile => ("AC5", "at byte 53, expected 52", |b| sub(b, "^Hello layout$", |t| lit(t, "| 52 | 4 | u32 | Election epoch", "| 53 | 4 | u32 | Election epoch")));
    ac5_denial_a_row_of_part_of_a_byte => ("AC5", "is not a whole number of bytes", |b| sub(b, "^Hello layout$", |t| lit(t, "| 52 | 4 | u32 | Election epoch", "| 52 | 31 bits | u32 | Election epoch")));

    // ---- AC1 (lines 546-636)
    ac1_denial_no_transport_role_row => ("AC1", "no row for transport role", |b| sub(b, "^Hello layout$", |t| lit(t, "| Transport role: 0 = none", "| Role of transport: 0 = none")));
    ac1_denial_a_module_id_of_the_wrong_width => ("AC1", "module id (C-11) is 32 bits", |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | 4 | u64 | Module id")));
    ac1_denial_no_counted_name_row => ("AC1", "no counted name row", |b| sub(b, "^Hello layout$", |t| lit(t, "| 76 | 1 × max 32 | u8 | Name:", "| 76 | 32 | u8 | Name:")));
    ac1_denial_a_name_without_its_encoding => ("AC1", "does not state its encoding", |b| sub(b, "^Hello layout$", |t| replace_in_line(&lit(t, "UTF-8 (RFC 3629), name length", "Unicode, name length"), r"^\| 76 \|", "valid UTF-8", "valid Unicode")));
    ac1_denial_capability_bits_not_32 => ("AC1", "capability bits are 16 bits", |b| sub(b, "^Hello layout$", |t| lit(t, "| 48 | 4 | u32 | Capability bits", "| 48 | 2 | u32 | Capability bits")));
    ac1_denial_no_holder_eligible_bit => ("AC1", "does not fix the holder-eligible bit", |b| sub(b, "^Hello layout$", |t| lit(t, "holder-eligible is bit 0", "holder-eligible is the low bit")));
    ac1_denial_no_counted_port_row => ("AC1", "no counted port row", |b| sub(b, "^Hello layout$", |t| lit(t, "| 108 | 7 × max 51 |", "| 108 | 7 |")));
    ac1_denial_the_counter_not_the_liveness_counter => ("AC1", "is not stated as the data-plane liveness counter", |b| hello(b, |t| replace_all(t, "(?i)liveness counter", "liveness tally")));
    ac1_denial_no_sentence_on_the_liveness_counter => ("AC1", "no sentence says that the message counter doubles", |b| hello(b, |t| replace_all(t, "liveness counter", "liveness tally")));
    ac1_denial_no_bundle_element_subsection => ("AC1", "no '### Bundle element' subsection", |b| hello(b, |t| rename_heading(t, "^Bundle element$", "Bundle entry")));
    ac1_denial_no_channel_map_row => ("AC1", "has no row for 'channel map'", |b| sub(b, "^Bundle element$", |t| lit(t, "| Channel map:", "| Channel widths:")));
    ac2_denial_bundle_element_rows_that_do_not_tile => ("AC2", "at byte 35, expected 34", |b| sub(b, "^Bundle element$", |t| lit(t, "| 34 | 2 × max 32 |", "| 35 | 2 × max 32 |")));
    ac2_denial_a_member_row_above_c08 => ("AC2", "allows 33 members, above C-08's 32", |b| sub(b, "^Bundle element$", |t| lit(t, "| 2 | 1 × max 32 |", "| 2 | 1 × max 33 |")));
    ac2_denial_a_member_row_not_at_c08 => ("AC2", "has max 31; C-08 fixes 32", |b| sub(b, "^Bundle element$", |t| lit(t, "| 2 | 1 × max 32 |", "| 2 | 1 × max 31 |")));
    ac2_denial_no_member_row_at_c08 => ("AC2", "has no counted member row with the maximum of 32", |b| sub(b, "^Bundle element$", |t| lit(&lit(t, "| 2 | 1 × max 32 |", "| 2 | 1 × max 31 |"), "| 34 | 2 × max 32 |", "| 34 | 2 × max 31 |")));
    ac1_denial_no_port_element_subsection => ("AC1", "no '### Port element' subsection", |b| hello(b, |t| rename_heading(t, "^Port element$", "Port entry")));
    ac1_denial_no_counted_content_type_row => ("AC1", "no counted content-type row", |b| sub(b, "^Port element$", |t| lit(t, "| 3 | 1 × max 4 |", "| 3 | 1 |")));
    ac1_denial_a_port_row_unequal_to_its_element => ("AC1", "its element sums to 64 bits", |b| sub(b, "^Port element$", |t| lit(t, "| 2 | 1 | u8 | Content-type count", "| 2 | 2 | u8 | Content-type count")));

    // ---- AC2 (lines 651-777)
    ac2_denial_the_bundle_row_not_counted => ("AC2", "is not a counted row", |b| sub(b, "^Hello layout$", |t| lit(t, "| 465 | 98 × max 8 |", "| 465 | 98 |")));
    ac2_denial_the_bundle_row_unequal_to_its_element => ("AC2", "the bundle row is 776 bits per bundle", |b| sub(b, "^Hello layout$", |t| lit(t, "| 465 | 98 × max 8 |", "| 465 | 97 × max 8 |")));
    ac2_denial_no_one_byte_bundle_count => ("AC2", "no one-byte bundle count row", |b| sub(b, "^Hello layout$", |t| lit(t, "| 75 | 1 | u8 | Bundle count", "| 75 | 2 | u8 | Bundle count")));
    ac2_denial_no_stated_bundles_per_hello => ("AC2", "states no bundles-per-hello maximum", |b| hello(b, |t| replace_all(t, r"at most (\d+) bundles", "no more than so many bundles")));
    ac2_denial_a_stated_maximum_unequal_to_the_row => ("AC2", "the stated bundles-per-hello maximum is 9", |b| hello(b, |t| lit(t, "A hello is at most 8 bundles", "A hello is at most 9 bundles")));
    ac2_denial_no_rule_for_more_bundles => ("AC2", "no rule or 'Open item'", |b| hello(b, |t| {
        let t = delete_unit(t, "^The rule for a source with more than 8 bundles");
        let t = lit(&lit(&t, "**Open item 2 ", "**Note 2 "), "**Open item 3 ", "**Note 3 ");
        replace_all(&t, r"\brule\b", "law")
    }));
    ac2_denial_too_few_bundles_for_c09 => ("AC2", "cannot carry C-09's 256 streams", |b| hello(b, |t| {
        let t = lit(t, "| 465 | 98 × max 8 |", "| 465 | 98 × max 7 |");
        let t = lit(&t, "A hello is at most 8 bundles", "A hello is at most 7 bundles");
        replace_all(&t, "256", "many")
    }));
    ac2_denial_no_one_bundle_figure => ("AC2", "the one-bundle figure is not stated", |b| hello(b, |t| lit(t, "= 563 bytes `[decision]`.", "= 563 `[decision]`.")));
    ac2_denial_a_one_bundle_figure_that_differs => ("AC2", "stated as 564 bytes; the tables compute 563", |b| hello(b, |t| lit(t, "= 563 bytes `[decision]`.", "= 564 bytes `[decision]`.")));
    ac2_denial_no_maximum_hello_size => ("AC2", "the maximum hello size is not stated", |b| hello(b, |t| replace_all(t, "(?i)maximum hello size", "largest hello")));
    ac2_denial_no_verdict_paragraph => ("AC2", "no 'Verdict' paragraph", |b| hello(b, |t| lit(t, VERDICT, "**Outcome: the hello fits.**")));
    ac2_denial_a_verdict_stating_neither => ("AC2", "states neither 'fits' nor 'falsified'", |b| hello(b, |t| verdict(t, "**Verdict: see above.** Nothing more `[decision]`.")));
    ac2_denial_a_verdict_stating_both => ("AC2", "says both 'fits' and 'falsified'", |b| hello(b, |t| lit(t, VERDICT, "**Verdict: the hello fits.** It is falsified.")));
    ac2_denial_a_verdict_saying_c17_falsifier_holds => ("AC2", "says C-17's falsifier holds", |b| hello(b, |t| lit(t, "C-17 holds, and its falsifier", "C-17 holds for the hello, and its falsifier")));
    ac2_denial_fits_without_bundle_rows => ("AC2", "fits but the hello layout carries no bundle rows", |b| sub(b, "^Hello layout$", no_bundle_rows));
    ac2_denial_fits_above_the_bound => ("AC2", "is 1347 bytes, above 1252", |b| hello(b, |t| lit(&lit(t, "| 465 | 98 × max 8 |", "| 465 | 98 × max 9 |"), "A hello is at most 8 bundles", "A hello is at most 9 bundles")));
    ac2_denial_fits_with_a_stated_maximum_that_differs => ("AC2", "stated as 1248 bytes; the table sums to 1249", |b| hello(b, |t| lit(t, "The maximum hello size is 1 249 bytes", "The maximum hello size is 1 248 bytes")));
    ac2_denial_falsified_with_bundle_rows => ("AC2", "still carries the bundle rows", |b| hello(b, |t| verdict(t, FALSIFIED)));
    ac2_denial_falsified_without_an_all_bundles_maximum => ("AC2", "the all-bundles maximum cannot be computed", |b| hello(b, |t| {
        let t = within_section(&verdict(t, FALSIFIED), "^Hello layout$", no_bundle_rows);
        replace_all(&t, r"at most (\d+) bundles", "no more than so many bundles")
    }));
    ac2_denial_falsified_with_an_all_bundles_maximum_within_the_bound => ("AC2", "not above 1252", |b| hello(b, |t| {
        let t = within_section(&verdict(t, FALSIFIED), "^Hello layout$", no_bundle_rows);
        lit(&t, "A hello is at most 8 bundles", "A hello is at most 1 bundles")
    }));
    ac2_denial_falsified_with_a_fallback_above_the_bound => ("AC2", "the fallback hello is 1347 bytes", |b| hello(b, |t| lit(&verdict(t, FALSIFIED), "| 465 | 98 × max 8 |", "| 465 | 98 × max 9 |")));
    ac2_denial_falsified_with_a_stated_maximum_that_differs => ("AC2", "the fallback table sums to 1249", |b| hello(b, |t| lit(&verdict(t, FALSIFIED), "The maximum hello size is 1 249 bytes", "The maximum hello size is 1 248 bytes")));
    ac2_denial_falsified_without_the_all_bundles_size_stated => ("AC2", "the all-bundles size 1347 bytes is not stated", |b| hello(b, |t| {
        let t = within_section(&verdict(t, FALSIFIED), "^Hello layout$", no_bundle_rows);
        lit(&t, "A hello is at most 8 bundles", "A hello is at most 9 bundles")
    }));
    ac2_denial_falsified_citing_c17_against_its_falsifier => ("AC2", "says C-17 is falsified", |b| hello(b, |t| verdict(t, FALSIFIED)));
    ac3_denial_falsified_without_a_fallback => ("AC3", "does not name the fallback", |b| hello(b, |t| {
        let t = verdict(t, "**Verdict: the hello is falsified.** Nothing more `[decision]`.");
        replace_all(&replace_all(&t, "(?i)split", "divid"), "(?i)handshake", "exchange")
    }));

    // ---- AC3 (lines 1088-1154)
    ac3_denial_no_alarm_subsection => ("AC3", "no '### Alarm field' subsection", |b| hello(b, |t| rename_heading(t, "^Alarm field$", "Warnings")));
    ac3_denial_an_alarm_table_of_five_rows => ("AC3", "has no table with one row per alarm", |b| sub(b, "^Alarm field$", |t| delete_row(t, r"^\| Dead \|")));
    ac3_denial_an_alarm_without_a_row => ("AC3", "no row for the alarm 'dead'", |b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Defunct | bit 2 |")));
    ac3_denial_an_alarm_without_a_bit => ("AC3", "the alarm 'dead' has no bit or value", |b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | two |")));
    ac3_denial_two_alarms_on_one_bit => ("AC3", "are not distinct", |b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 1 |")));
    ac3_denial_an_alarm_of_value_zero => ("AC3", "encoded as zero", |b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | value 0 |")));
    ac3_denial_an_alarm_beyond_the_field => ("AC3", "does not fit the alarm field's width", |b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 9 |")));
    ac3_denial_no_transport_role_subsection => ("AC3", "no '### Transport role' subsection", |b| hello(b, |t| rename_heading(t, "^Transport role$", "Roles")));
    ac3_denial_a_role_table_of_one_row => ("AC3", "has no table with its values", |b| sub(b, "^Transport role$", |t| delete_row(&delete_row(t, r"^\| 1 \| follower"), r"^\| 2 \| reference")));
    ac3_denial_the_roles_not_a_proposal => ("AC3", "not recorded as a proposal", |b| sub(b, "^Transport role$", |t| replace_all(t, "(?i)proposal", "draft")));
    ac3_denial_no_role_open_item => ("AC3", "states the transport role open question", |b| hello(b, |t| lit(t, "**Open item 1 — the transport role values.**", "**Item 1 — the transport role values.**")));
    ac3_denial_a_role_open_item_without_owner => ("AC3", "does not name epic 005 as its confirming owner", |b| sub(b, "^Open items$", |t| replace_in_line(t, r"^\*\*Open item 1 ", "(?i)owner", "Confirmed by")));
    ac3_denial_a_role_open_item_without_its_falsifier => ("AC3", "lacks its falsifier", |b| sub(b, "^Open items$", |t| lit(t, "epic 005 needs a role the encoding cannot express", "epic 005 needs another role")));

    // ---- AC4 (lines 1171-1261)
    ac4_denial_no_invalid_form_table => ("AC4", "no invalid-form table", |b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |", "| Case | Result |")));
    ac4_denial_fewer_than_ten_forms => ("AC4", "at least 10 are required", |b| sub(b, "^Validity rules$", |t| lit(t, "\n| foreign domain:", "\n\n| foreign domain:")));
    ac4_denial_an_invented_reason => ("AC4", "names 'WrongMagic', which is not in the closed enumeration", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "WrongMagic")));
    ac4_denial_an_exact_reason_missing => ("AC4", "must be counted under BadMagic", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadLength")));
    ac4_denial_domain_zero_mapped_to_foreign_domain => ("AC4", "domain 0 is mapped to ForeignDomain", |b| sub(b, "^Validity rules$", |t| lit(t, "UnconfiguredDomain — never ForeignDomain", "UnconfiguredDomain or ForeignDomain")));
    ac4_denial_a_version_mismatch_without_its_event => ("AC4", "raises a named event", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| version mismatch", r"\| drop and count under VersionMismatch.*$", "| drop and count under VersionMismatch |")));
    ac4_denial_a_form_naming_no_reason => ("AC4", "names no member of the closed enumeration and argues no non-rejection", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| message type 0", r"\| drop and count under BadLength.*$", "| drop it silently |")));
    ac4_denial_a_collision_row_without_its_argument => ("AC4", "needs a member reason or an argued non-rejection", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| module-id collision", "not a parse rejection", "dropped silently")));
    ac4_denial_no_oversize_row => ("AC4", "covers the 'oversize' case", |b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| oversize")));
    ac4_denial_no_counter_row => ("AC4", "liveness counter going backwards", |b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| liveness counter going backwards")));
    ac4_denial_no_name_row => ("AC4", "a name that is not valid in the chosen encoding", |b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| name bytes not valid")));
    ac4_denial_no_collision_row_nor_sentence => ("AC4", "is not stated as a non-rejection", |b| sub(b, "^Validity rules$", |t| delete_unit(&delete_row(t, r"^\| module-id collision"), "^A module-id collision is not a parse rejection")));
    ac4_denial_no_collision_sentence => ("AC4", "no sentence says that a module-id collision", |b| sub(b, "^Validity rules$", |t| delete_unit(t, "^A module-id collision is not a parse rejection")));

    // ---- Work 3, DoD, Behaviour (lines 1271-1296)
    work3_denial_no_stale_rule => ("Work3", "treats its copy as stale", |b| hello(b, |t| replace_all(t, r"(?i)\bstale\b", "outdated")));
    dod_denial_the_regime_stamp_fields_not_documented => ("DoD", "regime stamp fields the hello touches", |b| hello(b, |t| replace_all(t, r"(?i)version\s+number", "version count")));
    work3_denial_c18_not_cited => ("Work3", "travels in its own message", |b| hello(b, |t| replace_all(t, "C-18", "C-81")));
    behaviour_denial_ttl_missing => ("Behaviour", "multicast with TTL 1", |b| hello(b, |t| replace_all(t, "TTL 1", "TTL one")));
    behaviour_denial_the_hello_period_of_c30 => ("Behaviour", "the hello period is not restated", |b| consts(b, |t| replace_in_line(t, r"^\| C-30 \|", r"\| 1 s \|", "| 7 s |")));
    behaviour_denial_the_cs5_value_of_c13 => ("Behaviour", "CS5 is not stated with the value of C-13", |b| consts(b, |t| replace_in_line(t, r"^\| C-13 \|", r"control CS5 \(40\)", "control CS5 (41)")));
    behaviour_denial_the_peer_bound_of_c34 => ("Behaviour", "the peer map bound is not restated", |b| consts(b, |t| replace_in_line(t, r"^\| C-34 \|", "at most 64", "at most 97")));
    dod_denial_an_untagged_paragraph => ("DoD", "carries no evidence tag", |b| hello(b, |t| strip_tags(t, "^The hello carries no IPv4 address")));
    dod_denial_an_open_item_without_owner => ("DoD", "names no owner or no falsifier", |b| hello(b, |t| append(t, "**Open item 10 — x.** Still open `[decision]`.")));
    dod_denial_a_table_in_an_untagged_subsection => ("DoD", "a table of 'Transport role' follows no evidence-tagged paragraph", |b| sub(b, "^Transport role$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", "")));

    // ---- AC6 (lines 1331-1407)
    ac6_denial_wire_toml_missing => ("AC6", "does not exist", |b: &Docs| b.without(WIRE_TOML));
    ac6_denial_wire_toml_does_not_parse => ("AC6", "does not parse", |b| toml(b, |t| format!("{t}\n[[[broken")));
    ac6_denial_a_hello_named_message_not_control => ("AC6", "toml message 'hello port element' is a hello message", |b| toml(b, |t| after(t, "name = \"hello port element\"\n", "layer = \"control\"", "layer = \"media\"")));
    ac6_denial_no_hello_message => ("AC6", "declares no [[message]] named 'hello'", |b| toml(b, |t| lit(t, "name = \"hello\"\n", "name = \"hullo\"\n")));
    ac6_denial_the_hello_not_control => ("AC6", "the toml hello has layer 'media'", |b| hello_toml(b, "layer = \"control\"", "layer = \"media\""));
    ac6_denial_the_hello_dscp => ("AC6", "the toml hello has dscp 46", |b| hello_toml(b, "dscp = 40", "dscp = 46"));
    ac6_denial_the_hello_max_size => ("AC6", "the toml hello has max_size 1250", |b| hello_toml(b, "max_size = 1249", "max_size = 1250"));
    ac6_denial_the_hello_above_the_bound => ("AC6", "exceeds the 1252-byte control bound", |b| hello_toml(b, "max_size = 1249", "max_size = 1300"));
    ac6_denial_a_field_missing => ("AC6", "they must correspond one to one", |b| hello_toml(b, "[[message.field]]\nname = \"message_type\"\n", "[[message.gone]]\nname = \"message_type\"\n"));
    ac6_denial_a_field_misnamed => ("AC6", "toml hello field 'domain' is not named after", |b| hello_toml(b, "name = \"domain_id\"", "name = \"domain\""));
    ac6_denial_a_field_offset => ("AC6", "offset 33 bits", |b| hello_toml(b, "offset = 32\n", "offset = 33\n"));
    ac6_denial_a_counted_width => ("AC6", "toml hello field name: width 16 bits per element", |b| hello_toml(b, "offset = 608\nwidth = 8", "offset = 608\nwidth = 16"));
    ac6_denial_a_max_count => ("AC6", "toml hello field name: max_count 31", |b| hello_toml(b, "max_count = 32", "max_count = 31"));
    ac6_denial_a_count_naming_no_earlier_field => ("AC6", "its count must name an earlier field", |b| hello_toml(b, "count = \"name_length\"", "count = \"nonexistent\""));
    ac6_denial_a_width => ("AC6", "toml hello field domain_id: width 8 bits", |b| hello_toml(b, "offset = 32\nwidth = 16", "offset = 32\nwidth = 8"));
    ac6_denial_a_type => ("AC6", "toml hello field domain_id: type i16", |b| hello_toml(b, "offset = 32\nwidth = 16\ntype = \"u16\"", "offset = 32\nwidth = 16\ntype = \"i16\""));
    ac6_denial_fields_not_summing_to_max_size => ("AC6", "the toml hello fields compute", |b| hello_toml(b, "max_count = 8\n", "max_count = 9\n"));
    ac6_denial_the_port_field_without_its_nested_maximum => ("AC6", "does not state the maximum 4 of its nested count", |b| toml(b, |t| {
        let t = after(t, "name = \"port\"\noffset = 864", "at most 4 content-type codes", "at most four content-type codes");
        after(&t, "name = \"port\"\noffset = 864", "1 to 4 content types", "1 to four content types")
    }));
    ac6_denial_the_bundle_field_without_32 => ("AC6", "does not state the 32-member maximum", |b| toml(b, |t| after(t, "name = \"bundle\"\noffset = 3720", "1 to 32 members", "1 to many members")));
    ac6_denial_a_named_reason_not_listed => ("AC6", "does not list 'BadMagic'", |b| hello_toml(b, "\"BadMagic\", ", ""));
    ac6_denial_a_listed_reason_not_a_member => ("AC6", "names 'Bogus', which is not in the closed enumeration", |b| hello_toml(b, "rejection_reasons = [", "rejection_reasons = [\"Bogus\", "));

    // ---- Python semantics: denials the script rejects
    /// `\w`: `²` is a word character for Python, so `\bBadMagic\b` does not
    /// match `BadMagic²`.
    py_denial_a_reason_glued_to_a_superscript => ("AC4", "must be counted under BadMagic", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadMagic\u{b2}")));
    /// `\s`: U+001F is whitespace for Python, so `clean_start` strips it and
    /// the paragraph is an open item.
    py_denial_an_open_item_after_a_unit_separator => ("DoD", "names no owner or no falsifier", |b| hello(b, |t| append(t, "\u{1f}**Open item 10 — x.** Still open `[decision]`.")));
    /// `str.splitlines()`: U+2028 ends a line, so the validity table ends in
    /// its sixth row.
    py_denial_a_line_separator_inside_the_validity_table => ("AC4", "at least 10 are required", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| foreign domain", r"\| drop and count under ForeignDomain", "|\u{2028}drop and count under ForeignDomain")));
    /// `str.splitlines()`: so does a form feed.
    py_denial_a_form_feed_inside_the_validity_table => ("AC4", "at least 10 are required", |b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| foreign domain", r"\| drop and count under ForeignDomain", "|\u{c}drop and count under ForeignDomain")));
    /// `int()` is unbounded: a 21-digit offset is read, and does not tile.
    py_denial_an_offset_beyond_64_bits => ("AC5", "at byte 100000000000000000032, expected 32", |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 100000000000000000032 | 8 | u64 | Module id")));
    /// `\d` and `int()` read Arabic-Indic digits: ٣٣ is 33, out of place.
    py_denial_an_offset_in_arabic_indic_digits => ("AC5", "at byte 33, expected 32", |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| \u{663}\u{663} | 8 | u64 | Module id")));
    /// The subset reader keeps `"1249"` a str: unequal to 1249.
    py_denial_max_size_as_a_string => ("AC6", "the toml hello has max_size 1249; the markdown layout sums to 1249", |b| hello_toml(b, "max_size = 1249", "max_size = \"1249\""));
    /// The subset reader iterates a str's characters: no reason is listed.
    py_denial_rejection_reasons_as_a_string => ("AC6", "does not list 'BadMagic'", |b| hello_toml(b, "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]", "rejection_reasons = \"BadMagic\""));
    /// The subset reader cannot read `"""` strings.
    py_denial_a_triple_quoted_string => ("AC6", "does not parse", |b| hello_toml(b, "name = \"domain_id\"", "name = \"\"\"domain_id\"\"\""));
    /// The subset reader lets a later duplicate key win.
    py_denial_a_duplicate_dscp => ("AC6", "the toml hello has dscp 46", |b| hello_toml(b, "dscp = 40\n", "dscp = 40\ndscp = 46\n"));
    /// A number of rejection reasons is not iterable: TypeError.
    py_denial_rejection_reasons_a_number => ("CRASH", "rejection_reasons is not iterable", |b| hello_toml(b, "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]", "rejection_reasons = 5"));
    /// A str width times a count is a str, which the sum rejects: TypeError.
    py_denial_a_counted_width_as_a_string => ("CRASH", "adds a non-number", |b| hello_toml(b, "offset = 608\nwidth = 8", "offset = 608\nwidth = \"8\""));
    /// `key_of(r[3])` on a sync header row of three cells: IndexError.
    py_denial_a_sync_header_row_of_three_cells => ("CRASH", "IndexError", |b| wire(b, |t| within_section(t, "^Sync request and reply$", |s| lit(s, "| 7 | 1 | u8 | Authentication scheme (C-12); 0 = none in v1 (C-115) | 0 only |", "| 7 | 1 | u8 |"))));
    /// The 004 separator line: a cell that is not dashes and colons is no
    /// separator, and the table is dropped whole.
    py_denial_a_separator_with_a_letter => ("AC4", "no invalid-form table", |b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|---|-x-|")));
    /// A run of rows without a separator line is not a table.
    py_denial_a_headerless_alarm_table => ("AC3", "has no table with one row per alarm", |b| sub(b, "^Alarm field$", |t| lit(t, "| Alarm | Bit | Set while (C-173) |\n|---|---|---|\n", "")));
    /// `re.I`: Python matches `i` with `ı`, so this is an open item.
    py_denial_an_open_item_with_a_dotless_i => ("DoD", "names no owner or no falsifier", |b| hello(b, |t| append(t, "**Open \u{131}tem 10 — x.** Still open `[decision]`.")));
    /// `(?<!not )`: "not fits" is not a verdict word.
    py_denial_a_verdict_of_not_fits => ("AC2", "states neither 'fits' nor 'falsified'", |b| hello(b, |t| verdict(t, "**Verdict: the hello not fits here.** Nothing more `[decision]`.")));
}

accepts! {
    /// `\d`/`int()`: ٣٢ is 32.
    py_accepts_an_offset_in_arabic_indic_digits => |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| \u{663}\u{662} | 8 | u64 | Module id"));
    /// A combining accent is not a word character for Python: `\bdead\b`
    /// still finds "Dead\u{301}".
    py_accepts_a_combining_accent_after_an_alarm => |b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead\u{301} | bit 2 |"));
    /// `re.I`: "bınary" is "binary" to Python.
    py_accepts_a_dotless_i => |b| hello(b, |t| replace_all(t, "binary", "b\u{131}nary"));
    /// The 24-character look-back counts characters, not bytes.
    py_accepts_a_negation_within_24_characters_of_wide_ones => |b| sub(b, "^Validity rules$", |t| lit(t, "— never ForeignDomain", "— never ——————— ForeignDomain"));
    /// The subset reader: 40.0 and 0x28 equal 40.
    py_accepts_a_float_dscp => |b| hello_toml(b, "dscp = 40\n", "dscp = 40.0\n");
    py_accepts_a_hex_dscp => |b| hello_toml(b, "dscp = 40\n", "dscp = 0x28\n");
    /// The 004 separator line accepts colons inside a cell.
    py_accepts_a_separator_with_inner_colons => |b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|-:-|---|"));
    /// U+001F is whitespace for `str.strip()`.
    py_accepts_a_unit_separator_in_a_cell => |b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32\u{1f} | 8 | u64 | Module id"));
}

/// Every denial is rejected by the old script by the expected criterion
/// (`CRASH`: a traceback with no FAIL line), and every probe is accepted.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees() {
    let mut rows = all_denials();
    rows.extend(all_accepts());
    let differ = wp7_support::script_disagrees("004-16", &rows);
    assert!(differ.is_empty(), "{}", differ.join("\n"));
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (2e6ba78, "failing checks
/// for the hello layout and the 32-member size bound") fails what the
/// script failed: AC1 to AC6, Behaviour, DoD and Work3. Skipped when the
/// commit is not in the clone.
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    let commit = "2e6ba7805f875587090453369dc0aebe092de7e3";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        [
            "AC1",
            "AC2",
            "AC3",
            "AC4",
            "AC5",
            "AC6",
            "Behaviour",
            "DoD",
            "Work3"
        ],
        "{}",
        r.render()
    );
}
