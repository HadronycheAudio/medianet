# 028b-07 — `survey`: capability states and strict-mode readiness
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028a-03, 028b-02

## Context
`survey` (028b-02) lists named findings from regime stamps. This ticket adds the capability findings, which the design rulings now define:
- Each capability is stamped in one of four states: **present, absent, unverified, not requested**. "Not requested" marks a deliberate configuration choice, as distinct from a denial (C-129).
- Strict mode requires real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped (C-127).
- In strict mode an unverified capability is refused: unverified is treated as absent (C-128).
- Receive-timestamp sources (kernel, userspace, mac-hardware, interrupt) are reported with the same four states (C-131).
- Strict mode refuses to start with domain id 0 (C-11).
- Strict mode is hosted-only: a device stamps real-time scheduling and memory lock as present by construction (C-147).
- The control-isolation capability means housekeeping cannot delay the data path; it is present on hosted platforms by construction and absent or unverified on a device unless the implementer shows a bound (C-161).
- The commissioning configuration sets strict mode (C-165, field from 028a-03).
- A module whose thread layout's seqlock safety depends on a CPU pin refuses to start in every mode, strict or not, if that pin fails (C-177) or is unverified (C-201). Default priorities (clock above data) never depend on pinning. Such a module publishes no stamp, so it is not a capability gap: `survey --config` reports it as a missing module (028b-02).

A *capability gap* is therefore a capability in state absent or unverified. "Not requested" is reported for information only.

## Work
- Finding `capability.gap`: a module with a capability in state absent or unverified; name node, module, capability and state. Severity: refusal-level for real-time scheduling or memory lock on a module whose stamp shows strict mode (C-127, C-128); warning otherwise.
- Notice `capability.not_requested`: informational, listing capabilities deliberately not requested.
- Finding `strict.domain_unconfigured`: a module in strict mode on domain 0 (C-11).
- Finding `strict.config_mismatch` (with `--config`): a node whose stamped strict-mode state differs from the configuration (C-165).
- Include the receive-timestamp source and its state per module in `--json` output.

## Acceptance criteria
1. A fixture with a strict-mode module whose memory lock is `unverified` yields a refusal-level `capability.gap` (epic AC-7, part).
2. The same capability `unverified` on a non-strict module yields a warning-level `capability.gap`.
3. A capability in state `not requested` yields only the informational notice, and `survey` exits 0 if it is the only output.
4. A module whose CPU-idle restriction is absent yields a warning, never a refusal, even in strict mode (C-127).
5. With `--config`, a node configured strict whose stamp shows strict mode off yields `strict.config_mismatch`.
6. A device whose control-isolation capability is unverified yields a warning-level `capability.gap` (C-161).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Checking the configured strict mode offline (028a-09). Coalescing and EEE (028b-03).
