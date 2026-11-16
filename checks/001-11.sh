#!/usr/bin/env bash
# checks/001-11.sh — checks for ticket 001-11
# "Ethernet bit-error figure and loss floor".
#
# The deliverable is the draft doc/design/reports/REPORT-0a-ber.md (C-130),
# written by the author. It is a section that 001-13 includes in
# REPORT-0a.md, so this check reads only that draft and never depends on its
# headings. It states the IEEE 802.3 bit-error-ratio figure for 1000BASE-T and
# for 10GBASE-T, the recomputed loss floor, the C-118 soak prediction, the
# C-179 reopen count and a one-line verdict on C-90.
#
# The check never writes anywhere and never runs the POC. Its only other read
# is `git diff --quiet origin/main -- doc/design/02-constants.md`, which asserts
# that this ticket does not edit the constants table (001-14 does that). When
# origin/main does not exist (a clone without it) that check is skipped with a
# NOTE.
#
# Negation. Every keyword test below ignores an occurrence that its own clause
# negates: a clause is the text since the last `,` `;` `:` or `.`, and
# a negation is "not", "never", "no", "nor", "cannot", "without", "neither" or
# "n't" in that clause before the keyword. "1000BASE-T never governs" and
# "is not answered" therefore do not count.
#
# Override: REPORT_0A_BER=<path> reads another draft (a relative path is taken
# from the repository root). With the override the constants check is skipped
# with a NOTE, because a fixture is not the repository.
#
# The conventions below make the prose machine-checkable without fixing its
# wording. "Sentence" means a run of prose between full stops or semicolons.
# "Row" means a table row. "Paragraph" means a run of prose lines between
# blank lines (a list item starts a new paragraph). Fenced code is ignored.
#
# * (AC 1, citations.) For each of 1000BASE-T and 10GBASE-T, one sentence or
#   row names the PHY and a BER figure written as a power of ten (1e-12,
#   10^-12, 10⁻¹², 1 × 10^-12), and also contains:
#     - "IEEE" and "802.3";
#     - an edition designation, as the ticket words it: a published edition
#       ("802.3-2018", "802.3an-2006", "802.3ab-1999", "802.3 (2018 edition)",
#       "IEEE Std 802.3, 2018"), a working-group draft ("P802.3ab Draft 4.0",
#       "802.3ab D4.0"), or a named comment record or test suite with its
#       designation ("comment #155", "PMA test suite v2.5"). A bare date, or a
#       year outside such a designation, does not count. Design owner's
#       decision of 2026-11-16: working-group text is accepted for [standard]
#       provided it is named as what it is (draft, comment record or test
#       suite), never as the published edition. A citation of a draft, comment
#       record or test suite is accepted; a published edition is not forced;
#     - a clause with a subclause: "Clause 55.4.2" or "subclause 40.6.1.2"
#       (the keyword, then a dotted number with at least one dot);
#     - the tag [standard]. If the standard could not be consulted, [memory]
#       is accepted instead, but only with the fallback statement below.
#   Two complete citations of one PHY that disagree on the exponent FAIL.
#
# * (Governance.) One sentence names GbE or gigabit, says "govern" or "applies"
#   (not negated, see above), names exactly one of the two PHYs (1000BASE-T for
#   the GbE default links), gives a reason ("because", "since", "reason", "why"
#   or "due to") and carries [decision]. The governing BER is that PHY's cited
#   figure. Lead ruling 2026-11-16: 1000BASE-T is the governing PHY for the GbE
#   default links (the ticket's reference line and the design owner's decision).
#   This is ratified, not a design choice of this check.
#
# * (AC 2 and AC 3, loss-floor table.) A table whose header has cells matching
#   BER, per-packet / probability, hours, soak (not reopen) and reopen. Each row
#   gives a BER power of ten, the per-packet probability p, the mean hours
#   between losses per stream per link, the C-118 soak loss count and the C-179
#   reopen count. The check recomputes, from each row's BER:
#     p = BER × 1 320 × 8 = BER × 10 560
#     hours = 1 / (p × 2 000) / 3 600
#     soak = p × 64 000 × 86 400
#     reopen = 10 × soak
#   and requires each stated value to equal the recomputed one to two
#   significant figures. The governing row (the governing BER) must exist; any
#   other row is checked the same way. The draft must state the inputs
#   "1 320 bytes", "10 560 bits", "2 000 packets/s", "64 000 packets/s" and
#   "86 400", and a soak packet count of about 5.5e9.
#
# * (AC 4, verdict.) The ticket (001-11, AC 4) asks for "A one-line verdict:
#   C-90 confirmed, or corrected value proposed with the reason". Exactly one
#   line beginning with "verdict:" (case-insensitive, optional bold, list or
#   quote markers). Exactly one of the two verdict words, "confirmed" or
#   "corrected", must occur as a positive (not negated) verdict; a line that
#   carries both words, or only a negated one ("not corrected", "is correct and
#   is not corrected", "not confirmed"), is not a verdict. Its wording decides the kind:
#     "confirmed"          C-90 is confirmed;
#     "corrected" / "correction" / "revised" / "proposed" with a reason ("because", ...)
#                          and, in the same line, the corrected p, the C-118
#                          count and the C-179 reopen count (each to two
#                          significant figures, in any form);
#     "pending"            only when a [memory] figure is kept (fallback).
#   Consistency with the governing BER: 1e-12 requires "confirmed"; any other
#   governing BER requires a correction (or pending under the fallback). A
#   "confirmed" verdict with a [memory] figure FAILS.
#
# * (Fallback, C-132.) A BER figure or loss-floor row tagged [memory] is allowed
#   only when a paragraph says the standard's text "could not be consulted" (or
#   similar) and names the checker by ticket id (e.g. 001-12) or "task NNN". The
#   verdict must then say "pending". Without that statement, [memory] FAILS.
#
# * (DoD.) Every paragraph that carries a BER figure or a loss-floor quantity
#   (per-packet, soak, reopen, hours between losses) and a digit carries an
#   evidence tag. Every loss-floor row is tagged [standard] (or [memory] under
#   the fallback). The verdict line carries a tag.
#
# * (DoD, open items.) The paragraph on the sub-question "Is the loss floor C-90
#   computed from the right bit-error figure?" says it is answered (or handed to
#   L1, task 019; "answered" in the sense of the ticket, not "not answered" or
#   "unanswered") and names a falsifier. Any paragraph saying "open item" or
#   "open sub-question" names an owner (the word "owner", "L1"-"L3" or "task NNN")
#   and a falsifier.
#
# * (C-125, not applicable.) The not-applicable lines need no assertion.

