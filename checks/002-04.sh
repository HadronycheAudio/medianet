#!/usr/bin/env bash
# checks/002-04.sh — checks for ticket 002-04
# "Reading notes: sender.rs, stats.rs and tui.rs".
#
# The deliverable is three sub-sections of section (d) Reading notes of
#   doc/design/reports/archaeology/REPORT-0b.md
# written by the author after a read-only reading of the frozen POC at
# /home/dorian/workspace/hadropoc. This script reads the report only; it never
# writes anywhere and never touches the POC tree.
#
# Conventions required by this check, spelled out so the author can comply
# exactly. They make the prose machine-checkable without fixing its wording.
# The notes already present for audio.rs (002-02) and transport.rs /
# control.rs (002-03) are deliberately not examined and may sit anywhere in
# section (d).
#
#  * (AC 1, layout) Under the "## (d) ..." heading and before "## (e) ..."
#    there is exactly one heading whose text names sender.rs, exactly one
#    naming stats.rs and exactly one naming tui.rs (case-insensitive), at a
#    deeper level than (d). A heading that names two or three of the files is
#    treated as an umbrella container, not as a sub-section, and is ignored;
#    three distinct per-file headings are required. The three sub-sections
#    must be disjoint: no one may contain another. Each sub-section runs from
#    its heading to the next heading of the same or higher level (or to (e)).
#    "About half a page" is proxied, as in sibling ticket 002-03, by at most
#    30 non-empty lines per sub-section; 002-02 used 45 for "about one page".
#    Each sub-section must carry at least 3 DISTINCT "file.rs:<digits>"
#    pointers, at least 2 of them to its own file (sender.rs / stats.rs /
#    tui.rs) — the notes are a reading of that file, not a cross-reference.
#  * (AC 2, the three hops) Each hop named by the ticket is described inside
#    the corresponding sub-section, on one line or that line plus the next:
#      - sender.rs:435 — the record copy of the echo field. The block must
#        contain "rx_lost", a width/type marker (u32, 32-bit, 4-byte), a
#        kind/unit marker (packets, count, dimensionless, unitless, "no
#        unit"/"no conversion"/"unchanged"), a source marker (header, echo,
#        datagram, reply, returned) and the exact tag "[code]".
#      - stats.rs:324 — the statistics storage. The block must contain
#        "lost_fwd", a width/type marker (u64, 64-bit, 8-byte), a kind/unit
#        marker as above and "[code]".
#      - tui.rs:107-114 — the one-line display. Any pointer tui.rs:107 ..
#        tui.rs:114 is accepted. The block must contain "lost_fwd", a
#        width/type marker (u64, 64-bit, 8-byte), a kind/unit marker, a
#        display marker (one-line, display, print, status, format), a branch
#        marker (sent, lost_total, received) and "[code]".
#    The unit marker is deliberately loose: these three hops carry a packet
#    count, so "no unit conversion" is a correct description; the check only
#    requires that the question is answered.
#  * (Context fact, input to 002-08) The stats.rs note must say that the
#    single stored value is the latest record's and is shared by all return
#    streams, with no per-stream separation. A line naming "lost_fwd" or
#    "rx_lost" together with "stream(s)", or that line plus the next, must
#    carry: a file pointer, an evidence tag, a latest-record marker (latest,
#    last, newest, overwrites, replaces, wins, supersedes) and a no-per-stream
#    marker ("no per-stream", "not per-stream", "across all", "without
#    per-stream/separat...", "no separat...", pooled, merged, single/one
#    counter/value, "regardless of the stream", "indifferent to the stream").
#  * (AC 3, TX pacing lateness) A line, or that line plus the next, anywhere
#    in the three sub-sections that mentions "TX pacing lateness" or "tx_late"
#    must carry a sender.rs:<digits> pointer (the computation site — the label
#    in stats.rs is not the computation), a time unit (ns, nanoseconds, µs,
#    us, microseconds, ms, milliseconds) and an evidence tag.
#  * (DoD, evidence) Every line carrying a "file.rs:<digits>" pointer must
#    carry an evidence tag on itself or the next line. A line raising an open
#    item ("open item", "open question", "TODO", "TBD", "unresolved", unless
#    negated) must name an owner and a falsifier. "The frozen POC is not
#    modified" is not checkable from the report; checks/002-01.sh and
#    checks/003-01.sh guard the frozen tree.
#  * (Section marker) Section (d) keeps exactly one "verdict:" line and it
#    still says "pending". The verdict belongs to the whole section; the
#    reading tickets do not close it (002-13 does).
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
  printf '%s\n' "$1" | grep -Eq '\[(code|measured|kernel-doc|memory|decision|standard)\]'
}

