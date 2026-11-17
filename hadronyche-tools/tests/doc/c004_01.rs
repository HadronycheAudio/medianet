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
//! harness (`doccheck-equiv 004-01 --denials-only`).
//!
//! The script has no repository-level assertion; it prints a NOTE (not a
//! failure) when a placeholder section has no "to be filled by 004-TT"
//! note, which is not an assertion and is not ported.

use hadronyche_tools::doccheck::checks::c004_01::SPEC;
use hadronyche_tools::doccheck::testing::red_phase;
use hadronyche_tools::doccheck::assert_criterion;

use hadronyche_tools::doccheck::testing::report_of;

/// AC1 — "One byte order for all multi-byte integers, tagged `[decision]`
/// and justified."
#[test]
fn ac1_one_byte_order_tagged_and_justified() {
    assert_criterion(&report_of(&SPEC), "AC1");
}

/// AC2 — "Alignment is stated for widths 1, 2, 4, 8, 16 bytes and for
/// sub-byte bit fields."
#[test]
fn ac2_alignment_for_every_width() {
    assert_criterion(&report_of(&SPEC), "AC2");
}

/// AC3 — "f32 and Q1.31 are defined bit-exactly, 4 bytes each, under
/// bit-level-encoding headings."
#[test]
fn ac3_f32_and_q131_bit_exact() {
    assert_criterion(&report_of(&SPEC), "AC3");
}

/// AC4 — "The field-table format has exactly offset, width, type, meaning
/// and valid range, and bounds every variable-length part."
#[test]
fn ac4_the_field_table_format() {
    assert_criterion(&report_of(&SPEC), "AC4");
}

/// AC5 — "The 22 placeholder sections are present as distinct headings."
#[test]
fn ac5_the_placeholder_sections() {
    assert_criterion(&report_of(&SPEC), "AC5");
}

/// AC6 — "The four-layer table gives DSCP 40 / 48 / 46 / 40+46, with IPv4
/// and multicast TTL 1 stated."
#[test]
fn ac6_the_layer_table() {
    assert_criterion(&report_of(&SPEC), "AC6");
}

/// AC7 — "The Constants subsection lists every required id with the values
/// of 02-constants.md (which wins)."
#[test]
fn ac7_the_constants_subsection() {
    assert_criterion(&report_of(&SPEC), "AC7");
}

/// The tree at the commit that added the script fails every criterion.
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(
        &SPEC,
        "0a7d3276bb7a2f0b7dd6c44e0b2cd116db337417",
        &["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "AC7"],
    );
}

hadronyche_tools::denial_tests!(
    SPEC;
    a_sync_dscp_of_0,
    a_sync_dscp_of_007,
    a_sync_dscp_of_plus7,
    a_sync_dscp_of_1_2,
    a_sync_dscp_of_2e63,
    a_sync_dscp_of_20_digits,
    a_sync_dscp_of_45_digits,
    a_sync_dscp_of_1e13,
    a_sync_dscp_of_1e36,
    a_sync_dscp_of_superscript,
    a_sync_dscp_of_micro,
    ac7_denial_an_id_without_a_value,
    denial_wire_md_missing,
    ac1_denial_no_byte_order_heading,
    ac1_denial_no_single_order,
    ac1_denial_two_orders,
    ac1_denial_no_decision_tag,
    ac1_denial_no_justification,
    ac2_denial_no_alignment_heading,
    ac2_denial_width_1_not_stated,
    ac2_denial_width_2_not_stated,
    ac2_denial_width_4_not_stated,
    ac2_denial_width_8_not_stated,
    ac2_denial_width_16_not_stated,
    ac2_denial_bit_fields_not_stated,
    ac2_denial_an_empty_answer_does_not_count,
    ac3_denial_no_bit_level_heading,
    ac3_denial_f32_heading_not_named,
    ac3_denial_q131_heading_not_named,
    ac3_denial_f32_not_defined,
    ac3_denial_q131_not_defined,
    ac3_denial_not_four_bytes,
    ac3_denial_q131_not_four_bytes,
    ac3_denial_no_ieee_754,
    ac3_denial_no_exponent,
    ac3_denial_no_mantissa,
    ac3_denial_no_sign,
    ac3_denial_no_twos_complement,
    ac3_denial_no_31_fractional_bits,
    ac3_denial_no_fractional_scale,
    ac4_denial_no_field_table_heading,
    ac4_denial_not_five_columns,
    ac4_denial_no_bounded_variable_length_rule,
    ac5_denial_a_placeholder_section_missing,
    ac5_denial_two_placeholders_share_a_heading,
    ac6_denial_no_layer_table,
    ac6_denial_five_rows,
    ac6_denial_a_layer_missing,
    ac6_denial_two_rows_for_a_layer,
    ac6_denial_the_control_dscp,
    ac6_denial_the_sync_dscp,
    ac6_denial_the_media_dscp,
    ac6_denial_the_measurement_dscp,
    ac6_denial_no_ipv4,
    ac6_denial_no_multicast,
    ac6_denial_no_ttl_1,
    ac7_denial_constants_missing,
    ac7_denial_no_constants_subsection,
    ac7_denial_a_value_that_differs,
    ac7_denial_an_id_missing_from_the_subsection,
    ac7_denial_an_id_missing_from_the_source,
    a_dscp_beyond_i128,
    a_dscp_with_a_superscript_still_reads_its_number,
    a_decimal_width_is_not_a_width,
    crlf_line_ends_pass,
    a_trailing_tab_after_a_heading_passes,
    an_en_dash_for_an_empty_answer,
    a_micro_sign_in_a_constant_value,
);
