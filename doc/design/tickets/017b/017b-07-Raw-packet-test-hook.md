# 017b-07 — Test-only hook: raw packet send
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-05, epic 015

## Context
- Rule C-120: a test-only hook sends a raw packet on a media or control socket; it is stamped like any injection. The other C-120 hooks are 017b-06.
- Users: scenario `malformed` (017b-19, AC-23) sends malformed, wrong-version and foreign-domain packets; scenario `flood` (017b-21, AC-017.14) offers more packets than an input was admitted for.
- Sending from the module's own socket means the bad packets carry a real peer's source address and port, so per-input counters can attribute them.
- IPv4 only (C-100). The hook is not public API; it sits behind the same test-only feature as 017b-06 (names may be refined):
```rust
fn send_raw(&self, from: ModuleHandle, socket: SocketKind, to: std::net::SocketAddrV4, bytes: &[u8]);
```
- `SocketKind` distinguishes the media socket from the control socket; `ModuleHandle` comes from epic 015.

## Work
1. Implement `send_raw` on the module's media or control socket, bypassing the packetiser and validation, with the DSCP class of that socket (C-13).
2. Record each call in the injection log of 017b-05 (method, module, socket kind, destination, length, and a hash of the bytes rather than the bytes, so the record stays within stamp sections of 1 252 bytes, C-17, C-18).
3. Build the hook only under the test-only feature.

## Acceptance criteria
1. A raw packet sent from module A's media socket arrives at B with A's media source address and port, byte for byte.
2. The same from A's control socket arrives with A's control source address and port.
3. Each call appears in the stamp's injection record; 1 000 calls still serialise into sections within 1 252 bytes each.
4. A build without the feature does not expose the hook.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- What receivers do with the packets (017b-19, 017b-21); forced identity and epoch (017b-06).
