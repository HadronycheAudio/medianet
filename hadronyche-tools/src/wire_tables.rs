//! Machine-readable wire tables (ticket 004-02, C-133, C-130).
//!
//! The canonical field tables live in `doc/design/spec/wire.toml`, beside the
//! human-readable `wire.md` (C-130). This module is the checker that CI runs
//! unprivileged: it parses the file, round-trips its model, verifies that the
//! field offsets are contiguous and non-overlapping, verifies each declared
//! maximum size against the size computed from fields and maximum counts, and
//! verifies the 1 252-byte control bound (C-17), with the explicit path-MTU
//! probe exemption (C-116). From ticket 004-03 it also verifies that every
//! `rejection_reasons` entry is a member of the closed `Reject` enumeration,
//! which the file declares in `[[reject]]` tables and which the checker pins
//! (the enumeration is closed within a major version). From ticket 004-05 it
//! also parses and round-trips the `[[reserved]]` entries, the machine-
//! readable copy of wire.md's "Reserved fields" section (reserved values and
//! receiver outcomes, C-07, C-12, C-115).
//!
//! The file parser is the standard TOML library (the ticket's default), not a
//! hand-written parser, so it needs no fuzzing under C-95. The checker's own
//! logic (model extraction, canonical serialisation, layout and size rules) is
//! ordinary bounded iteration: no recursion, run time linear in the file size
//! (AC 5).
//!
//! One deliberate asymmetry pins the round-trip check: the parser accepts the
//! deprecated spelling `maximum_size` (and records which spelling it saw), so
//! the deprecation can be reported, while the canonical writer always emits
//! `max_size`. A file that uses the deprecated spelling therefore reparses to
//! a different model, and the checker reports [`FailureKind::RoundTrip`].

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: a tool of its own, not a document check

use std::fmt::Write as _;
use std::path::Path;

use toml::Value;

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
    /// One entry per `[[reject]]`, in file order (ticket 004-03): the closed
    /// `Reject` enumeration as the file declares it. Empty when the file
    /// declares none, as the 004-02 fixtures do; the membership check always
    /// runs against the closed set pinned in the checker.
    pub rejects: Vec<RejectReport>,
    /// One entry per `[[reserved]]`, in file order (ticket 004-05): the
    /// reserved fields' required v1 values, the machine-readable copy of
    /// wire.md's "Reserved fields" section. Empty when the file declares
    /// none, as the earlier fixtures do.
    pub reserves: Vec<ReservedReport>,
    /// One entry per `[[message]]`, in file order.
    pub messages: Vec<MessageReport>,
}

/// What the checker learned about one `[[reject]]` entry (ticket 004-03): one
/// variant of the closed `Reject` enumeration, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectReport {
    /// The variant name (`name`).
    pub name: String,
    /// The variant's `u8` value (`value`).
    pub value: u8,
    /// The one-line meaning (`meaning`), as in wire.md.
    pub meaning: String,
}

/// What the checker learned about one `[[reserved]]` entry (ticket 004-05):
/// one reserved field's required v1 value, the machine-readable copy of a row
/// of wire.md's "Reserved fields" section, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedReport {
    /// The field name (`name`), as in wire.md.
    pub name: String,
    /// Where the field lives (`message`).
    pub message: String,
    /// The bit count (`width`).
    pub width: u64,
    /// The required v1 value (`value`), rendered as text: `0` for the
    /// integer zero, the string otherwise ("zero", "live").
    pub value: String,
    /// Whether the field is live in v1 (`live`, C-115): live fields carry
    /// any value and never produce `ReservedNonZero`.
    pub live: bool,
    /// The one-line meaning (`meaning`), as in wire.md.
    pub meaning: String,
    /// The receiver outcome on a non-conforming value (`outcome`).
    pub outcome: String,
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

/// The 1 252-byte control bound (C-17), which also covers each regime stamp
/// section (C-18).
const CONTROL_BOUND_BYTES: u64 = 1252;

/// The explicit exemption value that lifts the control bound for the path-MTU
/// probe pair (C-116).
const EXEMPTION: &str = "C-116";

/// The only schema revision this checker implements.
const SCHEMA: i64 = 1;

/// The message layers, in schema order.
const LAYERS: [&str; 4] = ["control", "sync", "media", "measurement"];

/// The field types, in schema order: sub-byte unsigned (`uint`), fixed-width
/// unsigned and signed integers (C-10, C-11, C-12), the 4-byte sample (C-03),
/// the Q1.31 fixed-point sample, and declared padding.
const FIELD_TYPES: [&str; 14] = [
    "uint", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "q1.31",
    "pad",
];

