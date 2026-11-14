# 026b-02 — Firmware network, time and identity
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** M · **Depends on:** 026b-01, 026a-03, 026a-05, 026a-06, 026a-11

## Context

Deliverable: `examples/reference-device/` — firmware for one named board (chosen in 026b-01) with wired Ethernet and an audio codec, a sink for one 6-channel stream. This ticket brings up the non-audio half: `Transport`, `DeviceTime`, `Identity`, watchdog, and the main loop calling `Device::poll`.

Trait facts:
- `Transport::recv` is non-blocking; returns length, source and receive time on the device clock, with a C-131 source (`StampSource::{MacHardware, Interrupt, Userspace}`). Use MAC hardware timestamping if the part has it, else the receive interrupt.
- `Transport::send(to, class: DscpClass, packet)`; `join_discovery(group)`; `mtu()`. IPv4 only; multicast with TTL 1 (C-100). DSCP classes per C-13: sync CS6, media and probes EF, control CS5.
- The 1 280-byte fallback path MTU leaves 1 252 bytes of UDP payload for control messages (C-06, C-17, C-100); the path-MTU probe pair is sent at media size (C-116), so the stack must accept datagrams of media size.
- `DeviceTime::now_ns()` monotonic 64-bit; `declared_jitter_ns()`.
- `Identity`: `hardware_unique() -> [u8; 16]`, persistent `next_boot_counter() -> u64`, `true_random() -> Option<[u8;16]>`. Without a true random source, never derive id from a PRNG; mix unique id with boot counter (C-11). The module id is stable and the nonce changes per boot (C-46). `domain_id()` returns the stored domain id; a hello with domain 0 is ignored and counted by peers (C-173).
- Any fault is a whole-device reset via watchdog `[decision]`.
- Open sub-question owned here: can the device's network stack set DSCP and the IPv4 don't-fragment bit on send, and read DSCP on receive (for the observed-DSCP counter, C-47)? Each answer is stamped in a C-129 state; absent marking is shown by peer verification.
- Epic criterion 026-AC-11: no allocator and no libm symbol in the firmware image.

## Work

1. `examples/reference-device/` crate for the board's target: HAL init, Ethernet MAC/PHY, IPv4 network stack (no alloc), UDP sockets for control and media, multicast join with TTL 1.
2. `Transport` impl; rx timestamps from MAC hardware if available, else captured in the rx interrupt; set `Rx.source` accordingly.
3. Try to set DSCP and the IPv4 don't-fragment bit on send, and to read DSCP on receive; record each result in `EmbeddedProfile` (`dscp_marking`, `dont_fragment`, `rx_dscp_observation`) in its C-129 state.
4. `DeviceTime` from a free-running 64-bit extended hardware timer.
5. `Identity` from the MCU unique id, boot counter and domain id in flash/EEPROM (wear-aware), true RNG if present and enabled by a build flag (flag allows disabling it for 026b-10).
6. Watchdog enabled; fed once per main-loop iteration; panic handler resets.
7. Main loop: `loop { device.poll(); feed_watchdog(); }`.
8. CI: build the firmware image; run the 026a-01 symbol check on it.

## Acceptance criteria

1. Firmware image builds in CI for the board target; the symbol check finds no allocator and no libm symbol.
2. On the bench with one Linux holder and a non-zero domain id stored on the board, the board appears in the holder's peer map as a follower within 5 s of power-up, holder-eligible bit clear.
3. The board locks to the holder within the C-63 acquisition time (10 s, pending simulation), and the collector (026a-12) records its stamp, showing the C-131 timestamp source and the DSCP, don't-fragment and received-DSCP capability states.
4. A packet capture on the host shows multicast sent with TTL 1 and the C-13 DSCP class on each message type (or the marking stamped absent).
5. Boot counter increments on every power cycle (10 cycles observed); the stored domain id survives every cycle. With domain id 0 the board does not appear in the holder's map and the holder counts its ignored hellos (C-173).
6. A forced panic resets the board via the watchdog within the watchdog period.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Regime stamp fields touched are complete and documented
- [ ] The reference board, its timestamp source and its oscillator are named in the docs
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope

- Audio codec, DMA, timer capture (026b-03).
- Jitter measurement (026b-07), worst-case poll measurement (026b-06).
- Hardware acceptance tests (026b-08 to 026b-10, 026b-14 to 026b-18).
