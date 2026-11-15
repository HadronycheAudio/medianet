#!/usr/bin/env bash
# checks/002-11.sh — checks for ticket 002-11
# "Servo phase step measurement".
#
# The deliverable is the "### ... 002-11" subsection of section (e) Run
# confirmations of doc/design/reports/archaeology/REPORT-0b.md (deliverable
# (e) 2 of epic 002). It measures the phase step theta0_after - theta0_before
# published by each locked Servo::update of the POC (clock.rs, the 200 ms
# window), from the scratch copy of the POC, by a loopback run of >= 60 s
# (unprivileged), and states a confirmed or refuted result with a clock.rs
# pointer. The section (e) verdict line itself stays "pending" until 002-13
# closes the epic; the 002-11 subsection adds no verdict line (002-10 precedent:
# its result is a bold statement "**Starvation: refuted** ..." with a file:line
# pointer, not a "verdict:" line).
#
# This script reads the report and the committed companion files. It never
# writes anywhere, never runs the POC and never touches the frozen POC or the
# scratch copy. Running the measurements is the investigator's work.
#
# The conventions below make the prose machine-checkable without fixing its
# wording. The subsection runs from its "### " heading (naming 002-11) to the
# next heading of any level, outside code fences. A "block" is a run of prose
# lines, of table rows (lines starting with "|") or of fenced lines, split by
# blank lines, headings and kind changes.
#
# * (AC 1, count and duration.) The stated count of locked updates is the first
#   number directly bound to "locked update(s)" or "locked step(s)", in one of
#   the forms "262 locked updates", "locked updates: 262" or "262 updates ...
#   locked". It must be >= 200 and its block must carry [measured]. Thousands
#   may be grouped by a space or a comma ("1 040" or "1,040"). The run is
#   stated as >= 60 s ("60 s", "--duration 60", "at least 60", ">= 60 s"),
#   "unprivileged" and "loopback" appear in the subsection.
# * (AC 2, statistics.) One block (prose or table) carries the words delta or
#   step (or the glyph Δ) and median, p99, max and min, and gives for |Δ| in ns:
#   count, min, median, p99 and max. Prose form: "count 262, min 0 ns, median
#   41 ns, p99 118 ns, max 240 ns" (a label, optional "(...)", optional
#   "|...|" or "abs/delta/step" word, then the number and its unit ns, µs, us
#   or ms). Table form: a header row with the labels (a unit "(ns)" in the
#   header or in the cell) and a data row. Checks: count = the stated count;
#   min <= median <= p99 <= max; the block carries [measured]. The median update
#   interval is a positive number in ms in a block carrying "interval" and
#   "median" and [measured]. The value is measured, not asserted: no value is
#   expected.
# * (AC 2, histogram.) At least one fenced or table block with the word bin or
#   histogram: a table sums its count column (the header cell "count", else the
#   last column); a fence sums the trailing integer of each line. Every such
#   block has >= 3 bins and its counts sum to the stated count.
# * (AC 3, result and mechanism.) One prose block carries "phase step per
#   update" followed (within 30 characters, no period or semicolon) by
#   "confirmed" or "refuted", and [measured]. Its clock.rs pointers
#   (clock.rs:<n> or clock.rs:<n>-<m>) are verified against
#   `git -C <POC> show poc-untouched:src/clock.rs`: at least one pointer must
#   lie inside the body of `fn update` and cite a line that contains theta0 or
#   the update arithmetic (alpha, pred or err). If the POC repository or the
#   poc-untouched tag is unavailable, that verification is SKIPPED loudly.
#   The subsection says "poc-untouched" for the numbering.
# * (AC 4, frozen tree.) A paragraph states that the frozen-tree fingerprint
#   (002-01) is unchanged, and carries a tag. The subsection names 002-01, and
#   the mechanical guard checks/002-01.sh is run and must exit 0.
# * (AC 5, lock condition.) A paragraph names the lock condition used to
#   discard pre-lock updates ("lock", "P-04", "8 windows", "20 µs" and
#   "path delay"), states that it matches or does not match P-04 (match,
#   agree, differ, same as, identical), and carries a tag.
# * (Method and source.) A paragraph names the scratch copy (scratch or
#   poc-scratch) and loopback, carries a tag. The subsection says "poc-untouched"
#   with identical, equivalent or fingerprint. A paragraph says the frozen tree
#   was neither run nor written.
# * (Logging.) Logging is mandatory for this ticket. A paragraph places the
#   logging in the scratch copy and not the frozen tree. A companion diff
#   sits beside the report (*002-11*.diff/.patch, or a diff naming 002-11),
#   non-empty, tracked, unified, adding at least one logging line that adds the
#   published theta0, touching only scratch paths (no target/ or experiments/
#   paths, no frozen-tree path). The subsection says the diff is logging-only.
# * (Evidence log.) Every per-update data file the subsection names
#   (002-11*.log/.csv/.tsv/.txt/.json) must exist beside the report, be
#   tracked and be non-empty. No such name is required.
# * (Section (e) verdict, as checks/002-10.sh and 002-12.sh.) Section (e) keeps
#   EXACTLY ONE "verdict:" line in one of the allowed forms: the placeholder
#   "pending" (the state until 002-13), or the final form with "mechanism:" or
#   "explanation:". The 002-11 subsection adds no second verdict line.
# * (Open items.) If the subsection states an open item, open question or
#   unresolved point, it names an owner and a falsifier.
# * (Evidence tags.) Every paragraph of the subsection that carries a figure
#   (a digit) carries an evidence tag on itself or on the line right after it.
#   The subsection carries [measured].
#
# Exit status: 0 when every check passes, 1 otherwise. SKIP lines are reported
# but do not fail the check.

