# Relocated repository-state assertions of checks/004-18.sh (AC7, script
# lines 1270-1294, run "when the paths are not overridden").
# Sourced by scripts/repo-state-checks.sh, which defines pass, bad, tail_of
# and runs each tool once (wire_check_rc/out, wire_tables_rc/out/line). Run
# by hand (`bash scripts/repo-state.d/004-18.sh` from the repository root),
# it defines them and runs the tools itself.
if ! command -v pass >/dev/null 2>&1; then
    fail=0
    standalone_00418=1
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
    pass "004-18 AC7" "cargo run --bin wire-check reports checks clean"
else
    bad "004-18 AC7" "cargo run --bin wire-check did not pass: $(tail_of "$wire_check_out")"
fi
# `cargo test -p hadronyche-tools --test wire_tables`: exit 0, a result line,
# and the LAST result line at 0 failed and at least 12 passed (the count
# that stood before 004-18).
wt_pf=$(printf '%s\n' "${wire_tables_line:-}" | sed -nE 's/.* ([0-9]+) passed; *([0-9]+) failed;.*/\1 \2/p')
wt_p=${wt_pf% *}
wt_f=${wt_pf#* }
if [ "$wire_tables_rc" -ne 0 ] || [ -z "${wire_tables_line:-}" ]; then
    bad "004-18 AC7" "cargo test -p hadronyche-tools --test wire_tables failed: $(tail_of "$wire_tables_out")"
elif [ -n "$wt_pf" ] && [ "$wt_f" -eq 0 ] && [ "$wt_p" -ge 12 ]; then
    pass "004-18 AC7" "wire_tables has 0 failed and $wt_p passed (at least 12)"
else
    bad "004-18 AC7" "wire_tables must stay green with 0 failed and at least the 12 tests that stood before 004-18; got: $wire_tables_line"
fi
if [ "${standalone_00418:-0}" = 1 ]; then exit "$fail"; fi
