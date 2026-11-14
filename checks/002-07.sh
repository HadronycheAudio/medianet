#!/usr/bin/env bash
# checks/002-07.sh — checks for ticket 002-07
# "Loss counter: per-hop logging run".
#
# The deliverable is section (b) The 21.7 M counter of
#   doc/design/reports/archaeology/REPORT-0b.md
# plus two committed companion files beside it:
#   - the scratch-copy logging patch (*002-07*.diff or *.patch);
#   - the saved log of the 25 s instrumented run (*002-07*.log).
# The run must be `hadropoc loopback --duration 25` from the scratch copy
# (/home/dorian/workspace/hadronyche/poc-scratch), never the frozen tree.
#
# This script reads the report and the two companion files. It never writes
# anywhere and never touches the frozen POC or the scratch copy.
#
# The conventions below are spelled out so the author can comply exactly.
#
# * Companion diff (AC 1). A regular, non-empty, git-tracked file in
#   doc/design/reports/archaeology/ whose name contains "002-07" and ends in
#   ".diff" or ".patch" (a diff whose content names 002-07 is accepted as a
#   fallback). It must:
#     - carry a unified-diff header (`---`/`+++` or `diff --git`/`Index:`);
#     - contain at least one added line with a logging cue: eprintln!,
#       println!, print!, dbg!, log::, tracing, log_line, hop_log, stderr or
#       stdout (this is the "diff touches only logging" evidence);
#     - not name the frozen tree (/home/dorian/workspace/hadropoc) anywhere;
#     - in its header/path lines (`diff`, `---`, `+++`, `Index:`, `Only in`),
#       contain no absolute path outside a directory named poc-scratch;
#       relative paths and /dev/null are fine. (A `git diff` made inside the
#       scratch copy, or `diff -u src/x.rs.orig src/x.rs`, therefore passes;
#       `diff -ruN /home/dorian/workspace/hadropoc ...` does not.)
#   Section (b) must also state that the patch is logging-only / changes no
#   behaviour, and that the frozen-tree fingerprint is unchanged.
#
# * Companion log (AC 2). A regular, non-empty, git-tracked file in the same
#   directory whose name contains "002-07" and ends in ".log" (".txt" is
#   accepted as a fallback). Expected record grammar, case-insensitive,
#   fields in any order on one line:
#
#       [<timestamp>] hop=<1..5> module=<synth1|synth2|mixer|sender|stats|tui> ...
#
#   The hop tag is "hop" or "h", an optional separator from [ _.:#=-], then a
#   digit 1..5 that is followed by a non-alphanumeric character or the end of
#   line. Examples: hop=1  hop 1  hop1  h1  [hop2]  hop_3. "hop 10" does not
#   match, nor does "hop1x". The module tag is synth1/synth2/mixer, accepting
#   one space, underscore or hyphen between "synth" and the digit (so
#   "synth 1" is read as synth1). The check requires:
#     - at least one hop-tagged line for each hop 1, 2, 3, 4, 5;
#     - for hops 1 and 2 (the echo-side counter and header-write hops), at
#       least one hop-tagged line from each of synth1, synth2 and mixer;
#     - the three module names synth1, synth2 and mixer to appear on
#       hop-tagged lines. Hops 3-5 are sender-side (sender/stats/tui) and only
#       need their hop id, so the check does not force a module spelling on
#       them.
#   The check deliberately does NOT require a particular spelling for the
#   input/return-stream id, sequence number or raw value fields; the author
#   may name those freely. The log should be plain text (for example the
#   --no-tui output or stderr), not the full-screen TUI with escape codes.
#
# * Captured display (AC 3). Section (b) must contain a captured one-line
#   display line, inside a fenced code block or inline backticks, carrying
#   BOTH a "sent"/"received" count and a "lost" count, and tagged [measured].
#   For a fenced block the tag may sit on either fence line, inside the
#   block, on the caption line immediately before the opening fence, or as a
#   standalone tag line immediately after the closing fence; a neighbouring
#   prose line's tag does not count. For inline output the tag must be on the
#   line or an adjacent line. The lost figure must be >= 1 000 000, accepting
#   spaces, commas and underscores in the number and K/M/million suffixes; the
#   "(fwd ...)" field on the same line is accepted as the pooled counter
#   because that is where the display prints lost_fwd. OR the section must
#   state explicitly that the anomaly did not reproduce (not reproduced / did
#   not reproduce / not observed / ...). Both polarities are acceptable, but
#   the captured line must exist and be tagged. Silence is not acceptable.
#
# * Excerpt (work step 4). Section (b)'s fenced log blocks must contain, for
#   each hop 1..5, a run of at least 5 consecutive lines carrying that hop
#   tag. Runs are counted inside one fenced block; a blank line, a prose line,
#   or a line with no hop tag (or several hop tags) breaks every run. A line
#   carrying exactly one hop tag extends that hop's run. Every fenced block
#   that supplies such a run must be tagged [measured] under the same rule as
#   the captured display block above.
#
# * Rate limit (work step 1). Section (b) must state that logging was
#   rate-limited to at most 10 lines per second per hop (for example
#   "rate-limited to 10 lines/s/hop"). A hard per-second density check on the
#   log is deliberately not made: the log's timestamp grammar is the author's
#   choice, so a mechanical rate check would be brittle. The written contract
#   is what this check verifies.
#
# * DoD. Section (b) keeps exactly one verdict line and it still says
#   "pending" (the verdict belongs to 002-08). The run is stated to have come
#   from the scratch copy, with the frozen tree neither run nor written. The
#   25 s duration is stated. Every file:line pointer in prose carries an
#   evidence tag on itself or an adjacent line, and the diff/log/excerpt
#   claims carry tags.
#
# Exit status: 0 when every check passes, 1 otherwise.

