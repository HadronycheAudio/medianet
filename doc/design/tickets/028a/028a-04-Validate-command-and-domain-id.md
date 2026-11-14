# 028a-04 — `validate` command and domain id check
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** XS · **Depends on:** 028a-03

## Context
`hadronyche-commission validate <config>` is fully offline: it must run with no network. It is a list of independent checks over a parsed configuration; each check returns zero or more *refusals* (named, with the configuration objects involved) or *flags* (warnings). Later tickets add checks: scale (028a-05), ports (028a-06), trunks (028a-07), groups and E (028a-08).

Domain rule (C-11): the domain id is 16 bits, operator-configured; 0 means unconfigured, and strict mode refuses to start with it. The tool assigns and checks it.

## Work
- `validate` subcommand: read file, parse (028a-03), run the registered checks, print refusals and flags one per line with a stable identifier (e.g. `REFUSE domain.unconfigured`), exit 1 if any refusal, 0 otherwise.
- Check trait `Check { fn run(&Config) -> Vec<Finding> }` and a registry.
- Domain check: refuse a missing domain, `domain = 0` (C-11) and any value outside 1..=65535.
- Optional `--assign-domain` that writes a chosen non-zero id into the file when absent (the tool "assigns" the id).
- Test harness that runs `validate` inside a network namespace / sandbox with no network interfaces, or otherwise proves no socket is opened. If the sandbox needs a privilege, that test runs in the separate privileged nightly job and per-change CI uses the unprivileged socket audit (C-146).

## Acceptance criteria
1. `domain = 0`, a missing `domain` and `domain = 70000` are each refused with a named refusal; `domain = 17` passes.
2. `validate` succeeds on a valid fixture with no network available (test asserts no socket is created).
3. Exit code is 1 on any refusal, 0 on none.
4. `--assign-domain` writes a value in 1..=65535 and leaves the rest of the file semantically unchanged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal
- [ ] Every parser touched is fuzzed

## Out of scope
- Other checks (028a-05 to 028a-08, 028a-09). Runtime strict-mode behaviour is owned by the runtime, not this tool.
