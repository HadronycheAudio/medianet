# 015b-10 — Per-packet receive cost measurement
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-11, 015b-13, epic 016
## Context
- Definition of done item: per-packet receive cost measured on the development machine and recorded for task 019.
- Open sub-question owned by task 019 (L1): receive cost per packet guessed 2 to 4 µs, which is 13 to 26 % of the slice period for two bundles (C-93) `[memory]`; falsifier: two 96-channel bundles miss deadlines on one GbE port.
- Two-bundle load (C-92): 2 × 96 channels = 32 streams, 32 packets per 0.5 ms slice, 64 000 packets/s, 676 Mbit/s. A default packet carries 1 152 bytes of audio (6 channels × 192 bytes, no per-channel id byte, C-04) and is about 1 320 bytes on the wire (C-20).
- Another open sub-question owned by task 019: does interrupt coalescing bias kernel receive stamps enough to matter (falsifier: servo offset shifts when coalescing is toggled). Not measured here.
- Results go to `doc/design/reports/`, where task 019 reads them (C-130). Any run whose numbers enter the constants table runs in strict mode (C-147), so the recorded benchmark runs in strict mode (015b-13) with the task 016 real-time setup bound.
## Work
1. Benchmark, in strict mode (C-147): `recv_batch` + source filter + demux + reassembly of 32 default-size packets per 0.5 ms slice on loopback; report ns per packet (p50, p99) and the share of the slice period for the two-bundle load.
2. Record results with machine description and `[measured]` tag in a report under `doc/design/reports/`.
## Acceptance criteria
1. A benchmark runs reproducibly and reports per-packet cost p50 and p99 with default-size packets.
2. Results recorded in `doc/design/reports/` and tagged `[measured]`, compared with the C-93 guess of 2–4 µs (13–26 % of the slice period); the report states that the run was in strict mode (C-147).
## Definition of done
- [ ] Per-packet receive cost measured on the development machine and recorded for task 019
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "No new open item" — none raised (the open items are task 019's); "Trait surface compiles with a stub backend" — no trait change.
## Out of scope
- GbE hardware falsifier, the two-bundle capacity test and coalescing bias (task 019).
