//! The `no_std` vector reader (ticket 007a-03).
//!
//! Enumerates the vectors embedded in the crate at build time, without `std`
//! and without an allocator. Every item is a [`Vector`]: a copyable handle on
//! `'static` data yielding its [`Kind`], `id`, `spec_version` (exactly as the
//! file records it), the raw bytes of a wire vector, and its recorded
//! [`Outcome`]: the decoded structure of a valid wire vector, the rejection
//! reason of an invalid one.
//!
//! # Two embedded sets, both always present
//!
//! * **The conformance set**: every vector under the crate's `vectors/`
//!   directory (007a-05 onward), enumerated by [`all`], [`of_kind`], [`find`]
//!   and [`with_bytes`]. It is empty until 007a-05 lands data.
//! * **The examples**: the six hand-written example fixtures of 007a-02
//!   (`tests/fixtures/vectors/examples/`), enumerated only by [`examples`] and
//!   [`examples_of_kind`]. They are test fixtures, not conformance vectors: no
//!   example is reachable through the conformance functions.
//!
//! No cargo feature changes what any enumeration yields, so a device, a host
//! tool and a test all see the same two sets. On a device the examples cost
//! 5 048 bytes of read-only data (measured on x86_64, 2026-11-16, by walking
//! the tables and counting each distinct static once: the 6 `Vector` entries
//! at 120 bytes each, `Field`s at 40 bytes, `Value`s at 24 bytes, string bytes
//! and wire bytes). Another 64-bit target is the same; a 32-bit one is
//! smaller.
//!
//! # Nothing is parsed at run time
//!
//! The build script (`build.rs`) parses every vector file on the host and
//! generates constant Rust data: strings, numbers and nested slices, with the
//! wire bytes embedded by `include_bytes!`. A malformed file fails the build.
//! At run time the reader only walks static tables; the only functions that
//! take caller data are the lookups [`find`] (by id), [`with_bytes`] (by
//! encoding) and [`Reason::from_name`]. Those are what the `vector-reader`
//! fuzz target exercises.
//!
//! The reader does not decode wire bytes: the decoded structure and the
//! rejection reason are the ones the vector file *records*. Decoding is the
//! implementation under test's job (epic 009 onward, through 007a-04's runner).
//!
//! # File layout the build relies on
//!
//! Every `*.json` file below `vectors/` is a vector of the kind its directory
//! names, as 007a-02's validator (`validate_tree` in `tests/support`) also
//! requires. Files with other extensions are ignored unless a wire vector
//! names them as its `.bin`. The scripted inputs 007a-02 allows under
//! `vectors/reference/` ("any file type") must therefore not use the `.json`
//! extension; 007a-05 and 007b-18, which write them, must respect this.
//!
//! The order within a set is fixed: by kind in [`Kind::ALL`] order, then by
//! the file's path.

pub use crate::reason::Reason;

mod generated {
    //! The tables `build.rs` writes: one per set and kind, plus one
    //! `DECODED_n` static per wire-valid vector, shared by its outcome and
    //! its document.
    #![allow(
        clippy::approx_constant,
        clippy::excessive_precision,
        clippy::unreadable_literal
    )]
    // A kind whose directories are empty names none of these types.
    #[allow(unused_imports)]
    use super::{Case, Decoded, Field, Kind, Outcome, Reason, Rejection, Value, Vector};
    include!(concat!(env!("OUT_DIR"), "/vectors.rs"));
}

/// The six vector kinds of 007a-02, fixed by the directory a file sits in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Kind {
    /// `vectors/wire/valid/`: an encoding and its decoded structure.
    WireValid,
    /// `vectors/wire/invalid/`: a violating input and its rejection reason.
    WireInvalid,
    /// `vectors/control/`: a control state-machine scenario.
    Control,
    /// `vectors/timeline/`: position/time conversions and epoch switches.
    Timeline,
    /// `vectors/transport/`: a scripted transport scenario.
    Transport,
    /// `vectors/reference/`: reference echo and mixer scripts and hashes.
    Reference,
}

impl Kind {
    /// All six kinds, in enumeration order.
    pub const ALL: [Kind; 6] = [
        Kind::WireValid,
        Kind::WireInvalid,
        Kind::Control,
        Kind::Timeline,
        Kind::Transport,
        Kind::Reference,
    ];

    /// The directory below `vectors/` that holds this kind.
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

    /// The conformance table of this kind.
    fn table(self) -> &'static [Vector] {
        match self {
            Kind::WireValid => generated::WIRE_VALID,
            Kind::WireInvalid => generated::WIRE_INVALID,
            Kind::Control => generated::CONTROL,
            Kind::Timeline => generated::TIMELINE,
            Kind::Transport => generated::TRANSPORT,
            Kind::Reference => generated::REFERENCE,
        }
    }

    /// The example table of this kind.
    fn example_table(self) -> &'static [Vector] {
        match self {
            Kind::WireValid => generated::EXAMPLE_WIRE_VALID,
            Kind::WireInvalid => generated::EXAMPLE_WIRE_INVALID,
            Kind::Control => generated::EXAMPLE_CONTROL,
            Kind::Timeline => generated::EXAMPLE_TIMELINE,
            Kind::Transport => generated::EXAMPLE_TRANSPORT,
            Kind::Reference => generated::EXAMPLE_REFERENCE,
        }
    }
}

/// One embedded vector: a copyable handle on `'static` data.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector {
    kind: Kind,
    id: &'static str,
    spec_version: &'static str,
    bytes: &'static [u8],
    outcome: Outcome,
    document: &'static [Field],
}

