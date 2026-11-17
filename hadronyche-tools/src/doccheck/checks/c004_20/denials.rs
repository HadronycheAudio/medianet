//! The denial list of 004-20: the real documents with one edit, and what
//! the port (and the old script, run by `doccheck-equiv 004-20`) must
//! report. One or more per `fail` site of the script, then the denials aimed
//! at Python semantics (`py_denial_*`), then the acceptance probes
//! (`py_accepts_*`, no expectation: the port must pass, as the script
//! does). Consumed by `tests/doc_004_20.rs`, one test each.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
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

fn rr(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Receiver report$", edit)
}

fn rr_sub(t: &str, title: &str, edit: impl FnOnce(&str) -> String) -> String {
    rr(t, |s| within_section(s, title, edit))
}

fn pr(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^One-way probe$", edit)
}

fn pr_sub(t: &str, title: &str, edit: impl FnOnce(&str) -> String) -> String {
    pr(t, |s| within_section(s, title, edit))
}

fn fixed(t: &str, from: &str, to: &str) -> String {
    rr_sub(t, "^Fixed part$", |s| lit(s, from, to))
}

fn member(t: &str, from: &str, to: &str) -> String {
    rr_sub(t, "^Per-member part$", |s| lit(s, from, to))
}

fn rvalid(t: &str, line: &str, from: &str, to: &str) -> String {
    rr_sub(t, "^Validity rules$", |s| replace_in_line(s, line, from, to))
}

fn rr_toml(t: &str, from: &str, to: &str) -> String {
    after(t, "name = \"receiver report\"\n", from, to)
}

fn preq_toml(t: &str, from: &str, to: &str) -> String {
    after(t, "name = \"one-way probe request\"\n", from, to)
}

