#!/usr/bin/env bash
# checks/003-10.sh — acceptance check for ticket 003-10 "Update design set with
# baseline" (epic 003, deliverable 8).
#
# Ticket: doc/design/tickets/003/003-10-Update-design-set-with-baseline.md
#
# The design set (doc/design, tickets/ excluded) must carry the post-fix sync
# rates of the POC tag poc-baseline: one POC reference row next to P-02 in
# doc/design/02-constants.md, the same figures in doc/design/06-evidence-
# appendix.md, the C-130 exception with the freeze after poc-baseline, and the
# lock sub-question answered.  Read-only and unprivileged (C-146): nothing is
# created or modified, the POC is only read through `git show` and
# `git merge-base` with --no-optional-locks.
#
# Overrides (used for the throwaway fixture only):
#   DESIGN_DIR=/path/to/doc/design   (default: this tree's doc/design)
#   HADROPOC=/path/to/poc            (default: /home/dorian/workspace/hadropoc)
#
# RULINGS applied (lead, 003-10 brief; L1-L8):
#   L1  the two post-fix rates are READ from the sync table of BASELINE.md at
#       the tag poc-baseline (column "After"), never pinned.  The new P row and
#       one appendix entry each carry both figures, [measured], poc-baseline,
#       the machine (hackrobustus or i7-1365U) and CONFIG_HZ=250.  Any commit
#       named in such a paragraph must be an ancestor of the tag.
#   L2  P-02 and P-03 rows are pinned by sha256 of their exact text (frozen
#       rows, not snapshots).  Exactly one P row names poc-baseline; it sits
#       next to P-02; P-01 and P-04..P-06 still exist.  37 / 77 is not asserted.
#   L3  every freeze or no-code statement in doc/design (tickets/ excluded, and
#       the 003 epic excluded because it is the task definition that states the
#       exception) carries task 003 or C-130 in its paragraph.  C-130 names
#       poc-baseline; 01-architecture.md says the POC is frozen after it.
#   L4  lock: a [measured] paragraph records LOCKED at 4.1 s within the 25 s
#       run.  Known-inert: the true answer, none, is recorded (BASELINE.md
#       section 5: no in-scope family is known-inert; the audio-only
#       alternative of epic 003 does not apply).
#   L5  no paragraph says the POC runs headless "when there is no terminal"
#       (headless only with --no-tui); a paragraph with the POC, lost/loss and
#       a number must say the loss figure is "not citable".
#   L6  the evidence appendix discloses the HADROPOC_BIN pass-through wrapper
#       of the three poc-baseline smoke runs (a tagged paragraph).
#   L7  every paragraph mentioning poc-baseline carries an evidence tag; an
#       "Open item" paragraph that mentions poc-baseline or 003 names an owner
#       and a falsifier.
#   L8  doc/kickoff/check-design.py passes on the design set.
#
# Scope decisions (reported, not silent):
#   - tickets/ is never scanned (nobody may edit it).
#   - implementation/003-POC-smoke-fixes.md is excluded from L3 (it is the task
#     that owns the exception and quotes the history).  It is still scanned for
#     L5 (headless) and L7 (tags) and must be corrected for L5.
#   - reports/ is excluded from L3 only: those files are archived records of
#     earlier tasks (C-130) and are not rewritten; they are still scanned for
#     L5 (loss figures), L7 (tags) and the open-item rule.
#   - .csv and .diff data files are not scanned; .md, .toml, .sh, .py and .log
#     are.
#   - "freeze" statements are recognised by a fixed phrase list (FREEZE_RE) in a
#     paragraph that is about the POC (POC_RE, the paragraph or its heading).

set -u
cd "$(dirname "$0")/.."

exec python3 - <<'PY'
import hashlib
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path.cwd()
DESIGN = Path(os.environ.get("DESIGN_DIR") or (ROOT / "doc" / "design"))
POC = os.environ.get("HADROPOC") or "/home/dorian/workspace/hadropoc"
TAG = "poc-baseline"
CONSTANTS = DESIGN / "02-constants.md"
APPENDIX = DESIGN / "06-evidence-appendix.md"
ARCH = DESIGN / "01-architecture.md"
EPIC003 = "implementation/003-POC-smoke-fixes.md"

