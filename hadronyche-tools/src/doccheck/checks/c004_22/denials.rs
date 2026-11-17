//! The denial list of 004-22: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_22.rs` (one test each) and by `doccheck-equiv 004-22
//! --denials`. The last two are the review's port-weaker findings 1a and 1b
//! (PR #42, round 1).

use super::{ARCH_MD, WIRE_MD, WIRE_TOML};
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{anchored_replace, delete_row, rename_heading, replace_all, replace_in_line, strip_tags};

const TICKET: &str = "doc/design/tickets/004/004-22-Traceability-table-and-review.md";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "ac1_denial_a_missing_rule_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Rule 7 \|")),
        expect: &[("AC1", "no row for rule(s) [7]")],
    },
    Denial {
        name: "ac1_denial_a_rule_with_two_rows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| Rule 8 \|", r"Rule 8", "Rule 7")),
        expect: &[("AC1", "rule(s) [7] have more than one row")],
    },
    Denial {
        name: "ac1_denial_an_empty_mapping",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| Rule 6 \|", r"^.*$", "| Rule 6 | - | | | |")),
        expect: &[("AC1", "rule 6: the mapping cell is empty")],
    },
    Denial {
        name: "ac1_denial_a_mapping_that_resolves_to_no_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| Rule 6 \|",
            r"^.*$",
            "| Rule 6 | checksum | none | (No such heading) | implemented |",
        )
    }),
        expect: &[("AC1", "rule 6: no reference in its mapping cell resolves")],
    },
    Denial {
        name: "ac1_denial_a_dangling_anchor",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| Rule 1 \|", r"\|\s*$", " [x](#no-such-heading) |")),
        expect: &[("AC1", "names an anchor that is not a heading of doc/design/spec/wire.md")],
    },
    Denial {
        name: "ac1_denial_a_link_to_a_missing_file",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| Rule 1 \|", r"\|\s*$", " [x](../no-such-file.md) |")),
        expect: &[("AC1", "names a file that does not exist")],
    },
    Denial {
        name: "ac1_denial_a_field_that_is_not_in_wire_toml",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| Rule 1 \|", r"\|\s*$", " `no_such_field` |")),
        expect: &[("AC1", "cites `no_such_field`, which is not a field of wire.toml")],
    },
    Denial {
        name: "ac1_denial_no_traceability_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Traceability table$", "Mapping table")),
        expect: &[("AC1", "has no Traceability section"), ("AC2", ""), ("AC4", "")],
    },
    Denial {
        name: "ac2_denial_a_constant_named_only_outside_its_constant_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| C-105 \|")),
        expect: &[("AC2", "constant C-105 of the ticket's list is not named in the CONSTANT cell")],
    },
    Denial {
        name: "ac2_denial_a_constant_row_whose_mapping_resolves_nowhere",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| C-140 \|",
            r"\(Manifest, E and the cost bound\)",
            "(Nowhere)",
        )
    }),
        expect: &[("AC2", "constant C-140: no row naming it in the constants table has a mapping cell that resolves")],
    },
    Denial {
        name: "ac2_denial_a_width_that_differs_from_wire_toml",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-198 \|", r"32 bits", "16 bits")),
        expect: &[("AC2", "`current_a` is stated at 16 bits but wire.toml gives"), ("AC2", "wire-visible constant C-198 has no row stating its field")],
    },
    Denial {
        name: "ac2_denial_a_width_without_a_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-140 \|", r"32 bits, ns;", "32 bits;")),
        expect: &[("AC2", "no unit in the row for declared_e")],
    },
    Denial {
        name: "ac2_denial_a_field_named_without_its_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| C-198 \|",
            r"\| implemented \|",
            "| implemented, see `declared_e` |",
        )
    }),
        expect: &[("AC2", "`declared_e` is named in the row without its width")],
    },
    Denial {
        name: "ac2_denial_not_wire_visible_without_a_reason",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| C-17 \|",
            r"^.*$",
            "| C-17 | bound | Field-table format | not wire-visible |",
        )
    }),
        expect: &[("AC2", "a row says C-17 is not wire-visible without a reason of at least three words")],
    },
    Denial {
        name: "ac2_denial_the_ticket_has_no_constants_list",
        file: TICKET,
        edit: DenialEdit::Text(|t| t.replace("Constants traced:", "Constants:")),
        expect: &[("AC2", "the ticket has no 'Constants traced:' sentence")],
    },
    Denial {
        name: "ac3_denial_a_media_field_missing_from_the_field_column",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| `magic` \| media packet \|")),
        expect: &[("AC3", "names media field(s): magic")],
    },
    Denial {
        name: "ac3_denial_a_per_receiver_verdict_stops_the_ticket",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| `magic` \| media packet \|",
            r"\| none \|",
            "| per-receiver |",
        )
    }),
        expect: &[("AC3", "STOP: the audit finds a destination-specific or per-receiver media field (magic)"), ("AC3", "per-receiver field magic is not recorded as a discrepancy")],
    },
    Denial {
        name: "ac3_denial_a_row_without_a_verdict",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| `magic` \| media packet \|",
            r"\| none \|",
            "| maybe |",
        )
    }),
        expect: &[("AC3", "the audit row for magic states no verdict")],
    },
    Denial {
        name: "ac3_denial_no_conclusion",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^Conclusion: the media layer",
            r"no destination-specific and no per-receiver field",
            "only shared fields",
        )
    }),
        expect: &[("AC3", "the audit's conclusion does not state")],
    },
    Denial {
        name: "ac3_denial_no_audit_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Per-receiver field audit$", "Per-receiver fields")),
        expect: &[("AC3", "has no section headed 'audit'"), ("AC1", ""), ("AC4", "")],
    },
    Denial {
        name: "ac4_denial_no_review_record",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Review record$", "Record")),
        expect: &[("AC4", "no review record")],
    },
    Denial {
        name: "ac4_denial_no_reviewer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, "(?i)reviewer", "checker")),
        expect: &[("AC4", "names no reviewer")],
    },
    Denial {
        name: "ac4_denial_no_date",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\*\*Date:\*\*", "2026-11-16", "soon")),
        expect: &[("AC4", "states no date")],
    },
    Denial {
        name: "ac4_denial_the_architecture_heading_is_not_named",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, "(?i)protocol and packetisation", "wire layout section")),
        expect: &[("AC4", "names no architecture heading on protocol and packetisation")],
    },
    Denial {
        name: "ac4_denial_a_discrepancy_without_a_resolution",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| R-1 \|",
            r"^.*$",
            "| R-1 | 3.1 Slices and packets | a | b | c | | | |",
        )
    }),
        expect: &[("AC4", "a discrepancy has no resolution")],
    },
    Denial {
        name: "ac4_denial_a_resolution_changing_another_document_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\| R-1 \|",
            r"^.*$",
            "| R-1 | 3.1 Slices and packets | a | b | c | edit 01-architecture.md | | |",
        )
    }),
        expect: &[("AC4", "changes a design document other than wire.md without naming 004-23 or an owner")],
    },
    Denial {
        name: "ac4_denial_the_table_is_not_attached",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_all(t, r"\]\(#traceability-table\)", "](#review-record)")),
        expect: &[("AC4", "the traceability table is not attached")],
    },
    Denial {
        name: "ac4_denial_a_dangling_link_in_the_record",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\*\*Date:\*\*", r"$", " [x](#no-such-heading)")),
        expect: &[("AC4", "review record: link [x](#no-such-heading) names an anchor")],
    },
    Denial {
        name: "ac4_denial_work_3_a_subsection_neither_named_nor_cleared",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(&delete_row(t, r"^\| R-9 \| 3\.4"), r"^\| 3\.4 \|")),
        expect: &[("AC4", "Work 3: the architecture subsection 3.4")],
    },
    Denial {
        name: "ac4_denial_the_copy_index_contradiction_is_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| R-14 \| 3\.7")),
        expect: &[("AC4", "naming § 3.7 and 'retransmi'")],
    },
    Denial {
        name: "ac4_denial_the_verified_marking_is_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        let t = delete_row(t, r"^\| R-10 \| 3\.5");
        let t = delete_row(&t, r"^\| R-13 \| 3\.5");
        delete_row(&t, r"^\| 3\.5 \| three classes")
    }),
        expect: &[("AC4", "naming § 3.5 and 'verified'")],
    },
    Denial {
        name: "ac4_denial_the_residence_contradiction_is_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| R-12 \| 3\.2")),
        expect: &[("AC4", "no discrepancy row naming residence_time as a contradiction")],
    },
    Denial {
        name: "ac4_denial_the_record_claims_every_known_difference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\*\*Date:\*\*",
            r"$",
            " It holds every known difference.",
        )
    }),
        expect: &[("AC4", "the review record claims to hold every known difference")],
    },
    Denial {
        name: "ac4_denial_the_record_says_the_ticket_allows_its_placement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\*\*Date:\*\*",
            r"$",
            " Ticket 004-22 allows the record to be kept here.",
        )
    }),
        expect: &[("AC4", "says ticket 004-22 allows or permits the record's placement")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| strip_tags(t, r"^\*\*Purpose:\*\*")),
        expect: &[("DoD", "an untagged paragraph of 5+ words in the review record")],
    },
    Denial {
        name: "dod_denial_a_tag_in_the_wrong_case_does_not_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\*\*Purpose:\*\*", r"\[decision\]", "[Decision]")),
        expect: &[("DoD", "an untagged paragraph of 5+ words in the review record")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_a_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
        replace_in_line(
            t,
            r"^\*\*Date:\*\*",
            r"$",
            "\n\nOpen item: the owner is 004-23 and nothing else is said `[decision]`.",
        )
    }),
        expect: &[("DoD", "an open item in the review record names no owner or no falsifier")],
    },
    Denial {
        name: "dod_denial_a_main_ref",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\*\*Date:\*\*", r"$", " Compared with `main`.")),
        expect: &[("DoD", "the review record text refers to a main ref")],
    },
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("WIRE", "cannot read wire.md")],
    },
    Denial {
        name: "review_1a_a_link_climbing_above_the_repository",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| {
            replace_in_line(t, r"^\| C-07 \|", r"\|\s*$", " [w](../../../../doc/design/spec/wire.md) |")
        }),
        expect: &[("AC1", "names a file that does not exist")],
    },
    Denial {
        name: "review_1b_a_superscript_in_the_traceability_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^## Traceability table$", "table", "table²")),
        expect: &[
            ("AC4", "names an anchor that is not a heading"),
            ("AC4", "the traceability table is not attached"),
        ],
    },
    // ---- the review's 100 denials (PR #42 rounds 1 and 2), as posted:
    // expected = the old script's verdict, or REPERTOIRE outside the repertoire
    Denial {
        name: "review_r001_ac1_delete_the_rule_7_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 7 |", "| Rule 7 |", "| Rule 77 |")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r002_ac1_duplicate_rule_20_rule_21_renumbered",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 21 |", "| Rule 21 |", "| Rule 20 |")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r003_ac1_break_one_anchor_copy_index_c_07_c_7",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Traceability table", "(#copy-index-c-07)", "(#copy-index-c-7)")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r004_ac1_link_escaping_the_repo_root_doc_design_spec_wire_md",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-07 |", "| implemented: senders", "| [w](../../../../doc/design/spec/wire.md) implemented: senders")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r005_ac1_cite_a_non_field_snake_case_name",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 20 |", "`holder_module_id`", "`holder_module_idx`")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r006_ac2_change_one_width_frames_16_8_bits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-02 |", "`frames` (16 bits, frames;", "`frames` (8 bits, frames;")),
        expect: &[],
    },
    Denial {
        name: "review_r007_ac2_drop_one_unit_structural_latency",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "`structural_latency` (32 bits, samples)", "`structural_latency` (32 bits)")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r008_ac2_drop_one_constant_row_id_c_122_c_922",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-122 |", "| C-122 |", "| C-922 |")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r009_ac3_one_verdict_per_receiver_magic",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| `magic` | media packet | none |", "| `magic` | media packet | none |", "| `magic` | media packet | per-receiver |")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "review_r010_ac3_one_verdict_emptied_version",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| `version` | media packet | none |", "| `version` | media packet | none |", "| `version` | media packet |  |")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "review_r011_ac3_conclusion_negated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "Conclusion: the media layer", "has no destination-specific and no per-receiver field", "has a destination-specific and no per-receiver field")),
        expect: &[("AC3", "")],
    },
    Denial {
        name: "review_r012_ac3_one_field_dropped_from_the_audit_bar_origin",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| `bar_origin` |", "| `bar_origin` |", "| `bar_originx` |")),
        expect: &[("AC1", ""), ("AC3", "")],
    },
    Denial {
        name: "review_r013_ac4_date_removed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Date:**", "**Date:** 2026-11-16", "**Date:** mid-November")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r014_ac4_reviewer_tbd",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Reviewer:**", "**Reviewer:** Claude Opus 5.5, coder of ticket 004-22 in the implementation swarm; an independent Opus reviewer reviews the pull request that carries this record `[decision]`.", "**Reviewer:** TBD `[decision]`.")),
        expect: &[],
    },
    Denial {
        name: "review_r015_ac4_resolution_column_renamed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Discrepancies against", "| Kind | Resolution | Owner |", "| Kind | Outcome | Owner |")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r016_ac4_retransmi_wording_dropped_from_r_14",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| R-14 |", "retransmitting a slice", "resending a slice")),
        expect: &[],
    },
    Denial {
        name: "review_r017_ac4_over_claim_every_known_difference",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Method", "Each statement of 01-architecture.md", "This record holds every known difference of the epic. Each statement of 01-architecture.md")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r018_ac4_placement_claim_004_22_allows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Why here.**", "**Why here.**", "**Why here.** Ticket 004-22 allows the record to be kept here.")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r019_dod_strip_one_tag_purpose",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "as epic 004's definition of done requires `[decision]`.", "as epic 004's definition of done requires.")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r020_dod_tag_case_decision",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "as epic 004's definition of done requires `[decision]`.", "as epic 004's definition of done requires `[Decision]`.")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r021_dod_main_ref",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "the review record of ticket 004-22:", "the review record of ticket 004-22 (see origin/main):")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r022_dod_open_item_without_falsifier",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Method", "### Method\n", "### Method\n\nOpen item 9: the owner is 004-23 `[decision]`.\n")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r023_dod_paragraph_split_by_a_fence_3_3_words",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Method", "### Method\n", "### Method\n\nthree short words\n```\ncode\n```\nthree more words\n")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "review_r024_slug_superscript_in_the_traceability_heading",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Traceability table", "## Traceability table\n", "## Traceability table²\n")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r025_slug_duplicate_slug_interplay_heading_traceability_table_1_b",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Rules 1 to 22", "### Rules 1 to 22\n", "### Rules 1 to 22\n\n#### Traceability table\n")),
        expect: &[],
    },
    Denial {
        name: "review_r045_crlf_line_ends_in_wire_md",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "ALL\n", "\r\n")),
        expect: &[],
    },
    Denial {
        name: "review_r049_r2_b_before_a_combining_mark_main_branch_u_0301",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "the review record of ticket 004-22:", "the review record of ticket 004-22 (not the main branch\u{301}):")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r050_r2_b_before_superscript_main_branch_python_no_boundary",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "the review record of ticket 004-22:", "the review record of ticket 004-22 (main branch²):")),
        expect: &[],
    },
    Denial {
        name: "review_r051_r2_s_with_x1f_separators_untagged_5_word_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Method", "### Method\n", "### Method\n\none\u{1f}two\u{1f}three\u{1f}four\u{1f}five\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r052_r2_s_in_rule_re_rule_x1c7",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 7 |", "| Rule 7 |", "| Rule\u{1c}7 |")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r053_r2_strip_duplicate_rule_7_row_prefixed_by_x1c",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 8 |", "| Rule 8 |", "\u{1c}| Rule 7 | a | b | c | d |\n| Rule 8 |")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r054_r2_universal_newlines_lone_r_starts_other_inside_the_record",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "as epic 004's definition of done requires `[decision]`.", "as epic 004's definition of done requires `[decision]`.\r## Other")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r055_r2_u_2028_is_not_a_line_break_for_split_n",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "as epic 004's definition of done requires `[decision]`.", "as epic 004's definition of done requires `[decision]`.\u{2028}## Other")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r056_r2_re_i_with_dotted_capital_i_open_i_tem_without_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Method", "### Method\n", "### Method\n\nOPEN İTEM nine is listed here `[decision]`.\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r057_r2_re_i_dotted_i_on_a_required_word_revi_ewer_line",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Reviewer:**", "**Reviewer:**", "**REVİEWER:**")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r058_r2_d_non_ascii_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 7 |", "| Rule 7 |", "| Rule \u{667} |")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r059_r2_d_non_ascii_constant_id_c",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-122 |", "| C-122 |", "| C-\u{661}\u{662}\u{662} |")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r060_r2_normpath_absolute_link_doc_design_spec_wire_md",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-07 |", "| implemented: senders", "| [w](/doc/design/spec/wire.md) implemented: senders")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r061_r2_normpath_link_to_a_directory",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-07 |", "| implemented: senders", "| [w](../) implemented: senders")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r062_r2_normpath_a_inside_the_path",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-07 |", "| implemented: senders", "| [w](../spec/../spec/wire.md#traceability-table) implemented: senders")),
        expect: &[],
    },
    Denial {
        name: "review_r063_r2_tablerules_keep_headerless_duplicate_rule_run",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Constants of the ticket's list", "### Constants of the ticket's list\n", "### Constants of the ticket's list\n\n| Rule 9 | x | y |\n")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r064_r2_tablerules_cellwise_separator_on_the_constants_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Constant | What the constants table says |", "|---|---|---|---|---|", "|-:-|---|---|---|---|")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r065_r2_slug_combining_mark_in_heading_python_drops_it",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Traceability table", "## Traceability table\n", "## Traceability table\u{301}\n")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "review_r066_r2_tomllib_width_as_float_16_0_frames",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"frames\"", "width = 16", "width = 16.0")),
        expect: &[],
    },
    Denial {
        name: "review_r067_r2_tomllib_duplicate_key_invalid_toml",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| anchored_replace(t, "name = \"frames\"", "width = 16", "width = 16\nwidth = 16")),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "review_r100_r2_d_non_ascii_duplicate_extra_row_rule",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 8 |", "| Rule 8 |", "| Rule \u{667} | dup | x | y | z |\n| Rule 8 |")),
        // outside the repertoire: red by rule (R13)
        expect: &[("REPERTOIRE", "")],
    },
    // review round 3 (PR #42): expected = the old script's verdict at c02d6ca,
    // or REGEX_LIMIT where the script's answer lies past the backtracking limit
    Denial {
        name: "review_r3_001_backtracking_4000_backticked_names_then_zzz_32_bits_samples",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "| C-105 |", &format!("| C-105 {} x `zzz` (32 bits, samples) |", "`a`, ".repeat(4000)))),
        // the script finds AC2; the port's width pattern hits the backtracking
        // limit on the list first: red by rule (R19)
        expect: &[("REGEX_LIMIT", "")],
    },
    Denial {
        name: "review_r3_002_backtracking_6000_backticked_names_then_zzz_32_bits_samples",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "| C-105 |", &format!("| C-105 {} x `zzz` (32 bits, samples) |", "`a`, ".repeat(6000)))),
        // the script finds AC2; the port's width pattern hits the backtracking
        // limit on the list first: red by rule (R19)
        expect: &[("REGEX_LIMIT", "")],
    },
    Denial {
        name: "review_r3_003_lone_r_in_the_ticket_s_constants_line_adds_c_999_after_it",
        file: TICKET,
        edit: DenialEdit::Text(|t| anchored_replace(t, "Constants traced:", "Constants traced:", "Constants traced: C-999\r")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r3_004_crlf_ticket",
        file: TICKET,
        edit: DenialEdit::Text(|t| anchored_replace(t, "", "ALL\n", "\r\n")),
        expect: &[],
    },
    Denial {
        name: "review_r3_005_lone_r_starts_3_8_extra_in_the_architecture_s_section_3",
        file: ARCH_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### 3.7 Redundancy", "### 3.7 Redundancy", "### 3.7 Redundancy\r\r### 3.8 Extra\r")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r3_006_lone_r_inside_a_rule_row_row_breaks",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| Rule 7 |", "| Rule 7 |", "| Rule 7 |\r")),
        expect: &[("AC1", ""), ("DoD", "")],
    },
    Denial {
        name: "review_r3_007_tab_after_the_heading_hashes_ttraceability_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Traceability table", "## Traceability table\n", "##\tTraceability table\n")),
        expect: &[],
    },
    Denial {
        name: "review_r3_008_closing_hashes_traceability_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Traceability table", "## Traceability table\n", "## Traceability table ##\n")),
        expect: &[],
    },
    Denial {
        name: "review_r3_009_seven_hashes_are_not_a_heading_review_record",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Review record", "## Review record", "####### Review record")),
        expect: &[("AC1", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r3_010_indented_heading_review_record",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "## Review record", "## Review record", " ## Review record")),
        expect: &[("AC1", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r3_011_indented_fence_opens_code_before_the_audit_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Per-receiver field audit", "### Per-receiver field audit\n", "### Per-receiver field audit\n\n  ```\n")),
        expect: &[("AC3", ""), ("AC4", "")],
    },
    Denial {
        name: "review_r3_012_tilde_fence_is_not_a_fence_for_004_22",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Per-receiver field audit", "### Per-receiver field audit\n", "### Per-receiver field audit\n\n~~~\nplain\n~~~\n")),
        expect: &[],
    },
    Denial {
        name: "review_r3_013_escaped_bracket_x_nope_still_a_link_to_the_regex",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "**Purpose:**", "**Purpose:** \\[x](#nope)")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r3_014_nested_brackets_a_b_traceability_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "**Purpose:**", "**Purpose:** [a [b]](#traceability-table)")),
        expect: &[],
    },
    Denial {
        name: "review_r3_015_upper_case_anchor_traceability_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "**Purpose:**", "**Purpose:** [t](#Traceability-Table)")),
        expect: &[("AC4", "")],
    },
    Denial {
        name: "review_r3_016_s_unit_for_residence_time_in_a_wire_visible_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "`structural_latency` (32 bits, samples)", "`structural_latency` (32 bits, µs)")),
        expect: &[],
    },
    Denial {
        name: "review_r3_017_superscript_glued_to_a_unit_samples",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "`structural_latency` (32 bits, samples)", "`structural_latency` (32 bits, samples²)")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r3_018_en_dash_range_c_20_c_22_in_a_constants_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-20, C-21, C-22 |", "| C-20, C-21, C-22 |", "| C-20 – C-22 |")),
        expect: &[],
    },
    Denial {
        name: "review_r3_019_minus_sign_range_c_20_c_22_not_a_range",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-20, C-21, C-22 |", "| C-20, C-21, C-22 |", "| C-20 − C-22 |")),
        expect: &[("AC2", "")],
    },
    Denial {
        name: "review_r3_020_final_sigma_in_a_heading_and_its_reference_copy_index",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Copy index (C-07)", "### Copy index (C-07)", "### Copy index (C-07)Σ")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "review_r3_021_greek_heading_referenced_in_upper_case_notes_notes",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "### Rules 1 to 22", "### Rules 1 to 22\n", "### Rules 1 to 22\n\n#### ΔΣ notes\n\nShort note here `[decision]`.\n")),
        expect: &[],
    },
    Denial {
        name: "review_r3_022_empty_match_verdict_cell_none_with_tabs",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| `magic` | media packet | none |", "| `magic` | media packet | none |", "| `magic` | media packet |\tnone\t|")),
        expect: &[],
    },
    Denial {
        name: "review_r3_023_date_as_16_nov_2026_month_date",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Date:**", "**Date:** 2026-11-16", "**Date:** 16 Nov. 2026")),
        expect: &[],
    },
    Denial {
        name: "review_r3_024_date_as_16_november_2026",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Date:**", "**Date:** 2026-11-16", "**Date:** 16 November, 2026")),
        expect: &[],
    },
    Denial {
        name: "review_r3_025_main_branch_split_by_a_tab_b_and_s",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "**Purpose:**", "the review record of ticket 004-22:", "the review record of ticket 004-22 (main\tbranch):")),
        expect: &[],
    },
    Denial {
        name: "review_r3_057_backtracking_list_of_4000_then_zzz_32_bits_samples_at_the_en",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "| implemented |", &format!("| implemented {} x `zzz` (32 bits, samples) |", "`a`, ".repeat(4000)))),
        // the script finds it (slowly); the port hits the backtracking limit: red by rule (R19)
        expect: &[("REGEX_LIMIT", "")],
    },
    Denial {
        name: "review_r3_058_backtracking_list_of_6000_then_zzz_32_bits_samples_at_the_en",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "| implemented |", &format!("| implemented {} x `zzz` (32 bits, samples) |", "`a`, ".repeat(6000)))),
        // the script finds it (slowly); the port hits the backtracking limit: red by rule (R19)
        expect: &[("REGEX_LIMIT", "")],
    },
    Denial {
        name: "review_r3_059_control_zzz_32_bits_samples_at_the_end_of_the_c_105_row_no_l",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| anchored_replace(t, "| C-105 |", "| implemented |", "| implemented x `zzz` (32 bits, samples) |")),
        expect: &[("AC2", "")],
    },
];
