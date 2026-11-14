# 018-21 — Sleep as departure
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-06, 018-14, 018-16, 018-25, epic 012

## Context
- **C-124:** when an edge host sleeps, the sleep is a **departure**: both timebase loops are reset and the edge returns through normal admission (Locking, Aligning). The rule is shared by every edge, so the reset lives in `hadronyche-core` `edge/`; detection is per platform.
- Loop 1 is the core servo (epic 008); loop 2 is `DeviceLine` (018-02); the rate-loop filter is in `EdgeTimebase` (018-03). The alignment claim lapses on departure (018-06).
- On Linux, `CLOCK_MONOTONIC` does not advance during system suspend while `CLOCK_BOOTTIME` does, and an ALSA PCM open across a system suspend is left in the `SUSPENDED` state `[kernel-doc]`.
- `edge/` is `no_std`, no allocation.

## Work
1. Add `EdgeTimebase::reset()` and a `departure()` entry in `edge/` that resets `DeviceLine`, the rate-loop filter, `EmissionStats` windows and the alignment claim (`Lapsed { reason: Departure }`).
2. In `alsa/`, detect sleep for `AlsaSink` and `AlsaCapture`: a jump of `CLOCK_BOOTTIME − CLOCK_MONOTONIC` larger than one period, or the PCM in `SUSPENDED` state.
3. On detection: call `departure()`, ask the servo (epic 008) to reset loop 1, re-open or re-prepare the PCM, and re-enter admission (epic 012) so the edge passes through Locking and Aligning again.
4. Stamp the departure and its cause.
5. On return, strict mode is re-checked (018-25): a capability lost across the sleep refuses the restart in strict mode (C-128).

## Acceptance criteria
1. **Epic criterion 018-A10:** with a fake clock pair showing a 60 s `BOOTTIME − MONOTONIC` jump, the sink resets both loops, publishes E blank, and re-enters Locking then Aligning before emitting again.
2. A fake PCM reporting `SUSPENDED` triggers the same path for capture; the capture stream's session number is bumped on its restart (C-19).
3. No departure is detected in a 10 min fake run without suspend.
4. `departure()` builds `no_std` with no allocation.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Sleep detection on macOS and Windows (epics 021, 022).
- The servo's own reset (epic 008) and the admission engine (epic 012).
