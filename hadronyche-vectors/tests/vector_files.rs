//! Acceptance tests for ticket 007a-02 — vector file formats and version
//! stamp.
//!
//! # The tester's contract (fixed here; the coder implements it)
//!
//! The six vector schemas are typed Rust definitions in [`support`] (this test
//! target, where `std` is allowed; the crate itself stays `#![no_std]`), and
//! this file validates files against them. The contract is:
//!
//! * Kind is fixed by path (`wire/valid`, `wire/invalid`, `control`,
//!   `timeline`, `transport`, `reference`), as in the ticket's Context. The
//!   validator is [`support::validate_file`] and [`support::validate_tree`];
//!   the typed definitions are `WireValid`, `WireInvalid`, `Control`,
//!   `Timeline`, `Transport` and `Reference`.
//! * Every file carries `spec_version` (non-empty, opaque: epic 004's open
//!   sub-question owns its format; not the hello's regime-stamp version,
//!   C-18) and `id` (non-empty, unique across the crate).
//! * An invalid vector's `reason` is a member of the closed `Reject`
//!   enumeration of ticket 004-03: `TooShort` 0, `BadMagic` 1,
//!   `VersionMismatch` 2, `ForeignDomain` 3, `UnconfiguredDomain` 4,
//!   `UnknownStream` 5, `UnadmittedSource` 6, `BadLength` 7, `BadChecksum` 8,
//!   `BadPacketIndex` 9, `WrongEpoch` 10, `OversizeControl` 11,
//!   `ReservedNonZero` 12. Values 13..=255 are not valid names.
//! * The per-input time bound is C-164: linear in packet length, at most
//!   10 µs per packet. The bound is carried per file, so a later L1 revision
//!   needs no schema change.
//! * A control scenario's expected `state` carries the C-159 counters
//!   (`received`, `late`, `missing`, `marked_gap`, `other_epoch`, `too_early`,
//!   `duplicates`, `incomplete_reassemblies`, `demoted_rounds`,
//!   `interrupted_events`, `dead`) and a health state (C-109 `Demoted`
//!   included).
//!
//! The field tables per kind are the doc comments on the six typed
//! definitions in `tests/support/mod.rs`; that module is the normative schema.
//!
//! # Fixture ownership (the red/green split)
//!
//! | path below `tests/fixtures/vectors/` | owner | state |
//! |---|---|---|
//! | `examples/wire/valid/example.{json,bin}` | **coder** | missing: AC 1, AC 3 |
//! | `examples/wire/invalid/example.{json,bin}` | **coder** | missing: AC 3, AC 6 |
//! | `examples/control/example.json` | **coder** | missing: AC 3, AC 4 |
//! | `examples/timeline/example.json` | **coder** | missing: AC 3 |
//! | `examples/transport/example.json` | **coder** | missing: AC 3 |
//! | `examples/reference/example.json` | **coder** | missing: AC 3 |
//! | `negative/no-spec-version/control/example.json` | tester | committed red |
//! | `negative/duplicate-id/control/{a,b}.json` | tester | committed red |
//! | `negative/bad-reason/wire/invalid/example.{json,bin}` | tester | committed red |
//!
//! The six **example** fixtures are the coder's green work: they must be small
//! hand-written samples (ticket Work 7), not generated vectors. The three
//! **negative** fixtures are the tester's and must stay invalid for exactly
//! the reason they name. `vectors/**` (the crate's data root, 007a-05 onward)
//! is validated by [`work6_the_crate_data_root_validates`]; it is empty here.
//!
//! # Acceptance criteria → tests
//!
//! | AC | test |
//! |---|---|
//! | 1 (6 schemas, one per kind) | `ac1_*` |
//! | 2 (3 negative fixtures fail) | `ac2_*` |
//! | 3 (6 examples pass) | `ac3_*` |
//! | 4 (control carries a C-159 counter and a health state) | `ac4_*` |
//! | 5 (plain `cargo test`, unprivileged) | `ac5_*` |
//! | 6 (C-164 time bound in the invalid example) | `ac6_*` |
//!
//! # Known ambiguity in the ticket (reported to the lead)
//!
//! The ticket says the schemas are "a JSON schema (or equivalent typed
//! definitions in the crate, checked by test)" and, separately, that the
//! examples go "under a test fixtures directory". It does not say whether the
//! schema itself is code or data. This test target fixes it: the schema is
//! typed Rust in `tests/support/`, so it is checked by the compiler, can
//! express cross-field rules (the reason enumeration, the C-159 set, the
//! C-164 ceiling) that JSON Schema would need custom keywords for, and needs
//! no new dependency in the `no_std` crate. The crate's `#![no_std]` property
//! is untouched: `tests/` targets are hosted-only.

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use support::{
    C159_COUNTERS, Control, FIXTURES_ROOT, Health, Kind, REJECT_REASONS, Reference, SampleFormat,
    VECTORS_ROOT, Validated, WireInvalid, examples_root, validate_file, validate_json,
    validate_tree,
};

