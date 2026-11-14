# 022a-06 — `WinNet`: OsNet assembly and interface statistics
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-03, 022a-04, 022a-05, epic 016

## Context
- Part of epic 022a (Windows edge backend). The deliverable is `impl OsNet for WinNet { /* open_media, open_discovery, iface_stats */ }` (trait from epic 016), assembling the socket primitives of 022a-03 (IPv4, buffers sized to rate × A, connection-reset disabled, don't-fragment, interface-pinned multicast with TTL 1), 022a-04 (receive timestamps, batch receive) and 022a-05 (QoS-flow DSCP, TOS receive).
- Windows has **no per-socket drop counter** `[memory]`: overflow cannot be told from network loss. Stamp *drop accounting: absent* (C-129 state). Overflow accounting is report-only on Windows.
- DSCP classes (C-13): sync 48, media and one-way probes 46, control 40. Discovery carries hellos and regime stamps, which are control messages (C-17, C-18), so the discovery socket uses 40.
- Test-only injection hooks (C-120) include sending a raw packet on a media or control socket; they are stamped like any injection. The hook lives in core but must work through `WinNet` sockets.
- How the interface index is configured is an open sub-question owned by epic 016.

## Work
1. `pub struct WinNet` configured with the medianet interface index.
2. `open_media` → socket with media DSCP 46, receive timestamps, buffers sized to rate × A (C-96); `open_discovery` → IPv4 multicast socket pinned to the interface index, TTL 1, control DSCP 40.
3. `iface_stats` → what Windows provides per interface (packet / error / discard counters if available); the per-socket drop field reports **absent**, never zero.
4. A `NetCapabilities` struct exposing: receive-timestamp source with its C-129 state (022a-04, C-131), DSCP state per class (022a-05, with sync and control verification from 022a-21, C-152), drop accounting *absent*.
5. Raw-send path for the C-120 test hook on media and control sockets, compiled only in test builds.
6. Integration test: open media + discovery on loopback / a test interface, exchange datagrams, read stats.

## Acceptance criteria
1. `WinNet` compiles against the epic-016 `OsNet` trait and passes that trait's shared conformance tests, if epic 016 provides them.
2. `iface_stats` never reports a per-socket drop count of 0 as if measured; it reports *absent* (unit test).
3. `NetCapabilities` reports receive-timestamp source and state, DSCP state per class and drop accounting for use by the stamp (022a-17).
4. Media and discovery sockets both carry the 022a-03 settings and their DSCP class (46 and 40) (test reads them back).
5. In a test build, the C-120 raw-send hook sends a packet on a media and on a control socket; the hook is absent from release builds.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Firewall detection at startup (022a-08).
- The stamp rendering (022a-17).
- The injection API and its stamping (core).
- The panic-injection test hook (C-145): a core hook on threads, not on sockets.
