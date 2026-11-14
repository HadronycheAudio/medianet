# 015b-07 — Don't-fragment capture test
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** XS · **Depends on:** 015a-17, 015a-18
## Context
- Epic criterion 015b-A8: every packet leaves with don't-fragment set (verified by capture).
- IP fragmentation is forbidden; `IP_MTU_DISCOVER` is set to forbid fragmentation on every socket (015a-04). IPv4 only (C-100). Fallback path MTU 1 280 bytes, 1 252 bytes of UDP payload (C-06, C-100).
- Capturing needs `CAP_NET_RAW`. Inside an unprivileged user namespace with its own network namespace the test owns that capability over the namespace's loopback `[kernel-doc]`, so the capture can run in unprivileged CI provided the CI host allows unprivileged user namespaces.
- Traffic types the runtime sends: media, sync, probe pair (C-116), receiver report, hello/discovery, regime stamp sections (C-18).
- The DF checker is a tool binary in the `hadronyche-tools` crate (C-133); its output format is decided here and recorded in its documentation (C-206).
## Work
1. Run a two-module loopback scenario inside a user and network namespace, covering all six traffic types; capture with pcap.
2. A checker in `hadronyche-tools` that verifies the DF bit on every captured IPv4 packet from the runtime.
## Acceptance criteria
1. 100 % of captured runtime packets have DF set; 0 fragments; no IPv6 packet.
2. The capture covers all six traffic types.
3. The job runs unprivileged in CI; if the CI host forbids user namespaces, the job reports that explicitly rather than passing.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — the checker reads pcap through an existing library, no new parser; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "No new open item" — none raised; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- Path-MTU decision procedure (core, epics 012 and 014).
