//! The denial list of 002-03: the real report with one edit, and what the
//! port (and the old script) must report. Consumed by `tests/doc_002_03.rs`
//! and by `doccheck-equiv 002-03` against the script.
//!
//! One denial per `bad` site of the script, then the bash and mawk points
//! its code touches: the level as the count of `#` only, a heading inside
//! a code fence, a tab after the `#`s, CRLF, the two-line blocks clipped at
//! the sub-section's end, the lowercased awk cue, `×`/`÷` as one-byte
//! operators, a unit within 24 bytes, assignments to ALL_CAPS names, a
//! constant-id range read as bash integers (`030`, beyond 64 bits),
//! superscript digits against `[0-9]`, `\b` around `todo`, the open-item
//! negation window. The criteria are the script's verdict.

use super::REPORT;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{anchored_replace, rename_heading, replace_all, replace_in_line, within_section};

const T: &str = r"^transport\.rs — ";
const C: &str = r"^control\.rs — ";
const T_HEAD: &str = "### transport.rs — the transport state machine and its wire encoding";
const C_HEAD: &str = "### control.rs — the controller plane, not the control plane";
const C30: &str = "^- C-30 hello period 1 s";

fn in_t(t: &str, pattern: &str, to: &str) -> String {
    within_section(t, T, |s| replace_all(s, pattern, to))
}

fn in_c(t: &str, pattern: &str, to: &str) -> String {
    within_section(t, C, |s| replace_all(s, pattern, to))
}

/// `what` inserted as a line right before the control.rs heading (the end
/// of the transport.rs sub-section).
fn end_of_t(t: &str, what: &str) -> String {
    anchored_replace(t, C_HEAD, C_HEAD, &format!("{what}\n{C_HEAD}"))
}

/// `what` appended after the C-33 line (the end of the control.rs
/// sub-section).
fn end_of_c(t: &str, what: &str) -> String {
    let line = "- C-33 election window 400 ms";
    let at = t.find(line).expect("the C-33 line");
    let end = t[at..].find('\n').map_or(t.len(), |k| at + k);
    format!("{}\n{what}{}", &t[..end], &t[end..])
}

