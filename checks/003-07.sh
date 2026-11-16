#!/usr/bin/env bash
# checks/003-07.sh — acceptance check for ticket 003-07 "Smoke script"
# (epic 003, deliverable 5: tools/smoke.sh in the frozen proof of concept).
#
# The deliverable lives outside this repository: tools/smoke.sh in the POC at
# /home/dorian/workspace/hadropoc (the "POC"), written by the CODER under the
# epic-003 exception (C-130).  This script only READS the POC source and git
# history, builds and RUNS the POC (build output stays below the POC's
# `target/`, which its .gitignore excludes), and keeps every output under
# /tmp/opencode/003-07/.  It never writes into the POC tree.  Override the
# location with HADROPOC=/path (used for the throwaway fixture clone only).
#
# INTERFACE the coder must implement (fixed by the lead, ruling L1-L9):
#   * `#!/usr/bin/env bash` first line, `set -euo pipefail`, executable.
#   * Default binary: the POC's own target/release/hadropoc, relative to the
#     script's location.  Override: environment variable HADROPOC_BIN.
#   * Normal mode (no arguments of its own): runs
#       "$BIN" loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 \
#           --no-tui --csv "$tmp/smoke.csv" --metrics "$tmp/metrics.csv" "$@"
#     with "$tmp" from `mktemp -d`, removed by a trap on EXIT.  "$@" is passed
#     through, so a caller can add --group <addr:port>.  On a failed run the
#     script prints hadropoc's captured stderr (or its tail) before failing.
#   * Metrics-only mode:  tools/smoke.sh --check-metrics <file>
#     runs only the metrics check (see below) and exits 0 or non-zero.
#   * Output: every failing assertion prints one line starting with `FAIL`
#     (stderr) and exits 1; a passing run prints one line starting with `PASS`.
#   * Metrics check, in order: (1) every line after the first has the header's
#     field count, else FAIL naming "line N" (N = 1-based file line);
#     (2) no line after the first repeats the header, else FAIL naming
#     "line N"; (3) data rows = 100 +/- 2 (98..102) in total, else FAIL (no
#     line number needed); (4) column 4 is `name`, which identifies the module:
#     exactly four module names, each with 25 +/- 2 rows (23..27), else FAIL.
#
# ASSERTIONS THIS CHECK MAKES
#   AC0  history: poc-untouched is an ancestor of HEAD; every commit after it
#        carries an anchored 003-XX id; at least one commit belongs to 003-07
#        (red: none, so this FAILS now).
#   AC1  scope: the 003-07 commits touch exactly tools/smoke.sh, and that file
#        is mode 100755 at HEAD (no src/, no Cargo change).
#   AC2  static (on tools/smoke.sh, non-comment lines unless noted):
#        executable; bash shebang; `set -euo pipefail`; `mktemp -d`; a trap
#        that removes the directory; no sudo/setcap; the command carries
#        --no-tui, --duration 25, --sim-offset-ms 3, --sim-drift-ppm 20,
#        --csv, --metrics and "$@"; HADROPOC_BIN and the tree's
#        target/release/hadropoc default; a --check-metrics mode; NO line that
#        is an assertion mentioning a sync rate, latency or percentile (L6:
#        `grep -Ei 'sync.?(hz|rate)|latenc|percentile|p9[59]|p50'` matches
#        comment lines only); every comment that cites poc-untouched carries
#        [code], a `src/<file>.rs:<line>` and `key:"<text>"`, and the cited line
#        exists at the tag with the key within 3 lines (for tui.rs, clock.rs
#        and module.rs at least one such comment each).
#   R1   a clean 25 s run of the fixed build (direct, not through the script)
#        provides the real metrics file for the prepared cases below.
#   R2   `tools/smoke.sh` with HADROPOC_BIN = the fixed build, --group on an
#        isolated port: exit 0, a PASS line, and its TMPDIR left empty (trap).
#   R3   the same script with HADROPOC_BIN = a scratch extraction of the
#        poc-untouched tag: non-zero exit, a FAIL line naming a metrics
#        "line N" (the corrupt metrics file of the untouched build).
#   R4   pass-through: `tools/smoke.sh --group <g> --no-such-flag` exits
#        non-zero at once (hadropoc rejects the flag) and names it.
#   M    metrics-only mode on prepared files built from R1's file: good file
#        exit 0; one truncated line, two rows joined, a duplicated header (each
#        expected to name "line 40"); 97, 98, 102, 103 rows; per-module
#        23, 27 accepted, 22, 28 rejected (module-level cases).
#   CARGO  `cargo test` in the POC exits 0 with transport::tests::time_code and
#        transport::tests::scenario reported ok.
#   TREE the POC working tree is clean vs HEAD, except while the orchestrator
#        marker swarm/poc-write-in-flight exists in the shared checkout: then
#        this one assertion is a loud SKIP.
#
# Source facts this check relies on (verified at poc-untouched, see the
# comments in tools/smoke.sh): the headless summary is the last line matching
# `^\[ *[0-9]+s\] clock`, printed by tui.rs headless_line; it contains
# `clock LOCKED` (clock.rs:111), `sent N lost M (fwd K)` (tui.rs:109) and
# `(X/Y windows exact, ...)` (tui.rs:118).  The metrics header is 38 fields
# (module.rs:1292); the module name is field 4.
#
# Exit status: 0 all assertions pass, 1 otherwise (one FAIL line per point).
#
# Cost: `cargo build --release` (cached), three 25 s runs (R1, R2, R3), one
# 2 s rejected invocation (R4), metrics cases (fast), and one `cargo test`.
# Every run is unprivileged, on its own isolated multicast group
# (239.72.68.44 with a port in 47701-47760, never the default group), with
# --no-tui.  Before each run the script waits while another `hadropoc` process
# is active (bounded), and it never runs two POC runs at once.

