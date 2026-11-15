#!/usr/bin/env bash
# checks/002-10.sh — checks for ticket 002-10
# "Flood test of the unbounded drain".
#
# The deliverable is the flood-test result written into section (e) Run
# confirmations of doc/design/reports/archaeology/REPORT-0b.md (deliverable
# (e) 1 of epic 002): a baseline and a flooded run (each >= 25 s) of the
# frozen POC instrument, the distributions of sender-side "TX pacing
# lateness" (sender.rs:308, stats.rs:416, ns) and the mixer's ticker
# lateness for both runs, the nominal and the achieved flood rate, and a
# confirmed/refuted starvation verdict with a file:line mechanism pointer
# (module.rs:1669 or another). The section (e) verdict line itself stays
# "pending" until 002-13 closes the epic.
#
# This script reads the report and, when one exists, a companion
# scratch-copy diff. It never writes anywhere and never touches the frozen
# POC or the scratch copy. Running the measurements or building the flood
# source is the investigator's work, not this check's.
#
# The conventions below are spelled out so the author can comply exactly.
# They make the prose machine-checkable without fixing its wording. Section
# (e) runs from the "## (e) ..." heading to the next "## " heading; "### "
# subsections inside it belong to the section (other run-confirmation
# tickets fill (e) too, so every cue below is flood-specific).
#
# * (AC 1, flood rate — two polarities, both encoded.)
#   Common to both: the section must report the NOMINAL datagram rate of one
#   mixer input and the ACHIEVED flood rate, each as a number with a rate
#   unit (datagrams|packets|messages|msgs|pps|da per second, e.g. "2 000
#   datagrams per second"), each tagged [measured] (the ticket tags both
#   rates [measured]).
#   Polarity A (10x reached): the section states the 10x target ("10x",
#   "10×", "ten times") and the achieved rate is mechanically compared
#   against 10× the nominal rate — it must be within 10 % (ratio 9..11).
#   Polarity B (AC 5 branch): the section states that 10x could NOT be
#   reached ("could not be reached", "not achieved", "not reachable", ...);
#   then it must additionally state (i) the highest rate reached, (ii) that
#   the (e) 1 verdict holds only up to that rate, and (iii) the limiting
#   reason (P-01 datagram size, MTU, fragments, socket buffer, host/cpu
#   limit, ...).
#   The polarity is chosen from the section text: a "could not reach /
#   not achieved" statement selects polarity B; otherwise polarity A.
#   Recommended phrasing for polarity A: "the nominal rate ... is 2 000
#   datagrams per second [measured]; the flood achieved 19 900 datagrams
#   per second, within 0.5 % of 10x the nominal rate [measured]" — the
#   number is classified by its nearest preceding context ("nominal" vs
#   "flood/achieved"), so keep each rate's number within a sentence of its
#   keyword.
# * (AC 2, distributions.) For EACH of the two metrics — "TX pacing
#   lateness" (cue: tx pacing / pacing lateness) and the mixer's "ticker
#   lateness" (cue: ticker) — and EACH run (baseline, flooded), the section
#   must give median, p99 and max, each value carrying a time unit (ns, µs,
#   us or ms — the ticket names ns/µs/ms; the stored values are ns). The
#   three values may be inline ("median 120 ns, p99 340 ns, max 900 ns") or
#   a table row, but ONE line must carry BOTH the metric cue and the run
#   cue together with the three value+unit occurrences (for a table: the
#   row naming metric and run); the words median, p99 and max must appear
#   in the surrounding paragraph (for a table: in its header row). The
#   paragraph carries [measured].
# * (AC 3, starvation verdict.) The section must state "starvation:
#   confirmed" or "starvation: refuted" (confirmed/refuted after the word
#   starvation), and that statement must name the mechanism with a
#   file:line pointer (module.rs:1669 or another *.rs:<line>), tagged
#   [measured] — this is the run confirmation. Separately, section (e)
#   keeps EXACTLY ONE verdict line in one of the allowed forms (the amended
#   verdict-form convention of checks/002-05.sh/002-07.sh): the placeholder
#   "pending" (the state until 002-13) or the final form with "mechanism:"
#   or "explanation:". Later sanctioned completion does not break the check.
# * (AC 4, frozen tree.) The section states that the frozen-tree
#   fingerprint (002-01) is unchanged — the statement suffices here;
#   checks/002-01.sh guards the fingerprint mechanically.
# * (DoD, evidence tags.) Every line carrying a "file.rs:<digits>" pointer
#   (outside a fenced block) carries an evidence tag on itself or an
#   adjacent line, and the section carries [measured] evidence.
# * (DoD, flood source location.) The section states where the flood source
#   lives — a throwaway probe under probes/ (C-133) or a scratch-copy
#   option (C-130/C-133): a paragraph naming the flood source (probe /
#   standalone sender / flood generator) and its location (probes/ or the
#   scratch copy), tagged.
# * (DoD, run provenance.) The section states which tree the runs came from
#   and why that is acceptable, because the POC's working tree now carries
#   the 003-03/003-04 fixes. Two acceptable shapes: (i) poc-untouched-
#   equivalent — a paragraph with "poc-untouched" + scratch/checkout/tag +
#   identical/equivalent/fingerprint; (ii) post-fix — a paragraph with
#   post-fix/003-03/003-04/current build + scratch/frozen/working tree +
#   the justification that the pacing fix does not touch the drain
#   behaviour these runs measure. In both shapes the frozen tree must also
#   be stated as neither run nor written.
# * (DoD, run durations.) Each run's duration is stated as >= 25 s ("25 s",
#   "30 seconds", "--duration 25", "at least 25", ">= 25 s").
# * (DoD, logging — 002-07 convention, only if used.) If the section says
#   diagnostic logging was added (e.g. a ticker-lateness log in the scratch
#   copy), then: (i) a paragraph must state the logging lived in the
#   scratch copy, not the frozen tree; (ii) a companion diff must exist
#   beside the report (*002-10*.diff/.patch, or a diff whose content names
#   002-10 or the ticker-lateness logging) — tracked, non-empty, with a
#   unified-diff header, at least one added line with a logging cue, no
#   reference to the frozen tree, header paths only inside poc-scratch, and
#   no target//experiments/ paths; (iii) the section must state the diff is
#   logging-only / changes no behaviour. If no logging is claimed, none of
#   this is required.
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

