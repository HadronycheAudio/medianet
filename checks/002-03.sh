#!/usr/bin/env bash
# checks/002-03.sh — checks for ticket 002-03
# "Reading notes: transport.rs and control.rs".
#
# The deliverable is two sub-sections of section (d) Reading notes of
#   doc/design/reports/archaeology/REPORT-0b.md
# written by the author after a read-only reading of the frozen POC at
# /home/dorian/workspace/hadropoc. This script reads the report only; it never
# writes anywhere and never touches the POC tree.
#
# Conventions required by this check, spelled out so the author can comply
# exactly. They make the prose machine-checkable without fixing its wording.
#
#  * (AC 1, layout) Under the "## (d) ..." heading and before "## (e) ..."
#    there is exactly one heading whose text contains "transport.rs" and exactly
#    one containing "control.rs" (case-insensitive), at a deeper level than
#    (d), and they are two different headings. Each sub-section runs from its
#    heading to the next heading of the same or higher level (or to (e)).
#    "About half a page" is proxied by at most 30 non-empty lines per
#    sub-section; sibling ticket 002-02 allows 45 non-empty lines for "about
#    one page", so 30 is half a page plus slack. Each sub-section must carry at
#    least 3 DISTINCT "file.rs:<digits>" pointers, at least 2 of them to its
#    own file (transport.rs / control.rs).
#  * (AC 2, four facts) The four facts of the ticket's Context must each be
#    stated inside one of the two sub-sections, as a line (or that line plus
#    the immediately following one) which either
#      - contains an explicit not-in-files marker: "not in these files", "not
#        in the two files", "not in either file", "not found in these files",
#        "absent from these files", or
#      - is answered: contains a "file.rs:<digits>" pointer and the keywords
#        checked below for that fact.
#    A "not in these files" marker also satisfies the fact; AC 2 allows it.
#    Facts, anchors and required content:
#      1 who acts as transport reference — anchor "who acts ... reference",
#        "transport reference", "reference role/holder", or "sender/synth ...
#        is ... reference"; content the word "sender" (not merely a
#        "sender.rs" pointer);
#      2 how ctx.is_reference() is decided — anchor "is_reference", content
#        "module.rs" / "elected" / "election" / "routes";
#      3 listen-before-claim delay, its value and what it guards — anchor
#        "listen" or "claim", content "1.5 s" (or 1500 ms / 1 500 000 000 ns)
#        plus one of before/hear/guard/already/join/wait;
#      4 how transport datagrams are sent and on which output — anchor
#        "transport", "datagram" or "events", content "events" plus
#        output/connection/port/send.
#  * (AC 3, units) A line that quotes a number together with a quantity cue
#    (constant name, period, delay, rate, size, length, budget, window, lead,
#    expiry, election, listen, repeat, room, capacity, duration, deadline,
#    payload, "every", or a known dimensioned constant such as STATE_LEN, ROOM,
#    CYCLE_SECS, DIVIDER, SLOTS, *_NS, *_MS, *_SECS) is inspected number by
#    number: a unit must follow the number within 24 characters. Accepted units
#    include s, ms, µs, ns, seconds, Hz, kHz, bytes, bits, samples, ticks,
#    frames, fps, ppm, %, Q32.32/Q1.31/Q0.32, entries, slots, channels, slices,
#    blocks. Exemptions: "file.rs:NNN" pointers, "C-30" ids, Q-formats
#    and time codes are stripped first; a number that is the right-hand side of
#    an assignment to a non-dimensioned ALL_CAPS identifier (a kind code such
#    as T_STATE = 0, or VOLUME = 0) needs no unit; a number followed by an
#    arithmetic operator is a factor of an expression. This is a proxy: it
#    cannot judge whether number and unit denote the same quantity.
#  * (AC 4, C-30..C-33) Each of C-30, C-31, C-32 and C-33 must be accounted
#    for inside the two sub-sections (a range "C-30 to C-33", "C-30..C-33" or
#    "C-30-C-33" accounts for the four) by a line, or line plus next line, that
#    either says "not in these files" (or one of the AC 2 variants) or marks it
#    "match"/"matches" or "differs"/"differ" WITH a "file.rs:<digits>" pointer.
#    The ticket only constrains IDs found in the two files; requiring an
#    explicit disposition for all four is a structural proxy that prevents the
#    acceptance criterion from being vacuously satisfied by silence (in the
#    POC these timers live in module.rs, not in transport.rs or control.rs).
#    Any disposition containing "differ" must also name ticket 002-13.
#  * (DoD, evidence) Every fact statement and every C-30..C-33 disposition
#    must carry one of [code] [measured] [kernel-doc] [memory] [decision].
#    Every line carrying a "file.rs:<digits>" pointer must carry an evidence
#    tag on itself or the next line. A line raising an open item ("open item",
#    "open question", "TODO", "TBD", unless negated) must name an owner and a
#    falsifier. "The frozen POC is not modified" is not checkable from the
#    report; it is covered by checks/002-01.sh and checks/003-01.sh.
#  * (Section marker) Section (d) keeps exactly one "verdict:" line and it
#    still says "pending" (the verdict belongs to the whole section, not to one
#    file's notes, and the reading tickets are not the close-out ticket).
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
  printf '%s\n' "$1" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+'
}