set -u
export LC_ALL=C

SHARED_ROOT="${SHARED_ROOT:-/home/dorian/workspace/hadronyche/medianet}"
MARKER_PATH="${MARKER_PATH:-$SHARED_ROOT/swarm/poc-write-in-flight}"
POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
UNTOUCHED="poc-untouched"
OUT=/tmp/opencode/003-07
SMOKE="$POC/tools/smoke.sh"
BIN="$POC/target/release/hadropoc"
UNT_DIR="$OUT/untouched"                       # scratch extraction of the tag
UNT_BIN="$UNT_DIR/target/release/hadropoc"
mkdir -p "$OUT" || { echo "FAIL  cannot create $OUT"; exit 1; }

# Three isolated ports in 47701-47760, distinct for the three runs.
PORT_FIX=$(( 47701 + ($$ % 60) ))
PORT_UNT=$(( 47701 + (($$ + 20) % 60) ))
PORT_FLAG=$(( 47701 + (($$ + 40) % 60) ))

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

gitpoc() { git -C "$POC" "$@"; }
# Anchored ticket-id rule (same as 003-01..003-06): see checks/003-06.sh AC0.
ID_ANY='^[0-9a-f]+ (003-[0-9][0-9]([: ]|$)|[a-z]+\(003-[0-9][0-9]\)!?: )'

echo "== 003-07: smoke script (tools/smoke.sh in the POC) =="
echo "   POC:   $POC"
echo "   out:   $OUT"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ] || ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository"
  echo "== 003-07 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --verify -q "$UNTOUCHED^{commit}" >/dev/null 2>&1; then
  bad "precondition: tag $UNTOUCHED does not resolve in the POC"
  echo "== 003-07 FAILED =="
  exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
  bad "precondition: cargo not found in PATH"
  echo "== 003-07 FAILED =="
  exit 1
fi

# Commits of this ticket: first line references 003-07 (anchored rule).
ticket_commits() {
  gitpoc log --reverse --format='%H%x09%s' "$UNTOUCHED..HEAD" 2>/dev/null \
    | awk -F'\t' '$2 ~ /^(003-07([: ]|$)|[a-z]+\(003-07\)!?: )/ { print $1 }'
}