/// Read one fixture as JSON; a missing or unparsable file is a test failure
/// naming the file.
fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{}: not valid JSON: {e}", path.display()))
}

/// The coder's example fixture of `kind`, as JSON.
fn example(kind: Kind) -> Value {
    read_json(&support::example_path(kind))
}

/// A negative-fixture root: `tests/fixtures/vectors/negative/<case>/`.
fn negative(case: &str) -> PathBuf {
    Path::new(FIXTURES_ROOT).join("negative").join(case)
}

/// Unwrap a rejection, panicking with the acceptance when the input passed.
fn rejection(result: Result<Vec<Validated>, Vec<String>>, what: &str) -> Vec<String> {
    match result {
        Ok(docs) => panic!(
            "{what} must FAIL validation, but {} file(s) passed: {:?}",
            docs.len(),
            docs.iter().map(|d| &d.path).collect::<Vec<_>>()
        ),
        Err(errors) => errors,
    }
}

/// Assert that at least one error mentions every needle, and show all errors
/// when one is missing.
fn assert_mentions(errors: &[String], needles: &[&str], what: &str) {
    for needle in needles {
        assert!(
            errors.iter().any(|e| e.contains(needle)),
            "{what}: no error mentions {needle:?}; errors were:\n{}",
            errors.join("\n")
        );
    }
}

// -------------------------------------------------------------------- AC 1

/// AC 1: six schemas exist, one per kind, and the coder's six example fixtures
/// exist at the paths this target fixes.
#[test]
fn ac1_six_kinds_have_one_example_fixture_each() {
    assert_eq!(Kind::ALL.len(), 6, "six vector kinds, one schema each");
    for kind in Kind::ALL {
        let path = support::example_path(kind);
        assert!(
            path.is_file(),
            "{}: the {} schema has no example fixture (ticket Work 7: one small \
             hand-written example of each kind); expected a JSON file at this path",
            path.display(),
            kind.name()
        );
    }
}

/// AC 1: each example fixture is of the kind its directory names, so the six
/// schemas are exercised one per kind.
#[test]
fn ac1_each_example_validates_against_its_own_kind() {
    for kind in Kind::ALL {
        let path = support::example_path(kind);
        assert_eq!(
            Kind::from_path(&path),
            Some(kind),
            "{}: the path must name the {} kind",
            path.display(),
            kind.name()
        );
        let doc = match validate_file(&path) {
            Ok(doc) => doc,
            Err(errors) => panic!(
                "{} must validate as {}; errors:\n{}",
                path.display(),
                kind.name(),
                errors.join("\n")
            ),
        };
        assert_eq!(doc.kind, kind);
        assert!(!doc.id.trim().is_empty(), "{}: empty id", path.display());
        assert!(
            !doc.spec_version.trim().is_empty(),
            "{}: empty spec_version",
            path.display()
        );
    }
}

// -------------------------------------------------------------------- AC 2

