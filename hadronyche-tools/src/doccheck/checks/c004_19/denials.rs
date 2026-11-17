//! The denial list of 004-19: the real documents with one edit, and what
//! the port (and the old script, run by `doccheck-equiv 004-19`) must
//! report. One or more per `fail` site of the script, then the denials aimed
//! at Python semantics (`py_denial_*`), then the acceptance probes
//! (`py_accepts_*`, no expectation: the port must pass, as the script
//! does). Consumed by `tests/doc_004_19.rs`, one test each.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, within_section};
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

fn sync(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Sync request and reply$", edit)
}

fn sub(t: &str, title: &str, edit: impl FnOnce(&str) -> String) -> String {
    sync(t, |s| within_section(s, title, edit))
}

fn req(t: &str, from: &str, to: &str) -> String {
    sub(t, "^Request$", |s| lit(s, from, to))
}

fn rep(t: &str, from: &str, to: &str) -> String {
    sub(t, "^Reply$", |s| lit(s, from, to))
}

fn valid(t: &str, line: &str, from: &str, to: &str) -> String {
    sub(t, "^Validity rules$", |s| replace_in_line(s, line, from, to))
}

fn req_toml(t: &str, from: &str, to: &str) -> String {
    after(t, "name = \"sync request\"\n", from, to)
}

/// `re.sub` with a regex for the fixtures.
fn regex_replace(text: &str, pattern: &str, to: &str) -> String {
    rx(pattern).replace_all(text, to)
}

/// `pattern` replaced on the table rows only.
fn rows_only(t: &str, pattern: &str, to: &str) -> String {
    t.split('\n').map(|l| if l.trim_start().starts_with('|') { regex_replace(l, pattern, to) } else { l.to_string() }).collect::<Vec<_>>().join("\n")
}