# -------------------------------------------------------------------- AC 0
echo "-- AC0 history: poc-untouched ancestor, 003-XX ids, a 003-07 commit"
if gitpoc merge-base --is-ancestor "$UNTOUCHED" HEAD 2>/dev/null; then
  pass "AC0 $UNTOUCHED is an ancestor of HEAD"
else
  bad "AC0 $UNTOUCHED is not an ancestor of HEAD"
fi
bad_ids="$(gitpoc log --format='%h %s' "$UNTOUCHED..HEAD" 2>/dev/null | grep -Ev "$ID_ANY" || true)"
if [ -n "$bad_ids" ]; then
  bad "AC0 commits after $UNTOUCHED without a 003-XX id in the first line:"
  printf '%s\n' "$bad_ids" | head -n 5 | while IFS= read -r l; do note "$l"; done
else
  pass "AC0 every commit after $UNTOUCHED references a 003-XX ticket"
fi
T_COMMITS="$(ticket_commits)"
if [ -z "$T_COMMITS" ]; then
  bad "AC0 no commit after $UNTOUCHED references 003-07 (the smoke script is not committed in the POC)"
else
  pass "AC0 $(printf '%s\n' "$T_COMMITS" | wc -l | tr -d ' ') commit(s) reference 003-07: $(printf '%s\n' "$T_COMMITS" | cut -c1-12 | tr '\n' ' ')"
fi

# -------------------------------------------------------------------- AC 1
echo "-- AC1 scope: the 003-07 commits add exactly tools/smoke.sh (mode 100755)"
if [ -z "$T_COMMITS" ]; then
  bad "AC1 not evaluated: no 003-07 commit (see AC0)"
else
  paths_file="$OUT/ticket-paths.txt"
  : > "$paths_file"
  for c in $T_COMMITS; do
    gitpoc show --name-only --format= "$c" >> "$paths_file" 2>/dev/null
  done
  sort -u "$paths_file" -o "$paths_file"
  if [ "$(cat "$paths_file")" = "tools/smoke.sh" ]; then
    pass "AC1 the 003-07 commits touch exactly one path: tools/smoke.sh"
  else
    bad "AC1 the 003-07 commits touch paths other than tools/smoke.sh (no src/, no Cargo change allowed):"
    while IFS= read -r p; do note "$p"; done < "$paths_file"
  fi
  mode="$(gitpoc ls-tree HEAD -- tools/smoke.sh 2>/dev/null | awk '{ print $1 }')"
  if [ "$mode" = "100755" ]; then
    pass "AC1 tools/smoke.sh is mode 100755 at HEAD"
  else
    bad "AC1 tools/smoke.sh is not an executable file at HEAD (mode '${mode:-absent}')"
  fi
fi

# -------------------------------------------------------------------- AC 2
echo "-- AC2 static checks on tools/smoke.sh"
if [ ! -f "$SMOKE" ]; then
  bad "AC2 tools/smoke.sh not found in the POC: static checks not evaluated"
  SMOKE_OK=0
else
  SMOKE_OK=1
fi

# Non-comment, non-blank lines of the script.
code_lines() { grep -v -E '^[[:space:]]*(#|$)' "$SMOKE"; }
has_code() { code_lines | grep -Fq -e "$1"; }   # fixed string, anywhere in a code line

