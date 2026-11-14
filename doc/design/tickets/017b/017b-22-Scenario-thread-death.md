# 017b-22 — Scenario `thread-death`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-04, 017b-08, 017a-09, 017b-09, epic 015, epic 016

## Context
- `thread-death` is in the epic's scenario list (epic AC-017.16). Before the split of epic 017 it shared a ticket with `housekeeping`, which is now 017b-23.
- **Thread-death (settled):** a stalled data plane is visible to peers within 2 hello periods (C-42); the hello's 64-bit message counter is the liveness counter (C-115). Consumers classify the dead module's streams interrupted after 8 consecutive misses (C-38) and dead 2 s after the last slice with content or a marked gap (C-39, C-111). A connection alarm is raised on interrupted and dead (C-173).
- L0 runs unprivileged and non-strict: real-time scheduling and memory locking are "not requested" (C-127, C-129, C-147), so thread roles run without real-time priority.
- Injection: `Injector::kill_thread(module, role: ThreadRole)` (017b-04); a panic in the data or clock thread can also be injected with the hook of 017b-08 (C-145). Thread roles come from epic 016.
- Emitting the gap slice when a module misses its own deadline and raising the processing-overrun event are the hosted backend's duties (epic 015, C-144); this scenario observes them only through peers' classification.
- L0 rules `[decision]`: assert content, loss accounting, lock, classification and ground-truth error; never latency on loopback.

## Work
1. Scenario `thread-death`: for each `ThreadRole`, `kill_thread` on one module; assert the runtime reports the death as a named event; for the data-plane role, peers see the stall within 2 hello periods (C-42) and consumers classify its streams interrupted then dead (C-38, C-39, C-111), with an alarm on each (C-173).
2. Document the asserted behaviour in rustdoc, citing the C-ids and epics 015/016.

## Acceptance criteria
1. `hadronyche-l0 thread-death --duration 30` exits 0.
2. One named event per killed role.
3. For the data-plane role, peers flag the stall within 2 hello periods; affected streams go interrupted after 8 misses and dead 2 s (± one slice) after the last slice; one connection alarm is raised for each transition (C-173).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Thread model and real-time setup (epic 016).
- The `housekeeping` scenario and the restart policy (017b-23).
