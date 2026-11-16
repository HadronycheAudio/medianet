#!/usr/bin/env bash
# checks/003-08.sh — acceptance check for ticket 003-08 "Write BASELINE note"
# (epic 003, deliverable 7: BASELINE.md in the frozen proof of concept).
#
# The deliverable lives outside this repository: BASELINE.md at the root of the
# POC at /home/dorian/workspace/hadropoc (the "POC"), written by the CODER under
# the epic-003 exception (C-130) and committed as `docs(003-08): ...`.  This
# script only READS the POC git history and the POC's files at HEAD, reads the
# measurement notes of the shared checkout (SHARED_ROOT), and reads the design
# constants.  It never builds or runs the POC (no hadropoc, no tools/smoke.sh,
# no cargo) and never writes into the POC tree.  Override the POC location with
# HADROPOC=/path (used for the throwaway fixture clones only).
#
# RULINGS applied (lead, 003-08 brief; H1-H11):
#   H1  the tag poc-baseline is created by 003-09, so it may be absent here:
#       a NOTE, not a failure.  Once it exists it must point at a commit that
#       contains BASELINE.md.  poc-untouched must exist.  The document must
#       NAME both tags exactly.
#   H2  the three fixes, one paragraph each, naming exactly the files their
#       commits touch (computed from git at run time, anchored ids):
#       fix 1 = 003-03, fix 2 = 003-04, fix 3 = 003-06.
#   H3  four sync-rate figures (50 and 1000 Hz, before and after), each tagged
#       [measured], EQUAL to the figures in swarm/notes/003-02.md (before) and
#       swarm/notes/003-05.md (after), parsed at run time.  The 003-02 figures
#       are the literal (naive) method; 003-05's are duration-corrected.  The
#       document must name the method of each.
#   H4  the lock time from 003-05 ("LOCKED at ... s") within 25 s, [measured];
#       1000/s outside C-112's accepted range 10 to 200 per second.
#   H5  in-scope families audio, music, transport; NONE known-inert; a headless
#       --duration run shorter than about 5 s exits non-zero by design.
#   H6  the zero-sample line `check family <name>: 0 windows analysed`, equal to
#       the eprintln! at HEAD:src/main.rs (read at run time).
#   H7  the smoke run: a caller-supplied --group, the --check-metrics form, and
#       the script's OWN --no-tui (a caller must not pass it: hadropoc would get
#       the flag twice).  It asserts nothing about sync rate, latency or
#       percentiles.  tools/smoke.sh at HEAD is read at run time and must still
#       pass --no-tui to its own hadropoc command and append "$@".
#   H8  POC loss figures are never citable from the baseline.
#   H9  the POC wire is not the runtime wire: a baseline is not a wire reference.
#   H10 every factual paragraph carries [code], [measured] or [decision] and
#       only those three tags are used; no sync-rate figure, latency or
#       percentile is stated as a smoke assertion; "Open item" paragraphs name
#       an owner and a falsifier.
#   H11 the 003-08 commits (anchored id) add ONLY BASELINE.md at the POC root;
#       every commit after poc-untouched carries an anchored 003-XX id; the POC
#       tree is clean (loud SKIP of this one assertion while the orchestrator
#       marker swarm/poc-write-in-flight exists in the shared checkout).
#
# INTERFACE the coder implements (form, not wording, is machine-checked):
#   * BASELINE.md at the POC root, committed in a commit whose subject is
#     `docs(003-08): ...` (or `003-08: ...`), and that commit touches nothing else.
#   * Headings (any level; matched case-insensitively, first match wins):
#       tags | fix | sync | lock | famil | smoke | zero | wire | loss
#   * Paragraphs: a block of lines between blank lines (a table data row, a
#     bullet, a heading-free block).  Every prose block of 5 or more words, and
#     every block containing a digit, must carry one of [code] [measured]
#     [decision].  Headings, table header rows and fenced code are not claims.
#   * Figures in the sync section: every percentage and every row length
#     (number followed by s or ms) must appear in swarm/notes/003-05.md or in
#     the 002-12 sub-section of REPORT-0b (parsed at run time).
#   * Fix paragraphs: a block whose first words are "Fix 1", "Fix 2" or "Fix 3"
#     (optional bold/bullet marks).  Each names its ticket id 003-NN and the
#     src/ and tools/ paths its commits touch, and no other such path.
#   * Sync table: one markdown table in the sync section with rows whose first
#     cell carries 50 or 1000; cell 2 is the BEFORE figure (first number in the
#     cell), cell 3 the AFTER figure, each cell containing [measured].
#   * Smoke run: a fenced line beginning `tools/smoke.sh --group 239.72.68.44:<port>`
#     (or a placeholder); a fenced or inline `tools/smoke.sh --check-metrics <file>`;
#     no fenced line passes --no-tui to tools/smoke.sh (the script passes it
#     itself, verified against tools/smoke.sh at HEAD).
#   * Sync table: the header names a 003-02 column (before), a 003-05 column
#     (after) and a P-02 column (like-for-like before, clean, duration-corrected).
#     Cells are bound by header text, each figure `[measured]`.
#   * Zero-sample line: the literal `check family <name>: 0 windows analysed`.
#   * Opened items: a block starting "Open item" also contains "Owner:" and
#     "Falsifier:".
#
# ASSERTIONS THIS CHECK MAKES
#   AC0 history: poc-untouched is an ancestor of HEAD; every commit after it has
#       an anchored 003-XX id; at least one commit belongs to 003-08.
#   AC1 scope: the 003-08 commits add/modify exactly one path, BASELINE.md, at
#       the POC root.
#   AC2 BASELINE.md exists at the root of HEAD (read from git, not the work
#       tree).
#   AC3 the three fix commit sets exist (003-03, 003-04, 003-06); their file
#       lists are computed from `git show --name-only` (what `git show --stat`
#       names, without the counts).
#   AC4 tags: poc-untouched exists; poc-baseline is a NOTE when absent, and
#       when present points at a commit that contains BASELINE.md.
#   AC5 the document (python section below): sections, tags, fixes, figures
#       equal to the notes, machine identity, method, lock, range, families,
#       smoke, zero-sample line, wire, loss, open items.
#   TREE the POC working tree is clean (loud SKIP while the marker exists).

