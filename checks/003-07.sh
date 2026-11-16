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
# location with HADROPOC=/path (used for the throwaway fixture clones only).
#
# RULINGS applied (PR #23 review of the lead; the names used below):
#   R1  the smoke command carries --no-tui (no headless fallback in the POC).
#   R2  the script passes its arguments through to hadropoc (--group).
#   R3  the script runs correctly from any working directory (run from /tmp).
#   R5  the lock is judged from the metrics `lock` column (field 8, value 2 =
#       Locked) of EVERY module, not from the headless line, whose LOCKED is
#       the sender module's own view (REJECTED: "LOCKED on any headless line").
#   R6  "0 content mismatches" holds for EVERY exact-count pair the final
#       summary prints: audio, both synths' voice channels, playhead, clicks
#       and channel-exact; each pair needs X = Y and Y > 0.
#   R7  loss is fail-closed: `sent S lost N` must read 0.
#   R8  the poc-untouched scratch build is the untouched tag (the negative
#       case of the metrics file and of the summary).
#
# INTERFACE the coder implements:
#   * `#!/usr/bin/env bash` first line, `set -euo pipefail`, executable.
#   * Default binary: target/release/hadropoc of the tree.  Override: HADROPOC_BIN.
#   * Normal mode: "$BIN" loopback --duration 25 --sim-offset-ms 3
#     --sim-drift-ppm 20 --no-tui --csv "$tmp/smoke.csv" --metrics
#     "$tmp/metrics.csv" "$@", with "$tmp" from `mktemp -d` and a trap on EXIT
#     that removes it.  On a failed run the captured stderr is printed.
#   * Metrics-only mode: tools/smoke.sh --check-metrics <file> (exit 0 or 1).
#   * Output: a passing run prints a line starting with PASS; each failing
#     assertion prints one line starting with FAIL on stderr and exits 1.
#   * check_metrics (also reached by --check-metrics): field count against the
#     header, repeated header, data rows 98..102 in total, per module 23..27,
#     exactly four module names, and (R5) every module's lock column reaches 2
#     on at least one row.  Messages:
#       FAIL metrics: line N has F fields, the header has H
#       FAIL metrics: line N repeats the header
#       FAIL metrics: R data rows, expected 100 +/- 2
#       FAIL metrics: module M has R data rows, expected 25 +/- 2
#       FAIL metrics: module M never reached lock (column lock = 2)
#   * Summary pairs (R6): every pair below must have X = Y and Y > 0, with a
#     FAIL line that names the pair:
#       audio           (X/Y windows exact, ...)
#       synth1 voice    voice channels X/Y windows exact, max pitch
#       synth1 playhead synth playhead identical for X/Y datagrams
#       synth1 clicks   metronome: X/Y clicks on the right sample
#       synth1 channel  channel exact in X/Y windows
#       synth2 voice / playhead / clicks ("also synth2: ... X/Y clicks right")
#
# ASSERTIONS THIS CHECK MAKES
#   AC0  history: poc-untouched is an ancestor of HEAD; every commit after it
#        carries an anchored 003-XX id; at least one commit belongs to 003-07.
#   AC1  scope: the 003-07 commits touch exactly tools/smoke.sh (mode 100755).
#   AC2  static checks on tools/smoke.sh (non-comment lines unless noted):
#        executable; bash shebang; `set -euo pipefail`; `mktemp -d`; a trap that
#        removes the directory; no sudo/setcap; the command carries --no-tui,
#        --duration 25, --sim-offset-ms 3, --sim-drift-ppm 20, --csv, --metrics
#        and "$@"; HADROPOC_BIN; the default target/release/hadropoc; a
#        --check-metrics mode; a read of the metrics lock column ($8); no sync
#        rate, latency or percentile assertion (grep as in AC 2 of the ticket,
#        comment lines excepted); citations (see below).
#   CITE every citation of poc-untouched: a comment line that carries [code] and
#        `src/<file>.rs:<line>`, with the key (backticks, or a double-quoted
#        string with \" escapes) within 3 lines of that line at the tag.  Keys
#        that must each be cited: clock {} drift, sent {sent} lost, windows exact,
#        voice channels, playhead identical, clicks, channel exact, Locked = 2,
#        state as u8.
#   RUN1 a clean 25 s run of the fixed build (direct): exit 0, 98..102 data
#        rows, 23..27 per module; its output is the source of the summary and
#        the metrics cases below.
#   RUN2 tools/smoke.sh from /tmp with HADROPOC_BIN = the fixed build: exit 0, a
#        PASS line, and its TMPDIR left empty (checked with find).
#   RUN3 the same script with HADROPOC_BIN = a scratch extraction of the
#        poc-untouched tag: non-zero, a FAIL line naming a metrics "line N",
#        TMPDIR left empty.
#   RUN4 pass-through: `--group <g> --no-such-flag` exits non-zero at once and
#        names the flag; TMPDIR left empty.
#   M    --check-metrics on prepared files (built from RUN1's rows, normalised to
#        exactly 25 per module): good file accepted; truncated line, joined rows
#        and duplicated header each name "line 40"; 97 / 98 / 102 / 103 rows;
#        23, 27 accepted; 22, 28 rejected; a module whose lock never reads 2
#        (synth2, sender) rejected, naming it; a module whose lock reads 2 on
#        one row accepted.
#   S    stub hadropoc (outside the POC tree, /tmp/opencode/003-07/stub/): the
#        script's summary is taken from RUN1's real final line, varied by one
#        pair at a time.  All pairs exact: exit 0, PASS.  Each single pair with
#        X < Y (or Y = 0 for audio): exit non-zero with a FAIL line naming it.
#   RUN1/RUN2 repeat (ruling B): each is repeated ONCE, and only when its first
#        attempt fails with the loss message ("FAIL loss:").  Printed as
#        "NOTE RUN<n> repeated once after: <FAIL line>".  A failed repeat FAILS.
#   C3   both synth sections are REQUIRED and verified, in either order.  The
#        main section ("music N voices on <A>:", before " | also ") names synth A
#        and carries A's voice, playhead, clicks and channel-exact figures; the
#        "| also <B>:" section carries B's voice, playhead and clicks figures.
#        Audio, both voice pairs and the transport figures must be exact (X = Y,
#        Y > 0).  Messages the script must use:
#          FAIL summary: missing section "<name>" in the final line   (name: synth1 / synth2 / "<A> music" / "<A> transport")
#          FAIL summary: synth sections name "<A>" and "<B>"; expected synth1 and synth2
#          FAIL content: <synth> <pair> has X of Y exact   (the pair names its own synth)
#          FAIL content: <pair> has no count (0 of 0)
#        PASS line ends "synth1 and synth2 verified".  Stubs are built from two
#        fixed final lines embedded in this check (usual and swapped order).
#   CARGO cargo test passes; transport::tests::time_code and scenario ok.
#   TREE the POC working tree is clean vs HEAD, except while the orchestrator
#        marker swarm/poc-write-in-flight exists (then a loud SKIP).
#
# Exit status: 0 all assertions pass, 1 otherwise.
#
# Cost: cargo build (cached), three 25 s runs (RUN1, RUN2, RUN3), quick rejected
# invocations, metrics cases (fast), stub runs (fast), one cargo test.  Runs are
# unprivileged, each on its own isolated multicast group
# (239.72.68.44 with a port in 48061-48100), with --no-tui.  Before each run the
# script waits while another hadropoc process is active, and it never runs two
# POC runs at once.

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
STUB="$OUT/stub"                               # stub hadropoc, outside the POC
mkdir -p "$OUT" || { echo "FAIL  cannot create $OUT"; exit 1; }

