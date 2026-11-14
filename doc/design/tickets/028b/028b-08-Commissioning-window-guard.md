# 028b-08 — Commissioning-window guard
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-01, epic 013

## Context
Settled rule (C-107): the commissioning tool may drive test traffic only inside a declared commissioning window that every module stamps; outside it, the tool is read-only (it reads stamps and writes configuration). Test activity in this epic is: `burst-test` (028b-15), `soak` (028b-16), the marker loop and restarts of `certify-sink` (028b-12), and the pairwise capture of `certify-group` (028b-14). The announcement of a commissioned change of A by `propose-a --apply` (028b-11) is also allowed only inside a window (C-224). `survey`, `propose-a` without `--apply`, `validate` and `watch` are read-only and need no window.

Regime stamps are multicast on any regime change and refreshed every 10 s; a unicast request fetches one at once (C-18). A module that has not stamped the window, or whose stamp cannot be read, must be treated as outside it. Test-only injection is stamped like any injection (C-120), so test traffic is visible in the stamps of the modules that see it.

Window life cycle (C-153): the tool announces the window on the control group with an expiry time (028b-09); modules stamp it while it is open; it closes at expiry or on explicit close. While it is open, ordinary production measurements in stamps are unqualified; the tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window (C-178). Which stamp field shows the window and its expiry is for epic 013 to specify (remaining gap); the guard isolates that dependency in one predicate.

## Work
- Module `hadronyche_tools::window`: `WindowGuard::check(scope, run_duration) -> Result<(), NotInWindow { modules } | WindowTooShort { expiry }>`, where scope is every module in the domain (from stamps, 028b-01, and the configuration when given). It fetches a fresh stamp for any module whose latest stamp is older than one refresh period (C-18), and refuses when the stamped expiry falls before the end of the planned run (C-153).
- `WindowGuard::watch()`: during a run, signal as soon as any module's stamp shows the window closed, the expiry passes, or the module disappears, so the caller stops its test traffic.
- A single accessor `stamp_window(&Stamp) -> Option<WindowState { id, expiry }>` isolating the dependency on the stamp field epic 013 specifies.
- Fixture tests: all modules in window; one module out; one module silent; window closing mid-run; window expiring before the planned run ends.

## Acceptance criteria
1. With every module's stamp showing the window open, `check` passes.
2. With one module outside the window or silent, `check` fails naming that module.
3. A fixture where the window closes mid-run makes `watch` signal within one stamp refresh (10 s, C-18) of the closing stamp being multicast.
4. The guard itself sends nothing but stamp fetch requests.
5. A window whose stamped expiry is earlier than the end of the planned run fails `check` with the expiry named (C-153).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Opening or closing a window (028b-09). The commands that use the guard (028b-12, 028b-14, 028b-15, 028b-16).