/// The control.rs paragraph about C-30..C-33 and its four lines replaced by
/// `what`.
fn constants_as(t: &str, what: &str) -> String {
    let start = t.find("The control-plane timers of the constants table").expect("the paragraph");
    let last = t.find("- C-33 election window 400 ms").expect("the C-33 line");
    let end = t[last..].find('\n').map_or(t.len(), |k| last + k);
    format!("{}{what}{}", &t[..start], &t[end..])
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
        name: "locate_denial_no_transport_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, T, "transport — the transport state machine")),
        expect: &[("Locate", "no sub-section heading under (d) naming transport.rs")],
    },
    Denial {
        name: "locate_denial_no_control_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, C, "control — the controller plane")),
        expect: &[("Locate", "no sub-section heading under (d) naming control.rs")],
    },
    Denial {
        name: "locate_denial_two_transport_headings",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_c(t, "\n#### More on transport.rs\n\nNothing more [code].")),
        expect: &[("Locate", "2 headings under (d) name transport.rs")],
    },
    Denial {
        name: "locate_denial_two_control_headings",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_c(t, "\n#### More on control.rs\n\nNothing more [code].")),
        expect: &[("Locate", "2 headings under (d) name control.rs")],
    },
    Denial {
        name: "locate_denial_one_heading_for_both_files",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, T_HEAD, "### transport.rs", "### transport.rs and control.rs")),
        expect: &[("Locate", "the same heading")],
    },
    Denial {
        name: "locate_denial_a_subsection_not_deeper_than_d",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, T_HEAD, "### transport.rs", "## transport.rs")),
        expect: &[("AC1", "too long"), ("AC3", ""), ("Locate", "not deeper than section (d)")],
    },
    Denial {
        name: "locate_semantics_a_fenced_heading_counts",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_c(t, "\n```\n### an example: control.rs\n```")),
        expect: &[("Locate", "2 headings under (d) name control.rs")],
    },
    Denial {
        name: "locate_semantics_a_tab_after_the_hashes",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, T_HEAD, "### transport.rs", "###\ttransport.rs")),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_transport_too_long",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, &"filler\n".repeat(25))),
        expect: &[("AC1", "transport.rs: too long")],
    },
    Denial {
        name: "ac1_denial_fewer_than_three_pointers",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_t(t, r"\.rs:", ".rs line ")),
        expect: &[("AC1", "need at least 3"), ("AC3", "has no unit near")],
    },
    Denial {
        name: "ac1_denial_fewer_than_two_own_pointers",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_c(t, r"control\.rs:", "ctl.rs:")),
        expect: &[("AC1", "need at least 2 to its own file")],
    },
    Denial {
        name: "ac1_semantics_superscript_pointers_after_an_ascii_one",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = in_c(t, r"control\.rs:", "ctl.rs:");
            end_of_c(&t, "- see control.rs:² and control.rs:³ and ctl.rs:7 [code]")
        }),
        expect: &[("AC1", "need at least 2 to its own file")],
    },
    Denial {
        name: "ac2_denial_fact_answered_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_c(t, r"\[code\]", "")),
        expect: &[("AC2", "is answered (line"), ("AC4", ""), ("DoD", "")],
    },
    Denial {
        name: "ac2_denial_fact_not_stated",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_t(t, r"(?i)is_reference", "is_leader")),
        expect: &[("AC2", "fact not stated")],
    },
    Denial {
        name: "ac2_denial_fact_marked_not_in_files_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = in_t(t, r"(?i)is_reference", "is_leader");
            end_of_c(&t, "- is_reference: not in these files.")
        }),
        expect: &[("AC2", "is marked not in these files (line")],
    },
    Denial {
        name: "ac2_semantics_fact_marked_not_in_files_with_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = in_t(t, r"(?i)is_reference", "is_leader");
            end_of_c(&t, "- is_reference: Not In These Files [code].")
        }),
        expect: &[],
    },
    Denial {
        name: "ac3_denial_a_number_without_a_unit",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "The period is 40 here.")),
        expect: &[("AC3", "has no unit near")],
    },
    Denial {
        name: "ac3_semantics_a_unit_within_24_bytes",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "The period is 40 of them, in ms.")),
        expect: &[],
    },
    Denial {
        name: "ac3_semantics_multiplication_and_division_signs_are_factors",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "The period is 40 × the tick and 60 ÷ the rate.")),
        expect: &[],
    },
    Denial {
        name: "ac3_semantics_an_upper_case_cue_alone_is_not_checked_by_awk",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "It uses STATE_LEN 40 here.")),
        expect: &[],
    },
    Denial {
        name: "ac3_semantics_an_assignment_to_a_kind_code",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "The size code is T_SIZE = 4 here.")),
        expect: &[],
    },
    Denial {
        name: "ac3_semantics_an_assignment_to_a_dimensioned_name",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "The size is STATE_LEN = 40 here.")),
        expect: &[("AC3", "number 40 has no unit near")],
    },
    Denial {
        name: "ac4_denial_differs_not_flagged",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, C30, "— matches", "— differs")),
        expect: &[("AC4", "C-30 differs but is not flagged for 002-13")],
    },
    Denial {
        name: "ac4_semantics_differs_flagged_for_002_13",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, C30, "— matches", "— differs, flagged for 002-13")),
        expect: &[],
    },
    Denial {
        name: "ac4_denial_not_in_files_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "C-30 to C-33 are not in these files.")),
        expect: &[("AC2", "listen-before-claim"), ("AC4", "is marked not in these files but carries no evidence tag")],
    },
    Denial {
        name: "ac4_denial_matches_without_a_pointer",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "C-30 to C-33 each match the table [code].")),
        expect: &[("AC2", "listen-before-claim"), ("AC4", "without a file:line pointer")],
    },
    Denial {
        name: "ac4_denial_matches_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "C-30 to C-33 each match the table (module.rs:50).")),
        expect: &[("AC2", "listen-before-claim"), ("AC4", "but carries no evidence tag"), ("DoD", "")],
    },
    Denial {
        name: "ac4_denial_not_accounted_for",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "The timers live elsewhere.")),
        expect: &[("AC2", ""), ("AC4", "is not accounted for")],
    },
    Denial {
        name: "ac4_semantics_a_range_with_a_leading_zero",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "C-030 to C-033 are not in these files [code].")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "ac4_semantics_a_range_beyond_64_bits",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "C-30 to C-99999999999999999999 are not in these files [code].")),
        expect: &[("AC2", ""), ("AC4", "is not accounted for")],
    },
    Denial {
        name: "ac4_semantics_a_range_written_with_dots",
        file: REPORT,
        edit: DenialEdit::Text(|t| constants_as(t, "C-30..C-33 are not in these files [code].")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "dod_denial_a_pointer_line_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "See transport.rs:12 for the rest.")),
        expect: &[("DoD", "carries a file:line pointer but no evidence tag")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner_and_falsifier",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "Open question: who resets the serial?")),
        expect: &[("DoD", "raises an open item without an owner and a falsifier")],
    },
    Denial {
        name: "dod_semantics_a_negated_open_item",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "There is no open question left here.")),
        expect: &[],
    },
    Denial {
        name: "dod_semantics_todo_inside_a_word",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "The todolist and the TBDs stay.")),
        expect: &[],
    },
    Denial {
        name: "dod_semantics_the_owner_on_the_next_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_t(t, "TODO: who resets the serial?\nOwner 002-13, falsifier: a reset seen.")),
        expect: &[],
    },
    Denial {
        name: "dod_semantics_the_next_line_is_clipped_at_the_end",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_c(t, "TODO: who resets the serial?")),
        expect: &[("DoD", "raises an open item")],
    },
    Denial {
        name: "section_marker_denial_no_verdict_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**verdict: not a bug, explanation: `analyse`", "**verdict:", "**Verdict")),
        expect: &[("Section marker", "lost its 'verdict:' line")],
    },
    Denial {
        name: "section_marker_denial_two_verdict_lines",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_c(t, "\nverdict: pending")),
        expect: &[("Section marker", "has 2 'verdict:' lines")],
    },
    Denial {
        name: "section_marker_denial_a_verdict_in_no_allowed_form",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**verdict: not a bug, explanation: `analyse`", "explanation:", "because:")),
        expect: &[("Section marker", "not in an allowed form")],
    },
    Denial {
        name: "section_marker_semantics_pending_is_accepted",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let start = t.find("**verdict: not a bug, explanation: `analyse`").expect("the verdict");
            let end = t[start..].find('\n').map_or(t.len(), |k| start + k);
            format!("{}**verdict: PENDING**  {}", &t[..start], &t[end..])
        }),
        expect: &[],
    },
    Denial {
        name: "semantics_crlf_line_ends",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "ALL\n", "\r\n")),
        expect: &[],
    },
];
