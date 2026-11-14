#!/usr/bin/env bash
# checks/002-06.sh — checks for ticket 002-06
# "Inert families: music trace, minimal condition and verdict".
#
# The deliverable is section (a) Inert check families of
#   doc/design/reports/archaeology/REPORT-0b.md
# after tickets 002-05 (baseline run + transport trace) and 002-06 (music
# trace, minimal condition, final verdict and hand-off). This script reads
# the report only; it never writes anywhere and never touches the POC tree.
#
# The conventions below are spelled out so the author can comply exactly.
# Section (a) runs from the "## (a) ..." heading to the "## (b) ..." heading.
#
#  * (AC 1, three music points) Each point is located by its own marker
#    (case-insensitive) in a paragraph outside a fenced block and must be
#    answered, not restated as a question:
#      1 settling exclusions — marker "SETTLE_SECS", "settling", "settle
#        horizon" or "settle_until". The answer must name SETTLE_SECS, the
#        3 s value (3 s / 3 sec / 3-second / three seconds), say what it
#        does (skip/exclude/inert/silent/resume/conduct/not analysed), and
#        cite both named sites with a file:line pointer — audio.rs:604 and
#        audio.rs:644 (a pointer range covering the line counts), or carry
#        a [measured] log line.
#      2 the voices flag — marker "voices ... flag/mode/analyzer/path" or
#        "--voices". The answer must name the flag, state its value (true /
#        false / on / off / enabled / disabled / "voices: true", or
#        true/false for the synth or feed analyzer) and tie it to the music
#        family (music / analyse_voices / sum path / voice path), with a
#        pointer to main.rs or audio.rs, or a [measured] log line.
#      3 voice frames and place_voices — marker "place_voices". The answer
#        must name place_voices, say whether voice frames reach it (reach /
#        arrive / placed / popped / call / yes / no / never / not) and cite
#        audio.rs:601, 633 or 636 (range covering the line counts) or carry
#        a [measured] log line.
#    Every point must carry an evidence tag and at least one non-question
#    line that carries the pointer or the measured log line.
#  * (AC 2, minimal condition) EITHER a captured 25 s summary in a fenced
#    block that is tagged [measured] within three lines of a fence, has a
#    command line within 25 lines before it (or inside it) naming the binary
#    (hadropoc / cargo run), "loopback" and "--duration 25", and whose music
#    line and transport line each carry a positive window count — "N of M"
#    or "N/M" with N > 0 (e.g. "voice channels 1915/1915 windows exact" for
#    music, "playhead identical for 40880/40880 datagrams" for transport);
#    OR one paragraph that states that no such command line exists with the
#    frozen code, with the word "frozen", a reason cue (because / since /
#    reason / why / due to) and an evidence tag.
#  * (AC 3, final verdict) Section (a) carries exactly one "verdict:" line,
#    it is the last non-empty line of the section, it is no longer
#    "pending", and it begins with one of the two allowed forms:
#    "instrument bug, mechanism: ..." or "not a bug, explanation: ...".
#    The verdict (or a line within two lines of it) must name both "music"
#    and "transport", carry a [code] or [measured] tag, and carry a
#    file:line pointer, a captured count pair ("N of M" / "N/M"), or a
#    reference to the captured run/summary. The old
#    "**verdict: pending**" placeholder must be gone: keeping it next to a
#    final verdict gives two verdict lines and fails.
#  * (AC 4, hand-off) A heading or a bold phrase "for task 003"
#    (case-insensitive) followed by one paragraph of at least 80 non-space
#    characters that mentions both "music" and "transport", carries an
#    inert cue (inert / zero / 0 of 0 / no samples / no windows / never
#    analysed / not analysed / skipped / excluded) and an evidence tag.
#  * (DoD, evidence) Every line carrying a "file.rs:<digits>" pointer in
#    section (a) must carry an evidence tag on itself or an adjacent line.
#  * (DoD, logging) If section (a) says diagnostic logging was used
#    (temporary/diagnostic/debug logging, instrumented, eprintln, dbg!,
#    println!, "added a log", "logging was added"), it must also (a) contain
#    a [measured] log line, and (b) say the logging lived in the scratch
#    copy and not in the frozen tree — one paragraph containing "scratch"
#    and "frozen" and a negation.
#  * (DoD, run source) If section (a) holds a captured run block, a
#    paragraph must state the source: "scratch" plus "poc-untouched" (or
#    identical/equivalent) plus "frozen"/tag/checkout, or state that the
#    scratch copy was instrumented and the frozen tree was not touched.
#  * "The frozen POC is not modified" is not checkable from the report; it
#    is covered by checks/002-01.sh and checks/003-01.sh.
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

