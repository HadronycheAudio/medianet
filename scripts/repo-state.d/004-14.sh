# Relocated repository-state assertions of checks/004-14.sh.
# Sourced by scripts/repo-state-checks.sh: uses pass, bad, tail_of and the
# shared wire_check_rc / wire_check_out.
# From checks/004-14.sh (AC 5, "when not overridden"): when wire.toml carries
# no [budget] table (a table whose name contains "budget"), the 004-02
# checker must derive the totals: `cargo run --bin wire-check` exits 0 and
# prints the default total and every fallback total of the Size budget
# tables of wire.md. The script took the totals of its own default and
# fallback columns; here they are read as the first total of the first
# "**Total**" row and every total of the second one (the Size budget layout:
# default column first, then the fallback table). With a [budget] table,
# which is the layout today, the run is not required.
if grep -qiE '^\[\[?[^]]*budget[^]]*\]\]?[[:space:]]*$' doc/design/spec/wire.toml; then
    pass "004-14 AC5" "wire.toml carries a [budget] table; the wire-check run is not required"
else
    size_totals=$(awk '/^## Size budget/{f=1;next} /^## /{f=0} f && /^\| \*\*Total\*\*/' doc/design/spec/wire.md)
    default_total=$(printf '%s\n' "$size_totals" | sed -n 1p | awk -F'|' '{gsub(/[^0-9]/,"",$3); print $3}')
    fallback_totals=$(printf '%s\n' "$size_totals" | sed -n 2p | awk -F'|' '{for (i = 3; i < NF; i++) {gsub(/[^0-9]/,"",$i); print $i}}')
    missing=""
    for n in $default_total $fallback_totals; do
        printf '%s\n' "$wire_check_out" | grep -q "$n" || missing="$missing $n"
    done
    if [ "$wire_check_rc" -ne 0 ]; then
        bad "004-14 AC5" "wire.toml has no budget table, so cargo run --bin wire-check must derive and print the totals, but it did not pass: $(tail_of "$wire_check_out")"
    elif [ -z "$default_total" ] || [ -n "$missing" ]; then
        bad "004-14 AC5" "wire-check does not print the Size budget totals (default ${default_total:-?}; not printed:${missing:- none read})"
    else
        pass "004-14 AC5" "wire-check passes and prints the default and fallback totals"
    fi
fi

