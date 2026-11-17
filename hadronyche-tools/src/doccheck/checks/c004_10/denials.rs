//! The denial list of 004-10: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_10.rs` (one test each) and by `doccheck-equiv 004-10`
//! (each run against the old script as well). One denial per assertion of
//! the script, then the rows aimed at the Python semantics its code touches
//! (`py_*`, `toml_*`) and at the 13-name policy. An entry with no
//! expectation is one the port must pass, like the script: the agreement is
//! the point. A `CRASH` expectation is input on which the script raised
//! (red, no FAIL line), classified `ScriptCrashed` in `KNOWN`.

use super::{WIRE_MD, WIRE_TOML};
use crate::doccheck::denial::{Denial, DenialEdit};
#[allow(unused_imports)]
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::text::rx;

/// `edit` applied to the Measurement extension section of wire.md only.
fn sec(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Measurement extension$", edit)
}

/// `edit` applied to wire.toml's media packet `[[message]]` (its fields
/// included) only.
fn media(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    let start = t.find("name = \"media packet\"\n").expect("the media packet message");
    let end = start + t[start..].find("\n[[message]]\n").expect("a next message");
    format!("{}{}{}", &t[..start], edit(&t[start..end]), &t[end..])
}

/// The first line matching `line`: `from` (regex) → `to`.
fn line(t: &str, line: &str, from: &str, to: &str) -> String {
    replace_in_line(t, line, from, to)
}

/// `extra` appended to the first line matching `pattern`.
fn append(t: &str, pattern: &str, extra: &str) -> String {
    replace_in_line(t, pattern, "$", extra)
}

/// Every match of `pattern` replaced, with `$1` expansion.
fn re_all(t: &str, pattern: &str, to: &str) -> String {
    rx(pattern).replace_all(t, to).into_owned()
}

/// Names deleted from wire.md's Rejection reasons table.
#[allow(dead_code)]
fn fewer_names(t: &str, names: &[&str]) -> String {
    let mut t = t.to_string();
    for n in names {
        t = within_section(&t, "^Rejection reasons$", |s| delete_row(s, &format!(r"^\| \d+ \| {n} \|")));
    }
    t
}

const RULE8_PARA: &str = r"^No field of the extension identifies";
const HOP_ROW: &str = r"^\| \+0 \| 1 \| u8 \| hop count";
const RES_ROW: &str = r"^\| \+1 \| 4 \| u32 \| accumulated residence";
const DELAY_ROW: &str = r"^\| \+5 \| 4 \| u32 \| accumulated declared delay";
const HOP_FORM: &str = r"^\| hop count beyond the maximum";
const FLAG_FORM: &str = r"^\| measurement flag set but";
const RESERVED_FORM: &str = r"^\| reserved bits set";
const CAP_PARA: &str = r"^The C-137 header cap allows";
const VALIDITY_LEAD: &str = r"^Every malformed extension is dropped";

/// The extension's fields moved out of the media packet into a
/// `measurement extension` message (the section form).
fn section_form(t: &str) -> String {
    let t = media(t, |t| {
        let t = replace_all(t, "name = \"hop_count\"", "name = \"ext_a\"");
        let t = replace_all(&t, "name = \"residence_time\"", "name = \"ext_b\"");
        replace_all(&t, "name = \"declared_delay\"", "name = \"ext_c\"")
    });
    format!(
        "{t}\n[[message]]\nname = \"measurement extension\"\nsection_of = \"media packet\"\nlayer = \"media\"\ndscp = 46\nmax_size = 9\nrejection_reasons = [\"BadLength\", \"ReservedNonZero\"]\n\n\
         [[message.field]]\nname = \"hop_count\"\noffset = 0\nwidth = 8\ntype = \"u8\"\n\n\
         [[message.field]]\nname = \"residence_time\"\noffset = 8\nwidth = 32\ntype = \"u32\"\n\n\
         [[message.field]]\nname = \"declared_delay\"\noffset = 40\nwidth = 32\ntype = \"u32\"\n"
    )
}