set -u

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REPORT_REL="doc/design/reports/archaeology/REPORT-0b.md"
ARCH_REL="doc/design/reports/archaeology"
REPORT="$REPO_ROOT/$REPORT_REL"
ARCH="$REPO_ROOT/$ARCH_REL"
FROZEN_ROOT="/home/dorian/workspace/hadropoc"

pass=0
fail=0
skip=0

ok()   { printf 'PASS: %s\n' "$*"; pass=$((pass + 1)); }
bad()  { printf 'FAIL: %s\n' "$*"; fail=$((fail + 1)); }
skipped() { printf 'SKIP: %s\n' "$*"; skip=$((skip + 1)); }

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
  printf '\n1 check(s) failed; the document for 002-11 does not exist yet.\n'
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
# Section (e) verdict line: exactly one, pending form (as checks/002-10.sh and
# checks/002-12.sh). Checked over the whole section, so 002-11 adds none.
# ---------------------------------------------------------------------------

verdict_count=$(printf '%s\n' "$SECTION" \
  | grep -Ec '^[[:space:]]*(\*\*)?verdict:' || true)
if [ "$verdict_count" -eq 0 ]; then
  bad "section (e) lost its 'verdict:' line"
elif [ "$verdict_count" -gt 1 ]; then
  bad "section (e) has $verdict_count 'verdict:' lines; exactly one is required (002-11 adds none; 002-13 writes the final verdict)"
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
# Locate the 002-11 subsection
# ---------------------------------------------------------------------------

