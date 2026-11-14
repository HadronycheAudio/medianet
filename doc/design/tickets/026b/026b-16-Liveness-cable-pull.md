# 026b-16 — Liveness cable pull
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03, 026a-08, 026b-11

## Context

Liveness rules (unchanged from hosted):
- Interrupted after 8 consecutive missed slices (C-38); slices are 0.5 ms (C-02), so 4 ms. Only slices absent at the deadline count; a marked gap or a slice on another timeline epoch resets the count (C-102, C-135).
- Dead after 2 s (C-39), counted from the last slice received with content or a marked gap (C-111).
- Receiver report per bundle every 100 ms (C-40).
- Producer stops after 2 s without reports (C-41), counted from the last receiver report it received.

Epic criterion 026-AC-7 (Liveness): **pulling the cable**: the producer stops after 2 s without reports; on reconnect **within 2 s** of the last slice the stream resumes at the frozen offset; **after 2 s** it is re-admitted.

Both 2 s timers start at their own last event (C-41, C-111) and expire at nearly the same time during a cable pull. The test stays clear of the boundary: 1.0 s and 3.0 s disconnections.

## Work

1. Bench: Linux producer streaming to the board, board playing. Use a controllable switch port or relay to "pull" the cable with a known timestamp; the controller is a tool binary in `hadronyche-tools` (C-133).
2. Case A: disconnect for 1.0 s, reconnect.
3. Case B: disconnect for 3.0 s, reconnect.
4. Log producer state, board stream state, the board's per-connection counters (026b-11), and admission messages. Store logs in `doc/design/reports/` (C-130).

## Acceptance criteria

1. In case B, the producer stops sending 2 s ± 100 ms after the last receiver report it saw (C-41).
2. In case A, the stream resumes at the frozen offset (no re-admission exchange; same offset as before the pull).
3. In case B, the board declares the input dead 2 s ± 100 ms after the last slice it received (C-111), performs a fresh admission, and the stream resumes after it.
4. In both cases the board marks the input interrupted within 4 ms of the first slice absent at its deadline (C-38, C-102).
5. Both cases run 5 times each with identical outcomes.

## Definition of done

- [ ] Every new factual claim carries an evidence tag
- [ ] Result log recorded

## Out of scope

- Fault-induced reset (026b-17).
- Host-level liveness tests (026a-08).
