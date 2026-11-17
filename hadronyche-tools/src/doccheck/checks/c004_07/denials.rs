//! The denial list of 004-07: the real documents with one edit, and what the
//! port (and the old script) must report. Consumed by the tests and by
//! `doccheck-equiv 004-07 --denials-only`. The synthetic section (`SYN`) is a
//! small wire.md whose analysis passes every assertion; the denials the real
//! section cannot isolate start from it. An entry with no expectation is one
//! the port must pass (the script passes it too).

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{replace_all, replace_in_line, within_section};
use crate::doccheck::text::rx;

const SYN: &str = "# Doc\n\n### Timeline epoch width and wrap\n\nThe epoch is 65 536 values (C-10), sources C-113, C-213, C-136, C-32, C-33, C-37 `[decision]`. Pathological ceiling: one bump every 2 s `[decision]`; 65 536 × 2 s = 131 072 s, about 1.5 days. Plausible worst-case operating rate: one bump every 10 minutes `[memory]`; 65 536 × 10 minutes = 39 321 600 s, about 1.25 years. Verdict against the 1 year falsifier: C-10 is not falsified at the plausible rate `[decision]`; the pathological ceiling is far under a year and is answered here. A receiver decides current versus stale by comparing epochs by equality; at wrap the same value recurs.\n\n## Next\n";

/// `edit` applied to the synthetic wire.md (the real text is ignored).
fn syn(_t: &str, edit: impl FnOnce(&str) -> String) -> String {
    edit(SYN)
}

/// `edit` applied to the analysis section only.
fn sec(t: &str, edit: impl FnOnce(&str) -> String) -> String {
    within_section(t, "^Timeline epoch width and wrap$", edit)
}

/// `edit` applied to every line starting with `prefix`.
fn para(t: &str, prefix: &str, edit: impl Fn(&str) -> String) -> String {
    let mut hit = false;
    let out: Vec<String> = t
        .split('\n')
        .map(|l| {
            if l.starts_with(prefix) {
                hit = true;
                edit(l)
            } else {
                l.to_string()
            }
        })
        .collect();
    assert!(hit, "no line starts with {prefix:?}");
    out.join("\n")
}

fn re_all(t: &str, pattern: &str, to: &str) -> String {
    rx(pattern).replace_all_literal(t, to)
}

/// The plausible rate made one bump a minute: the wrap is under a year.
fn fast(t: &str) -> String {
    replace_all(t, "every 10 minutes", "every 1 minute")
}

