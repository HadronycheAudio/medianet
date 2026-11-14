# 021-07 — Local-network permission detection
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-04
## Context
Porting trap [memory]: recent macOS releases gate local-network access behind a **per-app permission prompt**; until granted, traffic is silently blocked. The edge must not run silently deaf: a missing permission is a named startup failure.

- The detection method is not fixed by the design. This ticket implements a self-addressed multicast probe as the working method; 021-08 confirms on hardware that it fires in every launch mode where traffic is blocked (falsifier recorded in the epic's open sub-questions: a blocked edge passes the probe).
- The probe is IPv4 multicast with TTL 1 on the pinned medianet interface (C-100).
## Work
- `permissions.rs`: `fn check_local_network() -> Result<(), StartupFailure>` run at startup after the medianet sockets are opened (021-04).
- Detection: a self-addressed multicast probe on the pinned medianet interface (send, expect loopback receipt within a bounded wait, e.g. 500 ms), plus any error code macOS returns when access is denied. Document the method chosen and its evidence tag.
- On failure, return the named failure `LocalNetworkPermissionMissing` and log a remediation hint (System Settings path to grant the permission).
- Hook the check into the backend's startup path so the edge does not announce itself when it fails.
## Acceptance criteria
1. With permission denied (manual hardware test, documented), startup fails with `LocalNetworkPermissionMissing` within 2 s and no hello is sent.
2. With permission granted, the check passes and adds less than 1 s to startup.
3. Unit test with a mocked socket that never receives the probe: the named failure is returned.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Whether the prompt appears for a command-line binary and how to pre-grant it (021-08). The commissioning checklist (epic 028).
