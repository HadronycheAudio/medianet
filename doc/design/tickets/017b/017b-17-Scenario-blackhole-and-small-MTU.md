# 017b-17 — Scenario `blackhole` and forced small path MTU
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-03, 017a-09, 017b-09, epic 010, epic 012

## Context
- AC-19 (L0 part): with `drop_above_bytes` set, the **probe pair detects at admission** and the connection **goes live at a 1 280-byte path MTU**.
- Settled rules:
  - IPv4 only (C-100). The 1 280-byte fallback (C-06) is a conservative path MTU giving 1 252 bytes of UDP payload (C-17, C-100). `drop_above_bytes` compares the IPv4 packet size (017b-03).
  - **Probe size (C-116):** the probe pair is exempt from the control message bound and is sent at the intended media packet size, so a blackhole below that size drops it.
  - The default slice fits one packet at the path MTU; a multi-packet slice is an explicit, stamped regime (C-05).
  - **Header cap (C-137):** a default slice fits 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum; in-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path. **Controllers (C-181, C-205):** a single-stream source's controllers share the audio packet only while headers plus controllers stay within that 100-byte cap; beyond it the source emits a second stream for them, and a bundle source's controllers always ride in their own member stream.
  - **PathMtuTooSmall (C-173):** applies when the probe at the 1 280-byte fallback also fails. A connection alarm is raised on refusal.
  - Incomplete reassemblies are a per-connection counter in the stamp (C-159).
  - **Reassembly deadlines:** an arrival-driven multi-packet input (the echo) has 2 slice periods (C-15); an aligned one (the mixer) has the consumer's own deadline for that position (C-110).
- Rule `[decision]`: loopback's MTU is 65 536, so it never exercises small packets by itself; L0 must **force a small configured path MTU** to exercise the multi-packet regime and the fallback.
- Slice = 48 frames at 96 kHz (C-01, C-02), 6 channels per stream, 1 152 bytes of audio, no per-channel id byte (C-04).
- Admission and connection states: epic 012; packetiser: epic 010.

## Work
1. Scenario `blackhole`: `set_link(sender, mixer, { drop_above_bytes: Some(1280), .. })` with the loopback path MTU otherwise configured; assert the probe pair (at media packet size, C-116) is lost during admission, the blackhole is detected, and the connection becomes live at a 1 280-byte path MTU; then run content checks.
2. A second run forces a configured path MTU smaller than one default slice packet, so every slice of both the parts and the audio streams is split across several packets; assert the multi-packet regime is stamped (C-05), exact-tier mismatches = 0, and reassembly respects C-15 at the echo and C-110 at the mixer.
3. A third run sets `drop_above_bytes` below 1 280 bytes, so the probe at the fallback also fails; assert the connection is refused with PathMtuTooSmall and a connection alarm is raised (C-173).

## Acceptance criteria
1. `hadronyche-l0 blackhole --duration 30` exits 0.
2. Admission log shows probe detection before the connection goes live; after admission no IPv4 packet exceeds 1 280 bytes and no UDP payload exceeds 1 252 bytes, at most 100 of them headers, any controllers sharing the packet, and checksum (C-137, C-181).
3. In the forced small-MTU run, the count of slices sent as more than one packet is > 0, the stamp records the multi-packet regime, and exact mismatches = 0; the incomplete-reassembly counter (C-159) equals the number of slices with a packet withheld past its deadline (zero without injected loss).
4. With `drop_above_bytes` below 1 280, the connection is refused with PathMtuTooSmall, never goes live, and one connection alarm is raised (C-173).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- AC-19 parts that need real networks (L1/L2, epics 019, 020).
