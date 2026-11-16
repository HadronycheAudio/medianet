//! `no_std` vector reader parser fuzz target (ticket 007a-03, C-95).
//!
//! Usage: `cargo run --bin vector-reader-fuzz -- [file ...]`
//!
//! With no arguments the seed corpus in `fuzz-corpus/vector-reader/` (kept
//! with this crate) is replayed. Each input is passed to the reader's own
//! wire decoder, `hadronyche_vectors::reader::decode_wire`, which must return
//! an outcome for any byte string without panicking or hanging (ticket
//! 007a-15 Work 2). The per-change budget is 60 s and the nightly budget 1 h
//! (C-95), applied by 007a-15 and 007b-21; the mutation engine and the
//! duration loop are that ticket's choice (C-192).
//!
//! RED-phase harness (tester's contract): `reader::decode_wire` is part of
//! the proposed API of `hadronyche-vectors/tests/reader.rs` and does not
//! exist yet.

use std::path::{Path, PathBuf};

use hadronyche_vectors::reader;

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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let files = inputs(&args);
    if files.is_empty() {
        eprintln!("vector-reader-fuzz: no inputs (empty corpus)");
        std::process::exit(2);
    }
    for file in &files {
        let bytes = std::fs::read(file).unwrap_or_else(|err| {
            eprintln!("vector-reader-fuzz: {}: {err}", file.display());
            std::process::exit(2);
        });
        // A panic here is the crash this target exists to find.
        let _outcome = reader::decode_wire(&bytes);
    }
    println!(
        "vector-reader-fuzz: {} input(s) decoded without a panic",
        files.len()
    );
}
