#!/usr/bin/env bash
# checks/002-05.sh — checks for ticket 002-05
# "Inert families: baseline run and transport trace".
#
# The deliverable is section (a) Inert check families of
#   doc/design/reports/archaeology/REPORT-0b.md
# filled by the author from a frozen-POC run and a read-only trace of four
# points, using the 002-03 reading notes. This script reads the report only;
# it never writes anywhere and never touches the POC tree.
#
# The conventions below are spelled out so the author can comply exactly.
# They make the prose machine-checkable without fixing its wording. Section
# (a) runs from the "## (a) ..." heading to the "## (b) ..." heading.
#
#  * (AC 1, captured run) Section (a) must contain a fenced code block
#    (``` ... ```) with the captured final summary of a 25 s run. The block
#    must hold a MUSIC check-family line and a TRANSPORT check-family line,
#    each carrying a count of the form "N of M", "N/M", or "N of M ..." in
#    the same line, plus a family cue (window, measurement, steady, moving,
#    datagram, click, playhead, change, pulse, NO DATA, PASS, FAIL).
#    "0 of 0" is accepted as readily as any non-zero count. The block must be
#    tagged [measured] on the fence line or within three lines before the
#    opening fence / after the closing fence.
#    The section must also state the run duration as 25 s (e.g. "25 s",
#    "25-second", "--duration 25") and state the reproduction status
#    explicitly: either "reproduced"/"confirmed"/"still present"/"same
#    anomaly", or the negation "did not reproduce"/"not reproduced"/"no
#    longer reproduces"/"not observed"/"does not occur". Both outcomes are
#    acceptable; silence is not.
#  * (AC 1, run source) One paragraph (or two adjacent paragraphs) must state
#    all three of: the tag/state "poc-untouched", the source actually used
#    (scratch / poc-scratch / checkout / tag / frozen), and equivalence
#    ("byte-identical", "content-identical", "identical", "equivalent").
#    The POC's working tree now carries epic-003 changes, so a run from
#    /home/dorian/workspace/hadropoc that is not tied to poc-untouched does
#    not satisfy this.
#  * (AC 2, four trace points) Each point is located by its own marker,
#    case-insensitive, in a paragraph outside the captured code block:
#      1 who is the transport reference — "transport reference" (or "who
#        acts as ... reference");
#      2 whether ctx.is_reference() becomes true — "is_reference";
#      3 the listen-before-claim delay — "listen"/"claim" or "C-32";
#      4 whether transport and music datagrams are routed via the events
#        connections — "events connection/output/datagram/feed/routing".
#    The answering paragraph may be extended by the paragraph immediately
#    after it when that one is not itself another trace point. It must:
#      - state the point's required content: (1) the word "sender" (outside
#        a file:line pointer); (2) "true"/"false"/"yes"/"no"/"becomes"/
#        "never"/"stays"/"remains" within 60 characters of is_reference;
#        (3) the value
#        1.5 s (or 1500 ms, or 1 500 000 000 ns) together with "C-32" or
#        "listen"/"claim"; (4) the words "transport", "music" and "events";
#      - carry at least one "file.rs:<digits>" pointer OR a [measured] log
#        line (a line that mentions log / printed / output / stdout /
#        stderr / eprintln / trace / "show", or quotes the output in
#        backticks, and whose own or adjacent line carries [measured]);
#      - carry an evidence tag;
#      - contain an explicit answer, not merely a restated question: at
#        least one non-question line (not ending in "?") must carry the
#        pointer or the measured log line. The required content is matched
#        on the answer text with the prompt sentence removed (a sentence
#        containing "whether" or ending in "?"), so the question itself
#        cannot satisfy it.
#  * (AC 3, interim conclusion) A paragraph containing "conclusion" must name
#    at least one concrete link of the chain — reference election, the
#    is_reference test, transport fill/fire, events routing, or synth
#    ingestion — and must contain a stop cue (stops/breaks/drops/discards/
#    never/fails/gated/rejected/does not reach/...), the word "transport",
#    a "file.rs:<digits>" pointer (or measured log line) and an evidence tag.
#  * (DoD, evidence) Every line carrying a "file.rs:<digits>" pointer must
#    carry an evidence tag on itself or an adjacent line. All four trace
#    points and the conclusion must be tagged. The captured run is
#    [measured]; source reading is [code].
#  * (DoD, logging) If section (a) says diagnostic logging was used
#    (temporary/diagnostic/debug logging, instrumented, eprintln, dbg!,
#    println!, "added a log", "logging was added"), it must also (a) contain
#    a [measured] log line, and (b) say the logging lived in the scratch
#    copy and not in the frozen tree — one paragraph containing "scratch"
#    and "frozen" and a negation (not / never / rather / instead / outside /
#    read-only). Merely quoting a log line as evidence does not trigger this
#    rule: the ticket asks where added logging lived, not whether output is
#    quoted.
#  * (Section marker) Section (a) keeps exactly one "verdict:" line and it
#    still says "pending": the verdict itself belongs to 002-06/002-13, not
#    to this ticket.
#  * "The frozen POC is not modified" is not checkable from the report; it is
#    covered by checks/002-01.sh and checks/003-01.sh.
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

