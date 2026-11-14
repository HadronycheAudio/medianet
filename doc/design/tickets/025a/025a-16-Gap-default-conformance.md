# 025a-16 — Gap default conformance per content type
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-03, 025a-05, 025a-10
## Context
Acceptance criterion 9 (gap behaviour): for each content type, the generated default matches the specification, and an output at p is marked degraded exactly when its input at p − L was gapped. (The reference-mixer half of AC 9 is 025a-19.)

The process function is `process(state, inputs at position p with per-input gap flags, position) -> outputs`. Gap defaults per content type, **overridable**:
- audio: **zero-fill**;
- dense controllers: **hold last value**;
- sparse keyed controllers: **rely on state refresh**;
- read-only transport state: **the last known state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded** (C-141, C-200). Before any transport state has arrived, the state is **stopped, playhead at zero, tempo 120 beats per minute** (C-184, C-199).

A dense controller with no prior value uses the **initial value declared in the manifest** (default 0) when a gap precedes any received value (C-158).

**Degraded marking** comes from dataflow and **follows the content** (C-122): an output at position p is degraded if an input it depends on was gapped at p − L, where L is that output's structural latency. With L = 0 this is the same slice. L is always a whole number of samples (C-105); an output flagged non-constant (recursive filter) follows its structural L like any other (C-104, C-122).

Still open: the refresh interval of sparse keyed controller state — wire specification epic 004. Without it, the sparse-keyed test can check that state is held but cannot bound when normal output returns.
## Work
1. Tests in the `dsl-conformance` job using corpus programs from 025a-05 and 025a-02: feed slices with the gap flag set on chosen inputs at chosen positions.
2. Audio: gapped slice treated as zeros; dense controller: last delivered value held; sparse keyed: no new events, voice state unchanged until the next refresh; transport state: last known state held, output degraded, with a stopped transport's playhead not advancing (C-200) and a playing transport's playhead moving on at its tempo (C-141), tested once with each; transport gapped before any state has arrived: stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199); dense controller gapped before any received value: declared initial value used (C-158).
3. Override: the 025a-05 override program uses its declared non-default behaviour.
4. Degraded marking: in a two-output program where outputs depend on different inputs, gapping one input marks only the dependent output degraded.
5. Degraded with latency: in a program with L > 0 (e.g. the 48-sample and 1 000-sample delays of 025a-02), a gap at input position q marks the output at q + L degraded, and not the output at q (C-122).
6. Degraded with non-constant delay: in a recursive-filter program of 025a-03, a gap at input position q marks the output at q + L degraded, L being the structural L in the manifest (C-104, C-105, C-122).
## Acceptance criteria
1. Audio gap: output equals the output computed with an all-zero input slice, bit for bit.
2. Dense controller gap: output equals the output computed with the last value repeated, bit for bit.
3. Sparse keyed gap: state after the gap equals state before it, and the next refresh restores normal output.
4. Override program follows its override, not the default.
5. Degraded flags match the dependency sets in `expect/` for every gapped input tested.
6. For L > 0, including a non-constant-delay output's structural L, the degraded flag appears at output position q + L for a gap at input position q, and only there (C-122).
7. Transport-state gap: the transport reader's output equals the output computed with the last known state held, the playhead unchanged for a stopped transport (C-200) and moved on at the held tempo for a playing one (C-141), and is marked degraded in both cases.
8. A dense controller gapped before any received value yields the output computed with the manifest's declared initial value (C-158), for the default 0 and for the non-default program of 025a-05.
9. A transport reader gapped before any transport state has arrived yields the output computed with a stopped transport, playhead at zero, tempo 120 beats per minute (C-184, C-199), marked degraded.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The sparse refresh interval is tested if answered, or left recorded as an open item with owner (epic 004) and falsifier
## Out of scope
- Reference mixer gap rule (025a-19); replay exclusion of degraded windows (025b-03).
