#!/usr/bin/env bash
# checks/002-13.sh — checks for ticket 002-13
# "Close-out: report audit and design updates" (last ticket of epic 002,
# POC archaeology, check 0b).
#
# Deliverable checked: doc/design/reports/archaeology/REPORT-0b.md (close-out
# additions), doc/design/02-constants.md (P rows only), doc/design/
# 06-evidence-appendix.md (five new entries) and the design set (no stale
# statement of the old sync rate). This script reads those files and never
# writes anywhere, never runs the POC and never touches the frozen POC.
#
# Overrides (fixture harness): REPORT_0B, CONSTANTS_MD, EVIDENCE_MD, DESIGN_ROOT
# (the root scanned by the stale-statement and loss-statement guards; default
# doc/design). The same rules apply in every mode; no check reads any git ref.
#
# Conventions the author must follow (wording is free, these forms are
# machine-checked):
#
# * (AC 1, checklist.) Before the first lettered heading "(a)".."(e)", one
#   ticked line per epic criterion: "- [x] 002-AC-n ..." (exactly one per id,
#   n = 1..7, no unticked line for an id). The line names its satisfying
#   section as "(a)".."(e)" or by the heading text, and carries a tag.
# * (AC 2, verdicts.) Each of the sections (a) to (e) has exactly one line
#   starting "verdict:" (optionally "**verdict:"), it is the LAST non-blank line
#   of its section (subsections of (e) go above it), it is not "pending", it
#   contains "mechanism:" or "explanation:" and a [code] or [measured] tag.
#   Sections (d) and (e) must use the literal forms "instrument bug,
#   mechanism:" or "not a bug, explanation:". (a), (b) and (c) keep their
#   own wording (checks 002-05, 002-07 and 002-09 own it).
# * (AC 3, sub-questions.) The table under "Open sub-questions" has a row for
#   each of the four investigation questions (voice comparison; I/O-pressure;
#   21.7 value; 5 ms timeout), each containing "answered" and a section
#   reference "(x)" or "section x". Any other row without "answered" names an
#   owner outside this epic and a falsifier.
# * (AC 4, evidence.) Verdict entries are identified BY CONTENT in every mode,
#   never by comparison with any git ref: a table row of the evidence appendix
#   whose FIRST cell carries a verdict letter "(a)".."(e)" and which mentions
#   REPORT-0b or "archaeology" is a verdict entry. At least one per letter; each
#   carries an evidence tag. Entries for other rows (e.g. the historical M-1 row)
#   are not verdict entries merely for mentioning REPORT-0b. No appendix row may
#   still say "Unresolved" with "Task 002" as its resolver (M-3, M-4 must be
#   annotated). The (e) entry must not say "not a bug".
# * (AC 5, constants.) Row P-02 of the constants table holds the two figures of
#   the line "**P-02 replacement** ..." (first bold "x / y"; the 50 Hz and 1000
#   Hz figures), carries [measured], says "strict" or "C-147" and "unprivileged"
#   or "POC", and no longer contains "37 / 77". The "Documents changed" list
#   names doc/design/02-constants.md and doc/design/06-evidence-appendix.md, every
#   path in it exists, and each bullet carries a tag. No check depends on any git
#   ref (later branches may legitimately edit C- rows).
# * (Stale sync-rate guard.) Any line of any .md under DESIGN_ROOT (default
#   doc/design; directories named tickets/ and the file REPORT-0b.md excluded)
#   that states 37 or 77 as an achieved sync or exchange rate (forms such as
#   "37/s", "37 per second", "37 exchanges", "37 completed", "77/s", "37 / 77",
#   "37 and 77"), in a line also carrying sync, exchange, configured, --sync-hz,
#   Hz, rate or 50/1000, must carry a marker: corrupt, superseded, earlier,
#   originally, corrected, 36.1 or 71.4. Every flagged file:line is printed.
#   "C-37", "AC-37", "37 bytes" and line numbers are not rate statements.
# * (Loss accounting, lead ruling.) No sentence or appendix row may say that the
#   loss counter or loss accounting (rx_lost, lost_fwd, loss display) is fixed by
#   task 003 ("fixed by 003", "003 fixes") without a negation in the sentence. The
#   report must carry a paragraph starting "Open item" that concerns the loss
#   accounting and names an owner and a falsifier.
# * (AC 2, (e) form, lead ruling.) Section (e) verdict must start "instrument
#   bug, mechanism:". Section (d) may use either literal form.
# * (DoD.) Every paragraph containing "open item" names an owner and a
#   falsifier.
#
# Exit status: 0 when every check passes, 1 otherwise.