/// The closed `Reject` enumeration (ticket 004-03): one `(name, u8 value)`
/// pair per variant, in wire.md table order. The enumeration is closed
/// within a major version — no file may declare a different set — so the
/// checker pins it here, as it pins the layers and the field types, and
/// accepts a `[[reject]]` declaration only when it matches exactly,
/// name-for-name and value-for-value. Values 13..=255 are reserved for a
/// future major version and never assigned here.
const REJECT: [(&str, u8); 13] = [
    ("TooShort", 0),
    ("BadMagic", 1),
    ("VersionMismatch", 2),
    ("ForeignDomain", 3),
    ("UnconfiguredDomain", 4),
    ("UnknownStream", 5),
    ("UnadmittedSource", 6),
    ("BadLength", 7),
    ("BadChecksum", 8),
    ("BadPacketIndex", 9),
    ("WrongEpoch", 10),
    ("OversizeControl", 11),
    ("ReservedNonZero", 12),
];

/// Which spelling of the maximum-size key the parser saw. The deprecated
/// `maximum_size` is accepted so it can be reported, but the canonical writer
/// emits `max_size`, so a file using the deprecated spelling cannot round-trip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SizeKey {
    /// The canonical `max_size` spelling.
    MaxSize,
    /// The deprecated `maximum_size` spelling.
    MaximumSize,
}

/// One `[[message.field]]` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    name: String,
    /// Bit position from the message start (`offset_unit = "bits"`).
    offset: u64,
    /// Bit count; per element when `count` is present (`width_unit = "bits"`).
    width: u64,
    field_type: String,
    meaning: String,
    valid: String,
    reserved: Option<String>,
    count: Option<String>,
    max_count: Option<u64>,
}

impl Field {
    /// The bits the field occupies: `width * max_count` for a counted field,
    /// `width` otherwise (the width is per element for counted fields).
    fn occupied_bits(&self, message: &str) -> Result<u64, Error> {
        let elements = self.max_count.unwrap_or(1);
        self.width.checked_mul(elements).ok_or_else(|| {
            parse_error(format!(
                "message {message}: field {}: width x max_count overflows u64",
                self.name
            ))
        })
    }
}

/// One `[[message]]` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    name: String,
    layer: String,
    dscp: u8,
    max_size: u64,
    size_key: SizeKey,
    section_of: Option<String>,
    exempt_from_control_bound: Option<String>,
    rejection_reasons: Vec<String>,
    fields: Vec<Field>,
}

/// One `[[reject]]` entry (ticket 004-03): a variant of the closed `Reject`
/// enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reject {
    name: String,
    value: u8,
    meaning: String,
}

/// The required v1 value of a reserved field (ticket 004-05): an integer
/// (e.g. `value = 0`) or free text (e.g. `value = "zero"`, `value = "live"`).
/// The split keeps the canonical writer format-stable, so the entry
/// round-trips.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ReservedValue {
    /// An integer value.
    Int(i64),
    /// A free-text value.
    Text(String),
}

impl ReservedValue {
    /// The value rendered as report text: the integer as decimal, the text
    /// as written.
    fn as_text(&self) -> String {
        match self {
            ReservedValue::Int(n) => n.to_string(),
            ReservedValue::Text(text) => text.clone(),
        }
    }
}

/// One `[[reserved]]` entry (ticket 004-05): a reserved field's required v1
/// value and receiver outcome, one row of wire.md's "Reserved fields"
/// section.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reserved {
    name: String,
    message: String,
    width: u64,
    value: ReservedValue,
    live: Option<bool>,
    meaning: String,
    outcome: String,
}

/// The whole parsed file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Model {
    schema: i64,
    offset_unit: String,
    width_unit: String,
    size_unit: String,
    rejects: Vec<Reject>,
    reserves: Vec<Reserved>,
    messages: Vec<Message>,
}

fn parse_error(message: impl std::fmt::Display) -> Error {
    Error {
        kind: FailureKind::Parse,
        message: message.to_string(),
    }
}

/// An integer value out of a TOML value.
fn as_int(value: &Value, what: &str) -> Result<i64, Error> {
    value
        .as_integer()
        .ok_or_else(|| parse_error(format!("{what} must be an integer, got {value:?}")))
}

/// A non-negative integer value out of a TOML value.
fn as_u64(value: &Value, what: &str) -> Result<u64, Error> {
    let raw = as_int(value, what)?;
    u64::try_from(raw).map_err(|_| parse_error(format!("{what} must be non-negative, got {raw}")))
}