set -u
export LC_ALL=C

SHARED_ROOT="${SHARED_ROOT:-/home/dorian/workspace/hadronyche/medianet}"
MARKER_PATH="${MARKER_PATH:-$SHARED_ROOT/swarm/poc-write-in-flight}"
NOTE02="$SHARED_ROOT/swarm/notes/003-02.md"
NOTE05="$SHARED_ROOT/swarm/notes/003-05.md"
REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
CONSTANTS="$REPO_ROOT/doc/design/02-constants.md"
POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
UNTOUCHED="poc-untouched"
BASELINE_TAG="poc-baseline"
DOC="BASELINE.md"
OUT=/tmp/opencode/003-08
mkdir -p "$OUT" || { echo "FAIL  cannot create $OUT"; exit 1; }

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

gitpoc() { git -C "$POC" "$@"; }
# Anchored ticket-id rule (same as 003-01..003-07): a first line carries a
# ticket id only as `003-NN:` / `003-NN ` or the scope `type(003-NN): `.
ID_ANY='^[0-9a-f]+ (003-[0-9][0-9]([: ]|$)|[a-z]+\(003-[0-9][0-9]\)!?: )'

echo "== 003-08: BASELINE note (BASELINE.md in the POC root) =="
echo "   POC:   $POC"
echo "   notes: $SHARED_ROOT/swarm/notes/"
echo "   out:   $OUT"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ] || ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository"
  echo "== 003-08 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --verify -q "$UNTOUCHED^{commit}" >/dev/null 2>&1; then
  bad "precondition: tag $UNTOUCHED does not resolve in the POC"
  echo "== 003-08 FAILED =="
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  bad "precondition: python3 not found in PATH (needed for the document checks)"
  echo "== 003-08 FAILED =="
  exit 1
fi

# Commits of a ticket: first line references it (anchored rule), oldest first.
ticket_commits() { # $1 = ticket id, e.g. 003-08
  gitpoc log --reverse --format='%H%x09%s' "$UNTOUCHED..HEAD" 2>/dev/null \
    | awk -F'\t' -v id="$1" '$2 ~ ("^" id "([: ]|$)|^[a-z]+[(]" id "[)]!?: ") { print $1 }'
}

# -------------------------------------------------------------- AC 0
echo "-- AC0 history: poc-untouched ancestor, anchored ids, a 003-08 commit"
if gitpoc merge-base --is-ancestor "$UNTOUCHED" HEAD 2>/dev/null; then
  pass "AC0 $UNTOUCHED is an ancestor of HEAD"
else
  bad "AC0 $UNTOUCHED is not an ancestor of HEAD"
fi
bad_ids="$(gitpoc log --format='%h %s' "$UNTOUCHED..HEAD" 2>/dev/null | grep -Ev "$ID_ANY" || true)"
if [ -n "$bad_ids" ]; then
  bad "AC0 commits after $UNTOUCHED without a 003-XX id in the first line:"
  printf '%s\n' "$bad_ids" | head -n 5 | while IFS= read -r l; do note "$l"; done
else
  pass "AC0 every commit after $UNTOUCHED references a 003-XX ticket"
fi
T08_COMMITS="$(ticket_commits 003-08)"
if [ -z "$T08_COMMITS" ]; then
  bad "AC0 no commit after $UNTOUCHED references 003-08 (BASELINE.md is not committed in the POC)"
else
  pass "AC0 $(printf '%s\n' "$T08_COMMITS" | wc -l | tr -d ' ') commit(s) reference 003-08: $(printf '%s\n' "$T08_COMMITS" | cut -c1-12 | tr '\n' ' ')"
fi

# -------------------------------------------------------------- AC 1
echo "-- AC1 scope: the 003-08 commits touch exactly BASELINE.md at the POC root"
if [ -n "$T08_COMMITS" ]; then
  : > "$OUT/t08-status"
  for c in $T08_COMMITS; do
    gitpoc show --name-status --format= "$c" 2>/dev/null >> "$OUT/t08-status"
  done
  awk -F'\t' '{ print $2 }' "$OUT/t08-status" 2>/dev/null | sort -u > "$OUT/t08-paths"
  ptxt="$(tr '\n' ' ' < "$OUT/t08-paths" | sed 's/ *$//')"
  if [ "$ptxt" = "$DOC" ]; then
    pass "AC1 the 003-08 commits touch exactly $DOC"
  else
    bad "AC1 the 003-08 commits touch '${ptxt:-nothing}' (allowed: exactly $DOC, nothing else)"
  fi
  if grep -Ev '^[AM]	BASELINE\.md$' "$OUT/t08-status" >/dev/null 2>&1; then
    bad "AC1 a 003-08 commit deletes, renames or otherwise changes a path other than an add or modify of $DOC:"
    grep -Ev '^[AM]	BASELINE\.md$' "$OUT/t08-status" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
else
  bad "AC1 not evaluated: no 003-08 commit"
fi

# -------------------------------------------------------------- AC 2
echo "-- AC2 $DOC exists at the root of HEAD"
DOC_OK=0
if gitpoc cat-file -e "HEAD:$DOC" 2>/dev/null; then
  gitpoc show "HEAD:$DOC" > "$OUT/$DOC" 2>/dev/null && DOC_OK=1
fi
if [ "$DOC_OK" -eq 1 ]; then
  pass "AC2 $DOC is at the POC root of HEAD ($(wc -l < "$OUT/$DOC" | tr -d ' ') lines)"
else
  bad "AC2 $DOC is not at the POC root of HEAD (no document to check)"
fi

# -------------------------------------------------------------- AC 3
echo "-- AC3 the three fix commit sets and the files they touch (git, at run time)"
FIX_FILES_03=""; FIX_FILES_04=""; FIX_FILES_06=""
for t in 003-03 003-04 003-06; do
  fc="$(ticket_commits "$t")"
  if [ -z "$fc" ]; then
    bad "AC3 no commit after $UNTOUCHED references $t (the fix is not committed in the POC)"
    continue
  fi
  files=""
  for c in $fc; do
    files="$files $(gitpoc show --name-only --format= "$c" 2>/dev/null | tr '\n' ' ')"
  done
  files="$(printf '%s\n' $files | sed '/^$/d' | sort -u | tr '\n' ' ' | sed 's/ *$//')"
  pass "AC3 $t commits: $(printf '%s\n' "$fc" | wc -l | tr -d ' ') commit(s); files: ${files:-none}"
  case "$t" in
    003-03) FIX_FILES_03="$files" ;;
    003-04) FIX_FILES_04="$files" ;;
    003-06) FIX_FILES_06="$files" ;;
  esac
