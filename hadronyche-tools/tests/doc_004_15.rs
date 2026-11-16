//! Ticket 004-15 — recompute capacity figures from the final layout. Port of
//! `checks/004-15.sh` (deleted with this port; `doccheck-equiv 004-15` reads
//! it from git history).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_15`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion and definition-of-done item) and on denial fixtures: the real
//! documents with one mutation applied in memory. Every denial is also run
//! against the old script by the ignored test
//! `old_script_agrees_on_the_denials` (see `wp9_support`).
//!
//! The script has no repository-level assertion (it reads documents only),
//! so nothing of it is in `scripts/repo-state-checks.sh`.
//!
//! Two parts of the script cannot be driven to a failure by a document:
//! the toml reader's uncaught error for `[[x]]` after `[x]` (the port
//! reports `CRASH`), and the `FAIL AC 3` for a constants table that is
//! updated while the section still says the constants stand is reachable
//! only together with the other exceed-branch failures.

#[macro_use]
mod wp9_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_15::{WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{
    delete_row, rename_heading, replace_all, replace_in_line, strip_tags, within_section,
};
use hadronyche_tools::doccheck::text::rx;
use hadronyche_tools::doccheck::wire_toml::WIRE_TOML;
use hadronyche_tools::doccheck::{Docs, Report, assert_criterion};

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

/// wire.md with `edit` applied to the "Wire size and capacity recompute"
/// subsection only.
fn sub(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(b, |t| {
        within_section(t, "^Wire size and capacity recompute$", edit)
    })
}

fn toml(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(WIRE_TOML, edit)
}

fn consts(b: &Docs, edit: impl FnOnce(&str) -> String) -> Docs {
    b.mutate(CONSTANTS, edit)
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

/// Replaces every match of the regex `pattern` in `t` (panics if none).
fn rep(t: &str, pattern: &str, to: &str) -> String {
    replace_all(t, pattern, to)
}

/// A wire-size table of one row appended to the subsection.
fn size_row(b: &Docs, cells: [&str; 5]) -> Docs {
    sub(b, |t| {
        append(
            t,
            &format!(
                "| Packet | UDP payload | Bytes on the wire | Per stream | Per channel |\n|---|---|---|---|---|\n| {} |",
                cells.join(" | ")
            ),
        )
    })
}

// ---------------------------------------------------------------- the criteria

/// AC1 — "`wire.md` states the new wire size per default packet and bit rate
/// per stream with the arithmetic shown."
///
/// Script: every equation of the subsection evaluates; the default packet's
/// on-wire size is UDP payload + 66 (the 004-14 default total); the four
/// Ethernet parts and their total 38 stand beside their names; the rate per
/// stream (wire x 8 x 2 000), per channel (/ 6) and per source for 24, 48 and
/// 96 channels with C-22's figures; every row of the wire-size table
/// recomputes.
#[test]
fn ac1_the_wire_size_and_the_bit_rates_with_their_arithmetic() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "The difference from 1 320 bytes / 21.1 Mbit/s is stated in absolute
/// and percent, and the per-source rates are compared with C-22."
///
/// Script: one unit with C-20, the absolute byte difference and the percent
/// (within 0.1 point); one with C-21 (21.1), the rate difference and its
/// percent; every difference and percent against C-22 evaluates; the share
/// against C-21 x streams is the per-stream share; a per-channel figure is a
/// computed one.
#[test]
fn ac2_the_differences_in_absolute_and_percent() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "If > 5 %, the constants table is updated (or an open item with
/// owner and falsifier is recorded)."
///
/// Script: a sentence states the 5 % rule with a direction that matches the
/// computed difference; within 5 %, a sentence says no constant is
/// recomputed; over it, C-20, C-21, C-22, C-24, C-25, C-26, C-90, C-92 and
/// C-118 are named and the table is updated or an open item with owner and
/// falsifier names C-20.
#[test]
fn ac3_the_five_percent_rule() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "`wire.md` states the controller-stream wire cost of C-219 as a
/// formula in controller bytes per slice, at 2 000 packets/s, with the 66
/// bytes of overhead shown."
///
/// Script: the 28 + 38 and the 66, a symbolic formula, 2 000 packets/s, a
/// bit-rate formula, the header term as the cap; worked examples
/// `H + c + 28 + 38 = N` and `N x 8 x 2 000 = R` that evaluate, with a controller
/// example.
#[test]
fn ac4_the_controller_stream_wire_cost() {
    assert_criterion(report(), "AC4");
}

/// AC5 (the script's) — optional toml keys that name a wire size, rate or
/// overhead agree with the computed value; an audio declared rate is the
/// cap form (cap + audio + 66 bytes). The ticket has four criteria; the
/// script's AC 5 is its own addition.
#[test]
fn ac5_toml_keys_and_the_declared_rate() {
    assert_criterion(report(), "AC5");
}

/// DoD — "Every new factual claim carries an evidence tag" and "No new open
/// item without an owner and a falsifier": prose of 5 or more
/// `[A-Za-z0-9]+` words carries a tag, open items name an owner and a
/// falsifier, and every Ethernet framing unit carries `[memory]` (with a
/// checking task) or `[standard]` (with a clause) (C-132).
#[test]
fn dod_evidence_tags_open_items_and_the_framing_tags() {
    assert_criterion(report(), "DoD");
}

/// DoD — "Design documents updated where behaviour differs": the document
/// side is setup (the files read, the constants and toml tables found).
#[test]
fn dod_the_inputs_read() {
    assert!(report().of("setup").is_empty(), "{}", report().render());
    assert!(report().of("CRASH").is_empty(), "{}", report().render());
}

// ---------------------------------------------------------------- denials

denials! {
    // ---- setup
    denial_wire_md_missing => ("setup", "cannot read wire.md", |b: &Docs| b.without(WIRE_MD));
    denial_wire_toml_missing => ("setup", "cannot read wire.toml", |b: &Docs| b.without(WIRE_TOML));
    denial_constants_missing => ("setup", "cannot read 02-constants.md", |b: &Docs| b.without(CONSTANTS));
    denial_a_constant_row_missing => ("setup", "is missing", |b| consts(b, |t| delete_row(t, r"^\| C-20 \|")));
    denial_c22_does_not_parse => ("setup", "C-22 does not parse", |b| consts(b, |t| rep(t, " ch: ", " channels: ")));
    denial_wire_toml_does_not_parse => ("setup", "does not parse", |b| toml(b, |t| format!("[[message.field]]\n{t}")));
    denial_the_toml_lacks_the_layout_tables => ("setup", "lacks the media packet", |b| toml(b, |t| rep(t, "name = \"slice checksum\"", "name = \"slice sum\"")));

    // ---- AC1
    ac1_denial_a_budget_default_total_that_differs => ("AC1", "default_total is 1199", |b| toml(b, |t| rep(t, "default_total = 1198", "default_total = 1199")));
    ac1_denial_no_wire_size_heading => ("AC1", "no heading naming the wire size", |b| wire(b, |t| rename_heading(t, "^Wire size and capacity recompute$", "Sizes")));
    ac1_denial_an_equation_that_does_not_hold => ("AC1", "arithmetic does not hold", |b| sub(b, |t| replace_in_line(t, "^The on-wire size of one default packet", "= 1 264 bytes", "= 1 265 bytes")));
    ac1_denial_the_default_wire_size_stated_wrongly => ("AC1", "the default packet's on-wire size is stated as 1263", |b| sub(b, |t| replace_in_line(t, "^The on-wire size of one default packet", r"1 198 \+ 8 \+ 20 \+ 38 = 1 264 bytes", "1 198 + 8 + 20 + 37 = 1 263 bytes")));
    ac1_denial_no_arithmetic_for_the_default_wire_size => ("AC1", "no arithmetic or table states the on-wire size", |b| sub(b, |t| rep(t, "1 198", "1 197")));
    ac1_denial_the_ethernet_header_not_beside_its_name => ("AC1", "the Ethernet header of 14 bytes is not stated", |b| sub(b, |t| rep(t, r"\b14\b", "15")));
    ac1_denial_the_fcs_not_beside_its_name => ("AC1", "the Ethernet FCS of 4 bytes is not stated", |b| sub(b, |t| rep(t, r"(?i)\bFCS\b|frame check|check sequence", "tail")));
    ac1_denial_the_preamble_not_beside_its_name => ("AC1", "the Ethernet preamble and start-of-frame delimiter of 8 bytes", |b| sub(b, |t| rep(t, r"\b8\b", "9")));
    ac1_denial_the_gap_not_beside_its_name => ("AC1", "the Ethernet inter-frame gap of 12 bytes", |b| sub(b, |t| rep(t, r"\b12\b", "13")));
    ac1_denial_the_ethernet_total_not_stated => ("AC1", "the Ethernet framing total of 38 bytes", |b| sub(b, |t| rep(t, r"\b38\b", "39")));
    ac1_denial_no_rate_per_stream => ("AC1", "no bit rate per stream of 20.2 Mbit/s", |b| sub(b, |t| rep(t, r"20\.22\d*", "20.99")));
    ac1_denial_no_rate_per_channel => ("AC1", "no bit rate per channel of 3.37 Mbit/s", |b| sub(b, |t| rep(t, r"3\.37", "3.99")));
    ac1_denial_no_unit_for_a_source => ("AC1", "no unit gives 24 channels: 4 streams", |b| sub(b, |t| rep(t, r"80\.90 Mbit/s", "81.50 Mbit/s")));
    ac1_denial_a_wire_size_row_without_a_column => ("AC1", "lacks a payload, wire, stream or channel cell", |b| sub(b, |t| append(t, "| Packet | UDP payload | Bytes on the wire |\n|---|---|---|\n| Default x | 1 198 | 1 198 + 8 + 20 + 38 = 1 264 |")));
    ac1_denial_a_wire_size_row_without_arithmetic => ("AC1", "states no payload or no on-the-wire arithmetic", |b| size_row(b, ["Default x", "tbd", "1 198 + 8 + 20 + 38 = 1 264", "x", "y"]));
    ac1_denial_a_wire_size_row_with_the_wrong_payload => ("AC1", "payload 1197 is not the 004-14 default total", |b| size_row(b, ["Default x", "1 197", "1 197 + 8 + 20 + 38 = 1 263", "1 263 × 8 × 2 000 = 20 208 000 bit/s, 20.21 Mbit/s", "3.37 Mbit/s"]));
    ac1_denial_a_wire_size_row_not_payload_plus_66 => ("AC1", "on the wire 1263 is not payload 1198 + 66", |b| size_row(b, ["Other x", "1 198", "1 198 + 8 + 20 + 37 = 1 263", "1 263 × 8 × 2 000 = 20 208 000 bit/s, 20.21 Mbit/s", "3.37 Mbit/s"]));
    ac1_denial_a_wire_size_row_without_a_rate => ("AC1", "states no Mbit/s rate", |b| size_row(b, ["Other x", "1 198", "1 198 + 8 + 20 + 38 = 1 264", "1 264 × 8 × 2 000 = 20 224 000 bit/s", "3.37 Mbit/s"]));
    ac1_denial_a_wire_size_row_with_the_wrong_rate => ("AC1", "rate 20.99 Mbit/s is not", |b| size_row(b, ["Other x", "1 198", "1 198 + 8 + 20 + 38 = 1 264", "1 264 × 8 × 2 000 = 20 224 000 bit/s, 20.99 Mbit/s", "3.37 Mbit/s"]));
    ac1_denial_a_wire_size_row_with_the_wrong_bit_rate_equation => ("AC1", "does not give 20224000 bit/s", |b| size_row(b, ["Other x", "1 198", "1 198 + 8 + 20 + 38 = 1 264", "1 264 × 8 × 2 000 = 20 224 001 bit/s, 20.22 Mbit/s", "3.37 Mbit/s"]));
    ac1_denial_a_wire_size_row_without_a_per_channel_rate => ("AC1", "states no per-channel rate", |b| size_row(b, ["Other x", "1 198", "1 198 + 8 + 20 + 38 = 1 264", "1 264 × 8 × 2 000 = 20 224 000 bit/s, 20.22 Mbit/s", "tbd"]));
    ac1_denial_a_wire_size_row_with_the_wrong_per_channel_rate => ("AC1", "per-channel 3.99 Mbit/s is not", |b| size_row(b, ["Other x", "1 198", "1 198 + 8 + 20 + 38 = 1 264", "1 264 × 8 × 2 000 = 20 224 000 bit/s, 20.22 Mbit/s", "3.99 Mbit/s"]));

    // ---- AC2
    ac2_denial_no_byte_difference => ("AC2", "no unit gives the byte difference", |b| sub(b, |t| rep(t, r"\b56\b", "55")));
    ac2_denial_a_percent_beside_c20_that_is_not_the_share => ("AC2", "a percent beside C-20", |b| sub(b, |t| replace_in_line(t, "^The difference from the design's derived figures", "56 bytes smaller, 4.2 % less", "56 bytes smaller, 4.9 % less")));
    ac2_denial_a_percent_beside_c21_that_is_not_the_share => ("AC2", "a percent beside C-21", |b| sub(b, |t| replace_in_line(t, "^The difference from the design's derived figures", r"\(20.224 Mbit/s\), 4.2 % less", "(20.224 Mbit/s), 4.9 % less")));
    ac2_denial_no_rate_difference => ("AC2", "no unit gives the rate difference", |b| sub(b, |t| replace_in_line(t, "^The difference from the design's derived figures", "0.88 Mbit/s lower", "0.99 Mbit/s lower")));
    ac2_denial_a_c22_difference_that_is_not_computed => ("AC2", "the 24-channel difference from C-22 is stated as -3.5", |b| sub(b, |t| replace_in_line(t, r"^\| 24 channels", "−3.10 Mbit/s", "−3.50 Mbit/s")));
    ac2_denial_a_c22_share_that_is_not_computed => ("AC2", "the 24-channel share of the C-22 difference is stated as 3 %", |b| sub(b, |t| replace_in_line(t, r"^\| 24 channels", "−3.7 %", "−3.0 %")));
    ac2_denial_a_share_against_c21_times_streams_that_is_not_the_per_stream_share => ("AC2", "the share against C-21 x streams is stated as -4.9 %", |b| sub(b, |t| replace_in_line(t, "^The difference from the design's derived figures", "every source is 4.2 % lower \\(−4.15 %\\)", "every source is 4.2 % lower (−4.9 %)")));
    ac2_denial_a_plain_share_against_c21_times_streams => ("AC2", "the share against C-21 x streams is stated as 4.9 %", |b| sub(b, |t| replace_in_line(t, "^The difference from the design's derived figures", "every source is 4.2 % lower", "every source is 4.9 % lower")));
    ac2_denial_a_per_channel_figure_that_is_not_computed => ("AC2", "a per-channel figure of 3.99 Mbit/s", |b| sub(b, |t| replace_in_line(t, "^The difference from the design's derived figures", "per channel it is 3.37 Mbit/s", "per channel it is 3.99 Mbit/s")));

    // ---- AC3
    ac3_denial_no_five_percent_sentence => ("AC3", "no sentence states the 5 % rule", |b| sub(b, |t| rep(t, "5 %", "five percent")));
    ac3_denial_a_five_percent_sentence_without_a_direction => ("AC3", "states no direction", |b| sub(b, |t| rep(&rep(t, "below the 5 %", "at the 5 %"), "within 5 %", "at 5 %")));
    ac3_denial_a_direction_that_contradicts_the_computed_difference => ("AC3", "says the difference exceeds 5 %, but the computed", |b| sub(b, |t| rep(t, "are both below the 5 %", "both exceed the 5 %")));
    ac3_denial_within_five_percent_without_the_constants_standing => ("AC3", "does not say that no constant is recomputed", |b| sub(b, |t| rx(r"(?i)no constant[^.;]{0,40}(?:recomput|changed|updated|re-?derived)|\bconstants?\b[^.;]{0,60}\bstand\b|not recomputed|constants? (?:are|is|remain|remains) unchanged").replace_all(t, "x").into_owned()));
    ac3_denial_over_five_percent_with_a_constant_not_named => ("AC3", "does not name C-118 as marked for recompute", |b| sub(&consts(b, |t| replace_in_line(t, r"^\| C-20 \|", "1 320", "1 200")), |t| rep(t, "C-118", "C-1l8")));
    ac3_denial_over_five_percent_but_the_section_says_the_constants_stand => ("AC3", "the section still says the constants stand", |b| consts(b, |t| replace_in_line(t, r"^\| C-20 \|", "1 320", "1 200")));
    ac3_denial_over_five_percent_with_neither_update_nor_open_item => ("AC3", "neither 'constants table updated' nor an open item", |b| consts(b, |t| replace_in_line(t, r"^\| C-20 \|", "1 320", "1 200")));

    // ---- AC4
    ac4_denial_no_controller_cost_paragraph => ("AC4", "no paragraph states the C-219 wire cost", |b| sub(b, |t| rep(&rep(t, "C-219", "C-2l9"), r"(?i)per slice|controller bytes", "x")));
    ac4_denial_the_overhead_not_shown_as_28_plus_38 => ("AC4", "not shown as 28 + 38 with 66", |b| sub(b, |t| rep(t, r"28 \+ 38", "28 plus 38")));
    ac4_denial_no_symbolic_formula => ("AC4", "no symbolic formula", |b| sub(b, |t| rep(t, r"= H \+ c \+ 28 \+ 38", "= H + c + 66")));
    ac4_denial_not_stated_at_2000_packets_a_second => ("AC4", "not stated at 2 000 packets/s", |b| sub(b, |t| rep(t, "2 000", "3 000")));
    ac4_denial_no_bit_rate_formula => ("AC4", "no bit-rate formula of the form bytes x 8 x 2 000", |b| sub(b, |t| rep(t, "× 2 000", "· 2 000")));
    ac4_denial_the_header_term_not_named_as_the_cap => ("AC4", "the header term of the formula is not named as the 100-byte cap", |b| sub(b, |t| rep(t, r"\b100\b", "101")));
    ac4_denial_a_worked_example_whose_header_term_is_not_a_budget_figure => ("AC4", "the header term is not a figure of the budget", |b| sub(b, |t| replace_in_line(t, "^Worked examples of the C-219 formula", r"100 \+ 30 \+ 28 \+ 38 = 196", "99 + 30 + 28 + 38 = 195")));
    ac4_denial_no_worked_example => ("AC4", "no worked example 'H + c + 28 + 38 = N'", |b| sub(b, |t| rep(t, r"\+ 28 \+ 38 =", "+ 66 =")));
    ac4_denial_no_controller_example => ("AC4", "has no controller example", |b| sub(b, |t| rep(&rep(&rep(t, r"100 \+ 30 \+ 28 \+ 38 = 196", "100 + 1 152 + 28 + 38 = 1 318"), r"100 \+ 1 028 \+ 28 \+ 38 = 1 194", "100 + 1 152 + 28 + 38 = 1 318"), r"64 \+ 30 \+ 28 \+ 38 = 160", "64 + 1 152 + 28 + 38 = 1 282")));
    ac4_denial_no_bit_rate_example => ("AC4", "no worked bit-rate example", |b| sub(b, |t| rep(t, "× 8 × 2 000 =", "× 8 × 2 000 is")));
    ac4_denial_a_bit_rate_example_that_does_not_use_a_controller_total => ("AC4", "does not use a controller example's byte count", |b| sub(b, |t| rep(&rep(&rep(t, r"\b196 × 8", "197 × 8"), r"1 194 × 8", "1 195 × 8"), r"\b160 × 8", "161 × 8")));

    // ---- AC5
    ac5_denial_an_audio_declared_rate_the_layout_does_not_compute => ("AC5", "a figure the layout does not compute", |b| sub(b, |t| rep(t, "21.09 Mbit/s, at 2 000", "21.5 Mbit/s, at 2 000")));
    ac5_denial_an_audio_declared_rate_that_is_not_the_cap_form => ("AC5", "is not the cap form", |b| sub(b, |t| rep(t, "21 088 000 bit/s, 21.09 Mbit/s, at 2 000", "20 224 000 bit/s, 20.22 Mbit/s, at 2 000")));
    ac5_denial_an_audio_declared_rate_with_other_bytes => ("AC5", "the audio declared rate's bytes are not 1318", |b| sub(b, |t| rep(t, "1 318 bytes per packet on the wire", "1 319 bytes per packet on the wire")));
    ac5_denial_an_overhead_key_that_differs => ("AC5", "the overhead of 66 bytes", |b| toml(b, |t| append(t, "[overhead_check]\nbytes = 64")));
    ac5_denial_an_ethernet_key_that_differs => ("AC5", "the Ethernet framing of 38 bytes", |b| toml(b, |t| append(t, "[ethernet_overhead]\nbytes = 37")));
    ac5_denial_an_ipv4_header_key_that_differs => ("AC5", "the IPv4 header of 20 bytes", |b| toml(b, |t| append(t, "[wire_sizes]\nipv4_header = 21")));
    ac5_denial_a_udp_header_key_that_differs => ("AC5", "the UDP header of 8 bytes", |b| toml(b, |t| append(t, "[wire_sizes]\nudp_header = 9")));
    ac5_denial_a_default_wire_key_that_differs => ("AC5", "the default on-wire size of 1264 bytes", |b| toml(b, |t| append(t, "[wire_sizes]\ndefault_wire = 1000")));
    ac5_denial_a_maximum_wire_key_that_differs => ("AC5", "the format maximum on-wire size of 1321 bytes", |b| toml(b, |t| append(t, "[wire_sizes]\nwire_maximum = 1000")));
    ac5_denial_a_stream_rate_key_that_differs => ("AC5", "the default per-stream rate of 20224000 bit/s", |b| toml(b, |t| append(t, "[rates]\ndefault_stream_rate = 1000")));

    // ---- DoD
    dod_denial_an_untagged_paragraph => ("DoD", "has no evidence tag", |b| sub(b, |t| strip_tags(t, "^The Ethernet figures are the ticket's own")));
    dod_denial_an_open_item_without_an_owner => ("DoD", "an open item names no owner or no falsifier", |b| sub(b, |t| append(t, "**Open item (x).** Whether the sizes hold is open `[decision]` until a later ticket.")));
    dod_denial_a_framing_row_without_a_tag => ("DoD", "has no [memory] or [standard] tag", |b| sub(b, |t| replace_in_line(t, r"^\| Ethernet inter-frame gap", r"`\[memory\]`", "")));
    dod_denial_a_memory_framing_row_without_a_checking_task => ("DoD", "names no checking task", |b| sub(b, |t| replace_in_line(t, r"^\| Ethernet inter-frame gap", "; ticket 004-22 pins it", "")));
    dod_denial_a_standard_framing_row_without_a_clause => ("DoD", "framing row tagged [standard] has no clause reference", |b| sub(b, |t| replace_in_line(t, r"^\| Ethernet preamble", r"`\[memory\]`", "`[standard]`")));
    dod_denial_a_standard_framing_sentence_without_a_clause => ("DoD", "framing sentence tagged [standard] has no clause reference", |b| sub(b, |t| replace_in_line(t, "^The Ethernet figures are the ticket's own", "38 bytes of Ethernet in all `\\[memory\\]`", "38 bytes of Ethernet in all `[standard]`")));
}

/// Every denial is rejected by the old script, except the ones listed here
/// (the script accepts a denial that breaks an assertion of its own header:
/// "script wrong"); the ignored test below finds them.
const SCRIPT_ACCEPTS: &[&str] = &[];

/// Runs every denial fixture against the old script (needs Python and the
/// script, from the working tree or from the history after its deletion):
/// `cargo test -p hadronyche-tools --test doc_004_15 -- --ignored`.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees_on_the_denials() {
    let all = all_denials();
    let accepted = wp9_support::script_accepts("004-15", &all);
    assert_eq!(accepted, SCRIPT_ACCEPTS, "the script accepts these denials");
}

// ---------------------------------------------------------------- red phase

/// The tree at the commit that added the script (d3e5ea6, "failing checks for
/// the capacity recompute") fails the criterion the script failed: AC1 (its
/// subsection did not exist, so the script stopped at the missing heading).
/// Skipped when the commit is not in the clone.
#[test]
fn red_phase_tree_fails_the_criteria_the_script_failed() {
    let commit = "d3e5ea6b1ef0ca0f96089a8c068c7afb34d3ed38";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(failed, ["AC1"], "{}", r.render());
}