# Three isolated ports in 48061-48100, distinct for the runs of this check.
PORT_FIX=$(( 48061 + ($$ % 40) ))
PORT_UNT=$(( 48061 + (($$ + 13) % 40) ))
PORT_FLAG=$(( 48061 + (($$ + 26) % 40) ))

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
if ! command -v python3 >/dev/null 2>&1; then
  bad "precondition: python3 not found in PATH (needed for the citation check)"
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
    if has_code "$tok"; then pass "AC2 command carries $tok"; else bad "AC2 command does not carry $tok (R1: --no-tui is required, the POC has no headless fallback)"; fi
  done
  if has_code '"$@"'; then pass "AC2 passes its arguments through (\"\$@\")"; else bad "AC2 does not pass its arguments through (\"\$@\" missing; --group must reach hadropoc)"; fi
  if has_code 'HADROPOC_BIN'; then pass "AC2 binary override HADROPOC_BIN"; else bad "AC2 no HADROPOC_BIN override"; fi
  if has_code 'target/release/hadropoc'; then pass "AC2 default binary is the tree's target/release/hadropoc"; else bad "AC2 no default binary target/release/hadropoc"; fi
  if has_code -- '--check-metrics'; then pass "AC2 metrics-only mode --check-metrics"; else bad "AC2 no --check-metrics mode"; fi
  if has_code '$8'; then
    pass "AC2 the metrics lock column (field 8) is read (R5)"
  else
    bad "AC2 the metrics lock column (field 8) is not read: a module's lock must be judged from the metrics file (R5)"
  fi
  sy="$(code_lines | grep -Ei 'sync.?(hz|rate)|latenc|percentile|p9[59]|p50' || true)"
  if [ -z "$sy" ]; then
    pass "AC2 no assertion line mentions a sync rate, a latency or a percentile"
  else
    bad "AC2 assertion line(s) mention a sync rate, a latency or a percentile:"
    printf '%s\n' "$sy" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi

  # CITE: citation lines (a comment with [code] and src/<file>.rs:<line>) must
  # carry a key that lies within 3 lines of the cited line at the tag.  Keys
  # are read from backticks or from a double-quoted string with \" escapes.
  cat > "$OUT/cite-check.py" <<'PY'
import re, subprocess, sys
smoke, poc, tag = sys.argv[1], sys.argv[2], sys.argv[3]
required = ["clock {} drift", "sent {sent} lost", "windows exact", "voice channels",
            "playhead identical", "clicks", "channel exact", "Locked = 2", "state as u8"]
src_cache = {}
def tag_lines(f):
    if f not in src_cache:
        r = subprocess.run(["git", "-C", poc, "show", f"{tag}:src/{f}"], capture_output=True, text=True)
        src_cache[f] = r.stdout.split("\n") if r.returncode == 0 else None
    return src_cache[f]