/// A string value out of a TOML value.
fn as_str<'a>(value: &'a Value, what: &str) -> Result<&'a str, Error> {
    value
        .as_str()
        .ok_or_else(|| parse_error(format!("{what} must be a string, got {value:?}")))
}

/// A required string key.
fn require_str(table: &toml::Table, key: &str, context: &str) -> Result<String, Error> {
    let value = table
        .get(key)
        .ok_or_else(|| parse_error(format!("{context}: missing `{key}`")))?;
    as_str(value, &format!("{context}: `{key}`")).map(str::to_owned)
}

/// An optional string key.
fn optional_str(table: &toml::Table, key: &str, context: &str) -> Result<Option<String>, Error> {
    match table.get(key) {
        None => Ok(None),
        Some(value) => as_str(value, &format!("{context}: `{key}`"))
            .map(str::to_owned)
            .map(Some),
    }
}

/// A required non-negative integer key.
fn require_u64(table: &toml::Table, key: &str, context: &str) -> Result<u64, Error> {
    let value = table
        .get(key)
        .ok_or_else(|| parse_error(format!("{context}: missing `{key}`")))?;
    as_u64(value, &format!("{context}: `{key}`"))
}

/// An optional non-negative integer key.
fn optional_u64(table: &toml::Table, key: &str, context: &str) -> Result<Option<u64>, Error> {
    match table.get(key) {
        None => Ok(None),
        Some(value) => as_u64(value, &format!("{context}: `{key}`")).map(Some),
    }
}

/// One of the unit declarations: the value must be the expected token.
fn check_unit(root: &toml::Table, key: &str, expected: &str) -> Result<(), Error> {
    let value = root
        .get(key)
        .ok_or_else(|| parse_error(format!("missing `{key}`")))?;
    let unit = as_str(value, &format!("`{key}`"))?;
    if unit != expected {
        return Err(parse_error(format!(
            "`{key} = \"{unit}\"` is unsupported; this checker implements revision {SCHEMA} with `{key} = \"{expected}\"`"
        )));
    }
    Ok(())
}

/// Parse one machine-readable tables file (schema revision 1) into the model.
///
/// The deprecated `maximum_size` spelling of `max_size` is accepted and
/// recorded ([`SizeKey::MaximumSize`]); every other deviation from the schema
/// is a [`FailureKind::Parse`] error.
fn parse_model(src: &str) -> Result<Model, Error> {
    let root = src
        .parse::<toml::Table>()
        .map_err(|error| parse_error(format!("not valid TOML: {error}")))?;
    let schema = require_u64(&root, "schema", "the file")?;
    if schema != SCHEMA.unsigned_abs() {
        return Err(parse_error(format!(
            "schema = {schema} is unsupported; this checker implements revision {SCHEMA}"
        )));
    }
    check_unit(&root, "offset_unit", "bits")?;
    check_unit(&root, "width_unit", "bits")?;
    check_unit(&root, "size_unit", "bytes")?;
    let rejects = parse_rejects(&root)?;
    let reserves = parse_reserved(&root)?;
    let entries = root
        .get("message")
        .ok_or_else(|| parse_error("missing [[message]] array"))?
        .as_array()
        .ok_or_else(|| parse_error("`message` must be an array of tables"))?;
    let mut messages = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let table = entry
            .as_table()
            .ok_or_else(|| parse_error(format!("message {index} is not a table")))?;
        messages.push(parse_message(table, index)?);
    }
    ensure_unique_names(&messages)?;
    Ok(Model {
        schema: SCHEMA,
        offset_unit: "bits".to_owned(),
        width_unit: "bits".to_owned(),
        size_unit: "bytes".to_owned(),
        rejects,
        reserves,
        messages,
    })
}

