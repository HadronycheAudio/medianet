//! Ticket 004-01 — wire.md skeleton and encoding conventions. Port of
//! `checks/004-01.sh`. The script is NOT deleted: `checks/004-03.sh` to
//! `004-09.sh` still run `bash checks/004-01.sh` as a gate and may not be
//! edited here; it goes when their ports land (the harness reads it from the
//! working tree until then).
//!
//! The check is `hadronyche_tools::doccheck::checks::c004_01`; these tests
//! run it on the repository's documents (one `#[test]` per acceptance
//! criterion) and on denial fixtures: the real documents with one mutation
//! applied in memory, each also run against the old script by the ignored
//! test `old_script_agrees_on_the_denials` (see `wp9_support`).
//!
//! The script has no repository-level assertion; it prints a NOTE (not a
//! failure) when a placeholder section has no "to be filled by 004-TT"
//! note, which is not an assertion and is not ported.

#[macro_use]
mod wp9_support;

use std::sync::OnceLock;

use hadronyche_tools::doccheck::checks::c004_01::{WIRE_MD, run};
use hadronyche_tools::doccheck::docs::{commit_exists, repo_root};
use hadronyche_tools::doccheck::mutate::{replace_all, replace_in_line, within_section};
use hadronyche_tools::doccheck::text::rx;
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

/// `edit` applied to the section of the heading matching `heading`.
fn sec(b: &Docs, heading: &str, edit: impl FnOnce(&str) -> String) -> Docs {
    wire(b, |t| within_section(t, heading, edit))
}

fn re_all(t: &str, pattern: &str, to: &str) -> String {
    rx(pattern).replace_all(t, to).into_owned()
}

