# 028b-02 — `survey`: framework, identity collisions and path MTU
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-01

## Context
`hadronyche-commission survey` is online and read-only: it collects regime stamps (via 028b-01) and lists named findings. This ticket builds the finding framework and the first findings; 028b-03 to 028b-06 and 028b-07 add the others (coalescing/EEE, DSCP, unverified platforms, foreign multicast, capability states).

Facts:
- Stamps are refreshed every 10 s and on any regime change; a unicast request fetches one at once (C-18).
- Every connection probes its path MTU at admission with a probe pair sent at the intended media packet size (C-116); the fallback is 1 280 bytes (C-06), that is 1 252 bytes of UDP payload over IPv4 (C-100). The tool lists connections not at the interface MTU.
- A connection whose probe at the 1 280-byte fallback also fails is refused with reason PathMtuTooSmall (C-173); the runtime raises a connection alarm on refusal (C-173).
- A module id seen with two instance nonces is a collision; third parties quarantine it until only one nonce has been seen for 3 s (C-46).

## Work
- `survey` subcommand: collect stamps for a configurable window (default 11 s, one refresh period plus margin, C-18), run finding rules, print one line per finding with a stable id, exit 1 if any finding.
- Optional `--config <file>` to compare against declared modules (missing / undeclared modules become findings).
- Finding `identity.collision`: one module id with more than one instance nonce, naming the id and the nodes seen.
- Finding `mtu.below_interface`: a connection whose probed path MTU is below the interface MTU (including connections at the 1 280-byte fallback, C-06), naming both ends and both values.
- Finding `mtu.too_small`: a connection refused with PathMtuTooSmall (C-173), naming both ends.
- Machine-readable output `--json`.
- Mark every survey run with whether it was taken inside a commissioning window: the production measurements it reads from stamps (latency, loss, alignment figures) are unqualified while a window is open (C-178).

## Acceptance criteria
1. On a stamp fixture where one connection's path MTU is 1 280 and its interface MTU 1 500, `survey` reports `mtu.below_interface` naming the connection (epic AC-7, part).
2. On a fixture with no gaps, `survey` exits 0 with no findings.
3. A fixture with one module id under two nonces yields `identity.collision`.
4. With `--config`, a declared module with no stamp yields a missing-module finding.
5. A fixture with a PathMtuTooSmall refusal yields `mtu.too_small`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Capability states and strict-mode readiness (028b-07), coalescing/EEE (028b-03), DSCP (028b-04), platform combinations (028b-05), foreign multicast (028b-06).

## Notes
- Which stamp field carries the path MTU per connection is not yet specified (remaining gap, owner epic 013). If missing when this ticket starts, request it from epic 013.