/// Parse the optional `[[reject]]` array: the closed `Reject` enumeration
/// (ticket 004-03). The array is optional — the 004-02 fixtures predate it —
/// but when present it must declare exactly the closed set pinned in
/// [`REJECT`], name-for-name and value-for-value, because the enumeration is
/// closed within a major version and no file may redefine it.
fn parse_rejects(root: &toml::Table) -> Result<Vec<Reject>, Error> {
    let entries = match root.get("reject") {
        None => return Ok(Vec::new()),
        Some(value) => value
            .as_array()
            .ok_or_else(|| parse_error("`reject` must be an array of tables"))?,
    };
    let mut rejects = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let context = format!("reject {index}");
        let table = entry
            .as_table()
            .ok_or_else(|| parse_error(format!("{context} is not a table")))?;
        let name = require_str(table, "name", &context)?;
        if name.is_empty() {
            return Err(parse_error(format!("{context}: `name` must be non-empty")));
        }
        let raw = require_u64(table, "value", &context)?;
        let value = u8::try_from(raw).map_err(|_| {
            parse_error(format!(
                "{context}: `value` = {raw} is outside the u8 range 0..=255"
            ))
        })?;
        let meaning = require_str(table, "meaning", &context)?;
        if meaning.is_empty() {
            return Err(parse_error(format!(
                "{context}: `meaning` must be non-empty"
            )));
        }
        rejects.push(Reject {
            name,
            value,
            meaning,
        });
    }
    let mut names: Vec<&str> = rejects.iter().map(|reject| reject.name.as_str()).collect();
    names.sort_unstable();
    names.dedup_by(|a, b| a == b);
    if names.len() != rejects.len() {
        return Err(parse_error("duplicate reject name; names must be unique"));
    }
    let mut values: Vec<u8> = rejects.iter().map(|reject| reject.value).collect();
    values.sort_unstable();
    values.dedup();
    if values.len() != rejects.len() {
        return Err(parse_error(
            "duplicate reject value; u8 values must be unique",
        ));
    }
    let mut declared: Vec<(&str, u8)> = rejects
        .iter()
        .map(|reject| (reject.name.as_str(), reject.value))
        .collect();
    declared.sort_unstable();
    let mut closed: Vec<(&str, u8)> = REJECT.to_vec();
    closed.sort_unstable();
    if declared != closed {
        return Err(parse_error(
            "the [[reject]] tables must declare exactly the closed Reject enumeration \
             (ticket 004-03, closed within a major version); this file declares a different set",
        ));
    }
    Ok(rejects)
}

/// Parse the optional `[[reserved]]` array (ticket 004-05): the reserved
/// fields' required v1 values, the machine-readable copy of wire.md's
/// "Reserved fields" section. The array is optional — the earlier fixtures
/// predate it — but an entry, once present, must be complete: a non-empty
/// unique `name`, a `message` location, a `width` of at least one bit, a
/// `value` (integer or string), a non-empty `meaning` and `outcome`, and an
/// optional `live` flag. A live field (C-115) carries any value, so it
/// cannot also be fixed at 0.
fn parse_reserved(root: &toml::Table) -> Result<Vec<Reserved>, Error> {
    let entries = match root.get("reserved") {
        None => return Ok(Vec::new()),
        Some(value) => value
            .as_array()
            .ok_or_else(|| parse_error("`reserved` must be an array of tables"))?,
    };
    let mut reserved = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let context = format!("reserved {index}");
        let table = entry
            .as_table()
            .ok_or_else(|| parse_error(format!("{context} is not a table")))?;
        let name = require_str(table, "name", &context)?;
        if name.is_empty() {
            return Err(parse_error(format!("{context}: `name` must be non-empty")));
        }
        let field_context = format!("reserved {name}");
        let message = require_str(table, "message", &field_context)?;
        if message.is_empty() {
            return Err(parse_error(format!(
                "{field_context}: `message` must be non-empty"
            )));
        }
        let width = require_u64(table, "width", &field_context)?;
        if width == 0 {
            return Err(parse_error(format!(
                "{field_context}: `width` must be at least 1 bit"
            )));
        }
        let value = match table.get("value") {
            None => {
                return Err(parse_error(format!(
                    "{field_context}: missing `value` (the required v1 value)"
                )));
            }
            Some(value) => {
                if let Some(n) = value.as_integer() {
                    ReservedValue::Int(n)
                } else if let Some(text) = value.as_str() {
                    ReservedValue::Text(text.to_owned())
                } else {
                    return Err(parse_error(format!(
                        "{field_context}: `value` must be an integer or a string, got {value:?}"
                    )));
                }
            }
        };
        let live = match table.get("live") {
            None => None,
            Some(value) => value
                .as_bool()
                .map(Some)
                .ok_or_else(|| parse_error(format!("{field_context}: `live` must be a boolean")))?,
        };
        let meaning = require_str(table, "meaning", &field_context)?;
        if meaning.is_empty() {
            return Err(parse_error(format!(
                "{field_context}: `meaning` must be non-empty"
            )));
        }
        let outcome = require_str(table, "outcome", &field_context)?;
        if outcome.is_empty() {
            return Err(parse_error(format!(
                "{field_context}: `outcome` must be non-empty"
            )));
        }
        if live == Some(true) && value == ReservedValue::Int(0) {
            return Err(parse_error(format!(
                "{field_context}: a live field carries any value in v1 (C-115); it cannot also be fixed at 0"
            )));
        }
        reserved.push(Reserved {
            name,
            message,
            width,
            value,
            live,
            meaning,
            outcome,
        });
    }
    let mut names: Vec<&str> = reserved.iter().map(|entry| entry.name.as_str()).collect();
    names.sort_unstable();
    names.dedup_by(|a, b| a == b);
    if names.len() != reserved.len() {
        return Err(parse_error("duplicate reserved name; names must be unique"));
    }
    Ok(reserved)
}

