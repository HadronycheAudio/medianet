//! The denial list of 004-06: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_06.rs` (one test each) and by `doccheck-equiv 004-06`
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

/// `edit` applied to the Media packet header section of wire.md only.
fn sec(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Media packet header$", edit)
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
#[allow(dead_code)]
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

const MAGIC_ROW: &str = r"^\| 0 \| 2 \| u16 \| Magic";
const VERSION_ROW: &str = r"^\| 2 \| 2 \| u16 \| Protocol version";
const DOMAIN_ROW: &str = r"^\| 4 \| 2 \| u16 \| Domain id";
const SESSION_ROW: &str = r"^\| 8 \| 4 \| u32 \| Session";
const FLAGS_ROW: &str = r"^\| 14 \| 1 \| u8 \| Flags";
const COPY_ROW: &str = r"^\| 15 \| 2 bits \| uint \| Copy index";
const RESERVED_ROW: &str = r"^\| 15 \| 6 bits \| uint \| Reserved";
const TOOSHORT_ROW: &str = r"^\| shorter than the 16-byte header";
const MAGIC_FORM: &str = r"^\| bad magic";
const VALIDITY_LEAD: &str = r"^Every malformed header is dropped";

/// The denials.
pub const DENIALS: &[Denial] = &[
    // setup
    Denial {
        name: "setup_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist")],
    },
    Denial {
        name: "setup_wire_md_missing_dod",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("DoD", "wire.md does not exist")],
    },
    Denial {
        name: "setup_no_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Media packet header$", "Media header")),
        expect: &[("AC6", "no 'Media packet header' section")],
    },
    Denial {
        name: "setup_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |_| "## Media packet header\n\nTo be filled by 004-06.\n".into())),
        expect: &[("DF", "still the 004-06 placeholder")],
    },
    Denial {
        name: "setup_empty_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |_| "## Media packet header\n".into())),
        expect: &[("AC4", "still the 004-06 placeholder")],
    },
    Denial {
        name: "setup_no_field_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\| Offset \(bytes\) \|", "| Position (bytes) |"))),
        expect: &[("AC3", "no field table")],
    },
    Denial {
        name: "setup_a_row_without_a_width_drops_the_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| 12 \| 2 \| u16", r"^\| 12 \| 2", "| 12 | two"))),
        expect: &[("AC1", "no field table")],
    },
    // AC1
    Denial {
        name: "ac1_no_copy_index_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "Copy index", "Copy slot"))),
        expect: &[("AC1", "has no copy index field")],
    },
    Denial {
        name: "ac1_domain_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, DOMAIN_ROW, r"^\| 4 \| 2", "| 4 | 4"))),
        expect: &[("AC1", "domain: the table gives 32 bits")],
    },
    Denial {
        name: "ac1_magic_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| {
        let t = line(t, MAGIC_ROW, "Magic 0x4d4e", "Version mark 0x4d4e");
        line(&t, VERSION_ROW, "Protocol version", "Protocol magic")
    })),
        expect: &[("AC1", "the magic sits at offset 2")],
    },
    Denial {
        name: "ac1_version_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| {
        let t = line(t, MAGIC_ROW, "Magic 0x4d4e", "Version mark 0x4d4e");
        line(&t, VERSION_ROW, "Protocol version", "Protocol magic")
    })),
        expect: &[("AC1", "the version sits at offset 0")],
    },
    Denial {
        name: "ac1_flags_below_a_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, FLAGS_ROW, r"^\| 14 \| 1 \|", "| 14 | 1 bit |"))),
        expect: &[("AC1", "the flags field is 1 bits")],
    },
    Denial {
        name: "ac1_no_magic_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "0x4d4e", "0x4d4f"))),
        expect: &[("AC1", "magic value 0x4d4e")],
    },
    Denial {
        name: "ac1_no_version_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "0x0100", "0x0101"), r"1\.0", "one"))),
        expect: &[("AC1", "version value 0x0100")],
    },
    Denial {
        name: "ac1_gap_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\bgap\b", "hole"))),
        expect: &[("AC1", "does not name the gap bit")],
    },
    Denial {
        name: "ac1_degraded_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\bdegraded\b", "impaired"))),
        expect: &[("AC1", "does not name the degraded bit")],
    },
    Denial {
        name: "ac1_versioning_magic",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "(?m)^magic = 0x4d4e$", "magic = 0x4d4f")),
        expect: &[("AC1", "[versioning] magic is 19791")],
    },
    Denial {
        name: "ac1_versioning_version",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "(?m)^version = 0x0100$", "version = 0x0101")),
        expect: &[("AC1", "[versioning] version is 257")],
    },
    // AC2
    Denial {
        name: "ac2_module_id_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "module id", "module key"))),
        expect: &[("AC2", "carries no module id")],
    },
    Denial {
        name: "ac2_destination_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "destination", "target"))),
        expect: &[("AC2", "carries no destination address")],
    },
    Denial {
        name: "ac2_acknowledgement_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, r"acknowledg\w*", "receipt"), "per-receiver", "receiver-wise"))),
        expect: &[("AC2", "carries no per-receiver acknowledgement")],
    },
    Denial {
        name: "ac2_not_field_by_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "(?i)field by field", "In short"), "field of the table", "column"))),
        expect: &[("AC2", "not field by field")],
    },
    Denial {
        name: "ac2_a_destination_specific_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, DOMAIN_ROW, "unconfigured domain", "unconfigured domain; the destination module"))),
        expect: &[("AC2", "destination-specific field")],
    },
    // AC3
    Denial {
        name: "ac3_stated_bits_disagree_with_the_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, COPY_ROW, r"bits 1\.\.0", "bits 2..0"))),
        expect: &[("AC3", "states bits 2..0 (3 bits) but its width is 2")],
    },
    Denial {
        name: "ac3_bits_not_whole_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, DOMAIN_ROW, r"^\| 4 \| 2", "| 4 | 12 bits"))),
        expect: &[("AC3", "a field of 12 bits")],
    },
    Denial {
        name: "ac3_a_gap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\| 12 \| 2 \| u16 \| Timeline epoch"))),
        expect: &[("AC3", "undeclared gap of 16 bits")],
    },
    Denial {
        name: "ac3_an_overlap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, SESSION_ROW, r"^\| 8", "| 6"))),
        expect: &[("AC3", "overlapping fields at bit 48")],
    },
    Denial {
        name: "ac3_ends_mid_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, RESERVED_ROW))),
        expect: &[("AC3", "not a whole byte boundary")],
    },
    Denial {
        name: "ac3_size_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "16 bytes", "sixteen bytes"))),
        expect: &[("AC3", "never states that size in bytes")],
    },
    Denial {
        name: "ac3_no_size_at_all",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| {
        let t = line(&delete_row(t, RESERVED_ROW), COPY_ROW, r"bits 1\.\.0 of byte 15", "low end of byte 15");
        replace_all(&t, r"(?i)\bbytes\b", "octets")
    })),
        expect: &[("AC3", "states no header size in bytes")],
    },
    // AC4
    Denial {
        name: "ac4_a_form_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\| wrong epoch"))),
        expect: &[("AC4", "no invalid form listed for: wrong epoch")],
    },
    Denial {
        name: "ac4_a_form_without_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, MAGIC_FORM, "count under BadMagic", "count it"))),
        expect: &[("AC4", "carries no member of the closed enumeration")],
    },
    Denial {
        name: "ac4_version_mismatch_without_its_event",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, r"^\| version mismatch", r"VersionMismatch event", "VersionMismatch signal"))),
        expect: &[("AC4", "without its named event")],
    },
    Denial {
        name: "ac4_a_reason_outside_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, TOOSHORT_ROW, " or BadHeader"))),
        expect: &[("AC4", "outside the closed enumeration: BadHeader")],
    },
    Denial {
        name: "ac4_fewer_than_ten_forms_with_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| {
        let t = line(t, MAGIC_FORM, "count under BadMagic", "count it");
        line(&t, r"^\| wrong epoch", "count under WrongEpoch", "count it")
    })),
        expect: &[("AC4", "only 9 invalid form(s)")],
    },
    Denial {
        name: "ac4_headerless_validity_run_is_dropped",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| delete_row(t, r"^\|---\|---\|$"))),
        expect: &[("AC4", "no invalid form listed for: shorter than")],
    },
    // AC5
    Denial {
        name: "ac5_session_without_c19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, SESSION_ROW, r"\(C-19\)", "(C-91)"))),
        expect: &[("AC5", "does not cite C-19")],
    },
    Denial {
        name: "ac5_degraded_without_c122",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "C-122", "C-121"))),
        expect: &[("AC5", "does not cite C-122")],
    },
    // AC6
    Denial {
        name: "ac6_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC6", "does not exist")],
    },
    Denial {
        name: "ac6_wire_toml_unreadable_triple_quotes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("note = \"\"\"valid TOML\"\"\"\n{t}")),
        expect: &[("AC6", "could not be read by this check")],
    },
    Denial {
        name: "ac6_reject_enumeration_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "(?m)^name = \"OversizeControl\"$", "name = \"Oversize\"")),
        expect: &[("AC6", "[[reject]] enumeration in wire.toml disagrees")],
    },
    Denial {
        name: "ac6_no_media_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "name = \"media packet\"", "name = \"media frame\"")),
        expect: &[("AC6", "has no [[message]] for the media packet")],
    },
    Denial {
        name: "ac6_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "layer = \"media\"", "layer = \"control\""))),
        expect: &[("AC6", "layer is 'control'")],
    },
    Denial {
        name: "ac6_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "dscp = 46", "dscp = 34"))),
        expect: &[("AC6", "dscp is 34")],
    },
    Denial {
        name: "ac6_max_size_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = \"96\""))),
        expect: &[("AC6", "max_size is not stated as an integer")],
    },
    Denial {
        name: "ac6_max_size_below_the_header",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "max_size = 96", "max_size = 12"))),
        expect: &[("AC6", "max_size 12 is smaller than the header's 16 bytes")],
    },
    Denial {
        name: "ac6_no_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "(?m)^rejection_reasons = .*\n", ""))),
        expect: &[("AC6", "rejection_reasons is missing or empty")],
    },
    Denial {
        name: "ac6_a_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "\"BadPacketIndex\"", "\"BadIndex\""))),
        expect: &[("AC6", "rejection reason 'BadIndex'")],
    },
    Denial {
        name: "ac6_no_field_agrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "offset = 122\nwidth = 6", "offset = 122\nwidth = 5"))),
        expect: &[("AC6", "no [[message.field]] row agrees")],
    },
    Denial {
        name: "ac6_offset_not_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"magic\"\noffset = 0", "name = \"magic\"\noffset = \"0\""))),
        expect: &[("AC6", "offset/width are not integers")],
    },
    Denial {
        name: "ac6_sub_byte_offset_outside_its_byte",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "offset = 120", "offset = 128"))),
        expect: &[("AC6", "offset 128 is not inside byte 15")],
    },
    Denial {
        name: "ac6_offset_mismatch",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"domain\"\noffset = 32", "name = \"domain\"\noffset = 40"))),
        expect: &[("AC6", "toml offset 40 bits vs wire.md offset 4 bytes")],
    },
    Denial {
        name: "ac6_width_mismatch",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"domain\"\noffset = 32\nwidth = 16", "name = \"domain\"\noffset = 32\nwidth = 8"))),
        expect: &[("AC6", "toml width 8 bits vs wire.md width 16 bits")],
    },
    // DF
    Denial {
        name: "df_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "(?i)don't-fragment", "No-split"), r"\bDF\b", "NF"))),
        expect: &[("DF", "does not state the don't-fragment requirement")],
    },
    Denial {
        name: "df_no_ipv4",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "IPv4", "IP version 4"))),
        expect: &[("DF", "does not name IPv4")],
    },
    Denial {
        name: "df_not_a_sender_obligation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "(?i)sender", "emitter"))),
        expect: &[("DF", "sender obligation")],
    },
    Denial {
        name: "df_not_outside_the_header",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(&replace_all(t, "outside this header", "beside this header"), "header does not carry", "header omits"))),
        expect: &[("DF", "outside the header")],
    },
    Denial {
        name: "df_no_c100",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "C-100", "C-101"))),
        expect: &[("DF", "does not cite C-100")],
    },
    // DoD
    Denial {
        name: "dod_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| strip_tags(t, r"^Don't-fragment \(rule 11\)"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    Denial {
        name: "dod_a_table_after_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| strip_tags(t, r"^Bit 7 first"))),
        expect: &[("DoD", "carries no evidence tag ([code] [measured] [kernel-doc] [standard] [memory] [decision]): | Bit | Name | Meaning |")],
    },
    // Python semantics: \w, \b, \s, splitlines, universal newlines, re.I,
    // int(), the scripts' TOML reader, the separator rule, the 13 names.
    Denial {
        name: "py_superscript_is_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, MAGIC_FORM, "BadMagic", "BadMagic²"))),
        expect: &[("AC4", "carries no member of the closed enumeration")],
    },
    Denial {
        name: "py_a_combining_mark_is_not_a_word_character",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, TOOSHORT_ROW, " or FooBar\u{301}"))),
        expect: &[("AC4", "outside the closed enumeration: FooBar")],
    },
    Denial {
        name: "py_unit_separator_is_whitespace_to_lstrip",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| append(t, r"^\| invalid flags combination", "\n\u{1f}| truncated | drop and count under FooBar |"))),
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
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, DOMAIN_ROW, "unconfigured domain", "unconfigured domain; the destınation module"))),
        expect: &[("AC2", "destination-specific field")],
    },
    Denial {
        name: "py_dotted_capital_i_matches_i_under_re_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, DOMAIN_ROW, "unconfigured domain", "unconfigured domain; the DESTİNATION module"))),
        expect: &[("AC2", "destination-specific field")],
    },
    Denial {
        name: "py_int_is_unbounded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, MAGIC_ROW, r"^\| 0 \|", "| 2305843009213693952 |"))),
        expect: &[("AC1", "the magic sits at offset 2305843009213693952")],
    },
    Denial {
        name: "py_int_reads_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| line(t, VERSION_ROW, r"^\| 2 \|", "| \u{662} |"))),
        expect: &[],
    },
    Denial {
        name: "py_separator_with_colons_is_a_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\|---\|---\|---\|---\|---\|", "|:-:|---|--:|:--|---|"))),
        expect: &[],
    },
    Denial {
        name: "py_separator_without_dash_is_no_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, r"\|---\|---\|---\|---\|---\|", "|===|===|===|===|===|"))),
        expect: &[("AC1", "no field table")],
    },
    Denial {
        name: "toml_float_equals_int",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "dscp = 46", "dscp = 46.0"))),
        expect: &[],
    },
    Denial {
        name: "toml_hex_int",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "dscp = 46", "dscp = 0x2e"))),
        expect: &[],
    },
    Denial {
        name: "toml_bool_is_an_int",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"magic\"\noffset = 0", "name = \"magic\"\noffset = false"))),
        expect: &[],
    },
    Denial {
        name: "toml_bare_word_is_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "layer = \"media\"", "layer = media"))),
        expect: &[],
    },
    Denial {
        name: "toml_duplicate_key_last_wins",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "dscp = 46\n", "dscp = 46\ndscp = 34\n"))),
        expect: &[("AC6", "dscp is 34")],
    },
    Denial {
        name: "toml_duplicate_key_same_value",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "dscp = 46\n", "dscp = 46\ndscp = 46\n"))),
        expect: &[],
    },
    Denial {
        name: "toml_reasons_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "(?m)^rejection_reasons = .*$", "rejection_reasons = \"TooShort\""))),
        expect: &[("AC6", "rejection_reasons is missing or empty")],
    },
    Denial {
        name: "toml_unhashable_reason_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, r#"rejection_reasons = \["TooShort","#, r#"rejection_reasons = [["TooShort"],"#))),
        expect: &[("CRASH", "raised an exception")],
    },
    Denial {
        name: "toml_bad_unicode_escape_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| media(t, |t| replace_all(t, "name = \"magic\"\n", "name = \"magic\"\nnote = \"\\uZZZZ\"\n"))),
        expect: &[("CRASH", "raised an exception")],
    },

    Denial {
        name: "ten_found_names_shrink_the_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| fewer_names(t, &["BadChecksum", "BadPacketIndex", "OversizeControl"])),
        expect: &[("AC6", "[[reject]] enumeration in wire.toml disagrees")],
    },
    Denial {
        name: "nine_found_names_keep_the_thirteen",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| fewer_names(t, &["BadChecksum", "BadPacketIndex", "OversizeControl", "UnadmittedSource"])),
        expect: &[],
    },
];
