# 026b-17 — Fault reset and rejoin
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03, 026b-02

## Context

On hosted machines housekeeping is supervised and restartable. On the device this is **unsatisfiable: any fault is a whole-device reset (watchdog)** `[decision]`. Fault injection may be absent on a device; a test-only hook is stamped like any injection (C-120), with the C-129 capability states. C-145 adds a test-only hook that injects a panic into a data, clock or housekeeping thread; the device has no threads, so the hook injects the panic inside control handling in `poll`.

Liveness: interrupted after 8 consecutive missed slices (C-38), 4 ms, counting only slices absent at the deadline (C-102); dead after 2 s (C-39) from the last slice with content or a marked gap (C-111); producer stops after 2 s without receiver reports (C-41); hello expiry 3 s for non-live peers (C-31). Identity: a reboot keeps the module id and draws a new instance nonce; third parties quarantine the id until only the new nonce has been seen for 3 s (C-11, C-46; 026a-03).

Epic criterion 026-AC-8 (Fault = reset): an injected **control-path fault** resets the device; peers classify it **interrupted within 4 ms** and **dead at 2 s**; it **rejoins with the same module id and a new nonce**.

Settled (C-168): the liveness of a sink-only device is judged by the producer timeout (C-41); interrupted and dead (C-38, C-39) describe streams a peer receives. The reference device sends no stream, so what peers observe of its reset is the producer timeout and the nonce change (C-46); 026-AC-8 is stated in those terms.

## Work

1. Add the C-145 panic-injection hook on the control path (triggered, for example, by a raw control packet sent with the C-120 raw-packet hook, or by a GPIO). Exclude it from release builds; any build that contains it stamps `fault_injection = Present` (C-120, C-129).
2. Bench: board receiving a stream from a Linux producer; trigger the fault.
3. Observe the producer/peers' timeline and the board's rejoin. Store the log in `doc/design/reports/` (C-130).

## Acceptance criteria

1. The fault causes a watchdog or panic reset (reset cause register shows it).
2. The producer stops sending to the board 2 s ± 100 ms after the last receiver report it received (C-41).
3. The board rejoins with the same module id and a different nonce; peers accept the id once only the new nonce has been seen for 3 s (C-46).
4. Repeated 10 times with identical outcomes.
5. The stamp's `fault_injection` field is `Present` in the build with the hook and not `Present` in the release build.
6. The result log states that the device's liveness was judged by the producer timeout, citing C-168.

## Definition of done

- [ ] Regime stamp fields touched are complete and documented
- [ ] The table "what weakens on no_std" is reflected field by field in the device's stamp
- [ ] Every new factual claim carries an evidence tag

## Out of scope

- Cable-pull liveness (026b-16).
