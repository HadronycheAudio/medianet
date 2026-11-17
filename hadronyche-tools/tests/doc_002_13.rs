//! Ticket 002-13 — close-out of epic 002 (POC archaeology): the report
//! audit and the design updates. Port of `checks/002-13.sh` (deleted with
//! this port; `doccheck-equiv 002-13` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c002_13`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion, per guard and per definition-of-done item) and on the denial
//! fixtures of its list: the real documents with one edit in memory.
//!
//! Not here, because they read the proof of concept (they are in
//! `scripts/repo-state.d/002-13.sh`, the script's own code): AC 7's
//! recomputation of the `poc-untouched` tag-tree fingerprint from the
//! proof of concept's git tag and its comparison with the report's
//! recorded value; the 27 frozen experiment-data files against
//! `checks/002-01-frozen-data.sha256` (no entry outside the four data
//! directories, none missing or changed, 27 entries, no file added or
//! missing). The port keeps the report's side of AC 7: the recorded value
//! equals the frozen tag's fingerprint (the constant the repo-state file
//! recomputes), and the re-check paragraph quotes it.
//!
//! Header and code disagree (followed: the code, PORTING.md ruling 8): the
//! header says "no check reads any git ref" and that verdict entries are
//! found "never by comparison with any git ref"; AC 7 reads the proof of
//! concept's tag with `git ls-tree`, `cat-file` and `rev-parse`.

use hadronyche_tools::doccheck::checks::c002_13::SPEC;
use hadronyche_tools::doccheck::testing::{red_phase, report_of};
use hadronyche_tools::doccheck::{Report, assert_criterion};

/// The check on the repository's working tree (computed once).
fn report() -> Report {
    report_of(&SPEC)
}

/// Panics when a failure's criterion starts with `prefix` (the colon form
/// names a free-text failure by its whole message).
fn assert_none_starting(r: &Report, prefix: &str) {
    let hits: Vec<String> = r
        .failures
        .iter()
        .filter(|f| f.criterion.starts_with(prefix))
        .map(|f| f.to_string())
        .collect();
    assert!(hits.is_empty(), "{}", hits.join("\n"));
}

// ---------------------------------------------------------------- the criteria

/// AC 1 — "The checklist shows all 7 epic criteria (002-AC-1 to 002-AC-7)
/// satisfied, each with a section reference."
///
/// Script: before the first lettered heading `(a)`..`(e)`, exactly one
/// ticked `- [x]` line per `002-AC-n` (n not followed by a digit), naming a
/// section `(x)` or a heading of the report, with an evidence tag.
#[test]
fn ac1_the_checklist_ticks_every_epic_criterion_with_its_section() {
    assert_criterion(&report(), "AC1");
}

/// AC 2 — "Five verdict lines exist, none \"pending\"."
///
/// Script: each section (a)..(e) (to the next level-2 heading) has exactly
/// one `verdict:` line, last of the section, not pending, with
/// "mechanism:" or "explanation:" and a [code] or [measured] tag; (d) in
/// one of the two literal forms, (e) "instrument bug, mechanism:".
#[test]
fn ac2_five_final_verdict_lines() {
    assert_criterion(&report(), "AC2");
}

/// AC 3 — "The four investigation sub-questions of the report are marked
/// answered."
///
/// Script: the "Open sub-questions" table has a row per question, answered
/// and naming its section; any other open row has an owner outside the
/// epic and a falsifier.
#[test]
fn ac3_the_four_sub_questions_are_answered() {
    assert_criterion(&report(), "AC3");
}

/// AC 4 — "The evidence appendix has 5 new entries, one per verdict, each
/// with an evidence tag."
///
/// Script: a row whose first cell carries the letter and which mentions
/// REPORT-0b or archaeology, tagged; the (e) entry does not say "not a
/// bug"; no row still "Unresolved" with "Task 002".
#[test]
fn ac4_one_tagged_appendix_entry_per_verdict() {
    assert_criterion(&report(), "AC4");
}