has_ptr() {
  printf '%s\n' "$1" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+'
}

# Count non-empty lines in a string.
count_lines() {
  printf '%s\n' "$1" | grep -c '[^[:space:]]' || true
}

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-04 does not exist yet.\n'
  exit 1
fi

# ---------------------------------------------------------------------------
# Locate section (d) and its (e) boundary
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
d_line=$(sed -n "${d_no}p" "$REPORT")
d_hashes=$(printf '%s' "$d_line" | sed -E 's/^(#+).*/\1/')
d_lvl=${#d_hashes}
ok "section (d) found at line $d_no (level $d_lvl), section (e) at line $e_no"

# Headings between (d) and (e) that name the wanted file and do NOT also name
# one of the two sibling files. Umbrella headings naming several files are
# containers, not sub-sections, and are skipped.
# Prints "<line> <level>" per match.
find_file_headings() {
  local want="$1" i line low names tok hashes
  i=$((d_no + 1))
  while [ "$i" -lt "$e_no" ]; do
    line=$(sed -n "${i}p" "$REPORT")
    if printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
      low=$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')
      names=0
      for tok in sender stats tui; do
        case "$low" in *"$tok.rs"*) names=$((names + 1)) ;; esac
      done
      if [ "$names" -eq 1 ]; then
        case "$low" in
          *"$want.rs"*)
            hashes=$(printf '%s' "$line" | sed -E 's/^(#+).*/\1/')
            printf '%s %s\n' "$i" "${#hashes}"
            ;;
        esac
      fi
    fi
    i=$((i + 1))
  done
}

S_LIST=$(find_file_headings sender)
ST_LIST=$(find_file_headings stats)
U_LIST=$(find_file_headings tui)

S_COUNT=$(printf '%s\n' "$S_LIST" | grep -c . || true)
ST_COUNT=$(printf '%s\n' "$ST_LIST" | grep -c . || true)
U_COUNT=$(printf '%s\n' "$U_LIST" | grep -c . || true)

missing_any=0
if [ "$S_COUNT" -eq 0 ]; then
  bad "no sub-section heading under (d) naming only sender.rs"
  missing_any=1
fi
if [ "$ST_COUNT" -eq 0 ]; then
  bad "no sub-section heading under (d) naming only stats.rs"
  missing_any=1
fi
if [ "$U_COUNT" -eq 0 ]; then
  bad "no sub-section heading under (d) naming only tui.rs"
  missing_any=1
fi
if [ "$missing_any" -eq 1 ]; then
  printf '\n%d check(s) failed; the sender.rs/stats.rs/tui.rs reading notes do not exist yet.\n' "$fail"
  exit 1
fi

if [ "$S_COUNT" -gt 1 ]; then
  bad "$S_COUNT headings under (d) name only sender.rs; one sub-section per file is required"
fi
if [ "$ST_COUNT" -gt 1 ]; then
  bad "$ST_COUNT headings under (d) name only stats.rs; one sub-section per file is required"
fi
if [ "$U_COUNT" -gt 1 ]; then
  bad "$U_COUNT headings under (d) name only tui.rs; one sub-section per file is required"
fi

S_START=$(printf '%s\n' "$S_LIST" | awk 'NR==1 {print $1}')
S_LVL=$(printf '%s\n' "$S_LIST" | awk 'NR==1 {print $2}')
ST_START=$(printf '%s\n' "$ST_LIST" | awk 'NR==1 {print $1}')
ST_LVL=$(printf '%s\n' "$ST_LIST" | awk 'NR==1 {print $2}')
U_START=$(printf '%s\n' "$U_LIST" | awk 'NR==1 {print $1}')
U_LVL=$(printf '%s\n' "$U_LIST" | awk 'NR==1 {print $2}')

if [ "$S_START" = "$ST_START" ] || [ "$S_START" = "$U_START" ] || [ "$ST_START" = "$U_START" ]; then
  bad "two of the sender.rs/stats.rs/tui.rs headings are the same heading; three distinct sub-sections are required"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi

if [ "$S_LVL" -le "$d_lvl" ] || [ "$ST_LVL" -le "$d_lvl" ] || [ "$U_LVL" -le "$d_lvl" ]; then
  bad "a sub-section heading is not deeper than section (d); the notes would fall outside (d)"
fi

