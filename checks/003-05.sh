#!/usr/bin/env bash
# checks/003-05.sh — acceptance check for ticket 003-05
# "Measure post-fix sync rate and lock" (epic 003).
#
# Deliverable: the measurement WORKING NOTES at swarm/notes/003-05.md.
# Like 003-02's notes they are committed working notes so that 003-08 (which
# writes BASELINE.md in the POC repository) and 003-09/003-10 can use them
# across agent hand-offs; no figure enters doc/design/** from this ticket.
#
# This check is READ-ONLY over the notes and over the POC's git state.  It
# never builds or runs the POC: the measurement belongs to the measurer.
# Override the notes path with NOTES=/path (only sensible for self-testing)
# and the POC location with HADROPOC=/path.
#
# The measure is the 003-02 method WITH the two hand-off corrections recorded
# in swarm/status.md and swarm/notes/003-02.md:
#   (a) the divisor is re-derived from the run itself (holder identity from
#       the intact holder rows, follower count from the `map: N modules`
#       line), not assumed 3;
#   (b) the DURATION CORRECTION: metrics rows cover ~1.02 s of housekeeping,
#       not 1 s, so the literal mean-served/3 overstates the rate by ~2.5-3.4 %.
#       The corrected rate is sum(served) / covered span / followers.
# AC1 therefore requires the notes to SHOW the corrected derivation (sum
# served, covered span, follower count), not only a bare result.
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  A figure for `--sync-hz 50` in exchanges/s, tagged `[measured]`, is the
#      duration-corrected one and lies between 49.0 and 51.0.
#      Encoding: locate the note's block for the rate — the section of a
#      heading that names it, else the heading-bounded section that mentions
#      it and not the other rate, else the prose paragraph that names it (plus
#      the next paragraph).  Command lines inside fenced code blocks are not
#      figures.  In that block require (a) the unit `exchanges/s`, `ex/s` or
#      `exchanges per second`; (b) `[measured]`; (c) the word `correct`
#      (the figure/derivation says it is duration-corrected); (d) each of the
#      three derivation ingredients with its value on the same line: `sum
#      served` + an integer, `covered span` (or `span ... covered`) + a value
#      with a time unit (s/ms), `followers = N` (or `N followers`); (e) at
#      least one exchanges/s number; (f) the NOTE's own arithmetic must hold:
#      the recorded corrected number must equal sum served / covered span /
#      followers within 0.5 exchanges/s (a derivation whose numbers do not
#      make its result is not a derivation); (g) that same number is within
#      49.0-51.0.  The literal mean-served/3 figure, when quoted, is ignored
#      for the gate; a note that records only the literal figure fails (f)/(g).
#
# AC2  A figure for `--sync-hz 1000` is recorded as a number, whatever it is,
#      tagged `[measured]`, with its derivation shown.
#      Encoding: same block lookup and same requirements (a),(b),(d),(e),(f)
#      as AC1, including the arithmetic check against the duration-corrected
#      derivation, but with NO range gate.  The tolerance is
#      max(2 exchanges/s, 1 % of the derived value).
#
# AC3  For each run: whether and WHEN the clock reached the locked state
#      (POC rule P-04: smoothed error below max(20 us, path delay/4) after
#      >= 8 windows).  Encoding, per rate block: a lock statement must be
#      present.  A "locked" statement must carry a time (`2.1 s`, `410 ms`) or
#      a window count (`window 9`, `8 windows`).  A "not locked" statement
#      ("did not lock", "never locked", "lock was not reached", "failed to
#      lock", "no lock") triggers the falsifier branch: the notes must say the
#      falsifier fired (the word `falsifier` in that run's block) and that it
#      is escalated (`escalat*` in the notes).  A stated lock time above 25 s
#      is treated the same way (the 25 s smoke run cannot have locked then).
#      Conditional sentences ("if lock was not reached ...") are ignored when
#      classifying the run.
#
# AC4  Figures carry machine identity, kernel CONFIG_HZ (250, P-03), the build
#      commit and the method reference.
#      Encoding: (a) a Machine section with host name, CPU, kernel version,
#      CONFIG_HZ = 250 and the P-03 citation (a non-250 value is a REPORTED
#      ANOMALY and additionally requires the notes to say so, as in 003-02);
#      (b) a commit hash of at least 12 hex digits that resolves in the POC
#      repository and DESCENDS from the POC's 003-04 commit (the fix must be
#      applied; a pre-fix hash such as poc-untouched's is rejected).  The
#      current POC HEAD is printed for information; a later descendant is
#      accepted because 003-06/003-07 may add POC commits after the build;
#      (c) the method reference: the notes mention `003-02`, the word
#      `method`, a correction word (`correct*`) and `003-04`.
#      Commit hashes are matched against the POC's own history, never
#      hard-coded.
#
# L1   Working-notes discipline (same lead decision as 003-02): the notes are
#      marked as WORKING NOTES and pair BASELINE.md with a pending marker;
#      when the default notes path is used the file must EXIST at that path.
#      The notes live in the machine-local swarm state directory at the
#      shared checkout ($SHARED_ROOT/swarm/notes/003-05.md on this machine's
#      disk); the swarm directory is untracked by the design owner's
#      directive (2026-11-14) and worktrees do not carry it, so tracked-ness
#      is no longer required — existence and content are.  (The existence
#      check is skipped for a NOTES override.)
#
# AC4-POC  The POC working tree is clean after the runs: `git status
#      --porcelain` in the POC is empty (ignored paths such as target/ are not
#      shown, so they are allowed).  This is reported LAST.  It honours the
#      orchestrated epic-003 write window exactly like checks/003-02.sh: when
#      the orchestrator marker `swarm/poc-write-in-flight` exists in the
#      machine-local swarm state directory at the shared checkout (or at
#      $MARKER_PATH), this single assertion is a loud SKIP,
#      never a failure.  Override the marker location with MARKER_PATH=/path.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per point).