ok_keys, files, bad = [], set(), []
for i, line in enumerate(open(smoke, encoding="utf-8").read().split("\n"), 1):
    if "[code]" not in line:
        continue
    m = re.search(r"src/([a-z_]+\.rs):([0-9]+)", line)
    if not m:
        continue
    if not line.lstrip().startswith("#"):
        bad.append(f"line {i}: citation is not in a comment")
        continue
    f, n = m.group(1), int(m.group(2))
    k = re.search(r"key:`([^`]+)`", line)
    if k:
        key = k.group(1)
    else:
        k = re.search(r'key:"((?:[^"\\]|\\.)*)"', line)
        if not k:
            bad.append(f"line {i}: {f}:{n} has no key:`...` or key:\"...\"")
            continue
        key = re.sub(r"\\(.)", r"\1", k.group(1))
    src = tag_lines(f)
    if src is None:
        bad.append(f"line {i}: {f} does not exist at {tag}")
        continue
    if n < 1 or n > len(src):
        bad.append(f"line {i}: {f}:{n} does not exist at {tag} ({len(src)} lines)")
        continue
    window = src[max(0, n - 4): n + 3]
    if not any(key in s for s in window):
        bad.append(f"line {i}: key {key!r} not within 3 lines of {f}:{n} at {tag}")
        continue
    ok_keys.append(key); files.add(f)
    print(f"CITE ok line {i}: {f}:{n} key {key!r}")
for f in ("tui.rs", "clock.rs", "module.rs"):
    if f not in files:
        bad.append(f"no citation of src/{f} at {tag}")
for r in required:
    if not any(r in k for k in ok_keys):
        bad.append(f"no valid citation with key {r!r}")
for b in bad:
    print("BAD " + b)
PY
  cite_out="$(python3 "$OUT/cite-check.py" "$SMOKE" "$POC" "$UNTOUCHED")"
  cite_bad="$(printf '%s\n' "$cite_out" | grep '^BAD ' || true)"
  n_ok="$(printf '%s\n' "$cite_out" | grep -c '^CITE ok' || true)"
  if [ -z "$cite_bad" ]; then
    pass "AC2 $n_ok citation(s) of poc-untouched verified (file:line, [code], key within 3 lines); every required key is cited"
  else
    bad "AC2 citations of poc-untouched:"
    printf '%s\n' "$cite_bad" | head -n 12 | while IFS= read -r l; do note "${l#BAD }"; done
  fi
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

# Rows per module and data rows of a metrics file; prints "rows m1=n m2=n ...".
row_counts() {
  awk -F, 'NR > 1 { rows++; c[$4]++ } END { printf "rows=%d", rows; for (m in c) printf " %s=%d", m, c[m]; printf "\n" }' "$1"
}

# ------------------------------------------------------------------ RUN1
# RUN1 and RUN2 are repeated ONCE, and only when the first attempt fails with the
# loss message (FAIL loss:) (ruling B).  The code repeats on that prefix only; a
# missing section is never a repeat cause.  A repeat that also fails, or any
# other failure, FAILS.  A repeat is never counted as a
# pass of the first attempt; the first failure is printed as a NOTE line.
echo "-- RUN1 clean 25 s run of the fixed build (the real metrics file and summary)"
GOOD="$OUT/good"
RUN1_VERDICT=""; RUN1_MSG=""; RUN1_RC=0; RUN1_COUNTS=""
run1_once() {
  rm -rf "$GOOD"; mkdir -p "$GOOD"
  note "running: $BIN loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --no-tui --csv $GOOD/smoke.csv --metrics $GOOD/metrics.csv --group 239.72.68.44:$PORT_FIX"
  timeout 120 "$BIN" loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --no-tui \
    --csv "$GOOD/smoke.csv" --metrics "$GOOD/metrics.csv" --group "239.72.68.44:$PORT_FIX" \
    > "$GOOD/out.txt" 2> "$GOOD/err.txt"
  RUN1_RC=$?
  RUN1_COUNTS="$(row_counts "$GOOD/metrics.csv" 2>/dev/null || true)"
  local total ok_mod m v last lost
  total="$(printf '%s' "$RUN1_COUNTS" | sed -nE 's/^rows=([0-9]+).*/\1/p')"
  ok_mod=1
  for m in mixer sender synth1 synth2; do
    v="$(printf '%s' "$RUN1_COUNTS" | sed -nE "s/.* $m=([0-9]+).*/\1/p")"
    if [ -z "$v" ] || [ "$v" -lt 23 ] || [ "$v" -gt 27 ]; then ok_mod=0; fi
  done
  last="$(grep -E '^\[ *[0-9]+s\] clock' "$GOOD/out.txt" | tail -n 1 || true)"
  lost="$(printf '%s\n' "$last" | sed -nE 's/.*sent [0-9]+ lost ([0-9]+) \(fwd.*/\1/p')"
  if [ "$RUN1_RC" -ne 0 ] || [ -z "$total" ] || [ "$total" -lt 98 ] || [ "$total" -gt 102 ] || [ "$ok_mod" -ne 1 ]; then
    RUN1_VERDICT=bad
    RUN1_MSG="RUN1 clean run: exit $RUN1_RC, metrics [$RUN1_COUNTS] (expected exit 0, 98..102 rows, 23..27 per module)"
  elif [ -z "$lost" ]; then
    RUN1_VERDICT=bad
    RUN1_MSG="RUN1 final line has no 'sent S lost N' field"
  elif [ "$lost" != "0" ]; then
    RUN1_VERDICT=repeat
    RUN1_MSG="FAIL loss: $lost lost datagrams in the last per-second line of RUN1"
  else
    RUN1_VERDICT=ok
    RUN1_MSG=""
  fi
}
RUN1_REPEATED=0
if [ "$BUILD_OK" -ne 1 ]; then
  bad "RUN1 not evaluated: build failed"
