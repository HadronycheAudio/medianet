# 017b-04 — Partition, hello suppression and thread-kill injection
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-01, epic 015, epic 016

## Context
- Remaining `Injector` methods in `hadronyche-std/src/inject.rs`:
```rust
fn partition(&self, groups: &[&[ModuleHandle]]);
fn suppress_hello(&self, module: ModuleHandle, on: bool);
fn kill_thread(&self, module: ModuleHandle, role: ThreadRole);
```
- Used by L0 scenarios `partition` (AC-36: after heal, one holder within 1.4 s = election window 400 ms (C-33) + hello 1 s (C-30)), `thread-death` (AC-017.16: a stalled data plane is visible to peers within 2 hello periods, C-42) and `housekeeping` (017b-23, which also uses the panic hook of 017b-08, C-145). Id collisions and foreign epochs are not produced with these methods: they use the test-only hooks of 017b-06 (C-120).
- `ThreadRole` comes from the real-time setup of epic 016; `ModuleHandle` from the hosted runtime (epic 015). Their exact definitions are owned by those epics.

## Work
1. `partition(groups)`: packets between modules in different groups are dropped; within a group they flow. `partition(&[all_modules])` (single group) heals. Modules not listed form their own isolated group (document this).
2. `suppress_hello(module, on)`: while `on`, the module's hello messages (period 1 s, C-30) are not delivered; other traffic flows.
3. `kill_thread(module, role)`: terminates (or permanently parks) the module's thread of that role in a way the runtime observes as thread death, without killing the process.
4. Counters for packets dropped by partition and hellos suppressed.

## Acceptance criteria
1. With groups `{A,B}` and `{C}`, A↔B packets are delivered and A↔C, B↔C packets are dropped (100 % over 10 000 packets); after a single-group call all pairs deliver again.
2. With `suppress_hello(A, true)`, zero hellos from A reach any peer over 5 s while A's media packets still arrive; with `false`, hellos resume within 1 s (C-30).
3. `kill_thread(A, role)` for each `ThreadRole` variant leaves the process alive and the targeted thread not running; the call is idempotent.
4. All of the above work without root or capabilities.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- What the runtime must do in response (election, epoch change, thread-death handling): asserted by scenarios 017b-20, 017b-22 and 017b-23.
- Clock and link injection (017b-02, 017b-03); test-only hooks (017b-06, 017b-07, 017b-08).
