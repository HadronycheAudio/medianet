#!/usr/bin/env bash
# checks/002-14.sh — checks for ticket 002-14
# "Sync rate baseline at the ends of the AC-7 range".
#
# The deliverable is an addition to part (e) 3 of section (e) Run
# confirmations of doc/design/reports/archaeology/REPORT-0b.md: a
# "### ... 002-14" subsection that measures the POC's sync rate at
# --sync-hz 10 and 200 (3 runs of >= 30 s each, unprivileged, single-writer
# metrics files) and puts those two rows in ONE table with the 50 and 1000 Hz
# rows of 002-12. Each of 10, 50 and 200 Hz is marked "within 2 %" or
# "not within 2 %" against the duration-corrected mean.
#
# This script reads the report and never writes anywhere, never runs the POC
# and never touches the frozen POC or the scratch copy. The report path can be
# overridden with REPORT_0B (used by the fixture harness); the frozen-tree
# fingerprint is always recomputed from the poc-untouched tag, read-only.
#
# Conventions (wording is free, these forms are machine-checked):
#
# * (AC 1, single-writer.) Quote the 002-09 checker output, one line per
#   metrics file, in the form
#     <path>: single-writer yes (lines N, parseable P, unix_ms violations V)
#   The 10 and 200 Hz runs (3 runs x 2 rates x 4 modules) give 24 files: at
#   least 24 distinct paths, each path under /tmp/opencode/ (never inside a
#   POC tree), each module (sender, synth1, synth2, mixer) in at least 6 of
#   them, every line "yes", P = N - 1 and V = 0. No "single-writer no" line.
#   If no checker line is quoted, a hand-rule paragraph is accepted instead:
#   it names the hand check ("by hand" / "hand rule"), unix_ms, parsing,
#   "100 %", a /tmp/opencode/ path and a non-zero file count, and is tagged.
# * (AC 2 and AC 3, the rate table.) ONE markdown table in the subsection whose
#   rows carry 10, 50, 200 and 1000 Hz. Its header must have a mean column
#   labelled "corrected", a "range" column, a "% of configured" column and a
#   "within 2 %" column. Each row: [measured] on the row; the mean lies in the
#   range (two numbers, min and max); the percentage equals mean / rate x 100
#   within 0.6 percentage point; the mark is "within 2 %" exactly when
#   |mean - rate| / rate <= 0.02, else "not within 2 %". The 50 and 1000 Hz
#   means must equal the corrected means stated in the 002-12 subsection
#   (within 0.05). The 1000 Hz row is outside C-112 (10 to 200) and may read
#   "n/a (outside C-112)" instead of a mark; a bare "within 2 %" never passes.
# * (AC 2, runs.) "3 runs" (or "three runs") in the subsection; each run >= 30 s
#   (e.g. "--duration 30", "30 s", "at least 30"); "unprivileged" and "strict"
#   (C-147) both stated; the figures go to the POC reference rows ("reference
#   row" or P-nn); the metrics column (served or asked) with its module.rs:<n>
#   pointer and the clock holder named.
# * (AC 4, frozen tree.) The poc-untouched tag-tree fingerprint recomputed from
#   git equals the value recorded in the Environment section, and the 27
#   frozen experiment-data files match checks/002-01-frozen-data.sha256 (the
#   same mechanism as checks/002-01.sh). The subsection says poc-untouched and
#   fingerprint, and the frozen tree is stated neither run nor written.
# * (DoD.) Every figure-bearing paragraph carries an evidence tag and the
#   subsection carries [measured]; the run command is stated (a line with
#   --sync-hz 10 and --sync-hz 200 as commands, --group 239.72.68.44:<port> and
#   --no-tui) and is not run from the frozen tree; any paragraph saying
#   "open item" also names an owner and a falsifier.
# * (Section (e) verdict, as checks/002-12.sh.) Section (e) keeps EXACTLY ONE
#   "verdict:" line in an allowed form, and it is the LAST non-blank line of
#   the section (subsections go above it); the 002-14 subsection adds none.
#
# Exit status: 0 when every check passes, 1 otherwise.

set -u

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REPORT_REL="doc/design/reports/archaeology/REPORT-0b.md"
REPORT="${REPORT_0B:-$REPO_ROOT/$REPORT_REL}"
POC_ROOT="/home/dorian/workspace/hadropoc"
FROZEN_ROOT="$POC_ROOT"
FROZEN_DATA_MANIFEST="$REPO_ROOT/checks/002-01-frozen-data.sha256"

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

