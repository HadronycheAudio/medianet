# 005-03 — Totality checker
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-02

## Context
- Epic 005 definition of done requires: "Transition tables are total (checked mechanically from the machine-readable form)". Acceptance criterion 005-AC-1: every (state, input) pair of every machine has an explicit row.
- The machine-readable form is `doc/design/spec/control.toml` (schema from 005-02): machines with `state`, `input` and `transition` entries; each transition has `state`, `input`, `guard`, `next`, `outputs`, `events`, `counters`, `invariants`, `ignore`.
- Valid invariant ids: L-1 to L-8. Valid events are the `ControlEvent` variants; valid refusal reasons are the 18 `RefuseReason` variants (C-173, C-185; both defined in 005-04). Valid counters are the per-connection counters of C-159 (which completes C-47): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead; and, for machine `election` only, the per-follower count of dropped sync requests (C-204).
- The TOML loader is the only parser this epic touches, so the definition-of-done line "Every parser touched is fuzzed" applies to it. Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- Location settled (C-133): harness and tool binaries live in the `hadronyche-tools` crate of the design set's repository, so the checker is a Rust binary there.
- Presentation (C-206): the checker's output line format and file names are decided by this epic and recorded in the tool's own documentation; only what another epic or an operator's script depends on goes into the specification.

## Work
1. Write the checker as a binary in the `hadronyche-tools` crate (C-133); record its invocation, output format and file names in the tool's documentation in `hadronyche-tools`, not in `control.md` (C-206); the Conventions section of `control.md` states only what the checker verifies. It loads `control.toml` and fails with a readable message when:
   - a (state, input) pair of a machine has no row (totality);
   - a pair has several rows whose guards are not declared mutually exclusive (guards on the same pair must be listed as an ordered or exhaustive set ending with an `otherwise` guard);
   - a `next` state, `state` or `input` is not declared in the machine;
   - an `invariants` entry is outside L-1 to L-8;
   - an `events` entry is not a declared `ControlEvent` variant name;
   - a `counters` entry is not a C-159 counter (or, in machine `election`, the C-204 dropped-request counter);
   - a state is unreachable from the initial state.
2. Run it in CI as an unprivileged job on every change to `control.toml`.
3. Add negative fixtures (missing row, unknown state, unknown L-id, unknown event, unknown counter) that must fail.
4. Fuzz the checker's input path (TOML loading and schema mapping) with a fuzz target at the C-95 budget: 60 s on every change, 1 hour nightly, corpus kept in the repository.

## Acceptance criteria
1. The checker exits 0 on the `example` machine of 005-02 (4 rows) and on the current `control.toml`.
2. Each of the 5 negative fixtures makes the checker exit non-zero with a message naming the machine, state and input concerned.
3. The checker lives in `hadronyche-tools` (C-133); the CI job runs without elevated privileges and is required for merge.
4. The fuzz target runs 60 s in the per-change CI job and 1 hour in the nightly job without a crash, and its corpus is committed (C-95).
5. The checker's output format and file names are documented in `hadronyche-tools`; `control.md` carries no tool output format or file name (C-206).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Filling the real tables (005-07, 005-10, 005-14, 005-16): they are checked by this tool when they land.
- The model check of the election (005-11).
- Vector generation (epic 007).