s_no=$(awk -v s="$e_no" -v t="$e_end" '
  NR > s && NR < t && /^###[[:space:]]/ && /002-11/ { print NR; exit }' "$REPORT")
if [ -z "$s_no" ]; then
  bad "section (e) has no '### ... 002-11' subsection: the servo phase-step measurement is absent"
  printf '\n%d passed, %d failed; the 002-11 content does not exist yet.\n' "$pass" "$fail"
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
ok "002-11 subsection found at line $s_no, body lines $SUB_FIRST-$SUB_LAST"

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
  ok "DoD: every figure-bearing paragraph of the 002-11 subsection carries an evidence tag"
else
  bad "DoD: figure-bearing paragraph(s) without an evidence tag (report lines:$untagged)"
fi

if has_measured "$SUBTXT"; then
  ok "DoD: the 002-11 subsection carries [measured] evidence"
else
  bad "DoD: the 002-11 subsection carries no [measured] evidence"
fi

# ---------------------------------------------------------------------------
# AC 1 (count), AC 2 (statistics, interval, histogram), AC 3 (result and the
# clock.rs pointer, verified against poc-untouched). Parsed in python3 because
# the statistics live in prose or in tables. The python prints one line per
# result: OK, BAD or SKIP, then a tab and the message.
# ---------------------------------------------------------------------------

read -r -d '' PYCODE <<'PY' || true
import re, subprocess, sys

path, first, last, poc = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
lines = open(path, encoding='utf-8').read().split('\n')


def emit(kind, msg):
    print(kind + '\t' + msg)


NUM_C = r'(?:\d{1,3}(?: \d{3})+|\d+)'
NUM_V = r'(?:\d+(?:[,_]\d{3})*(?:\.\d+)?)'
UNIT = r'(ns|µs|μs|us|ms)'
UNITS = {'ns': 1.0, 'µs': 1e3, 'μs': 1e3, 'us': 1e3, 'ms': 1e6}


def to_int(s):
    return int(re.sub(r'\D', '', s))


def to_float(s):
    return float(re.sub(r'[,_]', '', s))


kinds = []
infence = False
for n in range(first, last + 1):
    s = lines[n - 1]
    if re.match(r'^\s*```', s):
        infence = not infence
        kinds.append((n, s, 'fence'))
    elif infence:
        kinds.append((n, s, 'fence'))
    elif s.strip() == '' or re.match(r'^#+\s', s):
        kinds.append((n, s, None))
    elif s.lstrip().startswith('|'):
        kinds.append((n, s, 'table'))
    else:
        kinds.append((n, s, 'para'))

blocks = []
cur = None
for n, s, k in kinds:
    if k is None:
        cur = None
        continue
    if cur is None or cur['kind'] != k:
        cur = {'kind': k, 'start': n, 'end': n, 'lines': []}
        blocks.append(cur)
    cur['end'] = n
    cur['lines'].append(s)
for bl in blocks:
    bl['text'] = '\n'.join(bl['lines'])
    bl['flat'] = ' '.join(x.strip() for x in bl['lines'])
    bl['low'] = bl['text'].lower()


def tagged(bl):
    if '[measured]' in bl['text']:
        return True
    return bl['end'] < len(lines) and '[measured]' in lines[bl['end']]


def table_rows(bl):
    rows = []
    for s in bl['lines']:
        st = s.strip()
        if not st.startswith('|'):
            continue
        cells = [c.strip() for c in st.strip('|').split('|')]
        if all(re.fullmatch(r':?-{2,}:?', c) for c in cells):
            continue
        rows.append(cells)
    return rows


def cell_num(cell, default_unit):
    m = re.fullmatch(r'(' + NUM_V + r')\s*' + UNIT + r'?', cell.strip())
    if not m:
        return None
    unit = m.group(2) or default_unit
    if unit is None:
        return None
    return to_float(m.group(1)) * UNITS[unit.lower()]


def stats_prose(flat):
    res = {}
    for lab in ['min', 'median', 'p99', 'max']:
        m = re.search(r'\b' + lab + r'\b\s*(?:\([^)]{0,30}\)\s*)?(?:\|[^|]{0,30}\|\s*)?'
                      r'(?:(?:abs(?:olute)?|delta|step)\s*)?[:=]?\s*(?:of\s+|is\s+|was\s+)?'
                      r'(' + NUM_V + r')\s*' + UNIT + r'(?![A-Za-zµ])', flat, re.I)
        if m:
            res[lab] = to_float(m.group(1)) * UNITS[m.group(2).lower()]
    m = re.search(r'\bcount\b\s*(?:\([^)]{0,30}\)\s*)?(?:of\s+)?(?:\|[^|]{0,30}\|\s*)?'
                  r'[:=]?\s*(?:of\s+|is\s+|=\s*)?(' + NUM_C + r')(?![\d.])'
                  r'(?!\s*(?:ns|µs|μs|us|ms)\b)', flat, re.I)
    if m:
        res['count'] = float(to_int(m.group(1)))
    return res


def stats_table(bl):
    rows = table_rows(bl)
    if len(rows) < 2:
        return {}
    hdr = rows[0]
    cols = {}
    for lab in ['count', 'min', 'median', 'p99', 'max']:
        for i, h in enumerate(hdr):
            if re.match(lab + r'\b', h.lower()):
                cols[lab] = i
                break
    if len(cols) < 5:
        return {}
    hunit = {}
    for lab, i in cols.items():
        m = re.search(r'\((ns|µs|μs|us|ms)\)', hdr[i], re.I)
        hunit[lab] = m.group(1) if m else None
    for row in rows[1:]:
        res = {}
        good = True
        for lab, i in cols.items():
            if i >= len(row):
                good = False
                break
            if lab == 'count':
                if not re.fullmatch(NUM_C, row[i].strip()):
                    good = False
                    break
                res[lab] = float(to_int(row[i]))
            else:
                v = cell_num(row[i], hunit[lab])
                if v is None:
                    good = False
                    break
                res[lab] = v
        if good:
            return res
    return {}


def stats_of(bl):
    if bl['kind'] == 'table':
        return stats_table(bl)
    return stats_prose(bl['flat'])


def interval_ms(bl):
    if bl['kind'] == 'table':
        rows = table_rows(bl)
        if len(rows) < 2:
            return None
        hdr = rows[0]
        idx = [i for i, h in enumerate(hdr) if 'interval' in h.lower()]
        if not idx:
            return None
        i = idx[0]
        hu = re.search(r'\((ns|µs|μs|us|ms)\)', hdr[i], re.I)
        for row in rows[1:]:
            if i < len(row):
                v = cell_num(row[i], hu.group(1) if hu else None)
                if v is not None:
                    return v / 1e6
        return None
    m = re.search(r'interval[^0-9\n]{0,40}?(' + NUM_V + r')\s*' + UNIT + r'(?![A-Za-zµ])',
                  bl['flat'], re.I)
    if m:
        return to_float(m.group(1)) * UNITS[m.group(2).lower()] / 1e6
    return None


def hist_of(bl):
    if bl['kind'] == 'table':
        rows = table_rows(bl)
        if len(rows) < 2:
            return None
        hdr = [h.lower() for h in rows[0]]
        ci = next((i for i, h in enumerate(hdr) if 'count' in h), len(hdr) - 1)
        vals = []
        for row in rows[1:]:
            if ci < len(row) and re.fullmatch(NUM_C, row[ci].strip()):
                vals.append(to_int(row[ci]))
        return (len(vals), sum(vals))
    vals = []
    for s in bl['lines']:
        st = s.strip()
        if st.startswith('```'):
            continue
        m = re.search(r'(\d+)\s*\|?\s*$', st)
        if m:
            vals.append(int(m.group(1)))
    return (len(vals), sum(vals))


# AC 1 — the stated count of locked updates: the first match in document order.
CNT_PATS = [
    r'(' + NUM_C + r')\s+locked\s+(?:updates?|steps?)\b',
    r'locked\s+(?:updates?|steps?)\s*(?:\([^)]{0,30}\))?\s*[:=]\s*(' + NUM_C + r')(?![\d.])',
    r'(' + NUM_C + r')\s+(?:updates?|steps?)\b[^.;\n]{0,40}?\blocked\b',
]
cands = []
for bl in blocks:
    if bl['kind'] == 'fence':
        continue
    for pat in CNT_PATS:
        for m in re.finditer(pat, bl['flat'], re.I):
            cands.append((bl['start'], m.start(), to_int(m.group(1)), bl))
cands.sort(key=lambda c: (c[0], c[1]))

count = None
if not cands:
    emit('BAD', 'AC 1: no stated count of locked updates (e.g. "262 locked updates ... [measured]")')
else:
    _, _, count, cbl = cands[0]
    if count < 200:
        emit('BAD', 'AC 1: the stated count of locked updates is %d; at least 200 are required' % count)
    elif not tagged(cbl):
        emit('BAD', 'AC 1: the count of %d locked updates (lines %d-%d) carries no [measured] tag'
             % (count, cbl['start'], cbl['end']))
    else:
        emit('OK', 'AC 1: %d locked updates are stated as measured (lines %d-%d, [measured])'
             % (count, cbl['start'], cbl['end']))

# AC 2 — the |Δ| statistics: count, min, median, p99, max in ns.
stats_bl = None
stats = {}
for bl in blocks:
    if bl['kind'] == 'fence':
        continue
    if not re.search(r'delta|step|Δ', bl['text'], re.I):
        continue
    if not all(re.search(r'\b' + w + r'\b', bl['text'], re.I) for w in ['median', 'p99', 'max', 'min']):
        continue
    st = stats_of(bl)
    if all(k in st for k in ['count', 'min', 'median', 'p99', 'max']):
        stats_bl, stats = bl, st
        break

if stats_bl is None:
    emit('BAD', 'AC 2: no block gives count, min, median, p99 and max of |Δ| with units (ns/µs/ms)')
else:
    where = 'lines %d-%d' % (stats_bl['start'], stats_bl['end'])
    if count is not None and int(stats['count']) != count:
        emit('BAD', 'AC 2: the |Δ| count (%d, %s) differs from the stated count of locked updates (%d)'
             % (int(stats['count']), where, count))
    else:
        emit('OK', 'AC 2: the |Δ| count (%d, %s) equals the stated count of locked updates'
             % (int(stats['count']), where))
    order = [stats['min'], stats['median'], stats['p99'], stats['max']]
    if all(order[i] <= order[i + 1] for i in range(3)):
        emit('OK', 'AC 2: min <= median <= p99 <= max holds (%s, in ns: min %g, median %g, p99 %g, max %g)'
             % (where, order[0], order[1], order[2], order[3]))
    else:
        emit('BAD', 'AC 2: the |Δ| order is violated (%s, in ns: min %g, median %g, p99 %g, max %g); need min <= median <= p99 <= max'
             % (where, order[0], order[1], order[2], order[3]))
    if tagged(stats_bl):
        emit('OK', 'AC 2: the |Δ| statistics are tagged [measured] (%s)' % where)
    else:
        emit('BAD', 'AC 2: the |Δ| statistics (%s) carry no [measured] tag' % where)

interval = None
int_bl = None
for bl in blocks:
    if bl['kind'] == 'fence' or 'interval' not in bl['low'] or 'median' not in bl['low']:
        continue
    v = interval_ms(bl)
    if v is not None:
        interval, int_bl = v, bl
        break
if interval is None or interval <= 0:
    emit('BAD', 'AC 2: no positive median update interval in ms (a block with "median" and "interval" and a value in ms)')
elif not tagged(int_bl):
    emit('BAD', 'AC 2: the median update interval (lines %d-%d) carries no [measured] tag'
         % (int_bl['start'], int_bl['end']))
else:
    emit('OK', 'AC 2: median update interval %g ms, positive and [measured] (lines %d-%d)'
         % (interval, int_bl['start'], int_bl['end']))

# AC 2 — the histogram: every bin block sums to the stated count.
hist_blocks = []
for bl in blocks:
    if bl['kind'] == 'para':
        continue
    if re.search(r'\bbin|histogram', bl['low']):
        h = hist_of(bl)
        if h is not None:
            hist_blocks.append((bl, h))
if not hist_blocks:
    emit('BAD', 'AC 2: no histogram (a table or fenced block with bins and counts)')
else:
    good = True
    for bl, (nb, total) in hist_blocks:
        if nb < 3:
            good = False
            emit('BAD', 'AC 2: the histogram (lines %d-%d) has %d bins; at least 3 are required'
                 % (bl['start'], bl['end'], nb))
        elif count is not None and total != count:
            good = False
            emit('BAD', 'AC 2: the histogram counts (lines %d-%d) sum to %d, not the stated count %d'
                 % (bl['start'], bl['end'], total, count))
    if good:
        bl, (nb, total) = hist_blocks[0]
        emit('OK', 'AC 2: histogram of %d bins (lines %d-%d) whose counts sum to the stated count %s'
             % (nb, bl['start'], bl['end'], count))

# AC 3 — the result statement and its mechanism pointer.
RES = re.compile(r'phase[- ]step per update[^.;]{0,30}?\b(confirmed|refuted)\b', re.I)
res_blocks = [bl for bl in blocks if bl['kind'] == 'para' and RES.search(bl['flat'])]
if not res_blocks:
    emit('BAD', 'AC 3: no result statement "phase step per update: confirmed" or "refuted" (prose, [measured], with a clock.rs pointer)')
else:
    rbl = res_blocks[0]
    word = RES.search(rbl['flat']).group(1).lower()
    ptrs = re.findall(r'clock\.rs:(\d+)(?:[-–](\d+))?', rbl['flat'])
    if not ptrs:
        emit('BAD', 'AC 3: the result statement (lines %d-%d) carries no clock.rs:<line> pointer'
             % (rbl['start'], rbl['end']))
    elif not tagged(rbl):
        emit('BAD', 'AC 3: the result statement (lines %d-%d) carries no [measured] tag'
             % (rbl['start'], rbl['end']))
    else:
        src = None
        why = ''
        try:
            r = subprocess.run(['git', '-C', poc, 'show', 'poc-untouched:src/clock.rs'],
                               capture_output=True, text=True)
            if r.returncode == 0:
                src = r.stdout.split('\n')
            else:
                why = r.stderr.strip() or 'git show failed'
        except Exception as ex:
            why = str(ex)
        if src is None:
            emit('SKIP', 'AC 3: clock.rs pointer NOT verified: POC repository or tag poc-untouched unavailable at %s (%s)'
                 % (poc, why))
        else:
            upd = next((i for i, l in enumerate(src) if 'fn update(&mut self' in l), None)
            if upd is None:
                emit('BAD', 'AC 3: fn update not found in poc-untouched:src/clock.rs')
            else:
                end = next((i for i in range(upd + 1, len(src)) if src[i].rstrip() == '    }'), len(src) - 1)
                lo, hi = upd + 1, end + 1
                tok = re.compile(r'theta0|alpha|\bpred\b|\berr\b')
                mech = []
                bad_range = []
                for a, b in ptrs:
                    s_ = int(a)
                    e_ = int(b) if b else s_
                    if s_ < 1 or e_ < s_ or e_ > len(src):
                        bad_range.append('%d-%d' % (s_, e_))
                        continue
                    seg = src[s_ - 1:e_]
                    if lo <= s_ and e_ <= hi and any(tok.search(x) for x in seg):
                        mech.append('%d-%d' % (s_, e_))
                if bad_range:
                    emit('BAD', 'AC 3: clock.rs pointer(s) outside the file in poc-untouched: %s' % ', '.join(bad_range))
                elif not mech:
                    emit('BAD', 'AC 3: no clock.rs pointer of the result statement lies inside Servo::update (poc-untouched lines %d-%d) on a line with theta0 or the update arithmetic (cited: %s)'
                         % (lo, hi, ', '.join('%s-%s' % (a, b or a) for a, b in ptrs)))
                else:
                    emit('OK', 'AC 3: the result "phase step per update: %s" names the mechanism at clock.rs:%s (poc-untouched, inside Servo::update, lines %d-%d, theta0 or update arithmetic) [measured]'
                         % (word, mech[0], lo, hi))
PY

RESULTS=$(python3 -c "$PYCODE" "$REPORT" "$SUB_FIRST" "$SUB_LAST" "$FROZEN_ROOT" 2>&1 \
  || printf 'BAD\tAC 1-3: the analysis failed to run\n')
while IFS=$'\t' read -r kind msg; do
  case "$kind" in
    OK)   ok "$msg" ;;
    BAD)  bad "$msg" ;;
    SKIP) skipped "$msg" ;;
  esac
