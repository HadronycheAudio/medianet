# 015a-11 — Data thread receive side (bounded drain, filter, demux, ticker)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-06, 015a-10, epic 009, epic 010, epic 012
## Context
- File: `hadronyche-std/src/data_thread.rs`. One data thread per module `[decision]` (the POC's three threads per module are retired). Loop: bounded drain → core alignment ring → module process callback → paced send → receiver reports. This ticket covers the receive half and the ticker.
- **Bounded drain (C-16):** per input per cycle, at most 2 × the admitted packets per slice for that input; the cycle is one slice period (C-156). This is also the `recv_batch` `max`: one call per input per slice period.
- **Source filter:** only packets from the source address pinned at admission are parsed; others are counted and dropped.
- **One socket per bundle**, demultiplexed by the 16-bit stream number. A bundle is a group of aligned narrow streams from one source (6 channels each at defaults, C-04; at most 32 members, the controller member included, C-08, C-209; a bundle source's controllers ride in their own member stream, C-205, demultiplexed like any other member). A default slice is 1 152 bytes of audio (192 bytes per channel, no per-channel id byte, C-04).
- **Socket buffer:** declared rate × A, where A is the presentation group's audition offset (skew budget cap, C-96; no default, C-55). A fan-in outside any presentation group uses the A of the downstream group it feeds (the largest if several) or its configured budget (C-142); with neither, admission refuses its aligned inputs with NoSkewBudget (C-185, core epic 012), so this ticket never sizes a buffer without an A. The A of the downstream group reaches the fan-in in its consumers' receiver reports (C-198); a later change and the re-alignment it causes (C-186) are 015a-24. Computed here at admission and passed to `open_media` (015a-04). The alignment ring is also sized to A (C-96, epic 009).
- **Ticker on network time** (from the servo's published seqlock view, epic 008), not local monotonic time. Position: `position = (network_time − epoch_origin) × 96 000`; the wire carries the domain rate only (C-01).
- **L-5:** destinations and sources are pinned at admission and change only by re-admission.
- POC defects `[code]`: `src/module.rs:1669` drained every input completely before checking the deadline (one flooding input starved the ticker); `:1630` used `mono_ns()` as ticker epoch; `:1639` `routes = r.clone()` allocated on the data thread.
- No allocation on the data thread after go-live.
## Work
1. Per input: call `recv_batch` with `max = 2 × admitted packets per slice`, once per slice period (C-156); never loop until empty.
2. Drop and count packets whose `from` differs from the pinned source before parsing.
3. Demultiplex by 16-bit stream number into the core reassembler and alignment ring (epics 009, 010).
4. Ticker: read the network-time view from the seqlock; compute the next slice deadline in network time; convert to host time for waiting.
5. At admission: compute `rcvbuf = declared rate × A` and open the bundle socket with it; count a kernel clamp.
6. Receive admitted input sets from housekeeping via a bounded lock-free queue; apply them by swapping preallocated tables (no clone, no allocation).
## Acceptance criteria
1. With one input receiving 10× its admitted packets per slice, at most 2 × admitted packets are read from it per slice period (counter test).
2. Packets from a non-pinned source are never passed to the parser and increment a "foreign source" counter.
3. Packets for stream numbers outside the admitted bundle members are counted and dropped.
4. The ticker deadline follows the published network time: shifting the servo view by +1 ms shifts the next deadline by +1 ms.
5. The bundle socket is requested with declared rate × A for the admitted A.
6. No heap allocation per cycle after go-live (counting allocator unit test).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — packet parsing is core's (epics 009, 010); "Regime stamp fields touched" — counters reach the stamp in 015a-14 and 015a-19; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — measured in 015b-10.
## Out of scope
- Send side and process callback (015a-12); liveness and receiver reports (015a-13).
- Full flood-isolation test (015b-03).
