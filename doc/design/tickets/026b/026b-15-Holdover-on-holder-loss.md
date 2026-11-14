# 026b-15 — Holdover on holder loss
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03, 026a-06, 026b-14

## Context

Role `[decision]`: **follower only**. Holder eligibility is a capability bit in the hello; no v1 device sets it. After a holder loss, a holder-eligible module whose time is usable (Locked, or in holdover with its alignment claim not lapsed) takes over with frequency continuity and the timeline epoch is kept (C-213); a Locked candidate is preferred over one in holdover, the lowest module id among equals (C-221). Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the epoch (C-113). The device is never a candidate. Seen from the device, a takeover with usable time keeps the epoch, so its received stream is not restarted. Consequence for users: a medianet needs at least one holder-capable (Linux) node, or there is no audio.

Epic criterion 026-AC-3 (Follower only): the hello never carries the holder-eligible bit, even when the device has the lowest module id. With the Linux holder removed, the device goes to holdover, its bound grows at **1 µs/s** (C-64), and it **stops claiming presentation alignment once the bound exceeds its declared E** (C-65). The device enters holdover after the holdover entry delay (pending simulation, at or below the 1.5 s election silence, C-195, C-37), so it is already in holdover when the election on reply silence starts.

The margin is the device's declared E, certified in 026b-14 (C-73). From a lock bound of 25 µs (C-62) and E = 50 µs the claim would lapse after about 25 s; the expected lapse for the board is (E − locked bound) / 1 µs/s. At the lapse the clock state becomes Unlocked with time unusable: one-way figures and alignment claims stop (C-162). The device stays commissioned: its certified E is kept, the claim is suspended and resumes after relock without re-commissioning (C-180).

## Work

1. Bench: Linux holder + board locked and playing, with a certified E in its stamp (026b-14).
2. Capture all hellos from the board (packet capture on the host) for the whole run.
3. Remove the holder (unplug or stop it). Log the board's reported holdover bound and alignment claim (via stamp/telemetry) once per second.
4. Repeat once with the board forced (C-120 hook) to a module id lower than every other node's, and a second holder-eligible node present but not locked.
5. Restore the holder; confirm relock.
6. Repeat with a second holder-eligible Linux node locked to the first: remove the first holder; the second takes over (C-213, C-221). Log the timeline epoch, the stream's session number and the device's per-connection counters (026b-11) across the takeover.
7. Store the logs in `doc/design/reports/` (C-130).

## Acceptance criteria

1. 0 of all captured hellos carry the holder-eligible bit.
2. The board enters holdover after the holdover entry delay in force (C-195), no later than 1.5 s after the holder goes silent (C-37), and never becomes holder, even with the lowest module id (C-113).
3. Reported bound grows at 1 µs/s ± 0.1 µs/s over at least 60 s.
4. The alignment claim turns false within 1 s of the bound exceeding the declared E (C-65); the clock state reads Unlocked (C-162); audio output stops claiming alignment (silent or flagged per 026a-10).
5. After the holder returns, the board relocks and claims alignment again without re-commissioning; its stamp shows the same certified E throughout the run (C-180).
6. In the takeover run of Work 6 the timeline epoch and the stream's session number are unchanged, the stream is not re-admitted or restarted, and no other-epoch slice is counted (C-213).

## Definition of done

- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope

- User-facing documentation of the "at least one holder-capable node" rule (026b-22).
- A device with E unknown on hardware: it stops claiming alignment when its currently published bound exceeds 2 × C-62 (50 µs, about 25 s into holdover, C-169), covered by the host test in 026a-06.
