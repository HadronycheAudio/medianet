# 021-06 — Interface stats and absent drop accounting
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** XS · **Depends on:** 021-04
## Context
`OsNet::iface_stats` (trait from epic 016) on macOS.

- Porting trap [memory]: macOS has **no per-socket drop counter**; socket overflow cannot be told from network loss. Loss classification is weaker and the stamp says *drop accounting: absent*.
- Overflow accounting is report-only on macOS.
- Third parties see stream health only through the per-connection summary counters that producer and consumer publish in their own regime stamps, received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead (C-47, C-159), plus the observed DSCP. Core keeps those counters; on macOS a "missing" count cannot be split into socket overflow and network loss, which is why the stamp must say *drop accounting: absent* beside them.
## Work
- Implement `iface_stats` for the medianet interface with the counters macOS offers at interface level.
- Return the drop/overflow field as an explicit "absent" value (not zero) so core cannot treat it as "no drops".
- Expose `drop_accounting = Absent` for the regime stamp (021-20).
## Acceptance criteria
1. `iface_stats` returns interface counters that increase after sending 100 loopback datagrams (on loopback interface).
2. The per-socket drop field is the "absent" variant in every return; a unit test asserts it is never `Some(0)`.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
## Out of scope
Loss classification logic in core. Stamp assembly (021-20).