impl Vector {
    /// The kind, from the directory the file sits in.
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// The `id`, unique across the embedded set.
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// The `spec_version` the file was generated from, unchanged: an opaque
    /// string (its format is epic 004's open sub-question).
    pub const fn spec_version(&self) -> &'static str {
        self.spec_version
    }

    /// The content of the `.bin` file of a wire vector, byte for byte; empty
    /// for the other kinds.
    pub const fn bytes(&self) -> &'static [u8] {
        self.bytes
    }

    /// The recorded outcome: [`Outcome::Valid`] for wire valid,
    /// [`Outcome::Rejected`] for wire invalid, [`Outcome::NotWire`] otherwise.
    pub const fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// The whole vector file as a JSON object: every member, in key order.
    /// The scenario kinds' inputs and expectations, and an invalid vector's
    /// expected counters, events and time bound, are read from here.
    pub const fn document(&self) -> Value {
        Value::Object(self.document)
    }
}

/// What a vector file records as the result of presenting its input.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    /// A wire-valid vector: its bytes decode to this structure.
    Valid(Decoded),
    /// A wire-invalid vector: its bytes are rejected for this reason.
    Rejected(Rejection),
    /// A control, timeline, transport or reference vector: not a single
    /// decode; its expectations are in [`Vector::document`].
    NotWire,
}

/// The decoded structure a wire-valid vector records (its `decoded` member),
/// with the message type and the encoding case.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Decoded {
    message: &'static str,
    case: Case,
    value: Value,
}

impl Decoded {
    /// The message type, as named by the wire tables.
    pub const fn message(&self) -> &'static str {
        self.message
    }

    /// Whether the encoding is the minimal, typical or maximal one.
    pub const fn case(&self) -> Case {
        self.case
    }

    /// The decoded structure: a JSON object, field name to value. The
    /// field-level shape is 007a-05's.
    pub const fn value(&self) -> Value {
        self.value
    }
}

/// Which encoding of a message type a wire-valid vector holds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Case {
    /// The smallest legal encoding.
    Minimal,
    /// A representative encoding.
    Typical,
    /// The largest legal encoding.
    Maximal,
}

/// The rejection a wire-invalid vector records.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rejection {
    reason: Reason,
}

impl Rejection {
    /// The reason.
    pub const fn reason(&self) -> Reason {
        self.reason
    }

    /// The reason's name, as the vector file and `wire.md` spell it.
    pub const fn name(&self) -> &'static str {
        self.reason.name()
    }
}

/// A JSON value recorded in a vector file, as constant data.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// An integer that fits `i64`.
    Int(i64),
    /// An integer above `i64::MAX`.
    UInt(u64),
    /// A number with a fraction or an exponent.
    Float(f64),
    /// A string.
    Str(&'static str),
    /// An array.
    Array(&'static [Value]),
    /// An object: its members in key order.
    Object(&'static [Field]),
}

impl Value {
    /// The member `key` of an object; `None` for a missing member or a
    /// value that is not an object.
    pub fn get(self, key: &str) -> Option<Value> {
        self.as_object()?
            .iter()
            .find(|f| f.name == key)
            .map(|f| f.value)
    }

    /// The string, if this is one.
    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The boolean, if this is one.
    pub const fn as_bool(self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// The integer, if this is one that fits `i64`.
    pub const fn as_i64(self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }

    /// The integer, if this is a non-negative one.
    pub const fn as_u64(self) -> Option<u64> {
        match self {
            Value::Int(i) if i >= 0 => Some(i as u64),
            Value::UInt(u) => Some(u),
            _ => None,
        }
    }

    /// The array's items, if this is an array.
    pub const fn as_array(self) -> Option<&'static [Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The object's members, if this is an object.
    pub const fn as_object(self) -> Option<&'static [Field]> {
        match self {
            Value::Object(fields) => Some(fields),
            _ => None,
        }
    }
}

/// One member of a recorded JSON object.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Field {
    name: &'static str,
    value: Value,
}

impl Field {
    /// The member's name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The member's value.
    pub const fn value(&self) -> Value {
        self.value
    }
}

/// Every vector of the conformance set, by kind in [`Kind::ALL`] order, then
/// by path. Never yields an example.
pub fn all() -> impl Iterator<Item = Vector> + Clone {
    Kind::ALL.into_iter().flat_map(of_kind)
}

/// The conformance vectors of one kind, by path.
pub fn of_kind(kind: Kind) -> impl ExactSizeIterator<Item = Vector> + Clone {
    kind.table().iter().copied()
}

/// The conformance vector with this `id`, if any. Never finds an example.
pub fn find(id: &str) -> Option<Vector> {
    all().find(|v| v.id == id)
}

/// The conformance wire vectors (valid and invalid) whose bytes equal
/// `bytes`, in enumeration order: a lookup of the recorded outcome of an
/// encoding, not a decoder. Never yields an example.
pub fn with_bytes(bytes: &[u8]) -> impl Iterator<Item = Vector> {
    of_kind(Kind::WireValid)
        .chain(of_kind(Kind::WireInvalid))
        .filter(move |v| v.bytes == bytes)
}

/// The six example fixtures of 007a-02, by kind in [`Kind::ALL`] order: test
/// fixtures, not conformance vectors.
pub fn examples() -> impl Iterator<Item = Vector> + Clone {
    Kind::ALL.into_iter().flat_map(examples_of_kind)
}

/// The example fixtures of one kind.
pub fn examples_of_kind(kind: Kind) -> impl ExactSizeIterator<Item = Vector> + Clone {
    kind.example_table().iter().copied()
}
