//! `hadronyche-vectors` — the conformance data crate of the Hadronyche
//! Medianets protocol (epic 007).
//!
//! This crate sits at the workspace root of the design set's repository
//! (C-192) and holds the protocol's conformance data: wire vectors, control,
//! timeline, transport and reference data, under `vectors/`.
//!
//! Both profiles (hosted and embedded) are built from one `no_std` core
//! crate, so this crate must be usable without `std`.

#![no_std]