/// AC 2, negative fixture 1 of 3: a file with no `spec_version` must fail.
#[test]
fn ac2_a_file_without_spec_version_is_rejected() {
    let root = negative("no-spec-version");
    let file = root.join("control").join("example.json");
    let value = read_json(&file);
    assert!(
        value.get("spec_version").is_none(),
        "{}: the negative fixture must genuinely lack spec_version",
        file.display()
    );

    let errors = rejection(validate_tree(&root), "a file without spec_version");
    assert_mentions(&errors, &["spec_version"], "a file without spec_version");

    // The same document is otherwise well-formed, so the missing stamp is the
    // only reason it fails.
    let mut repaired = value.clone();
    repaired["spec_version"] = json!("1");
    assert!(
        validate_json(Kind::Control, &repaired).is_ok(),
        "the negative fixture must differ from the example schema only in its missing \
         spec_version; adding one made it invalid anyway"
    );
}

/// AC 2, negative fixture 2 of 3: two files sharing an `id` must fail, because
/// ids are unique across the crate.
#[test]
fn ac2_duplicate_id_is_rejected() {
    let root = negative("duplicate-id");
    let a = read_json(&root.join("control").join("a.json"));
    let b = read_json(&root.join("control").join("b.json"));
    assert_eq!(
        a["id"],
        b["id"],
        "{}: the two negative fixtures must share an id",
        root.display()
    );

    let errors = rejection(validate_tree(&root), "a duplicate id");
    assert_mentions(
        &errors,
        &["duplicate id", "ids are unique"],
        "a duplicate id",
    );
}

/// AC 2, negative fixture 3 of 3: a `reason` outside the closed `Reject`
/// enumeration of ticket 004-03 must fail.
#[test]
fn ac2_reason_outside_the_004_03_enumeration_is_rejected() {
    let root = negative("bad-reason");
    let file = root.join("wire").join("invalid").join("example.json");
    let value = read_json(&file);
    let reason = value["reason"]
        .as_str()
        .expect("the negative fixture's `reason` is a string");
    assert!(
        !REJECT_REASONS.contains(&reason),
        "{}: the negative fixture's reason {reason:?} must NOT be in the 004-03 \
         enumeration, otherwise it proves nothing",
        file.display()
    );

    let errors = rejection(validate_tree(&root), "a reason outside 004-03");
    assert_mentions(
        &errors,
        &["reason", "closed `Reject` enumeration", "004-03"],
        "a reason outside 004-03",
    );
}

/// The tester's fixture set stays pinned to the enumeration landed by 004-03:
/// thirteen names, in value order, none of them duplicated.
#[test]
fn ac2_the_reject_enumeration_is_the_closed_004_03_set() {
    let expected = [
        "TooShort",
        "BadMagic",
        "VersionMismatch",
        "ForeignDomain",
        "UnconfiguredDomain",
        "UnknownStream",
        "UnadmittedSource",
        "BadLength",
        "BadChecksum",
        "BadPacketIndex",
        "WrongEpoch",
        "OversizeControl",
        "ReservedNonZero",
    ];
    assert_eq!(REJECT_REASONS, expected, "the closed 004-03 enumeration");
    let mut unique = REJECT_REASONS.to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), REJECT_REASONS.len());
}

// -------------------------------------------------------------------- AC 3

/// AC 3: the six example fixtures all pass validation, and the ids are unique
/// across the example set.
#[test]
fn ac3_the_six_examples_pass_validation() {
    let root = examples_root();
    let docs = match validate_tree(&root) {
        Ok(docs) => docs,
        Err(errors) => panic!(
            "every example fixture must pass; errors:\n{}",
            errors.join("\n")
        ),
    };

    assert_eq!(
        docs.len(),
        6,
        "expected one example per kind below {}; found {:?}",
        root.display(),
        docs.iter().map(|d| &d.path).collect::<Vec<_>>()
    );
    for kind in Kind::ALL {
        assert!(
            docs.iter().any(|d| d.kind == kind),
            "no {} example validated below {}",
            kind.name(),
            root.display()
        );
    }

    let mut ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), docs.len(), "the six example ids must be unique");
}

// -------------------------------------------------------------------- AC 4

