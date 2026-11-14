# 029-24 — Id collision and departure marks
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-06, 029-11

## Context
- `hadronyche-monitor` (epic 029) shows who is on the medianet from hellos (period 1 s, C-30), each carrying a module id and an instance nonce (64 / 64 bits, C-11).
- **Id collision** (C-46): a module id seen with two instance nonces is quarantined by third parties until only one instance nonce has been seen for one hello-expiry period (3 s, C-31). The monitor is a third party and must show the collision rather than merge the two instances into one row. A collision can also be forced by the test-only hook of C-120, in which case it is stamped as an injection.
- **Departure** (C-124): a module that leaves — including a laptop edge going to sleep — and returns through normal admission (Locking, Aligning). A departure is not a fault and must not be shown as one.
- Presence: a peer is expired after 3 s without hellos (C-31). The monitor shows a module absent on the same rule and never removes it silently.
- The monitor derives these marks only from the hellos it observes; it sends nothing and changes no module's quarantine.

## Work
1. In `model.rs`, per module id, keep the set of instance nonces seen with the time each was last seen; `collision: bool` while more than one nonce has been seen within the last 3 s (C-46, C-31), cleared after 3 s with one nonce.
2. Presence per module: present while a hello arrived within the last 3 s (C-31) of observation time; otherwise absent, with the time it was last seen.
3. Departure and return: when an absent module reappears, show "returned" with its clock and connection states (Locking, Aligning) from the next stamp, in neutral style; a new instance nonce on return is a restart, not a collision, once the old nonce has gone silent for 3 s.
4. Medianet panel (029-11): an "id collision (quarantined)" mark in error style on the affected row, listing both nonces; an "absent since" mark; a "returned" mark.
5. Text capture for the documentation.

## Acceptance criteria
1. Hellos for one module id with two nonces interleaved render one row marked "id collision" with both nonces; after 3 s of hellos with only one nonce the mark clears.
2. A module whose hellos stop is shown absent 3 s after its last hello and stays listed.
3. A module that returns after 60 s of absence with a new nonce is shown "returned", not as a collision, and not in an error style.
4. All times are observation times, so the marks render identically from a recorded file (029-19).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Quarantine itself (runtime, C-46); the rest of the Medianet panel (029-11).
