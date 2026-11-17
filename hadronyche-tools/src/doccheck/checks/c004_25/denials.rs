//! The denial list of 004-25: the real documents with one edit, and what
//! the port and the old script must report on them (their failed criteria,
//! exactly; `CRASH` where the script stops with a Python exception).
//! Consumed by `tests/doc/c004_25.rs` (one test each) and by `doccheck-equiv
//! 004-25`, which runs each against the old script too. An entry with no
//! expectation is one both must pass.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;

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
    let re = crate::doccheck::text::rx(line);
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

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist"), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC6", "does not exist")],
    },
    Denial {
        name: "ac1_denial_the_control_bound_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 300")),
        expect: &[("AC1", "gives the control bound C-17 as 1 252")],
    },
    Denial {
        name: "ac3_denial_c159_not_eleven",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-159 \|", ", dead", "")),
        expect: &[("AC3", "eleven C-159 counters, read")],
    },
    Denial {
        name: "ac2_denial_c129_not_four",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-129 \|", ", not requested\\.", ".")),
        expect: &[("AC2", "four C-129 capability states, read")],
    },
    Denial {
        name: "ac2_denial_c131_not_four",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-131 \|", r"interrupt \(stamped", "interrupt: stamped")),
        expect: &[("AC2", "four C-131 receive-timestamp sources")],
    },
    Denial {
        name: "ac1_denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, "^The regime stamp is a module's declaration", "^", "To be filled. "))),
        expect: &[("AC1", "placeholder replaced")],
    },
    Denial {
        name: "dod_denial_the_merged_header_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 52 \| 1 \| u8 \| Section index", r"^\| 52 \| 1 \|", "| 52 | 2 |"))),
        expect: &[("DoD", "header rows (bytes 0 to 57) are unchanged")],
    },
    Denial {
        name: "ac1_denial_the_section_count_not_64",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 63"))),
        expect: &[("AC1", "allows 1 to 64 sections"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_code_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 1 to 15 \|", r"\| 004-25 \|", "| 004-99 |"))),
        expect: &[("AC1", "give 004-25 a block")],
    },
    Denial {
        name: "dod_denial_a_rejection_value_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 9 \| BadPacketIndex \|", r"^\| 9", "| 13")),
        expect: &[("DoD", "Rejection reasons enumeration is unchanged")],
    },
    Denial {
        name: "dod_denial_the_reserved_codes_row_removed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Section type codes not assigned in v1")),
        expect: &[("DoD", "still records the section type codes row")],
    },
    Denial {
        name: "ac1_denial_two_subsections_name_one_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Injections section$", "Calibration and injections section")),
        expect: &[("AC1", "names the calibration section type (found 2)"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_a_type_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Commissioning window section$", "Window section")),
        expect: &[("AC1", "seven section types are laid out"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_a_malformed_type_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Calibration factor", r"^\| 0 \| 8 \|", "| 0 | eight |"))),
        expect: &[("AC1", "Calibration factor section: the Width cell"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_a_type_without_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window section$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"))),
        expect: &[("AC1", "commissioning section type has no field table"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_an_unused_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_in_line(t, "^The maximum section size, which is also its fixed size, is 58 \\+ 8", "$", "\n\n| Offset | Width | Type | Meaning | Valid range |\n|---|---|---|---|---|\n| 0 | 1 | u8 | Spare | 0 only |"))),
        expect: &[("AC1", "field table(s) that no list row of it names")],
    },
    Denial {
        name: "ac1_denial_an_offset_neither_0_nor_58",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Calibration factor", r"^\| 0 ", "| 4 "))),
        expect: &[("AC1", "starts at offset 4"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window section$", |t| replace_in_line(t, r"^\| 8 \| 8 \| u64 \| Expiry time", r"^\| 8 ", "| 9 "))),
        expect: &[("AC1", "table does not tile"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_an_entry_table_off_its_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_in_line(t, r"^\| 1 \| 8 × max 149", "8 × max", "9 × max"))),
        expect: &[("AC1", "entry table sums to"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_type_over_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_in_line(t, r"^\| 1 \| 8 × max 149", "max 149", "max 160"))),
        expect: &[("AC1", "no more than the control bound"), ("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_section_count_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_all(t, "⌈256 / 149⌉", "256 over 149"))),
        expect: &[("AC1", "has no statement of the form"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_a_wrong_section_count_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_all(t, "⌈256 / 149⌉ = 2", "⌈256 / 149⌉ = 3"))),
        expect: &[("AC1", "the multi section count is wrong")],
    },
    Denial {
        name: "ac1_denial_the_size_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window section$", |t| replace_all(t, "58 \\+ 16 = 74 bytes", "58 + 16 = 74"))),
        expect: &[("AC1", "states its maximum section size as 74 bytes")],
    },
    Denial {
        name: "ac1_denial_a_type_code_outside_the_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, "Section type code 3 ", "Section type code 30 "))),
        expect: &[("AC1", "calibration section type has a type code inside the block")],
    },
    Denial {
        name: "ac1_denial_two_types_share_a_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, "Section type code 3 ", "Section type code 2 "))),
        expect: &[("AC1", "codes are distinct")],
    },
    Denial {
        name: "ac1_denial_an_inexact_product",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_all(t, "149 × 8 = 1 192", "149 × 8 = 1 193"))),
        expect: &[("AC1", "every product and sum stated in the regime stamp section is exact")],
    },
    Denial {
        name: "ac1_denial_an_inexact_sum",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, "58 \\+ 8 = 66", "58 + 8 = 67"))),
        expect: &[("AC1", "every product and sum stated in the regime stamp section is exact")],
    },
    Denial {
        name: "ac1_denial_the_excess_not_reported",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_all(t, "Open item 1", "Open item one"))),
        expect: &[("AC1", "or the text reports the excess and names Open item 1"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_stream_in_the_multi_packet_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_all(t, "(?i)stream", "flow"))),
        expect: &[("AC1", "the multi-packet section carries the stream number")],
    },
    Denial {
        name: "ac1_denial_no_expiry_time",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window section$", |t| replace_all(t, "(?i)expir", "end"))),
        expect: &[("AC1", "carries the expiry time")],
    },
    Denial {
        name: "ac1_denial_no_time_per_operation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Calibration factor", "abstract operation on the sender's target, in femtoseconds per operation", "abstract op on the sender's target, in femtoseconds per op"))),
        expect: &[("AC1", "carries the calibration factor, time per counted operation")],
    },
    Denial {
        name: "ac1_denial_no_epoch_in_the_injections_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Injections section$", |t| replace_all(t, "(?i)epoch", "era"))),
        expect: &[("AC1", "a stream forced onto a timeline epoch")],
    },
    Denial {
        name: "ac4_denial_no_panic_hook",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Injections section$", |t| replace_all(t, "(?i)panic", "abort"))),
        expect: &[("AC4", "the panic hook")],
    },
    Denial {
        name: "ac4_denial_no_control_isolation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Capability states section$", |t| replace_all(t, "(?i)isolat", "separat"))),
        expect: &[("AC4", "control-isolation capability")],
    },
    Denial {
        name: "ac1_denial_the_timestamp_source_without_a_state",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Receive-timestamp source section$", |t| replace_all(t, r"(?i)capabilit\w*\s+state|C-129", "x"))),
        expect: &[("AC1", "reports its source with a C-129 capability state")],
    },
    Denial {
        name: "ac2_denial_a_fifth_capability_state",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Capability states section$", |t| replace_in_line(t, r"^\| 4 \| not requested \|", "$", "\n| 5 | revoked | withdrawn |"))),
        expect: &[("AC2", "exactly the four C-129 states")],
    },
    Denial {
        name: "ac2_denial_a_timestamp_source_renamed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Receive-timestamp source section$", |t| replace_in_line(t, r"^\| 4 \| interrupt \|", "interrupt", "irq"))),
        expect: &[("AC2", "exactly the four C-131 sources")],
    },
    Denial {
        name: "ac3_denial_a_counter_without_its_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_in_line(t, r"^\| 39 \| 4 \| u32 \| Too-early", "Too-early: packets that arrived too early", "Premature: packets that arrived prematurely"))),
        expect: &[("AC3", "has its own row of the health section (matched 10)")],
    },
    Denial {
        name: "ac3_denial_a_counter_without_its_wrap_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_in_line(t, r"^\| 59 \| 4 \| u32 \| Dead", r"cumulative, wraps modulo 2\^32 \| any 32-bit value \(wrap rule below\)", "cumulative | any 32-bit value"))),
        expect: &[("AC3", "the counter dead states its wrap")],
    },
    Denial {
        name: "ac3_denial_a_counter_without_its_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_in_line(t, r"^\| 19 \| 4 \| u32 \| Received", "slices accepted", "items accepted"))),
        expect: &[("AC3", "the counter received states its unit")],
    },
    Denial {
        name: "ac3_denial_no_observed_dscp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_all(t, "(?i)observed DSCP", "seen DSCP"))),
        expect: &[("AC3", "carries the observed DSCP")],
    },
    Denial {
        name: "ac5_denial_fewer_than_five_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| VALIDITY_SUBS.iter().fold(t.to_string(), |t, h| within_section(&t, h, |t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Form | Outcome |")))),
        expect: &[("AC5", "at least 5 invalid forms")],
    },
    Denial {
        name: "ac5_denial_a_reason_outside_the_control_layer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Validity rules of the section types$", |t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadChecksum"))),
        expect: &[("AC5", "names a reason outside the control layer")],
    },
    Denial {
        name: "ac5_denial_a_silent_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Validity rules of the section types$", |t| replace_in_line(t, r"^\| a capability states section that lacks", r"\| drop the section.*$", "| dropped silently |"))),
        expect: &[("AC5", "names no Reject reason")],
    },
    Denial {
        name: "ac5_denial_reserved_bits_without_reserved_non_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Validity rules of the section types$", |t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadLength"))),
        expect: &[("AC5", "must name ReservedNonZero")],
    },
    Denial {
        name: "ac5_denial_a_length_form_without_bad_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Validity rules of the section types$", |t| replace_in_line(t, r"^\| section length inconsistent", r"\| drop the section.*$", "| drop the section and count under TooShort |"))),
        expect: &[("AC5", "a length form must name BadLength")],
    },
    Denial {
        name: "ac5_denial_a_count_form_without_bad_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Validity rules of the section types$", |t| replace_in_line(t, r"^\| entry count above its maximum", r"\| drop the section.*$", "| drop the section and count under TooShort |"))),
        expect: &[("AC5", "a count-above-maximum form must name BadLength")],
    },
    Denial {
        name: "ac5_denial_no_length_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| VALIDITY_SUBS.iter().fold(t.to_string(), |t, h| within_section(&t, h, |t| replace_all(t, "section length inconsistent", "section size inconsistent")))),
        expect: &[("AC5", "an invalid form covers: length inconsistent")],
    },
    Denial {
        name: "dod_denial_reserved_fields_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Reserved fields$", |t| replace_all(t, "(?i)regime stamp", "stamp"))),
        expect: &[("DoD", "is recorded in the Reserved fields table")],
    },
    Denial {
        name: "dod_denial_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window section$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""))),
        expect: &[("DoD", "the commissioning subsection carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| strip_tags(t, "^The unit agrees with the manifest's"))),
        expect: &[("DoD", "every prose paragraph of 5 or more words in the calibration subsection")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "Open item 99: the spare bits are unassigned `[decision]`.\n\n"))),
        expect: &[("DoD", "every Open item paragraph")],
    },
    Denial {
        name: "ac6_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[[broken")),
        expect: &[("AC6", "does not parse")],
    },
    Denial {
        name: "ac6_denial_the_merged_message_changed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(LEGACY), &LEGACY.replace("width = 16", "width = 8"))),
        expect: &[("AC6", "the merged regime stamp section message of wire.toml is unchanged")],
    },
    Denial {
        name: "ac6_denial_no_message_for_a_type",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"regime stamp calibration factor section\""), "name = \"regime stamp factor section\"")),
        expect: &[("AC6", "exactly one regime stamp message for the calibration section type (found 0)")],
    },
    Denial {
        name: "ac6_denial_a_max_size_off_the_table",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("max_size = 66", "max_size = 67"))),
        expect: &[("AC6", "the calibration message of wire.toml is control")],
    },
    Denial {
        name: "ac6_denial_a_reason_outside_the_control_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &format!("{CAL_HEAD}\"BadChecksum\", "))),
        expect: &[("AC6", "the calibration message names only control reasons")],
    },
    Denial {
        name: "ac6_denial_fields_that_disagree",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"calibration_factor\"\noffset = 464\nwidth = 64"), "name = \"calibration_factor\"\noffset = 464\nwidth = 32")),
        expect: &[("AC6", "the calibration fields of wire.toml agree")],
    },
    Denial {
        name: "ac6_denial_an_entry_message_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"regime stamp multi-packet entry\""), "name = \"regime stamp multi-packet item\"")),
        expect: &[("AC6", "one entry message per entry table of the multi section")],
    },
    Denial {
        name: "ac6_denial_an_entry_message_that_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(MULTI_ENTRY), &MULTI_ENTRY.replace("max_size = 8", "max_size = 9"))),
        expect: &[("AC6", "the entry table of regime stamp multi-packet entry agrees")],
    },
    Denial {
        name: "dod_denial_sender_rule_7_not_quoted",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_all(t, "a stamp that needs more cannot be sent", "a stamp that needs more is not sent"))),
        expect: &[("DoD", "the merged sender rule 7 is quoted")],
    },
    Denial {
        name: "dod_denial_the_health_maximum_off_the_constants",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-217 \|", "63 peers", "62 peers")),
        expect: &[("DoD", "the health maximum equals the constants' product")],
    },
    Denial {
        name: "dod_denial_the_product_without_its_constants",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_all(t, "C-217", "the peer map"))),
        expect: &[("DoD", "states the product with its constants C-09 and C-217")],
    },
    Denial {
        name: "dod_denial_the_report_without_the_total",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, REPORT_LINE, "1 799 against 64, an excess of 1 799", "many against 64, an excess"))),
        expect: &[("DoD", "states the total 1799 sections")],
    },
    Denial {
        name: "dod_denial_the_report_without_the_design_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_all(t, "design owner", "lead"))),
        expect: &[("DoD", "goes to the design owner")],
    },
    Denial {
        name: "dod_denial_the_report_without_rule_7",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, REPORT_LINE, "a stamp that needs more cannot be sent", "a stamp that needs more is not sent"))),
        expect: &[("DoD", "cites merged sender rule 7")],
    },
    Denial {
        name: "dod_denial_the_report_without_no_stamp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, REPORT_LINE, "sends no stamp at all", "sends nothing at all"))),
        expect: &[("DoD", "sends no stamp at all")],
    },
    Denial {
        name: "dod_denial_the_report_without_the_threshold",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| every_in_line(t, REPORT_LINE, "1 026", "many"))),
        expect: &[("DoD", "gives the threshold in entries")],
    },
    Denial {
        name: "dod_denial_the_report_untagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| strip_tags(t, REPORT_LINE))),
        expect: &[("DoD", "the report paragraph carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_a_capability_named_without_its_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Capability states section$", |t| replace_in_line(&delete_row(t, r"^\| 3 \| CPU idle states"), "^Ids 1 to 4 are the capabilities", "^", "Idle states matter to 013-06 `[decision]`. "))),
        expect: &[("DoD", "the capability \"idle states\" named with a ticket id")],
    },
    Denial {
        name: "dod_denial_the_stated_capability_count_wrong",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_all(t, "stamps nine capabilities", "stamps ten capabilities"))),
        expect: &[("DoD", "the stated count of 10 capabilities")],
    },
    Denial {
        name: "dod_denial_a_manifest_item_handed_back",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, "^Ids 1 to 4 are the capabilities", "^", "Manifest, Open item 9 is left to its owner `[decision]`. "))),
        expect: &[("DoD", "review C3")],
    },
    Denial {
        name: "dod_denial_no_admission_marker",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_all(&replace_all(t, "(?i)admission|request id|generation", "handshake"), "(?i)reset", "restart"))),
        expect: &[("DoD", "review C4")],
    },
    Denial {
        name: "ac6_denial_a_multi_line_string_the_reader_rejects",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n")),
        expect: &[("AC6", "does not parse")],
    },
    Denial {
        name: "ac1_denial_a_heading_after_a_python_line_break",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, "^This part lays out the seven section types", "^", "\u{2028}## Interlude\u{2028}"))),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "dod_denial_a_word_character_only_python_knows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_all(t, "stamps nine capabilities", "stamps nine² capabilities"))),
        expect: &[("DoD", "the stated count of -1 capabilities")],
    },
    Denial {
        name: "ac2_denial_a_state_table_without_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Capability states section$", |t| replace_all(t, r"\| Value \| State \| Meaning \|\n\|---\|---\|---\|\n", "| Value | State | Meaning |\n"))),
        expect: &[("AC2", "exactly the four C-129 states")],
    },
    Denial {
        name: "accepted_a_separator_row_only_the_scripts_accept",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Capability states section$", |t| replace_all(t, r"\| Value \| State \| Meaning \|\n\|---\|---\|---\|\n", "| Value | State | Meaning |\n| - | :-: | - - |\n"))),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_no_control_bound_where_the_script_stops",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| C-17 \|")),
        expect: &[("AC1", "gives the control bound C-17 as 1 252"), ("CRASH", "")],
    },

    // ---- numbers and Python's classes: big integers, underscores, signs,
    // leading zeros, superscripts, empty and missing cells, duplicate rows,
    // exponents, values that make the script raise
    Denial {
        name: "num_c17_ten_to_19",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "10000000000000000000")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_c17_ten_to_20",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "100000000000000000000")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_c17_ten_to_36",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1000000000000000000000000000000000000")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_c17_beyond_i128",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_c17_leading_zero",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "01252")),
        expect: &[],
    },
    Denial {
        name: "num_c17_leading_plus",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "+1252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_c17_superscript_after",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252²")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_c17_superscript_before",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "²1 252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_c17_underscore",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1_252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_enum_big_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 99999999999999999999 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 10000000000000000000000000000000000000000 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 00 |")),
        expect: &[],
    },
    Denial {
        name: "num_enum_empty_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "|  |")),
        expect: &[("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "num_enum_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 0² |")),
        expect: &[("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "num_enum_duplicate_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 13 | TooShort | a duplicate row | control, sync, media, measurement |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_duplicate_row_same_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 0 | TooShort | a duplicate row | control, sync, media, measurement |")),
        expect: &[],
    },
    Denial {
        name: "num_enum_missing_layers_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", r" \| control, sync, media, measurement \|", " |")),
        expect: &[("AC5", ""), ("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "num_toml_dscp_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = 4_0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = \"40\""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_padded_signed_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = \" +040 \""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = \"_40\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_dscp_superscript_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = \"40²\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_dscp_big",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = 100000000000000000000"))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = 10000000000000000000000000000000000000000"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_toml_dscp_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = 40.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_plus",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("dscp = 40", "dscp = +40"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("max_size = 66", "max_size = 66.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("max_size = 66", "max_size = \"66\""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_bool",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("max_size = 66", "max_size = true"))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_max_size_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("max_size = 66", "max_size = \"abc\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_reasons_unhashable",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("rejection_reasons = [", "rejection_reasons = [[\"TooShort\"], "))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_reasons_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(CAL_HEAD), &CAL_HEAD.replace("rejection_reasons = [", "rejection_reasons = 5\nspare = ["))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_calibration_empty_width_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_in_line(t, "^\\| 0 \\| 8 \\| u64 \\| Calibration factor", "^\\| 0 \\| 8 \\|", "| 0 |  |"))),
        expect: &[("AC1", ""), ("DoD", "")],
    },
    Denial {
        name: "num_calibration_row_cut_to_two_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_in_line(t, "^\\| 0 \\| 8 \\| u64 \\| Calibration factor", r"^.*$", "| 0 | 8 |"))),
        expect: &[("AC1", ""), ("DoD", "")],
    },
    Denial {
        name: "num_sum_exponent_exact",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (2**3 + 58 = 66)"))),
        expect: &[],
    },
    Denial {
        name: "num_sum_exponent_wrong",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (2**3 + 58 = 67)"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_sum_leading_zero_syntax_error",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (08 + 58 = 66)"))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_sum_beyond_i128_exact",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (2**130 - 2**130 + 66 = 66)"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_sum_negative_exponent",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (2**-1 + 65 = 66)"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_sum_big_exact",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (100000000000000000000 - 100000000000000000000 + 66 = 66)"))),
        expect: &[],
    },
    Denial {
        name: "num_sum_result_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (1 + 1 = 10000000000000000000000000000000000000000)"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_sum_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Calibration factor section$", |t| replace_all(t, &regex::escape("58 + 8 = 66 bytes"), "58 + 8 = 66 bytes (58 + 8² = 66)"))),
        expect: &[],
    },
    Denial {
        name: "num_section_count_leading_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 064"))),
        expect: &[],
    },
    Denial {
        name: "num_section_count_big",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 100000000000000000000"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_section_count_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 10000000000000000000000000000000000000000"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_section_count_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 64²"))),
        expect: &[],
    },
];
