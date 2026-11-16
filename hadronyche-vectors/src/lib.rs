//! `hadronyche-vectors` — the conformance data crate of the Hadronyche
//! Medianets protocol (epic 007).
//!
//! This crate sits at the workspace root of the design set's repository
//! (C-192) and holds the protocol's conformance data: wire vectors, control,
//! timeline, transport and reference data, under `vectors/`.
//!
//! Both profiles (hosted and embedded) are built from one `no_std` core
//! crate, so this crate must be usable without `std`.
//!
//! # Reader (ticket 007a-03)
//!
//! [`reader`] enumerates the embedded vectors without `std` and without an
//! allocator, as [`reader::Vector`] handles on `'static` data. Two sets are
//! always embedded, and no cargo feature changes either:
//!
//! * the conformance set, every vector under `vectors/` (empty until 007a-05):
//!   [`reader::all`], [`reader::of_kind`], [`reader::find`],
//!   [`reader::with_bytes`];
//! * the six example fixtures of 007a-02, which are test fixtures:
//!   [`reader::examples`], [`reader::examples_of_kind`] only.
//!
//! The data is embedded at build time: `build.rs` parses every vector file on
//! the host, fails the build on a malformed one, and generates constant Rust
//! tables (wire bytes by `include_bytes!`), so a device without a file system
//! iterates the same sets and nothing is parsed at run time (see [`reader`]).
//!
//! The crate has no features beyond `default` (empty), no normal
//! dependencies, no allocator and no `unsafe` code.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod reader;
mod reason;
