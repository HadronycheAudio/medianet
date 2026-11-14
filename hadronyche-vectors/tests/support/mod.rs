//! Test support for ticket 007a-02 — the six vector file schemas, as typed
//! Rust definitions, and the validator the acceptance tests drive.
//!
//! This module is compiled only into the `vector_files` test target, so it may
//! use `std`: the crate itself stays `#![no_std]` (007a-01 AC 1, and
//! `checks/007a-01.sh` builds it for a bare-metal target). Integration tests
//! cannot see `#[cfg(test)]` internals, so the whole schema lives here rather
//! than in `src/`.
//!
//! # The contract this module fixes
//!
//! Six vector kinds exist, one schema each (ticket Work 1). Every file is a
//! JSON object carrying the two mandatory fields of Work 2:
//!
//! * `spec_version` — a non-empty opaque string: the version of the
//!   specification the file was generated from. It is opaque on purpose
//!   (Context: the version format is epic 004's open sub-question), so a later
//!   format change needs no schema change. It is **not** the regime stamp
//!   version carried in the hello (C-18).
//! * `id` — a non-empty string, unique across the crate.
//!
//! A vector's kind is fixed by its path, exactly as the ticket's Context
//! lays the directories out:
//!
//! | kind | path component | typed definition |
//! |---|---|---|
//! | wire valid | `wire/valid` | [`WireValid`] |
//! | wire invalid | `wire/invalid` | [`WireInvalid`] |
//! | control scenario | `control` | [`Control`] |
//! | timeline | `timeline` | [`Timeline`] |
//! | transport | `transport` | [`Transport`] |
//! | reference | `reference` | [`Reference`] |
//!
//! The typed definitions below are the normative schema: their `deny_unknown_fields`
//! attributes reject a misnamed key, their required fields reject an absent
//! one, their enums reject a value outside the closed sets, and
//! [`VectorDoc::semantic_errors`] adds the rules JSON types cannot express
//! (the `reason` enumeration, the C-159 counter names, the C-164 bound, the
//! `.bin` sidecar of a wire vector). Unknown fields are rejected everywhere
//! except in free-form containers that a later ticket owns: `decoded` (wire
//! valid), the `args` of a control input and of a transport step, and the
//! `args` of a reference input.
//!
//! ## The rejection reason enumeration
//!
//! An invalid vector's `reason` is a member of the closed `Reject`
//! enumeration landed by ticket 004-03, values 0..=12, in this order
//! (`doc/design/spec/wire.toml`, `doc/design/spec/wire.md`):
//!
//! `TooShort` 0, `BadMagic` 1, `VersionMismatch` 2, `ForeignDomain` 3,
//! `UnconfiguredDomain` 4, `UnknownStream` 5, `UnadmittedSource` 6,
//! `BadLength` 7, `BadChecksum` 8, `BadPacketIndex` 9, `WrongEpoch` 10,
//! `OversizeControl` 11, `ReservedNonZero` 12. Values 13..=255 are reserved
//! for a future major version and are never valid names here.
//!
//! ## The C-159 counter names
//!
//! The per-connection counters a control scenario's expected `state` may
//! carry, snake_case as JSON keys (C-47, C-135, C-159): `received`, `late`,
//! `missing`, `marked_gap`, `other_epoch`, `too_early`, `duplicates`,
//! `incomplete_reassemblies`, `demoted_rounds`, `interrupted_events`, `dead`.
//! Any other key is a schema error, which is what makes the set a *named*
//! set rather than a free-form map.
//!
//! ## The health states
//!
//! [`Health`] carries the connection health states, C-109's `Demoted`
//! included: `Pending`, `Locking`, `Aligning`, `Live`, `Interrupted`,
//! `Demoted`, `Dead` (C-39, C-109, C-154).
//!
//! ## The per-input time bound
//!
//! [`TimeBound`] carries C-164, which is a proposal pending L1: linear in
//! packet length and at most 10 µs per packet. The bound lives in the file,
//! not in a crate-wide constant, so a later L1 revision needs no schema
//! change; 10 µs is therefore a ceiling here, not the only allowed value.