set -u

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REPORT_REL="doc/design/reports/archaeology/REPORT-0b.md"
REPORT="${REPORT_0B:-$REPO_ROOT/$REPORT_REL}"
CONSTANTS="${CONSTANTS_MD:-$REPO_ROOT/doc/design/02-constants.md}"
EVIDENCE="${EVIDENCE_MD:-$REPO_ROOT/doc/design/06-evidence-appendix.md}"
DESIGN_ROOT="${DESIGN_ROOT:-$REPO_ROOT/doc/design}"
POC_ROOT="/home/dorian/workspace/hadropoc"
FROZEN_DATA_MANIFEST="$REPO_ROOT/checks/002-01-frozen-data.sha256"

if [ ! -f "$REPORT" ]; then
  printf 'FAIL: report missing: %s\n\n1 check(s) failed.\n' "$REPORT_REL"
  exit 1
fi
for f in "$CONSTANTS" "$EVIDENCE"; do
  if [ ! -f "$f" ]; then printf 'FAIL: missing file: %s\n\n1 check(s) failed.\n' "$f"; exit 1; fi
done
if [ ! -d "$DESIGN_ROOT" ]; then printf 'FAIL: design root missing: %s\n\n1 check(s) failed.\n' "$DESIGN_ROOT"; exit 1; fi

REPORT="$REPORT" CONSTANTS="$CONSTANTS" EVIDENCE="$EVIDENCE" \
REPO_ROOT="$REPO_ROOT" DESIGN_DIR="$DESIGN_ROOT" POC_ROOT="$POC_ROOT" \
FROZEN_DATA_MANIFEST="$FROZEN_DATA_MANIFEST" \
python3 - <<'PY'
import os, re, subprocess, hashlib

E = os.environ
REPORT, CONSTANTS, EVIDENCE = E['REPORT'], E['CONSTANTS'], E['EVIDENCE']
REPO, DESIGN_DIR, POC = E['REPO_ROOT'], E['DESIGN_DIR'], E['POC_ROOT']
MANIFEST = E['FROZEN_DATA_MANIFEST']

passes = 0
fails = 0


def ok(msg):
    global passes
    passes += 1
    print('PASS: ' + msg)


def bad(msg):
    global fails
    fails += 1
    print('FAIL: ' + msg)


def read_lines(path):
    with open(path, encoding='utf-8') as fh:
        return fh.read().splitlines()


TAG = re.compile(r'\[(code|measured|kernel-doc|memory|decision)\]')
HEAD = re.compile(r'^(#{1,6})\s+(.*\S)\s*$')


def fenced_mask(lines):
    """True for lines inside a ``` fence (fence lines included)."""
    mask, inside = [], False
    for l in lines:
        if l.lstrip().startswith('```'):
            mask.append(True)
            inside = not inside
        else:
            mask.append(inside)
    return mask


rep = read_lines(REPORT)
rmask = fenced_mask(rep)


def heading_level(i):
    if rmask[i]:
        return 0
    m = HEAD.match(rep[i])
    return len(m.group(1)) if m else 0


def heading_index(pattern):
    rx = re.compile(pattern, re.I)
    for i in range(len(rep)):
        if heading_level(i) and rx.search(rep[i]):
            return i
    return -1


def section_end(i, level):
    """First index after heading i whose heading level is <= level (or EOF)."""
    for j in range(i + 1, len(rep)):
        lv = heading_level(j)
        if lv and lv <= level:
            return j
    return len(rep)


def blocks(lines):
    """Split into blocks: blank lines and bullet starts separate blocks."""
    out, cur = [], []
    for l in lines:
        if not l.strip():
            if cur:
                out.append('\n'.join(cur))
            cur = []
            continue
        if re.match(r'^\s*([-*]|\d+\.)\s', l) and cur:
            out.append('\n'.join(cur))
            cur = []
        cur.append(l)
    if cur:
        out.append('\n'.join(cur))
    return out