if [ "$SMOKE_OK" -eq 1 ]; then
  [ -x "$SMOKE" ] && pass "AC2 tools/smoke.sh is executable in the working tree" \
    || bad "AC2 tools/smoke.sh is not executable in the working tree"
  if head -n 1 "$SMOKE" | grep -Eq '^#!.*bash'; then
    pass "AC2 first line is a bash shebang"
  else
    bad "AC2 first line is not a bash shebang"
  fi
  if code_lines | grep -Eq '^[[:space:]]*set -euo pipefail[[:space:]]*$'; then
    pass "AC2 set -euo pipefail is present"
  else
    bad "AC2 set -euo pipefail is not present"
  fi
  if has_code 'mktemp -d'; then pass "AC2 creates its directory with mktemp -d"; else bad "AC2 no mktemp -d"; fi
  if code_lines | grep -E 'trap' | grep -q 'rm -rf'; then
    pass "AC2 a trap removes the temporary directory"
  else
    bad "AC2 no trap that removes the temporary directory (rm -rf in a trap)"
  fi
  sp="$(code_lines | grep -Eiw 'sudo|setcap' || true)"
  if [ -z "$sp" ]; then pass "AC2 no sudo or setcap (unprivileged)"; else bad "AC2 privileged command in the script:"; printf '%s\n' "$sp" | head -n 3 | while IFS= read -r l; do note "$l"; done; fi
  if has_code 'loopback'; then pass "AC2 runs hadropoc loopback"; else bad "AC2 no 'loopback' command in the script"; fi
  for tok in '--no-tui' '--duration 25' '--sim-offset-ms 3' '--sim-drift-ppm 20' '--csv' '--metrics'; do
    if has_code "$tok"; then pass "AC2 command carries $tok"; else bad "AC2 command does not carry $tok (the smoke command of epic 003 deliverable 5; --no-tui is required because the POC has no headless fallback, L1)"; fi
  done
  if has_code '"$@"'; then pass "AC2 passes its arguments through (\"\$@\")"; else bad "AC2 does not pass its arguments through (\"\$@\" missing; --group must reach hadropoc)"; fi
  if has_code 'HADROPOC_BIN'; then pass "AC2 binary override HADROPOC_BIN"; else bad "AC2 no HADROPOC_BIN override"; fi
  if has_code 'target/release/hadropoc'; then pass "AC2 default binary is the tree's target/release/hadropoc"; else bad "AC2 no default binary target/release/hadropoc"; fi
  if has_code '--check-metrics'; then pass "AC2 metrics-only mode --check-metrics"; else bad "AC2 no --check-metrics mode"; fi
  sy="$(code_lines | grep -Ei 'sync.?(hz|rate)|latenc|percentile|p9[59]|p50' || true)"
  if [ -z "$sy" ]; then
    pass "AC2 no assertion line mentions a sync rate, a latency or a percentile (L6)"
  else
    bad "AC2 assertion line(s) mention a sync rate, a latency or a percentile:"
    printf '%s\n' "$sy" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi

  # Pattern comments: every comment citing poc-untouched carries [code], a
  # src/<file>.rs:<line> and key:"<text>", and the key lies within 3 lines of
  # the cited line at the tag.
  cite_bad=""
  cites=""
  n_cite=0
  while IFS= read -r line; do
    n_cite=$(( n_cite + 1 ))
    case "$line" in
      \#*|[[:space:]]*\#*) ;;
      *) cite_bad="$cite_bad
line $line: not a comment"; continue ;;
    esac
    case "$line" in *"[code]"*) ;; *) cite_bad="$cite_bad
