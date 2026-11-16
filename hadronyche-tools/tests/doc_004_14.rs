//! Ticket 004-14 — media packet size budget at 1 472 and 1 280 bytes. Port
//! of `checks/004-14.sh` (deleted with this port; `doccheck-equiv 004-14`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_14`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory. Every denial is also run
//! against the old script by the ignored test
//! `old_script_agrees_on_the_denials` (see `wp9_support`).
//!
//! Not here, because it runs a tool (it is in `scripts/repo-state-checks.sh`,
//! under 004-14): when wire.toml carries no `[budget]` table, `cargo run
//! --bin wire-check` must pass and print the default and fallback totals.
//! The script skipped this run under its `WIRE_MD`/`WIRE_TOML` overrides.
//!
//! Two assertions of the script cannot be reached by any document: the
//! `wire.toml has no [[message]] named ...` failure of `msg()` is printed
//! only with the list of failures, but every message it looks up feeds a
//! part the script requires next, so the exit "does not declare what the
//! budget needs" that follows discards it; and the `a 1 472 column has the
//! wrong bound` failure tests a bound the script has just set. The port
//! reproduces both (the first is dropped by the same exit).

#[macro_use]
mod wp9_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_14::{WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{
    delete_row, delete_section, rename_heading, replace_all, replace_in_line, strip_tags,
    within_section,
};
use hadronyche_tools::doccheck::text::rx;
use hadronyche_tools::doccheck::wire_toml::WIRE_TOML;
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion, assert_fails};

const CONSTANTS: &str = "doc/design/02-constants.md";

fn base() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| run(base()))
}

fn wire(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(WIRE_MD, edit)
}

/// wire.md with `edit` applied to the Size budget section only.
fn size(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(b, |t| within_section(t, "^Size budget$", edit))
}

fn toml(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(WIRE_TOML, edit)
}

fn consts(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(CONSTANTS, edit)
}

/// Applies `f` to every line of `t` matching `pattern`.
fn map_lines(t: &str, pattern: &str, f: impl Fn(&str) -> String) -> String {
    let re = rx(pattern);
    let mut hit = false;
    let out: Vec<String> = t
        .split('\n')
        .map(|l| {
            if re.is_match(l) {
                hit = true;
                f(l)
            } else {
                l.to_string()
            }
        })
        .collect();
    assert!(hit, "map_lines: no line matches {pattern:?}");
    out.join("\n")
}

/// Deletes the first line matching `pattern`.
fn drop_line(t: &str, pattern: &str) -> String {
    let re = rx(pattern);
    let lines: Vec<&str> = t.split('\n').collect();
    let i = lines
        .iter()
        .position(|l| re.is_match(l))
        .unwrap_or_else(|| panic!("drop_line: no line matches {pattern:?}"));
    [&lines[..i], &lines[i + 1..]].concat().join("\n")
}

/// Replaces the first `from` after the first `anchor`.
fn after(t: &str, anchor: &str, from: &str, to: &str) -> String {
    let at = t
        .find(anchor)
        .unwrap_or_else(|| panic!("after: no {anchor:?}"));
    let rel = t[at..]
        .find(from)
        .unwrap_or_else(|| panic!("after: no {from:?}"));
    let i = at + rel;
    format!("{}{}{}", &t[..i], to, &t[i + from.len()..])
}