elif ! wait_for_poc; then
  bad "RUN1 not run: another hadropoc process stayed active for 600 s"
else
  run1_once
  if [ "$RUN1_VERDICT" = repeat ]; then
    printf 'NOTE RUN1 repeated once after: %s\n' "$RUN1_MSG"
    RUN1_REPEATED=1
    if wait_for_poc; then
      run1_once
    else
      RUN1_VERDICT=bad; RUN1_MSG="RUN1 repeat not run: another hadropoc process stayed active for 600 s"
    fi
  fi
  case "$RUN1_VERDICT" in
    ok)
      GOOD_OK=1
      case "$(grep -E '^\[ *[0-9]+s\] clock ' "$GOOD/out.txt" | tail -n 1)" in
        *'voices on synth1:'*) note 'RUN1 final line order: usual (voices on synth1, also synth2)' ;;
        *'voices on synth2:'*) note 'RUN1 final line order: swapped (voices on synth2, also synth1)' ;;
        *) note 'RUN1 final line order: unrecognised' ;;
      esac
      if [ "$RUN1_REPEATED" -eq 1 ]; then
        pass "RUN1 clean run passed on the single repeat; metrics $RUN1_COUNTS"
      else
        pass "RUN1 clean run exited 0; metrics $RUN1_COUNTS (ticket bounds: 98..102 rows, 23..27 per module)"
      fi ;;
    repeat) bad "RUN1 failed again after the single repeat: $RUN1_MSG" ;;
    *) bad "$RUN1_MSG" ;;
  esac
fi
GOOD_OK="${GOOD_OK:-0}"

# Normalised summary line: the last per-second line of RUN1's output.
R1_SUMMARY="$(grep -E '^\[ *[0-9]+s\] clock ' "$GOOD/out.txt" 2>/dev/null | tail -n 1 || true)"

# ------------------------------------------------------------------ RUN2
# Cleanup is checked with find (not ls): an empty directory has no entries.
echo "-- RUN2 tools/smoke.sh from /tmp on the fixed build: exit 0, PASS line, temporary directory removed"
RUN2_RC=0; RUN2_FAIL=""; RUN2_LEFT=""; RUN2_VERDICT=""
T2="$OUT/tmp-fixed"
run2_once() {
  rm -rf "$T2"; mkdir -p "$T2"
  note "running (cwd /tmp): HADROPOC_BIN=$BIN TMPDIR=$T2 bash $SMOKE --group 239.72.68.44:$PORT_FIX"
  ( cd /tmp && HADROPOC_BIN="$BIN" TMPDIR="$T2" timeout 240 bash "$SMOKE" --group "239.72.68.44:$PORT_FIX" ) \
    > "$OUT/smoke-fixed.out" 2>&1
  RUN2_RC=$?
  RUN2_FAIL="$(grep '^FAIL' "$OUT/smoke-fixed.out" | head -n 1 || true)"
  RUN2_LEFT="$(find "$T2" -mindepth 1 -print -quit)"
  if [ "$RUN2_RC" -eq 0 ] && grep -Eq '^PASS .*synth1 and synth2 verified' "$OUT/smoke-fixed.out" && [ -z "$RUN2_LEFT" ]; then
    RUN2_VERDICT=ok
  elif [ "$RUN2_RC" -ne 0 ] && printf '%s\n' "$RUN2_FAIL" | grep -Eq '^FAIL loss:'; then
    RUN2_VERDICT=repeat
  else
    RUN2_VERDICT=bad
  fi
}
RUN2_REPEATED=0
if [ "$SMOKE_OK" -ne 1 ] || [ "$BUILD_OK" -ne 1 ]; then
  bad "RUN2 not evaluated: tools/smoke.sh or the fixed build is missing"
elif ! wait_for_poc; then
  bad "RUN2 not run: another hadropoc process stayed active for 600 s"
else
  run2_once
  if [ "$RUN2_VERDICT" = repeat ]; then
    printf 'NOTE RUN2 repeated once after: %s\n' "$RUN2_FAIL"
    RUN2_REPEATED=1
    if wait_for_poc; then
      run2_once
    else
      RUN2_VERDICT=bad; RUN2_FAIL="RUN2 repeat not run: another hadropoc process stayed active for 600 s"
    fi
  fi
  case "$RUN2_VERDICT" in
    ok)
      if [ "$RUN2_REPEATED" -eq 1 ]; then
        pass "RUN2 passed on the single repeat: exit 0, PASS line, temporary directory removed"
      else
        pass "RUN2 tools/smoke.sh exits 0 with a PASS line; temporary directory removed (find)"
      fi
      note "$(grep '^PASS' "$OUT/smoke-fixed.out" | tail -n 1)" ;;
    repeat) bad "RUN2 failed again after the single repeat: $RUN2_FAIL" ;;
    *)
      if [ "$RUN2_RC" -ne 0 ]; then
        bad "RUN2 tools/smoke.sh exits $RUN2_RC on the fixed build: ${RUN2_FAIL:-no FAIL line}"
      elif [ -n "$RUN2_LEFT" ]; then
        bad "RUN2 the script left files behind in its temporary directory: $RUN2_LEFT"
      else
        bad "RUN2 no PASS line in the output"
      fi ;;
  esac
  tail -n 4 "$OUT/smoke-fixed.out" | while IFS= read -r l; do note "$l"; done