done

# -------------------------------------------------------------- AC 4
echo "-- AC4 tags: $UNTOUCHED exists; $BASELINE_TAG absent (NOTE) or contains $DOC"
if gitpoc tag --list | grep -Fxq "$UNTOUCHED"; then
  pass "AC4 git tag lists $UNTOUCHED"
else
  bad "AC4 git tag does not list $UNTOUCHED"
fi
if gitpoc tag --list | grep -Fxq "$BASELINE_TAG"; then
  if gitpoc cat-file -e "$BASELINE_TAG:$DOC" 2>/dev/null; then
    pass "AC4 $BASELINE_TAG exists and its commit contains $DOC"
  else
    bad "AC4 $BASELINE_TAG exists but its commit does not contain $DOC"
  fi
else
  printf 'NOTE  AC4 %s is not created yet (ticket 003-09 creates it on the commit that carries %s)\n' "$BASELINE_TAG" "$DOC"
fi

# -------------------------------------------------------------- DOCUMENT
echo "-- AC5 document content (sections, tags, fixes, figures, machine, method, lock, families, smoke, zero-sample, wire, loss)"
GIT_MAIN_RS="$OUT/main.rs"
if ! gitpoc show "HEAD:src/main.rs" > "$GIT_MAIN_RS" 2>/dev/null; then
  : > "$GIT_MAIN_RS"
  bad "AC5 HEAD:src/main.rs cannot be read (zero-sample wording cannot be verified)"
fi
if [ "$DOC_OK" -ne 1 ]; then
  bad "AC5 document checks not evaluated: $DOC is absent at HEAD"
else
  cat > "$OUT/doc-check.py" <<'PY'
import re, sys

doc_path, n02_path, n05_path, const_path, main_path, f03, f04, f06, report_path, script_path = sys.argv[1:11]
results = []


def ok(msg):
    results.append("PASS: " + msg)


def bad(msg):
    results.append("FAIL: " + msg)


def read(path, label):
    try:
        with open(path, encoding="utf-8") as h:
            return h.read()
    except OSError:
        bad("%s: cannot read %s" % (label, path))
        return ""


doc = read(doc_path, "document")
n02 = read(n02_path, "note 003-02")
n05 = read(n05_path, "note 003-05")
const = read(const_path, "constants")
main_rs = read(main_path, "HEAD:src/main.rs")
doc_lines = doc.split("\n")

TAG_RE = r"\[(code|measured|decision)\]"


def first(pattern, text, flags=0):
    m = re.search(pattern, text, flags)
    return m.group(1) if m else None


# ---------------------------------------------------------------- parse notes
# 003-02 (before): literal method, reported figures per rate.
rates02 = {}
for rate, val in re.findall(r"`--sync-hz (\d+)`:\s*\*\*([\d.]+) exchanges/s\*\*", n02):
    rates02.setdefault(int(rate), []).append(float(val))
for rate in (50, 1000):
    if len(rates02.get(rate, [])) != 1:
        bad("note 003-02: expected exactly one reported figure for --sync-hz %d, found %d" % (rate, len(rates02.get(rate, []))))
before_commit = first(r"commit `([0-9a-f]{40})`", n02)
duration02 = first(r"--duration (\d+)", n02)
host = first(r"- Host name: (.+)", n02)
cpu = first(r"- CPU: (.+)", n02)
kernel = first(r"- Kernel: (.+)", n02)
cpu = cpu.split(" (")[0].strip() if cpu else None
kernel = kernel.split(" (")[0].strip() if kernel else None
hz = first(r"CONFIG_HZ=(\d+) \(P-03\)", n02)

# 003-05 (after): duration-corrected figures per rate, at 4a4885e.
after05 = {}
sections05 = re.split(r"(?m)^### ", n05)
for sec in sections05:
    m = re.match(r"`--sync-hz (\d+)`", sec)
    if m and int(m.group(1)) not in after05:
        fig = re.search(r"\*\*([\d.]+) exchanges/s\*\*", sec)
        if fig:
            after05[int(m.group(1))] = float(fig.group(1))
for rate in (50, 1000):
    if rate not in after05:
        bad("note 003-05: no duration-corrected figure under the --sync-hz %d heading" % rate)
after_commit = first(r"HEAD `([0-9a-f]{40})`", n05)
lock_values = set(re.findall(r"LOCKED at \*\*([\d.]+) s\*\*", n05))
chimera = re.search(r"\((\d+) at the slow rate, (\d+) at the fast one\)", n05)
p02_text = first(r"(?m)^\| P-02 \|(.*)$", const) or ""
p02_pair = re.search(r"(\d+\.\d+) / (\d+\.\d+) completed", p02_text)
p02_runs = re.search(r"3 runs of (\d+) s", p02_text)

# Notes must yield every figure, or nothing below can be checked.
for label, val in (("003-02 commit", before_commit), ("003-02 duration", duration02),
                   ("003-02 host", host), ("003-02 CPU", cpu), ("003-02 kernel", kernel),
                   ("CONFIG_HZ", hz), ("003-05 commit", after_commit)):
    if not val:
        bad("note parse: %s not found in the notes" % label)
if len(lock_values) != 1:
    bad("note 003-05: the lock time 'LOCKED at **<s> s**' is not a single value (found %s)" % sorted(lock_values))
if not chimera:
    bad("note 003-05: the chimera bound '(<n> at the slow rate, <m> at the fast one)' not found")
if not p02_pair or not p02_runs:
    bad("constants: P-02 values (36.1 / 71.4) or run length (3 runs of 35 s) not found")
lock_s = list(lock_values)[0] if len(lock_values) == 1 else None

