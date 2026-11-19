//! The tests of every ported document check, in one test binary (one link,
//! libtest's parallelism across all checks, one shared document cache).
//!
//! Each check's tests are the module `tests/doc/c<id>.rs`; `build.rs`
//! lists them, so porters working in parallel never edit a shared file.
//!
//! - `cargo test` (debug) runs the criterion tests, the red-phase tests and
//!   the check that every denial has exactly one test;
//! - `cargo test --release -p hadronyche-tools --test doc` also runs every
//!   denial (ignored in debug builds): CI's "Denial tests (release)" step and
//!   the per-change suite.

include!(concat!(env!("OUT_DIR"), "/doc_tests.rs"));
