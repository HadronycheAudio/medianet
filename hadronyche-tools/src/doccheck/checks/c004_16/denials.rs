//! The denial list of 004-16: the real documents with one edit, and what
//! the port (and the old script, run by `doccheck-equiv 004-16`) must
//! report. One or more per `fail` site of the script, then the denials aimed
//! at Python semantics (`py_denial_*`), then the acceptance probes
//! (`py_accepts_*`, no expectation: the port must pass, as the script
//! does). Consumed by `tests/doc_004_16.rs`, one test each.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, delete_unit, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;

/// Replaces the first literal `from` (which must exist).
fn lit(t: &str, from: &str, to: &str) -> String {
    assert!(t.contains(from), "lit: {from:?} not found");
    t.replacen(from, to, 1)
}

/// Replaces the first `from` after the first `anchor`.
fn after(t: &str, anchor: &str, from: &str, to: &str) -> String {
    let at = t.find(anchor).unwrap_or_else(|| panic!("after: no {anchor:?}"));
    let rel = t[at..].find(from).unwrap_or_else(|| panic!("after: no {from:?}"));
    let i = at + rel;
    format!("{}{}{}", &t[..i], to, &t[i + from.len()..])
}

/// Appends a paragraph at the end of `t`.
fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n\n", t.trim_end())
}

/// The whole file (the edit names the file it applies to).
fn wire(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    edit(t)
}

/// wire.toml.
fn toml(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    edit(t)
}

/// The constants table.
fn consts(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    edit(t)
}

/// The Hello section.
fn hello(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Hello$", edit)
}

/// One `###` subsection of the Hello section.
fn sub(t: &str, title: &str, edit: impl FnOnce(&str) -> String) -> String {
    hello(t, |s| within_section(s, title, edit))
}

/// `from` replaced in the toml hello message.
fn hello_toml(t: &str, from: &str, to: &str) -> String {
    after(t, "name = \"hello\"\n", from, to)
}

const VERDICT: &str = "**Verdict: the hello fits.**";

/// The verdict paragraph replaced by `para`.
fn verdict(t: &str, para: &str) -> String {
    let start = t.find(VERDICT).expect("the verdict paragraph");
    let end = t[start..].find("\n\n").map_or(t.len(), |k| start + k);
    format!("{}{para}{}", &t[..start], &t[end..])
}

const FALSIFIED: &str = "**Verdict: the hello is falsified.** C-17 is falsified: the hello is split `[decision]`.";

