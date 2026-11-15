#!/usr/bin/env bash
# checks/002-08.sh — checks for ticket 002-08
# "Loss counter: first wrong hop and verdict".
#
# The deliverable is the completion of section (b) The 21.7 M counter of
#   doc/design/reports/archaeology/REPORT-0b.md
# (the 002-07 per-hop log and diff are already committed; this ticket adds
# the first-wrong-hop analysis, the sub-question answer, the second-finding
# note, the hand-off for task 003 and the final verdict line).
#
# This script reads the report only. It never writes anywhere and never
# touches the frozen POC, the scratch copy or the committed 002-07 log.
#
# CRITICAL ENCODING NOTE — non-reproduction (read before changing AC 1).
# The ticket's Work step 1 assumes the 002-07 log shows a wrong hop. It does
# not: 002-07's instrumented 25 s run did NOT reproduce the anomaly — all
# 1964 hop records log the raw value 0, and the 21.7 M value never entered
# the chain under those conditions (section (b) states this explicitly, and
# evidence appendix M-3 records the 21.7 M figure as [measured] by the
# product owner in the original observation). AC 1 is therefore encoded as
# an EITHER/OR, and BOTH polarities must pass:
#
#   (i)  REPRODUCED polarity — section (b) quotes (in a tagged fenced block)
#        log lines showing the last correct value (a hop-tagged record whose
#        loss figure is a small packet count, < 1000) and the first wrong
#        value (a hop-tagged record whose loss figure is >= 1 000 000,
#        the ~21.7 M scale); the figure grammar accepts lost=/rx_lost=/
#        lost_fwd=/fwd with optional K/M/million suffixes.
#
#   (ii) NON-REPRODUCED polarity — section (b) explicitly states that the
#        anomaly did not reproduce in the instrumented run (a negation
#        statement), AND identifies the first wrong hop from SOURCE ANALYSIS
#        instead: the static chain paragraph around the "first wrong hop"
#        naming carries a file:line pointer and a unit (so the reader can
#        see which hop's field could hold a ~21.7e6 value, with pointer and
#        unit).
#
# The check's header documents this either/or to the analyst. Either branch
# satisfies AC 1; silence satisfies neither.
#
# The checks, in ticket order:
#
# * AC 1 (first wrong hop). Section (b) names the first wrong hop as one of
#   hops 1-5: a line (outside fences) carrying a "first wrong/incorrect/bad
#   hop" or "first hop" cue, with a hop id 1..5 on that line or within the
#   next two lines (grammar: hop/h + optional separator + digit 1..5).
#   Additionally branch (i) or branch (ii) above must hold. The quoted
#   block of branch (i) must be tagged [measured] (tag on the line, a fence,
#   the caption before the opening fence, or a standalone tag line after the
#   closing fence).
#
# * AC 2 (source field). Section (b) contains a paragraph that names the
#   source field of the printed value with at least one file:line pointer,
#   its unit, and a tie to the printed value (a ~21.7 / million / 21 700 000
#   cue). Concretely, one paragraph must carry ALL of: a `.rs:<n>` pointer,
#   a snake_case field token (e.g. rx_lost, lost_fwd, ack_t4), a unit cue
#   (ns/nanoseconds/ms/milliseconds/packet/count/u32/u64) and a
#   printed-value cue — and an evidence tag.
#
# * AC 3 (sub-question). The sub-question "Is the 21.7 M value a decoding
#   error or a display error?" is answered with EXACTLY ONE of the three
#   literal outcomes: "decoding error" / "display error" / "real counter".
#   All three are encoded literally; exactly one of them must appear
#   UN-negated in section (b). The other two may appear only negated
#   (preceded within ~30 characters by not / neither / nor / no), so the
#   natural answer "a real counter, not a decoding error and not a display
#   error" passes and a waffle that affirms two outcomes fails.
#
# * AC 4 (verdict). Section (b) ends with EXACTLY ONE verdict line in FINAL
#   form with a named mechanism (the amended verdict-form rule of
#   checks/002-05.sh and checks/002-01.sh: pending OR final is accepted
#   while a section is being filled; HERE the final form is required,
#   because the verdict is this ticket's deliverable). Accepted final forms:
#   "**verdict: instrument bug, mechanism: …**" and
#   "**verdict: not a bug, explanation: …**", including the prefixed variant
#   "verdict: <finding> — mechanism: …". The body must not say "pending",
#   must contain "mechanism:" or "explanation:", the mechanism text after
#   the colon must have substance (>= 10 non-space characters), the line
#   must be tagged [code] or [measured], and it must be the last non-empty
#   line of section (b). Exactly one "verdict:" line in the section.
#
# * Work step 4 (second finding). Section (b) must address the
#   "latest record wins across all return streams" behaviour of stats.rs:324
#   (a mention of latest record / per-stream separation / pooled). EITHER
#   the section flags it as a bug in its own right — then owner+falsifier
#   framing must be present within the surrounding lines — OR the section
#   argues it is not a separate bug, and that argument must be present
#   (a not-a-separate-bug / same-bug / subsumed / covered-by cue near the
#   mention). Mentioning the behaviour and doing neither fails.
#
# * Work step 5 (hand-off). A "for task 003" marker (heading or bold) in
#   section (b), followed by a substantial one-paragraph note (>= 80
#   non-space characters) that names what task 003 fixes (counter / field /
#   per-stream / fix cue) and carries an evidence tag; it must sit BEFORE
#   the verdict line (the verdict ends the section).
#
# * DoD. Every file:line pointer in section (b) prose carries an evidence
#   tag on itself or an adjacent line; the section carries [measured]
#   evidence; the section states the frozen POC was untouched (frozen +
#   untouched/not-written cue, tagged); and exactly one verdict line
#   (reinforcing AC 4).
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