set -u

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REPORT_REL="doc/design/reports/archaeology/REPORT-0b.md"
ARCH_REL="doc/design/reports/archaeology"
REPORT="$REPO_ROOT/$REPORT_REL"
ARCH="$REPO_ROOT/$ARCH_REL"
FROZEN_ROOT="/home/dorian/workspace/hadropoc"

pass=0
fail=0

ok()  { printf 'PASS: %s\n' "$*"; pass=$((pass + 1)); }
bad() { printf 'FAIL: %s\n' "$*"; fail=$((fail + 1)); }

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Line number of the first heading (line beginning with #) matching the ERE.
heading_no() {
  local file="$1" re="$2"
  grep -Ein "$re" "$file" | awk -F: '/^[0-9]+:#/ { print $1; exit }'
}

# Print lines $2..$3 (1-based, inclusive) of file $1.
line_range() {
  local file="$1" from="$2" to="$3"
  [ "$from" -le "$to" ] || return 0
  sed -n "${from},${to}p" "$file"
}

has_tag() {
  printf '%s\n' "$1" | grep -Eq '\[(code|measured|kernel-doc|memory|decision)\]'
}

is_fence_line() {
  printf '%s\n' "$1" | grep -Eq '^[[:space:]]*```'
}

# Is the repository-relative path $1 tracked (staged or committed)?
is_tracked() {
  git -C "$REPO_ROOT" ls-files --error-unmatch -- "$1" >/dev/null 2>&1
}

# A line that is only a [measured] marker (optionally emphasised/punctuated).
tag_only_line() {
  printf '%s\n' "$1" | grep -Eq '^[[:space:]]*[*_]*\[measured\][*_]*[[:space:]]*[.;:]?[[:space:]]*$'
}

# Is the fenced block at lines $1-$2 tagged [measured]? A tag on either fence
# line, anywhere inside the block, on the caption line immediately before the
# opening fence, or as a standalone tag line immediately after the closing
# fence counts. A prose line after the block that merely mentions [measured]
# does NOT count, so a neighbouring claim's tag cannot leak onto this block.
block_tagged() {
  local fs="$1" fe="$2" k line
  if line_range "$REPORT" "$fs" "$fe" | grep -qF '[measured]'; then
    return 0
  fi
  k=$((fs - 1))
  while [ "$k" -ge 1 ]; do
    line=$(sed -n "${k}p" "$REPORT")
    [ -n "$(printf '%s' "$line" | tr -d '[:space:]')" ] && break
    k=$((k - 1))
  done
  if [ "$k" -ge 1 ] && printf '%s\n' "$line" | grep -qF '[measured]'; then
    return 0
  fi
  k=$((fe + 1))
  while [ "$k" -le "$TOTAL_LINES" ]; do
    line=$(sed -n "${k}p" "$REPORT")
    [ -n "$(printf '%s' "$line" | tr -d '[:space:]')" ] && break
    k=$((k + 1))
  done
  if [ "$k" -le "$TOTAL_LINES" ] && tag_only_line "$line"; then
    return 0
  fi
  return 1
}

# Extract the distinct hop ids 1..5 carried by one line, according to the
# grammar documented in the header comment.
line_hops() {
  printf '%s\n' "$1" \
    | grep -Eoi '(^|[^[:alnum:]_])(hop|h)[ _.:#=-]*[1-5]([^[:alnum:]]|$)' \
    | grep -oE '[1-5]' | sort -u
}

# Count log lines carrying hop $1.
hop_count() {
  grep -Eci "(^|[^[:alnum:]_])(hop|h)[ _.:#=-]*$1([^[:alnum:]]|$)" "$LOG_FILE" || true
}

# Count log lines carrying hop $1 and the module ERE $2.
hop_module_count() {
  grep -Ei "(^|[^[:alnum:]_])(hop|h)[ _.:#=-]*$1([^[:alnum:]]|$)" "$LOG_FILE" \
    | grep -Eci "(^|[^[:alnum:]_])$2([^[:alnum:]_]|$)" || true
}

# Count log lines carrying any hop 1..5 and the module ERE $1.
module_on_hop_count() {
  grep -Ei '(^|[^[:alnum:]_])(hop|h)[ _.:#=-]*[1-5]([^[:alnum:]]|$)' "$LOG_FILE" \
    | grep -Eci "(^|[^[:alnum:]_])$1([^[:alnum:]_]|$)" || true
}

# Print "1" when a lost/fwd figure on the line is >= 1 000 000, else "0".
# Accepts 21700000, 21,700,000, 21 700 000, 21_700_000, 21.7M, 21.7 million.
figure_in_millions() {
  local line="$1" norm v mult
  norm=$(printf '%s' "$line" | tr 'A-Z' 'a-z' | tr -d ',_')
  # Collapse whitespace between digit groups: "21 700 000" -> "21700000".
  while [[ "$norm" =~ ([0-9])[[:space:]]+([0-9]) ]]; do
    norm="${norm//${BASH_REMATCH[0]}/${BASH_REMATCH[1]}${BASH_REMATCH[2]}}"
  done
  local nums
  nums=$(printf '%s\n' "$norm" \
    | grep -oE '(lost|fwd)[^0-9]{0,12}[0-9]+(\.[0-9]+)?(k|m|million)?' \
    | grep -oE '[0-9]+(\.[0-9]+)?(k|m|million)?' || true)
  for v in $nums; do
    mult=1
    case "$v" in
      *million) mult=1000000; v=${v%million} ;;
      *m)       mult=1000000; v=${v%m} ;;
      *k)       mult=1000;    v=${v%k} ;;
    esac
    if awk -v x="$v" -v m="$mult" 'BEGIN { exit !(x * m >= 1000000) }'; then
      printf '1\n'
      return 0
    fi
  done
  printf '0\n'
}

