//! The denial list of 002-13: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_002_13.rs` (one test each) and by `doccheck-equiv 002-13
//! --denials`. At least one per assertion of the script that the port
//! keeps (AC 7's proof-of-concept reads are in
//! `scripts/repo-state.d/002-13.sh`), plus one per semantic point of the
//! script's code: `re.I` heading search against case-sensitive matches,
//! fence masking, the `(?!\d)` and look-behind rewrites, Python's `\w`,
//! the clause and cell scoping of the guards, the section extents, `[X]`,
//! `**verdict:`, the header row of the sub-question table, the
//! three-dash separator, directories as existing paths, the next-line
//! fingerprint, the first-offender `break` of the DoD rule.
//!
//! Criteria are the colon form's ([`crate::doccheck::fail_form`]): a
//! labelled message gives its label (`AC1`, `DoD`, `Hand-offs`), a free one
//! is its own criterion, spelled out whole below.

use super::{CONSTANTS, EVIDENCE, REPORT};
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{anchored_replace, delete_row, rename_heading};

const RISKS: &str = "doc/design/05-risks-and-open.md";
const ADR24: &str = "doc/design/adr/24-Numbers-governance.md";
const TICKET: &str = "doc/design/tickets/002/002-02-Reading-notes-audio-analyse.md";

/// `t` with `f` applied to its first line containing `needle`.
fn on_line(t: &str, needle: &str, f: fn(&str) -> String) -> String {
    let lines: Vec<&str> = t.split('\n').collect();
    let i = lines
        .iter()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("on_line: no line contains {needle:?}"));
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    out[i] = f(lines[i]);
    out.join("\n")
}

/// `t` with a blank line and `para` appended (`para` ends the file).
fn append(t: &str, para: &str) -> String {
    format!("{}\n\n{para}\n", t.trim_end_matches('\n'))
}

/// The 002-10 run-1 paragraph of the report replaced by `para`.
fn run1(t: &str, para: &str) -> String {
    let start = t
        .find("**Starvation: refuted**")
        .expect("the run-1 paragraph");
    let end = start + t[start..].find('\n').expect("its end");
    format!("{}{para}{}", &t[..start], &t[end..])
}

