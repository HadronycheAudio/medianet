# 019-01 — Crate and CLI skeleton
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** epic 015, epic 017

## Context
Epic 019 builds L1, the second rung of the test ladder `[decision]`: L0 = loopback in one process (task 017), **L1 = two Linux hosts on one wire through one switch**, L2 = a reference sink per platform (task 020), L3 = hosted plus embedded sink (task 027). L1 is the first place where packets cross a real NIC, switch, interrupt path and second crystal; loopback has an MTU of 65 536 and none of these.

The whole epic is delivered as crate `hadronyche-tools`, binary `hadronyche-l1`. The binary orchestrates two hosts over SSH, runs a named scenario, collects both hosts' record files and stamps and produces one report. This ticket creates only the crate, the binary and its command-line surface, so that later tickets can plug in orchestration (019-02), stamps (019-03), reporting (019-04) and scenarios (019-07 onward).

The scenario names fixed by the epic are: `baseline`, `two-bundles`, `asymmetry`, `coalescing`, `multi-packet`, `blackhole`, `serving`, `stress`, `rcvbuf-shrink` (AC-37, 019-18), `flood` (AC-11, 019-19) and `loss-soak`.

Settled rules this ticket applies: IP is IPv4 only in v1, multicast with TTL 1 (C-100); each parser is fuzzed 60 s on every change and 1 hour nightly, and the corpus is kept (C-95).

## Work
- Create crate `hadronyche-tools` in the design set's repository, where harness and tool binaries live (C-133), with binary target `hadronyche-l1`. Run output defaults to a directory outside the repository; published results go to `doc/design/reports/` (C-130), never into the frozen POC tree.
- CLI (subcommands):
  - `hadronyche-l1 run <scenario> --hosts <file> [--dry-run] [--out <dir>]`
  - `hadronyche-l1 list` — prints every registered scenario name with a one-line description.
  - `hadronyche-l1 report <dir>` — regenerates the report from an existing results directory (implementation in 019-04; here a stub returning a clear "not implemented" error).
- Hosts file: a small TOML (or the workspace's existing config format) naming exactly two hosts, `a` and `b`, each with SSH address, user, remote work directory, the network interface under test and that interface's IPv4 address. Parse it into a typed `HostPair` struct with validation (exactly two hosts, distinct addresses, interface name non-empty, test address IPv4; an IPv6 test address is rejected because v1 is IPv4 only, C-100).
- A `Scenario` trait (name, description, `plan(&HostPair) -> Plan`, `evaluate(&Results) -> Verdict`) and a registry. Register a no-op scenario `noop` used by tests only (hidden from `list` unless `--all`).
- `--dry-run` flag plumbed through to a `Mode::{Real, DryRun}` value passed to every scenario; no behaviour behind it yet beyond being visible in the run log.
- Unit tests for hosts-file parsing and the registry; a fuzz target for the hosts-file parser (`cargo fuzz` target `hosts_file`).

## Acceptance criteria
1. `cargo build -p hadronyche-tools` produces binary `hadronyche-l1`.
2. `hadronyche-l1 list` exits 0 and prints at least the scenario `noop` when run with `--all`.
3. A hosts file with 1 or 3 hosts, duplicated addresses, an empty interface name, or an IPv6 test address is rejected with a non-zero exit code and an error naming the offending field (one unit test per case, 5 tests).
4. `hadronyche-l1 run unknown-name ...` exits non-zero and lists the valid scenario names.
5. The `hosts_file` fuzz target runs 60 s on every change and 1 hour nightly without a crash, and its corpus is kept in the repository (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Crate builds without warnings on the workspace's CI toolchain

## Out of scope
- SSH, deployment and process control (019-02).
- Stamp collection (019-03) and report generation (019-04).
- Any real scenario (019-07 to 019-20, 019-23 to 019-25).
- The CI dry-run job on loopback (019-05).