def para_blocks(lines):
    out, cur = [], []
    for l in lines:
        if not l.strip():
            if cur:
                out.append('\n'.join(cur))
            cur = []
        else:
            cur.append(l)
    if cur:
        out.append('\n'.join(cur))
    return out


def cells(row):
    return [c.strip() for c in row.strip().strip('|').split('|')]


def is_table_row(l):
    return l.lstrip().startswith('|')


def is_sep(l):
    return bool(re.match(r'^\s*\|?\s*:?-{3,}', l))


# ---------------------------------------------------------------------------
# AC 1 — checklist of the seven epic criteria before the first lettered section
# ---------------------------------------------------------------------------
first_lettered = -1
for i in range(len(rep)):
    if heading_level(i) and re.match(r'^#+\s*\([a-e]\)', rep[i]):
        first_lettered = i
        break
if first_lettered < 0:
    bad('AC 1: no lettered section heading (a)..(e) found')
    pre = rep
else:
    pre = rep[:first_lettered]

head_texts = []
for i in range(len(rep)):
    if heading_level(i):
        t = HEAD.match(rep[i]).group(2)
        t = re.sub(r'\s*\([^)]*\)\s*$', '', t).strip()
        if len(t) >= 6:
            head_texts.append(t.lower())

for n in range(1, 8):
    rx = re.compile(r'002-AC-%d(?!\d)' % n)
    ticked = [l for l in pre if rx.search(l) and re.match(r'^\s*[-*]\s*\[[xX]\]', l)]
    unticked = [l for l in pre if rx.search(l) and re.match(r'^\s*[-*]\s*\[ \]', l)]
    if len(ticked) != 1:
        extra = ' (an unticked line exists)' if unticked else ''
        bad('AC 1: 002-AC-%d needs exactly one ticked "[x]" checklist line before section (a); found %d%s'
            % (n, len(ticked), extra))
        continue
    line = ticked[0]
    names_section = bool(re.search(r'\([a-e]\)', line)) or any(t in line.lower() for t in head_texts)
    if not names_section:
        bad('AC 1: checklist line for 002-AC-%d names no section (a)..(e) and no heading of the report' % n)
    elif not TAG.search(line):
        bad('AC 1: checklist line for 002-AC-%d carries no evidence tag' % n)
    else:
        ok('AC 1: 002-AC-%d ticked in the checklist, with its section and a tag' % n)

# ---------------------------------------------------------------------------
# AC 2 — exactly one verdict line per section (a)..(e), not pending, forms
# ---------------------------------------------------------------------------
for L in 'abcde':
    i = heading_index(r'^#+\s*\(%s\)' % L)
    if i < 0:
        bad('AC 2: section (%s) heading not found' % L)
        continue
    j = section_end(i, 2)
    sec = rep[i + 1:j]
    vl = [(k, l) for k, l in enumerate(sec) if re.match(r'^\s*(\*\*)?verdict:', l)]
    if len(vl) != 1:
        bad('AC 2: section (%s) must carry exactly one "verdict:" line; found %d' % (L, len(vl)))
        continue
    k, line = vl[0]
    body = re.sub(r'^\s*(\*\*)?verdict:\s*', '', line)
    body = re.sub(r'\s*\*\*\s*$', '', body).strip()
    if re.fullmatch(r'pending', body, re.I):
        bad('AC 2: section (%s) verdict is still pending' % L)
        continue
    nonblank = [idx for idx, l in enumerate(sec) if l.strip()]
    if nonblank[-1] != k:
        bad('AC 2: section (%s) verdict is not the last non-blank line of the section' % L)
        continue
    if not re.search(r'mechanism:|explanation:', body, re.I):
        bad('AC 2: section (%s) verdict has neither "mechanism:" nor "explanation:"' % L)
        continue
    if not re.search(r'\[(code|measured)\]', body):
        bad('AC 2: section (%s) verdict carries no [code] or [measured] tag' % L)
        continue
    if L == 'd' and not re.match(r'(instrument bug, mechanism:|not a bug, explanation:)', body):
        bad('AC 2: section (d) verdict must use one literal form: "instrument bug, mechanism:" or '
            '"not a bug, explanation:"')
        continue
    if L == 'e' and not re.match(r'instrument bug, mechanism:', body):
        bad('AC 2: section (e) verdict must use the form "instrument bug, mechanism:" (the sync pacing is an '
            'instrument bug in the design vocabulary: ADR 21, 00-context.md, appendix 0c); found: ' + body[:120])
        continue
    ok('AC 2: section (%s) has one final verdict, last in its section, tagged' % L)