fi

# ------------------------------------------------------------------ RUN3
echo "-- RUN3 tools/smoke.sh on the poc-untouched build: non-zero, FAIL line naming a metrics line number, temporary directory removed"
if [ "$SMOKE_OK" -ne 1 ] || [ "$UNT_OK" -ne 1 ]; then
  bad "RUN3 not evaluated: tools/smoke.sh or the untouched build is missing"
elif ! wait_for_poc; then
  bad "RUN3 not run: another hadropoc process stayed active for 600 s"
else
  T3="$OUT/tmp-untouched"; rm -rf "$T3"; mkdir -p "$T3"
  note "running: HADROPOC_BIN=$UNT_BIN TMPDIR=$T3 bash $SMOKE --group 239.72.68.44:$PORT_UNT"
  HADROPOC_BIN="$UNT_BIN" TMPDIR="$T3" timeout 240 bash "$SMOKE" --group "239.72.68.44:$PORT_UNT" \
    > "$OUT/smoke-untouched.out" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then pass "RUN3 tools/smoke.sh exits non-zero on the poc-untouched build (status $rc)"; else bad "RUN3 tools/smoke.sh exits 0 on the poc-untouched build (corrupt metrics file not caught)"; fi
  if grep '^FAIL' "$OUT/smoke-untouched.out" | grep -Eq 'line [0-9]+'; then
    pass "RUN3 the FAIL line names a metrics line number:"
    note "$(grep '^FAIL' "$OUT/smoke-untouched.out" | grep -E 'line [0-9]+' | head -n 1)"
  else
    bad "RUN3 no FAIL line naming a metrics line number"
  fi
  left="$(find "$T3" -mindepth 1 -print -quit)"
  if [ -z "$left" ]; then pass "RUN3 the temporary directory is removed (find finds nothing under TMPDIR)"; else bad "RUN3 the script left files behind: $left"; fi
fi

# ------------------------------------------------------------------ RUN4
echo "-- RUN4 pass-through: an unknown flag reaches hadropoc (fails at once, no run), temporary directory removed"
if [ "$SMOKE_OK" -ne 1 ] || [ "$BUILD_OK" -ne 1 ]; then
  bad "RUN4 not evaluated: tools/smoke.sh or the fixed build is missing"
else
  T4="$OUT/tmp-flag"; rm -rf "$T4"; mkdir -p "$T4"
  HADROPOC_BIN="$BIN" TMPDIR="$T4" timeout 60 bash "$SMOKE" --group "239.72.68.44:$PORT_FLAG" --no-such-flag \
    > "$OUT/smoke-flag.out" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ] && grep -q 'no-such-flag' "$OUT/smoke-flag.out"; then
    pass "RUN4 the unknown flag is passed to hadropoc and the script exits non-zero (status $rc)"
  else
    bad "RUN4 the script did not pass the unknown flag to hadropoc (status $rc; '$OUT/smoke-flag.out' must name --no-such-flag)"
  fi
  left="$(find "$T4" -mindepth 1 -print -quit)"
  if [ -z "$left" ]; then pass "RUN4 the temporary directory is removed"; else bad "RUN4 the script left files behind: $left"; fi
fi

# ------------------------------------------------------------------ M
echo "-- M metrics-only mode (tools/smoke.sh --check-metrics) on prepared files"
PREP="$OUT/prep"
rm -rf "$PREP"; mkdir -p "$PREP"
# mkcase <out> <src> <spec>: spec "name:N ..." gives each module's row count
# (default 25).  Rows are kept in order; a count above what the source holds
# appends copies of that module's first row.
mkcase() {
  awk -F, -v OFS=, -v spec="$3" '
    BEGIN { n = split(spec, kv, " "); for (i = 1; i <= n; i++) { split(kv[i], p, ":"); want[p[1]] = p[2] + 0 } }
    NR == 1 { hdr = $0; next }
    { rows[++nrows] = $0; m = $4; avail[m]++; if (!(m in first)) first[m] = $0 }
    END {
      print hdr
      for (r = 1; r <= nrows; r++) {
        split(rows[r], f, ","); m = f[4]; c[m]++
        w = (m in want) ? want[m] : 25
        if (c[m] <= w) print rows[r]
      }
      for (m in want) if (m in avail) for (k = avail[m]; k < want[m]; k++) print first[m]
    }' "$2" > "$1"
}
mk_ok=0
if [ "$SMOKE_OK" -ne 1 ]; then
  bad "M not evaluated: tools/smoke.sh is missing"
elif [ "$GOOD_OK" -ne 1 ]; then
  bad "M not evaluated: the clean run RUN1 did not produce an acceptable metrics file"
