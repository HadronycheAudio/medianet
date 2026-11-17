//! The denial list of 004-17: the real documents with one edit, and what
//! the port (and the old script, run by `doccheck-equiv 004-17`) must
//! report. One or more per `fail` site of the script, then the denials aimed
//! at Python semantics (`py_denial_*`), then the acceptance probes
//! (`py_accepts_*`, no expectation: the port must pass, as the script
//! does). Consumed by `tests/doc_004_17.rs`, one test each.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
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

/// The Election messages section.
fn el(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Election messages$", edit)
}

/// One `###` subsection of the Election messages section.
fn sub(t: &str, title: &str, edit: impl FnOnce(&str) -> String) -> String {
    el(t, |s| within_section(s, title, edit))
}

/// `from` replaced in the candidacy subsection.
fn cand(t: &str, from: &str, to: &str) -> String {
    sub(t, "^Candidacy$", |s| lit(s, from, to))
}

/// The Hello section.
fn hello(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Hello$", edit)
}

/// `from` replaced in the toml candidacy.
fn cand_toml(t: &str, from: &str, to: &str) -> String {
    after(t, "name = \"candidacy\"\n", from, to)
}

const RR: &str = "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]";
const TS_ROW: &str = "| 65 | 1 | u8 | Time state:";
const VALIDITY_HEAD: &str = "| Invalid form | Required outcome |";