# ---------------------------------------------------------------------------
# AC 3 — the four investigation sub-questions are answered in the table
# ---------------------------------------------------------------------------
oi = heading_index(r'Open sub-questions')
if oi < 0:
    bad('AC 3: "Open sub-questions" section not found')
else:
    oj = section_end(oi, 2)
    rows = [l for l in rep[oi + 1:oj] if is_table_row(l) and not is_sep(l)]
    rows = rows[1:]  # header row
    keys = {
        'voice comparison': r'voice comparison|bit-level',
        'I/O-pressure finding': r'I/O[- ]pressure',
        'the 21.7 M value': r'21\.7',
        'the 5 ms timeout': r'5 ms',
    }
    found = {}
    for row in rows:
        c = cells(row)
        answered = bool(re.search(r'\banswered\b', row, re.I))
        for name, pat in keys.items():
            if re.search(pat, c[0] if c else '', re.I):
                found[name] = row
        if not answered:
            owner = c[1] if len(c) > 1 else ''
            falsifier = c[2] if len(c) > 2 else ''
            if re.search(r'this epic|002-', owner, re.I):
                bad('AC 3: open sub-question not answered and owned inside this epic: %s' % c[0][:80])
            elif not owner or not falsifier:
                bad('AC 3: open sub-question not answered and names no owner or no falsifier: %s' % c[0][:80])
    for name in keys:
        if name not in found:
            bad('AC 3: investigation question "%s" has no row in the Open sub-questions table' % name)
            continue
        row = found[name]
        if not re.search(r'\banswered\b', row, re.I):
            bad('AC 3: "%s" is not marked answered' % name)
        elif not re.search(r'\([a-e]\)|section\s*\(?[a-e]\)?\b', row, re.I):
            bad('AC 3: "%s" is answered but names no section (x) of the report' % name)
        else:
            ok('AC 3: "%s" answered, with its section reference' % name)

# ---------------------------------------------------------------------------
# AC 4 — five new evidence-appendix entries, one per verdict, tagged
# ---------------------------------------------------------------------------
ev_lines = read_lines(EVIDENCE)
ev_rows = [l for l in ev_lines if is_table_row(l) and not is_sep(l)]
entries = {}
for r in ev_rows:
    c = cells(r)
    if not c or not re.search(r'REPORT-0b|archaeology', r, re.I):
        continue
    m = re.search(r'\(([a-e])\)', c[0])
    if m:
        entries.setdefault(m.group(1), []).append(r)
for L in 'abcde':
    rs = entries.get(L, [])
    if not rs:
        bad('AC 4: the evidence appendix has no verdict entry for (%s) (a row whose first cell carries "(%s)" '
            'and that mentions REPORT-0b)' % (L, L))
    elif not all(TAG.search(r) for r in rs):
        bad('AC 4: the verdict entry for (%s) carries no evidence tag' % L)
    else:
        ok('AC 4: evidence entry for verdict (%s) present and tagged' % L)
for r in entries.get('e', []):
    if re.search(r'not a bug', r, re.I):
        bad('AC 4: the appendix entry for verdict (e) says "not a bug"; the sync pacing is an instrument bug')

# M-3 / M-4: no appendix row still reads Unresolved with Task 002 as its resolver
for r in ev_rows:
    if re.search(r'\bUnresolved\b', r) and re.search(r'Task 002', r):
        bad('AC 4: an appendix row still says "Unresolved" with "Task 002" as its resolver: ' + r[:90])

# ---------------------------------------------------------------------------
# AC 5 — P-02 row, stale-statement guard, documents-changed list, C- rows
# ---------------------------------------------------------------------------
p02_line = next((l for l in rep if re.match(r'^\s*\**P-02 replacement\*\*', l)), None)
if p02_line is None:
    p02_line = next((l for l in rep if 'P-02 replacement' in l and re.search(r'replac', l, re.I)), None)
if p02_line is None:
    bad('AC 5: the report has no "P-02 replacement" line to parse the figures from')
    x = y = None