has_tag() {
  printf '%s\n' "$1" | grep -Eq '\[(code|measured|kernel-doc|memory|decision)\]'
}

has_ptr() {
  printf '%s\n' "$1" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+(-[0-9]+)?'
}

# Remove file:line pointers so that "sender.rs:408" is not the word "sender".
strip_ptrs() {
  sed -E 's/[A-Za-z0-9_.\/-]+\.rs:[0-9]+(-[0-9]+)?//g'
}

# The answer text of a chunk: pointers stripped, and the prompt sentence
# (the part that restates the question with "whether" or ends in "?")
# removed, so the required content cannot be satisfied by the question.
plain_answers() {
  printf '%s\n' "$1" | strip_ptrs | awk '
    {
      line = $0
      if (tolower(line) ~ /whether/) {
        p = index(tolower(line), "whether")
        tail = substr(line, p)
        n = length(tail)
        for (i = 1; i <= n; i++) {
          if (substr(tail, i, 1) == ".") {
            nxt = substr(tail, i + 1, 1)
            if (i == n || nxt == " " || nxt == "*" || nxt == "_") break
          }
        }
        if (i <= n) line = substr(line, 1, p - 1) substr(tail, i + 1)
      }
      while (match(line, /\?/)) line = substr(line, RSTART + 1)
      print line
    }'
}

is_fence_line() {
  printf '%s\n' "$1" | grep -Eq '^[[:space:]]*```'
}

# A line that looks like captured runtime evidence (a log line, a quoted
# output line), independent of its tag; the tag may sit on it or next to it.
has_log_cue() {
  printf '%s\n' "$1" | grep -Eqi '`|(^|[^[:alnum:]_])logs?([^[:alnum:]_]|$)|printed|printout|output|stdout|stderr|eprintln|trace|shows?'
}

