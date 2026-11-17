//! Ticket 004-26 — the manifest encoding in the admission handshake. Port
//! of `checks/004-26.sh` (deleted with this port; `doccheck-equiv 004-26`
//! reads it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_26`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory, at least one per
//! assertion of the script, each with the exact set of criteria the port
//! and the old script fail on it. `denials_agree_with_the_script` (ignored
//! by default: it needs Python and the script from git history) runs every
//! denial through the harness sandbox against the old script.
//!
//! Not here, because they run tools or inspect the repository (they are in
//! `scripts/repo-state.d/004-26.sh`): `cargo run --bin wire-check` reports
//! "checks clean" (AC6); `cargo test -p hadronyche-tools --test wire_tables`
//! exits 0 with 0 failed and at least 12 passed (AC6); the checker names no
//! ref containing main and runs no other check (DoD).
//!
//! Assertions no denial can fail (the script's own selection makes them
//! hold, so they are carried by the port and not denied): "the {none,
//! unknown} sentinel N fits the B-bit E field" and "the {none, unknown}
//! sentinel N lies outside its stated E range" — `encoding_of` only takes a
//! sentinel that fits its field and lies outside the row's range; "the
//! maximum manifest size, N bytes, fits the control bound" is a
//! `check(True)`.
//!
//! Header and code: the header's AC5 says "every outcome names a member"
//! of the Reject or refusal enumeration; the code skips the identical
//! repeat row (its outcome names no reason, checked separately), as the
//! port does. The header's DoD says the landed tables are checked "in the
//! way 004-18's check verifies them"; the code compares the first seven
//! rows of four admission tables with the Hello's common control header,
//! as the port does.

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_26::{SPEC, WIRE_MD, run};
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

fn man(edit: impl FnOnce(&str) -> String) -> Docs {
    sub("^Manifest$", edit)
}

/// The `###` section `heading` of the Manifest (the Hello has a Port
/// element and a Bundle element too).
fn man_sub(heading: &'static str, edit: impl FnOnce(&str) -> String) -> Docs {
    man(|t| within_section(t, heading, edit))
}