else
  G="$GOOD/metrics.csv"
  mkcase "$PREP/good.csv" "$G" "mixer:25 sender:25 synth1:25 synth2:25"
  if awk -F, 'NR > 1 { n++; c[$4]++ }
      END { exit !(n == 100 && c["mixer"] == 25 && c["sender"] == 25 && c["synth1"] == 25 && c["synth2"] == 25) }' \
      "$PREP/good.csv"; then
    mk_ok=1
  fi
  if [ "$mk_ok" -eq 1 ]; then
    mkcase "$PREP/rows97.csv"  "$G" "mixer:24 sender:24 synth1:24 synth2:25"
    mkcase "$PREP/rows98.csv"  "$G" "mixer:24 sender:24 synth1:25 synth2:25"
    mkcase "$PREP/rows102.csv" "$G" "mixer:26 sender:26 synth1:25 synth2:25"
    mkcase "$PREP/rows103.csv" "$G" "mixer:26 sender:26 synth1:26 synth2:25"
    mkcase "$PREP/mod23.csv"   "$G" "synth1:23 synth2:27"
    mkcase "$PREP/mod22.csv"   "$G" "synth1:22 synth2:28"
    mkcase "$PREP/mod28.csv"   "$G" "synth1:24 synth2:28"
    # One truncated line (line 40 keeps 20 fields), two rows joined (lines 40
    # and 41 on one line), and a duplicated header inserted at line 40.
    awk -F, 'NR == 40 { out = ""; for (i = 1; i <= 20; i++) out = out (i > 1 ? "," : "") $i; print out; next } { print }' "$PREP/good.csv" > "$PREP/trunc40.csv"
    awk 'NR == 40 { hold = $0; next } NR == 41 { print hold $0; next } { print }' "$PREP/good.csv" > "$PREP/joined40.csv"
    awk -v H="$(head -n 1 "$PREP/good.csv")" 'NR == 40 { print H } { print }' "$PREP/good.csv" > "$PREP/dup40.csv"
    # Lock (R5): one module's lock column never 2 -> rejected; it reads 2 on
    # one row only -> accepted.
    awk -F, -v OFS=, 'NR == 1 { print; next } $4 == "synth2" { $8 = 1 } { print }' "$PREP/good.csv" > "$PREP/lock-synth2-never.csv"
    awk -F, -v OFS=, 'NR == 1 { print; next } $4 == "sender" { $8 = 0 } { print }' "$PREP/good.csv" > "$PREP/lock-sender-never.csv"
    awk -F, -v OFS=, 'NR == FNR { if (FNR > 1 && $4 == "synth2") last = FNR; next }
      FNR == 1 { print; next } { if ($4 == "synth2") $8 = (FNR == last ? 2 : 1); print }' "$PREP/good.csv" "$PREP/good.csv" > "$PREP/lock-synth2-once.csv"
  else
    bad "M the prepared-file generator did not produce 100 rows at 25 per module"
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

if [ "$mk_ok" -eq 1 ]; then
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
  metrics_case "synth2 lock never reads 2"          "$PREP/lock-synth2-never.csv" fail 'module synth2 never reached lock'
  metrics_case "sender lock never reads 2"          "$PREP/lock-sender-never.csv" fail 'module sender never reached lock'
  metrics_case "synth2 lock reads 2 on one row"     "$PREP/lock-synth2-once.csv"  ok   -
fi

# ------------------------------------------------------------------ S
# C3: both synth sections are required and verified, in either order.  The stubs
# print FIXED final lines embedded in this check (captured once from real runs:
# one in the usual order, one swapped), never the current run's output.
echo "-- S final line: both synths verified in either order (C3); stubs from fixed lines"
if [ "$SMOKE_OK" -ne 1 ]; then
  bad "S not evaluated: tools/smoke.sh is missing"
elif [ "$mk_ok" -ne 1 ]; then
  bad "S not evaluated: the good metrics file was not prepared"
else
  rm -rf "$STUB"; mkdir -p "$STUB"
  cp "$PREP/good.csv" "$STUB/metrics-good.csv"
  cat > "$STUB/fixed-usual.txt" <<'FIXED_USUAL'
[  25s] clock LOCKED drift -34.96 ppm err 0.2 µs | rtt mean 0.087 p99 0.110 max 3.861 ms | fwd mean 0.009 p99 0.016 max 0.040 ms | ret mean 0.016 p99 0.022 max 0.068 ms | inside mean 0.062 p99 0.083 max 3.836 ms | Round trip: synth 2 mean 0.088 p99 0.113 max 3.928 ms | Round trip: feed only mean 0.084 p99 0.105 max 3.780 ms | playable(99.9%) 0.521 ms | sent 50156 lost 0 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (2347/2347 windows exact, 0 lost, 1 sender gaps), vs ideal: mixer -145.4 dB sender -145.4 dB | volume controller: PASS (0 of 2347 windows over epsilon 1e-4), 50156 messages, sent 0.9280 measured 0.9280, range 0.002..0.997, max error 2.5e-8 | music 32 voices on synth1: PASS (steady: 0 of 48328 over epsilon, max level error 2.3e-5; moving: 0 of 13272 over, max 2.5e-2), voice channels 1925/1925 windows exact, max pitch error 2.0e-3 Hz (1925 windows) | parts sent 19628 (637 notes started, 14 replaced while waiting), refreshed 382220, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: PASS (9 changes, 55 pulses, 790 repeats sent; synth playhead identical for 41097/41097 datagrams, 0 changes arrived late; metronome: 47/47 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 1925/1925 windows) | least accurate voice 0 (23 % of epsilon), now: pitch 1134.81 Hz (+1.3e-4), level 1.0000/1.0000, timbre 0.7500/0.7500, articulation 1.0000/1.0000 | also synth2: music PASS (0 of 48349 steady and 0 of 13283 moving measurements over epsilon), voice channels 1926/1926 windows exact, 0 lost, transport PASS (playhead identical for 41126/41126 datagrams, 47/47 clicks right) | mix peak 3.069 | playhead: STOPPED  bar 4 beat 6 (19.333)  6/8  tempo —  time code 01:00:08:00 @ 25
FIXED_USUAL
  cat > "$STUB/fixed-swapped.txt" <<'FIXED_SWAPPED'
