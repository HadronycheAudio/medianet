#!/usr/bin/env bash
# checks/002-12.sh — checks for ticket 002-12
# "Clean sync-rate baseline".
#
# The deliverable is the "### ... 002-12" subsection of section (e) Run
# confirmations of doc/design/reports/archaeology/REPORT-0b.md (deliverable
# (e) 3 of epic 002). It re-measures the POC's sync rate at --sync-hz 50 and
# 1000 from clean, single-writer metrics files (one per module, 3 runs of
# >= 30 s each, unprivileged), states the receive-timeout duration and the
# host CONFIG_HZ, names the pacing mechanism at module.rs:1126, gives the
# P-02 replacement figures for 002-13 and states a reusable command for
# 002-14. Section (e) keeps exactly one verdict line, in the pending form;
# 002-13 writes the final verdict.
#
# This script reads the report and, when one exists, a companion diff. It
# never writes anywhere, never runs the POC and never touches the frozen
# POC or the scratch copy. Running the measurements is the investigator's
# work, not this check's.
#
# The conventions below make the prose machine-checkable without fixing its
# wording. The subsection runs from its "### " heading (naming 002-12) to the
# next heading of any level, outside code fences. "Paragraph" below means a
# run of prose or table lines, outside fences and headings, between blank
# lines.
#
# * (AC 1, single-writer.) Quote the 002-09 checker output, one line per
#   metrics file, in the form
#     <path>: single-writer yes (lines N, parseable P, unix_ms violations V)
#   Every quoted file must read "yes", with P = N - 1 (every data line after
#   the header parses) and V = 0. The six runs (3 at 50 Hz, 3 at 1000 Hz) of
#   four modules give 24 files: at least 24 distinct paths must be quoted,
#   each module name (sender, synth1, synth2, mixer) in at least 6 of them.
#   No "single-writer no" line may appear. If no checker line is quoted, a
#   hand-rule paragraph is accepted instead: it names the hand check ("by
#   hand" / "hand rule"), unix_ms and its monotonicity, parsing, 24 files,
#   and carries a tag.
# * (AC 2, rates.) One line per rate, a prose or table line that carries the
#   rate token (`--sync-hz 50`, "50 Hz", ... for 50; the same for 1000), the
#   words "mean", "range" and "exchange" (or "/s"), a percentage, and
#   [measured] on the same line. The range is written with two numbers after
#   "range" (e.g. "range 37.2–38.6"). The percentage must equal mean / rate x
#   100 within 1 percentage point. The subsection states that the figures are
#   the duration-corrected ones (sum of served / covered span / followers) or
#   the naive ones, and names the metrics column (served or asked) in a
#   paragraph with its module.rs:<n> pointer, and the clock holder ("holder").
#   "3 runs" (or "three runs") appears in the subsection.
# * (AC 3, timeout and CONFIG_HZ.) A paragraph with "timeout" and the
#   effective receive-timeout duration in ms ("effective ... 13 ms", within 60
#   characters, either order is not needed: "effective" precedes the number).
#   CONFIG_HZ=<n> appears in the subsection, with <n> equal to the value in the
#   report's Environment section.
# * (AC 4, pacing mechanism.) A paragraph carrying module.rs:1126, a pacing
#   word (pac, deadline or timeout) and [measured]: the run confirmation of
#   the mechanism. The subsection says "poc-untouched" for the numbering.
# * (AC 5, P-02 replacement.) A line carrying "P-02", "replac" and [measured]
#   that also carries a figure within 1 of the 50 Hz mean and a figure within
#   1 of the 1000 Hz mean (both means as written in the rate lines).
# * (Method and source.) A paragraph naming the scratch copy (scratch or
#   poc-scratch), the method (distinct, separate, each module, per module, or
#   own metrics path/process) and metrics, carrying a tag. The subsection
#   says "poc-untouched" with identical, equivalent or fingerprint. A
#   paragraph says the frozen tree was neither run nor written. The subsection
#   says "unprivileged" and "strict".
# * (Reusable command for 002-14.) A line carrying --sync-hz and --metrics;
#   the subsection names 002-14 and calls the command reusable or a script.
#   No --sync-hz command may point at the frozen POC tree.
# * (Duration.) Runs are stated as >= 30 s ("30 s", "--duration 30", "at least
#   30", ">= 30 s").
# * (Evidence tags.) Every paragraph that carries a figure (a digit) carries
#   an evidence tag on itself or on the line right after it (the "[measured]"
#   trailer used after fenced blocks). The subsection carries [measured].
# * (Logging, 002-07 convention, only if claimed.) If the subsection says
#   diagnostic logging was added (e.g. "scratch-copy log", "ticklog"), a
#   paragraph must place it in the scratch copy and not the frozen tree, a
#   companion diff must sit beside the report (*002-12*.diff/.patch, or a diff
#   naming 002-12), non-empty, tracked, unified, touching only scratch paths,
#   and the subsection must say the diff is logging-only.
# * (Section (e) verdict, as checks/002-10.sh.) Section (e) keeps EXACTLY ONE
#   "verdict:" line in one of the allowed forms: the placeholder "pending"
#   (the state until 002-13), or the final form with "mechanism:" or
#   "explanation:". The 002-12 subsection adds no second verdict line.
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

