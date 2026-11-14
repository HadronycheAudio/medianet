# 024b-19 — Six-channel gain module cost-bound benchmark
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-12, 024b-13

## Context
Epic acceptance criterion 024b/AC-7: a generated 6-channel gain module's counted operations per 48-frame slice are identical on every CI target and do not exceed `cost_ops_per_slice`; its measured processing time on the CI machine does not exceed `cost_ops_per_slice` times that machine's calibration factor.

Settled rule (C-106): the cost bound counts abstract operations per slice, identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts with it. Slice 48 frames = 0.5 ms at 96 kHz (C-01, C-02); 6 channels (C-04); 4-byte samples (C-03).

The procedure that measures a calibration factor is settled (C-176): the DSL conformance suite (epic 025) runs a reference workload once per listed target and records time per operation; the value is carried in the regime stamp. Epic 025 depends on this epic, so this ticket measures the CI machine's factor with a provisional run of the same shape (one reference workload, time per counted operation), marked provisional; epic 025's C-176 value replaces it.

## Work
- DSL gain program on one 6-channel audio input (from the epic's corpus); build with `hadronyche-dsl build`.
- Count its operations per slice with the counting build of 024b-13 on all three CI targets.
- Measure the CI machine's calibration factor with the provisional run described above (C-176 shape).
- Time `process` over 10⁴ slices on the hosted CI machine; take the 99.9th percentile.
- Commit the procedure and results to `doc/design/reports/` (C-130), tagged `[measured]`.

## Acceptance criteria
1. The counted operations per slice are identical on x86-64, AArch64 and the 32-bit device target, and do not exceed `cost_ops_per_slice`.
2. The p99.9 processing time per slice on the CI machine does not exceed `cost_ops_per_slice` × the CI machine's calibration factor `[measured]`.
3. The p99.9 processing time is below 0.5 ms (one slice period) `[measured]`.
4. The provisional calibration run and the measured numbers are committed, with a note that epic 025's C-176 measurement replaces the factor.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Cost analysis that produces `cost_ops_per_slice` (epic 023).
- The calibration-factor procedure and its per-target values (epic 025, C-176); the factor field in the regime stamp and its use by admission (other epics).
- Device cost (024a-11, epic 026).
