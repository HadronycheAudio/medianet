# 014-14 — Lineage fan-in, vectors and fuzzing
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-13, epic 007
## Context
At a fan-in (a module with several inputs), the **critical input's** lineage values propagate: the input that determined the offset `[decision]`. Lineage = `{ hops: u8, residence_ns: u32, declared_delay_samples: u32 }` (014-13; declared delay is a whole number of domain-rate samples, C-105). The epic settles the bad-index case: an input index that does not exist, or an empty input list, yields no value (`None`) rather than a default. System criterion AC-1 requires extension vectors in both directions (encoding the vector values gives the vector bytes; parsing the vector bytes gives the vector values), kept in the vectors crate at the workspace root (C-192), with the layout and reject reasons supplied by epic 004 (C-193); every parser is fuzzed with the C-95 budget (60 s on every change, 1 hour nightly, corpus kept).

Signature from the epic:
```rust
impl Lineage { pub fn critical(inputs: &[Lineage], critical: usize) -> Option<Lineage>; }   // fan-in rule
```

## Work
1. Implement `Lineage::critical`: return `Some(inputs[critical])` unchanged, `None` for an out-of-range index or empty slice, without panicking.
2. Add lineage extension vectors (epic 007 format): absent, zero, typical, saturated hop count, residence and declared delay, a typical declared delay in whole samples.
3. Add a fuzz target `lineage_parse` (with the team's chosen fuzzing tool, C-192) on the extension parser with a round-trip property, run 60 s on every change and 1 hour nightly, corpus kept (C-95). The bad-input time bound of C-164 is checked in 014-25.

## Acceptance criteria
1. With three inputs of distinct values and `critical = 1`, the result is `Some` of input 1 field by field, and then `through` applies to it (epic AC-8, fan-in part).
2. Out-of-range `critical` and empty `inputs` return `None` and do not panic.
3. Lineage vectors pass in both directions (epic AC-1).
4. The fuzz target runs at the C-95 budget with zero panics.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Choosing which input is critical (the offset logic that determines it belongs to the module runtime).
