//! The denial list of 004-09: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_09.rs` (one test each) and by `doccheck-equiv 004-09`
//! (each run against the old script as well). One denial per assertion of
//! the script, each branch of the DEFECT detector (`defect_*`), then the rows
//! aimed at the Python semantics its code touches (`py_*`, `toml_*`) and at
//! the 13-name policy. An entry with no expectation is one the port must
//! pass, like the script. A `CRASH` expectation is input on which the script
//! raised (red, no FAIL line), classified `ScriptCrashed` in `KNOWN`.

use super::{WIRE_MD, WIRE_TOML};
use crate::doccheck::denial::{Denial, DenialEdit};
#[allow(unused_imports)]
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::text::rx;

/// `edit` applied to the Announcements section of wire.md only.
fn sec(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Announcements$", edit)
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


/// A prose sentence naming ForeignDomain.
fn foreign_sentence(t: &str) -> String {
    append(t, r"^Every malformed announcement is dropped", " A ForeignDomain packet is the header's case.")
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "setup_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC6", "does not exist")],
    },
    Denial {
        name: "setup_no_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Announcements$", "Announcement layouts")),
        expect: &[("AC1", "no 'Announcements' section")],
    },
    Denial {
        name: "setup_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |_| "## Announcements\n\nTo be filled by 004-09.\n".into())),
        expect: &[("DoD", "still the 004-09 placeholder")],
    },
    Denial {
        name: "ac1_presence_flag_not_tied",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)bit\s*5", "the fifth bit"))),
        expect: &[("AC1", "does not tie the presence flag")],
    },
    Denial {
        name: "ac1_no_container_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| Offset \(bytes\)", r"^\| Offset", "| Place"))),
        expect: &[("AC1", "no container field table")],
    },
    Denial {
        name: "ac1_no_skip_by_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)skip\w*", "pass over"))),
        expect: &[("AC1", "skip-by-length requirement")],
    },
    Denial {
        name: "ac1_skip_not_tied_to_versions",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(&re_all(t, r"(?i)minor", "later"), "004-04", "the versioning ticket"), r"(?i)same\s+major", "same family"))),
        expect: &[("AC1", "not tied to the version rule")],
    },
    Denial {
        name: "ac1_position_not_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"\| (\+\d+) \| 8 \| u64 \| position", "| $1 | 4 | u64 | position"))),
        expect: &[("AC1", "named future position as 64 bits")],
    },
    Denial {
        name: "ac1_future_never_said",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)future", "later"))),
        expect: &[("AC1", "FUTURE position")],
    },
    Denial {
        name: "ac1_a_type_never_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)transport[\s_-]*reference", "carrier swap"))),
        expect: &[("AC1", "never names the transport reference change")],
    },
    Denial {
        name: "ac1_a_type_without_a_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, r"^\| 0 \| end-of-stream \|", r"^\| 0 \|", "| zero |"), r"^\| \+0 \| 8 \| u64 \| position: the stream's last position", "end-of-stream boundary", "final boundary"))),
        expect: &[("AC1", "end-of-stream announcement has no stated TYPE CODE")],
    },
    Denial {
        name: "ac1_a_type_without_a_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, r"^Re-alignment \(type code 1\)$", |s| re_all(s, r"(?i)re[\s_-]*alignment", "re-anchoring"))),
        expect: &[("AC1", "re-alignment announcement has no field table")],
    },
    Denial {
        name: "ac1_no_byte_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)(\d+)(\s*)bytes?\b", "$1$2octets"))),
        expect: &[("AC1", "has no stated BYTE SIZE")],
    },
    Denial {
        name: "ac2_max_count_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "at most 4 announcements per packet", "at most four announcements per packet"), "at most 48 announcement bytes per packet", "at most forty-eight announcement bytes per packet"))),
        expect: &[("AC2", "maximum NUMBER of announcements")],
    },
    Denial {
        name: "ac2_max_bytes_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "at most 48 announcement bytes per packet", "at most forty-eight bytes per packet"))),
        expect: &[("AC2", "maximum ANNOUNCEMENT BYTES")],
    },
    Denial {
        name: "ac2_no_cap_arithmetic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, r"\b62\b", "sixty-two"), r"\b38\b", "thirty-eight"))),
        expect: &[("AC2", "does not show the C-137 cap arithmetic")],
    },
    Denial {
        name: "ac2_max_bytes_above_the_remainder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "48 announcement bytes", "50 announcement bytes"))),
        expect: &[("AC2", "do not fit the C-137 remainder")],
    },
    Denial {
        name: "ac2_numbers_not_self_consistent",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "48 announcement bytes", "47 announcement bytes"))),
        expect: &[("AC2", "not self-consistent")],
    },
    Denial {
        name: "ac3_unknown_type_not_skipped",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, r"^\\| unknown type code within the same major", r"(?i)skips", "passes"), r"^\\| unknown type code within the same major", r"(?i)skip by length", "pass"))),
        expect: &[("AC3", "does not state the skippable outcome")],
    },
    Denial {
        name: "ac3_unknown_type_rejected",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\\| unknown type code within the same major", " Count under BadLength."))),
        expect: &[("AC3", "is given a rejection outcome")],
    },
    Denial {
        name: "ac3_a_form_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\| duplicate type in one packet"))),
        expect: &[("AC3", "no invalid form listed for: duplicate")],
    },
    Denial {
        name: "ac3_count_above_not_bad_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| count above the maximum", "count under BadLength", "count under ReservedNonZero"))),
        expect: &[("AC3", "must be rejected under BadLength")],
    },
    Denial {
        name: "ac3_a_form_without_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| duplicate type in one packet", "count under BadLength", "count it"))),
        expect: &[("AC3", "carries no member")],
    },
    Denial {
        name: "ac3_gap_form_without_an_outcome",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, r"^\| prefix cut short", "non-gap", "non-empty"), r"^\| announcement in a gap slice", r"(?s).*", "| announcement in a gap slice | open |"))),
        expect: &[("AC3", "gap-slice form states no outcome")],
    },
    Denial {
        name: "ac3_a_reason_outside_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| count above the maximum", " or BadAnnouncement"))),
        expect: &[("AC3", "outside the closed enumeration: BadAnnouncement")],
    },
    Denial {
        name: "ac3_fewer_than_six",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| duplicate type in one packet", "count under BadLength", "count it"))),
        expect: &[("AC3", "only 5 invalid form(s)")],
    },
    Denial {
        name: "ac3_headerless_validity_run_is_dropped",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\|---\|---\|$"))),
        expect: &[("AC3", "no invalid form listed for")],
    },
    Denial {
        name: "ac4_c43_not_restated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "C-43", "rule forty-three"))),
        expect: &[("AC4", "C-43 is not restated")],
    },
    Denial {
        name: "ac4_c43_no_arithmetic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"\b32\b", "thirty-two"))),
        expect: &[("AC4", "does not show the arithmetic")],
    },
    Denial {
        name: "ac4_c43_not_a_sender_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)sender", "source"))),
        expect: &[("AC4", "C-43 restatement is not phrased as a sender rule")],
    },
    Denial {
        name: "ac4_c44_not_a_sender_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)sender", "source"))),
        expect: &[("AC4", "C-44 restatement is not phrased as a sender rule")],
    },
    Denial {
        name: "ac4_c44_not_restated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "C-44", "rule forty-four"))),
        expect: &[("AC4", "C-44 is not restated")],
    },
    Denial {
        name: "ac4_c44_no_member_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)member", "output"))),
        expect: &[("AC4", "every-member rule")],
    },
    Denial {
        name: "ac4_c44_no_duration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, r"(?i)until", "till"), r"(?i)passes", "goes by"))),
        expect: &[("AC4", "does not state the duration")],
    },
    Denial {
        name: "ac5_realignment_open_item_without_005",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "005", "006"))),
        expect: &[("AC5", "proposed re-alignment payload is not recorded")],
    },
    Denial {
        name: "ac5_transport_open_item_without_005",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "005", "006"))),
        expect: &[("AC5", "proposed transport reference change payload is not recorded")],
    },
    Denial {
        name: "ac5_no_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)falsif\w*", "test"))),
        expect: &[("AC5", "state no falsifier")],
    },
    Denial {
        name: "ac5_falsifier_not_the_tickets",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, r"(?i)payload\s+alone", "payload by itself"), r"(?i)cannot\s+act", "may not act"))),
        expect: &[("AC5", "falsifier is not the ticket's")],
    },
    Denial {
        name: "ac5_epoch_switch_session_unstated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "C-19", "C-91"))),
        expect: &[("AC5", "epoch-switch payload does not state")],
    },
    Denial {
        name: "ac6_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC6", "does not exist")],
    },
    Denial {
        name: "ac6_wire_toml_unreadable",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("note = \"\"\"x\"\"\"\n{t}")),
        expect: &[("AC6", "could not be read by this check")],
    },
    Denial {
        name: "ac6_no_announcement_fields",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| re_all(&replace_all(t, "name = \"media packet\"", "name = \"media frame\""), r#"(?m)^name = "([^"]*)announcement"$"#, "name = \"${1}notice\"")),
        expect: &[("AC6", "declares no announcement fields")],
    },
    Denial {
        name: "ac6_no_type_field",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| re_all(t, r#"(?m)^name = "([^"]*)type([^"]*)"$"#, "name = \"${1}kind${2}\"")),
        expect: &[("AC6", "have no type field")],
    },
    Denial {
        name: "ac6_position_not_u64",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "name = \"position\"\noffset = 16\nwidth = 64\ntype = \"u64\"", "name = \"position\"\noffset = 16\nwidth = 64\ntype = \"i64\"")),
        expect: &[("AC6", "not a 64-bit u64")],
    },
    Denial {
        name: "ac6_count_without_a_maximum",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "count = \"announcement_count\"\nmax_count = 4\n", "count = \"announcement_count\"\n")),
        expect: &[("AC6", "no fixed maximum")],
    },
    Denial {
        name: "ac6_count_maximum_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "count = \"announcement_count\"\nmax_count = 4\n", "count = \"announcement_count\"\nmax_count = 5\n")),
        expect: &[("AC6", "count field's maximum (5) disagrees")],
    },
    Denial {
        name: "ac6_no_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, r#"section_of = "media packet"
rejection_reasons = \["BadLength", "ReservedNonZero"\]
"#, "section_of = \"media packet\"\n")),
        expect: &[("AC6", "no rejection_reasons declared")],
    },
    Denial {
        name: "ac6_bad_length_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, r#"section_of = "media packet"
rejection_reasons = \["BadLength", "ReservedNonZero"\]"#, "section_of = \"media packet\"\nrejection_reasons = [\"ReservedNonZero\"]")),
        expect: &[("AC6", "do not include BadLength")],
    },
    Denial {
        name: "ac6_a_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, r#"section_of = "media packet"
rejection_reasons = \["BadLength", "ReservedNonZero"\]"#, "section_of = \"media packet\"\nrejection_reasons = [\"BadLength\", \"BadIndex\"]")),
        expect: &[("AC6", "rejection reason 'BadIndex'")],
    },
    Denial {
        name: "ac6_max_size_not_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "max_size = 12\nsection_of", "max_size = \"12\"\nsection_of")),
        expect: &[("AC6", "max_size is not an integer")],
    },
    Denial {
        name: "ac6_media_max_size_not_past_38",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = 38"))),
        expect: &[("AC6", "does not grow past the 38-byte")],
    },
    Denial {
        name: "defect_a_field_table_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| \+0 \| 8 \| u64 \| position: the stream's last position", r"\(Validity rules\) \|$", "(Validity rules) and at least one lead ahead |"))),
        expect: &[("defect", "a field-table cell makes the lead a validity condition")],
    },
    Denial {
        name: "defect_a_validity_row_rejects_inside_the_lead",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| count above the maximum", "\n| a position inside the lead | drop and count under BadLength |"))),
        expect: &[("defect", "rejects an announcement for sitting inside the lead")],
    },
    Denial {
        name: "defect_a_table_row_invalidates_for_the_lead",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| count above the maximum", "\n| a late notice | drop it when the position lies within 64 slices |"))),
        expect: &[("defect", "a table row rejects or invalidates")],
    },
    Denial {
        name: "defect_a_prose_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^Every malformed announcement is dropped", " An announcement within the lead is dropped."))),
        expect: &[("defect", "prose makes the lead a validity condition")],
    },
    Denial {
        name: "defect_a_list_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^Every malformed announcement is dropped", "\n\n- an announcement closer than a lead is invalid"))),
        expect: &[("defect", "a list item makes the lead")],
    },
    Denial {
        name: "defect_a_wire_toml_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| re_all(t, r#"(?m)^valid = "any 64-bit value; a position at or before the packet's slice position is dropped.*$"#, "valid = \"a position within the lead is dropped under BadLength\"")),
        expect: &[("defect", "wire.toml field (valid) makes the lead")],
    },
    Denial {
        name: "defect_no_reissue_valid_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "A re-issue inside the lead is valid", "A re-issue is fine"), "less than 64 slices ahead is accepted", "nearer is accepted"))),
        expect: &[("defect", "no statement that a re-issue (C-44) inside the lead is valid")],
    },
    Denial {
        name: "defect_lead_binds_every_issue",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\*\*Sender obligations\.\*\*", " The lead binds every issue of a change."))),
        expect: &[("defect", "bind every issue")],
    },
    Denial {
        name: "defect_a_receiver_polices_the_lead",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\*\*Sender obligations\.\*\*", " A receiver checks the lead."))),
        expect: &[("defect", "a receiver is stated to police")],
    },
    Denial {
        name: "defect_r6_a_slice_at_the_named_position",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\*\*Sender obligations\.\*\*", " The last re-issue rides in the slice at the named position."))),
        expect: &[("defect R6", "places an announcement in the slice at the named position")],
    },
    Denial {
        name: "defect_r6_no_stop_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "The named position passes once the sender's slice position reaches it, so no announcement rides in a slice whose slice position is at or after the position it names", "The named position is the end"))),
        expect: &[("defect R6", "does not state the stop rule")],
    },
    Denial {
        name: "defect_first_issue_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)\bfirst\b", "initial"))),
        expect: &[("defect", "not stated as the sender's obligation on the FIRST issue")],
    },
    Denial {
        name: "dod_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| strip_tags(t, r"^\*\*Maximums\.\*\*"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    Denial {
        name: "py_superscript_after_a_type_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, r"^\| 0 \| end-of-stream \|", r"^\| 0 \|", "| 0² |"), r"^\| \+0 \| 8 \| u64 \| position: the stream's last position", "end-of-stream boundary", "final boundary"))),
        expect: &[("AC1", "end-of-stream announcement has no stated TYPE CODE")],
    },
    Denial {
        name: "py_a_combining_mark_is_not_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| count above the maximum", " or FooBar\u{301}"))),
        expect: &[("AC3", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_superscript_is_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| count above the maximum", "count under BadLength", "count under BadLength²"))),
        expect: &[("AC3", "must be rejected under BadLength")],
    },
    Denial {
        name: "py_unit_separator_is_whitespace_to_lstrip",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| announcement count of 1 or more", "\n\u{1f}| truncated | drop and count under FooBar |"))),
        expect: &[("AC3", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_line_separator_starts_a_list_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, r"^\| count above the maximum"), r"^Every malformed announcement is dropped", "\u{2028}- count above the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_form_feed_starts_a_list_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, r"^\| count above the maximum"), r"^Every malformed announcement is dropped", "\u{c}- count above the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_next_line_starts_a_list_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, r"^\| count above the maximum"), r"^Every malformed announcement is dropped", "\u{85}- count above the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_bare_carriage_return_starts_a_list_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&delete_row(t, r"^\| count above the maximum"), r"^Every malformed announcement is dropped", "\r- count above the maximum: drop and count under BadLength"))),
        expect: &[],
    },
    Denial {
        name: "py_dotless_i_matches_i_under_re_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "Open item 1 — the re-alignment payload", "Open item 1 — the re-alıgnment payload"))),
        expect: &[],
    },
    Denial {
        name: "py_int_is_unbounded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "at most 4 announcements per packet", "at most 99999999999999999999 announcements per packet"))),
        expect: &[("AC2", "not self-consistent")],
    },
    Denial {
        name: "py_separator_without_dash_is_no_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\|---\|---\|---\|---\|---\|", r"(?s).*", "|===|===|===|===|===|"))),
        expect: &[("AC1", "no container field table")],
    },
    Denial {
        name: "toml_float_max_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "count = \"announcement_count\"\nmax_count = 4\n", "count = \"announcement_count\"\nmax_count = 4.0\n")),
        expect: &[("AC6", "no fixed maximum")],
    },
    Denial {
        name: "toml_hex_max_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "count = \"announcement_count\"\nmax_count = 4\n", "count = \"announcement_count\"\nmax_count = 0x4\n")),
        expect: &[],
    },
    Denial {
        name: "toml_duplicate_key_last_wins",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "max_size = 12\nsection_of", "max_size = 12\nmax_size = \"12\"\nsection_of")),
        expect: &[("AC6", "max_size is not an integer")],
    },
    Denial {
        name: "toml_message_a_number_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("message = 5\n{}", t.replace("[[message]]", "[[messages]]").replace("[[message.", "[[messages."))),
        expect: &[("CRASH", "raised an exception")],
    },
    Denial {
        name: "toml_unhashable_reason_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, r#"section_of = "media packet"
rejection_reasons = \["BadLength", "ReservedNonZero"\]"#, "section_of = \"media packet\"\nrejection_reasons = [[\"BadLength\"], \"ReservedNonZero\"]")),
        expect: &[("CRASH", "raised an exception")],
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
