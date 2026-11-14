# 013-07 — Network and connection sections
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-01, 013-02, 013-03

## Context
- Stamp group **Network**: interface, interface MTU, path MTU per connection, packets per slice, pacing mode, DSCP classes and peer-observed values, interrupt-coalescing state. `[decision]` All are regime fields (they change the `StampId`, 013-05).
- IP is **IPv4 only** in v1 (C-100): the interface is identified by name and IPv4 address; there is no IPv6 field. The path-MTU fallback is 1 280 bytes (C-06), giving 1 252 bytes of UDP payload.
- Default slice size fits one packet at the path MTU; multi-packet slices are an explicit, stamped regime (C-05). `packets_per_slice` is 1 at defaults.
- Builder signature from the epic: `pub fn network(&mut self, ..);` and `pub fn connection(&mut self, conn: ConnId, path_mtu: u16, packets_per_slice: u8, pacing: PacingMode);`
- A connection is named by its stream (source module id 64 bits, stream number 16 bits, session 32 bits; C-10, C-11, C-19, the session being the same for every consumer) and its peer module id. `ConnId` and `PacingMode` are owned by epic 004; until it defines them, use a placeholder alias and note it.
- Rule 8: **DSCP marking is verified by the receiving peer**, which reads the received byte and reports it; the stamp records the classes requested and the values the peer observed. No platform is trusted to self-report. Requested classes (C-13): **sync CS6 (48), media and one-way probes EF (46), control CS5 (40)**. `[decision]` Epic 014 delivers the peer-observed values into the builder. Separately, each consumer publishes the DSCP it observed per connection in its own per-connection summary (C-47, ticket 013-23), and the per-peer sync and control observations of C-152 are carried by 013-26.
- 013-AC-8 (AC-34, stamp part): the stamp shows requested DSCP classes and the peer-observed values, or "unverified".
- Hosted platforms: NIC interrupt coalescing biases software receive stamps; commissioning disables it on medianet interfaces and the stamp records its state. Which coalescing states can be read unprivileged per OS is an open sub-question owned by tasks 015, 021, 022 (falsifier: not readable; the field becomes "unverified" there and a commissioning checklist item).
- A path-MTU fallback is a regime change: it must change the `StampId`.
- No constant bounds the number of connection entries in one stamp (a source has at most 256 streams, C-09, and a medianet at most 64 modules, C-34). The builder writes as many as the caller buffer holds; a section longer than one control message is split by the rule of 013-13 (C-18).

## Work
1. Define the Network section: interface name and IPv4 address, interface MTU, pacing mode default, interrupt-coalescing state as a C-129 capability state (present, absent, unverified, not requested), and for each of the three traffic classes (sync, media/probes, control) the requested DSCP value and the peer-observed value or "unverified".
2. Define a per-connection entry: `ConnId`, `path_mtu: u16`, `packets_per_slice: u8`, `PacingMode`, repeatable.
3. Implement `StampBuilder::network`, `StampBuilder::connection` and the readers on `Stamp` (iterator over connections without allocation).
4. Extend the fuzz corpus; add field tables to `doc/design/spec/stamp.md` with the source of each field on each of the four profiles.

## Acceptance criteria
1. Round-trip of a stamp with the three requested classes 48, 46, 40 and observed values 48, 46, 40 returns them exactly.
2. A stamp with no peer observation for a class reads back as "unverified" for that class, not as 0.
3. Round-trip of 0, 1, 32 (one full bundle, C-08) and 64 connection entries; per-entry `path_mtu`, `packets_per_slice` and pacing mode are preserved.
4. Lowering `path_mtu` on one connection from 1 500 to 1 280 (C-06) changes `Stamp::id()`; changing an observed DSCP value changes it.
5. An IPv4 address round-trips; the section has no IPv6 form (C-100).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-06, C-13, C-100 unchanged); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- The peer reading and reporting the received DSCP byte, and feeding it into the stamp at run time (epic 014).
- The per-connection summary counters (013-23); per-peer observed sync and hello DSCP (C-152, 013-26).
- Reading coalescing state on each OS (tasks 015, 021, 022).
- Splitting an oversize connection section (013-13).
