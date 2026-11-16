//! The closed `Reject` enumeration of ticket 004-03, re-exported as
//! [`crate::reader::Reason`].
//!
//! One hand-written copy, pinned name for name and value for value to the
//! `[[reject]]` tables of `doc/design/spec/wire.toml` by `tests/reason_pin.rs`.
//! `build.rs` includes this same file to validate a vector's `reason`, so the
//! crate holds no second list. The file uses `core` only: it compiles both
//! into the `no_std` library and into the host build script.
//!
//! It is deliberately not generated from `wire.toml`: the build would then
//! read a file outside the crate's directory, so `cargo package` would not
//! carry it and the crate would not build outside this workspace (the
//! embedded and generated-code consumers, epics 025 and 026, may use it
//! there). The pin test catches any drift instead.

/// The closed `Reject` enumeration of ticket 004-03 (`doc/design/spec/wire.md`
/// "Rejection reasons", `wire.toml`), with its `u8` values. Values 13..=255
/// are reserved for a future major version.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Reason {
    /// 0
    TooShort = 0,
    /// 1
    BadMagic = 1,
    /// 2
    VersionMismatch = 2,
    /// 3
    ForeignDomain = 3,
    /// 4
    UnconfiguredDomain = 4,
    /// 5
    UnknownStream = 5,
    /// 6
    UnadmittedSource = 6,
    /// 7
    BadLength = 7,
    /// 8
    BadChecksum = 8,
    /// 9
    BadPacketIndex = 9,
    /// 10
    WrongEpoch = 10,
    /// 11
    OversizeControl = 11,
    /// 12
    ReservedNonZero = 12,
}

impl Reason {
    /// Every reason, in value order.
    pub const ALL: [Reason; 13] = [
        Reason::TooShort,
        Reason::BadMagic,
        Reason::VersionMismatch,
        Reason::ForeignDomain,
        Reason::UnconfiguredDomain,
        Reason::UnknownStream,
        Reason::UnadmittedSource,
        Reason::BadLength,
        Reason::BadChecksum,
        Reason::BadPacketIndex,
        Reason::WrongEpoch,
        Reason::OversizeControl,
        Reason::ReservedNonZero,
    ];

    /// The wire value.
    pub const fn value(self) -> u8 {
        self as u8
    }

    /// The name, as `wire.md` spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Reason::TooShort => "TooShort",
            Reason::BadMagic => "BadMagic",
            Reason::VersionMismatch => "VersionMismatch",
            Reason::ForeignDomain => "ForeignDomain",
            Reason::UnconfiguredDomain => "UnconfiguredDomain",
            Reason::UnknownStream => "UnknownStream",
            Reason::UnadmittedSource => "UnadmittedSource",
            Reason::BadLength => "BadLength",
            Reason::BadChecksum => "BadChecksum",
            Reason::BadPacketIndex => "BadPacketIndex",
            Reason::WrongEpoch => "WrongEpoch",
            Reason::OversizeControl => "OversizeControl",
            Reason::ReservedNonZero => "ReservedNonZero",
        }
    }

    /// The reason with this exact name, if any.
    pub fn from_name(name: &str) -> Option<Reason> {
        Reason::ALL.into_iter().find(|r| r.name() == name)
    }
}
