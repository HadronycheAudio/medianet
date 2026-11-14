//! `hadronyche-tools` — hosted-only tool code for the Hadronyche Medianets
//! design set (C-133).
//!
//! The crate holds the ticket checks and generators that are too heavy or too
//! hosted-specific for the protocol crates: the `gen` vector generator (epic
//! 007) and the machine-readable wire-table checker (ticket 004-02).

pub mod wire_tables;