# sha256 over the sorted (path, contents) concatenation, read from the named
# tag's tree through git. Copied from checks/002-01.sh (same convention).
compute_tag_fingerprint() {
  local root="$1" tag="$2"
  (
    git -C "$root" ls-tree -r "$tag" |
      awk -F'\t' '
        {
          split($1, meta, " ");
          if (meta[2] != "blob") next;
          path = $2;
          if (path ~ /(^|\/)(target|build|\.git)(\/|$)/) next;
          if (path ~ /(^|\/)\.gitignore$/) next;
          if (path ~ /^src\// || path ~ /^tools\// || path ~ /^experiments\// ||
              path == "Cargo.toml" || path == "Cargo.lock" || path == "README.md") {
            print path "\t" meta[3];
          }
        }
      ' |
      LC_ALL=C sort |
      while IFS=$'\t' read -r path oid; do
        printf 'path:%s\0' "$path"
        git -C "$root" cat-file blob "$oid"
        printf '\0'
      done |
      sha256sum | cut -d' ' -f1
  )
}

# Print the 64-hex value on the first line matching the extended regex, or on
# the immediately following non-empty line. Copied from checks/002-01.sh.
hash_for_regex() {
  local file="$1" re="$2" line no next
  no=$(grep -Ein "$re" "$file" | cut -d: -f1 | head -n 1)
  [ -n "$no" ] || return 1
  line=$(sed -n "${no}p" "$file")
  if [[ $line =~ ([0-9a-fA-F]{64}) ]]; then
    printf '%s\n' "${BASH_REMATCH[1]}"
    return 0
  fi
  next=$((no + 1))
  while :; do
    line=$(sed -n "${next}p" "$file")
    [ -n "$line" ] || return 1
    if [[ $line =~ ([0-9a-fA-F]{64}) ]]; then
      printf '%s\n' "${BASH_REMATCH[1]}"
      return 0
    fi
    return 1
  done
}

# ---------------------------------------------------------------------------
# Locate section (e) and its boundary (the next "## " heading)
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-14 does not exist yet.\n'
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
# Section (e) verdict line: exactly one, pending form (copied from
# checks/002-12.sh, the same established form). Checked over the whole
# section, so the 002-14 subsection adds none.
# ---------------------------------------------------------------------------

