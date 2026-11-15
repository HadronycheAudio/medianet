#!/usr/bin/env bash
# checks/003-06.sh — acceptance check for ticket 003-06
# "Fail headless run on zero-sample family" (epic 003, fix 3).
#
# The deliverable lives outside this repository: the fix in the frozen proof of
# concept at /home/dorian/workspace/hadropoc (the "POC"), applied by the CODER
# under the epic-003 exception.  This script only READS the POC source and git
# history; it builds and RUNS the POC (build output stays below the POC's
# `target/`, which its .gitignore excludes) and keeps every output under
# /tmp/opencode/003-06/.  It never writes into the POC tree.  Override the
# location with HADROPOC=/path (used for the throwaway fixture clone only).
#
# THE FIX BASE IS 4a4885e (the POC HEAD at red time, after 003-04).  The ticket's
# diff is measured per commit: the commits in BASE..HEAD whose first line
# references 003-06.  Every commit after BASE must reference some 003-XX id,
# because 003-07..003-10 add their own commits later and this check must stay
# green after them.
#
# Summary grammar this check reads (printed by the POC itself, tui.rs
# run_headless -> headless_line, one line per second, the last one is the
# final summary).  The window denominators are the counters the zero-sample
# rule is about:
#   audio      "(X/Y windows exact, N lost, N sender gaps)"  Y = AudioInfo.analysed
#                                                            (audio.rs:309)
#   music      "voice channels X/Y windows exact, max pitch" Y = AudioInfo.music_windows
#                                                            (audio.rs:349, the synth1 segment)
#   transport  "synth playhead identical for X/Y datagrams"   Y = AudioInfo.reports
#                                                            (audio.rs:376; the POC has no
#                                                            transport-window counter, its
#                                                            verdict is NO DATA on reports == 0)
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC0  History: BASE is an ancestor of HEAD; poc-untouched is an ancestor of
#      HEAD; every commit after poc-untouched has a first line matching
#      003-NN; every commit in BASE..HEAD references 003-NN; at least one commit
#      in BASE..HEAD references 003-06 (red: none, so this FAILS now).
#      "References 003-NN" is a token rule applied to the first line: the id
#      003-NN as a whole token, either as the legacy prefix `003-NN: ...` or as
#      a Conventional Commits scope `type(003-NN): ...`.  Same rule as 003-01
#      and 003-03/003-04 (`[^0-9A-Za-z]003-NN([^0-9]|$)`).
#
# AC1  Inert case (system AC-9, epic 003-AC4): `loopback --duration 1 --no-tui`
#      with the untouched smoke flags.  At 1 s music and transport have 0
#      windows (C-32 listen delay and settle horizon, REPORT-0b section (a)),
#      audio does not.  The check reads the families at zero FROM THE RUN'S OWN
#      FINAL SUMMARY, then requires:
#        (a) exit status non-zero;
#        (b) for every family the summary shows at zero: EXACTLY ONE line on
#            stderr that names the family (word match on audio / music /
#            transport) and carries a zero-window indication ("0 windows",
#            "0 of 0" or "zero"); that line is printed as found;
#        (c) no such line on stdout (the summary lines themselves start with
#            "[" and are excluded);
#        (d) no zero-family line on stderr for a family the summary shows
#            non-zero.
#      If the 1 s run shows no zero family at all, AC1 FAILS with "could not
#      produce an inert family" (it never passes vacuously).  A duration below
#      1 s prints no summary line at all, so 1 s is the shortest duration that
#      can be judged; no shorter duration is tried.
#
# AC2  Healthy case: `loopback --duration 25 --no-tui` with the untouched smoke
#      flags exits 0, prints NO zero-family line on stderr, and its final
#      summary shows all three denominators (audio, music, transport) > 0.  A
#      zero family in this run is reported as a failure of the run.
#
# AC3  TUI mode unchanged: no commit of this ticket touches src/tui.rs, and no
#      added or removed line of this ticket's src/main.rs changes mentions
#      run_tui or ratatui (the interactive branch is untouched).
#
# AC4  Scope: the ticket's commits (BASE..HEAD, first line 003-06) touch exactly
#      one path, src/main.rs (no audio.rs, clock.rs, wire files, tui.rs,
#      Cargo.toml or Cargo.lock); the changed lines number at most 60 (added +
#      deleted); every hunk of the ticket's src/main.rs changes lies inside the
#      function that contains the headless `run_headless(` call at that
#      commit's parent (the headless end-of-run path); the family set is ONE
#      list: exactly one added line that is a bracketed literal naming "audio",
#      "music" and "transport" together (accepted form: a single-line array or
#      slice literal); and at least one added line mentions `duration` (the
#      exit is gated on --duration).
#      The stderr line's wording is NOT pinned (C-206): AC1 requires only the
#      family name and a zero indication.
#
# AC5  The two existing tests in transport.rs still pass: `cargo test` in the
#      POC exits 0, `test transport::tests::time_code ... ok` and
#      `test transport::tests::scenario ... ok` are present, and no test
#      result line reports a failure.
#
# TREE  The POC working tree is clean vs HEAD (git status --porcelain empty),
#      except while the orchestrator marker swarm/poc-write-in-flight exists
#      in the shared checkout: then this one assertion is a loud SKIP.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per point).
#
# Cost: `cargo build --release` (cached), one 1 s run, one 25 s run, and one
# full `cargo test`.  Runs are unprivileged, each on its own isolated multicast
# group (239.72.68.44 with a port in 47501-47540, never the default group),
# with --no-tui.  Before each run the script waits while another
# `hadropoc loopback` process is active (bounded wait, then a loud note).

