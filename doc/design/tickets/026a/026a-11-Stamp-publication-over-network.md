# 026a-11 — Stamp publication over network
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-04, 026a-05, epic 013

## Context

The regime stamp is a **wire object** (epic 013). A device has **no files**, so it **publishes its stamp over the network** and a hosted collector records it `[decision]`. Deliverable: `stamp_publish.rs`.

Settled transport (C-18): the stamp travels in a **dedicated control message split into sections that each fit C-17** (at most 1 252 bytes of UDP payload). It is **multicast to the control group on any regime change and refreshed every 10 s**; a **unicast request gives an immediate fetch**. The hello carries only a **stamp version number**. IPv4 only, multicast with TTL 1 (C-100); control class CS5 (C-13).

Facts:
- The stamp includes the embedded profile from `profile.rs` (026a-04): profile "embedded, by construction", bounded/report-only control handling, reset-on-fault, capability fields in the four C-129 states, host-side replay, declared memory limit, no files, holder_eligible = false, receive-timestamp source (C-131), declared jitter, D with method, E (C-73, or unknown per C-103). Later tickets add per-connection counters (026b-11), the calibration factor (026b-19), the commissioning-window state (026a-13), and lateness percentiles, reclaimable latency and peer DSCP observations (026b-12). Real-time scheduling and memory lock are stamped present (C-147); control isolation per C-161.
- Epic criterion 026-AC-1 (AC-35, part): its stamp is published over the network and recorded by the hosted collector.
- Epic criterion 026-AC-10: the device's D (with method) and E are in its stamp.

## Work

1. `stamp_publish.rs`: build the stamp object (epic 013 encoding) from `EmbeddedProfile` + runtime values (identity, nonce, lock state, D, E, worst-case poll time) and encode it into fixed-size sections, each ≤ 1 252 bytes, using epic 013's section format; no allocation.
2. Bump the stamp version number on any field change and expose it to the hello (026a-05).
3. Multicast all sections to the control group on boot, on every version change, and every 10 s (C-18).
4. Answer a unicast stamp request (epic 013's request message) with all sections, unicast to the requester.

## Acceptance criteria

1. Host test: the device multicasts its stamp within 1 s of boot and then every 10 s ± 1 %.
2. Host test: changing any stamp field bumps the version number in the next hello and triggers an immediate multicast.
3. Host test: a stamp request is answered within 1 poll cycle, unicast, with sections equal to the latest multicast.
4. Every encoded section ≤ 1 252 bytes in the worst case (all optional fields present); asserted in a test.
5. Decoding the published sections with the epic-013 decoder yields every profile field, including D, D method and E.
6. Stamp request parser fuzzed at the C-95 budget.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] The table "what weakens on no_std" is reflected field by field in the device's stamp
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- The hosted collector (026a-12).
- Measuring D and E values (026b-14).
- Content of the per-connection counters, calibration factor, commissioning window and alignment figures (026b-11, 026b-19, 026a-13, 026b-12).
