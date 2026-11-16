//! Ticket 007a-03 review finding C2: `reader::Reason` is pinned to the closed
//! 004-03 enumeration in `doc/design/spec/wire.toml`.
//!
//! The reader's `Reason` is a hand copy of the enumeration, so a renumbered
//! or misspelled variant would otherwise pass every other test. This target
//! reads the `[[reject]]` tables of `wire.toml` with a small line parser (the
//! file's format is fixed by its own header comment: one `name = "..."` and
//! one `value = N` per table, in value order) and asserts that
//! `Reason::ALL` matches them name for name and value for value, and that
//! `from_name` round-trips every entry.

use std::fs;
use std::path::Path;

use hadronyche_vectors::reader::Reason;

/// One `[[reject]]` table of `wire.toml`.
#[derive(Debug, PartialEq)]
struct Pinned {
    name: String,
    value: u8,
}

/// The string value of a `key = "..."` line, if the line has that key.
fn string_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.trim_start().strip_prefix('=')?;
    let rest = rest.trim();
    let inner = rest.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_string())
}

/// The integer value of a `key = N` line, if the line has that key.
fn int_value(line: &str, key: &str) -> Option<u8> {
    let rest = line.strip_prefix(key)?.trim_start().strip_prefix('=')?;
    rest.trim().parse().ok()
}

/// Read every `[[reject]]` table of `doc/design/spec/wire.toml`, in file order.
fn wire_reasons() -> Vec<Pinned> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../doc/design/spec/wire.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));

    let mut out = Vec::new();
    let mut in_reject = false;
    let mut name: Option<String> = None;
    let mut value: Option<u8> = None;
    let mut close = |in_reject: bool, name: &mut Option<String>, value: &mut Option<u8>| {
        if in_reject {
            let (Some(n), Some(v)) = (name.take(), value.take()) else {
                panic!("{}: a [[reject]] table lacks name or value", path.display());
            };
            out.push(Pinned { name: n, value: v });
        }
    };
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            close(in_reject, &mut name, &mut value);
            in_reject = line == "[[reject]]";
        } else if in_reject {
            if let Some(n) = string_value(line, "name") {
                name = Some(n);
            } else if let Some(v) = int_value(line, "value") {
                value = Some(v);
            }
        }
    }
    close(in_reject, &mut name, &mut value);
    out
}

/// C2: `Reason::ALL` has the names and values of `wire.toml`, in order, and
/// `from_name` round-trips every entry.
#[test]
fn reason_is_pinned_to_wire_toml() {
    let pinned = wire_reasons();
    assert_eq!(
        pinned.len(),
        13,
        "wire.toml must declare the closed 13-entry Reject enumeration (004-03); found {}",
        pinned.len()
    );
    assert_eq!(
        Reason::ALL.len(),
        pinned.len(),
        "Reason::ALL must have one entry per wire.toml [[reject]] table"
    );

    for (i, (reason, want)) in Reason::ALL.iter().zip(&pinned).enumerate() {
        assert_eq!(
            reason.name(),
            want.name,
            "Reason::ALL[{i}] is named {:?}; wire.toml names entry {i} {:?}",
            reason.name(),
            want.name
        );
        assert_eq!(
            reason.value(),
            want.value,
            "{}: value {} in Reason, {} in wire.toml",
            want.name,
            reason.value(),
            want.value
        );
    }

    for want in &pinned {
        let found = Reason::from_name(&want.name)
            .unwrap_or_else(|| panic!("from_name({:?}) finds no reason", want.name));
        assert_eq!(
            found.name(),
            want.name,
            "from_name({:?}) must round-trip the name",
            want.name
        );
        assert_eq!(
            found.value(),
            want.value,
            "from_name({:?}) must round-trip the value",
            want.name
        );
    }

    assert!(
        Reason::from_name("NotAReason").is_none(),
        "a name outside the closed enumeration must not resolve"
    );
}
