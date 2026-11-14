# 010-10 — Phase offset from module id
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-01
## Context
- C-14: intra-slice egress spread is at most 25 % of the slice period (125 µs of the 500 µs period at C-02), **plus a per-source phase offset derived from the module id** `[decision]`.
- Why: without the offset, every source sharing the timebase fires on the same slice boundary and bursts arrive coherently at a switch port.
- C-34: at most 64 active modules per medianet. C-11: module ids are 64 bits.
- Settled in the epic: the offset is a deterministic function of the module id. Offsets take finitely many values, so distinctness for every pair of ids is not required; the goal is an even spread over 64 modules. Epic AC-6 checks distinct offsets for a fixed list of test ids only.
- Open items:
  - Derivation and acceptable sharing for 64 ids: this ticket proposes; L1 (task 019) sets the threshold. Falsifier: two of 64 ids share a phase slot and L1 shows queue drops attributable to it.
- Settled by C-155: phase offset plus spread stay inside the slice period; the phase offset is in [0, 75 %) of the period (0 to 374 999 ns of the 500 µs period at C-02), independent of the spread actually configured.
- Evidence tags available: `[code]`, `[decision]`, `[measured]`, `[kernel-doc]`, `[standard]` (C-132), `[memory]` (listed in the epic).
- C-130: the design note goes in the design set (`doc/design/`); the evaluation figures go in a report under `doc/design/reports/`, cited from the note.

## Work
- Write a short design note (in the pacing section of the architecture specification, `doc/design/01-architecture.md`, C-171) giving the derivation: a deterministic, `no_std` mixing of the 64-bit module id into a phase offset in nanoseconds in [0, 0.75 × slice_ns) (C-155).
- Implement `pub fn phase_offset_ns(module_id: u64, slice_ns: i64) -> i64` (name may be refined) in `hadronyche-core`.
- Add an evaluation test: for 64 ids (sequential 0..63, and 64 pseudo-random ids from a fixed seed) report the number of distinct phase slots and the largest number of ids on one slot. Record the figures with tag `[measured]` in a report under `doc/design/reports/` (C-130), cite them from the design note, and hand them to task 019.

## Acceptance criteria
1. `phase_offset_ns` is deterministic: same id and period give the same offset across runs and targets.
2. For a fixed list of id pairs (including 0/1, 1/2, u64::MAX/u64::MAX−1) the offsets differ (epic AC-6, part).
3. Every offset lies in [0, 375 000) ns for slice_ns = 500 000 (C-155), checked over the 128 evaluation ids and the edge ids 0 and u64::MAX.
4. The 64-id evaluation runs in CI and its output figures are written, tagged `[measured]`, in a report under `doc/design/reports/` that the design note cites.
5. Function is `no_std` and allocation-free.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The schedule itself (010-11).
- Whether the spread plus phase avoids queue drops on real switches: owned by L1 (task 019).