else:
    m = re.search(r'\*\*\s*(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)\s*\*\*', p02_line)
    if not m:
        m = re.search(r'(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)', p02_line)
    x, y = (m.group(1), m.group(2)) if m else (None, None)
    if x is None:
        bad('AC 5: no "x / y" figure pair on the P-02 replacement line')

const_lines = read_lines(CONSTANTS)
p02_row = None
for l in const_lines:
    if is_table_row(l):
        c = cells(l)
        if c and c[0] == 'P-02':
            p02_row = l
            break
if p02_row is None:
    bad('AC 5: row P-02 not found in the constants table')
elif x is not None:
    def has_num(s, n):
        return re.search(r'(?<![\d.])%s(?!\d)' % re.escape(n), s) is not None
    if not (has_num(p02_row, x) and has_num(p02_row, y)):
        bad('AC 5: P-02 row does not hold the clean 50 / 1000 Hz figures %s / %s from the report' % (x, y))
    elif not re.search(r'\[measured\]', p02_row):
        bad('AC 5: P-02 row is not tagged [measured]')
    elif not re.search(r'strict|C-147', p02_row, re.I):
        bad('AC 5: P-02 row does not say the figures come from runs without strict mode (C-147)')
    elif not re.search(r'unprivileged|POC', p02_row, re.I):
        bad('AC 5: P-02 row does not say the figures are POC / unprivileged figures')
    elif '37 / 77' in p02_row:
        bad('AC 5: P-02 row still reads "37 / 77"')
    else:
        ok('AC 5: P-02 holds %s / %s [measured] as POC figures from runs without strict mode' % (x, y))

# stale-statement guard (sync-rate statements of 37 or 77)
STALE_RX = re.compile(
    r'(?<![\w.,/-])(?:37|77)(?:\s*/\s*s\b|/s\b|\s*/\s*(?:37|77)\b|\s+and\s+(?:37|77)\b|'
    r'\s+(?:exchanges?|completed|per second)\b)|'
    r'\b(?:achieved|reached|delivered|gives?|gave)\s+(?:37|77)\b', re.I)
CTX_RX = re.compile(r'sync|exchange|configured|--sync-hz|\bhz\b|\brate\b|\b50\b|\b1000\b', re.I)
MARK = re.compile(r'corrupt|superseded|earlier|originally|corrected|36\.1|71\.4', re.I)
CLAUSE_SPLIT = r'(?<=\.)\s+|;\s+|:\s+'


def clauses(line):
    """A table row is split into cells; each cell and each prose line into clauses."""
    parts = cells(line) if is_table_row(line) else [line]
    out = []
    for part in parts:
        out.extend(u for u in re.split(CLAUSE_SPLIT, part) if u.strip())
    return out


stale_hits = []
for root, dirs, files in os.walk(DESIGN_DIR):
    dirs[:] = [d for d in dirs if d != 'tickets']
    for fn in sorted(files):
        if not fn.endswith('.md') or fn == 'REPORT-0b.md':
            continue
        path = os.path.join(root, fn)
        for n, l in enumerate(read_lines(path), 1):
            # a table row: the marker must sit in the same cell as the rate;
            # prose: the same clause
            if is_table_row(l):
                scoped = [(cl, c) for c in cells(l) for cl in re.split(CLAUSE_SPLIT, c)]
            else:
                scoped = [(cl, cl) for cl in re.split(CLAUSE_SPLIT, l)]
            for cl, scope in scoped:
                if STALE_RX.search(cl) and CTX_RX.search(cl) and not MARK.search(scope):
                    stale_hits.append((os.path.relpath(path, DESIGN_DIR), n, cl.strip()))
                    break
for rel, n, text in stale_hits:
    bad('stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) '
        '%s:%d: %s' % (rel, n, text[:220]))
if not stale_hits:
    ok('stale-statement guard: no design document states 37 or 77 as the achieved rate without a marker')

dc = heading_index(r'Documents changed')
if dc < 0:
    bad('AC 5: no "Documents changed" section in the report')
