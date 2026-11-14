# 017b-08 — Test-only hook: thread panic injection
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-05, epic 015, epic 016

## Context
- Rule C-145: an additional test-only hook injects a panic into a data, clock or housekeeping thread; it extends the hooks of C-120 and, like them, is stamped like any injection (017b-05).
- Why a panic and not `kill_thread` (017b-04): `kill_thread` stops a thread for good, while the housekeeping restart policy reacts to panics. C-139: housekeeping restarts with exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it. The restart machinery and the AC-20 (c) criterion are the hosted backend's (epic 015, C-144); this hook only triggers it.
- Users: scenario `housekeeping` (017b-23); scenario `thread-death` (017b-22) may use it for the data and clock roles.
- The hook is not public API; it sits behind the same test-only feature as 017b-06 and 017b-07 (shape; names may be refined):
```rust
fn inject_panic(&self, module: ModuleHandle, role: ThreadRole);
```
- `ThreadRole` comes from epic 016 and `ModuleHandle` from epic 015.

## Work
1. Implement `inject_panic(module, role)` for the data, clock and housekeeping roles: the targeted thread panics at its next safe point, through the same unwinding path as a real panic, so the runtime's panic handling (epic 015) sees it.
2. Record each call in the injection log of 017b-05 (method, module, role, true time of call).
3. Build the hook only under the test-only feature.

## Acceptance criteria
1. `inject_panic(m, Housekeeping)` makes m's housekeeping thread panic once; the runtime's panic handler records one panic for that role; the process stays alive.
2. The same holds for the data and clock roles.
3. Each call appears in the stamp's injection record with module and role.
4. A build without the feature does not expose the hook.
5. Works without root or capabilities.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The restart policy and its criterion (epic 015, C-139, C-144); asserting it in L0 (017b-23).
- Thread kill (017b-04); the other hooks (017b-06, 017b-07).
