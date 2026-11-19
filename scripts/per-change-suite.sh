#!/usr/bin/env bash
# Per-change suite (the "CI" of this repository).
# Unprivileged by definition (C-146): no real-time scheduling, no memory
# locking, no root. Run from the repository root.
#
# The orchestrator/integrator runs this after every merge to main; reviewers
# run it on the branch. Fuzz budgets (C-95) are scheduled separately
# (60 s per parser per change is the ticket's own line, nightly 1 h jobs are
# the integrator's).

set -u
cd "$(dirname "$0")/.."
fail=0

echo "== design set check =="
if ! python3 doc/kickoff/check-design.py doc/design; then
  echo "FAIL: design check"
  fail=1
fi

run_cargo() {
  local dir="$1"
  if [ -f "$dir/Cargo.toml" ]; then
    echo "== cargo test in $dir =="
    if ! (cd "$dir" && cargo test --quiet); then
      echo "FAIL: cargo test in $dir"
      fail=1
    fi
    echo "== cargo clippy in $dir =="
    if ! (cd "$dir" && cargo clippy --all-targets --quiet -- -D warnings); then
      echo "FAIL: cargo clippy in $dir"
      fail=1
    fi
  fi
}

# the root workspace (once it exists), then every top-level crate and
# every throwaway probe (C-133)
run_cargo .
for d in */; do
  [ -f "$d/Cargo.toml" ] || continue
  # skip members already covered by the root workspace run to save time:
  # cargo in a member dir errors if the workspace does not own it, so this
  # relies on the workspace excluding probes and listing its crates.
  run_cargo "$d"
done
# probes are nested one level deeper: probes/<name>/Cargo.toml
for d in probes/*/; do
  run_cargo "$d"
done

# function-length rule (design owner, 2026-11-14): "Every function that's
# over 100 lines of code must be split." Counting convention: code lines of
# the function body (non-blank, non-comment-only), enforced by
# scripts/function-length.py. Runs over every Rust source in this
# repository (target/, .git and doc/ excluded); the frozen POC at
# /home/dorian/workspace/hadropoc is out of scope.
echo "== function length (max 100 code lines) =="
if ! python3 scripts/function-length.py; then
  echo "FAIL: function length"
  fail=1
fi

# document checks (one executable per ticket under checks/, until the
# hadronyche-tools crate owns them; see swarm/README.md and
# hadronyche-tools/PORTING.md: a ported script is deleted, its test runs in
# the cargo test above)
if [ -d checks ]; then
  for c in checks/*.sh checks/*.py; do
    [ -e "$c" ] || continue
    echo "== check $c =="
    if [ "x${c##*.}" = xpy ]; then cmd="python3"; else cmd="bash"; fi
    if ! $cmd "$c"; then
      echo "FAIL: $c"
      fail=1
    fi
  done
fi

# the denial tests of the ported checks: ignored by the debug cargo test
# above, run here in release, every denial exactly once
# (hadronyche-tools/PORTING.md, "The denial list")
echo "== denial tests of the ported checks (release) =="
if ! cargo test --release --quiet -p hadronyche-tools --test doc; then
  echo "FAIL: denial tests of the ported checks (release)"
  fail=1
fi

# repository-state assertions of the ported checks: the tool runs and
# repository facts that checks/*.sh scripts asserted and that cannot live in
# `cargo test` (their document assertions are hadronyche-tools/tests/doc/c<id>.rs,
# run by the cargo test above); see hadronyche-tools/PORTING.md
echo "== repository-state checks =="
if ! bash scripts/repo-state-checks.sh; then
  echo "FAIL: scripts/repo-state-checks.sh"
  fail=1
fi

# conflict-marker guard: no merge resolution may leave markers behind
# (added after a leftover marker reached main in the 002-23 resolution)
echo "== conflict-marker guard =="
if grep -rlE '^(<<<<<<< |>>>>>>> |=======$)' doc/ checks/ swarm/*.md scripts/ 2>/dev/null | grep -v "^$"; then
  echo "FAIL: conflict markers present in the files above"
  fail=1
fi

exit $fail