# End of a sub-section: next heading of the same or higher level, or (e)-1.
section_end() {
  local start="$1" lvl="$2" j line hashes
  j=$((start + 1))
  while [ "$j" -lt "$e_no" ]; do
    line=$(sed -n "${j}p" "$REPORT")
    if printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
      hashes=$(printf '%s' "$line" | sed -E 's/^(#+).*/\1/')
      if [ "${#hashes}" -le "$lvl" ]; then
        printf '%s\n' "$((j - 1))"
        return 0
      fi
    fi
    j=$((j + 1))
  done
  printf '%s\n' "$((e_no - 1))"
}

S_END=$(section_end "$S_START" "$S_LVL")
ST_END=$(section_end "$ST_START" "$ST_LVL")
U_END=$(section_end "$U_START" "$U_LVL")

# The three sub-sections must be disjoint (none may contain another). With
# equal-level headings they always are; this catches a deeper heading nested
# under a shallower one.
check_disjoint() {
  local an="$1" as_="$2" ae="$3" bn="$4" bs="$5" be="$6" nested=0
  if [ "$as_" -lt "$bs" ] && [ "$bs" -le "$ae" ]; then
    bad "$bn sub-section (line $bs) lies inside the $an sub-section (lines $as_-$ae); sub-sections must be disjoint"
    nested=1
  fi
  if [ "$bs" -lt "$as_" ] && [ "$as_" -le "$be" ]; then
    bad "$an sub-section (line $as_) lies inside the $bn sub-section (lines $bs-$be); sub-sections must be disjoint"
    nested=1
  fi
  return 0
}
check_disjoint "sender.rs" "$S_START" "$S_END" "stats.rs" "$ST_START" "$ST_END"
check_disjoint "sender.rs" "$S_START" "$S_END" "tui.rs" "$U_START" "$U_END"
check_disjoint "stats.rs" "$ST_START" "$ST_END" "tui.rs" "$U_START" "$U_END"

ok "sender.rs sub-section: lines $S_START-$S_END"
ok "stats.rs sub-section: lines $ST_START-$ST_END"
ok "tui.rs sub-section: lines $U_START-$U_END"

# ---------------------------------------------------------------------------
# AC 1 — size and pointers, per sub-section
# ---------------------------------------------------------------------------

check_size_pointers() {
  local label="$1" start="$2" end="$3" own="$4" sec nonempty ptrs nptr ownptr nown
  sec=$(line_range "$REPORT" "$start" "$end")
  nonempty=$(count_lines "$sec")
  if [ "$nonempty" -le 30 ]; then
    ok "$label: compact ($nonempty non-empty lines, half-page proxy limit 30)"
  else
    bad "$label: too long ($nonempty non-empty lines, half-page proxy limit 30)"
  fi

  ptrs=$(printf '%s\n' "$sec" | grep -oE '[A-Za-z0-9_./-]+\.rs:[0-9]+' | sort -u)
  nptr=$(printf '%s\n' "$ptrs" | grep -c . || true)
  if [ "$nptr" -ge 3 ]; then
    ok "$label: $nptr distinct file:line pointers (need >= 3)"
  else
    bad "$label: $nptr distinct file:line pointer(s), need at least 3"
  fi

  ownptr=$(printf '%s\n' "$sec" | grep -oE "${own}\\.rs:[0-9]+" | sort -u)
  nown=$(printf '%s\n' "$ownptr" | grep -c . || true)
  if [ "$nown" -ge 2 ]; then
    ok "$label: $nown distinct $own.rs pointers (need >= 2 to its own file)"
  else
    bad "$label: $nown distinct $own.rs pointer(s), need at least 2 to its own file"
  fi
}

check_size_pointers "sender.rs" "$S_START" "$S_END" "sender"
check_size_pointers "stats.rs" "$ST_START" "$ST_END" "stats"
check_size_pointers "tui.rs" "$U_START" "$U_END" "tui"

# ---------------------------------------------------------------------------
# AC 2 — the three hops: field, type/width, unit, [code]
# ---------------------------------------------------------------------------

type32_re='(^|[^[:alnum:]_])(u32|32[ -]?bit|4[ -]?byte)([^[:alnum:]_]|$)'
type64_re='(^|[^[:alnum:]_])(u64|64[ -]?bit|8[ -]?byte)([^[:alnum:]_]|$)'
unit_re='(^|[^[:alnum:]_])(packets?|counts?|dimensionless|unitless)([^[:alnum:]_]|$)|no (unit|conversion|scaling)|unchanged'

