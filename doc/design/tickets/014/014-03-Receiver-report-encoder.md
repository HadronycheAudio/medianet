# 014-03 — Receiver report encoder
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-02, epic 004, epic 009, epic 012, epic 013
## Context
Media packets carry no per-receiver state `[decision]`, so measurement travels on a separate reverse unicast path whose port is given by the admission handshake (epic 012). The receiver report is sent by the consumer's **data thread**, one per **bundle** (not per member) every 100 ms (C-40), with DSCP CS5 (40) (C-13), unicast to the producer only; it is never multicast and there is no query message (C-47). It is also the producer's only evidence that the consumer is alive.

Report content `[decision]`:
- Header: `at_position: u64` (timeline position at which the report is made). There is no bundle-level session.
- Per member: `stream` (16-bit stream number, C-10) and that stream's `session` (32 bits, C-10; assigned by the source, one per stream, same for every consumer, C-19); `received`, `late` (arrived after the consumer's deadline; remedy is budget), `missing` (absent at the deadline; remedy is the network), `marked_gap` (upstream alive, loss further up), `other_epoch` (slices on another timeline epoch, never a miss, C-135), `too_early`, `duplicates`, `incomplete_reassemblies`, `demoted_rounds` (the separate Demoted count of C-109), `interrupted` and input `health` (`InputHealth`, from epic 009, including Demoted, C-109). This is the C-159 counter set less dead, which the producer derives itself; it is carried so the producer's stamp summary can match the consumer's (C-47, C-159). Counters count slices (014-02); missing can decrease after a late reclassification (C-108).
- Drop counters: socket overflow (`SO_RXQ_OVFL`, Linux since 2.6.33, cumulative, read on every receive `[kernel-doc]`); interface rx dropped and rx missed (deltas). On macOS and Windows there is no per-socket counter `[memory]`: the field is **absent**, not zero.
- Observed DSCP of received media (the receiver reads the IPv4 type-of-service byte, C-100; marking is verified by the peer, never self-reported). Expected media value 46 (C-13). Absent when unknown.
- Size: must fit one packet at the 1 280-byte path-MTU fallback over IPv4, that is at most 1 252 bytes of UDP payload (C-06, C-17, C-100), with the worst case of 32 members (C-08), each carrying every C-159 count. A bundle source's controller member counts among the 32 (C-209) and, being a member stream (C-205), has its own member record like any other. There is no split rule: if epic 004's layout does not fit 32 members, that is a defect returned to epic 004.
- Error path (settled by the epic): more than 32 members or an undersized buffer returns an error and writes nothing.

Shapes from the epic (names may be refined):
```rust
pub enum EncodeError { TooManyMembers, BufferTooSmall }
pub fn encode_report(at_position: u64, members: &[MemberCounters], drops: &DropCounters,
                     observed_dscp: Option<u8>, out: &mut [u8]) -> Result<usize, EncodeError>;
```

## Work
1. Define `MemberCounters` (with `session`) and `DropCounters` in the module `measure` with the widths decided in 014-02.
2. Implement `encode_report` following the receiver-report layout of epic 004: message type, position, member count, per-member records (stream, session, counters, health), drop counters with a presence flag per `Option`, observed DSCP with a presence flag. Returns bytes written.
3. Return `EncodeError::TooManyMembers` when `members.len() > 32` and `EncodeError::BufferTooSmall` when `out` is shorter than the encoded size, writing nothing in either case.
4. Compute the worst-case encoded size for 32 members and add it as a `const MAX_REPORT_LEN`.
5. No allocation, no locking, no panics on any input.

## Acceptance criteria
1. `MAX_REPORT_LEN` (32 members, all optional fields present) is ≤ 1 252 bytes (C-17, C-100); a unit test asserts it (epic AC-3).
2. A 16-member bundle encodes into **one** report (one message per bundle, not per member).
3. An absent `socket_overflow` is encoded differently from `Some(0)`; same for each other `Option` field.
4. Two members of one bundle with different sessions are encoded with their own sessions.
5. Every `InputHealth` value of epic 009, including Demoted, is encodable, and each C-159 count of a member (other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds) is encoded in its own field.
6. `encode_report` with `members.len() == 33` returns `TooManyMembers`, and with an undersized `out` returns `BufferTooSmall`; in both cases `out` is unchanged and nothing panics.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Report generation takes bounded time on the data thread and is shown not to allocate or lock
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Parsing (014-04), vectors and fuzzing (014-05).
- Reading `SO_RXQ_OVFL`, interface counters and the TOS byte from the OS (epics 013, 015).
- Sending the report on the reverse socket (epic 015).
- Per-connection summaries for the regime stamp (014-22).
- The consumer's current A in the report header (C-198, 014-26).
