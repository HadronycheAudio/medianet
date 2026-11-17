//! The denial list of 004-15: the real documents with one edit, and what the
//! port (and the old script) must report. Consumed by the tests and by
//! `doccheck-equiv 004-15 --denials-only`. An entry with no expectation is
//! one the port must pass (the script passes it too).
//!
//! Not in the list: "the section does not name C-118 as marked for recompute"
//! edits the constants and wire.md together; `tests/doc_004_15.rs` carries it.

use super::{CONSTANTS, WIRE_MD};
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{
    anchored_replace, delete_row, rename_heading, replace_all, replace_in_line, strip_tags,
    within_section,
};
use crate::doccheck::text::rx;
use crate::doccheck::wire_toml::WIRE_TOML;

/// `edit` applied to the "Wire size and capacity recompute" subsection only.
fn sub(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Wire size and capacity recompute$", edit)
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// Replaces every match of the regex `pattern` in `t` (panics if none).
fn rep(t: &str, pattern: &str, to: &str) -> String {
    replace_all(t, pattern, to)
}

/// A wire-size table of one row appended to the subsection.
fn size_row(t: &str, cells: [&str; 5]) -> String {
    sub(t, |s| {
        append(
            s,
            &format!(
                "| Packet | UDP payload | Bytes on the wire | Per stream | Per channel |\n|---|---|---|---|---|\n| {} |",
                cells.join(" | ")
            ),
        )
    })
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("setup", "cannot read wire.md")],
    },
    Denial {
        name: "denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("setup", "cannot read wire.toml")],
    },
    Denial {
        name: "denial_constants_missing",
        file: CONSTANTS,
        edit: DenialEdit::Delete,
        expect: &[("setup", "cannot read 02-constants.md")],
    },
    Denial {
        name: "denial_a_constant_row_missing",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| C-20 \|")),
        expect: &[("setup", "is missing")],
    },
    Denial {
        name: "denial_c22_does_not_parse",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| rep(t, " ch: ", " channels: ")),
        expect: &[("setup", "C-22 does not parse")],
    },
    Denial {
        name: "denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("[[message.field]]\n{t}")),
        expect: &[("setup", "does not parse")],
    },
    Denial {
        name: "denial_the_toml_lacks_the_layout_tables",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| rep(t, "name = \"slice checksum\"", "name = \"slice sum\"")),
        expect: &[("setup", "lacks the media packet")],
    },
    Denial {
        name: "ac1_denial_a_budget_default_total_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| rep(t, "default_total = 1198", "default_total = 1199")),
        expect: &[("AC1", "default_total is 1199")],
    },
    Denial {
        name: "ac1_denial_no_wire_size_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            rename_heading(t, "^Wire size and capacity recompute$", "Sizes")
        }),
        expect: &[("AC1", "no heading naming the wire size")],
    },
    Denial {
        name: "ac1_denial_an_equation_that_does_not_hold",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The on-wire size of one default packet",
                    "= 1 264 bytes",
                    "= 1 265 bytes",
                )
            })
        }),
        expect: &[("AC1", "arithmetic does not hold")],
    },
    Denial {
        name: "ac1_denial_the_default_wire_size_stated_wrongly",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The on-wire size of one default packet",
                    r"1 198 \+ 8 \+ 20 \+ 38 = 1 264 bytes",
                    "1 198 + 8 + 20 + 37 = 1 263 bytes",
                )
            })
        }),
        expect: &[("AC1", "the default packet's on-wire size is stated as 1263")],
    },
    Denial {
        name: "ac1_denial_no_arithmetic_for_the_default_wire_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, "1 198", "1 197"))),
        expect: &[("AC1", "no arithmetic or table states the on-wire size")],
    },
    Denial {
        name: "ac1_denial_the_ethernet_header_not_beside_its_name",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\b14\b", "15"))),
        expect: &[
            ("AC1", "the Ethernet header of 14 bytes is not stated"),
            ("AC2", ""),
        ],
    },
    Denial {
        name: "ac1_denial_the_fcs_not_beside_its_name",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(t, r"(?i)\bFCS\b|frame check|check sequence", "tail")
            })
        }),
        expect: &[("AC1", "the Ethernet FCS of 4 bytes is not stated")],
    },
    Denial {
        name: "ac1_denial_the_preamble_not_beside_its_name",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\b8\b", "9"))),
        expect: &[
            (
                "AC1",
                "the Ethernet preamble and start-of-frame delimiter of 8 bytes",
            ),
            ("AC4", ""),
        ],
    },
    Denial {
        name: "ac1_denial_the_gap_not_beside_its_name",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\b12\b", "13"))),
        expect: &[("AC1", "the Ethernet inter-frame gap of 12 bytes")],
    },
    Denial {
        name: "ac1_denial_the_ethernet_total_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\b38\b", "39"))),
        expect: &[
            ("AC1", "the Ethernet framing total of 38 bytes"),
            ("AC4", ""),
        ],
    },
    Denial {
        name: "ac1_denial_no_rate_per_stream",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"20\.22\d*", "20.99"))),
        expect: &[("AC1", "no bit rate per stream of 20.2 Mbit/s")],
    },
    Denial {
        name: "ac1_denial_no_rate_per_channel",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"3\.37", "3.99"))),
        expect: &[
            ("AC1", "no bit rate per channel of 3.37 Mbit/s"),
            ("AC2", ""),
        ],
    },
    Denial {
        name: "ac1_denial_no_unit_for_a_source",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"80\.90 Mbit/s", "81.50 Mbit/s"))),
        expect: &[("AC1", "no unit gives 24 channels: 4 streams")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_without_a_column",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                append(
                    t,
                    "| Packet | UDP payload | Bytes on the wire |\n|---|---|---|\n| Default x | 1 198 | 1 198 + 8 + 20 + 38 = 1 264 |",
                )
            })
        }),
        expect: &[("AC1", "lacks a payload, wire, stream or channel cell")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_without_arithmetic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                ["Default x", "tbd", "1 198 + 8 + 20 + 38 = 1 264", "x", "y"],
            )
        }),
        expect: &[("AC1", "states no payload or no on-the-wire arithmetic")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_with_the_wrong_payload",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Default x",
                    "1 197",
                    "1 197 + 8 + 20 + 38 = 1 263",
                    "1 263 × 8 × 2 000 = 20 208 000 bit/s, 20.21 Mbit/s",
                    "3.37 Mbit/s",
                ],
            )
        }),
        expect: &[("AC1", "payload 1197 is not the 004-14 default total")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_not_payload_plus_66",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Other x",
                    "1 198",
                    "1 198 + 8 + 20 + 37 = 1 263",
                    "1 263 × 8 × 2 000 = 20 208 000 bit/s, 20.21 Mbit/s",
                    "3.37 Mbit/s",
                ],
            )
        }),
        expect: &[("AC1", "on the wire 1263 is not payload 1198 + 66")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_without_a_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Other x",
                    "1 198",
                    "1 198 + 8 + 20 + 38 = 1 264",
                    "1 264 × 8 × 2 000 = 20 224 000 bit/s",
                    "3.37 Mbit/s",
                ],
            )
        }),
        expect: &[("AC1", "states no Mbit/s rate")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_with_the_wrong_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Other x",
                    "1 198",
                    "1 198 + 8 + 20 + 38 = 1 264",
                    "1 264 × 8 × 2 000 = 20 224 000 bit/s, 20.99 Mbit/s",
                    "3.37 Mbit/s",
                ],
            )
        }),
        expect: &[("AC1", "rate 20.99 Mbit/s is not")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_with_the_wrong_bit_rate_equation",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Other x",
                    "1 198",
                    "1 198 + 8 + 20 + 38 = 1 264",
                    "1 264 × 8 × 2 000 = 20 224 001 bit/s, 20.22 Mbit/s",
                    "3.37 Mbit/s",
                ],
            )
        }),
        expect: &[("AC1", "does not give 20224000 bit/s")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_without_a_per_channel_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Other x",
                    "1 198",
                    "1 198 + 8 + 20 + 38 = 1 264",
                    "1 264 × 8 × 2 000 = 20 224 000 bit/s, 20.22 Mbit/s",
                    "tbd",
                ],
            )
        }),
        expect: &[("AC1", "states no per-channel rate")],
    },
    Denial {
        name: "ac1_denial_a_wire_size_row_with_the_wrong_per_channel_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size_row(
                t,
                [
                    "Other x",
                    "1 198",
                    "1 198 + 8 + 20 + 38 = 1 264",
                    "1 264 × 8 × 2 000 = 20 224 000 bit/s, 20.22 Mbit/s",
                    "3.99 Mbit/s",
                ],
            )
        }),
        expect: &[("AC1", "per-channel 3.99 Mbit/s is not")],
    },
    Denial {
        name: "ac2_denial_no_byte_difference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\b56\b", "55"))),
        expect: &[("AC1", ""), ("AC2", "no unit gives the byte difference")],
    },
    Denial {
        name: "ac2_denial_a_percent_beside_c20_that_is_not_the_share",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The difference from the design's derived figures",
                    "56 bytes smaller, 4.2 % less",
                    "56 bytes smaller, 4.9 % less",
                )
            })
        }),
        expect: &[("AC2", "a percent beside C-20")],
    },
    Denial {
        name: "ac2_denial_a_percent_beside_c21_that_is_not_the_share",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The difference from the design's derived figures",
                    r"\(20.224 Mbit/s\), 4.2 % less",
                    "(20.224 Mbit/s), 4.9 % less",
                )
            })
        }),
        expect: &[("AC2", "a percent beside C-21")],
    },
    Denial {
        name: "ac2_denial_no_rate_difference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The difference from the design's derived figures",
                    "0.88 Mbit/s lower",
                    "0.99 Mbit/s lower",
                )
            })
        }),
        expect: &[("AC2", "no unit gives the rate difference")],
    },
    Denial {
        name: "ac2_denial_a_c22_difference_that_is_not_computed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(t, r"^\| 24 channels", "−3.10 Mbit/s", "−3.50 Mbit/s")
            })
        }),
        expect: &[(
            "AC2",
            "the 24-channel difference from C-22 is stated as -3.5",
        )],
    },
    Denial {
        name: "ac2_denial_a_c22_share_that_is_not_computed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(t, r"^\| 24 channels", "−3.7 %", "−3.0 %")
            })
        }),
        expect: &[(
            "AC2",
            "the 24-channel share of the C-22 difference is stated as 3 %",
        )],
    },
    Denial {
        name: "ac2_denial_a_share_against_c21_times_streams_that_is_not_the_per_stream_share",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The difference from the design's derived figures",
                    "every source is 4.2 % lower \\(−4.15 %\\)",
                    "every source is 4.2 % lower (−4.9 %)",
                )
            })
        }),
        expect: &[(
            "AC2",
            "the share against C-21 x streams is stated as -4.9 %",
        )],
    },
    Denial {
        name: "ac2_denial_a_plain_share_against_c21_times_streams",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The difference from the design's derived figures",
                    "every source is 4.2 % lower",
                    "every source is 4.9 % lower",
                )
            })
        }),
        expect: &[("AC2", "the share against C-21 x streams is stated as 4.9 %")],
    },
    Denial {
        name: "ac2_denial_a_per_channel_figure_that_is_not_computed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The difference from the design's derived figures",
                    "per channel it is 3.37 Mbit/s",
                    "per channel it is 3.99 Mbit/s",
                )
            })
        }),
        expect: &[("AC2", "a per-channel figure of 3.99 Mbit/s")],
    },
    Denial {
        name: "ac3_denial_no_five_percent_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, "5 %", "five percent"))),
        expect: &[("AC3", "no sentence states the 5 % rule")],
    },
    Denial {
        name: "ac3_denial_a_five_percent_sentence_without_a_direction",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(
                    &rep(t, "below the 5 %", "at the 5 %"),
                    "within 5 %",
                    "at 5 %",
                )
            })
        }),
        expect: &[("AC3", "states no direction")],
    },
    Denial {
        name: "ac3_denial_a_direction_that_contradicts_the_computed_difference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(t, "are both below the 5 %", "both exceed the 5 %")
            })
        }),
        expect: &[("AC3", "says the difference exceeds 5 %, but the computed")],
    },
    Denial {
        name: "ac3_denial_within_five_percent_without_the_constants_standing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rx(r"(?i)no constant[^.;]{0,40}(?:recomput|changed|updated|re-?derived)|\bconstants?\b[^.;]{0,60}\bstand\b|not recomputed|constants? (?:are|is|remain|remains) unchanged").replace_all(t, "x")
            })
        }),
        expect: &[("AC3", "does not say that no constant is recomputed")],
    },
    Denial {
        name: "ac3_denial_over_five_percent_but_the_section_says_the_constants_stand",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-20 \|", "1 320", "1 200")),
        expect: &[
            ("AC2", ""),
            ("AC3", "the section still says the constants stand"),
        ],
    },
    Denial {
        name: "ac3_denial_over_five_percent_with_neither_update_nor_open_item",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-20 \|", "1 320", "1 200")),
        expect: &[
            ("AC2", ""),
            ("AC3", "neither 'constants table updated' nor an open item"),
        ],
    },
    Denial {
        name: "ac4_denial_no_controller_cost_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(
                    &rep(t, "C-219", "C-2l9"),
                    r"(?i)per slice|controller bytes",
                    "x",
                )
            })
        }),
        expect: &[("AC4", "no paragraph states the C-219 wire cost")],
    },
    Denial {
        name: "ac4_denial_the_overhead_not_shown_as_28_plus_38",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"28 \+ 38", "28 plus 38"))),
        expect: &[("AC4", "not shown as 28 + 38 with 66")],
    },
    Denial {
        name: "ac4_denial_no_symbolic_formula",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"= H \+ c \+ 28 \+ 38", "= H + c + 66"))),
        expect: &[("AC4", "no symbolic formula")],
    },
    Denial {
        name: "ac4_denial_not_stated_at_2000_packets_a_second",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, "2 000", "3 000"))),
        expect: &[("AC1", ""), ("AC4", "not stated at 2 000 packets/s")],
    },
    Denial {
        name: "ac4_denial_no_bit_rate_formula",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, "× 2 000", "· 2 000"))),
        expect: &[("AC4", "no bit-rate formula of the form bytes x 8 x 2 000")],
    },
    Denial {
        name: "ac4_denial_the_header_term_not_named_as_the_cap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\b100\b", "101"))),
        expect: &[
            ("AC1", ""),
            (
                "AC4",
                "the header term of the formula is not named as the 100-byte cap",
            ),
        ],
    },
    Denial {
        name: "ac4_denial_a_worked_example_whose_header_term_is_not_a_budget_figure",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^Worked examples of the C-219 formula",
                    r"100 \+ 30 \+ 28 \+ 38 = 196",
                    "99 + 30 + 28 + 38 = 195",
                )
            })
        }),
        expect: &[("AC4", "the header term is not a figure of the budget")],
    },
    Denial {
        name: "ac4_denial_no_worked_example",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, r"\+ 28 \+ 38 =", "+ 66 ="))),
        expect: &[("AC4", "no worked example 'H + c + 28 + 38 = N'")],
    },
    Denial {
        name: "ac4_denial_no_controller_example",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(
                    &rep(
                        &rep(
                            t,
                            r"100 \+ 30 \+ 28 \+ 38 = 196",
                            "100 + 1 152 + 28 + 38 = 1 318",
                        ),
                        r"100 \+ 1 028 \+ 28 \+ 38 = 1 194",
                        "100 + 1 152 + 28 + 38 = 1 318",
                    ),
                    r"64 \+ 30 \+ 28 \+ 38 = 160",
                    "64 + 1 152 + 28 + 38 = 1 282",
                )
            })
        }),
        expect: &[("AC4", "has no controller example")],
    },
    Denial {
        name: "ac4_denial_no_bit_rate_example",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sub(t, |t| rep(t, "× 8 × 2 000 =", "× 8 × 2 000 is"))),
        expect: &[("AC4", "no worked bit-rate example")],
    },
    Denial {
        name: "ac4_denial_a_bit_rate_example_that_does_not_use_a_controller_total",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(
                    &rep(&rep(t, r"\b196 × 8", "197 × 8"), r"1 194 × 8", "1 195 × 8"),
                    r"\b160 × 8",
                    "161 × 8",
                )
            })
        }),
        expect: &[
            ("AC1", ""),
            ("AC4", "does not use a controller example's byte count"),
        ],
    },
    Denial {
        name: "ac5_denial_an_audio_declared_rate_the_layout_does_not_compute",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(t, "21.09 Mbit/s, at 2 000", "21.5 Mbit/s, at 2 000")
            })
        }),
        expect: &[("AC5", "a figure the layout does not compute")],
    },
    Denial {
        name: "ac5_denial_an_audio_declared_rate_that_is_not_the_cap_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(
                    t,
                    "21 088 000 bit/s, 21.09 Mbit/s, at 2 000",
                    "20 224 000 bit/s, 20.22 Mbit/s, at 2 000",
                )
            })
        }),
        expect: &[("AC1", ""), ("AC5", "is not the cap form")],
    },
    Denial {
        name: "ac5_denial_an_audio_declared_rate_with_other_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                rep(
                    t,
                    "1 318 bytes per packet on the wire",
                    "1 319 bytes per packet on the wire",
                )
            })
        }),
        expect: &[
            ("AC1", ""),
            ("AC5", "the audio declared rate's bytes are not 1318"),
        ],
    },
    Denial {
        name: "ac5_denial_an_overhead_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[overhead_check]\nbytes = 64")),
        expect: &[("AC5", "the overhead of 66 bytes")],
    },
    Denial {
        name: "ac5_denial_an_ethernet_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[ethernet_overhead]\nbytes = 37")),
        expect: &[("AC5", "the Ethernet framing of 38 bytes")],
    },
    Denial {
        name: "ac5_denial_an_ipv4_header_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[wire_sizes]\nipv4_header = 21")),
        expect: &[("AC5", "the IPv4 header of 20 bytes")],
    },
    Denial {
        name: "ac5_denial_a_udp_header_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[wire_sizes]\nudp_header = 9")),
        expect: &[("AC5", "the UDP header of 8 bytes")],
    },
    Denial {
        name: "ac5_denial_a_default_wire_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[wire_sizes]\ndefault_wire = 1000")),
        expect: &[("AC5", "the default on-wire size of 1264 bytes")],
    },
    Denial {
        name: "ac5_denial_a_maximum_wire_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[wire_sizes]\nwire_maximum = 1000")),
        expect: &[("AC5", "the format maximum on-wire size of 1321 bytes")],
    },
    Denial {
        name: "ac5_denial_a_stream_rate_key_that_differs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| append(t, "[rates]\ndefault_stream_rate = 1000")),
        expect: &[("AC5", "the default per-stream rate of 20224000 bit/s")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                strip_tags(t, "^The Ethernet figures are the ticket's own")
            })
        }),
        expect: &[("DoD", "has no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                append(
                    t,
                    "**Open item (x).** Whether the sizes hold is open `[decision]` until a later ticket.",
                )
            })
        }),
        expect: &[("DoD", "an open item names no owner or no falsifier")],
    },
    Denial {
        name: "dod_denial_a_framing_row_without_a_tag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(t, r"^\| Ethernet inter-frame gap", r"`\[memory\]`", "")
            })
        }),
        expect: &[("DoD", "has no [memory] or [standard] tag")],
    },
    Denial {
        name: "dod_denial_a_memory_framing_row_without_a_checking_task",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Ethernet inter-frame gap",
                    "; ticket 004-22 pins it",
                    "",
                )
            })
        }),
        expect: &[("DoD", "names no checking task")],
    },
    Denial {
        name: "dod_denial_a_standard_framing_row_without_a_clause",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(t, r"^\| Ethernet preamble", r"`\[memory\]`", "`[standard]`")
            })
        }),
        expect: &[(
            "DoD",
            "framing row tagged [standard] has no clause reference",
        )],
    },
    Denial {
        name: "dod_denial_a_standard_framing_sentence_without_a_clause",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |t| {
                replace_in_line(
                    t,
                    "^The Ethernet figures are the ticket's own",
                    "38 bytes of Ethernet in all `\\[memory\\]`",
                    "38 bytes of Ethernet in all `[standard]`",
                )
            })
        }),
        expect: &[(
            "DoD",
            "framing sentence tagged [standard] has no clause reference",
        )],
    },
    Denial {
        name: "c21_with_a_superscript_after_its_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(
                    s,
                    "^The difference from the design's derived figures",
                    "21.1 Mbit/s per stream",
                    "21.1\u{b2} Mbit/s per stream",
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_difference_written_with_a_superscript_minus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(s, r"^\| 24 channels", "−3.10 Mbit/s", "\u{207b}3.10 Mbit/s")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_percent_just_inside_the_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(
                    s,
                    "^The difference from the design's derived figures",
                    "56 bytes smaller, 4.2 % less",
                    "56 bytes smaller, 4.34 % less",
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_percent_just_outside_the_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(
                    s,
                    "^The difference from the design's derived figures",
                    "56 bytes smaller, 4.2 % less",
                    "56 bytes smaller, 4.35 % less",
                )
            })
        }),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "an_equation_exactly_on_its_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                append(s, "A check `[decision]` 50000 + 0 = 0.0 Mbit/s.")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "an_equation_just_inside_its_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                append(s, "A check `[decision]` 49999 + 0 = 0.0 Mbit/s.")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "an_equation_just_outside_its_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                append(s, "A check `[decision]` 50001 + 0 = 0.0 Mbit/s.")
            })
        }),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "an_equation_beyond_float_precision_fails",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                append(
                    s,
                    "A check `[decision]` 9007199254740993 + 0 = 9007199254740992.",
                )
            })
        }),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "an_equation_beyond_float_precision_holds",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                append(
                    s,
                    "A check `[decision]` 9007199254740992 + 1 = 9007199254740993.",
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "an_equation_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                append(
                    s,
                    "A check `[decision]` 100000000000000000000000000000000000000000 + 0 = 100000000000000000000000000000000000000000.",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_result_with_a_zero_decimal_passes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(
                    s,
                    "^The on-wire size of one default packet",
                    "= 1 264 bytes",
                    "= 1 264.0 bytes",
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_result_with_a_decimal_part_fails",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(
                    s,
                    "^The on-wire size of one default packet",
                    "= 1 264 bytes",
                    "= 1 264.5 bytes",
                )
            })
        }),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "a_payload_with_an_underscore",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sub(t, |s| {
                replace_in_line(
                    s,
                    r"^\| Default, mandatory parts only",
                    r"\| 1 198 \| 1 198 \+",
                    "| 1_198 | 1 198 +",
                )
            })
        }),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "a_c20_of_1e13",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-20 \|", "1 320", "10 000 000 000 000")
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_c20_beyond_i128",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^\| C-20 \|",
                "1 320",
                "100000000000000000000000000000000000000000",
            )
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_c20_of_zero_crashes",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-20 \|", "1 320", "0")),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "a_c21_of_zero_crashes",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-21 \|", "21.1", "0.0")),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "a_c04_of_zero_crashes",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-04 \|", "6 \\(", "0 (")),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "a_toml_value_with_trailing_tabs_and_a_comment_passes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"slice checksum\"",
                "max_size = 4",
                "max_size = 4\t  # four\t",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_field_before_any_message",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("[[message.field]]\n{t}")),
        expect: &[("setup", "")],
    },
    Denial {
        name: "an_array_table_after_a_table_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[zz]\n[[zz]]\n")),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "a_max_size_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"slice checksum\"",
                "max_size = 4",
                "max_size = 100000000000000000000000000000000000000000",
            )
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "fields_that_is_a_list_of_an_int_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "name = \"media packet\"",
                "name = \"media packet\"\nfields = [1]",
            )
        }),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "crlf_line_ends_pass",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| t.replace('\n', "\r\n")),
        expect: &[],
    },
];