#![allow(dead_code)] // the schemas document fields the criteria need not read

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

/// The crate's data root, `hadronyche-vectors/vectors/`: every file below it
/// is a vector and is validated (ticket Work 6).
pub const VECTORS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/vectors");

/// The test fixture root, `hadronyche-vectors/tests/fixtures/vectors/`.
pub const FIXTURES_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vectors");

/// The closed `Reject` enumeration of ticket 004-03, in value order.
pub const REJECT_REASONS: [&str; 13] = [
    "TooShort",
    "BadMagic",
    "VersionMismatch",
    "ForeignDomain",
    "UnconfiguredDomain",
    "UnknownStream",
    "UnadmittedSource",
    "BadLength",
    "BadChecksum",
    "BadPacketIndex",
    "WrongEpoch",
    "OversizeControl",
    "ReservedNonZero",
];

/// The C-159 per-connection counters, snake_case, as they appear in a control
/// scenario's expected `state.counters`.
pub const C159_COUNTERS: [&str; 11] = [
    "received",
    "late",
    "missing",
    "marked_gap",
    "other_epoch",
    "too_early",
    "duplicates",
    "incomplete_reassemblies",
    "demoted_rounds",
    "interrupted_events",
    "dead",
];

/// The six vector kinds, one schema each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// `vectors/wire/valid/*.bin` + `*.json`.
    WireValid,
    /// `vectors/wire/invalid/*.bin` + `*.json`.
    WireInvalid,
    /// `vectors/control/*.json`.
    Control,
    /// `vectors/timeline/*.json`.
    Timeline,
    /// `vectors/transport/*.json`.
    Transport,
    /// `vectors/reference/*.json`, plus the scripted inputs it names.
    Reference,
}

impl Kind {
    /// All six kinds, in the ticket's Context order.
    pub const ALL: [Kind; 6] = [
        Kind::WireValid,
        Kind::WireInvalid,
        Kind::Control,
        Kind::Timeline,
        Kind::Transport,
        Kind::Reference,
    ];

    /// The kind's name, as used in test failures.
    pub const fn name(self) -> &'static str {
        match self {
            Kind::WireValid => "wire valid",
            Kind::WireInvalid => "wire invalid",
            Kind::Control => "control",
            Kind::Timeline => "timeline",
            Kind::Transport => "transport",
            Kind::Reference => "reference",
        }
    }

    /// The path component that fixes the kind, below `vectors/`.
    pub const fn dir(self) -> &'static str {
        match self {
            Kind::WireValid => "wire/valid",
            Kind::WireInvalid => "wire/invalid",
            Kind::Control => "control",
            Kind::Timeline => "timeline",
            Kind::Transport => "transport",
            Kind::Reference => "reference",
        }
    }

    /// Infer the kind from a path that carries its directory component.
    /// Returns `None` when no component names a kind.
    pub fn from_path(path: &Path) -> Option<Kind> {
        let parts: Vec<&str> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        for pair in parts.windows(2) {
            match (pair[0], pair[1]) {
                ("wire", "valid") => return Some(Kind::WireValid),
                ("wire", "invalid") => return Some(Kind::WireInvalid),
                _ => {}
            }
        }
        for part in parts {
            match part {
                "control" => return Some(Kind::Control),
                "timeline" => return Some(Kind::Timeline),
                "transport" => return Some(Kind::Transport),
                "reference" => return Some(Kind::Reference),
                _ => {}
            }
        }
        None
    }
}

/// The path of the coder's example fixture of one kind, below
/// `tests/fixtures/vectors/examples/`.
pub fn example_path(kind: Kind) -> PathBuf {
    Path::new(FIXTURES_ROOT)
        .join("examples")
        .join(kind.dir())
        .join("example.json")
}

