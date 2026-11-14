# 023-07 — Bundle layout and channel map
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-06

## Context
- A stream carries 6 channels at defaults, 192 bytes per channel per slice, with **no per-channel id byte: the channel map is in the manifest** (C-04). The manifest's channel map is therefore the only record of which port channel travels in which member lane, and must be exact.
- Wider ports become **bundles** of at most 32 member streams (C-08), so a port is at most 32 × 6 = 192 channels at defaults. At most 256 streams per source (C-09). The 32 include a source's controller member (C-209): a source with controllers has at most 31 audio members (186 channels); that check is 023-32, this ticket lays out audio ports only.
- Epic 023-AC-7: a 96-channel port yields a **16-member bundle with a 96-entry channel map** (96 / 6 = 16).
- Epic 023-AC-1: a port wider than 192 channels, or an explicit bundle layout with more than 32 members or a member wider than 6 channels, is rejected with a clear diagnostic.
- Rule implemented here: default layout = ceil(channels / 6) members of 6 channels (last member may be partial), channel map = port channel i → (member i / 6, lane i % 6); explicit layout = author-given list of member widths (each 1..=6) and channel map, validated against the 32-member cap and C-09.
- Remaining gap (owner: epic 004): whether a member may be wider than 6 channels under a non-default slice length or the multi-packet regime (C-05). Until 004 answers, the width cap is 6.

## Work
1. `types::BundleLayout { members: Vec<MemberWidth>, channel_map: Vec<(member, lane)> }`.
2. `fn default_layout(channels) -> Result<BundleLayout, Diagnostic>`; error if channels > 192.
3. `fn validate_explicit(layout, channels) -> Result<(), Diagnostic>`: total width equals channels, member count ≤ 32, each member width ≤ 6, each channel mapped exactly once.
4. Ports with ≤ 6 channels have a single-member layout.

## Acceptance criteria
1. 96 channels → 16 members, each 6 wide, channel map of 96 entries with channel 95 → (15, 5).
2. 6 channels → 1 member; 7 channels → 2 members (6 + 1); 192 channels → 32 members.
3. 193 channels, with or without an explicit layout → diagnostic citing the 32 × 6 limit (C-04, C-08).
4. An explicit layout with a 7-channel member, or with 33 members → diagnostic.
5. An explicit layout mapping one channel twice → diagnostic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Using layouts in the checker (023-08) and serialising them (023-19, 023-21).
