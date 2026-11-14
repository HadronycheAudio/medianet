# 022a-16 — Edge-only: hello never carries the holder-eligible bit
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** XS · **Depends on:** 022a-01, epic 018

## Context
- Part of epic 022a (Windows edge backend).
- **Edge only** `[decision]`: the holder-eligible capability bit is never set; the build uses the edge-only feature. The Windows edge never holds the clock and never stands for election.
- 022-AC-7: the hello never carries the holder-eligible bit.
- The hello carries only the regime-stamp version number, not the stamp (C-18); election rules (C-113) consider holder-eligible candidates only, so this edge never wins one.

## Work
1. Test in `hadronyche-os-windows` that builds the edge's hello message through the normal startup path and asserts the holder-eligible bit is clear.
2. A compile-time guard (e.g. `#[cfg(feature = "...holder...")] compile_error!`) so the crate cannot be built with holder eligibility.

## Acceptance criteria
1. **(022-AC-7.)** The hello produced by the Windows edge has the holder-eligible bit clear (unit test).
2. Building the crate with the holder feature enabled fails to compile.

## Definition of done
- [ ] Tests green in CI, unprivileged

## Out of scope
- The hello format itself (epic 018 / core).
