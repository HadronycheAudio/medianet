#!/usr/bin/env bash
# checks/003-04.sh — acceptance check for ticket 003-04
# "Deadline-paced clock sync wait" (epic 003, fix 2).
#
# The deliverable lives outside this repository: the fix in the frozen proof of
# concept at /home/dorian/workspace/hadropoc (the "POC"), applied by the CODER
# under the epic-003 exception.  This script only READS the POC source and git
# history; it builds and RUNS the POC (build output stays below the POC's
# `target/`, which its .gitignore excludes) and points `--metrics` at a
# temporary directory below /tmp/opencode only.  It never writes into the POC
# tree.  Override the location with HADROPOC=/path.
#
# THE FIX BASE IS 7a2d2e4, not `poc-untouched`: 003-03 already changed
# src/module.rs (process-owned metrics writer), so AC4 measures the diff
# 7a2d2e4..HEAD and attributes to this ticket only what comes after 003-03.
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  "No fixed 5 ms read timeout remains on the clock socket; every wait in
#      the clock loop is bounded by the next request deadline and by 100 ms."
#
#      This is a SOURCE-STRUCTURE check: it is a proxy, not a proof.  What it
#      requires and forbids, exactly:
#        (a) FORBIDDEN anywhere in src/module.rs: a `set_read_timeout(...)`
#            whose Duration is a POSITIVE constant literal, i.e.
#            `Duration::from_millis(<n>)/from_micros(<n>)/from_secs(<n>)` with
#            n > 0.  At the fix base the only such call is the clock socket's
#            `clock_sock.set_read_timeout(Some(Duration::from_millis(5)))` in
#            `Node::start` (module.rs:1126).  The literal 5 ms form is also
#            asserted separately because the ticket names that value.  A
#            NON-constant argument (computed timeout) is allowed, and
#            `from_millis(0)`/`set_nonblocking(true)` are allowed because they
#            never make a receive wait.
#            Caveat: net::configure() (src/net.rs:44) leaves a *100 ms* read
#            timeout on sockets opened without busy polling.  That call is in
#            the network layer, not the clock loop, and this ticket must not
#            touch net.rs; the clock loop must not rely on it for pacing.
#        (b) REQUIRED in the body of `fn spawn_clock` (extracted from the
#            column-0 `fn spawn_clock(` line to the next column-0 `}`): a wait
#            computed from a deadline minus the current monotonic time, i.e.
#            a line matching
#                <identifier> (-|.saturating_sub() (mono_ns()|now|mono)
#            (for example `let wait = (next - mono_ns()).clamp(...)`; the data
#            thread's pattern at module.rs:1663-1667 is `(d - mono_ns())`).
#            A fixed timeout such as the current `next = now + period` pacing
#            does not match: `now >= next` and `next = now + period` contain
#            no deadline-minus-now expression.
#        (c) REQUIRED in the same body: a 100 ms ceiling constant, either
#            `100_000_000` (nanoseconds, as the data thread uses) or
#            `Duration::from_millis(100)`.
#        (d) REQUIRED in the same body: a wait primitive that can carry the
#            computed timeout — `ppoll(`, `poll(` or `set_read_timeout(`.
#            At the fix base the clock loop only has a blocking `net::recv_ts(`
#            on the 5 ms socket timeout, so (d) fails.
#      (b)+(c)+(d) together are the encoding of "timeout =
#      min(next deadline - now, 100 ms)"; they cannot prove the three
#      ingredients are wired to one another, which is why AC3 measures the
#      achieved rate.  (a) alone falsifies the literal bug.
#
# AC2  "Setting the stop flag ends the clock thread within 100 ms + one loop
#      iteration."  Encoding (behavioural): a headless run
#      `loopback --duration 3 --sync-hz 50 --no-tui` must exit within 13 s of
#      wall clock with status 0, or with status 1 whose log holds only
#      zero-family lines (`check family <name>: 0 windows ...`, since 003-06:
#      a 3 s run has no music or transport samples yet, so the POC
#      legitimately reports them; AC2 accepts that and nothing else).  The
#      3 s duration + a generous 10 s margin for
#      the stop flag, one loop iteration and thread joins), and its log must
#      show at least one per-second line at 3 s (the run reached its
#      duration; it did not die early).  A clock thread that ignored the stop
#      flag would keep the process alive after the headless loop breaks; the
#      POC joins all module handles before exiting.
#
# AC3  "With --sync-hz 50 the achieved rate is 49.0-51.0 exchanges/s", using
#      the method recorded in swarm/notes/003-02.md (the before figure was
#      ~38/s).  Encoding, exactly the written method:
#        1. Run `loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20
#           --sync-hz 50 --no-tui --metrics <tmp>/metrics.csv`.
#        2. RUN VALIDITY, before parsing — clean-run precondition: at least
#           one `map: N modules` line, ALL such lines read `map: 4 modules`,
#           exactly one join line for each non-sender module (sender,
#           synth1, synth2, mixer: the sender is the process's own first
#           module and does not log a join, so exactly 3 join lines), no
#           "another module is also called" and no "2 transport references"
#           warning (both are duplicate-module symptoms), exactly one clock
#           election (`candidates` line) and exactly one
#           `clock: <name> holds it (election 1)` line.  A run failing any of
#           these is discarded and re-run, up to 3 attempts; if all attempts
#           are invalid the criterion FAILS with the last reason (it does not
#           silently skip).
#        3. Intact-row grammar (003-02 "Reproducibility"): split on commas,
#           require exactly 38 fields, name and holder in
#           {sender,synth1,synth2,mixer}, holding and forced in {0,1}, lock a
#           small integer, turnaround mean/max (fields 29,30) and rtt mean/min
#           (32,33) matching <digits>.<digit>, and served/asked/datagrams/
#           connections/band/points (31,34,35,36,37,38) plain integers.
#           Rows failing the grammar (interleaved fragments of the known
#           metrics bug) are discarded.
#        4. CHIMERA GUARD: discard any intact row whose served or asked
#           exceeds followers x rate.  The written method bounds that at
#           exactly 3 x 50 = 150.  This check applies the same guard widened
#           by 25 % (187) on purpose: a correct 50 Hz run produces ~150
#           served per one-second window, so a strict 150 would systematically
#           discard the best windows and bias the measured rate downward.
#           187 still rejects the recorded chimeric value (~1.7e12) by ten
#           orders of magnitude.
#        5. DEGENERATE-FILE RULE: use only intact holder rows (name == holder
#           and holding == 1), require one single holder identity across
#           them, DROP the first intact holder window (it can cover the
#           election), and require at least 2 windows after the drop;
#           otherwise the run is discarded and re-run.
#        6. DIVISOR RE-DERIVED from the run, not assumed: follower count =
#           (module count from the `map: N modules` line) - 1 (the holder).
#           The run's holder identity is cross-checked against the clock
#           named on the last `map:` line; a mismatch invalidates the run.
#           rate = served completed exchanges over the time the kept windows
#           actually cover / followers.  The written method's arithmetic
#           assumes every row covers exactly 1 s ("its served value is the
#           number of completed exchanges in that second"); in this POC the
#           housekeeping loop runs every HELLO_NS = 1 s PLUS its own work, so
#           the rows cover ~1.02 s and the literal mean-served/3 arithmetic
#           inflates the rate by ~2.5 % (measured: a true 49.85/s fix reads
#           51.0-51.1 literally).  The check therefore divides the sum of the
#           kept windows' served values by the span between the timestamp of
#           the dropped first window and the last kept window — the same
#           completed-exchange count with the actual window duration instead
#           of an assumed second.  It reports the literal figure alongside
#           for comparison with 003-02's 38.4 (that figure carries the same
#           inflation; its duration-corrected value is ~37.5).
#        7. PASS when 49.0 <= rate <= 51.0.
#      The measured figure, window count and rejected-row count are printed.
#
#      ISOLATION (documented deviation, not a change of measured quantity):
#      the run uses `--group 239.72.68.44:<port>` so that a concurrent POC
#      instance on this machine (loopback modules find each other on the
#      default multicast group 239.72.68.1:4460) cannot join the run and
#      violate the clean-run precondition.  The group is an address, not a
#      timing parameter; 003-05's formal record uses the default group on an
#      idle machine.  The port is derived from this shell's PID.
#
# AC4  "The ticket's diff touches only the clock-thread wait code; wire
#      format, servo and analyzer files are unchanged."  Encoding, over
#      THE COMMITS OF THIS TICKET: the commits after 7a2d2e4 whose first line
#      references 003-04 (token rule: `003-04:` prefix or `type(003-04):`
#      scope).  A diff from 7a2d2e4 would also measure later epic-003 commits
#      (003-06 touches src/main.rs), so every measure below is the union over
#      these commits (or the per-commit hunk geometry):
#        (a) at least one 003-04 commit exists — an empty set cannot satisfy
#            the criterion vacuously; the fix must be committed in the POC.
#        (b) exactly one path is changed by these commits and it is src/module.rs.
#        (c) none of transport.rs, clock.rs, audio.rs, proto.rs, net.rs,
#            rt.rs, music.rs, main.rs differs.
#        (d) the module.rs diff is SMALL: added + deleted lines <= 60 (the
#            wait is a few lines; this rejects an accidental rewrite).
#        (e) GEOMETRY: every 0-context hunk must fall inside one of exactly
#            two allowed regions — the `fn spawn_clock` body in the respective
#            revision, or the clock-socket setup line
#            `clock_sock.set_read_timeout` +/- 3 lines in `Node::start`.
#            The 5 ms call lives in start(), the wait lives in spawn_clock;
#            nothing else in module.rs is "the clock-thread wait code".
#            Hunk line numbers are taken from `git diff -U0` and compared
#            against the spans located in 7a2d2e4 and HEAD.
#
# AC5  "The two existing tests in transport.rs still pass."
#      Encoding: `cargo test transport` in the POC, exit 0, one
#      `running 2 tests` line, both `test transport::tests::<name> ... ok`
#      lines and `test result: ok. 2 passed; 0 failed;`.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per point).
#
# Cost: `cargo build --release` (cached after the first run), one 3 s run, up
# to three 25 s runs (only when a run is invalid), and `cargo test transport`.