/// The root of the coder's six example fixtures.
pub fn examples_root() -> PathBuf {
    Path::new(FIXTURES_ROOT).join("examples")
}

// ---------------------------------------------------------------- schema ---
//
// Wire valid: one encoding in `bin`, its decoded structure in `decoded`
// (ticket Context: "one encoding in `.bin`, the decoded structure in the
// `.json`"). A message type has at least a minimal, a typical and a maximal
// encoding; `case` records which one this file is.

/// `vectors/wire/valid/*.json` — a valid encoding and its decoded structure.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireValid {
    /// Non-empty opaque specification version stamp.
    pub spec_version: String,
    /// Non-empty identifier, unique across the crate.
    pub id: String,
    /// The message type the encoding belongs to, as named by the wire tables.
    pub message: String,
    /// Whether this is the minimal, typical or maximal encoding of `message`.
    pub case: WireCase,
    /// The encoding's file name, relative to this JSON file.
    pub bin: String,
    /// The encoding's length in bytes; must equal the `.bin` file's size.
    pub length_bytes: u32,
    /// The decoded structure: field name to JSON value. The field-level shape
    /// is 007a-05's; this schema requires an object.
    pub decoded: Value,
}

/// Which encoding of a message type a wire-valid file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WireCase {
    /// The smallest legal encoding.
    Minimal,
    /// A representative encoding with every usual field set.
    Typical,
    /// The largest legal encoding, at the declared maximum size.
    Maximal,
}

/// `vectors/wire/invalid/*.json` — one violating encoding and its required
/// outcome (AC-23, ticket Work 3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireInvalid {
    /// Non-empty opaque specification version stamp.
    pub spec_version: String,
    /// Non-empty identifier, unique across the crate.
    pub id: String,
    /// The message type whose encoding is violated.
    pub message: String,
    /// The violating input, relative to this JSON file.
    pub bin: String,
    /// The violating input's length in bytes; must equal the `.bin` file's
    /// size.
    pub length_bytes: u32,
    /// The required rejection reason: a member of [`REJECT_REASONS`].
    pub reason: String,
    /// The other required outcomes: no state change, the counted drop and,
    /// optionally, the named event and the time bound.
    pub expected: InvalidExpected,
}

/// The required outcome of one invalid input.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvalidExpected {
    /// AC-23: a bad input never changes state. Must be `true`.
    pub no_state_change: bool,
    /// The expected drop counts: one per (input, reason) pair (AC-23).
    #[serde(default)]
    pub counter_increments: Vec<CounterIncrement>,
    /// The expected named events; `VersionMismatch` raises one.
    #[serde(default)]
    pub events: Vec<NamedEvent>,
    /// The per-input time bound (C-164); optional, present on the example.
    #[serde(default)]
    pub time_bound: Option<TimeBound>,
}

/// One expected (input, reason) counter increment.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterIncrement {
    /// The input the drop is counted on.
    pub input: String,
    /// The reason half of the key: a member of [`REJECT_REASONS`].
    pub reason: String,
    /// The increment; at least one.
    pub count: u32,
}

/// One expected named event.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedEvent {
    /// The event's name, non-empty.
    pub name: String,
}

/// The per-input time bound, C-164: linear in packet length and at most
/// 10 µs per packet.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeBound {
    /// Microseconds per packet; C-164 fixes the ceiling at 10.
    pub per_packet_us: u32,
    /// Whether the bound is linear in packet length; C-164 requires it.
    pub linear_in_length: bool,
}

/// `vectors/control/*.json` — an ordered scenario for a named state machine.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    /// Non-empty opaque specification version stamp.
    pub spec_version: String,
    /// Non-empty identifier, unique across the crate.
    pub id: String,
    /// The state machine the scenario drives.
    pub machine: Machine,
    /// The ordered steps; at least one.
    pub steps: Vec<ControlStep>,
}

