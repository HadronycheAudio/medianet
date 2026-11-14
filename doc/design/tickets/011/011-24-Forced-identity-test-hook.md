# 011-24 — Forced identity test hook
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-01

## Context
Epic AC 2 (AC-22) needs a forced id collision: two modules running with the same module id and different nonces. With 64-bit random ids this never happens naturally. C-120 (decided) lists, among the test-only injection hooks beyond the public injection API, "force a module id and nonce", and requires the hook to be stamped like any injection.

In core, identity comes only from an `IdSource` (011-01), so the hook is a forced `IdSource` plus a flag the platform can stamp.

## Work
1. Add `ForcedIdSource`, an `IdSource` that returns a given `(ModuleId, Nonce)` on its first draw and then falls back to a wrapped source (so a later `redraw_id` produces a fresh id), behind a test-only cargo feature (for example `test-hooks`) that is off by default.
2. Expose on `Discovery` a read-only flag, `identity_forced()`, true when the current identity came from the forced draw, for the platform to record as an injection in the regime stamp.
3. Use it in the collision tests of 011-10 and in the collision vectors of 011-21.

## Acceptance criteria
1. Two `Discovery` instances built with `ForcedIdSource` for the same id and different nonces advertise that id and their own nonces in their hellos.
2. After `redraw_id`, the instance's id differs from the forced id and `identity_forced()` is false.
3. A build of `hadronyche-core` without the test feature contains no `ForcedIdSource` (checked in the `no_std` CI job).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (core only exposes the flag; the injection record in the stamp belongs to the stamp's owner); "Design documents updated" (follows C-120); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- The other C-120 hooks (forcing a stream onto a timeline epoch, sending a raw packet on a socket): platform and data-plane epics.
- Writing the injection record into the regime stamp: the stamp message's owners, epic 013 (definition) and epic 015 (sending), C-144.
- The panic-injection hook for data, clock and housekeeping threads (C-145): platform epics, since core has no threads.
