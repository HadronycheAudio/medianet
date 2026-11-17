//! The denial list of 004-27: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_27.rs` (one test each) and by `doccheck-equiv 004-27
//! --denials`. The `review_*` entries are the review's findings 1c-1f and
//! 2 and its two "stricter" cases (PR #42, round 1); an entry with no
//! expectation is one the port must pass (the agreement is the point, or the
//! script's red is a classified difference).

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{anchored_replace, 
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
        expect: &[("AC1", "has a '## Path-MTU probe pair' section"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, "^The path-MTU probe pair is how", "^", "To be filled. ")),
        expect: &[("AC1", "not the placeholder"), ("AC2", "")],
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
        expect: &[("AC1", "tile from byte 0"), ("AC4", "")],
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
        expect: &[("AC1", "zero-valued padding row"), ("AC4", "")],
    },
    Denial {
        name: "ac1_denial_no_echoed_size_in_the_reply",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 52 \| 2 \| u16 \| Echoed intended packet size")),
        expect: &[("AC1", "echoes the intended packet size"), ("AC4", "")],
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
        expect: &[("AC1", "the fallback sequence is stated in order"), ("AC4", "")],
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
        expect: &[("AC3", "has 13 members"), ("AC4", "")],
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
        expect: &[("AC4", "does not parse"), ("AC2", "")],
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
        expect: &[("AC4", "wire.toml has no probe reply message"), ("AC2", "")],
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
        expect: &[("DoD", "every subsection of the probe section carries an evidence tag"), ("AC1", "")],
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
        expect: &[("AC1", "does not exist"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("DoD", "")],
    },
    Denial {
        name: "review_1c_a_line_separator_starts_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^The maximum is the largest UDP payload", "$", "\u{2028}### Extra\u{2028}plain words here")
        }),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_1d_a_form_feed_starts_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^The maximum is the largest UDP payload", "$", "\u{0c}### Extra\u{0c}plain words here")
        }),
        expect: &[("REPERTOIRE", "")],
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
        expect: &[("AC2", ""), ("AC4", "")],
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
    // ---- the review's 100 denials (PR #42 rounds 1 and 2), as posted:
    // expected = the old script's verdict, or REPERTOIRE outside the repertoire
    Denial {
        name: "review_r026_ac3_outcome_names_a_non_member_badmagik",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Matching a reply to its probe", "| drop and count under BadMagic |", "| drop and count under BadMagik |")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "review_r027_ac4_dscp_40_46_request",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40", "dscp = 46")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r028_ac2_exemption_removed_from_request",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "exempt_from_control_bound = \"C-116\"\n", "")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r029_ac2_exemption_added_to_reply",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe reply\"", "max_size = 54\n", "max_size = 54\nexempt_from_control_bound = \"C-116\"\n")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r030_ac4_rejection_reasons_a_string_not_an_array",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"ReservedNonZero\"]", "rejection_reasons = \"BadLength\"")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r031_ac4_triple_quoted_string_valid_toml",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "layer = \"control\"\n", "layer = \"control\"\nnote = \"\"\"valid TOML\"\"\"\n")),
        expect: &[("AC2", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r032_ac4_dscp_as_float_40_0",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40", "dscp = 40.0")),
        expect: &[],
    },
    Denial {
        name: "review_r033_ac1_width_changed_in_the_request_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "| 40 | 8 | u64 | Recipient", "| 40 | 4 | u64 | Recipient")),
        expect: &[("AC1", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r034_ac1_maximum_size_sentence_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "The reply's maximum size is 54 bytes", "The reply's maximum size is 56 bytes")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r035_ac1_reply_says_padded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "it carries no padding", "it is padded")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r036_ac1_cs5_40_cs5_48",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Path-MTU probe pair", "CS5 (40)", "CS5 (48)")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r037_ac1_rule_11_removed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Path-MTU probe pair", "Don't-fragment (rule 11)", "Don't-fragment")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r038_ac1_decision_tag_removed_from_the_ruling",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Sizes and the exemption", "the reply is a small acknowledgement", "the reply is an acknowledgement")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r039_ac1_hello_header_width_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Hello", "| 7 | 1 | u8 | Authentication scheme", "| 7 | 2 | u8 | Authentication scheme")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r040_ac2_a_second_exemption_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Sizes and the exemption", "under C-116", "under C-116 and C-118")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r041_dod_open_item_without_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Matching a reply to its probe", "### Open items\n", "### Open items\n\n**Open item 9 — x.** Owner: 004-23 `[decision]`.\n")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r042_dod_u_2028_starts_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "\n\n### Path-MTU probe reply\n", "\u{2028}### Extra\u{2028}plain words here\n\n### Path-MTU probe reply\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r043_dod_form_feed_starts_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "\n\n### Path-MTU probe reply\n", "\u{0c}### Extra\u{0c}plain words here\n\n### Path-MTU probe reply\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r044_ac3_separator_row_variant_in_validity_rules",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Matching a reply to its probe", "| Invalid form | Required outcome |\n|---|---|", "| Invalid form | Required outcome |\n|-:-|---|")),
        expect: &[],
    },
    Denial {
        name: "review_r046_crlf_line_ends_in_wire_md",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "ALL\n", "\r\n")),
        expect: &[],
    },
    Denial {
        name: "review_r047_const_c_17_bound_changed",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-17 |", "1 252", "1 253")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r048_ac1_fallback_number_removed_from_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Path-MTU probe pair", "the 1 280-byte fallback (C-06)", "the fallback (C-06)")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r068_r2_b_before_combining_mark_reply_is_padded_u_0301",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "it carries no padding", "it is padded\u{301}")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r069_r2_re_i_dotted_i_reply_i_s_padded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "it carries no padding", "it İS padded")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r070_r2_re_i_dotted_i_on_a_heading_vali_dity_rules",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Path-MTU probe pair", "### Validity rules", "### VALİDITY rules")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r071_r2_split_with_x1f_untagged_5_word_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Matching a reply to its probe", "### Matching a reply to its probe\n", "### Matching a reply to its probe\n\none\u{1f}two\u{1f}three\u{1f}four\u{1f}five\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r072_r2_lstrip_x1c_before_a_request_field_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "| 40 | 8 | u64 | Recipient", "\u{1c}| 40 | 8 | u64 | Recipient")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r073_r2_lstrip_extra_padding_row_in_the_reply_prefixed_by_x1f",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "\n\nThe reply's maximum size", "\n\u{1f}| 54 | 2 | u16 | Pad x | 0 only |\n\nThe reply's maximum size")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r074_r2_universal_newlines_lone_r_starts_extra",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "\n\n### Path-MTU probe reply\n", "\r### Extra\rplain words here\n\n### Path-MTU probe reply\n")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r075_r2_splitlines_x85_starts_extra",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "\n\n### Path-MTU probe reply\n", "\u{85}### Extra\u{85}plain words here\n\n### Path-MTU probe reply\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r076_r2_splitlines_x1d_starts_extra",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "\n\n### Path-MTU probe reply\n", "\u{1d}### Extra\u{1d}plain words here\n\n### Path-MTU probe reply\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r077_r2_d_non_ascii_width_magic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "| 0 | 2 | u16 | Magic", "| 0 | \u{662} | u16 | Magic")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r078_r2_d_non_ascii_maximum_size_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "The reply's maximum size is 54 bytes", "The reply's maximum size is \u{665}\u{664} bytes")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r079_r2_offset_cell_4_0_python_fullmatch_d_fails",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe request", "| 40 | 8 | u64 | Recipient", "| 4_0 | 8 | u64 | Recipient")),
        expect: &[("AC1", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r080_r2_toml_offset_as_string_384",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "offset = 384", "offset = \" +384 \"")),
        expect: &[],
    },
    Denial {
        name: "review_r081_r2_toml_offset_as_string_with_non_ascii_digits",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "offset = 384", "offset = \"\u{663}\u{668}\u{664}\"")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r082_r2_toml_offset_as_string_with_underscore_3_84",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "offset = 384", "offset = \"3_84\"")),
        expect: &[],
    },
    Denial {
        name: "review_r083_r2_toml_dscp_hex_0x28",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40", "dscp = 0x28")),
        expect: &[],
    },
    Denial {
        name: "review_r084_r2_toml_dscp_octal_0o50",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40", "dscp = 0o50")),
        expect: &[],
    },
    Denial {
        name: "review_r085_r2_toml_dscp_leading_zero_040_invalid_toml",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40", "dscp = 040")),
        expect: &[],
    },
    Denial {
        name: "review_r086_r2_toml_dscp_bool_true",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40", "dscp = true")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r087_r2_toml_duplicate_key_dscp",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "dscp = 40\n", "dscp = 46\ndscp = 40\n")),
        expect: &[],
    },
    Denial {
        name: "review_r088_r2_toml_literal_multi_line_string_three_single_quotes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "layer = \"control\"\n", "layer = \"control\"\nnote = '''valid TOML'''\n")),
        expect: &[("AC2", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r089_r2_toml_dotted_key_on_the_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "layer = \"control\"\n", "layer = \"control\"\nmeta.note = \"x\"\n")),
        expect: &[],
    },
    Denial {
        name: "review_r090_r2_toml_unicode_escape_in_layer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "layer = \"control\"", "layer = \"contro\\u006C\"")),
        expect: &[],
    },
    Denial {
        name: "review_r091_r2_toml_bare_unquoted_value_c_116_invalid_toml",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "exempt_from_control_bound = \"C-116\"", "exempt_from_control_bound = C-116")),
        expect: &[],
    },
    Denial {
        name: "review_r092_r2_toml_rejection_reasons_an_integer",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"ReservedNonZero\"]", "rejection_reasons = 5")),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "review_r093_r2_toml_rejection_reasons_an_inline_table",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "rejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"ReservedNonZero\"]", "rejection_reasons = { BadLength = 1 }")),
        expect: &[],
    },
    Denial {
        name: "review_r094_r2_toml_rejection_reasons_mixed_array",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "rejection_reasons = [\"TooShort\",", "rejection_reasons = [3, \"TooShort\",")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r095_r2_toml_comment_inside_multi_line_array",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "rejection_reasons = [\"TooShort\", ", "rejection_reasons = [ # first\n \"TooShort\", ")),
        expect: &[],
    },
    Denial {
        name: "review_r096_r2_toml_max_size_float_65507_0",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe request\"", "max_size = 65507", "max_size = 65507.0")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r097_r2_toml_max_size_date",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"path-MTU probe reply\"", "max_size = 54", "max_size = 1979-05-27")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r098_r2_tablerules_lineregex_separator_with_spaces",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "|---|---|---|---|---|", "| - | - | - | - | - |")),
        expect: &[],
    },
    Denial {
        name: "review_r099_r2_tablerules_drop_reply_table_without_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Path-MTU probe reply", "|---|---|---|---|---|\n", "")),
        expect: &[("AC1", ""), ("AC4", "")],
    },
];

/// Applies `edit` to the probe request's `[[message]]` block of wire.toml
/// only (up to its first `[[message.field]]`).
fn within_toml_message(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    let start = t.find(PROBE_REQUEST).expect("the probe request is in wire.toml");
    let end = t[start..].find("[[message.field]]").map_or(t.len(), |e| start + e);
    format!("{}{}{}", &t[..start], edit(&t[start..end]), &t[end..])
}
