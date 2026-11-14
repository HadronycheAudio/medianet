# 026a-06 — Follower servo and holdover
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** M · **Depends on:** 026a-05, epic 008

## Context

The device runs the **core follower servo** (epic 008) against the Linux holder; it does not reimplement sync logic `[decision]`. Requirements it inherits:

- 50 sync requests/s (C-36), inside the accepted range of 10 to 200 per second (C-112).
- Published time continuous while locked.
- Correction term at most 10 ppm from the holder's rate (C-60) and its rate of change at most 1 ppm/s (C-61). These bound the servo's correction term, not the drift estimate it tracks (C-119).
- Lock bound at most 25 µs (C-62, **pending simulation** — the value may change; tests read it from the constants, not a literal).
- Acquisition to Locked within 10 s of the first exchange (C-63, pending simulation).
- Holdover bound grows 1 µs/s (C-64). Holdover is entered after the holdover entry delay, pending simulation (epic 008b proposes it; the POC used 2 s) and expected at or below the election silence threshold of 1.5 s (C-37), so the device is already in holdover with a growing bound when the election on reply silence starts (C-195). Tests read the delay from the core servo, not a literal.
- The clock lock state and bound are published by the servo core (epic 008a) through the published time view (C-187); the device reads them from there and computes no bound of its own.
- Wall-clock step detection lives in the platform backend that builds each exchange from receive stamps; the servo only receives exchanges that passed it (C-121). On the device that backend is in `hadronyche-embedded`. `DeviceTime` is monotonic, so no step is expected; the detector still runs.
- Holder takeover (C-213, C-221): after a holder loss, a holder-eligible module whose time is usable (Locked, or in holdover with its alignment claim not lapsed, C-65, C-169) takes over with frequency continuity and the timeline epoch is kept; a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins. The epoch is bumped (C-113) only when no candidate has usable time. Seen from the device, a takeover with usable time is a change of holder with no epoch change: its servo rebases on the new holder (core, epic 008a), and its received streams are not restarted.
- Receive timestamps come from MAC hardware timestamping if the part has it, else from the receive interrupt (C-131 value in the stamp). `DeviceTime::declared_jitter_ns()` feeds the servo's error model.

**Alignment-claim margin (settled).** The claim lapses when the holdover bound exceeds the module's E; from the lock bound that takes (E − C-62) / C-64, about 25 s for E = 50 µs (C-65). The device's margin is its declared E (C-73). A device that declares E unknown (C-103) takes as its margin a fixed reference of twice the lock bound (2 × C-62, that is 50 µs) and stops claiming alignment when the bound it currently publishes exceeds that value, about 25 s into holdover at the growth of C-64 (C-169). When the claim lapses, the clock state becomes Unlocked with time unusable: one-way figures and alignment claims stop (C-162). A commissioned device stays commissioned while unlocked: its certified E stays in the stamp, its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, and then resumes without re-commissioning (C-180). The reference device certifies its E in 026b-14 before the hardware test 026b-15.

Epic criterion 026-AC-3: the hello never carries the holder-eligible bit. With the Linux holder removed, the device goes to holdover, its bound grows at 1 µs/s, and it stops claiming presentation alignment once the bound exceeds its declared E. Consequence to document for users: a medianet needs at least one holder-capable (Linux) node, or there is no audio.

## Work

1. Wire the core servo into `Device::poll`: emit sync requests at 50/s using `DeviceTime::now_ns`, feed responses with `Rx.rx_time_ns` and `Rx.source`.
2. In the device backend, build each exchange from receive stamps and run step detection on it before it reaches the servo (C-121); count rejected exchanges.
3. Expose the domain-line time (loop 1: device clock → domain line) to the audio edge (026a-10).
4. Holdover: enter holdover when the core servo does, after its holdover entry delay (C-195), at or before the 1.5 s reply silence that starts the election (C-37); take the bound, last locked bound + 1 µs × seconds in holdover (C-64), from the servo core's published time view (C-187).
5. Alignment claim: `claims_alignment` = locked, or (holdover and bound ≤ margin), where the margin is the declared E (C-65), or, when E is unknown, the fixed value 2 × C-62 read from the constants (50 µs today, C-169). The comparison uses the bound currently published, not the bound at the last lock. When the bound passes the margin, move the clock state to Unlocked with time unusable and stop one-way figures (C-162); leave the declared E and the certification state unchanged, and resume the claim on relock without re-commissioning (C-180).
6. Never transition to holder; if no holder is present at boot, stay unlocked and do not play.
7. On a takeover that keeps the epoch (C-213), follow the new holder through the core servo without resetting the alignment ring, the stream sessions or the admitted connections; only an epoch change (C-113) takes the other-epoch path of 026a-08.

## Acceptance criteria

1. Host test (simulated holder, device clock +50 ppm): device locks within the C-63 time; published time is continuous (no step) while locked; the servo's correction term stays within 10 ppm of the holder's rate and changes by at most 1 ppm/s at every update, while the drift estimate converges to +50 ppm (C-119).
2. Host test with the synthetic wired jitter profile C-94 (base 100 µs, exponential jitter mean 20 µs, 1 % bursts of 1 ms) and `declared_jitter_ns` set to the interrupt-stamp value assumed by the test: locked offset error stays within C-62.
3. Host test: an injected jump in the mock `DeviceTime` is rejected by the backend's step detector and never reaches the servo (C-121).
4. Host test: removing the holder puts the device in holdover after the core servo's holdover entry delay (C-195), no later than the 1.5 s of reply silence at which the election starts (C-37); reported bound after 10 s of holdover = locked bound + 10 µs ± 1 µs.
5. Host test: with declared E = 50 µs, `claims_alignment` turns false at the first instant the holdover bound exceeds E (about 25 s from a 25 µs lock bound, C-65), the clock state reads Unlocked with time unusable (C-162), and both return after relock; throughout, the stamp keeps E `Certified` with the same value and no re-commissioning occurs (C-180).
6. Host test: with E unknown, `claims_alignment` turns false at the first instant the currently published holdover bound exceeds 2 × C-62 (read from the constants; about 25 s from a 25 µs lock bound at C-64), the clock state reads Unlocked with time unusable (C-162), and both return after relock (C-169).
7. Sync request rate measured in test: 50 ± 1 per second.
8. Host test: with the simulated holder removed and a second simulated holder-eligible module in holdover with its claim not lapsed (C-213), the second module takes over with the epoch kept; the device follows it with its correction term's rate of change within C-61, no phase step in its published time, no re-admission and no restart of its received stream, and no other-epoch slice counted. With two candidates, one Locked and one in holdover, the device follows the Locked one (C-221).
9. Any sync-message parser path added here is fuzzed at the C-95 budget.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope

- Measuring real jitter and whether 25 µs is reachable with interrupt stamps (026b-07).
- Hardware holdover test (026b-15).
- Loop 2 (sample counter → device clock), 026a-10.
