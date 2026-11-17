//! The denial list of 004-14: the real documents with one edit, and what the
//! port (and the old script) must report. Consumed by the tests and by
//! `doccheck-equiv 004-14 --denials-only`. An entry with no expectation is
//! one the port must pass (the script passes it too).
//!
//! Not in the list: "the verdict says falsified without five channels" edits
//! two files (the constants and wire.md); `tests/doc_004_14.rs` carries it.

use super::{CONSTANTS, WIRE_MD};
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{
    anchored_replace, delete_row, delete_section, rename_heading, replace_all, replace_in_line,
    strip_tags, within_section,
};
use crate::doccheck::text::rx;
use crate::doccheck::wire_toml::WIRE_TOML;

/// `edit` applied to the Size budget section only.
fn size(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Size budget$", edit)
}

/// Applies `f` to every line of `t` matching `pattern`.
fn map_lines(t: &str, pattern: &str, f: impl Fn(&str) -> String) -> String {
    let re = rx(pattern);
    let mut hit = false;
    let out: Vec<String> = t
        .split('\n')
        .map(|l| {
            if re.is_match(l) {
                hit = true;
                f(l)
            } else {
                l.to_string()
            }
        })
        .collect();
    assert!(hit, "map_lines: no line matches {pattern:?}");
    out.join("\n")
}

/// Deletes the first line matching `pattern`.
fn drop_line(t: &str, pattern: &str) -> String {
    let re = rx(pattern);
    let lines: Vec<&str> = t.split('\n').collect();
    let i = lines
        .iter()
        .position(|l| re.is_match(l))
        .unwrap_or_else(|| panic!("drop_line: no line matches {pattern:?}"));
    [&lines[..i], &lines[i + 1..]].concat().join("\n")
}

/// Replaces the first `from` after the first `anchor`.
fn after(t: &str, anchor: &str, from: &str, to: &str) -> String {
    anchored_replace(t, anchor, from, to)
}

