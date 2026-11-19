//! `no_std` vector reader fuzz target (ticket 007a-03, C-95).
//!
//! Usage: `cargo run -p hadronyche-tools --bin vector-reader-fuzz -- [file ...]`
//!
//! With no arguments the seed corpus in `fuzz-corpus/vector-reader/` (kept
//! with this crate) is replayed. 007a-15 owns the mutation engine and the
//! duration loop (C-192); the per-change budget is 60 s and the nightly
//! budget 1 h (C-95), applied by 007a-15 and 007b-21.
//!
//! # What this target covers
//!
//! The reader parses nothing at run time: `hadronyche-vectors/build.rs`
//! parses the vector files on the host with `serde_json` and generates
//! constant tables, and a malformed file fails the build. The run-time code
//! that takes caller data is the lookups, so each input is fed to all three,
//! which must return for any bytes without panicking:
//!
//! * `reader::with_bytes` — the recorded outcome of an encoding, by bytes;
//! * `reader::find` — a vector by id (the input as UTF-8, when it is);
//! * `reader::Reason::from_name` — a rejection reason by name.
//!
//! The conformance set is empty until 007a-05, so each input is also matched
//! against the 007a-02 examples (`reader::examples`, by bytes and by id, with
//! the same comparisons the lookups make); the seed corpus holds the two
//! example encodings, so a default run matches and walks real data.
//!
//! Every vector found is then walked to the leaves of its recorded JSON.
//! This is not a wire decoder: decoding wire bytes is the implementation's
//! job (epic 009 onward), fuzzed through `UnderTest::decode` by 007a-15.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: a command-line tool

use std::path::{Path, PathBuf};

use hadronyche_vectors::reader::{self, Outcome, Reason, Value, Vector};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz-corpus/vector-reader")
}

fn inputs(args: &[String]) -> Vec<PathBuf> {
    if !args.is_empty() {
        return args.iter().map(PathBuf::from).collect();
    }
    let mut files: Vec<PathBuf> = match std::fs::read_dir(corpus_dir()) {
        Ok(entries) => entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect(),
        Err(err) => {
            eprintln!("vector-reader-fuzz: {}: {err}", corpus_dir().display());
            std::process::exit(2);
        }
    };
    files.sort();
    files
}

/// Count the leaves of a recorded JSON value.
fn leaves(value: Value) -> usize {
    match value {
        Value::Array(items) => items.iter().map(|v| leaves(*v)).sum(),
        Value::Object(fields) => fields.iter().map(|f| leaves(f.value())).sum(),
        _ => 1,
    }
}

/// Walk one vector found by a lookup.
fn walk(vector: Vector) -> usize {
    let outcome = match vector.outcome() {
        Outcome::Valid(decoded) => leaves(decoded.value()),
        Outcome::Rejected(rejection) => usize::from(rejection.reason().value()),
        Outcome::NotWire => 0,
    };
    outcome + leaves(vector.document()) + vector.bytes().len()
}

/// Feed one input to every run-time lookup of the reader; returns the number
/// of vectors and reasons it matched.
fn run_one(bytes: &[u8]) -> usize {
    let mut hits = 0;
    for vector in reader::with_bytes(bytes) {
        std::hint::black_box(walk(vector));
        hits += 1;
    }
    let text = std::str::from_utf8(bytes).ok();
    for vector in reader::examples() {
        if vector.bytes() == bytes || text == Some(vector.id()) {
            std::hint::black_box(walk(vector));
            hits += 1;
        }
    }
    if let Some(text) = text {
        if let Some(vector) = reader::find(text) {
            std::hint::black_box(walk(vector));
            hits += 1;
        }
        if Reason::from_name(text).is_some() {
            hits += 1;
        }
    }
    hits
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let files = inputs(&args);
    if files.is_empty() {
        eprintln!("vector-reader-fuzz: no inputs (empty corpus)");
        std::process::exit(2);
    }
    let mut hits = 0;
    for file in &files {
        let bytes = std::fs::read(file).unwrap_or_else(|err| {
            eprintln!("vector-reader-fuzz: {}: {err}", file.display());
            std::process::exit(2);
        });
        // A panic here is the crash this target exists to find.
        hits += run_one(&bytes);
    }
    println!(
        "vector-reader-fuzz: {} input(s) looked up without a panic ({} match(es) among {} conformance vector(s) and {} example(s))",
        files.len(),
        hits,
        reader::all().count(),
        reader::examples().count()
    );
}