set -u
export LC_ALL=C

POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
BASE="${BASE:-7a2d2e4}"                 # 003-03's fix; 003-04 diffs FROM here
TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/003-04.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

SRC="$POC/src/module.rs"
BIN="$POC/target/release/hadropoc"
# Isolated discovery group/port: see AC3 "ISOLATION" above.
GROUP="239.72.68.44:$(( 46000 + ($$ % 3000) ))"

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

gitpoc() { git -C "$POC" "$@"; }

echo "== 003-04: deadline-paced clock sync wait =="
echo "   POC:  $POC"
echo "   base: $BASE"
echo "   group: $GROUP (isolated; see AC3 note)"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ]; then
  bad "precondition: $POC does not exist"
  echo "== 003-04 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository"
  echo "== 003-04 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --verify -q "$BASE^{commit}" >/dev/null 2>&1; then
  bad "precondition: fix base $BASE does not resolve to a commit in the POC"
  echo "== 003-04 FAILED =="
  exit 1
fi
if [ ! -f "$SRC" ]; then
  bad "precondition: $SRC not found"
  echo "== 003-04 FAILED =="
  exit 1
fi

# -------------------------------------------------------------------- AC 1
echo "-- AC1 clock-loop wait derived from the next deadline, bounded by 100 ms"
# (a) no fixed positive read timeout remains in module.rs.
grep -n 'set_read_timeout' "$SRC" > "$TMP/read-timeouts" 2>/dev/null || true
: > "$TMP/const-timeouts"
while IFS= read -r hit; do
  [ -n "$hit" ] || continue
  val="$(printf '%s\n' "$hit" | sed -nE 's/.*from_(millis|micros|secs)[[:space:]]*\([[:space:]]*([0-9]+)[[:space:]]*\).*/\2/p')"
  if [ -n "$val" ] && [ "$val" -gt 0 ] 2>/dev/null; then
    printf '%s\n' "$hit" >> "$TMP/const-timeouts"
  fi
