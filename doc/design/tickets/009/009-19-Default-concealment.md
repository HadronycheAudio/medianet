# 009-19 — Default concealment
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-02, epic 004

## Context
Gap contract `[decision]`: when an input is a gap at a round (MarkedGap, OtherEpoch, Missing or Demoted) or the input is Interrupted, the module conceals. Default concealment is per content type, with a per-connection override:
- audio: zero-fill;
- dense controllers: hold last value; when a gap precedes any received value, the controller's initial value declared in the manifest, default 0 (C-158);
- sparse keyed controllers: rely on state refresh (no-op).

The per-connection override is a per-input `Conceal` implementor held by the consumer and chosen without allocation `[decision]` (epic Background).

Reference mixer rule: a missing input contributes zero and the output slice is marked degraded. The epic uses this mixer for the offline reference of EC-2 (diamond, 009-20).

- C-04 `[decision]`: 6 channels per stream at defaults; 192 bytes per channel per slice; 1 152 bytes of audio per slice; no per-channel id byte.
- C-03 `[decision]`: 4-byte samples, f32 or Q1.31. The reference mixer sums Q1.31 samples as integers (saturating, documented) so the result is bit-exact and order-independent; an f32 variant sums in input-index order.
- `ContentType` comes from epic 004.
- C-181 `[decision]`: a bundle source's controllers ride in their own member stream (C-205), so they reach the consumer as an input of their own content type; a single-stream source's controllers share the audio packet while headers plus controllers stay within the 100-byte cap (C-137). A slice of such a stream holds sections of two content types, and each section is concealed with its own type (sections located with epic 004's layout).

Transport state (C-141, C-199, C-200) is concealed by 009-27 through the same trait; this ticket adds the `positions_elapsed` parameter and passes it through.

## Work
In `hadronyche-core`, module `rx::conceal`:
```rust
pub trait Conceal { fn fill(&mut self, content: ContentType, out: &mut [u8], last: Option<&[u8]>,
                            initial: &[u8], positions_elapsed: u64); }
pub struct DefaultConceal; // audio zero-fill, dense hold-last or initial value (C-158), sparse no-op
impl Conceal for DefaultConceal { .. }
```
- Audio: `out.fill(0)`.
- Dense controller: copy `last` into `out` (length = min of both); if `last` is `None`, copy `initial`, the manifest's declared initial value for that controller (C-158; the manifest field comes from epic 004, default 0).
- Sparse keyed controller: leave `out` untouched.
- Mixed slice (C-181): the caller calls `fill` once per section, with that section's content type and sub-slices of `out`, `last` and `initial`; audio sections are zero-filled and controller sections held.
- Per-connection override: the consumer holds one `C: Conceal` per input (generic, so no allocation and no `dyn` box); document the choice.
- Add a test-only reference mixer that applies the rule "missing contributes zero, output degraded", used by 009-20.

## Acceptance criteria
1. Audio: a 1 152-byte buffer pre-filled with 0xAA becomes all zero.
2. Dense, `last = Some([1,2,3,4])` → `out == [1,2,3,4]`; `last = None` with `initial = [5,6,7,8]` → `out == [5,6,7,8]`; `last = None` with the default initial value → zeros (C-158).
3. Sparse: `out` is byte-for-byte unchanged.
4. A custom `Conceal` installed for one input is used for that input only.
4a. Mixed slice of a single-stream source (C-181): the audio section is all zero and the controller section equals its last received value (or its manifest initial value before any, C-158).
5. Reference mixer: 3 inputs, one `Missing` → output equals the sum of the 2 present inputs, `degraded == true`.
6. Reference mixer, Q1.31: permuting the input order gives the same bytes.
7. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Transport-state concealment (009-27, C-141).
- Concealment strategies beyond the defaults (module code).
- Telling the module about Interrupted (009-15) and the module API (other epics).
