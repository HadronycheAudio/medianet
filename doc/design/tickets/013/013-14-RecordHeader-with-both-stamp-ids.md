# 013-14 — RecordHeader with both stamp ids
**Epic:** 013-Regime-stamp-wire-object · **Effort:** XS · **Depends on:** 013-01, 013-03

## Context
- Records bind to stamps through a header: `pub struct RecordHeader { pub stamp: StampId, pub peer_stamp: Option<StampId>, pub family: CheckFamily }` in `hadronyche-core`. `[decision]`
- Rule 2: a measurement involving two endpoints carries **both profiles**. 013-AC-5: a two-endpoint record carries **both stamp ids**, and the collector can resolve both.
- In a certification record, "both stamps" are the regime stamps of the sink and of the holder it followed (C-191): `stamp` is the sink's, `peer_stamp` the holder's. The record itself is written by the commissioning tool (epic 028).
- `StampId(pub u32)` is content-derived over regime fields; counters do not change it (013-05).
- `CheckFamily` is owned by epics 015 and 017; until they define it, use a placeholder enum documented as such.
- Fuzz budget (C-95): 60 s per change, 1 hour nightly, corpus kept.

## Work
1. Define `RecordHeader` and a fixed-size encode/decode for it (`no_std`, no allocation).
2. Use the existing `CheckFamily` type if available; otherwise a placeholder enum documented as such.
3. Fuzz the header decoder (a target in the 013-03 harness).

## Acceptance criteria
1. Header round-trips with `peer_stamp = None` and `Some(id)`, for ids 0 and 4 294 967 295.
2. A header with `peer_stamp` present is distinguishable on the wire from one without.
3. Decoding truncated input returns an error, never panics.
4. A certification-record header with the sink's id as `stamp` and the holder's id as `peer_stamp` (C-191) round-trips.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (the header is not a stamp field); "Design documents updated where behaviour differs, constants table first" (no behaviour differs; follows C-191); "No new open item without an owner and a falsifier" (none opened; `CheckFamily` is already an epic open item); "Every new factual claim carries an evidence tag" (no factual claim); "The stamp format is versioned independently of the media format" (the header is not part of the stamp).

## Out of scope
- Writing headers into record sets (013-15), resolving ids (013-17).
- Defining `CheckFamily` (epics 015, 017).
