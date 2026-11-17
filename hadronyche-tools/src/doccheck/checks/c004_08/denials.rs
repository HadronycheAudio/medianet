//! The denial list of 004-08: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_08.rs` (one test each) and by `doccheck-equiv 004-08`
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

/// `edit` applied to the Slice header section of wire.md only.
fn sec(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Slice header$", edit)
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

const POSITION_ROW: &str = r"^\| 16 \| 8 \| u64 \| position";
const FRAMES_ROW: &str = r"^\| 24 \| 2 \| u16 \| frames";
const INDEX_ROW: &str = r"^\| 26 \| 2 \| u16 \| packet index";
const COUNT_ROW: &str = r"^\| 28 \| 2 \| u16 \| packet count";
const SEND_ROW: &str = r"^\| 30 \| 8 \| u64 \| send timestamp";
const SEND_PARA: &str = r"^The send timestamp is 64 bits";
const SHORT_ROW: &str = r"^\| too short for the slice header";
const INDEX_FORM: &str = r"^\| packet index ≥ packet count";
const ZERO_FORM: &str = r"^\| packet count = 0";
const RECEIVE_PARA: &str = r"^The slice header carries no receive timestamp";
const VALIDITY_LEAD: &str = r"^Every malformed slice header is dropped";

/// The section with no unit statement left (ns, µs, ms, samples); the wrap
/// keeps a duration in years.
fn no_unit(t: &str) -> String {
    let t = re_all(t, r"(?i)nanoseconds?", "ticks");
    let t = re_all(&t, r"\bns\b", "tk");
    let t = re_all(&t, r"1 ms\b", "1 tick");
    let t = re_all(&t, r"(?i)samples at", "samplings at");
    let t = re_all(&t, r"(?i)sample period", "sampling period");
    re_all(&t, r"(?i)one sample\b", "one sampling")
}

/// A validity bullet naming UnadmittedSource (no negation word near it).
fn unadmitted_bullet(t: &str) -> String {
    append(t, VALIDITY_LEAD, "\n\n- a datagram whose source lacks admission counts under UnadmittedSource [decision]")
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    // setup
    Denial {
        name: "setup_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("WS5", "does not exist")],
    },
    Denial {
        name: "setup_no_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Slice header$", "Slice prefix")),
        expect: &[("AC1", "no 'Slice header' section")],
    },
    Denial {
        name: "setup_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |_| "## Slice header\n\nTo be filled by 004-08.\n".into())),
        expect: &[("DoD", "still the 004-08 placeholder")],
    },
    Denial {
        name: "setup_no_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\| Offset \(bytes\) \|", "| Position (bytes) |"))),
        expect: &[("AC2", "no field table")],
    },
    // AC1
    Denial {
        name: "ac1_no_send_timestamp_region",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)send[\s_-]*timestamp", "emit stamp"))),
        expect: &[("AC1", "no send-timestamp decision")],
    },
    Denial {
        name: "ac1_no_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, r"(\d)-bit\b", "$1-wide"), r"(\d) bits\b", "$1 binary digits"))),
        expect: &[("AC1", "WIDTH is not stated in bits")],
    },
    Denial {
        name: "ac1_unit_too_coarse",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| {
        let t = no_unit(t);
        line(&t, SEND_PARA, r"The unit is ticks, one tick 1 tk", "The unit is milliseconds, one tick 1 ms")
    })),
        expect: &[("AC1", "cannot resolve one sample period")],
    },
    Denial {
        name: "ac1_no_clock",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)\bclock\b", "timer"))),
        expect: &[("AC1", "CLOCK SOURCE is not stated")],
    },
    Denial {
        name: "ac1_no_wrap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "wrap period", "roll-over"))),
        expect: &[("AC1", "WRAP PERIOD is not stated")],
    },
    Denial {
        name: "ac1_wrap_inside_a_reassembly_span",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, SEND_PARA, r"2\^64 ns, about 584 years", "2^19 ns"))),
        expect: &[("AC1", "wraps within one reassembly span")],
    },
    Denial {
        name: "ac1_no_reassembly_span",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)reassembl", "rebuild"))),
        expect: &[("AC1", "does not name the reassembly span")],
    },
    Denial {
        name: "ac1_no_alignment_span",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)align", "match"))),
        expect: &[("AC1", "does not name the alignment span")],
    },
    Denial {
        name: "ac1_no_resolution_arithmetic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "96 000", "96k"), "10 416", "ten thousand"))),
        expect: &[("AC1", "resolution arithmetic")],
    },
    Denial {
        name: "ac1_width_not_decision_tagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\[decision\]", "[code]"))),
        expect: &[("AC1", "width statement carries no [decision]")],
    },
    Denial {
        name: "ac1_unit_not_decision_tagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\[decision\]", "[code]"))),
        expect: &[("AC1", "unit statement carries no [decision]")],
    },
    Denial {
        name: "ac1_clock_not_decision_tagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\[decision\]", "[code]"))),
        expect: &[("AC1", "clock source statement carries no [decision]")],
    },
    Denial {
        name: "ac1_wrap_not_decision_tagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\[decision\]", "[code]"))),
        expect: &[("AC1", "wrap period statement carries no [decision]")],
    },
    // AC2
    Denial {
        name: "ac2_no_packet_index_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, INDEX_ROW, r"(?s).*", "| 26 | 2 | u16 | slot: this packet's slot within its slice, 0-based | 0 to packet count − 1 (slot < packet count) |"))),
        expect: &[("AC2", "has no packet index field")],
    },
    Denial {
        name: "ac2_position_not_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, POSITION_ROW, r"^\| 16 \| 8", "| 16 | 4"))),
        expect: &[("AC2", "position: the table gives 32 bits")],
    },
    Denial {
        name: "ac2_not_from_byte_16",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, POSITION_ROW, r"^\| 16", "| 15"))),
        expect: &[("AC2", "starts at byte 15")],
    },
    Denial {
        name: "ac2_stated_bits_disagree",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FRAMES_ROW, r"^\| 24 \| 2 \| u16 \| frames: how many frames of media time the slice spans", "| 24 | 2 bits | u16 | frames: how many frames of media time the slice spans, bits 3..0"))),
        expect: &[("AC2", "states bits 3..0 (4 bits) but its width is 2")],
    },
    Denial {
        name: "ac2_not_whole_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FRAMES_ROW, r"^\| 24 \| 2 \|", "| 24 | 12 bits |"))),
        expect: &[("AC2", "a field of 12 bits")],
    },
    Denial {
        name: "ac2_a_gap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, INDEX_ROW))),
        expect: &[("AC2", "undeclared gap of 16 bits")],
    },
    Denial {
        name: "ac2_an_overlap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, COUNT_ROW, r"^\| 28", "| 27"))),
        expect: &[("AC2", "overlapping fields at bit 216")],
    },
    Denial {
        name: "ac2_ends_mid_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, SEND_ROW, r"^\| 30 \| 8 \|", "| 30 | 60 bits |"))),
        expect: &[("AC2", "not a whole byte boundary")],
    },
    Denial {
        name: "ac2_frames_no_lower_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FRAMES_ROW, "1 to 65 535", "one to 65 535"))),
        expect: &[("AC2", "does not state the lower bound 1")],
    },
    Denial {
        name: "ac2_frames_no_upper_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FRAMES_ROW, r"\| 1 to 65 535 \[decision\]; 48 in the default regime \(C-02\) \|", "| at least 1 [decision] |"))),
        expect: &[("AC2", "states no upper bound")],
    },
    Denial {
        name: "ac2_index_not_bounded_by_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, INDEX_ROW, r"\| 0 to packet count − 1 \(packet index < packet count\) \|", "| 0 to 65 535 |"))),
        expect: &[("AC2", "does not bound it by the packet count")],
    },
    Denial {
        name: "ac2_count_default_not_one",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, COUNT_ROW, r"\| 1 in the default regime", "| one in the usual regime"))),
        expect: &[("AC2", "default regime's value 1")],
    },
    // AC3
    Denial {
        name: "ac3_no_multi_packet_regime",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(&re_all(t, r"packet count\s*>\s*1", "packet count above one"), r"(?i)multi-?packet", "split"), r"(?i)more than one packet", "several"))),
        expect: &[("AC3", "multi-packet regime is not defined")],
    },
    Denial {
        name: "ac3_hand_off_misses_frames_per_slice",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(t, r"(?i)frames? per slice", "slice frames"), r"(?i)slice length", "slice duration"))),
        expect: &[("AC3", "no hand-off to 004-25")],
    },
    // WS5
    Denial {
        name: "ws5_no_note",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)receive[\s_-]*timestamp", "arrival stamp"))),
        expect: &[("WS5", "no receive-timestamp note")],
    },
    Denial {
        name: "ws5_a_source_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RECEIVE_PARA, "mac-hardware", "nic"))),
        expect: &[("WS5", "does not name the C-131 source \"mac-hardware\"")],
    },
    Denial {
        name: "ws5_no_c129",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RECEIVE_PARA, "C-129", "C-130"))),
        expect: &[("WS5", "C-129's capability states")],
    },
    Denial {
        name: "ws5_no_regime_stamp_pointer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RECEIVE_PARA, "004-25's receive", "the receive"))),
        expect: &[("WS5", "does not point at the regime stamp")],
    },
    Denial {
        name: "ws5_not_said_outside_the_slice_header",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RECEIVE_PARA, "not in the slice header", "beside the slice header"))),
        expect: &[("WS5", "NOT in the slice header")],
    },
    // AC4
    Denial {
        name: "ac4_a_form_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, ZERO_FORM))),
        expect: &[("AC4", "no invalid form listed for: packet count = 0")],
    },
    Denial {
        name: "ac4_index_form_not_under_bad_packet_index",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, INDEX_FORM, "count under BadPacketIndex", "count under BadLength"), INDEX_FORM, "BadPacketIndex's own", "its own"))),
        expect: &[("AC4", "must be rejected under BadPacketIndex")],
    },
    Denial {
        name: "ac4_a_form_without_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, ZERO_FORM, "count under BadLength", "count it"))),
        expect: &[("AC4", "carries no member of the closed enumeration")],
    },
    Denial {
        name: "ac4_a_reason_outside_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, SHORT_ROW, " or BadSlice"))),
        expect: &[("AC4", "outside the closed enumeration: BadSlice")],
    },
    Denial {
        name: "ac4_fewer_than_five",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, ZERO_FORM, "count under BadLength", "count it"))),
        expect: &[("AC4", "only 3 invalid form(s)")],
    },
    Denial {
        name: "ac4_headerless_validity_run_is_dropped",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\|---\|---\|$"))),
        expect: &[("AC4", "no invalid form listed for: packet index")],
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
        name: "ac5_no_carrying_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "name = \"media packet\"", "name = \"media frame\"")),
        expect: &[("AC5", "has no [[message]] carrying the slice header")],
    },
    Denial {
        name: "ac5_no_slice_fields",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "name = \"media packet\"", "name = \"media frame\"")),
        expect: &[("AC5", "declares no slice-header fields")],
    },
    Denial {
        name: "ac5_no_field_agrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"frames\"\noffset = 192\nwidth = 16", "name = \"fcount\"\noffset = 192\nwidth = 8"))),
        expect: &[("AC5", "no [[message.field]] row agrees")],
    },
    Denial {
        name: "ac5_width_not_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"position\"\noffset = 128\nwidth = 64", "name = \"position\"\noffset = 128\nwidth = \"64\""))),
        expect: &[("AC5", "offset/width are not integers")],
    },
    Denial {
        name: "ac5_offset_mismatch",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"frames\"\noffset = 192", "name = \"frames\"\noffset = 200"))),
        expect: &[("AC5", "toml offset 200 bits vs wire.md offset 24 bytes")],
    },
    Denial {
        name: "ac5_width_mismatch",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"frames\"\noffset = 192\nwidth = 16", "name = \"frames\"\noffset = 192\nwidth = 8"))),
        expect: &[("AC5", "toml width 8 bits vs wire.md width 16 bits")],
    },
    Denial {
        name: "ac5_max_size_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = \"96\""))),
        expect: &[("AC5", "max_size is not stated as an integer")],
    },
    Denial {
        name: "ac5_max_size_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = 96.0"))),
        expect: &[("AC5", "max_size is not stated as an integer")],
    },
    Denial {
        name: "ac5_max_size_too_small",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = 30"))),
        expect: &[("AC5", "max_size 30 does not cover")],
    },
    Denial {
        name: "ac5_no_bad_packet_index",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, r#""BadPacketIndex", "#, ""))),
        expect: &[("AC5", "do not include BadPacketIndex")],
    },
    // DoD
    Denial {
        name: "dod_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| strip_tags(t, r"^The default regime is one packet per slice"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    Denial {
        name: "dod_regime_stamp_line_not_covered",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(&re_all(&re_all(t, r"(?i)complet\w*", "final"), r"(?i)documented", "written"), r"(?i)hand(ed|over|ing)|suppli", "give"))),
        expect: &[("DoD", "regime-stamp-fields-touched line")],
    },
    // Python semantics
    Denial {
        name: "py_superscript_is_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(&line(t, INDEX_FORM, "count under BadPacketIndex", "count under BadPacketIndex²"), INDEX_FORM, "BadPacketIndex's own", "BadPacketIndex²'s own"))),
        expect: &[("AC4", "must be rejected under BadPacketIndex")],
    },
    Denial {
        name: "py_a_combining_mark_is_not_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, SHORT_ROW, " or FooBar\u{301}"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_unit_separator_is_whitespace_to_lstrip",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| packet count inconsistent across", "\n\u{1f}| truncated | drop and count under FooBar |"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_line_separator_starts_a_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, VALIDITY_LEAD, "\u{2028}- a truncated header counts under FooBar"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_form_feed_starts_a_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, VALIDITY_LEAD, "\u{c}- a truncated header counts under FooBar"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_next_line_starts_a_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, VALIDITY_LEAD, "\u{85}- a truncated header counts under FooBar"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_bare_carriage_return_is_a_newline",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, VALIDITY_LEAD, "\r- a truncated header counts under FooBar"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_dotless_i_matches_i_under_re_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RECEIVE_PARA, "or interrupt", "or ınterrupt"))),
        expect: &[],
    },
    Denial {
        name: "py_dotted_capital_i_matches_i_under_re_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, RECEIVE_PARA, "or interrupt", "or İNTERRUPT"))),
        expect: &[],
    },
    Denial {
        name: "py_kelvin_sign_is_a_letter_to_the_unit_guard",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&no_unit(t), SEND_PARA, " The unit is \u{212a}ns."))),
        expect: &[("AC1", "UNIT is not stated")],
    },
    Denial {
        name: "py_greek_mu_folds_onto_micro",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(&no_unit(t), SEND_PARA, " The unit is \u{3bc}s [decision]."))),
        expect: &[],
    },
    Denial {
        name: "py_int_is_unbounded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, POSITION_ROW, r"^\| 16", "| 99999999999999999999999"))),
        expect: &[("AC2", "undeclared gap of 799999999999999999999688 bits")],
    },
    Denial {
        name: "py_int_unbounded_upper_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FRAMES_ROW, "1 to 65 535", "1 to 99999999999999999999999"))),
        expect: &[],
    },
    Denial {
        name: "py_power_too_large_for_a_float_crashes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, SEND_PARA, r"2\^64 ns", "2^1024 ns"))),
        expect: &[("CRASH", "raised an exception")],
    },
    Denial {
        name: "py_frames_too_large_for_a_float_crashes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^Every media packet carries a slice header", "^", &format!("A slice of {} frames is absurd. ", "9".repeat(400))))),
        expect: &[("CRASH", "raised an exception")],
    },
    Denial {
        name: "py_separator_without_dash_is_no_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\|---\|---\|---\|---\|---\|", "|===|===|===|===|===|"))),
        expect: &[("AC2", "no field table")],
    },
    Denial {
        name: "toml_hex_offset",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"position\"\noffset = 128", "name = \"position\"\noffset = 0x80"))),
        expect: &[],
    },
    Denial {
        name: "toml_duplicate_key_last_wins",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96\n", "max_size = 96\nmax_size = 30\n"))),
        expect: &[("AC5", "max_size 30 does not cover")],
    },
    Denial {
        name: "toml_reasons_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, r"(?m)^rejection_reasons = .*$", "rejection_reasons = \"BadPacketIndex\""))),
        expect: &[("AC5", "do not include BadPacketIndex")],
    },
    Denial {
        name: "toml_message_a_number_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("message = 5\n{t}").replace("[[message]]", "[[messages]]")),
        expect: &[("CRASH", "raised an exception")],
    },

    Denial {
        name: "ac1_no_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, no_unit)),
        expect: &[("AC1", "UNIT is not stated")],
    },
    Denial {
        name: "ac5_sub_byte_offset_outside_its_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FRAMES_ROW, r"^\| 24 \| 2 \|", "| 25 | 4 bits |"))),
        expect: &[("AC5", "offset 192 is not inside byte 25")],
    },
    Denial {
        name: "ten_found_names_shrink_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(&fewer_names(t, &["BadChecksum", "OversizeControl", "UnadmittedSource"]), unadmitted_bullet)),
        expect: &[("AC4", "outside the closed enumeration: UnadmittedSource")],
    },
    Denial {
        name: "nine_found_names_keep_the_thirteen",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(&fewer_names(t, &["BadChecksum", "OversizeControl", "UnadmittedSource", "UnknownStream"]), unadmitted_bullet)),
        expect: &[],
    },
];