const TAGS: &str = r"\[(?:decision|memory|measured|code|standard|kernel-doc)\]";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[
            ("AC1", "missing doc/design/spec/wire.md"),
            ("AC2", ""),
            ("AC3", ""),
            ("DoD", ""),
        ],
    },
    Denial {
        name: "denial_no_analysis_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            re_all(
                t,
                r"(?im)^(#{1,6}\s+).*(?:media packet header|epoch).*$",
                "${1}Plain",
            )
        }),
        expect: &[
            (
                "AC1",
                "neither an epoch wrap/bump section nor the Media packet header section",
            ),
            ("AC2", ""),
            ("AC3", ""),
            ("DoD", ""),
        ],
    },
    Denial {
        name: "denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| format!("{t}\n\nTo be written.\n"))),
        expect: &[
            ("AC1", "still a placeholder or empty"),
            ("AC2", ""),
            ("AC3", ""),
            ("DoD", ""),
        ],
    },
    Denial {
        name: "a_fenced_heading_is_found_first",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| format!("```\n## Epoch wrap\n```\n\nShort.\n\n{t}")),
        expect: &[("AC1", "never states the 65 536"), ("DoD", "")],
    },
    Denial {
        name: "ac1_denial_no_65536",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"65[\d,   ]*536|2\^16|2\*\*16", "N"))),
        expect: &[("AC1", "never states the 65 536")],
    },
    Denial {
        name: "ac1_denial_a_source_not_cited",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "C-213", "C-2l3"))),
        expect: &[("AC1", "missing source citation(s): C-213")],
    },
    Denial {
        name: "ac1_denial_no_pathological_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, r"(?i)patholog\w*|ceiling", "bound"))),
        expect: &[("AC1", "no pathological rate stated")],
    },
    Denial {
        name: "ac1_denial_the_ceiling_without_a_matching_wrap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| syn(t, |t| replace_all(t, "131 072 s, about 1.5 days", "9 s"))),
        expect: &[("AC1", "the pathological rate")],
    },
    Denial {
        name: "ac1_denial_the_plausible_rate_without_a_matching_wrap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                replace_all(
                    &replace_all(
                        &replace_all(t, "39 321 600 s", "100 s"),
                        "455 days",
                        "5 days",
                    ),
                    "1.25 years",
                    "5 years",
                )
            })
        }),
        expect: &[("AC1", "the plausible rate"), ("AC3", "")],
    },
    Denial {
        name: "ac1_denial_a_ceiling_faster_than_the_election_timers",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| replace_all(t, "1.9 s", "1.5 s"))),
        expect: &[("AC1", "faster than the election timers allow")],
    },
    Denial {
        name: "ac1_denial_a_plausible_rate_faster_than_the_ceiling",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| syn(t, |t| replace_all(t, "every 10 minutes", "every 1 s"))),
        expect: &[("AC1", "internal inconsistency"), ("AC3", "")],
    },
    Denial {
        name: "ac1_denial_no_one_year_threshold",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                re_all(t, r"(?i)1[\s ]?year|one\s+year|a\s+year|365\s*days", "x")
            })
        }),
        expect: &[("AC1", "1-year falsifier threshold is never named")],
    },
    Denial {
        name: "ac1_denial_no_verdict",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                re_all(t, r"(?i)falsified|C-10\s+(?:holds|stands|survives)", "x")
            })
        }),
        expect: &[("AC1", "no decidable verdict")],
    },
    Denial {
        name: "ac1_denial_a_contradictory_verdict",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| format!("{t}\n\nC-10 is falsified `[decision]`.\n"))),
        expect: &[("AC1", "the verdict is contradictory"), ("AC3", "")],
    },
    Denial {
        name: "ac1_denial_falsified_although_the_wrap_exceeds_a_year",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |t| {
                replace_all(t, "C-10 is not falsified", "C-10 is falsified")
            })
        }),
        expect: &[("AC1", "yet the analysis says falsified"), ("AC3", "")],
    },
    Denial {
        name: "ac1_denial_not_falsified_although_the_wrap_is_under_a_year",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, fast)),
        expect: &[("AC1", "yet the analysis says not falsified"), ("AC3", "")],
    },
    Denial {
        name: "ac1_denial_the_verdict_never_addresses_the_ceiling",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                para(t, "**Verdict against the falsifier.**", |l| {
                    re_all(l, r"(?i)patholog\w*|ceiling", "bound")
                })
            })
        }),
        expect: &[("AC1", "never addresses the ceiling")],
    },
    Denial {
        name: "window_in_characters",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                para(
                    &para(t, "**Verdict against the falsifier.**", |l| {
                        re_all(l, r"(?i)patholog\w*|ceiling", "bound")
                    }),
                    "**Open item.**",
                    |l| format!("{} {}", "\u{2014}".repeat(420), l),
                )
            })
        }),
        expect: &[("AC1", "never addresses the ceiling")],
    },
    Denial {
        name: "ac2_denial_no_comparison_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| re_all(t, "(?i)stale", "old")),
        expect: &[("AC2", "no comparison rule for 'current' vs 'stale' epochs")],
    },
    Denial {
        name: "ac3_denial_a_wrap_under_a_year_not_declared_falsified",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, fast)),
        expect: &[("AC1", ""), ("AC3", "does not declare C-10 falsified")],
    },
    Denial {
        name: "ac3_denial_falsified_without_a_replacement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |t| {
                fast(&replace_all(
                    t,
                    "C-10 is not falsified",
                    "C-10 is falsified",
                ))
            })
        }),
        expect: &[("AC1", ""), ("AC3", "no proposed replacement width")],
    },
    Denial {
        name: "ac3_denial_falsified_without_the_constants_table_flag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                re_all(
                    &fast(&replace_all(
                        t,
                        "C-10 is not falsified at the plausible rate",
                        "C-10 is falsified at the plausible rate",
                    )),
                    r"(?i)constants table",
                    "table",
                )
            })
        }),
        expect: &[("AC1", ""), ("AC3", "not flagged for the constants table")],
    },
    Denial {
        name: "ac3_denial_falsified_without_an_open_item_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                re_all(
                    &fast(&replace_all(
                        t,
                        "C-10 is not falsified at the plausible rate",
                        "C-10 is falsified at the plausible rate",
                    )),
                    r"(?i)owner",
                    "keeper",
                )
            })
        }),
        expect: &[
            ("AC1", ""),
            ("AC3", "no open item with an owner and a falsifier"),
        ],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                replace_in_line(
                    t,
                    "^The timeline epoch is 16 bits",
                    "^",
                    "Open item: later. ",
                )
            })
        }),
        expect: &[(
            "DoD",
            "raises an open item without an owner and a falsifier",
        )],
    },
    Denial {
        name: "dod_denial_fewer_than_three_tags",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                re_all(
                    t,
                    r"\[(?:decision|memory|measured|code|standard|kernel-doc)\]",
                    "",
                )
            })
        }),
        expect: &[("DoD", "evidence tag(s); at least 3 are required")],
    },
    Denial {
        name: "dod_denial_the_ceiling_without_a_rate_tag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            sec(t, |t| {
                re_all(
                    &re_all(t, r"\[memory\]", "[code]"),
                    r"\[decision\]",
                    "[code]",
                )
            })
        }),
        expect: &[(
            "DoD",
            "the pathological rate carries no [decision]/[memory] tag",
        )],
    },
    Denial {
        name: "dod_denial_the_verdict_without_a_tag",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| sec(t, |t| re_all(t, TAGS, ""))),
        expect: &[("DoD", "the verdict carries no evidence tag")],
    },
    Denial {
        name: "a_huge_plausible_interval_passes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(
                    &replace_all(
                        s,
                        "every 10 minutes",
                        "every 10000000000000000000000000000000000000000 minutes",
                    ),
                    "39 321 600 s, about 1.25 years",
                    "2.6e45 s",
                )
            })
        }),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "a_400_digit_interval",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(
                    s,
                    "every 10 minutes",
                    &format!("every 1{} minutes", "0".repeat(400)),
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_leading_dot_interval",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(s, "every 10 minutes", "every .5 minutes")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_comma_in_an_interval_passes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(s, "every 10 minutes", "every 1,0 minutes")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_superscript_after_an_interval",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(s, "every 10 minutes", "every 10\u{b2} minutes")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_stated_wrap_exactly_on_the_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(s, "39 321 600 s, about 1.25 years", "27 525 120 s")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_stated_wrap_just_inside_the_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(s, "39 321 600 s, about 1.25 years", "27 525 121 s")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "a_stated_wrap_just_outside_the_tolerance",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            syn(t, |s| {
                replace_all(s, "39 321 600 s, about 1.25 years", "27 525 119 s")
            })
        }),
        expect: &[],
    },
    Denial {
        name: "crlf_line_ends_pass",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| t.replace('\n', "\r\n")),
        expect: &[],
    },
    Denial {
        name: "crlf_line_ends_in_the_synthetic_section_pass",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| syn(t, |s| s.replace('\n', "\r\n"))),
        expect: &[],
    },
];
