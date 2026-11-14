# 028b-09 — `window open` and `window close` command
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-01, 028b-08, epic 013

## Context
Test traffic is allowed only inside a declared commissioning window that every module stamps; outside it the tool is read-only (C-107). The window life cycle is settled (C-153): the tool announces it on the control group with an expiry time; modules stamp it while it is open; it closes at expiry or on explicit close. While it is open, ordinary production measurements in stamps are unqualified; the tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window (C-178).

The control group is IPv4 multicast with TTL 1 (C-100); control traffic is marked CS5 (40) (C-13); a control message must fit 1 252 bytes of UDP payload (C-17). Stamps are multicast on any regime change, refreshed every 10 s, and fetched at once by unicast request (C-18), so the tool can confirm within one fetch that every module stamps the window.

The announcement and the close are the only messages the tool sends outside a window, besides stamp fetch requests (epic AC-10). Inside an open window the tool may also announce a commissioned change of a sink's own A (C-224, `propose-a --apply`, 028b-11). Their encoding, and the stamp field that shows the window and its expiry, are for epic 013 to specify (remaining gap, with epic 004 for the encoding).

## Work
- `window open --duration <d>`: send the announcement on the control group with the expiry time; then fetch stamps (028b-01) and report, through the guard (028b-08), which modules in scope stamp the window and which do not. If some do not after a fresh fetch, re-send the announcement once and report the modules still missing. There is no default duration: the operator states it.
- `window close`: send the explicit close; confirm from stamps that no module still stamps the window.
- `window status`: read-only; print the window and expiry each module stamps.
- Encode the messages with the types epic 013 provides (C-133: protocol types live in `hadronyche-core`); if they are missing, request them from epic 013 rather than defining them here.
- Fixture tests with recorded stamps; a socket audit showing only the announcement, the close and stamp fetches are sent.

## Acceptance criteria
1. On a lab rig, `window open --duration 1h` makes every running module stamp the window within one stamp fetch, and `window status` shows the same expiry for all of them.
2. After `window close`, no module stamps the window; with no close, the stamps stop showing it at expiry (C-153).
3. A module that does not stamp the window after the re-send is named in the output.
4. The socket audit shows nothing sent besides the announcement, the close and stamp fetch requests (epic AC-10).
5. `window open` without `--duration` is a usage error.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The guard that test commands use (028b-08). Defining the announcement encoding or the stamp field (epic 013). Module-side handling of the window (runtime epics).
