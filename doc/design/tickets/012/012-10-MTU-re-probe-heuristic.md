# 012-10 — MTU re-probe heuristic
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-09

## Context
Epic 012 rule `[decision]`: re-probe the path MTU when **large packets vanish while liveness holds**. Open sub-question owned by this task (L1, task 019, confirms): **re-probe heuristic thresholds**. Falsifier: a blackhole persists undetected, or re-probes fire on ordinary loss.

Related constants: Interrupted after 8 consecutive missed slices (4 ms, C-38), where only slices absent at the deadline count and a marked gap resets the count (C-102); Dead after 2 s without data (C-39), counted from the last slice with content, a marked gap or a slice on another timeline epoch (C-111, C-135); receiver report one per bundle every 100 ms (C-40). Path-MTU fallback 1 280 bytes (C-06). At defaults a slice fits 1 252 bytes of UDP payload (C-137), so a default media packet is at most 1 280 bytes at the IP layer (about 1 320 bytes on the wire with Ethernet overhead, C-20) and is not "large". Packets above 1 280 bytes appear only on connections whose slice size is above the default; only those connections need the detector, and their liveness comes from control traffic and packets at or below 1 280 bytes on the same path.

Expected clean-network loss is about 1.1e-8 per default packet (C-90, pending measurement), one loss per stream per link about every 13 hours; the loss alarm threshold comes from L1 (C-91). The "ordinary loss" level the heuristic must ignore is set against these.

The epic now lists `DataPlaneEvidence::LargePacketsVanished` (012-01); this ticket produces it.

## Work
- Propose thresholds, written in the constants table as pending-measurement with owner this task and L1 (task 019), and the falsifier above. Suggested form: re-probe when, over a window of N receiver reports (C-40, 100 ms each), packets above 1 280 bytes show loss ≥ X % while packets at or below 1 280 bytes from the same peer, including control messages, show loss ≤ Y %.
- `pub struct ReprobeDetector` (pure, `no_std`): fed per-report counters `{ large_sent, large_recv, small_sent, small_recv }`; returns `true` when the thresholds fire.
- The engine turns a `true` into `DataPlaneEvidence::LargePacketsVanished`, which restarts `MtuProbe` for that connection without leaving Live (012-15).

## Acceptance criteria
1. Blackhole injection (100 % loss above 1 280 bytes, 0 % below) fires within the proposed window.
2. Uniform random loss at the proposed "ordinary loss" level, far above the C-90 floor, on both sizes never fires over 10 000 simulated reports.
3. With liveness lost (small packets also gone), it does not fire (Interrupted/Dead handling owns that case).
4. On a connection at default size (no packet above 1 280 bytes, C-137) it never fires.
5. Thresholds, owner and falsifier are in the constants table and the epic open sub-questions.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- The probe exchange (012-09). Engine wiring of the evidence (012-15). Real-network confirmation (task 019).