[  25s] clock LOCKED drift +0.00 ppm err 0.0 µs | rtt mean 0.089 p99 0.124 max 1.617 ms | fwd mean 0.013 p99 0.023 max 1.133 ms | ret mean 0.013 p99 0.040 max 0.076 ms | inside mean 0.063 p99 0.096 max 1.529 ms | Round trip: synth 2 mean 0.085 p99 0.117 max 1.608 ms | Round trip: feed only mean 0.080 p99 0.112 max 1.103 ms | playable(99.9%) 0.529 ms | sent 50069 lost 0 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (2342/2342 windows exact, 0 lost, 1 sender gaps), vs ideal: mixer -145.4 dB sender -145.4 dB | volume controller: PASS (0 of 2342 windows over epsilon 1e-4), 50068 messages, sent 0.9816 measured 0.9816, range 0.002..0.997, max error 2.5e-8 | music 32 voices on synth2: PASS (steady: 0 of 48190 over epsilon, max level error 2.3e-5; moving: 0 of 13218 over, max 2.5e-2), voice channels 1919/1919 windows exact, max pitch error 2.0e-3 Hz (1919 windows) | parts sent 19592 (637 notes started, 14 replaced while waiting), refreshed 381504, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: PASS (9 changes, 55 pulses, 790 repeats sent; synth playhead identical for 40957/40957 datagrams, 0 changes arrived late; metronome: 47/47 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 1919/1919 windows) | least accurate voice 0 (23 % of epsilon), now: pitch 1125.00 Hz (+4.8e-14), level 1.0000/1.0000, timbre 0.7500/0.7500, articulation 1.0000/1.0000 | also synth1: music PASS (0 of 48213 steady and 0 of 13227 moving measurements over epsilon), voice channels 1920/1920 windows exact, 0 lost, transport PASS (playhead identical for 40986/40986 datagrams, 47/47 clicks right) | mix peak 2.643 | playhead: STOPPED  bar 4 beat 6 (19.333)  6/8  tempo —  time code 01:00:08:00 @ 25
FIXED_SWAPPED
  python3 - "$STUB" <<'PY'
import re, sys
stub = sys.argv[1]
U = open(stub + "/fixed-usual.txt", encoding="utf-8").read().rstrip("\n")
W = open(stub + "/fixed-swapped.txt", encoding="utf-8").read().rstrip("\n")
def at(t, sec):
    return re.sub(r'^\[ *\d+s\]', '[%4ds]' % sec, t, count=1)
def rep1(t, old, new):
    if t.count(old) != 1:
        sys.exit("fixed line: expected one %r" % old)
    return t.replace(old, new)
def dec(t, pat):
    m = re.compile(pat).search(t)
    if not m:
        sys.exit("fixed line: no match for %r" % pat)
    return t[:m.start(1)] + "%d" % (int(m.group(1)) - 1) + t[m.end(1):]
def zero_audio(t):
    out, n = re.subn(r'\(\d+/\d+ windows exact', '(0/0 windows exact', t, count=1)
    if n != 1:
        sys.exit("fixed line: no audio pair")
    return out
def rm_also(t):
    out, n = re.subn(r' \| also synth\d: [^|]*\| ', ' | ', t, count=1)
    if n != 1:
        sys.exit("fixed line: no also section")
    return out
def rm_transport(t):
    out, n = re.subn(r' \| transport: [^|]*\| ', ' | ', t, count=1)
    if n != 1:
        sys.exit("fixed line: no main transport section")
    return out
