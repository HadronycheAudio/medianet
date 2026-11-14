# 014-13 — Lineage through a hop
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-01, epic 004, epic 010
## Context
The lineage extension is an optional extension in the slice (production streams may omit it) `[decision]`. It carries:
- **hop count**;
- **accumulated residence time**: each module adds (its output send time − its input arrival time), the same idea as a transparent clock's correction field;
- **accumulated declared delay**: the sum of the structural delays L declared by the modules on the path. Each L is always a whole number of domain-rate samples (C-105), already including any whole-sample delay line the DSL compiler inserted at a join (C-104), so the accumulated value is an integer sum of samples. A resampler node exists only inside an edge module and its conversion delay is part of the edge's D, not of L (C-134).
- **Header cap**: a default slice has at most 100 bytes for all headers and the checksum; the extension fits inside that cap or is omitted on a fallback path (C-137).

Rules `[decision]`:
- Stamps denote content position and are **never restamped per hop**; a module with structural delay L stamps its output p − L. The extension is how deep-graph latency stays attributable: arrival lateness = network + residence + declared delay. The wire has one rate (C-01), so one sample is exactly 1 000 000 000 / 96 000 ns and the three terms convert to one unit.
- Residence is self-accounting (it already appears in downstream lateness). There is one event, "processing overrun", raised when a module completes after its deadline; there is **no** separate latency-violation event, and this ticket raises none. The hosted backend, epic 015, raises processing overrun (C-144).
- Positional delay cannot be observed from timestamps; it is verified by a latency probe tool, not by this path.
- Tests use injected integer timestamps, so expected results are exact (settled by the epic).

Still open (epic open sub-question 10, owner task 004 with this epic): range and saturation of `hops`, `residence_ns` and `declared_delay_samples` within the C-137 header cap, and the rule when send time precedes arrival time. Falsifier: a real path exceeds the field range, or a negative residence is observed on a locked module. This ticket implements the provisional rule below, isolated in one function so the answer changes one place.

Shape from the epic (epic 004 owns the extension layout, epic 010 where it is written in a slice):
```rust
pub struct Lineage { pub hops: u8, pub residence_ns: u32, pub declared_delay_samples: u32 }
impl Lineage { pub fn through(self, arrival_ns: i64, send_ns: i64, structural_delay_samples: u32) -> Lineage; }
```

## Work
1. Implement `Lineage` and `through`: `hops + 1`, `residence_ns + (send_ns - arrival_ns)`, `declared_delay_samples + structural_delay_samples` (integer addition, C-105).
2. Provisional rule pending open sub-question 10, marked `[decision]` provisional in code: saturate `hops` at 255, `residence_ns` at `u32::MAX` (about 4.29 s) and `declared_delay_samples` at the maximum of the width epic 004 gives it; if `send_ns < arrival_ns`, add 0.
3. Implement encode and parse of the extension per epic 004's layout, in the place in the slice given by epic 010; absence of the extension parses as `None`. The encoder writes the extension only when it fits the header cap of the slice being built and omits it otherwise, for example on a fallback path (C-137).
4. Do not add any latency-violation event.

## Acceptance criteria
1. Through three hops with injected residences 120 000, 45 000 and 300 000 ns and structural delays 48, 0 and 96 samples, starting from an empty lineage: `hops == 3`, `residence_ns == 465 000` exactly, `declared_delay_samples == 144` exactly (epic AC-8, first part).
2. 144 samples convert to exactly 1 500 000 ns (three slices, C-01, C-02), so declared delay and residence combine in one unit.
3. Saturation tests for `hops`, `residence_ns` and `declared_delay_samples` and the negative-residence case pass without panic.
4. A slice without the extension parses with lineage `None`; one with it round-trips.
5. No input stamp is altered by `through` (the function does not touch positions).
6. With a slice whose other headers leave less room than the extension needs under the C-137 cap, the extension is omitted and the slice parses with lineage `None`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target delivered in 014-14)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Fan-in rule, vectors and fuzzing (014-14).
- The "processing overrun" event (raised by the hosted backend, epic 015, C-144).
- Latency probe tool for positional delay.