/// The section with its "omitted" sentences reworded, so that the script's
/// second form ("a sentence on the fallback says nothing is omitted") does
/// not excuse the rule's lines, which are the lines that say "omit".
fn unexcused(t: &str) -> String {
    rx(r"\bomitted\b").replace_all(t, "dropped").into_owned()
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// Appends a one-column table whose caption is blank (so only `header`
/// names its case) to the Size budget section.
fn table(b: &Docs, header: &str, rows: &[&str]) -> Docs {
    let body: Vec<String> = rows.iter().map(|r| format!("| {r} |")).collect();
    size(b, |t| {
        format!(
            "{}\n\n\n\n\n\n| Part | {header} |\n|---|---|\n{}\n",
            t.trim_end(),
            body.join("\n")
        )
    })
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "The default-packet budget shows 6 channels plus all mandatory
/// headers fit in 1 472 bytes and states the spare bytes as a number (epic
/// AC-2)."
///
/// Script: a budget table (a Total row, a Spare row and an audio row; every
/// numeric column is one case) whose default column holds the mandatory
/// parts only, each part equal to the size the landed toml declares for it
/// (audio 6 × 48 × 4 from C-04, C-02, C-03), total at most 1 472, spare =
/// 1 472 minus total; the section cites C-100 and states the 1 472, 1 280
/// and 1 252 byte figures.
#[test]
fn ac1_the_default_packet_fits_and_the_spare_is_a_number() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The worst case (all optional parts at maximum) is also shown with
/// its spare or deficit."
///
/// Script: a worst case with the announcements at maximum and the extension,
/// and one with announcements at maximum and no extension; announcement
/// rows sum to the count byte plus 12 per announcement, the extension is 0
/// or the toml's size.
#[test]
fn ac2_the_worst_case_is_shown() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The 1 280-byte fallback budget uses 1 252 bytes of UDP payload,
/// shows 6 channels fit with headers and checksum at most 100 bytes
/// (C-137), and lists the optional parts omitted there."
///
/// Script: a fallback column at bound 1 252 with non-audio bytes at most
/// the cap of C-137; the section cites C-06 and C-137; an omission rule
/// naming announcements, the measurement extension and controller
/// messages with a priority (or a sentence on the fallback saying nothing
/// is omitted); a fallback that omits an announcement or the extension must
/// say it amends or scopes the landed sender obligations.
#[test]
fn ac3_the_fallback_budget_and_the_omission_rule() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "The open sub-question is answered with a verdict against its
/// falsifier."
///
/// Script: a paragraph containing "verdict"; with the mandatory headers and
/// checksum (46 bytes from the tables) within the cap it says C-137 holds
/// and not falsified; above it, falsified with "5 channels" and C-04 at 5
/// in the constants table.
#[test]
fn ac4_the_verdict_against_the_falsifier() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "Totals in `wire.md` match the sums from the machine-readable
/// tables (checked by the 004-02 checker or by hand with the arithmetic
/// shown)."
///
/// Script: every total is the sum of its rows and every component equals
/// the toml-computed size; the toml `[budget]` keys are bound to their case
/// by name tokens and equal the computed figures; wire.toml's media
/// packet, checksum, count and extension sizes agree. (The `wire-check` run
/// for a toml without a budget table is in repo-state-checks.sh.)
#[test]
fn ac5_the_totals_match_the_machine_readable_tables() {
    assert_criterion(report(), "AC5");
}

/// AC6 — "The controller allowance inside the cap for a single-stream
/// source (C-181) is stated as a number."
///
/// Script: the allowance table at both path sizes (allowance = bound -
/// audio - counted headers - checksum, or the cap less counted headers and
/// checksum), the upper-bound sentence, C-181's second stream, C-205's
/// audio member, and a sentence giving the sparse entries that fit beside
/// a transport state.
#[test]
fn ac6_the_controller_allowance_is_a_number() {
    assert_criterion(report(), "AC6");
}

/// DoD — "Every new factual claim carries an evidence tag" and "No new open
/// item without an owner and a falsifier": every unit of 5 or more
/// `[A-Za-z0-9]+` words of the section carries a tag; an "Open item" unit
/// names an owner and a falsifier.
#[test]
fn dod_evidence_tags_and_open_items() {
    assert_criterion(report(), "DoD");
}

/// Pointers — "Design documents updated where behaviour differs": no open
/// item owned by 004-14 and no "But see ... Budget" pointer remains, no
/// unresolved pointer to 004-14 (Budget) outside the section, the header-cap
/// open item is closed, the landed sentences the budget contradicts are
/// removed or cite the section, no provisional cap line is left.
#[test]
fn pointers_the_landed_sections_point_at_the_budget() {
    assert_criterion(report(), "pointer");
}

/// DoD — "The machine-readable tables parse and round-trip": the document
/// side is the toml reader and the checks of AC5; the tool side is in
/// repo-state-checks.sh.
#[test]
fn dod_the_machine_readable_tables_parse() {
    assert!(report().of("setup").is_empty(), "{}", report().render());
    assert!(report().of("CRASH").is_empty(), "{}", report().render());
}

// ---------------------------------------------------------------- denials

denials! {
    // ---- setup
    denial_wire_md_missing => ("setup", "cannot read wire.md", |b: &Docs| b.without(WIRE_MD));
    denial_wire_toml_missing => ("setup", "cannot read wire.toml", |b: &Docs| b.without(WIRE_TOML));
    denial_constants_missing => ("setup", "cannot read 02-constants.md", |b: &Docs| b.without(CONSTANTS));
    denial_a_constant_unreadable => ("setup", "is unreadable", |b| consts(b, |t| replace_all(t, "at most 100 bytes for all headers", "at most a hundred bytes for all headers")));
    denial_fallback_mtu_disagrees_with_the_control_bound => ("setup", "but C-17 states 1252", |b| consts(b, |t| replace_in_line(t, r"^\| C-06 \|", "1 280 bytes", "1 300 bytes")));
    denial_wire_toml_does_not_parse => ("setup", "does not parse", |b| toml(b, |t| format!("[[message.field]]\n{t}")));

    // ---- AC1
    ac1_denial_no_size_budget_section => ("AC1", "no level-2 heading 'Size budget'", |b| wire(b, |t| rename_heading(t, "^Size budget$", "Budget of sizes")));
    ac1_denial_a_column_not_named => ("AC1", "is not named as default, worst case or fallback", |b| table(b, "Alpha", &["Audio | 1 152", "Total | 1 152", "Spare | 320"]));
    ac1_denial_a_row_with_no_number => ("AC1", "has no number", |b| table(b, "Default", &["Audio | plenty", "Total | 5", "Spare | 1"]));
    ac1_denial_a_row_not_a_budget_part => ("AC1", "is not a recognised budget part", |b| table(b, "Default", &["Audio | 1 152", "Mystery | 3", "Total | 1 155", "Spare | 317"]));
    ac1_denial_no_spare_row_in_a_column => ("AC1", "has no spare row", |b| size(b, |t| format!("{}\n\n\n\n\n\n| Part | Default | Default again |\n|---|---|---|\n| Audio | 1 152 | 1 152 |\n| Total | 1 152 | 1 152 |\n| Spare | 320 |\n", t.trim_end())));
    ac1_denial_a_part_row_missing => ("AC1", "has no cksum row", |b| size(b, |t| delete_row(t, r"^\| Checksum \(CRC-32C\)")));
    ac1_denial_the_packet_header_differs_from_the_toml => ("AC1", "packet header [17] != 16", |b| size(b, |t| replace_in_line(t, r"^\| Packet header \| 16", r"\| 16 \| 16 \| 16 \|", "| 17 | 16 | 16 |")));
    ac1_denial_the_slice_header_differs_from_the_toml => ("AC1", "slice header [23] != 22", |b| size(b, |t| replace_in_line(t, r"^\| Slice header \| 22", r"\| 22 \| 22 \| 22 \|", "| 23 | 22 | 22 |")));
    ac1_denial_the_content_header_differs_from_the_toml => ("AC1", "content-message header [4] != 3", |b| size(b, |t| replace_in_line(t, r"^\| Content-message header", r"\| 3 \| 3 \| 3 \|", "| 4 | 3 | 3 |")));
    ac1_denial_the_audio_differs_from_c04_c02_c03 => ("AC1", "audio [1151] != 1152", |b| size(b, |t| replace_in_line(t, r"^\| Audio, 6", r"\| 1 152 \| 1 152 \| 1 152 \|", "| 1 151 | 1 152 | 1 152 |")));
    ac1_denial_the_checksum_differs_from_the_toml => ("AC1", "checksum [5] != 4", |b| size(b, |t| replace_in_line(t, r"^\| Checksum \(CRC", r"\| 4 \| 4 \| 4 \|", "| 5 | 4 | 4 |")));
    ac1_denial_a_total_over_the_bound => ("AC1", "exceeds the 1472-byte bound", |b| size(b, |t| replace_in_line(t, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 500")));
    ac1_denial_a_spare_that_is_not_the_bound_less_the_total => ("AC1", "spare 275 != 1472 - 1198", |b| size(b, |t| replace_in_line(t, r"^\| Spare against 1 472", "= 274", "= 275")));
    ac1_denial_the_default_column_carries_an_announcement => ("AC1", "must hold the mandatory parts only", |b| size(b, |t| replace_in_line(t, r"^\| Announcement container", r"\| 1 \| 1 \+ 4", "| 13 | 1 + 4")));
    ac1_denial_the_default_total_is_not_mandatory_plus_audio => ("AC1", "!= mandatory 46 + audio 1152", |b| size(b, |t| replace_in_line(&replace_in_line(t, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 199"), r"^\| Spare against 1 472", "= 274", "= 273")));
    ac1_denial_no_budget_table => ("AC1", "no budget table", |b| size(b, |t| replace_all(t, r"\*\*Total\*\*", "Sum")));
    ac1_denial_no_default_column => ("AC1", "no default 1 472 budget column", |b| size(b, |t| replace_all(t, r"Default: mandatory parts only", "Worst alpha")));
    ac1_denial_c100_not_cited => ("AC1", "does not cite C-100", |b| size(b, |t| replace_all(t, "C-100", "C-1OO")));
    ac1_denial_a_byte_figure_missing => ("AC1", "does not state the 1 472, 1 280 and 1 252 byte figures", |b| size(b, |t| replace_all(t, r"1 280", "1 281")));

    // ---- AC2
    ac2_denial_announcement_rows_not_the_count_plus_twelve_each => ("AC2", "announcement rows sum to 50", |b| size(b, |t| replace_in_line(t, r"^\| Announcement container", r"1 \+ 4 × 12 = 49 \| 1 \+ 4", "1 + 4 × 12 = 50 | 1 + 4")));
    ac2_denial_an_extension_that_is_not_the_toml_size => ("AC2", "extension 8 != 0 or 9", |b| size(b, |t| replace_in_line(t, r"^\| Measurement extension \| — \| 9", r"\| 9 \|", "| 8 |")));
    ac2_denial_a_worst_column_not_at_maximum => ("AC2", "announcements are not at maximum (25 != 49)", |b| size(b, |t| replace_in_line(t, r"^\| Announcement container", r"\| 1 \+ 4 × 12 = 49 \| 1 \+ 4", "| 1 + 2 × 12 = 25 | 1 + 4")));
    ac2_denial_no_worst_case_with_the_extension => ("AC2", "with all announcements and the extension", |b| size(b, |t| replace_all(t, r"Worst case: format maximum, all optional parts, with the extension", "Case two")));
    ac2_denial_no_worst_case_without_the_extension => ("AC2", "all announcements and no extension", |b| size(b, |t| replace_all(t, r"Worst case: format maximum, all announcements, no extension", "Case three")));

    // ---- AC3
    ac3_denial_no_fallback_column => ("AC3", "no 1 280-byte fallback budget column", |b| size(b, |t| delete_section(t, "^Budget on a fallback path")));
    ac3_denial_fallback_headers_above_the_cap => ("AC3", "above the 100-byte cap of C-137", |b| table(b, "Fallback", &["Packet header | 16", "Slice header | 22", "Announcement container | 1", "Content-message header | 3", "Measurement extension | 60", "Audio | 1 152", "Checksum | 4", "Total | 1 258", "Spare | -6"]));
    ac3_denial_c06_not_cited => ("AC3", "does not cite C-06", |b| size(b, |t| replace_all(t, r"C-06\b", "C-O6")));
    ac3_denial_c137_not_cited => ("AC3", "does not cite C-137", |b| size(b, |t| replace_all(t, "C-137", "C-13X")));
    ac3_denial_no_omission_rule_and_nothing_stated => ("AC3", "states neither an omission rule", |b| size(b, |t| t.split('\n').filter(|l| !l.to_lowercase().contains("omit")).collect::<Vec<_>>().join("\n")));
    ac3_denial_the_omission_rule_names_no_announcements => ("AC3", "does not say which announcements are omitted", |b| size(b, |t| map_lines(&unexcused(t), r"(?i)\bomit", |l| l.replace("announcement", "notice").replace("Announcement", "Notice"))));
    ac3_denial_the_omission_rule_names_no_extension => ("AC3", "does not say what happens to the measurement extension", |b| size(b, |t| map_lines(&unexcused(t), r"(?i)\bomit", |l| l.replace("measurement", "m.").replace("extension", "ext.").replace("Measurement", "M."))));
    ac3_denial_the_omission_rule_names_no_controller_messages => ("AC3", "does not say what happens to controller messages", |b| size(b, |t| map_lines(&unexcused(t), r"(?i)\bomit", |l| l.replace("controller", "sink").replace("Controller", "Sink"))));
    ac3_denial_the_omission_rule_states_no_priority => ("AC3", "states no priority", |b| size(b, |t| map_lines(&unexcused(t), r"(?i)\bomit", |l| rx(r"(?i)priorit\w*|in this order|\bfirst\b|\bthen\b|\bbefore\b|\bordered\b").replace_all(l, "x").into_owned())));
    ac3_denial_a_fallback_that_omits_without_amending => ("AC3", "amends or scopes the landed sender obligations", |b| size(b, |t| rx(r"(?i)\bamend\w*|\bscop\w*|\bsupersed\w*").replace_all(&unexcused(t), "x").into_owned()));

    // ---- AC4
    ac4_denial_no_verdict => ("AC4", "no verdict", |b| size(b, |t| rx(r"(?i)verdict").replace_all(t, "finding").into_owned()));
    ac4_denial_a_falsified_verdict_when_the_headers_fit => ("AC4", "the verdict says falsified", |b| size(b, |t| append(t, "Verdict: C-137 is falsified.")));
    ac4_denial_a_verdict_that_does_not_say_it_holds => ("AC4", "does not say C-137 holds", |b| size(b, |t| map_lines(t, r"(?i)verdict", |l| rx(r"(?i)\bholds\b|not falsified|\bconfirmed\b").replace_all(l, "x").into_owned())));
    ac4_denial_a_holding_verdict_when_the_headers_exceed_the_cap => ("AC4", "the verdict holds, but the mandatory headers", |b| consts(b, |t| replace_all(t, "at most 100 bytes for all headers", "at most 40 bytes for all headers")));
    ac4_denial_a_non_fit_verdict_must_report_falsified => ("AC4", "must report C-137 falsified", |b| consts(b, |t| replace_all(t, "at most 100 bytes for all headers", "at most 40 bytes for all headers")));
    ac4_denial_a_falsified_verdict_without_five_channels => ("AC4", "must say C-04 falls to 5 channels", |b| consts(&size(b, |t| rx(r"(?i)5 channels|five channels").replace_all(t, "fewer channels").into_owned()), |t| replace_all(t, "at most 100 bytes for all headers", "at most 40 bytes for all headers")));
    ac4_denial_a_falsified_verdict_needs_c04_at_five => ("AC4", "needs the constants table to show C-04 at 5 channels", |b| consts(b, |t| replace_all(t, "at most 100 bytes for all headers", "at most 40 bytes for all headers")));

    // ---- AC5: the components from wire.toml
    ac5_denial_the_toml_lacks_what_the_budget_needs => ("AC5", "does not declare what the budget needs", |b| toml(b, |t| replace_all(t, "name = \"announcement_count\"", "name = \"announcement_cnt\"")));
    ac5_denial_the_checksum_max_size_differs_from_its_field => ("AC5", "max_size does not equal its field width", |b| toml(b, |t| after(t, "name = \"slice checksum\"", "max_size = 4", "max_size = 5")));
    ac5_denial_the_audio_sample_differs_from_c03 => ("AC5", "audio sample is 4 bytes, C-03 says 3", |b| consts(b, |t| replace_in_line(t, r"^\| C-03 \|", "4 bytes per sample", "3 bytes per sample")));
    ac5_denial_the_announcement_count_is_not_one_byte => ("AC5", "announcement count is not one byte", |b| toml(b, |t| after(t, "name = \"announcement_count\"", "width = 8", "width = 16")));
    ac5_denial_the_extension_does_not_start_where_the_container_ends => ("AC5", "does not start where the maximal container ends", |b| toml(b, |t| after(t, "name = \"announcement_entry\"", "offset = ", "offset = 8")));
    ac5_denial_the_media_packet_max_size_is_not_the_sum => ("AC5", "'media packet' max_size is not header + slice", |b| toml(b, |t| after(t, "name = \"media packet\"", "max_size = 96", "max_size = 97")));

    // ---- AC5: the totals and the toml budget keys
    ac5_denial_a_total_that_is_not_the_sum_of_its_rows => ("AC5", "the parts sum to 1198, its total row says 1199", |b| size(b, |t| replace_in_line(t, r"^\| \*\*Total\*\* \| 1 198", "1 198", "1 199")));
    ac5_denial_no_budget_key_for_a_case => ("AC5", "no wire.toml budget key for the default case", |b| toml(b, |t| drop_line(t, r"^default_total = ")));
    ac5_denial_a_budget_key_that_is_not_the_total => ("AC5", "wire.toml key \"default_total\" = 1199", |b| toml(b, |t| replace_all(t, "default_total = 1198", "default_total = 1199")));
    ac5_denial_a_budget_key_that_is_not_the_part_size => ("AC5", "the packet header is 16 (landed table)", |b| toml(b, |t| replace_all(t, "packet_header = 16", "packet_header = 17")));
    ac5_denial_an_allowance_key_with_no_row => ("AC5", "has no such row", |b| size(b, |t| drop_line(&drop_line(t, r"^\| 1 472-byte path, a stream that could not"), r"^\| Fallback path, a stream that could not")));
    ac5_denial_an_allowance_key_that_differs_from_its_row => ("AC5", "but the row for could carry the extension is", |b| toml(b, |t| replace_all(t, "shared_allowance_could_carry_extension = 33", "shared_allowance_could_carry_extension = 34")));
    ac5_denial_an_allowance_key_with_no_allowance_table => ("AC5", "has no computed allowance table", |b| size(b, |t| replace_all(t, "Allowance for controller messages", "Quota for controller messages")));

    // ---- AC6
    ac6_denial_no_allowance_table => ("AC6", "no controller allowance table", |b| size(b, |t| replace_all(t, "Allowance for controller messages", "Quota for controller messages")));
    ac6_denial_an_allowance_table_without_a_column => ("AC6", "needs columns for the bound", |b| size(b, |t| replace_in_line(t, r"^\| Path \| Bound", r"\| Checksum \|", "| Tail |")));
    ac6_denial_a_row_naming_neither_path => ("AC6", "names neither the 1 472 path nor the fallback", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", r"1 472-byte path \(C-100\)", "Other path (C-100)")));
    ac6_denial_a_row_with_a_missing_number => ("AC6", "has a missing number", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", "100 − 63 − 4 = 33", "tbd")));
    ac6_denial_a_bound_that_is_neither_the_cap_nor_a_payload => ("AC6", "is neither the cap 100 nor a path payload", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", r"\| 100 \| 1 152", "| 99 | 1 152")));
    ac6_denial_the_1472_path_with_the_fallback_payload => ("AC6", "the 1 472 path takes the cap or 1472", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", r"\| 100 \| 1 152", "| 1 252 | 1 152")));
    ac6_denial_the_fallback_with_the_default_payload => ("AC6", "the fallback takes the cap or 1252", |b| size(b, |t| replace_in_line(t, r"^\| Fallback path \(1 280", r"\| 100 \| 1 152", "| 1 472 | 1 152")));
    ac6_denial_an_audio_that_differs => ("AC6", "audio 1151 != 1152", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", r"\| 1 152 \|", "| 1 151 |")));
    ac6_denial_a_checksum_that_differs => ("AC6", "checksum 5 != 4 (toml)", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", r"= 63 \| 4 \|", "= 63 | 5 |")));
    ac6_denial_counted_headers_that_are_not_a_landed_sum => ("AC6", "counted headers 62 are not the mandatory parts", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", "= 63", "= 62")));
    ac6_denial_an_allowance_that_is_not_the_bound_less_the_parts => ("AC6", "the allowance 33", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", "100 − 63 − 4 = 33", "100 − 63 − 4 = 34")));
    ac6_denial_a_negative_allowance => ("AC6", "is negative", |b| size(b, |t| replace_in_line(t, r"^\| 1 472-byte path \(C-100\), a stream", "100 − 63 − 4 = 33", "−5")));
    ac6_denial_the_allowance_at_one_path_only => ("AC6", "not stated at both path sizes", |b| size(b, |t| drop_line(&drop_line(&drop_line(&drop_line(t, r"^\| Fallback path \(1 280"), r"^\| Fallback path, a stream"), r"^\| 1 472-byte path, a stream that could not"), r"^\| 1 472-byte path \(C-100\)") ));
    ac6_denial_no_upper_bound_sentence => ("AC6", "the allowance is an upper bound", |b| size(b, |t| rx(r"(?i)upper bound|most a source may share").replace_all(t, "x").into_owned()));
    ac6_denial_no_second_stream_sentence => ("AC6", "a source may always use a second stream (C-181)", |b| size(b, |t| rx(r"(?i)second stream").replace_all(t, "other stream").into_owned()));
    ac6_denial_no_audio_member_sentence => ("AC6", "audio member counts no controller messages (C-205)", |b| size(b, |t| replace_all(t, "C-205", "C-2O5")));
    ac6_denial_no_sparse_entries_sentence => ("AC6", "no sentence gives the sparse entries", |b| size(b, |t| rx(r"(?i)transport").replace_all(t, "carriage").into_owned()));

    // ---- DoD
    dod_denial_an_untagged_paragraph => ("DoD", "has no evidence tag", |b| size(b, |t| strip_tags(t, "^The parts and their sizes are those")));
    dod_denial_an_open_item_without_an_owner => ("DoD", "names no owner or no falsifier", |b| size(b, |t| append(t, "**Open item (x).** Whether the budget holds is open `[decision]` until a later ticket.")));

    // ---- pointers
    pointer_denial_an_open_item_still_owned_by_004_14 => ("pointer", "'open item owned by 004-14' remains", |b| wire(b, |t| append(t, "The open item owned by 004-14 remains.")));
    pointer_denial_a_but_see_budget_pointer => ("pointer", "'But see ... Budget' pointer remains", |b| wire(b, |t| append(t, "(But see Content messages, Budget, for the count.)")));
    pointer_denial_an_unresolved_pointer_to_004_14 => ("pointer", "unresolved pointer to 004-14 (Budget)", |b| wire(b, |t| append(t, "An open item for 004-14 remains here.")));
    pointer_denial_a_provisional_cap_line => ("pointer", "a 'provisional' cap or shared-packet line remains", |b| wire(b, |t| append(t, "The shared-packet cap row is provisional.")));
    pointer_denial_a_landed_sentence_not_cited => ("pointer", "is neither removed nor cited to the Size budget section", |b| wire(b, |t| append(t, "All four announcements fit inside the 100-byte cap.")));
    pointer_denial_the_header_cap_open_item_not_closed => ("pointer", "is not marked closed or resolved", |b| wire(b, |t| map_lines(t, r"^\*\*Open item \(header cap\)", |l| rx(r"(?i)\b(closed|resolved|answered)\b").replace_all(l, "x").into_owned())));
}

/// Every denial is rejected by the old script, except the ones listed here:
/// each is a denial the script accepts although it breaks an assertion of
/// the script's own header ("script wrong"); the ignored test below finds
/// them.
const SCRIPT_ACCEPTS: &[&str] = &[];

/// Runs every denial fixture against the old script (needs Python and the
/// script, from the working tree or from the history after its deletion):
/// `cargo test -p hadronyche-tools --test doc_004_14 -- --ignored`.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees_on_the_denials() {
    let all = all_denials();
    let accepted = wp9_support::script_accepts("004-14", &all);
    assert_eq!(accepted, SCRIPT_ACCEPTS, "the script accepts these denials");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (4ae8912, "failing checks
/// for the media packet size budget") fails the same criteria as the
/// script: AC1 to AC4, AC6, DoD and the pointers. Skipped when the commit is
/// not in the clone.
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    let commit = "4ae89126ca507ec558c1453671efba975e9398bc";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "AC6", "DoD", "pointer"],
        "{}",
        r.render()
    );
}

#[test]
fn a_denial_needs_the_right_message() {
    // The fixture helpers are strict: a pattern that no line matches panics
    // instead of leaving the documents unchanged.
    let caught = std::panic::catch_unwind(|| drop_line("a\nb", "^c$"));
    assert!(caught.is_err());
    assert_fails(&run(&base().without(WIRE_MD)), "setup", "cannot read");
}
