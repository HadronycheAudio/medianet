# 015a-07 — Interface statistics (ifstats.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-01
## Context
- File: `hadronyche-os-linux/src/ifstats.rs`; backs `OsNet::iface_stats(iface) -> IfaceStats`.
- `IfaceStats` (epic): interface MTU, link speed (or unknown), `rx_dropped`, `tx_dropped` — the drop counters in `/sys/class/net/<if>/statistics` `[kernel-doc]`.
- Sampling: interface counters are sampled each time a regime stamp is built — on every regime change and at every 10 s refresh (C-18). Housekeeping does the sampling (015a-14); this ticket only provides the reader.
- The interface MTU is the upper bound for the path MTU; the path itself is verified by the probe pair at the intended media packet size (C-116); fallback path MTU 1 280 bytes, 1 252 bytes of UDP payload (C-06, C-100).
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
## Work
1. Read MTU (`/sys/class/net/<if>/mtu` or `SIOCGIFMTU`), speed (`/sys/class/net/<if>/speed`), and `rx_dropped`, `tx_dropped` from `/sys/class/net/<if>/statistics`.
2. Parse sysfs text with a pure parser; handle missing files, `-1` speed and non-numeric content without panicking.
3. Fuzz the parser to C-95 and keep the corpus.
4. Keep file handles open after the first read so sampling does not open files repeatedly.
## Acceptance criteria
1. On loopback (`lo`), `iface_stats` returns MTU equal to `/sys/class/net/lo/mtu` and drop counters equal to the sysfs values.
2. A missing or unreadable speed file, or a speed of `-1`, yields an explicit "unknown" value, not a panic or 0 presented as real.
3. The parser fuzzer runs 60 s on every change in CI and 1 hour nightly without a panic; the corpus is stored (C-95).
4. Repeated sampling (1 000 calls) performs no file `open` after the first (verified with a test hook).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Design documents updated" — no behaviour differs; "No new open item" — none raised; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — no receive path.
## Out of scope
- Writing the counters into the stamp (015a-14, 015a-18).
- The path-MTU decision procedure (core, epics 012 and 014).
