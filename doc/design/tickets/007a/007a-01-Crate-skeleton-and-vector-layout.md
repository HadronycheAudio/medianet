# 007a-01 — Crate skeleton and vector layout
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** XS · **Depends on:** none

## Context
- Epic 007 produces the implementation-agnostic conformance data for the Hadronyche Medianet protocol. Any implementation, hosted or embedded, is judged against it. Both profiles (hosted and embedded) are built from one `no_std` core crate, so the vector crate must be usable without `std` `[decision]`.
- All conformance data lives in one crate, `hadronyche-vectors`: data files plus a tiny `no_std` reader. A hosted-only generator, `gen`, is a tool binary and lives in the `hadronyche-tools` crate of the design set's repository (C-133).
- `hadronyche-vectors` sits at the workspace root of the design set's repository (C-192); record this in the crate's top-level doc comment. `conformance.md` lives at `doc/design/spec/conformance.md` and results at `doc/design/reports/` (C-130).
- The data directories fixed by the epic are:
  - `vectors/wire/valid/` (`*.bin` + `*.json`)
  - `vectors/wire/invalid/` (`*.bin` + `*.json`)
  - `vectors/control/` (`*.json`)
  - `vectors/timeline/` (`*.json`)
  - `vectors/transport/` (`*.json`)
  - `vectors/reference/` (any file type)
- This ticket only creates the empty structure and the CI job; later tickets of epic 007 fill it.

## Work
1. Create crate `hadronyche-vectors` with `#![no_std]` in `src/lib.rs` (no dependencies yet), at the workspace root of the design set's repository (C-192).
2. Create the six data directories listed above, each with a placeholder so it is tracked.
3. Add the hosted-only binary `gen` to the `hadronyche-tools` crate (C-133; create the crate if it does not exist yet; `std` allowed) with an empty `main` that exits 0.
4. Add a CI job that builds `hadronyche-vectors` for a `no_std` target and the host, builds `gen`, and runs `cargo test` for both, without root or special capabilities.

## Acceptance criteria
1. `hadronyche-vectors` builds with `#![no_std]` on the host and on at least one target without `std`.
2. `gen` in `hadronyche-tools` builds and runs on the host and exits with status 0.
3. All 6 directories listed in Context exist in the repository.
4. The CI job runs as an unprivileged user and is green.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Crate and directory names exactly match those in Context

## Out of scope
- The vector file formats and the version stamp (007a-02).
- The `no_std` reader (007a-03), the runner trait (007a-04), the generator logic (007a-05 onward).
