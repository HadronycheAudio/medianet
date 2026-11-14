# 026b-04 — Memory fit measurement
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03

## Context

Epic criterion 026-AC-2 (Memory fit): static memory for the **63-entry peer map** (C-217, C-34), **one 6-channel stream with A = 10 ms**, the **servo**, the **resampler node** and the **rate actuator** (C-189) fits the reference device with **at least 25 % headroom**; the figure is recorded.

Ring arithmetic (settled): alignment rings are sized to the presentation group's A (C-96) and hold wire slices at the 96 kHz domain rate, the only rate on the wire (C-01); conversion to 48 kHz happens after the ring, in the edge (C-80). One stream is 6 channels (C-04) × 4 bytes (C-03) × 96 000 × 0.010 s = **23 040 bytes** (192 bytes per channel per slice × 6 channels × 20 slices). This matches C-27's 370 KB for a 16-member bundle at 10 ms.

The firmware is no_std, no alloc, so all of this is static (`.bss`/`.data`) plus stack; module state and stack sizes are constants (epic 024).

## Work

1. Build the firmware with the 026-AC-2 configuration (63-entry peer map, C-217, one 6-channel stream, A = 10 ms, servo, resampler node, rate actuator).
2. Measure static RAM from the linker map (`.data` + `.bss`) and peak stack (stack painting or static analysis, e.g. `cargo-call-stack`).
3. Compute headroom = 1 − used / total RAM; record per-component figures (map, ring, servo, resampler node, rate actuator, network buffers, stamp section buffers, other).
4. Record the figures `[measured]` in `doc/design/reports/` (C-130) and in the device docs; update the stamp's declared memory limit if it changes.
5. Keep the measurement script in the `hadronyche-tools` crate (C-133).

## Acceptance criteria

1. A table records, in bytes: peer map, skew ring (expected 23 040 plus overhead), servo, resampler node, rate actuator, network stack, stamp section buffers, stack peak, total, available RAM.
2. Headroom ≥ 25 %. If not, the ticket fails and records the falsifier: reduce components or pick another board (026b-01).
3. The measurement is reproducible by a documented command in `hadronyche-tools`.

## Definition of done

- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope

- CPU budget (026b-05).
