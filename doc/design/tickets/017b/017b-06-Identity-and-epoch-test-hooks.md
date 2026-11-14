# 017b-06 — Test-only hooks: forced identity and forced epoch
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-05, epic 010, epic 015

## Context
- Rule C-120: beyond the public injection API, the runtime has test-only hooks to force a module id and nonce, force a stream onto a given timeline epoch, and send a raw packet on a media or control socket. They are stamped like any injection. This ticket covers the first two; raw send is 017b-07.
- Users: scenario `collision` (017b-18, AC-22) and scenario `epoch` (017b-16, AC-18).
- Widths (C-10, C-11): module id 64 bits, instance nonce 64 bits, timeline epoch 16 bits.
- Session number (C-19): the source bumps a stream's session number on timeline epoch change; the forced epoch must go through the same path so the bump happens.
- The hooks are not public API: they sit behind a test-only feature of `hadronyche-std` (shape from the epic; names may be refined):
```rust
pub trait TestHooks {
    fn force_identity(&self, module: ModuleHandle, id: u64, nonce: u64);
    fn force_epoch(&self, stream: StreamHandle, epoch: u16);
    // send_raw: 017b-07
}
```
- `ModuleHandle` and `StreamHandle` come from epic 015.

## Work
1. `force_identity(module, id, nonce)`: the module starts (or restarts) with that id and nonce instead of drawing them; the runtime's normal collision handling then applies.
2. `force_epoch(stream, epoch)`: the stream's packets carry the given timeline epoch from the next slice on, through the packetiser of epic 010, with the session number bumped as for any epoch change (C-19).
3. Record every call in the injection log of 017b-05.
4. Build the hooks only under the test-only feature.

## Acceptance criteria
1. Two modules forced to the same id with different nonces both start with that id; the runtime's collision handling is triggered.
2. After `force_epoch(s, e)`, every packet of stream `s` carries epoch `e`, and its session number is bumped once.
3. Each call appears in the stamp's injection record with its parameters.
4. A build without the feature does not expose the hooks.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Raw packet send (017b-07); the scenarios that use these hooks (017b-16, 017b-18).