/// Replaces `from` in every heading line whose text matches `pattern`.
fn heads(t: &str, pattern: &str, from: &str, to: &str) -> String {
    let (h, f) = (rx(&format!(r"(?i)^#{{1,6}}\s+.*(?:{pattern})")), rx(from));
    t.split('\n')
        .map(|l| {
            if h.is_match(l) {
                f.replace_all(l, to).into_owned()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sets cell `k` (0 = the id) of the first row matching `row` to `text`.
fn set_cell(t: &str, row: &str, k: usize, text: &str) -> String {
    let re = rx(row);
    let mut done = false;
    t.split('\n')
        .map(|l| {
            if !done && re.is_match(l) {
                done = true;
                let mut cells: Vec<String> = l.split('|').map(String::from).collect();
                cells[k + 1] = format!(" {text} ");
                cells.join("|")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn append(t: &str, extra: &str) -> String {
    format!("{}\n\n{extra}\n", t.trim_end())
}

const BO: &str = "^Byte order and alignment$";
const ALIGN: &str = "^Alignment and padding$";
const BITS: &str = "^Bit-level encoding of samples$";

/// AC1 — "One byte order for all multi-byte integers, tagged `[decision]`
/// and justified."
#[test]
fn ac1_one_byte_order_tagged_and_justified() {
    assert_criterion(report(), "AC1");
}

/// AC2 — "Alignment is stated for widths 1, 2, 4, 8, 16 bytes and for
/// sub-byte bit fields."
#[test]
fn ac2_alignment_for_every_width() {
    assert_criterion(report(), "AC2");
}

/// AC3 — "f32 and Q1.31 are defined bit-exactly, 4 bytes each, under
/// bit-level-encoding headings."
#[test]
fn ac3_f32_and_q131_bit_exact() {
    assert_criterion(report(), "AC3");
}

/// AC4 — "The field-table format has exactly offset, width, type, meaning
/// and valid range, and bounds every variable-length part."
#[test]
fn ac4_the_field_table_format() {
    assert_criterion(report(), "AC4");
}

/// AC5 — "The 22 placeholder sections are present as distinct headings."
#[test]
fn ac5_the_placeholder_sections() {
    assert_criterion(report(), "AC5");
}

/// AC6 — "The four-layer table gives DSCP 40 / 48 / 46 / 40+46, with IPv4
/// and multicast TTL 1 stated."
#[test]
fn ac6_the_layer_table() {
    assert_criterion(report(), "AC6");
}

/// AC7 — "The Constants subsection lists every required id with the values
/// of 02-constants.md (which wins)."
#[test]
fn ac7_the_constants_subsection() {
    assert_criterion(report(), "AC7");
}

denials! {
    denial_wire_md_missing => ("AC1", "does not exist", |b: &Docs| b.without(WIRE_MD));
    ac1_denial_no_byte_order_heading => ("AC1", "no byte-order section heading", |b| wire(b, |t| heads(t, r"byte[\s-]*order", r"(?i)byte[\s-]*order", "octet sequence")));
    ac1_denial_no_single_order => ("AC1", "does not state which single order", |b| sec(b, BO, |t| re_all(t, r"(?i)(?:big|little)[-\s]endian|least[-\s]+significant[-\s]+byte[-\s]+first|\bLSB[-\s]+first\b|network\s+byte\s+order|most[-\s]+significant[-\s]+byte[-\s]+first|\bMSB[-\s]+first\b", "x")));
    ac1_denial_two_orders => ("AC1", "more than one order", |b| sec(b, BO, |t| append(t, "Every multi-byte field is big-endian.")));
    ac1_denial_no_decision_tag => ("AC1", "does not carry the [decision] tag", |b| sec(b, BO, |t| replace_all(t, r"\[decision\]", "[choice]")));
    ac1_denial_no_justification => ("AC1", "no justification", |b| sec(b, BO, |t| re_all(t, r"(?i)\b(justif\w*|rationale|because|since|reason\w*|we choose|chosen|decided|therefore|thus|so\s+that|ensures?|avoids?|simplif\w*|minimi[sz]\w*|makes?|allows?|matches?|consistent|portable|network\s+byte\s+order|most[-\s]+significant|byte\s+first)\b", "x")));
    ac2_denial_no_alignment_heading => ("AC2", "no alignment section heading", |b| wire(b, |t| heads(t, "align", r"(?i)align\w*", "layout")));
    ac2_denial_width_1_not_stated => ("AC2", "for 1-byte fields", |b| sec(b, ALIGN, |t| re_all(t, r"\b1\b", "x")));
    ac2_denial_width_2_not_stated => ("AC2", "for 2-byte fields", |b| sec(b, ALIGN, |t| re_all(t, r"\b2\b", "x")));
    ac2_denial_width_4_not_stated => ("AC2", "for 4-byte fields", |b| sec(b, ALIGN, |t| re_all(t, r"\b4\b", "x")));
    ac2_denial_width_8_not_stated => ("AC2", "for 8-byte fields", |b| sec(b, ALIGN, |t| re_all(t, r"\b8\b", "x")));
    ac2_denial_width_16_not_stated => ("AC2", "for 16-byte fields", |b| sec(b, ALIGN, |t| re_all(t, r"\b16\b", "x")));
    ac2_denial_bit_fields_not_stated => ("AC2", "for sub-byte bit fields", |b| sec(b, ALIGN, |t| re_all(t, r"(?i)sub[-\s]?byte|bit[-\s]?fields?|bitfield|\bbits?\b", "x")));
    ac2_denial_an_empty_answer_does_not_count => ("AC2", "for 16-byte fields", |b| sec(b, ALIGN, |t| set_cell(t, r"^\| 16 bytes", 1, "\u{2014}")));
    ac3_denial_no_bit_level_heading => ("AC3", "no 'bit-level encoding' heading", |b| wire(b, |t| heads(t, r"bit[-\s]?level\s+encoding", r"(?i)bit[-\s]?level\s+encoding", "Sample layout")));
    ac3_denial_f32_heading_not_named => ("AC3", "names f32", |b| wire(b, |t| heads(t, r"f32 samples", "f32 samples", "32-bit floats")));
    ac3_denial_q131_heading_not_named => ("AC3", "names Q1.31", |b| wire(b, |t| heads(t, r"Q1\.31 samples", r"Q1\.31 samples", "fixed-point samples")));
    ac3_denial_f32_not_defined => ("AC3", "does not define f32", |b| sec(b, BITS, |t| re_all(t, r"(?i)\bf32\b", "float")));
    ac3_denial_q131_not_defined => ("AC3", "does not define Q1.31", |b| sec(b, BITS, |t| re_all(t, r"(?i)\bq1[.\s_]?31\b", "fixed")));
    ac3_denial_not_four_bytes => ("AC3", "f32 is not stated as 4 bytes", |b| sec(b, BITS, |t| re_all(t, r"(?i)\b4\s*[- ]?bytes?\b|\b32\s*[- ]?bits?\b", "x")));
    ac3_denial_q131_not_four_bytes => ("AC3", "Q1.31 is not stated as 4 bytes", |b| sec(b, BITS, |t| re_all(t, r"(?i)\b4\s*[- ]?bytes?\b|\b32\s*[- ]?bits?\b", "x")));
    ac3_denial_no_ieee_754 => ("AC3", "does not cite IEEE 754 binary32", |b| sec(b, BITS, |t| re_all(t, r"(?i)ieee\s*[- ]?754|binary32", "x")));
    ac3_denial_no_exponent => ("AC3", "does not name the exponent field", |b| sec(b, BITS, |t| re_all(t, r"(?i)\bexponent\b", "x")));
    ac3_denial_no_mantissa => ("AC3", "does not name the mantissa/fraction field", |b| sec(b, BITS, |t| re_all(t, r"(?i)\bmantissa\b|\bfraction\b", "x")));
    ac3_denial_no_sign => ("AC3", "does not name the sign bit", |b| sec(b, BITS, |t| re_all(t, r"(?i)\bsign\b", "x")));
    ac3_denial_no_twos_complement => ("AC3", "does not state two's complement", |b| sec(b, BITS, |t| re_all(t, r"(?i)two'?s\s+complement|2'?s\s+complement", "x")));
    ac3_denial_no_31_fractional_bits => ("AC3", "does not state its 31 fractional bits", |b| sec(b, BITS, |t| re_all(t, r"\b31\b", "x")));
    ac3_denial_no_fractional_scale => ("AC3", "does not state its fractional scale", |b| sec(b, BITS, |t| re_all(t, r"(?i)fraction\w*|\bscale\b|2\s*\^?\s*[-\u{2212}]?\s*31", "x")));
    ac4_denial_no_field_table_heading => ("AC4", "no field-table format section heading", |b| wire(b, |t| heads(t, r"field[\s-]*table", r"(?i)field[\s-]*table", "Message layout")));
    ac4_denial_not_five_columns => ("AC4", "no table with exactly the five columns", |b| wire(b, |t| re_all(t, r"(?m)^(\| Offset[^\n]*?)Valid range", "${1}Range")));
    ac4_denial_no_bounded_variable_length_rule => ("AC4", "no rule that every variable-length part has a stated maximum count", |b| sec(b, "^Field-table format$", |t| re_all(t, r"(?i)variable[\s-]*length", "dynamic")));
    ac5_denial_a_placeholder_section_missing => ("AC5", "placeholder section missing: checksum", |b| wire(b, |t| heads(t, "checksum", r"(?i)checksum", "crc")));
    ac5_denial_two_placeholders_share_a_heading => ("AC5", "shares its heading with", |b| wire(b, |t| replace_in_line(t, r"^## Slice header$", "Slice header", "Slice header checksum")));
    ac6_denial_no_layer_table => ("AC6", "no layer table with Layer and DSCP columns", |b| sec(b, "^Layer overview$", |t| replace_in_line(t, r"^\| Layer \| Carries", "Layer", "Tier")));
    ac6_denial_five_rows => ("AC6", "layer table has 5 rows, expected four layers", |b| sec(b, "^Layer overview$", |t| replace_in_line(t, r"^\| Measurement \|", "$", "\n| Extra | x | y | 40 |")));
    ac6_denial_a_layer_missing => ("AC6", "layer table has no sync row", |b| sec(b, "^Layer overview$", |t| replace_in_line(t, r"^\| Clock sync \|", "Clock sync", "Timing")));
    ac6_denial_two_rows_for_a_layer => ("AC6", "layer table has 2 sync rows, expected one", |b| sec(b, "^Layer overview$", |t| replace_in_line(t, r"^\| Media \|", "Media", "Media sync")));
    ac6_denial_the_control_dscp => ("AC6", "control layer DSCP is", |b| sec(b, "^Layer overview$", |t| set_cell(t, r"^\| Discovery and control", 3, "CS6 (48)")));
    ac6_denial_the_sync_dscp => ("AC6", "sync layer DSCP is", |b| sec(b, "^Layer overview$", |t| set_cell(t, r"^\| Clock sync", 3, "CS5 (40)")));
    ac6_denial_the_media_dscp => ("AC6", "media layer DSCP is", |b| sec(b, "^Layer overview$", |t| set_cell(t, r"^\| Media \|", 3, "CS5 (40)")));
    ac6_denial_the_measurement_dscp => ("AC6", "measurement layer DSCP is", |b| sec(b, "^Layer overview$", |t| set_cell(t, r"^\| Measurement \|", 3, "reports CS5 (40)")));
    ac6_denial_no_ipv4 => ("AC6", "layer overview does not state IPv4", |b| sec(b, "^Layer overview$", |t| re_all(t, r"(?i)\bIPv4\b", "IP")));
    ac6_denial_no_multicast => ("AC6", "layer overview does not state multicast", |b| sec(b, "^Layer overview$", |t| re_all(t, r"(?i)\bmulticast\b", "group")));
    ac6_denial_no_ttl_1 => ("AC6", "does not state multicast TTL 1", |b| sec(b, "^Layer overview$", |t| re_all(t, r"(?i)\bTTL\s*(?:=|of\s*)?1\b", "hop limit")));
    ac7_denial_constants_missing => ("AC7", "does not exist; cannot compare constants", |b: &Docs| b.without(CONSTANTS));
    ac7_denial_no_constants_subsection => ("AC7", "no Constants subsection", |b| wire(b, |t| heads(t, "constant", r"(?i)constants?", "Values")));
    ac7_denial_a_value_that_differs => ("AC7", "C-17 mismatch", |b| sec(b, "^Constants$", |t| replace_in_line(t, r"^\| C-17 \|", "1 252", "1 253")));
    ac7_denial_an_id_missing_from_the_subsection => ("AC7", "C-17 missing from the Constants subsection", |b| sec(b, "^Constants$", |t| t.split('\n').filter(|l| !l.contains("C-17")).collect::<Vec<_>>().join("\n")));
    ac7_denial_an_id_missing_from_the_source => ("AC7", "C-17 is not present in", |b: &Docs| b.mutate(CONSTANTS, |t: &str| t.split('\n').filter(|l| !l.starts_with("| C-17 |")).collect::<Vec<_>>().join("\n")));
    ac7_denial_an_id_without_a_value => ("AC7", "C-22 has no value in the Constants subsection", |b: &Docs| wire(&b.mutate(CONSTANTS, |t: &str| set_cell(t, r"^\| C-22 \|", 2, "tbd")), |t| within_section(t, "^Constants$", |s| set_cell(s, r"^\| C-22 \|", 2, "n/a"))));
}

/// Every denial is rejected by the old script, except the ones listed here.
const SCRIPT_ACCEPTS: &[&str] = &[];

/// `cargo test -p hadronyche-tools --test doc_004_01 -- --ignored`.
#[test]
#[ignore = "runs the old script: needs python3 and the script in the tree or its history"]
fn old_script_agrees_on_the_denials() {
    let all = all_denials();
    assert_eq!(wp9_support::script_accepts("004-01", &all), SCRIPT_ACCEPTS);
}

/// The tree at the commit that added the script fails every criterion.
#[test]
fn red_phase_tree_fails_every_criterion() {
    let commit = "0a7d3276bb7a2f0b7dd6c44e0b2cd116db337417";
    if !commit_exists(&repo_root(), commit) {
        eprintln!("skipped: {commit} is not in this clone");
        return;
    }
    let r = run(&Docs::at_commit(repo_root(), commit));
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    assert_eq!(
        failed,
        ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "AC7"],
        "{}",
        r.render()
    );
}