rep_text = read(report_path, "REPORT-0b")
m212 = re.search(r"^### [^\n]*002-12[^\n]*\n(.*?)(?=^### |\Z)", rep_text, re.M | re.S)
report_212 = m212.group(1) if m212 else ""
if not report_212:
    bad("sources: REPORT-0b's 002-12 sub-section is not found")
smoke_script = read(script_path, "tools/smoke.sh at HEAD")
src_text = n05 + "\n" + report_212
src_pct = {float(x) for x in re.findall(r"(\d+(?:\.\d+)?)\s*%", src_text)}
src_sec = set()
for num, unit in re.findall(r"(\d+(?:\.\d+)?)\s*(ms|s)\b", src_text):
    src_sec.add(round(float(num) / 1000 if unit == "ms" else float(num), 9))
runs02 = re.findall(r"\((\d+\.\d+) in run 1, (\d+\.\d+) in run 2\)", n02)

# ---------------------------------------------------------------- sections
sections = []
cur_head, cur_body = "(preamble)", []
for line in doc_lines:
    if re.match(r"^#{1,6} ", line):
        sections.append((cur_head, cur_body))
        cur_head, cur_body = line, []
    else:
        cur_body.append(line)
sections.append((cur_head, cur_body))


def section(pattern):
    for head, body in sections:
        if re.search(pattern, head, re.I) and head != "(preamble)":
            return "\n".join(body)
    return None


S_TAGS = section(r"tag")
S_FIX = section(r"fix")
S_SYNC = section(r"sync")
S_LOCK = section(r"lock")
S_FAM = section(r"famil")
S_SMOKE = section(r"smoke")
S_ZERO = section(r"zero")
S_WIRE = section(r"wire")
S_LOSS = section(r"loss")
for name, s in (("tags", S_TAGS), ("fixes", S_FIX), ("sync rate", S_SYNC), ("lock", S_LOCK),
                ("check families", S_FAM), ("smoke script", S_SMOKE), ("zero-sample line", S_ZERO),
                ("wire", S_WIRE), ("loss", S_LOSS)):
    if s is None:
        bad("section: no heading for '%s' (a heading matching the keyword is required)" % name)
    else:
        ok("section: '%s' present" % name)
S_TAGS = S_TAGS or ""
S_FIX = S_FIX or ""
S_SYNC = S_SYNC or ""
S_LOCK = S_LOCK or ""
S_FAM = S_FAM or ""
S_SMOKE = S_SMOKE or ""
S_ZERO = S_ZERO or ""
S_WIRE = S_WIRE or ""
S_LOSS = S_LOSS or ""

# ---------------------------------------------------------------- blocks
# Fenced code is excluded from prose; tables are split row by row; a table
# header (the row above a separator) and separator rows make no claim.
fence = False
clean = []  # (index, text) outside fenced code
for i, line in enumerate(doc_lines):
    if line.lstrip().startswith("```"):
        fence = not fence
        continue
    if not fence:
        clean.append((i, line))
header_rows = set()
for i, line in enumerate(doc_lines):
    if line.strip().startswith("|") and i + 1 < len(doc_lines) and re.match(r"^\s*\|?\s*:?-{3,}", doc_lines[i + 1]):
        header_rows.add(i)
blocks = []  # list of (first line index, text)
cur = []
cur_start = None
for i, line in clean:
    s = line.strip()
    if s == "" or re.match(r"^#{1,6} ", line):
        if cur:
            blocks.append((cur_start, "\n".join(cur)))
        cur, cur_start = [], None
        continue
    if s.startswith("|"):
        if cur:
            blocks.append((cur_start, "\n".join(cur)))
        cur, cur_start = [], None
        if i not in header_rows and not re.match(r"^\s*\|?\s*:?-{3,}", line):
            blocks.append((i, line))
        continue
    if re.match(r"^\s*([-*]|\d+\.)\s", line) and cur:
        blocks.append((cur_start, "\n".join(cur)))
        cur = []
    if not cur:
        cur_start = i
    cur.append(line)
if cur:
    blocks.append((cur_start, "\n".join(cur)))

# ---------------------------------------------------------------- AC 5a tags
def words(t):
    return len(re.findall(r"[A-Za-z0-9]+", t))
bad_tag = [(i + 1, t[:60]) for i, t in blocks
           if (re.search(r"\d", t) or words(t) >= 5) and not re.search(TAG_RE, t)]
if bad_tag:
    for ln, snippet in bad_tag[:5]:
        bad("tags: line %d carries a figure or identifier without [code], [measured] or [decision]: %s" % (ln, snippet))
else:
    ok("tags: every paragraph with a figure carries an evidence tag")
used = set(re.findall(r"\[([a-z][a-z-]*)\]", doc))
forbidden = sorted(used - {"code", "measured", "decision"})
if forbidden:
    bad("tags: only [code], [measured] and [decision] may be used; found: %s" % ", ".join("[%s]" % t for t in forbidden))
else:
    ok("tags: only [code], [measured] and [decision] are used")

# Open items: owner and falsifier.
for i, t in blocks:
    if t.lstrip().lstrip("*_-> ").startswith("Open item"):
        if "Owner:" in t and "Falsifier:" in t:
            ok("open item (line %d) names an owner and a falsifier" % (i + 1))
        else:
            bad("open item (line %d) lacks 'Owner:' or 'Falsifier:'" % (i + 1))

# ---------------------------------------------------------------- tags section
for name in ("poc-untouched", "poc-baseline"):
    if re.search(r"(?<![\w-])`?%s`?(?![\w-])" % re.escape(name), S_TAGS):
        ok("tags: names %s" % name)
    else:
        bad("tags: the Tags section does not name the tag %s" % name)
blk_untouched = [t for _, t in blocks if "poc-untouched" in t]
if any("first commit" in t and ".gitignore" in t for t in blk_untouched):
    ok("tags: poc-untouched is the first commit with .gitignore")
else:
    bad("tags: say what poc-untouched contains (the first commit: the untouched tree plus .gitignore)")
blk_baseline = [t for _, t in blocks if "poc-baseline" in t]
if any(re.search(r"three fixes", t) and re.search(r"smoke", t, re.I) and "BASELINE.md" in t for t in blk_baseline):
    ok("tags: poc-baseline is the commit with the three fixes, the smoke script and BASELINE.md")