done < "$TMP/read-timeouts"

if [ -s "$TMP/const-timeouts" ]; then
  bad "AC1 a fixed positive set_read_timeout(...) is still present in src/module.rs:"
  head -n 5 "$TMP/const-timeouts" | while IFS= read -r l; do note "$l"; done
else
  pass "AC1 no fixed positive set_read_timeout(...) remains in src/module.rs"
fi

# The ticket names the 5 ms value; assert the literal form separately.
if grep -Eq 'set_read_timeout[^;]*from_millis[[:space:]]*\([[:space:]]*5[[:space:]]*\)' "$SRC"; then
  bad "AC1 the fixed 5 ms clock-socket read timeout is still present ($(grep -n 'from_millis[[:space:]]*([[:space:]]*5' "$SRC" | head -n 1 | cut -d: -f1):5 ms)"
else
  pass "AC1 the fixed 5 ms clock-socket read timeout is gone"
fi

# (b)-(d): spawn_clock body, from the `fn spawn_clock(` line to the next
# column-0 `}`.  The function has no nested column-0 brace.
awk '/^fn spawn_clock\(/ { inside = 1 } inside { print } inside && /^}/ { exit }' "$SRC" > "$TMP/spawn-clock.rs"
if [ ! -s "$TMP/spawn-clock.rs" ]; then
  bad "AC1 cannot locate fn spawn_clock in src/module.rs"
else
  # (b) deadline (-) now, the shape of the data thread's computed timeout.
  if grep -En '[A-Za-z_][A-Za-z0-9_]*[[:space:]]*(-|\.saturating_sub\()[[:space:]]*(mono_ns\(\)|now|mono)([^A-Za-z0-9_]|$)' "$TMP/spawn-clock.rs" > "$TMP/wait-expr"; then
    pass "AC1 clock loop computes a wait from a deadline minus the current monotonic time:"
    head -n 3 "$TMP/wait-expr" | while IFS= read -r l; do note "$l"; done
  else
    bad "AC1 no '<deadline> - mono_ns()/now' wait computation in fn spawn_clock (the loop still paces by a fixed timeout)"
  fi

  # (c) 100 ms ceiling on any single wait.
  if grep -En '(100_000_000|from_millis[[:space:]]*\([[:space:]]*100[[:space:]]*\))' "$TMP/spawn-clock.rs" > "$TMP/ceiling"; then
    pass "AC1 clock loop carries a 100 ms ceiling on the wait:"
    head -n 3 "$TMP/ceiling" | while IFS= read -r l; do note "$l"; done
  else
    bad "AC1 no 100 ms ceiling (100_000_000 ns or from_millis(100)) in fn spawn_clock"
  fi

  # (d) a wait primitive that can carry that computed timeout.
  if grep -En '(ppoll|poll|set_read_timeout)[[:space:]]*\(' "$TMP/spawn-clock.rs" > "$TMP/wait-call"; then
    pass "AC1 clock loop uses a wait primitive that can carry the computed timeout:"
    head -n 3 "$TMP/wait-call" | while IFS= read -r l; do note "$l"; done
  else
    bad "AC1 fn spawn_clock has no ppoll/poll/set_read_timeout wait (it still blocks in net::recv_ts on the socket timeout)"
  fi
