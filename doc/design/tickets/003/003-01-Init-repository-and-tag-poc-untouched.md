# 003-01 — Init repository and tag poc-untouched
**Epic:** 003-POC-smoke-fixes · **Effort:** XS · **Depends on:** none

## Context
- Epic 003 (pre-implementation check **0c**) turns the proof of concept **hadropoc** into a citable baseline. hadropoc lives at `/home/dorian/workspace/hadropoc` (Rust, one binary, dependencies clap, libc, ratatui, rtrb) and is frozen as the reference instrument. Epic 003 is the only task allowed to modify it.
- The directory is not a git repository today `[measured]`. Before any fix is applied, the untouched tree must be committed and tagged so that later diffs show exactly what the fixes changed.
- `.gitignore` belongs to this first commit (epic deliverable 1). Epic criterion 003-AC1 therefore does not expect it in the diff between `poc-untouched` and `poc-baseline`.
- The POC has two unit tests, both in `transport.rs` `[code]`.

## Work
1. In `/home/dorian/workspace/hadropoc`: `git init`.
2. Add `.gitignore` excluding `target/` and `experiments/*/`.
3. Commit the untouched source tree (plus `.gitignore`) as the first commit. Do not change any source file.
4. Create the annotated tag `poc-untouched` on that commit (`git tag -a poc-untouched -m ...`).
5. Run `cargo test` on the tagged tree and record that the two `transport.rs` tests pass, as the reference for later tickets.

## Acceptance criteria
1. `git -C /home/dorian/workspace/hadropoc tag` lists `poc-untouched`, and `git cat-file -t poc-untouched` prints `tag` (annotated, not lightweight).
2. `git ls-files` contains no path under `target/` and none under `experiments/<dir>/`.
3. `git show --stat poc-untouched` shows exactly 1 commit with no parent, whose only file not present before the ticket is `.gitignore`.
4. `cargo test` at `poc-untouched` runs 2 tests in `transport.rs`, both passing.

## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No source file of the POC differs from its pre-ticket content

## Out of scope
- Any code fix (003-03, 003-04, 003-06).
- The second commit and the `poc-baseline` tag (003-09).
- Measuring the pre-fix sync rate (003-02).
- Adding CI to the POC (forbidden by epic criterion 003-AC1).
