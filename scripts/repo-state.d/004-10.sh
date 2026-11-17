# Relocated repository-state assertions of checks/004-10.sh (measurement
# extension). Sourced by scripts/repo-state-checks.sh, or run by hand
# from the repository root. The script ran these when WIRE_MD and WIRE_TOML
# were not overridden:
#   AC5: `cargo run --quiet --bin wire-check` exits 0 and prints "checks
#        clean"; `cargo test -p hadronyche-tools --test wire_tables` exits 0
#        and its last 'test result:' line is exactly 12 passed, 0 failed,
#        0 ignored;
#   DoD: `python3 doc/kickoff/check-design.py doc/design` exits 0 with OK on
#        its last line.
# shellcheck source=wp5-common.inc
. "$(dirname "${BASH_SOURCE[0]}")/wp5-common.inc"
if [ "$wire_check_rc" -eq 0 ] && printf '%s\n' "$wire_check_out" | grep -q 'checks clean'; then
    pass "004-10 AC5" "cargo run --bin wire-check reports checks clean"
else
    bad "004-10 AC5" "cargo run --bin wire-check did not pass: $(tail_of "$wire_check_out")"
fi
if [ "$wire_tables_rc" -eq 0 ] && [ "$wire_tables_pfi" = "12 0 0" ]; then
    pass "004-10 AC5" "wire_tables stays 12 passed, 0 failed, 0 ignored"
else
    bad "004-10 AC5" "wire_tables must exit 0 and stay 12 passed, 0 failed, 0 ignored; got: ${wire_tables_line:-no 'test result:' line}"
fi
if [ "$design_rc" -eq 0 ] && [ "$design_last" = "OK" ]; then
    pass "004-10 DoD" "the design checker prints OK"
else
    bad "004-10 DoD" "the design checker does not print OK: $(tail_of "$design_out")"
fi
if [ "${BASH_SOURCE[0]}" = "$0" ] && [ "$fail" -ne 0 ]; then
    exit 1
fi
