# 028b-12 — `certify-sink` command
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** M · **Depends on:** 028b-01, 028b-05, 028b-08, 028b-10, epic 008a, epic 020

## Context
`hadronyche-commission certify-sink <sink>` drives the single-sink marker measurement (the **L2 procedure**, epic 020) and records E, D and restart spread.
- **Commissioning window (C-107).** Driving the marker loop and restarting the sink is test activity, allowed only inside a declared commissioning window that every module stamps. The window is announced by the tool with an expiry time and closes at expiry or on explicit close (C-153, 028b-09). The command checks with the window guard (028b-08) before starting that the window is open everywhere and will not expire before the run ends, and stops if the window closes.
- E = 99.9th percentile of |actual emission − (T(p) + A)| over a **one-hour** run, maximum beside it (C-73).
- Targets: Linux reference sink 50 µs (C-70); macOS 100 µs (C-71); Windows WASAPI exclusive 1 ms (C-72). Pass criteria use these platform targets, never the E measured in the same run (C-149).
- Only the commissioned state gives a numeric E (C-140): the record this command writes is what turns a sink's E from unknown into a number.
- E has a self-monitored part (clock bound + rate tracking, from stamps) and an externally measured part (the uncertainty of D), measured by the marker loop. Budget split: Linux 20 µs clock + 20 µs tracking + 10 µs uncertainty of D (C-70); macOS 40 + 40 + 20 µs; Windows 100 + 100 + 800 µs (proposals, pending L2, C-150).
- D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170); an edge's 2:1 conversion delay is part of it (C-134).
- D must be repeatable across restarts: Linux ≤ 25 µs, macOS ≤ 50 µs, Windows ≤ 0.5 ms spread over **20 restarts** (C-74). An embedded device declares its own restart spread, measured at L3 (C-191); it is the limit for that device.
- "Both stamps" in a certification record are the regime stamps of the sink and of the holder it followed (C-191). If the holder is lost during the run, a holder-eligible module with usable time takes over with frequency continuity and the epoch kept, a Locked candidate before one in holdover and the lowest module id among equals (C-213, C-221); the sink's streams are not restarted, the run continues, and the record carries the stamps of each holder followed.
- The self-monitored part (clock bound and lock state) is what the servo core publishes through the time view (epic 008a, C-187).
- L2 marker offset: Linux constant within ±2 samples over the run; macOS and Windows constant within that platform's restart spread (C-75).
- L2 parameters (proposals, C-166): restart spread of D is max − min; sample tolerances are measured against the median of the first 60 s; the measurement window after a restart is 60 s; a one-sample slip passes on Linux and is reported as a slip event.
- Platform combinations outside the reference list (028b-05) are recorded as unverified.

## Work
- Implement the operator workflow of the L2 procedure: check the window (028b-08), prompt marker loop wiring, run the 1-hour measurement, compute p99.9 and max of the emission error.
- Restart loop: guide the operator (or use a documented restart hook) through 20 restarts, measure D in the 60 s window after each, report spread (max − min, C-166); for an embedded sink, check it against the device's declared restart spread (C-191).
- Compare against the OS target (C-70/71/72, C-149), spread limit (C-74) and marker-offset stability (C-75, tolerances against the median of the first 60 s, C-166); report pass/fail, slip events, and the self-monitored part and the uncertainty of D against the budget split (C-70, C-150).
- Write a `SinkRecord` (028b-10): E p99.9, E max, D, D spread, method `L2-marker`, date, hardware identity, platform verified flag, the regime stamps of the sink and of the holder it followed (C-191), and the window it was taken in; the result is a valid commissioning result although taken inside the window (C-178).

## Acceptance criteria
1. On the Linux reference sink, `certify-sink` reproduces the L2 figures of epic 020's Linux record within 20 % (epic AC-6).
2. D spread over 20 restarts is reported and checked against 25 µs / 50 µs / 0.5 ms for Linux / macOS / Windows, and against the declared restart spread for an embedded sink (C-191).
5. The written record carries the sink's and its holder's regime stamps (C-191).
3. An unverified platform tuple is recorded as such in the record.
4. Outside a commissioning window the command refuses to start, naming the modules that do not stamp the window (epic AC-10).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Group certificate and pairwise skew (028b-14).

## Notes
- The L2 reference figures are measurements owned by epic 020 (remaining gap until its Linux record exists).