/// The section with its "omitted" sentences reworded, so that the script's
/// second form ("a sentence on the fallback says nothing is omitted") does
/// not excuse the rule's lines, which are the lines that say "omit".
fn unexcused(t: &str) -> String {
    rx(r"\bomitted\b").replace_all(t, "dropped")
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// Appends a one-column table whose caption is blank (so only `header`
/// names its case) to the Size budget section.
fn table(t: &str, header: &str, rows: &[&str]) -> String {
    let body: Vec<String> = rows.iter().map(|r| format!("| {r} |")).collect();
    size(t, |s| {
        format!(
            "{}\n\n\n\n\n\n| Part | {header} |\n|---|---|\n{}\n",
            s.trim_end(),
            body.join("\n")
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
        name: "denial_a_constant_unreadable",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_all(
                t,
                "at most 100 bytes for all headers",
                "at most a hundred bytes for all headers",
            )
        }),
        expect: &[("setup", "is unreadable")],
    },
    Denial {
        name: "denial_fallback_mtu_disagrees_with_the_control_bound",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-06 \|", "1 280 bytes", "1 300 bytes")
        }),
        expect: &[("setup", "but C-17 states 1252")],
    },
    Denial {
        name: "denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("[[message.field]]\n{t}")),
        expect: &[("setup", "does not parse")],
    },
    Denial {
        name: "ac1_denial_no_size_budget_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Size budget$", "Budget of sizes")),
        expect: &[
            ("AC1", "no level-2 heading 'Size budget'"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
            ("AC5", ""),
            ("AC6", ""),
            ("pointer", ""),
        ],
    },
    Denial {
        name: "ac1_denial_a_column_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            table(
                t,
                "Alpha",
                &["Audio | 1 152", "Total | 1 152", "Spare | 320"],
            )
        }),
        expect: &[("AC1", "is not named as default, worst case or fallback")],
    },
    Denial {
        name: "ac1_denial_a_row_with_no_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            table(t, "Default", &["Audio | plenty", "Total | 5", "Spare | 1"])
        }),
        expect: &[("AC1", "has no number")],
    },
    Denial {
        name: "ac1_denial_a_row_not_a_budget_part",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            table(
                t,
                "Default",
                &[
                    "Audio | 1 152",
                    "Mystery | 3",
                    "Total | 1 155",
                    "Spare | 317",
                ],
            )
        }),
        expect: &[("AC1", "is not a recognised budget part")],
    },
    Denial {
        name: "ac1_denial_no_spare_row_in_a_column",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                format!(
                    "{}\n\n\n\n\n\n| Part | Default | Default again |\n|---|---|---|\n| Audio | 1 152 | 1 152 |\n| Total | 1 152 | 1 152 |\n| Spare | 320 |\n",
                    t.trim_end()
                )
            })
        }),
        expect: &[("AC1", "has no spare row"), ("AC2", ""), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_a_part_row_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| delete_row(t, r"^\| Checksum \(CRC-32C\)"))),
        expect: &[("AC1", "has no cksum row"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_packet_header_differs_from_the_toml",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Packet header \| 16",
                    r"\| 16 \| 16 \| 16 \|",
                    "| 17 | 16 | 16 |",
                )
            })
        }),
        expect: &[("AC1", "packet header [17] != 16"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_slice_header_differs_from_the_toml",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Slice header \| 22",
                    r"\| 22 \| 22 \| 22 \|",
                    "| 23 | 22 | 22 |",
                )
            })
        }),
        expect: &[("AC1", "slice header [23] != 22"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_content_header_differs_from_the_toml",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Content-message header",
                    r"\| 3 \| 3 \| 3 \|",
                    "| 4 | 3 | 3 |",
                )
            })
        }),
        expect: &[("AC1", "content-message header [4] != 3"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_audio_differs_from_c04_c02_c03",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Audio, 6",
                    r"\| 1 152 \| 1 152 \| 1 152 \|",
                    "| 1 151 | 1 152 | 1 152 |",
                )
            })
        }),
        expect: &[("AC1", "audio [1151] != 1152"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_checksum_differs_from_the_toml",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Checksum \(CRC",
                    r"\| 4 \| 4 \| 4 \|",
                    "| 5 | 4 | 4 |",
                )
            })
        }),
        expect: &[("AC1", "checksum [5] != 4"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_a_total_over_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(t, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 500")
            })
        }),
        expect: &[("AC1", "exceeds the 1472-byte bound"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_a_spare_that_is_not_the_bound_less_the_total",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(t, r"^\| Spare against 1 472", "= 274", "= 275")
            })
        }),
        expect: &[("AC1", "spare 275 != 1472 - 1198"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_default_column_carries_an_announcement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Announcement container",
                    r"\| 1 \| 1 \+ 4",
                    "| 13 | 1 + 4",
                )
            })
        }),
        expect: &[("AC1", "must hold the mandatory parts only"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_default_total_is_not_mandatory_plus_audio",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    &replace_in_line(t, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 199"),
                    r"^\| Spare against 1 472",
                    "= 274",
                    "= 273",
                )
            })
        }),
        expect: &[("AC1", "!= mandatory 46 + audio 1152"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_no_budget_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| replace_all(t, r"\*\*Total\*\*", "Sum"))),
        expect: &[("AC1", "no budget table"), ("AC2", ""), ("AC3", "")],
    },
    Denial {
        name: "ac1_denial_no_default_column",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_all(t, r"Default: mandatory parts only", "Worst alpha")
            })
        }),
        expect: &[
            ("AC1", "no default 1 472 budget column"),
            ("AC2", ""),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac1_denial_c100_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| replace_all(t, "C-100", "C-1OO"))),
        expect: &[("AC1", "does not cite C-100")],
    },
    Denial {
        name: "ac1_denial_a_byte_figure_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| replace_all(t, r"1 280", "1 281"))),
        expect: &[(
            "AC1",
            "does not state the 1 472, 1 280 and 1 252 byte figures",
        )],
    },
    Denial {
        name: "ac2_denial_announcement_rows_not_the_count_plus_twelve_each",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Announcement container",
                    r"1 \+ 4 × 12 = 49 \| 1 \+ 4",
                    "1 + 4 × 12 = 50 | 1 + 4",
                )
            })
        }),
        expect: &[("AC2", "announcement rows sum to 50"), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_an_extension_that_is_not_the_toml_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Measurement extension \| — \| 9",
                    r"\| 9 \|",
                    "| 8 |",
                )
            })
        }),
        expect: &[("AC2", "extension 8 != 0 or 9"), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_a_worst_column_not_at_maximum",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Announcement container",
                    r"\| 1 \+ 4 × 12 = 49 \| 1 \+ 4",
                    "| 1 + 2 × 12 = 25 | 1 + 4",
                )
            })
        }),
        expect: &[
            ("AC2", "announcements are not at maximum (25 != 49)"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac2_denial_no_worst_case_with_the_extension",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_all(
                    t,
                    r"Worst case: format maximum, all optional parts, with the extension",
                    "Case two",
                )
            })
        }),
        expect: &[
            ("AC1", ""),
            ("AC2", "with all announcements and the extension"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac2_denial_no_worst_case_without_the_extension",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_all(
                    t,
                    r"Worst case: format maximum, all announcements, no extension",
                    "Case three",
                )
            })
        }),
        expect: &[
            ("AC1", ""),
            ("AC2", "all announcements and no extension"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac3_denial_no_fallback_column",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| delete_section(t, "^Budget on a fallback path"))),
        expect: &[("AC3", "no 1 280-byte fallback budget column")],
    },
    Denial {
        name: "ac3_denial_fallback_headers_above_the_cap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            table(
                t,
                "Fallback",
                &[
                    "Packet header | 16",
                    "Slice header | 22",
                    "Announcement container | 1",
                    "Content-message header | 3",
                    "Measurement extension | 60",
                    "Audio | 1 152",
                    "Checksum | 4",
                    "Total | 1 258",
                    "Spare | -6",
                ],
            )
        }),
        expect: &[
            ("AC1", ""),
            ("AC2", ""),
            ("AC3", "above the 100-byte cap of C-137"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac3_denial_c06_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| replace_all(t, r"C-06\b", "C-O6"))),
        expect: &[("AC3", "does not cite C-06")],
    },
    Denial {
        name: "ac3_denial_c137_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| replace_all(t, "C-137", "C-13X"))),
        expect: &[("AC3", "does not cite C-137")],
    },
    Denial {
        name: "ac3_denial_no_omission_rule_and_nothing_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                t.split('\n')
                    .filter(|l| !l.to_lowercase().contains("omit"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }),
        expect: &[("AC3", "states neither an omission rule"), ("AC4", "")],
    },
    Denial {
        name: "ac3_denial_the_omission_rule_names_no_announcements",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                map_lines(&unexcused(t), r"(?i)\bomit", |l| {
                    l.replace("announcement", "notice")
                        .replace("Announcement", "Notice")
                })
            })
        }),
        expect: &[("AC3", "does not say which announcements are omitted")],
    },
    Denial {
        name: "ac3_denial_the_omission_rule_names_no_extension",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                map_lines(&unexcused(t), r"(?i)\bomit", |l| {
                    l.replace("measurement", "m.")
                        .replace("extension", "ext.")
                        .replace("Measurement", "M.")
                })
            })
        }),
        expect: &[(
            "AC3",
            "does not say what happens to the measurement extension",
        )],
    },
    Denial {
        name: "ac3_denial_the_omission_rule_names_no_controller_messages",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                map_lines(&unexcused(t), r"(?i)\bomit", |l| {
                    l.replace("controller", "sink")
                        .replace("Controller", "Sink")
                })
            })
        }),
        expect: &[("AC3", "does not say what happens to controller messages")],
    },
    Denial {
        name: "ac3_denial_the_omission_rule_states_no_priority",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                map_lines(&unexcused(t), r"(?i)\bomit", |l| {
                    rx(r"(?i)priorit\w*|in this order|\bfirst\b|\bthen\b|\bbefore\b|\bordered\b")
                        .replace_all(l, "x")
                })
            })
        }),
        expect: &[("AC3", "states no priority")],
    },
    Denial {
        name: "ac3_denial_a_fallback_that_omits_without_amending",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                rx(r"(?i)\bamend\w*|\bscop\w*|\bsupersed\w*").replace_all(&unexcused(t), "x")
            })
        }),
        expect: &[("AC3", "amends or scopes the landed sender obligations")],
    },
    Denial {
        name: "ac4_denial_no_verdict",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| rx(r"(?i)verdict").replace_all(t, "finding"))),
        expect: &[("AC4", "no verdict")],
    },
    Denial {
        name: "ac4_denial_a_falsified_verdict_when_the_headers_fit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| append(t, "Verdict: C-137 is falsified."))),
        expect: &[("AC4", "the verdict says falsified"), ("DoD", "")],
    },
    Denial {
        name: "ac4_denial_a_verdict_that_does_not_say_it_holds",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                map_lines(t, r"(?i)verdict", |l| {
                    rx(r"(?i)\bholds\b|not falsified|\bconfirmed\b").replace_all(l, "x")
                })
            })
        }),
        expect: &[("AC4", "does not say C-137 holds")],
    },
    Denial {
        name: "ac4_denial_a_holding_verdict_when_the_headers_exceed_the_cap",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_all(
                t,
                "at most 100 bytes for all headers",
                "at most 40 bytes for all headers",
            )
        }),
        expect: &[
            ("AC3", ""),
            ("AC4", "the verdict holds, but the mandatory headers"),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac4_denial_a_non_fit_verdict_must_report_falsified",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_all(
                t,
                "at most 100 bytes for all headers",
                "at most 40 bytes for all headers",
            )
        }),
        expect: &[
            ("AC3", ""),
            ("AC4", "must report C-137 falsified"),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac4_denial_a_falsified_verdict_needs_c04_at_five",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_all(
                t,
                "at most 100 bytes for all headers",
                "at most 40 bytes for all headers",
            )
        }),
        expect: &[
            ("AC3", ""),
            (
                "AC4",
                "needs the constants table to show C-04 at 5 channels",
            ),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac5_denial_the_toml_lacks_what_the_budget_needs",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            replace_all(
                t,
                "name = \"announcement_count\"",
                "name = \"announcement_cnt\"",
            )
        }),
        expect: &[("AC5", "does not declare what the budget needs")],
    },
    Denial {
        name: "ac5_denial_the_checksum_max_size_differs_from_its_field",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            after(
                t,
                "name = \"slice checksum\"",
                "max_size = 4",
                "max_size = 5",
            )
        }),
        expect: &[
            ("AC1", ""),
            ("AC5", "max_size does not equal its field width"),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac5_denial_the_audio_sample_differs_from_c03",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^\| C-03 \|",
                "4 bytes per sample",
                "3 bytes per sample",
            )
        }),
        expect: &[
            ("AC1", ""),
            ("AC3", ""),
            ("AC5", "audio sample is 4 bytes, C-03 says 3"),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac5_denial_the_announcement_count_is_not_one_byte",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            after(
                t,
                "name = \"announcement_count\"",
                "width = 8",
                "width = 16",
            )
        }),
        expect: &[
            ("AC1", ""),
            ("AC2", ""),
            ("AC5", "announcement count is not one byte"),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac5_denial_the_extension_does_not_start_where_the_container_ends",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            after(
                t,
                "name = \"announcement_entry\"",
                "offset = ",
                "offset = 8",
            )
        }),
        expect: &[
            ("AC2", ""),
            ("AC5", "does not start where the maximal container ends"),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac5_denial_the_media_packet_max_size_is_not_the_sum",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            after(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 97",
            )
        }),
        expect: &[("AC5", "'media packet' max_size is not header + slice")],
    },
    Denial {
        name: "ac5_denial_a_total_that_is_not_the_sum_of_its_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(t, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 199")
            })
        }),
        expect: &[
            ("AC1", ""),
            ("AC5", "the parts sum to 1198, its total row says 1199"),
        ],
    },
    Denial {
        name: "ac5_denial_no_budget_key_for_a_case",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| drop_line(t, r"^default_total = ")),
        expect: &[("AC5", "no wire.toml budget key for the default case")],
    },
    Denial {
        name: "ac5_denial_a_budget_key_that_is_not_the_total",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "default_total = 1198", "default_total = 1199")),
        expect: &[("AC5", "wire.toml key \"default_total\" = 1199")],
    },
    Denial {
        name: "ac5_denial_a_budget_key_that_is_not_the_part_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, "packet_header = 16", "packet_header = 17")),
        expect: &[("AC5", "the packet header is 16 (landed table)")],
    },
    Denial {
        name: "ac5_denial_an_allowance_key_with_no_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                drop_line(
                    &drop_line(t, r"^\| 1 472-byte path, a stream that could not"),
                    r"^\| Fallback path, a stream that could not",
                )
            })
        }),
        expect: &[("AC5", "has no such row"), ("AC6", "")],
    },
    Denial {
        name: "ac5_denial_an_allowance_key_that_differs_from_its_row",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            replace_all(
                t,
                "shared_allowance_could_carry_extension = 33",
                "shared_allowance_could_carry_extension = 34",
            )
        }),
        expect: &[("AC5", "but the row for could carry the extension is")],
    },
    Denial {
        name: "ac5_denial_an_allowance_key_with_no_allowance_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_all(
                    t,
                    "Allowance for controller messages",
                    "Quota for controller messages",
                )
            })
        }),
        expect: &[("AC5", "has no computed allowance table"), ("AC6", "")],
    },
    Denial {
        name: "ac6_denial_no_allowance_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_all(
                    t,
                    "Allowance for controller messages",
                    "Quota for controller messages",
                )
            })
        }),
        expect: &[("AC5", ""), ("AC6", "no controller allowance table")],
    },
    Denial {
        name: "ac6_denial_an_allowance_table_without_a_column",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(t, r"^\| Path \| Bound", r"\| Checksum \|", "| Tail |")
            })
        }),
        expect: &[("AC5", ""), ("AC6", "needs columns for the bound")],
    },
    Denial {
        name: "ac6_denial_a_row_naming_neither_path",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    r"1 472-byte path \(C-100\)",
                    "Other path (C-100)",
                )
            })
        }),
        expect: &[("AC6", "names neither the 1 472 path nor the fallback")],
    },
    Denial {
        name: "ac6_denial_a_row_with_a_missing_number",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    "100 − 63 − 4 = 33",
                    "tbd",
                )
            })
        }),
        expect: &[("AC6", "has a missing number")],
    },
    Denial {
        name: "ac6_denial_a_bound_that_is_neither_the_cap_nor_a_payload",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    r"\| 100 \| 1 152",
                    "| 99 | 1 152",
                )
            })
        }),
        expect: &[("AC6", "is neither the cap 100 nor a path payload")],
    },
    Denial {
        name: "ac6_denial_the_1472_path_with_the_fallback_payload",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    r"\| 100 \| 1 152",
                    "| 1 252 | 1 152",
                )
            })
        }),
        expect: &[("AC6", "the 1 472 path takes the cap or 1472")],
    },
    Denial {
        name: "ac6_denial_the_fallback_with_the_default_payload",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| Fallback path \(1 280",
                    r"\| 100 \| 1 152",
                    "| 1 472 | 1 152",
                )
            })
        }),
        expect: &[("AC6", "the fallback takes the cap or 1252")],
    },
    Denial {
        name: "ac6_denial_an_audio_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    r"\| 1 152 \|",
                    "| 1 151 |",
                )
            })
        }),
        expect: &[("AC6", "audio 1151 != 1152")],
    },
    Denial {
        name: "ac6_denial_a_checksum_that_differs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    r"= 63 \| 4 \|",
                    "= 63 | 5 |",
                )
            })
        }),
        expect: &[("AC6", "checksum 5 != 4 (toml)")],
    },
    Denial {
        name: "ac6_denial_counted_headers_that_are_not_a_landed_sum",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    "= 63",
                    "= 62",
                )
            })
        }),
        expect: &[("AC6", "counted headers 62 are not the mandatory parts")],
    },
    Denial {
        name: "ac6_denial_an_allowance_that_is_not_the_bound_less_the_parts",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    "100 − 63 − 4 = 33",
                    "100 − 63 − 4 = 34",
                )
            })
        }),
        expect: &[("AC6", "the allowance 33")],
    },
    Denial {
        name: "ac6_denial_a_negative_allowance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                replace_in_line(
                    t,
                    r"^\| 1 472-byte path \(C-100\), a stream",
                    "100 − 63 − 4 = 33",
                    "−5",
                )
            })
        }),
        expect: &[("AC6", "is negative")],
    },
    Denial {
        name: "ac6_denial_the_allowance_at_one_path_only",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                drop_line(
                    &drop_line(
                        &drop_line(
                            &drop_line(t, r"^\| Fallback path \(1 280"),
                            r"^\| Fallback path, a stream",
                        ),
                        r"^\| 1 472-byte path, a stream that could not",
                    ),
                    r"^\| 1 472-byte path \(C-100\)",
                )
            })
        }),
        expect: &[("AC5", ""), ("AC6", "not stated at both path sizes")],
    },
    Denial {
        name: "ac6_denial_no_upper_bound_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                rx(r"(?i)upper bound|most a source may share").replace_all(t, "x")
            })
        }),
        expect: &[("AC6", "the allowance is an upper bound")],
    },
    Denial {
        name: "ac6_denial_no_second_stream_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                rx(r"(?i)second stream").replace_all(t, "other stream")
            })
        }),
        expect: &[("AC6", "a source may always use a second stream (C-181)")],
    },
    Denial {
        name: "ac6_denial_no_audio_member_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| replace_all(t, "C-205", "C-2O5"))),
        expect: &[("AC6", "audio member counts no controller messages (C-205)")],
    },
    Denial {
        name: "ac6_denial_no_sparse_entries_sentence",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| size(t, |t| rx(r"(?i)transport").replace_all(t, "carriage"))),
        expect: &[("AC6", "no sentence gives the sparse entries")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| strip_tags(t, "^The parts and their sizes are those"))
        }),
        expect: &[("DoD", "has no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |t| {
                append(
                    t,
                    "**Open item (x).** Whether the budget holds is open `[decision]` until a later ticket.",
                )
            })
        }),
        expect: &[("DoD", "names no owner or no falsifier")],
    },
    Denial {
        name: "pointer_denial_an_open_item_still_owned_by_004_14",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| append(t, "The open item owned by 004-14 remains.")),
        expect: &[("pointer", "'open item owned by 004-14' remains")],
    },
    Denial {
        name: "pointer_denial_a_but_see_budget_pointer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| append(t, "(But see Content messages, Budget, for the count.)")),
        expect: &[("pointer", "'But see ... Budget' pointer remains")],
    },
    Denial {
        name: "pointer_denial_an_unresolved_pointer_to_004_14",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| append(t, "An open item for 004-14 remains here.")),
        expect: &[("pointer", "unresolved pointer to 004-14 (Budget)")],
    },
    Denial {
        name: "pointer_denial_a_provisional_cap_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| append(t, "The shared-packet cap row is provisional.")),
        expect: &[(
            "pointer",
            "a 'provisional' cap or shared-packet line remains",
        )],
    },
    Denial {
        name: "pointer_denial_a_landed_sentence_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            append(t, "All four announcements fit inside the 100-byte cap.")
        }),
        expect: &[(
            "pointer",
            "is neither removed nor cited to the Size budget section",
        )],
    },
    Denial {
        name: "pointer_denial_the_header_cap_open_item_not_closed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            map_lines(t, r"^\*\*Open item \(header cap\)", |l| {
                rx(r"(?i)\b(closed|resolved|answered)\b").replace_all(l, "x")
            })
        }),
        expect: &[("pointer", "is not marked closed or resolved")],
    },
    Denial {
        name: "a_string_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = \"96\"",
            )
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_float_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 96.0",
            )
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_max_size_with_an_underscore_passes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 9_6",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_hex_max_size_passes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 0x60",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_zero_padded_max_size_passes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 096",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_max_size_of_1e19",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 10000000000000000000",
            )
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_max_size_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = 100000000000000000000000000000000000000000",
            )
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_boolean_max_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "max_size = 96",
                "max_size = true",
            )
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "fields_that_is_an_int_then_a_field_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "name = \"media packet\"",
                "name = \"media packet\"\nfields = 3",
            )
        }),
        expect: &[("CRASH", "")],
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
        name: "a_message_name_that_is_a_list_crashes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"media packet\"",
                "name = \"media packet\"",
                "name = [\"media packet\"]",
            )
        }),
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
        name: "a_field_after_a_comment_line_passes",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"slice checksum\"",
                "layer = \"media\"",
                "# a comment\nlayer = \"media\"",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_zero_sparse_entry_size",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| {
            anchored_replace(
                t,
                "name = \"sparse keyed entry\"",
                "max_size = 8",
                "max_size = 0",
            )
        }),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "a_total_of_1e13",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| \*\*Total\*\* \| 1 198", "1 198", "10000000000000")
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_total_of_1e19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(
                    s,
                    r"^\| \*\*Total\*\* \| 1 198",
                    "1 198",
                    "10000000000000000000",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_total_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(
                    s,
                    r"^\| \*\*Total\*\* \| 1 198",
                    "1 198",
                    "100000000000000000000000000000000000000000",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_total_with_an_underscore",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1_198")
            })
        }),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "a_total_written_1e3",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1e3")
            })
        }),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "a_total_with_a_decimal_part",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 198.5")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_total_with_a_plus_sign",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| \*\*Total\*\* \| 1 198", "1 198", "+1 198")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_spare_with_a_minus_after_a_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| Spare against 1 472", "= 274", "= 274\u{b2}-1")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_spare_with_a_spaced_minus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| Spare against 1 472", "= 274", "= 274 -1")
            })
        }),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "a_spare_with_a_deficit_word",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            size(t, |s| {
                replace_in_line(s, r"^\| Spare against 1 472", "= 274", "= 274 deficit")
            })
        }),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "crlf_line_ends_pass",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| t.replace('\n', "\r\n")),
        expect: &[],
    },
    Denial {
        name: "a_constant_of_1e13",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-02 \|", "48 frames", "10000000000000 frames")
        }),
        expect: &[("INT_LIMIT", "")],
    },
];
