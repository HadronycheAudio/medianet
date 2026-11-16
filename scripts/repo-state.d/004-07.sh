# Relocated repository-state assertions of checks/004-07.sh (DoD, "when the
# document paths are not overridden"): the design checker prints OK, and
# checks/004-01.sh still passes (the analysis must not break the skeleton).
# Sourced by scripts/repo-state-checks.sh: uses pass, bad.
design_out=$(python3 doc/kickoff/check-design.py doc/design 2>&1)
design_rc=$?
design_last=$(printf '%s\n' "$design_out" | tail -n 1 | tr -d '[:space:]')
if [ "$design_rc" -eq 0 ] && [ "$design_last" = "OK" ]; then
    pass "004-07 DoD" "the design checker prints OK"
else
    bad "004-07 DoD" "the design checker does not print OK: $(printf '%s\n' "$design_out" | tail -n 8 | tr '\n' ' ')"
fi
if [ -f checks/004-01.sh ]; then
    if bash checks/004-01.sh >/dev/null 2>&1; then
        pass "004-07 DoD" "checks/004-01.sh still passes"
    else
        bad "004-07 DoD" "checks/004-01.sh no longer passes"
    fi
fi
