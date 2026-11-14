# 015b-04 — Housekeeping isolation tests (AC-20 a–d)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** M · **Depends on:** 015a-17, 015b-12, 015b-14
## Context
- AC-20 (015b-A2): housekeeping cannot damage a live stream. (a) hellos suppressed for 30 s: live streams continue. (b) holder hello suppressed while sync replies flow: no election. (c) housekeeping thread killed: restarted, streams unaffected, module marked control-degraded. (d) a hello with a changed address: no redirect. ((e) is 015b-02.)
- **L-4:** housekeeping admits unknown sources only; never touches a live data path. **L-5:** destinations pinned at admission, changed only by re-admission. **L-7:** housekeeping panic → restart + control-degraded. **L-8:** no shared locks.
- Elections start only on 1.5 s of sync-reply silence (C-37); hello expiry applies to non-live peers only (C-31). Case (b) therefore checks that the absence of hellos alone never starts an election.
- Test means: (a) and (b) use a scripted peer built from the core codecs that answers sync and carries media but sends no hellos — no hook is needed. (d) uses the C-120 raw-packet hook on a control socket to send a hello with a changed address (015b-12). (c) uses the panic-injection hook on the housekeeping thread (C-145, built in 015b-14). Restart follows C-139 (first restart after a 100 ms backoff; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, its live streams continue and it accepts no new connection). The housekeeping supervisor (015a-16) keeps a minimal refusal path that answers a connection request with ControlDegraded (C-185). AC-20 (c) is the hosted backend's (C-144).
- The holder answers sync requests on its clock thread, never on housekeeping (C-188), so a holder's sync replies continue while its housekeeping restarts.
- Harness binaries such as the scripted peer live in the `hadronyche-tools` crate (C-133); its output format is decided here and recorded in its documentation (C-206).
## Work
1. Scripted peer for (a) and (b) in `hadronyche-tools`; raw-packet hello for (d); panic-injection hook (015b-14) for (c), run once on a consumer and once on the holder, then repeated 5 times within 60 s for the restart stop.
2. Two-process (or two-runtime) loopback scenarios, one per case, each ≥ 30 s where timing matters.
## Acceptance criteria
1. (a) With hellos suppressed for 30 s, zero late slices and zero interrupted inputs.
2. (b) With holder hellos suppressed and sync replies flowing for 30 s, zero elections started; silencing the replies then starts an election after 1.5 s.
3. (c) After an injected housekeeping panic, a restart is observed after the 100 ms backoff (C-139), zero late slices, the control-degraded mark is set, and the stamp records the injection. On the holder, followers see no gap in sync replies during the restart (C-188).
4. (c) After 5 injected housekeeping panics within 60 s, no further restart occurs, the control-degraded alarm is raised, live streams show zero late slices, and a new connection request to the module is refused with ControlDegraded (C-139, C-185).
5. (d) After a hello with a changed address, every output packet still goes to the originally pinned address.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — read only; "Design documents updated" — no behaviour differs; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — 015b-10.
## Out of scope
- Static lock check (015b-02); fail-stop of data/clock threads (015b-05).