set -u
export LC_ALL=C

SHARED_ROOT="${SHARED_ROOT:-/home/dorian/workspace/hadronyche/medianet}"
MARKER_PATH="${MARKER_PATH:-$SHARED_ROOT/swarm/poc-write-in-flight}"
POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
BASE="${BASE:-4a4885e}"                # POC HEAD at red time; 003-06 diffs from here
UNTOUCHED="poc-untouched"
BIN="$POC/target/release/hadropoc"
OUT=/tmp/opencode/003-06
mkdir -p "$OUT" || { echo "FAIL  cannot create $OUT"; exit 1; }

# Isolated discovery groups/ports (never the default group).  The two runs of
# this check use different ports, both in 47501-47540.
PORT_SHORT=$(( 47501 + ($$ % 40) ))
PORT_HEALTHY=$(( 47501 + (($$ + 20) % 40) ))
GROUP_SHORT="239.72.68.44:$PORT_SHORT"
GROUP_HEALTHY="239.72.68.44:$PORT_HEALTHY"
FAMILIES="audio music transport"

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

gitpoc() { git -C "$POC" "$@"; }

echo "== 003-06: fail headless run on zero-sample family =="
echo "   POC:   $POC"
echo "   base:  $BASE (POC HEAD at red time)"
echo "   out:   $OUT"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ] || ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository"
  echo "== 003-06 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --verify -q "$BASE^{commit}" >/dev/null 2>&1; then
  bad "precondition: fix base $BASE does not resolve to a commit in the POC"
  echo "== 003-06 FAILED =="
  exit 1
fi
if [ ! -f "$POC/src/main.rs" ]; then
  bad "precondition: $POC/src/main.rs not found"
  echo "== 003-06 FAILED =="
  exit 1
fi

# Commits of this ticket: first line references 003-06.
ticket_commits() {
  gitpoc log --reverse --format='%H%x09%s' "$BASE..HEAD" 2>/dev/null \
    | awk -F'\t' '$2 ~ /(^|[^0-9A-Za-z])003-06($|[^0-9])/ { print $1 }'
}

# -------------------------------------------------------------------- AC 0
echo "-- AC0 history: fix base, poc-untouched, 003-XX ids, a 003-06 commit"
if gitpoc merge-base --is-ancestor "$BASE" HEAD 2>/dev/null; then
  pass "AC0 fix base $BASE is an ancestor of HEAD"
else
  bad "AC0 fix base $BASE is not an ancestor of HEAD"
fi
if gitpoc merge-base --is-ancestor "$UNTOUCHED" HEAD 2>/dev/null; then
  pass "AC0 $UNTOUCHED is an ancestor of HEAD"
else
  bad "AC0 $UNTOUCHED is not an ancestor of HEAD"
