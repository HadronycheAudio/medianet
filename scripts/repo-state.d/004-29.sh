# shellcheck shell=bash
# Repository-state assertions of ticket 004-29, relocated from
# checks/004-29.sh by its port (hadronyche-tools/tests/doc/c004_29.rs).
# Sourced by scripts/repo-state-checks.sh after its shared tool runs; uses
# its `pass`, `bad`, `tail_of` and the shared results `wire_check_rc`,
# `wire_check_out`, `wire_tables_rc`, `wire_tables_line`,
# `wire_tables_passed`, `wire_tables_failed` (each tool runs once).
#
# From checks/004-29.sh (AC5 "when the paths are not overridden"):
#   `cargo run --bin wire-check` exits 0 and reports "checks clean";
#   `cargo test -p hadronyche-tools --test wire_tables` exits 0, its last
#   'test result:' line has 0 failed and at least 12 passed (the count at
#   the merged text, MIN_WIRE_TABLES; 004-29).
#   The script's self-greps (`git.*main`, `checks/[0-9]`: the checker
#   reads the working tree only and runs no other check) are the
#   library-wide rule now (hadronyche-tools/clippy.toml and
#   doccheck::testing::source_rules), not a line here.
if [ "$wire_check_rc" -eq 0 ] && printf '%s\n' "$wire_check_out" | grep -q 'checks clean'; then
    pass "004-29 AC5" "cargo run --bin wire-check reports checks clean"
else
    bad "004-29 AC5" "cargo run --bin wire-check does not report checks clean: $(tail_of "$wire_check_out")"
fi
if [ "$wire_tables_rc" -eq 0 ] && [ -n "$wire_tables_passed" ] && [ "$wire_tables_failed" -eq 0 ] && [ "$wire_tables_passed" -ge 12 ]; then
    pass "004-29 AC5" "cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least 12 passed: $wire_tables_line"
else
    bad "004-29 AC5" "cargo test -p hadronyche-tools --test wire_tables must exit 0 with 0 failed and at least 12 passed: ${wire_tables_line:-no result line}"
fi