done <<< "$RESULTS"

# ---------------------------------------------------------------------------
# AC 1 — duration >= 60 s, unprivileged, a loopback run
# ---------------------------------------------------------------------------

if sub_has '(--duration[ =]*(6[0-9]|[7-9][0-9]|[0-9]{3,})([^0-9.]|$))|((^|[^0-9.])(6[0-9]|[7-9][0-9]|[0-9]{3,})[ ]*(s|sec|secs|seconds)([^[:alnum:]]|$))|(at least 60)|((>=|≥)[ ]*60)'; then
  ok "AC 1: the run duration is stated as >= 60 s"
else
  bad "AC 1: the run duration is not stated as >= 60 s ('60 s', '--duration 60', 'at least 60')"
fi

if sub_has 'unprivileged' && sub_has 'loopback'; then
  ok "AC 1: the run is stated as a loopback run, unprivileged"
else
  bad "AC 1: state that the run is 'hadropoc loopback' and unprivileged"
fi

# ---------------------------------------------------------------------------
# AC 4 — the frozen-tree fingerprint is unchanged; the mechanical guard is
# checks/002-01.sh, which must exit 0
# ---------------------------------------------------------------------------

if para_all 'frozen' 'fingerprint' \
     'unchanged|not (been )?modified|untouched|still (identical|matches)|remains (identical|unchanged)|matches (the )?(recorded|002-01)' \
     '\[(code|measured|kernel-doc|memory|decision)\]'; then
  ok "AC 4: the frozen-tree fingerprint is stated unchanged, with an evidence tag (lines $PARA_HIT)"