has_measured() {
  printf '%s\n' "$1" | grep -qF '[measured]'
}

has_ptr() {
  printf '%s\n' "$1" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+'
}

is_fence_line() {
  printf '%s\n' "$1" | grep -Eq '^[[:space:]]*```'
}

# Is the repository-relative path $1 tracked? In a non-git tree (the check's
# own fixture harness copies the script into a scratch tree) tracking cannot
# be evaluated and the file's existence is accepted instead.
is_tracked() {
  if ! git -C "$REPO_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    return 0
  fi
  git -C "$REPO_ROOT" ls-files --error-unmatch -- "$1" >/dev/null 2>&1
}

# ---------------------------------------------------------------------------
# Locate section (e) and its boundary (the next "## " heading; "### "
# subsections stay inside the section)
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-10 does not exist yet.\n'
  exit 1
fi

TOTAL_LINES=$(wc -l < "$REPORT")

e_no=$(heading_no "$REPORT" '^#+[[:space:]]*\(e\)')
if [ -z "$e_no" ]; then
  bad "section (e) Run confirmations not found (heading containing '(e)')"
  printf '\n%d check(s) failed.\n' "$fail"
  exit 1
fi
e_end=$(awk -v s="$e_no" 'NR > s && /^## /{ print NR; exit }' "$REPORT")
if [ -z "$e_end" ]; then
  e_end=$((TOTAL_LINES + 1))
fi
ok "section (e) found at line $e_no, bounded by the level-2 heading at line $e_end"

SECTION=$(line_range "$REPORT" "$((e_no + 1))" "$((e_end - 1))")
nonempty=$(printf '%s\n' "$SECTION" | grep -c '[^[:space:]]' || true)

# Placeholder guard is a diagnosis, not an early exit: the checks below
# still report their own missing evidence.
if [ "$nonempty" -lt 4 ] && printf '%s\n' "$SECTION" | grep -Eqi 'to be filled'; then
  bad "section (e) is still the placeholder ($nonempty non-empty lines); 002-10 has not been written yet"
fi