fi

# ------------------------------------------------------- build (AC2/AC3/AC5)
build_ok=0
cargo_ok=0
if command -v cargo >/dev/null 2>&1; then
  cargo_ok=1
  echo "   building: (cd $POC && cargo build --release)"
  ( cd "$POC" && CARGO_TERM_COLOR=never cargo build --release ) > "$TMP/build.log" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC2/AC3 cargo build --release failed (exit $rc)"
    tail -n 15 "$TMP/build.log" | while IFS= read -r l; do note "$l"; done
  elif [ ! -x "$BIN" ]; then
    bad "AC2/AC3 built binary not found at $BIN"
  else
    build_ok=1
  fi
else
  bad "AC2/AC3/AC5 cargo not found in PATH"
fi

run_timed() { # $1 = seconds, rest = command
  local secs="$1"; shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$secs" "$@"
  else
    "$@"
  fi
}

# -------------------------------------------------------------------- AC 2
echo "-- AC2 a short --duration run exits promptly and cleanly"
if [ "$build_ok" -ne 1 ]; then
  bad "AC2 not evaluated: build failed"
else
  rm -f "$TMP/short.log"
  short_start=$(date +%s%N)
  run_timed 40 "$BIN" loopback --duration 3 --sync-hz 50 --no-tui \
    --group "$GROUP" > "$TMP/short.log" 2>&1
  rc=$?
  short_end=$(date +%s%N)
  wall_ms=$(( (short_end - short_start) / 1000000 ))
  echo "   running: $BIN loopback --duration 3 --sync-hz 50 --no-tui --group $GROUP"
  # Exit status.  A 3 s run is too short for every check family to have
  # samples: music and transport are legitimately at 0 windows until the
  # claim and settle horizon have passed (REPORT-0b section (a)).  Since
  # 003-06 the POC then exits 1 and names each such family on stderr.  So
  # this run accepts exit 0, or exit 1 whose log holds nothing but the
  # stdout summary ("[" lines), warning: lines and zero-family lines.  Any
  # other non-zero status (124 = hung clock thread, a crash, an error line)
  # still fails.  What AC2 measures (promptness, clean stop, reaching 3 s)
  # is unchanged.
  zero_only=0
  if [ "$rc" -eq 1 ]; then
    other_lines="$(grep -Ev '^\[|^warning: |^check family .*(audio|music|transport).*(0 windows|0 of 0|zero)' "$TMP/short.log" || true)"
    zero_lines_n="$(grep -Eic '^check family .*(audio|music|transport).*(0 windows|0 of 0|zero)' "$TMP/short.log" || true)"
    if [ -z "$other_lines" ] && [ "$zero_lines_n" -ge 1 ]; then
      zero_only=1
    fi
  fi
  if [ "$rc" -ne 0 ] && [ "$zero_only" -ne 1 ]; then
    bad "AC2 short run exited with status $rc after ${wall_ms} ms (a hung clock thread shows up as timeout/124; only exit 1 with zero-family lines is accepted here)"
    tail -n 10 "$TMP/short.log" | while IFS= read -r l; do note "$l"; done
  elif [ "$wall_ms" -gt 13000 ]; then
    bad "AC2 short run took ${wall_ms} ms of wall clock for a 3 s duration (limit 13000 ms: duration + 10 s margin)"
  elif [ "$zero_only" -eq 1 ]; then
    pass "AC2 short run ended in ${wall_ms} ms with exit 1 and only zero-family lines (3 s is too short for music/transport samples; 003-06)"
    grep -Ei '^check family ' "$TMP/short.log" | while IFS= read -r l; do note "$l"; done
  else
    pass "AC2 short run exited 0 after ${wall_ms} ms (3 s duration + a 10 s margin)"
  fi
  if grep -Eq '\[[[:space:]]*3s\]' "$TMP/short.log"; then
    pass "AC2 the run reached its 3 s duration (a [   3s] line is present)"
  else
    bad "AC2 no [   3s] line in the run log: the process did not reach its duration"
  fi
fi