/// The layout without its bundle count and bundle rows.
fn no_bundle_rows(t: &str) -> String {
    let t = delete_row(t, r"^\| 75 \| 1 \| u8 \| Bundle count");
    delete_row(&t, r"^\| 465 \| 98 × max 8")
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "dod_denial_a_constant_unreadable",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most twelve hundred bytes"))),
        expect: &[("DoD", "constant for bound could not be read"), ("AC2", "")],
    },
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", ""), ("Behaviour", ""), ("DoD", ""), ("Work3", "")],
    },
    Denial {
        name: "ac4_denial_no_rejection_reasons_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Rejection reasons$", "Refusals"))),
        expect: &[("AC4", "no 'Rejection reasons' section")],
    },
    Denial {
        name: "ac4_denial_the_enumeration_not_thirteen",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| delete_row(t, r"^\| 9 \| BadPacketIndex \|"))),
        expect: &[("AC4", "has 12 names, not 13")],
    },
    Denial {
        name: "ac1_denial_no_hello_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Hello$", "Greeting"))),
        expect: &[("AC1", "no '## Hello' section"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", ""), ("Behaviour", ""), ("DoD", ""), ("Work3", "")],
    },
    Denial {
        name: "ac1_denial_a_layout_row_of_four_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |", "| 32 | 8 | u64 | Module id (C-11) of the sender |"))),
        expect: &[("AC1", "a row has fewer than five cells"), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_an_offset_not_a_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| x32 | 8 | u64 | Module id"))),
        expect: &[("AC1", "is not a byte offset"), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_width_not_a_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | eight | u64 | Module id"))),
        expect: &[("AC1", "is neither a byte count"), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_no_common_header_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| rename_heading(t, "^Common control header$", "Shared header"))),
        expect: &[("AC5", "no '### Common control header' subsection")],
    },
    Denial {
        name: "ac5_denial_the_header_subsection_has_no_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Common control header$", |t| lit(t, "| Offset (bytes) |", "| Position |"))),
        expect: &[("AC5", "has no five-column field table")],
    },
    Denial {
        name: "ac5_denial_a_header_row_of_no_known_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Common control header$", |t| lit(t, "| Magic 0x4d4e", "| Marker 0x4d4e"))),
        expect: &[("AC5", "must carry magic, version")],
    },
    Denial {
        name: "ac5_denial_a_header_width_against_the_constants",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Common control header$", |t| lit(t, "| 16 | 16 | u128 |", "| 16 | 15 | u128 |"))),
        expect: &[("AC5", "common control header tag is 120 bits")],
    },
    Denial {
        name: "ac5_denial_a_header_row_out_of_place",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Common control header$", |t| lit(t, "| 8 | 8 | u64 |", "| 9 | 8 | u64 |"))),
        expect: &[("AC5", "at byte 9, expected 8")],
    },
    Denial {
        name: "ac5_denial_the_header_not_32_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Common control header$", |t| lit(t, "| 16 | 16 | u128 |", "| 16 | 17 | u128 |"))),
        expect: &[("AC5", "sums to 33 bytes")],
    },
    Denial {
        name: "ac5_denial_no_stated_header_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "the common control header, 32 bytes,", "the common control header (thirty-two octets),"))),
        expect: &[("AC5", "no sentence states the common control header's size")],
    },
    Denial {
        name: "ac5_denial_a_stated_header_size_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "the common control header, 32 bytes,", "the common control header, 33 bytes,"))),
        expect: &[("AC5", "states 33 bytes")],
    },
    Denial {
        name: "ac5_denial_no_reuse_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "reuse", "adopt"))),
        expect: &[("AC5", "reuse the common control header")],
    },
    Denial {
        name: "ac5_denial_no_sync_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Sync request and reply$", "Sync messages"))),
        expect: &[("AC5", "no 'Sync request and reply' section")],
    },
    Denial {
        name: "ac5_denial_an_order_differing_from_sync_without_excuse",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Sync request and reply$", |s| lit(s, "| Magic 0x4d4e", "| Marker 0x4d4e")))),
        expect: &[("AC5", "differs from the sync header order")],
    },
    Denial {
        name: "ac5_denial_an_excuse_that_does_not_name_the_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| {
        let b = wire(b, |t| within_section(t, "^Sync request and reply$", |s| lit(s, "| Magic 0x4d4e", "| Marker 0x4d4e")));
        sub(&b, "^Common control header$", |t| append(t, "The hello header differs from the sync header in its first row `[decision]`."))
    }),
        expect: &[("AC5", "does not name the differing field 'magic'")],
    },
    Denial {
        name: "ac5_denial_no_layout_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| rename_heading(t, "^Hello layout$", "Hello fields"))),
        expect: &[("AC5", "no '### Hello layout' subsection"), ("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_the_layout_subsection_has_no_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| Offset (bytes) |", "| Position |"))),
        expect: &[("AC5", "the hello layout subsection has no five-column field table"), ("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_the_layout_not_starting_with_the_header",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| Magic 0x4d4e", "| Marker 0x4d4e"))),
        expect: &[("AC5", "does not start with the common control header rows"), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_a_layout_header_row_of_another_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 7 | 1 | u8 | Authentication scheme", "| 7 | 1 | i8 | Authentication scheme"))),
        expect: &[("AC5", "differs from the common control header in offset, width or type"), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_layout_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 52 | 4 | u32 | Election epoch", "| 53 | 4 | u32 | Election epoch"))),
        expect: &[("AC5", "at byte 53, expected 52"), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_a_row_of_part_of_a_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 52 | 4 | u32 | Election epoch", "| 52 | 31 bits | u32 | Election epoch"))),
        expect: &[("AC5", "is not a whole number of bytes"), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_transport_role_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| Transport role: 0 = none", "| Role of transport: 0 = none"))),
        expect: &[("AC1", "no row for transport role"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_module_id_of_the_wrong_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | 4 | u64 | Module id"))),
        expect: &[("AC1", "module id (C-11) is 32 bits"), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_counted_name_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 76 | 1 × max 32 | u8 | Name:", "| 76 | 32 | u8 | Name:"))),
        expect: &[("AC1", "no counted name row"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_name_without_its_encoding",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| replace_in_line(&lit(t, "UTF-8 (RFC 3629), name length", "Unicode, name length"), r"^\| 76 \|", "valid UTF-8", "valid Unicode"))),
        expect: &[("AC1", "does not state its encoding")],
    },
    Denial {
        name: "ac1_denial_capability_bits_not_32",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 48 | 4 | u32 | Capability bits", "| 48 | 2 | u32 | Capability bits"))),
        expect: &[("AC1", "capability bits are 16 bits"), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_holder_eligible_bit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "holder-eligible is bit 0", "holder-eligible is the low bit"))),
        expect: &[("AC1", "does not fix the holder-eligible bit")],
    },
    Denial {
        name: "ac1_denial_no_counted_port_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 108 | 7 × max 51 |", "| 108 | 7 |"))),
        expect: &[("AC1", "no counted port row"), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_the_counter_not_the_liveness_counter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "(?i)liveness counter", "liveness tally"))),
        expect: &[("AC1", "is not stated as the data-plane liveness counter")],
    },
    Denial {
        name: "ac1_denial_no_sentence_on_the_liveness_counter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "liveness counter", "liveness tally"))),
        expect: &[("AC1", "no sentence says that the message counter doubles")],
    },
    Denial {
        name: "ac1_denial_no_bundle_element_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| rename_heading(t, "^Bundle element$", "Bundle entry"))),
        expect: &[("AC1", "no '### Bundle element' subsection"), ("AC2", "")],
    },
    Denial {
        name: "ac1_denial_no_channel_map_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Bundle element$", |t| lit(t, "| Channel map:", "| Channel widths:"))),
        expect: &[("AC1", "has no row for 'channel map'")],
    },
    Denial {
        name: "ac2_denial_bundle_element_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Bundle element$", |t| lit(t, "| 34 | 2 × max 32 |", "| 35 | 2 × max 32 |"))),
        expect: &[("AC2", "at byte 35, expected 34")],
    },
    Denial {
        name: "ac2_denial_a_member_row_above_c08",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Bundle element$", |t| lit(t, "| 2 | 1 × max 32 |", "| 2 | 1 × max 33 |"))),
        expect: &[("AC2", "allows 33 members, above C-08's 32")],
    },
    Denial {
        name: "ac2_denial_a_member_row_not_at_c08",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Bundle element$", |t| lit(t, "| 2 | 1 × max 32 |", "| 2 | 1 × max 31 |"))),
        expect: &[("AC2", "has max 31; C-08 fixes 32")],
    },
    Denial {
        name: "ac2_denial_no_member_row_at_c08",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Bundle element$", |t| lit(&lit(t, "| 2 | 1 × max 32 |", "| 2 | 1 × max 31 |"), "| 34 | 2 × max 32 |", "| 34 | 2 × max 31 |"))),
        expect: &[("AC2", "has no counted member row with the maximum of 32")],
    },
    Denial {
        name: "ac1_denial_no_port_element_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| rename_heading(t, "^Port element$", "Port entry"))),
        expect: &[("AC1", "no '### Port element' subsection")],
    },
    Denial {
        name: "ac1_denial_no_counted_content_type_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Port element$", |t| lit(t, "| 3 | 1 × max 4 |", "| 3 | 1 |"))),
        expect: &[("AC1", "no counted content-type row")],
    },
    Denial {
        name: "ac1_denial_a_port_row_unequal_to_its_element",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Port element$", |t| lit(t, "| 2 | 1 | u8 | Content-type count", "| 2 | 2 | u8 | Content-type count"))),
        expect: &[("AC1", "its element sums to 64 bits")],
    },
    Denial {
        name: "ac2_denial_the_bundle_row_not_counted",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 465 | 98 × max 8 |", "| 465 | 98 |"))),
        expect: &[("AC2", "is not a counted row"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_the_bundle_row_unequal_to_its_element",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 465 | 98 × max 8 |", "| 465 | 97 × max 8 |"))),
        expect: &[("AC2", "the bundle row is 776 bits per bundle"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_no_one_byte_bundle_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 75 | 1 | u8 | Bundle count", "| 75 | 2 | u8 | Bundle count"))),
        expect: &[("AC2", "no one-byte bundle count row"), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_no_stated_bundles_per_hello",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, r"at most (\d+) bundles", "no more than so many bundles"))),
        expect: &[("AC2", "states no bundles-per-hello maximum")],
    },
    Denial {
        name: "ac2_denial_a_stated_maximum_unequal_to_the_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "A hello is at most 8 bundles", "A hello is at most 9 bundles"))),
        expect: &[("AC2", "the stated bundles-per-hello maximum is 9")],
    },
    Denial {
        name: "ac2_denial_no_rule_for_more_bundles",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| {
        let t = delete_unit(t, "^The rule for a source with more than 8 bundles");
        let t = lit(&lit(&t, "**Open item 2 ", "**Note 2 "), "**Open item 3 ", "**Note 3 ");
        replace_all(&t, r"\brule\b", "law")
    })),
        expect: &[("AC2", "no rule or 'Open item'")],
    },
    Denial {
        name: "ac2_denial_too_few_bundles_for_c09",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| {
        let t = lit(t, "| 465 | 98 × max 8 |", "| 465 | 98 × max 7 |");
        let t = lit(&t, "A hello is at most 8 bundles", "A hello is at most 7 bundles");
        replace_all(&t, "256", "many")
    })),
        expect: &[("AC2", "cannot carry C-09's 256 streams"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_no_one_bundle_figure",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "= 563 bytes `[decision]`.", "= 563 `[decision]`."))),
        expect: &[("AC2", "the one-bundle figure is not stated")],
    },
    Denial {
        name: "ac2_denial_a_one_bundle_figure_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "= 563 bytes `[decision]`.", "= 564 bytes `[decision]`."))),
        expect: &[("AC2", "stated as 564 bytes; the tables compute 563")],
    },
    Denial {
        name: "ac2_denial_no_maximum_hello_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "(?i)maximum hello size", "largest hello"))),
        expect: &[("AC2", "the maximum hello size is not stated")],
    },
    Denial {
        name: "ac2_denial_no_verdict_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, VERDICT, "**Outcome: the hello fits.**"))),
        expect: &[("AC2", "no 'Verdict' paragraph")],
    },
    Denial {
        name: "ac2_denial_a_verdict_stating_neither",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| verdict(t, "**Verdict: see above.** Nothing more `[decision]`."))),
        expect: &[("AC2", "states neither 'fits' nor 'falsified'")],
    },
    Denial {
        name: "ac2_denial_a_verdict_stating_both",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, VERDICT, "**Verdict: the hello fits.** It is falsified."))),
        expect: &[("AC2", "says both 'fits' and 'falsified'")],
    },
    Denial {
        name: "ac2_denial_a_verdict_saying_c17_falsifier_holds",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "C-17 holds, and its falsifier", "C-17 holds for the hello, and its falsifier"))),
        expect: &[("AC2", "says C-17's falsifier holds")],
    },
    Denial {
        name: "ac2_denial_fits_without_bundle_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", no_bundle_rows)),
        expect: &[("AC2", "fits but the hello layout carries no bundle rows"), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_fits_above_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(&lit(t, "| 465 | 98 × max 8 |", "| 465 | 98 × max 9 |"), "A hello is at most 8 bundles", "A hello is at most 9 bundles"))),
        expect: &[("AC2", "is 1347 bytes, above 1252"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_fits_with_a_stated_maximum_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "The maximum hello size is 1 249 bytes", "The maximum hello size is 1 248 bytes"))),
        expect: &[("AC2", "stated as 1248 bytes; the table sums to 1249")],
    },
    Denial {
        name: "ac2_denial_falsified_with_bundle_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| verdict(t, FALSIFIED))),
        expect: &[("AC2", "still carries the bundle rows")],
    },
    Denial {
        name: "ac2_denial_falsified_without_an_all_bundles_maximum",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| {
        let t = within_section(&verdict(t, FALSIFIED), "^Hello layout$", no_bundle_rows);
        replace_all(&t, r"at most (\d+) bundles", "no more than so many bundles")
    })),
        expect: &[("AC2", "the all-bundles maximum cannot be computed"), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_falsified_with_an_all_bundles_maximum_within_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| {
        let t = within_section(&verdict(t, FALSIFIED), "^Hello layout$", no_bundle_rows);
        lit(&t, "A hello is at most 8 bundles", "A hello is at most 1 bundles")
    })),
        expect: &[("AC2", "not above 1252"), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_falsified_with_a_fallback_above_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(&verdict(t, FALSIFIED), "| 465 | 98 × max 8 |", "| 465 | 98 × max 9 |"))),
        expect: &[("AC2", "the fallback hello is 1347 bytes"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_falsified_with_a_stated_maximum_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(&verdict(t, FALSIFIED), "The maximum hello size is 1 249 bytes", "The maximum hello size is 1 248 bytes"))),
        expect: &[("AC2", "the fallback table sums to 1249")],
    },
    Denial {
        name: "ac2_denial_falsified_without_the_all_bundles_size_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| {
        let t = within_section(&verdict(t, FALSIFIED), "^Hello layout$", no_bundle_rows);
        lit(&t, "A hello is at most 8 bundles", "A hello is at most 9 bundles")
    })),
        expect: &[("AC2", "the all-bundles size 1347 bytes is not stated"), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_falsified_citing_c17_against_its_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| verdict(t, FALSIFIED))),
        expect: &[("AC2", "says C-17 is falsified")],
    },
    Denial {
        name: "ac3_denial_falsified_without_a_fallback",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| {
        let t = verdict(t, "**Verdict: the hello is falsified.** Nothing more `[decision]`.");
        replace_all(&replace_all(&t, "(?i)split", "divid"), "(?i)handshake", "exchange")
    })),
        expect: &[("AC3", "does not name the fallback"), ("AC2", "")],
    },
    Denial {
        name: "ac3_denial_no_alarm_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| rename_heading(t, "^Alarm field$", "Warnings"))),
        expect: &[("AC3", "no '### Alarm field' subsection")],
    },
    Denial {
        name: "ac3_denial_an_alarm_table_of_five_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| delete_row(t, r"^\| Dead \|"))),
        expect: &[("AC3", "has no table with one row per alarm")],
    },
    Denial {
        name: "ac3_denial_an_alarm_without_a_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Defunct | bit 2 |"))),
        expect: &[("AC3", "no row for the alarm 'dead'")],
    },
    Denial {
        name: "ac3_denial_an_alarm_without_a_bit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | two |"))),
        expect: &[("AC3", "the alarm 'dead' has no bit or value")],
    },
    Denial {
        name: "ac3_denial_two_alarms_on_one_bit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 1 |"))),
        expect: &[("AC3", "are not distinct")],
    },
    Denial {
        name: "ac3_denial_an_alarm_of_value_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | value 0 |"))),
        expect: &[("AC3", "encoded as zero")],
    },
    Denial {
        name: "ac3_denial_an_alarm_beyond_the_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 9 |"))),
        expect: &[("AC3", "does not fit the alarm field's width")],
    },
    Denial {
        name: "ac3_denial_no_transport_role_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| rename_heading(t, "^Transport role$", "Roles"))),
        expect: &[("AC3", "no '### Transport role' subsection")],
    },
    Denial {
        name: "ac3_denial_a_role_table_of_one_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Transport role$", |t| delete_row(&delete_row(t, r"^\| 1 \| follower"), r"^\| 2 \| reference"))),
        expect: &[("AC3", "has no table with its values")],
    },
    Denial {
        name: "ac3_denial_the_roles_not_a_proposal",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Transport role$", |t| replace_all(t, "(?i)proposal", "draft"))),
        expect: &[("AC3", "not recorded as a proposal")],
    },
    Denial {
        name: "ac3_denial_no_role_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "**Open item 1 — the transport role values.**", "**Item 1 — the transport role values.**"))),
        expect: &[("AC3", "states the transport role open question")],
    },
    Denial {
        name: "ac3_denial_a_role_open_item_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Open items$", |t| replace_in_line(t, r"^\*\*Open item 1 ", "(?i)owner", "Confirmed by"))),
        expect: &[("AC3", "does not name epic 005 as its confirming owner"), ("DoD", "")],
    },
    Denial {
        name: "ac3_denial_a_role_open_item_without_its_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Open items$", |t| lit(t, "epic 005 needs a role the encoding cannot express", "epic 005 needs another role"))),
        expect: &[("AC3", "lacks its falsifier")],
    },
    Denial {
        name: "ac4_denial_no_invalid_form_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |", "| Case | Result |"))),
        expect: &[("AC4", "no invalid-form table")],
    },
    Denial {
        name: "ac4_denial_fewer_than_ten_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "\n| foreign domain:", "\n\n| foreign domain:"))),
        expect: &[("AC4", "at least 10 are required")],
    },
    Denial {
        name: "ac4_denial_an_invented_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "WrongMagic"))),
        expect: &[("AC4", "names 'WrongMagic', which is not in the closed enumeration")],
    },
    Denial {
        name: "ac4_denial_an_exact_reason_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadLength"))),
        expect: &[("AC4", "must be counted under BadMagic")],
    },
    Denial {
        name: "ac4_denial_domain_zero_mapped_to_foreign_domain",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "UnconfiguredDomain — never ForeignDomain", "UnconfiguredDomain or ForeignDomain"))),
        expect: &[("AC4", "domain 0 is mapped to ForeignDomain")],
    },
    Denial {
        name: "ac4_denial_a_version_mismatch_without_its_event",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| version mismatch", r"\| drop and count under VersionMismatch.*$", "| drop and count under VersionMismatch |"))),
        expect: &[("AC4", "raises a named event")],
    },
    Denial {
        name: "ac4_denial_a_form_naming_no_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| message type 0", r"\| drop and count under BadLength.*$", "| drop it silently |"))),
        expect: &[("AC4", "names no member of the closed enumeration and argues no non-rejection")],
    },
    Denial {
        name: "ac4_denial_a_collision_row_without_its_argument",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| module-id collision", "not a parse rejection", "dropped silently"))),
        expect: &[("AC4", "needs a member reason or an argued non-rejection")],
    },
    Denial {
        name: "ac4_denial_no_oversize_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| oversize"))),
        expect: &[("AC4", "covers the 'oversize' case")],
    },
    Denial {
        name: "ac4_denial_no_counter_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| liveness counter going backwards"))),
        expect: &[("AC4", "liveness counter going backwards")],
    },
    Denial {
        name: "ac4_denial_no_name_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| name bytes not valid"))),
        expect: &[("AC4", "a name that is not valid in the chosen encoding")],
    },
    Denial {
        name: "ac4_denial_no_collision_row_nor_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_unit(&delete_row(t, r"^\| module-id collision"), "^A module-id collision is not a parse rejection"))),
        expect: &[("AC4", "is not stated as a non-rejection")],
    },
    Denial {
        name: "ac4_denial_no_collision_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_unit(t, "^A module-id collision is not a parse rejection"))),
        expect: &[("AC4", "no sentence says that a module-id collision")],
    },
    Denial {
        name: "work3_denial_no_stale_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, r"(?i)\bstale\b", "outdated"))),
        expect: &[("Work3", "treats its copy as stale")],
    },
    Denial {
        name: "dod_denial_the_regime_stamp_fields_not_documented",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, r"(?i)version\s+number", "version count"))),
        expect: &[("DoD", "regime stamp fields the hello touches"), ("AC6", ""), ("Work3", "")],
    },
    Denial {
        name: "work3_denial_c18_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "C-18", "C-81"))),
        expect: &[("Work3", "travels in its own message")],
    },
    Denial {
        name: "behaviour_denial_ttl_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "TTL 1", "TTL one"))),
        expect: &[("Behaviour", "multicast with TTL 1")],
    },
    Denial {
        name: "behaviour_denial_the_hello_period_of_c30",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-30 \|", r"\| 1 s \|", "| 7 s |"))),
        expect: &[("Behaviour", "the hello period is not restated")],
    },
    Denial {
        name: "behaviour_denial_the_cs5_value_of_c13",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-13 \|", r"control CS5 \(40\)", "control CS5 (41)"))),
        expect: &[("Behaviour", "CS5 is not stated with the value of C-13"), ("AC6", "")],
    },
    Denial {
        name: "behaviour_denial_the_peer_bound_of_c34",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-34 \|", "at most 64", "at most 97"))),
        expect: &[("Behaviour", "the peer map bound is not restated")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| strip_tags(t, "^The hello carries no IPv4 address"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| append(t, "**Open item 10 — x.** Still open `[decision]`."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    Denial {
        name: "dod_denial_a_table_in_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Transport role$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""))),
        expect: &[("DoD", "a table of 'Transport role' follows no evidence-tagged paragraph")],
    },
    Denial {
        name: "ac6_denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC6", "does not exist")],
    },
    Denial {
        name: "ac6_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| format!("{t}\n[[[broken"))),
        expect: &[("AC6", "does not parse")],
    },
    Denial {
        name: "ac6_denial_a_hello_named_message_not_control",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| after(t, "name = \"hello port element\"\n", "layer = \"control\"", "layer = \"media\""))),
        expect: &[("AC6", "toml message 'hello port element' is a hello message")],
    },
    Denial {
        name: "ac6_denial_no_hello_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"hello\"\n", "name = \"hullo\"\n"))),
        expect: &[("AC6", "declares no [[message]] named 'hello'")],
    },
    Denial {
        name: "ac6_denial_the_hello_not_control",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "layer = \"control\"", "layer = \"media\"")),
        expect: &[("AC6", "the toml hello has layer 'media'")],
    },
    Denial {
        name: "ac6_denial_the_hello_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "dscp = 40", "dscp = 46")),
        expect: &[("AC6", "the toml hello has dscp 46")],
    },
    Denial {
        name: "ac6_denial_the_hello_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 1250")),
        expect: &[("AC6", "the toml hello has max_size 1250")],
    },
    Denial {
        name: "ac6_denial_the_hello_above_the_bound",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 1300")),
        expect: &[("AC6", "exceeds the 1252-byte control bound")],
    },
    Denial {
        name: "ac6_denial_a_field_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "[[message.field]]\nname = \"message_type\"\n", "[[message.gone]]\nname = \"message_type\"\n")),
        expect: &[("AC6", "they must correspond one to one")],
    },
    Denial {
        name: "ac6_denial_a_field_misnamed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "name = \"domain_id\"", "name = \"domain\"")),
        expect: &[("AC6", "toml hello field 'domain' is not named after")],
    },
    Denial {
        name: "ac6_denial_a_field_offset",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 32\n", "offset = 33\n")),
        expect: &[("AC6", "offset 33 bits")],
    },
    Denial {
        name: "ac6_denial_a_counted_width",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 608\nwidth = 8", "offset = 608\nwidth = 16")),
        expect: &[("AC6", "toml hello field name: width 16 bits per element")],
    },
    Denial {
        name: "ac6_denial_a_max_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_count = 32", "max_count = 31")),
        expect: &[("AC6", "toml hello field name: max_count 31")],
    },
    Denial {
        name: "ac6_denial_a_count_naming_no_earlier_field",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "count = \"name_length\"", "count = \"nonexistent\"")),
        expect: &[("AC6", "its count must name an earlier field")],
    },
    Denial {
        name: "ac6_denial_a_width",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 32\nwidth = 16", "offset = 32\nwidth = 8")),
        expect: &[("AC6", "toml hello field domain_id: width 8 bits")],
    },
    Denial {
        name: "ac6_denial_a_type",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 32\nwidth = 16\ntype = \"u16\"", "offset = 32\nwidth = 16\ntype = \"i16\"")),
        expect: &[("AC6", "toml hello field domain_id: type i16")],
    },
    Denial {
        name: "ac6_denial_fields_not_summing_to_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_count = 8\n", "max_count = 9\n")),
        expect: &[("AC6", "the toml hello fields compute")],
    },
    Denial {
        name: "ac6_denial_the_port_field_without_its_nested_maximum",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| {
        let t = after(t, "name = \"port\"\noffset = 864", "at most 4 content-type codes", "at most four content-type codes");
        after(&t, "name = \"port\"\noffset = 864", "1 to 4 content types", "1 to four content types")
    })),
        expect: &[("AC6", "does not state the maximum 4 of its nested count")],
    },
    Denial {
        name: "ac6_denial_the_bundle_field_without_32",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| after(t, "name = \"bundle\"\noffset = 3720", "1 to 32 members", "1 to many members"))),
        expect: &[("AC6", "does not state the 32-member maximum")],
    },
    Denial {
        name: "ac6_denial_a_named_reason_not_listed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "\"BadMagic\", ", "")),
        expect: &[("AC6", "does not list 'BadMagic'")],
    },
    Denial {
        name: "ac6_denial_a_listed_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [", "rejection_reasons = [\"Bogus\", ")),
        expect: &[("AC6", "names 'Bogus', which is not in the closed enumeration")],
    },
    // `\w`: `²` is a word character for Python, so `\bBadMagic\b` does not
    // match `BadMagic²`.
    Denial {
        name: "py_denial_a_reason_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadMagic\u{b2}"))),
        expect: &[("AC4", "must be counted under BadMagic")],
    },
    // `\s`: U+001F is whitespace for Python, so `clean_start` strips it and
    // the paragraph is an open item.
    Denial {
        name: "py_denial_an_open_item_after_a_unit_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| append(t, "\u{1f}**Open item 10 — x.** Still open `[decision]`."))),
        expect: &[("REPERTOIRE", "")],
    },
    // `str.splitlines()`: U+2028 ends a line, so the validity table ends in
    // its sixth row.
    Denial {
        name: "py_denial_a_line_separator_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| foreign domain", r"\| drop and count under ForeignDomain", "|\u{2028}drop and count under ForeignDomain"))),
        expect: &[("REPERTOIRE", "")],
    },
    // `str.splitlines()`: so does a form feed.
    Denial {
        name: "py_denial_a_form_feed_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| foreign domain", r"\| drop and count under ForeignDomain", "|\u{c}drop and count under ForeignDomain"))),
        expect: &[("REPERTOIRE", "")],
    },
    // `int()` is unbounded: a 21-digit offset is read, and does not tile.
    Denial {
        name: "py_denial_an_offset_beyond_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 100000000000000000032 | 8 | u64 | Module id"))),
        expect: &[("AC5", "at byte 100000000000000000032, expected 32"), ("AC6", "")],
    },
    // `\d` and `int()` read Arabic-Indic digits: ٣٣ is 33, out of place.
    Denial {
        name: "py_denial_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| \u{663}\u{663} | 8 | u64 | Module id"))),
        expect: &[("REPERTOIRE", "")],
    },
    // The subset reader keeps `"1249"` a str: unequal to 1249.
    Denial {
        name: "py_denial_max_size_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = \"1249\"")),
        expect: &[("AC6", "the toml hello has max_size 1249; the markdown layout sums to 1249")],
    },
    // The subset reader iterates a str's characters: no reason is listed.
    Denial {
        name: "py_denial_rejection_reasons_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]", "rejection_reasons = \"BadMagic\"")),
        expect: &[("AC6", "does not list 'BadMagic'")],
    },
    // The subset reader cannot read `"""` strings.
    Denial {
        name: "py_denial_a_triple_quoted_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "name = \"domain_id\"", "name = \"\"\"domain_id\"\"\"")),
        expect: &[("AC6", "does not parse")],
    },
    // The subset reader lets a later duplicate key win.
    Denial {
        name: "py_denial_a_duplicate_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "dscp = 40\n", "dscp = 40\ndscp = 46\n")),
        expect: &[("AC6", "the toml hello has dscp 46")],
    },
    // A number of rejection reasons is not iterable: TypeError.
    Denial {
        name: "py_denial_rejection_reasons_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]", "rejection_reasons = 5")),
        expect: &[("CRASH", "rejection_reasons is not iterable")],
    },
    // A str width times a count is a str, which the sum rejects: TypeError.
    Denial {
        name: "py_denial_a_counted_width_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 608\nwidth = 8", "offset = 608\nwidth = \"8\"")),
        expect: &[("CRASH", "adds a non-number"), ("AC6", "")],
    },
    // `key_of(r[3])` on a sync header row of three cells: IndexError.
    Denial {
        name: "py_denial_a_sync_header_row_of_three_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Sync request and reply$", |s| lit(s, "| 7 | 1 | u8 | Authentication scheme (C-12); 0 = none in v1 (C-115) | 0 only |", "| 7 | 1 | u8 |")))),
        expect: &[("CRASH", "IndexError")],
    },
    // The 004 separator line: a cell that is not dashes and colons is no
    // separator, and the table is dropped whole.
    Denial {
        name: "py_denial_a_separator_with_a_letter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|---|-x-|"))),
        expect: &[("AC4", "no invalid-form table")],
    },
    // A run of rows without a separator line is not a table.
    Denial {
        name: "py_denial_a_headerless_alarm_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Alarm | Bit | Set while (C-173) |\n|---|---|---|\n", ""))),
        expect: &[("AC3", "has no table with one row per alarm")],
    },
    // `re.I`: Python matches `i` with `ı`, so this is an open item.
    Denial {
        name: "py_denial_an_open_item_with_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| append(t, "**Open \u{131}tem 10 — x.** Still open `[decision]`."))),
        expect: &[("REPERTOIRE", "")],
    },
    // `(?<!not )`: "not fits" is not a verdict word.
    Denial {
        name: "py_denial_a_verdict_of_not_fits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| verdict(t, "**Verdict: the hello not fits here.** Nothing more `[decision]`."))),
        expect: &[("AC2", "states neither 'fits' nor 'falsified'")],
    },
    // `\d`/`int()`: ٣٢ is 32.
    Denial {
        name: "py_accepts_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| \u{663}\u{662} | 8 | u64 | Module id"))),
        expect: &[("REPERTOIRE", "")],
    },
    // A combining accent is not a word character for Python: `\bdead\b`
    // still finds "Dead\u{301}".
    Denial {
        name: "py_accepts_a_combining_accent_after_an_alarm",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead\u{301} | bit 2 |"))),
        expect: &[("REPERTOIRE", "")],
    },
    // `re.I`: "bınary" is "binary" to Python.
    Denial {
        name: "py_accepts_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| replace_all(t, "binary", "b\u{131}nary"))),
        expect: &[("REPERTOIRE", "")],
    },
    // The 24-character look-back counts characters, not bytes.
    Denial {
        name: "py_accepts_a_negation_within_24_characters_of_wide_ones",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "— never ForeignDomain", "— never ——————— ForeignDomain"))),
        expect: &[],
    },
    // The subset reader: 40.0 and 0x28 equal 40.
    Denial {
        name: "py_accepts_a_float_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "dscp = 40\n", "dscp = 40.0\n")),
        expect: &[],
    },
    Denial {
        name: "py_accepts_a_hex_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "dscp = 40\n", "dscp = 0x28\n")),
        expect: &[],
    },
    // The 004 separator line accepts colons inside a cell.
    Denial {
        name: "py_accepts_a_separator_with_inner_colons",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|-:-|---|"))),
        expect: &[],
    },
    // U+001F is whitespace for `str.strip()`.
    Denial {
        name: "py_accepts_a_unit_separator_in_a_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32\u{1f} | 8 | u64 | Module id"))),
        expect: &[("REPERTOIRE", "")],
    },
    // `T.get("max_size") != H_BYTES` and `> bound` on 10^19: Python compares exactly.
    Denial {
        name: "num_denial_max_size_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 10000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 10^36 is an int to Python (no clamp); the comparisons fail as for any other size.
    Denial {
        name: "num_denial_max_size_of_10_36",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 1000000000000000000000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 10^40 is beyond i128: the port is red by rule (INT_LIMIT); Python fails AC6.
    Denial {
        name: "num_denial_max_size_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // 2^127, one above i128::MAX: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_just_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 170141183460469231731687303715884105728")),
        expect: &[("INT_LIMIT", "")],
    },
    // The subset reader reads `1_249` as Python's int("1_249") would, or not at all.
    Denial {
        name: "num_accepts_max_size_with_an_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 1_249")),
        expect: &[],
    },
    // A leading `+`.
    Denial {
        name: "num_denial_max_size_with_a_plus",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = +1249")),
        expect: &[],
    },
    // Leading zeros: the subset reader accepts `040`-style ints.
    Denial {
        name: "num_denial_max_size_with_leading_zeros",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 01249")),
        expect: &[],
    },
    // A non-decimal form (0x4e1 = 1249).
    Denial {
        name: "num_denial_max_size_in_hex",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 0x4e1")),
        expect: &[],
    },
    // 1249.0 == 1249 in Python, but a float is not `isinstance(_, int)`.
    Denial {
        name: "num_denial_max_size_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 1249.0")),
        expect: &[],
    },
    // True is an int to `isinstance` and equals 1.
    Denial {
        name: "num_denial_max_size_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = true")),
        expect: &[("AC6", "")],
    },
    // An empty string.
    Denial {
        name: "num_denial_max_size_empty",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = \"\"")),
        expect: &[("AC6", "")],
    },
    // No max_size: `T.get("max_size")` is None.
    Denial {
        name: "num_denial_max_size_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249\n", "")),
        expect: &[("AC6", "")],
    },
    // A counted width beyond i128: `width * max_count` is INT_LIMIT.
    Denial {
        name: "num_denial_a_width_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 608\nwidth = 8", "offset = 608\nwidth = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // 10^20 bits × 51 fits i128; the comparisons fail.
    Denial {
        name: "num_denial_a_width_times_count_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 608\nwidth = 8", "offset = 608\nwidth = 100000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 16.0 == 16; the sum becomes a float and still equals max_size × 8.
    Denial {
        name: "num_denial_a_width_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 16.0")),
        expect: &[],
    },
    // A float of 10^19: != 16, and the float sum != max_size × 8.
    Denial {
        name: "num_denial_a_width_as_a_big_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 1e19")),
        expect: &[("AC6", "")],
    },
    // `inf` (the subset reader's float), summed.
    Denial {
        name: "num_denial_a_width_as_inf",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = inf")),
        expect: &[("AC6", "")],
    },
    // True + int is an int.
    Denial {
        name: "num_denial_a_width_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = true")),
        expect: &[("AC6", "")],
    },
    // `total_bits += [16]`: TypeError.
    Denial {
        name: "num_denial_a_width_as_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = [16]")),
        expect: &[("AC6", ""), ("CRASH", "")],
    },
    // `tfl.get("width", 0)`: the default 0.
    Denial {
        name: "num_denial_a_width_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "offset = 0\nwidth = 16\n", "offset = 0\n")),
        expect: &[("AC6", "")],
    },
    // `field = 5` before the hello's field tables (the reader decides).
    Denial {
        name: "num_denial_field_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249\n", "max_size = 1249\nfield = 5\n")),
        expect: &[("AC6", "")],
    },
    // A duplicate reason: `set()` drops it.
    Denial {
        name: "num_denial_duplicate_rejection_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [\"TooShort\", \"TooShort\", ")),
        expect: &[],
    },
    // `sorted(have - set(MEMBERS))` of a str and an int: TypeError.
    Denial {
        name: "num_denial_rejection_reasons_mixed_types",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [\"Bogus\", 7, \"TooShort\", ")),
        expect: &[("CRASH", "")],
    },
    // `set()` of a list item: unhashable, TypeError.
    Denial {
        name: "num_denial_rejection_reasons_with_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [[1], \"TooShort\", ")),
        expect: &[("CRASH", "")],
    },
    // Two ints and the members: the ints sort among themselves.
    Denial {
        name: "num_denial_rejection_reasons_only_numbers",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [3, 1, \"TooShort\", ")),
        expect: &[("AC6", "")],
    },
    // C-17 in digit groups to 10^19.
    Denial {
        name: "num_denial_the_bound_of_10_19",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 10 000 000 000 000 000 000 bytes"))),
        expect: &[],
    },
    // C-17 at 10^40: INT_LIMIT.
    Denial {
        name: "num_denial_the_bound_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 10 000 000 000 000 000 000 000 000 000 000 000 000 000 bytes"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `²` is not `\d`: NUM stops before it, and `\s*bytes` then fails.
    Denial {
        name: "num_denial_the_bound_glued_to_a_superscript",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 1 252² bytes"))),
        expect: &[("AC2", ""), ("DoD", "")],
    },
    // int("040") is 40; the Behaviour regex then looks for `(40)`.
    Denial {
        name: "num_denial_the_cs5_value_with_leading_zeros",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "control CS5 (40)", "control CS5 (040)"))),
        expect: &[],
    },
    // C-30 beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_hello_period_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-30 \|", r"\| 1 s \|", "| 10000000000000000000000000000000000000000 s |"))),
        expect: &[("INT_LIMIT", "")],
    },
    // 1 << 126 fits i128; its bit_length 127 exceeds the field.
    Denial {
        name: "num_denial_an_alarm_on_bit_126",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 126 |"))),
        expect: &[("AC3", "")],
    },
    // 1 << 127 is beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_an_alarm_on_bit_127",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 127 |"))),
        expect: &[("INT_LIMIT", "")],
    },
    // 1 << 10^19: Python raises (MemoryError or OverflowError); the port is INT_LIMIT.
    Denial {
        name: "num_denial_an_alarm_on_bit_10_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 10000000000000000000 |"))),
        expect: &[("INT_LIMIT", "")],
    },
    // int(hex, 16) of 2^128 - 1: INT_LIMIT.
    Denial {
        name: "num_denial_an_alarm_of_a_long_hex_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | 0xffffffffffffffffffffffffffffffff |"))),
        expect: &[("INT_LIMIT", "")],
    },
    // 0x10 = 16 fits the field? (bit_length 5)
    Denial {
        name: "num_denial_an_alarm_of_a_hex_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | 0x10 |"))),
        expect: &[("AC3", "")],
    },
    // `bit\s+(\d+)` stops before `²`: still bit 2.
    Denial {
        name: "num_accepts_an_alarm_bit_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 2² |"))),
        expect: &[],
    },
    // A duplicate row: the first match wins.
    Denial {
        name: "num_denial_a_duplicate_alarm_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| lit(t, "| Dead | bit 2 |", "| Dead | bit 2 |\n| Dead | bit 1 |"))),
        expect: &[],
    },
    // A row without its second cell: `rest` is "".
    Denial {
        name: "num_denial_an_alarm_row_of_one_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Alarm field$", |t| replace_in_line(t, r"^\| Dead \| bit 2 \|", r"\| Dead \| bit 2 \|.*$", "| Dead |"))),
        expect: &[("AC3", "")],
    },
    // int("032") is 32.
    Denial {
        name: "num_accepts_an_offset_with_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 032 | 8 | u64 | Module id"))),
        expect: &[],
    },
    // `\d+` refuses `+32`.
    Denial {
        name: "num_denial_an_offset_with_a_plus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| +32 | 8 | u64 | Module id"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // An empty Offset cell.
    Denial {
        name: "num_denial_an_empty_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "|  | 8 | u64 | Module id"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // An offset beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_an_offset_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 10000000000000000000000000000000000000000 | 8 | u64 | Module id"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `8²` is neither form of width_of.
    Denial {
        name: "num_denial_a_width_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | 8² | u64 | Module id"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // 10^36 bytes = 8 × 10^36 bits, within i128.
    Denial {
        name: "num_denial_a_width_of_10_36_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | 1000000000000000000000000000000000000 | u64 | Module id"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // 10^38 bytes × 8 is beyond i128: INT_LIMIT (the library's `width_of` saturates the product; the port's own size sum then overflows and records it).
    Denial {
        name: "num_denial_a_width_of_10_38_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | 100000000000000000000000000000000000000 | u64 | Module id"))),
        expect: &[("INT_LIMIT", "")],
    },
    // A bundle maximum of 10^19.
    Denial {
        name: "num_denial_a_counted_maximum_of_10_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 465 | 98 × max 8", "| 465 | 98 × max 10000000000000000000"))),
        expect: &[("AC2", ""), ("AC6", "")],
    },
    // A duplicate row: the layout no longer tiles.
    Denial {
        name: "num_denial_a_duplicate_layout_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |", "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |\n| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |"))),
        expect: &[("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // The maximum size in digit groups (NUM, to_int).
    Denial {
        name: "num_denial_a_stated_maximum_hello_size_in_groups",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "maximum hello size is 1 249 bytes", "maximum hello size is 1 249 000 bytes"))),
        expect: &[("AC2", "")],
    },
    // 2^63, beyond i64 and within i128.
    Denial {
        name: "num_denial_an_offset_of_2_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 9223372036854775808 | 8 | u64 | Module id"))),
        expect: &[("AC5", ""), ("AC6", "")],
    },
    // 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_an_offset_of_45_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 100000000000000000000000000000000000000000000 | 8 | u64 | Module id"))),
        expect: &[("INT_LIMIT", "")],
    },
    // 0: an overlap.
    Denial {
        name: "num_denial_an_offset_of_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 0 | 8 | u64 | Module id"))),
        expect: &[("AC5", ""), ("AC6", "")],
    },
    // `\d+` refuses 3_2.
    Denial {
        name: "num_denial_an_offset_with_an_underscore",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 3_2 | 8 | u64 | Module id"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // `µ` next to the digits.
    Denial {
        name: "num_denial_an_offset_after_a_micro_sign",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Hello layout$", |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32µ | 8 | u64 | Module id"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", ""), ("AC6", "")],
    },
    // C-17 at 2^63.
    Denial {
        name: "num_denial_the_bound_of_2_63",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 9223372036854775808 bytes"))),
        expect: &[],
    },
    // C-17 at 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_the_bound_of_45_digits",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 100000000000000000000000000000000000000000000 bytes"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `µ` before bytes.
    Denial {
        name: "num_denial_the_bound_after_a_micro_sign",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 1 252µ bytes"))),
        expect: &[("AC2", ""), ("DoD", "")],
    },
    // 2^63 in TOML.
    Denial {
        name: "num_denial_max_size_of_2_63",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 9223372036854775808")),
        expect: &[("AC6", "")],
    },
    // 45 digits in TOML: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_of_45_digits",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| hello_toml(b, "max_size = 1249", "max_size = 100000000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
];
