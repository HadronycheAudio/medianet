# Sourced by the WP5 files (004-06.sh, 004-08.sh, 004-09.sh, 004-10.sh) of
# scripts/repo-state.d (named .inc so a loop over *.sh does not source it
# on its own): each tool runs once per process, whoever sources
# first. When scripts/repo-state-checks.sh sources these files it already
# defines pass, bad, tail_of and the wire_check_* / wire_tables_* results;
# run by hand (`bash scripts/repo-state.d/004-06.sh` from the repository
# root), the definitions below stand in.
if ! declare -F pass >/dev/null; then
    fail=0
    pass() { echo "PASS $1: $2"; }
    bad() { echo "FAIL $1: $2"; fail=1; }
fi
declare -F tail_of >/dev/null || tail_of() { printf '%s\n' "$1" | tail -n 8 | tr '\n' ' '; }
if [ -z "${wire_check_rc+x}" ]; then
    wire_check_out=$(cargo run --quiet -p hadronyche-tools --bin wire-check 2>&1)
    wire_check_rc=$?
fi
if [ -z "${wire_tables_rc+x}" ]; then
    wire_tables_out=$(cargo test -p hadronyche-tools --test wire_tables 2>&1)
    wire_tables_rc=$?
    wire_tables_line=$(printf '%s\n' "$wire_tables_out" | grep 'test result:' | tail -n 1)
fi
# The last 'test result:' line as "passed failed ignored" (empty when none).
wire_tables_pfi=$(printf '%s\n' "${wire_tables_line:-}" | sed -nE 's/.* ([0-9]+) passed; *([0-9]+) failed; *([0-9]+) ignored.*/\1 \2 \3/p')
# The design checker: exit 0 and a last line OK.
if [ -z "${design_rc+x}" ]; then
    design_out=$(python3 doc/kickoff/check-design.py doc/design 2>&1)
    design_rc=$?
fi
design_last=$(printf '%s\n' "$design_out" | tail -n 1 | tr -d '[:space:]')
# The 004-01 check (the wire.md skeleton): its script while it exists, its
# Rust port (tests/doc_004_01.rs) once the script is deleted.
if [ -z "${check_004_01_rc+x}" ]; then
    if [ -f checks/004-01.sh ]; then
        check_004_01_out=$(bash checks/004-01.sh 2>&1)
    else
        check_004_01_out=$(cargo test -q -p hadronyche-tools --test doc_004_01 2>&1)
    fi
    check_004_01_rc=$?
fi
