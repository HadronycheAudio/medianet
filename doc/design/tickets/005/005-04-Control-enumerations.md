# 005-04 — Control enumerations
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-01

## Context
- Epic 005 deliverable 3: the closed enumerations of refusal reasons, connection states and control events used by all four control machines. They live in the `no_std` core crate `hadronyche-core`, which holds all control logic as pure state machines.
- Shape given by the epic ("names may be refined"):

```rust
pub enum RefuseReason {
    PortCeiling, PacketRate, SliceRate, InputCount, BufferMemory, CostBound,
    ContentMismatch, NotAnInput, VersionMismatch, ForeignDomain,
    PathMtuTooSmall, RequiredEExceeded, FollowerCap, ModuleCap, NameTaken,
    BadMemberList, // C-173
    ControlDegraded, NoSkewBudget, // C-185
}
pub enum ConnectionState {
    Requested, Refused, Pending, Admitted, Locking, Aligning,
    Live, Interrupted, Demoted, Dead, Ended,
}
pub enum ProducerState { Sending, Stopped }
pub enum ControlEvent {
    PeerJoined, PeerLeft, IdCollision, VersionMismatchSeen, ForeignDomainSeen,
    ElectionOpened, HolderChanged, TimelineEpochChanged, FollowerRefused,
    ConnectionRefused(RefuseReason), ConnectionState(ConnectionState),
    ControlDegraded,
}
```

- `Demoted` is the health state of C-109: an input persistently late with its offset already at the group's audition offset A, treated as gapped, counted separately, cleared only by re-admission.
- `ControlDegraded` is emitted by the housekeeping supervisor when it restarts housekeeping (L-7), not by a table row (epic decision).
- `RequiredEExceeded` also covers a sink with no numeric E, which declares E unknown, when the group sets a required E (C-103); only a commissioned sink has a numeric E, uncertified and self-monitored sinks declare it unknown (C-140).
- `BadMemberList` is the 16th refusal reason (C-173): an unknown or duplicate bundle member, or more than 32 members (C-08). `ForeignDomain` is for a different non-zero domain; a hello with domain 0 is ignored and counted, not refused (C-173). `PathMtuTooSmall` applies only when the probe at the 1 280-byte fallback also fails (C-173).
- C-185 closes the enumeration at 18 reasons: `ControlDegraded` (the target's housekeeping has stopped restarting, C-139; answered by the housekeeping supervisor, which keeps a minimal refusal path) and `NoSkewBudget` (a fan-in with no downstream group and no configured budget, C-142) join the 16 of C-173. `RefuseReason::ControlDegraded` (a refusal answered by the supervisor) is distinct from `ControlEvent::ControlDegraded` (the supervisor's restart event); both are emitted by the supervisor. `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, where they stop a module joining, not at admission (C-185).
- A connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded (C-173); how it is carried (new variant or derived) is decided in 005-26.
- 005-AC-6 (system criterion AC-12): the refusal-reason enumeration is closed; a request exceeding 70 % of a port (C-23) is refused with a named reason.
- Open sub-question owned by this ticket (epic 005): which event each non-admission refusal emits (where three of them arise is settled by C-185, above). `VersionMismatch` and `ForeignDomain` pair naturally with `VersionMismatchSeen` and `ForeignDomainSeen`, `FollowerCap` with `FollowerRefused`; `ModuleCap` and `NameTaken` have no paired event in the epic's shape. Falsifier: a refusal scenario cannot name the single event it expects.

## Work
1. In `hadronyche-core` (module path not fixed by the epic; propose `control::types` and record it in `control.md`), add `RefuseReason` (18 variants, C-185), `ConnectionState` (11), `ProducerState` (2) and `ControlEvent` (12), `#![no_std]`-compatible, `Copy`, `Eq`, `Debug`, no heap.
2. Do not mark the enums `#[non_exhaustive]` (they are closed); document that adding a variant is a spec change of `control.md`.
3. Add a stable numeric code per variant (for vectors and logs) and a `name()` returning the variant name as `&'static str`.
4. Decide the event emitted by each of the 5 non-admission refusals (`VersionMismatch`, `ForeignDomain`, `ModuleCap`, `NameTaken`, `FollowerCap`): reuse an existing variant, or add one with a documented spec change. Record the decision in "Decisions on open sub-questions" with a `[decision]` tag and restate the falsifier.
5. Write the "Enumerations" section of `control.md`: each variant, one line of meaning, and which machine (or the supervisor) emits it; `RefuseReason::ControlDegraded` names the housekeeping supervisor's minimal refusal path (C-185).
6. Unit tests: code/name round-trip for every variant; exhaustive `match` compile test.

## Acceptance criteria
1. `RefuseReason` has exactly 18 variants (C-185), `ConnectionState` exactly 11, `ProducerState` exactly 2; `ControlEvent` has 12 variants, or a documented addition from Work step 4 or from 005-26.
2. Each of the 18 refusal reasons maps to exactly one emitted event in the Enumerations section.
3. The crate builds with `no_std` and without `alloc`.
4. Every variant has a unique numeric code and a name; the round-trip test covers 100 % of variants.
5. `control.md` lists every variant with its emitter; `ControlDegraded` names the housekeeping supervisor.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The machines that raise these values (005-07, 005-10, 005-14, 005-16).
- Wire encoding of reasons (epic 004).
- Implementing the machines (epics 011, 012).