/// Message names must be unique (the schema's `name` identifies the message).
fn ensure_unique_names(messages: &[Message]) -> Result<(), Error> {
    let mut names: Vec<&str> = messages
        .iter()
        .map(|message| message.name.as_str())
        .collect();
    names.sort_unstable();
    names.dedup_by(|a, b| a == b);
    if names.len() != messages.len() {
        return Err(parse_error("duplicate message name; names must be unique"));
    }
    Ok(())
}

/// Parse one `[[message]]` table.
fn parse_message(table: &toml::Table, index: usize) -> Result<Message, Error> {
    let name = require_str(table, "name", &format!("message {index}"))?;
    if name.is_empty() {
        return Err(parse_error(format!(
            "message {index}: `name` must be non-empty"
        )));
    }
    let layer = require_str(table, "layer", &name)?;
    if !LAYERS.contains(&layer.as_str()) {
        return Err(parse_error(format!(
            "message {name}: unknown layer \"{layer}\"; expected one of {LAYERS:?}"
        )));
    }
    let dscp = require_u64(table, "dscp", &name)?;
    let dscp = u8::try_from(dscp)
        .ok()
        .filter(|value| *value <= 63)
        .ok_or_else(|| parse_error(format!("message {name}: dscp = {dscp} is outside 0..=63")))?;
    let (max_size, size_key) = parse_max_size(table, &name)?;
    let section_of = optional_str(table, "section_of", &name)?;
    let exempt_from_control_bound = optional_str(table, "exempt_from_control_bound", &name)?;
    if let Some(value) = exempt_from_control_bound.as_deref()
        && value != EXEMPTION
    {
        return Err(parse_error(format!(
            "message {name}: exempt_from_control_bound = \"{value}\" is unknown; the only exemption is \"{EXEMPTION}\" (C-116)"
        )));
    }
    let rejection_reasons = parse_rejection_reasons(table, &name)?;
    let fields = parse_fields(table, &name)?;
    Ok(Message {
        name,
        layer,
        dscp,
        max_size,
        size_key,
        section_of,
        exempt_from_control_bound,
        rejection_reasons,
        fields,
    })
}

/// Parse `max_size`, accepting the deprecated `maximum_size` spelling (and
/// rejecting the presence of both).
fn parse_max_size(table: &toml::Table, name: &str) -> Result<(u64, SizeKey), Error> {
    let canonical = table.get("max_size");
    let deprecated = table.get("maximum_size");
    match (canonical, deprecated) {
        (Some(value), None) => {
            let max_size = as_u64(value, &format!("message {name}: `max_size`"))?;
            Ok((max_size, SizeKey::MaxSize))
        }
        (None, Some(value)) => {
            let max_size = as_u64(
                value,
                &format!("message {name}: `maximum_size` (deprecated spelling of `max_size`)"),
            )?;
            Ok((max_size, SizeKey::MaximumSize))
        }
        (Some(_), Some(_)) => Err(parse_error(format!(
            "message {name}: both `max_size` and the deprecated `maximum_size` are present; use `max_size` only"
        ))),
        (None, None) => Err(parse_error(format!(
            "message {name}: missing `max_size` (the declared maximum size, bytes)"
        ))),
    }
}

/// Parse the per-message rejection-reason list: an array of non-empty
/// strings, each a member of the closed `Reject` enumeration (checked from
/// ticket 004-03).
fn parse_rejection_reasons(table: &toml::Table, name: &str) -> Result<Vec<String>, Error> {
    let value = table
        .get("rejection_reasons")
        .ok_or_else(|| parse_error(format!("message {name}: missing `rejection_reasons`")))?;
    let entries = value.as_array().ok_or_else(|| {
        parse_error(format!(
            "message {name}: `rejection_reasons` must be an array of strings"
        ))
    })?;
    let mut reasons = Vec::with_capacity(entries.len());
    for entry in entries {
        let reason = as_str(entry, &format!("message {name}: `rejection_reasons` entry"))?;
        if reason.is_empty() {
            return Err(parse_error(format!(
                "message {name}: an empty rejection reason would be skipped silently (C-125)"
            )));
        }
        if !REJECT.iter().any(|(variant, _)| *variant == reason) {
            return Err(parse_error(format!(
                "message {name}: rejection reason \"{reason}\" is not a member of the closed Reject enumeration (ticket 004-03, wire.md's Rejection reasons section)"
            )));
        }
        reasons.push(reason.to_owned());
    }
    Ok(reasons)
}