has_marker() {
  printf '%s\n' "$1" | tr '[:upper:]' '[:lower:]' \
    | grep -Eq 'not in (these|the two|either) files?|not found in (these|either)|absent from (these|both)'
}

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-03 does not exist yet.\n'
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

# List headings between (d) and (e) whose text matches the ERE.
# Prints "<line> <level>" per match.
find_headings() {
  local re="$1" i line low hashes lvl
  i=$((d_no + 1))
  while [ "$i" -lt "$e_no" ]; do
    line=$(sed -n "${i}p" "$REPORT")
    if printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
      low=$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')
      if printf '%s\n' "$low" | grep -Eq "$re"; then
        hashes=$(printf '%s' "$line" | sed -E 's/^(#+).*/\1/')
        lvl=${#hashes}
        printf '%s %s\n' "$i" "$lvl"
      fi
    fi
    i=$((i + 1))
  done
}

t_list=$(find_headings 'transport\.rs')
c_list=$(find_headings 'control\.rs')
t_count=$(printf '%s\n' "$t_list" | grep -c . || true)
c_count=$(printf '%s\n' "$c_list" | grep -c . || true)

if [ "$t_count" -eq 0 ] || [ "$c_count" -eq 0 ]; then
  [ "$t_count" -eq 0 ] && bad "no sub-section heading under (d) naming transport.rs"
  [ "$c_count" -eq 0 ] && bad "no sub-section heading under (d) naming control.rs"
  printf '\n%d check(s) failed; the transport.rs/control.rs reading notes do not exist yet.\n' "$fail"
  exit 1
fi

if [ "$t_count" -gt 1 ]; then
  bad "$t_count headings under (d) name transport.rs; one sub-section per file is required"
fi
if [ "$c_count" -gt 1 ]; then
  bad "$c_count headings under (d) name control.rs; one sub-section per file is required"
fi

T_START=$(printf '%s\n' "$t_list" | awk 'NR==1 {print $1}')
T_LVL=$(printf '%s\n' "$t_list" | awk 'NR==1 {print $2}')
C_START=$(printf '%s\n' "$c_list" | awk 'NR==1 {print $1}')
C_LVL=$(printf '%s\n' "$c_list" | awk 'NR==1 {print $2}')

if [ "$T_START" = "$C_START" ]; then
  bad "the transport.rs and control.rs headings are the same heading; two distinct sub-sections are required"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi

if [ "$T_LVL" -le "$d_lvl" ] || [ "$C_LVL" -le "$d_lvl" ]; then
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

T_END=$(section_end "$T_START" "$T_LVL")
C_END=$(section_end "$C_START" "$C_LVL")

ok "transport.rs sub-section: lines $T_START-$T_END"
ok "control.rs sub-section: lines $C_START-$C_END"

T_SEC=$(line_range "$REPORT" "$T_START" "$T_END")
C_SEC=$(line_range "$REPORT" "$C_START" "$C_END")

# ---------------------------------------------------------------------------
# AC 1 — size and pointers, per sub-section
# ---------------------------------------------------------------------------

check_size_pointers() {
  local label="$1" start="$2" end="$3" own="$4" sec nonempty ptrs nptr ownptr nown
  sec=$(line_range "$REPORT" "$start" "$end")
  nonempty=$(printf '%s\n' "$sec" | grep -c '[^[:space:]]' || true)
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

check_size_pointers "transport.rs" "$T_START" "$T_END" "transport"
check_size_pointers "control.rs" "$C_START" "$C_END" "control"

# ---------------------------------------------------------------------------
# AC 2 — the four facts, answered or marked "not in these files"
# ---------------------------------------------------------------------------

# fact_scan <anchor> <required ERE>... : sets FACT_STATE to one of
#   answer | notin | answer_notag | notin_notag | none
# and FACT_LINE to the matching report line number.
FACT_STATE=none
FACT_LINE=""

fact_scan() {
  local anchor="$1"; shift
  local pair start end k to block re allok
  FACT_STATE=none
  FACT_LINE=""
  for pair in "$T_START:$T_END" "$C_START:$C_END"; do
    start=${pair%%:*}
    end=${pair##*:}
    k=$start
    while [ "$k" -le "$end" ]; do
      if sed -n "${k}p" "$REPORT" | grep -Eqi "$anchor"; then
        to=$((k + 1))
        [ "$to" -gt "$end" ] && to=$end
        block=$(line_range "$REPORT" "$k" "$to")
        if has_ptr "$block"; then
          allok=1
          for re in "$@"; do
            if ! printf '%s\n' "$block" | grep -Eqi "$re"; then allok=0; break; fi
          done
          if [ "$allok" -eq 1 ]; then
            if has_tag "$block"; then
              FACT_STATE=answer; FACT_LINE=$k; return 0
            fi
            FACT_STATE=answer_notag; FACT_LINE=$k
            k=$((k + 1)); continue
          fi
        fi
        if has_marker "$block"; then
          if has_tag "$block"; then
            FACT_STATE=notin; FACT_LINE=$k; return 0
          fi
          FACT_STATE=notin_notag; FACT_LINE=$k
        fi
      fi
      k=$((k + 1))
    done
  done
  return 1
}

fact_result() {
  local desc="$1"
  case "$FACT_STATE" in
    answer)
      ok "fact answered with pointer and [code]: $desc (line $FACT_LINE)" ;;
    notin)
      ok "fact explicitly marked not in these files, with [code]: $desc (line $FACT_LINE)" ;;
    answer_notag)
      bad "fact $desc is answered (line $FACT_LINE) but carries no evidence tag" ;;
    notin_notag)
      bad "fact $desc is marked not in these files (line $FACT_LINE) but carries no evidence tag" ;;
    *)
      bad "fact not stated, not answered with a pointer, and not marked 'not in these files': $desc" ;;
  esac
}