heading_no() {
  local file="$1" re="$2"
  grep -Ein "$re" "$file" | awk -F: '/^[0-9]+:#/ { print $1; exit }'
}

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

# A line that is only a [measured] marker (optionally emphasised/punctuated).
tag_only_line() {
  printf '%s\n' "$1" | grep -Eq '^[[:space:]]*[*_]*\[measured\][*_]*[[:space:]]*[.;:]?[[:space:]]*$'
}

# Is the fenced block at lines $1-$2 tagged [measured]? A tag on either fence
# line, anywhere inside the block, on the caption line immediately before the
# opening fence, or as a standalone tag line immediately after the closing
# fence counts; a neighbouring prose claim's tag does not leak in.
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

# Distinct hop ids 1..5 on one line of text (same grammar as checks/002-07.sh).
# Reads the text from stdin.
line_hops() {
  grep -Eoi '(^|[^[:alnum:]_])(hop|h)[ _.:#=-]*[1-5]([^[:alnum:]]|$)' \
    | grep -oE '[1-5]' | sort -u
}

# Print every loss figure parsed from one line (lost/rx_lost/lost_fwd/fwd).
# "rx_lost=0", "lost_fwd=21734812", "lost 0 (fwd 0)", "lost 21.7M" all parse.
loss_figures() {
  printf '%s\n' "$1" | tr 'A-Z' 'a-z' | tr -d ',_' \
    | grep -oE '(lost|fwd)[^0-9]{0,12}[0-9]+(\.[0-9]+)?(k|m|million)?' \
    | grep -oE '[0-9]+(\.[0-9]+)?(k|m|million)?' || true
}

# Scale a parsed figure token to its numeric value.
figure_value() {
  local v="$1" mult=1
  case "$v" in
    *million) mult=1000000; v=${v%million} ;;
    *m)       mult=1000000; v=${v%m} ;;
    *k)       mult=1000;    v=${v%k} ;;
  esac
  awk -v x="$v" -v m="$mult" 'BEGIN { printf "%.0f\n", x * m }'
}

# ---------------------------------------------------------------------------
# Locate section (b) and its (c) boundary
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-08 does not exist yet.\n'
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
SECTION_LC=$(printf '%s\n' "$SECTION" | tr 'A-Z' 'a-z')

# Paragraph ranges inside section (b): "start end" absolute report lines.
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

# Fenced block ranges in section (b): "start end" per block (fences included).
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
# AC 1 — the first wrong hop is named, with either-or evidence
# ---------------------------------------------------------------------------