else
  bad "AC 4: the subsection does not state, with a tag, that the frozen-tree fingerprint is unchanged"
fi

if sub_has '002-01'; then
  if [ -f "$REPO_ROOT/checks/002-01.sh" ] && bash "$REPO_ROOT/checks/002-01.sh" >/dev/null 2>&1; then
    ok "AC 4: the fingerprint guard checks/002-01.sh exits 0 (deferred to 002-01)"
  else
    bad "AC 4: checks/002-01.sh is missing or does not exit 0; the frozen-tree fingerprint guard fails"
  fi
else
  bad "AC 4: the subsection does not name 002-01 as the fingerprint guard"
fi

# ---------------------------------------------------------------------------
# AC 5 — the lock condition used to discard pre-lock updates, and whether it
# matches P-04
# ---------------------------------------------------------------------------

if para_all 'lock' 'P-04' 'match|agree|differ|same as|identical' \
     '8 windows|eight windows' '20 ?(µs|us)' 'path delay|delay' \
     'discard|pre-lock|before (the )?lock' \
     '\[(code|measured|kernel-doc|memory|decision)\]'; then
  ok "AC 5: the lock condition used to discard pre-lock updates is stated, its match with P-04 is stated, tagged (lines $PARA_HIT)"
else
  bad "AC 5: state the lock condition used to discard pre-lock updates (smoothed error below max(20 µs, path delay / 4) after at least 8 windows) and whether it matches P-04, with a tag"
