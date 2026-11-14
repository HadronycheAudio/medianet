# 022a-17 — Windows regime stamp (`windows-std`)
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-06, 022a-07, 022a-10, 022a-13, 022a-14, 022a-20, 022a-21, epic 004, epic 008a, epic 015

## Context
- Part of epic 022a (Windows edge backend). Windows certified numbers are deliberately looser than Linux's, and the regime stamp says exactly how.
- Transport is settled (C-18): the stamp is a dedicated control message split into sections that each fit 1 252 bytes (C-17); it is multicast to the control group on any change and refreshed every 10 s; a unicast request fetches it immediately; the hello carries only the stamp version number. The layout is epic 004's and sending the message belongs to the hosted backend (epic 015, C-144); this ticket supplies the Windows fields and bumps the version when one changes.
- Capability states are settled (C-129): present, absent, unverified, not requested ("not requested" is a deliberate configuration choice, distinct from a denial). Receive-timestamp source values are *kernel* or *userspace* on Windows, each with a C-129 state (C-131).
- 022-AC-8 (AC-31) — the stamp contains:
  1. profile `windows-std`;
  2. OS build;
  3. audio device;
  4. driver;
  5. receive-timestamp source and its state (C-131);
  6. drop accounting *absent*;
  7. DSCP per class: *present* (peer-verified: media from a consumer's stamp, C-47; sync from the holder's stamp and control from peers' stamps, C-152), *unverified* or *absent* (marking refused) — 022a-05, 022a-21;
  8. pacing *batch per engine period* with the batch size;
  9. bounded real-time *absent*;
  10. real-time scheduling (MMCSS), memory lock and CPU-idle restriction, each in a C-129 state, and strict mode as set by the operator policy (022a-20, C-127, C-165);
  11. wake-latency self-test result (022a-07);
  12. D (*measured*) and E, numeric only in the certification state *commissioned*, otherwise E *unknown* (C-140, C-103);
  13. certification state (uncertified, self-monitored or commissioned, C-140);
  14. cost calibration factor (C-106), read from a data file populated by 022b-09;
  15. commissioning-window state (C-107, C-153): commissioning traffic is allowed only inside a declared window that every module stamps; the tool announces the window on the control group with an expiry time, modules stamp it while open, and it closes at expiry or on explicit close. While a window is open the stamp's ordinary production measurements (latency, loss and alignment figures) are marked unqualified; commissioning results such as the commissioned D are valid commissioning results, recorded by the commissioning tool with the window they were taken in, and are not marked unqualified (C-178);
  16. per-connection summary counters — received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159) — and the observed DSCP, for every connection where this edge is producer or consumer (C-47);
  17. lateness percentiles 50th, 99th, 99.9th and maximum over a rolling 60 s window, with the maximum since the connection went live beside them, for every connection where this edge is consumer (C-160, C-182);
  18. the DSCP observed on each peer's hellos (C-152), from 022a-21;
  19. control isolation *present*: on a hosted platform housekeeping cannot delay the data path, by construction of the thread model (C-161).
- Certification state rules:
  - exclusive mode not obtained → *uncertified* ("interface held") (022-AC-3);
  - any OS build / driver / device combination outside the short reference list → *unverified*, and E *unknown* (C-103);
  - only the certification state *commissioned* (a listed combination with a commissioned D) gives a numeric E; uncertified and self-monitored declare E *unknown* (C-140);
  - a device whose driver misreports latency without the commissioning measurement of D → uncertifiable;
  - a *commissioned* sink whose clock is unlocked stays *commissioned*; its alignment claim is suspended and does not count towards a group's required E until relock, then resumes without re-commissioning (C-180). The lock state comes from the published time view (epic 008a, C-187).
- The batch-size field format is an open sub-question owned by epic 004; until it is answered the field carries minimum, typical and maximum.
- Report-only on Windows: overflow accounting, DSCP marking, bounded real-time, sub-millisecond pacing.
- Trust statement: "Trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment."

## Work
1. A `windows_stamp()` builder collecting fields from: `NetCapabilities` (022a-06: rx-stamp source and state, drop accounting, DSCP per class), `WinRealtime` and strict mode (022a-07, 022a-20), `WasapiDevice` (022a-10: device, driver, format, cert state), sink/capture (022a-13/14: D measured, E or unknown, batch size, per-connection counters, lateness percentiles and observed DSCP), the hello-DSCP report and sync/control verification (022a-21), the calibration data file (022b-09), the commissioning-window state (core), the clock lock state from the published time view (epic 008a, C-187), OS build query.
2. Reference-list lookup: a data file in the crate (populated by 022b-07) of certified (OS build, device, driver) combinations; no match → *unverified* and E unknown.
3. Bump the stamp version on any field change and hand the stamp to the hosted backend's stamp sender (epic 015, C-144), which publishes it per C-18.
4. Document every field (name, values, source, C-id) in the stamp documentation.
5. Unit tests for each certification-state branch and each C-129 state.

## Acceptance criteria
1. **(022-AC-8.)** A stamp from a live edge contains all nineteen listed items, with literal values *absent* for drop accounting and bounded real-time, *present* for control isolation, and *batch per engine period* plus batch size for pacing (unit test with fakes).
2. Interface held → certification state *uncertified*.
3. A combination not on the reference list → *unverified* with E *unknown*; on the list with a commissioned D → *commissioned* with its numeric E; uncertified or self-monitored → E *unknown* (C-140).
4. D in the stamp is labelled *measured*; a missing commissioning D yields not-certified.
5. Changing any field (for example the DSCP state of a class) bumps the stamp version that the hello carries (C-18).
6. While a commissioning window is open, the counters and lateness figures are marked unqualified while the commissioned D is not (C-178); lateness percentiles cover a rolling 60 s window and the maximum since live is reported beside them (C-182).
7. With the clock Unlocked, a *commissioned* sink's certification state stays *commissioned* (C-180).
8. Each capability field takes only the four C-129 values; a deliberately disabled capability shows *not requested*, a denied one *absent*.
9. Field documentation exists and lists every field above.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Stamp splitting, multicast, refresh and unicast fetch (epic 004 layout; sending by epic 015, C-144).
- Populating the reference list (022b-07) and the calibration factor (022b-09).
- Verifying DSCP agreement with a real Linux peer (022b-06).