set -u
export LC_ALL=C

cd "$(dirname "${BASH_SOURCE[0]}")/.." || { echo "FAIL  cannot cd to repository root"; exit 1; }

# The swarm state directory is machine-local at the shared checkout
# (overridable); worktrees do not carry it (swarm/ is untracked by the
# design owner's directive, 2026-11-14).
SHARED_ROOT="${SHARED_ROOT:-/home/dorian/workspace/hadronyche/medianet}"
DEFAULT_NOTES="$SHARED_ROOT/swarm/notes/003-05.md"
NOTES="${NOTES:-$DEFAULT_NOTES}"
POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
MARKER_PATH="${MARKER_PATH:-$SHARED_ROOT/swarm/poc-write-in-flight}"
TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/003-05.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

echo "== 003-05: post-fix sync-rate and lock measurement notes =="
echo "   notes:  $NOTES"
echo "   POC:    $POC"
echo "   marker: $MARKER_PATH"

# Body of the first Markdown section whose heading line contains $2
# (case-insensitive), from after the heading to the next heading of the same
# or a higher level.  Exit 1 when no such heading exists.
section_body() { # $1 file, $2 name fragment
  awk -v name="$2" '
    { L[NR] = $0 }
    END {
      want = tolower(name)
      start = 0; lvl = 0; stop = 0
      for (i = 1; i <= NR; i++) {
        if (L[i] ~ /^#{1,6}[[:space:]]/) {
          h = L[i]; sub(/[^#].*/, "", h); l = length(h)
          if (start == 0) {
            if (index(tolower(L[i]), want) > 0) { start = i; lvl = l }
          } else if (l <= lvl) { stop = i - 1; break }
        }
      }
      if (start == 0) exit 1
      if (stop == 0) stop = NR
      for (i = start + 1; i <= stop; i++) print L[i]
    }' "$1"
}

# True when the file has a Markdown heading whose text contains $2
# (case-insensitive).
has_heading() { # $1 file, $2 name fragment
  awk -v name="$2" '
    { if ($0 ~ /^#{1,6}[[:space:]]/ && index(tolower($0), tolower(name)) > 0) { found = 1; exit } }
    END { exit !found }' "$1"
}

# Print the note's block for one sync rate (see the AC1 encoding).
# Exit 1 when the rate is named only inside a fenced code block or not at all.
rate_scope() { # $1 file, $2 rate
  awk -v rate="$2" '
    { L[NR] = $0 }
    function blank(s)  { return s ~ /^[[:space:]]*$/ }
    function ishead(s) { return s ~ /^#{1,6}[[:space:]]/ }
    function isfence(s){ return s ~ /^[[:space:]]*(```|~~~)/ }
    END {
      n = NR
      infence = 0
      for (i = 1; i <= n; i++) {
        if (isfence(L[i])) { infence = 1 - infence; fence[i] = 1; continue }
        fence[i] = infence
      }
      pat  = "--sync-hz[ =]+" rate "([^0-9]|$)"
      opat = "--sync-hz[ =]+" (rate == "50" ? "1000" : "50") "([^0-9]|$)"
      cell = "(^|[^0-9])" rate "([^0-9]|$)"

      # 1) a heading that names the rate, also as a bare "50" next to
      #    sync/hz/rate (e.g. "### 50 Hz").
      for (i = 1; i <= n; i++) {
        if (!ishead(L[i])) continue
        low = tolower(L[i])
        if (L[i] ~ pat || ((low ~ /(sync|hz|rate)/) && L[i] ~ cell)) {
          h = L[i]; sub(/[^#].*/, "", h); lvl = length(h)
          for (j = i + 1; j <= n; j++) {
            if (ishead(L[j])) {
              h2 = L[j]; sub(/[^#].*/, "", h2)
              if (length(h2) <= lvl) break
            }
            print L[j]
          }
          exit 0
        }
      }

      # 2) the heading-bounded section that mentions this rate and not the
      #    other one.
      nh = 0
      for (i = 1; i <= n; i++) if (ishead(L[i])) { nh++; hs[nh] = i }
      hs[nh + 1] = n + 1
      for (k = 1; k <= nh; k++) {
        s = hs[k] + 1; e = hs[k + 1] - 1
        hr = 0; ho = 0
        for (i = s; i <= e; i++) {
          if (fence[i]) continue
          if (L[i] ~ pat) hr = 1
          if (L[i] ~ opat) ho = 1
        }
        if (hr && !ho) { for (i = s; i <= e; i++) print L[i]; exit 0 }
      }

      # 3) the first prose paragraph naming the rate, plus the next paragraph
      #    when that one does not belong to the other rate.
      for (i = 1; i <= n; i++) {
        if (fence[i] || L[i] !~ pat) continue
        p = i
        while (p > 1 && !blank(L[p - 1]) && !fence[p - 1] && !ishead(L[p - 1])) p--
        q = i
        while (q < n && !blank(L[q + 1]) && !fence[q + 1]) q++
        for (j = p; j <= q; j++) print L[j]
        k = q + 1
        while (k <= n && blank(L[k])) k++
        if (k <= n && !ishead(L[k]) && !fence[k]) {
          e = k
          while (e < n && !blank(L[e + 1]) && !fence[e + 1]) e++
          other = 0
          for (j = k; j <= e; j++) if (L[j] ~ opat) other = 1
          if (!other) for (j = k; j <= e; j++) print L[j]
        }
        exit 0
      }
      exit 1
    }' "$1"
}

# First integer stated as the total served count (sum served / served total).
extract_sum() { # $1 block file
  awk '
    {
      line = tolower($0)
      if (match(line, /(sum[^0-9]{0,25}served|served[^0-9]{0,25}(sum|total)|total[^0-9]{0,25}served)[^0-9]{0,15}[0-9]+/)) {
        s = substr(line, RSTART, RLENGTH)
        match(s, /[0-9]+$/)
        print substr(s, RSTART, RLENGTH)
        exit
      }
    }' "$1"
}

# Covered span in seconds: "span = 12.00 s", "covered span: 12234 ms",
# "12.00 s covered".
extract_span() { # $1 block file
  awk '
    function to_seconds(v, u) {
      if (u ~ /^ms/ || u ~ /^milli/) return v / 1000.0
      return v
    }
    {
      line = tolower($0)
      if (match(line, /(covered[[:space:]]+)?span[[:space:]]*(=|:|of|is|was)?[[:space:]]*[0-9]+(\.[0-9]+)?[[:space:]]*(ms|milliseconds|s|sec|secs|seconds?)/)) {
        s = substr(line, RSTART, RLENGTH)
        match(s, /[0-9]+(\.[0-9]+)?[[:space:]]*(ms|milliseconds|s|sec|secs|seconds?)/)
        frag = substr(s, RSTART, RLENGTH)
        v = frag + 0
        u = frag; sub(/^[0-9.]+[[:space:]]*/, "", u)
        printf "%.4f\n", to_seconds(v, u)
        exit
      }
      if (match(line, /[0-9]+(\.[0-9]+)?[[:space:]]*(ms|milliseconds|s|sec|secs|seconds?)[[:space:]]*(covered[[:space:]]+)?span/)) {
        s = substr(line, RSTART, RLENGTH)
        match(s, /^[0-9]+(\.[0-9]+)?/)
        v = substr(s, RSTART, RLENGTH) + 0
        u = s; sub(/^[0-9.]+[[:space:]]*/, "", u); sub(/[^a-z].*/, "", u)
        printf "%.4f\n", to_seconds(v, u)
        exit
      }
    }' "$1"
}

# Follower count: "followers = 3", "3 followers", "follower count: 3".
extract_followers() { # $1 block file
  awk '
    {
      line = tolower($0)
      if (match(line, /followers?[[:space:]]*(=|:|is|was)[[:space:]]*[0-9]+/)) {
        s = substr(line, RSTART, RLENGTH)
        match(s, /[0-9]+$/)
        print substr(s, RSTART, RLENGTH)
        exit
      }
      if (match(line, /[0-9]+[[:space:]]*followers?([^a-z]|$)/)) {
        s = substr(line, RSTART, RLENGTH)
        match(s, /^[0-9]+/)
        print substr(s, RSTART, RLENGTH)
        exit
      }
      if (match(line, /followers?[[:space:]]+count[[:space:]]*(=|:|is)?[[:space:]]*[0-9]+/)) {
        s = substr(line, RSTART, RLENGTH)
        match(s, /[0-9]+$/)
        print substr(s, RSTART, RLENGTH)
        exit
      }
    }' "$1"
}

# Every "<number> exchanges/s|ex/s|exchanges per second" value in the block.
rate_numbers() { # $1 block file
  grep -Eo '[0-9]+(\.[0-9]+)?[[:space:]]*(exchanges?/s|ex/s|exchanges?[[:space:]]+per[[:space:]]+second)' "$1" 2>/dev/null \
    | sed -E 's/^([0-9]+(\.[0-9]+)?).*/\1/'
}

# Print the note's arithmetic: sum / span / followers.
derived_value() { # $1 sum, $2 span, $3 followers
  awk -v s="$1" -v sp="$2" -v f="$3" 'BEGIN {
    if (s == "" || sp == "" || f == "" || sp <= 0 || f <= 0) { print ""; exit }
    printf "%.4f\n", s / sp / f
  }'
}

# Closest recorded exchanges/s number to the derived value.
closest_number() { # $1 candidates file, $2 target
  awk -v t="$2" '
    BEGIN { best = ""; bd = -1 }
    {
      v = $1 + 0
      d = v - t; if (d < 0) d = -d
      if (bd < 0 || d < bd) { bd = d; best = $1 }
    }
    END { if (best != "") print best }' "$1"
}

# Evaluate one rate's note block.  $1 rate, $2 gated (1 = AC1 49.0-51.0).
check_rate_block() { # $1 rate, $2 gated
  local rate="$1" gated="$2" scope="$TMP/scope-$1.txt"
  local prefix sum span followers derived values cand tol gate_lo gate_hi

  if ! rate_scope "$NOTES" "$rate" > "$scope"; then
    bad "AC$([ "$gated" -eq 1 ] && echo 1 || echo 2) --sync-hz $rate: the notes have no prose/table block naming '--sync-hz $rate' (a command line inside a fenced block is not a figure)"
    return 1
  fi
  if [ ! -s "$scope" ]; then
    bad "AC$([ "$gated" -eq 1 ] && echo 1 || echo 2) --sync-hz $rate: empty note block"
    return 1
  fi

  if [ "$gated" -eq 1 ]; then prefix="AC1"; else prefix="AC2"; fi

  # unit + [measured]
  if grep -Eq 'exchanges?/s|ex/s|exchanges?[[:space:]]+per[[:space:]]+second' "$scope"; then
    pass "$prefix --sync-hz $rate figure is in exchanges/s"
  else
    bad "$prefix --sync-hz $rate: no rate figure with the unit exchanges/s (or ex/s)"
  fi
  if grep -Eqi '\[measured\]' "$scope"; then
    pass "$prefix --sync-hz $rate figure is tagged [measured]"
  else
    bad "$prefix --sync-hz $rate: the figure is not tagged [measured]"
  fi

  # derivation ingredients
  sum="$(extract_sum "$scope")"
  span="$(extract_span "$scope")"
  followers="$(extract_followers "$scope")"

  if [ -n "$sum" ]; then
    pass "$prefix --sync-hz $rate shows sum served = $sum"
  else
    bad "$prefix --sync-hz $rate: no 'sum served = <integer>' in the block (the duration-corrected derivation must show the total served, not only the mean)"
  fi
  if [ -n "$span" ]; then
    pass "$prefix --sync-hz $rate shows covered span = ${span} s"
  else
    bad "$prefix --sync-hz $rate: no 'covered span = <value> s|ms' in the block (the rows cover ~1.02 s, so the actual span must be stated)"
  fi
  if [ -n "$followers" ]; then
    pass "$prefix --sync-hz $rate shows follower count = $followers"
  else
    bad "$prefix --sync-hz $rate: no 'followers = <n>' (or '<n> followers') in the block (the divisor is re-derived from the run)"
  fi

  if [ "$gated" -eq 1 ]; then
    if grep -Eqi 'correct' "$scope"; then
      pass "AC1 --sync-hz 50 figure is identified as duration-corrected"
    else
      bad "AC1 --sync-hz 50: the block does not say the figure/derivation is duration-corrected (the literal mean-served/3 reads ~2.5-3.4 % high)"
    fi
  fi

  derived="$(derived_value "$sum" "$span" "$followers")"
  rate_numbers "$scope" > "$TMP/values-$rate.txt"
  if [ ! -s "$TMP/values-$rate.txt" ]; then
    bad "$prefix --sync-hz $rate: no number is attached to the exchanges/s unit"
    return 1
  fi

  if [ -z "$derived" ]; then
    bad "$prefix --sync-hz $rate: cannot compute sum served / covered span / followers from the block (missing or non-positive ingredient)"
    return 1
  fi

  cand="$(closest_number "$TMP/values-$rate.txt" "$derived")"
  note "$prefix --sync-hz $rate: sum $sum / span ${span}s / followers $followers = $derived exchanges/s; recorded numbers: $(tr '\n' ' ' < "$TMP/values-$rate.txt")"

  if [ "$gated" -eq 1 ]; then
    tol="0.5"
  else
    tol="$(awk -v v="$derived" 'BEGIN { t = v * 0.01; if (t < 2) t = 2; printf "%.3f", t }')"
  fi
  if awk -v a="$cand" -v b="$derived" -v t="$tol" 'BEGIN { d = a - b; if (d < 0) d = -d; exit !(d <= t) }'; then
    pass "$prefix --sync-hz $rate recorded figure $cand agrees with its own derivation ($derived, tolerance $tol)"
  else
    bad "$prefix --sync-hz $rate recorded figure $cand contradicts its own derivation ($derived exchanges/s from sum $sum / ${span}s / $followers, tolerance $tol)"
  fi

  if [ "$gated" -eq 1 ]; then
    if awk -v r="$cand" 'BEGIN { exit !(r >= 49.0 && r <= 51.0) }'; then
      pass "AC1 --sync-hz 50 duration-corrected rate $cand exchanges/s is within 49.0-51.0"
    else
      bad "AC1 --sync-hz 50 duration-corrected rate $cand exchanges/s is outside 49.0-51.0 (the pre-fix figure is ~38/s)"
    fi
  fi
}

# AC3: lock statement for one rate's block.
check_lock() { # $1 rate
  local rate="$1" scope="$TMP/scope-$1.txt" verdict
  verdict="$(awk '
    function blank(s)  { return s ~ /^[[:space:]]*$/ }
    BEGIN { nl = 0; pos = 0; t = "" }
    {
      low = tolower($0)
      if (low ~ /^[[:space:]]*if[[:space:]]/) next
      is_nl = 0
      if (low ~ /not[[:space:]]+(yet[[:space:]]+)?lock|never[[:space:]]+lock|did[[:space:]]+not[[:space:]]+lock|could[[:space:]]+not[[:space:]]+lock|failed[[:space:]]+to[[:space:]]+lock|no[[:space:]]+lock/) is_nl = 1
      if (low ~ /(^|[^a-z])lock(ed|ing|s)?[^.]{0,30}(not|never)[[:space:]]+(be[[:space:]]+)?(reached|achieved|attained)/) is_nl = 1
      if (low ~ /(^|[^a-z])lock(ed|ing|s)?[[:space:]]*(=|:)[[:space:]]*(no|false)/) is_nl = 1
      if (low ~ /(^|[^a-z])unlocked([^a-z]|$)|(^|[^a-z])locking([^a-z]|$)/) {
        if (low !~ /(^|[^a-z])locked[[:space:]]+(at|after|in|within)|reached[[:space:]]+lock|(^|[^a-z])locked[[:space:]]*\(/) is_nl = 1
      }
      if (is_nl) { nl = 1 }
      if (low ~ /(^|[^a-z])lock/) {
        if (match(low, /(^|[^a-z])lock[^.]{0,60}[0-9]+(\.[0-9]+)?[[:space:]]*(ms|milliseconds|s|sec|secs|seconds?)/)) {
          s = substr(low, RSTART, RLENGTH)
          match(s, /[0-9]+(\.[0-9]+)?[[:space:]]*(ms|milliseconds|s|sec|secs|seconds?)/)
          frag = substr(s, RSTART, RLENGTH)
          v = frag + 0
          u = frag; sub(/^[0-9.]+[[:space:]]*/, "", u)
          if (u ~ /^ms/ || u ~ /^milli/) v = v / 1000.0
          if (t == "") t = v
          pos = 1
        }
        if (low ~ /(^|[^a-z])lock[^.]{0,60}(window[s]?[[:space:]]*[0-9]+|[0-9]+[[:space:]]*windows?)/) pos = 1
      }
    }
    END {
      if (nl) { printf "notlocked|%s\n", t; exit }
      if (pos) { printf "locked|%s\n", t; exit }
      print "absent|"
    }' "$scope")"

  case "$verdict" in
    locked\|*)
      local tm="${verdict#locked|}"
      if [ -n "$tm" ]; then
        if awk -v t="$tm" 'BEGIN { exit !(t <= 25.0) }'; then
          pass "AC3 --sync-hz $rate: LOCKED reached at ${tm} s (P-04, within the 25 s run)"
        else
          bad "AC3 --sync-hz $rate: the stated lock time ${tm} s is after the 25 s run; the falsifier has fired and the notes must say so and escalate"
        fi
      else
        pass "AC3 --sync-hz $rate: LOCKED reached, stated by window count (P-04)"
      fi
      ;;
    notlocked\|*)
      if grep -Eqi 'falsifier' "$scope"; then
        pass "AC3 --sync-hz $rate: lock not reached and the block says the falsifier fired"
      else
        bad "AC3 --sync-hz $rate: lock was not reached, so the falsifier fired; the block must say so"
      fi
      if grep -Eqi 'escalat' "$NOTES"; then
        pass "AC3 --sync-hz $rate: the non-lock finding is escalated"
      else
        bad "AC3 --sync-hz $rate: lock was not reached but the notes do not say the finding is escalated"
      fi
      ;;
    *)
      bad "AC3 --sync-hz $rate: no lock statement (say whether the clock reached LOCKED and at what time or window)"
      ;;
  esac
}

# ----------------------------------------------------------------- precheck
if [ ! -f "$NOTES" ]; then
  bad "precondition: notes file '$NOTES' does not exist (green work of this ticket)"
  bad "AC1 not evaluated (no notes)"
  bad "AC2 not evaluated (no notes)"
  bad "AC3 not evaluated (no notes)"
  bad "AC4 not evaluated (no notes)"
  bad "L1  not evaluated (no notes)"
  notes_present=0
else
  notes_present=1
fi

if [ "$notes_present" -eq 1 ]; then

  # ---------------------------------------------------------------- AC 1/2
  echo "-- AC1/AC2 figures for both rates, each with the duration-corrected derivation"
  check_rate_block 50 1
  check_rate_block 1000 0

  # ------------------------------------------------------------------ AC 3
  echo "-- AC3 lock state and time per run (P-04)"
  if [ -s "$TMP/scope-50.txt" ]; then check_lock 50; else bad "AC3 --sync-hz 50 not evaluated (no note block for the rate)"; fi
  if [ -s "$TMP/scope-1000.txt" ]; then check_lock 1000; else bad "AC3 --sync-hz 1000 not evaluated (no note block for the rate)"; fi

  # ------------------------------------------------------------------ AC 4
  echo "-- AC4 machine identity, CONFIG_HZ 250 (P-03), build commit, method reference"
  machine="$(section_body "$NOTES" Machine)"
  if ! has_heading "$NOTES" Machine; then
    bad "AC4 no Markdown heading containing 'Machine'"
  else
    pass "AC4 Machine section found"

    if printf '%s\n' "$machine" | grep -Eiq '(^|[^a-z])host([[:space:]]*name)?[[:space:]]*[:|=][[:space:]]*[^[:space:]|]'; then
      pass "AC4 host name recorded: $(printf '%s\n' "$machine" | grep -Ei '(^|[^a-z])host([[:space:]]*name)?[[:space:]]*[:|=]' | head -n 1 | sed 's/^[[:space:]]*//')"
    else
      bad "AC4 no host-name field with a value in the Machine section (e.g. 'Host name: <host>')"
    fi

    if printf '%s\n' "$machine" | grep -Eiq '(^|[^a-z])(cpu|processor)([[:space:]]+(model|model name))?[[:space:]]*[:|=][[:space:]]*[^[:space:]|]'; then
      pass "AC4 CPU recorded: $(printf '%s\n' "$machine" | grep -Ei '(^|[^a-z])(cpu|processor)[[:space:]]*[:|=]' | head -n 1 | sed 's/^[[:space:]]*//')"
    else
      bad "AC4 no CPU field with a value in the Machine section (e.g. 'CPU: <model>')"
    fi

    if printf '%s\n' "$machine" | grep -Eiq 'kernel.*[0-9]+\.[0-9]+'; then
      pass "AC4 kernel version recorded: $(printf '%s\n' "$machine" | grep -Ei 'kernel.*[0-9]+\.[0-9]+' | head -n 1 | sed 's/^[[:space:]]*//')"
    else
      bad "AC4 no kernel version in the Machine section (e.g. 'Kernel: 6.12.0-...')"
    fi

    hz=$(printf '%s\n' "$machine" | grep -i 'CONFIG_HZ' | awk '
      {
        s = $0
        while (match(s, /[0-9]+/)) {
          pre = (RSTART > 1) ? substr(s, RSTART - 1, 1) : ""
          if (pre != "-") { print substr(s, RSTART, RLENGTH); found = 1; exit }
          s = substr(s, RSTART + RLENGTH)
        }
      }
      END { if (!found) exit 1 }' | head -n 1 || true)
    if [ -z "${hz:-}" ]; then
      bad "AC4 CONFIG_HZ is not recorded in the Machine section"
    elif [ "$hz" = "250" ]; then
      pass "AC4 CONFIG_HZ = 250 (matches P-03)"
    else
      bad "AC4 REPORTED ANOMALY: CONFIG_HZ = $hz, not 250 (P-03) — this is not the measuring machine; the notes must report this"
      if printf '%s\n' "$machine" | grep -Eiq 'anomal|not[[:space:]]+the[[:space:]]+measuring[[:space:]]+machine|mismatch|differs|not[[:space:]]+250'; then
        pass "AC4 notes report the CONFIG_HZ anomaly explicitly"
      else
        bad "AC4 notes do not report the CONFIG_HZ anomaly explicitly"
      fi
    fi

    if printf '%s\n' "$machine" | grep -Fq 'P-03'; then
      pass "AC4 cites P-03 for CONFIG_HZ"
    else
      bad "AC4 does not cite P-03 for CONFIG_HZ"
    fi

    note "informational: this host uname -n=$(uname -n 2>/dev/null || echo '?') uname -r=$(uname -r 2>/dev/null || echo '?') uname -m=$(uname -m 2>/dev/null || echo '?')"
    cfg="/boot/config-$(uname -r 2>/dev/null)"
    if [ -r "$cfg" ]; then
      live_hz=$(grep -Eo '^CONFIG_HZ=[0-9]+' "$cfg" 2>/dev/null | head -n 1 | cut -d= -f2 || true)
      if [ -n "${live_hz:-}" ]; then
        note "informational: this kernel's build config says CONFIG_HZ=$live_hz"
      fi
    fi
  fi

  # build commit: resolve in the POC and require descent from the 003-04 fix.
  if ! git -C "$POC" rev-parse --git-dir >/dev/null 2>&1; then
    bad "AC4 $POC is not a git repository; cannot validate the build commit"
  else
    fix_commit="$(git -C "$POC" log --all --format='%H%x09%s' 2>/dev/null \
      | awk -F'\t' '$2 ~ /^(003-04([: ]|$)|[a-z]+\(003-04\)!?: )/ { print $1; exit }')"
    if [ -z "$fix_commit" ]; then
      bad "AC4 the POC history has no 003-04 commit; the notes cannot be about the fixed build"
    else
      stated=""
      resolved=""
      for h in $(grep -Eo '[0-9a-f]{12,40}' "$NOTES" 2>/dev/null | sort -u); do
        r="$(git -C "$POC" rev-parse -q --verify "$h^{commit}" 2>/dev/null || true)"
        [ -n "$r" ] || continue
        if git -C "$POC" merge-base --is-ancestor "$fix_commit" "$r" 2>/dev/null; then
          stated="$h"; resolved="$r"; break
        fi
      done
      head_commit="$(git -C "$POC" rev-parse HEAD 2>/dev/null || true)"
      if [ -z "$stated" ]; then
        bad "AC4 no commit hash in the notes resolves in the POC to a build with 003-04 applied (fix commit ${fix_commit:0:12}); a pre-fix hash is not acceptable"
      else
        pass "AC4 build commit recorded as $stated (resolves to ${resolved:0:12}, descendant of 003-04 ${fix_commit:0:12})"
        if [ "$resolved" = "$head_commit" ]; then
          pass "AC4 the stated build commit is the current POC HEAD"
        else
          note "informational: the stated build commit is ${resolved:0:12}; current POC HEAD is ${head_commit:0:12} (later commits may exist; the measured build must include 003-04)"
        fi
        if grep -q '003-04' "$NOTES"; then
          pass "AC4 notes say the build has 003-04 applied"
        else
          bad "AC4 notes do not state that 003-04 is applied to the measured build"
        fi
      fi
    fi
  fi

  # method reference: 003-02's method with the corrections.
  echo "-- AC4 method reference (003-02 method, with the hand-off corrections)"
  if grep -q '003-02' "$NOTES"; then
    pass "AC4 method reference names 003-02"
  else
    bad "AC4 the notes do not reference 003-02's recorded method"
  fi
  if grep -Eqi 'method' "$NOTES"; then
    pass "AC4 method reference present"
  else
    bad "AC4 the notes do not name a method"
  fi
  if grep -Eqi 'correct' "$NOTES"; then
    pass "AC4 method reference carries the duration correction"
  else
    bad "AC4 the notes do not carry the duration correction (sum served / covered span / followers)"
  fi
  if grep -q '003-04' "$NOTES"; then
    pass "AC4 notes are tied to the 003-04 fix"
  else
    bad "AC4 the notes are not tied to 003-04 (the fix whose effect they measure)"
  fi
  if grep -Eqi 'map: [0-9]+ modules|clean[- ]run|chimera|degenerate' "$NOTES"; then
    pass "AC4 the notes mention the run-validity rules inherited from 003-02"
  else
    note "AC4 note: the notes do not mention the inherited run-validity rules (clean-run precondition, degenerate-file rule, chimera guard); the reference to 003-02's method is accepted, but the reviewer may ask for it"
  fi

  # ------------------------------------------------------------------ L 1
  echo "-- L1 working notes on disk at the default path, pending BASELINE.md"
  if grep -Eqi 'working[ -]notes' "$NOTES"; then
    pass "L1 notes are marked as WORKING NOTES"
  else
    bad "L1 notes are not marked as WORKING NOTES (add a 'Working notes' marker)"
  fi
  if grep -Eqi 'baseline\.md.*(pending|not yet|not final|to be written|written later|will[[:space:]]+(be[[:space:]]+)?writ)|(pending|not yet|not final|to be written|written later|will[[:space:]]+(be[[:space:]]+)?writ).*baseline\.md' "$NOTES"; then
    pass "L1 notes mark the figures as pending BASELINE.md"
  else
    bad "L1 notes do not mark the figures as pending BASELINE.md (003-08 writes it in the POC repository)"
  fi
  if [ "$NOTES" = "$DEFAULT_NOTES" ]; then
    # swarm/ is untracked by the design owner's directive (2026-11-14): the
    # notes are required in the machine-local swarm state directory at the
    # shared checkout, not in the git tree; worktrees do not carry it.
    if [ -f "$NOTES" ]; then
      pass "L1 notes exist at the default path $DEFAULT_NOTES on this machine's disk (tracked-ness not required: swarm/ is untracked per the design owner's 2026-11-14 directive)"
    else
      bad "L1 notes file does not exist at the default path $DEFAULT_NOTES"
    fi
  else
    note "L1 default-path existence check skipped (NOTES override: $NOTES)"
  fi

fi

# ---------------------------------------------------------------- AC4-POC
# Reported LAST: read-only observation of the POC working tree.  Ignored
# paths (target/, experiments/<dir>/) are not listed by git status, so they
# are allowed; anything else means a run left the frozen POC dirty.
# During the orchestrated epic-003 write window the tree is expected to be
# dirty: the marker makes this single assertion a loud SKIP, never a failure.
echo "-- AC4-POC POC working tree clean after the runs (read-only; reported last)"
if ! git -C "$POC" rev-parse --git-dir >/dev/null 2>&1; then
  bad "AC4-POC $POC is not a git repository; cannot report its working-tree state"
elif [ -e "$MARKER_PATH" ]; then
  printf 'SKIP: epic-003 write in flight (orchestrator marker present)\n'
  note "marker: $MARKER_PATH"
  note "AC4-POC POC tree-cleanliness assertion skipped; this is not a failure"
else
  porcelain=$(git -C "$POC" status --porcelain 2>&1)
  if [ -z "$porcelain" ]; then
    pass "AC4-POC git status --porcelain in $POC is empty (clean apart from ignored paths)"
  else
    bad "AC4-POC POC working tree is not clean (a run wrote into the POC):"
    printf '%s\n' "$porcelain" | head -n 10 | while IFS= read -r l; do note "$l"; done
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-05 PASSED =="
  exit 0
fi
echo "== 003-05 FAILED =="
exit 1