set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_REL="doc/design/reports/REPORT-0a-ber.md"

if [ -n "${REPORT_0A_BER:-}" ]; then
  REPORT="$REPORT_0A_BER"
  OVERRIDDEN=1
else
  REPORT="$REPO_ROOT/$REPORT_REL"
  OVERRIDDEN=0
fi
case "$REPORT" in
  /*) ;;
  *) REPORT="$REPO_ROOT/$REPORT" ;;
esac

pass=0
fail=0

ok()  { printf 'PASS: %s\n' "$*"; pass=$((pass + 1)); }
bad() { printf 'FAIL: %s\n' "$*"; fail=$((fail + 1)); }

# ---------------------------------------------------------------------------
# The draft: present, then the python analysis (one OK/BAD line per result)
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "draft not found: $REPORT (the author writes $REPORT_REL, C-130; 001-11 deliverable)"
else
  read -r -d '' PYCODE <<'PY' || true
import re, sys

path = sys.argv[1]
raw = open(path, encoding='utf-8').read()
out = []


def emit(kind, msg):
    out.append(kind + '\t' + msg)


SUPCH = '⁰¹²³⁴⁵⁶⁷⁸⁹'
SUP = str.maketrans(SUPCH, '0123456789')


def norm(s):
    s = s.replace(' ', ' ').replace(' ', ' ').replace('‑', '-')
    s = s.replace('−', '-').replace('⁻', '-').replace('×', 'x')
    s = re.sub(r'10\s*-([%s]+)' % SUPCH, lambda m: '10^-' + m.group(1).translate(SUP), s)
    s = re.sub(r'\^\{\s*(-?\d+)\s*\}', r'^\1', s)
    s = re.sub(r'[%s]+' % SUPCH, lambda m: m.group(0).translate(SUP), s)
    return s


BER_RE = re.compile(r'(?<![\w.])(?:1\s*x\s*)?(?:1[eE]|10\^)(-\d+)(?![\d.])')
SCI_RE = re.compile(r'(\d+(?:\.\d+)?)\s*(?:[eE]\s*([+-]?\d+)|x\s*10\^\s*([+-]?\d+))')
PLAIN_RE = re.compile(r'\d+(?:[ ,]\d{3})*(?:\.\d+)?')
TAG_RE = re.compile(r'\[(?:standard|memory|decision|measured|code|kernel-doc)\]')
BERKW = re.compile(r'\bBER\b|bit[- ]error', re.I)
SUBCL_RE = re.compile(r'(?:[Cc]lauses?|[Ss]ubclauses?|§|\bcl\.)\s*\d{1,2}[A-Za-z]?\.\d+')
MEMKW = re.compile(r'per[- ]packet|soak|reopen|hours between', re.I)
CONSULT_RE = re.compile(r'could not be consulted|could not consult|not been consulted|not consulted|unable to consult|could not access', re.I)
PHYS = ['1000BASE-T', '10GBASE-T']


def ber_exps(s):
    return [int(m.group(1)) for m in BER_RE.finditer(norm(s))]


def parse_num(s):
    n = norm(s)
    m = SCI_RE.search(n)
    if m:
        return float(m.group(1) + 'e' + (m.group(2) or m.group(3)))
    m = PLAIN_RE.search(n)
    if m:
        return float(re.sub(r'[ ,]', '', m.group(0)))
    return None


def all_nums(s):
    n = norm(s)
    vals = [float(m.group(1) + 'e' + (m.group(2) or m.group(3))) for m in SCI_RE.finditer(n)]
    vals += [float(re.sub(r'[ ,]', '', m.group(0))) for m in PLAIN_RE.finditer(n)]
    return vals


def sf2(x):
    return float('%.1e' % x)


def snip(s, n=110):
    s = ' '.join(s.split())
    return s if len(s) <= n else s[:n] + '...'


def cells(r):
    return [c.strip() for c in r.strip().strip('|').split('|')]


def is_sep(r):
    return bool(re.fullmatch(r'\|?[\s|:-]*\|?', r.strip())) and '-' in r


def find(hdr, pat, skip=None):
    for i, h in enumerate(hdr):
        if i == skip:
            continue
        if re.search(pat, h, re.I):
            return i
    return None


NEG_RE = re.compile(r"\b(?:not|never|no|nor|cannot|without|neither)\b|n't\b", re.I)
CLAUSE_END_RE = re.compile(r'[,;:.]')


def negated_at(s, pos):
    # the clause before pos: the text since the last clause boundary
    head = s[:pos]
    cut = 0
    for m in CLAUSE_END_RE.finditer(head):
        cut = m.end()
    return bool(NEG_RE.search(head[cut:]))


def positive(s, pat, flags=re.I):
    # True if pat occurs in s at least once, and some occurrence is not negated
    return any(not negated_at(s, m.start()) for m in re.finditer(pat, s, flags))


# ---------------------------------------------------------------------------
# Structure: lines outside fences, paragraphs, tables, units
# ---------------------------------------------------------------------------

nf = []
fence = False
for ln in raw.split('\n'):
    st = ln.strip()
    if st.startswith('```') or st.startswith('~~~'):
        fence = not fence
        continue
    if not fence:
        nf.append(ln)

paras, tables, para, cur = [], [], [], []


def flush():
    if para:
        paras.append(' '.join(para))
        del para[:]


for ln in nf:
    st = ln.strip()
    if st.startswith('|'):
        flush()
        cur.append(st)
        continue
    if cur:
        tables.append(cur)
        cur = []
    if not st or st.startswith('#'):
        flush()
        continue
    if re.match(r'(?:[-*+]|\d+[.)])\s+', st):
        flush()
    para.append(st)
flush()
if cur:
    tables.append(cur)

units = []
for p in paras:
    units += re.split(r'(?<=[.;])\s+', p)
for t in tables:
    units += [r for r in t if not is_sep(r)]

# ---------------------------------------------------------------------------
# Fallback (C-132) and [memory] figures
# ---------------------------------------------------------------------------

fallback = any(CONSULT_RE.search(p) and re.search(r'\b\d{3}-\d{2}\b|task \d{3}', p) for p in paras)
mem_units = [u for u in units if '[memory]' in u and (ber_exps(u) or MEMKW.search(u))]
memory_present = bool(mem_units)

# ---------------------------------------------------------------------------
# AC 1: citations
# ---------------------------------------------------------------------------


# An edition designation (T4, ticket wording "edition"). A year only counts as part
# of one of these. Drafts, comment records and test suites are designations too
# (design owner, 2026-11-16); they are not forced into a published edition.
EDITION_RES = [
    r'802\.3[a-z]{0,3}\s*-\s*(?:19|20)\d{2}',                        # 802.3-2018, 802.3an-2006
    r'802\.3[a-z]{0,3}\s*\(?\s*(?:19|20)\d{2}\s+edition',           # 802.3 (2018 edition)
    r'Std\.?\s*802\.3[a-z]{0,3}\s*,\s*(?:19|20)\d{2}\b',             # IEEE Std 802.3, 2018
    r'802\.3[a-z]{0,3}\b[^.;]{0,40}?(?:(?i:draft)\s*\d|\bD\d)',      # P802.3ab Draft 4.0, 802.3ab D4.0
    r'\b(?:comment|record|database|suite)\b[^.;]{0,40}?(?:#\d+|\bv\d+(?:\.\d+)*\b|\bD\d|(?i:draft)\s*\d)',
    r'\bv\d+(?:\.\d+)*\b[^.;]{0,20}?\b(?:suite|record|database)\b',  # v2.5 test suite
]


def missing(u):
    m = []
    if not ber_exps(u):
        m.append('a BER figure as a power of ten')
    if not BERKW.search(u):
        m.append('"BER" or "bit error" wording')
    if not (re.search(r'IEEE', u) and re.search(r'802\.3', u)):
        m.append('the IEEE 802.3 standard name')
    if not any(re.search(p, u) for p in EDITION_RES):
        m.append('an edition designation (802.3-YYYY, "802.3 (YYYY edition)", a draft such as "P802.3ab Draft 4.0", or a named comment record or test suite with its designation; a bare date does not count)')
    if not SUBCL_RE.search(u):
        m.append('a clause with a subclause (a dotted number, e.g. Clause 55.4.2)')
    if '[standard]' not in u:
        if '[memory]' in u and fallback:
            pass
        elif '[memory]' in u:
            m.append('[standard] (a [memory] tag needs the fallback statement and a checker ticket)')
        else:
            m.append('the [standard] tag')
    return m


cited = {}
for phy in PHYS:
    cands = [u for u in units if phy in u and (ber_exps(u) or BERKW.search(u))]
    complete, best = [], None
    for u in cands:
        miss = missing(u)
        if not miss:
            complete.append(u)
        elif best is None or len(miss) < len(best[1]):
            best = (u, miss)
    if complete:
        exps = sorted({ber_exps(u)[0] for u in complete})
        if len(exps) > 1:
            emit('BAD', 'AC 1: %s is cited with conflicting BER figures (%s); one figure per PHY'
                 % (phy, ', '.join('1e%d' % e for e in exps)))
            cited[phy] = exps[0]
        else:
            cited[phy] = exps[0]
            emit('OK', 'AC 1: %s cites BER 1e%d with standard, edition, clause and subclause, tagged %s: "%s"'
                 % (phy, exps[0], '[memory] (fallback)' if '[standard]' not in complete[0] else '[standard]', snip(complete[0])))
    elif best is not None:
        u, miss = best
        cited[phy] = ber_exps(u)[0] if ber_exps(u) else None
        emit('BAD', 'AC 1: no complete citation for %s; the closest sentence "%s" lacks: %s'
             % (phy, snip(u), '; '.join(miss)))
    else:
        cited[phy] = None
        emit('BAD', 'AC 1: no sentence or row names %s with a BER figure (1e-x or 10^-x) and "bit error"/BER wording' % phy)

# ---------------------------------------------------------------------------
# Governance: which cited figure governs the GbE default links
# ---------------------------------------------------------------------------

# T1: a negated keyword ("never governs", "does not govern") is not a statement
gov_units = [u for u in units
             if positive(u, r'govern|applies')
             and re.search(r'GbE|gigabit', u, re.I)
             and any(p in u for p in PHYS)]
e_gov = None
if not gov_units:
    emit('BAD', 'AC 4 (governance): no sentence positively says which cited figure governs the GbE default links ("govern" or "applies", not negated, with GbE and the PHY name)')
else:
    u = gov_units[0]
    named = [p for p in PHYS if p in u]
    if len(named) != 1:
        emit('BAD', 'AC 4 (governance): the governing sentence must name exactly one PHY; it names %d: "%s"' % (len(named), snip(u)))
    elif named[0] != '1000BASE-T':
        emit('BAD', 'AC 4 (governance): the GbE default links are gigabit copper (1000BASE-T); %s cannot be the governing figure: "%s"' % (named[0], snip(u)))
    else:
        e_gov = cited.get('1000BASE-T')
        reason_ok = bool(re.search(r'because|since|reason|\bwhy\b|due to', u, re.I))
        tag_ok = '[decision]' in u
        if not reason_ok:
            emit('BAD', 'AC 4 (governance): the governing sentence gives no reason (because/since/reason/why/due to): "%s"' % snip(u))
        if not tag_ok:
            emit('BAD', 'DoD (governance): the choice of governing figure carries no [decision] tag: "%s"' % snip(u))
        if reason_ok and tag_ok:
            emit('OK', 'AC 4 (governance): the 1000BASE-T figure governs the GbE default links, with a reason and [decision]: "%s"' % snip(u))

# ---------------------------------------------------------------------------
# AC 2 and AC 3: the loss-floor table and its recomputation
# ---------------------------------------------------------------------------

loss_rows, loss_tables = [], 0
for t in tables:
    hdr = cells(t[0])
    ber_i = find(hdr, r'^\W*ber\b')
    p_i = find(hdr, r'per[- ]packet|probab')
    h_i = find(hdr, r'hour')
    re_i = find(hdr, r'reopen')
    so_i = find(hdr, r'soak', skip=re_i)
    if None in (ber_i, p_i, h_i, re_i, so_i):
        continue
    loss_tables += 1
    for r in t[1:]:
        if is_sep(r):
            continue
        c = cells(r)
        g = lambda i: c[i] if i < len(c) else ''
        exps = ber_exps(g(ber_i))
        if not exps:
            emit('BAD', 'AC 2: loss-floor row without a BER power of ten: "%s"' % snip(r))
            continue
        loss_rows.append({'e': exps[0], 'row': r, 'p': parse_num(g(p_i)), 'h': parse_num(g(h_i)),
                          'so': parse_num(g(so_i)), 'ro': parse_num(g(re_i))})

if loss_tables == 0:
    emit('BAD', 'AC 2: no loss-floor table with BER, per-packet probability, hours, soak and reopen columns')
if e_gov is not None and not any(r['e'] == e_gov for r in loss_rows):
    emit('BAD', 'AC 2: no loss-floor row for the governing BER 1e%d' % e_gov)

for r in loss_rows:
    e = r['e']
    p_rec = 10560.0 * 10.0 ** e
    h_rec = 1.0 / (p_rec * 2000.0) / 3600.0
    so_rec = p_rec * 64000.0 * 86400.0
    ro_rec = 10.0 * so_rec
    lab = '1e%d' % e
    gov = ' (governing)' if e == e_gov else ''
    if r['p'] is None:
        emit('BAD', 'AC 2: %s%s row states no per-packet probability p' % (lab, gov))
    elif sf2(r['p']) != sf2(p_rec):
        emit('BAD', 'AC 2: %s%s per-packet p stated %s, recomputed %s = BER x 10 560 (2 significant figures)'
             % (lab, gov, '%.2e' % r['p'], '%.2e' % p_rec))
    else:
        emit('OK', 'AC 2: %s%s per-packet p %s matches BER x 10 560 = %s' % (lab, gov, '%.2e' % r['p'], '%.2e' % p_rec))
    if r['h'] is None:
        emit('BAD', 'AC 2: %s%s row states no mean hours between losses' % (lab, gov))
    elif sf2(r['h']) != sf2(h_rec):
        emit('BAD', 'AC 2: %s%s mean hours between losses stated %g, recomputed %.3g h = 1 / (p x 2 000) / 3 600 (2 significant figures)'
             % (lab, gov, r['h'], h_rec))
    else:
        emit('OK', 'AC 2: %s%s mean hours between losses %g agrees with recomputed %.3g h' % (lab, gov, r['h'], h_rec))
    if r['so'] is None:
        emit('BAD', 'AC 3: %s%s row states no C-118 soak loss count' % (lab, gov))
    elif sf2(r['so']) != sf2(so_rec):
        emit('BAD', 'AC 3: %s%s soak count stated %g, recomputed %.3g = p x 64 000 x 86 400 (2 significant figures)'
             % (lab, gov, r['so'], so_rec))
    else:
        emit('OK', 'AC 3: %s%s soak count %g agrees with recomputed %.3g' % (lab, gov, r['so'], so_rec))
    if r['ro'] is None:
        emit('BAD', 'AC 3: %s%s row states no C-179 reopen count' % (lab, gov))
    elif sf2(r['ro']) != sf2(ro_rec):
        emit('BAD', 'AC 3: %s%s reopen count stated %g, recomputed %.3g = 10 x soak (2 significant figures)'
             % (lab, gov, r['ro'], ro_rec))
    else:
        emit('OK', 'AC 3: %s%s reopen count %g agrees with 10 x soak = %.3g' % (lab, gov, r['ro'], ro_rec))
    if '[standard]' not in r['row'] and not ('[memory]' in r['row'] and fallback):
        emit('BAD', 'DoD: loss-floor row 1e%d is not tagged [standard] (C-132): "%s"' % (e, snip(r['row'])))

ntext = norm(raw)
for pat, label in [
    (r'(?<!\d)1[ ,]?320\s*-?\s*bytes?', 'the 1 320-byte input'),
    (r'10[ ,]?560\s*bits?', 'the 10 560-bit input'),
    (r'(?<!\d)2[ ,]?000\s*packets?\s*(?:/|per)\s*s\b', 'the 2 000 packets/s input'),
]:
    if re.search(pat, ntext, re.I):
        emit('OK', 'AC 2: states %s' % label)
    else:
        emit('BAD', 'AC 2: does not state %s' % label)

if re.search(r'64[ ,]?000\s*packets?\s*(?:/|per)\s*s\b', ntext, re.I) and re.search(r'86[ ,]?400', ntext):
    emit('OK', 'AC 3: states the soak inputs 64 000 packets/s and 86 400 s')
else:
    emit('BAD', 'AC 3: does not state the soak inputs (64 000 packets/s, 86 400 s)')

if re.search(r'(?<![\d.])5\.5\s*(?:e|x\s*10\^)\s*\+?9\b', ntext):
    emit('OK', 'AC 3: states the soak packet count of about 5.5e9')
else:
    emit('BAD', 'AC 3: does not state the soak packet count of about 5.5e9')

# ---------------------------------------------------------------------------
# AC 4: the verdict
# ---------------------------------------------------------------------------

VERDICT_RE = re.compile(r'^(?:[-*+]\s+|>\s*)?\**\s*verdict\s*\**\s*:\s*\**\s*(.*)$', re.I)
vmatches = []
for ln in nf:
    m = VERDICT_RE.match(ln.strip())
    if m:
        vmatches.append((ln.strip(), m.group(1).replace('*', '').strip()))

vtext = None
if len(vmatches) == 0:
    emit('BAD', 'AC 4: no verdict line (exactly one line beginning "verdict:")')
elif len(vmatches) > 1:
    emit('BAD', 'AC 4: %d verdict lines; exactly one is allowed' % len(vmatches))
    vtext = vmatches[0][1]
else:
    emit('OK', 'AC 4: exactly one verdict line')
    vtext = vmatches[0][1]

vline = vmatches[0][0] if vmatches else ''
if vtext is not None:
    # T2: exactly one of the two verdict words, positive (not "not corrected",
    # not "is correct"); a line with both words is ambiguous
    CONF_W = r'\bconfirmed\b'
    CORR_W = r'\b(?:corrected|correction|revised|proposed)\b'
    t = vtext.lower()
    words_conf = bool(re.search(CONF_W, vtext, re.I))
    words_corr = bool(re.search(CORR_W, vtext, re.I))
    has_conf = positive(vtext, CONF_W)
    has_corr = positive(vtext, CORR_W)
    if 'pending' in t:
        kind = 'pending'
    elif words_conf and words_corr:
        kind = 'ambiguous'
    elif has_conf:
        kind = 'confirmed'
    elif has_corr:
        kind = 'corrected'
    else:
        kind = 'unknown'

    if not TAG_RE.search(vline):
        emit('BAD', 'DoD: the verdict line carries no evidence tag')

    if e_gov is None:
        emit('BAD', 'AC 4: the governing BER is unknown, so the verdict cannot be checked')
    elif memory_present:
        if not fallback:
            emit('BAD', 'C-132: a [memory] BER figure or loss-floor row is kept without a statement that the standard could not be consulted and a named checker ticket: "%s"' % snip(mem_units[0]))
        if kind == 'confirmed':
            emit('BAD', 'AC 4: the verdict says "confirmed" while a [memory] figure is kept; it must say pending the check')
        elif kind != 'pending':
            emit('BAD', 'AC 4: a [memory] figure is kept, so the verdict must say the confirmation is pending (kind: %s)' % kind)
        else:
            emit('OK', 'AC 4: the verdict says pending while the [memory] figure is kept')
    elif e_gov == -12:
        if kind == 'confirmed':
            emit('OK', 'AC 4: governing BER 1e-12 matches C-90; the verdict confirms C-90')
        else:
            emit('BAD', 'AC 4: governing BER is 1e-12 (C-90\'s figure), so the verdict must be "C-90 confirmed"; it reads as %s' % kind)
    else:
        p_rec = 10560.0 * 10.0 ** e_gov
        so_rec = p_rec * 64000.0 * 86400.0
        ro_rec = 10.0 * so_rec
        if kind == 'confirmed':
            emit('BAD', 'AC 4: the verdict says "confirmed" but the governing BER is 1e%d, not C-90\'s 1e-12' % e_gov)
        elif kind == 'pending':
            emit('BAD', 'AC 4: a "pending" verdict is allowed only when a [memory] figure is kept; the governing BER is 1e%d' % e_gov)
        elif kind == 'corrected':
            if not re.search(r'because|since|reason|due to', vtext, re.I):
                emit('BAD', 'AC 4: the corrected verdict gives no reason (because/since/reason/due to)')
            vals = all_nums(vtext)

            def has(x):
                return any(sf2(v) == sf2(x) for v in vals if v > 0)
            for name, x in [('per-packet p %.2e' % p_rec, p_rec), ('C-118 soak count %.3g' % so_rec, so_rec),
                            ('C-179 reopen count %.3g' % ro_rec, ro_rec)]:
                if has(x):
                    emit('OK', 'AC 4: the corrected verdict states %s' % name)
                else:
                    emit('BAD', 'AC 4: the corrected verdict does not state %s (2 significant figures, governing BER 1e%d)' % (name, e_gov))
        else:
            emit('BAD', 'AC 4: the verdict is %s; it must say either "C-90 confirmed" or a corrected value with its reason' % kind)

# ---------------------------------------------------------------------------
# DoD: evidence tags, [memory], open sub-question and open items
# ---------------------------------------------------------------------------

if memory_present:
    if fallback:
        emit('OK', 'DoD: [memory] kept on a BER figure or the loss floor, with the statement that the standard could not be consulted and a named checker')
    else:
        emit('BAD', 'DoD: [memory] remains on a BER figure or the loss floor (C-132) with no fallback statement: "%s"' % snip(mem_units[0]))
else:
    emit('OK', 'DoD: no [memory] tag on a BER figure or on the loss floor')

tag_gap = None
for p in paras:
    if (ber_exps(p) or MEMKW.search(p)) and re.search(r'\d', p) and not TAG_RE.search(p):
        tag_gap = p
        break
if tag_gap is not None:
    emit('BAD', 'DoD: a paragraph with a BER figure or loss-floor quantity has no evidence tag: "%s"' % snip(tag_gap))
else:
    emit('OK', 'DoD: every paragraph with a BER figure or loss-floor quantity carries an evidence tag')

sq = [p for p in paras if re.search(r'right bit-error figure', norm(p), re.I)]
# the statement must be a positive one in its own sentences; the falsifier
# sentence ("... measured at L1 ...") is not a handoff and does not count
sq_stmts = [u for p in sq for u in re.split(r'(?<=[.;])\s+', p) if not re.search(r'falsifier', u, re.I)]
if not sq:
    emit('BAD', 'DoD (open sub-question): "Is the loss floor C-90 computed from the right bit-error figure?" is not stated')
elif any(positive(u, r'\banswered\b|\bL1\b|task 019') for u in sq_stmts) and any(re.search(r'falsifier', p, re.I) for p in sq):
    emit('OK', 'DoD (open sub-question): the sub-question is answered or handed to L1 (task 019), with its falsifier')
else:
    emit('BAD', 'DoD (open sub-question): the sub-question is neither answered nor handed to L1 (task 019) with a falsifier')

for p in paras:
    if re.search(r'open (?:item|sub-question)', p, re.I):
        if not (re.search(r'owner', p, re.I) or re.search(r'\bL[123]\b|task \d{3}', p)):
            emit('BAD', 'DoD (open item): no owner named: "%s"' % snip(p))
        if not re.search(r'falsifier', p, re.I):
            emit('BAD', 'DoD (open item): no falsifier named: "%s"' % snip(p))

for line in out:
    print(line)
PY

  RESULTS=$(python3 -c "$PYCODE" "$REPORT" 2>/dev/null) \
    || RESULTS=$(printf 'BAD\tthe draft analysis failed to run (python3 error on %s)' "$REPORT_REL")
  while IFS=$'\t' read -r kind msg; do
    case "$kind" in
      OK)  ok "$msg" ;;
      BAD) bad "$msg" ;;
      *)   [ -n "$kind" ] && bad "$kind $msg" ;;
    esac
  done <<< "$RESULTS"
fi

# ---------------------------------------------------------------------------
# Constants: this ticket does not edit doc/design/02-constants.md
# ---------------------------------------------------------------------------

# T5: origin/main, not the local main ref (which can be stale or absent)
if [ "$OVERRIDDEN" -eq 1 ]; then
  echo "NOTE: REPORT_0A_BER is set, so the constants-table check (git diff origin/main) is skipped"
elif ! git -C "$REPO_ROOT" rev-parse --verify -q 'origin/main^{commit}' >/dev/null 2>&1; then
  echo "NOTE: origin/main does not exist in this clone, so the constants-table check (git diff origin/main) is skipped"
elif git -C "$REPO_ROOT" diff --quiet origin/main -- doc/design/02-constants.md 2>/dev/null; then
  ok "constants: doc/design/02-constants.md is unchanged against origin/main (001-11 does not edit it)"
else
  bad "constants: doc/design/02-constants.md differs from origin/main; 001-11 must not edit it (001-14 does)"
fi

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
