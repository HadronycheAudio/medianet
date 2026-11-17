//! The denial list of 004-26: the real documents with one edit, and what
//! the port and the old script must report on them (their failed criteria,
//! exactly; `CRASH` where the script stops with a Python exception).
//! Consumed by `tests/doc/c004_26.rs` (one test each) and by `doccheck-equiv
//! 004-26`, which runs each against the old script too. An entry with no
//! expectation is one both must pass.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::text::rx;
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;

/// Every match of `pattern` (a regex) on the first line matching `line`
/// replaced.
fn every_in_line(text: &str, line: &str, pattern: &str, to: &str) -> String {
    let re = rx(line);
    let at = text
        .split('\n')
        .position(|l| re.is_match(l))
        .expect("a line matches");
    let p = rx(pattern);
    text.split('\n')
        .enumerate()
        .map(|(i, l)| {
            if i == at {
                p.replace_all_literal(l, to)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const REQ_E: &str = r"^\| 12 \| 4 \| u32 \| Required E";
const DEC_E: &str = r"^\| 16 \| 4 \| u32 \| Declared E";
const REPORT_P: &str = r"^\*\*Falsified: the one-message manifest";
const FIXED: &str = "name = \"manifest fixed part\"\nlayer = \"control\"\ndscp = 40\nmax_size = 31";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", "does not exist"), ("AC4", ""), ("AC5", ""), ("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC1", ""), ("AC6", "does not exist")],
    },
    Denial {
        name: "ac3_denial_the_control_bound_unread",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most many bytes")),
        expect: &[("AC3", "the control bound (C-17) reads as None")],
    },
    Denial {
        name: "dod_denial_the_hello_header_short",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Common control header$", |t| delete_row(t, r"^\| 8 \| 8 \| u64 \| Message counter"))),
        expect: &[("DoD", "the Hello common control header is readable (seven rows)")],
    },
    Denial {
        name: "ac3_denial_the_envelope_unread",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest section$", |t| replace_in_line(t, r"\| Section count", "1 to 255", "one or more"))),
        expect: &[("AC3", "the landed manifest section reads")],
    },
    Denial {
        name: "ac3_denial_the_envelope_maxima_differ",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest section$", |t| replace_in_line(t, r"\| Manifest bytes", "max 1196", "max 1195"))),
        expect: &[("AC3", "manifest bytes maximum equals its manifest length maximum")],
    },
    Denial {
        name: "ac3_denial_the_envelope_off_the_bound",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most 1 300 bytes")),
        expect: &[("AC3", "the landed manifest section computes to")],
    },
    Denial {
        name: "dod_denial_an_enumeration_short",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 9 \| BadPacketIndex \|")),
        expect: &[("DoD", "the landed enumerations read as 12 Reject names")],
    },
    Denial {
        name: "ac1_denial_no_manifest_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Manifest$", "Declaration")),
        expect: &[("AC1", "wire.md has a '## Manifest' section"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"))),
        expect: &[("AC1", "the Manifest section has field tables (0 found)"), ("AC2", ""), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_malformed_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Join element$", |t| replace_in_line(t, r"^\| 0 \| 4 \| u32 \| Inserted join delay", r"^\| 0 \| 4 \|", "| 0 | four |")))),
        expect: &[("AC1", "Manifest field table 5: the Width cell"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Bundle element$", |t| replace_in_line(t, r"^\| 1 \| 1 \| u8 \| Member count", r"^\| 1 ", "| 2 ")))),
        expect: &[("AC1", "does not follow the previous row")],
    },
    Denial {
        name: "ac1_denial_no_content_type_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "(?i)content[- ]types?", "kinds"))),
        expect: &[("AC1", "ports and content types have fields"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_channel_map_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "(?i)channel map", "channel table"))),
        expect: &[("AC1", "the channel map has a field")],
    },
    Denial {
        name: "ac1_denial_l_without_samples",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, r"^\| 0 \| 4 \| u32 \| Structural latency L", "(?i)sample", "unit"))),
        expect: &[("AC1", "L has a field whose valid range is in whole samples")],
    },
    Denial {
        name: "ac1_denial_no_counted_join_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, r"^\| 6438 \| 5 × max 255", "(?i)join", "jn"))),
        expect: &[("AC1", "the join-delay count is a counted row")],
    },
    Denial {
        name: "ac1_denial_join_delays_without_samples",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(&every_in_line(t, r"^\| 6438 \| 5 × max 255", "(?i)sample", "unit"), r"^\| 0 \| 4 \| u32 \| Inserted join delay", "(?i)sample", "unit"))),
        expect: &[("AC1", "each inserted join delay is a whole number of samples")],
    },
    Denial {
        name: "ac1_denial_no_frequency_flag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "(?i)frequency", "freq"))),
        expect: &[("AC1", "the frequency-dependent-delay flag")],
    },
    Denial {
        name: "ac1_denial_no_initial_values",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "(?i)initial value", "start value"))),
        expect: &[("AC1", "dense controller initial values")],
    },
    Denial {
        name: "ac1_denial_no_conversion_delay_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Port element$", |t| replace_all(t, "(?i)conversion delay", "resampler lag")))),
        expect: &[("AC1", "the conversion delay on a device-facing port (C-207) has a field")],
    },
    Denial {
        name: "ac1_denial_the_conversion_delay_without_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Port element$", |t| replace_in_line(t, r"^\| 1 \| 4 \| u32 \| Conversion delay", r"\| Conversion delay of a device-facing port.*$", "| Conversion delay of a device-facing port (C-207) | any value |")))),
        expect: &[("AC1", "the conversion delay states its unit")],
    },
    Denial {
        name: "ac1_denial_the_cost_bound_without_operations",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, r"^\| 4 \| 8 \| u64 \| Cost bound", "(?i)operation", "step"))),
        expect: &[("AC1", "the cost bound (C-106) has a field in operations")],
    },
    Denial {
        name: "ac1_denial_the_cost_bound_not_per_slice",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, r"^\| 4 \| 8 \| u64 \| Cost bound", "(?i)slice", "frame"))),
        expect: &[("AC1", "the cost bound is per slice")],
    },
    Denial {
        name: "ac1_denial_no_required_e",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, REQ_E, "(?i)required", "wanted"))),
        expect: &[("AC1", "required E per presentation group has a field"), ("AC2", "")],
    },
    Denial {
        name: "ac1_denial_no_declared_e",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, DEC_E, "(?i)declared", "stated"))),
        expect: &[("AC1", "declared E per sink has a field"), ("AC2", "")],
    },
    Denial {
        name: "ac1_denial_no_unit_of_e",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(&every_in_line(t, REQ_E, r"nanoseconds|\bns\b", "ticks"), DEC_E, r"nanoseconds|\bns\b", "ticks"))),
        expect: &[("AC1", "the unit of E is stated")],
    },
    Denial {
        name: "ac2_denial_a_code_inside_its_range",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, REQ_E, "0 to 4 294 967 293", "0 to 4 294 967 295"))),
        expect: &[("AC2", "lies outside the E range")],
    },
    Denial {
        name: "ac2_denial_no_none_encoding",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, REQ_E, r"\bnone\b", "absent"))),
        expect: &[("AC2", "the none encoding of required E is stated")],
    },
    Denial {
        name: "ac2_denial_no_unknown_encoding",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, DEC_E, r"\bunknown\b", "absent"))),
        expect: &[("AC2", "the unknown encoding of declared E is stated")],
    },
    Denial {
        name: "ac2_denial_a_flag_without_not_interpreted",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, REQ_E, "0 to 4 294 967 293 ns", "any value below 4 294 967 294 ns"))),
        expect: &[("AC2", "stated as not interpreted when the flag says none")],
    },
    Denial {
        name: "ac2_denial_a_flag_without_its_distinct_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(&replace_in_line(t, REQ_E, "0 to 4 294 967 293 ns", "any value below 4 294 967 294 ns"), "(?i)distinct|never|cannot|not a valid|no valid|differ", "odd"))),
        expect: &[("AC2", "a sentence states that none is distinct"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_the_two_sentinels_equal",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, DEC_E, "0xfffffffe", "0xffffffff"))),
        expect: &[("AC2", "the none and unknown sentinels are distinct numbers")],
    },
    Denial {
        name: "ac3_denial_the_maximum_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "18 977 bytes", "18 977 octets"))),
        expect: &[("AC3", "the maximum manifest size is stated in bytes"), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_more_sections_than_the_envelope_allows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest section$", |t| replace_in_line(t, r"\| Section count", "1 to 255", "1 to 15"))),
        expect: &[("AC3", "within the landed count maximum")],
    },
    Denial {
        name: "ac3_denial_a_denied_falsification",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^Proposed, for epic 005 to confirm", "^", "No constant is falsified here `[decision]`. "))),
        expect: &[("AC3", "denies a falsification")],
    },
    Denial {
        name: "ac3_denial_no_falsification_report",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, REPORT_P, "012-08", "012-99"))),
        expect: &[("AC3", "a paragraph reports as falsified")],
    },
    Denial {
        name: "ac3_denial_ac3_not_affirmed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(&replace_in_line(t, REPORT_P, "this falsifies the design statements", "this questions the design statements"), REPORT_P, r#"\(C-17\)", is falsified for the manifest"#, "(C-17)\", is not falsified for the manifest"))),
        expect: &[("AC3", "affirmatively falsifies the epic's AC-3")],
    },
    Denial {
        name: "ac3_denial_012_08_not_affirmed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(&replace_in_line(t, REPORT_P, "this falsifies the design statements", "this questions the design statements"), REPORT_P, r#"\(C-17\)", is falsified too"#, "(C-17)\", is not falsified too"))),
        expect: &[("AC3", "affirmatively falsifies 012-08 AC 2")],
    },
    Denial {
        name: "ac3_denial_the_report_without_the_excess",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, REPORT_P, "(?i)exceeds", "passes"))),
        expect: &[("AC3", "and that it exceeds")],
    },
    Denial {
        name: "ac3_denial_the_report_without_the_design_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, REPORT_P, "(?i)design owner", "lead"))),
        expect: &[("AC3", "goes to the design owner")],
    },
    Denial {
        name: "ac3_denial_the_report_without_the_split",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, REPORT_P, "(?i)split", "division"))),
        expect: &[("AC3", "proposes the split")],
    },
    Denial {
        name: "ac4_denial_the_falsifier_clause_not_met",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", "(?i)already met", "open"))),
        expect: &[("AC4", "already met at 1 252 bytes")],
    },
    Denial {
        name: "ac4_denial_no_join_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Open items$", |t| replace_all(t, r"(?i)\bjoin", "merge")))),
        expect: &[("AC4", "an open item records the join-delay bound")],
    },
    Denial {
        name: "ac4_denial_the_join_item_not_owned_by_004",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", r"(?i)\b(epic\s*)?004\b", "epic 005"))),
        expect: &[("AC4", "owned by epic 004")],
    },
    Denial {
        name: "ac5_denial_fewer_than_five_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_in_line(t, r"^\| held manifest bytes above the maximum", "$", "\n")))),
        expect: &[("AC5", "at least five invalid forms")],
    },
    Denial {
        name: "ac5_denial_an_outcome_without_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_in_line(t, r"^\| frames per slice of 0", "BadLength", "nothing")))),
        expect: &[("AC5", "names its outcome as a member")],
    },
    Denial {
        name: "ac5_denial_no_count_above_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_all(t, r"(?i)\bcount", "number")))),
        expect: &[("AC5", "a count above its maximum is an invalid form")],
    },
    Denial {
        name: "ac5_denial_the_length_form_without_bad_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_in_line(t, r"^\| length inconsistent", "BadLength", "TooShort")))),
        expect: &[("AC5", "a length inconsistent with the sections")],
    },
    Denial {
        name: "ac5_denial_no_channel_map_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_all(t, "(?i)channel[- ]map", "channel table")))),
        expect: &[("AC5", "a channel-map entry out of range")],
    },
    Denial {
        name: "ac5_denial_no_conversion_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_all(t, "(?i)conversion", "resampling")))),
        expect: &[("AC5", "a conversion delay on a port that is not device-facing")],
    },
    Denial {
        name: "ac5_denial_no_reserved_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_all(t, "(?i)reserved", "spare")))),
        expect: &[("AC5", "a non-zero reserved field")],
    },
    Denial {
        name: "ac5_denial_no_repeat_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| every_in_line(t, r"^\| duplicate section index", "section index", "section place")))),
        expect: &[("AC5", "an identical repeated section has an invalid-form row")],
    },
    Denial {
        name: "ac5_denial_the_repeat_a_rejection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_in_line(t, r"^\| duplicate section index", r"\|$", "and BadLength |")))),
        expect: &[("AC5", "the identical repeated section is not a rejection")],
    },
    Denial {
        name: "ac5_denial_no_manifest_length_disagreement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| every_in_line(t, r"^\| sections disagree on the manifest length", "manifest length", "manifest size")))),
        expect: &[("AC5", "disagree on the manifest length have")],
    },
    Denial {
        name: "ac5_denial_no_section_count_disagreement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| every_in_line(t, r"^\| sections disagree on the section count", "section count", "count of sections")))),
        expect: &[("AC5", "disagree on the section count have")],
    },
    Denial {
        name: "ac5_denial_a_disagreement_naming_two_reasons",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_in_line(t, r"^\| sections disagree on the section count", r"\|$", "and ForeignDomain |")))),
        expect: &[("AC5", "names exactly one Reject reason")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| strip_tags(t, "^The conversion delay's unit is whole samples"))),
        expect: &[("DoD", "carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_an_untagged_subsection_with_a_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Join element$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", "")))),
        expect: &[("DoD", "subsection 'Join element' with a table")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", "^", "Open item 10: the spare bits are unassigned `[decision]`.\n\n"))),
        expect: &[("DoD", "names an owner and a falsifier")],
    },
    Denial {
        name: "dod_denial_a_major_change_sentence_without_the_exception",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "Every addition to the manifest is a major change `[decision]`. "))),
        expect: &[("DoD", "excepts a new content-type code")],
    },
    Denial {
        name: "dod_denial_a_measured_maximum",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "The maximum is 51 ports `[measured]`. "))),
        expect: &[("DoD", "no stated maximum carries the [measured] tag")],
    },
    Denial {
        name: "dod_denial_a_chosen_number_without_decision",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "The bound of 9 is chosen, at most 9 `[code]`. "))),
        expect: &[("DoD", "chosen or not derived is tagged [decision]")],
    },
    Denial {
        name: "dod_denial_a_reserved_field_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Manifest join element bits 7..1 of byte 4")),
        expect: &[("DoD", "has a row in the landed Reserved fields table")],
    },
    Denial {
        name: "dod_denial_a_wait_without_bad_member_list",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "A receiver waits for every section `[decision]`. "))),
        expect: &[("DoD", "names BadMemberList as waiting")],
    },
    Denial {
        name: "dod_denial_a_landed_header_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Admission request$", |t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"\| u8 \|", "| i8 |"))),
        expect: &[("DoD", "landed 'Admission request' header rows equal")],
    },
    Denial {
        name: "dod_denial_a_landed_table_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Admission accept$", "Admission yes")),
        expect: &[("DoD", "landed 'Admission accept' table is present")],
    },
    Denial {
        name: "dod_denial_a_landed_table_without_fields",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Admission refusal$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"))),
        expect: &[("DoD", "landed 'Admission refusal' has a field table")],
    },
    Denial {
        name: "ac6_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[[broken")),
        expect: &[("AC1", ""), ("AC6", "does not parse")],
    },
    Denial {
        name: "ac6_denial_a_multi_line_string_the_reader_rejects",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n")),
        expect: &[("AC1", ""), ("AC6", "does not parse")],
    },
    Denial {
        name: "ac6_denial_no_owner_for_the_tooling_gap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, r"(?i)wire-check|schema|checker", "tool"))),
        expect: &[("AC6", "wire.toml cannot declare it as one message")],
    },
    Denial {
        name: "ac6_denial_a_table_without_its_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 5\nrejection_reasons = [\"BadLength\", \"ReservedNonZero\"]\n\n[[message.field]]\nname = \"inserted_join_delay\"\noffset = 0\nwidth = 32"), "name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 5\nrejection_reasons = [\"BadLength\", \"ReservedNonZero\"]\n\n[[message.field]]\nname = \"inserted_join_delay\"\noffset = 0\nwidth = 24")),
        expect: &[("AC6", "Manifest field table 5 has a [[message]]")],
    },
    Denial {
        name: "ac6_denial_a_message_with_the_wrong_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 5"), "name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 6")),
        expect: &[("AC6", "Manifest field table 5: a matching toml message")],
    },
    Denial {
        name: "ac6_denial_no_fixed_part_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"required_e\"\noffset = 96\nwidth = 32"), "name = \"required_e\"\noffset = 96\nwidth = 31")),
        expect: &[("AC6", "declares the manifest fixed part as a [[message]]")],
    },
    Denial {
        name: "ac6_denial_the_fixed_part_of_the_wrong_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("max_size = 31", "max_size = 32"))),
        expect: &[("AC6", "the fixed-part message has max_size equal")],
    },
    Denial {
        name: "ac1_denial_wire_toml_without_the_audio_bound",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("with frames per slice x channel count x 4 at most 65 535"), "with frames per slice x channel count x 4 bounded")),
        expect: &[("AC1", "states the audio-length bound (65 535)")],
    },
    Denial {
        name: "ac1_denial_an_l_that_saturates",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, r"^\| 0 \| 4 \| u32 \| Structural latency L", "67 108 863", "99 108 863"))),
        expect: &[("AC1", "times the landed maximum hop count")],
    },
    Denial {
        name: "ac1_denial_the_hop_count_unreadable",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, "maximum hop count of", "hop limit of")),
        expect: &[("AC1", "are all readable")],
    },
    Denial {
        name: "ac1_denial_no_audio_length_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "65 535", "sixty-five thousand"))),
        expect: &[("AC1", "the landed audio length bound")],
    },
    Denial {
        name: "ac2_denial_a_stray_prose_sentinel",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^The unit of E is the nanosecond", "^", "The none code of E is not 0x1234 `[decision]`. "))),
        expect: &[("AC2", "the prose sentinel for none agrees")],
    },
    Denial {
        name: "ac2_denial_wire_toml_states_another_sentinel",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("0xffffffff = none: the group sets no required E"), "0xfffffff0 = none: the group sets no required E")),
        expect: &[("AC2", "wire.toml's valid range for the none field")],
    },
    Denial {
        name: "ac4_denial_a_heading_after_a_python_line_break",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", "^", "\u{2028}## Interlude\u{2028}"))),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "ac2_denial_a_word_character_only_python_knows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| every_in_line(t, DEC_E, r"\bunknown\b", "unknown²"))),
        expect: &[("AC2", "the unknown encoding of declared E is stated")],
    },
    Denial {
        name: "ac5_denial_a_validity_table_without_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n")))),
        expect: &[("AC5", "at least five invalid forms")],
    },
    Denial {
        name: "accepted_a_separator_row_only_the_scripts_accept",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |m| within_section(m, "^Validity rules$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n")))),
        expect: &[],
    },

    // ---- numbers and Python's classes: big integers, underscores, signs,
    // leading zeros, superscripts, empty and missing cells, duplicate rows,
    // exponents, values that make the script raise
    Denial {
        name: "num_c17_ten_to_19",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "10000000000000000000")),
        expect: &[("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "num_c17_ten_to_20",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "100000000000000000000")),
        expect: &[("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "num_c17_ten_to_36",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1000000000000000000000000000000000000")),
        expect: &[("AC3", ""), ("AC6", "")],
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
        expect: &[("AC3", "")],
    },
    Denial {
        name: "num_c17_superscript_after",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252²")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "num_c17_superscript_before",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "²1 252")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "num_c17_underscore",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1_252")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "num_c17_hex",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "0x4E4")),
        expect: &[],
    },
    Denial {
        name: "num_c17_hex_upper",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "0X4E4")),
        expect: &[],
    },
    Denial {
        name: "num_c17_comma",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1,252")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "num_c17_narrow_nbsp",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252")),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "num_enum_big_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 99999999999999999999 |")),
        expect: &[],
    },
    Denial {
        name: "num_enum_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 10000000000000000000000000000000000000000 |")),
        expect: &[],
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
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 0² |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_duplicate_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 13 | TooShort | a duplicate row | control, sync, media, measurement |")),
        expect: &[],
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
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = 4_0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = \"40\""))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_padded_signed_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = \" +040 \""))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = \"_40\""))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_superscript_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = \"40²\""))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_big",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = 100000000000000000000"))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = 10000000000000000000000000000000000000000"))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_dscp_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = 40.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_plus",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("dscp = 40", "dscp = +40"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("max_size = 31", "max_size = 31.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("max_size = 31", "max_size = \"31\""))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_max_size_bool",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("max_size = 31", "max_size = true"))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_toml_max_size_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(FIXED), &FIXED.replace("max_size = 31", "max_size = \"abc\""))),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "num_required_e_empty_width_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", "^\\| 12 \\| 4 \\|", "| 12 |  |"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "num_required_e_row_cut_to_two_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", r"^.*$", "| 12 | 4 |"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "num_manifest_size_hex",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "maximum manifest size is 18 977 bytes", "maximum manifest size is 0x4A21 bytes"))),
        expect: &[],
    },
    Denial {
        name: "num_manifest_size_leading_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "maximum manifest size is 18 977 bytes", "maximum manifest size is 018 977 bytes"))),
        expect: &[],
    },
    Denial {
        name: "num_manifest_size_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "maximum manifest size is 18 977 bytes", "maximum manifest size is 18 977² bytes"))),
        expect: &[],
    },
    Denial {
        name: "num_manifest_size_big",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "maximum manifest size is 18 977 bytes", "maximum manifest size is 100000000000000000000 bytes"))),
        expect: &[],
    },
    Denial {
        name: "num_manifest_size_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "maximum manifest size is 18 977 bytes", "maximum manifest size is 10000000000000000000000000000000000000000 bytes"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_manifest_size_comma",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_all(t, "maximum manifest size is 18 977 bytes", "maximum manifest size is 18,977 bytes"))),
        expect: &[],
    },
    // ---- the re-proof checklist (points 3, 5 and 6): number forms, table
    // reader variants, enumeration ties, swaps and thresholds
    Denial {
        name: "chk_c17_zero",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "0")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "chk_c17_double_zero_seven",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "007")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "chk_c17_two_to_63",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "9223372036854775808")),
        expect: &[("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "chk_c17_forty_five_digits",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "111111111111111111111111111111111111111111111")),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "chk_c17_micro_before",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "µ1 252")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "chk_c17_micro_after",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252µ")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "chk_enum_swap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(&replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 1 |"), r"^\| 1 \| BadMagic \|", r"^\| 1 \|", "| 0 |")),
        expect: &[],
    },
    Denial {
        name: "chk_enum_tie",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 1 \| BadMagic \|", r"^\| 1 \|", "| 0 |")),
        expect: &[],
    },
    Denial {
        name: "chk_enum_twelve_members",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 12 \| ReservedNonZero \|")),
        expect: &[("AC5", ""), ("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "chk_enum_fourteen_members",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 13 | SpareReason | a fourteenth member | control |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "chk_enum_value_two_to_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 9223372036854775808 |")),
        expect: &[],
    },
    Denial {
        name: "chk_enum_value_micro",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| µ0 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "chk_enum_escaped_pipe",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", "fewer bytes", "fewer \\| bytes")),
        expect: &[],
    },
    Denial {
        name: "chk_required_e_row_escaped_pipe",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", "$", " \\| x"))),
        expect: &[],
    },
    Denial {
        name: "chk_required_e_row_blank_line_inside",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", "^", "\n"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "chk_required_e_row_ragged_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", "$", " extra |"))),
        expect: &[],
    },
    Denial {
        name: "chk_required_e_row_fenced_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", "^", "```\n"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", "")],
    },
    Denial {
        name: "chk_required_e_row_offset_double_zero_seven",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", r"^\| \d+", "| 007"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "chk_required_e_row_offset_plus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", r"^\| (\d+)", "| +7"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "chk_required_e_row_offset_forty_five_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Manifest$", |t| replace_in_line(t, "^\\| 12 \\| 4 \\| u32 \\| Required E", r"^\| \d+", "| 111111111111111111111111111111111111111111111"))),
        expect: &[("INT_LIMIT", "")],
    },
];
