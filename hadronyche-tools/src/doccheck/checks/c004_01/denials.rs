//! The denial list of 004-01: the real documents with one edit, and what the
//! port (and the old script) must report. Consumed by the tests and by
//! `doccheck-equiv 004-01 --denials-only`. An entry with no expectation is
//! one the port must pass (the script passes it too).
//!
//! Not in the list: "C-22 has no value in the Constants subsection" needs
//! the constants table and wire.md edited together (a denial edits one
//! file); `tests/doc_004_01.rs` carries it as a test.

use super::{CONSTANTS_MD, WIRE_MD};
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{replace_all, replace_in_line, within_section};
use crate::doccheck::text::rx;

const BO: &str = "^Byte order and alignment$";
const ALIGN: &str = "^Alignment and padding$";
const BITS: &str = "^Bit-level encoding of samples$";

fn re_all(t: &str, pattern: &str, to: &str) -> String {
    rx(pattern).replace_all_literal(t, to)
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// Replaces `from` in every heading line whose text matches `pattern`.
fn heads(t: &str, pattern: &str, from: &str, to: &str) -> String {
    let (h, f) = (rx(&format!(r"(?i)^#{{1,6}}\s+.*(?:{pattern})")), rx(from));
    t.split('\n')
        .map(|l| {
            if h.is_match(l) {
                f.replace_all_literal(l, to)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replaces `from` in every line matching `line`.
fn lines_replace(t: &str, line: &str, from: &str, to: &str) -> String {
    let (h, f) = (rx(line), rx(from));
    t.split('\n')
        .map(|l| {
            if h.is_match(l) {
                f.replace_all_literal(l, to)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sets cell `k` (0 = the id) of the first row matching `row` to `text`.
fn set_cell(t: &str, row: &str, k: usize, text: &str) -> String {
    let re = rx(row);
    let mut done = false;
    t.split('\n')
        .map(|l| {
            if !done && re.is_match(l) {
                done = true;
                let mut cells: Vec<String> = l.split('|').map(String::from).collect();
                cells[k + 1] = format!(" {text} ");
                cells.join("|")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `sec_layer`: `edit` applied to the Layer overview section only.
fn sec_layer(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Layer overview$", edit)
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[
            ("AC1", "does not exist"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
            ("AC5", ""),
            ("AC6", ""),
            ("AC7", ""),
        ],
    },
    Denial {
        name: "ac1_denial_no_byte_order_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            heads(
                t,
                r"byte[\s-]*order",
                r"(?i)byte[\s-]*order",
                "octet sequence",
            )
        }),
        expect: &[("AC1", "no byte-order section heading")],
    },
    Denial {
        name: "ac1_denial_no_single_order",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BO, |t| {
                re_all(
                    t,
                    r"(?i)(?:big|little)[-\s]endian|least[-\s]+significant[-\s]+byte[-\s]+first|\bLSB[-\s]+first\b|network\s+byte\s+order|most[-\s]+significant[-\s]+byte[-\s]+first|\bMSB[-\s]+first\b",
                    "x",
                )
            })
        }),
        expect: &[("AC1", "does not state which single order")],
    },
    Denial {
        name: "ac1_denial_two_orders",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BO, |t| {
                append(t, "Every multi-byte field is big-endian.")
            })
        }),
        expect: &[("AC1", "more than one order")],
    },
    Denial {
        name: "ac1_denial_no_decision_tag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BO, |t| replace_all(t, r"\[decision\]", "[choice]"))
        }),
        expect: &[("AC1", "does not carry the [decision] tag")],
    },
    Denial {
        name: "ac1_denial_no_justification",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BO, |t| {
                re_all(
                    t,
                    r"(?i)\b(justif\w*|rationale|because|since|reason\w*|we choose|chosen|decided|therefore|thus|so\s+that|ensures?|avoids?|simplif\w*|minimi[sz]\w*|makes?|allows?|matches?|consistent|portable|network\s+byte\s+order|most[-\s]+significant|byte\s+first)\b",
                    "x",
                )
            })
        }),
        expect: &[("AC1", "no justification")],
    },
    Denial {
        name: "ac2_denial_no_alignment_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| heads(t, "align", r"(?i)align\w*", "layout")),
        expect: &[("AC2", "no alignment section heading")],
    },
    Denial {
        name: "ac2_denial_width_1_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, ALIGN, |t| re_all(t, r"\b1\b", "x"))),
        expect: &[("AC2", "for 1-byte fields")],
    },
    Denial {
        name: "ac2_denial_width_2_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, ALIGN, |t| re_all(t, r"\b2\b", "x"))),
        expect: &[("AC2", "for 2-byte fields")],
    },
    Denial {
        name: "ac2_denial_width_4_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, ALIGN, |t| re_all(t, r"\b4\b", "x"))),
        expect: &[("AC2", "for 4-byte fields")],
    },
    Denial {
        name: "ac2_denial_width_8_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, ALIGN, |t| re_all(t, r"\b8\b", "x"))),
        expect: &[("AC2", "for 8-byte fields")],
    },
    Denial {
        name: "ac2_denial_width_16_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, ALIGN, |t| re_all(t, r"\b16\b", "x"))),
        expect: &[("AC2", "for 16-byte fields")],
    },
    Denial {
        name: "ac2_denial_bit_fields_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, ALIGN, |t| {
                re_all(
                    t,
                    r"(?i)sub[-\s]?byte|bit[-\s]?fields?|bitfield|\bbits?\b",
                    "x",
                )
            })
        }),
        expect: &[("AC2", "for sub-byte bit fields")],
    },
    Denial {
        name: "ac2_denial_an_empty_answer_does_not_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, ALIGN, |t| set_cell(t, r"^\| 16 bytes", 1, "\u{2014}"))
        }),
        expect: &[("AC2", "for 16-byte fields")],
    },
    Denial {
        name: "ac3_denial_no_bit_level_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            heads(
                t,
                r"bit[-\s]?level\s+encoding",
                r"(?i)bit[-\s]?level\s+encoding",
                "Sample layout",
            )
        }),
        expect: &[("AC3", "no 'bit-level encoding' heading")],
    },
    Denial {
        name: "ac3_denial_f32_heading_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| heads(t, r"f32 samples", "f32 samples", "32-bit floats")),
        expect: &[("AC3", "names f32")],
    },
    Denial {
        name: "ac3_denial_q131_heading_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            heads(
                t,
                r"Q1\.31 samples",
                r"Q1\.31 samples",
                "fixed-point samples",
            )
        }),
        expect: &[("AC3", "names Q1.31")],
    },
    Denial {
        name: "ac3_denial_f32_not_defined",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, BITS, |t| re_all(t, r"(?i)\bf32\b", "float"))),
        expect: &[("AC3", "does not define f32")],
    },
    Denial {
        name: "ac3_denial_q131_not_defined",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| re_all(t, r"(?i)\bq1[.\s_]?31\b", "fixed"))
        }),
        expect: &[("AC3", "does not define Q1.31")],
    },
    Denial {
        name: "ac3_denial_not_four_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| {
                re_all(t, r"(?i)\b4\s*[- ]?bytes?\b|\b32\s*[- ]?bits?\b", "x")
            })
        }),
        expect: &[("AC3", "f32 is not stated as 4 bytes")],
    },
    Denial {
        name: "ac3_denial_q131_not_four_bytes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| {
                re_all(t, r"(?i)\b4\s*[- ]?bytes?\b|\b32\s*[- ]?bits?\b", "x")
            })
        }),
        expect: &[("AC3", "Q1.31 is not stated as 4 bytes")],
    },
    Denial {
        name: "ac3_denial_no_ieee_754",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| re_all(t, r"(?i)ieee\s*[- ]?754|binary32", "x"))
        }),
        expect: &[("AC3", "does not cite IEEE 754 binary32")],
    },
    Denial {
        name: "ac3_denial_no_exponent",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| re_all(t, r"(?i)\bexponent\b", "x"))
        }),
        expect: &[("AC3", "does not name the exponent field")],
    },
    Denial {
        name: "ac3_denial_no_mantissa",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| {
                re_all(t, r"(?i)\bmantissa\b|\bfraction\b", "x")
            })
        }),
        expect: &[("AC3", "does not name the mantissa/fraction field")],
    },
    Denial {
        name: "ac3_denial_no_sign",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, BITS, |t| re_all(t, r"(?i)\bsign\b", "x"))),
        expect: &[("AC3", "does not name the sign bit")],
    },
    Denial {
        name: "ac3_denial_no_twos_complement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| {
                re_all(t, r"(?i)two'?s\s+complement|2'?s\s+complement", "x")
            })
        }),
        expect: &[("AC3", "does not state two's complement")],
    },
    Denial {
        name: "ac3_denial_no_31_fractional_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, BITS, |t| re_all(t, r"\b31\b", "x"))),
        expect: &[("AC3", "does not state its 31 fractional bits")],
    },
    Denial {
        name: "ac3_denial_no_fractional_scale",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, BITS, |t| {
                re_all(
                    t,
                    r"(?i)fraction\w*|\bscale\b|2\s*\^?\s*[-\u{2212}]?\s*31",
                    "x",
                )
            })
        }),
        expect: &[("AC3", "does not state its fractional scale")],
    },
    Denial {
        name: "ac4_denial_no_field_table_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            heads(
                t,
                r"field[\s-]*table",
                r"(?i)field[\s-]*table",
                "Message layout",
            )
        }),
        expect: &[("AC4", "no field-table format section heading")],
    },
    Denial {
        name: "ac4_denial_not_five_columns",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| lines_replace(t, r"^\| Offset", "Valid range", "Range")),
        expect: &[("AC4", "no table with exactly the five columns")],
    },
    Denial {
        name: "ac4_denial_no_bounded_variable_length_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Field-table format$", |t| {
                re_all(t, r"(?i)variable[\s-]*length", "dynamic")
            })
        }),
        expect: &[(
            "AC4",
            "no rule that every variable-length part has a stated maximum count",
        )],
    },
    Denial {
        name: "ac5_denial_a_placeholder_section_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| heads(t, "checksum", r"(?i)checksum", "crc")),
        expect: &[("AC5", "placeholder section missing: checksum")],
    },
    Denial {
        name: "ac5_denial_two_placeholders_share_a_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^## Slice header$",
                "Slice header",
                "Slice header checksum",
            )
        }),
        expect: &[("AC5", "shares its heading with")],
    },
    Denial {
        name: "ac6_denial_no_layer_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                replace_in_line(t, r"^\| Layer \| Carries", "Layer", "Tier")
            })
        }),
        expect: &[("AC6", "no layer table with Layer and DSCP columns")],
    },
    Denial {
        name: "ac6_denial_five_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                replace_in_line(t, r"^\| Measurement \|", "$", "\n| Extra | x | y | 40 |")
            })
        }),
        expect: &[("AC6", "layer table has 5 rows, expected four layers")],
    },
    Denial {
        name: "ac6_denial_a_layer_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                replace_in_line(t, r"^\| Clock sync \|", "Clock sync", "Timing")
            })
        }),
        expect: &[("AC6", "layer table has no sync row")],
    },
    Denial {
        name: "ac6_denial_two_rows_for_a_layer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                replace_in_line(t, r"^\| Media \|", "Media", "Media sync")
            })
        }),
        expect: &[("AC6", "layer table has 2 sync rows, expected one")],
    },
    Denial {
        name: "ac6_denial_the_control_dscp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                set_cell(t, r"^\| Discovery and control", 3, "CS6 (48)")
            })
        }),
        expect: &[("AC6", "control layer DSCP is")],
    },
    Denial {
        name: "ac6_denial_the_sync_dscp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                set_cell(t, r"^\| Clock sync", 3, "CS5 (40)")
            })
        }),
        expect: &[("AC6", "sync layer DSCP is")],
    },
    Denial {
        name: "ac6_denial_the_media_dscp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                set_cell(t, r"^\| Media \|", 3, "CS5 (40)")
            })
        }),
        expect: &[("AC6", "media layer DSCP is")],
    },
    Denial {
        name: "ac6_denial_the_measurement_dscp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                set_cell(t, r"^\| Measurement \|", 3, "reports CS5 (40)")
            })
        }),
        expect: &[("AC6", "measurement layer DSCP is")],
    },
    Denial {
        name: "ac6_denial_no_ipv4",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| re_all(t, r"(?i)\bIPv4\b", "IP"))
        }),
        expect: &[("AC6", "layer overview does not state IPv4")],
    },
    Denial {
        name: "ac6_denial_no_multicast",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                re_all(t, r"(?i)\bmulticast\b", "group")
            })
        }),
        expect: &[("AC6", "layer overview does not state multicast")],
    },
    Denial {
        name: "ac6_denial_no_ttl_1",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Layer overview$", |t| {
                re_all(t, r"(?i)\bTTL\s*(?:=|of\s*)?1\b", "hop limit")
            })
        }),
        expect: &[("AC6", "does not state multicast TTL 1")],
    },
    Denial {
        name: "ac7_denial_constants_missing",
        file: CONSTANTS_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC7", "does not exist; cannot compare constants")],
    },
    Denial {
        name: "ac7_denial_no_constants_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| heads(t, "constant", r"(?i)constants?", "Values")),
        expect: &[("AC7", "no Constants subsection")],
    },
    Denial {
        name: "ac7_denial_a_value_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Constants$", |t| {
                replace_in_line(t, r"^\| C-17 \|", "1 252", "1 253")
            })
        }),
        expect: &[("AC7", "C-17 mismatch")],
    },
    Denial {
        name: "ac7_denial_an_id_missing_from_the_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, "^Constants$", |t| {
                t.split('\n')
                    .filter(|l| !l.contains("C-17"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }),
        expect: &[("AC7", "C-17 missing from the Constants subsection")],
    },
    Denial {
        name: "ac7_denial_an_id_missing_from_the_source",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| {
            t.split('\n')
                .filter(|l| !l.starts_with("| C-17 |"))
                .collect::<Vec<_>>()
                .join("\n")
        }),
        expect: &[("AC7", "C-17 is not present in")],
    },
    Denial {
        name: "a_dscp_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec_layer(t, |s| {
                replace_in_line(
                    s,
                    r"^\| Clock sync \|",
                    r"CS6 \(48\)",
                    "CS6 (48000000000000000000000000000000000000000)",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_dscp_with_a_superscript_still_reads_its_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec_layer(t, |s| {
                replace_in_line(s, r"^\| Clock sync \|", r"CS6 \(48\)", "CS6 (48\u{b2})")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_decimal_width_is_not_a_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, ALIGN, |s| {
                lines_replace(
                    &re_all(s, r"(?i)\b(?:1|2|4|8)-byte|\b(?:1|2|4|8) byte", "5.5 byte"),
                    r"^\|",
                    r"\b16\b",
                    "1.6",
                )
            })
        }),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "crlf_line_ends_pass",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| t.replace('\n', "\r\n")),
        expect: &[],
    },
    Denial {
        name: "a_trailing_tab_after_a_heading_passes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, "^### Byte order$", "$", "\t")),
        expect: &[],
    },
    Denial {
        name: "an_en_dash_for_an_empty_answer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            within_section(t, ALIGN, |s| set_cell(s, r"^\| 16 bytes", 1, "\u{2013}"))
        }),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "a_micro_sign_in_a_constant_value",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-40 \|", "100 ms", "100 \u{b5}s")),
        expect: &[],
    },
];
