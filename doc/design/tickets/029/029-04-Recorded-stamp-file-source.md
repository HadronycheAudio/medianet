# 029-04 — Recorded stamp file source
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-01, 029-02, epic 015

## Context
- Devices cannot write files, so stamps travel on the network and a hosted collector (epic 015) records them. `[decision]` `hadronyche-monitor` (epic 029) can read such a recorded stamp file instead of the live network.
- Epic criterion 029-AC-6: the monitor renders identically from a recorded stamp file and from the live network. That requires the file source to produce the same `Observation` sequence the live source would.
- A file has one writer, owned by the process; the monitor only reads. `[decision]` The single-writer file service belongs to the hosted backend, epic 015 (C-144).
- Open question in the epic, owned by epic 015: the recorded file format, and whether it records hellos. Working assumption: it records hellos and stamp sections with their arrival times. If it records stamps only, hello-derived fields (holder, alarms, control-degraded, presence) cannot be replayed.

## Work
1. `FileSource` implementing `ObservationSource` (029-01), selected by `hadronyche-monitor --file <path>`.
2. Parse the recorded format produced by the epic 015 stamp collector; produce `Observation::Stamp` and `Observation::Hello` (if the file records hellos) in recorded order, with recorded timestamps preserved in the view. Stamp sections go through `StampAssembler` (029-02) exactly as in the live source.
3. Two pacing modes: as fast as possible (for tests and headless log formatting) and real-time (using recorded timestamps).
4. Open the file read-only; malformed records are skipped and counted, never panic.

## Acceptance criteria
1. A file recorded by the epic 015 collector from a two-module run yields the same sequence of observations (equality) as captured live by `LiveSource` (029-03) during that run.
2. A truncated file yields all complete records and then `None`; no panic.
3. A record with a corrupted byte is skipped and the skip counter is 1.
4. The file is opened with read-only flags (test asserts no write handle).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The record parser is exposed as a pure function so the 029-05 fuzz harness can drive it
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Defining the file format (epic 015); render equivalence test (029-19); fuzzing (029-05).
- If epic 015's answer is "stamps only", acceptance criterion 1 is reduced to stamps and the missing hello fields are recorded in the epic's open questions; this ticket does not invent a recording of its own.