# ---------------------------------------------------------------------------
# Locate section (b) and its (c) boundary
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-07 does not exist yet.\n'
  exit 1
fi

TOTAL_LINES=$(wc -l < "$REPORT")

b_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(b\)')
c_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(c\)')

if [ -z "$b_no" ]; then
  bad "section (b) The 21.7 M counter not found (heading containing '(b)')"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
if [ -z "$c_no" ] || [ "$c_no" -le "$b_no" ]; then
  bad "section (c) heading not found after section (b); cannot bound section (b)"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
ok "section (b) found at line $b_no, section (c) at line $c_no"

SECTION=$(line_range "$REPORT" "$((b_no + 1))" "$((c_no - 1))")
nonempty=$(printf '%s\n' "$SECTION" | grep -c '[^[:space:]]' || true)

# Placeholder guard is a diagnosis, not an early exit: the diff/log checks
# below still report their own missing evidence.
if [ "$nonempty" -lt 4 ] && printf '%s\n' "$SECTION" | grep -Eqi 'to be filled'; then
  bad "section (b) is still the placeholder ($nonempty non-empty lines); 002-07 has not been written yet"
fi

# Paragraph ranges inside section (b): non-empty runs, headings and blank
# lines split. Prints "start end" absolute report line numbers.
para_ranges_b() {
  local k=$((b_no + 1)) start=0 line
  while [ "$k" -lt "$c_no" ]; do
    line=$(sed -n "${k}p" "$REPORT")
    if [ -z "$(printf '%s' "$line" | tr -d '[:space:]')" ] \
       || printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
      if [ "$start" -gt 0 ]; then
        printf '%s %s\n' "$start" "$((k - 1))"
        start=0
      fi
    else
      [ "$start" -eq 0 ] && start=$k
    fi
    k=$((k + 1))
  done
  if [ "$start" -gt 0 ]; then
    printf '%s %s\n' "$start" "$((c_no - 1))"
  fi
}
RANGES=$(para_ranges_b)

