//! Ticket 004-29 — the commissioning window announcement and close
//! messages. Port of `checks/004-29.sh` (deleted with this port;
//! `doccheck-equiv 004-29` reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_29`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, each with the exact set of criteria the port
//! and the old script fail on it. `denials_agree_with_the_script` (ignored
//! by default: it needs Python and the script from git history) runs every
//! denial through the harness sandbox against the old script.
//!
//! Not here, because they run tools or inspect the repository (they are in
//! `scripts/repo-state.d/004-29.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC5); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed, the count at the merged
//! text (AC5); the checker names no ref containing main and runs no other
//! check (DoD).
//!
//! Header and code: the header's AC4 says the control violations "are not
//! ignored"; the code's non-rejection pattern is `not a rejection|never a
//! rejection|\bignored\b|no reason` (without 004-24's `\bvalid\b`), and an
//! outcome "ignore and count under UnconfiguredDomain" is not matched by
//! `\bignored\b`; the port follows the code. The placeholder of the section
//! is reported under DoD, as the code does (the header lists it under the
//! author contract).

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_29::{SPEC, WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::equiv::script::{Sandbox, script_source};
use hadronyche_tools::doccheck::mutate::{
    delete_row, delete_unit, rename_heading, replace_all, replace_in_line, strip_tags,
    within_section,
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

/// The section of this ticket.
fn cw(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Commissioning window messages$", edit)
}

/// A `###` subsection of the section (the doc has other "Receiver rules"
/// and "Validity rules").
fn cw_sub(heading: &'static str, edit: impl FnOnce(&str) -> String) -> Docs {
    cw(|t| within_section(t, heading, edit))
}

fn ann(edit: impl FnOnce(&str) -> String) -> Docs {
    cw_sub("^Window announcement$", edit)
}

fn clo(edit: impl FnOnce(&str) -> String) -> Docs {
    cw_sub("^Window close$", edit)
}

fn rul(edit: impl FnOnce(&str) -> String) -> Docs {
    cw_sub("^Receiver rules$", edit)
}

fn val(edit: impl FnOnce(&str) -> String) -> Docs {
    cw_sub("^Validity rules$", edit)
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

const EXP_ROW: &str = r"^\| 48 \| 8 \| u64 \| Expiry time";
const TOOL_ROW: &str = r"^\| 40 \| 8 \| u64 \| Tool identity";
const ANN_WID: &str = r"^\| 32 \| 8 \| u64 \| Window identifier: drawn";
const CLO_WID: &str =
    r"^\| 32 \| 8 \| u64 \| Window identifier: the identifier of the window this close";
const ANN_HEAD: &str = "name = \"commissioning window announcement\"\nlayer = \"control\"\ndscp = 40\nmax_size = 56\nrejection_reasons = [\"TooShort\", \"BadMagic\", \"VersionMismatch\", \"BadLength\", \"UnconfiguredDomain\", \"ForeignDomain\", \"OversizeControl\", \"ReservedNonZero\"]";
/// `from` replaced by `to` on every line that is not a table row.
fn prose_only(text: &str, from: &str, to: &str) -> String {
    text.split('\n')
        .map(|l| {
            if l.trim_start().starts_with('|') {
                l.to_string()
            } else {
                l.replace(from, to)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The announcement's expiry made a relative duration (the branch b of
/// Open item 11), with a tagged sentence that says so.
fn relative(t: &str) -> String {
    let t = every_in_line(
        t,
        EXP_ROW,
        "the absolute instant at which the window closes",
        "the relative duration after which the window closes",
    );
    replace_in_line(
        &t,
        r"^\*\*Expiry time\.\*\*",
        "The expiry is an absolute instant",
        "The expiry is a relative duration",
    )
}

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

/// AC1 — "Announcement and close each have a type code, a field table and a
/// maximum size ≤ 1 252 bytes (epic AC-3)." Script: one subsection each;
/// a field table starting with the Hello's common control header, tiling,
/// its size stated and within the bound, a Message type code in this
/// ticket's block, distinct; the window identifier, the tool identity, the
/// close's single identifier.
#[test]
fn ac1_both_messages_have_a_code_a_table_and_a_size() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The expiry time has a width, a unit and a clock (epic AC-8,
/// C-153)." Script: widths equal to the merged stamp's, a unit, one form,
/// the clock named from timeline.md's terms (network time), and for a
/// relative form its origin, its repeat rule and the merged stamp updated.
#[test]
fn ac2_the_expiry_has_a_width_a_unit_and_a_clock() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The open, expiry and explicit-close transitions of C-153 appear
/// as receiver rules." Script: three different numbered rules; the second
/// announcement's outcome; reviews F2 and F7.
#[test]
fn ac3_the_three_transitions_are_receiver_rules() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "At least 6 invalid forms with outcomes across the two messages."
/// Script: the control violations name their merged reason and are not
/// ignored; the three cases of the ticket; review F3.
#[test]
fn ac4_at_least_six_invalid_forms_with_outcomes() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "Passes the 004-02 checker." Script (document part): one message
/// per table in wire.toml, control at DSCP 40 with the table's size, a
/// non-empty list of control reasons, fields row for row.
#[test]
fn ac5_wire_toml_agrees_with_the_tables() {
    assert_criterion(report(), "AC5");
}

/// DoD — evidence tags (policy `PROSE_004_29`), open items with owner and
/// falsifier (`OpenItemPolicy::Leading`), merged tables unchanged, reserved
/// fields recorded, Open item 11 decided, review F1.
#[test]
fn dod_tags_open_items_merged_tables_and_open_item_11() {
    assert_criterion(report(), "DoD");
}

// ---------------------------------------------------------------- denials

denials! {
    denial_wire_md_missing: ["AC1", "AC2", "AC3", "AC4", "DoD"], "AC1", "does not exist", base().without(WIRE_MD);
    denial_wire_toml_missing: ["AC5"], "AC5", "does not exist", base().without(WIRE_TOML);
    ac1_denial_the_control_bound_constant: ["AC1"], "AC1", "the control bound (C-17) as 1 252", consts(|t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 300"));
    ac1_denial_no_control_bound_where_the_script_stops: ["AC1"], "AC1", "the script stops here with a Python exception", consts(|t| delete_row(t, r"^\| C-17 \|"));
    ac2_denial_the_control_dscp_constant: ["AC2", "AC5"], "AC2", "control CS5 as 40", consts(|t| replace_in_line(t, r"^\| C-13 \|", r"control CS5 \(40\)", "control CS5 (41)"));
    dod_denial_the_hello_header_changed: ["AC1", "DoD"], "DoD", "the merged common control header of the Hello section is unchanged", sub("^Common control header$", |t| delete_row(t, r"^\| 8 \| 8 \| u64 \| Message counter"));
    dod_denial_the_code_column_changed: ["DoD"], "DoD", "keep their code column", sub("^Control message type codes$", |t| delete_row(t, r"^\| 52 to 255 \|"));
    ac1_denial_no_code_block: ["AC1"], "AC1", "give this ticket a block", sub("^Control message type codes$", |t| replace_in_line(t, r"^\| 48 to 51 \|", "004-29", "004-99"));
    ac2_denial_the_stamp_expiry_renamed: ["AC2"], "AC2", "has one window identifier row and one expiry row", sub("^Commissioning window section$", |t| replace_in_line(t, r"^\| 8 \| 8 \| u64 \| Expiry time", "Expiry time", "End time"));
    dod_denial_the_enumeration_short: ["DoD"], "DoD", "keeps its 13 members", wire(|t| delete_row(t, r"^\| 9 \| BadPacketIndex \|"));
    dod_denial_the_placeholder: ["DoD"], "DoD", "placeholder replaced", cw(|t| replace_in_line(t, "^The commissioning window is how", "^", "To be filled. "));
    ac1_denial_no_announcement_subsection: ["AC1", "DoD"], "AC1", "names the announcement (found 0)", wire(|t| rename_heading(t, "^Window announcement$", "Window opening"));
    ac1_denial_no_close_subsection: ["AC1", "DoD"], "AC1", "names the close (found 0)", wire(|t| rename_heading(t, "^Window close$", "Window shutdown"));
    ac3_denial_no_receiver_rules_subsection: ["AC3", "DoD"], "AC3", "holds the receiver rules (found 0)", cw(|t| rename_heading(t, "^Receiver rules$", "Behaviour"));
    ac1_denial_no_announcement_table: ["AC1"], "AC1", "the announcement has no field table", ann(|t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"));
    ac1_denial_a_malformed_row: ["AC1", "AC5"], "AC1", "announcement: the Width cell", ann(|t| replace_in_line(t, TOOL_ROW, r"^\| 40 \| 8 \|", "| 40 | eight |"));
    ac1_denial_the_header_rows_changed: ["AC1", "AC5"], "AC1", "starts with the seven rows of the common control header", ann(|t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"^\| 7 \| 1 \|", "| 7 | 2 |"));
    ac1_denial_rows_that_do_not_tile: ["AC1", "AC5"], "AC1", "tiles from byte 0 with no gap or overlap", ann(|t| replace_in_line(t, TOOL_ROW, r"^\| 40 ", "| 41 "));
    ac1_denial_the_size_not_stated: ["AC1"], "AC1", "states its maximum size as its table sum, 56 bytes", ann(|t| replace_all(t, "is 56 bytes", "is 56"));
    ac1_denial_a_code_outside_the_block: ["AC1"], "AC1", "the announcement has a type code", ann(|t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 48", "48 only", "52 only"));
    ac1_denial_two_messages_one_code: ["AC1"], "AC1", "distinct type codes", clo(|t| replace_in_line(t, r"^\| 6 \| 1 \| u8 \| Message type: 49", "49 only", "48 only"));
    ac1_denial_no_window_identifier_row: ["AC1", "AC2"], "AC1", "the announcement has one window identifier row", ann(|t| replace_in_line(t, ANN_WID, "Window identifier: drawn", "Window key: drawn"));
    ac1_denial_no_tool_identity_row: ["AC1"], "AC1", "the announcing tool's identity", ann(|t| every_in_line(t, TOOL_ROW, "(?i)identit", "nam"));
    ac1_denial_the_identifier_not_stable: ["AC1"], "AC1", "stable and unique", ann(|t| replace_all(t, "(?i)unique", "distinct"));
    ac2_denial_a_window_identifier_off_the_stamp_width: ["AC1", "AC2", "AC5"], "AC2", "the announcement's window identifier is as wide as the merged stamp's", ann(|t| replace_in_line(t, ANN_WID, r"\| 8 \| u64", "| 4 | u32"));
    ac2_denial_an_expiry_off_the_stamp_width: ["AC1", "AC2", "AC5"], "AC2", "the announcement's expiry is as wide as the merged stamp's expiry", ann(|t| replace_in_line(t, EXP_ROW, r"\| 8 \| u64", "| 4 | u32"));
    ac2_denial_no_clock: ["AC2"], "AC2", "the announcement names its clock", ann(|t| replace_all(t, "(?i)network time", "the reference"));
    ac2_denial_the_expiry_without_a_unit: ["AC2"], "AC2", "the expiry row states its unit", ann(|t| every_in_line(t, EXP_ROW, r"(?i)nanosecond|\bns\b|\bsecond", "tick"));
    ac2_denial_the_expiry_in_two_forms: ["AC2"], "AC2", "states one form", ann(|t| replace_in_line(t, EXP_ROW, "the absolute instant", "the absolute instant, a relative duration,"));
    ac2_denial_the_form_not_in_a_tagged_sentence: ["AC2"], "AC2", "a tagged sentence of the announcement states the expiry form", ann(|t| prose_only(t, "absolute", "fixed"));
    ac2_denial_a_relative_expiry_without_its_origin: ["AC2"], "AC2", "states the instant it counts from", ann(|t| replace_all(&relative(t), "(?i)counted from|counts from|from the issue time|from the instant", "x"));
    ac2_denial_a_relative_expiry_without_the_repeat_rule: ["AC2"], "AC2", "how a late or repeated announcement derives the same expiry", ann(|t| replace_all(&relative(t), "(?i)same expiry|derives? the same", "x"));
    ac2_denial_a_relative_expiry_copied_verbatim: ["AC2"], "AC2", "no longer says the stamped expiry is copied", ann(relative);
    dod_denial_a_relative_expiry_without_the_status: ["AC2", "DoD"], "DoD", "carries a status sentence for the relative form", wire(|t| replace_in_line(&within_section(t, "^Window announcement$", relative), r"^\*\*Open item 11 — ", "Status, 2026-11-16", "Note"));
    ac1_denial_no_close_identifier: ["AC1", "AC2"], "AC1", "the close names the window it closes", clo(|t| replace_in_line(t, CLO_WID, "Window identifier: the identifier", "Window key: the key"));
    ac2_denial_the_close_identifier_off_the_stamp_width: ["AC1", "AC2", "AC5"], "AC2", "the close's window identifier is as wide", clo(|t| replace_in_line(t, CLO_WID, r"\| 8 \| u64", "| 4 | u32"));
    ac1_denial_the_close_with_a_second_field: ["AC1", "AC2", "AC5", "DoD"], "AC1", "the close lays out only the identifier", clo(|t| replace_in_line(t, CLO_WID, r"^\| 32 \| 8 \| u64 \| Window identifier: the identifier of the window this close ends, as its announcement carries it", "| 32 | 4 | u32 | Window identifier: the identifier of the window this close ends | any |\n| 36 | 4 | u32 | Spare, 0 only"));
    ac4_denial_fewer_than_six_forms: ["AC4"], "AC4", "at least 6 invalid forms", val(|t| replace_in_line(t, r"^\| a datagram of 4 to 31 bytes", "$", "\n"));
    ac4_denial_a_reason_outside_the_control_layer: ["AC4"], "AC4", "names a reason outside the control layer", val(|t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadChecksum"));
    ac4_denial_a_violation_without_its_reason: ["AC4"], "AC4", "must name BadMagic", val(|t| replace_in_line(t, r"^\| bad magic", "BadMagic", "BadLength"));
    ac4_denial_a_violation_marked_ignored: ["AC4"], "AC4", "is marked ignored or not a rejection", val(|t| replace_in_line(t, r"^\| foreign domain:", r"\|$", "; ignored |"));
    ac4_denial_the_version_mismatch_without_its_event: ["AC4"], "AC4", "does not raise the named VersionMismatch event", val(|t| every_in_line(t, r"^\| version mismatch", "(?i)event", "signal"));
    ac4_denial_a_form_without_an_outcome: ["AC4"], "AC4", "states no outcome", val(|t| replace_in_line(t, r"^\| window identifier 0", r"\| drop and count under BadLength.*$", "| dropped |"));
    ac4_denial_no_expiry_out_of_range_form: ["AC4"], "AC4", "covers the expiry out of range", val(|t| every_in_line(t, r"^\| expiry out of range", "(?i)out of range", "beyond limits"));
    ac4_denial_no_expiry_past_form: ["AC4"], "AC4", "covers an expiry already past", val(|t| every_in_line(t, r"^\| an announcement whose expiry is already past", "(?i)already past", "gone"));
    ac4_denial_no_unknown_close_form: ["AC4"], "AC4", "covers a close for an unknown window", val(|t| every_in_line(t, r"^\| a close for an unknown window", "(?i)unknown", "foreign"));
    ac3_denial_no_second_announcement_outcome: ["AC3"], "AC3", "a second announcement while a window is open has a stated outcome", cw(|t| replace_all(t, "(?i)second|another announcement|already open|while (a |the )?window is open", "x"));
    ac3_denial_no_open_rule: ["AC3"], "AC3", "a numbered rule for the open transition", rul(|t| replace_in_line(t, r"^1\. \*\*Open\.\*\*", r"\*\*Open\.\*\*", "**Start.**"));
    ac3_denial_no_expiry_rule: ["AC3"], "AC3", "a numbered rule for the expiry transition", rul(|t| replace_in_line(t, r"^2\. \*\*Expiry\.\*\*", r"\*\*Expiry\.\*\*", "**Timeout.**"));
    ac3_denial_no_explicit_close_rule: ["AC3"], "AC3", "a numbered rule for the explicit close", rul(|t| replace_in_line(t, r"^3\. \*\*Explicit close\.\*\*", r"\*\*Explicit close\.\*\*", "**Termination.**"));
    ac3_denial_two_transitions_in_one_rule: ["AC3"], "AC3", "three different receiver rules", rul(|t| replace_in_line(&delete_unit(t, r"^3\. \*\*Explicit close\.\*\*"), r"^2\. \*\*Expiry\.\*\*", r"\*\*Expiry\.\*\* A module that can evaluate the expiry closes its window when the expiry is reached", "**Expiry and explicit close.** A module that can evaluate the expiry closes its window when the expiry is reached or on a close message"));
    dod_denial_an_untagged_subsection: ["DoD"], "DoD", "the close subsection carries an evidence tag", clo(|t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""));
    dod_denial_an_untagged_paragraph: ["DoD"], "DoD", "every prose paragraph of 5 or more words in the section", ann(|t| strip_tags(t, r"^\*\*Tool identity\.\*\*"));
    dod_denial_an_open_item_without_an_owner: ["DoD"], "DoD", "every open item of the section names an owner and a falsifier", cw(|t| replace_in_line(t, r"^\*\*Open item 1 —", "^", "**Open item 11 — spare.** Nothing is decided here `[decision]`.\n\n"));
    dod_denial_a_reserved_field_not_recorded: ["DoD"], "DoD", "every reserved or accept-any field of the new messages", ann(|t| replace_in_line(t, TOOL_ROW, "any 64-bit value", "any 64-bit value, accept-any"));
    dod_denial_open_item_11_still_open: ["DoD"], "DoD", "Open item 11 of the regime stamp is no longer open", wire(|t| replace_in_line(t, r"^\*\*Open item 11 — ", "Status, 2026-11-16", "Note"));
    ac4_denial_no_outcome_after_an_explicit_close: ["AC4"], "AC4", "closed by an explicit close has a stated outcome", cw(|t| replace_all(t, r"(?i)explicitly closed|closed by|already closed|closed window|had closed|after (?:the |an |its |a )?(?:window'?s? )?(?:explicit )?close\b", "x"));
    dod_denial_a_reopening_without_an_open_item: ["DoD"], "DoD", "the reopening outcome after a close is an open item", cw(|t| within_section(&within_section(t, "^Open items$", |o| replace_all(o, "(?i)reopen|late|after|repeat", "x")), "^Receiver rules$", |r| replace_in_line(r, r"^\*\*Injections\.\*\*", "^", "An announcement after the explicit close reopens the window `[decision]`.\n\n")));
    ac3_denial_two_windows_not_stated: ["AC3"], "AC3", "two modules can hold different windows at once", cw(|t| replace_all(t, "(?i)at once|different|same time", "x"));
    ac3_denial_a_lost_close_not_stated: ["AC3"], "AC3", "whose close was lost is stated", cw(|t| replace_all(&replace_all(t, r"(?i)\blost\b", "missed"), r"(?i)\bnever\b", "not"));
    dod_denial_a_bare_liveness_denial: ["DoD"], "DoD", "a denial that the tool's message counter is a liveness counter", rul(|t| replace_in_line(t, r"^\*\*Injections\.\*\*", "^", "The tool's counter cannot be a liveness counter `[decision]`.\n\n"));
    dod_denial_a_section_type_without_the_cap: ["DoD"], "DoD", "restates the connection-cap threshold", rul(|t| replace_in_line(t, r"^\*\*Injections\.\*\*", "^", "The window is not a section type 9 `[decision]`.\n\n"));
    ac5_denial_wire_toml_does_not_parse: ["AC5"], "AC5", "does not parse", toml(|t| format!("{t}\n[[[broken"));
    ac5_denial_a_multi_line_string_the_reader_rejects: ["AC5"], "AC5", "does not parse", toml(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n"));
    ac5_denial_no_announcement_message: ["AC5"], "AC5", "exactly one commissioning window announcement message (found 0)", toml_swap("name = \"commissioning window announcement\"", "name = \"commissioning window opening\"");
    ac5_denial_a_max_size_off_the_table: ["AC5"], "AC5", "the announcement message of wire.toml is control", toml_swap(ANN_HEAD, &ANN_HEAD.replace("max_size = 56", "max_size = 57"));
    ac5_denial_no_reasons: ["AC5"], "AC5", "the announcement message names only control reasons", toml_swap(ANN_HEAD, "name = \"commissioning window announcement\"\nlayer = \"control\"\ndscp = 40\nmax_size = 56\nrejection_reasons = []");
    ac5_denial_fields_that_disagree: ["AC5"], "AC5", "the close fields of wire.toml agree", toml(|t| { let at = t.find("name = \"commissioning window close\"").expect("close"); let (a, b) = t.split_at(at); format!("{a}{}", b.replacen("width = 64", "width = 32", 1)) });
    ac1_denial_a_heading_after_a_python_line_break: ["AC1", "AC3", "AC4", "DoD"], "AC1", "names the announcement (found 0)", cw(|t| replace_in_line(t, "^The commissioning window is how", "^", "\u{2028}## Interlude\u{2028}"));
    ac1_denial_a_word_character_only_python_knows: ["AC1", "DoD"], "AC1", "names the close (found 0)", wire(|t| rename_heading(t, "^Window close$", "Window close²"));
    ac4_denial_a_validity_table_without_separator: ["AC4"], "AC4", "at least 6 invalid forms", val(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n"));
    accepted_a_separator_row_only_the_scripts_accept: [], "", "", val(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n"));
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

/// The tree at the commit that added the script (0243573, "failing checks
/// for the commissioning window messages") fails the criteria the script
/// failed there. Skipped when the commit is not in the clone.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "0243573af47644bdbf800423bebe9d6a52980d79";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC3", "AC4", "AC5", "DoD"],
        "{}",
        r.render()
    );
}