else:
    bad("tags: say what poc-baseline contains (the commit with the three fixes, the smoke script and this note)")
if "003-09" in S_TAGS:
    ok("tags: names ticket 003-09 as the creator of poc-baseline")
else:
    bad("tags: name ticket 003-09 as the ticket that creates poc-baseline")

# ---------------------------------------------------------------- fixes
fix_blocks = {1: [], 2: [], 3: []}
for i, t in blocks:
    m = re.match(r"^\s*[>*_-]*\s*\**\s*Fix ([123])\b", t)
    if m:
        fix_blocks[int(m.group(1))].append(t)
if any(len(v) != 1 for v in fix_blocks.values()):
    bad("fixes: exactly one paragraph per fix ('Fix 1', 'Fix 2', 'Fix 3'); found %s"
        % {k: len(v) for k, v in fix_blocks.items()})
expected = {1: ("003-03", set(f03.split())), 2: ("003-04", set(f04.split())), 3: ("003-06", set(f06.split()))}
for n in (1, 2, 3):
    tid, want = expected[n]
    if len(fix_blocks[n]) != 1:
        continue
    text = fix_blocks[n][0]
    if tid not in text:
        bad("fixes: the Fix %d paragraph does not name its ticket %s" % (n, tid))
    if not want:
        bad("fixes: git reports no files for %s (cannot check the Fix %d paragraph)" % (tid, n))
        continue
    named = set(re.findall(r"(?:src|tools)/[A-Za-z0-9_./-]+\.(?:rs|sh|md|toml)", text))
    if named == want:
        ok("fixes: Fix %d (%s) names exactly its files: %s" % (n, tid, " ".join(sorted(want))))
    else:
        bad("fixes: Fix %d (%s) names %s, git says %s (missing: %s; extra: %s)"
            % (n, tid, " ".join(sorted(named)) or "none", " ".join(sorted(want)),
               " ".join(sorted(want - named)) or "none", " ".join(sorted(named - want)) or "none"))

# ---------------------------------------------------------------- sync table
p02_value = {50: float(p02_pair.group(1)), 1000: float(p02_pair.group(2))} if p02_pair else {}
t_lines = S_SYNC.split("\n")
hdr_idx = None
for i, l in enumerate(t_lines):
    if l.strip().startswith("|") and i + 1 < len(t_lines) and re.match(r"^\s*\|?\s*:?-{3,}", t_lines[i + 1]):
        hdr_idx = i
        break
hdr_cells = [c.strip() for c in t_lines[hdr_idx].strip().strip("|").split("|")] if hdr_idx is not None else []
col_b = next((k for k, c in enumerate(hdr_cells) if "003-02" in c), None)
col_a = next((k for k, c in enumerate(hdr_cells) if "003-05" in c), None)
col_p = next((k for k, c in enumerate(hdr_cells) if "P-02" in c), None)
if col_b is None or col_a is None or col_p is None:
    bad("sync: the table header must name a 003-02 column (before), a 003-05 column (after) and a P-02 column")
elif not re.search(r"clean|duration-corrected", hdr_cells[col_p], re.I):
    bad("sync: the P-02 column must be labelled clean or duration-corrected (like-for-like before)")
else:
    ok("sync: the table has a 003-02 column, a 003-05 column and a P-02 column")
rows = {}
for line in t_lines:
    if not line.strip().startswith("|"):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) < 3:
        continue
    m = re.search(r"\b(50|1000)\b", cells[0])
    if m and ("--sync-hz" in cells[0] or re.fullmatch(r"`?\s*(50|1000)\s*`?", cells[0])):
        rate = int(m.group(1))
        rows.setdefault(rate, []).append(cells)
for rate in (50, 1000):
    if len(rows.get(rate, [])) != 1:
        bad("sync: the table needs exactly one row for --sync-hz %d (found %d)" % (rate, len(rows.get(rate, []))))
for rate in (50, 1000):
    if len(rows.get(rate, [])) != 1 or col_b is None or col_a is None or col_p is None:
        continue
    cells = rows[rate][0]
    if max(col_b, col_a, col_p) >= len(cells):
        bad("sync: --sync-hz %d row lacks the 003-02, 003-05 or P-02 cell" % rate)
        continue
    cb, ca, cp = cells[col_b], cells[col_a], cells[col_p]
    b_num = first(r"(\d+(?:\.\d+)?)", cb)
    a_num = first(r"(\d+(?:\.\d+)?)", ca)
    p_num = first(r"(\d+(?:\.\d+)?)", cp)
    if b_num is None or a_num is None or p_num is None:
        bad("sync: --sync-hz %d row has no before, after or P-02 figure" % rate)
        continue
    b_want = rates02.get(rate, [None])[0]
    a_want = after05.get(rate)
    if b_want is not None and abs(float(b_num) - b_want) < 1e-9:
        ok("sync: 003-02 before at --sync-hz %d is %s, equal to the 003-02 note" % (rate, b_num))
    else:
        bad("sync: 003-02 before at --sync-hz %d is %s, the 003-02 note reports %s" % (rate, b_num, b_want))
    if a_want is not None and abs(float(a_num) - a_want) < 1e-9:
        ok("sync: 003-05 after at --sync-hz %d is %s, equal to the 003-05 note" % (rate, a_num))
    else:
        bad("sync: 003-05 after at --sync-hz %d is %s, the 003-05 note reports %s" % (rate, a_num, a_want))
    p_want = p02_value.get(rate)
    if p_want is not None and abs(float(p_num) - p_want) < 1e-9:
        ok("sync: P-02 like-for-like before at --sync-hz %d is %s, equal to constants P-02" % (rate, p_num))
    else:
        bad("sync: P-02 like-for-like before at --sync-hz %d is %s, constants P-02 says %s" % (rate, p_num, p_want))
    if all("[measured]" in c for c in (cb, ca, cp)):
        ok("sync: the --sync-hz %d before, after and P-02 figures are each tagged [measured]" % rate)
    else:
        bad("sync: the --sync-hz %d before, after and P-02 figures must each carry [measured]" % rate)