# -------------------------------------------------------------------- AC 3
echo "-- AC3 achieved rate at --sync-hz 50 is 49.0-51.0 exchanges/s"
RATE_HZ=50
MIN_RATE=49.0
MAX_RATE=51.0

# Clean-run precondition from swarm/notes/003-02.md.  Prints a non-empty
# reason on stdout when the log is not a clean 4-module run.
log_problem() { # $1 = run log
  local log="$1" p n joins
  if [ ! -s "$log" ]; then
    echo "empty run log"
    return
  fi
  p="$(grep -E 'map: [0-9]+ modules' "$log" | grep -Ev 'map: 4 modules' | head -n 1 || true)"
  if [ -n "$p" ]; then
    echo "module count other than 4: $p"
    return
  fi
  if ! grep -qE 'map: 4 modules' "$log"; then
    echo "no 'map: 4 modules' line"
    return
  fi
  if grep -q 'another module is also called' "$log"; then
    echo "duplicate module name warning in the run log"
    return
  fi
  if grep -q '2 transport references' "$log"; then
    echo "two transport references in the run log"
    return
  fi
  joins="$(grep -cE 'net: [A-Za-z0-9_]+ joined from' "$log" || true)"
  if [ "$joins" -ne 3 ]; then
    echo "expected 3 module joins (synth1, synth2, mixer; the sender does not join), saw $joins"
    return
  fi
  for n in synth1 synth2 mixer; do
    if [ "$(grep -cE "net: $n joined from" "$log" || true)" -ne 1 ]; then
      echo "module $n did not join exactly once"
      return
    fi
  done
  if grep -qE 'net: sender joined from' "$log"; then
    echo "a second 'sender' joined the network (duplicate-module flake)"
    return
  fi
  if [ "$(grep -c 'candidates' "$log" || true)" -ne 1 ]; then
    echo "expected exactly one clock election, saw $(grep -c 'candidates' "$log" || true) candidate line(s)"
    return
  fi
  if [ "$(grep -c 'holds it (election' "$log" || true)" -ne 1 ] || ! grep -q 'holds it (election 1)' "$log"; then
    echo "expected exactly one 'clock: <name> holds it (election 1)' line"
    return
  fi
}

# Parse the metrics file with the 003-02 grammar and holder method.
# Emits key=value lines: problem, holder, windows, mean, rate, rejected,
# maxserved, followers, and one val=<served> per kept window.
parse_metrics() { # $1 = csv, $2 = followers, $3 = configured hz
  awk -F, -v nf="$2" -v hz="$3" '
    BEGIN { nh = 0; rejected = 0; maxserved = 0 }
    function intact() {
      if (NF != 38) return 0
      if ($4 !~ /^(sender|synth1|synth2|mixer)$/) return 0
      if ($5 !~ /^(sender|synth1|synth2|mixer)$/) return 0
      if ($6 !~ /^[01]$/ || $7 !~ /^[01]$/) return 0
      if ($8 !~ /^[0-9]+$/) return 0
      if ($29 !~ /^[0-9]+\.[0-9]$/ || $30 !~ /^[0-9]+\.[0-9]$/) return 0
      if ($32 !~ /^[0-9]+\.[0-9]$/ || $33 !~ /^[0-9]+\.[0-9]$/) return 0
      if ($31 !~ /^[0-9]+$/ || $34 !~ /^[0-9]+$/) return 0
      if ($35 !~ /^[0-9]+$/ || $36 !~ /^[0-9]+$/) return 0
      if ($37 !~ /^[0-9]+$/ || $38 !~ /^[0-9]+$/) return 0
      return 1
    }
    {
      if (!intact()) next
      s = $31 + 0; a = $34 + 0
      if (s > maxserved) maxserved = s
      if (s > nf * hz * 1.25 || a > nf * hz * 1.25) { rejected++; next }
      if ($6 + 0 == 1 && $4 == $5) {
        nh++
        val[nh] = s
        when[nh] = $1 + 0
        holder[$4] = 1
      }
    }
    END {
      nhh = 0
      for (h in holder) { nhh++; hname = h }
      print "nh=" nh
      print "rejected=" rejected
      print "maxserved=" maxserved
      if (nhh != 1) { print "problem=holder_count_" nhh; exit }
      print "holder=" hname
      if (nh - 1 < 2) { print "problem=too_few_intact_windows_" nh; exit }
      sum = 0
      for (i = 2; i <= nh; i++) sum += val[i]
      mean = sum / (nh - 1)
      span = (when[nh] - when[1]) / 1000.0
      print "windows=" (nh - 1)
      printf "mean=%.4f\n", mean
      printf "literal=%.4f\n", mean / nf
      printf "span_s=%.3f\n", span
      if (span <= 0) { print "problem=non_positive_window_span"; exit }
      printf "rate=%.4f\n", sum / span / nf
      print "problem="
      for (i = 2; i <= nh; i++) print "val=" val[i]
    }' "$1"
}