else:
    dj = section_end(dc, heading_level(dc))
    body = rep[dc + 1:dj]
    listed = set()
    bad_tag = False
    for l in body:
        paths = re.findall(r'\b((?:doc|checks|src|tools)/[A-Za-z0-9_./-]*[A-Za-z0-9_])', l)
        if not paths:
            continue
        for p in paths:
            listed.add(p)
            if not os.path.exists(os.path.join(REPO, p)):
                bad('AC 5: "Documents changed" lists a path that does not exist: %s' % p)
        if re.match(r'^\s*([-*]|\d+\.)\s', l) and not TAG.search(l):
            bad('AC 5: "Documents changed" entry carries no evidence tag: %s' % l.strip()[:120])
            bad_tag = True
    for must in ('doc/design/02-constants.md', 'doc/design/06-evidence-appendix.md'):
        if must not in listed:
            bad('AC 5: "Documents changed" does not list %s' % must)
    if not bad_tag and 'doc/design/02-constants.md' in listed:
        ok('AC 5: "Documents changed" lists the changed documents, paths exist, entries tagged')

# ---------------------------------------------------------------------------
# Loss accounting (lead ruling): no sentence says task 003 fixes it; open item
# ---------------------------------------------------------------------------
LOSS_RX = re.compile(r'loss counter|loss accounting|\brx_lost\b|\blost_fwd\b|loss display', re.I)
FIX003_RX = re.compile(
    r'\b(?:fix(?:e[sd])?|resolv(?:e[sd]?)|repair(?:s|ed)?)\b[^.;|]{0,40}\b003\b|'
    r'\b003\b[^.;|]{0,60}\bfix(?:es|ed)?\b|\bfixed by (?:task |ticket )?003\b', re.I)
NEG_RX = re.compile(r"\b(?:not|no|neither|nor|never|without|unfixed|cannot)\b|n't\b", re.I)


def loss_findings(line):
    """Clauses that say task 003 fixes the loss counter/accounting, with no negation
    in the clause. A table row must mention the loss counter somewhere; its
    fix clause is then checked cell by cell (negation is scoped to the clause)."""
    if is_table_row(line):
        if not LOSS_RX.search(line):
            return []
        cand = [cl for cell in cells(line) for cl in re.split(CLAUSE_SPLIT, cell)]
    else:
        cand = [u for u in re.split(r'(?<=\.)\s+', line) if LOSS_RX.search(u)]
    return [cl for cl in cand if FIX003_RX.search(cl) and not NEG_RX.search(cl)
            and (is_table_row(line) or LOSS_RX.search(cl))]


loss_hits = []
sources = [(REPORT, rep), (EVIDENCE, ev_lines)]
for root, dirs, files in os.walk(DESIGN_DIR):
    dirs[:] = [d for d in dirs if d != 'tickets']
    for fn in sorted(files):
        if fn.endswith('.md') and fn != 'REPORT-0b.md':
            p = os.path.join(root, fn)
            sources.append((p, read_lines(p)))
seen = set()
for path, lines in sources:
    if path in seen:
        continue
    seen.add(path)
    for n, l in enumerate(lines, 1):
        for u in loss_findings(l):
            loss_hits.append((os.path.basename(path), n, u.strip()))
for fn, n, text in loss_hits:
    bad('loss accounting stated as fixed by task 003 (no negation) %s:%d: %s' % (fn, n, text[:220]))
if not loss_hits:
    ok('no sentence states that task 003 fixes the POC loss counter or loss accounting')

loss_open = [p for p in para_blocks(rep)
             if re.match(r'^\W*Open item', p.strip(), re.I) and re.search(r'loss', p, re.I)]
if not loss_open:
    bad('no paragraph starting "Open item" about the POC loss accounting (owner and falsifier required)')
elif not any(re.search(r'owner', p, re.I) and re.search(r'falsifier', p, re.I) for p in loss_open):
    bad('the open item about the POC loss accounting names no owner or no falsifier')
else:
    ok('open item about the POC loss accounting names an owner and a falsifier')

# ---------------------------------------------------------------------------
# AC 6 — regime stamp "not applicable" line with its reason (C-125)
# ---------------------------------------------------------------------------
stamp_ok = False
for p in para_blocks(rep):
    if re.search(r'regime stamp', p, re.I) and re.search(r'not applicable', p, re.I) and \
       re.search(r'because|no stamp field|no runtime code', p, re.I):
        stamp_ok = True
        break
if stamp_ok:
    ok('AC 6: the regime stamp line is stated not applicable, with its reason')
else:
    bad('AC 6: no paragraph states "regime stamp" + "not applicable" + a reason (because / no stamp field / no runtime code)')