const SIZE_SENTENCE: &str = "Request and reply are both 74 bytes long";
const TAGS: &str = r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?";
const RR: &str = "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"ReservedNonZero\"]";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "dod_denial_a_constant_unreadable",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-36 \|", "50 per second", "fifty a second"))),
        expect: &[("DoD", "constants row for req_rate could not be read")],
    },
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", ""), ("DoD", ""), ("W1", ""), ("W2", ""), ("W4", ""), ("W5", "")],
    },
    Denial {
        name: "ac5_denial_no_rejection_reasons_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Rejection reasons$", "Refusals"))),
        expect: &[("AC5", "no 'Rejection reasons' section"), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_the_enumeration_not_thirteen",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| delete_row(t, r"^\| 9 \| BadPacketIndex \|"))),
        expect: &[("AC5", "has 12 names, not 13")],
    },
    Denial {
        name: "ac1_denial_no_sync_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Sync request and reply$", "Sync exchange"))),
        expect: &[("AC1", "no 'Sync request' section heading"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("DoD", ""), ("W1", ""), ("W2", ""), ("W4", ""), ("W5", "")],
    },
    Denial {
        name: "ac1_denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| {
        let head = t.split('\n').next().expect("heading").to_string();
        format!("{head}\n\nTo be filled by 004-19.\n")
    })),
        expect: &[("AC1", "still the 004-19 placeholder"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("DoD", ""), ("W1", ""), ("W2", ""), ("W4", ""), ("W5", "")],
    },
    Denial {
        name: "ac1_denial_no_reply_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Reply$", |t| lit(t, "| Offset (bytes) |", "| Position |"))),
        expect: &[("AC1", "needs a request field table and a reply field table"), ("AC4", "")],
    },
    Denial {
        name: "ac1_denial_fields_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| 40 | 8 | u64 | Receive timestamp", "| 41 | 8 | u64 | Receive timestamp")),
        expect: &[("AC1", "does not start where the previous field ends"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_fields_not_whole_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| 72 | 2 | u16 | Timeline epoch", "| 72 | 12 bits | u16 | Timeline epoch")),
        expect: &[("AC1", "not whole bytes"), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_sizes_that_differ",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 40 | 34 | pad |", "| 40 | 33 | pad |")),
        expect: &[("AC1", "do not have the same byte size"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_stated_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"(\d+)(\s*-?\s*)bytes?\b", "${1}${2}octets"))),
        expect: &[("AC1", "states no byte size")],
    },
    Denial {
        name: "ac1_denial_a_stated_size_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| lit(t, SIZE_SENTENCE, "Request and reply are both 75 bytes long"))),
        expect: &[("AC1", "the stated byte size")],
    },
    Denial {
        name: "ac1_denial_no_padding_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 40 | 34 | pad | Padding: brings the request to the reply's size, the room the reply's receive and send timestamps, holder module id, epoch origin and timeline epoch take; senders write zero (Reserved fields, general rule) | 0 only |", "| 40 | 34 | u8 | Filler: brings the request to the reply's size | any value |")),
        expect: &[("AC1", "no explicit padding or reserved field"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_padding_without_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "epoch origin and timeline epoch take; senders write zero (Reserved fields, general rule) | 0 only |", "epoch origin and timeline epoch take (Reserved fields, general rule) | any value |")),
        expect: &[("AC1", "the request padding has no required value of zero")],
    },
    Denial {
        name: "ac2_denial_no_echo_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| Echoed send timestamp: the request's send timestamp, echoed verbatim;", "| Copied send timestamp: the request's send timestamp, copied verbatim;")),
        expect: &[("AC2", "has no row that echoes the request's send timestamp")],
    },
    Denial {
        name: "ac2_denial_an_echo_not_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Reply$", |t| {
        let t = lit(t, "| 32 | 8 | u64 | Echoed send timestamp", "| 32 | 4 | u64 | Echoed send timestamp");
        lit(&t, "| 40 | 8 | u64 | Receive timestamp", "| 36 | 12 | u64 | Receive timestamp")
    })),
        expect: &[("AC2", "must be 64 bits"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_no_epoch_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Reply$", |t| rows_only(t, r"(?i)epoch", "line"))),
        expect: &[("AC2", "has no timeline epoch row")],
    },
    Denial {
        name: "ac2_denial_an_epoch_not_16_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Reply$", |t| {
        let t = lit(t, "| 72 | 2 | u16 | Timeline epoch", "| 72 | 4 | u16 | Timeline epoch");
        lit(&t, "| 64 | 8 | i64 | Epoch origin", "| 64 | 8 | i64 | Origin")
    })),
        expect: &[("AC2", "the reply's timeline epoch must be"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_no_module_id_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Reply$", |t| rows_only(t, r"(?i)module\s+id", "node"))),
        expect: &[("AC2", "has no holder module id row")],
    },
    Denial {
        name: "ac2_denial_a_module_id_not_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Reply$", |t| lit(&lit(t, "| 56 | 8 | u64 | Holder module id", "| 56 | 4 | u64 | Holder module id"), "| 64 | 8 | i64 | Epoch origin", "| 60 | 12 | i64 | Epoch origin"))),
        expect: &[("AC2", "the holder module id must be"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_no_domain_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Domain id (C-11); 0 is the unconfigured domain", "| Realm id (C-11); 0 is the unconfigured realm")),
        expect: &[("AC2", "the request table has no domain id row")],
    },
    Denial {
        name: "ac2_denial_a_domain_not_16_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Request$", |t| lit(&lit(t, "| 4 | 2 | u16 | Domain id", "| 4 | 1 | u16 | Domain id"), "| 6 | 1 | u8 | Message type", "| 5 | 2 | u8 | Message type"))),
        expect: &[("AC2", "the request domain id must be"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_a_domain_without_unconfigured",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |", "| Domain id (C-11) | 1 to 65535 |")),
        expect: &[("AC2", "does not state that 0 is the unconfigured domain")],
    },
    Denial {
        name: "ac3_denial_not_stateless",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, "(?i)stateless", "simple"))),
        expect: &[("AC3", "does not state that the exchange is stateless")],
    },
    Denial {
        name: "ac3_denial_no_echo_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| t.split('\n').map(|l| if l.trim_start().starts_with('|') { l.to_string() } else { regex_replace(l, r"(?i)\becho", "copy") }).collect::<Vec<_>>().join("\n"))),
        expect: &[("AC3", "does not state that the reply echoes")],
    },
    Denial {
        name: "ac3_denial_a_per_request_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| Holder module id (C-11): the module id of the holder that answers", "| Holder module id (C-11): the module id of the holder that answers, a token")),
        expect: &[("AC3", "only per-request holder state could fill ('token')")],
    },
    Denial {
        name: "ac4_denial_no_decision_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| t.split('\n').map(|l| if l.trim_start().starts_with('|') { l.to_string() } else { regex_replace(l, r"(?i)authentication|reservation", "scheme") }).collect::<Vec<_>>().join("\n"))),
        expect: &[("AC4", "no [decision]-tagged paragraph states the authentication-reservation choice")],
    },
    Denial {
        name: "ac4_denial_no_carry_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Authentication reservation$", |t| {
        let start = t.find("Sync messages carry the authentication reservation").expect("paragraph");
        let end = t[start..].find("\n\n").map_or(t.len(), |k| start + k);
        format!("{}Sync messages hold the authentication reservation of C-12 `[decision]`. Falsifier: a later sync message needs other room.{}", &t[..start], &t[end..])
    })),
        expect: &[("AC4", "does not say whether sync messages carry")],
    },
    Denial {
        name: "ac4_denial_no_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, "(?i)falsifier", "test"))),
        expect: &[("AC4", "has no falsifier"), ("DoD", "")],
    },
    Denial {
        name: "ac4_denial_carried_without_a_zero_scheme",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 7 | 1 | u8 | Authentication scheme (C-12); 0 = none in v1 (C-115) | 0 only |", "| 7 | 1 | u8 | Authentication scheme (C-12) | any |")),
        expect: &[("AC4", "the request table carries the reservation but has no 8-bit scheme field")],
    },
    Denial {
        name: "ac4_denial_carried_without_a_live_counter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Message counter (C-12): live (C-115), incremented", "| Message counter (C-12): kept (C-115), incremented")),
        expect: &[("AC4", "the request table carries the reservation but has no 64-bit live message counter")],
    },
    Denial {
        name: "ac4_denial_carried_without_a_zero_tag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Authentication tag (C-12): 128 zero bits in v1 (C-115) | 0 only |", "| Authentication tag (C-12): reserved in v1 (C-115) | any |")),
        expect: &[("AC4", "the request table carries the reservation but has no 128-bit zero tag")],
    },
    Denial {
        name: "ac4_denial_not_carried_with_its_fields",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Authentication reservation$", |t| lit(t, "Sync messages carry the authentication reservation of C-12", "Sync messages do not carry the authentication reservation of C-12"))),
        expect: &[("AC4", "the decision says sync messages do not carry the reservation")],
    },
    Denial {
        name: "ac5_denial_no_form_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |", "| Case | Result |"))),
        expect: &[("AC5", "no table of invalid forms")],
    },
    Denial {
        name: "ac5_denial_fewer_than_six_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "\n| foreign domain", "\n\n| foreign domain"))),
        expect: &[("AC5", "at least 6 are required")],
    },
    Denial {
        name: "ac5_denial_an_exact_reason_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", "BadMagic", "BadLength")),
        expect: &[("AC5", "must be counted under BadMagic")],
    },
    Denial {
        name: "ac5_denial_a_non_holder_without_a_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| message type not the one", r"\| drop and count under BadLength.*$", "| dropped silently |")),
        expect: &[("AC5", "has no member reason in its outcome")],
    },
    Denial {
        name: "ac5_denial_no_wrong_epoch_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(&delete_row(t, r"^\| reply with wrong epoch"), r"^\| duplicate or stale reply"))),
        expect: &[("AC5", "no invalid form for WrongEpoch")],
    },
    Denial {
        name: "ac5_denial_no_non_holder_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(&delete_row(t, r"^\| reply from a non-holder"), r"^\| message type not the one"))),
        expect: &[("AC5", "no invalid form for a reply from a non-holder")],
    },
    Denial {
        name: "ac5_denial_a_short_form_not_too_short",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| datagram too short", "TooShort", "BadLength")),
        expect: &[("AC5", "it must be counted under TooShort")],
    },
    Denial {
        name: "ac5_denial_a_size_form_mapped_to_too_short",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| datagram size not equal", "BadLength", "TooShort")),
        expect: &[("AC5", "it is mapped to TooShort")],
    },
    Denial {
        name: "ac5_denial_no_bad_length_size_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| delete_row(t, r"^\| datagram size not equal"))),
        expect: &[("AC5", "no invalid form for a size that is not the fixed size counted under BadLength")],
    },
    Denial {
        name: "ac5_denial_an_invented_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", "BadMagic", "BadMagic or WrongMagic")),
        expect: &[("AC5", "the outcome names 'WrongMagic'")],
    },
    Denial {
        name: "ac5_denial_counted_under_a_non_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| append(t, "Anything else is counted under Wrongish `[decision]`."))),
        expect: &[("AC5", "counts a rejection under 'Wrongish'")],
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
        name: "ac6_denial_no_sync_request",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"sync request\"\n", "name = \"sync ask\"\n"))),
        expect: &[("AC6", "declares no [[message]] named 'sync request'")],
    },
    Denial {
        name: "ac6_denial_no_sync_reply",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| lit(t, "name = \"sync reply\"\n", "name = \"sync answer\"\n"))),
        expect: &[("AC6", "declares no [[message]] named 'sync reply'")],
    },
    Denial {
        name: "ac6_denial_the_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "layer = \"sync\"", "layer = \"control\"")),
        expect: &[("AC6", "must have layer = \"sync\"")],
    },
    Denial {
        name: "ac6_denial_the_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 48", "dscp = 46")),
        expect: &[("AC6", "must have dscp = 48")],
    },
    Denial {
        name: "ac1_denial_no_integer_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 74.0")),
        expect: &[("AC1", "has no integer max_size")],
    },
    Denial {
        name: "ac1_denial_toml_fields_not_contiguous",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 32\n", "offset = 33\n")),
        expect: &[("AC1", "is not contiguous"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_toml_fields_not_summing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 75")),
        expect: &[("AC1", "fields sum to")],
    },
    Denial {
        name: "ac1_denial_toml_size_against_markdown",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 75")),
        expect: &[("AC1", "differs from the markdown size")],
    },
    Denial {
        name: "ac6_denial_a_listed_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "rejection_reasons = [", "rejection_reasons = [\"Bogus\", ")),
        expect: &[("AC6", "names 'Bogus', which is not a member")],
    },
    Denial {
        name: "ac6_denial_a_field_count_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "[[message.field]]\nname = \"magic\"", "[[message.gone]]\nname = \"magic\"")),
        expect: &[("AC6", "the markdown table has 9 fields, wire.toml 8"), ("AC1", "")],
    },
    Denial {
        name: "ac6_denial_a_field_width_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"magic\"\noffset = 0\nwidth = 16", "name = \"magic\"\noffset = 0\nwidth = 8")),
        expect: &[("AC6", "does not match wire.toml field"), ("AC1", "")],
    },
    Denial {
        name: "ac6_denial_a_field_type_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"magic\"\noffset = 0\nwidth = 16\ntype = \"u16\"", "name = \"magic\"\noffset = 0\nwidth = 16\ntype = \"i16\"")),
        expect: &[("AC6", "markdown type 'u16' at bit 0 does not match wire.toml type 'i16'")],
    },
    Denial {
        name: "ac1_denial_no_toml_pad_field",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| { let i = t.find("name = \"sync request\"").expect("request"); let j = i + t[i..].find("type = \"pad\"").expect("pad"); format!("{}type = \"blob\"{}", &t[..j], &t[j + 12..]) })),
        expect: &[("AC1", "has no pad field of width > 0"), ("AC6", "")],
    },
    Denial {
        name: "w4_denial_a_toml_rate_field",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"", "name = \"send_rate\"")),
        expect: &[("W4", "names a DSCP or rate field ('send_rate')")],
    },
    Denial {
        name: "ac6_denial_an_exact_reason_not_listed",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| replace_all(t, r#""BadMagic", "#, ""))),
        expect: &[("AC6", "rejection_reasons in wire.toml miss: BadMagic")],
    },
    Denial {
        name: "w1_denial_fewer_than_three_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Request$", |t| {
        let start = t.find("| 4 | 2 | u16 | Domain id").expect("row");
        let end = t[start..].find("\n\n").map_or(t.len(), |k| start + k);
        format!("{}{}", &t[..start], &t[end..])
    })),
        expect: &[("W1", "the request table has fewer than three header fields"), ("AC1", ""), ("AC2", ""), ("AC4", ""), ("AC6", "")],
    },
    Denial {
        name: "w1_denial_no_magic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |", "| Magic, the ASCII letters M and N, fixed by 004-04 | fixed |")),
        expect: &[("W1", "does not start with the 2-byte magic 0x4d4e")],
    },
    Denial {
        name: "w1_denial_no_version",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "v1 packs 1.0 as 0x0100 (004-04) | major 1, any minor (Versioning, Receiver behaviour); senders write 0x0100 |", "(004-04) | major 1, any minor (Versioning, Receiver behaviour) |")),
        expect: &[("W1", "is not the 2-byte v1 version 0x0100")],
    },
    Denial {
        name: "w1_denial_no_message_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| Message type: 1 = sync request (Message type codes)", "| Kind: 1 = sync request (Kinds)")),
        expect: &[("W1", "has no message type row")],
    },
    Denial {
        name: "w2_denial_no_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"(?i)nanosecond|\bns\b", "tick"))),
        expect: &[("W2", "unit (nanoseconds) is not stated")],
    },
    Denial {
        name: "w2_denial_no_clock",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, "(?i)monotonic", "steady"))),
        expect: &[("W2", "clock (monotonic) is not stated")],
    },
    Denial {
        name: "w2_denial_no_slice_header",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"(?i)slice[\s-]header", "frame head"))),
        expect: &[("W2", "does not relate its timestamps")],
    },
    Denial {
        name: "w2_denial_no_relation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"(?i)same\s+(?:unit|clock|basis|as)|consistent\s+with|identical\s+to|explicitly\s+different|differs?\s+from|different\s+from", "like"))),
        expect: &[("W2", "neither stated as consistent with")],
    },
    Denial {
        name: "w4_denial_no_cs6",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"CS6 \(48\)", "CS6"))),
        expect: &[("W4", "does not restate the DSCP as CS6 (48)")],
    },
    Denial {
        name: "w4_denial_no_ipv4_unicast",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, "(?i)unicast", "direct"))),
        expect: &[("W4", "does not restate IPv4 unicast")],
    },
    Denial {
        name: "w4_denial_no_request_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"\b50\b", "fifty"))),
        expect: &[("W4", "does not restate 50 requests per second")],
    },
    Denial {
        name: "w4_denial_no_accepted_rates",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"10 to 200 requests per second", "ten to two hundred requests per second"))),
        expect: &[("W4", "does not restate the accepted 10 to 200")],
    },
    Denial {
        name: "w4_denial_a_table_naming_a_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| Receive timestamp: the holder's monotonic clock", "| Receive timestamp: the holder's rate clock")),
        expect: &[("W4", "names a DSCP or a rate")],
    },
    Denial {
        name: "w5_denial_no_per_os_slot",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"(?i)per[- ]OS", "each system"))),
        expect: &[("W5", "no note slot for per-OS")],
    },
    Denial {
        name: "w5_denial_no_c131",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, "C-131", "C-999"))),
        expect: &[("W5", "does not name C-131")],
    },
    Denial {
        name: "w5_denial_no_regime_stamp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, r"(?i)regime\s+stamp", "report"))),
        expect: &[("W5", "lives in the regime stamp")],
    },
    Denial {
        name: "w5_denial_a_table_naming_a_source",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| Receive timestamp: the holder's monotonic clock", "| Receive timestamp: the holder's kernel clock")),
        expect: &[("W5", "a sync table names a receive-timestamp source")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Exchange rules$", |t| replace_all(t, TAGS, ""))),
        expect: &[("DoD", "a paragraph of the Sync request section carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_a_table_after_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Request$", |t| replace_all(t, TAGS, ""))),
        expect: &[("DoD", "a table of the Sync request section carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| append(&replace_all(t, r"(?i)\bowner\b", "lead"), "This open item stays open `[decision]` here now."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    // `\w`: `²` glued to the reason defeats `\bBadMagic\b`.
    Denial {
        name: "py_denial_a_reason_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", "BadMagic", "BadMagic\u{b2}")),
        expect: &[("AC5", "must be counted under BadMagic")],
    },
    // The 25-character look-back has no `no`: "no WrongMagic" is a name.
    Denial {
        name: "py_denial_a_reason_after_no",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", "BadMagic", "BadMagic, no WrongMagic")),
        expect: &[("AC5", "the outcome names 'WrongMagic'")],
    },
    // `str.splitlines()`: U+2028 ends the form table in its second row.
    Denial {
        name: "py_denial_a_line_separator_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{2028}drop and count under BadMagic")),
        expect: &[("REPERTOIRE", "")],
    },
    // `str.splitlines()`: a form feed too.
    Denial {
        name: "py_denial_a_form_feed_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{c}drop and count under BadMagic")),
        expect: &[("REPERTOIRE", "")],
    },
    // `int()` is unbounded: a 21-digit offset is read and does not tile.
    Denial {
        name: "py_denial_an_offset_beyond_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| 40 | 8 | u64 | Receive timestamp", "| 100000000000000000040 | 8 | u64 | Receive timestamp")),
        expect: &[("AC1", "does not start where the previous field ends"), ("AC6", "")],
    },
    // `\d`/`int()` read Arabic-Indic digits: ٤١ is 41.
    Denial {
        name: "py_denial_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| 40 | 8 | u64 | Receive timestamp", "| \u{664}\u{661} | 8 | u64 | Receive timestamp")),
        expect: &[("REPERTOIRE", "")],
    },
    // The subset reader keeps `"74"` a str: no integer max_size.
    Denial {
        name: "py_denial_max_size_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = \"74\"")),
        expect: &[("AC1", "has no integer max_size")],
    },
    // The subset reader cannot read `"""` strings.
    Denial {
        name: "py_denial_a_triple_quoted_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"magic\"", "name = \"\"\"magic\"\"\"")),
        expect: &[("AC6", "does not parse")],
    },
    // The subset reader lets a later duplicate key win.
    Denial {
        name: "py_denial_a_duplicate_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 48\n", "dscp = 48\ndscp = 46\n")),
        expect: &[("AC6", "must have dscp")],
    },
    // A str offset beside numbers cannot be sorted: TypeError.
    Denial {
        name: "py_denial_an_offset_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 32\n", "offset = \"32\"\n")),
        expect: &[("CRASH", "cannot be sorted")],
    },
    // A list rejection reason is unhashable: TypeError.
    Denial {
        name: "py_denial_a_reason_as_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, RR, "rejection_reasons = [[\"TooShort\"]]")),
        expect: &[("CRASH", "unhashable")],
    },
    // The 004 separator line: a letter in it drops the form table.
    Denial {
        name: "py_denial_a_separator_with_a_letter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|---|-x-|"))),
        expect: &[("AC5", "no table of invalid forms")],
    },
    // `re.I`: Python matches `i` with `ı`, so "open ıtem" is an open item.
    Denial {
        name: "py_denial_an_open_item_with_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| append(&replace_all(t, r"(?i)\bowner\b", "lead"), "This open \u{131}tem stays open `[decision]` here now."))),
        expect: &[("REPERTOIRE", "")],
    },
    // `\d`/`int()`: ٤٠ is 40.
    Denial {
        name: "py_accepts_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rep(b, "| 40 | 8 | u64 | Receive timestamp", "| \u{664}\u{660} | 8 | u64 | Receive timestamp")),
        expect: &[("REPERTOIRE", "")],
    },
    // The 25-character look-back counts characters: "never" within 25
    // characters of wide dashes negates the name.
    Denial {
        name: "py_accepts_a_negation_within_25_characters_of_wide_ones",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", "BadMagic", "BadMagic, never ———— WrongMagic")),
        expect: &[],
    },
    // `re.I`: "monotonıc" is "monotonic" to Python.
    Denial {
        name: "py_accepts_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| replace_all(t, "monotonic", "monoton\u{131}c"))),
        expect: &[("REPERTOIRE", "")],
    },
    // The subset reader: 48.0 and 0x30 equal 48.
    Denial {
        name: "py_accepts_a_float_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 48\n", "dscp = 48.0\n")),
        expect: &[],
    },
    Denial {
        name: "py_accepts_a_hex_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 48\n", "dscp = 0x30\n")),
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
        edit: DenialEdit::Text(|b| rep(b, "| 40 | 8 | u64 | Receive timestamp", "| 40\u{1f} | 8 | u64 | Receive timestamp")),
        expect: &[("REPERTOIRE", "")],
    },
    // A combining accent is no word character for Python: `\bBadMagic\b`
    // still finds it.
    Denial {
        name: "py_accepts_a_combining_accent_after_a_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| valid(b, r"^\| bad magic", "BadMagic", "BadMagic\u{301}")),
        expect: &[("REPERTOIRE", "")],
    },
    // `pos != ms * 8` and `ms != size_md` on 10^19.
    Denial {
        name: "num_denial_max_size_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 10000000000000000000")),
        expect: &[("AC1", "")],
    },
    // 10^40: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // 2^127: INT_LIMIT.
    Denial {
        name: "num_denial_max_size_just_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 170141183460469231731687303715884105728")),
        expect: &[("INT_LIMIT", "")],
    },
    // 74.0 is not `isinstance(_, int)`: "no integer max_size".
    Denial {
        name: "num_denial_max_size_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 74.0")),
        expect: &[("AC1", "")],
    },
    // True is an int (1): the sums differ.
    Denial {
        name: "num_denial_max_size_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = true")),
        expect: &[("AC1", "")],
    },
    // Leading zeros.
    Denial {
        name: "num_denial_max_size_with_leading_zeros",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 074")),
        expect: &[],
    },
    // 0x4a = 74.
    Denial {
        name: "num_denial_max_size_in_hex",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 0x4a")),
        expect: &[],
    },
    // An underscore.
    Denial {
        name: "num_denial_max_size_with_an_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74", "max_size = 7_4")),
        expect: &[],
    },
    // No max_size.
    Denial {
        name: "num_denial_max_size_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "max_size = 74\n", "")),
        expect: &[("AC1", "")],
    },
    // A sort key beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_an_offset_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // A sort key of 10^19: sorted last; nothing tiles.
    Denial {
        name: "num_denial_an_offset_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = 10000000000000000000")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // 256.0 sorts and compares equal to 256; the sum becomes a float.
    Denial {
        name: "num_denial_an_offset_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = 256.0")),
        expect: &[],
    },
    // A list key among ints: `sorted` raises TypeError.
    Denial {
        name: "num_denial_an_offset_as_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = [256]")),
        expect: &[("CRASH", "")],
    },
    // True sorts as 1.
    Denial {
        name: "num_denial_an_offset_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = true")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // nan as a sort key: every comparison is False.
    Denial {
        name: "num_denial_an_offset_as_nan",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = nan")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // No offset: the key is 0, the field sorts first.
    Denial {
        name: "num_denial_an_offset_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256\n", "name = \"send_timestamp\"\n")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // Two fields at offset 0: the sort is stable.
    Denial {
        name: "num_denial_a_duplicate_offset",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "name = \"send_timestamp\"\noffset = 256", "name = \"send_timestamp\"\noffset = 0")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // `0 + "16"`: TypeError.
    Denial {
        name: "num_denial_a_width_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = \"16\"")),
        expect: &[("CRASH", "")],
    },
    // A width beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_width_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    // 16.0: the position becomes a float.
    Denial {
        name: "num_denial_a_width_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 0\nwidth = 16", "offset = 0\nwidth = 16.0")),
        expect: &[],
    },
    // `"272" > 0`: TypeError (after `0 + ...` raised first).
    Denial {
        name: "num_denial_a_pad_width_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 320\nwidth = 272", "offset = 320\nwidth = \"272\"")),
        expect: &[("CRASH", "")],
    },
    // A pad of width 0: no pad field of width > 0.
    Denial {
        name: "num_denial_a_pad_width_of_zero",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 320\nwidth = 272", "offset = 320\nwidth = 0")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // A negative pad width.
    Denial {
        name: "num_denial_a_pad_width_negative",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 320\nwidth = 272", "offset = 320\nwidth = -272")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // A pad of 10^19 bits.
    Denial {
        name: "num_denial_a_pad_width_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "offset = 320\nwidth = 272", "offset = 320\nwidth = 10000000000000000000")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // 48.0 == 48.
    Denial {
        name: "num_denial_the_dscp_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 48", "dscp = 48.0")),
        expect: &[],
    },
    // "48" != 48.
    Denial {
        name: "num_denial_the_dscp_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "dscp = 48", "dscp = \"48\"")),
        expect: &[("AC6", "")],
    },
    // `set.update` of a list item: TypeError.
    Denial {
        name: "num_denial_rejection_reasons_with_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [[1], \"TooShort\", ")),
        expect: &[("CRASH", "")],
    },
    // An int reason: not a member.
    Denial {
        name: "num_denial_rejection_reasons_with_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| req_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [7, \"TooShort\", ")),
        expect: &[("AC6", "")],
    },
    // A markdown offset beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_an_md_offset_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 10000000000000000000000000000000000000000 | 8 | u64 | Send timestamp")),
        expect: &[("INT_LIMIT", "")],
    },
    // int("032") is 32.
    Denial {
        name: "num_denial_an_md_offset_with_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 032 | 8 | u64 | Send timestamp")),
        expect: &[],
    },
    // `\s*(\d+)\s*$` refuses +32: the row is left out.
    Denial {
        name: "num_denial_an_md_offset_with_a_plus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| +32 | 8 | u64 | Send timestamp")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // An empty offset: the row is left out.
    Denial {
        name: "num_denial_an_empty_md_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "|  | 8 | u64 | Send timestamp")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // `8²`: width_to_bits reads 8 bytes.
    Denial {
        name: "num_denial_an_md_width_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 32 | 8² | u64 | Send timestamp")),
        expect: &[],
    },
    // Digit groups: `0 008` is 8.
    Denial {
        name: "num_denial_an_md_width_in_groups",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 32 | 0 008 | u64 | Send timestamp")),
        expect: &[],
    },
    // `0_8`: the underscore is dropped, 8.
    Denial {
        name: "num_denial_an_md_width_with_an_underscore",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 32 | 0_8 | u64 | Send timestamp")),
        expect: &[],
    },
    // 10^36 bytes, within i128 in bits.
    Denial {
        name: "num_denial_an_md_width_of_10_36_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 32 | 1000000000000000000000000000000000000 | u64 | Send timestamp")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // 10^38 bytes: × 8 is beyond i128.
    Denial {
        name: "num_denial_an_md_width_of_10_38_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 32 | 100000000000000000000000000000000000000 | u64 | Send timestamp")),
        expect: &[("INT_LIMIT", "")],
    },
    // A duplicate row: nothing tiles.
    Denial {
        name: "num_denial_a_duplicate_md_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| req(b, "| 32 | 8 | u64 | Send timestamp", "| 32 | 8 | u64 | Send timestamp | x |\n| 32 | 8 | u64 | Send timestamp")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // `(\d[\d ,_]*\d|\d)\s*-?\s*bytes?`: 1074.
    Denial {
        name: "num_denial_a_stated_size_in_groups",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| lit(t, "both 74 bytes long", "both 1 074 bytes long"))),
        expect: &[("AC1", "")],
    },
    // A stated size beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_stated_size_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| lit(t, "both 74 bytes long", "both 10000000000000000000000000000000000000000 bytes long"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `74²`: not a stated size.
    Denial {
        name: "num_denial_a_stated_size_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| sync(b, |t| lit(t, "both 74 bytes long", "both 74² bytes long"))),
        expect: &[("AC1", "")],
    },
    // int("048") is 48.
    Denial {
        name: "num_denial_the_cs6_value_with_leading_zeros",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "sync CS6 (48)", "sync CS6 (048)"))),
        expect: &[],
    },
    // C-36 beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_request_rate_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "| 50 per second |", "| 10000000000000000000000000000000000000000 per second |"))),
        expect: &[("INT_LIMIT", "")],
    },
    // C-112 beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_accepted_rate_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "10 to 200 per second", "10 to 10000000000000000000000000000000000000000 per second"))),
        expect: &[("INT_LIMIT", "")],
    },
    // C-10 beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_epoch_width_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "16 / 32 / 16 bits", "16 / 32 / 10000000000000000000000000000000000000000 bits"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `16²`: the triple does not match.
    Denial {
        name: "num_denial_the_epoch_width_glued_to_a_superscript",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "16 / 32 / 16 bits", "16 / 32 / 16² bits"))),
        expect: &[("AC2", ""), ("DoD", "")],
    },
];