$line: no [code] tag"; continue ;; esac
    f="$(printf '%s\n' "$line" | sed -nE 's/.*src\/([a-z_]+\.rs):([0-9]+).*/\1/p')"
    n="$(printf '%s\n' "$line" | sed -nE 's/.*src\/([a-z_]+\.rs):([0-9]+).*/\2/p')"
    key="$(printf '%s\n' "$line" | sed -nE 's/.*key:"([^"]+)".*/\1/p')"
    if [ -z "$f" ] || [ -z "$n" ] || [ -z "$key" ]; then
      cite_bad="$cite_bad
$line: needs src/<file>.rs:<line> and key:\"<text>\""
      continue
    fi
    src="$(gitpoc show "$UNTOUCHED:src/$f" 2>/dev/null || true)"
    total="$(printf '%s\n' "$src" | wc -l | tr -d ' ')"
    if [ -z "$src" ] || [ "$n" -gt "$total" ] || [ "$n" -lt 1 ]; then
      cite_bad="$cite_bad
$f:$n does not exist at $UNTOUCHED ($total lines)"
      continue
    fi
    lo=$(( n > 3 ? n - 3 : 1 )); hi=$(( n + 3 ))
    if ! printf '%s\n' "$src" | sed -n "${lo},${hi}p" | grep -Fq -- "$key"; then
      cite_bad="$cite_bad
$f:$n: key \"$key\" not within 3 lines of the cited line at $UNTOUCHED"
      continue
    fi
    cites="$cites $f"
  done < <(grep -n 'poc-untouched' "$SMOKE" | sed -E 's/^[0-9]+://')
  if [ -n "$cite_bad" ]; then
    bad "AC2 pattern comments that do not cite the poc-untouched source:"
    printf '%s\n' "$cite_bad" | sed '/^$/d' | head -n 8 | while IFS= read -r l; do note "$l"; done
  else
    pass "AC2 $n_cite pattern comment(s) cite poc-untouched with [code], file:line and key, all verified"
  fi
  for need in tui.rs clock.rs module.rs; do
    case " $cites " in
      *" $need "*) pass "AC2 a pattern comment cites $need at $UNTOUCHED (summary, lock or header)" ;;
      *) bad "AC2 no pattern comment cites src/$need at $UNTOUCHED" ;;
    esac
  done
fi

# ---------------------------------------------------------------- build
echo "-- build: fixed build (HEAD) and the poc-untouched scratch extraction"
( cd "$POC" && CARGO_TERM_COLOR=never cargo build --release ) > "$OUT/build-fixed.log" 2>&1
rc=$?
if [ "$rc" -ne 0 ] || [ ! -x "$BIN" ]; then
  bad "build: cargo build --release in the POC failed (exit $rc)"
  tail -n 10 "$OUT/build-fixed.log" | while IFS= read -r l; do note "$l"; done
  BUILD_OK=0
else
  BUILD_OK=1
fi
if [ ! -x "$UNT_BIN" ]; then
  rm -rf "$UNT_DIR"
  mkdir -p "$UNT_DIR" && gitpoc archive "$UNTOUCHED" | tar -x -C "$UNT_DIR"
  ( cd "$UNT_DIR" && CARGO_TERM_COLOR=never cargo build --release ) > "$OUT/build-untouched.log" 2>&1
fi
if [ ! -x "$UNT_BIN" ]; then
  bad "build: scratch build of $UNTOUCHED failed (see $OUT/build-untouched.log)"
  UNT_OK=0
else
  UNT_OK=1
fi

# Bounded wait while another hadropoc process is active; false if it never ends.
wait_for_poc() {
  local waited=0
  while pgrep -x hadropoc >/dev/null 2>&1; do
    if [ "$waited" -ge 600 ]; then
      return 1
    fi
    [ "$waited" -eq 0 ] && note "waiting for an active hadropoc run to finish"
    sleep 5
    waited=$(( waited + 5 ))
  done
  return 0
}

# ------------------------------------------------------------------ R1
echo "-- R1 clean 25 s run of the fixed build (the real metrics file for the prepared cases)"
GOOD="$OUT/good"
rm -rf "$GOOD"; mkdir -p "$GOOD"
GOOD_OK=0
if [ "$BUILD_OK" -ne 1 ]; then
  bad "R1 not evaluated: build failed"
elif ! wait_for_poc; then
  bad "R1 not run: another hadropoc process stayed active for 600 s"
else
  note "running: $BIN loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --no-tui --csv $GOOD/smoke.csv --metrics $GOOD/metrics.csv --group 239.72.68.44:$PORT_FIX"
  timeout 120 "$BIN" loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --no-tui \
    --csv "$GOOD/smoke.csv" --metrics "$GOOD/metrics.csv" --group "239.72.68.44:$PORT_FIX" \
    > "$GOOD/out.txt" 2> "$GOOD/err.txt"
  rc=$?
  lines="$(wc -l < "$GOOD/metrics.csv" 2>/dev/null | tr -d ' ')"
  if [ "$rc" -eq 0 ] && [ "${lines:-0}" -eq 101 ]; then
    GOOD_OK=1
    pass "R1 clean run exited 0 and wrote a header plus 100 data rows"
  else
    bad "R1 clean run: exit $rc, metrics lines ${lines:-none} (expected exit 0 and 101 lines)"
  fi
