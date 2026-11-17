//! The denial list of 002-04: the real report with one edit, and what the
//! port (and the old script) must report. Consumed by `tests/doc_002_04.rs`
//! and by `doccheck-equiv 002-04` against the script.
//!
//! One denial per `bad` site of the script, then the bash points its code
//! touches: an umbrella heading, a fenced heading, the level as the count of
//! `#`, nested sub-sections, a tab after the `#`s, CRLF, the two-line block
//! clipped at the sub-section's end, the `tui.rs:107-114` range against
//! superscript digits, `[standard]` (Tag6), case folding of the required
//! words, `\b`-free word edges (`_ms`, `msx`). The criteria are the
//! script's verdict.

use super::REPORT;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{anchored_replace, rename_heading, replace_all, replace_in_line, within_section};

const S: &str = r"^sender\.rs — ";
const ST: &str = r"^stats\.rs — ";
const U: &str = r"^tui\.rs — ";
const S_HOP: &str = "^- Loss hop: the echo header field `rx_lost`";
const ST_HOP: &str = r"^- Loss hop: at stats\.rs:324";
const TX: &str = "^- TX pacing lateness: `fire` receives";
const U_HEAD: &str = "### tui.rs — the one-line status and the dashboard";

fn in_sec(t: &str, sec: &str, pattern: &str, to: &str) -> String {
    within_section(t, sec, |s| replace_all(s, pattern, to))
}

/// `what` inserted right before the tui.rs heading (the end of the
/// stats.rs sub-section).
fn end_of_stats(t: &str, what: &str) -> String {
    anchored_replace(t, U_HEAD, U_HEAD, &format!("{what}\n{U_HEAD}"))
}

