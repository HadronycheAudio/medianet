#!/usr/bin/env bash
# Repository-state assertions of the ported document checks.
#
# When a checks/<id>.sh script is ported to a Rust test
# (hadronyche-tools/tests/doc_<id>.rs), its assertions about DOCUMENTS move
# to the test, and its assertions that RUN TOOLS or read REPOSITORY STATE
# (cargo runs, git refs, the proof of concept) move here, nothing dropped.
# Running a tool from `cargo test` would be circular.
#
# Layout (parallel-safe): this file runs each shared tool ONCE (the lead's
# ruling: the suite runs wire-check once, not once per ticket) and exposes
# the results as variables; each ported ticket has its own file
# scripts/repo-state.d/<id>.sh, sourced in name order, which asserts on
# those results with its own named lines (per-ticket pins kept). A porter
# adds a file; a new shared tool is one more block below.
#
# Exposed to the per-ticket files:
#   wire_check_rc, wire_check_out           cargo run --bin wire-check
#   wire_tables_rc, wire_tables_out         cargo test --test wire_tables
#   wire_tables_line, wire_tables_passed,   its last 'test result:' line
#   wire_tables_failed
#   wire_tables_sum_passed, _sum_failed,    totals over every result line
#   wire_tables_results                     how many result lines
#   pass <label> <msg>, bad <label> <msg>, tail_of <text>
#
# The scripts ran their cargo commands with a 900-second timeout; so do we
# (timeout(1) exit 124 is a failure like any other).
#
# Run from anywhere; exits 1 when any assertion fails. The per-change suite
# (scripts/per-change-suite.sh) runs it after the remaining checks/ scripts.

set -u
cd "$(dirname "$0")/.."
fail=0
TOOL_TIMEOUT=900

pass() { echo "PASS $1: $2"; }
bad() { echo "FAIL $1: $2"; fail=1; }
tail_of() { printf '%s\n' "$1" | tail -n 4 | tr '\n' ' '; }

# ------------------------------------------------------------ shared tool runs
# cargo run --bin wire-check (004-02's checker on doc/design/spec/wire.toml)
wire_check_out=$(timeout "$TOOL_TIMEOUT" cargo run --quiet -p hadronyche-tools --bin wire-check 2>&1)
wire_check_rc=$?
# cargo test -p hadronyche-tools --test wire_tables
wire_tables_out=$(timeout "$TOOL_TIMEOUT" cargo test -p hadronyche-tools --test wire_tables 2>&1)
wire_tables_rc=$?
wire_tables_line=$(printf '%s\n' "$wire_tables_out" | grep 'test result:' | tail -n 1)
wire_tables_passed=$(printf '%s\n' "$wire_tables_line" | sed -nE 's/.* ([0-9]+) passed; ([0-9]+) failed.*/\1/p')
wire_tables_failed=$(printf '%s\n' "$wire_tables_line" | sed -nE 's/.* ([0-9]+) passed; ([0-9]+) failed.*/\2/p')
wire_tables_sum_passed=$(printf '%s\n' "$wire_tables_out" | sed -nE 's/^test result: [a-zA-Z]+\. ([0-9]+) passed; ([0-9]+) failed.*/\1/p' | awk '{s+=$1} END {print s+0}')
wire_tables_sum_failed=$(printf '%s\n' "$wire_tables_out" | sed -nE 's/^test result: [a-zA-Z]+\. ([0-9]+) passed; ([0-9]+) failed.*/\2/p' | awk '{s+=$1} END {print s+0}')
wire_tables_results=$(printf '%s\n' "$wire_tables_out" | grep -c '^test result:')
export wire_check_rc wire_check_out wire_tables_rc wire_tables_out wire_tables_line wire_tables_passed \
    wire_tables_failed wire_tables_sum_passed wire_tables_sum_failed wire_tables_results

# ------------------------------------------------------------ per ticket
found=0
for f in scripts/repo-state.d/*.sh; do
    [ -e "$f" ] || continue
    found=1
    # shellcheck source=/dev/null
    . "$f"
done
if [ "$found" -eq 0 ]; then
    bad "repo-state" "no file under scripts/repo-state.d"
fi

# ------------------------------------------------------------ end
if [ "$fail" -ne 0 ]; then
    echo "FAIL: repository-state assertions failed"
    exit 1
fi
echo "OK: repository-state assertions pass"
