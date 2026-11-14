# 029-02 — Stamp section reassembly and version tracking
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-01, epic 013, epic 015

## Context
- `hadronyche-monitor` (epic 029) is a read-only consumer of regime stamps and hellos. `[decision]`
- A regime stamp travels as a dedicated control message split into **sections that each fit the control message bound of 1 252 bytes of UDP payload** (C-17). It is **multicast to the control group on any regime change and refreshed every 10 s**; a unicast request gives an immediate fetch (C-18). Each hello (period 1 s, C-30) carries only a **stamp version number**.
- The monitor never sends the unicast request. It does not need to: every module's stamp reaches it within one refresh period (10 s) of starting.
- Open question in the epic, owned by epic 013 with epic 004: whether the version number changes only on a regime change or also when a refresh carries new summary counters. This ticket treats the version as an opaque, increasing number and does not decide what a regime is (029-06 does).

## Work
1. In `collect.rs` (or a submodule `collect/stamp.rs`), implement a pure, I/O-free `StampAssembler`:
   - `fn push_section(&mut self, module: ModuleId, bytes: &[u8]) -> Result<Option<StampView>, StampError>` — accepts one section, rejects any section longer than 1 252 bytes (C-17), returns a complete `StampView` once all sections of one stamp are present.
   - `fn on_hello(&mut self, hello: &HelloView) -> Freshness` — compares the hello's stamp version (C-18) with the last complete stamp version for that module and returns `Current`, `Stale { have, announced }` or `NoStampYet` (no complete stamp held; expected within 10 s of start, C-18).
2. Incomplete stamps are discarded when a section of a newer stamp version arrives for the same module; never mix sections of two versions.
3. A 10 s refresh carrying the version already held is accepted and replaces the held stamp (its counters may be newer).
4. Decoding uses the `hadronyche-core` decoder (epic 013, C-133); this ticket only adds sectioning and versioning.

## Acceptance criteria
1. A stamp split into 1, 2 and 5 sections reassembles to a `StampView` equal to the original.
2. A 1 253-byte section is rejected with `StampError`, no panic.
3. Sections of version n and n+1 interleaved never produce a stamp mixing both; the result is version n+1 only.
4. After a hello announcing version n+1 while version n is held, `on_hello` returns `Stale { have: n, announced: n+1 }`; the module is shown as stale until n+1 arrives (consumed by 029-06).
5. Out-of-order and duplicate sections are tolerated.
6. A refresh of the held version with changed counters replaces the held stamp and leaves `Freshness` at `Current`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] `push_section` is public so the 029-05 fuzz harness can drive it
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" — this ticket adds or changes no stamp field.

## Out of scope
- Sockets and group membership (029-03); the recorded file format (029-04); fuzz harness (029-05).
- Sending a stamp request: never done by the monitor (C-18 multicast makes it unnecessary).
