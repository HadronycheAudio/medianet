# 015a-06 — Batch receive with control messages (recv_batch)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-02, 015a-03, 015a-04
## Context
- File: `hadronyche-os-linux/src/net.rs`. Trait method: `fn recv_batch(&mut self, bufs: &mut [RxBuf], meta: &mut [RxMeta], max: usize) -> usize;` — receive up to `max` datagrams without blocking; returns the count.
- **Batch receive is required** (several datagrams per system call): two 96-channel bundles are 32 packets per 0.5 ms slice (C-92) `[decision]`. Implemented with `recvmmsg`.
- Batch size: the caller passes `max` = the drain bound C-16 (2 × admitted packets per slice, per input per slice period, C-156); the message, iovec and control vectors are preallocated at socket construction to the largest bound the socket will serve (64 for a 32-member bundle at one packet per slice, C-05, C-08).
- Per message, fill `RxMeta`: `len`, `from`, `rx_host_ns` (receive time on the host clock, `None` if discarded because of a wall-clock step), `source` (`StampSource::Kernel` when a kernel stamp was present, else `Userspace`; C-131), `dropped_total` (cumulative drops on this socket from `SO_RXQ_OVFL`, a cumulative count in a control message `[kernel-doc]`), `tos` (from `IP_RECVTOS`).
- Each receive-timestamp source is reported with one of the C-129 states (present, absent, unverified, not requested) (C-131). This ticket counts, per socket, which source each message used, so 015b-13 can stamp kernel stamps as present (seen), absent (requested, never delivered) or unverified (not yet observed).
- `SO_TIMESTAMPNS` stamps are `CLOCK_REALTIME` `[kernel-doc]`. Convert to host time (`CLOCK_MONOTONIC`) with the step detector of 015a-03: sample realtime-minus-monotonic before and after the batch; if it moved beyond the tolerance, set `rx_host_ns = None` for the batch and count the discards. Detection belongs here, in the backend (C-121).
- IPv4 only (C-100).
- The per-socket overflow counter is read with every receive; it feeds the regime stamp and the receiver report (015a-14).
- Results of the kernel spike (015a-02) decide fallbacks on kernels lacking per-message control messages.
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
## Work
1. Implement `recv_batch` over `recvmmsg(MSG_DONTWAIT)` with preallocated `mmsghdr`, `iovec` and control buffers sized at construction (no allocation per call); refuse `max` above the preallocated size.
2. Write a control-message parser (`SCM_TIMESTAMPNS`, `SO_RXQ_OVFL`, `IP_TOS`) as a pure function over a byte slice; tolerate truncated and unknown messages.
3. Fuzz the control-message parser (cargo-fuzz or equivalent) to C-95, and keep the corpus, seeded from real captures.
4. Count messages per `StampSource` per socket.
5. Integration tests on loopback.
## Acceptance criteria
1. A burst of 64 datagrams to one socket is received with one `recv_batch` call at `max = 64`, or with two at `max = 32`, each `RxMeta` with correct `len` and `from`.
2. `tos` equals the TOS byte the sender set (192, 184 or 160) on loopback.
3. With an induced overflow, `dropped_total` is `Some(n)` with n > 0 and non-decreasing across calls.
4. With a simulated realtime step between the before/after samples, `rx_host_ns` is `None` for that batch and the discard counter increases.
5. `recv_batch` on an empty socket returns 0 without blocking.
6. The parser fuzzer runs 60 s on every change in CI and 1 hour nightly without a panic; the corpus is stored in the repository (C-95).
7. No heap allocation inside `recv_batch` (checked with a counting allocator in a unit test).
8. On loopback, every message has `source = Kernel` and the per-source counter shows it.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Regime stamp fields touched" — counters reach the stamp in 015a-14 and 015b-13; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — measured in 015b-10.
## Out of scope
- Bounded drain policy and source filtering (015a-11).
- Putting drop counts into the stamp and report (015a-14).
- Per-packet receive cost measurement (015b-10).
