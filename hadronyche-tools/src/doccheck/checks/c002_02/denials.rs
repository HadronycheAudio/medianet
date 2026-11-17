//! The denial list of 002-02: the real report with one edit, and what the
//! port (and the old script) must report. Consumed by `tests/doc_002_02.rs`
//! (one test each) and by `doccheck-equiv 002-02` against the script.
//!
//! One denial per `bad` site of the script (the unreachable "sub-section is
//! empty" excepted: the sub-section text always holds its heading line),
//! then the bash-semantics points the script's code touches: case folding,
//! a heading inside a code fence, a tab after the `#`s, CRLF, trailing
//! spaces and emphasis, the line after a fact, the last line without a
//! newline, superscript digits against `[0-9]`, `\b` next to `²` and `_`,
//! `µ` as a letter, regex metacharacters in the text. The criteria are the
//! script's verdict on the same documents.

use super::REPORT;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{anchored_replace, rename_heading, replace_all, replace_in_line, within_section};

const SUB: &str = r"^audio\.rs — analyse";
const ANSWER: &str = "Voice comparison bit-level: yes";
const ROW: &str = "| Is the voice comparison bit-level? |";
const FACT: [&str; 5] = [
    "^- Comparison is per 1024-sample window",
    "^- A window is compared only when it is complete",
    "^- A window still lagging more than MAX_LAG_WINDOWS",
    "^- Settling: when the returned voice frames",
    "^- The sender's replica never applies gaps",
];

fn in_sub(t: &str, pattern: &str, to: &str) -> String {
    within_section(t, SUB, |s| replace_all(s, pattern, to))
}

/// The marker (and its `[code]`) removed from the lines matching the given
/// patterns.
fn unmark(t: &str, lines: &[&str]) -> String {
    lines.iter().fold(t.to_string(), |t, l| replace_in_line(&t, l, r"confirmed \[code\]", "noted"))
}

/// The answer turned to "no", with every tolerance word of the sub-section
/// removed (`toleran`, `eps\b`).
fn answer_no_untolerant(t: &str) -> String {
    let t = answer_as(t, "Voice comparison bit-level: no");
    let t = in_sub(&t, r"(?i)toleran", "allowan");
    in_sub(&t, r"(?i)eps\b", "eqs")
}

/// The sub-section's answer line replaced by `to` (the report's close-out
/// checklist quotes the same words earlier).
fn answer_as(t: &str, to: &str) -> String {
    within_section(t, SUB, |s| anchored_replace(s, ANSWER, ANSWER, to))
}

fn insert_after_answer(t: &str, what: &str) -> String {
    let answer = answer_line(t);
    answer_as(t, &format!("{answer}\n{what}"))
}

/// The answer line as it stands (after an earlier edit).
fn answer_line(t: &str) -> String {
    t.lines()
        .rev()
        .find(|l| l.starts_with("Voice comparison bit-level:"))
        .unwrap_or(ANSWER)
        .to_string()
}

/// The voices-flag entry numbered `²)` instead of bulleted, its reason
/// removed: it is an entry (and lacks a reason) only when the regex matcher
/// decides (`[0-9]` takes `²`) AND the superset DFA passes the line (some
/// other alternative, a `-`, is on it).
fn superscript_entry_without_reason(t: &str) -> String {
    let t = replace_in_line(t, "^- voices flag off:", "^- ", "²) ");
    replace_in_line(&t, r"^²\) voices flag off:", "because ", "")
}

/// The verdict statement line with `extra` added before its final period.
fn verdict_plus(t: &str, extra: &str) -> String {
    replace_in_line(t, "^Verdict statement:", r"\[code\]\.$", &format!("{extra} [code]."))
}

