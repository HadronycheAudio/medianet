#!/usr/bin/env bash
# checks/003-02.sh — acceptance check for ticket 003-02
# "Measure pre-fix sync rate".
#
# Deliverable: the measurement WORKING NOTES at swarm/notes/003-02.md
# (lead decision recorded in swarm/status.md: the ticket says "do not commit
# them yet", but 003-05 and 003-08 need them across agent hand-offs, so they
# are committed working notes; they are not a design or baseline document and
# no figure enters doc/design/** from this ticket).  003-08 writes BASELINE.md
# in the POC repository.
#
# The check is READ-ONLY: it never writes into the POC and never runs it.
# Override the notes path with NOTES=/path (only sensible for self-testing)
# and the POC location with HADROPOC=/path.
#
# AC4 (the POC tree-cleanliness report) honours the orchestrated epic-003
# write window: the orchestrator commits `swarm/poc-write-in-flight` in THIS
# repository; when present, that single assertion is reported as SKIP and
# never fails.  State is restored between commits, so the assertion is
# meaningful again once the marker is removed.  Override the marker location
# with MARKER_PATH=/path.
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  Two figures exist, one for `--sync-hz 50` and one for `--sync-hz 1000`,
#      each in exchanges/s, each tagged `[measured]`, each stating it was
#      measured at `poc-untouched`.  Encoding: for each rate we locate the
#      line (prose line or Markdown table row) that names that rate — either
#      the literal `--sync-hz <rate>` or a bare <rate> cell under a header
#      naming the sync rate (contains `sync`, e.g. `--sync-hz` or `sync rate`)
#      — and build a scope from it: the paragraph, or the
#      table header + row + the paragraph directly above/below the table.  In
#      that scope we require (a) the unit `exchanges/s` or `ex/s`, (b)
#      `[measured]` (case-insensitive), (c) `poc-untouched` (case-insensitive),
#      and (d) a number on the rate's own line/row (option values such as
#      `--duration 25` and hex hashes are stripped first, so the rate value
#      itself is not mistaken for the figure).  A heading naming the rate is
#      accepted when its immediately following block carries the figure.
#
# AC2  The measurement method is written down.  Encoding: a Markdown heading
#      containing "Method" (e.g. "## Method", "### Measurement method"); its
#      body must be at least 40 words OR at least 5 sentence-like fragments,
#      and must say that COMPLETED EXCHANGES are COUNTED, that the POC is not
#      modified, and must refer to the POC.  A recognisable observation
#      mechanism (tcpdump/pcap/counter/log/...) is reported but not required,
#      since the ticket leaves the method open.
#
# AC3  Machine identity and kernel CONFIG_HZ are recorded with the figures.
#      Encoding: a Markdown heading containing "Machine"; its body must carry
#      (a) a host-name field with a non-empty value, (b) a CPU field,
#      (c) a kernel field with a version number, (d) CONFIG_HZ = 250, and
#      (e) the citation P-03.  If CONFIG_HZ is recorded as anything other
#      than 250 the machine is not the measuring machine (P-03): the check
#      prints a REPORTED ANOMALY failure and additionally requires the notes
#      to state the anomaly explicitly.
#      The live host's uname values and the kernel's own CONFIG_HZ (when
#      /boot/config-$(uname -r) is readable) are printed as informational
#      notes only; the check does not assume it runs on the measuring machine.
#
# W5   Work step 5: the exact command lines and the commit are recorded.
#      Encoding: a (backslash-continuation-joined) line containing `hadropoc`
#      and `loopback` with `--duration 25`, `--sim-offset-ms 3`,
#      `--sim-drift-ppm 20` and the respective `--sync-hz <rate>`, for both
#      rates; no such line may run under sudo (the measurement is
#      unprivileged).  The notes must contain the commit that `poc-untouched`
#      resolves to in the POC (full hash, or an unambiguous 12-hex prefix).
#
# L1   Lead decision (swarm/status.md): the notes are committed WORKING NOTES
#      clearly marked as pending BASELINE.md.  Encoding: the text says
#      "working notes" and pairs `BASELINE.md` with a pending marker on one
#      line; when the default notes path is used, the file must be tracked by
#      git.  (Skipped for a NOTES override.)
#
# AC4  The POC working tree is unchanged by this ticket: `git status
#      --porcelain` in the POC is empty (ignored paths such as target/ are
#      not shown by git, so they are allowed).  This runs LAST and only
#      REPORTS the current state; the coder is bound to leave the tree clean
#      and the reviewer re-runs it.  When the orchestrator marker
#      `swarm/poc-write-in-flight` exists in this repository (or at
#      $MARKER_PATH), this single assertion is reported as SKIP and does not
#      fail: a serialised epic-003 write is in flight and the POC is expected
#      to be dirty.  The "POC is a git repository" precondition still holds.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per criterion).

