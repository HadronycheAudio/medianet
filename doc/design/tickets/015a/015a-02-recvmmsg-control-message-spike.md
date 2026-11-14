# 015a-02 — recvmmsg control-message spike on target kernels
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-01
## Context
- Open sub-question of epic 015a, owned by this task and scheduled for its first week: "Does `recvmmsg` return per-message `SO_RXQ_OVFL` and timestamps as expected on the target kernels?" Falsifier: a kernel in the support list returns neither.
- The receive path (015a-06) relies on: batch receive with `recvmmsg` (several datagrams per system call); `SO_TIMESTAMPNS` kernel receive stamps, which are `CLOCK_REALTIME` stamps `[kernel-doc]`; `SO_RXQ_OVFL`, which returns a cumulative drop count in a control message `[kernel-doc]`; `IP_RECVTOS`, which returns the received TOS byte.
- IPv4 only (C-100): only the IPv4 control messages are in scope.
- Batch size: the `recvmmsg` vector is the drain bound C-16, 2 × the admitted packets per slice; 64 for a 32-member bundle at one packet per slice (C-05, C-08). Two 96-channel bundles are 32 packets per 0.5 ms slice (C-92).
- Where things live (C-130, C-133): the probe program is a throwaway probe in `probes/`; the results go to `doc/design/reports/`.
- Supported OS versions are set after the receive-stamp probe task (001) reports; its probes test the current release of each OS and the oldest obtainable one (C-138). This spike therefore runs on the same two Linux kernels: the current release and the oldest obtainable one, plus any other kernel available to the team, and records which ones those were. Once task 001's report sets the support list, any listed kernel not yet covered is run before the ticket closes.
## Work
1. Write a small standalone probe program in `probes/` that opens a UDP/IPv4 socket with `SO_TIMESTAMPNS`, `SO_RXQ_OVFL` and `IP_RECVTOS` enabled, sends a burst to itself, and calls `recvmmsg` with a vector of 64 messages.
2. For each returned message, record whether a timestamp, an overflow count and a TOS byte were present.
3. Induce overflow (small `SO_RCVBUF`, burst larger than the buffer) and record whether the cumulative count increases and on which messages it appears.
4. Run on the current Linux release and the oldest obtainable kernel (C-138), and on any other kernel available to the team; record kernel version and results in a report under `doc/design/reports/`, with `[measured]` tags.
5. If any kernel returns neither stamp nor overflow per message, raise it as an open item with an owner and a falsifier and record the fallback (for example `StampSource::Userspace`, `dropped_total: None`).
## Acceptance criteria
1. The probe runs unprivileged and prints, per message of a 64-message `recvmmsg` call, the presence of timestamp, overflow count and TOS.
2. With an induced overflow, the reported cumulative count is > 0 on at least one message per batch after the overflow, or the absence is documented.
3. A results table (kernel version × {timestamp, SO_RXQ_OVFL, TOS}) exists in `doc/design/reports/`, each entry tagged `[measured]`.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — the probe prints raw presence and ships no parser (the production parser is fuzzed in 015a-06); "Regime stamp fields touched" — none; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a cost measurement (015b-10).
## Out of scope
- The production `recv_batch` implementation (015a-06).
- Interrupt-coalescing bias on stamps (owned by task 019).
- Choosing the support list of target kernels (set from task 001's report, C-138).