/// The Manifest's own Validity rules (the first heading of that name after
/// `## Manifest`).
fn man_validity(edit: impl FnOnce(&str) -> String) -> Docs {
    man(|t| within_section(t, "^Validity rules$", edit))
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

const REQ_E: &str = r"^\| 12 \| 4 \| u32 \| Required E";
const DEC_E: &str = r"^\| 16 \| 4 \| u32 \| Declared E";
const REPORT_P: &str = r"^\*\*Falsified: the one-message manifest";
const FIXED: &str = "name = \"manifest fixed part\"\nlayer = \"control\"\ndscp = 40\nmax_size = 31";

/// Panics when a failure of `criterion` contains `needle`.
fn assert_none(r: &Report, criterion: &str, needle: &str) {
    let hits: Vec<String> = r
        .of(criterion)
        .iter()
        .filter(|f| f.message.contains(needle))
        .map(|f| f.to_string())
        .collect();
    assert!(hits.is_empty(), "{}", hits.join("\n"));
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

/// AC1 — "Every item in Context has a field with width and unit."
/// Script: ports, content types, the channel map, L in whole samples, the
/// join-delay count and each join delay with its frequency flag, the dense
/// initial values, the conversion delay of a device-facing port with its
/// unit, the cost bound in operations per slice, required and declared E
/// with the unit of E; the tables model and tile; review points 4, T3 (a)
/// and T3 (f).
#[test]
fn ac1_every_context_item_has_a_field_with_width_and_unit() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "'None' for required E and 'unknown' for declared E have distinct
/// encodings that no valid E value can take." Script: a sentinel outside
/// the stated E range or a flag field, per encoding; distinct; review T3
/// (b), the prose and wire.toml state the table's codes.
#[test]
fn ac2_none_and_unknown_have_distinct_encodings() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "The maximum manifest size is stated in bytes and fits 1 252
/// bytes, or a falsified constant is reported (epic AC-3)." Script: the
/// constants and the landed envelope read; the computed size of the table
/// no counted row references is stated; the sections it needs fit the
/// landed count; above the bound, the report paragraph affirms the
/// falsification of AC-3 and 012-08, states the size, the excess, the
/// design owner and the split (review T1).
#[test]
fn ac3_the_maximum_manifest_size_is_stated_and_reported() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "The join-delay bound is recorded as an open item with owner and
/// falsifier." Script: an open item on the join bound, owned by epic 004,
/// saying the second clause of the falsifier is already met.
#[test]
fn ac4_the_join_delay_bound_is_an_open_item() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "At least 5 invalid forms with outcomes." Script: five rows, each
/// naming a member of the Reject or refusal enumeration, the five
/// categories of Work item 3, the repeat and disagreement rows.
#[test]
fn ac5_at_least_five_invalid_forms_with_outcomes() {
    assert_criterion(report(), "AC5");
}

/// AC6 — "Passes the 004-02 checker." Script (document part): wire.toml
/// parses; each field table within the bound has an agreeing message; the
/// fixed part is declared row for row (review T2). The checker run is in
/// scripts/repo-state.d/004-26.sh.
#[test]
fn ac6_wire_toml_agrees_with_the_tables() {
    assert_criterion(report(), "AC6");
}

/// DoD — "Every new factual claim carries an evidence tag" (policy
/// `PROSE_004_26`, and every subsection with a table) and "No new open item
/// without an owner and a falsifier"; the landed tables unchanged; the
/// review points labelled DoD.
#[test]
fn dod_tags_open_items_landed_tables_and_review_points() {
    assert_criterion(report(), "DoD");
}

/// DoD — "The machine-readable tables parse and round-trip": the document
/// side is AC6's parse.
#[test]
fn dod_the_tables_parse() {
    assert_none(report(), "AC6", "does not parse");
}

// ---------------------------------------------------------------- denials

denials! {
    denial_wire_md_missing: ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"], "AC3", "does not exist", base().without(WIRE_MD);
    denial_wire_toml_missing: ["AC1", "AC6"], "AC6", "does not exist", base().without(WIRE_TOML);
    ac3_denial_the_control_bound_unread: ["AC3"], "AC3", "the control bound (C-17) reads as None",
        consts(|t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most many bytes"));
    dod_denial_the_hello_header_short: ["DoD"], "DoD", "the Hello common control header is readable (seven rows)",
        sub("^Common control header$", |t| delete_row(t, r"^\| 8 \| 8 \| u64 \| Message counter"));
    ac3_denial_the_envelope_unread: ["AC3"], "AC3", "the landed manifest section reads",
        sub("^Manifest section$", |t| replace_in_line(t, r"\| Section count", "1 to 255", "one or more"));
    ac3_denial_the_envelope_maxima_differ: ["AC3"], "AC3", "manifest bytes maximum equals its manifest length maximum",
        sub("^Manifest section$", |t| replace_in_line(t, r"\| Manifest bytes", "max 1196", "max 1195"));
    ac3_denial_the_envelope_off_the_bound: ["AC3"], "AC3", "the landed manifest section computes to",
        consts(|t| replace_in_line(t, r"^\| C-17 \|", "at most 1 252 bytes", "at most 1 300 bytes"));
    dod_denial_an_enumeration_short: ["DoD"], "DoD", "the landed enumerations read as 12 Reject names",
        wire(|t| delete_row(t, r"^\| 9 \| BadPacketIndex \|"));
    ac1_denial_no_manifest_section: ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6"], "AC1", "wire.md has a '## Manifest' section",
        wire(|t| rename_heading(t, "^Manifest$", "Declaration"));
    ac1_denial_no_field_table: ["AC1", "AC2", "AC3", "AC6"], "AC1", "the Manifest section has field tables (0 found)",
        man(|t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"));
    ac1_denial_a_malformed_row: ["AC1", "AC6"], "AC1", "Manifest field table 5: the Width cell",
        man_sub("^Join element$", |t| replace_in_line(t, r"^\| 0 \| 4 \| u32 \| Inserted join delay", r"^\| 0 \| 4 \|", "| 0 | four |"));
    ac1_denial_rows_that_do_not_tile: ["AC1"], "AC1", "does not follow the previous row",
        man_sub("^Bundle element$", |t| replace_in_line(t, r"^\| 1 \| 1 \| u8 \| Member count", r"^\| 1 ", "| 2 "));
    ac1_denial_no_content_type_field: ["AC1", "DoD"], "AC1", "ports and content types have fields",
        man(|t| replace_all(t, "(?i)content[- ]types?", "kinds"));
    ac1_denial_no_channel_map_field: ["AC1"], "AC1", "the channel map has a field",
        man(|t| replace_all(t, "(?i)channel map", "channel table"));
    ac1_denial_l_without_samples: ["AC1"], "AC1", "L has a field whose valid range is in whole samples",
        man(|t| every_in_line(t, r"^\| 0 \| 4 \| u32 \| Structural latency L", "(?i)sample", "unit"));
    ac1_denial_no_counted_join_row: ["AC1"], "AC1", "the join-delay count is a counted row",
        man(|t| every_in_line(t, r"^\| 6438 \| 5 × max 255", "(?i)join", "jn"));
    ac1_denial_join_delays_without_samples: ["AC1"], "AC1", "each inserted join delay is a whole number of samples",
        man(|t| every_in_line(&every_in_line(t, r"^\| 6438 \| 5 × max 255", "(?i)sample", "unit"), r"^\| 0 \| 4 \| u32 \| Inserted join delay", "(?i)sample", "unit"));
    ac1_denial_no_frequency_flag: ["AC1"], "AC1", "the frequency-dependent-delay flag",
        man(|t| replace_all(t, "(?i)frequency", "freq"));
    ac1_denial_no_initial_values: ["AC1"], "AC1", "dense controller initial values",
        man(|t| replace_all(t, "(?i)initial value", "start value"));
    ac1_denial_no_conversion_delay_field: ["AC1"], "AC1", "the conversion delay on a device-facing port (C-207) has a field",
        man_sub("^Port element$", |t| replace_all(t, "(?i)conversion delay", "resampler lag"));
    ac1_denial_the_conversion_delay_without_unit: ["AC1"], "AC1", "the conversion delay states its unit",
        man_sub("^Port element$", |t| replace_in_line(t, r"^\| 1 \| 4 \| u32 \| Conversion delay", r"\| Conversion delay of a device-facing port.*$", "| Conversion delay of a device-facing port (C-207) | any value |"));
    ac1_denial_the_cost_bound_without_operations: ["AC1"], "AC1", "the cost bound (C-106) has a field in operations",
        man(|t| every_in_line(t, r"^\| 4 \| 8 \| u64 \| Cost bound", "(?i)operation", "step"));
    ac1_denial_the_cost_bound_not_per_slice: ["AC1"], "AC1", "the cost bound is per slice",
        man(|t| every_in_line(t, r"^\| 4 \| 8 \| u64 \| Cost bound", "(?i)slice", "frame"));
    ac1_denial_no_required_e: ["AC1", "AC2"], "AC1", "required E per presentation group has a field",
        man(|t| every_in_line(t, REQ_E, "(?i)required", "wanted"));
    ac1_denial_no_declared_e: ["AC1", "AC2"], "AC1", "declared E per sink has a field",
        man(|t| every_in_line(t, DEC_E, "(?i)declared", "stated"));
    ac1_denial_no_unit_of_e: ["AC1"], "AC1", "the unit of E is stated",
        man(|t| every_in_line(&every_in_line(t, REQ_E, r"nanoseconds|\bns\b", "ticks"), DEC_E, r"nanoseconds|\bns\b", "ticks"));
    ac2_denial_a_code_inside_its_range: ["AC2"], "AC2", "lies outside the E range",
        man(|t| replace_in_line(t, REQ_E, "0 to 4 294 967 293", "0 to 4 294 967 295"));
    ac2_denial_no_none_encoding: ["AC2"], "AC2", "the none encoding of required E is stated",
        man(|t| every_in_line(t, REQ_E, r"\bnone\b", "absent"));
    ac2_denial_no_unknown_encoding: ["AC2"], "AC2", "the unknown encoding of declared E is stated",
        man(|t| every_in_line(t, DEC_E, r"\bunknown\b", "absent"));
    ac2_denial_a_flag_without_not_interpreted: ["AC2"], "AC2", "stated as not interpreted when the flag says none",
        man(|t| replace_in_line(t, REQ_E, "0 to 4 294 967 293 ns", "any value below 4 294 967 294 ns"));
    ac2_denial_a_flag_without_its_distinct_sentence: ["AC2", "AC6"], "AC2", "a sentence states that none is distinct",
        man(|t| replace_all(&replace_in_line(t, REQ_E, "0 to 4 294 967 293 ns", "any value below 4 294 967 294 ns"), "(?i)distinct|never|cannot|not a valid|no valid|differ", "odd"));
    ac2_denial_the_two_sentinels_equal: ["AC2"], "AC2", "the none and unknown sentinels are distinct numbers",
        man(|t| replace_in_line(t, DEC_E, "0xfffffffe", "0xffffffff"));
    ac3_denial_the_maximum_not_stated: ["AC3", "AC6"], "AC3", "the maximum manifest size is stated in bytes",
        man(|t| replace_all(t, "18 977 bytes", "18 977 octets"));
    ac3_denial_more_sections_than_the_envelope_allows: ["AC3"], "AC3", "within the landed count maximum",
        sub("^Manifest section$", |t| replace_in_line(t, r"\| Section count", "1 to 255", "1 to 15"));
    ac3_denial_a_denied_falsification: ["AC3"], "AC3", "denies a falsification",
        man(|t| replace_in_line(t, "^Proposed, for epic 005 to confirm", "^", "No constant is falsified here `[decision]`. "));
    ac3_denial_no_falsification_report: ["AC3"], "AC3", "a paragraph reports as falsified",
        man(|t| every_in_line(t, REPORT_P, "012-08", "012-99"));
    ac3_denial_ac3_not_affirmed: ["AC3"], "AC3", "affirmatively falsifies the epic's AC-3",
        man(|t| replace_in_line(&replace_in_line(t, REPORT_P, "this falsifies the design statements", "this questions the design statements"), REPORT_P, r#"\(C-17\)", is falsified for the manifest"#, "(C-17)\", is not falsified for the manifest"));
    ac3_denial_012_08_not_affirmed: ["AC3"], "AC3", "affirmatively falsifies 012-08 AC 2",
        man(|t| replace_in_line(&replace_in_line(t, REPORT_P, "this falsifies the design statements", "this questions the design statements"), REPORT_P, r#"\(C-17\)", is falsified too"#, "(C-17)\", is not falsified too"));
    ac3_denial_the_report_without_the_excess: ["AC3"], "AC3", "and that it exceeds",
        man(|t| every_in_line(t, REPORT_P, "(?i)exceeds", "passes"));
    ac3_denial_the_report_without_the_design_owner: ["AC3"], "AC3", "goes to the design owner",
        man(|t| every_in_line(t, REPORT_P, "(?i)design owner", "lead"));
    ac3_denial_the_report_without_the_split: ["AC3"], "AC3", "proposes the split",
        man(|t| every_in_line(t, REPORT_P, "(?i)split", "division"));
    ac4_denial_the_falsifier_clause_not_met: ["AC4"], "AC4", "already met at 1 252 bytes",
        man(|t| every_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", "(?i)already met", "open"));
    ac4_denial_no_join_open_item: ["AC4"], "AC4", "an open item records the join-delay bound",
        man_sub("^Open items$", |t| replace_all(t, r"(?i)\bjoin", "merge"));
    ac4_denial_the_join_item_not_owned_by_004: ["AC4"], "AC4", "owned by epic 004",
        man(|t| every_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", r"(?i)\b(epic\s*)?004\b", "epic 005"));
    ac5_denial_fewer_than_five_forms: ["AC5"], "AC5", "at least five invalid forms",
        man_validity(|t| replace_in_line(t, r"^\| held manifest bytes above the maximum", "$", "\n"));
    ac5_denial_an_outcome_without_a_member: ["AC5"], "AC5", "names its outcome as a member",
        man_validity(|t| replace_in_line(t, r"^\| frames per slice of 0", "BadLength", "nothing"));
    ac5_denial_no_count_above_form: ["AC5"], "AC5", "a count above its maximum is an invalid form",
        man_validity(|t| replace_all(t, r"(?i)\bcount", "number"));
    ac5_denial_the_length_form_without_bad_length: ["AC5"], "AC5", "a length inconsistent with the sections",
        man_validity(|t| replace_in_line(t, r"^\| length inconsistent", "BadLength", "TooShort"));
    ac5_denial_no_channel_map_form: ["AC5"], "AC5", "a channel-map entry out of range",
        man_validity(|t| replace_all(t, "(?i)channel[- ]map", "channel table"));
    ac5_denial_no_conversion_form: ["AC5"], "AC5", "a conversion delay on a port that is not device-facing",
        man_validity(|t| replace_all(t, "(?i)conversion", "resampling"));
    ac5_denial_no_reserved_form: ["AC5"], "AC5", "a non-zero reserved field",
        man_validity(|t| replace_all(t, "(?i)reserved", "spare"));
    ac5_denial_no_repeat_row: ["AC5"], "AC5", "an identical repeated section has an invalid-form row",
        man_validity(|t| every_in_line(t, r"^\| duplicate section index", "section index", "section place"));
    ac5_denial_the_repeat_a_rejection: ["AC5"], "AC5", "the identical repeated section is not a rejection",
        man_validity(|t| replace_in_line(t, r"^\| duplicate section index", r"\|$", "and BadLength |"));
    ac5_denial_no_manifest_length_disagreement: ["AC5"], "AC5", "disagree on the manifest length have",
        man_validity(|t| every_in_line(t, r"^\| sections disagree on the manifest length", "manifest length", "manifest size"));
    ac5_denial_no_section_count_disagreement: ["AC5"], "AC5", "disagree on the section count have",
        man_validity(|t| every_in_line(t, r"^\| sections disagree on the section count", "section count", "count of sections"));
    ac5_denial_a_disagreement_naming_two_reasons: ["AC5"], "AC5", "names exactly one Reject reason",
        man_validity(|t| replace_in_line(t, r"^\| sections disagree on the section count", r"\|$", "and ForeignDomain |"));
    dod_denial_an_untagged_paragraph: ["DoD"], "DoD", "carries an evidence tag",
        man(|t| strip_tags(t, "^The conversion delay's unit is whole samples"));
    dod_denial_an_untagged_subsection_with_a_table: ["DoD"], "DoD", "subsection 'Join element' with a table",
        man_sub("^Join element$", |t| replace_all(t, r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?", ""));
    dod_denial_an_open_item_without_an_owner: ["DoD"], "DoD", "names an owner and a falsifier",
        man(|t| replace_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", "^", "Open item 10: the spare bits are unassigned `[decision]`.\n\n"));
    dod_denial_a_major_change_sentence_without_the_exception: ["DoD"], "DoD", "excepts a new content-type code",
        man(|t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "Every addition to the manifest is a major change `[decision]`. "));
    dod_denial_a_measured_maximum: ["DoD"], "DoD", "no stated maximum carries the [measured] tag",
        man(|t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "The maximum is 51 ports `[measured]`. "));
    dod_denial_a_chosen_number_without_decision: ["DoD"], "DoD", "chosen or not derived is tagged [decision]",
        man(|t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "The bound of 9 is chosen, at most 9 `[code]`. "));
    dod_denial_a_reserved_field_not_recorded: ["DoD"], "DoD", "has a row in the landed Reserved fields table",
        wire(|t| delete_row(t, r"^\| Manifest join element bits 7..1 of byte 4"));
    dod_denial_a_wait_without_bad_member_list: ["DoD"], "DoD", "names BadMemberList as waiting",
        man(|t| replace_in_line(t, "^A stream element is 15 bytes, fixed", "^", "A receiver waits for every section `[decision]`. "));
    dod_denial_a_landed_header_changed: ["DoD"], "DoD", "landed 'Admission request' header rows equal",
        sub("^Admission request$", |t| replace_in_line(t, r"^\| 7 \| 1 \| u8 \| Authentication scheme", r"\| u8 \|", "| i8 |"));
    dod_denial_a_landed_table_missing: ["DoD"], "DoD", "landed 'Admission accept' table is present",
        wire(|t| rename_heading(t, "^Admission accept$", "Admission yes"));
    dod_denial_a_landed_table_without_fields: ["DoD"], "DoD", "landed 'Admission refusal' has a field table",
        sub("^Admission refusal$", |t| replace_all(t, r"\| Offset \(bytes\) \|", "| At |"));
    ac6_denial_wire_toml_does_not_parse: ["AC1", "AC6"], "AC6", "does not parse",
        toml(|t| format!("{t}\n[[[broken"));
    ac6_denial_a_multi_line_string_the_reader_rejects: ["AC1", "AC6"], "AC6", "does not parse",
        toml(|t| format!("{t}\n[[message]]\nname = \"\"\"spare\"\"\"\n"));
    ac6_denial_no_owner_for_the_tooling_gap: ["AC6"], "AC6", "wire.toml cannot declare it as one message",
        man(|t| replace_all(t, r"(?i)wire-check|schema|checker", "tool"));
    ac6_denial_a_table_without_its_message: ["AC6"], "AC6", "Manifest field table 5 has a [[message]]",
        toml_swap("name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 5\nrejection_reasons = [\"BadLength\", \"ReservedNonZero\"]\n\n[[message.field]]\nname = \"inserted_join_delay\"\noffset = 0\nwidth = 32", "name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 5\nrejection_reasons = [\"BadLength\", \"ReservedNonZero\"]\n\n[[message.field]]\nname = \"inserted_join_delay\"\noffset = 0\nwidth = 24");
    ac6_denial_a_message_with_the_wrong_size: ["AC6"], "AC6", "Manifest field table 5: a matching toml message",
        toml_swap("name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 5", "name = \"manifest join element\"\nlayer = \"control\"\ndscp = 40\nmax_size = 6");
    ac6_denial_no_fixed_part_message: ["AC6"], "AC6", "declares the manifest fixed part as a [[message]]",
        toml_swap("name = \"required_e\"\noffset = 96\nwidth = 32", "name = \"required_e\"\noffset = 96\nwidth = 31");
    ac6_denial_the_fixed_part_of_the_wrong_size: ["AC6"], "AC6", "the fixed-part message has max_size equal",
        toml_swap(FIXED, &FIXED.replace("max_size = 31", "max_size = 32"));
    ac1_denial_wire_toml_without_the_audio_bound: ["AC1"], "AC1", "states the audio-length bound (65 535)",
        toml_swap("with frames per slice x channel count x 4 at most 65 535", "with frames per slice x channel count x 4 bounded");
    ac1_denial_an_l_that_saturates: ["AC1"], "AC1", "times the landed maximum hop count",
        man(|t| replace_in_line(t, r"^\| 0 \| 4 \| u32 \| Structural latency L", "67 108 863", "99 108 863"));
    ac1_denial_the_hop_count_unreadable: ["AC1"], "AC1", "are all readable",
        wire(|t| replace_all(t, "maximum hop count of", "hop limit of"));
    ac1_denial_no_audio_length_bound: ["AC1"], "AC1", "the landed audio length bound",
        man(|t| replace_all(t, "65 535", "sixty-five thousand"));
    ac2_denial_a_stray_prose_sentinel: ["AC2"], "AC2", "the prose sentinel for none agrees",
        man(|t| replace_in_line(t, "^The unit of E is the nanosecond", "^", "The none code of E is not 0x1234 `[decision]`. "));
    ac2_denial_wire_toml_states_another_sentinel: ["AC2"], "AC2", "wire.toml's valid range for the none field",
        toml_swap("0xffffffff = none: the group sets no required E", "0xfffffff0 = none: the group sets no required E");
    ac4_denial_a_heading_after_a_python_line_break: ["AC4"], "AC4", "an open item records the join-delay bound",
        man(|t| replace_in_line(t, r"^\*\*Open item 1 — the bound on inserted join", "^", "\u{2028}## Interlude\u{2028}"));
    ac2_denial_a_word_character_only_python_knows: ["AC2"], "AC2", "the unknown encoding of declared E is stated",
        man(|t| every_in_line(t, DEC_E, r"\bunknown\b", "unknown²"));
    ac5_denial_a_validity_table_without_separator: ["AC5"], "AC5", "at least five invalid forms",
        man_validity(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n"));
    accepted_a_separator_row_only_the_scripts_accept: [], "", "",
        man_validity(|t| replace_all(t, r"\| Invalid form \| Required outcome \|\n\|---\|---\|\n", "| Invalid form | Required outcome |\n| - - | :-: |\n"));
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

/// The tree at the commit that added the script (9b170b2, "failing checks
/// for the manifest encoding") fails every criterion, as the script did.
/// Skipped when the commit is not in the clone.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "9b170b2731f399aba01666587ff1c6995038686f";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"],
        "{}",
        r.render()
    );
}
