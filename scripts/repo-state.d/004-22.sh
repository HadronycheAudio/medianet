# 004-22 — sourced by scripts/repo-state-checks.sh (shared results there).
# From checks/004-22.sh (DoD, "when not overridden"): `cargo run --bin
# wire-check` exits 0, and `cargo test -p hadronyche-tools --test
# wire_tables` reports at least 12 passed and 0 failed (the count that stood
# at 004-22), summed over every 'test result:' line.
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
