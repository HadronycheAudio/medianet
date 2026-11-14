# 010-12 — Regime stamp exposure
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-09, 010-11
## Context
- The multi-packet regime is "an explicit, stamped regime" (C-05): the regime is recorded in the regime stamp.
- Epic definition of done requires: packets per slice and pacing mode are exposed for the regime stamp.
- C-18: the regime stamp travels in a dedicated control message split into sections that each fit one fallback-MTU packet; it is multicast to the control group on any regime change and refreshed every 10 s; the hello carries only a stamp version number. That message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144); this ticket only exposes the field values, and any change of them is a regime change epic 015 must publish.
- Pacing mode at this level is: spread in permille (default 250, C-14; 0 disables spreading; the override is operator policy, C-165) and whether a per-source phase offset is applied.
- The C-47 send counters are exposed separately (010-21).

## Work
- In `hadronyche-core`, add a small `no_std` struct (suggested `SendRegime { packets_per_slice: u8, spread_permille: u16, phase_offset: bool }`, names may be refined), deriving equality, and a getter on the `Packetiser` and on the bundle packetiser (one entry per member) returning it.
- Document each field for the epic 013 regime-stamp specification (in `doc/design/spec/`, C-130): meaning, unit, source, and that a change is a regime change under C-18.

## Acceptance criteria
1. For a 6-channel stream at MTU 1 500 the regime reports 1 packet per slice; for a 48-channel single stream at MTU 1 500 it reports the value of `packets_per_slice(48, 48, 1500, h)`.
2. With spread override 0 the regime reports `spread_permille == 0`; with defaults it reports 250.
3. Changing the path MTU so that `packets_per_slice` changes makes the new `SendRegime` compare unequal to the old one.
4. Every field of the struct is described in the regime-stamp specification.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Packets per slice and pacing mode are exposed for the regime stamp
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The regime stamp control message, its sections, version number and refresh (C-18; defined by epic 013, sent by epic 015, C-144).
- Send counters (010-21).
