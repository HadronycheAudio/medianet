//! The denial list of 006-01: the real documents with one edit, and what the
//! port (and the old script) must report. Consumed by `tests/doc.rs` (one
//! test each) and by `doccheck-equiv 006-01 --denials-only`. An entry with no
//! expectation is one the port must pass: the script passes it too (the
//! Python facts the check depends on: NFKC, `strip`, universal newlines).

use super::{CONSTANTS, SPEC_MD};
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{replace_all, replace_in_line};

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_timeline_missing",
        file: SPEC_MD,
        edit: DenialEdit::Delete,
        expect: &[
            ("AC1", "does not exist"),
            ("AC2", ""),
            ("AC3", ""),
            ("AC4", ""),
        ],
    },
    Denial {
        name: "denial_constants_missing",
        file: CONSTANTS,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist; cannot verify values")],
    },
    Denial {
        name: "ac1_denial_the_source_lacks_an_id",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-212 \|", "C-212", "C-2l2")),
        expect: &[("AC1", "02-constants.md lacks ids required by the ticket")],
    },
    Denial {
        name: "ac1_denial_no_constants_table",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| ID \| Constant", "ID", "Key")),
        expect: &[("AC1", "no constants table found")],
    },
    Denial {
        name: "ac1_denial_no_status_column",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| ID \| Constant", "Status", "Mood")),
        expect: &[("AC1", "constants table has no status column")],
    },
    Denial {
        name: "ac1_denial_a_row_with_an_unexpected_id",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-212 \|", "$", "\n| X-1 | a | b | decided |")
        }),
        expect: &[("AC1", "unexpected row id")],
    },
    Denial {
        name: "ac1_denial_a_duplicate_row",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-212 \|", "$", "\n| C-01 | a | b | decided |")
        }),
        expect: &[("AC1", "duplicate row C-01")],
    },
    Denial {
        name: "ac1_denial_a_missing_id",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-212 \|", "C-212", "C-2l2")),
        expect: &[("AC1", "constants table is missing ids: C-212")],
    },
    Denial {
        name: "ac1_denial_an_extra_id",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-212 \|", "$", "\n| C-03 | a | b | decided |")
        }),
        expect: &[("AC1", "ids not in the ticket's 47 rows: C-03")],
    },
    Denial {
        name: "ac1_denial_a_value_that_differs",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-01 \|", "96 000 Hz", "96 001 Hz")),
        expect: &[("AC1", "value mismatch against 02-constants.md")],
    },
    Denial {
        name: "ac1_denial_a_status_that_differs",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-209 \|", r"\| decided \|$", "| inherited |")
        }),
        expect: &[("AC1", "status mismatch against 02-constants.md")],
    },
    Denial {
        name: "ac1_denial_c61_not_pending_simulation",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-61 \|", "pending-simulation", "decided")
        }),
        expect: &[("AC1", "C-61 must be marked pending simulation")],
    },
    Denial {
        name: "ac1_denial_the_table_is_not_an_introduced_section",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^## Constants used by this document", "^## ", "# ")
        }),
        expect: &[("AC1", "not an introduced section")],
    },
    Denial {
        name: "ac1_denial_another_section_comes_first",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                "^## Constants used by this document",
                "^",
                "## Extras\n\nText.\n\n",
            )
        }),
        expect: &[("AC1", "not the first section after the introduction")],
    },
    Denial {
        name: "ac1_denial_the_table_is_under_a_level_3_heading",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^## Constants used by this document", "^## ", "### ")
        }),
        expect: &[("AC1", "not under a level-2 heading")],
    },
    Denial {
        name: "ac1_denial_no_sentence_says_the_constants_table_wins",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                "^Every later section refers",
                "the constants table wins",
                "the constants table decides",
            )
        }),
        expect: &[("AC1", "no sentence states that the constants table wins")],
    },
    Denial {
        name: "ac2_denial_a_section_missing",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                "^## Epoch width and wrap",
                "Epoch width and wrap",
                "Width",
            )
        }),
        expect: &[("AC2", "missing section heading: Epoch width and wrap")],
    },
    Denial {
        name: "ac2_denial_a_placeholder_without_its_ticket",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^To be written in 006-03", "006-03", "a later ticket")
        }),
        expect: &[("AC2", "does not name the ticket id that fills it")],
    },
    Denial {
        name: "ac2_denial_a_range_that_stops_short",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^To be written in 006-13\.\.15",
                r"006-13\.\.15",
                "006-13",
            )
        }),
        expect: &[("AC2", "(missing 006-14")],
    },
    Denial {
        name: "ac2_denial_a_stemmed_heading_still_matches_but_a_synonym_does_not",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^## Discontinuity event", "event", "occurrence")
        }),
        expect: &[("AC2", "missing section heading: Discontinuity event")],
    },
    Denial {
        name: "ac3_denial_no_retired_note",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_all(t, "(?i)retired POC behaviour", "Old clock")),
        expect: &[("AC3", "no 'Retired POC behaviour' note")],
    },
    Denial {
        name: "ac3_denial_no_code_tag",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^`\[code\]` In the POC", r"`\[code\]` ", "")
        }),
        expect: &[("AC3", "does not carry [code]")],
    },
    Denial {
        name: "ac3_denial_no_citation",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^`\[code\]` In the POC", "1630-1634", "1630-1635")
        }),
        expect: &[("AC3", "does not cite module.rs:1630-1634")],
    },
    Denial {
        name: "ac3_denial_a_note_that_is_not_a_heading_still_needs_the_tag",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                &replace_in_line(
                    t,
                    "^## Retired POC behaviour",
                    "^## Retired POC behaviour",
                    "**Retired POC behaviour**",
                ),
                r"^`\[code\]` In the POC",
                r"`\[code\]` ",
                "",
            )
        }),
        expect: &[("AC3", "does not carry [code]")],
    },
    Denial {
        name: "ac4_denial_an_unclosed_fence",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| append(t, "```\ncode")),
        expect: &[("AC4", "unclosed code fence")],
    },
    Denial {
        name: "fence_closer_with_an_info_string",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| append(t, "```\ncode\n```rust")),
        expect: &[("AC4", "unclosed code fence")],
    },
    Denial {
        name: "fence_closed_by_the_other_character",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| append(t, "```\ncode\n~~~")),
        expect: &[("AC4", "unclosed code fence")],
    },
    Denial {
        name: "ac4_denial_an_indented_heading",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| append(t, "  ## Indented")),
        expect: &[("AC4", "heading is indented")],
    },
    Denial {
        name: "ac4_denial_a_row_wider_than_its_header",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-209 \|", r"\| decided \|$", "| decided | extra |")
        }),
        expect: &[("AC4", "table row has 5 columns; its table header has 4")],
    },
    Denial {
        name: "ac4_denial_a_separator_narrower_than_the_header",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^\|---\|---\|---\|---\|$",
                r"\|---\|---\|---\|---\|",
                "|---|---|---|",
            )
        }),
        expect: &[("AC4", "table row has 3 columns; its table header has 4")],
    },
    Denial {
        name: "ac2_denial_a_superscript_in_a_heading",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^## Epoch width and wrap", "wrap", "wrap\u{b2}")
        }),
        expect: &[("AC2", "missing section heading: Epoch width and wrap")],
    },
    Denial {
        name: "a_superscript_minus_in_a_heading_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, "^## Epoch width and wrap", "wrap", "wrap\u{207b}")
        }),
        expect: &[],
    },
    Denial {
        name: "greek_mu_for_the_micro_sign_is_outside_the_repertoire",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_all(t, "\u{b5}", "\u{3bc}")),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "a_leading_tab_before_a_row_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-209 \|", "^", "\t")),
        expect: &[],
    },
    Denial {
        name: "a_trailing_tab_after_a_row_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-209 \|", "$", "\t")),
        expect: &[],
    },
    Denial {
        name: "crlf_line_ends_pass",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| t.replace('\n', "\r\n")),
        expect: &[],
    },
    Denial {
        name: "a_range_of_exactly_thirty_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^To be written in 006-13\.\.15",
                r"006-13\.\.15",
                "006-13..43",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_range_of_thirty_one_fails",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^To be written in 006-13\.\.15",
                r"006-13\.\.15",
                "006-13..44",
            )
        }),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "a_range_with_an_en_dash_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^To be written in 006-13\.\.15",
                r"006-13\.\.15",
                "006-13\u{2013}15",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_range_with_to_passes",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^To be written in 006-13\.\.15",
                r"006-13\.\.15",
                "006-13 TO 006-15",
            )
        }),
        expect: &[],
    },
    Denial {
        name: "a_reversed_range_fails",
        file: SPEC_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                r"^To be written in 006-13\.\.15",
                r"006-13\.\.15",
                "006-15..13",
            )
        }),
        expect: &[("AC2", "")],
    },
];