/// The denials.
pub const DENIALS: &[Denial] = &[
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
        expect: &[("AC1", "does not exist"), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("Behaviour", "")],
    },
    Denial {
        name: "ac4_denial_no_rejection_reasons_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Rejection reasons$", "Refusals"))),
        expect: &[("AC4", "no 'Rejection reasons' section"), ("AC6", "")],
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
        expect: &[("AC1", "no '## Hello' section"), ("AC3", "")],
    },
    Denial {
        name: "h_denial_a_hello_layout_row_of_four_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |", "| 32 | 8 | u64 | Module id (C-11) of the sender |"))),
        expect: &[("H", "a row has fewer than five cells")],
    },
    Denial {
        name: "h_denial_a_hello_offset_not_a_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 32 | 8 | u64 | Module id", "| x32 | 8 | u64 | Module id"))),
        expect: &[("H", "is not a byte offset")],
    },
    Denial {
        name: "h_denial_a_hello_width_not_a_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 32 | 8 | u64 | Module id", "| 32 | eight | u64 | Module id"))),
        expect: &[("H", "is neither a byte count")],
    },
    Denial {
        name: "ac3_denial_no_hello_election_epoch_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 52 | 4 | u32 | Election epoch:", "| 52 | 4 | u32 | Vote epoch:"))),
        expect: &[("AC3", "the hello layout has no election epoch row")],
    },
    Denial {
        name: "ac1_denial_no_election_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 2 to 15 | election messages |", "| 2 to 15 | vote messages |"))),
        expect: &[("AC1", "does not give the block for election messages")],
    },
    Denial {
        name: "ac1_denial_no_election_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Election messages$", "Elections"))),
        expect: &[("AC1", "no '## Election messages' section"), ("AC4", ""), ("AC5", ""), ("Behaviour", "")],
    },
    Denial {
        name: "ac1_denial_no_election_message",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, r"\| Offset \(bytes\) \|", "| Position |"))),
        expect: &[("AC1", "has no election message")],
    },
    Denial {
        name: "ac1_denial_a_row_of_four_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |", "| 32 | 8 | u64 | Module id (C-11) of the sender |")),
        expect: &[("AC1", "a row has fewer than five cells"), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_an_offset_not_a_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| x32 | 8 | u64 | Module id")),
        expect: &[("AC1", "is not a byte offset"), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_width_not_a_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| 32 | eight | u64 | Module id")),
        expect: &[("AC1", "is neither a byte count"), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_not_starting_with_the_header",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| Magic 0x4d4e", "| Marker 0x4d4e")),
        expect: &[("AC1", "does not start with the common control header rows"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_header_row_of_another_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 7 | 1 | u8 | Authentication scheme", "| 7 | 1 | i8 | Authentication scheme")),
        expect: &[("AC1", "differs from the common control header in offset, width or type"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 52 | 8 | u64 | Believed holder:", "| 53 | 8 | u64 | Believed holder:")),
        expect: &[("AC1", "at byte 53, expected 52"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_a_row_of_part_of_a_byte",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 64 | 1 | u8 | Holder eligibility", "| 64 | 4 bits | u8 | Holder eligibility")),
        expect: &[("AC1", "is not a whole number of bytes"), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_maximum_size_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "The candidacy's maximum size is 66 bytes", "The candidacy's largest form is 66 bytes")),
        expect: &[("AC1", "no sentence states its maximum size")],
    },
    Denial {
        name: "ac1_denial_a_maximum_size_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "The candidacy's maximum size is 66 bytes", "The candidacy's maximum size is 67 bytes")),
        expect: &[("AC1", "states a maximum size of 67 bytes")],
    },
    Denial {
        name: "ac1_denial_a_message_above_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 65 | 1 | u8 | Time state:", "| 65 | 1 × max 2000 | u8 | Time state:")),
        expect: &[("AC1", "is above the 1252-byte control bound"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_type_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| two only |")),
        expect: &[("AC1", "does not state the message's type code")],
    },
    Denial {
        name: "ac1_denial_a_code_outside_the_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| 16 only |")),
        expect: &[("AC1", "type code 16 is outside the block")],
    },
    Denial {
        name: "ac1_denial_a_shared_code",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Holder announcement$", |t| lit(t, "| 3 only |", "| 2 only |"))),
        expect: &[("AC1", "share type code 2")],
    },
    Denial {
        name: "ac2_denial_a_scheme_not_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "(C-12); 0 = none in v1 (C-115) | 0 only |", "(C-12); 0 = none in v1 (C-115) | nothing |")),
        expect: &[("AC2", "the authentication scheme row does not fix the v1 value 0")],
    },
    Denial {
        name: "ac2_denial_a_tag_not_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "128 zero bits in v1 (C-115) | 0 only |", "128 zero bits in v1 (C-115) | nothing |")),
        expect: &[("AC2", "the authentication tag row does not fix")],
    },
    Denial {
        name: "ac2_denial_a_counter_not_live",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Candidacy$", |t| replace_in_line(t, r"^\| 8 \| 8 \| u64 \| Message counter", r"live \(C-115\).*\(Hello, Liveness counter\)", "kept (C-115)"))),
        expect: &[("AC2", "the message counter row does not say it is live")],
    },
    Denial {
        name: "ac2_denial_no_c115_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "C-115", "C-999"))),
        expect: &[("AC2", "cites C-115 for the v1 values")],
    },
    Denial {
        name: "ac3_denial_no_module_id_of_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| 32 | 4 | u64 | Module id")),
        expect: &[("AC3", "no module id row of 64 bits"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_nonce_of_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 40 | 8 | u64 | Instance nonce", "| 40 | 4 | u64 | Instance nonce")),
        expect: &[("AC3", "no instance nonce row of 64 bits"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_election_epoch_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 48 | 4 | u32 | Election epoch:", "| 48 | 4 | u32 | Vote epoch:")),
        expect: &[("AC3", "no election epoch row"), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_an_election_epoch_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 48 | 4 | u32 | Election epoch:", "| 48 | 2 | u32 | Election epoch:")),
        expect: &[("AC3", "the election epoch is 16 bits"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_time_state",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, r"\| Time state:", "| Clock state:"))),
        expect: &[("AC3", "no election message carries a time state"), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_a_time_state_of_two_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, TS_ROW, "| 65 | 2 | u8 | Time state:")),
        expect: &[("AC3", "the time state is 16 bits"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_a_time_state_without_its_values",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Candidacy$", |t| replace_in_line(t, r"^\| 65 \| 1 \| u8 \| Time state:", "2 = Locked", "2: Locked"))),
        expect: &[("AC3", "does not state 'N = meaning'")],
    },
    Denial {
        name: "ac3_denial_two_time_state_encodings",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Candidacy$", |t| replace_in_line(t, r"^\| 65 \| 1 \| u8 \| Time state:", "2 = Locked", "2 = Locked, 3 = lapsed"))),
        expect: &[("AC3", "encode the time state two different ways")],
    },
    Denial {
        name: "ac3_denial_no_time_state_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "C-213", "C-999"))),
        expect: &[("AC3", "relates the election time state to the hello's")],
    },
    Denial {
        name: "ac3_denial_no_three_values_argument",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "C-162", "C-999"))),
        expect: &[("AC3", "no sentence argues why three values suffice")],
    },
    Denial {
        name: "ac3_denial_four_values_without_a_mapping",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| {
        let t = replace_all(t, "2 = Locked", "2 = Locked, 3 = lapsed");
        replace_all(&replace_all(&t, r"\bmap(s|ping|ped)?\b", "tie"), r"\bread\s+as\b", "taken as")
    })),
        expect: &[("AC3", "has no sentence that maps it")],
    },
    Denial {
        name: "ac3_denial_no_holder_eligibility",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| Holder eligibility:", "| Candidate flag:")),
        expect: &[("AC3", "no election message carries holder eligibility"), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_eligibility_without_bit_0",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Candidacy$", |t| {
        let i = t.find("| 64 | 1 | u8 | Holder eligibility").expect("row");
        let end = t[i..].find('\n').map_or(t.len(), |k| i + k);
        format!("{}{}{}", &t[..i], t[i..end].replace("bit 0", "bit zero"), &t[end..])
    })),
        expect: &[("AC3", "does not name bit 0")],
    },
    Denial {
        name: "ac3_denial_eligibility_of_two_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 64 | 1 | u8 | Holder eligibility", "| 64 | 2 | u8 | Holder eligibility")),
        expect: &[("AC3", "holder eligibility is 16 bits"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_believed_holder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 52 | 8 | u64 | Believed holder:", "| 52 | 4 | u64 | Believed holder:")),
        expect: &[("AC3", "carries the believed holder"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_winner_timeline_epoch",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Holder announcement$", |t| lit(t, "| 53 | 2 | u16 | Winner timeline epoch", "| 53 | 4 | u16 | Winner timeline epoch"))),
        expect: &[("AC3", "carries the winner's timeline epoch"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_kept_or_bumped_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "lowest", "smallest"))),
        expect: &[("AC3", "the winner's timeline epoch is kept")],
    },
    Denial {
        name: "ac3_denial_no_compared_unsigned_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "unsigned", "plainly"))),
        expect: &[("AC3", "compared unsigned")],
    },
    Denial {
        name: "ac3_denial_no_carries_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "(?i)carr", "bear"))),
        expect: &[("AC3", "which election message carries which field")],
    },
    Denial {
        name: "ac4_denial_counted_under_a_non_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "Badmagic"))),
        expect: &[("AC4", "counts under 'Badmagic'")],
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
        name: "ac4_denial_an_election_form_without_a_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| election epoch out of range: 0", r"\| drop and count under BadLength.*$", "| dropped silently |"))),
        expect: &[("AC4", "the election form 'election epoch out of range: 0")],
    },
    Denial {
        name: "ac4_denial_a_form_naming_no_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| message type 0", r"\| drop and count under BadLength.*$", "| dropped silently |"))),
        expect: &[("AC4", "names no member of the closed enumeration and argues no non-rejection")],
    },
    Denial {
        name: "ac4_denial_fewer_than_six_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, VALIDITY_HEAD, "| Case | Result |"))),
        expect: &[("AC4", "at least 6 are required")],
    },
    Denial {
        name: "ac4_denial_no_invalid_form_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Case | Result |"))),
        expect: &[("AC4", "no invalid-form table")],
    },
    Denial {
        name: "ac4_denial_no_oversize_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| oversize"))),
        expect: &[("AC4", "covers the 'oversize' case")],
    },
    Denial {
        name: "ac5_denial_no_owned_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| lit(t, "needs a message or field the set lacks", "needs more"))),
        expect: &[("AC5", "names epic 005 as the confirming owner")],
    },
    Denial {
        name: "ac5_denial_no_hand_off_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "(?i)reuse|confirm|differ", "keep"))),
        expect: &[("AC5", "reuse or confirm the hello's election epoch width")],
    },
    Denial {
        name: "behaviour_denial_ttl_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "TTL 1", "TTL one"))),
        expect: &[("Behaviour", "multicast with TTL 1")],
    },
    Denial {
        name: "behaviour_denial_cs5_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, r"CS5 \(40\)", "CS5"))),
        expect: &[("Behaviour", "CS5 (40) with C-13")],
    },
    Denial {
        name: "behaviour_denial_a_duration_not_restated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "400 ms", "four hundred ms"))),
        expect: &[("Behaviour", "C-33 (400 ms) is not restated")],
    },
    Denial {
        name: "behaviour_denial_a_timer_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| Believed holder epoch:", "| Window epoch:")),
        expect: &[("Behaviour", "is a timer"), ("AC6", "")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| strip_tags(t, r"^\*\*Why three values suffice"))),
        expect: &[("DoD", "carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| append(t, "**Open item 4 — x.** Still open `[decision]`."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    Denial {
        name: "dod_denial_a_table_in_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""))),
        expect: &[("DoD", "a table of 'Validity rules' follows no evidence-tagged paragraph")],
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
        name: "ac6_denial_no_candidacy_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"candidacy\"\n", "name = \"candidature\"\n"))),
        expect: &[("AC6", "declares no [[message]] for the election message 'Candidacy'")],
    },
    Denial {
        name: "ac6_denial_the_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "layer = \"control\"", "layer = \"media\"")),
        expect: &[("AC6", "has layer 'media'")],
    },
    Denial {
        name: "ac6_denial_the_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "dscp = 40", "dscp = 46")),
        expect: &[("AC6", "has dscp 46")],
    },
    Denial {
        name: "ac6_denial_the_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 67")),
        expect: &[("AC6", "has max_size 67")],
    },
    Denial {
        name: "ac6_denial_above_the_bound",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 1300")),
        expect: &[("AC6", "exceeds the 1252-byte bound")],
    },
    Denial {
        name: "ac6_denial_a_field_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "[[message.field]]\nname = \"election_epoch\"\n", "[[message.gone]]\nname = \"election_epoch\"\n")),
        expect: &[("AC6", "the markdown table has 14 rows")],
    },
    Denial {
        name: "ac6_denial_a_field_misnamed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"election_epoch\"", "name = \"epoch\"")),
        expect: &[("AC6", "toml field 'epoch' is not named after")],
    },
    Denial {
        name: "ac6_denial_a_field_offset",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 384\n", "offset = 385\n")),
        expect: &[("AC6", "offset 385 bits")],
    },
    Denial {
        name: "ac6_denial_a_field_width",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"election_epoch\"\noffset = 384\nwidth = 32", "name = \"election_epoch\"\noffset = 384\nwidth = 16")),
        expect: &[("AC6", "toml field election_epoch: width 16 bits")],
    },
    Denial {
        name: "ac6_denial_a_field_type",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"election_epoch\"\noffset = 384\nwidth = 32\ntype = \"u32\"", "name = \"election_epoch\"\noffset = 384\nwidth = 32\ntype = \"i32\"")),
        expect: &[("AC6", "toml field election_epoch: type i32")],
    },
    Denial {
        name: "behaviour_denial_a_toml_field_named_like_a_timer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"believed_holder_epoch\"", "name = \"believed_holder_delay\"")),
        expect: &[("Behaviour", "is named like a timer"), ("AC6", "")],
    },
    Denial {
        name: "ac6_denial_fields_not_summing_to_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"election_epoch\"\noffset = 384\nwidth = 32", "name = \"election_epoch\"\noffset = 384\nwidth = 40")),
        expect: &[("AC6", "compute a size other than max_size")],
    },
    Denial {
        name: "ac6_denial_a_named_reason_not_listed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "\"BadMagic\", ", "")),
        expect: &[("AC6", "does not list 'BadMagic'")],
    },
    Denial {
        name: "ac6_denial_a_listed_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "rejection_reasons = [", "rejection_reasons = [\"Bogus\", ")),
        expect: &[("AC6", "names 'Bogus', not in the closed enumeration")],
    },
    // `\w`: `²` glued to the reason defeats `\bBadMagic\b`.
    Denial {
        name: "py_denial_a_reason_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadMagic\u{b2}"))),
        expect: &[("AC4", "must be counted under BadMagic")],
    },
    // `\s`: U+001F is stripped by `clean_start`, so this is an open item.
    Denial {
        name: "py_denial_an_open_item_after_a_unit_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| append(t, "\u{1f}**Open item 4 — x.** Still open `[decision]`."))),
        expect: &[("REPERTOIRE", "")],
    },
    // `str.splitlines()`: U+2028 ends the shared table in its second row.
    Denial {
        name: "py_denial_a_line_separator_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{2028}drop and count under BadMagic"))),
        expect: &[("REPERTOIRE", "")],
    },
    // `str.splitlines()`: so does a form feed.
    Denial {
        name: "py_denial_a_form_feed_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| replace_in_line(t, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{c}drop and count under BadMagic"))),
        expect: &[("REPERTOIRE", "")],
    },
    // `int()` is unbounded.
    Denial {
        name: "py_denial_an_offset_beyond_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| 100000000000000000032 | 8 | u64 | Module id")),
        expect: &[("AC1", "at byte 100000000000000000032, expected 32"), ("AC6", "")],
    },
    // `\d`/`int()` read Arabic-Indic digits: ٣٣ is 33.
    Denial {
        name: "py_denial_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| \u{663}\u{663} | 8 | u64 | Module id")),
        expect: &[("REPERTOIRE", "")],
    },
    // `(?<![\w.])`: "v2 = Locked" states no value 2.
    Denial {
        name: "py_denial_a_time_state_value_after_a_letter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Candidacy$", |t| replace_in_line(t, r"^\| 65 \| 1 \| u8 \| Time state:", "2 = Locked", "v2 = Locked"))),
        expect: &[("AC3", "does not state 'N = meaning'")],
    },
    // `(?<![\d.])`: "1400 ms" does not state 400 ms.
    Denial {
        name: "py_denial_a_duration_inside_a_longer_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "400 ms", "1400 ms"))),
        expect: &[("Behaviour", "C-33 (400 ms) is not restated")],
    },
    // The subset reader keeps `"66"` a str.
    Denial {
        name: "py_denial_max_size_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = \"66\"")),
        expect: &[("AC6", "has max_size 66; the markdown table sums to 66")],
    },
    // The subset reader iterates a str's characters.
    Denial {
        name: "py_denial_rejection_reasons_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, RR, "rejection_reasons = \"BadMagic\"")),
        expect: &[("AC6", "does not list 'BadMagic'")],
    },
    // The subset reader cannot read `"""` strings.
    Denial {
        name: "py_denial_a_triple_quoted_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"election_epoch\"", "name = \"\"\"election_epoch\"\"\"")),
        expect: &[("AC6", "does not parse")],
    },
    // The subset reader lets a later duplicate key win.
    Denial {
        name: "py_denial_a_duplicate_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "dscp = 40\n", "dscp = 40\ndscp = 46\n")),
        expect: &[("AC6", "has dscp 46")],
    },
    // A number of rejection reasons is not iterable: TypeError.
    Denial {
        name: "py_denial_rejection_reasons_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, RR, "rejection_reasons = 5")),
        expect: &[("CRASH", "not iterable")],
    },
    // A str width times one is a str, which the sum rejects: TypeError.
    Denial {
        name: "py_denial_a_width_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "name = \"election_epoch\"\noffset = 384\nwidth = 32", "name = \"election_epoch\"\noffset = 384\nwidth = \"32\"")),
        expect: &[("CRASH", "adds a non-number"), ("AC6", "")],
    },
    // The 004 separator line: a letter in it drops the shared table.
    Denial {
        name: "py_denial_a_separator_with_a_letter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|---|-x-|"))),
        expect: &[("AC4", "at least 6 are required")],
    },
    // `re.I`: "Open ıtem" is an open item to Python.
    Denial {
        name: "py_denial_an_open_item_with_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| append(t, "**Open \u{131}tem 4 — x.** Still open `[decision]`."))),
        expect: &[("REPERTOIRE", "")],
    },
    // `\d`/`int()`: ٣٢ is 32.
    Denial {
        name: "py_accepts_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| \u{663}\u{662} | 8 | u64 | Module id")),
        expect: &[("REPERTOIRE", "")],
    },
    // The 24-character look-back counts characters, not bytes.
    Denial {
        name: "py_accepts_a_negation_within_24_characters_of_wide_ones",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "— never ForeignDomain", "— never ——————— ForeignDomain"))),
        expect: &[],
    },
    // `re.I`: "unsıgned" is "unsigned" to Python.
    Denial {
        name: "py_accepts_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| el(b, |t| replace_all(t, "unsigned", "uns\u{131}gned"))),
        expect: &[("REPERTOIRE", "")],
    },
    // A combining accent is no word character for Python: `\blive\b`
    // finds "live\u{301}".
    Denial {
        name: "py_accepts_a_combining_accent_after_live",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "live (C-115), the sender's", "live\u{301} (C-115), the sender's")),
        expect: &[("REPERTOIRE", "")],
    },
    // The subset reader: 40.0 and 0x28 equal 40.
    Denial {
        name: "py_accepts_a_float_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "dscp = 40\n", "dscp = 40.0\n")),
        expect: &[],
    },
    Denial {
        name: "py_accepts_a_hex_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "dscp = 40\n", "dscp = 0x28\n")),
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
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id", "| 32\u{1f} | 8 | u64 | Module id")),
        expect: &[("REPERTOIRE", "")],
    },
    // `T.get("max_size") != total` and `> bound` on 10^19.
    Denial {
        name: "num_denial_max_size_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 10000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 10^36: an int to Python, no clamp.
    Denial {
        name: "num_denial_max_size_of_10_36",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 1000000000000000000000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 10^40, beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // 2^127: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_just_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 170141183460469231731687303715884105728")),
        expect: &[("INT_LIMIT", "")],
    },
    // An underscore: as the subset reader reads it.
    Denial {
        name: "num_denial_max_size_with_an_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 6_6")),
        expect: &[],
    },
    // A leading `+`.
    Denial {
        name: "num_denial_max_size_with_a_plus",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = +66")),
        expect: &[],
    },
    // Leading zeros.
    Denial {
        name: "num_denial_max_size_with_leading_zeros",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 066")),
        expect: &[],
    },
    // 0x42 = 66.
    Denial {
        name: "num_denial_max_size_in_hex",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 0x42")),
        expect: &[],
    },
    // 66.0 == 66, but not `isinstance(_, int)`.
    Denial {
        name: "num_denial_max_size_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 66.0")),
        expect: &[],
    },
    // True: an int to isinstance, equal to 1.
    Denial {
        name: "num_denial_max_size_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = true")),
        expect: &[("AC6", "")],
    },
    // An empty string.
    Denial {
        name: "num_denial_max_size_empty",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = \"\"")),
        expect: &[("AC6", "")],
    },
    // No max_size.
    Denial {
        name: "num_denial_max_size_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66\n", "")),
        expect: &[("AC6", "")],
    },
    // A width beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_width_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // A width of 10^19: the sum stays an int.
    Denial {
        name: "num_denial_a_width_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 10000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 16.0 == 16; the sum becomes a float equal to max_size × 8.
    Denial {
        name: "num_denial_a_width_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 16.0")),
        expect: &[],
    },
    // `inf` in the sum.
    Denial {
        name: "num_denial_a_width_as_inf",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = inf")),
        expect: &[("AC6", "")],
    },
    // `nan` in the sum: never equal.
    Denial {
        name: "num_denial_a_width_as_nan",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = nan")),
        expect: &[("AC6", "")],
    },
    // 0.0 is falsy: `(width or 0)` is the int 0.
    Denial {
        name: "num_denial_a_width_of_zero_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 0.0")),
        expect: &[("AC6", "")],
    },
    // "" is falsy: `(width or 0)` is 0, no TypeError.
    Denial {
        name: "num_denial_a_width_as_an_empty_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = \"\"")),
        expect: &[("AC6", "")],
    },
    // True × 1 is the int 1.
    Denial {
        name: "num_denial_a_width_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = true")),
        expect: &[("AC6", "")],
    },
    // [16] × 1 is a list; `total_bits += [16]` raises TypeError.
    Denial {
        name: "num_denial_a_width_as_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = [16]")),
        expect: &[("AC6", ""), ("CRASH", "")],
    },
    // max_count 0 is falsy: `or 1`.
    Denial {
        name: "num_denial_a_max_count_of_zero",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 16\nmax_count = 0")),
        expect: &[],
    },
    // `field = 5` before the field tables (the reader decides).
    Denial {
        name: "num_denial_field_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 66\nfield = 5")),
        expect: &[("AC6", "")],
    },
    // A duplicate reason: `set()` drops it.
    Denial {
        name: "num_denial_duplicate_rejection_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [\"TooShort\", \"TooShort\", ")),
        expect: &[],
    },
    // `sorted()` of a str and an int: TypeError.
    Denial {
        name: "num_denial_rejection_reasons_mixed_types",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [\"Bogus\", 7, \"TooShort\", ")),
        expect: &[("CRASH", "")],
    },
    // `set()` of a list: unhashable, TypeError.
    Denial {
        name: "num_denial_rejection_reasons_with_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [[1], \"TooShort\", ")),
        expect: &[("CRASH", "")],
    },
    // Two ints and the members: the two ints sort among themselves.
    Denial {
        name: "num_denial_rejection_reasons_only_numbers",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [3, 1, \"TooShort\", ")),
        expect: &[("AC6", "")],
    },
    // C-17 at 10^40: INT_LIMIT.
    Denial {
        name: "num_denial_the_bound_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 10 000 000 000 000 000 000 000 000 000 000 000 000 000 bytes"))),
        expect: &[("INT_LIMIT", "")],
    },
    // C-17 at 10^19.
    Denial {
        name: "num_denial_the_bound_of_10_19",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 10 000 000 000 000 000 000 bytes"))),
        expect: &[],
    },
    // int("040") is 40.
    Denial {
        name: "num_denial_the_cs5_value_with_leading_zeros",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "control CS5 (40)", "control CS5 (040)"))),
        expect: &[],
    },
    // C-33 `400² ms`: const_dur finds no number before the unit.
    Denial {
        name: "num_denial_a_duration_with_a_superscript",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-33 \|", r"\| 400 ms \|", "| 400² ms |"))),
        expect: &[("Behaviour", "")],
    },
    // `[int(x) for x in re.findall(r"\d+", row[0])]` beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_election_block_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 2 to 15 | election messages |", "| 2 to 10000000000000000000000000000000000000000 | election messages |"))),
        expect: &[("INT_LIMIT", "")],
    },
    // int("015") is 15.
    Denial {
        name: "num_denial_the_election_block_with_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 2 to 15 | election messages |", "| 02 to 015 | election messages |"))),
        expect: &[],
    },
    // A second election row: the last one wins.
    Denial {
        name: "num_denial_a_duplicate_election_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 2 to 15 | election messages |", "| 2 to 15 | election messages |\n| 40 to 41 | election messages |"))),
        expect: &[("AC1", "")],
    },
    // `²` is not `\d`: the block is still 2 to 15.
    Denial {
        name: "num_denial_the_election_block_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| hello(b, |t| lit(t, "| 2 to 15 | election messages |", "| 2 to 15² | election messages |"))),
        expect: &[],
    },
    // A type code of 10^19: outside the block.
    Denial {
        name: "num_denial_a_type_code_of_10_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 6 | 1 | u8 | Message type: 2 = candidacy (Control message type codes) | 2 only |", "| 6 | 1 | u8 | Message type: 2 = candidacy (Control message type codes) | 10000000000000000000 only |")),
        expect: &[("AC1", "")],
    },
    // A type code beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_type_code_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 6 | 1 | u8 | Message type: 2 = candidacy (Control message type codes) | 2 only |", "| 6 | 1 | u8 | Message type: 2 = candidacy (Control message type codes) | 10000000000000000000000000000000000000000 only |")),
        expect: &[("INT_LIMIT", "")],
    },
    // int("002") is 2.
    Denial {
        name: "num_denial_a_type_code_with_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 6 | 1 | u8 | Message type: 2 = candidacy (Control message type codes) | 2 only |", "| 6 | 1 | u8 | Message type: 2 = candidacy (Control message type codes) | 002 only |")),
        expect: &[],
    },
    // The stated maximum beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_stated_size_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "maximum size is 66 bytes", "maximum size is 10000000000000000000000000000000000000000 bytes")),
        expect: &[("INT_LIMIT", "")],
    },
    // The stated maximum of 10^19 differs from the sum.
    Denial {
        name: "num_denial_a_stated_size_of_10_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "maximum size is 66 bytes", "maximum size is 10000000000000000000 bytes")),
        expect: &[("AC1", "")],
    },
    // `NUM` digit groups: 1066.
    Denial {
        name: "num_denial_a_stated_size_in_groups",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "maximum size is 66 bytes", "maximum size is 1 066 bytes")),
        expect: &[("AC1", "")],
    },
    // `66²`: NUM stops before `²`, `\s*bytes` fails.
    Denial {
        name: "num_denial_a_stated_size_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "maximum size is 66 bytes", "maximum size is 66² bytes")),
        expect: &[("AC1", "")],
    },
    // `(?<![\w.])(\d{1,3})\s*=` does not read 1000 =; the values are 1 and 2.
    Denial {
        name: "num_denial_a_time_state_value_of_four_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "Time state: 0 = time unusable", "Time state: 1000 = time unusable")),
        expect: &[("AC3", "")],
    },
    // int("00") is 0.
    Denial {
        name: "num_denial_a_time_state_value_with_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "Time state: 0 = time unusable", "Time state: 00 = time unusable")),
        expect: &[],
    },
    // `²` is not `\w`: the look-behind lets 0 through.
    Denial {
        name: "num_denial_a_time_state_value_after_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "Time state: 0 = time unusable", "Time state: ²0 = time unusable")),
        expect: &[("AC3", "")],
    },
    // An offset beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_an_offset_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id (C-11) of the sender", "| 10000000000000000000000000000000000000000 | 8 | u64 | Module id (C-11) of the sender")),
        expect: &[("INT_LIMIT", "")],
    },
    // An empty Offset cell.
    Denial {
        name: "num_denial_an_empty_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id (C-11) of the sender", "|  | 8 | u64 | Module id (C-11) of the sender")),
        expect: &[("AC1", ""), ("AC3", ""), ("AC6", "")],
    },
    // A duplicate row: the layout no longer tiles.
    Denial {
        name: "num_denial_a_duplicate_layout_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 32 | 8 | u64 | Module id (C-11) of the sender", "| 32 | 8 | u64 | Module id (C-11) of the sender | any 64-bit value |\n| 32 | 8 | u64 | Module id (C-11) of the sender")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // A member code beyond i128 (`int(row[0])`): INT_LIMIT.
    Denial {
        name: "num_denial_a_rejection_code_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| lit(t, "| 0 | TooShort |", "| 10000000000000000000000000000000000000000 | TooShort |"))),
        expect: &[],
    },
    // A duplicate member: the dict keeps 13 names.
    Denial {
        name: "num_denial_a_duplicate_rejection_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| lit(t, "| 1 | BadMagic |", "| 0 | TooShort | x | control |\n| 1 | BadMagic |"))),
        expect: &[],
    },
    // 2^63.
    Denial {
        name: "num_denial_a_type_code_of_2_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| 9223372036854775808 only |")),
        expect: &[("AC1", "")],
    },
    // 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_a_type_code_of_45_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| 100000000000000000000000000000000000000000000 only |")),
        expect: &[("INT_LIMIT", "")],
    },
    // 0: outside the block.
    Denial {
        name: "num_denial_a_type_code_of_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| 0 only |")),
        expect: &[("AC1", "")],
    },
    // `\d+` reads 0.
    Denial {
        name: "num_denial_a_type_code_with_an_underscore",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| 0_2 only |")),
        expect: &[("AC1", "")],
    },
    // `µ` before the digit.
    Denial {
        name: "num_denial_a_type_code_after_a_micro_sign",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| cand(b, "| 2 only |", "| µ2 only |")),
        expect: &[],
    },
    // 2^63.
    Denial {
        name: "num_denial_max_size_of_2_63",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 9223372036854775808")),
        expect: &[("AC6", "")],
    },
    // 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_of_45_digits",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| cand_toml(b, "max_size = 66", "max_size = 100000000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_the_bound_of_45_digits",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 1 252 bytes", "at most 100000000000000000000000000000000000000000000 bytes"))),
        expect: &[("INT_LIMIT", "")],
    },
];