set -u
export LC_ALL=C

cd "$(dirname "${BASH_SOURCE[0]}")/.." || { echo "FAIL  cannot cd to repository root"; exit 1; }

REPO_ROOT="$(pwd)"
NOTES="${NOTES:-swarm/notes/003-02.md}"
POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
MARKER_PATH="${MARKER_PATH:-$REPO_ROOT/swarm/poc-write-in-flight}"
TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/003-02.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

echo "== 003-02: pre-fix sync-rate measurement notes =="
echo "   notes: $NOTES"
echo "   POC:   $POC"
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

# For a sync rate, print "OK|<source line>" when the notes carry a figure for
# it that satisfies AC1, else "NO|<diagnostic>".  Exit 0/1 accordingly.
figure_for_rate() { # $1 = 50 or 1000
  awk -v rate="$1" '
    function isrow(s)  { return s ~ /^[[:space:]]*\|/ }
    function blank(s)  { return s ~ /^[[:space:]]*$/ }
    function ishead(s) { return s ~ /^#{1,6}[[:space:]]/ }
    { L[NR] = $0 }
    END {
      n = NR
      infence = 0
      for (i = 1; i <= n; i++) {
        if (L[i] ~ /^[[:space:]]*(```|~~~)/) { infence = 1 - infence; fence[i] = 1; continue }
        fence[i] = infence
      }
      # header of the table each row belongs to
      for (i = 1; i <= n; i++) {
        if (isrow(L[i])) {
          ts = i
          while (ts > 1 && !blank(L[ts-1]) && isrow(L[ts-1])) ts--
          thead[i] = L[ts]
        }
      }

      pat      = "--sync-hz[ =]+" rate "([^0-9]|$)"
      ratecell = "(^|[^0-9])" rate "([^0-9]|$)"
      diag = ""; best = -1

      for (i = 1; i <= n; i++) {
        if (fence[i]) { if (L[i] ~ pat) fenced = 1; continue }
        hit = 0
        if (L[i] ~ pat) hit = 1
        else if (isrow(L[i]) && index(tolower(thead[i]), "sync") > 0 && L[i] ~ ratecell) hit = 1
        if (!hit) continue

        # --- base scope: paragraph, or table header + row + caption --------
        scope = ""; numtext = ""
        if (isrow(L[i])) {
          ts = i
          while (ts > 1 && !blank(L[ts-1]) && isrow(L[ts-1])) ts--
          scope = thead[i] "\n" L[i]
          numtext = thead[i] "\n" L[i]
          q = i
          while (q < n && !blank(L[q+1]) && isrow(L[q+1])) q++
          k = ts - 1
          while (k > 0 && blank(L[k])) k--
          if (k > 0 && !fence[k] && !isrow(L[k]) && !ishead(L[k])) {
            p = k
            while (p > 1 && !blank(L[p-1]) && !isrow(L[p-1]) && !fence[p-1]) p--
            for (j = p; j <= k; j++) {
              scope = scope "\n" L[j]
              if (!ishead(L[j])) numtext = numtext "\n" L[j]
            }
          }
        } else {
          p = i
          while (p > 1 && !blank(L[p-1]) && !isrow(L[p-1]) && !fence[p-1]) p--
          q = i
          while (q < n && !blank(L[q+1]) && !isrow(L[q+1]) && !fence[q+1]) q++
          for (j = p; j <= q; j++) {
            scope = scope "\n" L[j]
            if (!ishead(L[j])) numtext = numtext "\n" L[j]
          }
        }

        # --- tolerant extension: the block right after (heading -> figure) -
        k = q + 1
        while (k <= n && blank(L[k])) k++
        if (k <= n && !fence[k]) {
          e = k
          if (isrow(L[k])) {
            while (e < n && !blank(L[e+1]) && isrow(L[e+1])) e++
          } else {
            while (e < n && !blank(L[e+1]) && !isrow(L[e+1]) && !fence[e+1]) e++
          }
          for (j = k; j <= e; j++) {
            scope = scope "\n" L[j]
            if (!ishead(L[j])) numtext = numtext "\n" L[j]
          }
        }

        hasunit = (index(scope, "exchanges/s") > 0 || index(scope, "ex/s") > 0)
        hastag  = (index(tolower(scope), "[measured]") > 0)
        hasprov = (index(tolower(scope), "poc-untouched") > 0)

        # figure number: strip option values and hex hashes, then digits
        s = numtext
        gsub(/--[a-z0-9-]+[ =][^[:space:]]+/, " ", s)
        gsub(/[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]+/, " ", s)
        hasfig = (s ~ /[0-9]/)

        if (hasunit && hastag && hasprov && hasfig) { print "OK|" L[i]; exit 0 }

        score = hasunit + hastag + hasprov + hasfig
        if (score > best) {
          best = score
          diag = "line " i ":"
          if (!hasunit) diag = diag " no unit (exchanges/s or ex/s) in scope;"
          if (!hastag)  diag = diag " no [measured] tag in scope;"
          if (!hasprov) diag = diag " scope does not say poc-untouched;"
          if (!hasfig)  diag = diag " no figure number in the rate line/row;"
          diag = diag " source: " L[i]
        }
      }
      if (diag == "") {
        if (fenced) diag = "the sync rate appears only inside a fenced code block; state the figure in prose or a table so its unit, [measured] tag and poc-untouched provenance are explicit"
        else diag = "no line or table row names --sync-hz " rate
      } else if (best == 0 && fenced) {
        diag = "the sync rate appears only inside a fenced code block; state the figure in prose or a table so its unit, [measured] tag and poc-untouched provenance are explicit"
      }
      else if (best == 0) diag = diag " (hint: a table of bare numbers needs a header that names the sync rate, e.g. --sync-hz or sync rate)"
      print "NO|" diag
      exit 1
    }' "$NOTES"
}

# ----------------------------------------------------------------- precheck
if [ ! -f "$NOTES" ]; then
  bad "precondition: notes file '$NOTES' does not exist (green work of this ticket)"
  bad "AC1 not evaluated (no notes)"
  bad "AC2 not evaluated (no notes)"
  bad "AC3 not evaluated (no notes)"
  bad "W5  not evaluated (no notes)"
  bad "L1  not evaluated (no notes)"
  notes_present=0
else
  notes_present=1
fi

if [ "$notes_present" -eq 1 ]; then

  # ------------------------------------------------------------------ AC 1
  echo "-- AC1 two exchanges/s figures, one per sync rate"
  for r in 50 1000; do
    out="$(figure_for_rate "$r")"
    case "$out" in
      OK\|*) pass "AC1 --sync-hz $r figure: $(printf '%s' "$out" | sed 's/^OK|//')" ;;
      *)     bad  "AC1 --sync-hz $r: $(printf '%s' "$out" | sed 's/^NO|//')" ;;
    esac
  done

  # ------------------------------------------------------------------ AC 2
  echo "-- AC2 measurement method"
  method="$(section_body "$NOTES" Method)"
  if ! has_heading "$NOTES" Method; then
    bad "AC2 no Markdown heading containing 'Method' (e.g. '## Method')"
  else
    pass "AC2 Method section found"
    mwords=$(printf '%s\n' "$method" | wc -w | tr -d ' ')
    msent=$(printf '%s\n' "$method" | tr '.!?' '\n\n\n' | grep -c '[^[:space:]]' || true)
    if [ "$mwords" -ge 40 ] || [ "$msent" -ge 5 ]; then
      pass "AC2 method is developed ($mwords words, $msent sentence fragments; need >=40 words or >=5 sentences)"
    else
      bad "AC2 method too short ($mwords words, $msent sentence fragments; need >=40 words or >=5 sentences)"
    fi

    # Fold the paragraph into one line so a wrapped phrase is still a phrase;
    # the character classes below still stop at a full stop (sentence end).
    low=$(printf '%s\n' "$method" | tr '\n' ' ' | tr 'A-Z' 'a-z' | tr -s ' \t' ' ')
    if printf '%s\n' "$low" | grep -Eq 'complet[a-z]*[^.]{0,40}exchang|exchang[a-z]*[^.]{0,40}complet'; then
      pass "AC2 method says how COMPLETED EXCHANGES are counted"
    else
      bad "AC2 method does not tie 'completed' to 'exchange' (say how completed exchanges are counted)"
    fi
    if printf '%s\n' "$low" | grep -q 'count'; then
      pass "AC2 method uses a counting step"
    else
      bad "AC2 method never says 'count' (how completed exchanges are counted)"
    fi
    nomod_re='without[[:space:]]+modif|without[[:space:]]+(any[[:space:]]+)?(code[[:space:]]+)?(change|chang|instrument|alter)|no[[:space:]]+(code[[:space:]]+)?(modif|chang|instrument)|unmodified|not[[:space:]]+modif|read[- ]only|does[[:space:]]+not[[:space:]]+modif|no[[:space:]]+source[[:space:]]+(chang|modif)'
    if printf '%s\n' "$low" | grep -Eq "$nomod_re"; then
      pass "AC2 method states the POC is not modified (no instrumentation)"
    else
      bad "AC2 method does not state that the POC is left unmodified (e.g. 'without modifying the POC')"
    fi
    if printf '%s\n' "$low" | grep -Eq 'poc|hadropoc'; then
      pass "AC2 method refers to the POC"
    else
      bad "AC2 method does not refer to the POC"
    fi
    if printf '%s\n' "$low" | grep -Eq 'tcpdump|pcap|packet|capture|sniff|reply|counter|stdout|status|metric|report|parse|trace'; then
      pass "AC2 method names an observation mechanism"
    else
      note "AC2 note: no familiar observation mechanism named (tcpdump/pcap/counter/log/...); reviewer must confirm the 5 % reproduction claim"
    fi
  fi

  # ------------------------------------------------------------------ AC 3
  echo "-- AC3 machine identity, CONFIG_HZ, P-03"
  machine="$(section_body "$NOTES" Machine)"
  if ! has_heading "$NOTES" Machine; then
    bad "AC3 no Markdown heading containing 'Machine'"
  else
    pass "AC3 Machine section found"

    if printf '%s\n' "$machine" | grep -Eiq '(^|[^a-z])host([[:space:]]*name)?[[:space:]]*[:|=][[:space:]]*[^[:space:]|]'; then
      pass "AC3 host name recorded: $(printf '%s\n' "$machine" | grep -Ei '(^|[^a-z])host([[:space:]]*name)?[[:space:]]*[:|=]' | head -n 1 | sed 's/^[[:space:]]*//')"
    else
      bad "AC3 no host-name field with a value in the Machine section (e.g. 'Host name: <host>')"
    fi

    if printf '%s\n' "$machine" | grep -Eiq '(^|[^a-z])(cpu|processor)([[:space:]]+(model|model name))?[[:space:]]*[:|=][[:space:]]*[^[:space:]|]'; then
      pass "AC3 CPU recorded: $(printf '%s\n' "$machine" | grep -Ei '(^|[^a-z])(cpu|processor)[[:space:]]*[:|=]' | head -n 1 | sed 's/^[[:space:]]*//')"
    else
      bad "AC3 no CPU field with a value in the Machine section (e.g. 'CPU: <model>')"
    fi

    if printf '%s\n' "$machine" | grep -Eiq 'kernel.*[0-9]+\.[0-9]+'; then
      pass "AC3 kernel version recorded: $(printf '%s\n' "$machine" | grep -Ei 'kernel.*[0-9]+\.[0-9]+' | head -n 1 | sed 's/^[[:space:]]*//')"
    else
      bad "AC3 no kernel version in the Machine section (e.g. 'Kernel: 6.12.0-...')"
    fi

    # CONFIG_HZ value: first number on the first CONFIG_HZ line that is not
    # part of a hyphenated reference such as P-03 (so "CONFIG_HZ (P-03): 250"
    # and "CONFIG_HZ=250 (P-03)" both yield 250, while "CONFIG_HZ_250=y" works
    # too).
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
      bad "AC3 CONFIG_HZ is not recorded in the Machine section"
    elif [ "$hz" = "250" ]; then
      pass "AC3 CONFIG_HZ = 250 (matches P-03)"
    else
      bad "AC3 REPORTED ANOMALY: CONFIG_HZ = $hz, not 250 (P-03) — this is not the measuring machine; the ticket requires reporting this before continuing"
      if printf '%s\n' "$machine" | grep -Eiq 'anomal|not[[:space:]]+the[[:space:]]+measuring[[:space:]]+machine|mismatch|differs|not[[:space:]]+250'; then
        pass "AC3 notes report the CONFIG_HZ anomaly explicitly"
      else
        bad "AC3 notes do not report the CONFIG_HZ anomaly explicitly"
      fi
    fi

    if printf '%s\n' "$machine" | grep -Fq 'P-03'; then
      pass "AC3 cites P-03 for CONFIG_HZ"
    else
      bad "AC3 does not cite P-03 for CONFIG_HZ"
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

  # ------------------------------------------------------------------ W 5
  echo "-- W5 exact command lines and poc-untouched commit"
  # join backslash continuations so a wrapped command is still one line
  joined="$TMP/notes-joined"
  sed -e ':a' -e '/\\$/N; s/\\\n/ /; ta' "$NOTES" > "$joined" 2>/dev/null || cp "$NOTES" "$joined"

  for r in 50 1000; do
    cmd="$(grep -E 'hadropoc' "$joined" 2>/dev/null \
      | grep -E 'loopback' \
      | grep -E -- '--sync-hz[ =]'"$r"'([^0-9]|$)' \
      | grep -E -- '--duration[ =]25' \
      | grep -E -- '--sim-offset-ms[ =]3' \
      | grep -E -- '--sim-drift-ppm[ =]20' \
      | head -n 1 || true)"
    if [ -n "$cmd" ]; then
      pass "W5 --sync-hz $r command line recorded: $cmd"
    else
      bad "W5 no 'hadropoc loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --sync-hz $r' command line"
    fi
  done

  if grep -Eq 'sudo[[:space:]].*hadropoc' "$joined" 2>/dev/null; then
    bad "W5 a recorded command runs hadropoc under sudo (the measurement is unprivileged)"
  else
    pass "W5 no recorded command uses sudo"
  fi

  if git -C "$POC" rev-parse --git-dir >/dev/null 2>&1; then
    commit=$(git -C "$POC" rev-parse -q --verify 'poc-untouched^{commit}' 2>/dev/null || true)
    if [ -z "$commit" ]; then
      bad "W5 poc-untouched does not resolve to a commit in $POC"
    elif grep -Fqi -- "$commit" "$joined"; then
      pass "W5 notes record the poc-untouched commit $commit"
    elif grep -Fqi -- "${commit:0:12}" "$joined"; then
      pass "W5 notes record the poc-untouched commit (abbreviated ${commit:0:12})"
    else
      bad "W5 notes do not record the poc-untouched commit (expected $commit or its 12-hex prefix)"
    fi
  else
    bad "W5 cannot verify the commit: $POC is not a git repository"
  fi

  # ------------------------------------------------------------------ L 1
  echo "-- L1 committed working notes, pending BASELINE.md (lead decision)"
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
  if [ "$NOTES" = "swarm/notes/003-02.md" ]; then
    if git ls-files --error-unmatch -- "$NOTES" >/dev/null 2>&1; then
      pass "L1 notes are committed (tracked by git)"
    else
      bad "L1 notes exist but are not committed (git does not track them)"
    fi
  else
    note "L1 tracking check skipped (NOTES override: $NOTES)"
  fi

fi

# -------------------------------------------------------------------- AC 4
# Reported LAST: read-only observation of the POC working tree.  Ignored
# paths (target/, experiments/<dir>/) are not listed by git status, so they
# are allowed; anything else means this ticket changed the frozen POC.
# During the orchestrated epic-003 write window the tree is expected to be
# dirty: the marker makes this single assertion a loud SKIP, never a failure.
echo "-- AC4 POC working tree unchanged (read-only; reported last)"
if ! git -C "$POC" rev-parse --git-dir >/dev/null 2>&1; then
  bad "AC4 $POC is not a git repository; cannot report its working-tree state"
elif [ -e "$MARKER_PATH" ]; then
  printf 'SKIP: epic-003 write in flight (orchestrator marker present)\n'
  note "marker: $MARKER_PATH"
  note "AC4 POC tree-cleanliness assertion skipped; this is not a failure"
else
  porcelain=$(git -C "$POC" status --porcelain 2>&1)
  if [ -z "$porcelain" ]; then
    pass "AC4 git status --porcelain in $POC is empty (clean apart from ignored paths)"
  else
    bad "AC4 POC working tree is not clean:"
    printf '%s\n' "$porcelain" | head -n 10 | while IFS= read -r l; do note "$l"; done
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-02 PASSED =="
  exit 0
fi
echo "== 003-02 FAILED =="
exit 1
