# 028a-05 — `validate`: scale limits and holder-capable node
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** XS · **Depends on:** 028a-04

## Context
A medianet has hard scale limits, and the tool refuses configurations above them:
- At most **64 active modules** per medianet; commissioning refuses more (C-34).
- At most **64 followers per holder** (C-35). Every module other than the holder follows it, so with at most 64 modules a holder has at most 63 real followers (C-114). The module limit therefore enforces the follower limit; there is no separate follower check. The holder is elected at run time (C-113) and is not named in the configuration.
- The same limit is why every module's peer map holds 63 entries: a module holds no entry for itself (C-217). A 65th module could not be held by a full map, so the module limit is the only check.
- At least one **holder-capable** node must exist. Linux hosts are the backbone (clock holder, elections, conformance, commissioning); macOS and Windows machines are edges; embedded devices are followers. Holder-capable means Linux.

## Work
- Check `scale.modules`: count modules declared in the configuration; refuse if > 64 (C-34). Document in the check that this also bounds followers per holder at 63 (C-114).
- Check `scale.no_holder`: refuse if no node has OS Linux.
- Fixtures and tests at 64 modules (pass) and 65 (refuse), and for the no-holder case.

## Acceptance criteria
1. A configuration with 64 modules passes this check; with 65 modules it is refused (epic AC-3).
2. A configuration whose nodes are only macOS, Windows and embedded is refused as having no holder-capable node (epic AC-3).
3. A 64-module configuration produces no follower refusal (63 followers at most, C-114).
4. All tests run with no network.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal

## Out of scope
- Runtime elections and holder selection (other epics). The 64-follower load and refusal of a 65th follower, which the runtime tests with synthetic followers (C-114). Whether a module is "active" at run time; offline, every declared module counts.