/// The control state machines, from epic 005 (C-193).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Machine {
    /// Module discovery.
    Discovery,
    /// Clock-holder election.
    Election,
    /// Stream admission.
    Admission,
    /// Connection health.
    Connection,
}

/// One `(time, input) -> expected` step of a control scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlStep {
    /// The step's time, ns.
    pub time_ns: i64,
    /// The input presented at that time.
    pub input: ControlInput,
    /// The expected outcome at that time.
    pub expected: ControlExpected,
}

/// A control input.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlInput {
    /// The input's kind, non-empty.
    pub kind: String,
    /// Free-form input arguments (007a-04's `ControlInput`).
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
}

/// The outcome expected after one control step.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlExpected {
    /// The outputs the step produces (007a-04's `ControlOutput`).
    #[serde(default)]
    pub outputs: Vec<Value>,
    /// The named events raised; connection alarms (C-173) live here.
    #[serde(default)]
    pub events: Vec<NamedEvent>,
    /// The resulting state.
    pub state: ControlState,
}

/// The resulting state of a control step: the stamp's per-connection counters
/// (C-159) and the health state (C-109).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlState {
    /// C-159 counters, keyed by their snake_case names; a subset of
    /// [`C159_COUNTERS`].
    #[serde(default)]
    pub counters: BTreeMap<String, u64>,
    /// The connection health state (C-109).
    pub health: Health,
    /// The lateness percentiles (C-160), when the stamp reports them.
    #[serde(default)]
    pub lateness_percentiles: Option<LatenessPercentiles>,
    /// The connection alarms raised (C-173).
    #[serde(default)]
    pub alarms: Vec<NamedEvent>,
}

/// The connection health states (C-39, C-109, C-154).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Health {
    /// Not yet admitted.
    Pending,
    /// Acquiring the clock.
    Locking,
    /// Aligning.
    Aligning,
    /// Satisfied (C-154).
    Live,
    /// Interrupted (C-38).
    Interrupted,
    /// Demoted at the ratchet ceiling (C-109).
    Demoted,
    /// Dead (C-39).
    Dead,
}

/// The lateness percentiles of C-160.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LatenessPercentiles {
    /// 50th percentile, ns.
    pub p50_ns: u64,
    /// 99th percentile, ns.
    pub p99_ns: u64,
    /// 99.9th percentile, ns.
    pub p999_ns: u64,
    /// Maximum since the connection went live, ns.
    pub max_ns: u64,
}

/// `vectors/timeline/*.json` — position/time conversions and epoch-switch
/// scenarios (epic 006).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    /// Non-empty opaque specification version stamp.
    pub spec_version: String,
    /// Non-empty identifier, unique across the crate.
    pub id: String,
    /// Position/time conversions; a file carries at least one conversion or
    /// at least one epoch switch.
    #[serde(default)]
    pub conversions: Vec<PositionConversion>,
    /// Epoch-switch scenarios.
    #[serde(default)]
    pub epoch_switches: Vec<EpochSwitch>,
}

/// One position/time conversion.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionConversion {
    /// The position, in domain-rate samples.
    pub position: u64,
    /// The network time the position maps to, ns.
    pub time_ns: i64,
}

/// One timeline-epoch switch scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpochSwitch {
    /// The epoch the scenario starts on (16 bits, C-10).
    pub from_epoch: u16,
    /// The epoch it switches to.
    pub to_epoch: u16,
    /// The position at which the switch is announced.
    pub at_position: u64,
    /// The named events the switch raises.
    #[serde(default)]
    pub expected_events: Vec<NamedEvent>,
}

/// `vectors/transport/*.json` — a scripted transport scenario and its
/// expected result (C-141, C-184).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transport {
    /// Non-empty opaque specification version stamp.
    pub spec_version: String,
    /// Non-empty identifier, unique across the crate.
    pub id: String,
    /// The scripted scenario; at least one step.
    pub script: Vec<TransportStep>,
    /// The expected result.
    pub expected: TransportExpected,
}