/// Parse the `[[message.field]]` array of one message.
fn parse_fields(table: &toml::Table, name: &str) -> Result<Vec<Field>, Error> {
    let entries = match table.get("field") {
        None => return Ok(Vec::new()),
        Some(value) => value.as_array().ok_or_else(|| {
            parse_error(format!(
                "message {name}: `field` must be an array of tables"
            ))
        })?,
    };
    let mut fields = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let field_table = entry
            .as_table()
            .ok_or_else(|| parse_error(format!("message {name}: field {index} is not a table")))?;
        fields.push(parse_field(field_table, name, index)?);
    }
    let mut names: Vec<&str> = fields.iter().map(|field| field.name.as_str()).collect();
    names.sort_unstable();
    names.dedup_by(|a, b| a == b);
    if names.len() != fields.len() {
        return Err(parse_error(format!(
            "message {name}: duplicate field name; field names must be unique within the message"
        )));
    }
    Ok(fields)
}

/// Parse one `[[message.field]]` table.
fn parse_field(table: &toml::Table, message: &str, index: usize) -> Result<Field, Error> {
    let context = format!("message {message}: field {index}");
    let name = require_str(table, "name", &context)?;
    if name.is_empty() {
        return Err(parse_error(format!("{context}: `name` must be non-empty")));
    }
    let field_context = format!("message {message}: field {name}");
    let offset = require_u64(table, "offset", &field_context)?;
    let width = require_u64(table, "width", &field_context)?;
    if width == 0 {
        return Err(parse_error(format!(
            "{field_context}: `width` must be at least 1 bit"
        )));
    }
    let field_type = require_str(table, "type", &field_context)?;
    if !FIELD_TYPES.contains(&field_type.as_str()) {
        return Err(parse_error(format!(
            "message {message}: field {name}: unknown type \"{field_type}\"; expected one of {FIELD_TYPES:?}"
        )));
    }
    let meaning = require_str(table, "meaning", &field_context)?;
    let valid = require_str(table, "valid", &field_context)?;
    let reserved = optional_str(table, "reserved", &field_context)?;
    let count = optional_str(table, "count", &field_context)?;
    let max_count = optional_u64(table, "max_count", &field_context)?;
    if count.is_some() != max_count.is_some() {
        return Err(parse_error(format!(
            "{field_context}: `count` and `max_count` go together; the count field names the list, `max_count` fixes its maximum"
        )));
    }
    Ok(Field {
        name,
        offset,
        width,
        field_type,
        meaning,
        valid,
        reserved,
        count,
        max_count,
    })
}

/// Escape one string as a TOML basic string body (without the quotes).
fn escape_toml_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ if (character as u32) < 0x20 || (character as u32) == 0x7f => {
                let _ = write!(out, "\\u{:04X}", character as u32);
            }
            _ => out.push(character),
        }
    }
    out
}

/// Append `key = "value"` to the canonical output.
fn write_quoted(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "{key} = \"{}\"", escape_toml_string(value));
}