# ---------------------------------------------------------------------------
# AC 7 — frozen tree: the 002-14 mechanism (poc-untouched tag tree + manifest)
# ---------------------------------------------------------------------------
def hash_for(pattern):
    rx = re.compile(pattern, re.I)
    for i, l in enumerate(rep):
        if rx.search(l):
            m = re.search(r'([0-9a-fA-F]{64})', l)
            if m:
                return m.group(1)
            if i + 1 < len(rep):
                m = re.search(r'([0-9a-fA-F]{64})', rep[i + 1])
                if m:
                    return m.group(1)
            return None
    return None


def tag_fingerprint(root, tag):
    res = subprocess.run(['git', '-C', root, 'ls-tree', '-r', tag],
                         capture_output=True, text=True)
    if res.returncode != 0:
        return None
    items = []
    for line in res.stdout.splitlines():
        meta, path = line.split('\t', 1)
        parts = meta.split()
        if parts[1] != 'blob':
            continue
        if re.search(r'(^|/)(target|build|\.git)(/|$)', path):
            continue
        if re.search(r'(^|/)\.gitignore$', path):
            continue
        if path.startswith(('src/', 'tools/', 'experiments/')) or \
           path in ('Cargo.toml', 'Cargo.lock', 'README.md'):
            items.append('%s\t%s' % (path, parts[2]))
    h = hashlib.sha256()
    for item in sorted(items):
        path, oid = item.split('\t')
        blob = subprocess.run(['git', '-C', root, 'cat-file', 'blob', oid],
                              capture_output=True).stdout
        h.update(b'path:' + path.encode() + b'\0' + blob + b'\0')
    return h.hexdigest()


recorded = hash_for(r'poc-untouched tag-tree fingerprint')
tag_ok = subprocess.run(['git', '-C', POC, 'rev-parse', '--verify', '--quiet',
                         'refs/tags/poc-untouched^{commit}'],
                        capture_output=True).returncode == 0
computed = tag_fingerprint(POC, 'poc-untouched') if tag_ok else None
if not recorded:
    bad('AC 7: the Environment section records no 64-hex poc-untouched tag-tree fingerprint')
elif not computed:
    bad('AC 7: cannot recompute the poc-untouched tag-tree fingerprint in %s' % POC)
elif computed.lower() != recorded.lower():
    bad('AC 7: frozen-tree fingerprint changed (recorded=%s computed=%s)' % (recorded, computed))
else:
    ok('AC 7: poc-untouched tag-tree fingerprint recomputed from git matches the recorded value')

entries = []
try:
    for l in read_lines(MANIFEST):
        if not l.strip() or l.startswith('#'):
            continue
        f = l.split()
        entries.append((f[0], f[1]))
except OSError:
    bad('AC 7: frozen-data manifest missing: checks/002-01-frozen-data.sha256')
if entries:
    mism = 0
    allowed = ('experiments/pilot/', 'experiments/run1/', 'experiments/run2/', 'experiments/run3/')
    for exp, path in entries:
        if not path.startswith(allowed):
            bad('AC 7: manifest entry outside the frozen experiment-data dirs: %s' % path)
            mism += 1
            continue
        fp = os.path.join(POC, path)
        if not os.path.isfile(fp):
            bad('AC 7: frozen experiment data missing: %s' % path)
            mism += 1
            continue
        with open(fp, 'rb') as fh:
            actual = hashlib.sha256(fh.read()).hexdigest()
        if actual != exp:
            bad('AC 7: frozen experiment data changed: %s' % path)
            mism += 1
    present = []
    for d in ('experiments/pilot', 'experiments/run1', 'experiments/run2', 'experiments/run3'):
        for root, _, files in os.walk(os.path.join(POC, d)):
            for fn in files:
                present.append(os.path.relpath(os.path.join(root, fn), POC))
    if len(entries) != 27:
        bad('AC 7: manifest has %d entries, expected 27' % len(entries))
    if sorted(p for _, p in entries) != sorted(present):
        bad('AC 7: frozen experiment-data files added or missing under experiments/')
    elif mism == 0 and len(entries) == 27:
        ok('AC 7: 27 frozen untracked experiment-data files match checks/002-01-frozen-data.sha256')