/// `what` inserted at the end of the tui.rs sub-section (before the
/// section's verdict line).
fn end_of_tui(t: &str, what: &str) -> String {
    let v = "\n\n**verdict: not a bug, explanation: `analyse`";
    anchored_replace(t, v, v, &format!("{what}{v}"))
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
        name: "locate_denial_no_sender_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, S, "sender — deadlines")),
        expect: &[("Locate", "naming only sender.rs")],
    },
    Denial {
        name: "locate_denial_no_stats_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, ST, "stats — record ingestion")),
        expect: &[("Locate", "naming only stats.rs")],
    },
    Denial {
        name: "locate_denial_no_tui_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, U, "tui — the one-line status")),
        expect: &[("Locate", "naming only tui.rs")],
    },
    Denial {
        name: "locate_semantics_an_umbrella_heading_is_no_subsection",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, U, "tui.rs and stats.rs — the display")),
        expect: &[("Locate", "naming only tui.rs")],
    },
    Denial {
        name: "locate_denial_two_sender_headings",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "#### More on sender.rs\n\nNothing more [code].\n")),
        expect: &[("Locate", "2 headings under (d) name only sender.rs")],
    },
    Denial {
        name: "locate_denial_two_stats_headings",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "#### More on stats.rs\n\nNothing more [code].\n")),
        expect: &[("Locate", "2 headings under (d) name only stats.rs")],
    },
    Denial {
        name: "locate_denial_two_tui_headings",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "#### Then tui.rs\n\nNothing more [code].\n")),
        expect: &[("Locate", "2 headings under (d) name only tui.rs"), ("AC1", ""), ("AC2", "")],
    },
    Denial {
        name: "locate_semantics_a_fenced_heading_counts",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "```\n# tui.rs\n```\n")),
        expect: &[("Locate", "2 headings under (d) name only tui.rs")],
    },
    Denial {
        name: "locate_denial_a_subsection_not_deeper_than_d",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, U_HEAD, "### tui.rs", "## tui.rs")),
        expect: &[("Locate", "not deeper than section (d)")],
    },
    Denial {
        name: "locate_denial_a_subsection_inside_another",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, U_HEAD, "### tui.rs", "#### tui.rs")),
        expect: &[("Locate", "tui.rs sub-section (line")],
    },
    Denial {
        name: "locate_denial_a_subsection_inside_a_later_one",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = rename_heading(t, S, "sender — deadlines, wire encoding and the return path");
            end_of_tui(&t, "\n#### Notes on sender.rs\n\n- See sender.rs:435 [code].")
        }),
        expect: &[("Locate", "sender.rs sub-section (line"), ("AC1", ""), ("AC2", "")],
    },
    Denial {
        name: "locate_semantics_a_tab_after_the_hashes",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, U_HEAD, "### tui.rs", "###\ttui.rs")),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_too_long",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, &"filler\n".repeat(25))),
        expect: &[("AC1", "stats.rs: too long")],
    },
    Denial {
        name: "ac1_denial_fewer_than_three_pointers",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sec(t, U, r"\.rs:", ".rs line ")),
        expect: &[("AC1", "need at least 3"), ("AC2", "")],
    },
    Denial {
        name: "ac1_denial_fewer_than_two_own_pointers",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sec(t, U, r"tui\.rs:", "ui.rs:")),
        expect: &[("AC1", "need at least 2 to its own file"), ("AC2", "")],
    },
    Denial {
        name: "ac2_denial_sender_hop_not_mentioned",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sec(t, S, r"sender\.rs:435", "sender.rs:436")),
        expect: &[("AC2", "sender.rs:435 hop is not mentioned")],
    },
    Denial {
        name: "ac2_denial_sender_hop_lacks_its_width",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, S_HOP, "a u32, 4-byte count", "a count")),
        expect: &[("AC2", "sender.rs:435 hop is mentioned at line")],
    },
    Denial {
        name: "ac2_denial_stats_hop_lacks_the_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            // the hop line and the next one (its block) lose their tags
            let t = replace_in_line(t, ST_HOP, r"\[code\]\.$", ".");
            replace_in_line(&t, r"^- `snapshot` \(stats\.rs:392\)", r"\[code\]", "")
        }),
        expect: &[("AC2", "stats.rs:324 hop is mentioned at line"), ("Context fact", ""), ("DoD", "")],
    },
    Denial {
        name: "ac2_denial_tui_hop_lacks_the_branch",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sec(t, U, "(?i)sent|lost_total|received", "x")),
        expect: &[("AC2", "tui.rs:107-114 hop is mentioned at line")],
    },
    Denial {
        name: "ac2_semantics_a_superscript_in_the_tui_range",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sec(t, U, r"tui\.rs:107-114", "tui.rs:11²")),
        expect: &[("AC2", "tui.rs:107-114 hop is not mentioned")],
    },
    Denial {
        name: "ac2_semantics_the_words_fold_case",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_in_line(t, S_HOP, "a u32, 4-byte count of packets", "a U32, 4-BYTE COUNT OF PACKETS")),
        expect: &[],
    },
    Denial {
        name: "context_denial_no_streams_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| in_sec(t, ST, "(?i)streams?", "paths")),
        expect: &[("Context fact", "does not say what the loss display takes")],
    },
    Denial {
        name: "context_denial_no_pooled_marker",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            replace_in_line(
                t,
                ST_HOP,
                "it is one pooled value across all return streams with no per-stream separation, regardless of the stream that",
                "it is the value that",
            )
        }),
        expect: &[("Context fact", "(line 399) but lacks: latest-record-marker no-per-stream-marker")],
    },
    Denial {
        name: "ac3_denial_no_tx_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| replace_all(t, "(?i)tx pacing lateness|tx_late", "lateness")),
        expect: &[("AC3", "no line in the three notes mentions")],
    },
    Denial {
        name: "ac3_denial_tx_line_without_a_unit",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = replace_in_line(t, TX, r"\bin ns\b", "raw");
            let t = in_sec(&t, ST, r"(?i)tx pacing lateness|tx_late", "lateness");
            in_sec(&t, U, r"(?i)tx pacing lateness|tx_late", "lateness")
        }),
        expect: &[("AC3", "lacks: time-unit")],
    },
    Denial {
        name: "ac3_semantics_underscore_ms_is_no_unit",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = replace_in_line(t, TX, r"\bin ns\b", "in tick_ms");
            let t = in_sec(&t, ST, r"(?i)tx pacing lateness|tx_late", "lateness");
            in_sec(&t, U, r"(?i)tx pacing lateness|tx_late", "lateness")
        }),
        expect: &[("AC3", "lacks: time-unit")],
    },
    Denial {
        name: "dod_denial_a_pointer_line_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "See stats.rs:12 for the rest.\n")),
        expect: &[("DoD", "carries a file:line pointer but no evidence tag")],
    },
    Denial {
        name: "dod_semantics_standard_is_a_tag_here",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "See stats.rs:12 for the rest [standard].\n")),
        expect: &[],
    },
    Denial {
        name: "dod_denial_an_open_item_without_owner",
        file: REPORT,
        edit: DenialEdit::Text(|t| end_of_stats(t, "Unresolved: who drains the ring?\n")),
        expect: &[("DoD", "raises an open item without an owner and a falsifier")],
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
        edit: DenialEdit::Text(|t| end_of_stats(t, "verdict: pending\n")),
        expect: &[("Section marker", "has 2 'verdict:' lines")],
    },
    Denial {
        name: "section_marker_denial_a_verdict_in_no_allowed_form",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**verdict: not a bug, explanation: `analyse`", "explanation:", "because:")),
        expect: &[("Section marker", "not in an allowed form")],
    },
    Denial {
        name: "semantics_crlf_line_ends",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "ALL\n", "\r\n")),
        expect: &[],
    },
];
