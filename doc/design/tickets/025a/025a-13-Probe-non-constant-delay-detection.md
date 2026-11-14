# 025a-13 — Probe: non-constant delay detection
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-12
## Context
The compiler flags outputs reached through **recursive filters as having non-constant (frequency-dependent) delay**, a property beside their structural L, which is a whole number of samples (C-104, C-105). Acceptance criterion AC-33 (latency) of epic 025a requires that programs flagged non-constant-delay **are reported as such by the probe (delay varies with marker spectrum)**.

The epic's probe shape (025a-12) already carries this: `ProbeResult { output, delay: Delay, confidence }` with `Delay::Constant(Samples)` or `Delay::NonConstant { min, max }`, delays in whole domain-rate samples (C-105). This ticket implements the detection that fills `NonConstant`.
## Work
1. Add a probe mode that runs several markers with different spectra (e.g. band-limited pseudo-random codes in low, mid and high bands, plus an impulse) and compares measured delays per output.
2. Implement and document the rule: delays differing between markers by more than a documented tolerance ⇒ `NonConstant { min, max }` over the markers; otherwise `Constant`.
3. Tests with a hand-written one-pole low-pass and a biquad (non-constant) and pure delays (constant).
## Acceptance criteria
1. One-pole and biquad modules are reported `NonConstant`.
2. Pure delays of 0, 48 and 1 000 samples and a linear-phase FIR of 31 taps (L = 15) are reported `Constant` with the exact value.
3. The decision rule and tolerance are in the code docs.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Running over the corpus (025a-15); CLI (025a-14).
