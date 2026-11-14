# 008b-05 — Case 05: holdover
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-12, 008a-15, 008b-04
## Context
Simulation case 5: holdover — bound grows by 1 µs/s and true error stays inside it. Epic criterion A5 (system AC-6): holdover bound grows by 1 µs/s; the state is never reported usable beyond the lapse.

Rule `[decision]`: in holdover the bound grows by 1 µs per second (C-64, decided, a 1 ppm floor); with a 25 µs lock bound (C-62) against E = 50 µs, the target E of the Linux reference sink (C-70, C-149), alignment claims lapse after about 25 s (C-65, derived). C-64 is decided, so this case checks it; a counter-example goes to the design owner rather than replacing it here. Holdover free-runs at the last rate, so true error growth depends on the crystal drift rate and thermal wander of follower against holder.

State after the lapse, settled by C-162 `[decision]`: `Unlocked`, with time unusable (one-way figures and alignment claims stop); 008a-12 implements it.

Lapse when E is unknown, settled by C-169 `[decision]`: the lapse rule needs a fixed reference, so an unknown E is taken as twice the lock bound (2 × C-62, 50 µs at current values); claims stop when the published bound exceeds it, about 25 s into holdover at C-64.

Holdover entry delay, settled by C-195 `[decision]` except for its value: the constant is at most the election silence threshold of 1.5 s (C-37), a hard limit, and its value within that limit is pending simulation (the POC used 2 s), so that a follower is already in holdover with a growing bound when it calls an election. This ticket proposes the value from this case, case 4 (008b-04) and case 10 (008b-11); the design owner records it in C-195. If no value at or below 1.5 s works, that is reported as a finding (C-195).

## Work
1. Named test `case_05_holdover`: lock under `WIRED_PROFILE` (C-94), then cut the link for 60 s; crystals with drift rate and thermal wander from the two-clock model at several magnitudes; at least 20 seeds.
2. Assert bound slope, true error ≤ bound throughout holdover, and state `Unlocked` with `is_usable()` false after the lapse (C-162). Repeat once with the target E unset in `ServoConfig`: the lapse reference is then 2 × the configured lock bound (C-169), computed from configuration, not a literal.
3. Find the largest drift rate and wander at which true error stays inside the C-64 bound; record in `doc/design/reports/008-simulation-results.md` C-64 as "checked" or "counter-example found" and C-65 as "checked".
4. Sweep `holdover_after_ns` over {0.5, 1, 1.5, 2, 4} s with the burst set of 008b-04 (2 s and 4 s are reported for comparison with the POC only); propose the C-195 value, the shortest value at or below 1.5 s (C-37) with no burst-induced holdover and no honest-lock violation, in the results file, tagged `[measured]` (simulation), with owner (this epic proposes, design owner records in C-195) and falsifier. If no value at or below 1.5 s qualifies, record the counter-example as a finding for the design owner; a larger value is never proposed (C-195 hard limit).
5. Reply-silence check: with `holdover_after_ns` at the proposed value, stop replies after lock and read the view 1.5 s (C-37) after the last reply.
6. Restore the link and check re-acquisition without a phase step.

## Acceptance criteria
1. `bound_ns` slope in holdover is 1 000 ns/s ± 1 %.
2. 0 samples in holdover with |true error| > `bound_ns`, for every seed and the documented wander range.
3. No sample with `is_usable()` true, and every sample in state `Unlocked` (C-162), after the bound exceeds E = 50 µs (C-70; ≈ 25 s after entry from a 25 µs bound, C-65) until re-acquisition.
4. Results file has C-64 and C-65 lines and the proposed C-195 value, at or below 1.5 s (C-37), with owner and falsifier, or the counter-example showing that no such value works.
5. With the target E unset, the lapse occurs when `bound_ns` first exceeds 2 × the configured lock bound (C-169), for every seed.
6. In the reply-silence check, the view 1.5 s after the last reply is in holdover with `bound_ns` larger than at holdover entry, for every seed (C-195).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Holdover logic itself (008a-12).
