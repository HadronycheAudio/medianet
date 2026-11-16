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
//! allocator: [`reader::all`], [`reader::of_kind`], [`reader::find`] and
//! [`reader::with_bytes`] yield [`reader::Vector`] handles on `'static` data.
//!
//! The data is embedded at build time: `build.rs` parses every vector file on
//! the host, fails the build on a malformed one, and generates constant Rust
//! tables (wire bytes by `include_bytes!`), so a device without a file system
//! iterates the same set and nothing is parsed at run time. The embedded set
//! is every vector under `vectors/`, plus, with the non-default `examples`
//! feature, the six example fixtures of 007a-02 (see [`reader`]).
//!
//! # Features
//!
//! * `examples` (off by default): also embed the 007a-02 example fixtures.
//!
//! No feature pulls in `std` or an allocator. The crate has no `unsafe` code.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod reader;