attempt_ok=0
attempt_reason="not run"
attempt=1
rate=""
mean=""
windows=""
literal=""
span_s=""
holder=""
followers=""
rejected=""
maxserved=""
kept_file=""
while [ "$attempt" -le 3 ] && [ "$attempt_ok" -eq 0 ]; do
  metrics="$TMP/metrics-$attempt.csv"
  log="$TMP/run-$attempt.log"
  rm -f "$metrics" "$log"
  attempt_reason=""
  if [ "$build_ok" -ne 1 ]; then
    attempt_reason="build failed"
    break
  fi
  echo "   attempt $attempt: $BIN loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --sync-hz $RATE_HZ --no-tui --metrics <tmp> --group $GROUP"
  run_timed 120 "$BIN" loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 \
    --sync-hz "$RATE_HZ" --no-tui --group "$GROUP" --metrics "$metrics" > "$log" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    attempt_reason="exit status $rc"
  elif [ ! -s "$metrics" ]; then
    attempt_reason="no metrics file"
  else
    attempt_reason="$(log_problem "$log")"
  fi

  if [ -z "$attempt_reason" ]; then
    # Follower count re-derived from the run's own map line: loopback has 4
    # modules, one of which holds the clock; the others follow it.
    modules="$(grep -oE 'map: [0-9]+ modules' "$log" | head -n 1 | grep -oE '[0-9]+')"
    if [ -z "$modules" ] || [ "$modules" -lt 2 ]; then
      attempt_reason="cannot derive the module count from the map line"
    else
      followers=$(( modules - 1 ))
      parse_out="$TMP/parse-$attempt.txt"
      parse_metrics "$metrics" "$followers" "$RATE_HZ" > "$parse_out"
      p_rate=""; p_problem=""; p_holder=""; p_windows=""; p_mean=""
      p_rejected=""; p_max=""; p_literal=""; p_span=""
      while IFS='=' read -r k v; do
        case "$k" in
          rate) p_rate="$v" ;;
          problem) p_problem="$v" ;;
          holder) p_holder="$v" ;;
          windows) p_windows="$v" ;;
          mean) p_mean="$v" ;;
          literal) p_literal="$v" ;;
          span_s) p_span="$v" ;;
          rejected) p_rejected="$v" ;;
          maxserved) p_max="$v" ;;
        esac
      done < "$parse_out"
      if [ -n "$p_problem" ]; then
        attempt_reason="metrics: $p_problem"
      elif [ -z "$p_rate" ]; then
        attempt_reason="metrics: no rate could be derived"
      else
        # Holder identity from the intact rows must agree with the clock the
        # last map line reports; a disagreement means the run is not clean.
        map_holder="$(grep 'map: 4 modules' "$log" | tail -n 1 | sed -nE 's/.*clock: ([A-Za-z0-9_]+) \(election [0-9]+\).*/\1/p')"
        if [ -n "$map_holder" ] && [ "$map_holder" != "$p_holder" ]; then
          attempt_reason="holder mismatch: metrics say $p_holder, map line says $map_holder"
        else
          attempt_ok=1
          rate="$p_rate"; mean="$p_mean"; windows="$p_windows"
          literal="$p_literal"; span_s="$p_span"
          holder="$p_holder"; rejected="$p_rejected"; maxserved="$p_max"
          kept_file="$parse_out"
        fi
      fi
    fi
  fi
  attempt=$((attempt + 1))
done

if [ "$attempt_ok" -ne 1 ]; then
  bad "AC3 no valid measurement run: $attempt_reason"
else
  note "holder=$holder  modules=4  followers=$followers  configured=${RATE_HZ}/s"
  note "kept holder windows=$windows  mean served=$mean  covered span=${span_s}s  chimera-bound=$(awk -v n="$followers" -v h="$RATE_HZ" 'BEGIN { printf "%.0f", n*h*1.25 }')  rows over bound=$rejected  max served=$maxserved"
  note "literal 003-02 arithmetic (mean served / followers, assumes 1 s windows): ${literal} exchanges/s"
  note "kept window served values:"
  { grep '^val=' "$kept_file" | cut -d= -f2 | tr '\n' ' '; echo; } | fold -s -w 100 \
    | while IFS= read -r l; do note "  $l"; done
  if awk -v r="$rate" -v lo="$MIN_RATE" -v hi="$MAX_RATE" 'BEGIN { exit !(r >= lo && r <= hi) }'; then
    pass "AC3 achieved rate ${rate} exchanges/s per follower is within ${MIN_RATE}-${MAX_RATE} (served over the ${span_s}s covered by the kept windows / ${followers} followers)"
  else
    bad "AC3 achieved rate ${rate} exchanges/s per follower is outside ${MIN_RATE}-${MAX_RATE} (literal 003-02 figure ${literal}; the pre-fix figure is ~38/s)"
  fi
fi