/// One scripted transport action.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportStep {
    /// When the action happens, ns.
    pub at_ns: i64,
    /// The action's kind, non-empty (tempo, time signature, stop, ...).
    pub kind: String,
    /// Free-form action arguments; 007b owns the set.
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
}

/// The expected result of a transport scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportExpected {
    /// What the scenario asserts, non-empty.
    pub result: String,
    /// Expected click positions, in samples.
    #[serde(default)]
    pub clicks: Vec<i64>,
    /// Named events raised.
    #[serde(default)]
    pub events: Vec<NamedEvent>,
    /// Gap-class counters, keyed like [`C159_COUNTERS`].
    #[serde(default)]
    pub counters: BTreeMap<String, u64>,
}

/// `vectors/reference/*.json` — scripted inputs and the expected output hash of
/// a reference module, in f32 or Q1.31 (007b owns the hash algorithm and the
/// script's inner format; this schema reserves the fields).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    /// Non-empty opaque specification version stamp.
    pub spec_version: String,
    /// Non-empty identifier, unique across the crate.
    pub id: String,
    /// The reference module.
    pub reference: ReferenceModule,
    /// The sample format (`f32` or `q1.31`).
    pub sample_format: SampleFormat,
    /// The scripted inputs; at least one.
    pub input_script: Vec<ReferenceInput>,
    /// The expected output hash, opaque until 007b-18 picks the algorithm.
    pub expected_output_hash: String,
    /// The gap-related counts.
    pub gap_counts: GapCounts,
}

/// The reference module a file is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceModule {
    /// The reference echo.
    Echo,
    /// The reference mixer.
    Mixer,
}

/// The sample format of a reference vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SampleFormat {
    /// IEEE 754 single precision (C-03).
    #[serde(rename = "f32")]
    F32,
    /// Q1.31 fixed point (C-03).
    #[serde(rename = "q1.31")]
    Q1_31,
}

/// One scripted reference input.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceInput {
    /// When the input is presented, ns.
    pub at_ns: i64,
    /// The input's kind, non-empty; 007b owns the set.
    pub kind: String,
    /// Free-form input arguments; 007b owns the set.
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
}

/// The gap-related counts of a reference run: windows touched by loss and
/// degraded slices are excluded and counted.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GapCounts {
    /// Windows excluded because loss or a degraded slice touched them.
    pub excluded_windows: u64,
    /// Output slices marked degraded (C-122).
    pub degraded_slices: u64,
}

// ------------------------------------------------------------ validation ---

/// A document that passed validation, with the fields every kind carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated {
    /// The validated file.
    pub path: PathBuf,
    /// Its kind, from its path.
    pub kind: Kind,
    /// Its `id`.
    pub id: String,
    /// Its `spec_version`.
    pub spec_version: String,
}

impl Validated {
    /// The common fields, in the order the error messages print them.
    fn new(path: &Path, kind: Kind, id: String, spec_version: String) -> Validated {
        Validated {
            path: path.to_path_buf(),
            kind,
            id,
            spec_version,
        }
    }
}

/// A vector document whose schema this module fixes.
pub trait VectorDoc: for<'de> Deserialize<'de> {
    /// The kind this document implements.
    const KIND: Kind;
    /// The mandatory specification version stamp.
    fn spec_version(&self) -> &str;
    /// The mandatory crate-wide identifier.
    fn id(&self) -> &str;
    /// The rules the typed fields cannot express, with `json_path` present
    /// when the document came from a file (for the `.bin` sidecar).
    fn semantic_errors(&self, json_path: Option<&Path>) -> Vec<String>;
}

impl VectorDoc for WireValid {
    const KIND: Kind = Kind::WireValid;

    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn semantic_errors(&self, json_path: Option<&Path>) -> Vec<String> {
        let mut errors = Vec::new();
        non_empty("message", &self.message, &mut errors);
        non_empty("bin", &self.bin, &mut errors);
        if !self.decoded.is_object() {
            errors.push("`decoded` must be an object (the decoded structure)".to_owned());
        }
        bin_errors(json_path, &self.bin, self.length_bytes, &mut errors);
        errors
    }
}