fi

# ------------------------------------------------------------------ R2
echo "-- R2 tools/smoke.sh on the fixed build: exit 0, PASS line, temporary directory removed"
if [ "$SMOKE_OK" -ne 1 ] || [ "$BUILD_OK" -ne 1 ]; then
  bad "R2 not evaluated: tools/smoke.sh or the fixed build is missing"
elif ! wait_for_poc; then
  bad "R2 not run: another hadropoc process stayed active for 600 s"
else
  T2="$OUT/tmp-fixed"; rm -rf "$T2"; mkdir -p "$T2"
  note "running: HADROPOC_BIN=$BIN TMPDIR=$T2 bash $SMOKE --group 239.72.68.44:$PORT_FIX"
  HADROPOC_BIN="$BIN" TMPDIR="$T2" timeout 240 bash "$SMOKE" --group "239.72.68.44:$PORT_FIX" \
    > "$OUT/smoke-fixed.out" 2>&1
  rc=$?
  if [ "$rc" -eq 0 ]; then pass "R2 tools/smoke.sh exits 0 on the fixed build"; else bad "R2 tools/smoke.sh exits $rc on the fixed build"; fi
  if grep -q '^PASS' "$OUT/smoke-fixed.out"; then
    pass "R2 a PASS line is printed:"
    note "$(grep '^PASS' "$OUT/smoke-fixed.out" | head -n 1)"
  else
    bad "R2 no PASS line in the output"
  fi
  left="$(ls -A "$T2" 2>/dev/null || true)"
  if [ -z "$left" ]; then pass "R2 the temporary directory is removed (TMPDIR left empty)"; else bad "R2 the script left files behind in its temporary directory: $left"; fi
  tail -n 4 "$OUT/smoke-fixed.out" | while IFS= read -r l; do note "$l"; done
fi

# ------------------------------------------------------------------ R3
echo "-- R3 tools/smoke.sh on the poc-untouched build: non-zero, FAIL line naming a metrics line number"
if [ "$SMOKE_OK" -ne 1 ] || [ "$UNT_OK" -ne 1 ]; then
  bad "R3 not evaluated: tools/smoke.sh or the untouched build is missing"
elif ! wait_for_poc; then
  bad "R3 not run: another hadropoc process stayed active for 600 s"
else
  T3="$OUT/tmp-untouched"; rm -rf "$T3"; mkdir -p "$T3"
  note "running: HADROPOC_BIN=$UNT_BIN TMPDIR=$T3 bash $SMOKE --group 239.72.68.44:$PORT_UNT"
  HADROPOC_BIN="$UNT_BIN" TMPDIR="$T3" timeout 240 bash "$SMOKE" --group "239.72.68.44:$PORT_UNT" \
    > "$OUT/smoke-untouched.out" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then pass "R3 tools/smoke.sh exits non-zero on the poc-untouched build (status $rc)"; else bad "R3 tools/smoke.sh exits 0 on the poc-untouched build (corrupt metrics file not caught)"; fi
  if grep -q '^FAIL' "$OUT/smoke-untouched.out" && grep '^FAIL' "$OUT/smoke-untouched.out" | grep -Eq 'line [0-9]+'; then
    pass "R3 the FAIL line names a metrics line number:"
    note "$(grep '^FAIL' "$OUT/smoke-untouched.out" | grep -E 'line [0-9]+' | head -n 1)"
  else
    bad "R3 no FAIL line naming a metrics line number"
  fi
  left="$(ls -A "$T3" 2>/dev/null || true)"
  if [ -z "$left" ]; then pass "R3 the temporary directory is removed (TMPDIR left empty)"; else bad "R3 the script left files behind: $left"; fi
