# 021-14 — Declared D from device properties
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-10, epic 018, epic 024a
## Context
The edge contract [decision]: `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. D is the sink's total plumbing delay from hand-off to emission (driver buffer + resampler + converter); for capture, the mirror, from acoustic instant to the sample being available (C-170). Capture stamps a sample with `T⁻¹(capture_time − D_capture)`.

- macOS mechanism [memory]: D = device latency + stream latency + safety offset + buffer size properties, plus the resampler node's delay when the device is not at 96 kHz, plus the rate actuator's delay, plus a commissioning measurement. Check: L2.
- Typical D a few ms (C-81). Target restart spread of D at most 50 µs over 20 restarts (C-74).
- The wire carries 96 kHz only (C-01). A 48 kHz device converts 2:1 through the **kit's resampler node** (C-80), built by epic 024a; this epic builds no resampler (C-197). The resampler may appear only inside an edge module, and its conversion delay is part of D (C-134). That delay is counted only in D and never in L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207). The delay is always a whole number of domain-rate samples (C-105), read as the conversion delay the manifest reports; the node exists only for the 2:1 and 1:2 ratios (C-105). D adds conversion delay × 10⁹ / 96 000 ns. A device already at 96 kHz has no node term.
- Two kinds of resampler [decision, C-189]: besides the fixed node, every edge runs the kit's steerable rate actuator, whose ratio varies by parts per million to track the device crystal; it runs whatever the device rate, and its delay is part of D (and never of L, C-207). Its delay is read from the kit's rate-actuator trait (epic 024a, C-210).
- The regime stamp records D and its method (epic criterion 021-AC7, AC-31). The full set of allowed method values is defined with the regime stamp message by epic 013 (C-144); this ticket uses `DeviceProperties` and `Commissioned`.
## Work
- `fn declared_d(device, direction) -> DeclaredD { ns, method }` summing, per direction (output for sink, input for capture): device latency, stream latency, safety offset, buffer frame size (converted at the device's nominal rate), plus the resampler node's conversion delay as its manifest reports it on the device-facing port (C-207) when the device rate differs from 96 kHz, plus the rate actuator's delay as the kit's rate-actuator trait reports it, at every device rate (C-189, C-210). Neither delay is ever added to the module's L.
- Sum all terms in one integer unit and round once to ns at the end.
- `method` = `DeviceProperties` here; a later commissioning override (021-22) sets `Commissioned`.
- Re-read properties at every stream start.
## Acceptance criteria
1. Unit test with fake property values (e.g. latency 24 frames, stream 32, safety 16, buffer 512 at 48 kHz, resampler conversion delay = 96 domain samples) returns the exact sum in ns.
2. Unit test: the conversion delay is taken from the manifest's device-facing port as a whole number of domain-rate samples (C-105, C-207) and adds delay × 10⁹ / 96 000 ns, rounded once with the other terms; the module's L is unchanged by it (C-207); a 96 kHz device adds no node term.
3. Manual hardware test: built-in output D is reported and lies in the "few ms" range (C-81) or the deviation is recorded.
4. Unit test: the rate actuator's delay, taken from a fake rate-actuator trait, is added to D for both a 48 kHz and a 96 kHz device and leaves L unchanged (C-189, C-207).
5. Over 20 restarts of the stream (manual), the declared D value is identical each time (spread of the *declared* value is 0; measured spread is 021-23).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
## Out of scope
The resampler node itself (the kit, epic 024a, C-80, C-197). Commissioning measurement of D and comparison with properties (021-22). L2 restart-spread measurement (021-23).
