//! The denial list of 004-27: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_27.rs` (one test each) and by `doccheck-equiv 004-27
//! --denials`. The `review_*` entries are the review's findings 1c-1f and
//! 2 and its two "stricter" cases (PR #42, round 1); an entry with no
//! expectation is one the port must pass (the agreement is the point, or the
//! script's red is a classified difference).

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{
    delete_row, delete_section, delete_unit, rename_heading, replace_all, replace_in_line, strip_tags, within_section,
};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;

const PROBE_SECTION: &str = "^Path-MTU probe pair$";
const PROBE_REQUEST: &str = "name = \"path-MTU probe request\"\n";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "ac1_denial_no_probe_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Path-MTU probe pair$", "Probe pair")),
        expect: &[("AC1", "has a '## Path-MTU probe pair' section")],
    },
    Denial {
        name: "ac1_denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, "^The path-MTU probe pair is how", "^", "To be filled. ")),
        expect: &[("AC1", "not the placeholder")],
    },
    Denial {
        name: "ac1_denial_a_header_row_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme")),
        expect: &[("AC1", "seven common control header rows")],
    },
    Denial {
        name: "ac1_denial_rows_that_do_not_tile",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| 48 \| 4 \| u32 \| Request id: the request id of the admission",
            r"^\| 48",
            "| 49",
        )
    }),
        expect: &[("AC1", "tile from byte 0")],
    },
    Denial {
        name: "ac1_denial_a_maximum_size_that_differs_from_the_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            "^The reply's maximum size is 54 bytes",
            "54 bytes",
            "56 bytes",
        )
    }),
        expect: &[("AC1", "maximum size equal to the sum")],
    },
    Denial {
        name: "ac1_denial_a_type_code_outside_the_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| 6 \| 1 \| u8 \| Message type: 33",
            r"33 only",
            "36 only",
        )
    }),
        expect: &[("AC1", "inside the block the hello gives the probe pair")],
    },
    Denial {
        name: "ac1_denial_the_block_not_assigned_to_this_ticket",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| 32 to 35 \| path-MTU probe pair",
            "004-27",
            "004-99",
        )
    }),
        expect: &[("AC1", "block assigned to 004-27")],
    },
    Denial {
        name: "ac1_denial_no_padding_row_in_the_request",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 54 \| 1 × max 65453 \| pad")),
        expect: &[("AC1", "zero-valued padding row")],
    },
    Denial {
        name: "ac1_denial_no_echoed_size_in_the_reply",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 52 \| 2 \| u16 \| Echoed intended packet size")),
        expect: &[("AC1", "echoes the intended packet size")],
    },
    Denial {
        name: "ac1_denial_no_matching_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_section(t, "^Matching a reply to its probe$")),
        expect: &[("AC1", "a matching subsection says how a reply identifies its probe")],
    },
    Denial {
        name: "ac1_denial_no_df_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, r"(?i)rule 11", "the rule")),
        expect: &[("AC1", "the DF requirement is stated")],
    },
    Denial {
        name: "ac1_denial_no_owner_ruling_on_the_acknowledgement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, r"small acknowledgement", "short message")),
        expect: &[("AC1", "the owner's ruling")],
    },
    Denial {
        name: "ac1_denial_a_padded_reply",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            "^The probe reply is the consumer side's answer",
            "$",
            " The reply is padded.",
        )
    }),
        expect: &[("AC1", "does not say the reply is padded")],
    },
    Denial {
        name: "ac1_denial_the_fallback_sequence_out_of_order",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, "PathMtuTooSmall", "PathMtuFailed")),
        expect: &[("AC1", "the fallback sequence is stated in order")],
    },
    Denial {
        name: "ac1_denial_the_fallback_arithmetic_broken",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(t, r"^\| C-06 \|", "1 280", "1 300")
    }),
        expect: &[("AC1", "the fallback arithmetic holds")],
    },
    Denial {
        name: "ac1_denial_the_admission_request_intended_size_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        within_section(t, "^Admission request$", |s| {
            replace_all(
                s,
                r"\| u16 \| Intended packet size",
                "| u16 | Declared packet size",
            )
        })
    }),
        expect: &[("AC1", "the admission request still carries the intended packet size")],
    },
    Denial {
        name: "ac1_denial_the_toml_type_code_outside_the_block",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, r#"valid = "33 only""#, r#"valid = "36 only""#)),
        expect: &[("AC1", "carries a message type inside the probe block")],
    },
    Denial {
        name: "ac2_denial_the_reply_also_exempt",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe reply\"\n",
            "name = \"path-MTU probe reply\"\nexempt_from_control_bound = \"C-116\"\n",
        )
    }),
        expect: &[("AC2", "exactly one message of wire.toml carries the exemption")],
    },
    Denial {
        name: "ac2_denial_an_exemption_citing_another_constant",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            "^In wire.toml only the probe request carries",
            "$",
            " It is also exempt under C-18.",
        )
    }),
        expect: &[("AC2", "every sentence of the section that says exempt cites C-116")],
    },
    Denial {
        name: "ac3_denial_fewer_than_four_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| delete_section(t, "^Validity rules$"))),
        expect: &[("AC3", "at least 4 invalid forms")],
    },
    Denial {
        name: "ac3_denial_an_outcome_naming_no_member",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| replace_in_line(t, r"^\| bad magic", "BadMagic", "WrongMagic"))),
        expect: &[("AC3", "not members: [\"WrongMagic\"]")],
    },
    Denial {
        name: "ac3_denial_non_zero_padding_without_reserved_non_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| replace_in_line(t, r"^\| padding non-zero", "ReservedNonZero", "BadLength"))),
        expect: &[("AC3", "non-zero padding is dropped and counted under ReservedNonZero")],
    },
    Denial {
        name: "ac3_denial_no_unmatched_reply_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| delete_row(t, r"^\| a reply that matches no outstanding probe"))),
        expect: &[("AC3", "a reply that matches no outstanding probe has an outcome")],
    },
    Denial {
        name: "ac3_denial_no_probe_size_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| {
        delete_row(
            t,
            r"^\| probe size different from its declared intended size",
        )
    })),
        expect: &[("AC3", "a probe size different from its declared intended size")],
    },
    Denial {
        name: "ac3_denial_the_enumeration_not_thirteen",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| \d+ \| BadMagic \|")),
        expect: &[("AC3", "has 13 members")],
    },
    Denial {
        name: "ac4_denial_the_reply_above_the_bound",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe reply\"\nlayer = \"control\"\ndscp = 40\nmax_size = 54",
            "name = \"path-MTU probe reply\"\nlayer = \"control\"\ndscp = 40\nmax_size = 1300",
        )
    }),
        expect: &[("AC4", "declares a max_size at most the")],
    },
    Denial {
        name: "ac4_denial_the_wrong_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe request\"\nlayer = \"control\"\ndscp = 40",
            "name = \"path-MTU probe request\"\nlayer = \"control\"\ndscp = 46",
        )
    }),
        expect: &[("AC4", "is layer control at dscp")],
    },
    Denial {
        name: "ac4_denial_a_refusal_missing_from_wire_toml",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, r#"name = "PathMtuTooSmall""#, r#"name = "PathTooSmall""#)),
        expect: &[("AC4", "the refusal enumeration of wire.toml equals")],
    },
    Denial {
        name: "ac4_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[[broken")),
        expect: &[("AC4", "does not parse")],
    },
    Denial {
        name: "ac4_denial_no_probe_reply_in_wire_toml",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
        replace_all(
            t,
            "name = \"path-MTU probe reply\"",
            "name = \"path-MTU answer\"",
        )
    }),
        expect: &[("AC4", "wire.toml has no probe reply message")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| strip_tags(t, r"^Don't-fragment \(rule 11\)"))),
        expect: &[("DoD", "every prose paragraph of 5 or more words in the probe section carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        let t = delete_unit(t, "^A probe is identified by its request id");
        delete_unit(&t, "^The responder needs no match")
    }),
        expect: &[("DoD", "every subsection of the probe section carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owning_ticket",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, PROBE_SECTION, |t| {
        replace_in_line(
            t,
            r"^\*\*Open item 7",
            "$",
            "\n\n**Open item 8 — x.** Owner: the design owner. Falsifier: a run `[decision]`.",
        )
    })),
        expect: &[("DoD", "every open item of the probe section names an owner and a falsifier")],
    },
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist")],
    },
    Denial {
        name: "review_1c_a_line_separator_starts_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^The maximum is the largest UDP payload", "$", "\u{2028}### Extra\u{2028}plain words here")
        }),
        expect: &[("DoD", "every subsection of the probe section carries an evidence tag")],
    },
    Denial {
        name: "review_1d_a_form_feed_starts_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^The maximum is the largest UDP payload", "$", "\u{0c}### Extra\u{0c}plain words here")
        }),
        expect: &[("DoD", "every subsection of the probe section carries an evidence tag")],
    },
    Denial {
        name: "review_1e_rejection_reasons_as_a_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            within_toml_message(t, |m| replace_in_line(m, "^rejection_reasons = ", r"\[.*\]$", "\"BadLength\""))
        }),
        expect: &[("AC4", "rejection_reasons are members of the closed Reject enumeration")],
    },
    Denial {
        name: "review_1f_a_triple_quoted_string_the_subset_reader_rejects",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| t.replacen(PROBE_REQUEST, &format!("{PROBE_REQUEST}note = \"\"\"valid TOML\"\"\"\n"), 1)),
        expect: &[],
    },
    Denial {
        name: "review_stricter_dscp_written_as_a_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| within_toml_message(t, |m| replace_in_line(m, "^dscp = 40$", "40", "40.0"))),
        // Python's `40.0 == 40` holds and the port compares as Python does
        // (py_eq_int): both pass, no "port stricter (typed)" needed
        expect: &[],
    },
    Denial {
        name: "review_2_a_separator_with_an_inner_colon",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, PROBE_SECTION, |s| {
                within_section(s, "^Validity rules$", |v| replace_in_line(v, r"^\|---\|---\|$", r"^.*$", "|-:-|---|"))
            })
        }),
        expect: &[],
    },
];

/// Applies `edit` to the probe request's `[[message]]` block of wire.toml
/// only (up to its first `[[message.field]]`).
fn within_toml_message(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    let start = t.find(PROBE_REQUEST).expect("the probe request is in wire.toml");
    let end = t[start..].find("[[message.field]]").map_or(t.len(), |e| start + e);
    format!("{}{}{}", &t[..start], edit(&t[start..end]), &t[end..])
}