# Paragraphs of section (e): non-empty runs, headings and blank lines split.
# Prints "start end" absolute report line numbers.
RANGES=$(awk -v s="$e_no" -v t="$e_end" -v f="$REPORT" '
  BEGIN {
    start = 0
    k = s + 1
    while (k < t) {
      cmd = "sed -n " k "p " f
      cmd | getline line
      close(cmd)
      if (line ~ /^[[:space:]]*$/ || line ~ /^#+[[:space:]]/) {
        if (start > 0) { print start, k - 1; start = 0 }
      } else if (start == 0) {
        start = k
      }
      k++
    }
    if (start > 0) print start, t - 1
  }')

# Is report line $1 inside a fenced code block within (e)?
is_inside_fence_e() {
  local n="$1" k=$((e_no + 1)) f=0
  while [ "$k" -lt "$n" ]; do
    is_fence_line "$(sed -n "${k}p" "$REPORT")" && f=$((1 - f))
    k=$((k + 1))
  done
  [ "$f" -eq 1 ]
}

# ---------------------------------------------------------------------------
# Rate parsing (AC 1). The text is joined to one stream (sentences wrap);
# for every "<number> [k|m] <rate-noun>" occurrence the preceding ~80
# characters classify it: "nominal" wins over "flood/achieved" when both
# appear, so a sentence citing the nominal rate inside the flood sentence
# is not misread as the flood rate. Prints the first matching rate in
# datagrams per second (k/m suffixes applied), or nothing.
# ---------------------------------------------------------------------------

first_rate() {
  local ctxre="$1"
  printf '%s' "$SECTION" | tr '\n' ' ' | awk -v ctxre="$ctxre" '
    {
      rest = tolower($0)
      while (match(rest, /[0-9][0-9.,_ ]*[0-9][ ]*(k|m)?[ ]*(datagrams?|packets?|messages?|msgs?|pps|da)([^a-z0-9]|$)/)) {
        m   = substr(rest, RSTART, RLENGTH)
        pre = substr(rest, 1, RSTART)
        ctx = substr(pre, length(pre) > 80 ? length(pre) - 79 : 1)
        if (ctx ~ ctxre) {
          num = m
          sub(/[^a-z0-9]$/, "", num)
          sub(/(datagrams?|packets?|messages?|msgs?|pps|da)$/, "", num)
          sub(/[ ]+$/, "", num)
          mult = 1
          if (num ~ /k$/)      { mult = 1000;   sub(/k$/, "", num) }
          else if (num ~ /m$/) { mult = 1000000; sub(/m$/, "", num) }
          gsub(/[.,_ ]/, "", num)
          if (num ~ /^[0-9]+$/) { printf "%s\n", num * mult; exit }
        }
        if (RLENGTH < 2) break
        rest = substr(rest, RSTART + RLENGTH - 1)
      }
    }'
}

# ---------------------------------------------------------------------------
# AC 1 — nominal rate and achieved flood rate (polarity A or the AC-5 branch)
# ---------------------------------------------------------------------------

NOMINAL=$(first_rate 'nominal')
FLOODRATE=$(first_rate 'flood|achiev')

# A rate figure: a number (with separators) directly followed by a rate noun.
rate_fig_re='[0-9][0-9.,_ ]*[[:space:]]*(k|m)?[[:space:]]*(datagrams?|packets?|messages?|msgs?|pps|da)([^[:alnum:]]|$)'

nominal_para=""
flood_para=""
while read -r s e; do
  [ -n "$s" ] || continue
  p=$(line_range "$REPORT" "$s" "$e")
  if [ -z "$nominal_para" ] \
     && printf '%s\n' "$p" | grep -Eqi 'nominal' \
     && printf '%s\n' "$p" | grep -Eqi "$rate_fig_re"; then
    nominal_para="$p"
  fi
  if [ -z "$flood_para" ] \
     && printf '%s\n' "$p" | grep -Eqi 'flood|achiev' \
     && printf '%s\n' "$p" | grep -Eqi "$rate_fig_re"; then
    flood_para="$p"
  fi
done <<EOF
$RANGES
EOF

if [ -n "$NOMINAL" ] && [ -n "$nominal_para" ] && has_measured "$nominal_para"; then
  ok "AC 1: nominal datagram rate reported and tagged [measured] ($NOMINAL/s)"
else
  bad "AC 1: the nominal datagram rate of one mixer input is not reported as a tagged number+rate-unit figure (e.g. '2 000 datagrams per second ... [measured]')"
fi

if [ -n "$FLOODRATE" ] && [ -n "$flood_para" ] && has_measured "$flood_para"; then
  ok "AC 1: achieved flood rate reported and tagged [measured] ($FLOODRATE/s)"
else
  bad "AC 1: the achieved flood rate is not reported as a tagged number+rate-unit figure (e.g. 'achieved 19 900 datagrams per second ... [measured]')"
fi

ac5_re='(could not|cannot|can not|unable to|was not|were not|did not)[^.]{0,40}(reach|attain|achiev)|not reachable|unreachable'
if printf '%s\n' "$SECTION" | grep -Eqi "$ac5_re"; then
  # --- Polarity B: the AC 5 branch -----------------------------------------
  ok "AC 1: polarity B (AC 5 branch) detected — 10x stated as not reached"
  if printf '%s\n' "$SECTION" | grep -Eqi 'highest rate'; then
    ok "AC 1: the highest rate reached is stated"
  else
    bad "AC 1: the AC 5 branch requires the highest rate reached to be stated ('highest rate ...')"
  fi
  if printf '%s\n' "$SECTION" | grep -Eqi '(verdict|finding|result|starvation|confirmation)[^.]{0,120}only up to|holds only up to|only up to (that|this|the) rate'; then
    ok "AC 1: the section states the verdict holds only up to the highest rate"
  else
    bad "AC 1: the AC 5 branch requires the statement that the (e) 1 verdict holds only up to the highest rate ('holds only up to ...')"
  fi
  if printf '%s\n' "$SECTION" | grep -Eqi 'P-01|MTU|fragment|datagram size|packet size|socket buffer|rcvbuf|kernel|host limit|cpu limit|loopback limit|bandwidth'; then
    ok "AC 1: the limiting reason is named"
  else
    bad "AC 1: the AC 5 branch requires the limiting reason to be named (P-01 datagram size, MTU/fragments, socket buffer, host limit, ...)"
  fi
else
  # --- Polarity A: 10x within 10 % ------------------------------------------
  if [ -z "$NOMINAL" ] || [ -z "$FLOODRATE" ]; then
    bad "AC 1: cannot verify the 10x relation — the nominal and flood rates must both be parseable numbers with a rate unit (datagrams/packets/messages/msgs/pps/da per second)"
  else
    if printf '%s\n' "$SECTION" | grep -Eqi '10[[:space:]]*[x×]|ten[[:space:]]+times|10[[:space:]]+times'; then
      ok "AC 1: the 10x target is stated"
    else
      bad "AC 1: the 10x flood target is not stated ('10x', '10×' or 'ten times')"
    fi
    if awk -v n="$NOMINAL" -v a="$FLOODRATE" 'BEGIN {
        exit !(a >= 9 * n && a <= 11 * n)
      }'; then
      ok "AC 1: the achieved flood rate is within 10 % of 10x the nominal rate ($FLOODRATE vs 10 x $NOMINAL)"
    else
      bad "AC 1: the achieved flood rate ($FLOODRATE/s) is NOT within 10 % of 10x the nominal rate ($NOMINAL/s); if 10x could not be reached, state that explicitly (AC 5 branch: highest rate, verdict holds only up to it, limiting reason)"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# AC 2 — median/p99/max of both latenesses for both runs, with units
