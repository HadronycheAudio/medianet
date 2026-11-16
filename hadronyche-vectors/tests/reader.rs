//! Acceptance tests for ticket 007a-03 — the `no_std` vector reader.
//!
//! # API contract (binding where marked BINDING)
//!
//! The reader API is the one `src/reader.rs` implements; the names below are
//! the ones these tests use.
//!
//! * `reader::all()`, `reader::of_kind(kind)`, `reader::find(id)` and
//!   `reader::with_bytes(bytes)` enumerate the **conformance set**: every
//!   vector under the crate's `vectors/` directory (007a-05 onward).
//! * **BINDING:** `reader::examples()` and `reader::examples_of_kind(kind)`
//!   enumerate the six example fixtures of 007a-02, which are test fixtures
//!   and not conformance vectors. They are always embedded, in a table of
//!   their own. **No cargo feature changes what any enumeration yields**, and
//!   no example id is reachable through `all()`, `of_kind()`, `find()` or
//!   `with_bytes()`.
//! * **BINDING:** `reader::Reason` (with `Reason::ALL`, `name()`, `value()`
//!   and `from_name()`) is pinned to the closed 004-03 enumeration of
//!   `doc/design/spec/wire.toml`; `tests/reason_pin.rs` reads that file.
//!
//! Each `Vector` yields `kind()`, `id()`, `spec_version()` (unchanged from the
//! file), `bytes()` (the `.bin` content of a wire vector), and `outcome()`:
//! `Outcome::Valid` for a wire-valid vector, `Outcome::Rejected(r)` for a
//! wire-invalid one with `r.name()` the file's `reason`, `Outcome::NotWire`
//! for the other kinds. The types other than those named here are not bound.
//!
//! # What these tests fix (the ticket's words, nothing more)
//!
//! * AC 2: `examples()` yields exactly the six example fixtures of 007a-02,
//!   with `id` and `spec_version` unchanged.
//! * AC 3: for every example naming a `.bin` file, `bytes()` equals
//!   `std::fs::read` of that file, byte for byte.
//! * Work 1: a wire-valid example yields `Outcome::Valid`; a wire-invalid one
//!   yields `Outcome::Rejected` whose name is the `reason` its JSON declares.
//! * Work 2: each kind enumerates only its own vectors, for the conformance
//!   set and for the examples. The other kinds may be empty today.
//! * Work 3 and the DoD line on versions: every embedded vector, and every
//!   example, has a non-empty `id` and `spec_version`.
//! * Separation (BINDING): no example id is reachable through the conformance
//!   lookups.
//!
//! The fixture side reuses the 007a-02 schema module (`support`), so the file
//! names and counts are the ones 007a-02 fixed.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use hadronyche_vectors::reader::{self, Kind, Outcome, Vector};
use serde_json::Value;
use support::{Kind as FixtureKind, REJECT_REASONS};

/// One example fixture of 007a-02: its JSON file, the parsed document, and the
/// kind its directory names.
struct Example {
    path: PathBuf,
    json: Value,
    kind: FixtureKind,
}

/// Recursively collect every `example.json` below `dir`.
fn collect_examples(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .path();
        if path.is_dir() {
            collect_examples(&path, out);
        } else if path.file_name().is_some_and(|n| n == "example.json") {
            out.push(path);
        }
    }
}

/// The six example fixtures of 007a-02, sorted by path.
fn examples() -> Vec<Example> {
    let mut paths = Vec::new();
    collect_examples(&support::examples_root(), &mut paths);
    paths.sort();
    assert_eq!(
        paths.len(),
        6,
        "007a-02 fixes six example fixtures below {}; found {:?}",
        support::examples_root().display(),
        paths
    );
    paths
        .into_iter()
        .map(|path| {
            let text =
                fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let json = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("{}: not valid JSON: {e}", path.display()));
            let kind = FixtureKind::from_path(&path)
                .unwrap_or_else(|| panic!("{}: names no vector kind", path.display()));
            Example { path, json, kind }
        })
        .collect()
}