if any(re.search(r"not comparable|not like for like", t, re.I) and "38" in t and "75" in t for _, t in blocks):
    ok("sync: 38 / 75 (003-02, literal) are stated not comparable with the after column")
else:
    bad("sync: state in one sentence that 003-02's 38 / 75 are literal figures and not comparable (not like for like) with the after column")

# Method, provenance and machine identity (parsed values).
def need(label, cond, msg):
    if cond:
        ok(label + ": " + msg)
    else:
        bad(label + ": " + msg)


need("sync", re.search(r"duration-corrected", S_SYNC, re.I) is not None,
     "the after figures are stated as duration-corrected")
need("sync", re.search(r"literal|naive", S_SYNC, re.I) is not None,
     "the before figures are stated as the literal (naive) method")
need("sync", any(re.search(r"literal|naive", t, re.I) and re.search(r"\bhigh\b", t, re.I)
                 and re.search(r"(more|longer) than (1 s|one second)", t, re.I) for _, t in blocks),
     "the literal method is stated to read high, because a metrics row covers more than 1 s")
bad_src = []
for num in re.findall(r"(\d+(?:\.\d+)?)\s*%", S_SYNC):
    if float(num) not in src_pct:
        bad_src.append(num + " %")
for num, unit in re.findall(r"(\d+(?:\.\d+)?)\s*(ms|s)\b", S_SYNC):
    if round(float(num) / 1000 if unit == "ms" else float(num), 9) not in src_sec:
        bad_src.append(num + " " + unit)
if bad_src:
    bad("sync: unsourced figure(s) %s: every percentage and row length must appear in swarm/notes/003-05.md or REPORT-0b's 002-12 sub-section"
        % ", ".join(sorted(set(bad_src))))
else:
    ok("sync: every percentage and duration in the sync section is sourced (003-05 note or REPORT-0b 002-12)")
need("sync", re.search(r"covered span", S_SYNC, re.I) is not None and re.search(r"per follower", S_SYNC, re.I) is not None,
     "the corrected method is stated (sum served / covered span / followers, per follower)")
if chimera:
    need("sync", re.search(r"followers\s*[×x*]\s*rate\s*[×x*]\s*1\.10", S_SYNC) is not None
         and str(chimera.group(1)) in doc and str(chimera.group(2)) in doc,
         "the chimera bound is followers x rate x 1.10 (%s at the slow rate, %s at the fast one)" % (chimera.group(1), chimera.group(2)))
need("sync", re.search(r"chimera", S_SYNC, re.I) is not None, "the chimera guard is named")
if host and cpu and kernel and hz:
    need("sync", host in S_SYNC and cpu in S_SYNC and kernel in S_SYNC and ("CONFIG_HZ=%s" % hz) in S_SYNC and "P-03" in S_SYNC,
         "machine identity: host %s, CPU %s, kernel %s, CONFIG_HZ=%s (P-03)" % (host, cpu, kernel, hz))
need("sync", re.search(r"unprivileged", S_SYNC, re.I) is not None, "the runs are stated unprivileged")
need("sync", re.search(r"not (in )?strict mode|without strict mode", S_SYNC, re.I) is not None,
     "the runs are stated not in strict mode")
need("sync", "C-127" in doc and "C-147" in doc, "C-127 and C-147 are cited")
if before_commit:
    need("sync", before_commit[:7] in S_SYNC and "003-02" in S_SYNC and "poc-untouched" in S_SYNC,
         "the before figures are from 003-02 at poc-untouched (%s)" % before_commit[:7])
if after_commit:
    need("sync", after_commit[:7] in S_SYNC and "003-05" in S_SYNC,
         "the after figures are from 003-05 at %s" % after_commit[:7])
if duration02 and p02_runs and p02_pair:
    need("sync", ("%s s" % duration02) in S_SYNC and ("%s s" % p02_runs.group(1)) in S_SYNC and "P-02" in S_SYNC
         and p02_pair.group(1) in S_SYNC and p02_pair.group(2) in S_SYNC,
         "P-02 (%s / %s, %s s runs) is stated beside the 003-02 figures (%s s runs)"
         % (p02_pair.group(1), p02_pair.group(2), p02_runs.group(1), duration02))
    need("sync", any(re.search(r"not the same runs?", t, re.I) for _, t in blocks),
         "both pre-fix figure sets are named with their known runs and method, and state that they are not the same runs")
if len(runs02) == 2:
    ok("sync: swarm/notes/003-02.md records two runs per rate")
    if re.search(r"single[- ]runs?", doc, re.I):
        bad("sync: the document says 'single run(s)' but swarm/notes/003-02.md records two runs per rate (38.4 / 37.5; 75.7 / 74.7)")
    else:
        ok("sync: no 'single run' claim for 003-02")
else:
    bad("sync: swarm/notes/003-02.md does not show the two per-run figures per rate (the run count cannot be checked)")
need("sync", any("003-02" in t and re.search(r"\b(two|2) runs\b", t, re.I) for _, t in blocks),
     "003-02's figures are stated as two runs per rate")

for lit, what in (('windows exact, [0-9]+ lost', 'the windows-field lost count'),
                  ('"name"', 'the header name column (field 4)'),
                  ('"lock"', 'the header lock column (field 8)'),
                  ('synth1', 'the synth1 name'), ('synth2', 'the synth2 name')):
    if lit in smoke_script:
        ok("SCRIPT tools/smoke.sh at HEAD asserts %s" % what)
    else:
        bad("SCRIPT tools/smoke.sh at HEAD no longer contains %s (%s): the smoke section must be revised" % (lit, what))

# ---- hand-off statements (C4, C6) and the duplicate --no-tui rule
def block_has(*pats):
    return any(all(re.search(p, t, re.I) for p in pats) for _, t in blocks)

need("claims", block_has(r"slot-0|first-listed", r"\blost\b", r"\bonly\b"),
     "lost covers only the first-listed (slot-0) synth's return path")
need("claims", block_has(r"repeat", r"FAIL loss"), "the per-change suite may repeat a clean run once after FAIL loss")
need("claims", block_has(r"repeat", r"003-09|three runs", r"never|no repeat"),
     "the script itself and 003-09's three runs never repeat")
