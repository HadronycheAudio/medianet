# 010-11 — Intra-slice pacing schedule
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-10
## Context
Pacing `[decision]`:
- The packets of one slice period (for a 96-channel source: 16 packets) are spread over at most 25 % of the period (C-14: 125 µs of the 500 µs period).
- Each source adds a phase offset derived from its module id (010-10).
- Pacing runs in userspace on the deadline clock so per-packet send timestamps stay true. Per-output override: 0 disables spreading; the override comes from operator policy in the commissioning configuration (C-165) and reaches `pace` as `spread_permille`.
- C-155: phase offset plus spread stay inside the slice period; the phase offset is in [0, 75 %) of the period (010-10), so with a spread of at most 25 % every instant falls before the next slice boundary.
- This epic computes the schedule (a list of send instants); the backend (epic 015) executes it.
- Settled in the epic:
  - The schedule covers all packets of one source's slice period; at defaults one packet per stream, so up to 256 (C-09). The packet count and `len` are therefore `u16`.
  - The capacity P is a const parameter chosen by the caller; a request for more than P packets is refused with an error.
  - A spread above 250 ‰ violates C-14 ("at most 25 %") and is refused with an error.
- `[code]` POC: the send timestamp was taken once before a per-destination `sendto` loop (`sender.rs:290`, `module.rs:979-986`); retired.

Signature (names may be refined):
```rust
pub struct PaceSchedule<const P: usize> { pub at_ns: [i64; P], pub len: u16 }
pub fn pace<const P: usize>(slice_start_ns: i64, slice_ns: i64, packets: u16, spread_permille: u16, module_id: u64) -> Result<PaceSchedule<P>, PaceError>;
```

## Work
- Implement `PaceSchedule`, `PaceError` and `pace` in `hadronyche-core`, `no_std`, no allocation.
- Instants: `at_ns[i] = slice_start_ns + phase + i × spread_ns / packets`, where `spread_ns = slice_ns × spread_permille / 1000` and `phase = phase_offset_ns(module_id, ...)` (010-10). Instants are non-decreasing.
- `spread_permille = 0`: all instants equal `slice_start_ns + phase`.
- `spread_permille > 250` → `Err(PaceError::SpreadTooLarge)`; `packets > P` → `Err(PaceError::Capacity)`.
- Provide `DEFAULT_SPREAD_PERMILLE = 250` (from 010-01) as the default; the per-output override is a parameter.

## Acceptance criteria
1. For 16 packets, slice_ns = 500 000, spread 250 ‰: every instant lies in [start + phase, start + phase + 125 000] (epic AC-6).
2. The same call for module ids 1 and 2 yields schedules whose first instants differ (epic AC-6).
3. Spread 0 yields `len` instants all equal to a single value (epic AC-6).
4. `len == packets` and instants are non-decreasing for packets = 1, 4, 16, 32, 256 (with P ≥ 256).
5. Same inputs give the same schedule (determinism).
6. `packets > P` and `spread_permille = 251` return the matching error without panic (epic AC-6).
7. For every module id in the 010-10 evaluation set, spread 250 ‰ and packets = 1, 16, 256, every instant lies in [slice_start_ns, slice_start_ns + slice_ns) (C-155).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Executing the schedule on the deadline clock and sending (epic 015).
- Exposing pacing mode in the regime stamp (010-12).
- Field validation of queue drops (task 019).
