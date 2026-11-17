//! The denial list of 004-18: the real documents with one edit, and what
//! the port (and the old script, run by `doccheck-equiv 004-18`) must
//! report. One or more per `fail` site of the script, then the denials aimed
//! at Python semantics (`py_denial_*`), then the acceptance probes
//! (`py_accepts_*`, no expectation: the port must pass, as the script
//! does). Consumed by `tests/doc_004_18.rs`, one test each.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, delete_section, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::text::rx;

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

/// The Admission messages section.
fn ad(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Admission messages$", edit)
}

/// One `###` subsection of the Admission messages section.
fn sub(t: &str, title: &str, edit: impl FnOnce(&str) -> String) -> String {
    ad(t, |s| within_section(s, title, edit))
}

fn req(t: &str, from: &str, to: &str) -> String {
    sub(t, "^Admission request$", |s| lit(s, from, to))
}

fn acc(t: &str, from: &str, to: &str) -> String {
    sub(t, "^Admission accept$", |s| lit(s, from, to))
}

fn refusal_row(t: &str, from: &str, to: &str) -> String {
    sub(t, "^Refusal reasons$", |s| lit(s, from, to))
}

fn req_toml(t: &str, from: &str, to: &str) -> String {
    after(t, "name = \"admission request\"\n", from, to)
}

/// `pattern` replaced on the lines that are not table rows.
fn prose_only(t: &str, pattern: &str, to: &str) -> String {
    let re = rx(pattern);
    let out: Vec<String> = t
        .split('\n')
        .map(|l| if l.trim_start().starts_with('|') { l.to_string() } else { re.replace_all(l, to) })
        .collect();
    out.join("\n")
}

/// `pattern` replaced on the table rows only.
fn rows_only(t: &str, pattern: &str, to: &str) -> String {
    let re = rx(pattern);
    let out: Vec<String> = t
        .split('\n')
        .map(|l| if l.trim_start().starts_with('|') { re.replace_all(l, to) } else { l.to_string() })
        .collect();
    out.join("\n")
}

/// `pattern` replaced everywhere on the first line matching `line`.
fn on_line(t: &str, line: &str, pattern: &str, to: &str) -> String {
    let (lre, re) = (rx(line), rx(pattern));
    let mut hit = false;
    let out: Vec<String> = t
        .split('\n')
        .map(|l| {
            if !hit && lre.is_match(l) {
                hit = true;
                re.replace_all(l, to)
            } else {
                l.to_string()
            }
        })
        .collect();
    assert!(hit, "on_line: no line matches {line:?}");
    out.join("\n")
}