# 1 — who acts as transport reference.
fact_scan 'who acts[^.]*reference|transport[ -]reference|reference (role|holder)|(sender|synth)[^.]*(is|acts as|serves as|defaults? to)[^.]*reference|reference[^.]{0,30}(is|equals)[^.]{0,20}(sender|synth)([^[:alnum:]_.]|$)' \
  '(^|[^[:alnum:]_])sender([^[:alnum:]_.]|$)'
fact_result "who acts as transport reference"

# 2 — how ctx.is_reference() is decided.
fact_scan 'is_reference' 'module\.rs|elected|election|routes'
fact_result "how ctx.is_reference() is decided"

# 3 — listen-before-claim: value, unit and what it guards.
fact_scan 'listen|claim' \
  '(1[.,]5[^0-9]{0,3}(s|sec|secs|second|seconds))|(1500[[:space:]]*ms)|(1[ _]?500[ _]?000[ _]?000[[:space:]]*ns)' \
  'before|hear|guard|already|join|wait'
fact_result "listen-before-claim delay, its value and what it guards"

# 4 — transport datagrams and the output connection.
fact_scan 'transport|datagram|events' 'events' 'output|connection|port|\bsends?\b|\bsent\b|\bsending\b'
fact_result "how transport datagrams are sent and on which output connection"

# ---------------------------------------------------------------------------
# AC 3 — every quoted constant has a value and a unit
# ---------------------------------------------------------------------------

quantity_re='(^|[^[:alnum:]_])(constant|const|timeout|period|delay|interval|rate|size|length|budget|window|lead|expiry|expir|election|listen|repeat|room|capacity|duration|deadline|payload|every|per second|bytes|samples?|ticks?|frames?)([^[:alnum:]_]|$)|_(LEN|NS|MS|SECS|BYTES|SLOTS)([^[:alnum:]_]|$)|(^|[^[:alnum:]_])(STATE_LEN|PULSE_LEN|REPORT_LEN|ROOM|CYCLE_SECS|DIVIDER|SLOTS|LISTEN_NS|HELLO_NS|EXPIRE_NS|ELECTION_NS)([^[:alnum:]_]|$)'