# -------------------------------------------------------------------- AC 5
# Run before AC4 so a build problem is reported together with the other
# runtime evidence; AC4 is static and last.
echo "-- AC5 two transport tests pass"
if [ "$cargo_ok" -ne 1 ]; then
  bad "AC5 cargo not found in PATH"
else
  echo "   running: (cd $POC && cargo test transport)"
  ( cd "$POC" && CARGO_TERM_COLOR=never cargo test transport ) > "$TMP/cargo-test.log" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC5 cargo test transport failed (exit $rc)"
    tail -n 20 "$TMP/cargo-test.log" | while IFS= read -r l; do note "$l"; done
  else
    if grep -Fxq 'running 2 tests' "$TMP/cargo-test.log"; then
      pass "AC5 cargo reports exactly 2 tests in the transport filter"
    else
      bad "AC5 expected 'running 2 tests', got: $(grep -E '^running [0-9]+ tests?$' "$TMP/cargo-test.log" | head -n 1)"
    fi
    names="$(grep -E '^test transport::tests::[A-Za-z0-9_]+ \.\.\. ok$' "$TMP/cargo-test.log" || true)"
    n="$(printf '%s\n' "$names" | sed '/^$/d' | wc -l)"
    if [ "$n" -eq 2 ]; then
      pass "AC5 both transport tests pass:"
      printf '%s\n' "$names" | while IFS= read -r l; do note "$l"; done
    else
      bad "AC5 expected 2 passing transport tests, found $n"
      grep -E '^test transport::' "$TMP/cargo-test.log" | head -n 10 | while IFS= read -r l; do note "$l"; done
    fi
    if grep -Eq '^test result: ok\. 2 passed; 0 failed;' "$TMP/cargo-test.log"; then
      pass "AC5 test result: ok. 2 passed; 0 failed"
    else
      bad "AC5 no 'test result: ok. 2 passed; 0 failed' line"
      grep -E '^test result:' "$TMP/cargo-test.log" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi
  fi
fi

# -------------------------------------------------------------------- AC 4
echo "-- AC4 diff of this ticket's commits limited to the clock-thread wait code"
# THIS TICKET'S commits: the commits after BASE whose first line references
# 003-04 (token rule).  Later epic-003 commits (003-06 touches src/main.rs) are
# judged by their own checks, so the bound is the union of the 003-04 diffs,
# not a diff from BASE (which would age as later tickets land).
t04_commits="$(gitpoc log --reverse --format='%H%x09%s' "$BASE"..HEAD 2>/dev/null \
  | awk -F'\t' '$2 ~ /(^|[^0-9A-Za-z])003-04($|[^0-9])/ { print $1 }')"
if [ -z "$t04_commits" ]; then
  bad "AC4 no commit after $BASE references 003-04: the clock-wait fix is not committed in the POC; AC4 cannot be satisfied by an empty diff"
