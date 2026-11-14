# 023-21 — Manifest binary form and round trip
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-19, 023-20, epic 004, epic 013

## Context
- `manifest.rs` serialisation: **binary form defined with epics 004/013** (content message definitions and stamp fields) and exchanged per epic 012 (manifest exchange).
- Settled inputs: L is always a whole number of domain-rate samples **in the manifest and ABI** (C-105; the rational form is withdrawn); an edge module's device-facing ports carry their conversion delay, also whole samples (C-134); each dense controller input carries its initial value (C-158); a join may be flagged for a frequency-dependent path (C-104); the channel map is carried in the manifest because the wire has no per-channel id byte (C-04); the manifest lists inserted join delays and opt-outs (C-104); cost is in abstract operations (C-106).
- Remaining gaps: the layout itself (field order, widths, endianness, width of L) — owner epic 004 with epic 013; the value of `abi_version` and its versioning policy — owner epic 012.
- A manifest must fit any control message bound that epic 012's exchange imposes; C-17 limits one control message to 1 252 bytes of UDP payload, so a large manifest may need sections (as the regime stamp does, C-18). The exchange is epic 012's; this ticket keeps the encoding compact and reports its size.
- 023-AC-7: the manifest **round-trips through its binary form**. 023-AC-9: the decoder is fuzzed (023-22), so it must be total: every malformed input returns an error.
- Output location (C-130): the layout is a specification in `doc/design/spec/`.

## Work
1. Agree the wire layout with epics 004/013 owners; write it down (field order, widths, endianness, length prefixes, version field) in `doc/design/spec/`.
2. `Manifest::encode(&self) -> Vec<u8>` and `Manifest::decode(&[u8]) -> Result<Manifest, DecodeError>`, no panics, bounded allocation (length fields capped, e.g. bundle members ≤ 32 per C-08, streams ≤ 256 per C-09).
3. Determinism class encoded as the epic 013 stamp value.
4. Report the encoded size of the largest corpus manifest and of a 32-member-bundle manifest to epic 012.

## Acceptance criteria
1. `decode(encode(m)) == m` for every corpus program's manifest and for ≥ 10 000 property-generated manifests, including L = 0 and the largest L the layout allows, flagged joins, dense initial values and edge conversion delays.
2. Truncated input at every byte offset of a valid encoding → `Err`, no panic.
3. Unknown `abi_version` → `Err(UnsupportedAbi)`.
4. A length field claiming 2^32 entries → `Err` without allocating more than the input size × a small constant.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (decoder fuzzed in 023-22)
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Fuzz harness (023-22); manifest exchange protocol and its sectioning (epic 012).
