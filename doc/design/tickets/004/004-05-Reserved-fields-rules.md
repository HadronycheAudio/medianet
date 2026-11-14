# 004-05 — Reserved fields and their required values
**Epic:** 004-Wire-specification · **Effort:** XS · **Depends on:** 004-01, 004-03
## Context
`doc/design/spec/wire.md` must list reserved fields and their required value. Two are fixed by the design `[decision]`:
- C-07: **copy index**, 2 bits in the media packet header; status reserved, that is wire space kept for a deferred feature (duplication). Dedup rule (media rule 9): by stream, position and packet index; the first valid copy wins; there is no separate sequence number.
- C-12: **authentication reservation** in control messages: scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits.
- C-115 (settled): in v1 the scheme is 0 and the tag is zero; the 64-bit counter is live because it is the liveness counter.
- Which copy-index values a v1 receiver accepts is not settled by any constant; it is this ticket's decision (falsifier: the deferred duplication feature needs a value v1 receivers reject, forcing a major version).

Rejection reason available (004-03): `ReservedNonZero`.

## Work
1. Add a "Reserved fields" section to `wire.md` stating, for each reserved field: location (message and field name), width, required value in v1, receiver behaviour on a non-conforming value.
2. Copy index: state which values v1 senders emit and what receivers accept (all four values must be accepted to allow duplication, or only 0; decide and tag `[decision]`), and restate the dedup rule.
3. Authentication reservation (C-115): v1 senders set scheme = 0 and tag = 128 zero bits; the counter is live because it carries liveness; state the receiver outcome for scheme ≠ 0 and for a non-zero tag with scheme = 0 (`ReservedNonZero`, drop and count).
4. State the general rule for padding and reserved bits added by other tickets (sender zeroes; receiver outcome).
5. Mirror the rules in the machine-readable file as a "reserved value" attribute.

## Acceptance criteria
1. Copy index (2 bits, C-07) and authentication reservation (8 + 64 + 128 bits, C-12) each have a required v1 value and a receiver outcome.
2. The section states, citing C-115, that the 64-bit counter of C-12 is live in v1 because it is the liveness counter, while scheme and tag are zero.
3. A general rule for padding/reserved bits exists and names `ReservedNonZero` where it applies.
4. The machine-readable file carries the reserved values and passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Placing the copy index in the header layout (004-06) and the reservation in each control message (004-16 to 004-18).
- Any authentication scheme (later major/minor versions).
