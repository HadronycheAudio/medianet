# 026b-01 — Reference board selection
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** XS · **Depends on:** none

## Context

Epic 026 brings the Hadronyche runtime to a `no_std` microcontroller and builds one **reference device**: firmware for one named board with wired Ethernet and an audio codec, acting as a sink that joins a medianet as a **follower** `[decision]`. The epic leaves the proposal of a board to this ticket; every firmware ticket (026b-02 onwards) needs it. Reference hardware is proposed by the implementing team, approved by the product owner and recorded in the constants table (C-172).

Facts that constrain the choice:

- Wired Ethernet port, 100 Mbit is the sizing basis: about 19 channels at the 70 % admission ceiling (C-26, C-23). IPv4 only, multicast with TTL 1 (C-100).
- An audio codec driven by DMA, with a timer able to capture the DMA position (loop 2 of the two-loop timebase uses (frame count, timer-captured time) pairs). 2 or 6 outputs: the device receives a whole 6-channel stream (C-04) and plays the channels its codec has.
- Device audio clock 48 kHz; the wire stays at 96 kHz and the device converts 2:1 (C-80, C-01).
- Receive timestamps come from MAC hardware timestamping if the part has it, else from the receive interrupt; the source is reported with a C-131 value (mac-hardware, interrupt, or userspace for a clock read after receive).
- A hardware unique identifier and persistent storage for a boot counter are required (identity rule, C-11); a true random source is optional.
- A hardware watchdog is required: on the device any fault is a whole-device reset `[decision]`.
- Static memory must hold the 63-entry peer map (C-217, C-34), one 6-channel stream at A = 10 ms (ring of 6 × 4 bytes × 96 000 × 0.010 = 23 040 bytes, sized to A at the domain rate, C-96, C-01), the servo and the resampler, with at least 25 % headroom (026-AC-2, verified in 026b-04).
- Open sub-question owned by this epic: can the device's network stack set DSCP and the IPv4 don't-fragment bit, and read the DSCP of received packets (needed for the observed-DSCP counter, C-47)? If not, those are stamped absent.

## Work

1. Shortlist candidate boards meeting the constraints above.
2. Propose one; record a short decision note in `doc/design/reports/` (C-130), to be quoted in `docs/device-porting.md` (ticket 026b-21):
   - board name and MCU part;
   - Ethernet MAC/PHY and whether the MAC supports hardware receive timestamping (expected `StampSource`);
   - audio codec, its output count, its interface (I2S/SAI or similar), DMA and timer-capture capability;
   - oscillator part and its stated tolerance;
   - RAM and flash sizes;
   - whether the core has a double-precision FPU (affects the `f64` cost question of 026b-05);
   - hardware unique id size, persistent storage for the boot counter, presence of a true RNG;
   - watchdog;
   - whether the intended network stack can set DSCP and the IPv4 don't-fragment bit, and read received DSCP (answer "yes", "no" or "to be verified in 026b-02").
3. Record a first rough memory estimate against the 026-AC-2 workload so an unsuitable part is rejected early.
4. Submit the proposal to the product owner for approval (C-172); on approval, 026b-22 records the board in the constants table.

## Acceptance criteria

1. The decision note names exactly one board, its MCU, its timestamp source (MAC hardware or interrupt) and its oscillator part.
2. The note lists RAM size and a rough static-memory estimate for: 63-entry peer map (C-217), a 23 040-byte ring, servo and resampler; the estimate leaves at least 25 % headroom.
3. The note states hardware unique id, boot-counter storage, true-RNG presence, double-precision FPU presence and watchdog presence.
4. The note records the DSCP / don't-fragment / received-DSCP answers or explicitly defers them to 026b-02.
5. The product owner's approval of the board is recorded in the note (C-172).
6. Every factual claim in the note carries an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[standard]` or `[decision]`); datasheet facts that are not measured are tagged `[memory]` until verified (C-132).

## Definition of done

- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The reference board, its timestamp source and its oscillator are named in the docs
- [ ] Board purchased / available to the team (at least two units, needed for the identity test 026b-10)

## Out of scope

- Measuring timestamp jitter (026b-07), memory fit (026b-04), CPU budget (026b-05).
- Writing the porting guide (026b-21); this ticket only supplies its board section.
- Any firmware.