fi
bad_ids="$(gitpoc log --format='%h %s' "$UNTOUCHED..HEAD" 2>/dev/null | grep -Ev '[^0-9A-Za-z]003-[0-9][0-9]([^0-9]|$)' || true)"
if [ -n "$bad_ids" ]; then
  bad "AC0 commits after $UNTOUCHED without a 003-XX id in the first line:"
  printf '%s\n' "$bad_ids" | head -n 5 | while IFS= read -r l; do note "$l"; done
else
  pass "AC0 every commit after $UNTOUCHED references a 003-XX ticket"
fi
bad_base="$(gitpoc log --format='%h %s' "$BASE..HEAD" 2>/dev/null | grep -Ev '[^0-9A-Za-z]003-[0-9][0-9]([^0-9]|$)' || true)"
if [ -n "$bad_base" ]; then
  bad "AC0 commits after $BASE without a 003-XX id in the first line:"
  printf '%s\n' "$bad_base" | head -n 5 | while IFS= read -r l; do note "$l"; done
else
  pass "AC0 every commit after $BASE references a 003-XX ticket"
fi
T_COMMITS="$(ticket_commits)"
if [ -z "$T_COMMITS" ]; then
  bad "AC0 no commit after $BASE references 003-06 (the fix is not committed in the POC)"
else
  pass "AC0 $(printf '%s\n' "$T_COMMITS" | wc -l | tr -d ' ') commit(s) reference 003-06: $(printf '%s\n' "$T_COMMITS" | cut -c1-12 | tr '\n' ' ')"
fi

# -------------------------------------------------------------- build
if ! command -v cargo >/dev/null 2>&1; then
  bad "AC1/AC2/AC5 cargo not found in PATH"
  echo "== 003-06 FAILED =="
  exit 1
fi
echo "   building: (cd $POC && cargo build --release)"
( cd "$POC" && CARGO_TERM_COLOR=never cargo build --release ) > "$OUT/build.log" 2>&1
rc=$?
if [ "$rc" -ne 0 ] || [ ! -x "$BIN" ]; then
  bad "AC1/AC2 cargo build --release failed (exit $rc)"
  tail -n 15 "$OUT/build.log" | while IFS= read -r l; do note "$l"; done
  BUILD_OK=0
else
  BUILD_OK=1
fi

# Wait while another POC loopback run is active (suite overlap), bounded.
wait_for_poc() {
  local waited=0
  while pgrep -x hadropoc >/dev/null 2>&1; do
    if [ "$waited" -ge 300 ]; then
      note "NOTE: another hadropoc loopback run is still active after 300 s; running concurrently (isolated group, not timing-sensitive)"
      return
    fi
    [ "$waited" -eq 0 ] && note "waiting for an active hadropoc loopback run to finish"
    sleep 5
    waited=$(( waited + 5 ))
  done
}

# The last per-second summary line of a headless run.
summary_line() { grep -E '^\[ *[0-9]+s\] clock' "$1" | tail -n 1; }

# Denominators of the three families' window counters, from the summary.
# Prints "audio=<n> music=<n> transport=<n>" or nothing when unreadable.
family_counts() {
  local l a m t
  l="$(summary_line "$1")"
  [ -n "$l" ] || return 0
  a="$(printf '%s\n' "$l" | sed -nE 's/.*\([0-9]+\/([0-9]+) windows exact, [0-9]+ lost, [0-9]+ sender gaps\).*/\1/p')"
  m="$(printf '%s\n' "$l" | sed -nE 's/.*voice channels [0-9]+\/([0-9]+) windows exact, max pitch.*/\1/p')"
  t="$(printf '%s\n' "$l" | sed -nE 's/.*synth playhead identical for [0-9]+\/([0-9]+) datagrams.*/\1/p')"
  [ -n "$a" ] && [ -n "$m" ] && [ -n "$t" ] || return 0
  printf 'audio=%s music=%s transport=%s\n' "$a" "$m" "$t"
}

# Zero-window line for family $1 in file $2 (stderr or stdout-without-summary).
zero_lines() { # $1 family, $2 file
  grep -Ew "$1" "$2" 2>/dev/null | grep -Ei '0 windows|0 of 0|zero' || true
}

