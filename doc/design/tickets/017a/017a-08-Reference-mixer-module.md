# 017a-08 — Reference mixer as position-aligned fan-in
**Epic:** 017a-Reference-modules · **Effort:** M · **Depends on:** 017a-03, epic 009, epic 015

## Context
- POC mixer (`src/mixer.rs`) `[code]`: sums the sender's multi-channel feed with a controller-set volume, and mixes synth inputs **on arrival** using each input's latest block, with no alignment (`mixer.rs:108-124`). That behaviour is **retired**.
- New rules `[decision]`:
  - The reference mixer is a **fan-in that fires at the fixed time `T(p) + offset`** for position `p`, using the alignment ring of epic 009.
  - **On a gap:** a missing input contributes zero and the output slice is marked **degraded**; the flag follows the content through the mixer's structural latency L: the output stamped q carries the content of input position q and is degraded if an input at q was gapped (C-122). L is a whole number of domain-rate samples (C-105); the mixer contains no resampler node, which may appear only inside an edge module (C-134).
  - Content domains: audio inputs, a dense controller (volume), read-only transport state. No libm in the signal path.
- **Offset (settled):** the offset is each input's alignment offset from epic 009: measured during aligning (4 s, C-50) as the maximum observed lateness plus headroom (C-51, C-52: the largest of 25 % of the observed maximum, one slice period, twice the published clock bound), then raised by the ratchet one slice period at a time up to the presentation group's audition offset A (C-53). A has no default and is set per presentation group (C-55). The alignment ring is sized to A (C-96).
- **A for a fan-in outside any presentation group (settled, C-142):** the mixer inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission. With no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused with reason NoSkewBudget (C-185). Following a later change of the inherited A (C-186, C-198, C-208, C-211) is ticket 017a-13. In L0 the mixer feeds the reference sender, not a presentation group, so each scenario configures an explicit budget.
- **Transport state on a gap (settled, C-141):** the last known transport state is held and the playhead advanced only if that state is playing; a stopped transport's playhead does not advance (C-200; 017a-03); the output is marked degraded. Before any transport state has arrived, the view reads stopped, playhead at zero, 120 beats per minute (C-184, C-199).
- **Late versus missing (settled, C-108):** at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span, it is reclassified late (missing −1, late +1) and discarded.
- **Multi-packet input (settled, C-110):** an aligned multi-packet input is reassembled until the mixer's own deadline for that position.
- Slice = 48 frames = 0.5 ms at 96 kHz (C-01, C-02); 6 channels per stream, 1 152 bytes of audio per slice, no per-channel id byte (C-04).
- Gap-class behaviour from epic 009 (asserted in L0 by 017b-14): interrupted after 8 consecutive missing or late slices, a marked gap or another-epoch slice resetting the count (C-38, C-102); dead 2 s after the last slice with content or a marked gap (C-39, C-111); resume from interrupted is immediate at the frozen offset.
- **Dense controller (settled, C-192):** the reference dense controller is the mixer's volume, so the mixer is the reference module that exercises the dense-controller domain. As a dense controller it declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158). The volume reaches the mixer in the sender's audio feed packet, within the 100-byte header cap (C-181, C-137; see 017a-09).

## Work
1. Implement `hadronyche-reference/src/mixer.rs` against the module API of epic 015: N audio inputs, one volume input of the dense-controller domain (C-192) with its initial value declared in the manifest (default 0, C-158), one audio output.
2. Take the ceiling A from the downstream presentation group learned at admission, or from the explicit budget in configuration; refuse aligned inputs with reason NoSkewBudget when neither exists (C-142, C-185).
3. Fire at `T(p) + offset` per position via epic 009's alignment ring; sum inputs present for `p` sample by sample with the volume applied, in integer or bit-reproducible float arithmetic (add/multiply only).
4. For each missing input at `p`: contribute zero, set the degraded flag on the output slice stamped `p` (C-122); when the transport input is gapped, hold its state and advance its playhead only while playing (C-141, C-200).
5. Provide an offline function `mix_offline(inputs, volume) -> output` using the same code path, for diamond comparisons (AC-10).
6. Header comment citing POC `src/mixer.rs` and noting the retired on-arrival behaviour.

## Acceptance criteria
1. Two inputs with known content and different arrival delays (0 ms and 3 ms): output for every `p` equals `mix_offline` bit for bit.
2. Dropping input B at position `p`: output `p` equals input A × volume and carries the degraded flag; neighbouring slices are not degraded.
3. Output for position `p` is emitted at `T(p) + offset`, never on arrival of an input. An input slice arriving after the firing time but within the ring span is not mixed: `p` is degraded, and the input's counters show missing then reclassified late (C-108).
4. Volume change at position `p` affects output from `p` exactly.
5. A two-packet input slice whose second packet arrives before the mixer's deadline is mixed; after the deadline it is gapped (C-110).
6. No libm call in `mixer.rs`.
7. With an explicit budget configured, the mixer's ceiling and ring size equal that budget; with no downstream presentation group and no budget, its aligned inputs are refused with reason NoSkewBudget (C-142, C-185).
8. The volume is declared in the manifest as a dense controller (C-192); a gap before any received volume value applies the manifest's initial value (C-158).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- Interrupted/dead classification, ratchet and realignment logic themselves (epic 009); their L0 assertions (017b-14, 017b-15).
- The sender's replica of the mixer (017a-09).
- Following a change of the inherited A (017a-13).