/// A prose sentence naming ForeignDomain.
fn foreign_sentence(t: &str) -> String {
    append(t, RULE8_PARA, " A ForeignDomain packet is the header's case.")
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    // setup
    Denial {
        name: "setup_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC5", "does not exist")],
    },
    Denial {
        name: "setup_no_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Measurement extension$", "Measurement extension block")),
        expect: &[("AC4", "no 'Measurement extension' section")],
    },
    Denial {
        name: "setup_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |_| "## Measurement extension\n\nTo be filled by 004-10.\n".into())),
        expect: &[("DoD", "still the 004-10 placeholder")],
    },
    Denial {
        name: "setup_no_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\| Offset \(bytes\) \|", "| Position (bytes) |"))),
        expect: &[("AC1", "no field table")],
    },
    // AC1
    Denial {
        name: "ac1_no_delay_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, DELAY_ROW))),
        expect: &[("AC1", "no field-table row for the accumulated declared delay")],
    },
    Denial {
        name: "ac1_width_not_whole_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"^\| \+0 \| 1 \|", "| +0 | 6 bits |"))),
        expect: &[("AC1", "width is not a whole number of bytes")],
    },
    Denial {
        name: "ac1_type_not_unsigned",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"\| u8 \|", "| i8 |"))),
        expect: &[("AC1", "is not a fixed-width unsigned integer")],
    },
    Denial {
        name: "ac1_width_disagrees_with_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"\| u8 \|", "| u16 |"))),
        expect: &[("AC1", "disagrees with its type u16")],
    },
    Denial {
        name: "ac1_no_saturation_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RES_ROW, r"never wraps", "never overflows"))),
        expect: &[("AC1", "states no saturation rule")],
    },
    Denial {
        name: "ac1_hop_no_maximum",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"\| 0 to 64: .*\|$", "| any value |"))),
        expect: &[("AC1", "states no maximum")],
    },
    Denial {
        name: "ac1_hop_above_64",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"\| 0 to 64:", "| 0 to 65:"))),
        expect: &[("AC1", "exceeds the 64 modules of C-34")],
    },
    Denial {
        name: "ac1_hop_does_not_fit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"^\| \+0 \| 1 \| u8 \|", "| +0 | 5 bits | u5 |"))),
        expect: &[("AC1", "does not fit its width")],
    },
    Denial {
        name: "ac1_residence_unit_untagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, r"(?i)\b(?:ns|µs|us|ms)\b|nanoseconds|microseconds|milliseconds", "ticks"), r"(?i)samples at the domain rate", "domain samples"))),
        expect: &[("AC1", "residence-time unit is not stated")],
    },
    Denial {
        name: "ac1_delay_not_whole_samples",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, "C-105", "C-106"))),
        expect: &[("AC1", "whole-sample sum")],
    },
    Denial {
        name: "ac1_no_forwarding_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)forward", "relay"))),
        expect: &[("AC1", "no forwarding update rule")],
    },
    Denial {
        name: "ac1_no_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)epic 004", "the wire owner"))),
        expect: &[("AC1", "open item on the widths is not recorded")],
    },
    Denial {
        name: "ac1_falsifier_not_the_tickets",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"largest L\b", "biggest L"))),
        expect: &[("AC1", "the falsifier is not the ticket's")],
    },
    // AC2
    Denial {
        name: "ac2_size_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)(\d+)(\s*-?\s*)(bytes?)", "$1$2octets"))),
        expect: &[("AC2", "byte size is not stated as a number")],
    },
    Denial {
        name: "ac2_size_not_the_widths",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^The measurement extension is 9 bytes", "9 bytes", "10 bytes"))),
        expect: &[("AC2", "does not equal the sum of its field widths")],
    },
    Denial {
        name: "ac2_not_contiguous",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, DELAY_ROW, r"^\| \+5 \|", "| +6 |"))),
        expect: &[("AC2", "not contiguous")],
    },
    Denial {
        name: "ac2_no_c137_arithmetic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"\b62\b", "sixty-two"))),
        expect: &[("AC2", "C-137 arithmetic is not shown")],
    },
    Denial {
        name: "ac2_no_checksum_remainder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, CAP_PARA, r"4 bytes remain", "four bytes remain"))),
        expect: &[("AC2", "checksum remainder is not stated")],
    },
    Denial {
        name: "ac2_fit_not_said",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)\bfits?\b|\bfitting\b", "sits"))),
        expect: &[("AC2", "does not say whether the extension fits")],
    },
    Denial {
        name: "ac2_too_big_without_fallback",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, "The measurement extension is 9 bytes", "The measurement extension is 14 bytes"), r"(?i)fallback", "backup"))),
        expect: &[("AC2", "no omission on a fallback path is stated")],
    },
    // AC3
    Denial {
        name: "ac3_a_form_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, RESERVED_FORM))),
        expect: &[("AC3", "no invalid form listed for: reserved bits set")],
    },
    Denial {
        name: "ac3_a_form_without_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_FORM, r"count under BadLength", "count it"))),
        expect: &[("AC3", "carries no member")],
    },
    Denial {
        name: "ac3_a_form_under_the_wrong_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_FORM, r"count under BadLength", "count under WrongEpoch"))),
        expect: &[("AC3", "must be rejected under BadLength")],
    },
    Denial {
        name: "ac3_flag_form_under_the_wrong_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, FLAG_FORM, "count under BadLength", "count under WrongEpoch"), FLAG_FORM, "not TooShort", "not that"))),
        expect: &[("AC3", "must be rejected under BadLength or TooShort")],
    },
    Denial {
        name: "ac3_a_reason_outside_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, RULE8_PARA, " A BadExtension case does not arise."))),
        expect: &[("AC3", "outside the closed enumeration: BadExtension")],
    },
    Denial {
        name: "ac3_fewer_than_three",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_FORM, r"count under BadLength", "count it"))),
        expect: &[("AC3", "only 2 invalid form(s)")],
    },
    Denial {
        name: "ac3_headerless_validity_run_is_dropped",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\|---\|---\|$"))),
        expect: &[("AC3", "no invalid form listed for: flag set")],
    },
    // AC4
    Denial {
        name: "ac4_no_rule_8_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"rule 8", "the rule"))),
        expect: &[("AC4", "no statement that no field identifies")],
    },
    // AC5
    Denial {
        name: "ac5_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC5", "does not exist")],
    },
    Denial {
        name: "ac5_wire_toml_unreadable",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("note = \"\"\"x\"\"\"\n{t}")),
        expect: &[("AC5", "could not be read by this check")],
    },
    Denial {
        name: "ac5_no_container",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_count = 4\n", ""))),
        expect: &[("AC5", "declares no counted announcement region")],
    },
    Denial {
        name: "ac5_no_extension_fields",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(&replace_all(&replace_all(t, "name = \"hop_count\"", "name = \"ext_a\""), "name = \"residence_time\"", "name = \"ext_b\""), "name = \"declared_delay\"", "name = \"ext_c\""))),
        expect: &[("AC5", "declares no measurement-extension fields")],
    },
    Denial {
        name: "ac5_a_field_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"declared_delay\"", "name = \"ext_c\""))),
        expect: &[("AC5", "has no field for the accumulated declared delay")],
    },
    Denial {
        name: "ac5_width_not_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"hop_count\"\noffset = 696\nwidth = 8", "name = \"hop_count\"\noffset = 696\nwidth = \"8\""))),
        expect: &[("AC5", "width is not an integer")],
    },
    Denial {
        name: "ac5_width_not_whole_bytes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"hop_count\"\noffset = 696\nwidth = 8", "name = \"hop_count\"\noffset = 696\nwidth = 7"))),
        expect: &[("AC5", "is not whole bytes")],
    },
    Denial {
        name: "ac5_offset_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"residence_time\"\noffset = 704", "name = \"residence_time\"\noffset = 712"))),
        expect: &[("AC5", "wire.toml places it 16 bits into the extension, wire.md 8 bits")],
    },
    Denial {
        name: "ac5_width_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"residence_time\"\noffset = 704\nwidth = 32", "name = \"residence_time\"\noffset = 704\nwidth = 16"))),
        expect: &[("AC5", "wire.toml width 16 bits")],
    },
    Denial {
        name: "ac5_type_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| rx(r#"(name = "declared_delay"\noffset = \d+\nwidth = \d+\ntype = )"u32""#).replace(t, r#"${1}"u64""#).into_owned())),
        expect: &[("AC5", "wire.toml type 'u64'")],
    },
    Denial {
        name: "ac5_total_disagrees_with_the_stated_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"hop_count\"\noffset = 696\nwidth = 8", "name = \"hop_count\"\noffset = 696\nwidth = 16"))),
        expect: &[("AC5", "fields total")],
    },
    Denial {
        name: "ac5_max_size_not_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = \"96\""))),
        expect: &[("AC5", "max_size is not an integer")],
    },
    Denial {
        name: "ac5_media_max_size_too_small",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = 95"))),
        expect: &[("AC5", "does not hold the container's maximum end")],
    },
    Denial {
        name: "ac5_max_size_above_the_cap",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = 101"))),
        expect: &[("AC5", "exceeds the C-137 header cap")],
    },
    Denial {
        name: "ac5_no_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| re_all(t, r"(?m)^rejection_reasons = .*\n", ""))),
        expect: &[("AC5", "declares no rejection_reasons")],
    },
    Denial {
        name: "ac5_a_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "\"BadPacketIndex\"", "\"BadIndex\""))),
        expect: &[("AC5", "rejection reason 'BadIndex'")],
    },
    Denial {
        name: "ac5_reserved_non_zero_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, ", \"ReservedNonZero\"", ""))),
        expect: &[("AC5", "do not include ReservedNonZero")],
    },
    // DoD
    Denial {
        name: "dod_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| strip_tags(t, r"^A processor carries the extension on its output"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    // Python semantics
    Denial {
        name: "py_superscript_is_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_FORM, r"count under BadLength", "count under BadLength²"))),
        expect: &[("AC3", "carries no member")],
    },
    Denial {
        name: "py_a_combining_mark_is_not_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, RULE8_PARA, " Nothing is a FooBar\u{301}."))),
        expect: &[("AC3", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_unit_separator_is_whitespace_to_lstrip",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_FORM, r"^\|", "\u{1f}|"))),
        expect: &[],
    },
    Denial {
        name: "py_line_separator_starts_a_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, HOP_FORM), VALIDITY_LEAD, "\u{2028}- hop count beyond the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_form_feed_starts_a_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, HOP_FORM), VALIDITY_LEAD, "\u{c}- hop count beyond the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_next_line_starts_a_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, HOP_FORM), VALIDITY_LEAD, "\u{85}- hop count beyond the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_bare_carriage_return_is_a_newline",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, HOP_FORM), VALIDITY_LEAD, "\r- hop count beyond the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_dotless_i_matches_i_under_re_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RULE8_PARA, "No field of", "No fıeld of"))),
        expect: &[],
    },
    Denial {
        name: "py_int_is_unbounded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"\| 0 to 64:", "| 0 to 99999999999999999999999:"))),
        expect: &[("AC1", "exceeds the 64 modules of C-34")],
    },
    Denial {
        name: "py_missing_offset_crashes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, HOP_ROW, r"^\| \+0 \|", "| + |"))),
        expect: &[("CRASH", "raised an exception")],
    },
    Denial {
        name: "py_separator_without_dash_is_no_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\|---\|---\|---\|---\|---\|", "|===|===|===|===|===|"))),
        expect: &[("AC1", "no field table")],
    },
    Denial {
        name: "toml_float_width_is_not_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"hop_count\"\noffset = 696\nwidth = 8", "name = \"hop_count\"\noffset = 696\nwidth = 8.0"))),
        expect: &[("AC5", "width is not an integer")],
    },
    Denial {
        name: "toml_duplicate_key_last_wins",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96\n", "max_size = 96\nmax_size = 101\n"))),
        expect: &[("AC5", "exceeds the C-137 header cap")],
    },
    Denial {
        name: "toml_message_a_number_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("message = 0\n{}", t.replace("[[message]]", "[[messages]]").replace("[[message.", "[[messages."))),
        expect: &[("CRASH", "raised an exception")],
    },
    Denial {
        name: "toml_unhashable_reason_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, r#"rejection_reasons = \["TooShort","#, r#"rejection_reasons = [["TooShort"],"#))),
        expect: &[("CRASH", "raised an exception")],
    },

    Denial {
        name: "ac2_no_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                let t = line(t, HOP_ROW, r"^\| \+0 \|", "| + |");
                let t = line(&t, RES_ROW, r"^\| \+1 \| 4 \|", "| +1 | 1 |");
                line(&t, DELAY_ROW, r"^\| \+5 \| 4 \|", "| +5 | 1 |")
            })
        }),
        expect: &[("AC2", "states no offset")],
    },
    Denial {
        name: "ac5_section_form_passes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(section_form),
        expect: &[],
    },
    Denial {
        name: "ac5_section_form_without_section_of",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(&section_form(t), "section_of = \"media packet\"", "section_of = \"media\"")),
        expect: &[("AC5", "does not declare section_of")],
    },
    Denial {
        name: "ac5_section_form_max_size_not_the_bytes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(&section_form(t), "max_size = 9\n", "max_size = 10\n")),
        expect: &[("AC5", "max_size 10 != extension bytes 9")],
    },
    Denial {
        name: "ac5_section_form_media_too_small",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(&section_form(t), |t| replace_all(t, "max_size = 96", "max_size = 95"))),
        expect: &[("AC5", "media packet max_size does not hold")],
    },
    Denial {
        name: "ac5_section_form_media_above_the_cap",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(&section_form(t), |t| replace_all(t, "max_size = 96", "max_size = 101"))),
        expect: &[("AC5", "media packet max_size 101 exceeds")],
    },
    Denial {
        name: "toml_bare_word_section_of",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(&section_form(t), "section_of = \"media packet\"", "section_of = media packet")),
        expect: &[],
    },
    Denial {
        name: "ten_found_names_shrink_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(&fewer_names(t, &["BadChecksum", "OversizeControl", "ForeignDomain"]), foreign_sentence)),
        expect: &[("AC3", "outside the closed enumeration: ForeignDomain")],
    },
    Denial {
        name: "nine_found_names_keep_the_thirteen",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(&fewer_names(t, &["BadChecksum", "OversizeControl", "ForeignDomain", "UnknownStream"]), foreign_sentence)),
        expect: &[],
    },
];