# ------------------------------------------------------------------ AC 1
echo "-- AC1 inert case: --duration 1 --no-tui; zero families named on stderr and exit non-zero"
if [ "$BUILD_OK" -ne 1 ]; then
  bad "AC1 not evaluated: build failed"
else
  wait_for_poc
  echo "   running: $BIN loopback --duration 1 --no-tui --group $GROUP_SHORT"
  timeout 60 "$BIN" loopback --duration 1 --no-tui --group "$GROUP_SHORT" \
    > "$OUT/inert.out" 2> "$OUT/inert.err"
  rc=$?
  counts="$(family_counts "$OUT/inert.out")"
  if [ -z "$counts" ]; then
    bad "AC1 cannot read the window counts from the run's summary line (stdout: $OUT/inert.out)"
    summary_line "$OUT/inert.out" | head -c 300 | while IFS= read -r l; do note "$l"; done
  else
    note "summary windows at exit: $counts (audio: windows compared; music: voice windows; transport: datagrams)"
    zero_fams=""
    for f in $FAMILIES; do
      v="${counts#*"$f="}"; v="${v%%[!0-9]*}"
      [ "$v" = "0" ] && zero_fams="$zero_fams $f"
    done
    zero_fams="${zero_fams# }"
    if [ -z "$zero_fams" ]; then
      bad "AC1 could not produce an inert family: at --duration 1 every family shows non-zero windows ($counts); the check cannot judge the exit path"
    else
      note "families at zero in this run: $zero_fams"
      if [ "$rc" -ne 0 ]; then
        pass "AC1 inert run exited non-zero (status $rc)"
      else
        bad "AC1 inert run exited 0 although these families have zero windows: $zero_fams (the zero-sample failure is missing)"
      fi
      for f in $zero_fams; do
        hits="$(zero_lines "$f" "$OUT/inert.err")"
        n="$(printf '%s' "$hits" | grep -c . || true)"
        if [ "$n" -eq 1 ]; then
          pass "AC1 stderr names zero family $f on exactly one line:"
          note "$hits"
        elif [ "$n" -eq 0 ]; then
          bad "AC1 stderr names no zero family $f (no line with '$f' and '0 windows' / '0 of 0' / 'zero')"
        else
          bad "AC1 stderr names zero family $f on $n lines (one line per family is required):"
          printf '%s\n' "$hits" | while IFS= read -r l; do note "$l"; done
        fi
        misplaced="$(zero_lines "$f" "$OUT/inert.out" | grep -v '^\[' || true)"
        if [ -n "$misplaced" ]; then
          bad "AC1 the zero-family line for $f is on STDOUT (it must go to stderr):"
          printf '%s\n' "$misplaced" | while IFS= read -r l; do note "$l"; done
        fi
      done
      for f in $FAMILIES; do
        case " $zero_fams " in *" $f "*) continue ;; esac
        extra="$(zero_lines "$f" "$OUT/inert.err")"
        if [ -n "$extra" ]; then
          bad "AC1 stderr reports a zero family $f although the summary shows it non-zero:"
          printf '%s\n' "$extra" | while IFS= read -r l; do note "$l"; done
        fi
      done
    fi
  fi
  if [ "$rc" -ne 0 ] && [ -z "$counts" ]; then
    note "run exit status: $rc"
  fi
  tail -n 3 "$OUT/inert.err" | while IFS= read -r l; do note "stderr: $l"; done
fi

# ------------------------------------------------------------------ AC 2
echo "-- AC2 healthy case: --duration 25 --no-tui; exit 0, no zero family, all families non-zero"
if [ "$BUILD_OK" -ne 1 ]; then
  bad "AC2 not evaluated: build failed"
