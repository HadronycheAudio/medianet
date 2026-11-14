#!/usr/bin/env bash
# checks/002-02.sh — checks for ticket 002-02
# "Reading notes: audio.rs analyse and analyse_voices".
#
# The deliverable is the sub-section "audio.rs — analyse / analyse_voices"
# inside section (d) Reading notes of
#   doc/design/reports/archaeology/REPORT-0b.md
# written by the author after a read-only reading of the frozen POC at
# /home/dorian/workspace/hadropoc. This script reads the report only; it never
# writes anywhere and never touches the POC tree.
#
# Required conventions, spelled out here so the author can comply exactly.
# They are deliberate: they make the notes machine-checkable without fixing
# the prose.
#
#  * (AC 1) The sub-section is the first heading between the "## (d) ..."
#    heading and the "## (e) ..." heading whose text contains both "audio.rs"
#    and "analyse" (or "analysis"), case-insensitive. Its text runs from that
#    heading to the next heading of the same or higher level (or to the (e)
#    heading). It must have at most 45 non-empty lines, and at least 4
#    DISTINCT "audio.rs:<digits>" pointers.
#  * (AC 2) One line, after stripping surrounding whitespace and markdown
#    emphasis (* and _), must read exactly "Voice comparison bit-level: yes"
#    or "Voice comparison bit-level: no" (the yes/no word case-insensitive).
#    If the answer is "no", the sub-section must name the tolerance: it must
#    contain one of "tolerance", "threshold", "epsilon" / "eps".
#  * (AC 3) For each of the five loss-handling facts, at least one line in the
#    sub-section must state it (the keywords below) and that line, or the line
#    immediately following it, must carry BOTH a confirmation marker
#    ("confirmed" or "corrected") and an evidence tag "[code]".
#  * (AC 4) Exclusion rules are recognised as list entries: lines starting
#    with "-", "*", "+", a digit followed by "." or ")", or a table row
#    starting with "|". At least the five named exclusion categories must be
#    covered by such entries (incomplete, lag/MAX_LAG_WINDOWS, sender gap,
#    settling/SETTLE_SECS, voices flag off). An entry that carries a
#    confirmation marker ("confirmed"/"corrected") belongs to the AC 3 fact
#    list and is judged there, not here. EVERY other entry naming one of those
#    categories must contain (a) an "audio.rs:<digits>" pointer and (b) an
#    explicit reason marker: one of "because", "since", "reason", "so that",
#    "to avoid", "otherwise". List entries that name no exclusion are not
#    constrained by this check.
#  * (AC 5) The verdict statement is a line whose first non-space characters
#    (after optional "> ", "- ", "**" / "*" emphasis) are "Verdict statement:"
#    and which then has text. The statement must be one sentence: after
#    removing "audio.rs:<digits>" pointers, decimal numbers and the
#    abbreviations "e.g."/"i.e.", at most one terminal ".", "!" or "?".
#  * (DoD) In the "## Open sub-questions" table, the row containing
#    "voice comparison bit-level" (case-insensitive) must still be a table row
#    (starts with "|") and must contain "answered" plus a standalone "yes" or
#    "no".
#
# Exit status: 0 when every check passes, 1 otherwise.

set -u

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REPORT_REL="doc/design/reports/archaeology/REPORT-0b.md"
REPORT="$REPO_ROOT/$REPORT_REL"

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

# Strip leading list bullets / quote markers / whitespace and surrounding
# markdown emphasis, then squeeze interior whitespace runs for comparison.
normalise() {
  local s="$1"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s#- }"; s="${s#\* }"; s="${s#+ }"; s="${s#> }"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s#\*\*}"; s="${s%\*\*}"
  s="${s#"${s%%[![:space:]]*}"}"
  printf '%s' "$s"
}

# Does the text (any line of $1) match all EREs given after it?
# (unused generic helper kept out; each check is written inline for clarity)

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-02 does not exist yet.\n'
  exit 1
fi

