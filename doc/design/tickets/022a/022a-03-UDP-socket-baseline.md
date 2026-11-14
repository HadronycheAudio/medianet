# 022a-03 — UDP socket baseline (IPv4, buffers, connection-reset, don't-fragment, pinned multicast)
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-01, epic 016

## Context
- Part of epic 022a (Windows edge backend). `net.rs` implements `OsNet` / `DatagramSocket` (traits from epic 016). This ticket builds the socket-opening primitive every later networking ticket uses.
- **IPv4 only** in v1; multicast is sent with **TTL 1** (C-100). The 1 280-byte path-MTU fallback (C-06) leaves 1 252 bytes of UDP payload (C-100); control messages must fit it (C-17).
- Windows porting traps `[memory]`:
  - **Connection-reset on UDP.** A socket that sent to a dead peer starts failing its receives with a connection-reset error unless that behaviour is switched off with a socket control. Switch it off on **every** socket.
  - **Default receive buffer is 64 KB**; size it explicitly to the declared rate times the skew budget.
  - **Multicast must be joined and sent on the medianet interface by index**; a laptop has Wi-Fi up beside it.
  - **Don't-fragment** is an IPv4 socket option; check by capture on the wire.
- Buffer sizing is settled: the skew budget is capped at the presentation group's A (C-96), so the receive buffer is the sum of the declared stream rates times A, never below the 64 KB default. One default stream is 2 000 packets/s of about 1 320 bytes on the wire, about 21.1 Mbit/s (C-20, C-21, C-22).
- The path-MTU probe pair is sent at the intended media packet size, exempt from the control bound (C-116); it relies on don't-fragment so that an oversize probe is not fragmented.
- Rules that apply unchanged: don't-fragment on every packet; 6 channels per stream, 1 152 bytes of audio per slice (C-04); slices are 0.5 ms (C-02); one wire rate, 96 kHz (C-01).
- How the medianet interface index is configured and chosen is an open sub-question owned by epic 016; this ticket takes the index as an explicit input.
- Product frame: medianets are wired, isolated LANs whose only traffic is this runtime's.

## Work
1. In `net.rs`, an internal `open_udp(params) -> Result<WinSocket>` that, on every socket:
   - opens an IPv4 socket only (no dual-stack, no IPv6 path);
   - disables UDP connection-reset reporting via the socket control;
   - sets don't-fragment;
   - sets the receive buffer (and send buffer) explicitly to `rcvbuf_bytes(declared_rate_bytes_per_s, a_ns)`, a pure function returning `max(64 KB, rate × A)`; reads the value back and reports it if the OS clamped it; re-sizes when the group's A changes;
   - for multicast: joins the group on the medianet interface **by index**, sets the outgoing multicast interface by the same index, sets multicast TTL 1, and disables multicast loopback unless the trait requires it.
2. Interface selection takes an explicit interface index (from configuration per epic 016); never `INADDR_ANY` for multicast.
3. `DatagramSocket` send / receive (single datagram, non-blocking) on top of `WinSocket`; a send larger than the path MTU with don't-fragment set returns a named "message too large" error to the caller.
4. Tests:
   - unit test of `rcvbuf_bytes` (floor and product);
   - loopback integration test: socket A sends to a port nobody listens on, then socket B sends to A; A's next receive returns B's datagram with no error (022-AC-4 in miniature);
   - a test that reads back don't-fragment, multicast TTL and buffer sizes from the socket.

## Acceptance criteria
1. **(022-AC-4, Connection-reset.)** After the peer process is killed, the edge's sockets keep receiving from other peers with no error: integration test with a child process peer that is killed, then a second peer sends 100 datagrams; all 100 are received and no receive returns a connection-reset error.
2. Every socket opened through `open_udp` is IPv4 and has don't-fragment set (read back in a test); a 1 400-byte send to a 1 280-byte-MTU path is not fragmented and the sender sees the named error (manual wire capture recorded for 022b-08).
3. Receive buffer read back is ≥ `rcvbuf_bytes(...)` or the clamp is reported as a warning with both numbers; `rcvbuf_bytes` returns 64 KB when rate × A is smaller.
4. Multicast join and outgoing interface both use the configured interface index and TTL 1; a test on a two-interface host (or a mocked API layer) shows no traffic chosen via the other interface.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The connection-reset, don't-fragment and 64 KB-default facts are recorded as `[measured]` or corrected, for 022b-08

## Out of scope
- Receive timestamps and batch receive (022a-04).
- DSCP marking and TOS-byte receive (022a-05).
- `OsNet::open_media` / `open_discovery` / `iface_stats` assembly (022a-06).
- The path-MTU probe logic itself (core); this ticket only guarantees don't-fragment and the error path.
- Firewall detection (022a-08).