h01 = hash_for(r'frozen[- ]tree fingerprint, taken')
recheck_ok = False
for p in para_blocks(rep):
    if re.search(r're-?check|re-?checked|re-?verified|recheck', p, re.I) and \
       re.search(r'fingerprint', p, re.I) and \
       re.search(r'unchanged|identical|match', p, re.I) and \
       ((computed and computed in p) or (h01 and h01 in p)):
        recheck_ok = True
        break
if recheck_ok:
    ok('AC 7: the report states the re-check of the frozen-tree fingerprint, unchanged')
else:
    bad('AC 7: no paragraph states the re-check of the frozen-tree fingerprint (with "fingerprint", '
        '"unchanged"/"identical"/"match" and the value)')

# ---------------------------------------------------------------------------
# Hand-offs
# ---------------------------------------------------------------------------
hi = heading_index(r'Hand-off')
if hi < 0:
    bad('Hand-offs: no heading containing "Hand-off"')
else:
    hj = section_end(hi, heading_level(hi))
    hb = blocks(rep[hi + 1:hj])
    specs = [
        ('003', lambda b: re.search(r'\b003\b', b) and re.search(r'confirm', b, re.I) and re.search(r'\(a\)', b),
         'hand-off to 003 (bugs confirmed, the (a) answer)'),
        ('007+017', lambda b: re.search(r'007', b) and re.search(r'017', b) and re.search(r'exact.tier', b, re.I)
         and re.search(r'analyse', b, re.I), 'hand-off to 007 and 017 (exact-tier baseline, what analyse compares)'),
        ('011', lambda b: re.search(r'\b011\b', b) and re.search(r'I/O', b) and
         re.search(r'inherit|dropped', b, re.I), 'hand-off to 011 (I/O-pressure rule inherited or dropped)'),
    ]
    for key, pred, label in specs:
        matches = [b for b in hb if pred(b)]
        if not matches:
            bad('Hand-offs: missing %s' % label)
        elif not TAG.search(matches[0]):
            bad('Hand-offs: the %s carries no evidence tag' % label)
        else:
            ok('Hand-offs: %s' % label)

# ---------------------------------------------------------------------------
# 002-10 open item (run-1 degradation, owned by 002-13 in the 002-10 text)
# ---------------------------------------------------------------------------
s10 = heading_index(r'^#+\s*.*002-10')
if s10 < 0:
    bad('002-10 open item: the 002-10 subsection was not found')
else:
    e10 = section_end(s10, 3)
    sub = rep[s10 + 1:e10]
    cands = [p for p in para_blocks(sub)
             if re.search(r'run 1', p, re.I) and
             re.search(r'sign|falsifier|open item|undecided|owner', p, re.I)]
    if not cands:
        bad('002-10 open item: no run-1 degradation paragraph found in the 002-10 subsection')
    else:
        p = cands[0]
        flat = ' '.join(p.split())
        still_open = re.search(r'open item|undecided|owner', flat, re.I)
        closed = (not still_open and re.search(r'(^|\W)F\s*=\s*\d+', flat) and
                  re.search(r'(^|\W)B\s*=\s*\d+', flat) and TAG.search(flat) and
                  re.search(r'sign[- ]test', flat, re.I))
        mo = re.search(r'owner[^.;]*', flat, re.I)
        reowned = bool(mo and re.search(r'\b\d{3}-\d{2}\b|milestone', mo.group(0)) and
                       '002' not in mo.group(0) and re.search(r'falsifier', flat, re.I))
        if closed:
            ok('002-10 open item: closed with the paired sign-test counts, [measured]')
        elif reowned:
            ok('002-10 open item: re-owned by a named later ticket or milestone, with its falsifier')
        else:
            bad('002-10 open item (run-1 degradation) is still owned by this epic / 002-13 with no sign-test '
                'result: current text: ' + flat[:1500])

# DoD: every paragraph stating an open item names an owner and a falsifier
for p in para_blocks(rep):
    if re.search(r'open item', p, re.I):
        if not (re.search(r'owner', p, re.I) and re.search(r'falsifier', p, re.I)):
            bad('DoD: an open-item paragraph names no owner or no falsifier: ' + ' '.join(p.split())[:120])
            break
else:
    ok('DoD: every open-item paragraph names an owner and a falsifier')

print('')
total = passes + fails
if fails:
    print('%d check(s) failed, %d passed.' % (fails, passes))
else:
    print('%d passed, 0 failed' % passes)
raise SystemExit(1 if fails else 0)
PY
