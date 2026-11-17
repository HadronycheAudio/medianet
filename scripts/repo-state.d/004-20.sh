# Relocated repository-state assertions of checks/004-20.sh (AC6 and DoD, script
# lines 1495-1539, run "when the paths are not overridden").
# Sourced by scripts/repo-state-checks.sh, which defines pass, bad, tail_of
# and runs each tool once (wire_check_rc/out, wire_tables_rc/out/line). Run
# by hand (`bash scripts/repo-state.d/004-20.sh` from the repository root),
# it defines them and runs the tools itself.
if ! command -v pass >/dev/null 2>&1; then
    fail=0
    standalone_00420=1
    pass() { echo "PASS $1: $2"; }
    bad() { echo "FAIL $1: $2"; fail=1; }
    tail_of() { printf '%s\n' "$1" | tail -n 4 | tr '\n' ' '; }
fi
if [ -z "${wire_check_rc:-}" ]; then
    wire_check_out=$(timeout 900 cargo run --quiet -p hadronyche-tools --bin wire-check 2>&1)
    wire_check_rc=$?
fi
if [ -z "${wire_tables_rc:-}" ]; then
    wire_tables_out=$(timeout 900 cargo test -p hadronyche-tools --test wire_tables 2>&1)
    wire_tables_rc=$?
    wire_tables_line=$(printf '%s\n' "$wire_tables_out" | grep 'test result:' | tail -n 1)
fi
# `cargo run --bin wire-check`: exit 0 and "checks clean" in its output.
if [ "$wire_check_rc" -eq 0 ] && printf '%s\n' "$wire_check_out" | grep -q 'checks clean'; then
    pass "004-20 AC6" "cargo run --bin wire-check reports checks clean"
else
    bad "004-20 AC6" "cargo run --bin wire-check did not pass: $(tail_of "$wire_check_out")"
fi
# `cargo test -p hadronyche-tools --test wire_tables`: exit 0; a result line;
# its counts readable; the LAST result line exactly 12 passed, 0 failed, 0
# ignored (each a separate assertion of the script).
wt_counts=$(printf '%s\n' "${wire_tables_line:-}" | sed -nE 's/.* ([0-9]+) passed; *([0-9]+) failed; *([0-9]+) ignored.*/\1 \2 \3/p')
if [ "$wire_tables_rc" -ne 0 ]; then
    bad "004-20 AC6" "cargo test -p hadronyche-tools --test wire_tables failed: $(tail_of "$wire_tables_out")"
fi
if [ -z "${wire_tables_line:-}" ]; then
    bad "004-20 AC6" "the wire_tables test run printed no 'test result:' line"
elif [ -z "$wt_counts" ]; then
    bad "004-20 AC6" "could not read the test counts from: $wire_tables_line"
elif [ "$wt_counts" = "12 0 0" ]; then
    pass "004-20 AC6" "wire_tables stays 12 passed, 0 failed, 0 ignored"
else
    bad "004-20 AC6" "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: $wire_tables_line"
fi
# `python3 doc/kickoff/check-design.py doc/design`: exit 0 and "OK" as the
# last line of its output (run here unless the sourcing script ran it).
if [ -z "${design_check_rc:-}" ]; then
    design_check_out=$(timeout 300 python3 doc/kickoff/check-design.py doc/design 2>&1)
    design_check_rc=$?
fi
design_last=$(printf '%s\n' "$design_check_out" | sed '/^[[:space:]]*$/d' | tail -n 1 | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
if [ "$design_check_rc" -eq 0 ] && [ "$design_last" = "OK" ]; then
    pass "004-20 DoD" "the design checker prints OK"
else
    bad "004-20 DoD" "the design checker does not print OK: $(tail_of "$design_check_out")"
fi
if [ "${standalone_00420:-0}" = 1 ]; then exit "$fail"; fi
