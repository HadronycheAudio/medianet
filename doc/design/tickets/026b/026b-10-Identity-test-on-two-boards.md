# 026b-10 — Identity test on two boards
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** XS · **Depends on:** 026b-02

## Context

Identity: module id 64 bits, instance nonce 64 bits, domain id 16 bits with 0 = unconfigured (C-11). Devices without a true random source must not draw the id from a PRNG; they mix a hardware unique id with a persistent boot counter `[decision]`. The module id is stable across reboots and the nonce changes on every boot; third parties quarantine an id seen with two nonces until only one has been seen for 3 s (C-46, C-31).

Epic criterion 026-AC-4 (Identity): with the true-random source **disabled**, two devices flashed with **identical firmware** boot with **different ids and nonces**; **rebooting one keeps its id and changes its nonce**.

The firmware (026b-02) has a build flag to disable the true RNG.

## Work

1. Build one firmware image with the true RNG disabled; flash it to two boards.
2. Boot both on a medianet with a Linux holder; read their (module id, nonce) from the holder's map or the stamp collector.
3. Reboot board A three times; record ids and nonces, and how long the holder quarantines board A's id after each reboot.
4. Store the result in `doc/design/reports/` (C-130).

## Acceptance criteria

1. Board A and board B have different module ids and different nonces.
2. Each of board A's 3 reboots keeps its module id and yields a nonce different from all previous ones.
3. After each reboot, board A's id is usable again once only the new nonce has been seen for 3 s (C-46).
4. Binary images on both boards have the same hash (identical firmware).

## Definition of done

- [ ] Every new factual claim carries an evidence tag
- [ ] Result (ids, nonces, image hash) recorded

## Out of scope

- Derivation logic and host tests (026a-03).