/// AC 5 — "P-02 in `02-constants.md` holds the clean 50 and 1000 Hz
/// figures from 002-12, tagged `[measured]`, as POC figures from runs
/// without strict mode (C-147); any other changed document is listed in
/// the report."
///
/// Script: the P-02 row holds the report's "P-02 replacement" pair, is
/// [measured], says strict or C-147 and unprivileged or POC, and no longer
/// reads "37 / 77"; the "Documents changed" list names the constants and
/// the appendix, every path in it exists, each bullet is tagged.
#[test]
fn ac5_p02_holds_the_clean_figures_and_the_changes_are_listed() {
    assert_criterion(&report(), "AC5");
}

/// The stale sync-rate guard (AC 5, the documents repeating a changed
/// statement): no `.md` under `doc/design` (tickets and the report aside)
/// states 37 or 77 as an achieved sync rate without a marker in the same
/// clause (prose) or cell (table).
#[test]
fn ac5_no_design_document_states_the_stale_sync_rate() {
    assert_none_starting(&report(), "stale sync-rate statement");
}

/// The loss-accounting ruling: no sentence or row says task 003 fixes the
/// loss counter without a negation, and the report's "Open item" on the
/// loss accounting names an owner and a falsifier.
#[test]
fn loss_accounting_is_not_said_fixed_by_003_and_stays_an_owned_open_item() {
    let r = report();
    assert_none_starting(&r, "loss accounting stated as fixed by task 003");
    assert_none_starting(&r, "no paragraph starting \"Open item\"");
    assert_none_starting(&r, "the open item about the POC loss accounting");
}

/// AC 6 — "The report carries the \"not applicable\" line for regime
/// stamp fields with its reason."
#[test]
fn ac6_the_regime_stamp_line_is_not_applicable_with_its_reason() {
    assert_criterion(&report(), "AC6");
}

/// AC 7 — "Frozen-tree fingerprint identical to the 002-01 value."
///
/// The report's side: the recorded `poc-untouched` tag-tree fingerprint is
/// the frozen tag's, and a paragraph states the re-check with the value.
/// The recomputation from git and the manifest are in
/// `scripts/repo-state.d/002-13.sh`.
#[test]
fn ac7_the_recorded_fingerprint_is_the_frozen_tags_and_the_recheck_is_stated() {
    assert_criterion(&report(), "AC7");
}

/// The hand-offs to 003, to 007 and 017, and to 011, each tagged.
#[test]
fn hand_offs_to_003_007_017_and_011_are_tagged() {
    assert_criterion(&report(), "Hand-offs");
}

/// The 002-10 open item (run-1 degradation): closed with the sign-test
/// counts, tagged, or re-owned outside the epic with a falsifier.
#[test]
fn the_002_10_open_item_is_closed_or_reowned() {
    let r = report();
    assert_criterion(&r, "002-10 open item");
    assert_none_starting(&r, "002-10 open item");
}

/// DoD — "No new open item without an owner and a falsifier."
#[test]
fn dod_every_open_item_paragraph_names_an_owner_and_a_falsifier() {
    assert_criterion(&report(), "DoD");
}

