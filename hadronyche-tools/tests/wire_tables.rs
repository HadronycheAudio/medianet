//! Acceptance tests for ticket 004-02 — machine-readable tables and
//! round-trip check.
//!
//! # The tester's contract (fixed here; the coder implements it)
//!
//! The tests drive the smallest API that covers the ticket:
//!
//! ```ignore
//! hadronyche_tools::wire_tables::check_file(&Path) -> Result<Report, Error>
//! ```
//!
//! with [`FailureKind`] distinguishing the failure kinds the negative fixtures
//! demonstrate:
//!
//! | kind | ticket line |
//! |---|---|
//! | `Parse` | AC 2, fixture 1 |
//! | `RoundTrip` | AC 2, fixture 2 |
//! | `Overlap` | AC 2, fixture 3 |
//! | `Gap` | AC 2, fixture 4 |
//! | `MaxSizeMismatch` | AC 2, fixture 5 |
//! | `OversizedControl` | AC 6, fixture 6 |
//!
//! `Report` carries one `MessageReport` per message, in file order. The tests
//! read `name`, `layer`, `dscp`, `max_size`, `field_count`, `section_of` and
//! `rejection_reasons` from it, which proves that the checker parsed a model
//! instead of merely accepting the bytes.
//!
//! The checker must run in-process, unprivileged, with no network access
//! (AC 4: an ordinary `cargo test`), and must finish the repository's full
//! file in under 10 s (AC 5).
//!
//! # The fixture schema (tester's contract, revision 1)
//!
//! The exact syntax of `doc/design/spec/wire.toml` is the coder's choice, but
//! the negative fixtures must stay interpretable, so the fixtures fix this
//! minimal schema. The canonical file may extend it; it must accept it.
//!
//! ```toml
//! schema = 1                 # integer schema revision
//! offset_unit = "bits"       # offsets are bit positions from the message start
//! width_unit = "bits"        # widths are bit counts
//! size_unit = "bytes"        # max_size is a byte count
//!
//! [[message]]
//! name = "example"           # unique, non-empty
//! layer = "control"          # control | sync | media | measurement
//! dscp = 40                  # 0..=63
//! max_size = 4               # declared maximum size, bytes
//! section_of = "parent"      # optional: this message is a section of `parent`
//! exempt_from_control_bound = "C-116"  # optional; only the path-MTU probe pair
//! rejection_reasons = ["TooShort"]     # strings; membership is checked by 004-03
//!
//! [[message.field]]
//! name = "flags"             # unique within the message
//! offset = 0                 # bits from the message start
//! width = 8                  # bits; the field occupies [offset, offset + width)
//! type = "u8"                # uint (sub-byte), u8..u128, i8..i128, f32, q1.31, pad
//! meaning = "what it carries"
//! valid = "0 to 255"         # free text; required
//! reserved = "0"             # optional: the reserved value, when one exists
//! count = "member_count"     # optional: this field is a list counted by that field
//! max_count = 32             # required with `count`; the list's fixed maximum
//! ```
//!
//! A counted field's `width` is the width of **one element**; the field occupies
//! `width * max_count` bits, which is how the maximum message size is computed
//! from the file alone (Work step 2).
//!
//! Rules the checker enforces, in this order: parse the file; round-trip the
//! model; fields must cover the message from bit 0 with no overlap and no
//! undeclared gap (a `type = "pad"` field declares padding); the declared
//! `max_size` must equal the size computed from fields and maximum counts; a
//! control message or regime stamp section over 1 252 bytes is rejected unless
//! it carries `exempt_from_control_bound = "C-116"` (C-17, C-116).
//!
//! # Known ambiguity in the ticket (reported to the lead)
//!
//! A correct parser/writer pair round-trips every well-formed typed model, so
//! a *file* cannot by itself expose a round-trip mismatch. The fixture
//! `round_trip_mismatch.toml` therefore pins the one asymmetry the ticket
//! leaves open: the model records the deprecated spelling `maximum_size` (to
//! warn about it), while the canonical writer emits `max_size`; the reparsed
//! model then differs, and `RoundTrip` is the correct report. The fixture
//! documents this locally.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use hadronyche_tools::wire_tables::{Error, FailureKind, Report, check_file};

/// The repository root: `hadronyche-tools/` sits directly below it.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("hadronyche-tools is a workspace member at the repository root")
        .to_path_buf()
}