block_has_log() {
  local block="$1" l i j
  local -a lines=()
  while IFS= read -r l; do
    lines+=("$l")
  done <<EOF
$block
EOF
  for ((i = 0; i < ${#lines[@]}; i++)); do
    has_log_cue "${lines[$i]}" || continue
    for j in $((i - 1)) "$i" $((i + 1)); do
      [ "$j" -ge 0 ] && [ "$j" -lt "${#lines[@]}" ] || continue
      if printf '%s\n' "${lines[$j]}" | grep -qF '[measured]'; then
        return 0
      fi
    done
  done
  return 1
}

# At least one non-question line carries a pointer or a measured log line:
# this is what separates an answer from a restated question. A log line may
# take its [measured] tag from an adjacent line.
has_declarative_evidence() {
  local block="$1" l i j
  local -a lines=()
  while IFS= read -r l; do
    lines+=("$l")
  done <<EOF
$block
EOF
  for ((i = 0; i < ${#lines[@]}; i++)); do
    l=${lines[$i]}
    [ -n "$l" ] || continue
    printf '%s\n' "$l" | grep -Eq '^#+[[:space:]]' && continue
    case "$l" in *\?*) continue ;; esac
    has_ptr "$l" && return 0
    has_log_cue "$l" || continue
    for j in $((i - 1)) "$i" $((i + 1)); do
      [ "$j" -ge 0 ] && [ "$j" -lt "${#lines[@]}" ] || continue
      if printf '%s\n' "${lines[$j]}" | grep -qF '[measured]'; then
        return 0
      fi
    done
  done
  return 1
}

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-05 does not exist yet.\n'
  exit 1
fi

# ---------------------------------------------------------------------------
# Locate section (a) and its (b) boundary
# ---------------------------------------------------------------------------

a_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(a\)')
b_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(b\)')

if [ -z "$a_no" ]; then
  bad "section (a) Inert check families not found (heading containing '(a)')"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
if [ -z "$b_no" ] || [ "$b_no" -le "$a_no" ]; then
  bad "section (b) heading not found after section (a); cannot bound section (a)"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
a_line=$(sed -n "${a_no}p" "$REPORT")
a_hashes=$(printf '%s' "$a_line" | sed -E 's/^(#+).*/\1/')
a_lvl=${#a_hashes}
ok "section (a) found at line $a_no (level $a_lvl), section (b) at line $b_no"

SECTION=$(line_range "$REPORT" "$((a_no + 1))" "$((b_no - 1))")
nonempty=$(printf '%s\n' "$SECTION" | grep -c '[^[:space:]]' || true)

# Placeholder guard: the skeleton's "To be filled by ..." sentence with no
# pointer and no captured block is not a section.
if [ "$nonempty" -lt 4 ] \
   && printf '%s\n' "$SECTION" | grep -Eqi 'to be filled' \
   && ! printf '%s\n' "$SECTION" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+'; then
  bad "section (a) is still the placeholder ($nonempty non-empty lines, no pointers, no captured run)"
  printf '\n1 check(s) failed; the section for 002-05 has not been written yet.\n'
  exit 1
fi

# Is report line $1 inside a fenced code block within (a)?
is_inside_fence() {
  local n="$1" k=$((a_no + 1)) f=0
  while [ "$k" -lt "$n" ]; do
    is_fence_line "$(sed -n "${k}p" "$REPORT")" && f=$((1 - f))
    k=$((k + 1))
  done
  [ "$f" -eq 1 ]
}

# Paragraphs of section (a): non-empty runs, headings and blank lines split.
# Prints "start end" per paragraph, absolute report line numbers.
para_ranges() {
  local k=$((a_no + 1)) start=0 line
  while [ "$k" -lt "$b_no" ]; do
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
    printf '%s %s\n' "$start" "$((b_no - 1))"
  fi
}

TRACE_MARKERS='transport reference|is_reference|listen|C-32|events (connection|connections|output|datagram|feed|routing)|conclusion'

# Paragraphs (outside fenced blocks) whose text matches ERE $1. Each is
# extended by the immediately following paragraph when the two are separated
# by exactly one blank line and that paragraph is not itself a trace point.
candidates_for() {
  local re="$1" ranges s e k hit ns ne found
  ranges=$(para_ranges)
  while read -r s e; do
    [ -n "$s" ] || continue
    hit=0
    k=$s
    while [ "$k" -le "$e" ]; do
      if ! is_inside_fence "$k" \
         && sed -n "${k}p" "$REPORT" | grep -Eqi "$re"; then
        hit=1
        break
      fi
      k=$((k + 1))
    done
    [ "$hit" -eq 1 ] || continue
    ns=""; ne=""
    while read -r found e2; do
      [ -n "$found" ] || continue
      if [ "$found" -gt "$e" ]; then ns=$found; ne=$e2; break; fi
    done <<EOF2
$ranges
EOF2
    if [ -n "$ns" ] && [ "$ns" -eq $((e + 2)) ]; then
      if ! line_range "$REPORT" "$ns" "$ne" | grep -Eqi "$TRACE_MARKERS"; then
        printf '%s %s\n' "$s" "$ne"
        continue
      fi
    fi
    printf '%s %s\n' "$s" "$e"
  done <<EOF2
$ranges
EOF2

  # A deeper heading inside (a) may be the anchor ("### Trace point 2 — …"),
  # with the answer in the paragraph that follows it. Treat heading + answer
  # as one candidate chunk.
  k=$((a_no + 1))
  while [ "$k" -lt "$b_no" ]; do
    line=$(sed -n "${k}p" "$REPORT")
    if printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]' \
       && printf '%s\n' "$line" | grep -Eqi "$re"; then
      j=$((k + 1))
      while [ "$j" -lt "$b_no" ] \
            && [ -z "$(sed -n "${j}p" "$REPORT" | tr -d '[:space:]')" ]; do
        j=$((j + 1))
      done
      if [ "$j" -lt "$b_no" ]; then
        e=$j
        while [ $((e + 1)) -lt "$b_no" ] \
              && [ -n "$(sed -n "$((e + 1))p" "$REPORT" | tr -d '[:space:]')" ] \
              && ! sed -n "$((e + 1))p" "$REPORT" | grep -Eq '^#+[[:space:]]'; do
          e=$((e + 1))
        done
        printf '%s %s\n' "$k" "$e"
      fi
    fi
    k=$((k + 1))
  done
}

# Does a chunk state every required ERE, with evidence and an answer?
chunk_ok() {
  local block="$1"; shift
  local plain
  plain=$(plain_answers "$block")
  for re in "$@"; do
    printf '%s\n' "$plain" | grep -Eqi "$re" || return 1
  done
  if ! has_ptr "$block" && ! block_has_log "$block"; then return 1; fi
  has_tag "$block" || return 1
  has_declarative_evidence "$block" || return 1
  return 0
}

# ---------------------------------------------------------------------------
# AC 1 — captured 25 s run, family counts, reproduction status
# ---------------------------------------------------------------------------

# Collect fenced blocks in (a): "start end" per block (fence lines included).
blocks=""
in_f=0
f_start=0
k=$((a_no + 1))
while [ "$k" -lt "$b_no" ]; do
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

# Print the line if the family named by $2 appears with a count ("N of M" or
# "N/M") and a family cue before the next family keyword on the same line.
# This keeps a music-family match from being satisfied by the transport
# count that follows it on a shared final-summary line.
family_count_line() {
  local content="$1" fam="$2" cues="$3"
  local line l rest cut idx
  while IFS= read -r line; do
    l=$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')
    case "$l" in *"$fam"*) ;; *) continue ;; esac
    rest=${l#*"$fam"}
    for cut in transport audio volume clock 'map:' 'net:'; do
      [ "$cut" = "$fam" ] && continue
      if [[ "$rest" == *"$cut"* ]]; then
        idx=${rest%%"$cut"*}
        rest=$idx
      fi
    done
    if printf '%s\n' "$rest" | grep -Eq '[0-9]+([[:space:]]+of[[:space:]]+[0-9]+|/[0-9]+)' \
       && printf '%s\n' "$rest" | grep -Eqi "$cues"; then
      printf '%s\n' "$line"
      return 0
    fi
  done <<EOF
$content
EOF
  return 1
}

have_block=0
have_music=0
have_transport=0
have_tagged=0
music_where=""
transport_where=""

while read -r fs fe; do
  [ -n "$fs" ] || continue
  have_block=1
  content=$(line_range "$REPORT" "$((fs + 1))" "$((fe - 1))")
  mline=$(family_count_line "$content" 'music' 'window|measurement|steady|moving|datagram|click|no data|pass|fail')
  tline=$(family_count_line "$content" 'transport' 'datagram|click|playhead|change|pulse|no data|pass|fail|window')
  [ -n "$mline" ] && { have_music=1; [ -z "$music_where" ] && music_where="block $fs-$fe"; }
  [ -n "$tline" ] && { have_transport=1; [ -z "$transport_where" ] && transport_where="block $fs-$fe"; }
  lo=$((fs - 3)); [ "$lo" -lt 1 ] && lo=1
  hi=$((fe + 3))
  if line_range "$REPORT" "$lo" "$hi" | grep -qF '[measured]'; then
    have_tagged=1
  fi
done <<EOF
$blocks
EOF

if [ "$have_block" -eq 0 ]; then
  bad "AC 1: no captured run output (fenced code block) in section (a)"
else
  if [ "$have_music" -eq 1 ]; then
    ok "AC 1: captured block holds a music check-family count line ($music_where)"
  else
    bad "AC 1: captured block has no music check-family count line (need 'N of M' or 'N/M' plus a family cue)"
  fi
  if [ "$have_transport" -eq 1 ]; then
    ok "AC 1: captured block holds a transport check-family count line ($transport_where)"
  else
    bad "AC 1: captured block has no transport check-family count line (need 'N of M' or 'N/M' plus a family cue)"
  fi
  if [ "$have_tagged" -eq 1 ]; then
    ok "AC 1: the captured run output is tagged [measured]"
  else
    bad "AC 1: the captured run output is not tagged [measured] (tag the fence or a line within 3 of it)"
  fi
fi

run_duration_re='(--duration[[:space:]]*25([^0-9]|$))|((^|[^0-9])25[[:space:]]*(s|sec|secs|seconds?)([^[:alnum:]]|$))|((^|[^0-9])25-second)'
if printf '%s\n' "$SECTION" | grep -Eqi "$run_duration_re"; then
  ok "AC 1: section (a) states a 25 s run"
else
  bad "AC 1: section (a) does not state the 25 s duration ('25 s', '25-second' or '--duration 25')"
fi

# Reproduction status: explicit, either polarity.
status=none
status_line=""
while IFS= read -r l; do
  low=$(printf '%s' "$l" | tr '[:upper:]' '[:lower:]')
  [ -n "$low" ] || continue
  if printf '%s\n' "$low" | grep -Eq 'not reproduc|did not reproduc|didn.t reproduc|does not reproduc|no longer reproduc|failed to reproduc|could not reproduc|not observed|does not occur|not seen|no anomaly'; then
    status=negative
    status_line=$l
    break
  fi
  if printf '%s\n' "$low" | grep -Eq 'reproduc|confirmed|still present|same anomaly|anomaly is present|observed again|appears again'; then
    status=positive
    status_line=$l
  fi
done <<EOF
$SECTION
EOF

case "$status" in
  positive)
    ok "AC 1: the section states that the anomaly reproduced (\"$(printf '%s' "$status_line" | cut -c1-100)\")" ;;
  negative)
    ok "AC 1: the section states that the anomaly did NOT reproduce (\"$(printf '%s' "$status_line" | cut -c1-100)\")" ;;
  *)
    bad "AC 1: the section does not state whether the anomaly reproduced or not reproduced" ;;
esac

# Run source: poc-untouched equivalent (scratch copy or a checkout of the tag).
source_ok=0
source_where=""
ranges=$(para_ranges)
prev=""
while read -r s e; do
  [ -n "$s" ] || continue
  txt=$(line_range "$REPORT" "$s" "$e")
  combined="$prev"$'\n'"$txt"
  if printf '%s\n' "$combined" | grep -qi 'poc-untouched' \
     && printf '%s\n' "$combined" | grep -Eqi 'scratch|poc-scratch|checkout|tag|frozen' \
     && printf '%s\n' "$combined" | grep -Eqi 'byte-identical|content-identical|identical|equivalen'; then
    source_ok=1
    source_where="lines $s-$e"
    break
  fi
  prev="$txt"
done <<EOF
$ranges
EOF

if [ "$source_ok" -eq 1 ]; then
  ok "AC 1: the run source is stated as poc-untouched-equivalent ($source_where)"
else
  bad "AC 1: the section does not state the run source as poc-untouched-equivalent (poc-untouched + scratch/checkout/tag + identical/equivalent)"
fi

# ---------------------------------------------------------------------------
# AC 2 — the four trace points, each answered with a pointer or a log line
# ---------------------------------------------------------------------------

trace_point() {
  local label="$1" anchor="$2"
  shift 2
  local candidates first="" s e block plain missing re before

  candidates=$(candidates_for "$anchor")
  if [ -z "$candidates" ]; then
    bad "AC 2 point $label: no paragraph in section (a) answering this point (marker not found)"
    return
  fi

  while read -r s e; do
    [ -n "$s" ] || continue
    block=$(line_range "$REPORT" "$s" "$e")
    [ -z "$first" ] && first="$s $e"
    if chunk_ok "$block" "$@"; then
      ok "AC 2 point $label: answered with evidence (lines $s-$e)"
      return
    fi
  done <<EOF
$candidates
EOF

  s=${first% *}
  e=${first#* }
  block=$(line_range "$REPORT" "$s" "$e")
  plain=$(plain_answers "$block")
  before=$fail
  for re in "$@"; do
    if ! printf '%s\n' "$plain" | grep -Eqi "$re"; then
      bad "AC 2 point $label: paragraph (lines $s-$e) does not state the required content: $re"
    fi
  done
  if ! has_ptr "$block" && ! block_has_log "$block"; then
    bad "AC 2 point $label: neither a file:line pointer nor a [measured] log line (lines $s-$e)"
  fi
  if ! has_tag "$block"; then
    bad "AC 2 point $label: no evidence tag on the answer (lines $s-$e)"
  fi
  if ! has_declarative_evidence "$block"; then
    bad "AC 2 point $label: the point is restated as a question but never answered with a pointer or log line (lines $s-$e)"
  fi
  if [ "$fail" -eq "$before" ]; then
    ok "AC 2 point $label: answered (lines $s-$e)"
  fi
}

# 1 — who is the transport reference.
trace_point "1 (transport reference)" \
  'transport reference|who acts[^.]*reference|reference role' \
  '(^|[^[:alnum:]_])sender([^[:alnum:]_]|$)'

# 2 — whether ctx.is_reference() becomes true.
trace_point "2 (is_reference)" \
  'is_reference' \
  'is_reference[^.]{0,60}(true|false|yes|no|never|stays|remains|becomes)|(true|false|yes|no|never|stays|remains|becomes)[^.]{0,60}is_reference'

# 3 — the listen-before-claim delay.
trace_point "3 (listen-before-claim)" \
  'listen|claim|C-32' \
  '1[.,]5[^0-9]{0,3}(s|sec|secs|second|seconds)|1500[[:space:]]*ms|1[ _]?500[ _]?000[ _]?000[[:space:]]*ns' \
  'C-32|listen|claim'

# 4 — transport and music datagrams via the events connections.
trace_point "4 (events routing)" \
  'events (connection|connections|output|datagram|feed|routing)|via (the )?events' \
  '(^|[^[:alnum:]_])transport([^[:alnum:]_]|$)' \
  '(^|[^[:alnum:]_])music([^[:alnum:]_]|$)' \
  'events'

# ---------------------------------------------------------------------------
# AC 3 — interim conclusion naming the step where transport state stops
# ---------------------------------------------------------------------------

conclusion_ok=0
conclusion_where=""
candidates=$(candidates_for 'interim conclusion|(^|[^[:alnum:]_])conclusion([^[:alnum:]_]|$)')
while read -r s e; do
  [ -n "$s" ] || continue
  block=$(line_range "$REPORT" "$s" "$e")
  plain=$(printf '%s\n' "$block" | strip_ptrs)
  link=0
  printf '%s\n' "$plain" | grep -Eqi '(^|[^[:alnum:]_])elect(s|ed|ing|ion|ions)?([^[:alnum:]_]|$)' && link=1
  printf '%s\n' "$plain" | grep -q 'is_reference' && link=1
  printf '%s\n' "$plain" | grep -Eqi '(^|[^[:alnum:]_])(fill|fire)([^[:alnum:]_]|$)' && link=1
  printf '%s\n' "$plain" | grep -Eqi 'events[^.]{0,30}(routing|route|routed|connection|output|datagram)|routed[^.]{0,30}events' && link=1
  printf '%s\n' "$plain" | grep -Eqi 'synth[^.]{0,60}(ingest|accept|ignore|drop|filter|reject)|ingest' && link=1
  [ "$link" -eq 1 ] || continue
  printf '%s\n' "$plain" | grep -Eqi 'stop|stops|stopped|break|breaks|broken|drop|drops|dropped|discard|discards|discarded|never|fail|fails|blocked|gate|gated|rejected|does not reach|do not reach|not reach' || continue
  printf '%s\n' "$plain" | grep -Eqi 'transport' || continue
  { has_ptr "$block" || block_has_log "$block"; } || continue
  has_tag "$block" || continue
  conclusion_ok=1
  conclusion_where="lines $s-$e"
  break
done <<EOF
$candidates
EOF

if [ "$conclusion_ok" -eq 1 ]; then
  ok "AC 3: the interim conclusion names a chain link and a stop with evidence ($conclusion_where)"
else
  bad "AC 3: no concluded paragraph names where transport state stops (need a chain link: election, is_reference, fill/fire, events routing or synth ingestion; plus a stop cue, 'transport', a pointer/log line and a tag)"
fi

# ---------------------------------------------------------------------------
# DoD — evidence tags on pointer-bearing lines
# ---------------------------------------------------------------------------

tag_problems=0
k=$((a_no + 1))
while [ "$k" -lt "$b_no" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  if has_ptr "$l" && ! is_inside_fence "$k"; then
    lo=$((k - 1))
    hi=$((k + 1))
    if ! has_tag "$(line_range "$REPORT" "$lo" "$hi")"; then
      bad "DoD: line $k carries a file:line pointer but no evidence tag on itself or an adjacent line"
      tag_problems=$((tag_problems + 1))
    fi
  fi
  k=$((k + 1))
done
[ "$tag_problems" -eq 0 ] && ok "DoD: every pointer-bearing line carries an evidence tag"

# ---------------------------------------------------------------------------
# DoD — diagnostic logging, if used, lives in the scratch copy
# ---------------------------------------------------------------------------

if printf '%s\n' "$SECTION" | grep -Eqi 'temporary (log|logging)|diagnostic (log|logging)|debug (log|logging|print)|instrument(ed|ation)|eprintln|dbg!|println!|added (a )?log|added logging|inserted (a )?log|log line was added|logging was added'; then
  if ! printf '%s\n' "$SECTION" | grep -Eqi 'scratch'; then
    bad "DoD: diagnostic logging is mentioned but the section does not say it lived in the scratch copy"
  fi
  log_place_ok=0
  log_para_seen=0
  ranges=$(para_ranges)
  while read -r s e; do
    [ -n "$s" ] || continue
    block=$(line_range "$REPORT" "$s" "$e")
    if ! printf '%s\n' "$block" | grep -Eqi 'temporary (log|logging)|diagnostic (log|logging)|debug (log|logging|print)|instrument(ed|ation)|eprintln|dbg!|println!|added (a )?log|added logging|inserted (a )?log|log line was added|logging was added'; then
      continue
    fi
    log_para_seen=1
    if printf '%s\n' "$block" | grep -Eqi 'scratch' \
       && printf '%s\n' "$block" | grep -Eqi 'frozen' \
       && printf '%s\n' "$block" | grep -Eqi 'not|never|rather|instead|outside|read-only|untouched|unmodified|left alone'; then
      log_place_ok=1
    fi
  done <<EOF
$ranges
EOF
  if [ "$log_para_seen" -eq 0 ]; then
    bad "DoD: the logging mention could not be located in a paragraph"
  elif [ "$log_place_ok" -eq 1 ]; then
    ok "DoD: diagnostic logging is stated to live in the scratch copy, not the frozen tree"
  else
    bad "DoD: the paragraph that mentions diagnostic logging does not say it lived in the scratch copy, not in the frozen tree"
  fi
  if printf '%s\n' "$SECTION" | grep -qF '[measured]' \
     && block_has_log "$SECTION"; then
    ok "DoD: a [measured] log line is present for the logging"
  else
    bad "DoD: diagnostic logging is mentioned but no [measured] log line is recorded"
  fi
else
  ok "DoD: no diagnostic logging is claimed, so no scratch-copy statement is required"
fi

# ---------------------------------------------------------------------------
# Section marker — exactly one pending verdict in (a)
# ---------------------------------------------------------------------------

verdict_count=$(printf '%s\n' "$SECTION" \
  | grep -Eci '(^|[^[:alnum:]])[*_]*verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "section (a) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "section (a) has $verdict_count 'verdict:' lines; exactly one section-level verdict is required"
else
  if printf '%s\n' "$SECTION" | grep -Eqi '(^|[^[:alnum:]])[*_]*verdict:[^A-Za-z]*pending'; then
    ok "section (a) still carries its one pending verdict marker"
  else
    bad "section (a)'s verdict marker no longer says 'pending' (the verdict belongs to 002-06/002-13)"
  fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
if [ "$fail" -gt 0 ]; then
  exit 1
fi