# Subsection text contains a match of the ERE $1 (case-insensitive).
sub_has() {
  printf '%s\n' "$SUBTXT" | grep -Eqi -- "$1"
}

# Some paragraph of the subsection matches ALL the ERE patterns given (case-
# insensitive). Sets PARA_HIT to "start end" on success.
para_all() {
  local s e p pat good
  while read -r s e; do
    [ -n "$s" ] || continue
    p=$(line_range "$REPORT" "$s" "$e")
    good=1
    for pat in "$@"; do
      if ! printf '%s\n' "$p" | grep -Eqi -- "$pat"; then
        good=0
        break
      fi
    done
    if [ "$good" -eq 1 ]; then
      PARA_HIT="$s $e"
      return 0
    fi
  done <<EOF
$PARAS
EOF
  return 1
}

# ---------------------------------------------------------------------------
# Locate section (e) and its boundary (the next "## " heading)
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-12 does not exist yet.\n'
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

# ---------------------------------------------------------------------------
# Section (e) verdict line: exactly one, pending form (as checks/002-10.sh).
# Checked over the whole section, so the 002-12 subsection adds none.
# ---------------------------------------------------------------------------

verdict_count=$(printf '%s\n' "$SECTION" \
  | grep -Ec '^[[:space:]]*(\*\*)?verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "section (e) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "section (e) has $verdict_count 'verdict:' lines; exactly one is required (002-12 adds none; 002-13 writes the final verdict)"
else
  verdict_line=$(printf '%s\n' "$SECTION" \
    | grep -E '^[[:space:]]*(\*\*)?verdict:' | head -n 1)
  verdict_body=$(printf '%s\n' "$verdict_line" \
    | sed -E 's/^[[:space:]]*(\*\*)?verdict:[[:space:]]*//; s/[[:space:]]*\*\*[[:space:]]*$//')
  if printf '%s' "$verdict_body" | grep -Eqi '^pending$'; then
    ok "section (e) has exactly one verdict line (pending; 002-13 owns the section verdict)"
  elif printf '%s' "$verdict_body" | grep -Eqi 'mechanism:'; then
    ok "section (e) has exactly one verdict line (final form, mechanism)"
  elif printf '%s' "$verdict_body" | grep -Eqi 'explanation:'; then
    ok "section (e) has exactly one verdict line (final form, explanation)"
  else
    bad "section (e) has one verdict line but it is not in an allowed form: $verdict_line"
  fi
fi

# ---------------------------------------------------------------------------
# Locate the 002-12 subsection
# ---------------------------------------------------------------------------

s_no=$(awk -v s="$e_no" -v t="$e_end" '
  NR > s && NR < t && /^###[[:space:]]/ && /002-12/ { print NR; exit }' "$REPORT")
if [ -z "$s_no" ]; then
  bad "section (e) has no '### ... 002-12' subsection: the clean sync-rate baseline is absent"
  printf '\n%d passed, %d failed; the 002-12 content does not exist yet.\n' "$pass" "$fail"
  exit 1
fi

s_end=$(awk -v s="$s_no" -v t="$e_end" '
  NR <= s { next }
  NR >= t { print NR; exit }
  /^[[:space:]]*```/ { f = !f; next }
  !f && /^#+[[:space:]]/ { print NR; exit }
' "$REPORT")
if [ -z "$s_end" ]; then
  s_end=$e_end
fi

SUB_FIRST=$((s_no + 1))
SUB_LAST=$((s_end - 1))
SUBTXT=$(line_range "$REPORT" "$SUB_FIRST" "$SUB_LAST")
ok "002-12 subsection found at line $s_no, body lines $SUB_FIRST-$SUB_LAST"

# Prose paragraphs of the subsection: "start end" absolute report line
# numbers, fenced blocks and headings excluded.
PARAS=$(awk -v a="$SUB_FIRST" -v b="$SUB_LAST" '
  function flush() { if (st) { print st, en; st = 0 } }
  NR < a { next }
  NR > b { exit }
  /^[[:space:]]*```/ { flush(); f = !f; next }
  f { next }
  /^[[:space:]]*$/ || /^#+[[:space:]]/ { flush(); next }
  { if (!st) st = NR; en = NR }
  END { flush() }
' "$REPORT")

# ---------------------------------------------------------------------------
# DoD — evidence tags: every figure-bearing paragraph carries a tag on itself
# or on the line right after it; the subsection carries [measured].
# ---------------------------------------------------------------------------

untagged=""
while read -r s e; do
  [ -n "$s" ] || continue
  p=$(line_range "$REPORT" "$s" "$e")
  printf '%s\n' "$p" | grep -q '[0-9]' || continue
  nxt=$(line_range "$REPORT" "$((e + 1))" "$((e + 1))")
  if ! has_tag "$p" && ! has_tag "$nxt"; then
    untagged="$untagged $s-$e"
  fi
done <<EOF
$PARAS
EOF

if [ -z "$untagged" ]; then
  ok "DoD: every figure-bearing paragraph of the 002-12 subsection carries an evidence tag"
else
  bad "DoD: figure-bearing paragraph(s) without an evidence tag (report lines:$untagged)"
fi

if has_measured "$SUBTXT"; then
  ok "DoD: the 002-12 subsection carries [measured] evidence"
else
  bad "DoD: the 002-12 subsection carries no [measured] evidence"
fi

# ---------------------------------------------------------------------------
# AC 1 — every metrics file used is single-writer: 100 % of data lines parse
# (parseable = lines - 1, the header) and 0 unix_ms regressions
# ---------------------------------------------------------------------------

quoted=$(printf '%s\n' "$SUBTXT" \
  | grep -E ': single-writer (yes|no) \(lines [0-9]+, parseable [0-9]+, unix_ms violations [0-9]+\)' || true)

if [ -n "$quoted" ]; then
  rows=$(printf '%s\n' "$quoted" | awk '
    match($0, /: single-writer (yes|no) \(lines [0-9]+, parseable [0-9]+, unix_ms violations [0-9]+\)/) {
      path = substr($0, 1, RSTART - 1)
      sub(/^[ \t`*|-]+/, "", path)
      tail = substr($0, RSTART)
      yes = (tail ~ /: single-writer yes/) ? "yes" : "no"
      nums = tail
      gsub(/[^0-9 ]/, " ", nums)
      n = split(nums, a, " ")
      print path "|" yes "|" a[1] "|" a[2] "|" a[3]
    }')
  stats=$(printf '%s\n' "$rows" | awk -F'|' '
    {
      if (seen[$1]++) next
      d++
      if ($2 == "yes" && $3 - 1 == $4 && $5 == 0) c++
      if ($1 ~ /sender/) s++
      if ($1 ~ /synth1/) s1++
      if ($1 ~ /synth2/) s2++
      if ($1 ~ /mixer/) mx++
    }
    END { print d + 0, c + 0, s + 0, s1 + 0, s2 + 0, mx + 0 }')
  set -- $stats
  distinct=$1; clean=$2; c_send=$3; c_s1=$4; c_s2=$5; c_mx=$6
  if [ "$distinct" -ge 24 ] && [ "$clean" -eq "$distinct" ] \
     && [ "$c_send" -ge 6 ] && [ "$c_s1" -ge 6 ] && [ "$c_s2" -ge 6 ] && [ "$c_mx" -ge 6 ]; then
    ok "AC 1: $distinct metrics files quoted from the 002-09 checker, all single-writer yes, every data line parses (parseable = lines - 1) and 0 unix_ms violations (sender $c_send, synth1 $c_s1, synth2 $c_s2, mixer $c_mx)"
  else
    bad "AC 1: the quoted checker lines must cover all 24 files of the six runs (distinct $distinct, clean $clean, per module sender $c_send synth1 $c_s1 synth2 $c_s2 mixer $c_mx; each needs >= 24 distinct, all clean, >= 6 per module)"
  fi
  if printf '%s\n' "$SUBTXT" | grep -Eq ': single-writer no '; then
    bad "AC 1: a quoted metrics file reads 'single-writer no'"
  fi
else
  hand=0
  if para_all 'by hand|hand rule|hand-check|hand check' 'unix_ms' \
       'monotonic|regress|never decrease' 'pars' '24|twenty-four' \
       '\[(code|measured|kernel-doc|memory|decision)\]'; then
    hand=1
  fi
  if [ "$hand" -eq 1 ]; then
    ok "AC 1: single-writer stated by the hand rule for 24 files (parse and unix_ms monotonicity), tagged"
  else
    bad "AC 1: no single-writer statement: quote the 002-09 checker lines (<path>: single-writer yes (lines N, parseable P, unix_ms violations V)) for all 24 files, or give the hand rule for them"
  fi
fi

# ---------------------------------------------------------------------------
# AC 2 — achieved exchanges/s at 50 and 1000 Hz: mean, range over 3 runs,
# percentage of the configured rate, all [measured] on the rate's line
# ---------------------------------------------------------------------------

# Prose and table lines of the subsection (fences excluded) that carry the
# rate token R. Prints "lineno mean pct range_ok measured" per candidate; "-"
# when a part is missing.
rate_scan() {
  awk -v a="$SUB_FIRST" -v b="$SUB_LAST" -v R="$1" '
    NR < a { next }
    NR > b { exit }
    /^[[:space:]]*```/ { f = !f; next }
    f { next }
    {
      low = tolower($0)
      if (!match($0, "(^|[^0-9.,])" R "([^0-9.,%]|$)")) next
      if (low !~ /mean/ || low !~ /range/ || $0 !~ /%/) next
      if (low !~ /exchange/ && low !~ /\/s/ && low !~ /per second/) next
      mean = "-"; pct = "-"; rng = 0
      meas = (index($0, "[measured]") > 0) ? 1 : 0
      if (match(low, /mean[^0-9]*[0-9]+([.,][0-9]+)?/)) {
        m = substr(low, RSTART, RLENGTH); gsub(/[^0-9.,]/, "", m); gsub(/,/, ".", m); mean = m
      }
      if (match($0, /[0-9]+([.,][0-9]+)?[ ]*%/)) {
        p = substr($0, RSTART, RLENGTH); sub(/[ ]*%$/, "", p); gsub(/,/, ".", p); pct = p
      }
      if (match(low, /range[^0-9]*[0-9]+([.,][0-9]+)?[^0-9]+[0-9]+/)) rng = 1
      print NR, mean, pct, rng, meas
    }
  ' "$REPORT"
}

# Prints "ok <mean> <pct>", "inconsistent <mean> <pct>" or "incomplete".
rate_result() {
  local R="$1" cand n mean pct rng meas
  cand=$(rate_scan "$R")
  [ -n "$cand" ] || { echo incomplete; return; }
  while read -r n mean pct rng meas; do
    [ "$mean" != "-" ] && [ "$pct" != "-" ] && [ "$rng" = 1 ] && [ "$meas" = 1 ] || continue
    if awk -v m="$mean" -v p="$pct" -v R="$R" \
         'BEGIN { d = p - m / R * 100; if (d < 0) d = -d; exit !(d <= 1.0) }'; then
      echo "ok $mean $pct"
    else
      echo "inconsistent $mean $pct"
    fi
    return
  done <<EOF
$cand
EOF
  echo incomplete
}

R50=$(rate_result 50)
R1000=$(rate_result 1000)

M50=""
M1000=""
case "$R50" in
  ok*) set -- $R50; M50=$2
       ok "AC 2: achieved exchanges/s at --sync-hz 50 reported as mean $2 with range and $3 % of the configured rate, [measured]" ;;
  inconsistent*) set -- $R50; M50=$2
       bad "AC 2: the 50 Hz percentage ($3 %) is not mean/50 x 100 for mean $2" ;;
  *) bad "AC 2: no [measured] line for --sync-hz 50 giving the mean, a 'range A–B' of two numbers over the 3 runs, and the percentage of the configured rate (the line needs the words mean, range, exchange/s and a %)" ;;
esac
case "$R1000" in
  ok*) set -- $R1000; M1000=$2
       ok "AC 2: achieved exchanges/s at --sync-hz 1000 reported as mean $2 with range and $3 % of the configured rate, [measured]" ;;
  inconsistent*) set -- $R1000; M1000=$2
       bad "AC 2: the 1000 Hz percentage ($3 %) is not mean/1000 x 100 for mean $2" ;;
  *) bad "AC 2: no [measured] line for --sync-hz 1000 giving the mean, a 'range A–B' of two numbers over the 3 runs, and the percentage of the configured rate (the line needs the words mean, range, exchange/s and a %)" ;;
