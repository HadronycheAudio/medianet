# 022a-04 — Receive timestamps and batch receive
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-02, 022a-03, epic 001, epic 016

## Context
- Part of epic 022a (Windows edge backend), `net.rs`.
- Receive timestamp on Windows `[memory]`: a Winsock timestamping control for UDP receive, stamped in QPC units, exists on recent Windows 10 and 11 builds. **Gate it by a capability bit.** Task 001 probes it. If absent: userspace read of the host clock immediately after receive, stamped *userspace*.
- Rule: kernel timestamps are used only if task 001 confirms them **and** the build supports them; otherwise userspace.
- Source values are settled (C-131): *kernel* (hosted OS kernel stamp) and *userspace* (clock read after receive) are the only two that exist on Windows (*mac-hardware* and *interrupt* are embedded only). Each is reported with one of the four capability states of C-129: present, absent, unverified, not requested. Until task 001 confirms the control on a build, *kernel* is at most *unverified* on that build.
- Batch receive on Windows `[memory]`: there is no multi-message receive; use a loop of non-blocking receives with control data. Its cost must be measured.
- Open sub-question (owner: task 001): is the UDP receive-timestamping control present, and on which builds? The probes test the current Windows release and the oldest obtainable one, and the supported OS versions are set after task 001 reports (C-138). Falsifier: absent → the clock bound rises; the Windows E budget allots 100 µs of the 1 ms target to the clock (C-150, proposal pending L2), so the 1 ms target (C-72) is revisited only if the measured clock share exceeds that allotment. E on Windows is dominated by D uncertainty (800 µs of the budget, C-150) and scheduling spikes, not by the clock.
- The receive-timestamp source is a regime-stamp field (022-AC-8).
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- These stamps feed the sync-exchange builder and its wall-clock step check (022a-19, C-121).

## Work
1. Capability probe at socket open: try to enable the receive-timestamp control; record `RxStampSource::{Kernel, Userspace}` with a `CapState::{Present, Absent, Unverified, NotRequested}` (C-129, C-131) and expose it as a capability bit.
2. Receive path: request control data with each receive; parse the control-message buffer to extract the QPC timestamp; convert with the shared `qpc_ticks_to_ns` (022a-02). If no timestamp is present, take `QpcClock::now_ns()` immediately after the receive returns and tag the sample *userspace*. Always also record that post-receive userspace read (needed by 022a-19).
3. Control-message parser as a pure function `parse_rx_cmsgs(&[u8]) -> Result<RxMeta>` (timestamp, and a slot for the TOS byte filled by 022a-05). Fuzz it to the C-95 budget.
4. Batch receive: `recv_batch(&mut [RxSlot]) -> usize` looping non-blocking receives until would-block or the slice is full.
5. Benchmark: cost per datagram of `recv_batch` at the rate of one default stream (2 000 packets/s of 1 152 bytes of audio plus headers, C-04, C-22) and at a burst of 20 queued datagrams (one Windows capture batch at a 10 ms period); record the numbers.

## Acceptance criteria
1. On a build where the control is available, ≥ 99 % of received datagrams carry a kernel timestamp and the source reports *kernel*; otherwise 100 % carry a userspace timestamp and the source reports *userspace*; the C-129 state is reported with the source (selection unit-tested with a mocked capability).
2. Kernel timestamps never exceed the userspace read taken just after the receive (sanity check over 10 000 datagrams on loopback).
3. `parse_rx_cmsgs` has a fuzz target run for 60 s on every change in CI and 1 hour nightly, with the corpus kept (C-95); malformed buffers return an error, never a panic.
4. `recv_batch` drains a burst of 40 queued datagrams in one call and returns 40.
5. Per-datagram cost of `recv_batch` is measured and written down with its evidence tag `[measured]`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag
- [ ] Note in code and in the result record: the receive-timestamp claim stays `[memory]` until task 001 publishes its Windows result

## Out of scope
- Building sync exchanges and the step check (022a-19).
- Replacing the receive-timestamp claim in the design text with task 001's `[measured]` statement (022b-08).
- TOS byte extraction (022a-05).
- The regime stamp itself (022a-17).
