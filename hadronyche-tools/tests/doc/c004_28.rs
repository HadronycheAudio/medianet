//! Ticket 004-28 — the regime stamp sections added after the split: the
//! observed DSCP of peers and the alignment group. Port of
//! `checks/004-28.sh` (deleted with this port; `doccheck-equiv 004-28`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_28`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, each with the exact set of criteria the port
//! and the old script fail on it. `denials_agree_with_the_script` (ignored
//! by default: it needs Python and the script from git history) runs every
//! denial through the harness sandbox against the old script.
//!
//! Not here, because they run tools or inspect the repository (they are in
//! `scripts/repo-state.d/004-28.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed, the count at the merged
//! text (AC5); the checker names no ref containing main and runs no other
//! check (DoD).
//!
//! Not denied: the budget's "fits with its arithmetic stated" branch is not
//! reachable from the current documents without rewriting the 004-25 and
//! 004-28 tables (the types need 904 sections, the overflow branch is the
//! live one); the "budget cannot be computed" assertion is denied with a
//! new type missing.
//!
//! Header and code: the header's AC4 lists "the outcomes Work 3 fixes";
//! the code checks the five categories and, for the DSCP category, only
//! when a DSCP field is wider than six bits (`DSCP_WIDE_FLAG`), as the port
//! does. The header's AC5 says "wire_tables 0 failed, at least the merged
//! count"; the code pins 12, kept in the repository-state file.

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_28::{SPEC, WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::equiv::script::{Sandbox, script_source};
use hadronyche_tools::doccheck::mutate::{
    delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section,
};
use hadronyche_tools::doccheck::ticket::CONSTANTS_MD;
use hadronyche_tools::doccheck::wire_toml::WIRE_TOML;
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion, assert_fails};

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

fn wire(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(WIRE_MD, edit)
}

fn sub(heading: &'static str, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(|t| within_section(t, heading, edit))
}

fn stamp(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Regime stamp$", edit)
}

fn dscp(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Observed DSCP of peers section$", edit)
}

fn align(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Alignment group section$", edit)
}

fn count(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Section count with the types of 004-28$", edit)
}

fn toml(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(WIRE_TOML, edit)
}

fn toml_swap(from: &str, to: &str) -> Docs {
    toml(|t| replace_all(t, &regex::escape(from), to))
}

fn consts(edit: impl FnOnce(&str) -> String) -> Docs {
    base().mutate(CONSTANTS_MD, edit)
}

