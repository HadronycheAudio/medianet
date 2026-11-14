# 013-08 — Clock and alignment sections
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02, 013-03

## Context
- Stamp group **Clock**: discipline mode, lock state and bound at each one-way figure, timeline epoch (16 bits, C-10). `[decision]`
- Stamp group **Alignment**: offset, the lateness percentiles and the maximum since the connection went live, reclaimable latency, audition offset A, required E (or none), sink certification state. `[decision]`
- Clock and the Alignment fields offset, A, required E and certification state are regime fields (013-05). The lateness percentiles, the maximum since live and the reclaimable latency are reported values that move during a run, so by the epic's definition they are counters: they go in a separate alignment-measurements section flagged "counter" and do not change the `StampId`.
- Rule 1: a timeline epoch change is a regime change (new `StampId`, new record set).
- Rule 3: one-way figures are blank until the clock is locked and carry the bound thereafter. The stamp carries the lock state and bound; the producers of one-way figures (epics 015, 017) do the blanking.
- Lock bound: at most E/2, that is 25 µs against the Linux E (C-62, pending simulation).
- Publisher of the lock state and bound (C-187): the servo core (epic 008a), through the published time view. The producer of the Clock section reads them from that view; this ticket carries them and computes nothing.
- After the holdover claim lapses (C-65) the clock is Unlocked with time unusable (C-162): the stamp shows it unlocked with no bound. For a sink whose E is unknown the lapse is judged against twice the lock bound (2 × C-62, 50 µs; C-169); the stamp shows the same result.
- Offset estimator: maximum observed lateness plus headroom; **percentiles are reported, not used** (C-51). The stamp carries the **50th, 99th and 99.9th percentiles and the maximum** (C-160).
- Window of these figures (C-182): the percentiles, the reclaimable latency (and the cannot-meet-A alarm of C-174, not stamped here) use a **rolling 60 s window**; the **maximum since the connection went live** is reported beside them. The percentile maximum is therefore the 60 s maximum, and the since-live maximum is a separate field. `[decision]`
- Reclaimable latency (C-157): offset − (current maximum lateness + headroom), when positive; published in the Alignment group.
- Audition offset A: no default, one per presentation group (C-55); alignment rings are sized to A (C-96).
- Which A is stamped (C-198): the module's **current** A, its own or the one it inherited (a fan-in outside any presentation group inherits the downstream group's A, C-142). The receiver report (C-40) carries that A upstream one hop per report, and each fan-in that inherited the old value re-aligns (C-186). A is a regime field, so a change of the inherited A changes the `StampId` and starts a new record set like any other change. `[decision]`
- A fan-in with no downstream presentation group uses the explicit budget in its configuration (C-142); with neither, its aligned inputs are refused with NoSkewBudget (C-185). The A field therefore has a "none" value, distinct from any number.
- Rule 12: certification failures of a sink (it cannot bound its emission error E) are in the stamp, in the Alignment group. **A sink with no certified E declares E as unknown** (C-103); a presentation group may set a required E or none (C-103). Only a **commissioned** sink gives a numeric E; **uncertified** and **self-monitored** sinks declare E unknown (C-140).
- A commissioned sink whose clock is unlocked **stays commissioned** (C-180): its certification state and E are unchanged, its alignment claim is suspended (it does not count towards a group's required E) until the clock is locked again, then resumes without re-commissioning. The suspension is read from the Clock section's lock state; the certification field does not change on unlock or relock.
- E targets: **Linux 50 µs (C-70), macOS 100 µs (C-71), Windows 1 ms (C-72)**, all pending measurement. Definition of E (C-73): **99.9th percentile of absolute emission error over a one-hour run; maximum reported beside it.**
- Builder signatures: `pub fn clock(&mut self, ..);` `pub fn alignment(&mut self, ..);`

## Work
1. Define the Clock section: discipline mode, lock state, current bound, timeline epoch.
2. Define the Alignment section (regime): offset, audition offset A (own, inherited or configured budget, C-142; or "none", C-185), required E or "none", and a sink-certification field with five states: not a sink; commissioned (with measured E p99.9 and maximum, per C-73, C-140); failed (with the measured values); uncertified (E unknown); self-monitored (E unknown) (C-103, C-140).
3. Define the alignment-measurements section (counter): lateness at the 50th, 99th and 99.9th percentiles and the maximum (C-160) over the rolling 60 s window (C-182) as fixed fields, the maximum lateness since the connection went live (C-182), and the reclaimable latency (C-157, over the same 60 s window), "none" when not positive. State the window in the spec table.
4. Implement `StampBuilder::clock`, `StampBuilder::alignment` (writing both alignment sections) and readers on `Stamp`.
5. Extend the fuzz corpus; add field tables, units and per-profile sources to `doc/design/spec/stamp.md`.

## Acceptance criteria
1. Round-trip preserves every Clock and Alignment field.
2. An unlocked clock reads back as unlocked with no bound value (not 0).
3. Changing the timeline epoch changes `Stamp::id()`.
4. The five sink-certification states round-trip and are distinguishable; only commissioned carries a numeric E (C-140); uncertified and self-monitored read as E unknown and are distinct from "failed".
5. Required E "none" round-trips and is distinct from any numeric value.
6. The four 60 s lateness values (p50, p99, p99.9, maximum), the since-live maximum and the reclaimable latency (a value and "none") round-trip, with the since-live maximum distinct from the 60 s maximum (C-182); changing any of them leaves `Stamp::id()` unchanged.
7. Units are stated for every time field (ns or µs) in the spec table.
8. Changing the stamped A, whether own or inherited (C-198), changes `Stamp::id()`.
9. A = "none" (C-185) round-trips and is distinct from any numeric value.
10. A commissioned sink with its clock unlocked reads back as commissioned with its numeric E and the Clock section unlocked; relocking leaves the certification field byte-identical (C-180).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-51, C-62, C-73, C-103, C-140, C-142, C-157, C-160, C-162, C-169, C-180, C-182, C-185, C-187, C-198 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Blanking one-way figures before lock in the reports themselves (epics 015, 017).
- Measuring E or running certification (other epics).
- Computing the percentiles, the since-live maximum and the reclaimable latency (the alignment epics); this ticket carries the values.
- Publishing the lock state and bound (the servo core, epic 008a, C-187); admission's use of a suspended claim (C-180); the cannot-meet-A alarm (C-174).