esac

if sub_has '(3|three) (runs?|repeats|repetitions)'; then
  ok "AC 2: the mean and range are stated over 3 runs"
else
  bad "AC 2: the subsection does not state that the mean and range are over 3 runs ('3 runs')"
fi

if sub_has 'corrected|naive|covered span|covered-span'; then
  ok "AC 2: the subsection states which figure is reported (duration-corrected or naive)"
else
  bad "AC 2: the subsection does not state whether the figures are duration-corrected or naive"
fi

if para_all '(served|asked)' 'module\.rs:[0-9]+' && sub_has 'holder'; then
  ok "AC 2: the metrics column (served or asked) and its module.rs pointer are named, from the clock holder's rows"
else
  bad "AC 2: name the metrics column used (served or asked) with its module.rs:<n> pointer, and say the figures come from the clock holder's peer metrics"
fi

# ---------------------------------------------------------------------------
# AC 3 — effective receive-timeout duration in ms, and the host CONFIG_HZ
# ---------------------------------------------------------------------------

if para_all 'effective[^;]{0,60}[0-9]+(\.[0-9]+)? ?ms' 'timeout'; then
  ok "AC 3: the effective receive-timeout duration is given in ms (lines $PARA_HIT)"
else
  bad "AC 3: give the effective receive-timeout duration in ms ('effective timeout ... 13 ms')"