fi

# ------------------------------------------------------------------ R4
echo "-- R4 pass-through: an unknown flag reaches hadropoc (fails at once, no run)"
if [ "$SMOKE_OK" -ne 1 ] || [ "$BUILD_OK" -ne 1 ]; then
  bad "R4 not evaluated: tools/smoke.sh or the fixed build is missing"
else
  T4="$OUT/tmp-flag"; rm -rf "$T4"; mkdir -p "$T4"
  HADROPOC_BIN="$BIN" TMPDIR="$T4" timeout 60 bash "$SMOKE" --group "239.72.68.44:$PORT_FLAG" --no-such-flag \
    > "$OUT/smoke-flag.out" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ] && grep -q 'no-such-flag' "$OUT/smoke-flag.out"; then
    pass "R4 the unknown flag is passed to hadropoc and the script exits non-zero (status $rc)"
  else
    bad "R4 the script did not pass the unknown flag to hadropoc (status $rc; '$OUT/smoke-flag.out' must name --no-such-flag)"
  fi
fi

# ------------------------------------------------------------------ M
echo "-- M metrics-only mode (tools/smoke.sh --check-metrics) on prepared files"
PREP="$OUT/prep"
rm -rf "$PREP"; mkdir -p "$PREP"
# mkcase <out> <src> <spec>: keeps the first N rows per module (N from spec,
# 25 when absent; N above 25 appends copies of the module's first row).
mkcase() {
  awk -F, -v spec="$3" '
    BEGIN { n = split(spec, kv, " "); for (i = 1; i <= n; i++) { split(kv[i], p, ":"); want[p[1]] = p[2] + 0 } }
    NR == 1 { print; next }
    { m = $4; c[m]++; if (!(m in first)) first[m] = $0
      w = (m in want) ? want[m] : 25
      if (c[m] <= (w < 25 ? w : 25)) print }
    END { for (m in want) for (k = 25; k < want[m]; k++) print first[m] }
  ' "$2" > "$1"
}
mk_ok=0
if [ "$GOOD_OK" -ne 1 ]; then
  bad "M not evaluated: the clean run R1 did not produce a metrics file"
else
  G="$GOOD/metrics.csv"
  mkcase "$PREP/good.csv" "$G" ""
  cmp -s "$PREP/good.csv" "$G" && mk_ok=1
  if [ "$mk_ok" -ne 1 ]; then
    bad "M the prepared-file generator does not reproduce the clean file (cannot build the cases)"
  else
    mkcase "$PREP/rows97.csv"  "$G" "mixer:24 sender:24 synth1:24"
    mkcase "$PREP/rows98.csv"  "$G" "mixer:24 sender:24"
    mkcase "$PREP/rows102.csv" "$G" "mixer:26 sender:26"
    mkcase "$PREP/rows103.csv" "$G" "mixer:26 sender:26 synth1:26"
    mkcase "$PREP/mod23.csv"   "$G" "synth1:23 synth2:27"
    mkcase "$PREP/mod22.csv"   "$G" "synth1:22 synth2:28"
    mkcase "$PREP/mod28.csv"   "$G" "synth1:24 synth2:28"
    # One truncated line (line 40 keeps 20 fields), two rows joined (lines 40
    # and 41 on one line), and a duplicated header inserted at line 40.
    awk -F, 'NR == 40 { out = ""; for (i = 1; i <= 20; i++) out = out (i > 1 ? "," : "") $i; print out; next } { print }' "$G" > "$PREP/trunc40.csv"
    awk 'NR == 40 { hold = $0; next } NR == 41 { print hold $0; next } { print }' "$G" > "$PREP/joined40.csv"
    awk -v H="$(head -n 1 "$G")" 'NR == 40 { print H } { print }' "$G" > "$PREP/dup40.csv"
  fi
fi

