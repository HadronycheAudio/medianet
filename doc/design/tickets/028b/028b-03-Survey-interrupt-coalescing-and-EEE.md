# 028b-03 — `survey`: interrupt coalescing, energy-efficient Ethernet and timestamp source
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-02, epic 019

## Context
Interrupt coalescing biases software receive stamps on every platform (bias measured at L1, epic 019). It must be disabled on medianet interfaces; the tool checks and records the state. Energy-efficient Ethernet (EEE) must also be checked. `survey` reports findings from stamps (028b-01, 028b-02).

The receive-timestamp source of each module is one of kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded) or interrupt (stamped in the receive interrupt, embedded), each reported with one of the four capability states present, absent, unverified, not requested (C-129, C-131). Coalescing matters most for the software sources (kernel, userspace).

## Work
- Finding `nic.coalescing_enabled`: a module whose stamp reports interrupt coalescing enabled on its medianet interface; name node, interface, reported setting and the module's receive-timestamp source (C-131).
- Finding `nic.eee_enabled`: same for EEE.
- Finding `nic.state_unknown`: the stamp does not report the state (e.g. platform cannot query it).
- Record the observed state and timestamp source per node in the survey output (`--json`) so it can be kept with the certificate (028b-10).
- If the stamp does not carry these fields, specify them for epic 013 with platform query methods (Linux ethtool coalesce/EEE settings; macOS and Windows per the findings of epics 021, 022).

## Acceptance criteria
1. On a fixture where one Linux node reports coalescing enabled on its medianet NIC, `survey` reports `nic.coalescing_enabled` naming the node, interface and timestamp source (epic AC-7, part).
2. On a real Linux host with coalescing enabled (`rx-usecs` > 0), `survey` reports it; after disabling it, the finding disappears. Changing the NIC setting needs a privilege, so this test runs in the separate privileged nightly job (C-146).
3. EEE enabled produces `nic.eee_enabled`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Changing NIC settings (operator does it per the checklists 028b-20 to 028b-23). Bias measurement itself (epic 019).

## Notes
- Which stamp field carries coalescing/EEE state is not yet specified (remaining gap, owner epic 013).
