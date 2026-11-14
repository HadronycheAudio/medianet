# 007a-04 — UnderTest runner trait
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-03

## Context
- Any implementation that wants to be judged against the conformance vectors implements one runner trait. The epic gives its shape; names may be refined `[decision]`:

```rust
// shape, names may be refined
pub trait UnderTest {
    fn decode(&mut self, layer: Layer, bytes: &[u8]) -> Result<Decoded, Reject>;
    fn encode(&mut self, msg: &Decoded, out: &mut [u8]) -> usize;
    fn control_step(&mut self, machine: Machine, now_ns: i64, input: ControlInput) -> ControlOutput;
}
```

- The driver must check vectors in both directions: decode of valid bytes gives the decoded structure; encode of the decoded structure gives the same bytes; decode of invalid bytes gives the expected `Reject` reason.
- Epic criterion 007-AC8 (system criterion AC-23) requires that malformed, wrong-version and foreign-domain inputs produce no crash and no state change, are counted per input and per reason, and that a version mismatch produces a named event. The epic (deliverable 3) therefore extends the trait, still `no_std`, with: reading drop counters per input and per reason; reading the named events emitted; reading a state snapshot or digest; setting the local domain and the sources pinned at admission. The types `Layer`, `Decoded`, `Reject`, `Machine`, `ControlInput` and `ControlOutput` are defined here to match the vector schemas of 007a-02.
- On a real implementation the driver may reach the runtime's test-only injection hooks: force a module id and nonce, force a stream onto a given timeline epoch, send a raw packet on a media or control socket (C-120). The trait must not require more than these hooks plus the observation extension above. The panic-injection hook of C-145 is not used by conformance vectors: it serves the housekeeping-restart criterion, which belongs to the hosted backend (epic 015, C-144).
- The control state machines (`Machine`) are discovery, election (clock holder), admission and connection, whose state tables and scenario scripts come from epic 005 (C-193). Keep `Machine` an enumeration that can grow without breaking the trait. `ControlInput` must be able to carry a reading of a clock that keeps running through suspend, needed by the holder-stall vectors (C-136, 007b-04); connection alarms (C-173) are read as named events.
- Consumers: epics 009–014 (core), 025 (generated code), 026 (embedded).

## Work
1. In `hadronyche-vectors`, define `UnderTest` and the types `Layer`, `Decoded`, `Reject`, `Machine`, `ControlInput`, `ControlOutput`, matching the vector schemas of 007a-02.
2. Add the observation and configuration extension listed in Context. Keep it `no_std`. Record the chosen names in the crate docs, and map each to the C-120 hook it uses on a real implementation where one applies.
3. Write a driver `run_wire(&mut impl UnderTest)` and `run_control(&mut impl UnderTest)` that iterate the vectors through the reader of 007a-03 and return a report: number passed, failed, with vector ids.
4. Give `Machine` one variant per machine of C-193: discovery, election, admission, connection.
5. Write a test that runs the driver against a fake `UnderTest` built for the 007a-02 fixtures and checks pass and fail are reported correctly.

## Acceptance criteria
1. The trait and all its types compile under `#![no_std]`.
2. With a fake implementation that answers correctly, the driver reports 100 % pass on the fixtures; with one that returns a wrong `Reject` reason for 1 fixture, the driver reports exactly 1 failure naming that vector id.
3. Encode-direction checks compare the full output buffer byte for byte and the returned length.
4. The extension lets a test assert "no state change" and "counter for reason R increased by 1" on one input.
5. The extension lets a test set the local domain and the pinned sources before an input.
6. `Machine` has one variant per machine of C-193 (discovery, election, admission, connection).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Any real implementation of the trait (epics 009–014, 025, 026).
- The CI job that runs all vectors (007b-22).