# ---------------------------------------------------------------------------

# Count value+unit (ns/µs/us/ms) occurrences on one line.
count_vals() {
  printf '%s\n' "$1" \
    | grep -oE '[0-9][0-9.,_ ]*[0-9][[:space:]]*(ns|µs|μs|us|ms)([^[:alnum:]]|$)|[0-9][[:space:]]*(ns|µs|μs|us|ms)([^[:alnum:]]|$)' \
    | wc -l
}

# For one (metric, run) pair: a paragraph carrying the metric cue and the
# words median/p99/max, with a line carrying BOTH the metric cue and the
# run cue and at least three value+unit occurrences, tagged [measured].
check_dist() {
  local label="$1" mre="$2" rre="$3"
  local s e para l found=0 fs=0 fe=0
  while read -r s e; do
    [ -n "$s" ] || continue
    para=$(line_range "$REPORT" "$s" "$e")
    printf '%s\n' "$para" | grep -Eqi "$mre" || continue
    printf '%s\n' "$para" | grep -Eqi 'median' || continue
    printf '%s\n' "$para" | grep -Eqi 'p99' || continue
    printf '%s\n' "$para" | grep -Eqi 'max'  || continue
    while IFS= read -r l; do
      if printf '%s\n' "$l" | grep -Eqi "$mre" \
         && printf '%s\n' "$l" | grep -Eqi "$rre" \
         && [ "$(count_vals "$l")" -ge 3 ]; then
        found=1
        fs=$s; fe=$e
        break
      fi
    done <<EOF
$para
EOF
    [ "$found" -eq 1 ] && break
  done <<EOF
$RANGES
EOF
  if [ "$found" -eq 1 ] && has_measured "$(line_range "$REPORT" "$fs" "$fe")"; then
    ok "AC 2: $label distribution (median/p99/max with units) present [measured] (lines $fs-$fe)"
  elif [ "$found" -eq 1 ]; then
    bad "AC 2: the $label distribution (lines $fs-$fe) carries no [measured] tag"
  else
    bad "AC 2: no $label distribution with median, p99 and max, each value carrying a time unit (ns/µs/ms), for that run (e.g. 'median 120 ns, p99 340 ns, max 900 ns')"
  fi
}