const RR: &str = "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]";
const TAGS: &str = r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "ac2_denial_an_admission_row_of_four_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Sender module id (C-11): the source side's module id | any 64-bit value |", "| 32 | 8 | u64 | Sender module id (C-11): the source side's module id |")),
        expect: &[("AC2", "a row has fewer than five cells")],
    },
    Denial {
        name: "ac2_denial_an_admission_offset_not_a_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Sender module id", "| x32 | 8 | u64 | Sender module id")),
        expect: &[("AC2", "is not a byte offset")],
    },
    Denial {
        name: "ac2_denial_an_admission_width_not_a_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Sender module id", "| 32 | eight | u64 | Sender module id")),
        expect: &[("AC2", "is neither a byte count")],
    },
    Denial {
        name: "h_denial_a_hello_offset_not_a_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Hello$", |s| lit(s, "| 32 | 8 | u64 | Module id", "| x32 | 8 | u64 | Module id")))),
        expect: &[("H", "is not a byte offset")],
    },
    Denial {
        name: "ac1_denial_a_receiver_report_offset_not_a_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Receiver report$", |s| lit(s, "| 34 | 4 | u32 | Current A", "| x34 | 4 | u32 | Current A")))),
        expect: &[("AC1", "receiver report: the Offset cell")],
    },
    Denial {
        name: "dod_denial_a_constant_unreadable",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most twelve hundred bytes"))),
        expect: &[("DoD", "constant for bound could not be read")],
    },
    Denial {
        name: "behaviour_denial_a_duration_unreadable",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-33 \|", "400 ms", "four hundred ms"))),
        expect: &[("Behaviour", "constant for window could not be read")],
    },
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist")],
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
        expect: &[("AC1", "no '## Hello' section")],
    },
    Denial {
        name: "ac3_denial_no_hello_election_epoch",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Hello$", |s| lit(s, "| 52 | 4 | u32 | Election epoch:", "| 52 | 4 | u32 | Vote epoch:")))),
        expect: &[("AC3", "the hello layout has no election epoch row")],
    },
    Denial {
        name: "ac2_denial_c185_not_eighteen",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-185 \|", r" and NoSkewBudget \(", " and ("))),
        expect: &[("AC2", "C-185 names 17 refusal reasons")],
    },
    Denial {
        name: "ac2_denial_c185_without_a_discovery_reason",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-185 \|", "ModuleCap, NameTaken", "ModuleLimit, NameTaken"))),
        expect: &[("AC2", "discovery-raised reason ModuleCap is not in the C-185 set")],
    },
    Denial {
        name: "ac1_denial_no_admission_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Admission messages$", "Admission"))),
        expect: &[("AC1", "no '## Admission messages' section")],
    },
    Denial {
        name: "ac2_denial_a_refusal_listed_twice",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| refusal_row(b, "| 17 | NoSkewBudget |", "| 18 | PortCeiling | twice | admission |\n| 17 | NoSkewBudget |")),
        expect: &[("AC2", "refusal reason PortCeiling is listed twice")],
    },
    Denial {
        name: "ac2_denial_no_refusal_enumeration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| delete_section(t, "^Refusal reasons$"))),
        expect: &[("AC2", "has no numbered refusal enumeration")],
    },
    Denial {
        name: "ac2_denial_refusal_names_differ_from_c185",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| refusal_row(b, "| 17 | NoSkewBudget |", "| 17 | NoSkewLimit |")),
        expect: &[("AC2", "names differ from C-185")],
    },
    Denial {
        name: "ac2_denial_refusal_values_not_unique",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| refusal_row(b, "| 17 | NoSkewBudget |", "| 16 | NoSkewBudget |")),
        expect: &[("AC2", "values are not unique")],
    },
    Denial {
        name: "ac2_denial_a_refusal_value_outside_a_u8",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| refusal_row(b, "| 17 | NoSkewBudget |", "| 300 | NoSkewBudget |")),
        expect: &[("AC2", "has value 300, outside a u8")],
    },
    Denial {
        name: "ac2_denial_a_discovery_reason_not_marked",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Refusal reasons$", |t| on_line(t, r"^\| 8 \| VersionMismatch", "(?i)discover", "join"))),
        expect: &[("AC2", "VersionMismatch is not marked as raised at discovery")],
    },
    Denial {
        name: "ac2_denial_another_reason_marked_discovery",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| refusal_row(b, "| 0 | PortCeiling | bit/s", "| 0 | PortCeiling | discovery bit/s")),
        expect: &[("AC2", "PortCeiling is marked as discovery-raised")],
    },
    Denial {
        name: "ac2_denial_no_discovery_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| prose_only(t, "(?i)discover", "join"))),
        expect: &[("AC2", "says that VersionMismatch, NameTaken and ModuleCap are raised at discovery")],
    },
    Denial {
        name: "ac2_denial_no_c185_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| prose_only(t, "C-185", "C-999"))),
        expect: &[("AC2", "cites C-185 for the refusal enumeration")],
    },
    Denial {
        name: "ac2_denial_refusal_values_out_of_c185_order",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Refusal reasons$", |t| lit(&lit(t, "| 0 | PortCeiling |", "| 1 | PortCeiling |"), "| 1 | PacketRate |", "| 0 | PacketRate |"))),
        expect: &[("AC2", "C-185's order gives 0")],
    },
    Denial {
        name: "ac1_denial_fewer_than_two_messages",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, r"\| Offset \(bytes\) \|", "| Position |"))),
        expect: &[("AC1", "needs at least two messages")],
    },
    Denial {
        name: "ac1_denial_no_admission_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Hello$", |s| lit(s, "| 16 to 31 | admission handshake messages", "| 16 to 31 | handshake messages")))),
        expect: &[("AC1", "does not give the admission block")],
    },
    Denial {
        name: "ac1_denial_not_starting_with_the_header",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| Magic 0x4d4e", "| Marker 0x4d4e")),
        expect: &[("AC1", "does not start with the common control header rows")],
    },
    Denial {
        name: "ac1_denial_a_header_row_of_another_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 7 | 1 | u8 | Authentication scheme", "| 7 | 1 | i8 | Authentication scheme")),
        expect: &[("AC1", "differs from the common control header in offset, width or type")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 52 | 4 | u32 | Pinned destination address", "| 53 | 4 | u32 | Pinned destination address")),
        expect: &[("AC1", "at byte 53, expected 52")],
    },
    Denial {
        name: "ac1_denial_a_row_of_part_of_a_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 56 | 2 | u16 | Pinned destination port", "| 56 | 12 bits | u16 | Pinned destination port")),
        expect: &[("AC1", "is not a whole number of bytes")],
    },
    Denial {
        name: "ac4_denial_no_maximum_size_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "The accept's maximum size is 62 bytes", "The accept's largest form is 62 bytes")),
        expect: &[("AC4", "no sentence states its maximum size")],
    },
    Denial {
        name: "ac4_denial_a_maximum_size_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "The accept's maximum size is 62 bytes", "The accept's maximum size is 63 bytes")),
        expect: &[("AC4", "states a maximum size of 63 bytes")],
    },
    Denial {
        name: "ac4_denial_a_message_above_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 79 | 5 × max 32 |", "| 79 | 5 × max 320 |")),
        expect: &[("AC4", "is above the 1252-byte control bound")],
    },
    Denial {
        name: "ac1_denial_no_type_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 17 only |", "| seventeen only |")),
        expect: &[("AC1", "does not state the message's type code")],
    },
    Denial {
        name: "ac1_denial_a_code_outside_the_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 17 only |", "| 40 only |")),
        expect: &[("AC1", "type code 40 is outside the admission block")],
    },
    Denial {
        name: "ac1_denial_a_shared_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 17 only |", "| 16 only |")),
        expect: &[("AC1", "share type code 16")],
    },
    Denial {
        name: "ac2_denial_a_scheme_not_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "(C-12); 0 = none in v1 (C-115) | 0 only |", "(C-12); 0 = none in v1 (C-115) | nothing |")),
        expect: &[("AC2", "the authentication scheme row does not fix the v1 value 0")],
    },
    Denial {
        name: "ac2_denial_a_tag_not_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "128 zero bits in v1 (C-115) | 0 only |", "128 zero bits in v1 (C-115) | nothing |")),
        expect: &[("AC2", "the authentication tag row does not fix")],
    },
    Denial {
        name: "ac2_denial_a_counter_not_live",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission accept$", |t| replace_in_line(t, r"^\| 8 \| 8 \| u64 \| Message counter", r"live \(C-115\).*\(Hello, Liveness counter\)", "kept (C-115)"))),
        expect: &[("AC2", "the message counter row does not say it is live")],
    },
    Denial {
        name: "ac2_denial_no_c115_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| prose_only(t, "C-115", "C-999"))),
        expect: &[("AC2", "cites C-115 for the v1 values")],
    },
    Denial {
        name: "ac1_denial_no_refusal_reason_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| rows_only(t, r"(?i)refusal\s+reason", "answer"))),
        expect: &[("AC1", "carries a refusal reason field")],
    },
    Denial {
        name: "ac1_denial_a_refusal_reason_of_two_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission refusal$", |t| lit(t, "| 52 | 1 | u8 | Refusal reason", "| 52 | 2 | u8 | Refusal reason"))),
        expect: &[("AC1", "the refusal reason is 16 bits")],
    },
    Denial {
        name: "ac2_denial_a_refusal_field_naming_a_discovery_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission refusal$", |t| lit(t, "0 to 17, except the three discovery-raised values 8, 13 and 14", "0 to 17, ModuleCap included"))),
        expect: &[("AC2", "valid range names ModuleCap")],
    },
    Denial {
        name: "ac1_denial_no_bit_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Declared wire rate: bit/s on the wire", "| Declared wire rate: speed on the wire")),
        expect: &[("AC1", "declared rate in bit/s (Context")],
    },
    Denial {
        name: "ac1_denial_a_bit_rate_too_narrow",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 62 | 8 | u64 | Declared wire rate", "| 62 | 3 | u64 | Declared wire rate")),
        expect: &[("AC1", "the declared rate in bit/s is 24 bits")],
    },
    Denial {
        name: "ac1_denial_no_packet_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission request$", |t| replace_all(t, r"(?i)packets? per second|packets?\s*/\s*s", "frames each second"))),
        expect: &[("AC1", "declared rate in packets/s")],
    },
    Denial {
        name: "ac1_denial_a_packet_rate_too_narrow",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 70 | 4 | u32 | Declared packet rate", "| 70 | 1 | u32 | Declared packet rate")),
        expect: &[("AC1", "the packets/s rate is 8 bits")],
    },
    Denial {
        name: "ac1_denial_no_slice_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Declared slice rate:", "| Declared slicing:")),
        expect: &[("AC1", "carries a slice rate")],
    },
    Denial {
        name: "ac1_denial_a_slice_rate_too_narrow",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 74 | 4 | u32 | Declared slice rate", "| 74 | 1 | u32 | Declared slice rate")),
        expect: &[("AC1", "the slice rate is 8 bits")],
    },
    Denial {
        name: "ac1_denial_no_c219_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| prose_only(t, "C-219", "C-999"))),
        expect: &[("AC1", "cites C-219 for the declared rates")],
    },
    Denial {
        name: "ac1_denial_no_member_list_of_32",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 79 | 5 × max 32 |", "| 79 | 5 × max 31 |")),
        expect: &[("AC1", "bundle members with a maximum count of 32")],
    },
    Denial {
        name: "ac1_denial_no_reverse_port",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 58 | 2 | u16 | Reverse measurement port", "| 58 | 2 | u16 | Return measurement port")),
        expect: &[("AC1", "reverse measurement port as a 16-bit port")],
    },
    Denial {
        name: "ac1_denial_no_address",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| rows_only(t, "(?i)address", "host"))),
        expect: &[("AC1", "carries a pinned address")],
    },
    Denial {
        name: "ac1_denial_an_address_of_two_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 52 | 4 | u32 | Pinned destination address", "| 52 | 2 | u32 | Pinned destination address")),
        expect: &[("AC1", "the pinned address is 16 bits")],
    },
    Denial {
        name: "behaviour_denial_no_ipv4",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, "C-100", "C-999"))),
        expect: &[("Behaviour", "unicast over IPv4 (C-100)")],
    },
    Denial {
        name: "behaviour_denial_no_cs5",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, r"CS5 \(40\)", "CS5"))),
        expect: &[("Behaviour", "DSCP CS5 (40) with C-13")],
    },
    Denial {
        name: "ac1_denial_no_manifest_reference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| rows_only(t, "004-26", "004-2x"))),
        expect: &[("AC1", "refers to the manifest laid out in 004-26")],
    },
    Denial {
        name: "ac1_denial_no_probe_reference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "the path-MTU probe pair (004-27) first probes", "the path-MTU probe pair first probes")),
        expect: &[("AC1", "refers to the path-MTU probe pair laid out in 004-27")],
    },
    Denial {
        name: "ac1_denial_no_downstream_a",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| rows_only(t, "(?i)downstream", "onward"))),
        expect: &[("AC1", "carries the downstream A of C-142")],
    },
    Denial {
        name: "ac1_denial_no_receiver_report_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| t.split('\n').map(|l| if l.starts_with('#') { l.replace("eceiver report", "eceiver summary").replace("eceiver Report", "eceiver summary") } else { l.to_string() }).collect::<Vec<_>>().join("\n"))),
        expect: &[("AC1", "has no receiver report section")],
    },
    Denial {
        name: "ac1_denial_no_current_a_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Receiver report$", |s| rows_only(s, r"(?i)current\s+a\b", "present budget")))),
        expect: &[("AC1", "has no current A row")],
    },
    Denial {
        name: "ac1_denial_a_downstream_a_without_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission accept$", |t| on_line(t, r"^\| 58 \| 4 \| u32 \| Downstream A", r"(?i)\bns\b|nanoseconds", "units"))),
        expect: &[("AC1", "the downstream A has no unit")],
    },
    Denial {
        name: "ac1_denial_a_downstream_a_of_another_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 58 | 4 | u32 | Downstream A", "| 58 | 2 | u32 | Downstream A")),
        expect: &[("AC1", "the downstream A is 16 bits")],
    },
    Denial {
        name: "ac1_denial_a_downstream_a_without_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 58 | 4 | u32 | Downstream A", "| 58 | 0 | u32 | Downstream A")),
        expect: &[("AC1", "the downstream A has no width")],
    },
    Denial {
        name: "ac5_denial_a_timer_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Request id: chosen by the source side", "| Pending id: chosen by the source side")),
        expect: &[("AC5", "is a timer; C-45")],
    },
    Denial {
        name: "ac5_denial_c45_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, "C-45", "C-999"))),
        expect: &[("AC5", "C-45 (Pending has no timeout) is not cited")],
    },
    Denial {
        name: "settled_denial_c103_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, "C-103", "C-999"))),
        expect: &[("Settled", "C-103 is not cited")],
    },
    Denial {
        name: "ac6_denial_no_request_message",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| rename_heading(t, "^Admission request$", "Admission ask"))),
        expect: &[("AC6", "no admission request message")],
    },
    Denial {
        name: "ac6_denial_a_refusal_before_a_drop",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission request$", |t| {
        let row = t.split('\n').find(|l| l.starts_with("| member count 0 |")).expect("row").to_string();
        let t = lit(t, &format!("{row}\n"), "");
        lit(&t, "| declared rates above what the receiving side can take |", &format!("{row}\n| declared rates above what the receiving side can take |"))
    })),
        expect: &[("AC6", "lists a refusal before a drop")],
    },
    Denial {
        name: "ac6_denial_no_order_hand_off",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| prose_only(t, "005-14", "005-99"))),
        expect: &[("AC6", "hands the order among refusals to 005-14")],
    },
    Denial {
        name: "ac6_denial_no_foreign_domain_citation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, r"Open item 7\)", "the list)"))),
        expect: &[("AC6", "does not cite an open item by number")],
    },
    Denial {
        name: "hand_off_denial_no_012_01",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, "012-01", "012-99"))),
        expect: &[("Hand-off", "does not mention 012-01")],
    },
    Denial {
        name: "ac6_denial_counted_under_a_non_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "Badmagic"))),
        expect: &[("AC6", "counts under 'Badmagic'")],
    },
    Denial {
        name: "ac6_denial_an_invented_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "WrongMagic"))),
        expect: &[("AC6", "names 'WrongMagic', which is in neither enumeration")],
    },
    Denial {
        name: "ac6_denial_an_exact_reason_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadLength"))),
        expect: &[("AC6", "must be counted under BadMagic")],
    },
    Denial {
        name: "ac6_denial_a_member_form_without_bad_member_list",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission request$", |t| replace_in_line(t, r"^\| member count above 32", "BadMemberList", "InputCount"))),
        expect: &[("AC6", "must be refused with BadMemberList")],
    },
    Denial {
        name: "ac6_denial_a_discovery_reason_refused",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Admission refusal$", |t| replace_in_line(t, r"^\| a refusal carrying a reason raised at discovery", r"\| drop and count under BadLength", "| refused with ModuleCap, then counted under BadLength"))),
        expect: &[("AC6", "is refused with a discovery-raised reason")],
    },
    Denial {
        name: "ac6_denial_a_form_naming_no_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| message type 0", r"\| drop and count under BadLength.*$", "| dropped silently |"))),
        expect: &[("AC6", "names no member of the closed enumerations and argues no non-rejection")],
    },
    Denial {
        name: "ac6_denial_fewer_than_six_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |", "| Case | Result |"))),
        expect: &[("AC6", "at least 6")],
    },
    Denial {
        name: "ac6_denial_fewer_than_eight_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Case | Result |"))),
        expect: &[("AC6", "at least 8 are required")],
    },
    Denial {
        name: "ac6_denial_no_oversize_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| oversize"))),
        expect: &[("AC6", "covers the 'oversize' case")],
    },
    Denial {
        name: "dod_denial_no_owned_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, "needs a message or field", "needs more"))),
        expect: &[("DoD", "names owner 004, epic 005 as the confirming owner")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| strip_tags(t, r"^\*\*Who sends what"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| append(t, "**Open item 99 — x.** Still open `[decision]`."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    Denial {
        name: "dod_denial_a_table_in_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_all(t, TAGS, ""))),
        expect: &[("DoD", "a table of 'Validity rules' follows no evidence-tagged paragraph")],
    },
    Denial {
        name: "ac7_denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC7", "does not exist")],
    },
    Denial {
        name: "ac7_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| format!("{t}\n[[[broken"))),
        expect: &[("AC7", "does not parse")],
    },
    Denial {
        name: "ac7_denial_no_request_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"admission request\"\n", "name = \"admission ask\"\n"))),
        expect: &[("AC7", "declares no [[message]] for the admission message 'Admission request'")],
    },
    Denial {
        name: "ac7_denial_the_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "layer = \"control\"", "layer = \"media\"")),
        expect: &[("AC7", "has layer 'media'")],
    },
    Denial {
        name: "ac7_denial_the_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 40", "dscp = 46")),
        expect: &[("AC7", "has dscp 46")],
    },
    Denial {
        name: "ac7_denial_the_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 239", "max_size = 240")),
        expect: &[("AC7", "has max_size 240")],
    },
    Denial {
        name: "ac7_denial_a_field_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "[[message.field]]\nname = \"request_id\"\n", "[[message.gone]]\nname = \"request_id\"\n")),
        expect: &[("AC7", "the markdown table has 20 rows")],
    },
    Denial {
        name: "ac7_denial_a_field_misnamed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"", "name = \"request\"")),
        expect: &[("AC7", "toml field 'request' is not named after")],
    },
    Denial {
        name: "ac7_denial_a_field_offset",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 384\n", "offset = 385\n")),
        expect: &[("AC7", "offset 385 bits")],
    },
    Denial {
        name: "ac7_denial_a_field_width",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"\noffset = 384\nwidth = 32", "name = \"request_id\"\noffset = 384\nwidth = 16")),
        expect: &[("AC7", "toml field request_id: width 16 bits")],
    },
    Denial {
        name: "ac7_denial_a_max_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_count = 32", "max_count = 31")),
        expect: &[("AC7", "max_count 31")],
    },
    Denial {
        name: "ac7_denial_a_field_type",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"\noffset = 384\nwidth = 32\ntype = \"u32\"", "name = \"request_id\"\noffset = 384\nwidth = 32\ntype = \"i32\"")),
        expect: &[("AC7", "toml field request_id: type i32")],
    },
    Denial {
        name: "dod_denial_a_toml_field_named_like_a_timer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"", "name = \"request_timeout\"")),
        expect: &[("DoD", "is named like a timer")],
    },
    Denial {
        name: "ac7_denial_fields_not_summing_to_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"\noffset = 384\nwidth = 32", "name = \"request_id\"\noffset = 384\nwidth = 40")),
        expect: &[("AC7", "compute a size other than max_size")],
    },
    Denial {
        name: "ac7_denial_a_named_reason_not_listed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "\"BadMagic\", ", "")),
        expect: &[("AC7", "does not list 'BadMagic'")],
    },
    Denial {
        name: "ac7_denial_a_listed_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "rejection_reasons = [", "rejection_reasons = [\"PortCeiling\", ")),
        expect: &[("AC7", "names 'PortCeiling', not a member of the closed Reject enumeration")],
    },
    Denial {
        name: "ac7_denial_the_toml_refusals_differ",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"NoSkewBudget\"\nvalue = 17", "name = \"NoSkewBudget\"\nvalue = 18"))),
        expect: &[("AC7", "declares no refusal enumeration equal")],
    },
    Denial {
        name: "ac7_denial_the_toml_refusals_out_of_order",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"NoSkewBudget\"\nvalue = 17", "name = \"NoSkewBudgetX\"\nvalue = 17"))),
        expect: &[("AC7", "wire.toml gives refusal reason NoSkewBudget the value None")],
    },
    // `\w`: `²` glued to the reason defeats `\bBadMagic\b`.
    Denial {
        name: "py_denial_a_reason_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadMagic\u{b2}"))),
        expect: &[("AC6", "must be counted under BadMagic")],
    },
    // `\s`: U+001F is stripped by `clean_start`, so this is an open item.
    Denial {
        name: "py_denial_an_open_item_after_a_unit_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| append(t, "\u{1f}**Open item 99 — x.** Still open `[decision]`."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    // `str.splitlines()`: U+2028 ends the shared table in its second row.
    Denial {
        name: "py_denial_a_line_separator_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{2028}drop and count under BadMagic"))),
        expect: &[("AC6", "covers the 'magic' case")],
    },
    // `str.splitlines()`: a form feed too.
    Denial {
        name: "py_denial_a_form_feed_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{c}drop and count under BadMagic"))),
        expect: &[("AC6", "covers the 'magic' case")],
    },
    // `int()` is unbounded: a 21-digit refusal value is outside a u8.
    Denial {
        name: "py_denial_a_refusal_value_beyond_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| refusal_row(b, "| 17 | NoSkewBudget |", "| 100000000000000000017 | NoSkewBudget |")),
        expect: &[("AC2", "has value 100000000000000000017, outside a u8")],
    },
    // `\d`/`int()` read Arabic-Indic digits: ٣٣ is 33, out of place.
    Denial {
        name: "py_denial_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 32 | 8 | u64 | Sender module id", "| \u{663}\u{663} | 8 | u64 | Sender module id")),
        expect: &[("AC1", "at byte 33, expected 32")],
    },
    // `(?!s)` under `re.I` refuses the long s too: "Open itemſ 7" is no
    // citation.
    Denial {
        name: "py_denial_a_citation_followed_by_a_long_s",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| replace_all(t, r"Open item 7\)", "Open item\u{17f} 7)"))),
        expect: &[("AC6", "does not cite an open item by number")],
    },
    // The subset reader keeps `"239"` a str.
    Denial {
        name: "py_denial_max_size_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 239", "max_size = \"239\"")),
        expect: &[("AC7", "has max_size 239; the markdown table sums to 239")],
    },
    // The subset reader iterates a str's characters.
    Denial {
        name: "py_denial_rejection_reasons_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, RR, "rejection_reasons = \"BadMagic\"")),
        expect: &[("AC7", "does not list 'BadMagic'")],
    },
    // The subset reader keeps a refusal value `"17"` a str, which is no
    // int: the refusal is missing from the toml enumeration.
    Denial {
        name: "py_denial_a_refusal_value_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"NoSkewBudget\"\nvalue = 17", "name = \"NoSkewBudget\"\nvalue = \"17\""))),
        expect: &[("AC7", "NoSkewBudget the value None")],
    },
    // The subset reader cannot read `"""` strings.
    Denial {
        name: "py_denial_a_triple_quoted_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"", "name = \"\"\"request_id\"\"\"")),
        expect: &[("AC7", "does not parse")],
    },
    // The subset reader lets a later duplicate key win.
    Denial {
        name: "py_denial_a_duplicate_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 40\n", "dscp = 40\ndscp = 46\n")),
        expect: &[("AC7", "has dscp 46")],
    },
    // A number of rejection reasons is not iterable: TypeError.
    Denial {
        name: "py_denial_rejection_reasons_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, RR, "rejection_reasons = 5")),
        expect: &[("CRASH", "not iterable")],
    },
    // A str width times one is a str, which the sum rejects: TypeError.
    Denial {
        name: "py_denial_a_width_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"request_id\"\noffset = 384\nwidth = 32", "name = \"request_id\"\noffset = 384\nwidth = \"32\"")),
        expect: &[("CRASH", "adds a non-number")],
    },
    // The 004 separator line: a letter in it drops the shared table.
    Denial {
        name: "py_denial_a_separator_with_a_letter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|---|-x-|"))),
        expect: &[("AC6", "at least 6")],
    },
    // `re.I`: "Open ıtem" is an open item to Python.
    Denial {
        name: "py_denial_an_open_item_with_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| ad(b, |t| append(t, "**Open \u{131}tem 99 — x.** Still open `[decision]`."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    // `\d`/`int()`: ٣٢ is 32.
    Denial {
        name: "py_accepts_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| 32 | 8 | u64 | Sender module id", "| \u{663}\u{662} | 8 | u64 | Sender module id")),
        expect: &[],
    },
    // The 24-character look-back counts characters, not bytes.
    Denial {
        name: "py_accepts_a_negation_within_24_characters_of_wide_ones",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "— never ForeignDomain", "— never ——————— ForeignDomain"))),
        expect: &[],
    },
    // The subset reader: 40.0 and 0x28 equal 40; a refusal value `true` is
    // an int equal to 1.
    Denial {
        name: "py_accepts_a_float_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 40\n", "dscp = 40.0\n")),
        expect: &[],
    },
    Denial {
        name: "py_accepts_a_hex_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 40\n", "dscp = 0x28\n")),
        expect: &[],
    },
    Denial {
        name: "py_accepts_a_refusal_value_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"PacketRate\"\nvalue = 1", "name = \"PacketRate\"\nvalue = true"))),
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
        edit: DenialEdit::Text(|b| acc(b, "| 32 | 8 | u64 | Sender module id", "| 32\u{1f} | 8 | u64 | Sender module id")),
        expect: &[],
    },
    // A combining accent is no word character for Python: `\bA\b` finds
    // "A\u{301}".
    Denial {
        name: "py_accepts_a_combining_accent_after_a",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| acc(b, "| Downstream A:", "| Downstream A\u{301}:")),
        expect: &[],
    },
];