AUDIO = r'\((\d+)/\d+ windows exact'
cases = {
    "usual-all-exact": ([U], "ok"),
    "swapped-all-exact": ([W], "ok"),
    "usual-also-pair-bad": ([dec(U, r'also synth2:.*?voice channels (\d+)/\d+ windows exact')], "fail"),
    "swapped-also-pair-bad": ([dec(W, r'also synth1:.*?voice channels (\d+)/\d+ windows exact')], "fail"),
    "usual-main-pair-bad": ([dec(U, r'voices on synth1:.*?voice channels (\d+)/\d+ windows exact, max pitch')], "fail"),
    "swapped-main-pair-bad": ([dec(W, r'voices on synth2:.*?voice channels (\d+)/\d+ windows exact, max pitch')], "fail"),
    "usual-main-playhead-bad": ([dec(U, r'synth playhead identical for (\d+)/\d+ datagrams')], "fail"),
    "swapped-main-playhead-bad": ([dec(W, r'synth playhead identical for (\d+)/\d+ datagrams')], "fail"),
    "usual-main-clicks-bad": ([dec(U, r'metronome: (\d+)/\d+ clicks on the right sample')], "fail"),
    "usual-main-channel-bad": ([dec(U, r'channel exact in (\d+)/\d+ windows')], "fail"),
    "usual-no-also": ([rm_also(U)], "fail"),
    "swapped-no-also": ([rm_also(W)], "fail"),
    "usual-both-synth1": ([rep1(U, "also synth2:", "also synth1:")], "fail"),
    "usual-unknown-name": ([rep1(U, "also synth2:", "also synth3:")], "fail"),
    "usual-audio-bad": ([dec(U, AUDIO)], "fail"),
    "usual-audio-zero": ([zero_audio(U)], "fail"),
    "usual-no-transport": ([rm_transport(U)], "fail"),
    "swapped-no-transport": ([rm_transport(W)], "fail"),
    "usual-look-back": ([at(U, 24), dec(at(U, 25), AUDIO)], "fail"),
    "usual-final-only": ([at(U, 24), rm_also(at(U, 25))], "fail"),
}
TEMPLATE = """#!/usr/bin/env bash
metrics=""
while [ $# -gt 0 ]; do
  case "$1" in
    --metrics) metrics="$2"; shift 2 ;;
    *) shift ;;
  esac
done
cp "__STUB__/metrics-good.csv" "$metrics"
cat "__STUB__/summary-__NAME__.txt"
exit 0
"""
for name, (lines, _) in cases.items():
    with open(f"{stub}/summary-{name}.txt", "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    with open(f"{stub}/stub-{name}.sh", "w", encoding="utf-8") as fh:
        fh.write(TEMPLATE.replace("__STUB__", stub).replace("__NAME__", name))
    import os
    os.chmod(f"{stub}/stub-{name}.sh", 0o755)
PY
  if [ $? -ne 0 ]; then
    bad "S could not prepare the stub summaries from the fixed lines"
  else
    stub_case() { # name, expect ok|fail, phrases ('|'-separated; each must appear in the PASS line, or in the FAIL output)
      local name="$1" expect="$2" phrases="$3" T out rc p miss=""
      local -a parts
      IFS='|' read -r -a parts <<< "$phrases"
      T="$OUT/tmp-stub-$name"; rm -rf "$T"; mkdir -p "$T"
      out="$(HADROPOC_BIN="$STUB/stub-$name.sh" TMPDIR="$T" timeout 60 bash "$SMOKE" --group "239.72.68.44:$PORT_UNT" 2>&1)"; rc=$?
      if [ "$expect" = ok ]; then
        for p in "${parts[@]}"; do
          printf '%s\n' "$out" | grep '^PASS' | grep -qiF -- "$p" || miss="$miss [$p]"
        done
        if [ "$rc" -eq 0 ] && [ -z "$miss" ]; then
          pass "S $name: exit 0; PASS line: $(printf '%s\n' "$out" | grep '^PASS' | tail -n 1 | sed -E 's/.*, (synth1 and synth2 verified)$/\1/')"
        else
          bad "S $name: expected exit 0 and a PASS line with [$phrases]; got exit $rc, missing:${miss:- none}"
          printf '%s\n' "$out" | tail -n 3 | while IFS= read -r l; do note "$l"; done
        fi
      else
        if [ "$rc" -eq 0 ]; then
          bad "S $name: accepted (exit 0), must be rejected"
        else
          for p in "${parts[@]}"; do
            printf '%s\n' "$out" | grep '^FAIL' | grep -qiF -- "$p" || miss="$miss [$p]"
          done
          if [ -z "$miss" ]; then
            pass "S $name: rejected (exit $rc): $(printf '%s\n' "$out" | grep '^FAIL' | head -n 1 | cut -c1-150)"
          else
            bad "S $name: rejected (exit $rc) but no FAIL line contains:$miss"
            printf '%s\n' "$out" | tail -n 3 | while IFS= read -r l; do note "$l"; done
          fi
        fi
      fi
      rm -rf "$T"
    }
    stub_case usual-all-exact ok "synth1 and synth2 verified"
    stub_case swapped-all-exact ok "synth1 and synth2 verified"
    stub_case usual-also-pair-bad fail "synth2 voice"
    stub_case swapped-also-pair-bad fail "synth1 voice"
    stub_case usual-main-pair-bad fail "synth1 voice"
    stub_case swapped-main-pair-bad fail "synth2 voice"
    stub_case usual-main-playhead-bad fail "synth1 playhead"
    stub_case swapped-main-playhead-bad fail "synth2 playhead"
    stub_case usual-main-clicks-bad fail "synth1 clicks"
    stub_case usual-main-channel-bad fail "synth1 channel"
    stub_case usual-no-also fail "missing section|synth2"
    stub_case swapped-no-also fail "missing section|synth1"
    stub_case usual-both-synth1 fail "expected synth1 and synth2"
    stub_case usual-unknown-name fail "synth3|expected"
    stub_case usual-audio-bad fail "audio"
    stub_case usual-audio-zero fail "audio"
    stub_case usual-no-transport fail "missing section|transport"
    stub_case swapped-no-transport fail "missing section|transport"
    stub_case usual-look-back fail "audio"
    stub_case usual-final-only fail "missing section|synth2"
  fi
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
