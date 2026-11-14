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
  fi
}

# the workspace, once it exists
run_cargo .

# every throwaway probe (C-133)
for d in probes/*/; do
  run_cargo "$d"
done

exit $fail