fi

ENV_HZ=$(grep -Eo 'CONFIG_HZ=[0-9]+' "$REPORT" | head -n 1 | grep -Eo '[0-9]+$')
SUB_HZ=$(printf '%s\n' "$SUBTXT" | grep -Eo 'CONFIG_HZ[^0-9]{0,6}[0-9]+' \
  | tail -n 1 | grep -Eo '[0-9]+$' || true)
if [ -z "$SUB_HZ" ]; then
  bad "AC 3: the host CONFIG_HZ is not recorded in the subsection (e.g. 'CONFIG_HZ=250')"
elif [ -n "$ENV_HZ" ] && [ "$SUB_HZ" != "$ENV_HZ" ]; then
  bad "AC 3: the subsection's CONFIG_HZ=$SUB_HZ does not match the Environment section's CONFIG_HZ=$ENV_HZ"
else
  ok "AC 3: the host CONFIG_HZ=$SUB_HZ is recorded and matches the Environment section"
fi

# ---------------------------------------------------------------------------
# AC 4 — run confirmation naming the pacing mechanism at module.rs:1126
# (poc-untouched numbering)
# ---------------------------------------------------------------------------

if para_all 'module\.rs:1126' '(pac|deadline|timeout)' '\[measured\]'; then
  ok "AC 4: the run confirmation names the pacing mechanism at module.rs:1126, [measured] (lines $PARA_HIT)"
