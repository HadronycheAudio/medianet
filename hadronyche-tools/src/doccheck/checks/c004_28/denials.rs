//! The denial list of 004-28: the real documents with one edit, and what
//! the port and the old script must report on them (their failed criteria,
//! exactly; `CRASH` where the script stops with a Python exception).
//! Consumed by `tests/doc/c004_28.rs` (one test each) and by `doccheck-equiv
//! 004-28`, which runs each against the old script too. An entry with no
//! expectation is one both must pass.

use super::WIRE_MD;
use crate::doccheck::denial::{Denial, DenialEdit};
use crate::doccheck::mutate::{delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section};
use crate::doccheck::text::rx;
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;

/// `edit` applied to every table row of `text` that holds an outcome
/// ("drop the section").
fn form_rows(text: &str, from: &str, to: &str) -> String {
    text.split('\n')
        .map(|l| {
            if l.starts_with("| ") && l.contains("drop the section") {
                l.replace(from, to)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every match of `pattern` (a regex) on the first line matching `line`
/// replaced.
fn every_in_line(text: &str, line: &str, pattern: &str, to: &str) -> String {
    let re = rx(line);
    let at = text
        .split('\n')
        .position(|l| re.is_match(l))
        .expect("a line matches");
    let p = rx(pattern);
    text.split('\n')
        .enumerate()
        .map(|(i, l)| {
            if i == at {
                p.replace_all_literal(l, to)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const REPORT_P: &str = r"^\*\*Reported: the 64-section bound is exceeded further";
const HELLO_ROW: &str = r"^\| 8 \| 1 \| u8 \| Hello DSCP code";
const SYNC_ROW: &str = r"^\| 9 \| 1 \| u8 \| Sync DSCP code";
const DROP_ROW: &str = r"^\| 10 \| 4 \| u32 \| Dropped sync requests";
const SINCE_ROW: &str = r"^\| 47 \| 8 \| i64 \| Maximum of the input's lateness since";
const RECL_ROW: &str = r"^\| 55 \| 8 \| i64 \| Reclaimable latency";
const DSCP_HEAD: &str = "name = \"regime stamp observed DSCP of peers section\"\nlayer = \"control\"\ndscp = 40\nmax_size = 941\nsection_of = \"regime stamp\"\nrejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]";
const DSCP_ENTRY: &str =
    "name = \"regime stamp observed DSCP entry\"\nlayer = \"control\"\ndscp = 40\nmax_size = 14";

/// The denials.
pub const DENIALS: &[Denial] = &[
    Denial {
        name: "denial_wire_md_missing",
        file: WIRE_MD,
        edit: DenialEdit::Delete,
        expect: &[("AC1", "does not exist"), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("DoD", "")],
    },
    Denial {
        name: "denial_wire_toml_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Delete,
        expect: &[("AC5", "does not exist")],
    },
    Denial {
        name: "ac1_denial_the_control_bound_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 300")),
        expect: &[("AC1", "gives the control bound C-17 as 1 252")],
    },
    Denial {
        name: "ac2_denial_the_followers_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-114 \|", "at most 63 real followers", "at most 62 real followers")),
        expect: &[("AC2", "the constants give 63 peers and 63 followers")],
    },
    Denial {
        name: "ac2_denial_the_control_dscp_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-13 \|", r"control CS5 \(40\)", "control CS5 (41)")),
        expect: &[("AC2", "the constants give control CS5 as 40"), ("AC5", "")],
    },
    Denial {
        name: "ac3_denial_the_window_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-182 \|", "rolling 60 s window", "rolling 61 s window")),
        expect: &[("AC3", "rolling 60 s window (C-182) (read")],
    },
    Denial {
        name: "ac2_denial_the_rate_constant",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-112 \|", "10 to 200 per second", "10 to 201 per second")),
        expect: &[("AC2", "10 to 200 per second (C-112)")],
    },
    Denial {
        name: "ac1_denial_the_placeholder",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, "^The regime stamp is a module's declaration", "^", "To be filled. "))),
        expect: &[("AC1", "placeholder replaced")],
    },
    Denial {
        name: "dod_denial_the_merged_header_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 52 \| 1 \| u8 \| Section index", r"^\| 52 \| 1 \|", "| 52 | 2 |"))),
        expect: &[("DoD", "header rows (bytes 0 to 57) are unchanged")],
    },
    Denial {
        name: "ac1_denial_the_section_count_not_64",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 63"))),
        expect: &[("AC1", "allows 1 to 64 sections")],
    },
    Denial {
        name: "dod_denial_a_rejection_value_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 9 \| BadPacketIndex \|", r"^\| 9", "| 13")),
        expect: &[("DoD", "Rejection reasons enumeration is unchanged")],
    },
    Denial {
        name: "dod_denial_the_reserved_codes_row_removed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Section type codes not assigned in v1")),
        expect: &[("DoD", "still records the section type codes row")],
    },
    Denial {
        name: "dod_denial_the_stamp_reserved_rows_removed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Reserved fields$", |t| replace_all(t, "(?i)regime stamp", "stamp"))),
        expect: &[("DoD", "keeps its four regime stamp rows")],
    },
    Denial {
        name: "ac1_denial_no_code_block",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\| 16 to 31 \|", r"\| 004-28 \|", "| 004-99 |"))),
        expect: &[("AC1", "give 004-28 a block")],
    },
    Denial {
        name: "dod_denial_a_merged_type_missing",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Calibration factor section$", "Factor section")),
        expect: &[("AC1", ""), ("DoD", "the seven merged section types of 004-25 are found once each")],
    },
    Denial {
        name: "ac1_denial_the_merged_types_need_more_sections",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Multi-packet regime section$", |t| replace_all(t, "⌈256 / 149⌉ = 2", "⌈400 / 149⌉ = 3"))),
        expect: &[("AC1", "need 7 sections at their maxima (computed 8)")],
    },
    Denial {
        name: "dod_denial_the_merged_health_entry_changed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Per-connection health section$", |t| replace_in_line(t, r"^\| 1 \| 65 × max 18", "max 18", "max 17"))),
        expect: &[("AC1", ""), ("DoD", "the merged health entry is 65 bytes")],
    },
    Denial {
        name: "ac1_denial_no_dscp_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Observed DSCP of peers section$", "Observed codes of peers section")),
        expect: &[("AC1", "names the observed DSCP of peers section type (found 0)"), ("AC2", "")],
    },
    Denial {
        name: "ac2_denial_no_dscp_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Observed DSCP of peers section$", "Observed codes of peers section")),
        expect: &[("AC1", ""), ("AC2", "the observed DSCP section is missing")],
    },
    Denial {
        name: "ac1_denial_no_usable_alignment_table",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"))),
        expect: &[("AC1", "the alignment section type has no usable field table"), ("AC3", "")],
    },
    Denial {
        name: "ac3_denial_no_alignment_section",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"))),
        expect: &[("AC1", ""), ("AC3", "the alignment group section is missing")],
    },
    Denial {
        name: "ac1_denial_a_type_over_the_bound",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| 1 \| 63 × max 18", "max 18", "max 20"))),
        expect: &[("AC1", "no more than the control bound"), ("AC5", "")],
    },
    Denial {
        name: "ac1_denial_the_size_not_stated",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, r"58 \+ 883 = 941 bytes", "58 + 883 = 941"))),
        expect: &[("AC1", "states its maximum section size as 941 bytes")],
    },
    Denial {
        name: "ac1_denial_a_code_of_004_25",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "Section type code 16 ", "Section type code 3 "))),
        expect: &[("AC1", "type code inside the block the merged table gives 004-28")],
    },
    Denial {
        name: "ac1_denial_no_count_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉", "63 over 63"))),
        expect: &[("AC1", "has no statement of the form")],
    },
    Denial {
        name: "ac1_denial_a_wrong_count_statement",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈63 / 63⌉ = 2"))),
        expect: &[("AC1", "the dscp section count is")],
    },
    Denial {
        name: "ac1_denial_an_inexact_product",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "63 × 14 = 882", "63 × 14 = 883"))),
        expect: &[("AC1", "every product and sum stated in the dscp subsection is exact")],
    },
    Denial {
        name: "ac1_denial_the_report_without_the_total",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| every_in_line(t, REPORT_P, r"\b904\b", "many"))),
        expect: &[("AC1", "states the total 904 sections")],
    },
    Denial {
        name: "ac1_denial_the_report_without_the_design_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "design owner", "lead"))),
        expect: &[("AC1", "goes to the design owner")],
    },
    Denial {
        name: "ac1_denial_the_report_without_rule_7",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| every_in_line(t, REPORT_P, "a stamp that needs more cannot be sent", "a stamp that needs more is not sent"))),
        expect: &[("AC1", "cites merged sender rule 7")],
    },
    Denial {
        name: "ac1_denial_the_report_without_no_stamp",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| every_in_line(t, REPORT_P, "(?i)no stamp", "nothing"))),
        expect: &[("AC1", "sends no stamp at all")],
    },
    Denial {
        name: "ac1_denial_the_report_without_the_threshold",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| every_in_line(t, REPORT_P, "15 120", "many"))),
        expect: &[("AC1", "gives the threshold in entries")],
    },
    Denial {
        name: "ac1_denial_the_report_without_the_cap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| every_in_line(t, REPORT_P, r"(?i)\bcap", "limit"))),
        expect: &[("AC1", "names the connection cap")],
    },
    Denial {
        name: "dod_denial_the_report_untagged",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| strip_tags(t, REPORT_P))),
        expect: &[("DoD", "the overflow report paragraph carries an evidence tag")],
    },
    Denial {
        name: "ac1_denial_no_cap_condition",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "18⌉", "18 ⌉"))),
        expect: &[("AC1", "the cap condition is stated in closed form")],
    },
    Denial {
        name: "ac1_denial_the_cap_condition_without_its_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "(?i)assum", "tak"))),
        expect: &[("AC1", "the figure width it assumes")],
    },
    Denial {
        name: "ac1_denial_the_cap_condition_without_open_item_17",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "Open item 17", "Open item 99"))),
        expect: &[("AC1", "names Open item 17")],
    },
    Denial {
        name: "ac1_denial_the_cap_condition_without_006_19",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "006-19", "006-99"))),
        expect: &[("AC1", "the cap condition names 006-19")],
    },
    Denial {
        name: "ac1_denial_the_own_limit_not_exact",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "× 18 = 504", "× 18 = 505"))),
        expect: &[("AC1", "limit for its own entry size")],
    },
    Denial {
        name: "ac1_denial_no_alternative_entry_size",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Section count with the types of 004-28$", |t| replace_all(t, "⌊1 193 /", "⌊1 194 /"))),
        expect: &[("AC1", "limit for one alternative entry size")],
    },
    Denial {
        name: "ac1_denial_no_budget_without_both_types",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| rename_heading(t, "^Alignment group section$", "Group section")),
        expect: &[("AC1", "the budget cannot be computed"), ("AC3", ""), ("AC4", "")],
    },
    Denial {
        name: "ac2_denial_a_narrow_peer_id",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Peer module id", r"\| 8 \| u64", "| 4 | u32"))),
        expect: &[("AC1", ""), ("AC2", "names its peer by a 64-bit module id"), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_the_hello_dscp_without_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(t, HELLO_ROW, r"\b63\b", "sixty-three"))),
        expect: &[("AC2", "observed on the peer's hellos")],
    },
    Denial {
        name: "ac2_denial_the_sync_dscp_not_holder_only",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(&every_in_line(t, SYNC_ROW, "(?i)holder", "leader"), DROP_ROW, "(?i)holder", "leader"))),
        expect: &[("AC2", "observed on sync requests, holder only")],
    },
    Denial {
        name: "ac2_denial_the_counter_without_its_rate",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(t, DROP_ROW, r"\b200\b|C-204", "x"))),
        expect: &[("AC2", "names the accepted rate of 200")],
    },
    Denial {
        name: "ac2_denial_the_counter_without_a_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, DROP_ROW, r"^\| 10 \| 4 \|", "| 10 | 0 |"))),
        expect: &[("AC1", ""), ("AC2", "has a width (C-204)"), ("AC5", "")],
    },
    Denial {
        name: "ac2_denial_the_counter_without_a_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(t, DROP_ROW, r"(?i)requests?|per second", "items"))),
        expect: &[("AC2", "has a unit: requests")],
    },
    Denial {
        name: "ac2_denial_the_counter_with_two_rules",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(t, DROP_ROW, "counted in", "saturating, counted in"))),
        expect: &[("AC2", "exactly one wrap or saturation rule")],
    },
    Denial {
        name: "ac2_denial_a_state_field_without_four_states",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(t, HELLO_ROW, "Hello DSCP code:", "Hello DSCP code, a C-129 state:"))),
        expect: &[("AC2", "enumerates four states")],
    },
    Denial {
        name: "ac2_denial_a_not_observed_code_inside_0_to_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(t, HELLO_ROW, "130, 131 or 132", "30, 131 or 132"))),
        expect: &[("AC2", "the not-observed encoding is stated and told")],
    },
    Denial {
        name: "ac2_denial_no_dscp_0_to_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(&every_in_line(t, HELLO_ROW, "0 to 63", "0 through 63"), SYNC_ROW, "0 to 63", "0 through 63"))),
        expect: &[("AC2", "the DSCP proper is 0 to 63")],
    },
    Denial {
        name: "ac2_denial_no_entry_order",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "(?i)order", "sequence"))),
        expect: &[("AC2", "the entry order of the observed DSCP section")],
    },
    Denial {
        name: "ac2_denial_no_constant_for_the_count",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "C-34|C-114|C-217", "C-99"))),
        expect: &[("AC2", "entry count derives from the constants")],
    },
    Denial {
        name: "ac2_denial_the_count_not_the_peers",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈64 / 63⌉ = 2"))),
        expect: &[("AC1", ""), ("AC2", "holds at most 63 entries")],
    },
    Denial {
        name: "dod_denial_no_sender_rule_3",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "sender rule 3", "sender rule 4"))),
        expect: &[("DoD", "falls under sender rule 3")],
    },
    Denial {
        name: "ac3_denial_a_key_off_the_health_width",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| 8 \| 2 \| u16 \| Stream number", r"\| 2 \| u16", "| 4 | u32"))),
        expect: &[("AC1", ""), ("AC3", "the health key's stream number"), ("AC5", "")],
    },
    Denial {
        name: "ac3_denial_a_figure_renamed",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| 31 \| 8 \| i64 \| 99\.9th percentile", r"99\.9th", "99.95th"))),
        expect: &[("AC3", "exactly the four lateness figures")],
    },
    Denial {
        name: "ac3_denial_a_figure_without_a_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| 15 \| 8 \| i64 \| 50th percentile", ", in nanoseconds", ""))),
        expect: &[("AC3", "each lateness figure has a width and a unit")],
    },
    Denial {
        name: "ac3_denial_no_rolling_window",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_all(t, "rolling 60 s", "rolling 60-second"))),
        expect: &[("AC3", "states its window as a rolling 60 s window in wire.md")],
    },
    Denial {
        name: "ac3_denial_the_since_live_maximum_without_a_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, SINCE_ROW, ", in nanoseconds", ""))),
        expect: &[("AC3", "the maximum since the connection went live is one figure")],
    },
    Denial {
        name: "ac3_denial_the_reclaimable_latency_without_a_unit",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| every_in_line(t, RECL_ROW, "(?i)nanoseconds|slice|sample|packet|second", "units"))),
        expect: &[("AC3", "the reclaimable latency is one figure")],
    },
    Denial {
        name: "ac3_denial_the_reclaimable_latency_unbounded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| every_in_line(t, RECL_ROW, r"\bA\b|audition", "the bound"))),
        expect: &[("AC3", "bounded by A")],
    },
    Denial {
        name: "ac3_denial_the_count_without_its_constant",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| every_in_line(t, "⌈16 128 / 18⌉", r"C-\d+", "x"))),
        expect: &[("AC3", "stated with the constant it derives from")],
    },
    Denial {
        name: "ac3_denial_no_split",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_all(t, "(?i)split|several sections|each section", "spread"))),
        expect: &[("AC3", "the split of the alignment type")],
    },
    Denial {
        name: "ac3_denial_no_alignment_order",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_all(t, "(?i)order", "sequence"))),
        expect: &[("AC3", "the entry order of the alignment group")],
    },
    Denial {
        name: "ac3_denial_no_no_sample_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| every_in_line(t, SINCE_ROW, "no live sample", "no data"))),
        expect: &[("AC3", "states the value it carries when it has no sample")],
    },
    Denial {
        name: "ac3_denial_the_no_sample_value_excluded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_all(&every_in_line(t, SINCE_ROW, r"\(order rule below\) \|$", "(order rule below); other than -2^63 when a sample exists |"), "(?i)first live sample", "first live datum"))),
        expect: &[("AC3", "admitted by its own valid range")],
    },
    Denial {
        name: "ac3_denial_the_order_rule_silent_on_no_sample",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| every_in_line(t, r"^\*\*Order rule and the empty window", r"holds for|no-sample|−2\^63|exempt", "x"))),
        expect: &[("AC3", "stated to hold or be exempt")],
    },
    Denial {
        name: "ac4_denial_a_form_rejecting_the_no_sample_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| a maximum since live holding its no-sample value", "$", "\n| a maximum since live of -2^63 | drop the section and count under BadLength |"))),
        expect: &[("AC4", "rejects the since-live maximum")],
    },
    Denial {
        name: "ac4_denial_fewer_than_five_forms",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(&within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Form | Outcome |")), "^Alignment group section$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Form | Outcome |"))),
        expect: &[("AC4", "at least 5 invalid forms with outcomes in the new subsections")],
    },
    Denial {
        name: "ac4_denial_a_reason_outside_the_control_layer",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadChecksum"))),
        expect: &[("AC4", "names a reason outside the control layer")],
    },
    Denial {
        name: "ac4_denial_a_silent_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, r"^\| a dropped sync request counter other than 0", r"\| drop the section.*$", "| dropped silently |"))),
        expect: &[("AC4", "names no Reject reason")],
    },
    Denial {
        name: "ac4_denial_reserved_bits_without_reserved_non_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadLength"))),
        expect: &[("AC4", "must name ReservedNonZero")],
    },
    Denial {
        name: "ac4_denial_a_length_form_without_bad_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, r"^\| section length inconsistent", "BadLength", "TooShort"))),
        expect: &[("AC4", "a length form must name BadLength")],
    },
    Denial {
        name: "ac4_denial_a_count_form_without_bad_length",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, r"^\| entry count above its maximum", "BadLength", "TooShort"))),
        expect: &[("AC4", "a count-above-maximum form must name BadLength")],
    },
    Denial {
        name: "ac4_denial_no_dscp_form",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| form_rows(t, "DSCP", "code"))),
        expect: &[("AC4", "an invalid form covers: a DSCP value above 63")],
    },
    Denial {
        name: "dod_denial_an_untagged_subsection",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""))),
        expect: &[("DoD", "the dscp subsection carries an evidence tag")],
    },
    Denial {
        name: "dod_denial_an_untagged_paragraph",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| strip_tags(t, r"^\*\*Widths and units\.\*\*"))),
        expect: &[("DoD", "every prose paragraph of 5 or more words in the alignment subsection")],
    },
    Denial {
        name: "dod_denial_a_shared_address_without_its_source",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Alignment group section$", |t| replace_in_line(t, r"^\*\*Widths and units\.\*\*", "^", "Two modules on one host share an address `[decision]`. "))),
        expect: &[("DoD", "review C4")],
    },
    Denial {
        name: "dod_denial_an_open_item_without_an_owner",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "Open item 99: the spare bits are unassigned `[decision]`.\n\n"))),
        expect: &[("DoD", "every Open item paragraph")],
    },
    Denial {
        name: "dod_denial_a_new_reserved_field_not_recorded",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| Alignment entry bits 7..1 of byte 14")),
        expect: &[("DoD", "every reserved field of the new types")],
    },
    Denial {
        name: "ac5_denial_wire_toml_does_not_parse",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[[broken")),
        expect: &[("AC5", "does not parse")],
    },
    Denial {
        name: "ac5_denial_a_multi_line_string_the_reader_rejects",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n")),
        expect: &[("AC5", "does not parse")],
    },
    Denial {
        name: "ac5_denial_no_message_for_a_type",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"regime stamp observed DSCP of peers section\""), "name = \"regime stamp observed codes of peers section\"")),
        expect: &[("AC5", "exactly one regime stamp message for the dscp section type (found 0)")],
    },
    Denial {
        name: "ac5_denial_a_max_size_off_the_table",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("max_size = 941", "max_size = 942"))),
        expect: &[("AC5", "the dscp message of wire.toml is control")],
    },
    Denial {
        name: "ac5_denial_no_reasons",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), "name = \"regime stamp observed DSCP of peers section\"\nlayer = \"control\"\ndscp = 40\nmax_size = 941\nsection_of = \"regime stamp\"\nrejection_reasons = []")),
        expect: &[("AC5", "the dscp message names only control reasons")],
    },
    Denial {
        name: "ac5_denial_fields_that_disagree",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"entry_count\"\noffset = 464\nwidth = 8\ntype = \"u8\"\nmeaning = \"entry count: how many peer entries follow\""), "name = \"entry_count\"\noffset = 464\nwidth = 16\ntype = \"u8\"\nmeaning = \"entry count: how many peer entries follow\"")),
        expect: &[("AC5", "the dscp fields of wire.toml agree")],
    },
    Denial {
        name: "ac5_denial_an_entry_message_missing",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape("name = \"regime stamp observed DSCP entry\""), "name = \"regime stamp observed DSCP item\"")),
        expect: &[("AC5", "one entry message per entry table of the dscp section")],
    },
    Denial {
        name: "ac5_denial_an_entry_message_that_disagrees",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_ENTRY), &DSCP_ENTRY.replace("max_size = 14", "max_size = 15"))),
        expect: &[("AC5", "the entry table of regime stamp observed DSCP entry agrees")],
    },
    Denial {
        name: "ac2_denial_a_heading_after_a_python_line_break",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Regime stamp$", |t| replace_in_line(t, "^Section type code 16 ", "^", "\u{2028}## Interlude\u{2028}"))),
        expect: &[("REPERTOIRE", "")],
    },
    Denial {
        name: "ac2_denial_a_word_character_only_python_knows",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| every_in_line(&every_in_line(t, DROP_ROW, r"\b200\b", "200²"), DROP_ROW, "C-204", "C-2x4"))),
        expect: &[("AC2", "names the accepted rate of 200")],
    },
    Denial {
        name: "ac4_denial_a_validity_table_without_separator",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(&within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n")), "^Alignment group section$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n"))),
        expect: &[("AC4", "at least 5 invalid forms with outcomes in the new subsections")],
    },
    Denial {
        name: "accepted_a_separator_row_only_the_scripts_accept",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n"))),
        expect: &[],
    },
    Denial {
        name: "ac1_denial_no_control_bound_where_the_script_stops",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| C-17 \|")),
        expect: &[("AC1", "gives the control bound C-17 as 1 252"), ("CRASH", "")],
    },

    // ---- numbers and Python's classes: big integers, underscores, signs,
    // leading zeros, superscripts, empty and missing cells, duplicate rows,
    // exponents, values that make the script raise
    Denial {
        name: "num_c17_ten_to_19",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "10000000000000000000")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_c17_ten_to_20",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "100000000000000000000")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_c17_ten_to_36",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1000000000000000000000000000000000000")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_c17_beyond_i128",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "10000000000000000000000000000000000000000")),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_c17_leading_zero",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "01252")),
        expect: &[],
    },
    Denial {
        name: "num_c17_leading_plus",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "+1252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_c17_superscript_after",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252²")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_c17_superscript_before",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "²1 252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_c17_underscore",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1_252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "num_enum_big_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 99999999999999999999 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 10000000000000000000000000000000000000000 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_leading_zeros",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 00 |")),
        expect: &[],
    },
    Denial {
        name: "num_enum_empty_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "|  |")),
        expect: &[("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "num_enum_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 0² |")),
        expect: &[("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "num_enum_duplicate_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 13 | TooShort | a duplicate row | control, sync, media, measurement |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "num_enum_duplicate_row_same_value",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 0 | TooShort | a duplicate row | control, sync, media, measurement |")),
        expect: &[],
    },
    Denial {
        name: "num_enum_missing_layers_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", r" \| control, sync, media, measurement \|", " |")),
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "num_toml_dscp_underscore",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = 4_0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = \"40\""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_padded_signed_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = \" +040 \""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = \"_40\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_dscp_superscript_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = \"40²\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_toml_dscp_big",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = 100000000000000000000"))),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "num_toml_dscp_beyond_i128",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = 10000000000000000000000000000000000000000"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_toml_dscp_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = 40.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_dscp_plus",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("dscp = 40", "dscp = +40"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_float",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("max_size = 941", "max_size = 941.0"))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("max_size = 941", "max_size = \"941\""))),
        expect: &[],
    },
    Denial {
        name: "num_toml_max_size_bool",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("max_size = 941", "max_size = true"))),
        expect: &[("AC5", "")],
    },
    Denial {
        name: "num_toml_max_size_bad_string",
        file: WIRE_TOML,
        edit: DenialEdit::Text(|t| replace_all(t, &regex::escape(DSCP_HEAD), &DSCP_HEAD.replace("max_size = 941", "max_size = \"abc\""))),
        expect: &[("CRASH", "")],
    },
    Denial {
        name: "num_hello_dscp_empty_width_cell",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", "^\\| 8 \\| 1 \\|", "| 8 |  |"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", "")],
    },
    Denial {
        name: "num_hello_dscp_row_cut_to_two_cells",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", r"^.*$", "| 8 | 1 |"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", "")],
    },
    Denial {
        name: "num_sum_exponent_exact",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (2**3 + 933 = 941)"))),
        expect: &[],
    },
    Denial {
        name: "num_sum_exponent_wrong",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (2**3 + 933 = 942)"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_sum_leading_zero_syntax_error",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (08 + 933 = 941)"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_sum_beyond_i128_exact",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (2**130 - 2**130 + 941 = 941)"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_sum_negative_exponent",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (2**-1 + 940 = 941)"))),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "num_sum_result_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (1 + 1 = 10000000000000000000000000000000000000000)"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_sum_product_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (100000000000000000000 × 100000000000000000000 = 100000000000000000000)"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_sum_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, &regex::escape("58 + 883 = 941 bytes"), "58 + 883 = 941 bytes (58 + 883² = 941)"))),
        expect: &[],
    },
    Denial {
        name: "num_count_statement_leading_zero",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈063 / 63⌉ = 01"))),
        expect: &[],
    },
    Denial {
        name: "num_count_statement_big",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈100000000000000000000 / 63⌉ = 1"))),
        expect: &[("AC1", ""), ("AC2", "")],
    },
    Denial {
        name: "num_count_statement_beyond_i128",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈10000000000000000000000000000000000000000 / 63⌉ = 1"))),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "num_count_statement_superscript",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈63² / 63⌉ = 1"))),
        expect: &[("AC1", "")],
    },
    // ---- the re-proof checklist (points 3, 5 and 6): number forms, table
    // reader variants, enumeration ties, swaps and thresholds
    Denial {
        name: "chk_c17_zero",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "0")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "chk_c17_double_zero_seven",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "007")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "chk_c17_two_to_63",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "9223372036854775808")),
        expect: &[("AC1", "")],
    },
    Denial {
        name: "chk_c17_forty_five_digits",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "111111111111111111111111111111111111111111111")),
        expect: &[("INT_LIMIT", "")],
    },
    Denial {
        name: "chk_c17_micro_before",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "µ1 252")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "chk_c17_micro_after",
        file: CONSTANTS_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 252µ")),
        expect: &[("AC1", ""), ("CRASH", "")],
    },
    Denial {
        name: "chk_enum_swap",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(&replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 1 |"), r"^\| 1 \| BadMagic \|", r"^\| 1 \|", "| 0 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "chk_enum_tie",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 1 \| BadMagic \|", r"^\| 1 \|", "| 0 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "chk_enum_twelve_members",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| delete_row(t, r"^\| 12 \| ReservedNonZero \|")),
        expect: &[("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "chk_enum_fourteen_members",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 12 \| ReservedNonZero \|", "$", "\n| 13 | SpareReason | a fourteenth member | control |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "chk_enum_value_two_to_63",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| 9223372036854775808 |")),
        expect: &[("DoD", "")],
    },
    Denial {
        name: "chk_enum_value_micro",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", r"^\| 0 \|", "| µ0 |")),
        expect: &[("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "chk_enum_escaped_pipe",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| replace_in_line(t, r"^\| 0 \| TooShort \|", "fewer bytes", "fewer \\| bytes")),
        expect: &[],
    },
    Denial {
        name: "chk_hello_dscp_row_escaped_pipe",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", "$", " \\| x"))),
        expect: &[],
    },
    Denial {
        name: "chk_hello_dscp_row_blank_line_inside",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", "^", "\n"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", "")],
    },
    Denial {
        name: "chk_hello_dscp_row_ragged_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", "$", " extra |"))),
        expect: &[],
    },
    Denial {
        name: "chk_hello_dscp_row_fenced_row",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", "^", "```\n"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC3", ""), ("AC4", ""), ("AC5", ""), ("DoD", "")],
    },
    Denial {
        name: "chk_hello_dscp_row_offset_double_zero_seven",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", r"^\| \d+", "| 007"))),
        expect: &[("AC1", ""), ("AC5", "")],
    },
    Denial {
        name: "chk_hello_dscp_row_offset_plus",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", r"^\| (\d+)", "| +7"))),
        expect: &[("AC1", ""), ("AC2", ""), ("AC5", "")],
    },
    Denial {
        name: "chk_hello_dscp_row_offset_forty_five_digits",
        file: WIRE_MD,
        edit: DenialEdit::Text(|t| within_section(t, "^Observed DSCP of peers section$", |t| replace_in_line(t, "^\\| 8 \\| 1 \\| u8 \\| Hello DSCP code", r"^\| \d+", "| 111111111111111111111111111111111111111111111"))),
        expect: &[("INT_LIMIT", "")],
    },
];
