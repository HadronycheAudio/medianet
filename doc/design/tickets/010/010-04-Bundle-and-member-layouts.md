# 010-04 — Bundle and member layouts
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-03, epic 004
## Context
Bundles `[decision]`: streams stay narrow; a wide source emits a **bundle** of aligned streams.
- A bundle is a group id, a member list and a channel map, declared in the manifest. At defaults each member carries 6 channels (C-04); a 96-channel source is 16 members. The channel map lives in the manifest, not in the packets (C-04: no per-channel id byte).
- Members are independent on the wire: per-member gaps and per-member interrupted/dead.
- Each member stream has its own session number, assigned by the source (C-19); a consumer's single admission handshake lists the members it wants; a narrow sink subscribes only to the members it needs.
- Controllers ride in their own member. Settled by C-205 (and C-181): a bundle source's controllers are always in their own member stream, whatever their bytes per slice; the shared-packet case of C-181 exists only for a single-stream source (010-18).
- C-08: at most 32 members per bundle, the controller member included. C-209: the controller member counts towards the 32, so a source with controllers has at most 31 audio members (186 channels at defaults); a wider source with controllers is rejected at compile time (by the compiler, from the manifest). C-09: at most 256 streams per source. C-10: stream number is 16 bits.
- Streams are identified by source and stream number, never by destination.

Shapes given by the epic (names may be refined):
```rust
pub struct StreamId { pub number: u16 }
pub struct BundleLayout<const M: usize> {       // M <= 32
    pub group: u16, pub members: [MemberLayout; M], pub len: u8,
}
pub struct MemberLayout { pub stream: StreamId, pub content: ContentType, pub channels: u8, pub first_channel: u16 }
```
`ContentType` comes from the epic 004 layouts.

## Work
- Add the three types in `hadronyche-core`, `no_std`, no allocation.
- Add a const constructor that builds an audio bundle from a total channel count, a channels-per-member figure (6 at defaults, C-04; never more than `channels_per_packet`, 010-03, unless the member is meant to use the multi-packet regime) and a first stream number, filling `first_channel` contiguously; the last member may carry fewer channels.
- Add a constructor or method that appends a controller member (its own member, `ContentType` for controllers). For a bundle source with controllers the controller member is mandatory (C-205): the layout offers no way to place controller content in an audio member, and no controller-size test is applied. The controller member records the controller bytes per slice declared in the manifest, which 010-24 uses for its wire cost (C-219). Cite C-205 in the doc comment.
- Reject at construction: M > 32 (C-08), more members than M, a controller member appended to a layout that already has 32 audio members (the controller member counts towards the 32, C-209), stream numbers that overflow 16 bits, overlapping channel maps.
- Accessors needed by epic 012: member list and per-member stream id.

## Acceptance criteria
1. 24 channels at 6 per member yields 4 audio members; 48 yields 8; 96 yields 16 (epic AC-3, stream counts).
2. For 96 channels, member k has `first_channel == 6 * k` and `channels == 6`, k = 0..15.
3. 20 channels at 6 per member yields 4 members, the last with 2 channels.
4. A layout with 33 members is rejected (compile-time for `M > 32`, runtime error for overflow of `len`).
5. A bundle with a controller member has that member with the controller content type and 0 audio channels, and does not change the audio member count.
6. All types and constructors compile in the `no_std` CI job.
7. A bundle source with controllers (at least 2 audio members), built with controller bytes per slice of 1 and of 100, always yields a controller member, and no audio member has the controller content type (C-205).
8. 186 channels at 6 per member with controllers yields 31 audio members plus the controller member (32 in all); 192 channels at 6 per member with controllers is rejected with an error, without panic (C-209).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Session numbers and the bundle packetiser (010-06, 010-14).
- Wire rate per layout (010-05).
- Manifest parsing and the admission handshake (epic 012).
- The compile-time rejection of a source with controllers wider than 31 audio members (C-209): the compiler's manifest check; this ticket only refuses to build such a layout.
- Where controllers go when a source has a single 6-channel stream and no bundle (010-18).