# Locate the "first wrong hop" naming line (outside fences).
fw_no=""
k=$((b_no + 1))
while [ "$k" -lt "$c_no" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  if ! is_inside_fence_b "$k" \
     && printf '%s\n' "$l" | grep -Eqi 'first[ -]?(wrong|incorrect|bad|corrupt)|first hop'; then
    fw_no=$k
    break
  fi
  k=$((k + 1))
done

fw_hop=""
if [ -z "$fw_no" ]; then
  bad "AC 1: section (b) does not name the first wrong hop (need a 'first wrong hop' cue with a hop number 1-5)"
else
  fw_lo=$((fw_no - 1)); [ "$fw_lo" -lt "$((b_no + 1))" ] && fw_lo=$((b_no + 1))
  fw_hi=$((fw_no + 2)); [ "$fw_hi" -ge "$c_no" ] && fw_hi=$((c_no - 1))
  fw_hop=$(line_range "$REPORT" "$fw_lo" "$fw_hi" | line_hops | head -n 1)
  if [ -n "$fw_hop" ]; then
    ok "AC 1: the first wrong hop is named: hop $fw_hop (lines $fw_lo-$fw_hi)"
  else
    bad "AC 1: the first-wrong-hop cue (line $fw_no) carries no hop id 1-5 on itself or the next two lines"
  fi
fi

# Branch (i): quoted log lines — last correct (small) and first wrong (millions).
small_seen=0
millions_seen=0
millions_block_tagged=0
millions_where=""
while read -r fs fe; do
  [ -n "$fs" ] || continue
  k=$((fs + 1))
  while [ "$k" -lt "$fe" ]; do
    l=$(sed -n "${k}p" "$REPORT")
    if [ -n "$(printf '%s\n' "$l" | line_hops)" ]; then
      figs=$(loss_figures "$l")
      if [ -n "$figs" ]; then
        lo=$(for v in $figs; do figure_value "$v"; done | sort -n | head -n 1)
        hi=$(for v in $figs; do figure_value "$v"; done | sort -n | tail -n 1)
        if [ "$hi" -ge 1000000 ]; then
          millions_seen=1
          millions_where="lines $fs-$fe"
          block_tagged "$fs" "$fe" && millions_block_tagged=1
        fi
        if [ "$lo" -lt 1000 ]; then
          small_seen=1
        fi
      fi
    fi
    k=$((k + 1))
  done
done <<EOF
$blocks
EOF

# Branch (ii): explicit non-reproduction + source-derived identification
# (pointer + unit around the first-wrong-hop naming).
neg_re='not reproduc|did not reproduc|does not reproduc|no longer reproduc|failed to reproduc|could not reproduc|not observed|does not occur|never (entered|reached|occurred|recurred)|not seen|no anomaly|anomaly (is )?absent|did not recur|did not appear|never carried'
neg_seen=0
if printf '%s\n' "$SECTION" | grep -Eqi "$neg_re"; then
  neg_seen=1
fi

src_id=0
if [ -n "$fw_no" ]; then
  w_lo=$((fw_no - 3)); [ "$w_lo" -lt "$((b_no + 1))" ] && w_lo=$((b_no + 1))
  w_hi=$((fw_no + 3)); [ "$w_hi" -ge "$c_no" ] && w_hi=$((c_no - 1))
  w=$(line_range "$REPORT" "$w_lo" "$w_hi")
  if printf '%s\n' "$w" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+' \
     && printf '%s\n' "$w" | grep -Eqi '(^|[^a-zA-Z])(ns|nanosecond|nanoseconds|ms|millisecond|milliseconds|packet|packets|count|u32|u64)([^a-zA-Z]|$)'; then
    src_id=1
  fi
fi

branch_i=0
branch_ii=0
if [ "$small_seen" -eq 1 ] && [ "$millions_seen" -eq 1 ] && [ "$millions_block_tagged" -eq 1 ]; then
  branch_i=1
fi
if [ "$neg_seen" -eq 1 ] && [ "$src_id" -eq 1 ]; then
  branch_ii=1
fi

if [ -n "$fw_hop" ]; then
  if [ "$branch_i" -eq 1 ]; then
    ok "AC 1: branch (i) — quoted log lines show the last correct and first wrong value ($millions_where), block tagged [measured]"
  elif [ "$branch_ii" -eq 1 ]; then
    ok "AC 1: branch (ii) — non-reproduction stated and the first wrong hop identified from source analysis with pointer and unit (lines $w_lo-$w_hi)"
  elif [ "$millions_seen" -eq 1 ] && [ "$millions_block_tagged" -eq 0 ]; then
    bad "AC 1: quoted log lines show a wrong (millions-scale) value ($millions_where) but that block carries no [measured] tag"
  elif [ "$millions_seen" -eq 1 ]; then
    bad "AC 1: quoted log lines show a millions-scale value but no small (last-correct) hop record alongside it"
  elif [ "$neg_seen" -eq 1 ]; then
    bad "AC 1: non-reproduction is stated but the source-derived identification lacks a file:line pointer or a unit near the first-wrong-hop naming (lines $w_lo-$w_hi)"
  else
    bad "AC 1: neither branch holds — quote the last-correct and first-wrong log lines (tagged [measured]), or state the non-reproduction and identify the hop from source analysis with pointer and unit"
  fi
fi

# ---------------------------------------------------------------------------
# AC 2 — the source field of the printed value, with pointer and unit
# ---------------------------------------------------------------------------

ac2=0
ac2_where=""
ac2_tag=0
while read -r s e; do
  [ -n "$s" ] || continue
  block=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$block" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+' \
     && printf '%s\n' "$block" | grep -Eq '[a-z][a-z0-9]*_[a-z0-9_]+' \
     && printf '%s\n' "$block" | grep -Eqi '(^|[^a-zA-Z])(ns|nanosecond|nanoseconds|ms|millisecond|milliseconds|packet|packets|count|u32|u64)([^a-zA-Z]|$)' \
     && printf '%s\n' "$block" | grep -Eqi '21[.,]?7|million|21[[:space:]_,.]?000[[:space:]_,.]?000|printed (value|figure|number)|value (as )?printed'; then
    ac2=1
    ac2_where="lines $s-$e"
    has_tag "$block" && ac2_tag=1
  fi
done <<EOF
$RANGES
EOF

if [ "$ac2" -eq 0 ]; then
  bad "AC 2: no paragraph names the source field of the printed value with a file:line pointer, its unit and the printed-value tie (~21.7 M)"
elif [ "$ac2_tag" -eq 0 ]; then
  bad "AC 2: the source-field identification carries no evidence tag ($ac2_where)"
else
  ok "AC 2: the source field is named with a pointer and its unit ($ac2_where)"
fi

# ---------------------------------------------------------------------------
# AC 3 — the sub-question answered with exactly one of the three outcomes
# ---------------------------------------------------------------------------

# For each literal outcome, count affirmed (un-negated) occurrences in
# section (b). Two safeguards keep the QUOTED QUESTION itself from counting
# as an answer: text up to and including the last '?' on a line is dropped
# (the question restatement), and an occurrence whose preceding ~30
# characters end in a question-context or negation cue (not / neither / nor /
# no / sub-question / whether / asked) counts as non-affirming. So the
# natural answer "the value is a real counter, not a decoding error and not
# a display error" passes and a waffle that affirms two outcomes fails.
outcome_status() {
  printf '%s' "$SECTION_LC" | awk -v p="$1" '
    {
      line = $0
      if (index(line, "?") > 0) {
        n = split(line, parts, "?")
        line = parts[n]
      }
      start = 1
      while ((i = index(substr(line, start), p)) > 0) {
        abs = start + i - 1
        lo = abs - 30; if (lo < 1) lo = 1
        pre = substr(line, lo, abs - lo)
        if (pre ~ /(not|neither|nor|no)([[:space:]]+(an?|the))?[^a-z]{0,8}$/ \
            || pre ~ /(sub-?question|whether|asked)[^a-z]{0,8}$/) neg++
        else aff++
        start = abs + length(p)
      }
    }
    END { printf "%d %d\n", aff + 0, neg + 0 }'
}

affirmed_outcomes=""
n_aff=0
for phrase in "decoding error" "display error" "real counter"; do
  st=$(outcome_status "$phrase")
  aff=${st%% *}
  if [ "$aff" -ge 1 ]; then
    affirmed_outcomes="$affirmed_outcomes '$phrase'"
    n_aff=$((n_aff + 1))
  fi
done

if [ "$n_aff" -eq 0 ]; then
  bad "AC 3: the sub-question is not answered — none of 'decoding error', 'display error', 'real counter' appears un-negated in section (b)"
elif [ "$n_aff" -gt 1 ]; then
  bad "AC 3: the sub-question answer is ambiguous — more than one outcome appears affirmed:$affirmed_outcomes; exactly one is required"
else
  ok "AC 3: the sub-question is answered with exactly one outcome:$affirmed_outcomes"
fi

# ---------------------------------------------------------------------------
# AC 4 — exactly one verdict line, final form, named mechanism, ends section
# ---------------------------------------------------------------------------

verdict_nos=$(printf '%s\n' "$SECTION" \
  | grep -En '^[[:space:]]*(\*\*)?verdict:' | cut -d: -f1 || true)
verdict_count=$(printf '%s\n' "$verdict_nos" | grep -c . || true)

v_no=""
if [ "$verdict_count" -eq 0 ]; then
  bad "AC 4: section (b) has no 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "AC 4: section (b) has $verdict_count 'verdict:' lines; exactly one is required"
else
  v_abs=$(printf '%s\n' "$verdict_nos" | head -n 1)
  v_no=$((b_no + v_abs))
  verdict_line=$(sed -n "${v_no}p" "$REPORT")
  verdict_body=$(printf '%s\n' "$verdict_line" \
    | sed -E 's/^[[:space:]]*(\*\*)?verdict:[[:space:]]*//; s/[[:space:]]*\*\*[[:space:]]*$//')

  if printf '%s' "$verdict_body" | grep -Eqi 'pending'; then
    bad "AC 4: the verdict line still says 'pending' (line $v_no); ticket 002-08 must write the FINAL verdict"
  elif printf '%s' "$verdict_body" | grep -Eqi 'mechanism:'; then
    mech=$(printf '%s' "$verdict_body" | sed -E 's/^[^:]*mechanism:[[:space:]]*//')
    mech_chars=$(printf '%s' "$mech" | tr -d '[:space:]' | wc -c)
    if [ "$mech_chars" -ge 10 ]; then
      ok "AC 4: the verdict is in final form with a named mechanism (line $v_no)"
    else
      bad "AC 4: the verdict says 'mechanism:' but names no mechanism of substance (line $v_no)"
    fi
  elif printf '%s' "$verdict_body" | grep -Eqi 'explanation:'; then
    ok "AC 4: the verdict is in final form (not a bug, explanation) (line $v_no)"
  else
    bad "AC 4: the verdict line is in neither final form ('instrument bug, mechanism: …' / 'not a bug, explanation: …'): $verdict_line"
  fi

  if printf '%s\n' "$verdict_line" | grep -qE '\[(code|measured)\]'; then
    ok "AC 4: the verdict is tagged [code] or [measured]"
  else
    bad "AC 4: the verdict (line $v_no) carries no [code]/[measured] evidence tag"
  fi

  # The verdict must be the last non-empty line of section (b).
  vlast=""
  k=$((c_no - 1))
  while [ "$k" -gt "$b_no" ]; do
    l=$(sed -n "${k}p" "$REPORT")
    if [ -n "$(printf '%s' "$l" | tr -d '[:space:]')" ]; then
      vlast=$k
      break
    fi
    k=$((k - 1))
  done
  if [ -n "$vlast" ] && [ "$vlast" -eq "$v_no" ]; then
    ok "AC 4: the verdict is the last non-empty line of section (b)"
  else
    bad "AC 4: section (b) does not end with the verdict (last non-empty line is $vlast, verdict at $v_no)"
  fi
fi

# ---------------------------------------------------------------------------
# Work step 4 — the latest-record-wins behaviour: second finding or argued away
# ---------------------------------------------------------------------------

lrw_re='latest record|latest-record|last record wins|per-stream separation|no per-stream|pooled'
lrw_lines=""
k=$((b_no + 1))
while [ "$k" -lt "$c_no" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  if printf '%s\n' "$l" | grep -Eqi "$lrw_re"; then
    lrw_lines="$lrw_lines $k"
  fi
  k=$((k + 1))
done

bug_cue='second finding|in its own right|separate bug|own bug|also a bug|additional (bug|finding)|bug of its own|finding in its own|its own finding'
notbug_cue='not a (separate|second|independent|distinct)|not a bug in its own right|not (counted|treated|reported|flagged) as a separate|same (instrument )?bug|part of the same|subsum|covered by (the|this)|no separate finding|not a separate finding|single (bug|finding|defect)|one (bug|defect|mechanism)'

if [ -z "$(printf '%s' "$lrw_lines" | tr -d '[:space:]')" ]; then
  bad "Work step 4: section (b) does not address the 'latest record wins across all return streams' behaviour (stats.rs:324)"
else
  # Per line the not-a-separate-bug cue wins over the bug cue: "not a
  # separate bug" contains the substring "separate bug", so a plain cue
  # search would misread the argued-away polarity as flagged.
  flagged=0
  flagged_where=""
  for ln in $lrw_lines; do
    l=$(sed -n "${ln}p" "$REPORT")
    if printf '%s\n' "$l" | grep -Eqi "$notbug_cue"; then
      continue
    fi
    lo=$((ln - 2)); [ "$lo" -lt "$((b_no + 1))" ] && lo=$((b_no + 1))
    hi=$((ln + 2)); [ "$hi" -ge "$c_no" ] && hi=$((c_no - 1))
    if line_range "$REPORT" "$lo" "$hi" | grep -Eqi "$bug_cue"; then
      flagged=1
      flagged_where="line $ln"
      break
    fi
  done
  if [ "$flagged" -eq 1 ]; then
    ln=$flagged_where; ln=${ln#line }
    lo=$((ln - 5)); [ "$lo" -lt "$((b_no + 1))" ] && lo=$((b_no + 1))
    hi=$((ln + 5)); [ "$hi" -ge "$c_no" ] && hi=$((c_no - 1))
    w=$(line_range "$REPORT" "$lo" "$hi")
    if printf '%s\n' "$w" | grep -Eqi 'owner' && printf '%s\n' "$w" | grep -Eqi 'falsifier'; then
      ok "Work step 4: the latest-record-wins behaviour is flagged as a bug in its own right, with owner and falsifier (line $ln)"
    else
      bad "Work step 4: the second finding is flagged (line $ln) but the owner+falsifier framing is missing (lines $lo-$hi)"
    fi
  else
    argued=0
    for ln in $lrw_lines; do
      lo=$((ln - 4)); [ "$lo" -lt "$((b_no + 1))" ] && lo=$((b_no + 1))
      hi=$((ln + 4)); [ "$hi" -ge "$c_no" ] && hi=$((c_no - 1))
      if line_range "$REPORT" "$lo" "$hi" | grep -Eqi "$notbug_cue"; then
        argued=1
        break
      fi
    done
    if [ "$argued" -eq 1 ]; then
      ok "Work step 4: the latest-record-wins behaviour is argued not to be a separate bug (near line $ln)"
    else
      bad "Work step 4: the latest-record-wins behaviour is mentioned but neither flagged as a second finding (with owner+falsifier) nor argued away as not a separate bug"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Work step 5 — one-paragraph hand-off note for task 003, before the verdict
# ---------------------------------------------------------------------------

ho_no=""
k=$((b_no + 1))
while [ "$k" -lt "$c_no" ]; do
  if ! is_inside_fence_b "$k" && sed -n "${k}p" "$REPORT" | grep -Eqi 'for task 003'; then
    ho_no=$k
    break
  fi
  k=$((k + 1))
done

if [ -z "$ho_no" ]; then
  bad "Work step 5: no hand-off note 'for task 003' in section (b)"
else
  hline=$(sed -n "${ho_no}p" "$REPORT")
  is_heading=0
  printf '%s\n' "$hline" | grep -Eq '^#+[[:space:]]' && is_heading=1
  is_bold=0
  printf '%s\n' "$hline" | grep -Eqi '\*\*[^*]*for task 003[^*]*\*\*' && is_bold=1
  if [ "$is_heading" -eq 0 ] && [ "$is_bold" -eq 0 ]; then
    bad "Work step 5: the 'for task 003' hand-off is neither a heading nor bold (line $ho_no)"
  else
    htail=$(printf '%s' "$hline" | awk '
      { i = index(tolower($0), "for task 003")
        if (i > 0) { s = substr($0, i + 12); sub(/^[[:space:]*_.:\xe2\x80\x94-]+/, "", s); print s } }')
    htext="$htail"
    hstart=$((ho_no + 1))
    while [ "$hstart" -lt "$c_no" ] \
          && [ -z "$(sed -n "${hstart}p" "$REPORT" | tr -d '[:space:]')" ]; do
      hstart=$((hstart + 1))
    done
    hend=$hstart
    while [ "$hend" -lt "$c_no" ]; do
      hline2=$(sed -n "${hend}p" "$REPORT")
      if [ -z "$(printf '%s' "$hline2" | tr -d '[:space:]')" ] \
         || printf '%s\n' "$hline2" | grep -Eq '^#+[[:space:]]' \
         || printf '%s\n' "$hline2" | grep -Eq '^[[:space:]]*(\*\*)?verdict:'; then
        break
      fi
      htext="$htext
$hline2"
      hend=$((hend + 1))
    done

    hchars=$(printf '%s' "$htext" | tr -d '[:space:]' | wc -c)
    if [ "$hchars" -lt 80 ]; then
      bad "Work step 5: the hand-off note for task 003 has no paragraph of substance ($hchars non-space characters; need at least 80)"
    else
      if printf '%s\n' "$htext" | grep -Eqi 'counter|rx_lost|lost_fwd|per-stream|fix|repair'; then
        ok "Work step 5: the hand-off note says what task 003 fixes (line $ho_no)"
      else
        bad "Work step 5: the hand-off note does not say what task 003 fixes (need a counter/field/fix cue)"
      fi
      if printf '%s\n' "$htext" | grep -qE '\[(code|measured)\]'; then
        ok "Work step 5: the hand-off note carries an evidence tag"
      else
        bad "Work step 5: the hand-off note carries no evidence tag"
      fi
      if [ -n "$v_no" ] && [ "$ho_no" -lt "$v_no" ]; then
        ok "Work step 5: the hand-off note sits before the verdict line"
      else
        bad "Work step 5: the hand-off note (line $ho_no) must come before the verdict (line ${v_no:-none})"
      fi
    fi
  fi
fi

# ---------------------------------------------------------------------------
# DoD — evidence tags on every claim, frozen POC untouched, one verdict line
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

frozen_stmt=0
frozen_tagged=0
frozen_where=""
while read -r s e; do
  [ -n "$s" ] || continue
  block=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$block" | grep -Eqi 'frozen' \
     && printf '%s\n' "$block" | grep -Eqi 'untouched|not (been )?(written|modified|run)|neither run nor written|unmodified|left alone|was not (run|written)|no write'; then
    frozen_stmt=1
    frozen_where="lines $s-$e"
    has_tag "$block" && frozen_tagged=1
  fi
done <<EOF
$RANGES
EOF
if [ "$frozen_stmt" -eq 0 ]; then
  bad "DoD: section (b) does not state that the frozen POC was left untouched (frozen + untouched/not-written cue)"
elif [ "$frozen_tagged" -eq 0 ]; then
  bad "DoD: the frozen-POC-untouched statement carries no evidence tag ($frozen_where)"
else
  ok "DoD: section (b) states the frozen POC was untouched, with an evidence tag ($frozen_where)"
fi

# Exactly one verdict line (reinforcing AC 4; reported here even when AC 4
# already failed on form, so the author sees the count independently).
if [ "$verdict_count" -eq 1 ]; then
  ok "DoD: section (b) carries exactly one verdict line"
else
  bad "DoD: section (b) carries $verdict_count verdict line(s); exactly one is required"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