# L2: frozen rows, pinned by the sha256 of the exact line (no newline).
P02_SHA = "bd9b54d70227a6e567df073e55338e5dc540455d1029b12ea7dff9de9641d900"
P03_SHA = "efa69476086bc34c84f08ca4c4eca981e1c59edb5c47042e35b46646a28b3e7c"
P_KEEP = ["P-01", "P-02", "P-03", "P-04", "P-05", "P-06"]

TAG_RE = re.compile(r"\[(code|measured|decision|kernel-doc|memory|standard)\]")
EXC_RE = re.compile(r"task 003|tasks? [\d ,and]*003|epic 003|\bC-130\b|003-(AC|\d)", re.I)
POC_RE = re.compile(r"\bpoc\b|hadropoc|proof of concept", re.I)
FREEZE_RE = re.compile(
    r"frozen (POC|proof|tree|reference|instrument|baseline)"
    r"|POC (is |was |stays |remains )?frozen"
    r"|\bis not modified\b|\bnot modified\b|\bnever modified\b"
    r"|no code goes|goes into (the )?(hadropoc|POC)|written into"
    r"|nothing (else )?(is )?written|nothing else writes|changes only in"
    r"|only change (the POC|ever)|(?<!poc-)\buntouched\b|modified again|writes to it",
    re.I,
)
LOSS_NUM_RE = re.compile(
    r"\b(lost|loss)\b[\"'`*]?\s*(?:\w+\s+){0,2}\d"
    r"|\d[\d ,.]*\s*(million|datagrams?|packets?)?\s*(lost|loss)\b"
    r"|\bzero\b[\w\s-]{0,20}\b(lost|loss)\b"
    r"|\b(lost|loss)\b[\w\s-]{0,20}\bzero\b",
    re.I,
)
NOT_CITABLE_RE = re.compile(r"not (citable|citeable)", re.I)
HEADLESS_NO_TERM_RE = re.compile(r"\bno terminal\b|\bwithout (a )?terminal\b", re.I)
MACHINE_RE = re.compile(r"hackrobustus|i7-1365U")

results = []  # (group, ok, message)


def ok(group, msg):
    results.append((group, True, msg))


def bad(group, msg):
    results.append((group, False, msg))


def git(*args):
    return subprocess.run(
        ["git", "--no-optional-locks", "-C", POC, *args],
        capture_output=True, text=True,
    )


