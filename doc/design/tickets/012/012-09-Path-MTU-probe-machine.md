# 012-09 — Path MTU probe machine
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-01, 012-08

## Context
Path-MTU probe pair `[decision]`: at admission the two ends exchange a padded don't-fragment packet at the intended media packet size and an acknowledgement. The probe is exempt from the control message bound C-17 (C-116). On a pure layer-2 path a too-small hop drops silently and sends no error back, so **silence is the answer only because an acknowledgement is expected**. On failure, probe again at the 1 280-byte fallback (C-06), which leaves 1 252 bytes of UDP payload over IPv4 (C-100); a slice that no longer fits one packet then needs the explicit, stamped multi-packet regime (C-05). If the probe at 1 280 bytes also fails, the request is refused with `PathMtuTooSmall` (C-173). At defaults a slice fits 1 252 bytes of UDP payload (C-137), so the default intended packet is already at the fallback size and a single probe decides. The path MTU of each connection goes in the regime stamp, a dedicated sectioned control message (C-18); the record that carries it is 012-23. The intended size is `Request.intended_packet_bytes: u16`; the result is `Accept.path_mtu: u16`.

Still open in the epic:
- Probe timeout value: owner this ticket proposes, L1 (task 019) confirms; falsifier: probes on a healthy path time out, or a failed probe delays admission beyond what operators accept. This ticket introduces `MTU_PROBE_TIMEOUT_NS` as a configuration parameter with a proposed initial value, written in the constants table as a pending-measurement proposal; it is not a settled constant.

Settled: `PathMtuTooSmall` applies when the probe at the 1 280-byte fallback also fails (C-173).

## Work
- `pub struct MtuProbe` in `hadronyche-core`: pure state machine, no I/O, no allocation.
  - `start(now_ns, intended: u16) -> ProbeSend` (caller encodes and sends a padded DF probe of `intended` bytes via the 012-08 codec);
  - `on_ack(now_ns, size: u16) -> MtuOutcome`;
  - `on_tick(now_ns) -> Option<ProbeSend | MtuOutcome>`: when the timeout elapses without an ack for a probe above 1 280 bytes, ask the caller to send a second probe at 1 280 bytes; when the timeout elapses without an ack for a probe at or below 1 280 bytes, return `TooSmall`;
  - `MtuOutcome::{Confirmed(u16), Fallback(u16), TooSmall}`, where `Fallback(1280)` means the 1 280-byte probe was acknowledged and `TooSmall` maps to `PathMtuTooSmall` (C-173); an ack for a different size or after the outcome is ignored.
- Expose `path_mtu()` for the stamp record (012-23); document the field (name, unit, meaning) next to the type.

## Acceptance criteria
1. Ack received before timeout → `Confirmed(intended)`.
2. No ack for an intended size above 1 280 (size-dependent drop injected) → a 1 280-byte probe is requested exactly when `now ≥ start + MTU_PROBE_TIMEOUT_NS`, not before; its ack gives `Fallback(1280)`.
3. A late ack after fallback does not change the outcome.
4. No ack for the 1 280-byte probe, or for a first probe at or below 1 280 bytes, → `TooSmall` one timeout after that probe (C-173). The reading that a first probe at or below 1 280 bytes counts as the fallback probe is recorded in the module docs.
5. The path-MTU stamp field is documented.
6. The proposed timeout appears in the constants table as pending-measurement with owner L1 (task 019) and the falsifier above.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser; the probe codec is 012-08
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no connection transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Re-probe when large packets vanish (012-10). Driving the probe from the engine (012-12, 012-13, 012-14). Socket DF flags (platform epics). The multi-packet regime (epic 010). Encoding the `PathMtuTooSmall` refusal (012-13, 012-14).
