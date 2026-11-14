# 009-22 — Design documents and epic close-out
**Epic:** 009-Gap-and-reassembly-core · **Effort:** XS · **Depends on:** 009-01 to 009-16, 009-18 to 009-21, 009-23 to 009-30

## Context
Epic 009 (receive side of the data plane in `hadronyche-core`: source filter, copy-index dedup, reassembly, alignment ring, fixed-time fan-in, gap classes, degraded flag and its delay through L, input health states, offset estimator and ratchet, default concealment) ends with documentation and a check of its definition of done.

Where things go (C-130 `[decision]`): design documents and the constants table (`doc/design/02-constants.md`) in the design set's repository; any report or result in `doc/design/reports/`; the evidence appendix is `doc/design/06-evidence-appendix.md`. Nothing is written into the frozen POC tree. Code stays in `hadronyche-core` (tests and `fuzz/` beside it); this epic has no probe or tool code (C-133). Evidence tags: `[code]`, `[kernel-doc]`, `[decision]`, and `[standard]` for a figure cited from a named published standard with its clause (C-132).

Rulings this epic now implements, to be reflected wherever a design document still states the earlier assumption `[decision]`:
- C-101 persistently late; C-108 late versus missing; C-109 Demoted (no automatic retry; the former ticket 009-17 was removed); C-102 miss definition; C-110 aligned reassembly deadline; C-111 start of the dead threshold; C-135 other-epoch slices restart the dead clock; C-19 session (009-24); C-47 and C-159 stamp counters; C-95 fuzz budget; C-96 and C-142 rings sized to A, inherited A or configured budget; C-100 IPv4 only; C-105 (whole-sample L; the rational-L rule is withdrawn), C-134 and C-122 degraded flag through L (009-23); C-156 drain cycle; C-157 and C-160 alignment-group figures (009-25) over the rolling 60 s window of C-182, which the cannot-meet-A test also uses (009-13, 009-25, 009-26); C-163 last ratchet step; C-174 cannot-meet-A (009-26); C-158, C-141 (as revised), C-200, C-184 and C-199 concealment (009-19, 009-27); C-186 re-alignment of a fan-in on a change of inherited A (009-29); C-208 re-alignment position picked by the fan-in one lead (C-43) ahead and announced in its output, for a change of A and for a ratchet step (009-16, 009-29); C-211 a change of A deeper than the compiled ring clamps the ceiling and raises cannot-meet-A (009-26, 009-29), and `check_depth` is the commissioning tool's check (009-11); C-216 the alarm stays raised until a later change of A that fits (009-26, 009-29) and the receiver report carries the A given, not the clamped value (009-30); C-215 a ratchet step is a re-alignment at a position announced one lead ahead, never silently on the next round (009-16); C-214 a sink switches at its own position one lead ahead, unannounced (sink epics; noted in 009-29); C-148 the commissioning proposal of A and the ladder runs use the maximum since live, not the rolling figure (009-25); C-185 NoSkewBudget refusal reason (009-11); C-198 current A carried in the receiver report and inherited A taken from downstream reports (009-30); C-08 as revised and C-209 (controller member counts towards the 32), C-181 and C-205 (controllers in their own member stream, or sharing a single-stream source's audio packet: concealed section by section, 009-19); C-178 production figures and counters unqualified while a commissioning window is open (009-14, 009-25); C-187 clock bound published by the servo core 008a (009-13); C-192 vectors crate at the workspace root and fuzzing tool of the team's choice, C-193 sources of the vectors (009-08, 009-09); C-164 bad-input bound (009-28); C-144 gap slice emission in epic 015; C-04 slice size (6 × 192 B = 1 152 B audio; C-27 arithmetic 16 × 20 × 1 152 B = 368 640 B ≈ 370 KB).

Choices made inside this epic that must be written down `[decision]`:
- `InputCounters` fields kept outside the C-159 stamp list: `reassembly_missing_packets` and `rejected` (receiver report, 009-01).
- `Inserted::TooEarly`, `Inserted::Late { within_span }`, `InputAt::OtherEpoch`, `RoundInput::Demoted` (009-10, 009-12); `FanIn` ring capacity const generics.
- Reading of C-111 "received": any slice with content or a marked gap reaching the ring, on time or late (009-15); other-epoch slices restart the clock by C-135.
- Lock check by a clippy disallowed-types list (009-02).
- Per-input `C: Conceal` override (009-19).
- Reading of C-182 "since the connection went live" across a C-186 re-alignment: the maximum since live and the rolling window are kept, since the connection is not re-admitted (009-29).

Open sub-questions to keep recorded with owner and falsifier (from the epic):
- C-16 drain bound vs legitimate bursts: L1 / L2 Windows (tasks 019, 022).
- Maximum-plus-headroom over 4 s sufficient: task 008, then L1.
- Slot eviction when all SLOTS are busy: this epic (009-06), then L1.
- No lateness sample during aligning: epic 005.
- Interrupted input returning on a new session: epic 005.
- C-164 bad-input bound: L1 measures the 10 µs figure.
Closed: maximum skew budget (C-96, C-142), carrier of a changed A (C-198), announced position of a C-198-triggered re-alignment (C-208), a changed A deeper than the ring (C-211), alarm and reported A after a clamped change (C-216), ratchet step at an announced position (C-215), where a sink switches (C-214), transport playhead through a gap (C-141 as revised, C-200), default tempo (C-199), demoted retry (C-109), cycle of C-16 (C-156), reclaimable latency (C-157), window of the lateness figures (C-182), dense controller initial value (C-158), stamp class of the extra counters (C-159), C-122 wording (C-122 as revised).

## Work
1. Update the design documents where behaviour differs, constants table first (any C-xx row changed or added, each with an evidence tag). This epic proposes no new constant; if a row would be needed, it goes to the design owner rather than being written here.
2. Document the regime stamp fields touched: the consumer's C-159 per-connection counters produced by `stamp_summary`; the alignment-group figures (reclaimable latency, lateness percentiles, C-157, C-160, over a rolling 60 s window with the maximum since live beside them, C-182); the degraded flag on output slices (including through L); the multi-packet regime fields read by the reassembler. The observed DSCP is the backend's.
3. Document the counters exposed for the receiver report (epic 014): all `InputCounters` fields.
4. Walk the epic's definition-of-done list and tick each item with a link to the evidence (CI job, test, fuzz target, doc section). Per C-125, any line found to have no subject is marked not applicable with the reason (the epic records none).

## Acceptance criteria
1. Every ruling and choice listed in Context appears in a design document with an evidence tag.
2. The constants table reflects any changed or added constant before any prose change.
3. Every still-open sub-question is recorded with owner and falsifier; each closed one points to its rule.
4. All 9 epic definition-of-done items are ticked with evidence or marked not applicable with a reason (C-125).
5. No file is written outside the locations of C-130 and C-133, and none into the frozen POC tree.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Late, missing, marked and rejected counters are all exposed for the receiver report and the stamp

## Out of scope
- Any code change (all in the other tickets).
- Documents of other epics, except where this epic's behaviour differs from them.