else
  bad "AC 4: no [measured] statement naming the pacing mechanism with module.rs:1126"
fi

if sub_has 'poc-untouched'; then
  ok "AC 4: the subsection states the poc-untouched numbering of its pointers"
else
  bad "AC 4: the subsection does not say its pointers use poc-untouched numbering"
fi

# ---------------------------------------------------------------------------
# AC 5 — P-02 replacement: a tagged line with the clean 50 and 1000 Hz figures
# ---------------------------------------------------------------------------

p02_line=""
if [ -n "$M50" ] && [ -n "$M1000" ]; then
  p02_line=$(awk -v a="$SUB_FIRST" -v b="$SUB_LAST" -v m50="$M50" -v m1k="$M1000" '
    function absv(x) { return x < 0 ? -x : x }
    NR < a { next }
    NR > b { exit }
    /^[[:space:]]*```/ { f = !f; next }
    f { next }
    {
      low = tolower($0)
      if (low !~ /p-02/ || low !~ /replac/ || index($0, "[measured]") == 0) next
      line = $0; h50 = 0; h1k = 0
      while (match(line, /[0-9]+([.,][0-9]+)?/)) {
        v = substr(line, RSTART, RLENGTH); gsub(/,/, ".", v); v = v + 0
        if (absv(v - m50) <= 1) h50 = 1
        if (absv(v - m1k) <= 1) h1k = 1
        line = substr(line, RSTART + RLENGTH)
      }
      if (h50 && h1k) { print NR; exit }
    }
  ' "$REPORT")
fi
if [ -n "$p02_line" ]; then
  ok "AC 5: a tagged P-02 replacement line carries the clean 50 Hz and 1000 Hz figures (line $p02_line)"
else
  bad "AC 5: no tagged 'P-02 replacement' line carrying the clean 50 Hz and 1000 Hz figures in a form that replaces P-02 (002-13 copies it)"
fi

# ---------------------------------------------------------------------------
# Method and source: scratch copy with a distinct metrics path per module (or
# separate processes); the frozen tree neither run nor written; unprivileged;
# not strict mode
# ---------------------------------------------------------------------------

if para_all 'scratch' '(distinct|separate|each module|per[- ]module|own)' 'metrics' \
     '\[(code|measured|kernel-doc|memory|decision)\]'; then
  ok "DoD: the method and source are stated (scratch copy, one metrics path per module or separate processes), tagged (lines $PARA_HIT)"
else
  bad "DoD: state the method and source: the scratch copy (poc-scratch) with a distinct --metrics path per module, or each module as a separate process, tagged"
fi

if sub_has 'poc-untouched' && sub_has 'identical|equivalent|fingerprint'; then
  ok "DoD: the source is shown poc-untouched-equivalent (identical, equivalent or fingerprint)"
else
  bad "DoD: the source is not shown poc-untouched-equivalent (poc-untouched + identical/equivalent/fingerprint)"
fi

if para_all 'frozen[^.]*(neither|never|not |untouched|unmodified)[^.]*(run|written|modified)'; then
  ok "DoD: the frozen tree is stated neither run nor written (lines $PARA_HIT)"
else
  bad "DoD: state that the frozen tree at $FROZEN_ROOT was neither run nor written"
fi

if sub_has 'unprivileged' && sub_has 'strict'; then
  ok "DoD: the runs are stated unprivileged and not in strict mode"
else
  bad "DoD: state that the runs were unprivileged and not in strict mode (C-147)"
fi

# ---------------------------------------------------------------------------
# Reusable command for 002-14; durations >= 30 s; 3 runs per rate
# ---------------------------------------------------------------------------

if printf '%s\n' "$SUBTXT" | grep -- '--sync-hz' | grep -q -- '--metrics'; then
  if printf '%s\n' "$SUBTXT" \
       | grep -Eq "(cd[[:space:]]+|/target/release/)${FROZEN_ROOT}([^-[:alnum:]_]|$)|${FROZEN_ROOT}/target/" \
     || printf '%s\n' "$SUBTXT" | grep -- '--sync-hz' \
       | grep -Eq "${FROZEN_ROOT}([^-[:alnum:]_]|$)"; then
    bad "DoD: a command of the subsection runs from the frozen tree ($FROZEN_ROOT); it must run from the scratch copy"
  elif sub_has '002-14' && sub_has 'reusable|reuse|script'; then
    ok "DoD: a reusable command for 002-14 is stated (--sync-hz and --metrics), from the scratch copy"
  else
    bad "DoD: the command for 002-14 must be called reusable (or a script) and named for 002-14"
  fi
else
  bad "DoD: no reusable command stated (a line with --sync-hz and --metrics) for 002-14"
fi

if sub_has '(--duration[ =]*(3[0-9]|[4-9][0-9]|[0-9]{3,})([^0-9.]|$))|((^|[^0-9.])(3[0-9]|[4-9][0-9]|[0-9]{3,})[ ]*(s|sec|secs|seconds)([^[:alnum:]]|$))|(at least 30)|((>=)[ ]*30)'; then
  ok "DoD: the run durations are stated as >= 30 s"
else
  bad "DoD: the run duration is not stated as >= 30 s ('30 s', '--duration 30', 'at least 30')"
fi

# ---------------------------------------------------------------------------
# DoD — diagnostic logging, if claimed, lives in the scratch copy, with a
# logging-only companion diff beside the report (002-07 convention)
# ---------------------------------------------------------------------------

log_claim_re='diagnostic (log|logging)|temporary (log|logging)|added (a )?(log|logging)|logging was added|scratch-copy log|ticklog|hoplog|eprintln'

if sub_has "$log_claim_re"; then
  log_place=0
  while read -r s e; do
    [ -n "$s" ] || continue
    p=$(line_range "$REPORT" "$s" "$e")
    if printf '%s\n' "$p" | grep -Eqi "$log_claim_re" \
       && printf '%s\n' "$p" | grep -Eqi 'scratch' \
       && printf '%s\n' "$p" | grep -Eqi 'frozen' \
       && printf '%s\n' "$p" | grep -Eqi 'not|never|outside|untouched|unmodified|neither'; then
      log_place=1
      break
    fi
  done <<EOF
$PARAS
EOF
  if [ "$log_place" -eq 1 ]; then
    ok "DoD: the added logging is stated to live in the scratch copy, not the frozen tree"
  else
    bad "DoD: logging is mentioned but no paragraph says it lived in the scratch copy and not in the frozen tree"
  fi

  DIFF_FILE=""
  for f in "$ARCH"/*002-12*.diff "$ARCH"/*002-12*.patch; do
    [ -f "$f" ] && { DIFF_FILE="$f"; break; }
  done
  if [ -z "$DIFF_FILE" ]; then
    for f in "$ARCH"/*.diff "$ARCH"/*.patch; do
      [ -f "$f" ] || continue
      if grep -Eq '002-12' "$f"; then
        DIFF_FILE="$f"
        break
      fi
    done
  fi

  if [ -z "$DIFF_FILE" ]; then
    bad "DoD: logging is claimed but no companion scratch diff exists beside the report (expect *002-12*.diff or *.patch)"
  else
    diff_rel=${DIFF_FILE#"$REPO_ROOT"/}
    if [ ! -s "$DIFF_FILE" ]; then
      bad "DoD: companion diff is empty: $diff_rel"
    elif ! is_tracked "$diff_rel"; then
      bad "DoD: companion diff exists but is not committed: $diff_rel"
    elif ! grep -Eq '^(diff --git |--- |\+\+\+ |Index: )' "$DIFF_FILE"; then
      bad "DoD: $diff_rel is not a unified diff"
    elif grep -qF "$FROZEN_ROOT" "$DIFF_FILE"; then
      bad "DoD: $diff_rel names the frozen tree ($FROZEN_ROOT)"
    elif ! grep -Eq '^\+[^+].*(eprintln!|println!|print!|dbg!|log::|tracing|log_line|stderr|stdout)' "$DIFF_FILE"; then
      bad "DoD: $diff_rel adds no logging line"
    else
      ok "DoD: companion diff present, committed, unified, logging-only and scratch-only: $diff_rel"
    fi
  fi

  if sub_has 'logging[- ]only|only (adds? )?logging|no behaviou?r(al)? change|behaviou?r[- ]preserving|changes no behaviou?r'; then
    ok "DoD: the subsection states the diff is logging-only / changes no behaviour"
  else
    bad "DoD: the subsection does not state that the scratch diff is logging-only / changes no behaviour"
  fi
else
  ok "DoD: no diagnostic logging is claimed, so no scratch-copy statement or companion diff is required"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
