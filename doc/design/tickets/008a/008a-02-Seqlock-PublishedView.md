# 008a-02 — Seqlock PublishedView
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-01
## Context
The servo publishes its view (offset at a reference time, drift, state, error bound, timeline epoch) through a seqlock read by the data threads `[decision]`. The POC already used a single-writer seqlock (`SharedSync`) `[code]`; this ticket writes the new one in `hadronyche-core`.

Seqlock invariant `[decision]`: the writer must never be preempted mid-write by a higher-priority reader on the same core, or the reader spins forever. In strict mode the data and clock threads run with real-time scheduling (C-127), which is where a priority inversion of this kind can occur; strict mode is hosted-only, and a device has real-time scheduling present by construction (C-147), so the invariant applies there too. The writer is the clock thread and the readers are data threads; with the default priorities (clock above data) the invariant never depends on core pinning. If a thread layout's seqlock safety does depend on a CPU pin and that pin fails (C-177) or is unverified (C-201), the module refuses to start in every mode, strict or not. The doc comment says so.

Must compile for the `no_std`, no-allocator target chosen in 008a-01 (epic criterion A9).

## Work
In `hadronyche-core`:

```rust
pub struct PublishedView { /* seqlock */ }
impl PublishedView {
    pub fn write(&self, v: &TimeView);   // single writer
    pub fn read(&self) -> TimeView;      // wait-free in practice
}
```

1. Implement with a sequence counter (odd = write in progress) and atomics only; fixed size; no allocation; `no_std`.
2. Provide a `const fn new(initial: TimeView)` or equivalent so it can live in a static.
3. Write the priority invariant as a doc comment on `PublishedView` itself: single writer; the writer must not be preempted mid-write by a higher-priority reader on the same core; consequence if violated (reader spins forever); the default priorities (clock writer above data readers) satisfy it without pinning; a layout that relies on a CPU pin must refuse to start in every mode when the pin fails or is unverified (C-177, C-201).
4. Tests: single-thread write/read; multi-thread stress (one writer, N readers) checking no torn `TimeView` is ever returned.

## Acceptance criteria
1. Stress test: 1 writer, 4 readers, at least 10^6 writes; every read returns a `TimeView` equal to one that was written (no torn reads). Runs unprivileged.
2. `read` returns after a write completes without any lock or syscall (verified by code review: atomics only).
3. The doc comment on `PublishedView` states the priority invariant in full, including the pin rule of C-177 and C-201.
4. Module builds for the 008a-01 no_std target with no allocator.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The seqlock priority invariant is documented at the type
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Enforcing thread priorities or core pinning, and the refusal to start on a failed or unverified pin (C-177, C-201) (later wiring epics, e.g. epic 015).
- What the servo writes (008a-11).
