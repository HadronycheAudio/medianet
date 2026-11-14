# 028a-01 — Crate and CLI skeleton
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** XS · **Depends on:** none

## Context
Epic 028 builds the operator-facing commissioning tool, binary `hadronyche-commission`. Tool binaries live in the crate `hadronyche-tools` in the design set's repository (C-133); other epics put their harness binaries in the same crate, so this ticket adds the binary to that crate, creating the crate if it does not exist yet. Nothing is written into the frozen POC tree (C-130). The binary has these subcommands (each implemented by a later ticket of epic 028a or 028b):

| Subcommand | Purpose |
|---|---|
| `validate <config>` | offline configuration check |
| `survey` | online, read-only collection of regime stamps and findings |
| `propose-a <group>` | propose the minimum feasible audition offset A (read-only; `--apply` announces it inside a commissioning window, C-224) |
| `certify-sink <sink>` | single-sink marker measurement (E, D, restart spread), inside a commissioning window |
| `certify-group <group>` | group membership certificate |
| `burst-test`, `soak` | switch burst test; loss soak; inside a commissioning window |
| `watch` | post-hoc alarms (read-only) |
| `window open`, `window close` | announce or close a commissioning window on the control group (C-153) |

Hard rule for the whole tool (epic AC-10, C-107): outside a declared commissioning window that every module stamps, the tool only reads stamps and writes configuration (and sends the window announcement or close itself, C-153); inside the window it may drive test traffic and announce a commissioned change of A (C-224). It never takes part in a live (non-test) data path.

## Work
- Add binary target `hadronyche-commission` to crate `hadronyche-tools` in the design set's repository (C-133).
- Add a command-line parser with the nine subcommands above (`validate`, `survey`, `propose-a`, `certify-sink`, `certify-group`, `burst-test`, `soak`, `watch`, `window`), each a stub that prints "not implemented" and exits with a distinct non-zero code.
- `--version` prints the tool version (later embedded in certificates, ticket 028b-10).
- Define a common exit-code convention: 0 = success / no finding, 1 = refusal or finding, 2 = usage or I/O error. Document it in the crate's module doc.
- Add the binary to CI (build, `clippy`, tests), running unprivileged.

## Acceptance criteria
1. `hadronyche-commission --help` lists exactly the 9 subcommand names `validate`, `survey`, `propose-a`, `certify-sink`, `certify-group`, `burst-test`, `soak`, `watch`, `window`.
2. `hadronyche-commission --version` prints a version string matching the crate version.
3. Every stub exits with code 2 or a documented "not implemented" code, never 0.
4. CI builds and tests the binary as an unprivileged user.
5. The binary lives in `hadronyche-tools` in the design set's repository; no file under the frozen POC tree changes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The binary has no dependency on any runtime data-path module (send/receive of media slices)

## Out of scope
- Configuration parsing (028a-03), every subcommand's behaviour (028a-04 onward and the tickets of epic 028b), the commissioning-window guard (028b-08) and announcement (028b-09).
