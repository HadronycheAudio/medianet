# 025b-05 — Replay acceptance: DSL echo under injected loss
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** S · **Depends on:** 025a-18, 025b-04, epic 017
## Context
Acceptance criterion 7 (replay): **a one-minute L0 recording of the DSL echo under injected loss replays identically outside excluded windows, with the excluded-window count reported; a deliberately corrupted recording is reported at the correct slice.**

- DSL echo: `hadronyche-reference/dsl/echo.hdsl` (025a-18).
- Recorder: `hadronyche-std/src/record.rs` (025b-02); replay: `hadronyche-replay` (025b-03/025b-04).
- L0 harness: epic 017.
- `[decision]` Windows of 1 024 samples touched by loss or by a degraded slice are excluded from exact comparison and counted.
- The degraded flag follows the content: an output at p is degraded if its input at p − L was gapped (C-122).
- One minute at slice 48 frames = 0.5 ms (C-01, C-02) is 120 000 slices.
- The expected clean-network loss floor (about one loss per stream per link every 13 hours, C-90) is far too low to produce loss in one minute, so loss must be injected.

Still open, owned by epic 017: the injected-loss rate and pattern of the L0 harness. If it is not defined when this ticket starts, use the L0 harness's loss injection with a rate and pattern chosen here, state them in the test, and ask epic 017 to confirm.
## Work
1. A CI test that runs the L0 harness with the DSL echo (float and fixed), injects loss at a documented rate and pattern, records one minute, and replays it.
2. Compute the expected excluded-window count from the injected gaps and the echo's L (C-122).
3. Corrupt one recorded output hash at a chosen slice outside excluded windows and replay again.
## Acceptance criteria
1. The uncorrupted replay returns identical, with a non-zero excluded-window count equal to the count computed from the injected loss.
2. The corrupted replay reports exactly the corrupted slice and output.
3. Both float and fixed variants pass.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Cross-target replay (025b-06).
