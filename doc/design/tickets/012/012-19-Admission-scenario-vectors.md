# 012-19 — Admission scenario vectors
**Epic:** 012-Admission-handshake · **Effort:** M · **Depends on:** 012-14, 012-17, 012-18, 012-24, epic 004, epic 005, epic 007

## Context
Epic 012 criterion 012-A1 (system criterion AC-1): all admission and connection scenario vectors pass. Vectors use the epic 007 format. The engine is `Admission<CONNS>` with `request`, `cancel`, `on_message`, `on_peer_map`, `on_tick`, `on_data_plane`, driven with explicit `now_ns`, so scenarios are deterministic and run unprivileged. This ticket builds the two-engine harness and the admission-phase vectors; the life-cycle vectors after Live are 012-25, which reuses the harness. The harness is test code inside `hadronyche-core`'s tests; if it grows into a binary it goes in the `hadronyche-tools` crate (C-133). The vector data lives in the vectors crate at the workspace root (C-192).

Sources of the conformance vectors (C-193) `[decision]`: epic 005 supplies the state tables and scenario scripts for admission and connection, and epic 004 the wire layouts and reject reasons. This ticket runs epic 005's admission scripts on the harness; the vectors below are added where a script does not already cover them.

Key numbers `[decision]`: 70 % ceiling (C-23); 96 ch source 16 streams, 338 Mbit/s, 32 000 packets/s (C-22); two 96 ch bundles on one GbE port are 676 Mbit/s and 64 000 packets/s and a third is refused (C-92); MTU fallback 1 280 bytes (C-06), 1 252 bytes of UDP payload over IPv4 (C-100); the probe is sent at the intended media packet size (C-116), and `PathMtuTooSmall` applies when the 1 280-byte probe also fails (C-173); Pending has no timeout (C-45); sessions are source-assigned, one per stream (C-19); a sink with unknown E is refused from a group that sets a required E (C-103). Test-only hooks available beyond the public injection API: force a module id and nonce, force a stream onto a given timeline epoch, send a raw packet on a media or control socket (C-120).

## Work
Write the harness (two engines, requester and consumer, connected by an in-memory link with fault injection, including size-dependent drop), run every admission scenario script of epic 005 on it (C-193), and add vectors for at least:
1. 012-A2 (AC-12): GbE port; two 96-channel bundles (338 Mbit/s each, 676 Mbit/s total) admitted, the third refused with `PortCeiling`; graph view shows it unsatisfied; no retry over 60 s.
2. 012-A4 (AC-30): sink E > group required E → `RequiredEExceeded`, never Live; sink with unknown E → refused from a group that sets a required E, admitted to a group that sets none (C-103).
3. 012-A5 (AC-19): size-dependent drop injected → probe fails, admission at 1 280, path MTU exposed in the stamp record; then large packets vanish while liveness holds → re-probe; with drop at every size → `PathMtuTooSmall` (C-173).
4. 012-A10: Pending with no timeout over 600 s, then the target appears → Requested → Admitted; a second Pending request is cancelled and disappears from the graph view (C-45).
5. 012-A14 (C-19): two consumers of one stream record the same session from the request.
6. Full happy path Requested → Admitted → Locking → Aligning → Live → Ended; the graph view shows the connection satisfied only while Live (C-154).
7. C-173: a peer whose hello carries domain 0 is absent from the peer map, so a request to it stays Pending and no refusal is sent; a peer in a different non-zero domain is refused with `ForeignDomain`.

## Acceptance criteria
1. Every admission scenario script supplied by epic 005 (C-193) runs on the harness and passes, and all vectors listed above exist in the epic 007 format, in the vectors crate (C-192), and pass in CI.
2. Each vector asserts the exact sequence of `ConnEvent`s and outbox messages.
3. The run is deterministic (two runs produce identical traces).
4. The harness is reusable by 012-20 and 012-25 without copying.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser; vectors feed the 012-21 corpus
- [ ] Regime stamp fields touched are complete and documented — not applicable: the path-MTU field is read, not defined, here
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: tests only
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable here: covered by 012-25
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Life-cycle vectors after Live (012-25). Coverage audit of every `RefuseReason` (012-20). Real-network tests (task 019). Device run (epic 026).
