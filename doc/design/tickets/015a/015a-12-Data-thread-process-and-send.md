# 015a-12 — Data thread process callback and paced send
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-09, 015a-11
## Context
- File: `hadronyche-std/src/data_thread.rs`. Loop: bounded drain → core alignment ring → module process callback → paced send → receiver reports. This ticket covers the process callback and paced send.
- The runtime owns *when and whether* content moves; module code owns *what* it is.
- Send rules `[decision]`: don't-fragment on every packet; by default a slice is one packet (C-05) at the path MTU; fallback path MTU 1 280 bytes, 1 252 bytes of UDP payload (C-06, C-100). Media DSCP EF (46, C-13). Packets of one slice are spread over at most 25 % of the period (125 µs at defaults, C-14) plus a per-source phase offset; per-output override 0 disables spreading.
- A default packet carries 6 channels × 192 bytes = 1 152 bytes of audio (C-04) and is about 1 320 bytes on the wire (C-20). Header cap (C-137): a default slice fits 1 252 bytes of UDP payload, 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum; in-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path. The packetiser enforces this (epic 010); this ticket checks it on what leaves.
- The per-output pacing override is operator policy from the commissioning configuration (C-165).
- When a module misses its own deadline, the hosted backend emits the gap slice and raises the processing-overrun event (C-144). This ticket counts missed deadlines; the gap slice and the event are 015a-20.
- The path MTU used by the packetiser is the one core selected after the probe pair (C-116); a `MessageTooBig` from `send_to` is counted and reported, never fragmented.
- **L-5:** destinations are pinned at admission and change only by re-admission; a hello carrying a new address never redirects a live stream.
- Packetiser and schedule come from core (epic 010).
## Work
1. After alignment, invoke the module process callback once per slice with the aligned inputs and preallocated output buffers.
2. Packetise outputs with the core packetiser at the path MTU selected for the connection.
3. Send through the `Pacer` (015a-09) to the destinations pinned at admission only.
4. Count missed deadlines (callback + send not complete before next slice deadline) and `MessageTooBig` send errors.
## Acceptance criteria
1. One process callback per slice period over 10 000 slices on loopback.
2. Every output packet is sent to the pinned destination; changing the address in housekeeping's discovery map without re-admission leaves the destination unchanged.
3. No output packet exceeds the selected path MTU; at the fallback, no UDP payload exceeds 1 252 bytes, and the headers of a default slice take at most 100 bytes (C-137).
4. A missed-deadline counter exists and is 0 in a nominal loopback run of 60 s.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none here; "Design documents updated" — no behaviour differs; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — send side.
## Out of scope
- Receiver reports and liveness (015a-13).
- Gap slice and processing-overrun event on a missed deadline (015a-20).
- The path-MTU decision procedure (core, epics 012 and 014).
- DF capture verification (015b-07).
