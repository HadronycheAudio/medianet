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

# document checks (one executable per ticket under checks/, until the
# hadronyche-tools crate owns them; see swarm/README.md)
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

exit $fail
