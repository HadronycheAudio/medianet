# 004-27 — sourced by scripts/repo-state-checks.sh (shared results there).
# From checks/004-27.sh (AC4, "when the paths are not overridden"): `cargo
# run --bin wire-check` exits 0 and reports "checks clean"; `cargo test -p
# hadronyche-tools --test wire_tables` exits 0, its last 'test result:' line
# has 0 failed and at least 12 passed (the count that stood before 004-27).
# The script's self-greps (DoD "reads the working tree only and runs no
# other check") are not here: the library test
# doccheck::checks::tests::checks_read_only_their_docs scans every check's
# and test file's code for processes, files, the environment and any
# other document source (lead ruling R4).
if [ "$wire_check_rc" -eq 0 ] && printf '%s\n' "$wire_check_out" | grep -q 'checks clean'; then
    pass "004-27 AC4" "cargo run --bin wire-check reports checks clean"
else
    bad "004-27 AC4" "cargo run --bin wire-check does not report checks clean: $(tail_of "$wire_check_out")"
fi
if [ "$wire_tables_rc" -eq 0 ] && [ -n "$wire_tables_passed" ] && [ "$wire_tables_failed" -eq 0 ] && [ "$wire_tables_passed" -ge 12 ]; then
    pass "004-27 AC4" "cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least 12 passed: $wire_tables_line"
else
    bad "004-27 AC4" "cargo test -p hadronyche-tools --test wire_tables must exit 0 with 0 failed and at least 12 passed: ${wire_tables_line:-no result line}"
fi