# ---------------------------------------------------------------------------
# Locate section (d) Reading notes and the sub-section inside it
# ---------------------------------------------------------------------------

d_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(d\)')
e_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(e\)')

if [ -z "$d_no" ]; then
  bad "section (d) Reading notes not found (heading containing '(d)')"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
if [ -z "$e_no" ] || [ "$e_no" -le "$d_no" ]; then
  bad "section (e) heading not found after section (d); cannot bound the reading notes"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
ok "section (d) found at line $d_no, section (e) at line $e_no"

sub_no=""
sub_lvl=0
i=$((d_no + 1))
while [ "$i" -lt "$e_no" ]; do
  line=$(sed -n "${i}p" "$REPORT")
  if printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
    hashes=${line%%[! #]*}
    lvl=${#hashes}
    low=$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')
    case "$low" in
      *audio.rs*)
        case "$low" in
          *analyse*|*analysis*)
            sub_no=$i; sub_lvl=$lvl; break
            ;;
        esac
        ;;
    esac
  fi
  i=$((i + 1))
done

if [ -z "$sub_no" ]; then
  bad "no sub-section heading under (d) containing both 'audio.rs' and 'analyse'/'analysis'"
  printf '\n%d check(s) failed; the audio.rs reading notes do not exist yet.\n' "$fail"
  exit 1
fi
ok "sub-section found at line $sub_no: $(sed -n "${sub_no}p" "$REPORT")"

# End of the sub-section: next heading of the same or higher level, or (e).
sub_end=$((e_no - 1))
j=$((sub_no + 1))
while [ "$j" -lt "$e_no" ]; do
  line=$(sed -n "${j}p" "$REPORT")
  if printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
    hashes=${line%%[! #]*}
    if [ "${#hashes}" -le "$sub_lvl" ]; then
      sub_end=$((j - 1))
      break
    fi
  fi
  j=$((j + 1))
done

SUB=$(line_range "$REPORT" "$sub_no" "$sub_end")
if [ -z "$SUB" ]; then
  bad "sub-section is empty"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi

# ---------------------------------------------------------------------------
# AC 1 — size and pointers
# ---------------------------------------------------------------------------

nonempty=$(printf '%s\n' "$SUB" | grep -c '[^[:space:]]')
if [ "$nonempty" -le 45 ]; then
  ok "sub-section is compact ($nonempty non-empty lines, limit 45)"
else
  bad "sub-section is too long ($nonempty non-empty lines, limit 45 — about one page)"
fi

ptrs=$(printf '%s\n' "$SUB" | grep -oE 'audio\.rs:[0-9]+' | sort -u)
nptr=$(printf '%s\n' "$ptrs" | grep -c . || true)
if [ "$nptr" -ge 4 ]; then
  ok "sub-section carries $nptr distinct audio.rs:NNN pointers"
else
  bad "sub-section has $nptr distinct audio.rs:NNN pointer(s), need at least 4"
fi

# ---------------------------------------------------------------------------
# AC 2 — the bit-level answer, exactly yes or no
# ---------------------------------------------------------------------------

answer=""
while IFS= read -r raw; do
  n=$(normalise "$raw")
  nl=$(printf '%s' "$n" | tr '[:upper:]' '[:lower:]')
  nl="${nl%"${nl##*[![:space:]]}"}"
  case "$nl" in
    'voice comparison bit-level: yes') answer=yes; break ;;
    'voice comparison bit-level: no')  answer=no;  break ;;
  esac
done <<EOF
$SUB
EOF

if [ -n "$answer" ]; then
  ok "bit-level question answered exactly: 'Voice comparison bit-level: $answer'"
  if [ "$answer" = "no" ]; then
    if printf '%s\n' "$SUB" | grep -Eqi 'toleran|threshold|epsilon|eps\b'; then
      ok "answer is 'no' and the sub-section names the tolerance used"
    else
      bad "answer is 'no' but the sub-section names no tolerance (need one of: tolerance, threshold, epsilon/eps)"
    fi
  fi
else
  bad "missing an exact line 'Voice comparison bit-level: yes' or '...: no' (markdown emphasis and list bullets are tolerated)"
fi

# ---------------------------------------------------------------------------
# AC 3 — the five loss-handling facts, marked confirmed/corrected [code]
# ---------------------------------------------------------------------------

# For each fact: human label, then the EREs that must ALL be found on the
# same statement line (or split over that line and the immediately next one).
fact_labels=(
  "1024-sample comparison window"
  "window compared only when complete on both sides"
  "256-window lag drop (MAX_LAG_WINDOWS, about 2.7 s)"
  "1-second silence gives a 3-second settling exclusion"
  "sender's replica never applies gaps"
)

# fact_ok <from-line> <to-line> : are all needle EREs present in the block?
fact_block_ok() {
  local from="$1" to="$2" block
  block=$(line_range "$REPORT" "$from" "$to")
  shift 2
  local re
  for re in "$@"; do
    printf '%s\n' "$block" | grep -Eqi "$re" || return 1
  done
  return 0
}

# fact_marked <from-line> <to-line>
fact_marked() {
  local block
  block=$(line_range "$REPORT" "$1" "$2")
  printf '%s\n' "$block" | grep -Eqi '(^|[^[:alnum:]])(confirmed|corrected)([^[:alnum:]]|$)' || return 1
  printf '%s\n' "$block" | grep -Fq '[code]' || return 1
}

# Find the first statement line for a fact: a line matching its first ERE
# (the anchor) for which anchor + continuation satisfy all needles.
# fact_find <anchor-re> <needles...> ; prints "from to" or nothing.
fact_find() {
  local anchor="$1"; shift
  local k ln
  for ((k = sub_no + 1; k <= sub_end; k++)); do
    ln=$(sed -n "${k}p" "$REPORT")
    printf '%s\n' "$ln" | grep -Eqi "$anchor" || continue
    if fact_block_ok "$k" "$((k + 1))" "$@"; then
      printf '%s %s\n' "$k" "$((k + 1))"
      return 0
    fi
  done
  return 1
}

# fact 1: a 1024-sample window
if span=$(fact_find '1024|1[ ,]024' '1024|1[ ,]024' 'window'); then
  set -- $span
  if fact_marked "$1" "$2"; then
    ok "fact marked confirmed/corrected [code]: ${fact_labels[0]}"
  else
    bad "fact '${fact_labels[0]}' is stated (line $1) but not marked confirmed/corrected with [code]"
  fi
else
  bad "fact '${fact_labels[0]}' not stated (need '1024' and 'window')"
fi

# fact 2: complete on both sides
if span=$(fact_find 'complete' 'complete' 'both'); then
  set -- $span
  if fact_marked "$1" "$2"; then
    ok "fact marked confirmed/corrected [code]: ${fact_labels[1]}"
  else
    bad "fact '${fact_labels[1]}' is stated (line $1) but not marked confirmed/corrected with [code]"
  fi
else
  bad "fact '${fact_labels[1]}' not stated (need 'complete' and 'both')"
fi

# fact 3: 256-window lag and the ~2.7 s figure
if span=$(fact_find '256|MAX_LAG_WINDOWS' '256|MAX_LAG_WINDOWS' '2[.,]7'); then
  set -- $span
  if fact_marked "$1" "$2"; then
    ok "fact marked confirmed/corrected [code]: ${fact_labels[2]}"
  else
    bad "fact '${fact_labels[2]}' is stated (line $1) but not marked confirmed/corrected with [code]"
  fi
else
  bad "fact '${fact_labels[2]}' not stated (need '256' and the '2.7' s figure on one statement)"
fi

# fact 4: 1 s silence -> 3 s settling
if span=$(fact_find 'settle|settling' 'settle|settling' '(1[ -]?s|1 second|1-second)' '(3[ -]?s|3 second|3-second|SETTLE_SECS)'); then
  set -- $span
  if fact_marked "$1" "$2"; then
    ok "fact marked confirmed/corrected [code]: ${fact_labels[3]}"
  else
    bad "fact '${fact_labels[3]}' is stated (line $1) but not marked confirmed/corrected with [code]"
  fi
else
  bad "fact '${fact_labels[3]}' not stated (need settling, a 1-second silence and a 3-second exclusion)"
fi

# fact 5: sender's replica never applies gaps
if span=$(fact_find 'replica' 'replica' 'gap'); then
  set -- $span
  if fact_marked "$1" "$2"; then
    ok "fact marked confirmed/corrected [code]: ${fact_labels[4]}"
  else
    bad "fact '${fact_labels[4]}' is stated (line $1) but not marked confirmed/corrected with [code]"
  fi
else
  bad "fact '${fact_labels[4]}' not stated (need 'replica' and 'gap')"
fi

# ---------------------------------------------------------------------------
# AC 4 — every exclusion entry carries a pointer and a stated reason
# ---------------------------------------------------------------------------

# Exclusion categories: label + ERE. All five must be covered.
exc_labels=(
  "window incomplete (not complete on both sides)"
  "lag over MAX_LAG_WINDOWS"
  "sender gap"
  "SETTLE_SECS settling"
  "voices flag off"
)
exc_res=(
  'incomplete'
  'lag|MAX_LAG_WINDOWS'
  'sender[ -]?gap'
  'settle|SETTLE_SECS'
  'voices[ -]?flag|voices.*off|flag off|voices=false|voices = false'
)

reason_re='because|since|reason|so that|to avoid|otherwise'

entry_ptr=0
entry_bad=0
entry_any=0
while IFS= read -r ln; do
  [ -n "$(printf '%s' "$ln" | tr -d '[:space:]')" ] || continue
  # Is it a list entry?
  if ! printf '%s\n' "$ln" | grep -Eq '^[[:space:]]*(-|\*|\+|>|[0-9]+[.)])[[:space:]]|^[[:space:]]*\|'; then
    continue
  fi
  # Does it name an exclusion?
  names_exc=0
  for re in "${exc_res[@]}"; do
    if printf '%s\n' "$ln" | grep -Eqi -- "$re"; then names_exc=1; break; fi
  done
  [ "$names_exc" -eq 1 ] || continue
  # Fact-list entries are governed by AC 3, not by this per-entry rule.
  printf '%s\n' "$ln" | grep -Eqi '(^|[^[:alnum:]])(confirmed|corrected)([^[:alnum:]]|$)' && continue
  entry_any=$((entry_any + 1))
  missing=""
  printf '%s\n' "$ln" | grep -Eq 'audio\.rs:[0-9]+' || missing="$missing pointer"
  printf '%s\n' "$ln" | grep -Eqi -- "$reason_re" || missing="$missing reason"
  if [ -z "$missing" ]; then
    entry_ptr=$((entry_ptr + 1))
  else
    entry_bad=$((entry_bad + 1))
    bad "exclusion entry lacks$missing: $(printf '%s' "$ln" | cut -c1-110)"
  fi
done <<EOF
$SUB
EOF

if [ "$entry_any" -eq 0 ]; then
  bad "no exclusion list entry found (expected bullet or table rows naming incomplete/LAG/sender gap/settling/voices flag)"
else
  if [ "$entry_bad" -eq 0 ]; then
    ok "all $entry_any exclusion entries carry an audio.rs:NNN pointer and a stated reason"
  else
    bad "$entry_bad of $entry_any exclusion entries lack a pointer or a stated reason"
  fi

  # Coverage of the five named categories, by an entry that is complete.
  missing_cat=""
  for idx in "${!exc_labels[@]}"; do
    covered=0
    while IFS= read -r ln; do
      printf '%s\n' "$ln" | grep -Eq '^[[:space:]]*(-|\*|\+|>|[0-9]+[.)])[[:space:]]|^[[:space:]]*\|' || continue
      printf '%s\n' "$ln" | grep -Eqi -- "${exc_res[$idx]}" || continue
      printf '%s\n' "$ln" | grep -Eq 'audio\.rs:[0-9]+' || continue
      printf '%s\n' "$ln" | grep -Eqi -- "$reason_re" || continue
      covered=1; break
    done <<EOF
$SUB
EOF
    [ "$covered" -eq 1 ] || missing_cat="$missing_cat; ${exc_labels[$idx]}"
  done
  if [ -z "$missing_cat" ]; then
    ok "all five named exclusion categories are covered by a complete entry"
  else
    bad "exclusion categories missing a complete entry (pointer + reason):$missing_cat"
  fi
fi

# ---------------------------------------------------------------------------
# AC 5 — the one-sentence verdict statement
# ---------------------------------------------------------------------------

verdict=""
while IFS= read -r raw; do
  n=$(normalise "$raw")
  n="${n#"${n%%[![:space:]]*}"}"
  case "$n" in
    'Verdict statement:'*)
      verdict="${n#Verdict statement:}"
      break
      ;;
  esac
done <<EOF
$SUB
EOF

if [ -z "$verdict" ] || [ -z "$(printf '%s' "$verdict" | tr -d '[:space:]')" ]; then
  bad "missing the required verdict line: a line starting with 'Verdict statement:' followed by the one-sentence statement"
else
  ok "verdict statement line present"
  # One sentence: drop pointers, decimals, and e.g./i.e., then count terminals.
  cleaned=$(printf '%s' "$verdict" \
    | sed -E 's/audio\.rs:[0-9]+//g; s/[0-9]+\.[0-9]+//g; s/\be\.g\.//g; s/\bi\.e\.//g')
  # Drop a trailing [code]/[measured] tag before counting.
  cleaned=$(printf '%s' "$cleaned" | sed -E 's/\[(code|measured|kernel-doc|memory|decision)\]//g')
  terms=$(printf '%s' "$cleaned" | grep -oE '[.!?]' | wc -l | tr -d ' ')
  words=$(printf '%s' "$cleaned" | tr -s '[:space:]' '\n' | grep -c '[[:alnum:]]')
  if [ "$terms" -le 1 ] && [ "$words" -ge 6 ]; then
    ok "verdict statement is one sentence ($words words, $terms terminal mark(s))"
  else
    bad "verdict statement is not one sentence ($words words, $terms terminal mark(s); need <=1 and >=6 words)"
  fi
fi

# ---------------------------------------------------------------------------
# DoD — the open sub-question is marked answered in the table
# ---------------------------------------------------------------------------

q_no=$(heading_no "$REPORT" '^#+[[:space:]]*Open sub-questions')
if [ -z "$q_no" ]; then
  bad "no '## Open sub-questions' section found"
else
  row=""
  while IFS= read -r ln; do
    if printf '%s\n' "$ln" | grep -qi 'voice comparison bit-level'; then
      row="$ln"; break
    fi
  done < <(sed -n "$((q_no + 1)),\$p" "$REPORT")

  if [ -z "$row" ]; then
    bad "sub-questions table has no row about the voice comparison being bit-level"
  else
    ok "sub-questions table still has the voice-comparison row"
    if ! printf '%s\n' "$row" | grep -Eq '^[[:space:]]*\|'; then
      bad "the voice-comparison row is no longer a table row (must still start with '|')"
    fi
    if printf '%s\n' "$row" | grep -qi 'answered'; then
      if printf '%s\n' "$row" | grep -Eqi '(^|[^[:alnum:]])(yes|no)([^[:alnum:]]|$)'; then
        ok "the voice-comparison row is marked answered with yes/no"
      else
        bad "the voice-comparison row says 'answered' but does not state yes or no"
      fi
    else
      bad "the voice-comparison row is not marked answered (must contain 'answered' plus yes/no)"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
if [ "$fail" -gt 0 ]; then
  exit 1
fi
