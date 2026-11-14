//! Machine-readable wire tables (ticket 004-02, C-133, C-130).
//!
//! The canonical field tables live in `doc/design/spec/wire.toml`, beside the
//! human-readable `wire.md` (C-130). This module is the checker that CI runs
//! unprivileged: it parses the file, round-trips its model, verifies that the
//! field offsets are contiguous and non-overlapping, verifies each declared
//! maximum size against the size computed from fields and maximum counts, and
//! verifies the 1 252-byte control bound (C-17), with the explicit path-MTU
//! probe exemption (C-116).
//!
//! **RED phase:** the types below and [`check_file`] are the tester's contract
//! for ticket 004-02. Every body is the coder's work; the integration test
//! `tests/wire_tables.rs` is the acceptance contract.

use std::path::Path;

/// Why the checker rejected a tables file.
///
/// One variant per failure kind that ticket 004-02 requires a negative
/// fixture to demonstrate (AC 2 and AC 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// The file is not TOML, or it does not match the schema (missing or
    /// wrongly typed key).
    Parse,
    /// The parsed model could not be re-serialised and re-parsed into an
    /// identical model (Work step 3, second bullet).
    RoundTrip,
    /// Two fields of one message occupy the same bit.
    Overlap,
    /// A bit between fields (or before the first field, or after the last) is
    /// occupied by no field and is not a declared `pad` field.
    Gap,
    /// The message's declared `max_size` differs from the size computed from
    /// its fields and maximum counts (Work step 3, third bullet).
    MaxSizeMismatch,
    /// A control message or regime stamp section exceeds 1 252 bytes and does
    /// not carry the explicit path-MTU probe exemption (C-17, C-116, AC 6).
    OversizedControl,
}

/// A rejection: the failure kind plus a human-readable message naming the
/// message and field (or the file position) that caused it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The failure kind the checker reports.
    pub kind: FailureKind,
    /// What failed, in enough detail to fix the table.
    pub message: String,
}

/// A clean check: the model that was parsed, one entry per message, in file
/// order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// One entry per `[[message]]`, in file order.
    pub messages: Vec<MessageReport>,
}

/// What the checker learned about one message. The tests use this to prove
/// that the file was parsed as a model rather than skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageReport {
    /// The message name (`name`).
    pub name: String,
    /// The message's `layer` token.
    pub layer: String,
    /// The message's DiffServ codepoint (`dscp`), 0 to 63.
    pub dscp: u8,
    /// The declared maximum size in bytes (`max_size`).
    pub max_size: u64,
    /// The number of `[[message.field]]` entries, padding included.
    pub field_count: usize,
    /// The parent message when this message is a regime stamp section
    /// (`section_of`).
    pub section_of: Option<String>,
    /// The rejection reasons this message can produce, in file order.
    pub rejection_reasons: Vec<String>,
}

/// Parse, round-trip and check one machine-readable tables file.
///
/// # Errors
///
/// Returns [`Error`] with the appropriate [`FailureKind`] on the first
/// violation found, in this order: parse, round-trip, overlap, gap, declared
/// versus computed maximum size, and the 1 252-byte control bound.
pub fn check_file(path: &Path) -> Result<Report, Error> {
    let _ = path;
    todo!("004-02: implement the machine-readable wire-table checker")
}