impl VectorDoc for WireInvalid {
    const KIND: Kind = Kind::WireInvalid;

    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn semantic_errors(&self, json_path: Option<&Path>) -> Vec<String> {
        let mut errors = Vec::new();
        non_empty("message", &self.message, &mut errors);
        non_empty("bin", &self.bin, &mut errors);
        reject_reason("reason", &self.reason, &mut errors);
        if !self.expected.no_state_change {
            errors.push(
                "`expected.no_state_change` must be true: a bad input never changes state \
                 (AC-23)"
                    .to_owned(),
            );
        }
        for (i, inc) in self.expected.counter_increments.iter().enumerate() {
            non_empty(
                &format!("expected.counter_increments[{i}].input"),
                &inc.input,
                &mut errors,
            );
            reject_reason(
                &format!("expected.counter_increments[{i}].reason"),
                &inc.reason,
                &mut errors,
            );
            if inc.count == 0 {
                errors.push(format!(
                    "`expected.counter_increments[{i}].count` must be at least 1"
                ));
            }
        }
        for (i, event) in self.expected.events.iter().enumerate() {
            non_empty(
                &format!("expected.events[{i}].name"),
                &event.name,
                &mut errors,
            );
        }
        if let Some(bound) = &self.expected.time_bound {
            if bound.per_packet_us == 0 || bound.per_packet_us > 10 {
                errors.push(format!(
                    "`expected.time_bound.per_packet_us` is {}: C-164 bounds one bad input \
                     at at most 10 µs per packet",
                    bound.per_packet_us
                ));
            }
            if !bound.linear_in_length {
                errors.push(
                    "`expected.time_bound.linear_in_length` must be true: C-164 bounds the \
                     work linear in packet length"
                        .to_owned(),
                );
            }
        }
        bin_errors(json_path, &self.bin, self.length_bytes, &mut errors);
        errors
    }
}

impl VectorDoc for Control {
    const KIND: Kind = Kind::Control;

    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn semantic_errors(&self, _json_path: Option<&Path>) -> Vec<String> {
        let mut errors = Vec::new();
        if self.steps.is_empty() {
            errors.push("`steps` must hold at least one step".to_owned());
        }
        for (i, step) in self.steps.iter().enumerate() {
            non_empty(
                &format!("steps[{i}].input.kind"),
                &step.input.kind,
                &mut errors,
            );
            for (j, event) in step.expected.events.iter().enumerate() {
                non_empty(
                    &format!("steps[{i}].expected.events[{j}].name"),
                    &event.name,
                    &mut errors,
                );
            }
            for (j, alarm) in step.expected.state.alarms.iter().enumerate() {
                non_empty(
                    &format!("steps[{i}].expected.state.alarms[{j}].name"),
                    &alarm.name,
                    &mut errors,
                );
            }
            for name in step.expected.state.counters.keys() {
                if !C159_COUNTERS.contains(&name.as_str()) {
                    errors.push(format!(
                        "`steps[{i}].expected.state.counters.{name}` is not a C-159 counter \
                         (the closed set is {})",
                        C159_COUNTERS.join(", ")
                    ));
                }
            }
            if let Some(p) = &step.expected.state.lateness_percentiles
                && !(p.p50_ns <= p.p99_ns && p.p99_ns <= p.p999_ns && p.p999_ns <= p.max_ns)
            {
                errors.push(format!(
                    "`steps[{i}].expected.state.lateness_percentiles` must be ordered \
                     p50 <= p99 <= p999 <= max (C-160)"
                ));
            }
        }
        errors
    }
}