fi

# ---------------------------------------------------------------------------
# Method and source: the scratch copy, loopback, poc-untouched-equivalent; the
# frozen tree neither run nor written
# ---------------------------------------------------------------------------

if para_all '(scratch|poc-scratch)' 'loopback' '\[(code|measured|kernel-doc|memory|decision)\]'; then
  ok "DoD: the runs are stated as scratch-copy loopback runs, tagged (lines $PARA_HIT)"
else
  bad "DoD: state that the runs are 'hadropoc loopback' runs of the scratch copy (poc-scratch), tagged"
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

# ---------------------------------------------------------------------------
# DoD — diagnostic logging lives in the scratch copy; a logging-only companion
# diff sits beside the report (mandatory for this ticket)
# ---------------------------------------------------------------------------

if para_all 'scratch' 'log' 'frozen' 'not|never|outside|neither|rather than'; then
  ok "DoD: the logging is stated to live in the scratch copy, not the frozen tree (lines $PARA_HIT)"
else
  bad "DoD: no paragraph says the per-update logging lives in the scratch copy and not in the frozen tree"
fi

DIFF_FILE=""
for f in "$ARCH"/*002-11*.diff "$ARCH"/*002-11*.patch; do
  [ -f "$f" ] && { DIFF_FILE="$f"; break; }
done
if [ -z "$DIFF_FILE" ]; then
  for f in "$ARCH"/*.diff "$ARCH"/*.patch; do
    [ -f "$f" ] || continue
    if grep -Eq '002-11' "$f"; then
      DIFF_FILE="$f"
      break
    fi
  done
fi

if [ -z "$DIFF_FILE" ]; then
  bad "DoD: logging is required but no companion scratch diff exists beside the report (expect *002-11*.diff or *.patch)"
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
  elif ! grep -Eq '^\+[^+].*theta0' "$DIFF_FILE"; then
    bad "DoD: $diff_rel does not log the published theta0"
  elif grep -Eq '^(diff --git .*(target/|experiments/)|Only in .*(target|experiments))' "$DIFF_FILE"; then
    bad "DoD: $diff_rel touches target/ or experiments/, not logging only"
  else
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
          *) path_bad=$((path_bad + 1)) ;;
        esac
      done
    done < "$DIFF_FILE"
    if [ "$path_bad" -eq 0 ]; then
      ok "DoD: companion diff present, committed, unified, logging-only (adds the theta0 log line) and scratch-only: $diff_rel"
    else
      bad "DoD: $diff_rel has $path_bad header path(s) outside poc-scratch"
    fi
  fi
fi

if para_all 'logging[- ]only|only (adds? )?logging|no behaviou?r(al)? change|behaviou?r[- ]preserving|changes no behaviou?r'; then
  ok "DoD: the subsection states the diff is logging-only / changes no behaviour"
else
  bad "DoD: the subsection does not state that the scratch diff is logging-only / changes no behaviour"
fi

# ---------------------------------------------------------------------------
# DoD — an evidence log (raw or summarised per-update data) named by the
# subsection must exist beside the report, be tracked and be non-empty
# ---------------------------------------------------------------------------

logs=$(printf '%s\n' "$SUBTXT" | grep -Eo '002-11[A-Za-z0-9._-]*\.(log|csv|tsv|txt|jsonl?)' | sort -u || true)
if [ -z "$logs" ]; then
  ok "DoD: no evidence log is referenced by the subsection (none required)"
else
  log_bad=0
  while read -r name; do
    [ -n "$name" ] || continue
    f="$ARCH/$name"
    if [ ! -s "$f" ]; then
      bad "DoD: evidence log referenced but missing or empty: $ARCH_REL/$name"
      log_bad=1
    elif ! is_tracked "$ARCH_REL/$name"; then
      bad "DoD: evidence log exists but is not committed: $ARCH_REL/$name"
      log_bad=1
    fi
  done <<EOF
$logs
EOF
  [ "$log_bad" -eq 0 ] && ok "DoD: every referenced evidence log exists beside the report, is tracked and non-empty"
fi

# ---------------------------------------------------------------------------
# DoD — open items name an owner and a falsifier (keyword level)
# ---------------------------------------------------------------------------

if sub_has 'open (item|question|sub-question)|unresolved|not established|TODO'; then
  if sub_has 'owner' && sub_has 'falsifier'; then
    ok "DoD: the open item(s) of the subsection name an owner and a falsifier"
  else
    bad "DoD: the subsection states an open item but does not name an owner and a falsifier"
  fi
else
  ok "DoD: no open item is stated by the subsection"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

printf '\n%d passed, %d failed, %d skipped\n' "$pass" "$fail" "$skip"
[ "$fail" -eq 0 ]