/// The denials.
pub const DENIALS: &[Denial] = &[
    // ------------------------------------------------ files the script opens
    Denial {
        name: "pre_denial_report_missing",
        file: REPORT,
        edit: DenialEdit::Delete,
        expect: &[("report missing", "REPORT-0b.md")],
    },
    Denial {
        name: "pre_denial_constants_missing",
        file: CONSTANTS,
        edit: DenialEdit::Delete,
        expect: &[("missing file", "02-constants.md")],
    },
    Denial {
        name: "pre_denial_evidence_missing",
        file: EVIDENCE,
        edit: DenialEdit::Delete,
        expect: &[("missing file", "06-evidence-appendix.md")],
    },
    // ------------------------------------------------ AC 1
    Denial {
        name: "ac1_denial_no_lettered_heading_uppercase_letters_still_found_by_ac2",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = anchored_replace(t, "\n## (a) ", "## (a) ", "## (A) ");
            let t = anchored_replace(&t, "\n## (b) ", "## (b) ", "## (B) ");
            let t = anchored_replace(&t, "\n## (c) ", "## (c) ", "## (C) ");
            let t = anchored_replace(&t, "\n## (d) ", "## (d) ", "## (D) ");
            anchored_replace(&t, "\n## (e) ", "## (e) ", "## (E) ")
        }),
        expect: &[("AC1", "no lettered section heading")],
    },
    Denial {
        name: "ac1_denial_criterion_unticked",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "- [x] 002-AC-3", "- [ ] 002-AC-3")),
        expect: &[("AC1", "002-AC-3 needs exactly one ticked \"[x]\" checklist line before section (a); found 0 (an unticked line exists)")],
    },
    Denial {
        name: "ac1_denial_criterion_ticked_twice",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "", "- [x] 002-AC-4", "- [x] 002-AC-4 again, section (c) [measured]\n- [x] 002-AC-4")
        }),
        expect: &[("AC1", "002-AC-4 needs exactly one ticked \"[x]\" checklist line before section (a); found 2")],
    },
    Denial {
        name: "ac1_denial_a_longer_id_is_not_the_criterion",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "- [x] 002-AC-5 ", "- [x] 002-AC-55 ")),
        expect: &[("AC1", "002-AC-5 needs exactly one ticked \"[x]\" checklist line before section (a); found 0")],
    },
    Denial {
        name: "ac1_denial_upper_x_and_star_bullet_tick",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = anchored_replace(t, "", "- [x] 002-AC-6", "- [X] 002-AC-6");
            anchored_replace(&t, "", "- [x] 002-AC-7", "* [x] 002-AC-7")
        }),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_checklist_line_names_no_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- [x] 002-AC-5", |_| {
                "- [x] 002-AC-5 — the voice comparison is bit-level `[code]`".into()
            })
        }),
        expect: &[("AC1", "checklist line for 002-AC-5 names no section")],
    },
    Denial {
        name: "ac1_denial_a_heading_text_names_the_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- [x] 002-AC-5", |_| {
                "- [x] 002-AC-5 — see AUDIO.RS — ANALYSE / ANALYSE_VOICES: bit-level `[code]`".into()
            })
        }),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_checklist_line_untagged",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "- [x] 002-AC-5", "`[code]`", "")),
        expect: &[("AC1", "checklist line for 002-AC-5 carries no evidence tag")],
    },
    Denial {
        name: "ac1_denial_a_fenced_heading_is_no_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "## (a) Inert check families", |l| format!("```\n{l}\n```"))
        }),
        expect: &[("AC2", "section (a) heading not found")],
    },
    // ------------------------------------------------ AC 2
    Denial {
        name: "ac2_denial_no_verdict_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "**verdict: I/O-pressure", "**Verdict: I/O-pressure")),
        expect: &[("AC2", "section (c) must carry exactly one \"verdict:\" line; found 0")],
    },
    Denial {
        name: "ac2_denial_two_verdict_lines",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "", "**verdict: I/O-pressure", "verdict: pending\n\n**verdict: I/O-pressure")
        }),
        expect: &[("AC2", "section (c) must carry exactly one \"verdict:\" line; found 2")],
    },
    Denial {
        name: "ac2_denial_verdict_pending_any_case",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "**verdict: not a bug, explanation: `analyse`", |_| "**verdict: Pending**".into())
        }),
        expect: &[("AC2", "section (d) verdict is still pending")],
    },
    Denial {
        name: "ac2_denial_verdict_not_last",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "**verdict: instrument bug, mechanism: sync requests", |l| {
                format!("{l}\n\nA note after the verdict.")
            })
        }),
        expect: &[("AC2", "section (e) verdict is not the last non-blank line")],
    },
    Denial {
        name: "ac2_denial_verdict_without_mechanism",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "**verdict: I/O-pressure", |l| l.replacen("mechanism:", "mechanism -", 1))
        }),
        expect: &[("AC2", "section (c) verdict has neither \"mechanism:\" nor \"explanation:\"")],
    },
    Denial {
        name: "ac2_denial_verdict_without_code_or_measured",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "**verdict: not a bug, explanation: the music", |l| {
                l.replace("[code]", "[decision]").replace("[measured]", "[decision]")
            })
        }),
        expect: &[("AC2", "section (a) verdict carries no [code] or [measured] tag")],
    },
    Denial {
        name: "ac2_denial_d_not_a_literal_form",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "", "**verdict: not a bug, explanation: `analyse`", "**verdict: no bug, explanation: `analyse`")
        }),
        expect: &[("AC2", "section (d) verdict must use one literal form")],
    },
    Denial {
        name: "ac2_denial_e_not_an_instrument_bug",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "", "**verdict: instrument bug, mechanism: sync requests", "**verdict: not a bug, explanation: sync requests")
        }),
        expect: &[("AC2", "found: not a bug, explanation: sync requests")],
    },
    Denial {
        name: "ac2_denial_section_heading_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, r"^\(b\) The 21\.7 M counter", "The 21.7 M counter")),
        expect: &[("AC2", "section (b) heading not found")],
    },
    Denial {
        name: "ac2_denial_a_level_2_subsection_ends_the_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "### 002-14 — ", "## 002-14 — ")),
        expect: &[("AC2", "section (e) must carry exactly one \"verdict:\" line; found 0")],
    },
    Denial {
        name: "ac2_denial_plain_verdict_without_stars",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "**verdict: not a bug, explanation: `analyse`", |l| {
                l.trim_start_matches("**").trim_end_matches("**").to_string()
            })
        }),
        expect: &[],
    },
    // ------------------------------------------------ AC 3
    Denial {
        name: "ac3_denial_no_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Open sub-questions$", "Sub-questions")),
        expect: &[("AC3", "\"Open sub-questions\" section not found")],
    },
    Denial {
        name: "ac3_denial_open_and_owned_inside_the_epic",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "| Is the 21.7 M value", "answered: **neither**", "open: **neither**")
        }),
        expect: &[("AC3", "owned inside this epic: Is the 21.7 M value a decoding error or a display error?")],
    },
    Denial {
        name: "ac3_denial_open_without_a_falsifier",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "| Why does a 5 ms timeout", "| Why", "| Is the clock thread fair? | 011-02 |  |\n| Why")
        }),
        expect: &[("AC3", "names no owner or no falsifier: Is the clock thread fair?")],
    },
    Denial {
        name: "ac3_denial_question_row_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Why does a 5 ms timeout")),
        expect: &[("AC3", "\"the 5 ms timeout\" has no row")],
    },
    Denial {
        name: "ac3_denial_answered_without_a_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Is the voice comparison", " (section (d))", "")),
        expect: &[("AC3", "\"voice comparison\" is answered but names no section")],
    },
    Denial {
        name: "ac3_denial_the_first_row_is_the_header",
        file: REPORT,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Question \| Owner \| Falsifier \|")),
        expect: &[("AC3", "\"voice comparison\" has no row")],
    },
    Denial {
        name: "ac3_denial_a_two_dash_separator_is_a_row",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Question | Owner | Falsifier |", "|---|---|---|", "|--|--|--|")),
        expect: &[],
    },
    // ------------------------------------------------ AC 4
    Denial {
        name: "ac4_denial_no_entry_for_c",
        file: EVIDENCE,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| REPORT-0b \(c\) \|")),
        expect: &[("AC4", "no verdict entry for (c)")],
    },
    Denial {
        name: "ac4_denial_entry_untagged",
        file: EVIDENCE,
        edit: DenialEdit::Text(|t| on_line(t, "| REPORT-0b (d) |", |l| l.replace("[code]", "code"))),
        expect: &[("AC4", "the verdict entry for (d) carries no evidence tag")],
    },
    Denial {
        name: "ac4_denial_e_entry_says_not_a_bug",
        file: EVIDENCE,
        edit: DenialEdit::Text(|t| on_line(t, "| REPORT-0b (e) |", |l| format!("{l} not a bug |"))),
        expect: &[("AC4", "says \"not a bug\"")],
    },
    Denial {
        name: "ac4_denial_m3_still_unresolved",
        file: EVIDENCE,
        edit: DenialEdit::Text(|t| {
            on_line(t, "| M-3 |", |l| {
                l.replace("**Resolved by task 002**", "Unresolved; Task 002")
            })
        }),
        expect: &[("AC4", "still says \"Unresolved\" with \"Task 002\"")],
    },
    Denial {
        name: "ac4_denial_the_letter_must_be_in_the_first_cell",
        file: EVIDENCE,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "| REPORT-0b (b) |", "| REPORT-0b b |")),
        expect: &[("AC4", "no verdict entry for (b)")],
    },
    // ------------------------------------------------ AC 5, P-02
    Denial {
        name: "ac5_denial_no_replacement_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| t.replace("P-02 replacement", "P-02 change")),
        expect: &[("AC5", "no \"P-02 replacement\" line")],
    },
    Denial {
        name: "ac5_denial_no_figure_pair",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "**P-02 replacement**", |_| {
                "**P-02 replacement**: the figures of the table above [measured]".into()
            })
        }),
        expect: &[("AC5", "no \"x / y\" figure pair")],
    },
    Denial {
        name: "ac5_denial_constants_row_missing",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "| P-02 |", "| P-2 |")),
        expect: &[("AC5", "row P-02 not found")],
    },
    Denial {
        name: "ac5_denial_row_other_figures",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| P-02 |", "36.1 / 71.4", "36.2 / 71.4")),
        expect: &[("AC5", "does not hold the clean 50 / 1000 Hz figures 36.1 / 71.4")],
    },
    Denial {
        name: "ac5_denial_a_figure_after_a_digit_is_not_it",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| P-02 |", "36.1 / 71.4", "136.1 / 71.4")),
        expect: &[("AC5", "does not hold the clean")],
    },
    Denial {
        name: "ac5_denial_a_figure_before_a_digit_is_not_it",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| P-02 |", "36.1 / 71.4", "36.1 / 71.42")),
        expect: &[("AC5", "does not hold the clean")],
    },
    Denial {
        name: "ac5_denial_a_figure_before_a_dot_is_it",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| P-02 |", "36.1 / 71.4", "36.1. / 71.4.")),
        expect: &[],
    },
    Denial {
        name: "ac5_denial_row_not_measured",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| on_line(t, "| P-02 |", |l| l.replace("`[measured]`", "`[decision]`"))),
        expect: &[("AC5", "P-02 row is not tagged [measured]")],
    },
    Denial {
        name: "ac5_denial_row_without_strict",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            on_line(t, "| P-02 |", |l| l.replace("without strict mode (C-147)", "plain"))
        }),
        expect: &[("AC5", "without strict mode (C-147)")],
    },
    Denial {
        name: "ac5_denial_row_not_poc_any_case",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            on_line(t, "| P-02 |", |l| {
                l.replace("POC", "Old")
                    .replace("poc-untouched", "tagged")
                    .replace("unprivileged", "plain")
            })
        }),
        expect: &[("AC5", "POC / unprivileged figures")],
    },
    Denial {
        name: "ac5_denial_row_still_37_77",
        file: CONSTANTS,
        edit: DenialEdit::Text(|t| {
            on_line(t, "| P-02 |", |l| l.replacen(" (at `10`", " (37 / 77 corrupt; at `10`", 1))
        }),
        expect: &[("AC5", "P-02 row still reads \"37 / 77\"")],
    },
    // ------------------------------------------------ stale sync-rate guard
    Denial {
        name: "stale_denial_unmarked_rate",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The POC sync achieved 37 exchanges per second.")),
        expect: &[("stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 05-risks-and-open.md:151: The POC sync achieved 37 exchanges per second.", "")],
    },
    Denial {
        name: "stale_denial_marker_in_another_clause",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The sync rate was 37/s; superseded later.")),
        expect: &[("stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 05-risks-and-open.md:151: The sync rate was 37/s", "")],
    },
    Denial {
        name: "stale_denial_marker_after_a_full_stop",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "Sync gave 77 at 1000. Corrected since.")),
        expect: &[("stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 05-risks-and-open.md:151: Sync gave 77 at 1000.", "")],
    },
    Denial {
        name: "stale_denial_no_split_without_a_space",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "Sync gave 77 at 1000.Corrected since.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_marker_in_the_same_clause",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The POC sync achieved 37 exchanges per second, a figure later superseded.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_table_cell_scope",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "| x | sync at 37 / 77 | superseded |")),
        expect: &[("stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 05-risks-and-open.md:151: sync at 37 / 77", "")],
    },
    Denial {
        name: "stale_denial_context_in_another_clause",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The sync rate: 37/s.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_identifiers_are_not_rates",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The sync rules C-37 exchanges, AC-77/s and 1.37 completed, x/37 per second.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_a_python_word_character_before_the_number",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The sync count x²37 exchanges and _77/s.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_rate_without_context",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "We saw 37 completed.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_tickets_are_not_scanned",
        file: TICKET,
        edit: DenialEdit::Text(|t| append(t, "The POC sync achieved 37 exchanges per second.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_the_report_is_not_scanned",
        file: REPORT,
        edit: DenialEdit::Text(|t| append(t, "The POC sync achieved 37 exchanges per second.")),
        expect: &[],
    },
    Denial {
        name: "stale_denial_a_marker_removed_from_history",
        file: ADR24,
        edit: DenialEdit::Text(|t| {
            on_line(t, "37", |l| {
                l.replace("corrupt", "x")
                    .replace("superseded", "x")
                    .replace("Superseded", "x")
                    .replace("36.1", "x")
                    .replace("71.4", "x")
                    .replace("earlier", "x")
                    .replace("originally", "x")
                    .replace("corrected", "x")
            })
        }),
        expect: &[("stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) adr/24-Numbers-governance.md:16: - a sync rate of 37 per second at a configured 50, and 77 at a configured 1 000, read from that x file (x by the clean figures x and x, P-02, REPORT-0b section (e)).", "")],
    },
    // ------------------------------------------------ AC 5, Documents changed
    Denial {
        name: "ac5_denial_no_documents_changed_section",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Documents changed$", "Documents touched")),
        expect: &[("AC5", "no \"Documents changed\" section")],
    },
    Denial {
        name: "ac5_denial_listed_path_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            anchored_replace(t, "### Documents changed", "`doc/design/04-definition-of-done.md`", "`doc/design/04-definition-of-done-x.md`")
        }),
        expect: &[("AC5", "lists a path that does not exist: doc/design/04-definition-of-done-x.md")],
    },
    Denial {
        name: "ac5_denial_entry_untagged",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- `doc/design/04-definition-of-done.md`", |l| l.replace(" [measured]", ""))
        }),
        expect: &[("AC5", "entry carries no evidence tag: - `doc/design/04-definition-of-done.md`")],
    },
    Denial {
        name: "ac5_denial_constants_not_listed",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- `doc/design/02-constants.md` — P-02", |_| "- the constants table [measured]".into())
        }),
        expect: &[("AC5", "does not list doc/design/02-constants.md")],
    },
    Denial {
        name: "ac5_denial_a_directory_is_an_existing_path",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- `doc/design/04-definition-of-done.md`", |l| format!("{l} (see `doc/design/adr`)"))
        }),
        expect: &[],
    },
    // ------------------------------------------------ loss accounting
    Denial {
        name: "loss_denial_fixed_by_003",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The loss counter is fixed by task 003.")),
        expect: &[("loss accounting stated as fixed by task 003 (no negation) 05-risks-and-open.md:151: The loss counter is fixed by task 003.", "")],
    },
    Denial {
        name: "loss_denial_negated",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The loss counter is not fixed by task 003.")),
        expect: &[],
    },
    Denial {
        name: "loss_denial_contraction_negates",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The loss counter isn't fixed by 003.")),
        expect: &[],
    },
    Denial {
        name: "loss_denial_negation_in_the_next_sentence",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "The loss accounting is resolved by 003. Nothing else is not.")),
        expect: &[("loss accounting stated as fixed by task 003 (no negation) 05-risks-and-open.md:151: The loss accounting is resolved by 003.", "")],
    },
    Denial {
        name: "loss_denial_table_cell",
        file: EVIDENCE,
        edit: DenialEdit::Text(|t| append(t, "| X-1 | rx_lost | 003 fixes it |")),
        expect: &[("loss accounting stated as fixed by task 003 (no negation) 06-evidence-appendix.md:147: 003 fixes it", "")],
    },
    Denial {
        name: "loss_denial_prose_splits_at_full_stops_only",
        file: RISKS,
        edit: DenialEdit::Text(|t| append(t, "Loss display: 003 fixes it; nothing more.")),
        expect: &[("loss accounting stated as fixed by task 003 (no negation) 05-risks-and-open.md:151: Loss display: 003 fixes it; nothing more.", "")],
    },
    Denial {
        name: "loss_denial_open_item_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "Open item — the POC's loss accounting", "Note — the POC's loss accounting")),
        expect: &[("no paragraph starting \"Open item\" about the POC loss accounting (owner and falsifier required)", "")],
    },
    Denial {
        name: "loss_denial_open_item_without_falsifier",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "Open item — the POC's loss accounting", |l| l.replace("Falsifier:", "Disproof:"))
        }),
        expect: &[
            ("the open item about the POC loss accounting names no owner or no falsifier", ""),
            ("DoD", "Open item — the POC's loss accounting"),
        ],
    },
    Denial {
        name: "loss_denial_bold_open_item",
        file: REPORT,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "Open item — the POC's loss accounting", "**Open item** — the POC's loss accounting")),
        expect: &[],
    },
    // ------------------------------------------------ AC 6
    Denial {
        name: "ac6_denial_no_reason",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "Regime stamp fields:", |l| l.replace("because", "as").replace("no runtime code", "nothing"))
        }),
        expect: &[("AC6", "")],
    },
    Denial {
        name: "ac6_denial_applicable",
        file: REPORT,
        edit: DenialEdit::Text(|t| on_line(t, "Regime stamp fields:", |l| l.replace("is not applicable", "is applicable"))),
        expect: &[("AC6", "")],
    },
    // ------------------------------------------------ AC 7, the report's side
    Denial {
        name: "ac7_denial_no_recorded_fingerprint",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- poc-untouched tag-tree fingerprint", |l| {
                l.replace("ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675", "(see below)")
            })
        }),
        expect: &[("AC7", "records no 64-hex poc-untouched tag-tree fingerprint")],
    },
    Denial {
        name: "ac7_denial_fingerprint_changed",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- poc-untouched tag-tree fingerprint", |l| l.replace("26ea7675`", "26ea7676`"))
        }),
        expect: &[("AC7", "frozen-tree fingerprint changed (recorded=ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7676 computed=ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675)")],
    },
    Denial {
        name: "ac7_denial_uppercase_fingerprint_matches",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "- poc-untouched tag-tree fingerprint", |l| {
                l.replace(
                    "ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675",
                    &"ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675".to_uppercase(),
                )
            })
        }),
        expect: &[],
    },
    Denial {
        name: "ac7_denial_fingerprint_on_the_next_line",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = on_line(t, "- poc-untouched tag-tree fingerprint", |l| {
                l.replace("`ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675`", "below")
            });
            anchored_replace(&t, "taken 2026-11-14: below", "  - The original", "  - `ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675`. The original")
        }),
        expect: &[],
    },
    Denial {
        name: "ac7_denial_no_recheck_paragraph",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "Frozen-tree re-check (002-13", |l| {
                l.replace("`ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675`", "the recorded value")
            })
        }),
        expect: &[("AC7", "no paragraph states the re-check")],
    },
    Denial {
        name: "ac7_denial_recheck_quoting_the_frozen_tree_fingerprint",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            on_line(t, "Frozen-tree re-check (002-13", |l| {
                l.replace("ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675", "1c39ae238eeafbc75e3910b5b1a1796718b28fc96bc88c5b4243877da2daa873")
            })
        }),
        expect: &[],
    },
    // ------------------------------------------------ hand-offs
    Denial {
        name: "handoffs_denial_no_heading",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Hand-offs$", "Next steps")),
        expect: &[("Hand-offs", "no heading containing \"Hand-off\"")],
    },
    Denial {
        name: "handoffs_denial_003_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| on_line(t, "- **To 003 (POC smoke fixes).**", |l| l.replace("confirmed", "listed"))),
        expect: &[("Hand-offs", "missing hand-off to 003")],
    },
    Denial {
        name: "handoffs_denial_007_017_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| on_line(t, "- **To 007 and 017", |l| l.replace("017", "018"))),
        expect: &[("Hand-offs", "missing hand-off to 007 and 017")],
    },
    Denial {
        name: "handoffs_denial_011_untagged",
        file: REPORT,
        edit: DenialEdit::Text(|t| on_line(t, "- **To 011 (the I/O-pressure", |l| l.replace(" [measured]", ""))),
        expect: &[("Hand-offs", "the hand-off to 011 (I/O-pressure rule inherited or dropped) carries no evidence tag")],
    },
    // ------------------------------------------------ the 002-10 open item
    Denial {
        name: "open_002_10_denial_subsection_missing",
        file: REPORT,
        edit: DenialEdit::Text(|t| rename_heading(t, "^002-10 — flood test", "Flood test of the unbounded drain")),
        expect: &[("002-10 open item", "the 002-10 subsection was not found")],
    },
    Denial {
        name: "open_002_10_denial_no_run_1_paragraph",
        file: REPORT,
        edit: DenialEdit::Text(|t| run1(t, "**Starvation: refuted** at 10x nominal [measured].")),
        expect: &[("002-10 open item", "no run-1 degradation paragraph")],
    },
    Denial {
        name: "open_002_10_denial_still_owned_by_the_epic",
        file: REPORT,
        edit: DenialEdit::Text(|t| run1(t, "Run 1 degradation is undecided; owner 002-13, falsifier: a sign test.")),
        expect: &[("002-10 open item (run-1 degradation) is still owned by this epic / 002-13 with no sign-test result: current text: Run 1 degradation is undecided; owner 002-13, falsifier: a sign test.", "")],
    },
    Denial {
        name: "open_002_10_denial_reowned_by_a_later_ticket",
        file: REPORT,
        edit: DenialEdit::Text(|t| run1(t, "Run 1 degradation is undecided; owner 011-04, falsifier: a flooded run whose p99 doubles.")),
        expect: &[],
    },
    Denial {
        name: "open_002_10_denial_closed_without_a_tag",
        file: REPORT,
        edit: DenialEdit::Text(|t| run1(t, "Run 1: sign-test F = 0, B = 3 decided host noise.")),
        expect: &[("002-10 open item (run-1 degradation) is still owned by this epic / 002-13 with no sign-test result: current text: Run 1: sign-test F = 0, B = 3 decided host noise.", "")],
    },
    Denial {
        name: "open_002_10_denial_closed_with_the_counts",
        file: REPORT,
        edit: DenialEdit::Text(|t| run1(t, "Run 1: sign-test F = 0, B = 3 decided host noise [measured].")),
        expect: &[],
    },
    // ------------------------------------------------ DoD
    Denial {
        name: "dod_denial_open_item_without_owner",
        file: REPORT,
        edit: DenialEdit::Text(|t| append(t, "An open item remains here.")),
        expect: &[("DoD", "names no owner or no falsifier: An open item remains here.")],
    },
    Denial {
        name: "dod_denial_only_the_first_offender",
        file: REPORT,
        edit: DenialEdit::Text(|t| {
            let t = anchored_replace(t, "", "## Environment", "First open item.\n\n## Environment");
            append(&t, "Second open item.")
        }),
        expect: &[("DoD", "First open item.")],
    },
    // ------------------------------------------------ the repertoire guard
    Denial {
        name: "repertoire_denial_outside_character_in_a_scanned_file",
        file: ADR24,
        edit: DenialEdit::Text(|t| append(t, "Café.")),
        expect: &[("REPERTOIRE", "U+00E9")],
    },
];