impl VectorDoc for Timeline {
    const KIND: Kind = Kind::Timeline;

    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn semantic_errors(&self, _json_path: Option<&Path>) -> Vec<String> {
        let mut errors = Vec::new();
        if self.conversions.is_empty() && self.epoch_switches.is_empty() {
            errors.push(
                "a timeline file must carry at least one conversion or one epoch switch \
                 (ticket Context)"
                    .to_owned(),
            );
        }
        for (i, switch) in self.epoch_switches.iter().enumerate() {
            if switch.from_epoch == switch.to_epoch {
                errors.push(format!(
                    "`epoch_switches[{i}]`: `from_epoch` and `to_epoch` must differ"
                ));
            }
            for (j, event) in switch.expected_events.iter().enumerate() {
                non_empty(
                    &format!("epoch_switches[{i}].expected_events[{j}].name"),
                    &event.name,
                    &mut errors,
                );
            }
        }
        errors
    }
}

impl VectorDoc for Transport {
    const KIND: Kind = Kind::Transport;

    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn semantic_errors(&self, _json_path: Option<&Path>) -> Vec<String> {
        let mut errors = Vec::new();
        if self.script.is_empty() {
            errors.push("`script` must hold at least one step".to_owned());
        }
        for (i, step) in self.script.iter().enumerate() {
            non_empty(&format!("script[{i}].kind"), &step.kind, &mut errors);
        }
        non_empty("expected.result", &self.expected.result, &mut errors);
        for (j, event) in self.expected.events.iter().enumerate() {
            non_empty(
                &format!("expected.events[{j}].name"),
                &event.name,
                &mut errors,
            );
        }
        for name in self.expected.counters.keys() {
            if !C159_COUNTERS.contains(&name.as_str()) {
                errors.push(format!(
                    "`expected.counters.{name}` is not a C-159 counter (the closed set is {})",
                    C159_COUNTERS.join(", ")
                ));
            }
        }
        errors
    }
}

impl VectorDoc for Reference {
    const KIND: Kind = Kind::Reference;

    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn semantic_errors(&self, _json_path: Option<&Path>) -> Vec<String> {
        let mut errors = Vec::new();
        if self.input_script.is_empty() {
            errors.push("`input_script` must hold at least one input".to_owned());
        }
        for (i, input) in self.input_script.iter().enumerate() {
            non_empty(&format!("input_script[{i}].kind"), &input.kind, &mut errors);
        }
        non_empty(
            "expected_output_hash",
            &self.expected_output_hash,
            &mut errors,
        );
        errors
    }
}

/// Validate an in-memory document against the schema of `kind`. Used for the
/// schema's negative cases that need no file, such as an unknown counter name
/// or an unknown sample format.
pub fn validate_json(kind: Kind, value: &Value) -> Result<(), Vec<String>> {
    check(kind, value, None).map(|_| ())
}

/// Validate one vector file against the schema its path names.
pub fn validate_file(path: &Path) -> Result<Validated, Vec<String>> {
    let kind = Kind::from_path(path).ok_or_else(|| {
        vec![format!(
            "{}: cannot infer the vector kind from the path; expected a `wire/valid`, \
             `wire/invalid`, `control`, `timeline`, `transport` or `reference` component",
            path.display()
        )]
    })?;
    let text = fs::read_to_string(path).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| vec![format!("{}: not valid JSON: {e}", path.display())])?;
    let doc = check(kind, &value, Some(path))?;
    Ok(Validated::new(path, kind, doc.id, doc.spec_version))
}

/// Validate every `*.json` file below `root`, and require the ids to be unique
/// across the set (Work 2: "unique across the crate").
pub fn validate_tree(root: &Path) -> Result<Vec<Validated>, Vec<String>> {
    let mut files = Vec::new();
    walk_json(root, &mut files).map_err(|e| vec![format!("{}: {e}", root.display())])?;
    files.sort();

    let mut errors = Vec::new();
    let mut accepted = Vec::new();
    for file in files {
        match validate_file(&file) {
            Ok(doc) => accepted.push(doc),
            Err(mut e) => errors.append(&mut e),
        }
    }

    let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();
    for doc in &accepted {
        if let Some(first) = seen.get(&doc.id) {
            errors.push(format!(
                "duplicate id \"{}\": {} and {} (ids are unique across the crate)",
                doc.id,
                first.display(),
                doc.path.display()
            ));
        } else {
            seen.insert(doc.id.clone(), doc.path.clone());
        }
    }

    if errors.is_empty() {
        Ok(accepted)
    } else {
        Err(errors)
    }
}

