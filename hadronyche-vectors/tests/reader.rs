//! Acceptance tests for ticket 007a-03 — the `no_std` vector reader.
//!
//! # PROPOSED API — the coder may rename; the acceptance assertions are what bind
//!
//! These tests name an API that does not exist yet, so this target fails to
//! compile until the coder lands it (the red state). The proposal, derived
//! from the ticket's Work 1 to 3 ("an enumeration over embedded vectors
//! yielding `id`, `spec_version`, the raw bytes, and either the decoded
//! structure or the rejection reason"):
//!
//! ```text
//! // hadronyche-vectors/src/reader.rs (no_std; data embedded at build time)
//! pub mod reader {
//!     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//!     pub enum Kind { WireValid, WireInvalid, Control, Timeline, Transport, Reference }
//!
//!     /// Every embedded vector, in a fixed order.
//!     pub fn all() -> impl Iterator<Item = Vector>;
//!     /// The embedded vectors of one kind.
//!     pub fn of_kind(kind: Kind) -> impl Iterator<Item = Vector>;
//!
//!     pub struct Vector;
//!     impl Vector {
//!         pub fn kind(&self) -> Kind;
//!         pub fn id(&self) -> &str;
//!         pub fn spec_version(&self) -> &str;
//!         pub fn bytes(&self) -> &[u8];      // the .bin content for wire vectors
//!         pub fn outcome(&self) -> Outcome;
//!     }
//!
//!     pub enum Outcome { Valid(Decoded), Rejected(Rejection) }
//!     pub struct Decoded;                    // opaque to these tests
//!     pub struct Rejection;
//!     impl Rejection { pub fn name(&self) -> &'static str; } // "TooShort", ...
//! }
//! ```
//!
//! The data the reader sees is whatever the crate embeds at build time. The
//! tests do not fix which directory that is; they fix only that the six
//! example fixtures of 007a-02 are among the enumeration.
//!
//! # What these tests fix (the ticket's words, nothing more)
//!
//! * AC 2: enumerating the embedded vectors yields each of the six example
//!   fixtures of 007a-02, with its `id` and `spec_version` unchanged. The
//!   check is a superset check: extra embedded vectors are allowed, a missing
//!   example is not.
//! * AC 3: for every example that names a `.bin` file, the bytes the reader
//!   returns equal `std::fs::read` of that file, byte for byte.
//! * Work 1: a wire-valid example yields `Outcome::Valid`; a wire-invalid one
//!   yields `Outcome::Rejected` whose name is the `reason` its JSON declares,
//!   and that name is one of the closed 004-03 enumeration (`support`).
//! * Work 2: each of the six kinds can be enumerated on its own, and only
//!   yields vectors of that kind. The other four kinds may be empty today.
//! * Work 3 (and the DoD line on spec versions): every embedded vector has a
//!   non-empty `id` and `spec_version`.
//!
//! The fixture side reuses the 007a-02 schema module (`support`), so the
//! file names and counts are the ones 007a-02 fixed, not new ones.

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

/// Every vector the reader embeds.
fn embedded() -> Vec<Vector> {
    reader::all().collect()
}

/// The single embedded vector with this id; a missing or duplicated id is a
/// failure naming the id.
fn find<'a>(all: &'a [Vector], id: &str) -> &'a Vector {
    let hits: Vec<&Vector> = all.iter().filter(|v| v.id() == id).collect();
    assert_eq!(
        hits.len(),
        1,
        "the reader must enumerate the example {id:?} exactly once; it yielded {}",
        hits.len()
    );
    hits[0]
}

// ------------------------------------------------------------- AC 2

/// AC 2: enumerating the reader yields each of the six examples, with its `id`
/// and `spec_version` unchanged, and the ids of the enumeration are unique.
#[test]
fn ac2_enumeration_yields_each_example_with_id_and_spec_version_unchanged() {
    let all = embedded();

    let mut ids: Vec<&str> = all.iter().map(|v| v.id()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), before, "the enumeration yields a duplicated id");

    for example in examples() {
        let id = str_field(&example, "id");
        let spec = str_field(&example, "spec_version");
        let got = find(&all, id);
        assert_eq!(
            got.spec_version(),
            spec,
            "{}: spec_version must come back unchanged",
            example.path.display()
        );
        assert_eq!(
            got.kind(),
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
    let all = embedded();
    let mut checked = 0;
    for example in examples() {
        let Some(bin) = example.json.get("bin").and_then(Value::as_str) else {
            continue;
        };
        let bin_path = example
            .path
            .parent()
            .unwrap_or_else(|| panic!("{}: no parent", example.path.display()))
            .join(bin);
        let expected =
            fs::read(&bin_path).unwrap_or_else(|e| panic!("{}: {e}", bin_path.display()));
        let id = str_field(&example, "id");
        let got = find(&all, id).bytes();
        assert_eq!(
            got.len(),
            expected.len(),
            "{}: the reader returned {} bytes, the file holds {}",
            bin_path.display(),
            got.len(),
            expected.len()
        );
        assert_eq!(
            got,
            expected.as_slice(),
            "{}: the reader's bytes differ from the file",
            bin_path.display()
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "no example names a .bin file; 007a-02 fixes at least one wire example"
    );
}

/// Work 1: a wire-valid example yields a decoded structure; a wire-invalid one
/// yields a rejection named by its declared `reason`, which is a member of the
/// closed 004-03 enumeration.
#[test]
fn work1_wire_examples_decode_or_reject_with_their_declared_reason() {
    let all = embedded();
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
        match (example.kind, find(&all, id).outcome()) {
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

/// Work 2: each of the six kinds enumerates on its own and yields only vectors
/// of that kind; every embedded vector belongs to exactly one kind; and each
/// example is found under the kind its directory names.
#[test]
fn work2_each_kind_enumerates_only_its_own_vectors() {
    let all = embedded();
    let kinds = [
        Kind::WireValid,
        Kind::WireInvalid,
        Kind::Control,
        Kind::Timeline,
        Kind::Transport,
        Kind::Reference,
    ];

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
    }
    assert_eq!(
        total,
        all.len(),
        "every embedded vector must belong to exactly one kind"
    );

    for example in examples() {
        let id = str_field(&example, "id");
        let kind = api_kind(example.kind);
        assert!(
            reader::of_kind(kind).any(|v| v.id() == id),
            "{}: the example {id:?} is not enumerated under {kind:?}",
            example.path.display()
        );
    }
}

// ------------------------------------------------------------- Work 3

/// Work 3 and the DoD line on versions: every embedded vector carries a
/// non-empty `id` and a non-empty `spec_version`.
#[test]
fn work3_every_embedded_vector_carries_a_nonempty_id_and_spec_version() {
    for v in embedded() {
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
