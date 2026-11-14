# 024a-02 — ABI version check with named reason
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** XS · **Depends on:** 024a-01

## Context
Every module carries `ModuleInfo.abi_version` (u16). The current version is `pub const ABI_VERSION: u16 = 1;` in `hadronyche-core::abi`. A runtime must refuse a module built against a different ABI version and say why (epic acceptance criterion 024a/AC-4: "A module built against `ABI_VERSION` 1 is rejected by a runtime expecting another version, with a named reason.").

## Work
- In `hadronyche-core/src/abi/`, add the check specified by the epic, usable by the runtime (epic 009) before a module is loaded:
  ```rust
  pub enum AbiError { VersionMismatch { module: u16, runtime: u16 } }
  pub fn check_module_info(info: &ModuleInfo, runtime_abi: u16) -> Result<(), AbiError>;
  ```
- `AbiError` implements `core::fmt::Display` with a stable, named reason (e.g. `abi version mismatch: module 1, runtime 2`).
- Tests build a module with `abi_version: 1` and check it against runtime versions 1 and 2.

## Acceptance criteria
1. `check_module_info(&M::INFO, 1)` returns `Ok(())` for a module with `abi_version` 1.
2. `check_module_info(&M::INFO, 2)` returns `Err(AbiError::VersionMismatch { module: 1, runtime: 2 })`.
3. The `Display` text of that error contains both version numbers and a fixed reason name; a test asserts the exact string.
4. The function is `no_std`, allocation-free.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Wiring the check into the runtime's module loader (epic 009).
- Any other compatibility rule (minor versions, feature flags): not in this epic.
