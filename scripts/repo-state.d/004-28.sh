# shellcheck shell=bash
# Repository-state assertions of ticket 004-28, relocated from
# checks/004-28.sh by its port (hadronyche-tools/tests/doc_004_28.rs).
# Sourced by scripts/repo-state-checks.sh after its shared tool runs; uses
# its `pass`, `bad`, `tail_of` and the shared results `wire_check_rc`,
# `wire_check_out`, `wire_tables_rc`, `wire_tables_line`,
# `wire_tables_passed`, `wire_tables_failed` (each tool runs once).
#
# From checks/004-28.sh (AC5 "when the paths are not overridden", and DoD
# "the checker reads the working tree only and runs no other check"):
#   `cargo run --bin wire-check` exits 0 and reports "checks clean";
#   `cargo test -p hadronyche-tools --test wire_tables` exits 0, its last
#   'test result:' line has 0 failed and at least 12 passed (the count at
#   the merged text, MIN_WIRE_TABLES; 004-28).
#   The script grepped ITSELF for `git.*main` and for `checks/[0-9]`; the
#   check is now Rust, so the same intent is asserted on the port's sources:
#   no line matching `git.*main`, and no spawned process (Command or
#   std::process), which is how a Rust check would run a sibling check or
#   read a ref.
if [ "$wire_check_rc" -eq 0 ] && printf '%s\n' "$wire_check_out" | grep -q 'checks clean'; then
    pass "004-28 AC5" "cargo run --bin wire-check reports checks clean"
else
    bad "004-28 AC5" "cargo run --bin wire-check does not report checks clean: $(tail_of "$wire_check_out")"
fi
if [ "$wire_tables_rc" -eq 0 ] && [ -n "$wire_tables_passed" ] && [ "$wire_tables_failed" -eq 0 ] && [ "$wire_tables_passed" -ge 12 ]; then
    pass "004-28 AC5" "cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least 12 passed: $wire_tables_line"
else
    bad "004-28 AC5" "cargo test -p hadronyche-tools --test wire_tables must exit 0 with 0 failed and at least 12 passed: ${wire_tables_line:-no result line}"
fi
port_src="hadronyche-tools/src/doccheck/checks/c004_28 hadronyche-tools/tests/doc_004_28.rs"
# shellcheck disable=SC2086
if ! ls $port_src >/dev/null 2>&1; then
    bad "004-28 DoD" "the port's sources are missing: $port_src"
fi
# shellcheck disable=SC2086
if grep -rnE 'git.*main' $port_src >/dev/null; then
    bad "004-28 DoD" "the port names a repository ref containing main; it must read the working tree only"
else
    pass "004-28 DoD" "the port names no repository ref containing main"
fi
# shellcheck disable=SC2086
if grep -rnE 'Command|std::process' $port_src >/dev/null; then
    bad "004-28 DoD" "the port spawns a process; it must run no other check"
else
    pass "004-28 DoD" "the port spawns no process, so it runs no sibling check"
fi