/// The table row moved to the end of the file (`newline`: with a final
/// newline).
fn row_last(t: &str, newline: bool) -> String {
    let i = t.find(ROW).expect("the row");
    let end = t[i..].find('\n').map_or(t.len(), |k| i + k + 1);
    let row = t[i..end].trim_end_matches('\n').to_string();
    let rest = format!("{}{}", &t[..i], &t[end..]);
    format!("{rest}{row}{}", if newline { "\n" } else { "" })
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "helpers_denial_report_missing",
        file: REPORT,
        edit: DenialEdit::Delete,
        expect: &[("Helpers", "report missing")],
    },
    Denial {
        name: "locate_denial_no_section_d",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, r"^\(d\) Reading notes$", "Reading notes")),
        expect: &[("Locate", "section (d) Reading notes not found")],
    },
    Denial {
        name: "locate_denial_no_section_e",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, r"^\(e\) Run confirmations$", "Run confirmations")),
        expect: &[("Locate", "section (e) heading not found after section (d)")],
    },
    Denial {
        name: "locate_semantics_an_e_heading_in_a_fence_before_d_counts",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## (a) Inert check families", "## (a)", "```\n## (e) fenced\n```\n\n## (a)")),
        expect: &[("Locate", "section (e) heading not found after section (d)")],
    },
    Denial {
        name: "locate_denial_no_audio_subsection",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, SUB, "audio.rs — reading notes")),
        expect: &[("Locate", "no sub-section heading under (d)")],
    },
    Denial {
        name: "locate_semantics_headings_fold_case",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = rename_heading(t, r"^\(d\) Reading notes$", "(D) READING NOTES");
            rename_heading(&t, SUB, "AUDIO.RS — ANALYSIS of analyse_voices")
        }),
        expect: &[],
    },
    Denial {
        name: "locate_semantics_a_fenced_heading_is_the_subsection",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## (d) Reading notes", "## (d) Reading notes\n", "## (d) Reading notes\n\n```\n### audio.rs analyse (an example, fenced)\n```\n")),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", "")],
    },
    Denial {
        name: "locate_semantics_a_tab_after_the_hashes_lowers_the_level",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### audio.rs — analyse", "### audio.rs", "###\taudio.rs")),
        expect: &[("AC1", "too long"), ("AC4", "")],
    },
    Denial {
        name: "locate_semantics_trailing_spaces_on_the_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### audio.rs — analyse / analyse_voices", "analyse_voices", "analyse_voices   ")),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_too_long",
        file: REPORT,
        edit: DenialEdit::Text(|t| insert_after_answer(t, &"filler\n".repeat(34))),
        expect: &[("AC1", "sub-section is too long")],
    },
    Denial {
        name: "ac1_denial_fewer_than_four_pointers",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, r"audio\.rs:", "audio.rs line ")),
        expect: &[("AC1", "distinct audio.rs:NNN pointer(s), need at least 4"), ("AC4", ""), ("AC5", "")],
    },
    Denial {
        name: "ac1_semantics_superscript_pointers_are_not_counted",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = in_sub(t, r"audio\.rs:", "audio.rs line ");
            insert_after_answer(&t, "See audio.rs:² audio.rs:³ audio.rs:¹ audio.rs:⁴ audio.rs:⁵.")
        }),
        expect: &[("AC1", "need at least 4"), ("AC4", ""), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_answer_no_without_tolerance",
        file: REPORT,
        edit: DenialEdit::Text(answer_no_untolerant),
        expect: &[("AC2", "names no tolerance")],
    },
    Denial {
        name: "ac2_semantics_eps_before_a_superscript_is_a_word_end",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = answer_no_untolerant(t);
            within_section(&t, SUB, |s| anchored_replace(s, "Voice comparison bit-level: no", "no", "no\nWithin eps² of the reference."))
        }),
        expect: &[],
    },
    Denial {
        name: "ac2_semantics_eps_before_an_underscore_is_not",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = answer_no_untolerant(t);
            within_section(&t, SUB, |s| anchored_replace(s, "Voice comparison bit-level: no", "no", "no\nWithin eps_ref of the reference."))
        }),
        expect: &[("AC2", "names no tolerance")],
    },
    Denial {
        name: "ac2_denial_answer_not_exact",
        file: REPORT,
        edit: DenialEdit::Text(|t| answer_as(t, "Voice comparison bit-level: yes, exactly")),
        expect: &[("AC2", "missing an exact line")],
    },
    Denial {
        name: "ac2_semantics_trailing_spaces_after_the_bold_hide_its_end",
        file: REPORT,
        edit: DenialEdit::Text(|t| answer_as(t, "-   **VOICE comparison bit-level: Yes**  \t")),
        expect: &[("AC2", "missing an exact line")],
    },
    Denial {
        name: "ac2_semantics_bullet_bold_and_case_are_tolerated",
        file: REPORT,
        edit: DenialEdit::Text(|t| answer_as(t, "- **VOICE comparison bit-level: Yes**")),
        expect: &[],
    },
    Denial {
        name: "ac2_semantics_a_carriage_return_after_the_bold_answer",
        file: REPORT,
        edit: DenialEdit::Text(|t| answer_as(t, "**Voice comparison bit-level: yes**\r")),
        expect: &[("AC2", "missing an exact line")],
    },
    Denial {
        name: "ac3_denial_fact_1_not_stated",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "1024", "4096")),
        expect: &[("AC3", "'1024-sample comparison window' not stated")],
    },
    Denial {
        name: "ac3_denial_fact_1_not_marked",
        file: REPORT,
        edit: DenialEdit::Text(|t| unmark(t, &FACT[0..2])),
        expect: &[("AC3", "'1024-sample comparison window' is stated")],
    },
    Denial {
        name: "ac3_denial_fact_2_not_stated",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "on both sides", "on each side")),
        expect: &[("AC3", "'window compared only when complete on both sides' not stated")],
    },
    Denial {
        name: "ac3_denial_fact_2_not_marked",
        file: REPORT,
        edit: DenialEdit::Text(|t| unmark(t, &FACT[1..3])),
        expect: &[("AC3", "'window compared only when complete on both sides' is stated")],
    },
    Denial {
        name: "ac3_denial_fact_3_not_stated",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "about 2.7 s", "about three seconds")),
        expect: &[("AC3", "'256-window lag drop (MAX_LAG_WINDOWS, about 2.7 s)' not stated")],
    },
    Denial {
        name: "ac3_denial_fact_3_not_marked",
        file: REPORT,
        edit: DenialEdit::Text(|t| unmark(t, &FACT[2..4])),
        expect: &[("AC3", "'256-window lag drop (MAX_LAG_WINDOWS, about 2.7 s)' is stated")],
    },
    Denial {
        name: "ac3_denial_fact_4_not_stated",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "SETTLE_SECS = 3 s", "SETTLE_SECS = three seconds")),
        expect: &[],
    },
    Denial {
        name: "ac3_denial_fact_4_not_marked",
        file: REPORT,
        edit: DenialEdit::Text(|t| unmark(t, &FACT[3..5])),
        expect: &[("AC3", "'1-second silence gives a 3-second settling exclusion' is stated")],
    },
    Denial {
        name: "ac3_denial_fact_5_not_stated",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "replica", "copy")),
        expect: &[("AC3", "'sender's replica never applies gaps' not stated")],
    },
    Denial {
        name: "ac3_denial_fact_5_not_marked",
        file: REPORT,
        edit: DenialEdit::Text(|t| unmark(t, &FACT[4..5])),
        expect: &[("AC3", "'sender's replica never applies gaps' is stated")],
    },
    Denial {
        name: "ac3_semantics_the_next_line_carries_the_marker",
        file: REPORT,
        edit: DenialEdit::Text(|t| unmark(t, &FACT[0..1])),
        expect: &[],
    },
    Denial {
        name: "ac3_semantics_mu_glued_to_the_marker_is_a_letter",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = replace_in_line(t, FACT[4], "— confirmed", "—µconfirmed");
            replace_in_line(&t, FACT[4], "audio.rs:576-587", "audio.rs:576-587 (confirmed later)")
        }),
        expect: &[],
    },
    Denial {
        name: "ac3_semantics_mu_glued_to_the_marker_hides_it",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, FACT[4], "— confirmed", "—µconfirmed")),
        expect: &[("AC3", "'sender's replica never applies gaps' is stated")],
    },
    Denial {
        name: "ac3_semantics_a_dot_is_not_in_the_digit_group_bracket",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "1024", "1.024")),
        expect: &[("AC3", "'1024-sample comparison window' not stated")],
    },
    Denial {
        name: "ac4_denial_entry_without_reason",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, "^- voices flag off:", "because ", "")),
        expect: &[("AC4", "exclusion entry lacks reason")],
    },
    Denial {
        name: "ac4_denial_entry_without_pointer",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, "^- voices flag off:", r"\(audio\.rs:511.*\)", "(see the code)")),
        expect: &[("AC4", "exclusion entry lacks pointer")],
    },
    Denial {
        name: "ac4_denial_a_category_without_a_complete_entry",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, "sender gap", "sender hole")),
        expect: &[("AC4", "exclusion categories missing a complete entry (pointer + reason):; sender gap")],
    },
    Denial {
        name: "ac4_denial_no_exclusion_entry",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sub(t, r"(?m)^- ", "")),
        expect: &[("AC4", "no exclusion list entry found")],
    },
    Denial {
        name: "ac4_semantics_a_superscript_numbered_entry_is_no_entry",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = superscript_entry_without_reason(t);
            // no `-`, and no digits before a `)` or `.`: nothing else for the
            // superset DFA to find on the line
            replace_in_line(&t, r"^²\) voices flag off:", r"audio\.rs:664-667, audio\.rs:715-723\)\.", "audio.rs:664, audio.rs:715 ) .")
        }),
        expect: &[("AC4", "no exclusion list entry found")],
    },
    Denial {
        name: "ac4_semantics_a_superscript_numbered_entry_with_a_hyphen_is_one",
        file: REPORT,
        edit: DenialEdit::Text(superscript_entry_without_reason),
        expect: &[("AC4", "exclusion entry lacks reason: ²) voices flag off")],
    },
    Denial {
        name: "ac5_denial_no_verdict_statement",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, "^Verdict statement:", "^Verdict statement:", "Verdict:")),
        expect: &[("AC5", "missing the required verdict line")],
    },
    Denial {
        name: "ac5_denial_two_sentences",
        file: REPORT,
        edit: DenialEdit::Text(|t| verdict_plus(t, ". It is exact")),
        expect: &[("AC5", "is not one sentence")],
    },
    Denial {
        name: "ac5_semantics_decimals_pointers_and_eg_are_not_terminals",
        file: REPORT,
        edit: DenialEdit::Text(|t| verdict_plus(t, " (e.g. 2.7 and audio.rs:12)")),
        expect: &[],
    },
    Denial {
        name: "ac5_semantics_a_superscript_decimal_alone_keeps_its_period",
        file: REPORT,
        edit: DenialEdit::Text(|t| verdict_plus(t, " (⁹.5)")),
        expect: &[("AC5", "is not one sentence")],
    },
    Denial {
        name: "ac5_semantics_a_superscript_decimal_after_an_ascii_one_is_removed",
        file: REPORT,
        edit: DenialEdit::Text(|t| verdict_plus(t, " (1.5 and ².5)")),
        expect: &[],
    },
    Denial {
        name: "dod_denial_no_open_subquestions_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Open sub-questions$", "Sub-questions")),
        expect: &[("DoD", "no '## Open sub-questions' section found")],
    },
    Denial {
        name: "dod_denial_no_row",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, ROW, "voice comparison bit-level", "voice comparison exact")),
        expect: &[("DoD", "has no row about the voice comparison")],
    },
    Denial {
        name: "dod_denial_not_a_table_row",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, ROW, "| Is the", "Is the")),
        expect: &[("DoD", "no longer a table row")],
    },
    Denial {
        name: "dod_denial_answered_without_yes_or_no",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, ROW, "answered: **yes**", "answered: **affirmative**")),
        expect: &[("DoD", "does not state yes or no")],
    },
    Denial {
        name: "dod_denial_not_answered",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, ROW, "answered:", "closed:")),
        expect: &[("DoD", "not marked answered")],
    },
    Denial {
        name: "dod_semantics_the_row_as_an_unterminated_last_line_is_lost",
        file: REPORT,
        edit: DenialEdit::Text(|t| row_last(t, false)),
        expect: &[("DoD", "has no row about the voice comparison")],
    },
    Denial {
        name: "dod_semantics_the_row_as_a_terminated_last_line_is_found",
        file: REPORT,
        edit: DenialEdit::Text(|t| row_last(t, true)),
        expect: &[],
    },
    Denial {
        name: "semantics_crlf_line_ends",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "ALL\n", "\r\n")),
        expect: &[],
    },
];