# hop_check <label> <start> <end> <pointer-ERE> <required-ERE>...
# A hop is described by the first pointer-bearing line whose text, plus the
# immediately following line, matches every required ERE.
hop_check() {
  local label="$1" start="$2" end="$3" ptr_re="$4"
  shift 4
  local k to block line missing first="" missing_first="" found=0 re
  for ((k = start; k <= end; k++)); do
    line=$(sed -n "${k}p" "$REPORT")
    if printf '%s\n' "$line" | grep -Eq "$ptr_re"; then
      found=1
      [ -z "$first" ] && first=$k
      to=$((k + 1))
      [ "$to" -gt "$end" ] && to=$end
      block=$(line_range "$REPORT" "$k" "$to")
      missing=""
      for re in "$@"; do
        if ! printf '%s\n' "$block" | grep -Eqi "$re"; then
          missing="$missing
    - /$re/"
        fi
      done
      if [ -z "$missing" ]; then
        ok "$label described (line $k) with field, width, unit and [code]"
        return 0
      fi
      [ -z "$missing_first" ] && missing_first="$missing"
    fi
  done
  if [ "$found" -eq 0 ]; then
    bad "$label is not mentioned: no line in its sub-section carries a pointer matching /$ptr_re/"
  else
    bad "$label is mentioned at line $first but the description lacks:$missing_first"
  fi
}

hop_check "sender.rs:435 hop" "$S_START" "$S_END" 'sender\.rs:435([^0-9]|$)' \
  'rx_lost' \
  "$type32_re" \
  "$unit_re" \
  'header|echo|datagram|reply|returned' \
  '\[code\]'

hop_check "stats.rs:324 hop" "$ST_START" "$ST_END" 'stats\.rs:324([^0-9]|$)' \
  'lost_fwd' \
  "$type64_re" \
  "$unit_re" \
  '\[code\]'

hop_check "tui.rs:107-114 hop" "$U_START" "$U_END" 'tui\.rs:(10[7-9]|11[0-4])([^0-9]|$)' \
  'lost_fwd' \
  "$type64_re" \
  "$unit_re" \
  'one[- ]line|display|print|status|format' \
  'sent|lost_total|received' \
  '\[code\]'

# ---------------------------------------------------------------------------
# Context fact — the loss value is the latest record's, pooled across streams
# ---------------------------------------------------------------------------

latest_re='(^|[^[:alnum:]_])(latest|last|newest|overwrit(e|es|ten|ing)|replac(e|es|ed|ing)|wins?|supersed(e|es|ed))([^[:alnum:]_]|$)'
pooled_re='no per[ -]?stream|not per[ -]?stream|across all|without (per[ -]?stream|separat)|no (per[ -]?stream )?separat|pooled|merged|single (counter|value|number|lost)|one (counter|value|number)|regardless of (the )?stream|indifferent to (the )?stream'

k=$ST_START
stream_first=""
stream_missing=""
stream_seen=0
stream_ok=0
while [ "$k" -le "$ST_END" ]; do
  line=$(sed -n "${k}p" "$REPORT")
  if printf '%s\n' "$line" | grep -Eqi 'lost_fwd|rx_lost' \
     && printf '%s\n' "$line" | grep -Eqi 'streams?'; then
    stream_seen=1
    [ -z "$stream_first" ] && stream_first=$k
    to=$((k + 1))
    [ "$to" -gt "$ST_END" ] && to=$ST_END
    block=$(line_range "$REPORT" "$k" "$to")
    missing=""
    has_ptr "$block" || missing="$missing pointer"
    has_tag "$block" || missing="$missing evidence-tag"
    printf '%s\n' "$block" | grep -Eqi "$latest_re" || missing="$missing latest-record-marker"
    printf '%s\n' "$block" | grep -Eqi "$pooled_re" || missing="$missing no-per-stream-marker"
    if [ -z "$missing" ]; then
      ok "stats.rs note states that lost_fwd is the latest record's value pooled across streams (line $k)"
      stream_ok=1
      break
    fi
    [ -z "$stream_missing" ] && stream_missing="$missing"
  fi
  k=$((k + 1))
done
if [ "$stream_ok" -eq 0 ]; then
  if [ "$stream_seen" -eq 0 ]; then
    bad "stats.rs note does not say what the loss display takes across return streams (need a line naming lost_fwd/rx_lost together with stream(s))"
  else
    bad "stats.rs note names lost_fwd across streams (line $stream_first) but lacks:$stream_missing"
  fi
fi

# ---------------------------------------------------------------------------
# AC 3 — site and unit of "TX pacing lateness"
# ---------------------------------------------------------------------------

tx_unit_re='(^|[^[:alnum:]_])(ns|nanoseconds?|µs|us|microseconds?|ms|milliseconds?)([^[:alnum:]_]|$)'