need("claims", block_has(r"poc-baseline", r"\bcite"), "citations and vectors come only from the poc-baseline tag")
need("claims", block_has(r"--group", r"cross-?wire", r"own"),
     "a citable run uses its own --group, because two instances on one group cross-wire")
need("claims", block_has(r"4a4885e", r"poc-baseline"),
     "the after figures, measured at 4a4885e, hold for poc-baseline because later commits do not touch the clock path")
for _, t in blocks:
    if "--no-tui" in t and re.search(r"repeat|twice|multiple times|duplicat", t, re.I) and re.search(r"fail|reject|error|exit", t, re.I):
        if "[measured]" in t:
            ok("claims: the repeated --no-tui failure is tagged [measured]")
        else:
            bad("claims: a sentence says a repeated --no-tui fails; it must carry [measured] (run it, quote hadropoc's error)")

# ---------------------------------------------------------------- lock and range
if lock_s is not None:
    lock_ok = any(lock_s + " s" in t and "25 s" in t and "[measured]" in t and re.search(r"(within|inside|in) (the )?25 s", t)
                  for _, t in blocks)
    need("lock", lock_ok, "the clock locks within 25 s after the fix (%s s, [measured], from 003-05)" % lock_s)
range_ok = any("1000" in t and "outside" in t and "C-112" in t and re.search(r"10 to 200|10-200|10–200", t) for _, t in blocks)
need("lock", range_ok, "1000 per second is stated outside the accepted C-112 range, 10 to 200 per second")

# ---------------------------------------------------------------- families
for fam in ("audio", "music", "transport"):
    need("families", re.search(r"\b%s\b" % fam, S_FAM, re.I) is not None, "the family %s is named" % fam)
need("families", re.search(r"no (check )?famil(y|ies)[^.\n]{0,60}known-inert|none[^.\n]{0,60}known-inert|known-inert[^.\n]{0,60}\bnone\b", S_FAM, re.I) is not None,
     "no family is known-inert (stated explicitly)")
for line in doc_lines:
    if "known-inert" in line and re.search(r"music|transport|audio", line, re.I) and not re.search(r"\b(no|none|not)\b", line, re.I):
        bad("families: a line claims a family is known-inert: %s" % line.strip()[:80])
need("families", re.search(r"(?<![0-9])5 s", S_FAM) is not None and re.search(r"non-?zero", S_FAM, re.I) is not None
     and re.search(r"by design", S_FAM, re.I) is not None,
     "a headless --duration run shorter than about 5 s exits non-zero by design")

# ---------------------------------------------------------------- smoke
fenced_cmds = []
fence = False
for line in doc_lines:
    if line.lstrip().startswith("```"):
        fence = not fence
        continue
    if fence:
        fenced_cmds.append(line.strip())
if any(re.match(r"^(\S+=\S+\s+)*tools/smoke\.sh\s+--group\s+239\.72\.68\.44:(\d{1,5}|<port>|PORT)(\s|$)", c) for c in fenced_cmds):
    ok("smoke: a fenced run command passes a caller-supplied group: tools/smoke.sh --group 239.72.68.44:<port>")
else:
    bad("smoke: no fenced run command of the form `tools/smoke.sh --group 239.72.68.44:<port>` (the caller's isolation group, passed as an argument)")
if re.search(r"tools/smoke\.sh\s+--check-metrics\s+\S+", S_SMOKE):
    ok("smoke: the metrics-only form `tools/smoke.sh --check-metrics <file>` is written out")
else:
    bad("smoke: write out the metrics-only form `tools/smoke.sh --check-metrics <file>`")
if re.search(r"(script|smoke)[^.\n]{0,80}(passes|supplies)[^.\n]{0,20}--no-tui", S_SMOKE, re.I):
    ok("smoke: the prose says the script itself passes --no-tui to hadropoc")
else:
    bad("smoke: say in prose that the script itself passes --no-tui to hadropoc (not a caller's command)")
bad_fenced = []
fence = False
for i, line in enumerate(doc_lines):
    if line.lstrip().startswith("```"):
        fence = not fence
        continue
    if fence and re.search(r"smoke\.sh\b.*--no-tui", line):
        bad_fenced.append(i + 1)
if bad_fenced:
    bad("smoke: a command passes --no-tui to tools/smoke.sh (lines %s); the script passes it itself, so hadropoc gets the flag twice and the run fails"
        % bad_fenced[:5])
else:
    ok("smoke: no command passes --no-tui to tools/smoke.sh")
for lit, what in (("HADROPOC_BIN", "the binary override HADROPOC_BIN"),
                  ("--check-metrics", "the metrics-only mode --check-metrics"),
                  ("--group", "the caller-supplied --group argument"),
                  ("exit status 0", "exit status 0"),
                  ("sent S lost 0", "the 'sent S lost 0' assertion"),
                  ("voices on", "the voices-on synth sections (either synth)"),
                  ("sender gaps", "the sender gaps note")):
    need("smoke", lit in S_SMOKE, "names %s" % what)
need("smoke", re.search(r"98\D{1,8}102", S_SMOKE) is not None, "the row range 98 to 102 is stated")
need("smoke", re.search(r"23\D{1,8}27", S_SMOKE) is not None, "the per-module range 23 to 27 is stated")
need("smoke", re.search(r"four modules|4 modules", S_SMOKE, re.I) is not None, "four modules are stated")
need("smoke", re.search(r"lock.{0,60}column", S_SMOKE, re.I | re.S) is not None and "Locked" in S_SMOKE
     and re.search(r"headless", S_SMOKE, re.I) is not None,
     "every module's lock column is judged, not the headless clock field")
need("smoke", re.search(r"\b101\b", S_SMOKE) is not None and "--no-tui" in S_SMOKE and re.search(r"headless", S_SMOKE, re.I) is not None,
     "the POC has no headless fallback: without --no-tui and a terminal it exits 101")
need("smoke", re.search(r"windows[^.\n]{0,60}\blost\b[^.\n]{0,20}\b0\b|\blost\b[^.\n]{0,20}\b0\b[^.\n]{0,20}windows", S_SMOKE) is not None,
     "the windows-field loss count is asserted to be 0 (N lost in the audio windows)")
