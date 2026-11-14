# 017b-14 — Scenario `gap-classes`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-03, 017b-04, 017a-08, 017a-09, 017b-09, epic 009

## Context
- AC-16: a marked gap propagates with the degraded flag; interrupted after 8 consecutive misses (C-38); dead after 2 s (C-39); resume from interrupted is immediate at the frozen offset.
- Settled rules:
  - **What counts as a miss (C-102):** only slices absent at the deadline — missing, or arriving late. A marked gap or a slice on another timeline epoch proves the upstream alive and resets the count.
  - **Start of the dead threshold (C-111):** 2 s counted from the last slice received with content or a marked gap.
  - **Degraded flag (C-122):** stamps denote content position; the output stamped q carries the content of input position q (emitted L later) and is degraded if that input was gapped. A marked gap at the input therefore yields a degraded output at the same position.
  - **Alarms (C-173):** a connection alarm is raised on interrupted and on dead. Only Live connections are shown satisfied; Interrupted is shown by name as unsatisfied (C-154).
  - **Transport on a gap (C-141, C-200):** the last known transport state is held and the playhead advanced as that state dictates: a playing transport moves on at its tempo, a stopped one does not advance; the output is degraded. Before any transport state has arrived it is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199).
- Reference mixer on a gap `[decision]`: a missing input contributes zero and the output slice is marked degraded.
- Slice = 0.5 ms (C-01, C-02), so 2 s = 4 000 slices.
- Injection: link loss and delay (017b-03), set and cleared on slice boundaries, or `partition` (017b-04), to cut an input for an exact number of slices; marked gaps come from the sender's harness control (017a-09).

## Work
1. Scenario `gap-classes` with these phases on one mixer input:
   - (a) drop 1 slice; (b) drop 7 consecutive; (c) drop 8 consecutive;
   - (d) 4 dropped then 4 late (delayed beyond the offset) consecutive slices;
   - (e) 7 dropped, 1 marked gap, 7 dropped;
   - (f) cut for 1.5 s then restore;
   - (g) the sender emits only marked gaps for 3 s;
   - (h) cut for 2.5 s then restore.
2. Assertions per phase:
   - (a), (b), (e): input never interrupted (the marked gap in (e) resets the count, C-102); each gapped slice gives one degraded output slice, and the flag propagates downstream.
   - (c), (d): interrupted at the 8th consecutive miss, late slices counting as misses (C-102).
   - (f): interrupted, then on restore the first arriving slice is used immediately at the offset frozen at interruption (no realign).
   - (g): never interrupted and never dead (C-102, C-111); every output slice degraded.
   - (h): dead 2 s after the last slice with content or a marked gap (C-111).

## Acceptance criteria
1. `hadronyche-l0 gap-classes --duration 60` exits 0.
2. Interrupted is entered after exactly 8 consecutive missing or late slices, never after 7, and never when a marked gap breaks the run.
3. Dead is declared 2 s (± one slice) after the last slice with content or a marked gap; 3 s of marked gaps never cause dead.
4. Resume from interrupted: first output slice after restore is non-degraded and uses the frozen offset (offset before == offset after).
5. Degraded output slice count equals gapped input slice count (missing, late and marked gap).
6. One connection alarm is raised on each entry into interrupted and on dead, none in phases (a), (b), (e) and (g) (C-173); while interrupted the connection is reported unsatisfied (C-154).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Realignment and ratchet (017b-15); epoch mismatches (017b-16).