else
  wait_for_poc
  echo "   running: $BIN loopback --duration 25 --no-tui --group $GROUP_HEALTHY"
  timeout 120 "$BIN" loopback --duration 25 --no-tui --group "$GROUP_HEALTHY" \
    > "$OUT/healthy.out" 2> "$OUT/healthy.err"
  rc=$?
  if [ "$rc" -eq 0 ]; then
    pass "AC2 healthy run exited 0"
  else
    bad "AC2 healthy run exited with status $rc"
    tail -n 5 "$OUT/healthy.err" | while IFS= read -r l; do note "$l"; done
  fi
  hc=""
  for f in $FAMILIES; do
    hits="$(zero_lines "$f" "$OUT/healthy.err")"
    [ -n "$hits" ] && hc="$hc $f"
  done
  if [ -z "$hc" ]; then
    pass "AC2 healthy run prints no zero-family line on stderr"
  else
    bad "AC2 healthy run prints a zero-family line for:$hc"
  fi
  hcounts="$(family_counts "$OUT/healthy.out")"
  if [ -z "$hcounts" ]; then
    bad "AC2 cannot read the window counts from the final summary"
  else
    hzero=""
    for f in $FAMILIES; do
      v="${hcounts#*"$f="}"; v="${v%%[!0-9]*}"
      [ "$v" = "0" ] && hzero="$hzero $f"
    done
    if [ -z "$hzero" ]; then
      pass "AC2 final summary shows all three families non-zero ($hcounts)"
    else
      bad "AC2 the healthy run has zero windows for:$hzero ($hcounts): a failure of the run, not of the check"
    fi
  fi
fi

# ------------------------------------------------------------------ AC 3
echo "-- AC3 interactive (TUI) path unchanged"
tc="$(ticket_commits)"
tui_touch=""
for c in $tc; do
  if gitpoc show --name-only --format= "$c" 2>/dev/null | grep -qx 'src/tui.rs'; then
    tui_touch="$tui_touch ${c:0:12}"
  fi
done
if [ -z "$tui_touch" ]; then
  pass "AC3 no ticket commit touches src/tui.rs (the interactive run_tui path)"
else
  bad "AC3 ticket commit(s) touch src/tui.rs:$tui_touch"
fi
tui_lines=""
for c in $tc; do
  hit="$(gitpoc show -U0 --format= "$c" -- src/main.rs 2>/dev/null | grep -E '^[-+][^-+]' | grep -E 'run_tui|ratatui' || true)"
  [ -n "$hit" ] && tui_lines="$tui_lines
$hit"
done
if [ -z "$tui_lines" ]; then
  pass "AC3 no added or removed line of this ticket's src/main.rs mentions run_tui or ratatui"
else
  bad "AC3 this ticket changes the interactive branch of src/main.rs:"
  printf '%s\n' "$tui_lines" | head -n 5 | while IFS= read -r l; do note "$l"; done
fi

# ------------------------------------------------------------------ AC 4
echo "-- AC4 scope: exactly src/main.rs, headless exit path only, one family list"
if [ -z "$tc" ]; then
  bad "AC4 not evaluated: no 003-06 commit (see AC0)"