/// A string field of a fixture document, panicking with the file's name.
fn str_field<'a>(example: &'a Example, key: &str) -> &'a str {
    example
        .json
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{}: no string field `{key}`", example.path.display()))
}

/// The reader's name for a fixture kind.
fn api_kind(kind: FixtureKind) -> Kind {
    match kind {
        FixtureKind::WireValid => Kind::WireValid,
        FixtureKind::WireInvalid => Kind::WireInvalid,
        FixtureKind::Control => Kind::Control,
        FixtureKind::Timeline => Kind::Timeline,
        FixtureKind::Transport => Kind::Transport,
        FixtureKind::Reference => Kind::Reference,
    }
}

/// The `.bin` bytes an example names, if it names one.
fn bin_bytes(example: &Example) -> Option<Vec<u8>> {
    let bin = example.json.get("bin").and_then(Value::as_str)?;
    let path = example
        .path
        .parent()
        .unwrap_or_else(|| panic!("{}: no parent", example.path.display()))
        .join(bin);
    Some(fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
}

/// Every vector of the conformance set.
fn conformance() -> Vec<Vector> {
    reader::all().collect()
}

/// Every example the reader embeds.
fn embedded_examples() -> Vec<Vector> {
    reader::examples().collect()
}

/// The single vector with this id in `list`; a missing or duplicated id is a
/// failure naming the id.
fn find_in<'a>(list: &'a [Vector], id: &str) -> &'a Vector {
    let hits: Vec<&Vector> = list.iter().filter(|v| v.id() == id).collect();
    assert_eq!(
        hits.len(),
        1,
        "the reader must enumerate the example {id:?} exactly once; it yielded {}",
        hits.len()
    );
    hits[0]
}

// ------------------------------------------------------------- AC 2

/// AC 2: `examples()` yields exactly the six example fixtures, each with its
/// `id` and `spec_version` unchanged, and the ids are unique.
#[test]
fn ac2_examples_enumeration_yields_each_example_with_id_and_spec_version_unchanged() {
    let list = embedded_examples();
    let fixtures = examples();

    let mut got: Vec<&str> = list.iter().map(|v| v.id()).collect();
    got.sort_unstable();
    let mut want: Vec<&str> = fixtures.iter().map(|e| str_field(e, "id")).collect();
    want.sort_unstable();
    assert_eq!(
        got, want,
        "examples() must yield exactly the six 007a-02 example ids"
    );

    for example in &fixtures {
        let id = str_field(example, "id");
        let spec = str_field(example, "spec_version");
        let vector = find_in(&list, id);
        assert_eq!(
            vector.spec_version(),
            spec,
            "{}: spec_version must come back unchanged",
            example.path.display()
        );
        assert_eq!(
            vector.kind(),
            api_kind(example.kind),
            "{}: the reader must report the kind the directory names",
            example.path.display()
        );
    }
}

// ------------------------------------------------------------- AC 3