# Does TEXT contain a file:line pointer to FILE covering line WANT?
# A pointer range ("audio.rs:603-604") covers every line in it, and a
# comma-separated list ("audio.rs:604, 644") covers each listed line.
ptr_covers() {
  local file="$1" want="$2" text="$3" p path nums seg a b
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    path=${p%%:*}
    [ "${path##*/}" = "$file" ] || continue
    nums=${p#*:}
    while IFS= read -r seg; do
      seg=$(printf '%s' "$seg" | tr -d '[:space:]')
      [ -n "$seg" ] || continue
      a=${seg%%-*}
      b=${seg##*-}
      if [ "$a" -le "$want" ] 2>/dev/null && [ "$want" -le "$b" ] 2>/dev/null; then
        return 0
      fi
    done <<EOF2
$(printf '%s' "$nums" | tr ',' '\n')
EOF2
  done <<EOF
$(printf '%s\n' "$text" | grep -Eo '[A-Za-z0-9_./-]+\.rs:[0-9]+(-[0-9]+)?([[:space:]]*,[[:space:]]*[0-9]+(-[0-9]+)?)*' || true)
EOF
  return 1
}

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-06 does not exist yet.\n'
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

if [ "$nonempty" -lt 4 ]; then
  bad "section (a) is empty or still a placeholder ($nonempty non-empty lines)"
  printf '\n1 check(s) failed; the section for 002-06 has not been written yet.\n'
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

POINT_MARKERS='SETTLE_SECS|settling|settle horizon|settle_until|voices[^.]{0,60}(flag|mode|analy[sz]er|path)|--voices|place_voices'

# Paragraphs (outside fenced blocks) whose text matches ERE $1. Each is
# extended by the immediately following paragraph when the two are separated
# by exactly one blank line and that paragraph is not itself another point.
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
      if ! line_range "$REPORT" "$ns" "$ne" | grep -Eqi "$POINT_MARKERS"; then
        printf '%s %s\n' "$s" "$ne"
        continue
      fi
    fi
    printf '%s %s\n' "$s" "$e"
  done <<EOF2
$ranges
EOF2

  # A deeper heading inside (a) may be the anchor ("### Point 1 — settling"),
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

# ---------------------------------------------------------------------------
# Fenced blocks in section (a)
# ---------------------------------------------------------------------------

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

block_count=$(printf '%s\n' "$blocks" | grep -c '[0-9]' || true)

# ---------------------------------------------------------------------------
# AC 2 — minimal condition: a 25 s summary with both families non-zero,
# or an explicit statement that no such command line exists
# ---------------------------------------------------------------------------

# Print the first line of CONTENT that belongs to family FAM and carries a
# positive count pair. The family text is cut at the next family keyword or
# a "|" so a shared final-summary line cannot satisfy music with transport.
family_positive() {
  local content="$1" fam="$2"
  local line l rest cut
  while IFS= read -r line; do
    l=$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')
    case "$l" in *"$fam"*) ;; *) continue ;; esac
    rest=${l#*"$fam"}
    for cut in transport audio volume clock 'map:' 'net:' '|'; do
      [ "$cut" = "$fam" ] && continue
      if [[ "$rest" == *"$cut"* ]]; then
        rest=${rest%%"$cut"*}
      fi
    done
    if printf '%s\n' "$rest" | awk '
      {
        s = $0
        while (match(s, /[0-9]+[ \t]+of[ \t]+[0-9]+/)) {
          t = substr(s, RSTART, RLENGTH)
          split(t, a, /[ \t]+of[ \t]+/)
          if (a[1] + 0 > 0 && a[2] + 0 > 0) exit 0
          s = substr(s, RSTART + RLENGTH)
        }
        s = $0
        while (match(s, /[0-9]+\/[0-9]+/)) {
          t = substr(s, RSTART, RLENGTH)
          split(t, a, "/")
          if (a[1] + 0 > 0 && a[2] + 0 > 0) exit 0
          s = substr(s, RSTART + RLENGTH)
        }
        exit 1
      }'; then
      printf '%s\n' "$line"
      return 0
    fi
  done <<EOF
$content
EOF
  return 1
}

command_near() {
  local fs="$1" fe="$2" lo hi
  lo=$((fs - 25)); [ "$lo" -lt 1 ] && lo=1
  hi=$((fe + 2))
  line_range "$REPORT" "$lo" "$hi" | grep -Ei 'hadropoc|cargo[[:space:]]+run' \
    | grep -i 'loopback' \
    | grep -Eq -- '--duration[[:space:]=]*25' && return 0
  return 1
}

ac2_ok=0
ac2_where=""
while read -r fs fe; do
  [ -n "$fs" ] || continue
  content=$(line_range "$REPORT" "$((fs + 1))" "$((fe - 1))")
  lo=$((fs - 3)); [ "$lo" -lt 1 ] && lo=1
  hi=$((fe + 3))
  line_range "$REPORT" "$lo" "$hi" | grep -qF '[measured]' || continue
  command_near "$fs" "$fe" || continue
  mline=$(family_positive "$content" 'music') || continue
  tline=$(family_positive "$content" 'transport') || continue
  ac2_ok=1
  ac2_where="block $fs-$fe"
  ac2_music=$(printf '%s' "$mline" | cut -c1-90)
  ac2_transport=$(printf '%s' "$tline" | cut -c1-90)
  break
done <<EOF
$blocks
EOF

# The alternative: an explicit statement that no such command line exists
# with the frozen code, with a reason.
ac2_none=0
if [ "$ac2_ok" -eq 0 ]; then
  ranges=$(para_ranges)
  while read -r s e; do
    [ -n "$s" ] || continue
    [ -n "$(line_range "$REPORT" "$s" "$e" | grep -Ei 'no (such )?command line|no command line|none exists|no combination|no flag|no wiring|no way')" ] || continue
    [ -n "$(line_range "$REPORT" "$s" "$e" | grep -i 'frozen')" ] || continue
    [ -n "$(line_range "$REPORT" "$s" "$e" | grep -Ei 'because|since|reason|why|due to|owing to|therefore')" ] || continue
    line_range "$REPORT" "$s" "$e" | grep -qF '[measured]' || line_range "$REPORT" "$s" "$e" | grep -qF '[code]' || continue
    ac2_none=1
    ac2_none_where="lines $s-$e"
    break
  done <<EOF
$ranges
EOF
fi

if [ "$ac2_ok" -eq 1 ]; then
  ok "AC 2: captured 25 s summary with music and transport windows > 0 and its command line ($ac2_where)"
elif [ "$ac2_none" -eq 1 ]; then
  ok "AC 2: the section states that no such command line exists with the frozen code and gives the reason ($ac2_none_where)"
else
  bad "AC 2: no captured 25 s summary with a command line and music>0 + transport>0 counts, and no explicit statement that no such command line exists (with the reason)"
fi

# ---------------------------------------------------------------------------
# AC 1 — the three music points, each answered with a pointer or a log line
# ---------------------------------------------------------------------------

# Site requirements: a code answer must point at the named place; a measured
# log line may stand in for the pointer (AC 1 says "pointer or log line").
site_settling() {
  local b="$1"
  { ptr_covers audio.rs 604 "$b" && ptr_covers audio.rs 644 "$b"; } \
    || block_has_log "$b"
}

site_voices_flag() {
  local b="$1"
  { ptr_covers main.rs 408 "$b" || ptr_covers main.rs 406 "$b" \
      || ptr_covers main.rs 405 "$b" || ptr_covers audio.rs 511 "$b" \
      || ptr_covers audio.rs 715 "$b" || ptr_covers audio.rs 722 "$b"; } \
    || block_has_log "$b"
}

site_place_voices() {
  local b="$1"
  { ptr_covers audio.rs 601 "$b" || ptr_covers audio.rs 633 "$b" \
      || ptr_covers audio.rs 636 "$b"; } \
    || block_has_log "$b"
}

chunk_ok() {
  local block="$1" site="$2"
  shift 2
  local plain
  plain=$(plain_answers "$block")
  for re in "$@"; do
    printf '%s\n' "$plain" | grep -Eqi "$re" || return 1
  done
  if ! has_ptr "$block" && ! block_has_log "$block"; then return 1; fi
  if [ -n "$site" ] && ! "$site" "$block"; then return 1; fi
  has_tag "$block" || return 1
  has_declarative_evidence "$block" || return 1
  return 0
}

trace_point() {
  local label="$1" anchor="$2" site="$3"
  shift 3
  local candidates first="" s e block plain missing re before

  candidates=$(candidates_for "$anchor")
  if [ -z "$candidates" ]; then
    bad "AC 1 point $label: no paragraph in section (a) answering this point (marker not found)"
    return
  fi

  while read -r s e; do
    [ -n "$s" ] || continue
    block=$(line_range "$REPORT" "$s" "$e")
    [ -z "$first" ] && first="$s $e"
    if chunk_ok "$block" "$site" "$@"; then
      ok "AC 1 point $label: answered with evidence (lines $s-$e)"
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
      bad "AC 1 point $label: paragraph (lines $s-$e) does not state the required content: $re"
    fi
  done
  if ! has_ptr "$block" && ! block_has_log "$block"; then
    bad "AC 1 point $label: neither a file:line pointer nor a [measured] log line (lines $s-$e)"
  fi
  if [ -n "$site" ] && ! "$site" "$block"; then
    bad "AC 1 point $label: the answer does not cite the point's file:line site(s) and carries no [measured] log line (lines $s-$e)"
  fi
  if ! has_tag "$block"; then
    bad "AC 1 point $label: no evidence tag on the answer (lines $s-$e)"
  fi
  if ! has_declarative_evidence "$block"; then
    bad "AC 1 point $label: the point is restated as a question but never answered with a pointer or log line (lines $s-$e)"
  fi
  if [ "$fail" -eq "$before" ]; then
    ok "AC 1 point $label: answered (lines $s-$e)"
  fi
}

# 1 — settling exclusions at audio.rs:604 / audio.rs:644 with SETTLE_SECS.
trace_point "1 (settling exclusions)" \
  'SETTLE_SECS|settling|settle horizon|settle_until' \
  'site_settling' \
  'settle_secs' \
  '(^|[^0-9])3[[:space:]]*(s|sec|secs|second|seconds)([^[:alnum:]]|$)|3-second|three seconds' \
  'skip|skipp|exclu|inert|silent|silence|resum|conduct|not analysed|never analysed|zero'

# 2 — the voices flag in the analyzer.
trace_point "2 (voices flag)" \
  'voices[^.]{0,60}(flag|mode|analy[sz]er|path)|--voices' \
  'site_voices_flag' \
  'voices' \
  'voices[[:space:]]*[:=][[:space:]]*(true|false)|(true|false)[^.]{0,50}(synth|analy[sz]er|feed)|(synth|analy[sz]er|feed)[^.]{0,50}(true|false)|(flag|mode)[^.]{0,40}(on|off|true|false|enabled|disabled)|(on|off|true|false|enabled|disabled)[^.]{0,40}(flag|mode)' \
  'music|analyse_voices|sum path|voice path'

# 3 — whether voice frames reach place_voices.
trace_point "3 (voice frames / place_voices)" \
  'place_voices' \
  'site_place_voices' \
  'place_voices' \
  'voice' \
  'frame' \
  '(^|[^[:alnum:]_])(yes|no|true|false|reach|reaches|reached|arrive|arrives|arrived|placed|pop|popped|call|calls|called|never|not)([^[:alnum:]_]|$)'

# ---------------------------------------------------------------------------
# AC 3 — exactly one final verdict line, last in section (a), both families
# ---------------------------------------------------------------------------

last_nonempty() {
  local k=$((b_no - 1)) l
  while [ "$k" -gt "$a_no" ]; do
    l=$(sed -n "${k}p" "$REPORT")
    if [ -n "$(printf '%s' "$l" | tr -d '[:space:]')" ]; then
      printf '%s\n' "$k"
      return 0
    fi
    k=$((k - 1))
  done
  return 1
}

verdict_nos=$(printf '%s\n' "$SECTION" | grep -Ein '(^|[^[:alnum:]_])[*_]*verdict[*_]*:' | cut -d: -f1 || true)
verdict_count=$(printf '%s\n' "$verdict_nos" | grep -c '[0-9]' || true)

if [ "$verdict_count" -eq 0 ]; then
  bad "AC 3: section (a) has no 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "AC 3: section (a) has $verdict_count 'verdict:' lines; exactly one final verdict is required (remove or replace the pending placeholder)"
else
  vrel=$(printf '%s\n' "$verdict_nos" | head -n 1)
  v_no=$((a_no + vrel))
  vline=$(sed -n "${v_no}p" "$REPORT")
  vbody=$(printf '%s' "$vline" | sed -E 's/^.*verdict[*_]*:[[:space:]]*//')

  # Pending placeholder is not a final verdict.
  if printf '%s' "$vbody" | grep -Eqi '^[*_[:space:]]*pending'; then
    bad "AC 3: the one verdict line is still 'pending' (line $v_no); the final verdict must replace it"
  else
    vform=$(printf '%s' "$vbody" | sed -E 's/^[[:space:]*_]+//')
    if printf '%s' "$vform" | grep -Eqi '^instrument bug,[[:space:]]*[*_]*mechanism:'; then
      ok "AC 3: the verdict line uses the 'instrument bug, mechanism:' form (line $v_no)"
    elif printf '%s' "$vform" | grep -Eqi '^not a bug,[[:space:]]*[*_]*explanation:'; then
      ok "AC 3: the verdict line uses the 'not a bug, explanation:' form (line $v_no)"
    else
      bad "AC 3: the verdict line is not in an allowed final form ('instrument bug, mechanism: …' or 'not a bug, explanation: …') at line $v_no"
    fi

    lo=$((v_no - 1)); [ "$lo" -lt 1 ] && lo=1
    hi=$((v_no + 1))
    vctx=$(line_range "$REPORT" "$lo" "$hi")

    if printf '%s\n' "$vctx" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+(-[0-9]+)?|[0-9]+[[:space:]]+of[[:space:]]+[0-9]+|[0-9]+/[0-9]+|captured (run|summary|output)|run output|summary (line|block)|25 s summary'; then
      ok "AC 3: the verdict carries a file:line pointer or captured run output"
    else
      bad "AC 3: the verdict carries neither a file:line pointer nor captured run output (line $v_no)"
    fi

    if printf '%s\n' "$vctx" | grep -qE '\[(code|measured)\]'; then
      ok "AC 3: the verdict is tagged [code] or [measured]"
    else
      bad "AC 3: the verdict is not tagged [code] or [measured] (line $v_no)"
    fi

    vwide_lo=$((v_no - 2)); [ "$vwide_lo" -lt 1 ] && vwide_lo=1
    vwide_hi=$((v_no + 2))
    vwide=$(line_range "$REPORT" "$vwide_lo" "$vwide_hi")
    if printf '%s\n' "$vwide" | grep -Eqi '(^|[^[:alnum:]_])music([^[:alnum:]_]|$)' \
       && printf '%s\n' "$vwide" | grep -Eqi '(^|[^[:alnum:]_])transport([^[:alnum:]_]|$)'; then
      ok "AC 3: the verdict or its immediate context names both families (music and transport)"
    else
      bad "AC 3: the verdict and its immediate context (lines $vwide_lo-$vwide_hi) do not name both families (music and transport)"
    fi

    vlast=$(last_nonempty || true)
    if [ -n "$vlast" ] && [ "$vlast" -eq "$v_no" ]; then
      ok "AC 3: the verdict is the last non-empty line of section (a)"
    else
      bad "AC 3: section (a) does not end with the verdict (last non-empty line is $vlast, verdict at $v_no); put the hand-off before the verdict"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# AC 4 — one-paragraph hand-off note "for task 003"
# ---------------------------------------------------------------------------

ho_no=""
k=$((a_no + 1))
while [ "$k" -lt "$b_no" ]; do
  if ! is_inside_fence "$k" && sed -n "${k}p" "$REPORT" | grep -Eqi 'for task 003'; then
    ho_no=$k
    break
  fi
  k=$((k + 1))
done

if [ -z "$ho_no" ]; then
  bad "AC 4: no hand-off note 'for task 003' in section (a)"
else
  hline=$(sed -n "${ho_no}p" "$REPORT")
  is_heading=0
  printf '%s\n' "$hline" | grep -Eq '^#+[[:space:]]' && is_heading=1
  is_bold=0
  printf '%s\n' "$hline" | grep -Eqi '\*\*[^*]*for task 003[^*]*\*\*' && is_bold=1
  if [ "$is_heading" -eq 0 ] && [ "$is_bold" -eq 0 ]; then
    bad "AC 4: the 'for task 003' hand-off is neither a heading nor bold (line $ho_no)"
  else
    # Text after the marker on the same line, if any.
    htail=$(printf '%s' "$hline" | awk '
      { i = index(tolower($0), "for task 003")
        if (i > 0) { s = substr($0, i + 12); sub(/^[[:space:]*_.:\xe2\x80\x94-]+/, "", s); print s } }')
    # The body starts with the text after the marker on its own line (empty
    # for a heading or a bare bold marker) and continues through the next
    # paragraph.
    htext="$htail"
    hstart=$((ho_no + 1))
    while [ "$hstart" -lt "$b_no" ] \
          && [ -z "$(sed -n "${hstart}p" "$REPORT" | tr -d '[:space:]')" ]; do
      hstart=$((hstart + 1))
    done

    hend=$hstart
    while [ "$hend" -lt "$b_no" ]; do
      hline2=$(sed -n "${hend}p" "$REPORT")
      if [ -z "$(printf '%s' "$hline2" | tr -d '[:space:]')" ] \
         || printf '%s\n' "$hline2" | grep -Eq '^#+[[:space:]]'; then
        break
      fi
      htext="$htext
$hline2"
      hend=$((hend + 1))
    done

    hchars=$(printf '%s' "$htext" | tr -d '[:space:]' | wc -c)

    if [ "$hchars" -lt 80 ]; then
      bad "AC 4: the hand-off note for task 003 has no paragraph of substance ($hchars non-space characters; need at least 80)"
    else
      if printf '%s\n' "$htext" | grep -Eqi '(^|[^[:alnum:]_])music([^[:alnum:]_]|$)' \
         && printf '%s\n' "$htext" | grep -Eqi '(^|[^[:alnum:]_])transport([^[:alnum:]_]|$)'; then
        ok "AC 4: the hand-off note names both inert families (music and transport)"
      else
        bad "AC 4: the hand-off note does not name both families (music and transport)"
      fi
      if printf '%s\n' "$htext" | grep -Eqi 'inert|zero|0 of 0|no samples|no windows|never analysed|not analysed|skipped|excluded'; then
        ok "AC 4: the hand-off note says what made the families inert"
      else
        bad "AC 4: the hand-off note does not say what made the families inert (need an inert/zero/skipped cue)"
      fi
      if printf '%s\n' "$htext" | grep -qE '\[(code|measured)\]'; then
        ok "AC 4: the hand-off note carries an evidence tag"
      else
        bad "AC 4: the hand-off note carries no evidence tag"
      fi
    fi
  fi
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
# DoD — run source stated for the captured run(s)
# ---------------------------------------------------------------------------

if [ "$block_count" -eq 0 ]; then
  ok "DoD: no captured run block in section (a), so no run-source statement is required"
else
  run_source_ok=0
  ranges=$(para_ranges)
  while read -r s e; do
    [ -n "$s" ] || continue
    block=$(line_range "$REPORT" "$s" "$e")
    if printf '%s\n' "$block" | grep -qi 'scratch'; then
      if printf '%s\n' "$block" | grep -Eqi 'poc-untouched|byte-identical|content-identical|identical|equivalen|unmodified'; then
        if printf '%s\n' "$block" | grep -Eqi 'frozen|checkout|tag'; then
          run_source_ok=1
          break
        fi
      fi
      if printf '%s\n' "$block" | grep -Eqi 'instrument(ed|ation)|diagnostic log|debug log|logging'; then
        if printf '%s\n' "$block" | grep -Eqi 'not|never|outside|rather|instead'; then
          run_source_ok=1
          break
        fi
      fi
    fi
  done <<EOF
$ranges
EOF
  if [ "$run_source_ok" -eq 1 ]; then
    ok "DoD: the source of the captured run is stated (scratch copy / poc-untouched-equivalent)"
  else
    bad "DoD: section (a) has a captured run but no paragraph states its source (scratch copy, poc-untouched-equivalent or instrumented scratch, frozen untouched)"
  fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
if [ "$fail" -gt 0 ]; then
  exit 1
fi