/// `pattern` (with `$1` expansion) replaced on the lines that are, or are
/// not, table rows.
fn on_rows(t: &str, rows: bool, pattern: &str, to: &str) -> String {
    let re = rx(pattern);
    t.split('\n')
        .map(|l| if l.trim_start().starts_with('|') == rows { re.replace_all(l, to) } else { l.to_string() })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The first paragraph of the Authentication reservation subsection
/// replaced by `para`.
fn auth_para(t: &str, para: &str) -> String {
    rr_sub(t, "^Authentication reservation$", |s| {
        let start = s.find("The receiver report and the one-way probe do not carry").expect("paragraph");
        let end = s[start..].find("\n\n").map_or(s.len(), |k| start + k);
        format!("{}{para}{}", &s[..start], &s[end..])
    })
}

const TAGS: &str = r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "dod_denial_a_constant_unreadable",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| replace_in_line(t, r"^\| C-40 \|", "every 100 ms", "ten times a second"))),
        expect: &[("DoD", "constants row for report_ms could not be read")],
    },
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", ""), ("DoD", "")],
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
        name: "ac2_denial_no_receiver_report_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^Receiver report$", "Consumer report"))),
        expect: &[("AC2", "the Receiver report section is missing"), ("AC1", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "ac4_denial_no_probe_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| rename_heading(t, "^One-way probe$", "Probe exchange"))),
        expect: &[("AC4", "the One-way probe section is missing"), ("AC1", ""), ("AC5", ""), ("AC6", ""), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_a_width_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 26 | 4 | u32 | Socket drop", "| 26 | four | u32 | Socket drop")),
        expect: &[("AC1", "the width of 'Socket drop' is not stated"), ("AC2", ""), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 26 | 4 | u32 | Socket drop", "| 27 | 4 | u32 | Socket drop")),
        expect: &[("AC1", "the row at byte 27 ('Socket drop') does not start")],
    },
    Denial {
        name: "ac1_denial_fields_not_whole_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 34 | 4 | u32 | Current A", "| 34 | 31 bits | u32 | Current A")),
        expect: &[("AC1", "not whole bytes"), ("AC3", ""), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_per_member_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| rename_heading(t, "^Per-member part$", "Rows"))),
        expect: &[("AC1", "needs one fixed-part field table and one per-member field table"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_probe_reply_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| rename_heading(t, "^Reply$", "Answer"))),
        expect: &[("AC1", "needs exactly one request field table and one reply field table"), ("W1", "")],
    },
    Denial {
        name: "ac1_denial_not_unicast_only",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"(?i)\bunicast\b", "direct"))),
        expect: &[("AC1", "does not say the report is unicast only")],
    },
    Denial {
        name: "ac1_denial_no_query_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"(?i)quer(?:y|ies)", "poll"))),
        expect: &[("AC1", "no query message")],
    },
    Denial {
        name: "ac2_denial_maximum_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| lit(t, "= 1 222 bytes", "= 1 222 octets"))),
        expect: &[("AC2", "is not stated in bytes")],
    },
    Denial {
        name: "ac2_denial_a_size_sentence_with_another_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| lit(t, "= 1 222 bytes", "= 1 223 bytes"))),
        expect: &[("AC2", "a size sentence states 1223 bytes")],
    },
    Denial {
        name: "ac2_denial_no_member_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 1 | u8 | Rows:")),
        expect: &[("AC2", "the fixed part has no member count field"), ("AC6", "")],
    },
    Denial {
        name: "ac2_denial_a_member_count_maximum_not_32",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Fixed part$", |t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Member count", r"\| 1 to 32 \|", "| 1 to 31 |"))),
        expect: &[("AC2", "the member count's valid range has maximum")],
    },
    Denial {
        name: "ac3_denial_no_bundle_identity",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Bundle identity:", "| Source identity:")),
        expect: &[("AC3", "the fixed part has no bundle identity field")],
    },
    Denial {
        name: "ac3_denial_a_bundle_identity_not_a_module_id",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Bundle identity: the module id (C-11) of the bundle's source", "| Bundle identity: the source (C-11) of the bundle")),
        expect: &[("AC3", "the bundle identity must be the bundle source's module id")],
    },
    Denial {
        name: "ac3_denial_a_counter_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| member(b, "| Late:", "| Tardy:")),
        expect: &[("AC3", "no field for the C-159 counter 'late'")],
    },
    Denial {
        name: "ac3_denial_a_counter_without_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| member(b, "| 6 | 4 | u32 | Late:", "| 6 | 0 | u32 | Late:")),
        expect: &[("AC3", "the counter 'late' has no width"), ("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_wrap_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"(?i)\b(wraps?|modulo|saturates?)\b", "rolls"))),
        expect: &[("AC3", "states no wrap-or-saturate rule")],
    },
    Denial {
        name: "ac3_denial_no_socket_drop",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Socket drop:", "| Socket loss:")),
        expect: &[("AC3", "the report has no socket drop counter field")],
    },
    Denial {
        name: "ac3_denial_a_socket_drop_without_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 26 | 4 | u32 | Socket drop", "| 26 | 0 | u32 | Socket drop")),
        expect: &[("AC3", "the socket drop counter needs a width"), ("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_no_observed_dscp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| member(b, "| Observed DSCP:", "| Observed code:")),
        expect: &[("AC3", "the report has no observed DSCP field")],
    },
    Denial {
        name: "ac3_denial_a_dscp_of_five_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| member(b, "| 36 | 6 bits | uint | Observed DSCP", "| 36 | 5 bits | uint | Observed DSCP")),
        expect: &[("AC3", "must be exactly 6 bits, not 5"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_a_dscp_range_not_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| member(b, "0 to 63; senders write", "0 to 62; senders write")),
        expect: &[("AC3", "valid range must end at 63")],
    },
    Denial {
        name: "ac3_denial_no_current_a",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Current A: the consumer's current A (C-198)", "| Current budget: the consumer's current budget (C-199)")),
        expect: &[("AC3", "no field for the consumer's current A")],
    },
    Denial {
        name: "ac3_denial_a_without_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 34 | 4 | u32 | Current A", "| 34 | 0 | u32 | Current A")),
        expect: &[("AC3", "the current A has no stated width"), ("AC2", ""), ("AC6", "")],
    },
    Denial {
        name: "ac3_denial_a_without_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Fixed part$", |t| replace_in_line(&replace_in_line(t, r"^\| 34 \| 4 \| u32 \| Current A", "in nanoseconds", "in ticks"), r"^\| 34 \| 4 \| u32 \| Current A", "295 ns:", "295:"))),
        expect: &[("AC3", "the current A has no stated unit")],
    },
    Denial {
        name: "ac3_denial_a_without_range",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Fixed part$", |t| replace_in_line(t, r"^\| 34 \| 4 \| u32 \| Current A", r"\| 0 to 4 294 967 295 ns: every value is valid; 0 = no A known \|", "| any value |"))),
        expect: &[("AC3", "the current A has no stated valid range")],
    },
    Denial {
        name: "ac3_denial_no_reclassification_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(&replace_all(t, "C-108", "C-999"), "(?i)reclassif", "regroup"))),
        expect: &[("AC3", "C-108 reclassification")],
    },
    Denial {
        name: "ac3_denial_no_interrupted_events_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"(?i)\bevents?\b", "entries"))),
        expect: &[("AC3", "interrupted EVENTS")],
    },
    Denial {
        name: "ac3_denial_no_demoted_rounds_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"(?i)\brounds?\b", "turns"))),
        expect: &[("AC3", "demoted-rounds counter")],
    },
    Denial {
        name: "ac3_denial_no_004_18_binding_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| on_rows(t, false, "004-18", "the handshake"))),
        expect: &[("AC3", "the downstream A of 004-18 (C-142)")],
    },
    Denial {
        name: "w1_denial_no_magic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Magic 0x4d4e", "| Marker 0x4d4e")),
        expect: &[("W1", "the receiver report fixed part does not start with the 2-byte magic")],
    },
    Denial {
        name: "w1_denial_no_version",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Protocol version", "| Protocol edition")),
        expect: &[("W1", "has no 2-byte version field at offset 2")],
    },
    Denial {
        name: "w1_denial_no_domain",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Domain id (C-11)", "| Realm id (C-11)")),
        expect: &[("W1", "has no 2-byte domain id field at offset 4")],
    },
    Denial {
        name: "w1_denial_no_message_type",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Message type: 1 = receiver report (Message type codes below)", "| Kind: 1 = receiver report (Kinds below)")),
        expect: &[("W1", "the receiver report fixed part has no message type row")],
    },
    Denial {
        name: "ac3_denial_admission_placeholder_without_open_item",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| {
        let b = wire(b, |t| within_section(t, "^Admission messages$", |s| format!("{}\n\nTo be filled by 004-18.\n", s.split('\n').next().expect("heading"))));
        rr(&b, |t| replace_all(t, r"\*\*Open item", "**Item"))
    }),
        expect: &[("AC3", "the Admission messages section is a placeholder")],
    },
    Denial {
        name: "ac3_denial_no_downstream_a_in_admission",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Admission messages$", |s| on_rows(&on_rows(s, true, r"(?i)downstream a\b", "onward budget"), true, "(?i)downstream", "onward")))),
        expect: &[("AC3", "has no downstream A field to compare with")],
    },
    Denial {
        name: "ac3_denial_a_downstream_a_of_another_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| within_section(t, "^Admission accept$", |s| lit(s, "| 58 | 4 | u32 | Downstream A", "| 58 | 2 | u32 | Downstream A")))),
        expect: &[("AC3", "does not have the width and unit of the downstream A")],
    },
    Denial {
        name: "ac4_denial_no_quantities_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| on_rows(t, false, "(?i)delay", "lag"))),
        expect: &[("AC4", "states which delays its timestamps yield")],
    },
    Denial {
        name: "ac4_denial_no_quantities_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| replace_all(t, "(?i)falsifier", "test"))),
        expect: &[("AC4", "the probe-quantities decision has no falsifier")],
    },
    Denial {
        name: "ac4_denial_no_authentication_decision",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| {
        let t = within_section(t, "^Receiver report$", |s| on_rows(s, false, "(?i)authentication|reservation", "scheme"));
        within_section(&t, "^One-way probe$", |s| on_rows(s, false, "(?i)authentication|reservation", "scheme"))
    })),
        expect: &[("AC4", "no [decision]-tagged paragraph states the authentication-reservation choice")],
    },
    Denial {
        name: "ac4_denial_a_decision_not_covering_both",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| auth_para(b, "These messages do not carry the authentication reservation of C-12 `[decision]`. Falsifier: a later authentication scheme is to cover these after all.")),
        expect: &[("AC4", "covers both the report and the probe")],
    },
    Denial {
        name: "ac4_denial_a_decision_saying_neither",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| auth_para(b, "The receiver report and the one-way probe treat the authentication reservation of C-12 elsewhere `[decision]`. Falsifier: a later scheme covers these messages after all.")),
        expect: &[("AC4", "does not say whether the messages carry")],
    },
    Denial {
        name: "ac4_denial_a_decision_without_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| auth_para(b, "The receiver report and the one-way probe do not carry the authentication reservation of C-12 `[decision]`.")),
        expect: &[("AC4", "the authentication decision has no falsifier")],
    },
    Denial {
        name: "ac4_denial_carried_without_the_fields",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| auth_para(b, "The receiver report and the one-way probe carry the authentication reservation of C-12 `[decision]`. Falsifier: a later authentication scheme is to cover more of these.")),
        expect: &[("AC4", "the decision carries the reservation but the report fixed part table lacks")],
    },
    Denial {
        name: "ac4_denial_not_carried_with_a_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 24 | 2 | u16 | Report number:", "| 24 | 2 | u16 | Message counter:")),
        expect: &[("AC4", "but the report fixed part table has the field 'Message counter'")],
    },
    Denial {
        name: "ac4_denial_no_probe_identifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr_sub(b, "^Request$", |t| lit(t, "| Requester identifier:", "| Requester module:"))),
        expect: &[("AC4", "the probe request has no identifier field")],
    },
    Denial {
        name: "ac4_denial_no_echo",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr_sub(b, "^Reply$", |t| on_rows(t, true, r"(?i)\becho", "copi"))),
        expect: &[("AC4", "the probe reply does not echo")],
    },
    Denial {
        name: "ac4_denial_no_request_timestamp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr_sub(b, "^Request$", |t| lit(t, "| Send timestamp: the requester's network time", "| Send instant: the requester's network time"))),
        expect: &[("AC4", "the probe request has no 64-bit timestamp field")],
    },
    Denial {
        name: "ac4_denial_no_slice_header_relation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| on_rows(t, false, r"(?i)slice[\s-]header", "frame head"))),
        expect: &[("AC4", "consistent with (or differ from) the slice-header")],
    },
    Denial {
        name: "ac4_denial_no_monotonic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| replace_all(t, "(?i)monotonic", "steady"))),
        expect: &[("AC4", "unit (nanoseconds) and clock (monotonic)")],
    },
    Denial {
        name: "ac4_denial_no_whole_byte_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr_sub(b, "^Request$", |t| lit(t, "| 24 | 24 | pad |", "| 24 | 23 bits | pad |"))),
        expect: &[("AC4", "has no whole-byte size"), ("AC1", ""), ("AC6", "")],
    },
    Denial {
        name: "ac4_denial_sizes_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| replace_all(t, "48 bytes", "48 octets"))),
        expect: &[("AC4", "the probe sizes are not stated")],
    },
    Denial {
        name: "ac4_denial_a_size_sentence_with_another_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| lit(t, "= 48 bytes", "= 49 bytes"))),
        expect: &[("AC4", "a probe size sentence states 49 bytes")],
    },
    Denial {
        name: "ac5_denial_fewer_than_six_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Validity rules$", |t| lit(t, "\n| bad magic", "\n\n| bad magic"))),
        expect: &[("AC5", "at least 6 required")],
    },
    Denial {
        name: "ac5_denial_a_row_outside_the_list_without_a_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| bad magic", r"\| drop and count under BadMagic", "| dropped silently")),
        expect: &[("AC5", "is outside the ticket's list")],
    },
    Denial {
        name: "ac5_denial_an_exact_reason_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| version mismatch", r"\| drop and count under VersionMismatch, and raise the VersionMismatch event", "| drop and count under BadLength, and raise an event")),
        expect: &[("AC5", "must be counted under VersionMismatch")],
    },
    Denial {
        name: "ac5_denial_an_unknown_bundle_without_a_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| unknown bundle", r"\| drop and count under UnadmittedSour.*$", "| dropped silently |")),
        expect: &[("AC5", "needs a member reason or an argued non-rejection")],
    },
    Denial {
        name: "ac5_denial_dscp_without_its_argument",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| observed DSCP above 63", r"\| cannot occur.*$", "| dropped silently |")),
        expect: &[("AC5", "the DSCP-above-63 form must say it cannot occur")],
    },
    Denial {
        name: "ac5_denial_no_a_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Validity rules$", |t| delete_row(t, r"^\| current A out of its valid range"))),
        expect: &[("AC5", "no invalid form for A out of its valid range")],
    },
    Denial {
        name: "ac5_denial_an_invented_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| bad magic", "BadMagic", "BadMagic or WrongMagic")),
        expect: &[("AC5", "the outcome names 'WrongMagic'")],
    },
    Denial {
        name: "ac5_denial_counted_under_a_non_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| append(t, "Anything else is counted under Wrongish `[decision]`."))),
        expect: &[("AC5", "counts a rejection under 'Wrongish'")],
    },
    Denial {
        name: "ac6_denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC6", "does not exist"), ("AC1", "")],
    },
    Denial {
        name: "ac6_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| format!("{t}\n[[[broken"))),
        expect: &[("AC6", "does not parse"), ("AC1", "")],
    },
    Denial {
        name: "ac1_denial_no_toml_report",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| replace_all(t, "name = \"receiver report", "name = \"rx summary"))),
        expect: &[("AC1", "declares no [[message]] named 'receiver report'"), ("AC6", "")],
    },
    Denial {
        name: "ac1_denial_no_toml_probe",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| rx(r#"(name = "[^"\n]*)probe"#).replace_all(t, "${1}check"))),
        expect: &[("AC1", "declares no [[message]] for the one-way probe"), ("AC6", "")],
    },
    Denial {
        name: "ac6_denial_the_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "layer = \"measurement\"", "layer = \"control\"")),
        expect: &[("AC6", "receiver report in wire.toml must have layer")],
    },
    Denial {
        name: "ac6_denial_the_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "dscp = 40", "dscp = 46")),
        expect: &[("AC6", "receiver report in wire.toml must have dscp = 40")],
    },
    Denial {
        name: "ac6_denial_no_reasons_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "rejection_reasons = [", "rejection_reasons = \"x\"\nold = [")),
        expect: &[("AC6", "has no rejection_reasons list")],
    },
    Denial {
        name: "ac6_denial_a_reason_not_a_member",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "rejection_reasons = [", "rejection_reasons = [\"Bogus\", ")),
        expect: &[("AC6", "names 'Bogus', which is not a member")],
    },
    Denial {
        name: "ac6_denial_probe_field_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "[[message.field]]\nname = \"message_type\"", "[[message.gone]]\nname = \"message_type\"")),
        expect: &[("AC6", "probe request: the markdown table has 8 fields, wire.toml 7")],
    },
    Denial {
        name: "ac6_denial_probe_field_width",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "width = 64", "width = 32")),
        expect: &[("AC6", "probe request: markdown field 'Requester identifier'")],
    },
    Denial {
        name: "ac6_denial_probe_field_type",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "type = \"u16\"", "type = \"i16\"")),
        expect: &[("AC6", "probe request: markdown type 'u16' of 'Magic 0x4d4e")],
    },
    Denial {
        name: "ac6_denial_report_field_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "[[message.field]]\nname = \"message_type\"", "[[message.gone]]\nname = \"message_type\"")),
        expect: &[("AC6", "expected the 11 fixed fields plus one counted member field")],
    },
    Denial {
        name: "ac6_denial_member_max_count",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_count = 32", "max_count = 31")),
        expect: &[("AC6", "the counted member field must have")],
    },
    Denial {
        name: "ac6_denial_member_count_name",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "count = \"member_count\"", "count = \"members\"")),
        expect: &[("AC6", "count must name the member count field")],
    },
    Denial {
        name: "ac6_denial_report_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 1223")),
        expect: &[("AC6", "max_size 1223 differs from the markdown maximum 1222")],
    },
    Denial {
        name: "ac6_denial_no_probe_reply_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| toml(b, |t| replace_all(t, "probe reply\"", "probe answer\""))),
        expect: &[("AC6", "declares no [[message]] for the probe reply")],
    },
    Denial {
        name: "ac6_denial_probe_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "max_size = 48", "max_size = 49")),
        expect: &[("AC6", "probe request in wire.toml: max_size 49")],
    },
    Denial {
        name: "ac6_denial_a_named_reason_not_declared",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, ", \"UnknownStream\"", "")),
        expect: &[("AC6", "miss 'UnknownStream'")],
    },
    Denial {
        name: "ac6_denial_a_reason_without_the_measurement_layer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| replace_in_line(t, r"^\| 0 \| TooShort \|", "measurement", "media2"))),
        expect: &[("AC6", "the enumeration row for 'TooShort' does not list the measurement layer")],
    },
    Denial {
        name: "c40_denial_no_period",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, "100 ms", "tenth of a second"))),
        expect: &[("C-40", "every 100 ms")],
    },
    Denial {
        name: "c40_denial_not_per_bundle",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"(?i)\bbundle\b", "group"))),
        expect: &[("C-40", "the report is per bundle"), ("AC3", ""), ("AC5", "")],
    },
    Denial {
        name: "c41_denial_no_timeout",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"\b2 s\b", "two s"))),
        expect: &[("C-41", "2 s timeout")],
    },
    Denial {
        name: "c13_denial_no_cs5",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"CS5 \(40\)", "CS5"))),
        expect: &[("C-13", "CS5 (40)")],
    },
    Denial {
        name: "c40_denial_a_rate_field",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 24 | 2 | u16 | Report number:", "| 24 | 2 | u16 | Report rate:")),
        expect: &[("C-40", "names a rate or interval field ('Report rate')")],
    },
    Denial {
        name: "ac1_denial_forward_media_with_receiver_state",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| append(t, "Forward media packets carry receiver state here `[decision]`."))),
        expect: &[("AC1", "forward media packets carry receiver state")],
    },
    Denial {
        name: "c13_denial_no_ef",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| replace_all(t, r"EF \(46\)", "EF"))),
        expect: &[("C-13", "EF (46)")],
    },
    Denial {
        name: "c100_denial_no_reverse_port",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| wire(b, |t| {
        let t = within_section(t, "^Receiver report$", |s| replace_all(s, "(?i)reverse", "return"));
        within_section(&t, "^One-way probe$", |s| replace_all(s, "(?i)reverse", "return"))
    })),
        expect: &[("C-100", "on the reverse port")],
    },
    Denial {
        name: "dod_denial_a_table_after_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Fixed part$", |t| replace_all(t, TAGS, ""))),
        expect: &[("DoD", "a table of the Receiver report section carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| strip_tags(t, "^No field carries a rate"))),
        expect: &[("DoD", "a paragraph of the Receiver report section carries no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| append(t, "**Open item 6 — x.** Still open `[decision]`."))),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    // `\w`: `²` glued to the reason defeats `\bVersionMismatch\b`.
    Denial {
        name: "py_denial_a_reason_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| version mismatch", "VersionMismatch, and raise the VersionMismatch event", "VersionMismatch\u{b2}, and raise an event")),
        expect: &[("AC5", "must be counted under VersionMismatch")],
    },
    // The 25-character look-back has no `no`: "no WrongMagic" is a name.
    Denial {
        name: "py_denial_a_reason_after_no",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| bad magic", "BadMagic", "BadMagic, no WrongMagic")),
        expect: &[("AC5", "the outcome names 'WrongMagic'")],
    },
    // `str.splitlines()`: U+2028 ends the report's form table in its
    // second row.
    Denial {
        name: "py_denial_a_line_separator_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{2028}drop and count under BadMagic")),
        expect: &[("REPERTOIRE", "")],
    },
    // `str.splitlines()`: a form feed too.
    Denial {
        name: "py_denial_a_form_feed_inside_the_validity_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| bad magic", r"\| drop and count under BadMagic", "|\u{c}drop and count under BadMagic")),
        expect: &[("REPERTOIRE", "")],
    },
    // `int()` is unbounded: a 21-digit offset is read and does not tile.
    Denial {
        name: "py_denial_an_offset_beyond_64_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 26 | 4 | u32 | Socket drop", "| 100000000000000000026 | 4 | u32 | Socket drop")),
        expect: &[("AC1", "the row at byte 100000000000000000026")],
    },
    // `\d`/`int()` read Arabic-Indic digits: ٢٧ is 27.
    Denial {
        name: "py_denial_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 26 | 4 | u32 | Socket drop", "| \u{662}\u{667} | 4 | u32 | Socket drop")),
        expect: &[("REPERTOIRE", "")],
    },
    // The subset reader keeps `"1222"` a str.
    Denial {
        name: "py_denial_max_size_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = \"1222\"")),
        expect: &[("AC6", "max_size 1222 differs from the markdown maximum 1222")],
    },
    // The subset reader cannot read `"""` strings.
    Denial {
        name: "py_denial_a_triple_quoted_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "name = \"magic\"", "name = \"\"\"magic\"\"\"")),
        expect: &[("AC6", "does not parse"), ("AC1", "")],
    },
    // The subset reader lets a later duplicate key win.
    Denial {
        name: "py_denial_a_duplicate_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "dscp = 40\n", "dscp = 40\ndscp = 46\n")),
        expect: &[("AC6", "must have dscp = 40")],
    },
    // A list rejection reason is unhashable: TypeError.
    Denial {
        name: "py_denial_a_reason_as_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "rejection_reasons = [", "rejection_reasons = [[\"x\"], ")),
        expect: &[("CRASH", "unhashable")],
    },
    // The 004 separator line: a letter in it drops the report's form
    // table.
    Denial {
        name: "py_denial_a_separator_with_a_letter",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|---|-x-|"))),
        expect: &[("AC5", "at least 6 required")],
    },
    // `re.I`: "Open ıtem" is an open item to Python.
    Denial {
        name: "py_denial_an_open_item_with_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| append(t, "**Open \u{131}tem 6 — x.** Still open `[decision]`."))),
        expect: &[("REPERTOIRE", "")],
    },
    // `\d`/`int()`: ٢٦ is 26.
    Denial {
        name: "py_accepts_an_offset_in_arabic_indic_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 26 | 4 | u32 | Socket drop", "| \u{662}\u{666} | 4 | u32 | Socket drop")),
        expect: &[("REPERTOIRE", "")],
    },
    // `\s`: U+001F is whitespace for Python: "2\u{1f}s" restates C-41.
    Denial {
        name: "py_accepts_a_unit_separator_in_a_duration",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| replace_all(t, r"\b2 s\b", "2\u{1f}s"))),
        expect: &[("REPERTOIRE", "")],
    },
    // The 25-character look-back counts characters, not bytes.
    Denial {
        name: "py_accepts_a_negation_within_25_characters_of_wide_ones",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rvalid(b, r"^\| bad magic", "BadMagic", "BadMagic, never ———— WrongMagic")),
        expect: &[],
    },
    // `re.I`: "monotonıc" is "monotonic" to Python.
    Denial {
        name: "py_accepts_a_dotless_i",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| pr(b, |t| replace_all(t, "monotonic", "monoton\u{131}c"))),
        expect: &[("REPERTOIRE", "")],
    },
    // The subset reader: 40.0 and 0x28 equal 40.
    Denial {
        name: "py_accepts_a_float_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "dscp = 40\n", "dscp = 40.0\n")),
        expect: &[],
    },
    Denial {
        name: "py_accepts_a_hex_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "dscp = 40\n", "dscp = 0x28\n")),
        expect: &[],
    },
    // The 004 separator line accepts colons inside a cell.
    Denial {
        name: "py_accepts_a_separator_with_inner_colons",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr_sub(b, "^Validity rules$", |t| lit(t, "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|-:-|---|"))),
        expect: &[],
    },
    // A combining accent is no word character for Python: `\bA\b` still
    // finds the A row.
    Denial {
        name: "py_accepts_a_combining_accent_after_a",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| Current A: the consumer's", "| Current A\u{301}: the consumer's")),
        expect: &[("REPERTOIRE", "")],
    },
    // `T_REP.get("max_size") != MAX_REP` on 10^19.
    Denial {
        name: "num_denial_max_size_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 10000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 10^40 compared with an int: Python fails AC6; the port reads no int there.
    Denial {
        name: "num_denial_max_size_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 10000000000000000000000000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 1222.0 == 1222.
    Denial {
        name: "num_denial_max_size_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 1222.0")),
        expect: &[],
    },
    // "1222" != 1222.
    Denial {
        name: "num_denial_max_size_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = \"1222\"")),
        expect: &[("AC6", "")],
    },
    // Leading zeros.
    Denial {
        name: "num_denial_max_size_with_leading_zeros",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 01222")),
        expect: &[],
    },
    // An underscore.
    Denial {
        name: "num_denial_max_size_with_an_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 1_222")),
        expect: &[],
    },
    // 0x4c6 = 1222.
    Denial {
        name: "num_denial_max_size_in_hex",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 0x4c6")),
        expect: &[],
    },
    // No max_size: None != 1222.
    Denial {
        name: "num_denial_max_size_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222\n", "")),
        expect: &[("AC6", "")],
    },
    // A max_count beyond i128 compared with 32.
    Denial {
        name: "num_denial_a_max_count_of_10_40",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "count = \"member_count\"\nmax_count = 32", "count = \"member_count\"\nmax_count = 10000000000000000000000000000000000000000")),
        expect: &[("AC6", "")],
    },
    // True != 32.
    Denial {
        name: "num_denial_a_max_count_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "count = \"member_count\"\nmax_count = 32", "count = \"member_count\"\nmax_count = true")),
        expect: &[("AC6", "")],
    },
    // 32.0 == 32.
    Denial {
        name: "num_denial_a_max_count_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "count = \"member_count\"\nmax_count = 32", "count = \"member_count\"\nmax_count = 32.0")),
        expect: &[],
    },
    // `el.get("count") != tfields[i].get("name")`: 7 != "member_count".
    Denial {
        name: "num_denial_a_count_as_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "count = \"member_count\"", "count = 7")),
        expect: &[("AC6", "")],
    },
    // A probe max_size of 10^19.
    Denial {
        name: "num_denial_the_probe_max_size_of_10_19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "max_size = 48", "max_size = 10000000000000000000")),
        expect: &[("AC6", "")],
    },
    // 46.0 == 46.
    Denial {
        name: "num_denial_the_probe_dscp_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "dscp = 46", "dscp = 46.0")),
        expect: &[],
    },
    // True != 46.
    Denial {
        name: "num_denial_the_probe_dscp_as_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| preq_toml(b, "dscp = 46", "dscp = true")),
        expect: &[("AC6", "")],
    },
    // `reason not in MEMBER_SET` hashes a list: TypeError.
    Denial {
        name: "num_denial_rejection_reasons_with_a_list",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [[1], \"TooShort\", ")),
        expect: &[("CRASH", "")],
    },
    // An int reason: not a member, then `sorted(ALL_TOML_REASONS)` raises TypeError.
    Denial {
        name: "num_denial_rejection_reasons_mixed_types",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [7, \"TooShort\", ")),
        expect: &[("AC6", ""), ("CRASH", "")],
    },
    // A duplicate reason: the set drops it.
    Denial {
        name: "num_denial_duplicate_rejection_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [\"TooShort\", \"TooShort\", ")),
        expect: &[],
    },
    // `field = 5` before the field tables (the reader decides).
    Denial {
        name: "num_denial_the_report_field_a_number",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 1222\nfield = 5")),
        expect: &[("AC1", ""), ("AC6", "")],
    },
    // A markdown offset beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_an_offset_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 10000000000000000000000000000000000000000 | 1 | u8 | Member count:")),
        expect: &[("INT_LIMIT", "")],
    },
    // An offset of 10^19: a gap.
    Denial {
        name: "num_denial_an_offset_of_10_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 10000000000000000000 | 1 | u8 | Member count:")),
        expect: &[("AC1", "")],
    },
    // `offset_bytes` takes an optional `+`: +7 is 7.
    Denial {
        name: "num_denial_an_offset_with_a_plus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| +7 | 1 | u8 | Member count:")),
        expect: &[],
    },
    // int("007") is 7.
    Denial {
        name: "num_denial_an_offset_with_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 007 | 1 | u8 | Member count:")),
        expect: &[],
    },
    // An empty offset: the row is left out.
    Denial {
        name: "num_denial_an_empty_offset",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "|  | 1 | u8 | Member count:")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    // `7²`: not an offset, the row is left out.
    Denial {
        name: "num_denial_an_offset_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7² | 1 | u8 | Member count:")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    // 10^38 bytes: × 8 is beyond i128.
    Denial {
        name: "num_denial_a_width_of_10_38_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 100000000000000000000000000000000000000 | u8 | Member count:")),
        expect: &[("INT_LIMIT", "")],
    },
    // A width of 10^19 bytes.
    Denial {
        name: "num_denial_a_width_of_10_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 10000000000000000000 | u8 | Member count:")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    // `1²`: the first `\d+` is 1, one byte.
    Denial {
        name: "num_denial_a_width_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 1² | u8 | Member count:")),
        expect: &[],
    },
    // A zero width.
    Denial {
        name: "num_denial_a_width_of_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 0 | u8 | Member count:")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    // A duplicate row: an overlap.
    Denial {
        name: "num_denial_a_duplicate_fixed_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 1 | u8 | Member count: a copy | 1 to 32 |\n| 7 | 1 | u8 | Member count:")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    // `max_int` of the valid range beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_member_count_maximum_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 1 | u8 | Member count, at most 10000000000000000000000000000000000000000:")),
        expect: &[],
    },
    // `stated_bytes` digit groups: 1222000.
    Denial {
        name: "num_denial_a_stated_maximum_in_groups",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| lit(t, "38 + 32 × 37 = 1 222 bytes", "38 + 32 × 37 = 1 222 000 bytes"))),
        expect: &[("AC2", "")],
    },
    // A stated size beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_a_stated_maximum_of_10_40",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| lit(t, "38 + 32 × 37 = 1 222 bytes", "38 + 32 × 37 = 10000000000000000000000000000000000000000 bytes"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `1 222²`: `stated_bytes` finds no size there.
    Denial {
        name: "num_denial_a_stated_maximum_glued_to_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| rr(b, |t| lit(t, "38 + 32 × 37 = 1 222 bytes", "38 + 32 × 37 = 1 222² bytes"))),
        expect: &[("AC2", "")],
    },
    // C-40 beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_report_period_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "every 100 ms", "every 10000000000000000000000000000000000000000 ms"))),
        expect: &[("INT_LIMIT", "")],
    },
    // int("0100") is 100; the restatement must say 100 ms.
    Denial {
        name: "num_denial_the_report_period_with_leading_zeros",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "every 100 ms", "every 0100 ms"))),
        expect: &[],
    },
    // C-41 of 10^19 s: not restated.
    Denial {
        name: "num_denial_the_timeout_of_10_19",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "2 s without", "10000000000000000000 s without"))),
        expect: &[],
    },
    // C-08 beyond i128: INT_LIMIT.
    Denial {
        name: "num_denial_the_members_maximum_of_10_40",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "at most 32, the controller", "at most 10000000000000000000000000000000000000000, the controller"))),
        expect: &[("INT_LIMIT", "")],
    },
    // `EF\s*\((\d+)\)` fails on `46²`: the constant is unread.
    Denial {
        name: "num_denial_the_ef_value_glued_to_a_superscript",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "EF (46)", "EF (46²)"))),
        expect: &[("AC6", ""), ("DoD", "")],
    },
    // 2^63.
    Denial {
        name: "num_denial_an_offset_of_2_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 9223372036854775808 | 1 | u8 | Member count:")),
        expect: &[("AC1", "")],
    },
    // 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_an_offset_of_45_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 100000000000000000000000000000000000000000000 | 1 | u8 | Member count:")),
        expect: &[("INT_LIMIT", "")],
    },
    // `\d+` refuses 0_7.
    Denial {
        name: "num_denial_an_offset_with_an_underscore",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 0_7 | 1 | u8 | Member count:")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC6", "")],
    },
    // `µ` after the digit.
    Denial {
        name: "num_denial_a_width_after_a_micro_sign",
        file: WIRE_MD,
        edit: DenialEdit::Text(|b| fixed(b, "| 7 | 1 | u8 | Member count:", "| 7 | 1µ | u8 | Member count:")),
        expect: &[],
    },
    // 2^63.
    Denial {
        name: "num_denial_max_size_of_2_63",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|b| rr_toml(b, "max_size = 1222", "max_size = 9223372036854775808")),
        expect: &[("AC6", "")],
    },
    // 45 digits: INT_LIMIT.
    Denial {
        name: "num_denial_the_report_period_of_45_digits",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|b| consts(b, |t| lit(t, "every 100 ms", "every 100000000000000000000000000000000000000000000 ms"))),
        expect: &[("INT_LIMIT", "")],
    },
];