/// Nothing else fails on the working tree (the script printed no FAIL).
#[test]
fn the_working_tree_passes() {
    let r = report();
    assert!(r.passed(), "{}", r.render());
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script fails exactly what the
/// script failed there (the harness's comparison (b)): every criterion,
/// the stale statements of the design set one by one, the loss sentence of
/// section (b), the missing loss open item and the still-open 002-10 item.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "5791356a850d3cc0653cf9e96161fddb77eab8ef",
        &[
            "002-10 open item (run-1 degradation) is still owned by this epic / 002-13 with no sign-test result: current text: **Starvation: refuted** for the sender's ticker loop at 10× nominal on this host [measured]. Definition used for this verdict: starvation means the ticker is denied service — a block is skipped (the skip at module.rs:1694 is taken, so the fire count falls below one per 500 µs block period) or a fire is delayed past the skip threshold of 20 block periods (20 × 500 µs = 10 ms); lateness that grows without reaching that threshold is degradation, not starvation [decision]. Mechanism: the sender's data-thread drain loop at module.rs:1669 does empty every input socket before the deadline check at module.rs:1687, exactly as read from code, and the flood does reach it as 26 000 echoes per second instead of 6 000 — but the loop keeps firing: every block still fires (the ticker fired 59 961 times in each of the six runs, 2 000 times in every full second, so the skip of missed blocks at module.rs:1694 never ran), the worst lateness in any flooded run is 3 846.1 µs against the skip threshold of 20 block periods = 10 ms, and that maximum is of the order of the baseline's own (968.2 to 2 380.9 µs), a few scheduler ticks (CONFIG_HZ=250, Environment) on a host where the runs had no real-time scheduling (`warning: no real-time scheduling` in every run) [measured]. At the median the ticker is later in the baseline (3.0 to 4.0 µs) than flooded (0.0 µs): with a continuous stream of arrivals the loop is already awake and iterating when the deadline comes (the `continue` at module.rs:1688-1690, th",
            "AC1",
            "AC2",
            "AC3",
            "AC4",
            "AC5",
            "AC6",
            "AC7",
            "Hand-offs",
            "loss accounting stated as fixed by task 003 (no negation) REPORT-0b.md:281: **For task 003 (the POC loss-counter fix).** The fix must bound the accounting: carry the sending module's session through the echo (instead of the echoing module's own id, synth.rs:155) and reset the per-input span stat",
            "no paragraph starting \"Open item\" about the POC loss accounting (owner and falsifier required)",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 02-constants.md:275: 37 / 77 completed exchanges per second",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 03-acceptance-criteria.md:27: The POC behaviour (37/s at 50, 77/s at 1 000, P-02)",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 04-definition-of-done.md:24: - **No smoke assertion may depend on sync-rate timing.** The POC's timeout-paced sync delivered 37 and 77 exchanges per second where 50 and 1 000 were configured",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 05-risks-and-open.md:83: Measured 37/s and 77/s for 50 and 1 000 configured",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) 06-evidence-appendix.md:11: `--sync-hz 50` gives 37 completed exchanges/s",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) adr/10-Servo-redesign.md:14: 37 completed exchanges per second at a configured 50, and 77 at a configured 1 000 (P-02) `[measured]`, on a kernel ticking at 250 Hz (P-03).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) adr/24-Numbers-governance.md:16: - a sync rate of 37 per second at a configured 50, and 77 at a configured 1 000 (P-02).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/002-POC-archaeology.md:28: **Sync rate.** `--sync-hz 50` achieved 37 completed exchanges per second",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/003-POC-smoke-fixes.md:27: `--sync-hz 50` achieved 37 completed exchanges per second, `--sync-hz 1000` achieved 77 (P-02).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/008a-Servo-core-and-simulation-harness.md:40: measured 37 exchanges/s at a configured 50 and 77 at a configured 1 000 on a `CONFIG_HZ=250` kernel `[measured]` (P-02, P-03).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/008b-Servo-simulation-cases-and-results.md:39: 37 exchanges/s at a configured 50 and 77 at a configured 1 000 on a `CONFIG_HZ=250` kernel `[measured]` (P-02, P-03).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/013-Regime-stamp-wire-object.md:17: and an achieved sync rate of 37/s where 50 was configured.",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/015a-Hosted-runtime-core.md:37: 37 completed exchanges/s at `--sync-hz 50` and 77/s at `--sync-hz 1000` `[measured]`, on a kernel with `CONFIG_HZ=250` `[measured]` (P-02, P-03).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/015b-Hosted-runtime-verification-and-strict-mode.md:27: sync paced by a 5 ms receive timeout gave 37 exchanges/s at `--sync-hz 50` `[measured]` on `CONFIG_HZ=250` `[measured]` (P-02, P-03).",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/015b-Hosted-runtime-verification-and-strict-mode.md:66: The POC value (37/s at 50) must not reproduce.",
            "stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) implementation/017b-L0-harness-and-scenarios.md:31: - **No assertion depends on sync-rate timing.** The POC achieved 37 exchanges/s at a configured 50 and 77/s at 1 000 on a 250 Hz kernel `[measured]` (P-02, P-03)",
        ],
    );
}

// ---------------------------------------------------------------- denials