def cells_of(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def read_blocks(path):
    """Split a file into paragraphs: blank-line blocks, one block per table row,
    one block per list item, one block per heading.  Each block knows the
    nearest heading above it."""
    md = path.suffix == ".md"
    out = []
    heading = ""
    cur = None

    def flush():
        nonlocal cur
        if cur is not None and cur["text"].strip():
            out.append(cur)
        cur = None

    for i, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
        if not line.strip():
            flush()
            continue
        if md:
            hm = re.match(r"^#{1,6}\s+(.*)$", line)
            if hm:
                flush()
                heading = hm.group(1)
                out.append({"file": path, "line": i, "text": line, "heading": heading})
                continue
            if line.startswith("|"):
                flush()
                out.append({"file": path, "line": i, "text": line, "heading": heading})
                continue
            if re.match(r"^\s*([-*]|\d+\.)\s", line):
                flush()
                cur = {"file": path, "line": i, "text": line, "heading": heading}
                continue
        if cur is None:
            cur = {"file": path, "line": i, "text": line, "heading": heading}
        else:
            cur["text"] += "\n" + line
    flush()
    return out


def corpus(exclude=(), suffixes=(".md", ".toml", ".sh", ".py", ".log")):
    blocks = []
    for p in sorted(DESIGN.rglob("*")):
        if not p.is_file() or p.suffix not in suffixes:
            continue
        rel = p.relative_to(DESIGN).as_posix()
        if rel.split("/")[0] == "tickets" or any(rel == x or rel.startswith(x + "/") for x in exclude):
            continue
        blocks += read_blocks(p)
    return blocks


def rel(b):
    return b["file"].relative_to(DESIGN).as_posix()


def quote(b, n=110):
    s = " ".join(b["text"].split())
    return s[:n] + ("..." if len(s) > n else "")


def fig_re(s):
    return re.compile(r"(?<![\d.])" + re.escape(s) + r"(?![\d])")


def is_ancestor(sha):
    r = git("cat-file", "-e", sha + "^{commit}")
    if r.returncode != 0:
        return False, "commit %s does not exist in the POC" % sha
    r = git("merge-base", "--is-ancestor", sha, TAG)
    if r.returncode == 0:
        return True, ""
    return False, "commit %s is not an ancestor of %s" % (sha, TAG)


def hex_commits(text):
    out = []
    for tok in re.findall(r"\b[0-9a-f]{7,40}\b", text):
        if re.search(r"\d", tok) and re.search(r"[a-f]", tok) and tok not in out:
            out.append(tok)
    return out


def read_baseline_figures():
    r = git("show", "%s:BASELINE.md" % TAG)
    if r.returncode != 0:
        return None, "cannot read BASELINE.md at %s: %s" % (TAG, r.stderr.strip())
    lines = r.stdout.split("\n")
    start = next((i for i, l in enumerate(lines) if l.startswith("## 3. Sync rate")), None)
    if start is None:
        return None, "BASELINE.md has no section '## 3. Sync rate'"
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    sec = lines[start:end]
    hdr = next((l for l in sec if l.startswith("| Configured rate")), None)
    if hdr is None:
        return None, "BASELINE.md sync table header not found"
    col = next((k for k, c in enumerate(cells_of(hdr)) if c.startswith("After")), None)
    if col is None:
        return None, "BASELINE.md sync table has no 'After' column"
    figs = {}
    for l in sec:
        m = re.match(r"^\|\s*`--sync-hz (50|1000)`\s*\|", l)
        if m:
            c = cells_of(l)
            n = re.search(r"\d+(?:\.\d+)?", c[col])
            figs[m.group(1)] = n.group(0) if n else None
    if figs.get("50") is None or figs.get("1000") is None:
        return None, "BASELINE.md 'After' figures for --sync-hz 50 and 1000 not found"
    return (figs["50"], figs["1000"]), ""


def p_rows(text):
    rows = []
    in_sec = False
    for i, l in enumerate(text.split("\n"), 1):
        if l.startswith("## POC reference values"):
            in_sec = True
            continue
        if in_sec and l.startswith("## "):
            in_sec = False
        if in_sec and re.match(r"^\| P-\d\d \|", l):
            rows.append((i, l, cells_of(l)[0]))
    return rows


def run():
    G_SETUP = "setup"
    if not DESIGN.is_dir():
        bad(G_SETUP, "design directory missing: %s" % DESIGN)
        return report()
    for f in (CONSTANTS, APPENDIX, ARCH):
        if not f.is_file():
            bad(G_SETUP, "missing design file: %s" % f)
    if results and not all(ok_ for _, ok_, _ in results):
        return report()
    constants = CONSTANTS.read_text(encoding="utf-8")
    appendix_blocks = read_blocks(APPENDIX)
    arch_blocks = read_blocks(ARCH)
    figs, err = read_baseline_figures()
    if figs is None:
        bad(G_SETUP, err)
        fig50 = fig1000 = None
    else:
        fig50, fig1000 = figs
        ok(G_SETUP, "figures read from BASELINE.md at %s: 50 -> %s, 1000 -> %s" % (TAG, fig50, fig1000))
    git_ok = git("rev-parse", "--verify", "-q", TAG + "^{commit}").returncode == 0
    if git_ok:
        ok(G_SETUP, "tag %s exists in the POC" % TAG)
    else:
        bad(G_SETUP, "tag %s does not exist in the POC at %s" % (TAG, POC))

    # ---- AC1: POC reference row next to P-02; P-02 unchanged ----
    G1 = "AC1 constants table"
    rows = p_rows(constants)
    p02 = [r for r in rows if r[2] == "P-02"]
    p03 = [r for r in rows if r[2] == "P-03"]
    if len(p02) != 1:
        bad("AC1 P-02 unchanged", "P-02 row found %d times" % len(p02))
    else:
        h = hashlib.sha256(p02[0][1].encode("utf-8")).hexdigest()
        if h == P02_SHA:
            ok("AC1 P-02 unchanged", "P-02 row (line %d) is byte-identical to the frozen row" % p02[0][0])
        else:
            bad("AC1 P-02 unchanged", "P-02 row (line %d) changed: sha256 %s" % (p02[0][0], h))
    if len(p03) != 1:
        bad("AC1 P-02 unchanged", "P-03 row found %d times" % len(p03))
    else:
        h = hashlib.sha256(p03[0][1].encode("utf-8")).hexdigest()
        if h == P03_SHA:
            ok("AC1 P-02 unchanged", "P-03 row (line %d) is byte-identical to the frozen row" % p03[0][0])
        else:
            bad("AC1 P-02 unchanged", "P-03 row (line %d) changed: sha256 %s" % (p03[0][0], h))
    ids = [r[2] for r in rows]
    missing = [k for k in P_KEEP if ids.count(k) != 1]
    if not missing:
        ok("AC1 P-02 unchanged", "rows P-01 to P-06 each present once (no existing id removed or renamed)")
    else:
        bad("AC1 P-02 unchanged", "existing POC rows missing or duplicated: %s" % ", ".join(missing))

    new = [r for r in rows if TAG in r[1]]
    if len(new) != 1:
        bad(G1, "exactly one POC reference row must name %s (found %d)" % (TAG, len(new)))
    else:
        ln, line, rid = new[0]
        if rid in P_KEEP:
            bad(G1, "the %s row reuses an existing id %s" % (TAG, rid))
        else:
            ok(G1, "new POC row %s names %s" % (rid, TAG))
        if p02 and abs(ln - p02[0][0]) == 1:
            ok(G1, "new row %s sits next to P-02 (lines %d and %d)" % (rid, ln, p02[0][0]))
        else:
            bad(G1, "new row %s is not next to P-02 (line %d vs %d)" % (rid, ln, p02[0][0] if p02 else -1))
        if fig50 and fig1000:
            for f in (fig50, fig1000):
                if fig_re(f).search(line):
                    ok(G1, "new row %s carries figure %s (read from BASELINE.md)" % (rid, f))
                else:
                    bad(G1, "new row %s lacks figure %s from BASELINE.md at %s" % (rid, f, TAG))
        if "[measured]" in line:
            ok(G1, "new row %s is tagged [measured]" % rid)
        else:
            bad(G1, "new row %s is not tagged [measured]" % rid)
        if "CONFIG_HZ=250" in line and MACHINE_RE.search(line):
            ok(G1, "new row %s names the machine (host or CPU) and CONFIG_HZ=250" % rid)
        else:
            bad(G1, "new row %s lacks the machine (hackrobustus or i7-1365U) and CONFIG_HZ=250" % rid)
        for sha in hex_commits(line):
            good, why = is_ancestor(sha)
            if good:
                ok(G1, "commit %s named in the new row is an ancestor of %s" % (sha, TAG))
            else:
                bad(G1, "new row %s: %s" % (rid, why))

    # the C-130 row carries the freeze after the tag (C-130 is the exception)
    c130 = [l for l in constants.split("\n") if l.startswith("| C-130 |")]
    if len(c130) == 1 and TAG in c130[0] and re.search(r"frozen", c130[0], re.I) and EXC_RE.search(c130[0]):
        ok("AC3 rule", "C-130 names task 003 as the exception and the freeze after %s" % TAG)
    else:
        bad("AC3 rule", "C-130 row must name task 003 as the exception and say the POC is frozen after %s" % TAG)

    # ---- AC2: evidence appendix carries the same figures ----
    G2 = "AC2 evidence appendix"
    entry = None
    for b in appendix_blocks:
        if TAG in b["text"] and fig50 and fig1000 and fig_re(fig50).search(b["text"]) and fig_re(fig1000).search(b["text"]):
            entry = b
            break
    if entry is None:
        bad(G2, "no appendix paragraph carries both figures (%s, %s) with %s" % (fig50, fig1000, TAG))
    else:
        miss = []
        if "[measured]" not in entry["text"]:
            miss.append("[measured]")
        if not MACHINE_RE.search(entry["text"]):
            miss.append("machine (hackrobustus or i7-1365U)")
        if "CONFIG_HZ=250" not in entry["text"]:
            miss.append("CONFIG_HZ=250")
        if miss:
            bad(G2, "appendix line %d lacks: %s" % (entry["line"], "; ".join(miss)))
        else:
            ok(G2, "appendix line %d carries both figures, [measured], %s, machine and CONFIG_HZ=250" % (entry["line"], TAG))
        for sha in hex_commits(entry["text"]):
            good, why = is_ancestor(sha)
            if good:
                ok(G2, "commit %s named in the appendix is an ancestor of %s" % (sha, TAG))
            else:
                bad(G2, "appendix line %d: %s" % (entry["line"], why))
    unpriv = [b for b in appendix_blocks if re.search(r"unprivileged", b["text"], re.I)
              and re.search(r"\bnot\b[^.\n]{0,30}\bstrict\b", b["text"], re.I) and "C-147" in b["text"]]
    if unpriv:
        ok(G2, "appendix states the runs were unprivileged and not strict mode (C-147)")
    else:
        bad(G2, "appendix must state that the POC runs were unprivileged and not in strict mode (C-147)")
    wrap = [b for b in appendix_blocks if "HADROPOC_BIN" in b["text"]
            and re.search(r"wrapper|pass-through|passthrough", b["text"], re.I)
            and re.search(r"standard output|stdout", b["text"], re.I)
            and TAG_RE.search(b["text"])]
    if wrap:
        ok("L6 wrapper disclosure", "appendix line %d discloses the HADROPOC_BIN pass-through wrapper" % wrap[0]["line"])
    else:
        bad("L6 wrapper disclosure", "appendix lacks a tagged paragraph that discloses the HADROPOC_BIN wrapper (pass-through, stdout only) of the three poc-baseline runs")
    fixes_ok = {
        "deadline-paced sync (fix 2)": r"deadline",
        "one process-owned writer per metrics file (fix 1)": r"one writer|single[- ]writer|process-owned",
        "non-zero exit on a zero-sample family (fix 3)": r"non-zero|zero[- ](sample|window)",
    }
    for what, pat in fixes_ok.items():
        if any(TAG in b["text"] and re.search(pat, b["text"], re.I) for b in appendix_blocks):
            ok(G2, "appendix records the baseline fix: %s" % what)
        else:
            bad(G2, "appendix does not record the baseline fix: %s" % what)
    if any(TAG in b["text"] and re.search(r"loss counter", b["text"], re.I)
           and re.search(r"not fixed|unfixed|kept|keeps|still", b["text"], re.I) for b in appendix_blocks):
        ok(G2, "appendix records that the loss-counter bug is kept at %s" % TAG)
    else:
        bad(G2, "appendix must record that the loss-counter bug is kept (not fixed) at %s" % TAG)
    if any(TAG in b["text"] and re.search(r"\blosse?s?\b|\blost\b|\bloss\b", b["text"], re.I)
           and NOT_CITABLE_RE.search(b["text"]) for b in appendix_blocks):
        ok("L5 loss figures", "appendix says the POC loss figures at %s are not citable" % TAG)
    else:
        bad("L5 loss figures", "appendix must say the POC loss figures at %s (including 0 lost) are not citable" % TAG)

    # the architecture (C-171's home) says the POC is frozen after the tag
    if any(TAG in b["text"] and re.search(r"frozen|freeze", b["text"], re.I) and re.search(r"\bafter\b", b["text"], re.I)
           for b in arch_blocks):
        ok("AC3 rule", "01-architecture.md says the POC is frozen after %s" % TAG)
    else:
        bad("AC3 rule", "01-architecture.md must say the POC is frozen after %s (C-171 home)" % TAG)

    # ---- AC3 / L3: the unqualified rule ----
    G3 = "AC3 rule"
    offenders = []
    for b in corpus(exclude=(EPIC003, "reports")):
        if FREEZE_RE.search(b["text"]) and POC_RE.search(b["text"] + " " + b["heading"]):
            if not EXC_RE.search(b["text"]):
                offenders.append(b)
    if not offenders:
        ok(G3, "every freeze or no-code statement carries task 003 or C-130")
    for b in offenders:
        bad(G3, "%s:%d: freeze or no-code statement without task 003 or C-130: \"%s\"" % (rel(b), b["line"], quote(b)))

    # ---- L4: lock sub-question and known-inert answer ----
    G4 = "L4 lock and known-inert"
    lock_ok = [b for b in corpus(exclude=())
               if re.search(r"4\.1 ?s\b", b["text"]) and re.search(r"LOCK", b["text"], re.I)
               and re.search(r"25 ?s\b", b["text"]) and "[measured]" in b["text"]
               and (TAG in b["text"] or POC_RE.search(b["text"]))]
    if lock_ok:
        ok(G4, "%s:%d records LOCKED at 4.1 s within the 25 s run, [measured]" % (rel(lock_ok[0]), lock_ok[0]["line"]))
    else:
        bad(G4, "no [measured] paragraph records that deadline pacing still reaches LOCKED at 4.1 s within the 25 s run")
    inert_ok = [b for b in corpus(exclude=(EPIC003,))
                if "known-inert" in b["text"] and TAG_RE.search(b["text"])
                and re.search(r"\b(none|not|no)\b", b["text"], re.I) and TAG in b["text"]]
    if inert_ok:
        ok(G4, "%s:%d records that no in-scope check family is known-inert at %s" % (rel(inert_ok[0]), inert_ok[0]["line"], TAG))
    else:
        bad(G4, "no tagged paragraph records, for %s, that no in-scope check family is known-inert" % TAG)

    # ---- L5: headless only with --no-tui; loss figures not citable ----
    G5 = "L5 headless and loss"
    for b in corpus(exclude=()):
        if HEADLESS_NO_TERM_RE.search(b["text"]):
            bad(G5, "%s:%d: says headless when there is no terminal (headless is --no-tui only): \"%s\"" % (rel(b), b["line"], quote(b)))
    if not any(HEADLESS_NO_TERM_RE.search(b["text"]) for b in corpus(exclude=())):
        ok(G5, "no paragraph says the POC runs headless when there is no terminal")
    for b in corpus(exclude=(EPIC003,)):
        ctx = b["text"] + " " + b["heading"]
        if POC_RE.search(ctx) and LOSS_NUM_RE.search(b["text"]) and not NOT_CITABLE_RE.search(b["text"]):
            bad(G5, "%s:%d: POC loss figure with no 'not citable' statement: \"%s\"" % (rel(b), b["line"], quote(b)))

    # ---- L7: tags on paragraphs that mention the tag; open items ----
    G7 = "AC4 tags"
    for b in corpus(exclude=(EPIC003,)):
        if b["text"].lstrip().startswith("#"):
            continue
        if TAG in b["text"] and not TAG_RE.search(b["text"]):
            bad(G7, "%s:%d: paragraph mentions %s without an evidence tag: \"%s\"" % (rel(b), b["line"], TAG, quote(b)))
    G5b = "AC5 open items"
    for b in corpus(exclude=(EPIC003,)):
        t = b["text"].lstrip().lstrip("*_ ").lower()
        if t.startswith("open item") and (TAG in b["text"] or "003" in b["text"]):
            if not (re.search(r"\bowner\b", b["text"], re.I) and re.search(r"\bfalsifier\b", b["text"], re.I)):
                bad(G5b, "%s:%d: open item for %s without owner and falsifier: \"%s\"" % (rel(b), b["line"], "task 003 or " + TAG, quote(b)))

    # ---- L8: the design-set checker ----
    G8 = "L8 kickoff check"
    checker = ROOT / "doc" / "kickoff" / "check-design.py"
    if checker.is_file():
        r = subprocess.run([sys.executable, str(checker), str(DESIGN)], capture_output=True, text=True)
        if r.returncode == 0:
            ok(G8, "doc/kickoff/check-design.py passes on the design set")
        else:
            bad(G8, "doc/kickoff/check-design.py fails: %s" % (r.stdout + r.stderr).strip()[:300])
    else:
        bad(G8, "doc/kickoff/check-design.py not found")

    report()


def report():
    print("003-10: design set updated with the poc-baseline sync rates")
    print("  design: %s" % DESIGN)
    print("  poc:    %s (tag %s)" % (POC, TAG))
    current = None
    for group, passed, message in results:
        if group != current:
            print("  %s" % group)
            current = group
        print("    %s %s" % ("OK  " if passed else "FAIL", message))
    failures = sum(1 for _, p, _ in results if not p)
    if failures == 0:
        print("RESULT: PASS (%d checks)" % len(results))
        sys.exit(0)
    print("RESULT: FAIL (%d of %d checks failed)" % (failures, len(results)))
    sys.exit(1)


if __name__ == "__main__":
    run()
PY
