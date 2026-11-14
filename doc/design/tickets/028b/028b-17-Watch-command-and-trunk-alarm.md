# 028b-17 — `watch` command and trunk-rate alarm
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028a-02, 028a-03, 028a-07, 028b-01

## Context
After deployment the tool alarms post hoc from the regime stamps; `watch` is read-only (C-107). The runtime never computes trunk feasibility; trunk capacities are declared per link and **per direction** (C-117) (`[[link]] name, a, b, rate_mbit`), and each `[[connection]]` lists its trunks in `path`; the direction of each crossing is derived as in 028a-07. `watch` compares the observed rate per link and direction (summed from per-connection `received` counters in stamps, C-47) with the declared capacity. Admission ceiling 70 % of link rate (C-23); per-stream rate about 21.1 Mbit/s at 2 000 packets/s (C-21), about 1 320 bytes per packet on the wire (C-20).

Stamps are refreshed every 10 s and on any regime change (C-18). Session numbers are per stream, assigned by the source and bumped on restart and timeline epoch change (C-19); the tool treats a session change, or a counter that goes backwards, as a new counter baseline rather than as negative traffic. A holder takeover by a candidate with usable time keeps the timeline epoch (C-213, C-221), so it bumps no session and restarts no stream; only an epoch bump when no candidate has usable time (C-113) does.

A controller stream's packets are smaller than audio packets: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of overhead), at 2 000 packets/s (C-219); the sizes come from the configuration (`controller_bytes`, `header_bytes`, 028a-03) through `capacity` (028a-02).

## Work
- `watch --config <file>`: subscribe to stamps (028b-01) continuously; at each stamp refresh compute the observed rate per declared trunk and direction from per-connection packet counters × 1 320 bytes (C-20) for audio streams, and × the C-219 packet size for controller streams, along each connection's `path`.
- Alarm `trunk.over_declared`: observed load in one direction > declared `rate_mbit` × 0.70, or > declared rate; name link, direction, observed, declared and top contributing connections.
- Counter baseline handling for session changes (C-19).
- Alarm framework (one line per alarm with stable id, timestamps, raise/clear) used by 028b-18. Alarms raised while a commissioning window is open are marked as such, since ordinary production measurements in stamps (latency, loss, alignment figures) are unqualified while a window is open (C-178) and the window carries test traffic.

## Acceptance criteria
1. With a trunk declared smaller than its observed load in one direction, `watch` raises the named alarm `trunk.over_declared` from the stamps, naming the direction (epic AC-8), in a fixture replay and on a lab rig.
2. When the load drops back, the alarm clears.
3. A fixture with a source restart (session bump) raises no false alarm; a fixture with a holder takeover that keeps the epoch (C-213) shows no counter baseline reset.
5. A fixture whose trunk carries a controller stream sums it at the C-219 size, not at 1 320 bytes per packet.
4. `watch` sends nothing on the medianet except stamp fetch requests (epic AC-10).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Loss, late slices and A alarms (028b-18).