# metrics_case <label> <file> <ok|fail> <grep pattern or ->
metrics_case() {
  local out rc
  out="$(timeout 30 bash "$SMOKE" --check-metrics "$2" 2>&1)"; rc=$?
  if [ "$3" = "ok" ]; then
    if [ "$rc" -eq 0 ]; then pass "M $1 accepted (exit 0)"; else bad "M $1 rejected (exit $rc): $(printf '%s' "$out" | tail -n 1)"; fi
  else
    if [ "$rc" -eq 0 ]; then
      bad "M $1 accepted (exit 0), must be rejected"
    elif ! printf '%s\n' "$out" | grep -q '^FAIL'; then
      bad "M $1 rejected (exit $rc) without a FAIL line"
    elif [ "$4" != "-" ] && ! printf '%s\n' "$out" | grep -Eq "$4"; then
      bad "M $1 rejected but the message does not match '$4':"
      printf '%s\n' "$out" | head -n 2 | while IFS= read -r l; do note "$l"; done
    else
      pass "M $1 rejected: $(printf '%s\n' "$out" | grep '^FAIL' | head -n 1)"
    fi
  fi
}

if [ "$SMOKE_OK" -ne 1 ]; then
  bad "M not evaluated: tools/smoke.sh is missing"
elif [ "$mk_ok" -eq 1 ]; then
  metrics_case "clean 100-row file"                 "$PREP/good.csv"    ok   -
  metrics_case "one truncated line (line 40)"       "$PREP/trunc40.csv" fail 'line 40'
  metrics_case "two rows joined (lines 40 and 41)"  "$PREP/joined40.csv" fail 'line 40'
  metrics_case "duplicated header (line 40)"        "$PREP/dup40.csv"   fail 'line 40'
  metrics_case "98 data rows (lower bound)"         "$PREP/rows98.csv"  ok   -
  metrics_case "102 data rows (upper bound)"        "$PREP/rows102.csv" ok   -
  metrics_case "97 data rows"                       "$PREP/rows97.csv"  fail -
  metrics_case "103 data rows"                      "$PREP/rows103.csv" fail -
  metrics_case "synth1 23 and synth2 27 rows"       "$PREP/mod23.csv"   ok   -
  metrics_case "synth1 22 and synth2 28 rows"       "$PREP/mod22.csv"   fail -
  metrics_case "synth2 28 rows (total 102)"         "$PREP/mod28.csv"   fail -
fi

# ------------------------------------------------------------- CARGO
echo "-- CARGO cargo test: transport.rs tests pass"
( cd "$POC" && CARGO_TERM_COLOR=never cargo test ) > "$OUT/test.log" 2>&1
rc=$?
if [ "$rc" -eq 0 ]; then
  pass "CARGO cargo test exited 0"
else
  bad "CARGO cargo test exited with status $rc"
  tail -n 15 "$OUT/test.log" | while IFS= read -r l; do note "$l"; done
fi
for t in time_code scenario; do
  if grep -Eq "^test transport::tests::$t \.\.\. ok\$" "$OUT/test.log"; then
    pass "CARGO transport::tests::$t ... ok"
  else
    bad "CARGO transport::tests::$t is not reported as ok"
  fi
done
if grep -Eq '^test result: FAILED|[1-9][0-9]* failed;' "$OUT/test.log"; then
  bad "CARGO cargo test reports a failing test"
fi

# ------------------------------------------------------------------ TREE
echo "-- TREE POC working tree clean vs HEAD (read-only; reported last)"
if [ -e "$MARKER_PATH" ]; then
  printf 'SKIP: epic-003 write in flight (orchestrator marker present)\n'
  note "marker: $MARKER_PATH"
  note "TREE tree-cleanliness assertion skipped; this is not a failure"
else
  porcelain="$(gitpoc status --porcelain 2>&1)"
  if [ -z "$porcelain" ]; then
    pass "TREE git status --porcelain in $POC is empty"
  else
    bad "TREE POC working tree is not clean:"
    printf '%s\n' "$porcelain" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-07 PASSED =="
  exit 0
fi
echo "== 003-07 FAILED =="
exit 1