/// The identity fields every kind carries, after validation.
#[derive(Debug)]
struct DocIdentity {
    id: String,
    spec_version: String,
}

/// Deserialize `value` to the kind's typed document and run every rule.
fn check(kind: Kind, value: &Value, json_path: Option<&Path>) -> Result<DocIdentity, Vec<String>> {
    match kind {
        Kind::WireValid => check_doc::<WireValid>(value, json_path),
        Kind::WireInvalid => check_doc::<WireInvalid>(value, json_path),
        Kind::Control => check_doc::<Control>(value, json_path),
        Kind::Timeline => check_doc::<Timeline>(value, json_path),
        Kind::Transport => check_doc::<Transport>(value, json_path),
        Kind::Reference => check_doc::<Reference>(value, json_path),
    }
}

fn check_doc<T: VectorDoc>(
    value: &Value,
    json_path: Option<&Path>,
) -> Result<DocIdentity, Vec<String>> {
    let at = match json_path {
        Some(path) => path.display().to_string(),
        None => "<in-memory>".to_owned(),
    };
    let doc: T = match serde_json::from_value(value.clone()) {
        Ok(doc) => doc,
        Err(e) => return Err(vec![format!("{at}: schema: {e}")]),
    };

    let mut errors = Vec::new();
    if doc.spec_version().trim().is_empty() {
        errors.push(format!(
            "{at}: `spec_version` must be a non-empty opaque string (the specification \
             version the file was generated from)"
        ));
    }
    if doc.id().trim().is_empty() {
        errors.push(format!("{at}: `id` must be a non-empty string"));
    }
    errors.extend(
        doc.semantic_errors(json_path)
            .into_iter()
            .map(|e| format!("{at}: {e}")),
    );

    if errors.is_empty() {
        Ok(DocIdentity {
            id: doc.id().to_owned(),
            spec_version: doc.spec_version().to_owned(),
        })
    } else {
        Err(errors)
    }
}

/// Recursively collect `*.json` files below `root`, in sorted order.
fn walk_json(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(root)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk_json(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "json") {
            out.push(path);
        }
    }
    Ok(())
}

fn non_empty(field: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("`{field}` must be a non-empty string"));
    }
}

fn reject_reason(field: &str, value: &str, errors: &mut Vec<String>) {
    if !REJECT_REASONS.contains(&value) {
        errors.push(format!(
            "`{field}` is \"{value}\", not a member of the closed `Reject` enumeration of \
             ticket 004-03 ({})",
            REJECT_REASONS.join(", ")
        ));
    }
}

/// Check a wire vector's `.bin` sidecar: it exists and its size equals
/// `length_bytes`. A no-op for an in-memory document.
fn bin_errors(json_path: Option<&Path>, bin: &str, length_bytes: u32, errors: &mut Vec<String>) {
    let Some(json_path) = json_path else {
        return;
    };
    if bin.trim().is_empty() {
        return;
    }
    let bin_path = json_path.parent().unwrap_or(Path::new(".")).join(bin);
    match fs::metadata(&bin_path) {
        Ok(meta) if meta.len() == u64::from(length_bytes) => {}
        Ok(meta) => errors.push(format!(
            "`length_bytes` is {length_bytes} but {} is {} bytes",
            bin_path.display(),
            meta.len()
        )),
        Err(e) => errors.push(format!(
            "the encoding {} is not readable: {e}",
            bin_path.display()
        )),
    }
}