/// AC 4: the control example carries at least one C-159 counter and a health
/// state (`Demoted`) in its expected state, and the schema constrains both.
#[test]
fn ac4_control_example_carries_c159_counters_and_demoted_health() {
    let value = example(Kind::Control);
    let doc: Control =
        serde_json::from_value(value.clone()).expect("the control example matches the schema");

    assert!(!doc.steps.is_empty(), "the control example has no steps");
    let mut counter_step = None;
    for step in &doc.steps {
        for name in step.expected.state.counters.keys() {
            assert!(
                C159_COUNTERS.contains(&name.as_str()),
                "the control example uses {name:?}, which is not a C-159 counter (the \
                 schema must require the named set: {})",
                C159_COUNTERS.join(", ")
            );
        }
        if !step.expected.state.counters.is_empty() {
            counter_step = Some(&step.expected.state);
        }
    }
    assert!(
        counter_step.is_some(),
        "the control example's expected state must carry at least one C-159 counter \
         (AC 4)"
    );
    assert!(
        doc.steps
            .iter()
            .any(|s| s.expected.state.health == Health::Demoted),
        "the control example's expected state must carry the health state Demoted \
         (C-109, AC 4); health values seen: {:?}",
        doc.steps
            .iter()
            .map(|s| s.expected.state.health)
            .collect::<Vec<_>>()
    );

    // The schema *requires* the named set rather than accepting any key: an
    // unknown counter is rejected in memory, beside the example.
    let mut mutated = value;
    mutated["steps"][0]["expected"]["state"]["counters"]["not_a_c159_counter"] = json!(1);
    assert!(
        validate_json(Kind::Control, &mutated).is_err(),
        "the control schema must reject a counter name outside C-159"
    );

    // The C-160 lateness percentiles are part of the same state, when present.
    let mut mutated = read_json(&support::example_path(Kind::Control));
    mutated["steps"][0]["expected"]["state"]["lateness_percentiles"] = json!({
        "p50_ns": 10,
        "p99_ns": 5, // deliberately out of order
        "p999_ns": 30,
        "max_ns": 40
    });
    assert!(
        validate_json(Kind::Control, &mutated).is_err(),
        "the control schema must reject unordered lateness percentiles (C-160)"
    );
}

// -------------------------------------------------------------------- AC 5

/// AC 5: the validation runs in a plain `cargo test`, unprivileged. The test
/// target opens files with `std::fs` only: no socket, no device, no root.
/// Where the kernel exposes it, this also asserts the effective uid is not 0,
/// the same rule `scripts/repo-state.d/007a-01.sh` AC 4 applies to the suite.
#[test]
fn ac5_runs_as_a_plain_unprivileged_cargo_test() {
    // The fixtures are readable with no privilege escalation.
    let entries =
        std::fs::read_dir(FIXTURES_ROOT).unwrap_or_else(|e| panic!("{FIXTURES_ROOT}: {e}"));
    assert!(entries.count() > 0, "{FIXTURES_ROOT} is empty");

    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        let effective_uid = status
            .lines()
            .find(|line| line.starts_with("Uid:"))
            .and_then(|line| line.split_whitespace().nth(2));
        if let Some(uid) = effective_uid {
            assert_ne!(
                uid, "0",
                "AC 5: the vector tests must run unprivileged (effective uid 0)"
            );
        }
    }
}

// -------------------------------------------------------------------- AC 6

/// AC 6: the invalid example carries a per-input time bound equal to the C-164
/// values, and the schema refuses a bound above them.
#[test]
fn ac6_invalid_example_carries_the_c164_time_bound() {
    let doc: WireInvalid = serde_json::from_value(example(Kind::WireInvalid))
        .expect("the wire-invalid example matches the schema");

    let bound = doc
        .expected
        .time_bound
        .as_ref()
        .expect("the invalid example must carry `expected.time_bound` (AC 6, C-164)");
    assert_eq!(
        bound.per_packet_us, 10,
        "the invalid example's bound must be the C-164 value: at most 10 µs per packet"
    );
    assert!(
        bound.linear_in_length,
        "the invalid example's bound must be linear in packet length (C-164)"
    );

    // At least one of the schema's optional bad-input fields is used, so the
    // file proves the schema can express AC-23's required outcomes.
    assert!(
        !doc.expected.counter_increments.is_empty() || !doc.expected.events.is_empty(),
        "the invalid example must use at least one optional field: the expected counter \
         increments (one per input and reason) or the expected named events (AC-23)"
    );
    assert!(
        doc.expected.no_state_change,
        "a bad input never changes state (AC-23)"
    );

    // The C-164 ceiling is enforced by the schema, not only asserted here.
    let mut value = example(Kind::WireInvalid);
    value["expected"]["time_bound"]["per_packet_us"] = json!(11);
    assert!(
        validate_json(Kind::WireInvalid, &value).is_err(),
        "the invalid schema must reject a time bound above 10 µs per packet (C-164)"
    );
}

