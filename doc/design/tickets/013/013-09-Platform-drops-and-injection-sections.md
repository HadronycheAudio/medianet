# 013-09 — Platform, drops and injection sections
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02, 013-03

## Context
- Stamp group **Platform**: OS build, audio device, driver. `[decision]` (The cost calibration factor of C-106 also belongs here; it is added by 013-25.)
- Stamp group **Drops**: per-socket overflow, interface drop counters, measurement-buffer overflow. `[decision]`
- Stamp group **Injection**: any active injection, always stamped. `[decision]` (The commissioning window state of C-107 also belongs here; it is added by 013-25.)
- **Regime fields and counters (epic 013, settled from C-18 and C-47).** Platform and Injection are regime fields: a change changes the `StampId`. Drops are counters: they are refreshed with the stamp every 10 s (C-18) and **do not** change the `StampId` (013-05).
- Rule 7: any active injection is written into the stamp, so an injected run can never pass as a real one. Injection kinds: clock offset, clock drift, packet loss, packet delay, reordering, per-hop delay, and the test-only hooks of C-120 (force a module id and nonce; force a stream onto a given timeline epoch; send a raw packet on a media or control socket), and the hook of C-145 (inject a panic into a data, clock or housekeeping thread), all **stamped like any injection**. Ten kinds in all. Injection may be absent on embedded, which is itself stamped (capability `fault_injection = Absent`).
- Rule 11: measurement-buffer overflow is counted and stamped, because the samples lost under load are the worst ones and silent loss flatters the tail.
- `[code]` Retired POC behaviour: measurement rings that drop silently.
- Linux exposes a per-socket drop counter via `SO_RXQ_OVFL` `[kernel-doc]`; macOS and Windows have none `[memory]`.
- Toggling an injection is a regime change: it must change the `StampId`.
- Builder signatures: `pub fn platform(&mut self, ..);` `pub fn drops(&mut self, ..);` `pub fn injection(&mut self, ..);`

## Work
1. Platform section: OS build, audio device, driver (bounded-length strings, written without allocation).
2. Drops section, flagged "counter" for 013-05: per-socket overflow counter, interface drop counters, measurement-buffer overflow counter; each counter may be "not available" on a profile.
3. Injection section, flagged "regime": zero or more active injections, each with one of the ten kinds and its parameters; an empty list means "none active" and is still written.
4. Implement the three builder methods and readers on `Stamp`; provide `Stamp::injection_active() -> bool`.
5. Extend the fuzz corpus; add field tables and per-profile sources to `doc/design/spec/stamp.md`.

## Acceptance criteria
1. Round-trip preserves all Platform, Drops and Injection fields, including one of each of the 10 injection kinds.
2. A stamp with no active injection still contains the Injection section, and `injection_active()` is false; with one injection of any kind (including each C-120 hook and the C-145 panic hook) it is true.
3. A counter marked "not available" reads back as such, distinct from 0.
4. Adding or removing an injection changes `Stamp::id()`; incrementing a drop counter does **not** change it.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-120 and C-145 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Reading the Linux `SO_RXQ_OVFL` counter into the stamp and the induced-overflow test (013-21).
- Calibration factor and commissioning window fields (013-25).
- Implementing the injection mechanisms and test hooks themselves (other epics).