tx_seen=0
tx_first=""
tx_missing=""
tx_ok=0
for pair in "$S_START:$S_END" "$ST_START:$ST_END" "$U_START:$U_END"; do
  start=${pair%%:*}
  end=${pair##*:}
  k=$start
  while [ "$k" -le "$end" ]; do
    line=$(sed -n "${k}p" "$REPORT")
    if printf '%s\n' "$line" | grep -Eqi 'tx pacing lateness|tx_late'; then
      tx_seen=1
      [ -z "$tx_first" ] && tx_first=$k
      to=$((k + 1))
      [ "$to" -gt "$end" ] && to=$end
      block=$(line_range "$REPORT" "$k" "$to")
      missing=""
      printf '%s\n' "$block" | grep -Eq 'sender\.rs:[0-9]+' || missing="$missing sender.rs-computation-pointer"
      printf '%s\n' "$block" | grep -Eqi "$tx_unit_re" || missing="$missing time-unit"
      has_tag "$block" || missing="$missing evidence-tag"
      if [ -z "$missing" ]; then
        ok "TX pacing lateness computation site (sender.rs) and unit given (line $k)"
        tx_ok=1
        break 2
      fi
      [ -z "$tx_missing" ] && tx_missing="$missing"
    fi
    k=$((k + 1))
  done
done
if [ "$tx_ok" -eq 0 ]; then
  if [ "$tx_seen" -eq 0 ]; then
    bad "no line in the three notes mentions 'TX pacing lateness' or tx_late"
  else
    bad "TX pacing lateness is mentioned (line $tx_first) but the statement lacks:$tx_missing"
  fi
fi

# ---------------------------------------------------------------------------
# DoD — evidence tags on pointer-bearing lines, and owned open items
# ---------------------------------------------------------------------------

check_tag_discipline() {
  local label="$1" start="$2" end="$3" k to block
  k=$start
  while [ "$k" -le "$end" ]; do
    if has_ptr "$(sed -n "${k}p" "$REPORT")"; then
      to=$((k + 1))
      [ "$to" -gt "$end" ] && to=$end
      block=$(line_range "$REPORT" "$k" "$to")
      if ! has_tag "$block"; then
        bad "$label line $k carries a file:line pointer but no evidence tag"
      fi
    fi
    k=$((k + 1))
  done
}

check_tag_discipline "sender.rs" "$S_START" "$S_END"
check_tag_discipline "stats.rs" "$ST_START" "$ST_END"
check_tag_discipline "tui.rs" "$U_START" "$U_END"

check_open_items() {
  local label="$1" start="$2" end="$3" k to block low
  k=$start
  while [ "$k" -le "$end" ]; do
    low=$(sed -n "${k}p" "$REPORT" | tr '[:upper:]' '[:lower:]')
    if printf '%s\n' "$low" | grep -Eq 'open (item|question)|\btodo\b|\btbd\b|\bunresolved\b' \
       && ! printf '%s\n' "$low" | grep -Eq '(^|[^a-z])(no|none|not)[^.]{0,30}open'; then
      to=$((k + 1))
      [ "$to" -gt "$end" ] && to=$end
      block=$(line_range "$REPORT" "$k" "$to")
      if ! printf '%s\n' "$block" | grep -qi 'owner' \
         || ! printf '%s\n' "$block" | grep -qi 'falsifier'; then
        bad "$label line $k raises an open item without an owner and a falsifier"
      fi
    fi
    k=$((k + 1))
  done
}

check_open_items "sender.rs" "$S_START" "$S_END"
check_open_items "stats.rs" "$ST_START" "$ST_END"
check_open_items "tui.rs" "$U_START" "$U_END"

# ---------------------------------------------------------------------------
# Section marker — (d) still carries exactly one pending verdict line
# ---------------------------------------------------------------------------

verdict_count=$(sed -n "$((d_no + 1)),$((e_no - 1))p" "$REPORT" \
  | grep -Eci '(^|[^[:alnum:]])\*{0,2}verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "section (d) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "section (d) has $verdict_count 'verdict:' lines; exactly one section-level verdict is required"
else
  if sed -n "$((d_no + 1)),$((e_no - 1))p" "$REPORT" | grep -Eqi '(^|[^[:alnum:]])\*{0,2}verdict:[^A-Za-z]*pending'; then
    ok "section (d) still carries its one pending verdict marker"
  else
    bad "section (d)'s verdict marker no longer says 'pending'"
  fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
if [ "$fail" -gt 0 ]; then
  exit 1
fi