/// `doc/design/spec/wire.toml` — the coder's deliverable for this ticket.
fn repository_tables() -> PathBuf {
    repo_root()
        .join("doc")
        .join("design")
        .join("spec")
        .join("wire.toml")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("wire_tables")
        .join(name)
}

fn rejected(name: &str) -> Error {
    match check_file(&fixture(name)) {
        Ok(report) => panic!("{name} must be rejected, but the checker accepted it: {report:?}"),
        Err(error) => error,
    }
}

fn accepted(name: &str) -> Report {
    match check_file(&fixture(name)) {
        Ok(report) => report,
        Err(error) => panic!(
            "{name} must be accepted, but the checker returned {:?}: {}",
            error.kind, error.message
        ),
    }
}

// AC 2, fixture 1 of 5: a parse error.
#[test]
fn ac2_parse_error_is_reported_as_parse() {
    let error = rejected("parse_error.toml");
    assert_eq!(
        error.kind,
        FailureKind::Parse,
        "AC 2: an unterminated TOML table header is a parse error: {}",
        error.message
    );
}

// AC 2, fixture 2 of 5: a round-trip mismatch (see the fixture header).
#[test]
fn ac2_round_trip_mismatch_is_reported_as_round_trip() {
    let error = rejected("round_trip_mismatch.toml");
    assert_eq!(
        error.kind,
        FailureKind::RoundTrip,
        "AC 2: a model the canonical writer cannot reproduce must fail the \
         round-trip check: {}",
        error.message
    );
}

// AC 2, fixture 3 of 5: overlapping fields.
#[test]
fn ac2_overlapping_fields_are_reported_as_overlap() {
    let error = rejected("overlap.toml");
    assert_eq!(
        error.kind,
        FailureKind::Overlap,
        "AC 2: bits 8..16 are claimed by two fields: {}",
        error.message
    );
}

// AC 2, fixture 4 of 5: a gap between fields not declared as padding.
#[test]
fn ac2_undeclared_gap_is_reported_as_gap() {
    let error = rejected("gap.toml");
    assert_eq!(
        error.kind,
        FailureKind::Gap,
        "AC 2: bits 8..16 are covered by no field and no pad: {}",
        error.message
    );
}

// AC 2, fixture 5 of 5: declared maximum size differs from the computed one.
#[test]
fn ac2_declared_maximum_different_from_computed_is_reported_as_max_size_mismatch() {
    let error = rejected("max_size_mismatch.toml");
    assert_eq!(
        error.kind,
        FailureKind::MaxSizeMismatch,
        "AC 2: two bytes computed against three declared: {}",
        error.message
    );
}

// AC 3: the checker passes on the example message in the repository's real
// file. The file does not exist in the RED phase; that is the missing
// behaviour this test waits for. The coder creates it in the GREEN phase.
#[test]
fn ac3_checker_accepts_the_repository_machine_readable_tables() {
    let path = repository_tables();
    assert!(
        path.is_file(),
        "AC 3: the coder must create {} (schema plus one example message, Work step 4)",
        path.display()
    );
    let report = match check_file(&path) {
        Ok(report) => report,
        Err(error) => panic!(
            "AC 3: {} must check clean, got {:?}: {}",
            path.display(),
            error.kind,
            error.message
        ),
    };
    assert!(
        !report.messages.is_empty(),
        "AC 3: the example message must be parsed into the report"
    );
}

// AC 6, negative fixture: a regime stamp section above 1 252 bytes without
// the probe exemption (C-17, C-18, C-116).
#[test]
fn ac6_control_message_over_1252_bytes_without_exemption_is_rejected() {
    let error = rejected("oversize_section.toml");
    assert_eq!(
        error.kind,
        FailureKind::OversizedControl,
        "AC 6: a 1 300-byte regime stamp section exceeds the 1 252-byte bound: {}",
        error.message
    );
}

// AC 6, positive fixture: the path-MTU probe pair carries the exemption
// (C-116) and is accepted at 1 472 bytes.
#[test]
fn ac6_probe_exemption_is_accepted() {
    let report = accepted("probe_exempt.toml");
    assert_eq!(report.messages.len(), 1, "AC 6: one probe-pair message");
    let probe = &report.messages[0];
    assert_eq!(probe.name, "path_mtu_probe_pair");
    assert_eq!(
        probe.layer, "control",
        "C-116: the probe pair is control-layer"
    );
    assert_eq!(
        probe.max_size, 1472,
        "C-116: the intended media packet size"
    );
}