check_dist "TX pacing lateness, baseline run" 'tx[ _-]?pacing|pacing lateness' 'baseline|no flood|unflooded|non-flooded'
check_dist "TX pacing lateness, flooded run"  'tx[ _-]?pacing|pacing lateness' 'flood'
check_dist "mixer ticker lateness, baseline run" 'ticker' 'baseline|no flood|unflooded|non-flooded'
check_dist "mixer ticker lateness, flooded run"  'ticker' 'flood'

# ---------------------------------------------------------------------------
# DoD (needed before AC 3 reads cleanly) — run durations >= 25 s per run
# ---------------------------------------------------------------------------

dur_re='(--duration[[:space:]]*(2[5-9]|[3-9][0-9]+)([^0-9]|$))|((^|[^0-9.])(2[5-9]|[3-9][0-9]+)[[:space:]]*(s|sec|secs|second|seconds)([^[:alnum:]]|$))|((2[5-9]|[3-9][0-9]+)-second)|(at least 25)|((≥|>=)[[:space:]]*25)'

check_duration() {
  local label="$1" rre="$2" s e para
  while read -r s e; do
    [ -n "$s" ] || continue
    para=$(line_range "$REPORT" "$s" "$e")
    if printf '%s\n' "$para" | grep -Eqi "$rre" && printf '%s\n' "$para" | grep -Eqi "$dur_re"; then
      ok "DoD: the $label run's duration (>= 25 s) is stated (lines $s-$e)"
      return 0
    fi
  done <<EOF
$RANGES
EOF
  bad "DoD: the $label run's duration is not stated as >= 25 s ('25 s', '30 seconds', '--duration 25', 'at least 25')"
}

check_duration "baseline" 'baseline|no flood|unflooded|non-flooded'
check_duration "flooded"  'flood'

# ---------------------------------------------------------------------------
# AC 3 — starvation confirmed/refuted with a mechanism pointer; the section
# (e) verdict line stays in an allowed form (pending until 002-13)
# ---------------------------------------------------------------------------

starv=0
starv_where=""
starv_line=""
while read -r s e; do
  [ -n "$s" ] || continue
  para=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$para" | grep -Eqi 'starvation' \
     && printf '%s\n' "$para" | grep -Eqi 'confirmed|refuted' \
     && has_ptr "$para" \
     && has_measured "$para"; then
    starv=1
    starv_where="lines $s-$e"
    starv_line=$(printf '%s\n' "$para" | grep -Ei 'starvation' | head -n 1)
    break
  fi
done <<EOF
$RANGES
EOF

if [ "$starv" -eq 1 ]; then
  ok "AC 3: the run confirmation states starvation confirmed/refuted with a file:line mechanism pointer ($starv_where): $(printf '%s' "$starv_line" | cut -c1-120)"
else
  bad "AC 3: no [measured] statement of 'starvation: confirmed' or 'refuted' naming the mechanism with a file:line pointer (module.rs:1669 or another)"
fi

