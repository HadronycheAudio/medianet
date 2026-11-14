# 018-14 — AlsaSink rate loop, E self-monitoring and alarms
**Epic:** 018-Linux-ALSA-edge · **Effort:** M · **Depends on:** 018-03, 018-04, 018-05, 018-06, 018-13

## Context
- Wires the shared `edge/` core into `AlsaSink`: `DeviceLine` fed from ALSA status pairs (018-10), `EdgeTimebase::update` then `ratio()` → `RateActuator::set_ratio` on the kit's steerable rate actuator (epic 024a, C-197; its ratio varies by ppm around nominal, the fixed 2:1 conversion of a 48 kHz device being the kit's resampler node, C-189), `emission_error_ns` per period, `EmissionStats` (99.9th percentile + max, C-73), `AlignmentClaim`/`CertificationState`.
- The rate-actuator trait is the kit's, defined by epic 024a with its implementation (C-210); its ratio range is ±1 000 ppm around the nominal ratio, which contains the ±100 ppm crystal offsets of 018-A1 and the drift ramp of C-119.
- **L2 handover (C-212, C-202):** triggered as a holder loss: the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. The run asserts emission error within the target E during holdover and frequency continuity at takeover (rate change within C-61, no phase step); a run in which holdover was not observed is failed, not passed. The rig therefore needs a second holder-eligible Linux node (provided by epic 020; the fake test uses two fake holders).
- Rate-loop bandwidth well below 1 Hz (C-68). Servo envelope: slew ≤ 10 ppm (C-60), rate change ≤ 1 ppm/s (C-61), frequency-continuous handover. These bound the servo's correction term, not the drift it tracks (C-119); the resampler ratio follows the real device drift.
- L2 drift-ramp scenario (C-119): injected crystal drift sweeping ±100 ppm at no more than 0.5 ppm/s; pass is E held throughout.
- Holdover: clock bound grows 1 µs/s (C-64); claim lapses after about 25 s (C-65).
- E target on the Linux reference sink: **50 µs** (C-70, pending measurement) = 20 µs clock bound + 20 µs rate tracking + 10 µs D uncertainty; the servo lock bound must be ≤ E/2 = **25 µs** (C-62).
- A sink that cannot meet A raises the connection alarm "cannot meet A" (C-173). Its condition is settled (C-174: maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows) and is evaluated in 018-24 from input lateness; this ticket raises the "alignment claim lapsed" alarm.
- After the claim lapses the clock is Unlocked and time unusable (C-162): E stays blank. The certification state is unchanged: a *commissioned* sink stays commissioned with its claim suspended, and resumes on relock without re-commissioning (C-180).
- Pass criteria mean the target E (C-149): the 20 µs rate-tracking budget is a share of the 50 µs target, not of a measured E.
- Alarms use the hosted runtime's alarm path (epic 015).

## Work
1. Status-pair polling each period → `DeviceLine::update`.
2. Each period: `EdgeTimebase::update`; `set_ratio(ratio() / k)`, where k is the fixed node's conversion factor (2 for a 48 kHz device, 1 for a 96 kHz device), so the actuator receives only the ppm deviation around its nominal 1.0 (C-189).
3. Each submitted period: compute emission error; record into `EmissionStats`; publish the current E (p99.9 and max) to the runtime's metrics; **blank while not locked** (`None` from 018-04).
4. Evaluate `AlignmentClaim` each period; raise the "alignment claim lapsed" alarm via the epic 015 alarm path.

## Acceptance criteria
1. **Epic criterion 018-A2:** with a fake backend, an E value is published for every submitted period once locked; before lock, the published value is blank (not zero).
2. **Epic criterion 018-A3:** forcing holdover, after 25 s the sink reports that it no longer claims presentation alignment and raises the alarm; a *commissioned* sink still reports *commissioned* (C-180).
3. With a fake device crystal at +100 ppm, the sink's emission error settles within 20 µs after 30 s and stays there for 10 min of simulated time.
4. A simulated clock-holder handover with a 50 ppm crystal difference between holders (C-78; on fakes the whole 50 ppm is injected, while the L2 run makes it up per C-223), triggered as a holder loss (the old holder's fake stops answering, about 2 s of holdover, then the new holder takes over, C-212), causes no slip (offset returns within ±1 sample), keeps the emission error within the 50 µs target E (C-149) throughout the holdover (C-202), and shows frequency continuity at takeover (no phase step, rate change within C-61, at most 1 ppm per second); the test asserts that holdover was observed (C-212) — contribution to AC-26.
5. Drift ramp (C-119): a fake device crystal sweeping from −100 ppm to +100 ppm at 0.5 ppm/s keeps the emission error within the 20 µs rate-tracking budget throughout, while the servo correction term on the fake view stays within C-60 and C-61.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The one-hour E ≤ 50 µs certification and the hardware drift ramp (epic 020).
- Stamp serialization (018-18), admission (018-15), departure on sleep (018-21).
- The "cannot meet A" alarm (018-24).