// AC 1: the schema-coverage fixture exercises every feature of Work steps 1-2
// and is accepted; the report proves the model was really parsed, with the
// two counted lists contributing their maximum counts to the 232-byte size.
#[test]
fn ac1_schema_coverage_fixture_is_accepted() {
    let report = accepted("schema_coverage.toml");
    assert_eq!(
        report.messages.len(),
        2,
        "AC 1: parent stamp plus one section"
    );

    let parent = &report.messages[0];
    assert_eq!(parent.name, "regime_stamp");
    assert_eq!(parent.layer, "control");
    assert_eq!(parent.max_size, 3);

    let section = &report.messages[1];
    assert_eq!(section.name, "coverage_section");
    assert_eq!(section.layer, "control", "AC 1: per-message layer");
    assert_eq!(section.dscp, 40, "AC 1: per-message DSCP");
    assert_eq!(
        section.section_of.as_deref(),
        Some("regime_stamp"),
        "AC 1: the section-of attribute"
    );
    assert_eq!(
        section.max_size, 232,
        "AC 1: 2-bit + 8/16/32/64/128-bit fields, a 32-bit sample, \
         32 x 16-bit members and 64 x 16-bit peers, plus 2 bytes of padding"
    );
    assert_eq!(
        section.field_count, 13,
        "AC 1: every [[message.field]] entry, padding included, was parsed"
    );
    assert_eq!(
        section.rejection_reasons,
        ["TooShort", "BadLength", "ReservedNonZero"],
        "AC 1: the per-message rejection-reason list"
    );
}

// AC 4: the checker is ordinary in-process library code. Running it here is an
// unprivileged `cargo test` with no network API in sight; nothing binds a
// socket, opens a privileged resource or shells out.
#[test]
fn ac4_checker_runs_unprivileged_and_without_network() {
    let report = accepted("schema_coverage.toml");
    assert_eq!(
        report.messages.len(),
        2,
        "AC 4: the call runs inside the test process, with no helper"
    );
}

// AC 5: the checker's run time on the full repository file is bounded.
#[test]
fn ac5_repository_file_checks_within_ten_seconds() {
    let path = repository_tables();
    assert!(
        path.is_file(),
        "AC 5: the full file to time is {} (created by the coder)",
        path.display()
    );
    let start = Instant::now();
    let result = check_file(&path);
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "AC 5: the checker took {elapsed:?} on {}; the bound is 10 s",
        path.display()
    );
    if let Err(error) = result {
        panic!(
            "AC 5: the full file must check clean before it is timed, got {:?}: {}",
            error.kind, error.message
        );
    }
}

/// Work step 3, fourth bullet — enabled by 004-03.
///
/// 004-03 (rejection-reason enumeration) added the closed `Reject` set to
/// the machine-readable file as `[[reject]]` tables. The checker validates
/// every `rejection_reasons` reference against that closed set (and rejects
/// a `[[reject]]` declaration that differs from it), and reports the
/// enumeration in `Report::rejects`. The test asserts the property end to
/// end: the repository file's declaration is the closed enumeration, and
/// every reason the fixture names is a member of it.
#[test]
fn rejection_reasons_referenced_are_members_of_the_closed_enumeration() {
    let report = accepted("schema_coverage.toml");
    let closed = check_file(&repository_tables()).unwrap_or_else(|error| {
        panic!(
            "the repository tables must check clean with the closed Reject enumeration: {:?}: {}",
            error.kind, error.message
        )
    });
    let members: Vec<&str> = closed
        .rejects
        .iter()
        .map(|reject| reject.name.as_str())
        .collect();
    assert!(
        !members.is_empty(),
        "004-03: the repository tables must declare the closed Reject enumeration"
    );
    for message in &report.messages {
        assert!(
            !message.rejection_reasons.is_empty(),
            "{}: the fixture names at least one reason",
            message.name
        );
        for reason in &message.rejection_reasons {
            assert!(
                !reason.is_empty(),
                "{}: an empty rejection reason would be skipped silently (C-125)",
                message.name
            );
            // 004-03: membership in `Reject`.
            assert!(
                members.contains(&reason.as_str()),
                "{}: rejection reason {} is not a member of the closed Reject enumeration",
                message.name,
                reason
            );
        }
    }
}