verdict_count=$(printf '%s\n' "$SECTION" \
  | grep -Ec '^[[:space:]]*(\*\*)?verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "section (e) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "section (e) has $verdict_count 'verdict:' lines; exactly one is required (002-14 adds none; 002-13 writes the final verdict)"
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

# Position: the section's LAST non-blank line is its verdict line, so a
# subsection added to (e) goes above the verdict, never below it (002-14
# review, finding 2). Every accepted verdict form passes this, pending or final.
if [ "$verdict_count" -eq 1 ]; then
  last_nb=$(printf '%s\n' "$SECTION" | grep -n '[^[:space:]]' | tail -n 1 | cut -d: -f2-)
  if printf '%s\n' "$last_nb" | grep -Eq '^[[:space:]]*(\*\*)?verdict:'; then
    ok "section (e) ends with its verdict line (the last non-blank line of the section is the verdict)"
  else
    bad "section (e) verdict line is not the last non-blank line of the section (last line: $last_nb); the verdict stays last and new subsections go above it"
  fi
fi

# ---------------------------------------------------------------------------
# AC 4 — frozen tree: the poc-untouched tag-tree fingerprint is unchanged, and
# the 27 untracked experiment-data files match their manifest. Same mechanism
# as checks/002-01.sh (copied); 002-12.sh has no fingerprint code of its own.
# ---------------------------------------------------------------------------

tag_hash=$(hash_for_regex "$REPORT" 'poc-untouched tag-tree fingerprint' || true)
if [ -z "$tag_hash" ]; then
  bad "AC 4: the Environment section records no 64-hex 'poc-untouched tag-tree fingerprint' to compare with"
elif ! git -C "$POC_ROOT" rev-parse --verify --quiet 'refs/tags/poc-untouched^{commit}' >/dev/null 2>&1; then
  bad "AC 4: tag 'poc-untouched' not found in the POC repository at $POC_ROOT"
else
  computed_tag=$(compute_tag_fingerprint "$POC_ROOT" 'poc-untouched' || true)
  if [ -z "$computed_tag" ]; then
    bad "AC 4: could not compute the 'poc-untouched' tag tree fingerprint in $POC_ROOT"
  elif [ "${computed_tag,,}" = "${tag_hash,,}" ]; then
    ok "AC 4: frozen-tree fingerprint unchanged: poc-untouched tag-tree fingerprint recomputed from git matches the recorded value $computed_tag"
  else
    bad "AC 4: frozen-tree fingerprint changed (recorded=$tag_hash computed=$computed_tag)"
  fi
fi

if [ ! -f "$FROZEN_DATA_MANIFEST" ]; then
  bad "AC 4: frozen-data manifest missing: checks/002-01-frozen-data.sha256"
else
  manifest_entries=0
  manifest_unexpected=0
  manifest_mismatch=0
  while read -r expected path; do
    [ -n "${expected:-}" ] || continue
    case "$expected" in \#*) continue ;; esac
    manifest_entries=$((manifest_entries + 1))
    case "${path:-}" in
      experiments/pilot/*|experiments/run1/*|experiments/run2/*|experiments/run3/*) ;;
      *)
        bad "AC 4: manifest entry outside the frozen experiment-data dirs: ${path:-<missing>}"
        manifest_unexpected=$((manifest_unexpected + 1))
        continue
        ;;
    esac
    if [ ! -f "$POC_ROOT/$path" ]; then
      bad "AC 4: frozen experiment data missing: $path"
      manifest_mismatch=$((manifest_mismatch + 1))
      continue
    fi
    actual=$(sha256sum -- "$POC_ROOT/$path" | cut -d' ' -f1)
    if [ "$actual" != "$expected" ]; then
      bad "AC 4: frozen experiment data changed: $path (expected=$expected actual=$actual)"
      manifest_mismatch=$((manifest_mismatch + 1))
    fi
  done < "$FROZEN_DATA_MANIFEST"

  if [ "$manifest_entries" -ne 27 ]; then
    bad "AC 4: frozen-data manifest has $manifest_entries entries, expected 27"
  fi
  manifest_paths=$(grep -v '^#' "$FROZEN_DATA_MANIFEST" | awk '{ print $2 }' | LC_ALL=C sort)
  present_paths=$(cd "$POC_ROOT" &&
    find experiments/pilot experiments/run1 experiments/run2 experiments/run3 -type f |
    LC_ALL=C sort)
  if [ "$manifest_paths" != "$present_paths" ]; then
    added=$(LC_ALL=C comm -13 <(printf '%s\n' "$manifest_paths") <(printf '%s\n' "$present_paths") || true)
    gone=$(LC_ALL=C comm -23 <(printf '%s\n' "$manifest_paths") <(printf '%s\n' "$present_paths") || true)
    [ -z "$added" ] || bad "AC 4: files added under the frozen experiment-data dirs: $(printf '%s' "$added" | tr '\n' ' ')"
    [ -z "$gone" ] || bad "AC 4: frozen experiment-data files missing from the working tree: $(printf '%s' "$gone" | tr '\n' ' ')"
  fi
  if [ "$manifest_entries" -eq 27 ] && [ "$manifest_unexpected" -eq 0 ] &&
     [ "$manifest_mismatch" -eq 0 ] && [ "$manifest_paths" = "$present_paths" ]; then
    ok "AC 4: 27 frozen untracked experiment-data files match checks/002-01-frozen-data.sha256"
  fi
fi

# ---------------------------------------------------------------------------
# Locate the 002-14 subsection (a "### " heading naming 002-14 inside (e))
# ---------------------------------------------------------------------------

s_no=$(awk -v s="$e_no" -v t="$e_end" '
  NR > s && NR < t && /^###[[:space:]]/ && /002-14/ { print NR; exit }' "$REPORT")
if [ -z "$s_no" ]; then
  bad "section (e) has no '### ... 002-14' subsection: the 10 and 200 Hz sync-rate baseline and the one-table (e) 3 rows are absent"
  printf '\n%d passed, %d failed; the 002-14 content does not exist yet.\n' "$pass" "$fail"
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
ok "002-14 subsection found at line $s_no, body lines $SUB_FIRST-$SUB_LAST"

# The 002-12 subsection (the source of the 50 and 1000 Hz corrected means).
s12_no=$(awk -v s="$e_no" -v t="$e_end" '
  NR > s && NR < t && /^###[[:space:]]/ && /002-12/ { print NR; exit }' "$REPORT")
S12_FIRST=0
S12_LAST=-1
if [ -n "$s12_no" ]; then
  s12_end=$(awk -v s="$s12_no" -v t="$e_end" '
    NR <= s { next }
    NR >= t { print NR; exit }
    /^[[:space:]]*```/ { f = !f; next }
    !f && /^#+[[:space:]]/ { print NR; exit }
  ' "$REPORT")
  [ -n "$s12_end" ] || s12_end=$e_end
  S12_FIRST=$((s12_no + 1))
  S12_LAST=$((s12_end - 1))
fi

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
  ok "DoD: every figure-bearing paragraph of the 002-14 subsection carries an evidence tag"
else
  bad "DoD: figure-bearing paragraph(s) without an evidence tag (report lines:$untagged)"
fi

if printf '%s\n' "$SUBTXT" | grep -qF '[measured]'; then
  ok "DoD: the 002-14 subsection carries [measured] evidence"
else
  bad "DoD: the 002-14 subsection carries no [measured] evidence"
fi

# ---------------------------------------------------------------------------
# AC 1 — every metrics file used for 10 and 200 Hz is single-writer: 100 % of
# data lines parse (parseable = lines - 1) and 0 unix_ms regressions; the
# files are named under /tmp/opencode/ and their count is non-zero
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
      if ($1 !~ /^\/tmp\/opencode\// || $1 ~ /hadropoc|poc-scratch/) bp++
      if ($1 ~ /sender/) s++
      if ($1 ~ /synth1/) s1++
      if ($1 ~ /synth2/) s2++
      if ($1 ~ /mixer/) mx++
    }
    END { print d + 0, c + 0, s + 0, s1 + 0, s2 + 0, mx + 0, bp + 0 }')
  set -- $stats
  distinct=$1; clean=$2; c_send=$3; c_s1=$4; c_s2=$5; c_mx=$6; badpath=$7
  if [ "$distinct" -ge 24 ] && [ "$clean" -eq "$distinct" ] && [ "$badpath" -eq 0 ] \
     && [ "$c_send" -ge 6 ] && [ "$c_s1" -ge 6 ] && [ "$c_s2" -ge 6 ] && [ "$c_mx" -ge 6 ]; then
    ok "AC 1: $distinct metrics files of the 10 and 200 Hz runs quoted from the 002-09 checker, all under /tmp/opencode/, all single-writer yes, every data line parses (parseable = lines - 1), 0 unix_ms violations (sender $c_send, synth1 $c_s1, synth2 $c_s2, mixer $c_mx)"
  else
    bad "AC 1: the quoted checker lines must cover the 24 files of the six 10/200 Hz runs (distinct $distinct, clean $clean, outside /tmp/opencode/ or in a POC tree $badpath, per module sender $c_send synth1 $c_s1 synth2 $c_s2 mixer $c_mx; need >= 24 distinct, all clean and under /tmp/opencode/, >= 6 per module)"
  fi
  if printf '%s\n' "$SUBTXT" | grep -Eq ': single-writer no '; then
    bad "AC 1: a quoted metrics file reads 'single-writer no'"
  fi
else
  if para_all 'by hand|hand rule|hand-check|hand check' 'single-writer' 'unix_ms' \
       'pars' '100 ?%' '/tmp/opencode/' \
       '\[(code|measured|kernel-doc|memory|decision)\]' \
     && printf '%s\n' "$SUBTXT" | grep -Eq '[1-9][0-9]* (metrics )?(files|csv)'; then
    ok "AC 1: single-writer stated by the hand rule for the 10 and 200 Hz files (parse 100 %, 0 unix_ms regressions), tagged, with a non-zero file count (lines $PARA_HIT)"
  else
    bad "AC 1: no single-writer statement for the 10 and 200 Hz runs: quote the 002-09 checker lines (<path>: single-writer yes (lines N, parseable P, unix_ms violations V)) for all 24 files, or give the hand rule with a non-zero file count under /tmp/opencode/"
  fi
fi

# ---------------------------------------------------------------------------
# AC 2 and AC 3 — the one rate table (rows 10, 50, 200, 1000 Hz), the header
# labels, the range, the percentage, the marks and the 50/1000 Hz means
# (python3: arithmetic on the table cells, 002-12 means taken from its text)
# ---------------------------------------------------------------------------

TABLE_OUT=$(python3 - "$REPORT" "$SUB_FIRST" "$SUB_LAST" "$S12_FIRST" "$S12_LAST" <<'PY' 2>&1
import re
import sys

rep = sys.argv[1]
a, b, a12, b12 = (int(x) for x in sys.argv[2:6])
with open(rep, encoding="utf-8") as fh:
    L = fh.read().split("\n")

RATES = (10, 50, 200, 1000)
TOL_PCT = 0.6
TOL_MEAN_12 = 0.05
RATE_RE = re.compile(r"(?<![\d.,])(10|50|200|1000)(?![\d.,])")


def region(lo, hi):
    out, fence = [], False
    for n in range(lo, hi + 1):
        t = L[n - 1]
        if re.match(r"^\s*```", t):
            fence = not fence
            continue
        if not fence:
            out.append((n, t))
    return out


def cells(t):
    s = t.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip() for c in s.split("|")]


def is_sep(cs):
    return all(re.fullmatch(r":?-+:?", c) for c in cs if c != "")


def nums(s):
    return [float(x.replace(",", ".")) for x in re.findall(r"\d+(?:[.,]\d+)?", s)]


def rate_of(cell):
    m = RATE_RE.search(cell)
    return int(m.group(1)) if m else None


def cell(cs, i):
    return cs[i] if (i is not None and i < len(cs)) else ""


def stated_12(rate):
    rx = re.compile(r"(?<![\d.,])%d(?![\d.,])" % rate)
    for n, t in region(a12, b12):
        low = t.lower()
        if "mean" in low and "corrected" in low and "naive" not in low and rx.search(t):
            m = re.search(r"mean[^0-9]*(\d+(?:\.\d+)?)", low)
            if m:
                return float(m.group(1)), n
    return None, None


# Markdown tables of the subsection (fenced blocks excluded).
blocks, cur = [], []
for n, t in region(a, b):
    if t.strip().startswith("|"):
        cur.append((n, t))
    elif cur:
        blocks.append(cur)
        cur = []
if cur:
    blocks.append(cur)

parsed = []
for blk in blocks:
    rows = [(n, t, cells(t)) for n, t in blk]
    rows = [r for r in rows if not is_sep(r[2])]
    if len(rows) < 2:
        continue
    hdr, data = rows[0], rows[1:]
    by_rate = {}
    for n, t, cs in data:
        first = next((c for c in cs if c), "")
        r = rate_of(first)
        if r is not None:
            by_rate.setdefault(r, []).append((n, t, cs))
    parsed.append((hdr, by_rate))

chosen = None
for hdr, by_rate in parsed:
    if all(r in by_rate for r in RATES):
        chosen = (hdr, by_rate)
        break

if chosen is None:
    found = sorted({r for _, br in parsed for r in br})
    print("FAIL: AC 3: no single table carries a row for each of 10, 50, 200 and 1000 Hz (rates found in the subsection tables: %s)"
          % (found or "none"))
    sys.exit(0)

hdr_n, hdr_by = chosen
hdr_t = hdr_n[1]
lh = [c.lower() for c in hdr_n[2]]


def find(pred):
    for i, c in enumerate(lh):
        if pred(c):
            return i
    return None


mean_i = find(lambda c: "mean" in c)
range_i = find(lambda c: "range" in c)
mark_i = find(lambda c: "within" in c)
pct_i = find(lambda c: ("%" in c or "percent" in c)
             and not any(w in c for w in ("within", "mean", "range")))
missing = [name for name, i in (("mean", mean_i), ("range", range_i),
                                ("% of configured", pct_i), ("within 2 %", mark_i))
           if i is None]
if missing:
    print("FAIL: AC 2: the rate table header lacks column(s) %s (header: %s)"
          % (", ".join(missing), hdr_t.strip()))
    sys.exit(0)

print("PASS: AC 3: one table carries the 10, 50, 200 and 1000 Hz rows (lines %d-%d)"
      % (hdr_n[0], max(r[0] for rs in hdr_by.values() for r in rs)))

if "corrected" in lh[mean_i]:
    print("PASS: AC 2: the rate table's mean column is labelled corrected (duration-corrected figures)")
else:
    print("FAIL: AC 2: the rate table's mean column is not labelled corrected (header: %s)" % hdr_t.strip())

for R in RATES:
    for n, t, cs in hdr_by[R]:
        errs = []
        if "[measured]" not in t:
            errs.append("DoD: the %d Hz row (line %d) lacks [measured]" % (R, n))
        mn = nums(cell(cs, mean_i))
        if not mn:
            print("FAIL: AC 2: the %d Hz row (line %d) has no mean value" % (R, n))
            continue
        mean = mn[0]
        rg = nums(cell(cs, range_i))
        if len(rg) < 2:
            errs.append("AC 2: the %d Hz row (line %d) range does not give two numbers (min and max)" % (R, n))
        else:
            lo, hi = rg[0], rg[1]
            if not (lo <= hi and lo - 1e-9 <= mean <= hi + 1e-9):
                errs.append("AC 2: the %d Hz row (line %d) mean %.2f is outside its stated range %.2f-%.2f"
                            % (R, n, mean, lo, hi))
        pc = re.search(r"(\d+(?:\.\d+)?)\s*%", cell(cs, pct_i))
        pct = float(pc.group(1)) if pc else (nums(cell(cs, pct_i)) or [None])[0]
        if pct is None:
            errs.append("AC 2: the %d Hz row (line %d) has no percentage of the configured rate" % (R, n))
        else:
            exp_pct = mean / R * 100
            if abs(pct - exp_pct) > TOL_PCT:
                errs.append("AC 2: the %d Hz row (line %d) states %.1f %%, but mean/%d x 100 = %.2f %%"
                            % (R, n, pct, R, exp_pct))
        dev = abs(mean - R) / R
        within_exp = dev <= 0.02 + 1e-12
        mark = cell(cs, mark_i)
        low = mark.lower()
        says_not = re.search(r"not\s+within", low) is not None
        says_within = re.search(r"within\s*2\s*%", low) is not None
        # AC 3 asks for the mark only at 10, 50 and 200 Hz (C-112 range 10-200):
        # the 1000 Hz row may read "n/a (outside C-112)" instead. A bare
        # "within 2 %" on that row still fails below.
        na_1000 = R == 1000 and re.search(r"n/a|not applicable", low) is not None and not says_within
        mark_ok = na_1000 or (within_exp and says_within and not says_not) or ((not within_exp) and says_not)
        if not mark_ok:
            errs.append("AC 3: the %d Hz row (line %d) is marked '%s', but its corrected mean %.2f is %.2f %% from %d Hz, so it must read '%s'"
                        % (R, n, mark, mean, dev * 100, R,
                           "within 2 %" if within_exp else "not within 2 %"))
        if R in (50, 1000):
            s12, s12n = stated_12(R)
            if s12 is None:
                errs.append("AC 2: no corrected mean for %d Hz found in the 002-12 subsection to bind the %d Hz row to" % (R, R))
            elif abs(mean - s12) > TOL_MEAN_12:
                errs.append("AC 2: the %d Hz row mean %.2f differs from the 002-12 corrected mean %.2f (line %d)"
                            % (R, mean, s12, s12n))
        if errs:
            for e in errs:
                print("FAIL: " + e)
        else:
            print("PASS: AC 2/3: %d Hz row (line %d): mean %.2f corrected, range %.2f-%.2f, %.1f %% of configured, mark '%s' consistent (|mean - %d| / %d = %.2f %%)"
                  % (R, n, mean, rg[0], rg[1], pct, mark.strip(), R, R, dev * 100))
PY
)

while IFS= read -r line; do
  case "$line" in
    PASS:*) pass=$((pass + 1)) ;;
    FAIL:*) fail=$((fail + 1)) ;;
    '') continue ;;
    *) line="FAIL: unexpected output: $line"; fail=$((fail + 1)) ;;
  esac
  printf '%s\n' "$line"
done <<< "$TABLE_OUT"

# ---------------------------------------------------------------------------
# AC 2 — runs, durations, unprivileged, strict, reference rows, metrics column
# ---------------------------------------------------------------------------

if sub_has '(3|three) runs'; then
  ok "AC 2: the 10 and 200 Hz figures are stated as the mean and range of 3 runs"
else
  bad "AC 2: the subsection does not state that the mean and range are over 3 runs ('3 runs')"
fi

if sub_has '(--duration[ =]*(3[0-9]|[4-9][0-9]|[0-9]{3,})([^0-9.]|$))|((^|[^0-9.])(3[0-9]|[4-9][0-9]|[0-9]{3,})[ ]*(s|sec|secs|seconds)([^[:alnum:]]|$))|(at least 30)|((>=)[ ]*30)'; then
  ok "AC 2: each run is stated as >= 30 s"
else
  bad "AC 2: the run duration is not stated as >= 30 s ('30 s', '--duration 30', 'at least 30')"
fi

if sub_has 'unprivileged' && sub_has 'strict'; then
  ok "AC 2: the runs are stated unprivileged and not in strict mode (C-147)"
else
  bad "AC 2: state that the runs were unprivileged and not in strict mode (C-147)"
fi

if sub_has 'reference row|P-[0-9][0-9]'; then
  ok "AC 2: the figures are stated to go only to the POC reference rows (P-nn)"
else
  bad "AC 2: say the figures go only to the POC reference rows (P-nn), not to a C- row (C-147)"
fi

if para_all '(served|asked)' 'module\.rs:[0-9]+' && sub_has 'holder'; then
  ok "AC 2: the metrics column (served or asked) and its module.rs pointer are named, from the clock holder's rows"
else
  bad "AC 2: name the metrics column used (served or asked) with its module.rs:<n> pointer, and say the figures come from the clock holder's peer metrics"
fi

# ---------------------------------------------------------------------------
# AC 4 (subsection wording) and DoD — the frozen tree is stated neither run
# nor written; the run command names the isolated group and --no-tui; no
# command runs from the frozen tree; open items carry owner and falsifier
# ---------------------------------------------------------------------------

if sub_has 'poc-untouched' && sub_has 'fingerprint'; then
  ok "AC 4: the subsection states the frozen-tree fingerprint against poc-untouched"
else
  bad "AC 4: the subsection does not say the frozen-tree fingerprint is poc-untouched and unchanged"
fi

if para_all 'frozen[^.]*(neither|never|not |untouched|unmodified)[^.]*(run|written|modified)'; then
  ok "DoD: the frozen tree is stated neither run nor written (lines $PARA_HIT)"
else
  bad "DoD: state that the frozen tree at $FROZEN_ROOT was neither run nor written"
fi

cmdline=$(printf '%s\n' "$SUBTXT" | grep -- '--sync-hz' \
  | grep -- '--group 239\.72\.68\.44:' | grep -- '--no-tui' || true)
if printf '%s\n' "$cmdline" | grep -Eq -- '--sync-hz 10([^0-9]|$)' \
   && printf '%s\n' "$cmdline" | grep -Eq -- '--sync-hz 200([^0-9]|$)'; then
  ok "DoD: the run commands at 10 and 200 Hz are stated with an isolated --group 239.72.68.44:<port> and --no-tui"
else
  bad "DoD: state the run commands for --sync-hz 10 and --sync-hz 200, each with --group 239.72.68.44:<port> and --no-tui"
fi

if printf '%s\n' "$SUBTXT" | grep -E -- 'hadropoc (sender|loopback|synth[12]?|mixer)' \
     | grep -- '--sync-hz' | grep -qF -- "$FROZEN_ROOT"; then
  bad "DoD: a --sync-hz command of the subsection points at the frozen POC tree ($FROZEN_ROOT)"
else
  ok "DoD: no --sync-hz command points at the frozen POC tree"
fi

open_bad=""
while read -r s e; do
  [ -n "$s" ] || continue
  p=$(line_range "$REPORT" "$s" "$e")
  if printf '%s\n' "$p" | grep -qi 'open item'; then
    if ! printf '%s\n' "$p" | grep -qi 'owner' || ! printf '%s\n' "$p" | grep -qi 'falsifier'; then
      open_bad="$open_bad $s-$e"
    fi
  fi
done <<EOF
$PARAS
EOF
if [ -z "$open_bad" ]; then
  ok "DoD: any new open item in the 002-14 subsection names an owner and a falsifier"
else
  bad "DoD: open item text without owner and falsifier (report lines:$open_bad)"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