verdict_count=$(printf '%s\n' "$SECTION" \
  | grep -Ec '^[[:space:]]*(\*\*)?verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "AC 3: section (e) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "AC 3: section (e) has $verdict_count 'verdict:' lines; exactly one is required"
else
  verdict_line=$(printf '%s\n' "$SECTION" \
    | grep -E '^[[:space:]]*(\*\*)?verdict:' | head -n 1)
  verdict_body=$(printf '%s\n' "$verdict_line" \
    | sed -E 's/^[[:space:]]*(\*\*)?verdict:[[:space:]]*//; s/[[:space:]]*\*\*[[:space:]]*$//')
  if printf '%s' "$verdict_body" | grep -Eqi '^pending$'; then
    ok "AC 3: section (e) has exactly one verdict line (pending; 002-13 owns the section verdict)"
  elif printf '%s' "$verdict_body" | grep -Eqi 'mechanism:'; then
    ok "AC 3: section (e) has exactly one verdict line (final form, mechanism)"
  elif printf '%s' "$verdict_body" | grep -Eqi 'explanation:'; then
    ok "AC 3: section (e) has exactly one verdict line (final form, explanation)"
  else
    bad "AC 3: section (e) has one verdict line but it is not in an allowed form: $verdict_line"
  fi
fi

# ---------------------------------------------------------------------------
# AC 4 — the frozen-tree fingerprint is unchanged (statement suffices; the
# mechanical guard is checks/002-01.sh)
# ---------------------------------------------------------------------------

fp_stmt=0
fp_s=0
fp_e=0
while read -r s e; do
  [ -n "$s" ] || continue
  para=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$para" | grep -Eqi 'frozen' \
     && printf '%s\n' "$para" | grep -Eqi 'fingerprint' \
     && printf '%s\n' "$para" \
          | grep -Eqi 'unchanged|not (been )?modified|untouched|still (identical|matches)|remains (identical|unchanged)|same (value|fingerprint)|matches (the )?(recorded|002-01)'; then
    fp_stmt=1
    fp_s=$s; fp_e=$e
    break
  fi
done <<EOF
$RANGES
EOF

if [ "$fp_stmt" -eq 1 ] && has_tag "$(line_range "$REPORT" "$fp_s" "$fp_e")"; then
  ok "AC 4: the section states the frozen-tree fingerprint is unchanged, with an evidence tag (lines $fp_s-$fp_e)"
elif [ "$fp_stmt" -eq 1 ]; then
  bad "AC 4: the frozen-tree-fingerprint statement (lines $fp_s-$fp_e) carries no evidence tag"
else
  bad "AC 4: the section does not state that the frozen-tree fingerprint (002-01) is unchanged"
fi

# ---------------------------------------------------------------------------
# DoD — flood source location (probes/ or the scratch copy, C-133/C-130)
# ---------------------------------------------------------------------------

src_stmt=0
src_s=0
src_e=0
while read -r s e; do
  [ -n "$s" ] || continue
  para=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$para" | grep -Eqi 'flood (source|sender|generator|traffic)|standalone (udp )?sender|(throwaway )?probe' \
     && printf '%s\n' "$para" | grep -Eqi 'probes/|probe directory|scratch'; then
    src_stmt=1
    src_s=$s; src_e=$e
    break
  fi
done <<EOF
$RANGES
EOF

if [ "$src_stmt" -eq 1 ] && has_tag "$(line_range "$REPORT" "$src_s" "$src_e")"; then
  ok "DoD: the flood source location is stated (probes/ or the scratch copy), with an evidence tag (lines $src_s-$src_e)"
elif [ "$src_stmt" -eq 1 ]; then
  bad "DoD: the flood-source-location statement (lines $src_s-$src_e) carries no evidence tag"
else
  bad "DoD: the section does not state where the flood source lives (a probe under probes/ per C-133, or a scratch-copy option)"
fi

# ---------------------------------------------------------------------------
# DoD — run provenance (poc-untouched-equivalent, or the post-fix build with
# its justification), and the frozen tree neither run nor written
# ---------------------------------------------------------------------------

prov=0
prov_s=0
prov_e=0
prov_shape=""
while read -r s e; do
  [ -n "$s" ] || continue
  para=$(line_range "$REPORT" "$s" "$e")
  printf '%s\n' "$para" | grep -Eqi 'run|loopback' || continue
  if printf '%s\n' "$para" | grep -Eqi 'poc-untouched' \
     && printf '%s\n' "$para" | grep -Eqi 'scratch|checkout|poc-scratch|tag' \
     && printf '%s\n' "$para" | grep -Eqi 'byte-identical|content-identical|identical|equivalen|fingerprint'; then
    prov=1
    prov_s=$s; prov_e=$e
    prov_shape="poc-untouched-equivalent"
    break
  fi
  if printf '%s\n' "$para" | grep -Eqi 'post-fix|003-03|003-04|current (working )?(tree|build)' \
     && printf '%s\n' "$para" | grep -Eqi 'scratch|frozen|working tree|checkout' \
     && printf '%s\n' "$para" | grep -Eqi 'does not touch|not touch|not affected|unaffected|independent of|drain'; then
    prov=1
    prov_s=$s; prov_e=$e
    prov_shape="post-fix build, justified"
    break
  fi
done <<EOF
$RANGES
EOF

if [ "$prov" -eq 0 ]; then
  bad "DoD: the runs' provenance is not stated (poc-untouched-equivalent scratch copy, or the post-fix 003-03/003-04 build with the justification that the pacing fix does not touch the drain behaviour)"
else
  runsrc=0
  while read -r s e; do
    [ -n "$s" ] || continue
    para=$(line_range "$REPORT" "$s" "$e")
    if printf '%s\n' "$para" | grep -Eqi 'scratch' \
       && printf '%s\n' "$para" | grep -Eqi 'run|loopback' \
       && printf '%s\n' "$para" | grep -Eqi 'frozen' \
       && printf '%s\n' "$para" | grep -Eqi 'not|never|rather|instead|outside|untouched|unmodified|left alone|neither'; then
      runsrc=1
      break
    fi
  done <<EOF
$RANGES
EOF
  if [ "$runsrc" -eq 1 ]; then
    ok "DoD: the runs' provenance is stated (lines $prov_s-$prov_e, $prov_shape), with the frozen tree neither run nor written"
  else
    bad "DoD: the provenance paragraph does not state that the frozen tree was neither run nor written"
  fi
  if has_tag "$(line_range "$REPORT" "$prov_s" "$prov_e")"; then
    ok "DoD: the provenance statement carries an evidence tag"
  else
    bad "DoD: the provenance statement (lines $prov_s-$prov_e) carries no evidence tag"
  fi
fi

# ---------------------------------------------------------------------------
# DoD — evidence tags on pointer-bearing lines; [measured] evidence present
# ---------------------------------------------------------------------------

tag_problems=0
k=$((e_no + 1))
while [ "$k" -lt "$e_end" ]; do
  l=$(sed -n "${k}p" "$REPORT")
  if printf '%s\n' "$l" | grep -Eq '[A-Za-z0-9_./-]+\.rs:[0-9]+' \
     && ! is_inside_fence_e "$k"; then
    if ! has_tag "$(line_range "$REPORT" "$((k - 1))" "$((k + 1))")"; then
      bad "DoD: line $k carries a file:line pointer but no evidence tag on itself or an adjacent line"
      tag_problems=$((tag_problems + 1))
    fi
  fi
  k=$((k + 1))
done
[ "$tag_problems" -eq 0 ] && ok "DoD: every pointer-bearing line carries an evidence tag"

if has_measured "$SECTION"; then
  ok "DoD: section (e) carries [measured] evidence"
else
  bad "DoD: section (e) carries no [measured] evidence"
fi

# ---------------------------------------------------------------------------
# DoD — if diagnostic logging was added, it lives in the scratch copy and a
# logging-only companion diff sits beside the report (002-07 convention)
# ---------------------------------------------------------------------------

log_claim_re='temporary (log|logging)|diagnostic (log|logging)|debug (log|logging|print)|instrument(ed|ation)|eprintln|dbg!|println!|added (a )?log|added logging|logging was added|log of (the )?(deadline|tick|ticker)|ticker (log|logging)|scratch-copy log'

if printf '%s\n' "$SECTION" | grep -Eqi "$log_claim_re"; then
  log_place_ok=0
  while read -r s e; do
    [ -n "$s" ] || continue
    para=$(line_range "$REPORT" "$s" "$e")
    if printf '%s\n' "$para" | grep -Eqi "$log_claim_re" \
       && printf '%s\n' "$para" | grep -Eqi 'scratch' \
       && printf '%s\n' "$para" | grep -Eqi 'frozen|not in the frozen' \
       && printf '%s\n' "$para" | grep -Eqi 'not|never|rather|instead|outside|untouched|unmodified|left alone|neither'; then
      log_place_ok=1
      break
    fi
  done <<EOF
$RANGES
EOF
  if [ "$log_place_ok" -eq 1 ]; then
    ok "DoD: the added logging is stated to live in the scratch copy, not the frozen tree"
  else
    bad "DoD: logging is mentioned but no paragraph says it lived in the scratch copy and not in the frozen tree"
  fi

  find_diff() {
    local f
    for f in "$ARCH"/*002-10*.diff "$ARCH"/*002-10*.patch; do
      [ -f "$f" ] || continue
      printf '%s\n' "$f"
      return 0
    done
    # Fallback: a diff whose content identifies this ticket's logging
    # (002-10, or the ticker-lateness logging). Deliberately NOT the bare
    # word "flood": the committed 002-07 diff mentions "flooding" in a
    # comment and must not be picked up here.
    for f in "$ARCH"/*.diff "$ARCH"/*.patch; do
      [ -f "$f" ] || continue
      if grep -Eq '002-10|ticker[ _-]?laten' "$f"; then
        printf '%s\n' "$f"
        return 0
      fi
    done
    return 1
  }

  DIFF_FILE=$(find_diff || true)
  if [ -z "$DIFF_FILE" ]; then
    bad "DoD: logging is claimed but no companion scratch diff exists beside the report (expect *002-10*.diff or *.patch)"
  else
    diff_rel=${DIFF_FILE#"$REPO_ROOT"/}
    if [ ! -s "$DIFF_FILE" ]; then
      bad "DoD: companion diff is empty: $diff_rel"
    elif ! is_tracked "$diff_rel"; then
      bad "DoD: companion diff exists but is not committed: $diff_rel"
    else
      ok "DoD: companion diff present and committed: $diff_rel"
    fi

    if grep -Eq '^(diff --git |--- |\+\+\+ |Index: )' "$DIFF_FILE"; then
      ok "DoD: diff carries a unified-diff header"
    else
      bad "DoD: $diff_rel is not a unified diff (no ---/+++/diff --git/Index: header)"
    fi

    if grep -Eq '^\+[^+].*(eprintln!|println!|print!|dbg!|log::|tracing|log_line|stderr|stdout)' "$DIFF_FILE"; then
      ok "DoD: diff adds at least one logging line"
    else
      bad "DoD: diff has no added line with a logging cue (eprintln!, println!, dbg!, log::, tracing, stderr/stdout, ...)"
    fi

    if grep -qF "$FROZEN_ROOT" "$DIFF_FILE"; then
      bad "DoD: diff names the frozen tree ($FROZEN_ROOT); it must reference only scratch-copy paths"
    else
      ok "DoD: diff does not name the frozen tree"
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
            bad "DoD: diff header references a non-scratch absolute path: $p"
            path_bad=$((path_bad + 1))
            ;;
        esac
      done
    done < "$DIFF_FILE"
    [ "$path_bad" -eq 0 ] && ok "DoD: diff header/path lines reference only scratch-copy paths"

    if grep -Eq '^(diff --git .*(target/|experiments/)|Only in .*(target|experiments))' "$DIFF_FILE"; then
      bad "DoD: diff touches target/ or experiments/, not logging only"
    else
      ok "DoD: diff does not touch target/ or experiments/"
    fi
  fi

  logging_stmt=0
  while read -r s e; do
    [ -n "$s" ] || continue
    para=$(line_range "$REPORT" "$s" "$e")
    if printf '%s\n' "$para" \
         | grep -Eqi 'logging[- ]only|only (adds? )?logging|no behaviou?r(al)? change|behaviou?r[- ]preserving|does not change (the )?behaviou?r'; then
      logging_stmt=1
      break
    fi
  done <<EOF
$RANGES
EOF
  if [ "$logging_stmt" -eq 1 ]; then
    ok "DoD: the section states the diff is logging-only / changes no behaviour"
  else
    bad "DoD: the section does not state that the scratch diff is logging-only / changes no behaviour"
  fi
else
  ok "DoD: no diagnostic logging is claimed, so no scratch-copy statement or companion diff is required"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