need("smoke", "`name`" in S_SMOKE and "`lock`" in S_SMOKE, "the header's name and lock columns are named")
need("smoke", re.search(r"synth1", S_SMOKE) is not None and re.search(r"synth2", S_SMOKE) is not None
     and re.search(r"exactly[^.\n]{0,60}synth1|synth1 and synth2[^.\n]{0,60}exact", S_SMOKE, re.I) is not None,
     "the two synth names are exactly synth1 and synth2")
need("smoke", re.search(r"asserts?\s+nothing", S_SMOKE, re.I) is not None
     and re.search(r"sync rate", S_SMOKE, re.I) and re.search(r"latency", S_SMOKE, re.I)
     and re.search(r"percentile", S_SMOKE, re.I),
     "it asserts nothing about sync rate, latency or percentiles")
neg_bad = []
for line in S_SMOKE.split("\n"):
    if re.search(r"sync rate|latency|percentile|p99|rtt", line, re.I) and not re.search(r"\b(not|nothing|no|never)\b", line, re.I):
        neg_bad.append(line.strip()[:70])
if neg_bad:
    bad("smoke: a line states sync rate, latency or percentiles without saying they are not asserted: %s" % neg_bad[0])
else:
    ok("smoke: no sync-rate, latency or percentile is stated as an assertion")

# ---------------------------------------------------------------- zero-sample
m = re.search(r'eprintln!\("(check family \{\}: 0 windows analysed)"', main_rs)
if not m:
    bad("zero-sample: HEAD:src/main.rs has no eprintln! of the zero-sample line (wording cannot be verified)")
else:
    line_text = m.group(1).replace("{}", "<name>")
    if line_text in S_ZERO:
        ok("zero-sample: the Zero-sample section quotes the source wording: `%s`" % line_text)
    else:
        bad("zero-sample: the Zero-sample section does not quote the exact line `%s` (from HEAD:src/main.rs)" % line_text)
need("zero-sample", re.search(r"stderr", S_ZERO, re.I) is not None and re.search(r"exit status 1", S_ZERO) is not None,
     "the line is on stderr and the exit status is 1")
need("zero-sample", re.search(r"audio, music, transport", S_ZERO) is not None,
     "one line per family, in the order audio, music, transport")
need("zero-sample", re.search(r"analy[sz]er", S_ZERO) is not None, "judged only in a process that has an analyzer")
need("zero-sample", re.search(r"slot-0|first-listed|first listed", S_ZERO, re.I) is not None,
     "music and transport are judged on the first-listed (slot-0) synth")
need("zero-sample", "--synths 0" in S_ZERO and re.search(r"fails?", S_ZERO) is not None,
     "--synths 0 fails on music and transport")

# ---------------------------------------------------------------- wire
for lit in ("P-01", "C-01", "C-04", "C-100"):
    need("wire", lit in S_WIRE, "names %s" % lit)
need("wire", re.search(r"not (a )?wire reference", doc, re.I) is not None,
     "the baseline is a behavioural reference, not a wire reference")

# ---------------------------------------------------------------- loss
need("loss", re.search(r"not citable", S_LOSS, re.I) is not None, "POC loss figures are not citable")
need("loss", "0 lost" in S_LOSS and "loss figures" in S_LOSS, "the smoke script's '0 lost' and the loss figures are named as never cited")
need("loss", "lost 5" in S_LOSS and re.search(r"not reproduced", S_LOSS, re.I) is not None,
     "one unexplained 'lost 5' run was seen and not reproduced")

for r in results:
    print(r)
PY
  gitpoc show "HEAD:tools/smoke.sh" > "$OUT/smoke-head.sh" 2>/dev/null || : > "$OUT/smoke-head.sh"
  py_out="$(python3 -I "$OUT/doc-check.py" "$OUT/$DOC" "$NOTE02" "$NOTE05" "$CONSTANTS" "$GIT_MAIN_RS" \
      "$FIX_FILES_03" "$FIX_FILES_04" "$FIX_FILES_06" "$REPO_ROOT/doc/design/reports/archaeology/REPORT-0b.md" "$OUT/smoke-head.sh")"
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC5 document checker crashed (python exit $rc)"
  fi
  while IFS= read -r line; do
    case "$line" in
      PASS:*) pass "${line#PASS: }" ;;
      FAIL:*) bad "${line#FAIL: }" ;;
      '') ;;
      *) bad "AC5 unexpected checker output: $line" ;;
    esac
  done <<< "$py_out"
fi

# ------------------------------------------------ SCRIPT (verified at run time)
echo "-- SCRIPT tools/smoke.sh at HEAD passes --no-tui to its own hadropoc command and appends \"\$@\""
if gitpoc show "HEAD:tools/smoke.sh" > "$OUT/smoke-head.sh" 2>/dev/null; then
  if grep -Eq 'loopback .*--no-tui' "$OUT/smoke-head.sh" && grep -Fq '"$@"' "$OUT/smoke-head.sh"; then
    pass "SCRIPT tools/smoke.sh passes --no-tui to hadropoc and appends \"\$@\" (so a caller must not add --no-tui)"
  else
    bad "SCRIPT tools/smoke.sh at HEAD no longer passes --no-tui to its hadropoc command or no longer appends \"\$@\": the smoke section of this check must be revised, not silently accepted"
  fi
else
  bad "SCRIPT tools/smoke.sh cannot be read at HEAD"
fi

# ---------------------------------------------------------------- TREE
echo "-- TREE POC working tree clean vs HEAD (read-only; reported last)"
if [ -e "$MARKER_PATH" ]; then
  printf 'SKIP: epic-003 write in flight (orchestrator marker present)\n'
  note "marker: $MARKER_PATH"
  note "TREE tree-cleanliness assertion skipped; this is not a failure"
else
  porcelain="$(gitpoc status --porcelain 2>&1)"
  if [ -z "$porcelain" ]; then
    pass "TREE git status --porcelain in $POC is empty"
  else
    bad "TREE POC working tree is not clean:"
    printf '%s\n' "$porcelain" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-08 PASSED =="
  exit 0
fi
echo "== 003-08 FAILED =="
exit 1