/// AC 3: for every example that names a `.bin` file, the reader's bytes equal
/// the file's content byte for byte.
#[test]
fn ac3_bytes_of_every_bin_example_equal_the_file_byte_for_byte() {
    let list = embedded_examples();
    let mut checked = 0;
    for example in examples() {
        let Some(expected) = bin_bytes(&example) else {
            continue;
        };
        let id = str_field(&example, "id");
        let got = find_in(&list, id).bytes();
        assert_eq!(
            got.len(),
            expected.len(),
            "{id}: the reader returned {} bytes, the file holds {}",
            got.len(),
            expected.len()
        );
        assert_eq!(
            got,
            expected.as_slice(),
            "{id}: the reader's bytes differ from the file"
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "no example names a .bin file; 007a-02 fixes at least one wire example"
    );
}

/// Work 1: a wire-valid example yields `Outcome::Valid`; a wire-invalid one
/// yields `Outcome::Rejected` named by its declared `reason`, which is a
/// member of the closed 004-03 enumeration.
#[test]
fn work1_wire_examples_decode_or_reject_with_their_declared_reason() {
    let list = embedded_examples();
    let wire: Vec<Example> = examples()
        .into_iter()
        .filter(|e| matches!(e.kind, FixtureKind::WireValid | FixtureKind::WireInvalid))
        .collect();
    assert_eq!(
        wire.len(),
        2,
        "007a-02 fixes one wire-valid and one wire-invalid example"
    );

    for example in &wire {
        let id = str_field(example, "id");
        match (example.kind, find_in(&list, id).outcome()) {
            (FixtureKind::WireValid, Outcome::Valid(_)) => {}
            (FixtureKind::WireInvalid, Outcome::Rejected(rejection)) => {
                let declared = str_field(example, "reason");
                assert_eq!(
                    rejection.name(),
                    declared,
                    "{}: the rejection reason must be the one the file declares",
                    example.path.display()
                );
                assert!(
                    REJECT_REASONS.contains(&declared),
                    "{}: the declared reason {declared:?} is outside the 004-03 enumeration",
                    example.path.display()
                );
            }
            _ => panic!(
                "{}: a {} vector must yield {}",
                example.path.display(),
                example.kind.name(),
                if example.kind == FixtureKind::WireValid {
                    "a decoded structure (Outcome::Valid)"
                } else {
                    "a rejection reason (Outcome::Rejected)"
                }
            ),
        }
    }
}

// ------------------------------------------------------------- Work 2

/// Work 2: each kind enumerates only its own vectors, both in the conformance
/// set and in the examples; every conformance vector belongs to exactly one
/// kind; and each example is found under the kind its directory names.
#[test]
fn work2_each_kind_enumerates_only_its_own_vectors() {
    let kinds = Kind::ALL;

    let mut total = 0;
    for kind in kinds {
        let vectors: Vec<Vector> = reader::of_kind(kind).collect();
        for v in &vectors {
            assert_eq!(
                v.kind(),
                kind,
                "of_kind({kind:?}) yielded {:?}, which is of another kind",
                v.id()
            );
        }
        total += vectors.len();

        let examples_of: Vec<Vector> = reader::examples_of_kind(kind).collect();
        for v in &examples_of {
            assert_eq!(
                v.kind(),
                kind,
                "examples_of_kind({kind:?}) yielded {:?}, which is of another kind",
                v.id()
            );
        }
    }
    assert_eq!(
        total,
        conformance().len(),
        "every conformance vector must belong to exactly one kind"
    );

    for example in examples() {
        let id = str_field(&example, "id");
        let kind = api_kind(example.kind);
        assert!(
            reader::examples_of_kind(kind).any(|v| v.id() == id),
            "{}: the example {id:?} is not enumerated under {kind:?}",
            example.path.display()
        );
    }
}

// ------------------------------------------------------- separation

/// BINDING: no example id is reachable through the conformance lookups
/// `all()`, `of_kind()`, `find()` or `with_bytes()`, whatever the feature set.
#[test]
fn examples_are_not_reachable_through_the_conformance_lookups() {
    let all = conformance();
    for example in examples() {
        let id = str_field(&example, "id");
        assert!(
            all.iter().all(|v| v.id() != id),
            "{id}: an example is reachable through all()"
        );
        assert!(
            reader::find(id).is_none(),
            "{id}: an example is reachable through find()"
        );
        for kind in Kind::ALL {
            assert!(
                reader::of_kind(kind).all(|v| v.id() != id),
                "{id}: an example is reachable through of_kind({kind:?})"
            );
        }
        if let Some(bytes) = bin_bytes(&example) {
            assert!(
                reader::with_bytes(&bytes).all(|v| v.id() != id),
                "{id}: an example is reachable through with_bytes()"
            );
        }
    }
}

// ------------------------------------------------------------- Work 3

/// Work 3 and the DoD line on versions: every conformance vector and every
/// example carries a non-empty `id` and a non-empty `spec_version`.
#[test]
fn work3_every_embedded_vector_carries_a_nonempty_id_and_spec_version() {
    for v in conformance().into_iter().chain(embedded_examples()) {
        assert!(
            !v.id().trim().is_empty(),
            "an embedded vector has an empty id"
        );
        assert!(
            !v.spec_version().trim().is_empty(),
            "the embedded vector {:?} has an empty spec_version",
            v.id()
        );
    }
}