/// `edit` applied to both new subsections.
fn both_new(edit: fn(&str) -> String) -> Docs {
    wire(|t| {
        within_section(
            &within_section(t, "^Observed DSCP of peers section$", edit),
            "^Alignment group section$",
            edit,
        )
    })
}

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
    let re = regex::Regex::new(line).expect("line pattern");
    let at = text
        .split('\n')
        .position(|l| re.is_match(l))
        .expect("a line matches");
    let p = regex::Regex::new(pattern).expect("pattern");
    text.split('\n')
        .enumerate()
        .map(|(i, l)| {
            if i == at {
                p.replace_all(l, regex::NoExpand(to)).into_owned()
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

/// One denial: its name, the exact set of criteria both the port and the
/// old script fail on it (empty: both pass), the criterion and message the
/// port must report, and the mutated documents. Plain data, so the list can
/// move into the shared denial-list form.
struct Denial {
    name: &'static str,
    fails: &'static [&'static str],
    criterion: &'static str,
    needle: &'static str,
    docs: fn() -> Docs,
}

/// Checks one denial on the port: the exact failed set, and the targeted
/// message.
fn deny(d: &Denial) {
    let r = run(&(d.docs)());
    let got: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(got, d.fails, "{}: failed criteria\n{}", d.name, r.render());
    if !d.fails.is_empty() {
        assert_fails(&r, d.criterion, d.needle);
    }
}

/// Each entry becomes a `Denial` in `DENIALS` and a `#[test]` of it.
macro_rules! denials {
    ($($name:ident: [$($f:literal),*], $crit:literal, $needle:literal, $docs:expr;)*) => {
        const DENIALS: &[Denial] = &[$(Denial {
            name: stringify!($name),
            fails: &[$($f),*],
            criterion: $crit,
            needle: $needle,
            docs: || $docs,
        },)*];
        $(
            #[test]
            fn $name() {
                deny(DENIALS.iter().find(|d| d.name == stringify!($name)).expect("listed"));
            }
        )*
    };
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "Two section types exist, each with a type code and a maximum size
/// ≤ 1 252 bytes (epic AC-3)." Script: the constants and the merged text
/// read; the two types (one subsection each) with sizes within the bound,
/// stated, codes in 004-28's block and not 004-25's, ⌈M / F⌉ statements,
/// exact arithmetic; the merged types need 7 sections; the budget: the
/// complete overflow report and the cap condition (review C2).
#[test]
fn ac1_two_section_types_with_codes_and_sizes() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The DSCP section covers both C-152 observations (hellos, every
/// module; sync requests, holder only) and, in a holder's stamp, the
/// per-follower dropped-request counter of C-204 with a width."
#[test]
fn ac2_the_dscp_section_covers_both_observations_and_the_counter() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The alignment section carries exactly the four lateness figures
/// of C-160 (rolling 60 s window, C-182), the maximum since live (C-182)
/// and the reclaimable latency of C-157, each with a width and a unit (epic
/// AC-8)." Also review C1, the no-sample value of the maximum since live.
#[test]
fn ac3_the_alignment_section_carries_the_figures() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "At least 5 invalid forms with outcomes." Script: in the two new
/// subsections, each naming control reasons, ReservedNonZero and BadLength
/// where required, the categories of Work 3; no form rejects the since-live
/// no-sample value.
#[test]
fn ac4_at_least_five_invalid_forms_with_outcomes() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "Passes the 004-02 checker." Script (document part): wire.toml
/// parses; one regime stamp message per new type, control at DSCP 40 with
/// the table's max_size, a non-empty list of control reasons, fields
/// agreeing row for row; one entry message per entry table, bit for bit.
#[test]
fn ac5_wire_toml_agrees_with_the_tables() {
    assert_criterion(report(), "AC5");
}

/// DoD — evidence tags (subsection and prose paragraphs, policy
/// `PROSE_004_29`), open items with owner and falsifier, the merged tables
/// unchanged and the new reserved fields recorded, review C4 and the
/// hand-off on sender rule 3.
#[test]
fn dod_tags_open_items_and_merged_tables() {
    assert_criterion(report(), "DoD");
}

// ---------------------------------------------------------------- denials

denials! {
    denial_wire_md_missing: ["AC1", "AC2", "AC3", "AC4", "DoD"], "AC1", "does not exist", base().without(WIRE_MD);
    denial_wire_toml_missing: ["AC5"], "AC5", "does not exist", base().without(WIRE_TOML);
    ac1_denial_the_control_bound_constant: ["AC1"], "AC1", "gives the control bound C-17 as 1 252", consts(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 300"));
    ac2_denial_the_followers_constant: ["AC2"], "AC2", "the constants give 63 peers and 63 followers", consts(|t| replace_in_line(t, r"^\| C-114 \|", "at most 63 real followers", "at most 62 real followers"));
    ac2_denial_the_control_dscp_constant: ["AC2", "AC5"], "AC2", "the constants give control CS5 as 40", consts(|t| replace_in_line(t, r"^\| C-13 \|", r"control CS5 \(40\)", "control CS5 (41)"));
    ac3_denial_the_window_constant: ["AC3"], "AC3", "rolling 60 s window (C-182) (read", consts(|t| replace_in_line(t, r"^\| C-182 \|", "rolling 60 s window", "rolling 61 s window"));
    ac2_denial_the_rate_constant: ["AC2"], "AC2", "10 to 200 per second (C-112)", consts(|t| replace_in_line(t, r"^\| C-112 \|", "10 to 200 per second", "10 to 201 per second"));
    ac1_denial_the_placeholder: ["AC1"], "AC1", "placeholder replaced", stamp(|t| replace_in_line(t, "^The regime stamp is a module's declaration", "^", "To be filled. "));
    dod_denial_the_merged_header_changed: ["DoD"], "DoD", "header rows (bytes 0 to 57) are unchanged", stamp(|t| replace_in_line(t, r"^\| 52 \| 1 \| u8 \| Section index", r"^\| 52 \| 1 \|", "| 52 | 2 |"));
    ac1_denial_the_section_count_not_64: ["AC1"], "AC1", "allows 1 to 64 sections", stamp(|t| replace_in_line(t, r"^\| 53 \| 1 \| u8 \| Section count", "1 to 64", "1 to 63"));
    dod_denial_a_rejection_value_changed: ["DoD"], "DoD", "Rejection reasons enumeration is unchanged", wire(|t| replace_in_line(t, r"^\| 9 \| BadPacketIndex \|", r"^\| 9", "| 13"));
    dod_denial_the_reserved_codes_row_removed: ["DoD"], "DoD", "still records the section type codes row", wire(|t| delete_row(t, r"^\| Section type codes not assigned in v1"));
    dod_denial_the_stamp_reserved_rows_removed: ["DoD"], "DoD", "keeps its four regime stamp rows", sub("^Reserved fields$", |t| replace_all(t, "(?i)regime stamp", "stamp"));
    ac1_denial_no_code_block: ["AC1"], "AC1", "give 004-28 a block", stamp(|t| replace_in_line(t, r"^\| 16 to 31 \|", r"\| 004-28 \|", "| 004-99 |"));
    dod_denial_a_merged_type_missing: ["AC1", "DoD"], "DoD", "the seven merged section types of 004-25 are found once each", wire(|t| rename_heading(t, "^Calibration factor section$", "Factor section"));
    ac1_denial_the_merged_types_need_more_sections: ["AC1"], "AC1", "need 7 sections at their maxima (computed 8)", sub("^Multi-packet regime section$", |t| replace_all(t, "⌈256 / 149⌉ = 2", "⌈400 / 149⌉ = 3"));
    dod_denial_the_merged_health_entry_changed: ["AC1", "DoD"], "DoD", "the merged health entry is 65 bytes", sub("^Per-connection health section$", |t| replace_in_line(t, r"^\| 1 \| 65 × max 18", "max 18", "max 17"));
    ac1_denial_no_dscp_subsection: ["AC1", "AC2"], "AC1", "names the observed DSCP of peers section type (found 0)", wire(|t| rename_heading(t, "^Observed DSCP of peers section$", "Observed codes of peers section"));
    ac2_denial_no_dscp_section: ["AC1", "AC2"], "AC2", "the observed DSCP section is missing", wire(|t| rename_heading(t, "^Observed DSCP of peers section$", "Observed codes of peers section"));
    ac1_denial_no_usable_alignment_table: ["AC1", "AC3"], "AC1", "the alignment section type has no usable field table", align(|t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"));
    ac3_denial_no_alignment_section: ["AC1", "AC3"], "AC3", "the alignment group section is missing", align(|t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"));
    ac1_denial_a_type_over_the_bound: ["AC1", "AC5"], "AC1", "no more than the control bound", align(|t| replace_in_line(t, r"^\| 1 \| 63 × max 18", "max 18", "max 20"));
    ac1_denial_the_size_not_stated: ["AC1"], "AC1", "states its maximum section size as 941 bytes", dscp(|t| replace_all(t, r"58 \+ 883 = 941 bytes", "58 + 883 = 941"));
    ac1_denial_a_code_of_004_25: ["AC1"], "AC1", "type code inside the block the merged table gives 004-28", dscp(|t| replace_all(t, "Section type code 16 ", "Section type code 3 "));
    ac1_denial_no_count_statement: ["AC1"], "AC1", "has no statement of the form", dscp(|t| replace_all(t, "⌈63 / 63⌉", "63 over 63"));
    ac1_denial_a_wrong_count_statement: ["AC1"], "AC1", "the dscp section count is", dscp(|t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈63 / 63⌉ = 2"));
    ac1_denial_an_inexact_product: ["AC1"], "AC1", "every product and sum stated in the dscp subsection is exact", dscp(|t| replace_all(t, "63 × 14 = 882", "63 × 14 = 883"));
    ac1_denial_the_report_without_the_total: ["AC1"], "AC1", "states the total 904 sections", count(|t| every_in_line(t, REPORT_P, r"\b904\b", "many"));
    ac1_denial_the_report_without_the_design_owner: ["AC1"], "AC1", "goes to the design owner", count(|t| replace_all(t, "design owner", "lead"));
    ac1_denial_the_report_without_rule_7: ["AC1"], "AC1", "cites merged sender rule 7", count(|t| every_in_line(t, REPORT_P, "a stamp that needs more cannot be sent", "a stamp that needs more is not sent"));
    ac1_denial_the_report_without_no_stamp: ["AC1"], "AC1", "sends no stamp at all", count(|t| every_in_line(t, REPORT_P, "(?i)no stamp", "nothing"));
    ac1_denial_the_report_without_the_threshold: ["AC1"], "AC1", "gives the threshold in entries", count(|t| every_in_line(t, REPORT_P, "15 120", "many"));
    ac1_denial_the_report_without_the_cap: ["AC1"], "AC1", "names the connection cap", count(|t| every_in_line(t, REPORT_P, r"(?i)\bcap", "limit"));
    dod_denial_the_report_untagged: ["DoD"], "DoD", "the overflow report paragraph carries an evidence tag", count(|t| strip_tags(t, REPORT_P));
    ac1_denial_no_cap_condition: ["AC1"], "AC1", "the cap condition is stated in closed form", count(|t| replace_all(t, "18⌉", "18 ⌉"));
    ac1_denial_the_cap_condition_without_its_width: ["AC1"], "AC1", "the figure width it assumes", count(|t| replace_all(t, "(?i)assum", "tak"));
    ac1_denial_the_cap_condition_without_open_item_17: ["AC1"], "AC1", "names Open item 17", count(|t| replace_all(t, "Open item 17", "Open item 99"));
    ac1_denial_the_cap_condition_without_006_19: ["AC1"], "AC1", "the cap condition names 006-19", count(|t| replace_all(t, "006-19", "006-99"));
    ac1_denial_the_own_limit_not_exact: ["AC1"], "AC1", "limit for its own entry size", count(|t| replace_all(t, "× 18 = 504", "× 18 = 505"));
    ac1_denial_no_alternative_entry_size: ["AC1"], "AC1", "limit for one alternative entry size", count(|t| replace_all(t, "⌊1 193 /", "⌊1 194 /"));
    ac1_denial_no_budget_without_both_types: ["AC1", "AC3", "AC4"], "AC1", "the budget cannot be computed", wire(|t| rename_heading(t, "^Alignment group section$", "Group section"));
    ac2_denial_a_narrow_peer_id: ["AC1", "AC2", "AC5"], "AC2", "names its peer by a 64-bit module id", dscp(|t| replace_in_line(t, r"^\| 0 \| 8 \| u64 \| Peer module id", r"\| 8 \| u64", "| 4 | u32"));
    ac2_denial_the_hello_dscp_without_63: ["AC2"], "AC2", "observed on the peer's hellos", dscp(|t| every_in_line(t, HELLO_ROW, r"\b63\b", "sixty-three"));
    ac2_denial_the_sync_dscp_not_holder_only: ["AC2"], "AC2", "observed on sync requests, holder only", dscp(|t| every_in_line(&every_in_line(t, SYNC_ROW, "(?i)holder", "leader"), DROP_ROW, "(?i)holder", "leader"));
    ac2_denial_the_counter_without_its_rate: ["AC2"], "AC2", "names the accepted rate of 200", dscp(|t| every_in_line(t, DROP_ROW, r"\b200\b|C-204", "x"));
    ac2_denial_the_counter_without_a_width: ["AC1", "AC2", "AC5"], "AC2", "has a width (C-204)", dscp(|t| replace_in_line(t, DROP_ROW, r"^\| 10 \| 4 \|", "| 10 | 0 |"));
    ac2_denial_the_counter_without_a_unit: ["AC2"], "AC2", "has a unit: requests", dscp(|t| every_in_line(t, DROP_ROW, r"(?i)requests?|per second", "items"));
    ac2_denial_the_counter_with_two_rules: ["AC2"], "AC2", "exactly one wrap or saturation rule", dscp(|t| every_in_line(t, DROP_ROW, "counted in", "saturating, counted in"));
    ac2_denial_a_state_field_without_four_states: ["AC2"], "AC2", "enumerates four states", dscp(|t| every_in_line(t, HELLO_ROW, "Hello DSCP code:", "Hello DSCP code, a C-129 state:"));
    ac2_denial_a_not_observed_code_inside_0_to_63: ["AC2"], "AC2", "the not-observed encoding is stated and told", dscp(|t| every_in_line(t, HELLO_ROW, "130, 131 or 132", "30, 131 or 132"));
    ac2_denial_no_dscp_0_to_63: ["AC2"], "AC2", "the DSCP proper is 0 to 63", dscp(|t| every_in_line(&every_in_line(t, HELLO_ROW, "0 to 63", "0 through 63"), SYNC_ROW, "0 to 63", "0 through 63"));
    ac2_denial_no_entry_order: ["AC2"], "AC2", "the entry order of the observed DSCP section", dscp(|t| replace_all(t, "(?i)order", "sequence"));
    ac2_denial_no_constant_for_the_count: ["AC2"], "AC2", "entry count derives from the constants", dscp(|t| replace_all(t, "C-34|C-114|C-217", "C-99"));
    ac2_denial_the_count_not_the_peers: ["AC1", "AC2"], "AC2", "holds at most 63 entries", dscp(|t| replace_all(t, "⌈63 / 63⌉ = 1", "⌈64 / 63⌉ = 2"));
    dod_denial_no_sender_rule_3: ["DoD"], "DoD", "falls under sender rule 3", dscp(|t| replace_all(t, "sender rule 3", "sender rule 4"));
    ac3_denial_a_key_off_the_health_width: ["AC1", "AC3", "AC5"], "AC3", "the health key's stream number", align(|t| replace_in_line(t, r"^\| 8 \| 2 \| u16 \| Stream number", r"\| 2 \| u16", "| 4 | u32"));
    ac3_denial_a_figure_renamed: ["AC3"], "AC3", "exactly the four lateness figures", align(|t| replace_in_line(t, r"^\| 31 \| 8 \| i64 \| 99\.9th percentile", r"99\.9th", "99.95th"));
    ac3_denial_a_figure_without_a_unit: ["AC3"], "AC3", "each lateness figure has a width and a unit", align(|t| replace_in_line(t, r"^\| 15 \| 8 \| i64 \| 50th percentile", ", in nanoseconds", ""));
    ac3_denial_no_rolling_window: ["AC3"], "AC3", "states its window as a rolling 60 s window in wire.md", align(|t| replace_all(t, "rolling 60 s", "rolling 60-second"));
    ac3_denial_the_since_live_maximum_without_a_unit: ["AC3"], "AC3", "the maximum since the connection went live is one figure", align(|t| replace_in_line(t, SINCE_ROW, ", in nanoseconds", ""));
    ac3_denial_the_reclaimable_latency_without_a_unit: ["AC3"], "AC3", "the reclaimable latency is one figure", align(|t| every_in_line(t, RECL_ROW, "(?i)nanoseconds|slice|sample|packet|second", "units"));
    ac3_denial_the_reclaimable_latency_unbounded: ["AC3"], "AC3", "bounded by A", align(|t| every_in_line(t, RECL_ROW, r"\bA\b|audition", "the bound"));
    ac3_denial_the_count_without_its_constant: ["AC3"], "AC3", "stated with the constant it derives from", align(|t| every_in_line(t, "⌈16 128 / 18⌉", r"C-\d+", "x"));
    ac3_denial_no_split: ["AC3"], "AC3", "the split of the alignment type", align(|t| replace_all(t, "(?i)split|several sections|each section", "spread"));
    ac3_denial_no_alignment_order: ["AC3"], "AC3", "the entry order of the alignment group", align(|t| replace_all(t, "(?i)order", "sequence"));
    ac3_denial_no_no_sample_value: ["AC3"], "AC3", "states the value it carries when it has no sample", align(|t| every_in_line(t, SINCE_ROW, "no live sample", "no data"));
    ac3_denial_the_no_sample_value_excluded: ["AC3"], "AC3", "admitted by its own valid range", align(|t| replace_all(&every_in_line(t, SINCE_ROW, r"\(order rule below\) \|$", "(order rule below); other than -2^63 when a sample exists |"), "(?i)first live sample", "first live datum"));
    ac3_denial_the_order_rule_silent_on_no_sample: ["AC3"], "AC3", "stated to hold or be exempt", align(|t| every_in_line(t, r"^\*\*Order rule and the empty window", r"holds for|no-sample|−2\^63|exempt", "x"));
    ac4_denial_a_form_rejecting_the_no_sample_value: ["AC4"], "AC4", "rejects the since-live maximum", align(|t| replace_in_line(t, r"^\| a maximum since live holding its no-sample value", "$", "\n| a maximum since live of -2^63 | drop the section and count under BadLength |"));
    ac4_denial_fewer_than_five_forms: ["AC4"], "AC4", "at least 5 invalid forms with outcomes in the new subsections", both_new(|t| replace_all(t, r"\| Invalid form \| Required outcome \|", "| Form | Outcome |"));
    ac4_denial_a_reason_outside_the_control_layer: ["AC4"], "AC4", "names a reason outside the control layer", align(|t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadChecksum"));
    ac4_denial_a_silent_form: ["AC4"], "AC4", "names no Reject reason", dscp(|t| replace_in_line(t, r"^\| a dropped sync request counter other than 0", r"\| drop the section.*$", "| dropped silently |"));
    ac4_denial_reserved_bits_without_reserved_non_zero: ["AC4"], "AC4", "must name ReservedNonZero", align(|t| replace_in_line(t, r"^\| reserved bits non-zero", "ReservedNonZero", "BadLength"));
    ac4_denial_a_length_form_without_bad_length: ["AC4"], "AC4", "a length form must name BadLength", dscp(|t| replace_in_line(t, r"^\| section length inconsistent", "BadLength", "TooShort"));
    ac4_denial_a_count_form_without_bad_length: ["AC4"], "AC4", "a count-above-maximum form must name BadLength", dscp(|t| replace_in_line(t, r"^\| entry count above its maximum", "BadLength", "TooShort"));
    ac4_denial_no_dscp_form: ["AC4"], "AC4", "an invalid form covers: a DSCP value above 63", dscp(|t| form_rows(t, "DSCP", "code"));
    dod_denial_an_untagged_subsection: ["DoD"], "DoD", "the dscp subsection carries an evidence tag", dscp(|t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""));
    dod_denial_an_untagged_paragraph: ["DoD"], "DoD", "every prose paragraph of 5 or more words in the alignment subsection", align(|t| strip_tags(t, r"^\*\*Widths and units\.\*\*"));
    dod_denial_a_shared_address_without_its_source: ["DoD"], "DoD", "review C4", align(|t| replace_in_line(t, r"^\*\*Widths and units\.\*\*", "^", "Two modules on one host share an address `[decision]`. "));
    dod_denial_an_open_item_without_an_owner: ["DoD"], "DoD", "every Open item paragraph", stamp(|t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "Open item 99: the spare bits are unassigned `[decision]`.\n\n"));
    dod_denial_a_new_reserved_field_not_recorded: ["DoD"], "DoD", "every reserved field of the new types", wire(|t| delete_row(t, r"^\| Alignment entry bits 7..1 of byte 14"));
    ac5_denial_wire_toml_does_not_parse: ["AC5"], "AC5", "does not parse", toml(|t| format!("{t}\n[[[broken"));
    ac5_denial_a_multi_line_string_the_reader_rejects: ["AC5"], "AC5", "does not parse", toml(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n"));
    ac5_denial_no_message_for_a_type: ["AC5"], "AC5", "exactly one regime stamp message for the dscp section type (found 0)", toml_swap("name = \"regime stamp observed DSCP of peers section\"", "name = \"regime stamp observed codes of peers section\"");
    ac5_denial_a_max_size_off_the_table: ["AC5"], "AC5", "the dscp message of wire.toml is control", toml_swap(DSCP_HEAD, &DSCP_HEAD.replace("max_size = 941", "max_size = 942"));
    ac5_denial_no_reasons: ["AC5"], "AC5", "the dscp message names only control reasons", toml_swap(DSCP_HEAD, "name = \"regime stamp observed DSCP of peers section\"\nlayer = \"control\"\ndscp = 40\nmax_size = 941\nsection_of = \"regime stamp\"\nrejection_reasons = []");
    ac5_denial_fields_that_disagree: ["AC5"], "AC5", "the dscp fields of wire.toml agree", toml(|t| replace_all(t, &regex::escape("name = \"entry_count\"\noffset = 464\nwidth = 8\ntype = \"u8\"\nmeaning = \"entry count: how many peer entries follow\""), "name = \"entry_count\"\noffset = 464\nwidth = 16\ntype = \"u8\"\nmeaning = \"entry count: how many peer entries follow\""));
    ac5_denial_an_entry_message_missing: ["AC5"], "AC5", "one entry message per entry table of the dscp section", toml_swap("name = \"regime stamp observed DSCP entry\"", "name = \"regime stamp observed DSCP item\"");
    ac5_denial_an_entry_message_that_disagrees: ["AC5"], "AC5", "the entry table of regime stamp observed DSCP entry agrees", toml_swap(DSCP_ENTRY, &DSCP_ENTRY.replace("max_size = 14", "max_size = 15"));
    ac2_denial_a_heading_after_a_python_line_break: ["AC1", "AC2", "AC3", "AC4", "DoD"], "AC2", "the observed DSCP section is missing", stamp(|t| replace_in_line(t, "^Section type code 16 ", "^", "\u{2028}## Interlude\u{2028}"));
    ac2_denial_a_word_character_only_python_knows: ["AC2"], "AC2", "names the accepted rate of 200", dscp(|t| every_in_line(&every_in_line(t, DROP_ROW, r"\b200\b", "200²"), DROP_ROW, "C-204", "C-2x4"));
    ac4_denial_a_validity_table_without_separator: ["AC4"], "AC4", "at least 5 invalid forms with outcomes in the new subsections", both_new(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n"));
    accepted_a_separator_row_only_the_scripts_accept: [], "", "", dscp(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n"));
    ac1_denial_no_control_bound_where_the_script_stops: ["AC1"], "AC1", "the script stops here with a Python exception",
        consts(|t| delete_row(t, r"^\| C-17 \|"));
}

/// Every denial fixture, run against the old script in the harness sandbox
/// (the documents exported from the same in-memory fixture): the script's
/// failed criteria must be exactly the denial's, as the port's are.
/// Ignored by default: it needs Python and the script (from the working
/// tree, or from git history once deleted).
#[test]
#[ignore = "runs the old script; needs python3 and git history"]
fn denials_agree_with_the_script() {
    let (text, from) =
        script_source(&repo_root(), SPEC.id).expect("the script, in the tree or its history");
    let control = Sandbox::new(base(), SPEC.id, &text, SPEC.script_env)
        .expect("sandbox")
        .run();
    assert!(
        control.passed,
        "the script must pass on the unmutated documents: {}",
        control.tail
    );
    let mut disagree = Vec::new();
    for d in DENIALS {
        let v = Sandbox::new(&(d.docs)(), SPEC.id, &text, SPEC.script_env)
            .expect("sandbox")
            .run();
        let got: Vec<&str> = v.criteria.iter().map(String::as_str).collect();
        let port: Vec<String> = run(&(d.docs)()).failed_criteria().into_iter().collect();
        println!(
            "DENIAL {} | port {:?} | script {:?} passed={}",
            d.name, port, got, v.passed
        );
        if got != d.fails || v.passed != d.fails.is_empty() {
            disagree.push(format!(
                "{}: the script fails {got:?} (passed {}), the denial says {:?} ({})",
                d.name, v.passed, d.fails, v.tail
            ));
        }
    }
    assert!(
        disagree.is_empty(),
        "script {from}:\n{}",
        disagree.join("\n")
    );
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (26fb108, "failing checks
/// for the observed DSCP and alignment group sections") fails AC1 to AC5,
/// as the script did there. Skipped when the commit is not in the clone.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "26fb108f8927395c36333f5c1a007e86b42cbd76";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "AC5"],
        "{}",
        r.render()
    );
}