# Is report line $1 inside a fenced code block within section (b)?
is_inside_fence_b() {
  local n="$1" k=$((b_no + 1)) f=0
  while [ "$k" -lt "$n" ]; do
    is_fence_line "$(sed -n "${k}p" "$REPORT")" && f=$((1 - f))
    k=$((k + 1))
  done
  [ "$f" -eq 1 ]
}

# ---------------------------------------------------------------------------
# AC 1 — committed scratch-copy logging diff
# ---------------------------------------------------------------------------

find_diff() {
  local f
  for f in "$ARCH"/*002-07*.diff "$ARCH"/*002-07*.patch; do
    [ -f "$f" ] || continue
    printf '%s\n' "$f"
    return 0
  done
  # Fallback: any diff/patch whose content identifies ticket 002-07.
  for f in "$ARCH"/*.diff "$ARCH"/*.patch; do
    [ -f "$f" ] || continue
    if grep -Eq '002-07|hop[ _.:#=-]*[1-5]' "$f"; then
      printf '%s\n' "$f"
      return 0
    fi
  done
  return 1
}

DIFF_FILE=$(find_diff || true)
if [ -z "$DIFF_FILE" ]; then
  bad "AC 1: no companion scratch diff beside the report (expect *002-07*.diff or *.patch)"
else
  diff_rel=${DIFF_FILE#"$REPO_ROOT"/}
  if [ ! -s "$DIFF_FILE" ]; then
    bad "AC 1: companion diff is empty: $diff_rel"
  elif ! is_tracked "$diff_rel"; then
    bad "AC 1: companion diff exists but is not committed: $diff_rel"
  else
    ok "AC 1: companion diff present and committed: $diff_rel"
  fi

  if grep -Eq '^(diff --git |--- |\+\+\+ |Index: )' "$DIFF_FILE"; then
    ok "AC 1: diff carries a unified-diff header"
  else
    bad "AC 1: $diff_rel is not a unified diff (no ---/+++/diff --git/Index: header)"
  fi

  if grep -Eq '^\+[^+].*(eprintln!|println!|print!|dbg!|log::|tracing|log_line|hop_log|stderr|stdout)' "$DIFF_FILE"; then
    ok "AC 1: diff adds at least one logging line"
  else
    bad "AC 1: diff has no added line with a logging cue (eprintln!, println!, dbg!, log::, tracing, stderr/stdout, ...)"
  fi

  if grep -qF "$FROZEN_ROOT" "$DIFF_FILE"; then
    bad "AC 1: diff names the frozen tree ($FROZEN_ROOT); it must reference only scratch-copy paths"
  else
    ok "AC 1: diff does not name the frozen tree"
  fi

  path_bad=0
  while IFS= read -r line; do
    case "$line" in
      diff\ *|---\ *|+++\ *|Index:\ *|Only\ in\ *) ;;
      *) continue ;;
    esac
    for p in $(printf '%s\n' "$line" | grep -oE '(^|[[:space:]])/[^[:space:]]+' \
                 | sed -E 's/^[[:space:]]*//' || true); do
      case "$p" in
        */poc-scratch|*/poc-scratch/*|/dev/null) ;;
        *)
          bad "AC 1: diff header references a non-scratch absolute path: $p"
          path_bad=$((path_bad + 1))
          ;;
      esac
    done
  done < "$DIFF_FILE"
  if [ "$path_bad" -eq 0 ]; then
    ok "AC 1: diff header/path lines reference only scratch-copy paths"
  fi

  if grep -Eq '^(diff --git .*(target/|experiments/)|Only in .*(target|experiments))' "$DIFF_FILE"; then
    bad "AC 1: diff touches target/ or experiments/, not logging only"
  else
    ok "AC 1: diff does not touch target/ or experiments/"
  fi
fi

# Statement: the patch is logging-only.
logging_stmt=0
logging_tagged=0
logging_where=""
while read -r s e; do
  [ -n "$s" ] || continue
  block=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$block" \
       | grep -Eqi 'logging[- ]only|only (adds? )?logging|no behaviou?r(al)? change|behaviou?r[- ]preserving|does not change (the )?behaviou?r'; then
    logging_stmt=1
    logging_where="lines $s-$e"
    has_tag "$block" && logging_tagged=1
  fi
done <<EOF
$RANGES
EOF
if [ "$logging_stmt" -eq 0 ]; then
  bad "AC 1: section (b) does not state that the diff is logging-only / changes no behaviour"
elif [ "$logging_tagged" -eq 0 ]; then
  bad "AC 1: the logging-only statement carries no evidence tag ($logging_where)"
else
  ok "AC 1: section (b) states the diff is logging-only, with an evidence tag ($logging_where)"
fi

# Statement: the frozen-tree fingerprint is unchanged.
fp_stmt=0
fp_tagged=0
fp_where=""
while read -r s e; do
  [ -n "$s" ] || continue
  block=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$block" | grep -Eqi 'frozen' \
     && printf '%s\n' "$block" | grep -Eqi 'fingerprint' \
     && printf '%s\n' "$block" \
          | grep -Eqi 'unchanged|not (been )?modified|untouched|still (identical|matches)|remains (identical|unchanged)|same (value|fingerprint)|matches (the )?(recorded|002-01)'; then
    fp_stmt=1
    fp_where="lines $s-$e"
    has_tag "$block" && fp_tagged=1
  fi
done <<EOF
$RANGES
EOF
if [ "$fp_stmt" -eq 0 ]; then
  bad "AC 1: section (b) does not state that the frozen-tree fingerprint is unchanged"
elif [ "$fp_tagged" -eq 0 ]; then
  bad "AC 1: the frozen-tree-fingerprint statement carries no evidence tag ($fp_where)"
else
  ok "AC 1: section (b) states the frozen-tree fingerprint is unchanged, with an evidence tag ($fp_where)"
fi

# ---------------------------------------------------------------------------
# AC 2 — committed log with all five hops from synth1, synth2 and mixer
# ---------------------------------------------------------------------------

find_log() {
  local f
  for f in "$ARCH"/*002-07*.log; do
    [ -f "$f" ] || continue
    printf '%s\n' "$f"
    return 0
  done
  for f in "$ARCH"/*002-07*.txt; do
    [ -f "$f" ] || continue
    printf '%s\n' "$f"
    return 0
  done
  # Fallback: any .log in the directory that carries the hop grammar.
  for f in "$ARCH"/*.log; do
    [ -f "$f" ] || continue
    if grep -Eqi '(^|[^[:alnum:]_])(hop|h)[ _.:#=-]*[1-5]([^[:alnum:]]|$)' "$f"; then
      printf '%s\n' "$f"
      return 0
    fi
  done
  return 1
}

LOG_FILE=$(find_log || true)
if [ -z "$LOG_FILE" ]; then
  bad "AC 2: no companion log beside the report (expect *002-07*.log)"
else
  log_rel=${LOG_FILE#"$REPO_ROOT"/}
  if [ ! -s "$LOG_FILE" ]; then
    bad "AC 2: companion log is empty: $log_rel"
  elif ! is_tracked "$log_rel"; then
    bad "AC 2: companion log exists but is not committed: $log_rel"
  else
    ok "AC 2: companion log present and committed: $log_rel"
  fi

  missing_hops=""
  for h in 1 2 3 4 5; do
    n=$(hop_count "$h")
    if [ "$n" -gt 0 ]; then
      ok "AC 2: log has $n line(s) for hop $h"
    else
      missing_hops="$missing_hops $h"
    fi
  done
  if [ -n "$missing_hops" ]; then
    bad "AC 2: log has no hop-tagged line for hop(s):$missing_hops"
  fi

  # Echo-side hops 1 and 2 happen in both synths and the mixer.
  for h in 1 2; do
    for spec in 'synth[ _-]?1:synth1' 'synth[ _-]?2:synth2' 'mixer:mixer'; do
      mre=${spec%%:*}
      mname=${spec#*:}
      n=$(hop_module_count "$h" "$mre")
      if [ "$n" -gt 0 ]; then
        ok "AC 2: hop $h has $n line(s) from $mname"
      else
        bad "AC 2: hop $h has no line from $mname"
      fi
    done
  done

  # The three module names must appear on hop-tagged lines (the readable
  # "both synths and the mixer" requirement).
  for spec in 'synth[ _-]?1:synth1' 'synth[ _-]?2:synth2' 'mixer:mixer'; do
    mre=${spec%%:*}
    mname=${spec#*:}
    n=$(module_on_hop_count "$mre")
    if [ "$n" -gt 0 ]; then
      ok "AC 2: $mname appears on $n hop-tagged line(s)"
    else
      bad "AC 2: $mname never appears on a hop-tagged line"
    fi
  done
fi

# ---------------------------------------------------------------------------
# Work step 2 — the 25 s instrumented run came from the scratch copy
# ---------------------------------------------------------------------------

run_src=0
run_src_tagged=0
run_src_where=""
while read -r s e; do
  [ -n "$s" ] || continue
  block=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$block" | grep -Eqi 'scratch' \
     && printf '%s\n' "$block" | grep -Eqi 'run|loopback|instrumented' \
     && printf '%s\n' "$block" | grep -Eqi 'frozen' \
     && printf '%s\n' "$block" | grep -Eqi 'not|never|rather|instead|outside|untouched|unmodified|left alone|neither'; then
    run_src=1
    run_src_where="lines $s-$e"
    has_tag "$block" && run_src_tagged=1
  fi
done <<EOF
$RANGES
EOF
if [ "$run_src" -eq 0 ]; then
  bad "Work step 2: section (b) does not state that the run came from the scratch copy, not the frozen tree"
elif [ "$run_src_tagged" -eq 0 ]; then
  bad "Work step 2: the scratch-copy run statement carries no evidence tag ($run_src_where)"
else
  ok "Work step 2: section (b) states the run came from the scratch copy, not the frozen tree ($run_src_where)"
fi

run_duration_re='(--duration[[:space:]]*25([^0-9]|$))|((^|[^0-9])25[[:space:]]*(s|sec|secs|seconds?)([^[:alnum:]]|$))|((^|[^0-9])25-second)'
if printf '%s\n' "$SECTION" | grep -Eqi "$run_duration_re"; then
  ok "Work step 2: section (b) states a 25 s run"
else
  bad "Work step 2: section (b) does not state the 25 s duration ('25 s', '25-second' or '--duration 25')"
fi

# ---------------------------------------------------------------------------
# AC 3 — captured display line, lost in the millions or explicit non-repro
# ---------------------------------------------------------------------------

# Fenced block ranges in section (b): "start end" for each block (fences
# included).
blocks=""
in_f=0
f_start=0
k=$((b_no + 1))
while [ "$k" -lt "$c_no" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  if is_fence_line "$l"; then
    if [ "$in_f" -eq 0 ]; then
      in_f=1
      f_start=$k
    else
      in_f=0
      blocks="$blocks$f_start $k"$'\n'
    fi
  fi
  k=$((k + 1))
done

# Block range containing report line $1, or nothing.
block_for() {
  local n="$1" fs fe
  while read -r fs fe; do
    [ -n "$fs" ] || continue
    if [ "$n" -gt "$fs" ] && [ "$n" -lt "$fe" ]; then
      printf '%s %s\n' "$fs" "$fe"
      return 0
    fi
  done <<EOF
$blocks
EOF
  return 1
}

display_seen=0
display_tagged=0
display_millions=0
display_text=""
display_where=""
k=$((b_no + 1))
while [ "$k" -lt "$c_no" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  is_candidate=0
  if printf '%s\n' "$l" | grep -Eqi '(sent|received)[[:space:]]*[:=]?[[:space:]]*[0-9]' \
     && printf '%s\n' "$l" | grep -Eqi 'lost[[:space:]]*[:=]?[[:space:]]*[0-9]'; then
    is_candidate=1
  fi
  [ "$is_candidate" -eq 1 ] || { k=$((k + 1)); continue; }

  if bspan=$(block_for "$k"); then
    set -- $bspan
    fs=$1; fe=$2
    display_seen=1
    # The tag may sit on the line, the fence, the caption before the opening
    # fence, or as a standalone tag line after the closing fence.
    if block_tagged "$fs" "$fe"; then
      display_tagged=1
      if [ -z "$display_text" ]; then
        display_text="$l"
        display_where="lines $fs-$fe"
      fi
      if [ "$(figure_in_millions "$l")" = "1" ]; then
        display_millions=1
      fi
    fi
  else
    # Inline captured output in backticks on a prose line.
    if printf '%s\n' "$l" | grep -Eq '`[^`]*lost[^`]*`'; then
      display_seen=1
      if line_range "$REPORT" "$((k - 1))" "$((k + 1))" | grep -qF '[measured]'; then
        display_tagged=1
        if [ -z "$display_text" ]; then
          display_text="$l"
          display_where="line $k"
        fi
        if [ "$(figure_in_millions "$l")" = "1" ]; then
          display_millions=1
        fi
      fi
    fi
  fi
  k=$((k + 1))
done

neg_re='not reproduc|did not reproduc|does not reproduc|no longer reproduc|failed to reproduc|could not reproduc|not observed|does not occur|not seen|no anomaly|anomaly (is )?absent|did not recur|did not appear'
neg_seen=0
if printf '%s\n' "$SECTION" | grep -Eqi "$neg_re"; then
  neg_seen=1
fi

if [ "$display_seen" -eq 0 ]; then
  bad "AC 3: section (b) has no captured display line (a fenced or backticked line with 'sent'/'received' and 'lost')"
elif [ "$display_tagged" -eq 0 ]; then
  bad "AC 3: the captured display line is not tagged [measured] (tag the line, its fence, or within three lines)"
elif [ "$display_millions" -eq 1 ]; then
  ok "AC 3: the captured display line shows a lost figure in the millions ($display_where): $(printf '%s' "$display_text" | cut -c1-140)"
elif [ "$neg_seen" -eq 1 ]; then
  ok "AC 3: the anomaly is explicitly stated not to have reproduced, with a captured display line ($display_where)"
else
  bad "AC 3: the captured display's lost figure is below 1 000 000 and section (b) does not state that the anomaly did not reproduce"
fi

# ---------------------------------------------------------------------------
# Work step 4 — at least 5 consecutive records per hop in the excerpt
# ---------------------------------------------------------------------------

declare -A best_run=()
for h in 1 2 3 4 5; do best_run[$h]=0; done
excerpt_blocks=0
excerpt_tag_bad=0

while read -r fs fe; do
  [ -n "$fs" ] || continue
  declare -A run=()
  for h in 1 2 3 4 5; do run[$h]=0; done
  block_best=0
  k=$((fs + 1))
  while [ "$k" -lt "$fe" ]; do
    l=$(sed -n "${k}p" "$REPORT")
    hops=$(line_hops "$l")
    nh=$(printf '%s\n' "$hops" | grep -c . || true)
    if [ "$nh" -eq 1 ]; then
      h=$hops
      run[$h]=$(( ${run[$h]:-0} + 1 ))
      for x in 1 2 3 4 5; do
        [ "$x" = "$h" ] || run[$x]=0
      done
      if [ "${run[$h]}" -gt "${best_run[$h]}" ]; then
        best_run[$h]=${run[$h]}
      fi
      [ "${run[$h]}" -ge 5 ] && block_best=1
    else
      for x in 1 2 3 4 5; do run[$x]=0; done
    fi
    k=$((k + 1))
  done
  if [ "$block_best" -eq 1 ]; then
    excerpt_blocks=$((excerpt_blocks + 1))
    if ! block_tagged "$fs" "$fe"; then
      bad "Work step 4: excerpt block at lines $fs-$fe has 5+ consecutive records but is not tagged [measured]"
      excerpt_tag_bad=$((excerpt_tag_bad + 1))
    fi
  fi
done <<EOF
$blocks
EOF

if [ "$excerpt_blocks" -eq 0 ]; then
  bad "Work step 4: no fenced excerpt block supplies a run of 5 consecutive hop records in section (b)"
fi

excerpt_missing=""
for h in 1 2 3 4 5; do
  if [ "${best_run[$h]}" -ge 5 ]; then
    ok "Work step 4: excerpt has ${best_run[$h]} consecutive record(s) for hop $h"
  else
    excerpt_missing="$excerpt_missing hop$h(${best_run[$h]})"
  fi
done
if [ -n "$excerpt_missing" ]; then
  bad "Work step 4: excerpt lacks 5 consecutive records for:$excerpt_missing"
fi
if [ "$excerpt_tag_bad" -gt 0 ]; then
  bad "Work step 4: $excerpt_tag_bad excerpt block(s) missing the [measured] tag"
fi

# ---------------------------------------------------------------------------
# Work step 1 — the 10 lines/s/hop rate limit is stated
# ---------------------------------------------------------------------------

rate_re='rate[- ]?limit[^.]{0,80}(10|ten)|(10|ten)[^.]{0,30}(lines?|records?)[^.]{0,30}(per second|/s|s([^[:alnum:]]|$)|second)|at most (10|ten)[^.]{0,30}(lines?|records?)|(10|ten) (lines?|records?) (per|/) ?(second|s|sec)'
if printf '%s\n' "$SECTION" | grep -Eqi "$rate_re"; then
  ok "Work step 1: section (b) states the <=10 lines/s/hop logging rate limit"
else
  bad "Work step 1: section (b) does not state the <=10 lines/s/hop rate limit (e.g. 'rate-limited to 10 lines/s/hop')"
fi

# ---------------------------------------------------------------------------
# DoD — evidence tags, pending verdict, scratch/frozen discipline
# ---------------------------------------------------------------------------

tag_problems=0
k=$((b_no + 1))
while [ "$k" -lt "$c_no" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  if printf '%s\n' "$l" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+' \
     && ! is_inside_fence_b "$k"; then
    window=$(line_range "$REPORT" "$((k - 1))" "$((k + 1))")
    if ! printf '%s\n' "$window" | grep -qF '[measured]' \
       && ! has_tag "$window"; then
      bad "DoD: line $k carries a file:line pointer but no evidence tag on itself or an adjacent line"
      tag_problems=$((tag_problems + 1))
    fi
  fi
  k=$((k + 1))
done
[ "$tag_problems" -eq 0 ] && ok "DoD: every pointer-bearing line carries an evidence tag"

if printf '%s\n' "$SECTION" | grep -qF '[measured]'; then
  ok "DoD: section (b) carries [measured] evidence"
else
  bad "DoD: section (b) carries no [measured] evidence"
fi

verdict_count=$(printf '%s\n' "$SECTION" \
  | grep -Eci '(^|[^[:alnum:]])[*_]*verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "DoD: section (b) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "DoD: section (b) has $verdict_count 'verdict:' lines; exactly one is required"
elif printf '%s\n' "$SECTION" | grep -Eqi '(^|[^[:alnum:]])[*_]*verdict:[^A-Za-z]*pending'; then
  ok "DoD: section (b) still carries its one pending verdict marker (002-08 owns the verdict)"
else
  bad "DoD: section (b)'s verdict marker no longer says 'pending' (the verdict belongs to 002-08)"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
