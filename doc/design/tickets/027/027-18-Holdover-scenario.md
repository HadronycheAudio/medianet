# 027-18 — Holdover scenario
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-11, 027-22, epic 004

## Context
- L3 rig: Linux sink on capture channel 1, embedded device on channel 2; skew = ch2 − ch1, inter-channel skew subtracted. Pass rule (C-79): p99.9 |skew| ≤ E_linux + E_device, max reported.
- Scenario `holdover` (fixed by the epic): holder removed for 60 s; both sinks must stop claiming alignment once their bound exceeds their margin.
- Rules: in holdover the clock bound grows 1 µs/s (C-64); alignment claims lapse after about 25 s (C-65). Lock bound at most E/2, 25 µs against C-70 (C-62, pending simulation). After the claim lapses the clock state is Unlocked, with time unusable: one-way figures and alignment claims stop (C-162). The lapse is therefore observable as the transition of the stamped clock state to Unlocked.
- **Margin (settled):** C-65 is derived as (C-70 − C-62) / C-64, which places the lapse at the moment the clock bound reaches the sink's E. The margin is therefore each sink's own E: about 25 s into holdover for the Linux sink (bound starting at the 25 µs lock bound), and (E_device − bound at holdover entry) / (1 µs/s) for the device. If a sink's bound at entry is below its lock bound, its lapse comes later than this estimate.
- **Holdover entry (C-195):** a sink enters holdover a short delay after the holder stops answering; the delay is pending simulation (proposed by epic 008b; the POC used 2 s) and is expected at or below the election silence threshold of 1.5 s (C-37). The expected lapse is therefore counted from holdover entry, read from the stamped clock state, not from the removal instant.
- **Certification through holdover (C-180):** a commissioned sink whose clock is unlocked stays commissioned; its alignment claim is suspended and resumes once the clock is locked again, without re-commissioning. The eligibility gate (027-24) therefore still passes during and after this scenario.
- **027-AC-6, Holdover honesty:** with the holder removed, each sink stops claiming presentation alignment once its bound exceeds its margin (its E); skew measured after that point is reported but not asserted.
- Reading the claim and the bound: stamps are fetched by unicast request and every regime change is multicast (C-18, ticket 027-22). The clock state and bound are published by the servo core (epic 008a) through its time view (C-187); which stamp fields carry them is still open, owner epic 004 (stamp layout) with epic 005.
- No handover must happen (C-213): a holder-eligible module whose time is usable — Locked, or in holdover with its alignment claim not lapsed — takes over with frequency continuity, and once no candidate has usable time the lowest-id holder-eligible candidate wins and bumps the epoch (C-113). So during the removal no holder-eligible module may remain: the second Linux host is off or not holder-eligible, and so is any other module on the rig, or the scenario becomes a handover or an epoch change.

## Work
1. `RigControl::remove_holder(duration)` / `restore_holder()`; with the second Linux host powered off or not eligible, and no other holder-eligible module present, so no takeover and no epoch bump happen (C-213, C-113).
2. Scenario `holdover`: baseline, holder removed for 60 s, holder restored, recovery window. Fetch each sink's stamp at ≥ 1 Hz and log every multicast regime change; record clock state and bound from the fields epic 004 defines.
3. Report: per sink, time from removal to holdover entry (C-195), bound at entry, time from entry to claim lapse, bound at lapse, E (the margin), expected lapse time computed from C-64; skew before lapse asserted against the limit, skew after lapse reported only; after the holder is restored, the time to re-lock and to the claim resuming, and the certification state throughout (C-180).

## Acceptance criteria
1. **027-AC-6.** Each sink's alignment claim lapses no later than the moment its bound exceeds its E (tolerance one poll interval or the multicast update time, whichever is shorter). For the Linux sink this is expected around 25 s after holdover entry (C-65, C-195).
2. Up to the earlier of the two lapses, `p999_abs_ns <= limit_ns`.
3. Skew after the lapse is in the report, flagged "not asserted".
4. Neither sink claims alignment at any time after its lapse until the holder is restored; from the lapse on, each sink's stamped clock state is Unlocked (C-162).
5. No takeover happens and the timeline epoch is unchanged during the removal, as read from the stamps (C-213, C-113).
6. Each sink's stamped certification state stays commissioned throughout the scenario, and after the holder is restored and the clock is locked again its alignment claim resumes without re-commissioning (C-180).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Regime stamp fields touched are complete and documented (clock state and bound, as read)
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Design documents updated" — 027-20; "The rig is photographed or diagrammed" — 027-09; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Holder handover (027-14).
- Changing the holdover constants C-64 / C-65.
- Defining the stamp fields for claim and bound (epic 004).