/// Serialise the model back to TOML in the canonical form.
///
/// The writer always emits the canonical `max_size` spelling, even when the
/// model was parsed from the deprecated `maximum_size`; that asymmetry is what
/// the round-trip check pins (see the module header).
fn write_canonical(model: &Model) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "schema = {}", model.schema);
    write_quoted(&mut out, "offset_unit", &model.offset_unit);
    write_quoted(&mut out, "width_unit", &model.width_unit);
    write_quoted(&mut out, "size_unit", &model.size_unit);
    for reject in &model.rejects {
        out.push_str("\n[[reject]]\n");
        write_quoted(&mut out, "name", &reject.name);
        let _ = writeln!(out, "value = {}", reject.value);
        write_quoted(&mut out, "meaning", &reject.meaning);
    }
    for reserved in &model.reserves {
        out.push_str("\n[[reserved]]\n");
        write_quoted(&mut out, "name", &reserved.name);
        write_quoted(&mut out, "message", &reserved.message);
        let _ = writeln!(out, "width = {}", reserved.width);
        match &reserved.value {
            ReservedValue::Int(n) => {
                let _ = writeln!(out, "value = {n}");
            }
            ReservedValue::Text(text) => write_quoted(&mut out, "value", text),
        }
        if let Some(live) = reserved.live {
            let _ = writeln!(out, "live = {live}");
        }
        write_quoted(&mut out, "meaning", &reserved.meaning);
        write_quoted(&mut out, "outcome", &reserved.outcome);
    }
    for message in &model.messages {
        out.push_str("\n[[message]]\n");
        write_quoted(&mut out, "name", &message.name);
        write_quoted(&mut out, "layer", &message.layer);
        let _ = writeln!(out, "dscp = {}", message.dscp);
        let _ = writeln!(out, "max_size = {}", message.max_size);
        if let Some(parent) = &message.section_of {
            write_quoted(&mut out, "section_of", parent);
        }
        if let Some(exemption) = &message.exempt_from_control_bound {
            write_quoted(&mut out, "exempt_from_control_bound", exemption);
        }
        write_reason_list(&mut out, &message.rejection_reasons);
        for field in &message.fields {
            out.push_str("\n[[message.field]]\n");
            write_quoted(&mut out, "name", &field.name);
            let _ = writeln!(out, "offset = {}", field.offset);
            let _ = writeln!(out, "width = {}", field.width);
            write_quoted(&mut out, "type", &field.field_type);
            write_quoted(&mut out, "meaning", &field.meaning);
            write_quoted(&mut out, "valid", &field.valid);
            if let Some(reserved) = &field.reserved {
                write_quoted(&mut out, "reserved", reserved);
            }
            if let Some(count) = &field.count {
                write_quoted(&mut out, "count", count);
            }
            if let Some(max_count) = field.max_count {
                let _ = writeln!(out, "max_count = {max_count}");
            }
        }
    }
    out
}

/// Append `rejection_reasons = [...]` to the canonical output.
fn write_reason_list(out: &mut String, reasons: &[String]) {
    let quoted: Vec<String> = reasons
        .iter()
        .map(|reason| format!("\"{}\"", escape_toml_string(reason)))
        .collect();
    let _ = writeln!(out, "rejection_reasons = [{}]", quoted.join(", "));
}

/// Check, per message, that the fields cover the message from bit 0 with no
/// overlap and no undeclared gap. A `type = "pad"` field is an ordinary field
/// for coverage, which is how padding is declared.
fn check_layout(model: &Model) -> Result<(), Error> {
    for message in &model.messages {
        let mut spans: Vec<(u64, u64, &str)> = Vec::with_capacity(message.fields.len());
        for field in &message.fields {
            let bits = field.occupied_bits(&message.name)?;
            let end = field.offset.checked_add(bits).ok_or_else(|| {
                parse_error(format!(
                    "message {}: field {}: offset + bits overflows u64",
                    message.name, field.name
                ))
            })?;
            spans.push((field.offset, end, field.name.as_str()));
        }
        spans.sort_unstable_by_key(|span| (span.0, span.1));
        let mut cursor = 0u64;
        for (start, end, name) in spans {
            if start < cursor {
                return Err(Error {
                    kind: FailureKind::Overlap,
                    message: format!(
                        "message {}: field {name} starts at bit {start}, inside the previous field, which reaches bit {cursor}",
                        message.name
                    ),
                });
            }
            if start > cursor {
                return Err(Error {
                    kind: FailureKind::Gap,
                    message: format!(
                        "message {}: bits {cursor}..{start} are covered by no field; declare them with a `pad` field",
                        message.name
                    ),
                });
            }
            cursor = end;
        }
    }
    Ok(())
}

/// Check, per message, that the declared `max_size` equals the size computed
/// from the fields and their maximum counts (a counted field occupies
/// `width * max_count` bits).
fn check_sizes(model: &Model) -> Result<(), Error> {
    for message in &model.messages {
        let mut total_bits = 0u64;
        for field in &message.fields {
            let bits = field.occupied_bits(&message.name)?;
            total_bits = total_bits.checked_add(bits).ok_or_else(|| {
                parse_error(format!(
                    "message {}: the field sizes overflow u64",
                    message.name
                ))
            })?;
        }
        if !total_bits.is_multiple_of(8) {
            return Err(Error {
                kind: FailureKind::MaxSizeMismatch,
                message: format!(
                    "message {}: the fields cover {total_bits} bits, which is not a whole number of bytes, but max_size = {} is a byte count",
                    message.name, message.max_size
                ),
            });
        }
        let computed = total_bits / 8;
        if computed != message.max_size {
            return Err(Error {
                kind: FailureKind::MaxSizeMismatch,
                message: format!(
                    "message {}: max_size = {} bytes, but the fields and maximum counts compute to {computed} bytes",
                    message.name, message.max_size
                ),
            });
        }
    }
    Ok(())
}

