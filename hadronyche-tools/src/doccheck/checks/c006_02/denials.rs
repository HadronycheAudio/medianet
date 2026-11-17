//! The denial list of 006-02: the real documents with one edit, and what the
//! port (and the old script) must report. Consumed by the tests and by
//! `doccheck-equiv 006-02 --denials-only`. An entry with no expectation is
//! one the port must pass (the script passes it too).

use super::SPEC_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{replace_all, replace_in_line};
use crate::doccheck::text::rx;

/// `edit` applied to the section only: the text from its heading to the
/// next `## `, so an earlier use of the same words cannot be hit.
fn section(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    let a = t.find("## Domain line and position map").expect("heading");
    let z = t[a + 3..].find("\n## ").map_or(t.len(), |i| a + 3 + i);
    format!("{}{}{}", &t[..a], edit(&t[a..z]), &t[z..])
}

fn drop_lines(t: &str, pattern: &str) -> String {
    rx(&format!(r"(?m)^(?:{pattern}).*\n")).replace_all(t, "")
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_timeline_missing",
        file: SPEC_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "missing doc/design/spec/timeline.md")],
    },
    Denial {
        name: "denial_no_heading",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                "^## Domain line and position map$",
                "position map",
                "position maps",
            )
        }),
        expect: &[
            ("AC1", "has no '## Domain line and position map' heading"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "heading_is_case_sensitive",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                "^## Domain line and position map$",
                "Domain line",
                "Domain Line",
            )
        }),
        expect: &[
            ("AC1", "has no '## Domain line and position map' heading"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "a_fenced_heading_is_found_first",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| format!("```\n## Domain line and position map\n```\n{t}")),
        expect: &[
            ("AC1", "no 'position_at(...) = ...' formula"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "denial_the_placeholder",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                "^## Domain line and position map$",
                "$",
                "\n\nTo be written in 006-02.",
            )
        }),
        expect: &[
            ("AC1", "still the 006-02 placeholder"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac1_denial_no_position_formula",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, "position_at", "pos_at"))),
        expect: &[
            ("AC1", "no 'position_at(...) = ...' formula"),
            ("AC2", ""),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac1_denial_no_time_formula",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, "time_of", "tm_of"))),
        expect: &[
            ("AC1", "no 'time_of(...) = ...' formula"),
            ("AC2", ""),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac1_denial_no_floor",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(t, "(?i)floor\\w*|round(?:ed|ing)?\\s+down", "x")
            })
        }),
        expect: &[("AC1", "position_at does not state floor")],
    },
    Denial {
        name: "ac1_denial_no_ceil",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(t, "(?i)ceil\\w*|round(?:ed|ing)?\\s+up", "x")
            })
        }),
        expect: &[("AC1", "time_of does not state ceil")],
    },
    Denial {
        name: "ac1_denial_no_128_bit",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, "128", "256"))),
        expect: &[("AC1", "does not name a 128-bit")],
    },
    Denial {
        name: "ac2_denial_identity_with_another_variable",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(
                    t,
                    r"position_at\(time_of\(p\)\) == p",
                    "position_at(time_of(p)) == q",
                )
            })
        }),
        expect: &[("AC2", "does not state position_at(time_of(p)) == p")],
    },
    Denial {
        name: "ac2_denial_no_second_identity",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(
                    t,
                    r"time_of\(position_at\(t\)\) <= t",
                    "time_of(position_at(t)) <= u",
                )
            })
        }),
        expect: &[("AC2", "does not state time_of(position_at(t)) <= t")],
    },
    Denial {
        name: "ac2_denial_no_500000",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, r"500 000", "600 000"))),
        expect: &[
            ("AC2", "does not give 48 samples = 500 000 ns"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac2_denial_no_exactness_argument",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(
                    t,
                    r"(?i)exact\w*|without rounding|whole nanosecond|no rounding",
                    "x",
                )
            })
        }),
        expect: &[(
            "AC2",
            "no argument that 48-sample slice boundaries are exact",
        )],
    },
    Denial {
        name: "ac2_denial_exactness_without_boundaries",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, r"(?i)boundar\w*", "edge"))),
        expect: &[(
            "AC2",
            "exactness argument does not mention slice boundaries",
        )],
    },
    Denial {
        name: "ac2_denial_no_12_125000_reduction",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, r"125 000", "125 001"))),
        expect: &[("AC2", "does not show the 12 / 125 000 reduction")],
    },
    Denial {
        name: "ac2_denial_no_31250_3_reduction",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, r"31 250", "31 251"))),
        expect: &[
            ("AC2", "does not show the 31 250 / 3 reduction"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac3_denial_no_off_boundary_rule",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(
                    t,
                    r"(?i)off[\s-]*(?:a\s+)?(?:slice\s+)?boundar\w*|inside\s+(?:a\s+)?slice|within\s+(?:a\s+)?slice|not\s+on\s+a\s+slice\s+boundar\w*",
                    "x",
                )
            })
        }),
        expect: &[(
            "AC3",
            "does not state the rounding rule for positions off a slice boundary",
        )],
    },
    Denial {
        name: "ac3_denial_fewer_than_three_off_boundary_rows",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                drop_lines(
                    t,
                    r"\| time_of \| p = (?:1|2|3|4|49) \||\| position_at \| origin \+ (?:15 000|31 249) ns",
                )
            })
        }),
        expect: &[
            ("AC3", "fewer than three off-boundary example rows"),
            ("AC5", ""),
        ],
    },
    Denial {
        name: "ac4_denial_before_the_origin_without_a_decision_tag",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, r"\[decision\]", "[choice]"))),
        expect: &[("AC4", "not specified and tagged [decision]")],
    },
    Denial {
        name: "ac4_denial_before_the_origin_without_a_result",
        file: SPEC_MD,
        edit: DenialEdit::Text(|_t| {
            "## Domain line and position map\n\nA negative time is a `[decision]` panic.\n"
                .to_string()
        }),
        expect: &[
            ("AC1", ""),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", "does not say what is returned"),
            ("AC5", ""),
            ("AC6", ""),
        ],
    },
    Denial {
        name: "ac5_denial_a_decimal_value",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_in_line(
                    t,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 5 | origin + 52 083.5 ns |",
                )
            })
        }),
        expect: &[("AC5", "is not an integer")],
    },
    Denial {
        name: "ac5_denial_a_row_with_one_number",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_in_line(
                    t,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p | origin |",
                )
            })
        }),
        expect: &[("AC5", "fewer than two numeric values")],
    },
    Denial {
        name: "ac5_denial_a_row_that_fails_the_formula",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(t, r"origin \+ 10 417 ns", "origin + 10 418 ns")
            })
        }),
        expect: &[("AC5", "row fails the section's own formula")],
    },
    Denial {
        name: "ac5_denial_fewer_than_six_rows",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, r"\|---\|---\|---\|", "|-|-|-|"))),
        expect: &[
            ("AC3", ""),
            ("AC5", "0 verified row(s); at least 6 are required"),
        ],
    },
    Denial {
        name: "ac5_denial_one_direction_only",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| drop_lines(t, r"\| position_at \|"))),
        expect: &[("AC5", "does not give both directions")],
    },
    Denial {
        name: "ac5_denial_no_slice_boundary_row",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                drop_lines(
                    t,
                    r"\| time_of \| p = (?:48|96) \||\| position_at \| origin \+ 500 000 ns",
                )
            })
        }),
        expect: &[("AC5", "no slice-boundary example row")],
    },
    Denial {
        name: "ac6_denial_not_the_only_rate",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |t| {
                replace_all(t, r"(?i)\bonly\b|no other|single|sole", "x")
            })
        }),
        expect: &[("AC6", "does not state that 96 kHz is the only rate")],
    },
    Denial {
        name: "ac6_denial_a_citation_missing",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| section(t, |t| replace_all(t, "C-80", "C-8O"))),
        expect: &[("AC6", "missing citation(s): C-80")],
    },
    Denial {
        name: "a_row_with_1e19_positions_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 10000000000000000000 | origin + 104166666666666666666667 ns |",
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_row_with_1e36_positions_is_beyond_i128",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 1000000000000000000000000000000000000 | origin + 10416666666666666666666666666666666666667 ns |",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_row_with_1e40_positions_is_beyond_i128",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 10000000000000000000000000000000000000000 | origin + 104166666666666666666666666666666666666666667 ns |",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "a_position_written_1e3",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 1e3 | origin + 10 416 667 ns |",
                )
            })
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_position_written_dot_5",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = .5 | origin + 10 417 ns |",
                )
            })
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_position_with_a_plus_sign_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(s, r"^\| time_of \| p = 49 ", "p = 49", "p = +49")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_position_with_a_superscript",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(s, r"^\| time_of \| p = 49 ", "p = 49", "p = 49\u{b2}")
            })
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_position_with_an_underscore_separator",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 1 000 | origin + 10_416_667 ns |",
                )
            })
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_negative_position",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = -1 | origin + -10 417 ns |",
                )
            })
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "a_whole_decimal_position_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(s, r"^\| time_of \| p = 3 ", "p = 3", "p = 3.0")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_comma_grouped_position",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_in_line(
                    s,
                    r"^\| position_at \| origin \+ 510 417 ns",
                    "$",
                    "\n| time_of | p = 1,000 | origin + 10 416 667 ns |",
                )
            })
        }),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "another_declared_origin",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| replace_all(s, "origin_ns = 0", "origin_ns = 5"))
        }),
        expect: &[],
    },
    Denial {
        name: "a_declared_origin_beyond_i128",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            section(t, |s| {
                replace_all(
                    s,
                    "origin_ns = 0",
                    "origin_ns = 99999999999999999999999999999999999999999999",
                )
            })
        }),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "crlf_line_ends_pass",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| t.replace('\n', "\r\n")),
        expect: &[],
    },
];