hadronyche_tools::denial_tests!(
    SPEC;
    pre_denial_report_missing,
    pre_denial_constants_missing,
    pre_denial_evidence_missing,
    ac1_denial_no_lettered_heading_uppercase_letters_still_found_by_ac2,
    ac1_denial_criterion_unticked,
    ac1_denial_criterion_ticked_twice,
    ac1_denial_a_longer_id_is_not_the_criterion,
    ac1_denial_upper_x_and_star_bullet_tick,
    ac1_denial_checklist_line_names_no_section,
    ac1_denial_a_heading_text_names_the_section,
    ac1_denial_checklist_line_untagged,
    ac1_denial_a_fenced_heading_is_no_heading,
    ac2_denial_no_verdict_line,
    ac2_denial_two_verdict_lines,
    ac2_denial_verdict_pending_any_case,
    ac2_denial_verdict_not_last,
    ac2_denial_verdict_without_mechanism,
    ac2_denial_verdict_without_code_or_measured,
    ac2_denial_d_not_a_literal_form,
    ac2_denial_e_not_an_instrument_bug,
    ac2_denial_section_heading_missing,
    ac2_denial_a_level_2_subsection_ends_the_section,
    ac2_denial_plain_verdict_without_stars,
    ac3_denial_no_section,
    ac3_denial_open_and_owned_inside_the_epic,
    ac3_denial_open_without_a_falsifier,
    ac3_denial_question_row_missing,
    ac3_denial_answered_without_a_section,
    ac3_denial_the_first_row_is_the_header,
    ac3_denial_a_two_dash_separator_is_a_row,
    ac4_denial_no_entry_for_c,
    ac4_denial_entry_untagged,
    ac4_denial_e_entry_says_not_a_bug,
    ac4_denial_m3_still_unresolved,
    ac4_denial_the_letter_must_be_in_the_first_cell,
    ac5_denial_no_replacement_line,
    ac5_denial_no_figure_pair,
    ac5_denial_constants_row_missing,
    ac5_denial_row_other_figures,
    ac5_denial_a_figure_after_a_digit_is_not_it,
    ac5_denial_a_figure_before_a_digit_is_not_it,
    ac5_denial_a_figure_before_a_dot_is_it,
    ac5_denial_row_not_measured,
    ac5_denial_row_without_strict,
    ac5_denial_row_not_poc_any_case,
    ac5_denial_row_still_37_77,
    stale_denial_unmarked_rate,
    stale_denial_marker_in_another_clause,
    stale_denial_marker_after_a_full_stop,
    stale_denial_no_split_without_a_space,
    stale_denial_marker_in_the_same_clause,
    stale_denial_table_cell_scope,
    stale_denial_context_in_another_clause,
    stale_denial_identifiers_are_not_rates,
    stale_denial_a_python_word_character_before_the_number,
    stale_denial_rate_without_context,
    stale_denial_tickets_are_not_scanned,
    stale_denial_the_report_is_not_scanned,
    stale_denial_a_marker_removed_from_history,
    ac5_denial_no_documents_changed_section,
    ac5_denial_listed_path_missing,
    ac5_denial_entry_untagged,
    ac5_denial_constants_not_listed,
    ac5_denial_a_directory_is_an_existing_path,
    loss_denial_fixed_by_003,
    loss_denial_negated,
    loss_denial_contraction_negates,
    loss_denial_negation_in_the_next_sentence,
    loss_denial_table_cell,
    loss_denial_prose_splits_at_full_stops_only,
    loss_denial_open_item_missing,
    loss_denial_open_item_without_falsifier,
    loss_denial_bold_open_item,
    ac6_denial_no_reason,
    ac6_denial_applicable,
    ac7_denial_no_recorded_fingerprint,
    ac7_denial_fingerprint_changed,
    ac7_denial_uppercase_fingerprint_matches,
    ac7_denial_fingerprint_on_the_next_line,
    ac7_denial_no_recheck_paragraph,
    ac7_denial_recheck_quoting_the_frozen_tree_fingerprint,
    handoffs_denial_no_heading,
    handoffs_denial_003_missing,
    handoffs_denial_007_017_missing,
    handoffs_denial_011_untagged,
    open_002_10_denial_subsection_missing,
    open_002_10_denial_no_run_1_paragraph,
    open_002_10_denial_still_owned_by_the_epic,
    open_002_10_denial_reowned_by_a_later_ticket,
    open_002_10_denial_closed_without_a_tag,
    open_002_10_denial_closed_with_the_counts,
    dod_denial_open_item_without_owner,
    dod_denial_only_the_first_offender,
    repertoire_denial_outside_character_in_a_scanned_file,
);
