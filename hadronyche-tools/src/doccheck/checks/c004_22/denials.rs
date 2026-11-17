//! The denial list of 004-22: the real documents with one edit, and what
//! the port (and the old script) must report. Consumed by
//! `tests/doc_004_22.rs` (one test each) and by `doccheck-equiv 004-22
//! --denials`. The last two are the review's port-weaker findings 1a and 1b
//! (PR #42, round 1).

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags};

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
        expect: &[("AC1", "has no Traceability section")],
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
        expect: &[("AC3", "has no section headed 'audit'")],
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
];
