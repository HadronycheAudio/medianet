# 028a-07 — `validate`: trunk capacities per direction
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** S · **Depends on:** 028a-02, 028a-03, 028a-04

## Context
With several switches, trunk (switch-to-switch) links are invisible to endpoints, so **the runtime never computes trunk feasibility** `[decision]`. Trunk capacities are declared per link in the commissioning configuration (`[[link]] name, a, b, rate_mbit`, 028a-03), and each `[[connection]]` lists the trunks it crosses in `path`. A declared capacity is **per direction** (C-117). The tool validates the requested graph against the declared trunks offline and refuses a configuration that exceeds the ceiling in either direction.

Direction: each node declares the switch it attaches to (028a-03). A connection starts at the producer's switch; walking `path` in order, each link is crossed from the switch the walk is at towards the other one (`a`→`b` or `b`→`a`).

Facts: per-source wire rates from `capacity` (028a-02): 6 ch = 21.1 Mbit/s, 24 ch = 84, 48 ch = 169, 96 ch = 338 Mbit/s (C-21, C-22). Admission ceiling 70 % of link rate (C-23). No multicast fan-out in v1: each connection is a separate copy. A connection from an output with a controller stream also carries that stream, at one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of overhead), 2 000 packets/s (C-219).

## Work
- Check `trunk.unknown`: refuse a connection whose `path` names an undeclared link.
- Check `trunk.path_broken`: refuse a connection whose `path` does not form a chain from the producer's switch to the consumer's switch.
- Check `trunk.ceiling`: for each declared link and each direction, sum the wire rate of every connection crossing it in that direction, controller streams included (C-219) and refuse when the sum > 70 % of `rate_mbit`. The refusal names the link, the direction, its load, its ceiling and every contributing connection.
- Tests for each refusal.

## Acceptance criteria
1. A graph that puts 800 Mbit/s in one direction on a declared 1 000 Mbit/s trunk is refused offline (ceiling 700 Mbit/s per direction), naming the link, the direction and the connections (epic AC-1).
2. 690 Mbit/s in one direction on the same trunk passes; 400 Mbit/s in each direction (800 in total) passes (C-117).
3. A path naming an undeclared link is refused, naming the connection and the link; a path that does not chain is refused.
4. A connection whose source has a controller stream adds that stream's C-219 rate to every trunk direction it crosses.
5. All tests run with no network.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal

## Out of scope
- Post-hoc trunk alarms from observed rates (028b-17). Endpoint ports (028a-06).