unit_re='(^|[^[:alnum:]_])(ms|milliseconds?|µs|us|microseconds?|ns|nanoseconds?|s|sec|secs|seconds?|minutes?|hours?|Hz|kHz|MHz|bytes?|KiB|KB|bits?|samples?|ticks?|frames?|fps|ppm|%|Q32\.32|Q1\.31|Q0\.32|slots?|entries|channels?|controllers?|slices?|blocks?)([^[:alnum:]_]|$)|per second'

# Emit one line per numeric token that has no unit after it. Pointers,
# C-ids, Q-formats and time codes are stripped first; a number that is the
# right-hand side of an assignment to a non-dimensioned ALL_CAPS identifier
# (a kind code such as T_STATE = 0, or VOLUME = 0) is exempt, and a number
# followed by an arithmetic operator is a factor, not a quoted constant.
unit_offenders() {
  local line="$1"
  printf '%s\n' "$line" | awk -v cue="$quantity_re" -v unit="$unit_re" '
    {
      low = tolower($0)
      if (low !~ cue) next
      s = $0
      gsub(/[A-Za-z0-9_.\/-]+\.rs:[0-9]+(-[0-9]+)?/, "", s)
      gsub(/C-[0-9]+/, "", s)
      gsub(/[0-9][0-9][0-9]-[0-9][0-9]/, "", s)
      gsub(/Q[0-9]+\.[0-9]+/, "", s)
      gsub(/[0-9][0-9]?:[0-9][0-9]:[0-9][0-9][:;][0-9][0-9]/, "", s)
      gsub(/\^[ ]*[0-9]+/, "", s)
      gsub(/<<[ ]*[0-9]+/, "", s)
      while (match(s, /[0-9]+([.,][0-9]+)?/)) {
        nstart = RSTART; nlen = RLENGTH; adv = nstart + nlen
        num = substr(s, nstart, nlen)
        before = substr(s, 1, nstart - 1)
        after = substr(s, adv, 24)
        tail = after
        sub(/^[ \t]*/, "", tail)
        op = substr(tail, 1, 1)
        if (op ~ /[+*\/xX<×-]/) { s = substr(s, adv); continue }
        if (tolower(after) ~ unit) { s = substr(s, adv); continue }
        prev = before
        sub(/[ \t]+$/, "", prev)
        prevch = substr(prev, length(prev), 1)
        if (prevch ~ /[\^+*\/<]/) { s = substr(s, adv); continue }
        # Dimensionless identifier assignment: `VOLUME` = 0, T_STATE = 0.
        if (match(before, /([A-Z][A-Z0-9_]{2,})[`*_]*[ \t]*=[ \t]*$/)) {
          ident = substr(before, RSTART, RLENGTH)
          sub(/[`*_]*[ \t]*=[ \t]*$/, "", ident)
          if (ident !~ /(_LEN|_NS|_MS|_SECS|_BYTES|_SLOTS)$/ \
              && ident != "ROOM" && ident != "CYCLE_SECS" \
              && ident != "DIVIDER" && ident != "SLOTS" && ident != "ONE") {
            s = substr(s, adv); continue
          }
        }
        printf "number %s has no unit near: %s\n", num, after
        s = substr(s, adv)
      }
    }'
}

check_units() {
  local label="$1" start="$2" end="$3" k line offenders nbad=0 nlines=0
  k=$start
  while [ "$k" -le "$end" ]; do
    line=$(sed -n "${k}p" "$REPORT")
    if [ -z "$(printf '%s' "$line" | tr -d '[:space:]')" ] \
       || printf '%s\n' "$line" | grep -Eq '^#+[[:space:]]'; then
      k=$((k + 1)); continue
    fi
    if printf '%s\n' "$line" | grep -Eq '[0-9]' \
       && printf '%s\n' "$line" | grep -Eqi "$quantity_re"; then
      nlines=$((nlines + 1))
      offenders=$(unit_offenders "$line")
      if [ -n "$offenders" ]; then
        nbad=$((nbad + 1))
        while IFS= read -r o; do
          bad "$label line $k: $o"
        done <<EOF
$offenders
EOF
      fi
    fi
    k=$((k + 1))
  done
  if [ "$nbad" -eq 0 ]; then
    ok "$label: all $nlines quantity-bearing lines give a unit for every quoted number"
  else
    bad "$label: $nbad of $nlines quantity-bearing lines lack a unit for a quoted number"
  fi
}

