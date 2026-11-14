# 029-08 — Honest blanks, failed families, injection and commissioning labels
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-06, 029-07

## Context
- The proof of concept showed `[measured]` two whole check families reporting "0 of 0" while the run looked green. Rules since then `[decision]`:
  - A check family with zero samples is a **failed run**; the monitor shows it as **failed**, not blank.
  - One-way latency figures are **blank until the clock is locked**, and carry the clock's **error bound** afterwards.
  - Injection: any active injection is always stamped (Injection group of the regime stamp), including the test-only hooks (forced module id and nonce, forced timeline epoch, raw packet send, C-120; injected thread panic, C-145); an injected run must be visibly labelled as injected.
- Commissioning (C-107): the commissioning tool may drive test traffic only inside a declared commissioning window that every module stamps. Traffic in that window is not normal operation, so the monitor labels it like an injection. The tool announces the window with an expiry time; modules stamp it while open; it closes at expiry or on explicit close (C-153). What an open window disqualifies (C-178): ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in.
- Epic criterion 029-AC-2 ("Honest blanks"), which contributes to the global criteria AC-9 and AC-31.
- Open question in the epic, owned by epic 013: where check-family sample counts are published. Working assumption: in the stamp.

## Work
1. A display-value type `Shown<T>` used by every view: `Value(T)`, `ValueWithBound(T, bound)`, `BlankUnlocked`, `Failed(reason)`, `Unknown` (the runtime declares the value unknown, for example E of a sink that is not commissioned, C-103, C-140), `NotPublished`. Views must not format raw numbers directly.
2. `fn one_way(&self, conn) -> Shown<Duration>`: `BlankUnlocked` unless both endpoints' clocks are Locked (029-07; a lapsed holdover counts as Unlocked, C-162); afterwards `ValueWithBound` carrying the larger of the two endpoints' bounds.
3. `fn family_status(&self, family) -> Shown<Ratio>`: a family with 0 samples in the current series returns `Failed("0 samples")`.
4. `fn injection(&self, module) -> Option<InjectionLabel>` from the Injection group; a model-wide `run_is_injected()` true if any module stamps an active injection.
5. `fn commissioning(&self, module) -> bool` from the Commissioning group; a model-wide `run_in_commissioning()` true if any module stamps an open commissioning window. Production figures (latency, loss, alignment) from a series that overlaps a window carry the label `unqualified` (C-153, C-178); commissioning results (D, restart spread, certification state) are never labelled `unqualified` and are shown with the window they were taken in when the stamp carries it (C-178).

## Acceptance criteria
1. Before lock, every one-way latency cell is `BlankUnlocked`; after both endpoints lock with bounds 8 µs and 14 µs, the cell is `ValueWithBound(_, 14 µs)`.
2. A family with 0 samples yields `Failed`; with 1 or more samples it yields a value.
3. A stamp with an active injection (including a C-120 hook) makes `run_is_injected()` true and labels that module; removing it starts a new series (029-06) whose label is clear.
4. A stamp with an open commissioning window makes `run_in_commissioning()` true, labels that module, and marks the latency, loss and alignment figures of the overlapping series `unqualified`; a sink's D and certification state taken in that window are not marked `unqualified` (C-178).
5. A grep/lint check shows no view formats a measurement without going through `Shown`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Rendering colours and placement (each panel ticket, 029-11 to 029-17); headless representation (029-18).
- Computing sample counts in the monitor: if the epic 013 stamp exposes no family counts, the family is shown `NotPublished` and the gap is reported to epic 013.