else
  paths_file="$OUT/ticket-paths.txt"
  : > "$paths_file"
  for c in $tc; do
    gitpoc show --name-only --format= "$c" >> "$paths_file" 2>/dev/null
  done
  sort -u "$paths_file" -o "$paths_file"
  if [ "$(cat "$paths_file")" = "src/main.rs" ]; then
    pass "AC4 the ticket's commits touch exactly one path: src/main.rs"
  else
    bad "AC4 the ticket's commits touch paths other than src/main.rs (allowed: src/main.rs only; forbidden: audio.rs, clock.rs, wire files, tui.rs, Cargo.toml, Cargo.lock):"
    while IFS= read -r p; do note "$p"; done < "$paths_file"
  fi

  changed=0
  for c in $tc; do
    n="$(gitpoc show --numstat --format= "$c" -- src/main.rs 2>/dev/null | awk '{ s += $1 + $2 } END { print s + 0 }')"
    changed=$(( changed + n ))
  done
  if [ "$changed" -gt 0 ] && [ "$changed" -le 60 ]; then
    pass "AC4 the ticket changes $changed line(s) of src/main.rs (limit 60)"
  else
    bad "AC4 the ticket changes $changed line(s) of src/main.rs (must be 1..60)"
  fi

  # Every hunk must lie inside the function that holds the headless call at
  # the commit's parent.
  region_bad=""
  for c in $tc; do
    parent="$(gitpoc rev-parse "$c^" 2>/dev/null)"
    hl="$(gitpoc show "$parent:src/main.rs" 2>/dev/null | grep -n 'run_headless(' | head -n 1 | cut -d: -f1)"
    if [ -z "$hl" ]; then
      region_bad="$region_bad ${c:0:12}(no run_headless call at parent)"
      continue
    fi
    span="$(gitpoc show "$parent:src/main.rs" 2>/dev/null | awk -v L="$hl" '
      /^(pub )?fn / { if (NR <= L) s = NR }
      /^}/ { if (NR > L && !e) e = NR }
      END { print s + 0, e + 0 }')"
    lo_s="${span% *}"; hi_s="${span#* }"
    while IFS= read -r h; do
      # @@ -a[,b] +c[,d] @@
      oldspec="$(printf '%s\n' "$h" | sed -nE 's/^@@ -([0-9]+)(,([0-9]+))? .*/\1 \3/p')"
      a="${oldspec%% *}"; b="${oldspec#* }"; [ "$b" = "$oldspec" ] && b=1; [ -z "$b" ] && b=1
      if [ "$b" -eq 0 ]; then lo="$a"; hi=$(( a + 1 )); else lo="$a"; hi=$(( a + b - 1 )); fi
      if [ "$lo_s" -eq 0 ] || [ "$lo" -lt "$lo_s" ] || [ "$hi" -gt "$hi_s" ]; then
        region_bad="$region_bad ${c:0:12}:$a,$b"
      fi
    done < <(gitpoc show -U0 --format= "$c" -- src/main.rs 2>/dev/null | grep '^@@')
  done
  if [ -z "$region_bad" ]; then
    pass "AC4 every hunk lies inside the function holding the headless run_headless call"
  else
    bad "AC4 hunk(s) outside the headless function (the diff must touch only the headless exit path):$region_bad"
  fi

  # One family list: exactly one added line naming all three families in a
  # bracketed literal.
  added="$OUT/ticket-added.txt"
  : > "$added"
  for c in $tc; do
    gitpoc show -U0 --format= "$c" -- src/main.rs 2>/dev/null | grep '^+' | grep -v '^+++' >> "$added"
  done
  list_lines="$(grep -F '"audio"' "$added" | grep -F '"music"' | grep -F '"transport"' | grep -F '[' | grep -F ']' || true)"
  nlist="$(printf '%s' "$list_lines" | grep -c . || true)"
  if [ "$nlist" -eq 1 ]; then
    pass "AC4 one family list in src/main.rs:"
    note "$(printf '%s' "$list_lines" | sed 's/^+//')"
  else
    bad "AC4 the family set is not one list: expected exactly one added line that is a bracketed literal naming \"audio\", \"music\" and \"transport\" (found $nlist)"
  fi

  if grep -q 'duration' "$added"; then
    pass "AC4 the change is gated on --duration (an added line mentions duration)"
  else
    bad "AC4 no added line mentions duration: the exit is not gated on the --duration path"
  fi
fi

# ------------------------------------------------------------------ AC 5
echo "-- AC5 cargo test: transport.rs tests pass"
if ! command -v cargo >/dev/null 2>&1; then
  bad "AC5 cargo not found in PATH"
else
  ( cd "$POC" && CARGO_TERM_COLOR=never cargo test ) > "$OUT/test.log" 2>&1
  rc=$?
  if [ "$rc" -eq 0 ]; then
    pass "AC5 cargo test exited 0"
  else
    bad "AC5 cargo test exited with status $rc"
    tail -n 15 "$OUT/test.log" | while IFS= read -r l; do note "$l"; done
  fi
  for t in time_code scenario; do
    if grep -Eq "^test transport::tests::$t \.\.\. ok\$" "$OUT/test.log"; then
      pass "AC5 transport::tests::$t ... ok"
    else
      bad "AC5 transport::tests::$t is not reported as ok"
    fi
  done
  if grep -Eq '^test result: FAILED|[1-9][0-9]* failed;' "$OUT/test.log"; then
    bad "AC5 cargo test reports a failing test"
  fi
fi

# ----------------------------------------------------------------- TREE
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
  echo "== 003-06 PASSED =="
  exit 0
fi
echo "== 003-06 FAILED =="
exit 1