else
  note "ticket commits: $(printf '%s' "$t04_commits" | cut -c1-12 | tr '\n' ' ')"
  : > "$TMP/changed"
  for c in $t04_commits; do
    gitpoc show --name-only --format= "$c" >> "$TMP/changed" 2>/dev/null || true
  done
  sed '/^$/d' "$TMP/changed" | sort -u > "$TMP/changed-u"
  n_changed="$(wc -l < "$TMP/changed-u" | tr -d ' ')"
  if [ "$n_changed" -eq 0 ]; then
    bad "AC4 the 003-04 commit(s) change no path: the clock-wait fix is absent"
  elif [ "$n_changed" -ne 1 ] || [ "$(cat "$TMP/changed-u")" != "src/module.rs" ]; then
    bad "AC4 expected exactly one changed path, src/module.rs, in the 003-04 commit(s); got $n_changed path(s):"
    head -n 8 "$TMP/changed-u" | while IFS= read -r p; do note "$p"; done
  else
    pass "AC4 exactly one changed path in the 003-04 commit(s): src/module.rs"

    hits="$(while IFS= read -r p; do
              b="${p##*/}"
              case "$b" in
                transport.rs|clock.rs|audio.rs|proto.rs|net.rs|rt.rs|music.rs|main.rs) printf '%s\n' "$p" ;;
              esac
            done < "$TMP/changed-u")"
    if [ -z "$hits" ]; then
      pass "AC4 no wire/servo/analyzer/audio/network/entry-point file was touched"
    else
      bad "AC4 files outside the clock-thread wait code were touched:"
      printf '%s\n' "$hits" | head -n 8 | while IFS= read -r p; do note "$p"; done
    fi

    stat="$(for c in $t04_commits; do gitpoc show --numstat --format= "$c" -- src/module.rs 2>/dev/null; done \
      | awk '{ a += $1; d += $2 } END { print a + 0, d + 0 }')"
    set -- $stat
    added="${1:-0}"; deleted="${2:-0}"
    total=$(( added + deleted ))
    if [ "$total" -le 60 ]; then
      pass "AC4 module.rs diff is small: +$added/-$deleted = $total changed line(s) (limit 60)"
    else
      bad "AC4 module.rs diff is too large: +$added/-$deleted = $total changed line(s), expected <= 60 (the wait is a few lines)"
    fi

    # (e) hunk geometry, per 003-04 commit: each hunk stays in the clock-socket
    # setup line or in fn spawn_clock, in that commit's parent and in the commit.
    span_of() { awk '/^fn spawn_clock\(/ { s = NR } s && /^}/ { printf "%d:%d\n", s, NR; exit }' "$1"; }
    in_spans() { # $1 start, $2 end, $3 "s:e ..." list
      local s="$1" e="$2" sp a b
      for sp in $3; do
        a="${sp%%:*}"; b="${sp##*:}"
        if [ "$s" -ge "$a" ] && [ "$e" -le "$b" ]; then return 0; fi
      done
      return 1
    }
    nhunks=0
    bad_hunks=0
    : > "$TMP/bad-hunks"
    geom_err=0
    for c in $t04_commits; do
      parent="$(gitpoc rev-parse "$c^" 2>/dev/null || true)"
      gitpoc show "$parent:src/module.rs" > "$TMP/old-module.rs" 2>/dev/null || : > "$TMP/old-module.rs"
      gitpoc show "$c:src/module.rs" > "$TMP/new-module.rs" 2>/dev/null || : > "$TMP/new-module.rs"
      old_span="$(span_of "$TMP/old-module.rs")"
      new_span="$(span_of "$TMP/new-module.rs")"
      old_to="$(grep -n 'clock_sock\.set_read_timeout' "$TMP/old-module.rs" 2>/dev/null | head -n 1 | cut -d: -f1)"
      new_to="$(grep -n 'clock_sock\.set_read_timeout' "$TMP/new-module.rs" 2>/dev/null | head -n 1 | cut -d: -f1)"
      if [ -z "$old_span" ] || [ -z "$new_span" ]; then
        bad "AC4 cannot locate fn spawn_clock in the parent/commit revision of module.rs (${c:0:12})"
        geom_err=1
        continue
      fi
      old_spans="$old_span"
      if [ -n "$old_to" ]; then
        os=$(( old_to - 3 )); [ "$os" -lt 1 ] && os=1
        old_spans="$old_spans $os:$(( old_to + 3 ))"
      fi
      new_spans="$new_span"
      if [ -n "$new_to" ]; then
        ns=$(( new_to - 3 )); [ "$ns" -lt 1 ] && ns=1
        new_spans="$new_spans $ns:$(( new_to + 3 ))"
      fi

      gitpoc show -U0 --format= "$c" -- src/module.rs > "$TMP/diff-module.patch" 2>/dev/null || true
      awk '
        /^@@ / {
          s = $0
          sub(/^@@ -/, "", s)
          split(s, parts, " ")
          split(parts[1], o, ",")
          n = parts[2]; sub(/^\+/, "", n); split(n, nn, ",")
          oc = (o[2] == "") ? 1 : o[2] + 0
          nc = (nn[2] == "") ? 1 : nn[2] + 0
          print o[1] + 0, oc, nn[1] + 0, nc
        }' "$TMP/diff-module.patch" > "$TMP/hunks"

      while read -r os oc ns nc; do
        [ -n "${os:-}" ] || continue
        nhunks=$(( nhunks + 1 ))
        okhunk=1
        if [ "$oc" -eq 0 ]; then
          ocheck=$(( os + 1 )); ocount=1
        else
          ocheck="$os"; ocount="$oc"
        fi
        in_spans "$ocheck" "$(( ocheck + ocount - 1 ))" "$old_spans" || okhunk=0
        if [ "$nc" -gt 0 ]; then
          in_spans "$ns" "$(( ns + nc - 1 ))" "$new_spans" || okhunk=0
        fi
        if [ "$okhunk" -ne 1 ]; then
          bad_hunks=$(( bad_hunks + 1 ))
          printf '%s: hunk old %s+%s new %s+%s\n' "${c:0:12}" "$os" "$oc" "$ns" "$nc" >> "$TMP/bad-hunks"
        fi
      done < "$TMP/hunks"
    done

    if [ "$geom_err" -ne 0 ]; then
      :
    elif [ "$nhunks" -eq 0 ]; then
      bad "AC4 the 003-04 commit(s) have no hunks in src/module.rs (the clock-wait fix is absent)"
    elif [ "$bad_hunks" -eq 0 ]; then
      pass "AC4 all $nhunks hunk(s) of the 003-04 commit(s) fall inside the clock-socket setup line or fn spawn_clock"
    else
      bad "AC4 $bad_hunks of $nhunks hunk(s) fall outside the clock-thread wait code:"
      head -n 8 "$TMP/bad-hunks" | while IFS= read -r l; do note "$l"; done
    fi
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-04 PASSED =="
  exit 0
fi
echo "== 003-04 FAILED =="
exit 1
