#!/usr/bin/env bash
# Repository-state assertions of the ported document checks.
#
# When a checks/<id>.sh script is ported to a Rust test
# (hadronyche-tools/tests/doc_<id>.rs), its assertions about DOCUMENTS move
# to the test, and its assertions about REPOSITORY STATE or that RUN TOOLS
# (cargo runs, git refs, the proof of concept) move here, per ticket id, with
# nothing dropped. Running a tool from `cargo test` would be circular (the
# test would run cargo on itself), so they run once per suite run instead.
#
# Each tool runs ONCE (the lead's ruling: deduplicated, the suite runs
# wire-check once, not once per ticket); each ticket keeps its own named
# assertion on the shared result, including any per-ticket pin such as
# "at least 12 passed".
#
# Run from anywhere; exits 1 when any assertion fails. The per-change suite
# (scripts/per-change-suite.sh) runs it after the remaining checks/ scripts.

set -u
cd "$(dirname "$0")/.."
fail=0

pass() { echo "PASS $1: $2"; }
bad() { echo "FAIL $1: $2"; fail=1; }

# ------------------------------------------------------------ shared tool runs
# cargo run --bin wire-check (004-02's checker on doc/design/spec/wire.toml)
wire_check_out=$(cargo run --quiet -p hadronyche-tools --bin wire-check 2>&1)
wire_check_rc=$?
# cargo test -p hadronyche-tools --test wire_tables
wire_tables_out=$(cargo test -p hadronyche-tools --test wire_tables 2>&1)
wire_tables_rc=$?
wire_tables_line=$(printf '%s\n' "$wire_tables_out" | grep 'test result:' | tail -n 1)
wire_tables_passed=$(printf '%s\n' "$wire_tables_line" | sed -nE 's/.* ([0-9]+) passed; ([0-9]+) failed.*/\1/p')
wire_tables_failed=$(printf '%s\n' "$wire_tables_line" | sed -nE 's/.* ([0-9]+) passed; ([0-9]+) failed.*/\2/p')
# the passed/failed totals over every 'test result:' line (004-22 sums them)
wire_tables_sum_passed=$(printf '%s\n' "$wire_tables_out" | sed -nE 's/^test result: [a-zA-Z]+\. ([0-9]+) passed; ([0-9]+) failed.*/\1/p' | awk '{s+=$1} END {print s+0}')
wire_tables_sum_failed=$(printf '%s\n' "$wire_tables_out" | sed -nE 's/^test result: [a-zA-Z]+\. ([0-9]+) passed; ([0-9]+) failed.*/\2/p' | awk '{s+=$1} END {print s+0}')
wire_tables_results=$(printf '%s\n' "$wire_tables_out" | grep -c '^test result:')

tail_of() { printf '%s\n' "$1" | tail -n 4 | tr '\n' ' '; }

# ------------------------------------------------------------ 004-22
# From checks/004-22.sh (DoD, "when not overridden"):
#   `cargo run --bin wire-check` exits 0, and `cargo test -p hadronyche-tools
#   --test wire_tables` reports at least 12 passed and 0 failed (the count
#   that stood at 004-22), summed over every 'test result:' line.
if [ "$wire_check_rc" -eq 0 ]; then
    pass "004-22 DoD" "cargo run --bin wire-check exits 0"
else
    bad "004-22 DoD" "cargo run --bin wire-check did not pass: $(tail_of "$wire_check_out")"
fi
if [ "$wire_tables_results" -eq 0 ]; then
    bad "004-22 DoD" "the wire_tables test run printed no 'test result:' line"
elif [ "$wire_tables_sum_failed" -eq 0 ] && [ "$wire_tables_sum_passed" -ge 12 ]; then
    pass "004-22 DoD" "wire_tables reports $wire_tables_sum_passed passed (at least 12) and 0 failed"
else
    bad "004-22 DoD" "wire_tables must report at least 12 passed and 0 failed; got $wire_tables_sum_passed passed, $wire_tables_sum_failed failed"
fi

# ------------------------------------------------------------ end
if [ "$fail" -ne 0 ]; then
    echo "FAIL: repository-state assertions failed"
    exit 1
fi
echo "OK: repository-state assertions pass"
