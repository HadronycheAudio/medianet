//! The denial list of 004-29: the real documents with one edit, and what
//! the port and the old script must report on them (their failed criteria,
//! exactly; `CRASH` where the script stops with a Python exception).
//! Consumed by `tests/doc/c004_29.rs` (one test each) and by `doccheck-equiv
//! 004-29`, which runs each against the old script too. An entry with no
//! expectation is one both must pass.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, delete_unit, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
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

const EXP_ROW: &str = r"^\| 48 \| 8 \| u64 \| Expiry time";
const TOOL_ROW: &str = r"^\| 40 \| 8 \| u64 \| Tool identity";
const ANN_WID: &str = r"^\| 32 \| 8 \| u64 \| Window identifier: drawn";
const CLO_WID: &str =
    r"^\| 32 \| 8 \| u64 \| Window identifier: the identifier of the window this close";
const ANN_HEAD: &str = "name = \"commissioning window announcement\"\nlayer = \"control\"\ndscp = 40\nmax_size = 56\nrejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]";
/// `from` replaced by `to` on every line that is not a table row.
fn prose_only(text: &str, from: &str, to: &str) -> String {
    text.split('\n')
        .map(|l| {
            if l.trim_start().starts_with('|') {
                l.to_string()
            } else {
                l.replace(from, to)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The announcement's expiry made a relative duration (the branch b of
/// Open item 11), with a tagged sentence that says so.
fn relative(t: &str) -> String {
    let t = every_in_line(
        t,
        EXP_ROW,
        "the absolute instant at which the window closes",
        "the relative duration after which the window closes",
    );
    replace_in_line(
        &t,
        r"^\*\*Expiry time\.\*\*",
        "The expiry is an absolute instant",
        "The expiry is a relative duration",
    )
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("DoD", "")],
    },
    Denial {
        name: "denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC5", "does not exist")],
    },
    Denial {
        name: "ac1_denial_the_control_bound_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 300")),
        expect: &[("AC1", "the control bound (C-17) as 1 252")],
    },
    Denial {
        name: "ac1_denial_no_control_bound_where_the_script_stops",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| C-17 \|")),
        expect: &[("AC1", "gives the control bound (C-17) as 1 252"), ("CRASH", "")],
    },
    Denial {
        name: "ac2_denial_the_control_dscp_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-13 \|", r"control CS5 \(40\)", "control CS5 (41)")),
        expect: &[("AC2", "control CS5 as 40"), ("AC5", "")],
    },
    Denial {
        name: "dod_denial_the_hello_header_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Common control header$", |t| delete_row(t, r"^\| 8 \| 8 \| u64 \| Message counter"))),
        expect: &[("AC1", ""), ("DoD", "the merged common control header of the Hello section is unchanged")],
    },
    Denial {
        name: "dod_denial_the_code_column_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Control message type codes$", |t| delete_row(t, r"^\| 52 to 255 \|"))),
        expect: &[("DoD", "keep their code column")],
    },
    Denial {
        name: "ac1_denial_no_code_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Control message type codes$", |t| replace_in_line(t, r"^\| 48 to 51 \|", "004-29", "004-99"))),
        expect: &[("AC1", "give this ticket a block")],
    },
    Denial {
        name: "ac2_denial_the_stamp_expiry_renamed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window section$", |t| replace_in_line(t, r"^\| 8 \| 8 \| u64 \| Expiry time", "Expiry time", "End time"))),
        expect: &[("AC2", "has one window identifier row and one expiry row")],
    },
    Denial {
        name: "dod_denial_the_enumeration_short",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 9 \| BadPacketIndex \|")),
        expect: &[("DoD", "keeps its 13 members")],
    },
    Denial {
        name: "dod_denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^The commissioning window is how", "^", "To be filled. "))),
        expect: &[("DoD", "placeholder replaced")],
    },
    Denial {
        name: "ac1_denial_no_announcement_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Window announcement$", "Window opening")),
        expect: &[("AC1", "names the announcement (found 0)"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_close_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Window close$", "Window shutdown")),
        expect: &[("AC1", "names the close (found 0)"), ("DoD", "")],
    },
    Denial {
        name: "ac3_denial_no_receiver_rules_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| rename_heading(t, "^Receiver rules$", "Behaviour"))),
        expect: &[("AC3", "holds the receiver rules (found 0)"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_announcement_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |")))),
        expect: &[("AC1", "the announcement has no field table")],
    },
    Denial {
        name: "ac1_denial_a_malformed_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, TOOL_ROW, r"^\| 40 \| 8 \|", "| 40 | eight |")))),
        expect: &[("AC1", "announcement: the Width cell"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_header_rows_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"^\| 7 \| 1 \|", "| 7 | 2 |")))),
        expect: &[("AC1", "starts with the seven rows of the common control header"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, TOOL_ROW, r"^\| 40 ", "| 41 ")))),
        expect: &[("AC1", "tiles from byte 0 with no gap or overlap"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_size_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "is 56 bytes", "is 56")))),
        expect: &[("AC1", "states its maximum size as its table sum, 56 bytes")],
    },
    Denial {
        name: "ac1_denial_a_code_outside_the_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 48", "48 only", "52 only")))),
        expect: &[("AC1", "the announcement has a type code")],
    },
    Denial {
        name: "ac1_denial_two_messages_one_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window close$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 49", "49 only", "48 only")))),
        expect: &[("AC1", "distinct type codes")],
    },
    Denial {
        name: "ac1_denial_no_window_identifier_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, ANN_WID, "Window identifier: drawn", "Window key: drawn")))),
        expect: &[("AC1", "the announcement has one window identifier row"), ("AC2", "")],
    },
    Denial {
        name: "ac1_denial_no_tool_identity_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| every_in_line(t, TOOL_ROW, "(?i)identit", "nam")))),
        expect: &[("AC1", "the announcing tool's identity")],
    },
    Denial {
        name: "ac1_denial_the_identifier_not_stable",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "(?i)unique", "distinct")))),
        expect: &[("AC1", "stable and unique")],
    },
    Denial {
        name: "ac2_denial_a_window_identifier_off_the_stamp_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, ANN_WID, r"\| 8 \| u64", "| 4 | u32")))),
        expect: &[("AC1", ""), ("AC2", "the announcement's window identifier is as wide as the merged stamp's"), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_an_expiry_off_the_stamp_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, EXP_ROW, r"\| 8 \| u64", "| 4 | u32")))),
        expect: &[("AC1", ""), ("AC2", "the announcement's expiry is as wide as the merged stamp's expiry"), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_no_clock",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "(?i)network time", "the reference")))),
        expect: &[("AC2", "the announcement names its clock")],
    },
    Denial {
        name: "ac2_denial_the_expiry_without_a_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| every_in_line(t, EXP_ROW, r"(?i)nanosecond|\bns\b|\bsecond", "tick")))),
        expect: &[("AC2", "the expiry row states its unit")],
    },
    Denial {
        name: "ac2_denial_the_expiry_in_two_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, EXP_ROW, "the absolute instant", "the absolute instant, a relative duration,")))),
        expect: &[("AC2", "states one form")],
    },
    Denial {
        name: "ac2_denial_the_form_not_in_a_tagged_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| prose_only(t, "absolute", "fixed")))),
        expect: &[("AC2", "a tagged sentence of the announcement states the expiry form")],
    },
    Denial {
        name: "ac2_denial_a_relative_expiry_without_its_origin",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(&relative(t), "(?i)counted from|counts from|from the issue time|from the instant", "x")))),
        expect: &[("AC2", "states the instant it counts from")],
    },
    Denial {
        name: "ac2_denial_a_relative_expiry_without_the_repeat_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(&relative(t), "(?i)same expiry|derives? the same", "x")))),
        expect: &[("AC2", "how a late or repeated announcement derives the same expiry")],
    },
    Denial {
        name: "ac2_denial_a_relative_expiry_copied_verbatim",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", relative))),
        expect: &[("AC2", "no longer says the stamped expiry is copied")],
    },
    Denial {
        name: "dod_denial_a_relative_expiry_without_the_status",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(&within_section(t, "^Window announcement$", relative), r"^\*\*Open item 11 — ", "Status, 2026-11-16", "Note")),
        expect: &[("AC2", ""), ("DoD", "carries a status sentence for the relative form")],
    },
    Denial {
        name: "ac1_denial_no_close_identifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window close$", |t| replace_in_line(t, CLO_WID, "Window identifier: the identifier", "Window key: the key")))),
        expect: &[("AC1", "the close names the window it closes"), ("AC2", "")],
    },
    Denial {
        name: "ac2_denial_the_close_identifier_off_the_stamp_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window close$", |t| replace_in_line(t, CLO_WID, r"\| 8 \| u64", "| 4 | u32")))),
        expect: &[("AC1", ""), ("AC2", "the close's window identifier is as wide"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_close_with_a_second_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window close$", |t| replace_in_line(t, CLO_WID, r"^\| 32 \| 8 \| u64 \| Window identifier: the identifier of the window this close ends, as its announcement carries it", "| 32 | 4 | u32 | Window identifier: the identifier of the window this close ends | any |\n| 36 | 4 | u32 | Spare, 0 only")))),
        expect: &[("AC1", "the close lays out only the identifier"), ("AC2", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "ac4_denial_fewer_than_six_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_in_line(t, r"^\| a datagram of 4 to 31 bytes", "$", "\n")))),
        expect: &[("AC4", "at least 6 invalid forms")],
    },
    Denial {
        name: "ac4_denial_a_reason_outside_the_control_layer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadChecksum")))),
        expect: &[("AC4", "names a reason outside the control layer")],
    },
    Denial {
        name: "ac4_denial_a_violation_without_its_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadLength")))),
        expect: &[("AC4", "must name BadMagic")],
    },
    Denial {
        name: "ac4_denial_a_violation_marked_ignored",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_in_line(t, r"^\| foreign domain:", r"\|$", "; ignored |")))),
        expect: &[("AC4", "is marked ignored or not a rejection")],
    },
    Denial {
        name: "ac4_denial_the_version_mismatch_without_its_event",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| every_in_line(t, r"^\| version mismatch", "(?i)event", "signal")))),
        expect: &[("AC4", "does not raise the named VersionMismatch event")],
    },
    Denial {
        name: "ac4_denial_a_form_without_an_outcome",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_in_line(t, r"^\| window identifier 0", r"\| drop and count under BadLength.*$", "| dropped |")))),
        expect: &[("AC4", "states no outcome")],
    },
    Denial {
        name: "ac4_denial_no_expiry_out_of_range_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| every_in_line(t, r"^\| expiry out of range", "(?i)out of range", "beyond limits")))),
        expect: &[("AC4", "covers the expiry out of range")],
    },
    Denial {
        name: "ac4_denial_no_expiry_past_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| every_in_line(t, r"^\| an announcement whose expiry is already past", "(?i)already past", "gone")))),
        expect: &[("AC4", "covers an expiry already past")],
    },
    Denial {
        name: "ac4_denial_no_unknown_close_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| every_in_line(t, r"^\| a close for an unknown window", "(?i)unknown", "foreign")))),
        expect: &[("AC4", "covers a close for an unknown window")],
    },
    Denial {
        name: "ac3_denial_no_second_announcement_outcome",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_all(t, "(?i)second|another announcement|already open|while (a |the )?window is open", "x"))),
        expect: &[("AC3", "a second announcement while a window is open has a stated outcome")],
    },
    Denial {
        name: "ac3_denial_no_open_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Receiver rules$", |t| replace_in_line(t, r"^1\. \*\*Open\.\*\*", r"\*\*Open\.\*\*", "**Start.**")))),
        expect: &[("AC3", "a numbered rule for the open transition")],
    },
    Denial {
        name: "ac3_denial_no_expiry_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Receiver rules$", |t| replace_in_line(t, r"^2\. \*\*Expiry\.\*\*", r"\*\*Expiry\.\*\*", "**Timeout.**")))),
        expect: &[("AC3", "a numbered rule for the expiry transition")],
    },
    Denial {
        name: "ac3_denial_no_explicit_close_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Receiver rules$", |t| replace_in_line(t, r"^3\. \*\*Explicit close\.\*\*", r"\*\*Explicit close\.\*\*", "**Termination.**")))),
        expect: &[("AC3", "a numbered rule for the explicit close")],
    },
    Denial {
        name: "ac3_denial_two_transitions_in_one_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Receiver rules$", |t| replace_in_line(&delete_unit(t, r"^3\. \*\*Explicit close\.\*\*"), r"^2\. \*\*Expiry\.\*\*", r"\*\*Expiry\.\*\* A module that can evaluate the expiry closes its window when the expiry is reached", "**Expiry and explicit close.** A module that can evaluate the expiry closes its window when the expiry is reached or on a close message")))),
        expect: &[("AC3", "three different receiver rules")],
    },
    Denial {
        name: "dod_denial_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window close$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", "")))),
        expect: &[("DoD", "the close subsection carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| strip_tags(t, r"^\*\*Tool identity\.\*\*")))),
        expect: &[("DoD", "every prose paragraph of 5 or more words in the section")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "**Open item 11 — spare.** Nothing is decided here `[decision]`.\n\n"))),
        expect: &[("DoD", "every open item of the section names an owner and a falsifier")],
    },
    Denial {
        name: "dod_denial_a_reserved_field_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, TOOL_ROW, "any 64-bit value", "any 64-bit value, accept-any")))),
        expect: &[("DoD", "every reserved or accept-any field of the new messages")],
    },
    Denial {
        name: "dod_denial_open_item_11_still_open",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\*\*Open item 11 — ", "Status, 2026-11-16", "Note")),
        expect: &[("DoD", "Open item 11 of the regime stamp is no longer open")],
    },
    Denial {
        name: "ac4_denial_no_outcome_after_an_explicit_close",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_all(t, r"(?i)explicitly closed|closed by|already closed|closed window|had closed|after (?:the |an |its |a )?(?:window'?s? )?(?:explicit )?close\b", "x"))),
        expect: &[("AC4", "closed by an explicit close has a stated outcome")],
    },
    Denial {
        name: "dod_denial_a_reopening_without_an_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(&within_section(t, "^Open items$", |o| replace_all(o, "(?i)reopen|late|after|repeat", "x")), "^Receiver rules$", |r| replace_in_line(r, r"^\*\*Injections\.\*\*", "^", "An announcement after the explicit close reopens the window `[decision]`.\n\n")))),
        expect: &[("DoD", "the reopening outcome after a close is an open item")],
    },
    Denial {
        name: "ac3_denial_two_windows_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_all(t, "(?i)at once|different|same time", "x"))),
        expect: &[("AC3", "two modules can hold different windows at once")],
    },
    Denial {
        name: "ac3_denial_a_lost_close_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_all(&replace_all(t, r"(?i)\blost\b", "missed"), r"(?i)\bnever\b", "not"))),
        expect: &[("AC3", "whose close was lost is stated")],
    },
    Denial {
        name: "dod_denial_a_bare_liveness_denial",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Receiver rules$", |t| replace_in_line(t, r"^\*\*Injections\.\*\*", "^", "The tool's counter cannot be a liveness counter `[decision]`.\n\n")))),
        expect: &[("DoD", "a denial that the tool's message counter is a liveness counter")],
    },
    Denial {
        name: "dod_denial_a_section_type_without_the_cap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Receiver rules$", |t| replace_in_line(t, r"^\*\*Injections\.\*\*", "^", "The window is not a section type 9 `[decision]`.\n\n")))),
        expect: &[("DoD", "restates the connection-cap threshold")],
    },
    Denial {
        name: "ac5_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[[broken")),
        expect: &[("AC5", "does not parse")],
    },
    Denial {
        name: "ac5_denial_a_multi_line_string_the_reader_rejects",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n")),
        expect: &[("AC5", "does not parse")],
    },
    Denial {
        name: "ac5_denial_no_announcement_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"commissioning window announcement\""), "name = \"commissioning window opening\"")),
        expect: &[("AC5", "exactly one commissioning window announcement message (found 0)")],
    },
    Denial {
        name: "ac5_denial_a_max_size_off_the_table",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("max_size = 56", "max_size = 57"))),
        expect: &[("AC5", "the announcement message of wire.toml is control")],
    },
    Denial {
        name: "ac5_denial_no_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), "name = \"commissioning window announcement\"\nlayer = \"control\"\ndscp = 40\nmax_size = 56\nrejection_reasons = []")),
        expect: &[("AC5", "the announcement message names only control reasons")],
    },
    Denial {
        name: "ac5_denial_fields_that_disagree",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| { let at = t.find("name = \"commissioning window close\"").expect("close"); let (a, b) = t.split_at(at); format!("{a}{}", b.replacen("width = 64", "width = 32", 1)) }),
        expect: &[("AC5", "the close fields of wire.toml agree")],
    },
    Denial {
        name: "ac1_denial_a_heading_after_a_python_line_break",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^The commissioning window is how", "^", "\u{2028}## Interlude\u{2028}"))),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "ac1_denial_a_word_character_only_python_knows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Window close$", "Window close²")),
        expect: &[("AC1", "names the close (found 0)"), ("DoD", "")],
    },
    Denial {
        name: "ac4_denial_a_validity_table_without_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n")))),
        expect: &[("AC4", "at least 6 invalid forms")],
    },
    Denial {
        name: "accepted_a_separator_row_only_the_scripts_accept",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Validity rules$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n")))),
        expect: &[],
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
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "num_enum_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 0² |")),
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
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
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "num_toml_dscp_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = 4_0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = \"40\""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_padded_signed_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = \" +040 \""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = \"_40\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_dscp_superscript_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = \"40²\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_dscp_big",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = 100000000000000000000"))),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "num_toml_dscp_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = 10000000000000000000000000000000000000000"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_toml_dscp_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = 40.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_plus",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("dscp = 40", "dscp = +40"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("max_size = 56", "max_size = 56.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("max_size = 56", "max_size = \"56\""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_bool",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("max_size = 56", "max_size = true"))),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "num_toml_max_size_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(ANN_HEAD), &ANN_HEAD.replace("max_size = 56", "max_size = \"abc\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_tool_empty_width_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 40 \| 8 \| u64 \| Tool identity", r"^\| 40 \| 8 \|", "| 40 |  |")))),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "num_tool_row_cut_to_two_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 40 \| 8 \| u64 \| Tool identity", r"^.*$", "| 40 | 8 |")))),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "num_size_superscript_before",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "fixed size, is 56 bytes", "fixed size, is ²56 bytes")))),
        expect: &[],
    },
    Denial {
        name: "num_size_superscript_after",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "fixed size, is 56 bytes", "fixed size, is 56² bytes")))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_size_leading_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "fixed size, is 56 bytes", "fixed size, is 056 bytes")))),
        expect: &[],
    },
    Denial {
        name: "num_size_big",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "fixed size, is 56 bytes", "fixed size, is 100000000000000000000 bytes")))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_size_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_all(t, "fixed size, is 56 bytes", "fixed size, is 10000000000000000000000000000000000000000 bytes")))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_type_code_leading_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 48", r"\| 48 only \|", "| 048 only |")))),
        expect: &[],
    },
    Denial {
        name: "num_type_code_big",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 48", r"\| 48 only \|", "| 100000000000000000000 only |")))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_type_code_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 48", r"\| 48 only \|", "| 10000000000000000000000000000000000000000 only |")))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_type_code_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| within_section(t, "^Window announcement$", |t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 48", r"\| 48 only \|", "| 48² only |")))),
        expect: &[("AC1", "")],
    },
    // ---- the re-proof checklist (points 3, 5 and 6): number forms, table
    // reader variants, enumeration ties, swaps and thresholds
    Denial {
        name: "chk_c17_zero",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "0")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "chk_c17_double_zero_seven",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "007")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "chk_c17_two_to_63",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "9223372036854775808")),
        expect: &[("AC1", "")],
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
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "chk_c17_micro_after",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252µ")),
        expect: &[("AC1", ""), ("CRASH", "")],
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
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
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
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "chk_enum_escaped_pipe",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", "fewer bytes", "fewer \\| bytes")),
        expect: &[],
    },
    Denial {
        name: "chk_tool_row_escaped_pipe",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", "$", " \\| x"))),
        expect: &[],
    },
    Denial {
        name: "chk_tool_row_blank_line_inside",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", "^", "\n"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", "")],
    },
    Denial {
        name: "chk_tool_row_ragged_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", "$", " extra |"))),
        expect: &[],
    },
    Denial {
        name: "chk_tool_row_fenced_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", "^", "```\n"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "chk_tool_row_offset_double_zero_seven",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", r"^\| \d+", "| 007"))),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "chk_tool_row_offset_plus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", r"^\| (\d+)", "| +7"))),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "chk_tool_row_offset_forty_five_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Commissioning window messages$", |t| replace_in_line(t, "^\\| 40 \\| 8 \\| u64 \\| Tool identity", r"^\| \d+", "| 111111111111111111111111111111111111111111111"))),
        expect: &[("INT_LIMIT", "")],
    },
];