/// Check the 1 252-byte control bound (C-17): it applies to every control-layer
/// message and to every regime stamp section (a message with `section_of`),
/// and the only way past it is the explicit path-MTU probe exemption (C-116).
fn check_control_bound(model: &Model) -> Result<(), Error> {
    for message in &model.messages {
        let bound_applies = message.layer == "control" || message.section_of.is_some();
        if !bound_applies || message.max_size <= CONTROL_BOUND_BYTES {
            continue;
        }
        if message.exempt_from_control_bound.as_deref() == Some(EXEMPTION) {
            continue;
        }
        return Err(Error {
            kind: FailureKind::OversizedControl,
            message: format!(
                "message {}: max_size = {} bytes exceeds the {CONTROL_BOUND_BYTES}-byte control bound (C-17); only the path-MTU probe pair may carry exempt_from_control_bound = \"{EXEMPTION}\" (C-116)",
                message.name, message.max_size
            ),
        });
    }
    Ok(())
}

/// Build the public report from the parsed model, in file order.
fn build_report(model: &Model) -> Report {
    let rejects = model
        .rejects
        .iter()
        .map(|reject| RejectReport {
            name: reject.name.clone(),
            value: reject.value,
            meaning: reject.meaning.clone(),
        })
        .collect();
    let reserves = model
        .reserves
        .iter()
        .map(|reserved| ReservedReport {
            name: reserved.name.clone(),
            message: reserved.message.clone(),
            width: reserved.width,
            value: reserved.value.as_text(),
            live: reserved.live.unwrap_or(false),
            meaning: reserved.meaning.clone(),
            outcome: reserved.outcome.clone(),
        })
        .collect();
    let messages = model
        .messages
        .iter()
        .map(|message| MessageReport {
            name: message.name.clone(),
            layer: message.layer.clone(),
            dscp: message.dscp,
            max_size: message.max_size,
            field_count: message.fields.len(),
            section_of: message.section_of.clone(),
            rejection_reasons: message.rejection_reasons.clone(),
        })
        .collect();
    Report {
        rejects,
        reserves,
        messages,
    }
}

/// Parse, round-trip and check one machine-readable tables file.
///
/// The checks run in this order: parse (from ticket 004-03 this includes
/// rejecting any `rejection_reasons` entry that is not a member of the closed
/// `Reject` enumeration, and any `[[reject]]` declaration that differs from
/// it; from ticket 004-05 the `[[reserved]]` entries are parsed and validated
/// too), round-trip (re-serialise the model, re-parse, compare, reserved
/// entries included), per-message field coverage (overlap, then gap), the
/// declared-versus-computed maximum size, and the 1 252-byte control bound.
///
/// # Errors
///
/// Returns [`Error`] with the appropriate [`FailureKind`] on the first
/// violation found.
pub fn check_file(path: &Path) -> Result<Report, Error> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| parse_error(format!("cannot read {}: {error}", path.display())))?;
    check_str(&source)
}

/// Run the whole check on an in-memory tables file. This is the body of
/// [`check_file`], separated so the round-trip check can reuse the parser.
fn check_str(source: &str) -> Result<Report, Error> {
    let model = parse_model(source)?;
    let canonical = write_canonical(&model);
    let reparsed = parse_model(&canonical).map_err(|error| Error {
        kind: FailureKind::RoundTrip,
        message: format!(
            "the canonical serialisation does not reparse (a bug in the writer): {}",
            error.message
        ),
    })?;
    if reparsed != model {
        let detail = if model
            .messages
            .iter()
            .any(|message| message.size_key == SizeKey::MaximumSize)
        {
            "the deprecated `maximum_size` spelling was used; the canonical writer emits `max_size`, so the reparsed model differs"
        } else {
            "the reparsed model differs from the parsed model"
        };
        return Err(Error {
            kind: FailureKind::RoundTrip,
            message: format!("round-trip failed: {detail}"),
        });
    }
    check_layout(&model)?;
    check_sizes(&model)?;
    check_control_bound(&model)?;
    Ok(build_report(&model))
}