// ------------------------------------------------------- shared invariants

/// Work 2, shared by every kind: `spec_version` is a non-empty opaque string
/// and `id` is non-empty and unique. `validate_tree` enforces uniqueness; this
/// test pins the presence on the six examples, and that the stamp is not the
/// regime-stamp version number of the hello (C-18) by requiring a string.
#[test]
fn every_example_carries_a_non_empty_spec_version_and_id() {
    for kind in Kind::ALL {
        let value = example(kind);
        for field in ["spec_version", "id"] {
            let text = value[field]
                .as_str()
                .unwrap_or_else(|| panic!("{}: `{field}` must be a string", kind.name()));
            assert!(
                !text.trim().is_empty(),
                "{}: `{field}` must be non-empty",
                kind.name()
            );
        }
    }
}

/// An unknown field is a schema error: the six schemas are closed, so a typo
/// cannot silently pass.
#[test]
fn schemas_reject_unknown_fields() {
    let mut value = example(Kind::Timeline);
    value["not_a_schema_field"] = json!(1);
    assert!(
        validate_json(Kind::Timeline, &value).is_err(),
        "the timeline schema must reject an unknown field"
    );
}

/// Work 5: the reference schema fixes the sample format (`f32` or `q1.31`),
/// the input script, the expected output hash and the gap-related counts.
#[test]
fn reference_schema_fixes_format_script_hash_and_gap_counts() {
    let doc: Reference =
        serde_json::from_value(example(Kind::Reference)).expect("the reference example matches");
    assert!(
        matches!(doc.sample_format, SampleFormat::F32 | SampleFormat::Q1_31),
        "the reference sample format is f32 or q1.31"
    );
    assert!(
        !doc.input_script.is_empty(),
        "the reference example must script at least one input"
    );
    assert!(
        !doc.expected_output_hash.trim().is_empty(),
        "the reference example must carry an expected output hash (007b-18 owns the algorithm)"
    );
    // `gap_counts` is mandatory in the type, so deserializing proves it is
    // present; both counts are u64.

    // The sample format is a closed set: a third format is rejected.
    let mut value = example(Kind::Reference);
    value["sample_format"] = json!("f64");
    assert!(
        validate_json(Kind::Reference, &value).is_err(),
        "the reference schema must accept only `f32` and `q1.31`"
    );
}

/// Ticket Work 6: the validation covers every file under the crate's data
/// root, `hadronyche-vectors/vectors/`. Later tickets fill it; today it holds
/// placeholders only, so the check must pass on an empty set (and must not
/// crash on the `.gitkeep` files).
#[test]
fn work6_the_crate_data_root_validates() {
    let docs = match validate_tree(Path::new(VECTORS_ROOT)) {
        Ok(docs) => docs,
        Err(errors) => panic!(
            "every vector under {} must validate; errors:\n{}",
            VECTORS_ROOT,
            errors.join("\n")
        ),
    };
    for doc in &docs {
        assert!(
            !doc.spec_version.trim().is_empty(),
            "{}: empty spec_version",
            doc.path.display()
        );
    }
}

/// Path kind inference is the schema's selector: a path that names no kind is
/// refused rather than guessed.
#[test]
fn a_path_without_a_kind_is_refused() {
    let path = Path::new(FIXTURES_ROOT)
        .join("negative")
        .join("orphan.json");
    let error = validate_file(&path).expect_err("a path with no kind must not validate");
    assert_mentions(&error, &["cannot infer the vector kind"], "an orphan path");
}