check_units "transport.rs" "$T_START" "$T_END"
check_units "control.rs" "$C_START" "$C_END"

# ---------------------------------------------------------------------------
# AC 4 — C-30..C-33 accounted for; matches/differs with a pointer;
#        anything that differs flagged for 002-13.
# ---------------------------------------------------------------------------

# line_covers_id <line> <id-number>
line_covers_id() {
  local line="$1" id="$2" ranges r a b
  if printf '%s\n' "$line" | grep -Eq "(^|[^A-Za-z0-9])C-${id}([^0-9]|$)"; then
    return 0
  fi
  ranges=$(printf '%s\n' "$line" \
    | grep -oE 'C-[0-9]+[[:space:]]*(to|through|\.\.|-)[[:space:]]*C-[0-9]+' || true)
  [ -n "$ranges" ] || return 1
  while IFS= read -r r; do
    [ -n "$r" ] || continue
    a=$(printf '%s' "$r" | sed -E 's/^C-([0-9]+).*/\1/')
    b=$(printf '%s' "$r" | sed -E 's/.*C-([0-9]+)$/\1/')
    if [ "$a" -le "$id" ] 2>/dev/null && [ "$b" -ge "$id" ] 2>/dev/null; then
      return 0
    fi
  done <<EOF
$ranges
EOF
  return 1
}

check_const_id() {
  local id="$1" pair start end k to block state=unaccounted accounted=0 first=""
  local differ_re='(^|[^[:alnum:]_])differ(s|ed|ing)?([^[:alnum:]_]|$)'
  local verdict_re='(^|[^[:alnum:]_])(match|matches|matched|matching|differ|differs|differed|differing)([^[:alnum:]_]|$)'
  for pair in "$T_START:$T_END" "$C_START:$C_END"; do
    start=${pair%%:*}
    end=${pair##*:}
    k=$start
    while [ "$k" -le "$end" ]; do
      if line_covers_id "$(sed -n "${k}p" "$REPORT")" "$id"; then
        to=$((k + 1))
        [ "$to" -gt "$end" ] && to=$end
        block=$(line_range "$REPORT" "$k" "$to")
        # A "differs" claim must be flagged for 002-13 wherever it appears,
        # even if another line also accounts for the same id as matching.
        if printf '%s\n' "$block" | grep -Eqi "$differ_re" \
           && ! printf '%s\n' "$block" | grep -q '002-13'; then
          bad "C-$id differs but is not flagged for 002-13 (line $k)"
          return 0
        fi
        if has_marker "$block"; then
          if has_tag "$block"; then
            accounted=1
            [ -z "$first" ] && first=$k
          else
            state=notin_notag
          fi
        elif printf '%s\n' "$block" | grep -Eqi "$verdict_re"; then
          if ! has_ptr "$block"; then
            state=verdict_noptr
          elif ! has_tag "$block"; then
            state=verdict_notag
          else
            accounted=1
            [ -z "$first" ] && first=$k
          fi
        fi
      fi
      k=$((k + 1))
    done
  done
  if [ "$accounted" -eq 1 ]; then
    ok "C-$id accounted for with a pointer or not-in-files marker, with [code] (line $first)"
    return 0
  fi
  case "$state" in
    notin_notag)      bad "C-$id is marked not in these files but carries no evidence tag" ;;
    verdict_noptr)    bad "C-$id is marked matches/differs without a file:line pointer" ;;
    verdict_notag)    bad "C-$id is marked matches/differs but carries no evidence tag" ;;
    *)                bad "C-$id is not accounted for (no matches/differs and no 'not in these files' marker)" ;;
  esac
}

for id in 30 31 32 33; do
  check_const_id "$id"
done

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

check_tag_discipline "transport.rs" "$T_START" "$T_END"
check_tag_discipline "control.rs" "$C_START" "$C_END"

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

check_open_items "transport.rs" "$T_START" "$T_END"
check_open_items "control.rs" "$C_START" "$C_END"

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
