#!/usr/bin/env bash
# 004-23 — design document updates and epic 004 close-out.
#
# RED-phase document check. It validates the design set against the four
# acceptance criteria and the definition-of-done items of ticket 004-23, under
# the binding rulings of the lead of 2026-11-16 (quoted below). Every list it
# needs is read at run time: the constant groups from the ticket's Context,
# the constant rows from 02-constants.md, the open sub-questions from the
# epic README, the heading anchors from wire.md and 05-risks-and-open.md.
#
# Overrides (self-testing on fixtures only):
#   WIRE_MD            the specification (default doc/design/spec/wire.md)
#   CONSTANTS_MD       the constants table (default doc/design/02-constants.md)
#   RISKS_MD           the risks and open items (default doc/design/05-risks-and-open.md)
#   EPIC_README_MD     the epic 004 README (default doc/design/tickets/004/README.md)
#   TICKET_MD          the ticket (default doc/design/tickets/004/004-23-*.md)
#   POC_DIR            the proof-of-concept repository (default /home/dorian/workspace/hadropoc)
# When any of WIRE_MD, CONSTANTS_MD, RISKS_MD, EPIC_README_MD or TICKET_MD is
# overridden, the origin/main comparison (R1) and the cargo runs (R6) are
# skipped with a NOTE. POC_DIR is never a fixture: R5 always runs.
#
# Lead rulings (lead ruling 2026-11-16; they override the ticket where they differ):
#   R1  "Nobody in the swarm edits doc/design/02-constants.md, any file under
#        doc/design/tickets/, doc/design/adr/ or doc/design/reports/. The check
#        must FAIL if any of these differs from origin/main."
#   R2  "Instead, 'constants table first' is met by a DRAFT LIST for the design
#        owner, a section of wire.md (any heading that contains the words
#        'constants' and 'draft', case-insensitive)."
#   R3  "every row of the epic README's table 'Open sub-questions owned by this
#        epic' must be answered in wire.md or 05-risks-and-open.md ... or
#        states an open item with BOTH an owner and a falsifier."
#   R4  "a tag check over wire.md ... reuse the house tag rule ... do NOT invent
#        a stricter rule than the house one. Allowed tags are exactly the six."
#   R5  "zero proof-of-concept files changed: rev-parse HEAD equals
#        rev-parse 'poc-baseline^{commit}' and status --porcelain is empty."
#   R6  "cargo run --quiet --bin wire-check must report checks clean."
#   R7  "where the ticket or these rulings leave wording, layout or placement
#        free, accept every conforming form ... for 'confirmed unchanged' and
#        'answered' require the positive form and reject sentences containing
#        'not' directly before the keyword."
#   R8  the mutations of the rulings: each must turn a green fixture red.
#
# The evidence tag rule (R4), adopted from the house rule already in
# checks/004-22.sh, 004-27.sh and 004-29.sh and from the table rule of
# checks/004-21.sh: the allowed tags are exactly [decision], [code],
# [measured], [kernel-doc], [standard] (C-132) and [memory]. A unit of wire.md
# is a paragraph or a list item, outside code fences and headings, after
# markdown links are reduced to their text. A unit of five or more words must
# contain one of the six tags; a unit of fewer words (a navigation or label
# line) is not a factual claim and is not counted. A table must contain a tag,
# or the nearest preceding unit of five or more words in its section must
# carry one. The count of units that fail is reported; 0 is required. Bracketed
# words other than the six are not policed: `[budget]` is a TOML table name.
#
# What is checked, per acceptance criterion and definition-of-done item:
#
#   AC1  Ticket: "Each of the four constant groups above is either confirmed
#        unchanged or updated in the constants table, with the reason." (R2:
#        the draft list replaces the table edit.) Each of the four constant
#        groups of the ticket's Context ("C-04 and C-137", "C-10", "C-17",
#        "C-20 / C-21 / C-22"; the label before the
#        colon of each bullet) is covered, id by id, inside the draft section
#        (a heading of wire.md containing "constants" and "draft") or a section
#        that it links to by one hop. An id is covered by EITHER a unit that
#        names it with the positive keyword "confirmed unchanged" / "unchanged"
#        / "confirmed", not preceded by "not", "no" or "never" within two
#        words, followed by a reason of three or more words; OR a draft entry.
#        A draft entry is a table row under a header with "current" and
#        "proposed" columns, or a labelled paragraph or list item ("Current:",
#        "Proposed:", "Justification:" or "Reference:"); its constant ids come
#        before the first label. Each entry must carry: a current text that is
#        verbatim (whitespace-normalised) in the 02-constants.md row of each of
#        its ids, or "no row exists yet" where no such row exists; a proposed
#        text, non-empty and different from the current text; and a
#        justification that resolves to a heading of wire.md (a markdown anchor
#        that is a heading slug, or a heading name or "Section, Subsection" in
#        plain text). An entry that fails is a FAIL; an id with neither a
#        confirmed statement nor a valid entry is a FAIL.
#   AC2  Ticket: "Every open sub-question in the epic has a recorded answer or
#        remains an open item with owner and falsifier." Each row of the epic README's table "Open sub-questions owned by this
#        epic" is answered: for each ticket id in the row's ticket column, some
#        unit of wire.md or 05-risks-and-open.md names that id and either
#        records an answer (answer, answered, decided, settled, fixed or
#        closed, not preceded by "not", "no" or "never" within two words) or
#        states an open item (the word "open" with the words "owner" and
#        "falsifier" in the same unit). The README itself is not edited (R1).
#   AC3  Ticket: "A tag check over wire.md finds 0 untagged factual claims."
#        The tag rule above, over the whole of wire.md. The count of untagged
#        claims is reported; the required count is 0.
#   AC4  Ticket: "Zero POC files changed." Zero proof-of-concept files changed (R5): HEAD of POC_DIR equals
#        poc-baseline^{commit} and its status is empty. Skipped with a NOTE
#        when POC_DIR is absent (CI).
#   DoD  "Design documents updated where behaviour differs, constants table
#        first" (ticket DoD 2): R1 (no edit to 02-constants.md, tickets/, adr/ or reports/
#        against origin/main) and AC1 (the draft list).
#        "No new open item without an owner and a falsifier": every unit of
#        wire.md that declares an open item ("Open item" at the start of a
#        unit, after bold markers) names an owner and a falsifier.
#        "Every new factual claim carries an evidence tag": AC3.
#        "The machine-readable tables parse and round-trip": R6, cargo run
#        --bin wire-check reports checks clean, and cargo test -p
#        hadronyche-tools --test wire_tables passes at least 12 tests.
#        "The frozen POC is not modified": AC4.
#        "Tests green in CI, unprivileged": not asserted here; CI status is
#        outside this script.
#
# Not checked: the Work items 5 and 6 of the ticket (epic 001 notes) and the
# queued Review record items (X-8, X-10, X-12, X-21, X-50, X-51 and others):
# neither the ticket's acceptance criteria nor the rulings fix their wording,
# so asserting them would make a design choice.

set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
CONSTANTS="${CONSTANTS_MD:-doc/design/02-constants.md}"
RISKS="${RISKS_MD:-doc/design/05-risks-and-open.md}"
EPIC_README="${EPIC_README_MD:-doc/design/tickets/004/README.md}"
TICKET="${TICKET_MD:-}"
if [ -z "$TICKET" ]; then
    TICKET=$(ls doc/design/tickets/004/004-23-*.md 2>/dev/null | head -n 1)
fi
POC="${POC_DIR:-/home/dorian/workspace/hadropoc}"

overridden=0
if [ -n "${WIRE_MD:-}" ] || [ -n "${CONSTANTS_MD:-}" ] || [ -n "${RISKS_MD:-}" ] || [ -n "${EPIC_README_MD:-}" ] || [ -n "${TICKET_MD:-}" ]; then
    overridden=1
fi

python3 - "$WIRE" "$CONSTANTS" "$RISKS" "$EPIC_README" "$TICKET" "$POC" "$overridden" <<'PY'
import os
import re
import subprocess
import sys

wire_path, const_path, risks_path, readme_path, ticket_path, poc_dir, overridden = sys.argv[1:8]
overridden = overridden == "1"
fails = []
notes = []


def fail(label, msg):
    fails.append((label, msg))


# ---------------------------------------------------------------- helpers
TAGS = ("decision", "code", "measured", "kernel-doc", "standard", "memory")
TAG_RE = re.compile(r"\[(" + "|".join(TAGS) + r")\]")
C_RE = re.compile(r"\bC-(\d{1,3})\b")
TICKET_ID_RE = re.compile(r"(?<![\w-])\d{3}-\d{2}[a-z]?(?![\w-])")
ANSWER_RE = re.compile(r"\b(answers?|answered|decided|settled|fixed|closed)\b", re.I)
KEEP_RE = re.compile(r"\b(confirmed unchanged|unchanged|confirmed)\b", re.I)
LABEL_RE = re.compile(
    r"(?i)(?<![A-Za-z])(current|proposed|justification|justified by|rationale|basis|reference)"
    r"(?:\s+(?:text|value|form))?\s*:")
NO_ROW_RE = re.compile(r"no\s+(such\s+)?row\s+(exists|yet)", re.I)
TABLE_SEP_RE = re.compile(r"^:?-+:?$")


def canon(n):
    return f"C-{int(n):02d}" if int(n) < 100 else f"C-{int(n)}"


def ids_in(text):
    return {canon(m.group(1)) for m in C_RE.finditer(text)}


def ws(t):
    return re.sub(r"\s+", " ", re.sub(r"[`*]", "", t)).strip()


def norm(t):
    return ws(t).lower().rstrip(".")


def strip_links(t):
    return re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", t)


def slugify(t):
    t = strip_links(t)
    t = re.sub(r"[`*~]", "", t).strip().lower()
    t = re.sub(r"[^\w\- ]", "", t)
    return t.replace(" ", "-")


def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip() for c in s.split("|")]


def clean_value(t):
    t = re.sub(r"[`*]", "", t or "").strip()
    t = t.strip("\"'“”").strip()
    return t.rstrip(".;, ").strip()


class Doc:
    def __init__(self, path):
        with open(path, encoding="utf-8") as fh:
            self.text = fh.read()
        self.lines = self.text.split("\n")
        self.in_code = []
        fence = False
        for line in self.lines:
            if line.strip().startswith("```"):
                fence = not fence
                self.in_code.append(True)
            else:
                self.in_code.append(fence)
        self.heads = []
        seen = {}
        for i, line in enumerate(self.lines):
            if self.in_code[i]:
                continue
            m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
            if not m:
                continue
            text = m.group(2).strip()
            slug = slugify(text)
            if slug in seen:
                seen[slug] += 1
                slug = f"{slug}-{seen[slug]}"
            else:
                seen[slug] = 0
            self.heads.append({"idx": i, "level": len(m.group(1)), "text": text, "slug": slug})
        self.slugs = {h["slug"] for h in self.heads}
        self.head_norms = set()
        for h in self.heads:
            self.head_norms.add(norm(h["text"]))
            self.head_norms.add(norm(re.sub(r"^\d+(\.\d+)*\.?\s+", "", h["text"])))

    def section_range(self, h):
        end = len(self.lines)
        for later in self.heads:
            if later["idx"] > h["idx"] and later["level"] <= h["level"]:
                end = later["idx"]
                break
        return h["idx"], end

    def units(self, lo=0, hi=None):
        """Paragraphs ('p'), list items ('li'), table rows ('row'), tables ('table') and headings ('h')."""
        hi = len(self.lines) if hi is None else hi
        out = []
        cur = None

        def flush():
            nonlocal cur
            if cur is not None:
                out.append(cur)
                cur = None

        i = lo
        while i < hi:
            line = self.lines[i]
            s = line.strip()
            if self.in_code[i]:
                flush()
                i += 1
                continue
            if not s:
                flush()
                i += 1
                continue
            if re.match(r"^#{1,6}\s", s):
                flush()
                out.append({"kind": "h", "text": s, "line": i + 1})
                i += 1
                continue
            if s.startswith("|"):
                flush()
                j = i
                rows = []
                while j < hi and not self.in_code[j] and self.lines[j].strip().startswith("|"):
                    rows.append((j, split_row(self.lines[j])))
                    j += 1
                header = None
                data = rows
                if len(rows) > 1 and all(TABLE_SEP_RE.match(c) for c in rows[1][1] if c):
                    header = [ws(c).lower() for c in rows[0][1]]
                    data = rows[2:]
                out.append({"kind": "table", "text": "\n".join(self.lines[k] for k in range(i, j)),
                            "line": i + 1})
                for k, cells in data:
                    out.append({"kind": "row", "cells": cells, "header": header,
                                "text": " | ".join(cells), "line": k + 1})
                i = j
                continue
            if re.match(r"^([-*]|\d+\.)\s+", s):
                flush()
                cur = {"kind": "li", "lines": [s], "line": i + 1}
            elif cur is None:
                cur = {"kind": "p", "lines": [s], "line": i + 1}
            else:
                cur["lines"].append(s)
            i += 1
        flush()
        for u in out:
            if "lines" in u:
                u["text"] = " ".join(u.pop("lines"))
        return out

    def find_heads(self, pattern):
        return [h for h in self.heads if re.search(pattern, h["text"], re.I)]


def has_tag(text):
    return bool(TAG_RE.search(text))


def words(text):
    return re.findall(r"\S+", strip_links(text))


def keyword_positive(text, regex):
    """True when a match of regex is not preceded by not / no / never within two words."""
    for m in regex.finditer(text):
        before = text[:m.start()]
        if re.search(r"\b(not|no|never)\b(\s+\S+){0,2}\s*$", before, re.I):
            continue
        return m
    return None


def confirmed_reason(text):
    """A 'confirmed unchanged' unit with a reason of three or more words after the keyword."""
    for m in KEEP_RE.finditer(text):
        before = text[:m.start()]
        if re.search(r"\b(not|no|never)\b(\s+\S+){0,2}\s*$", before, re.I):
            continue
        reason = re.sub(r"^[\s:;,\-—–.()*]*(because|since|as|that)?\b", "", text[m.end():], flags=re.I)
        if len(re.findall(r"[A-Za-z][A-Za-z'\-]*", reason)) >= 3:
            return True
    return False


def answered(text):
    return keyword_positive(text, ANSWER_RE) is not None


def open_with_owner_and_falsifier(text):
    return bool(re.search(r"\bopen\b", text, re.I)) and bool(re.search(r"\bowner\b", text, re.I)) \
        and bool(re.search(r"\bfalsifier\b", text, re.I))


def reference_resolves(text, doc):
    """A markdown anchor that is a heading slug, or a plain heading name / 'Section, Subsection'."""
    anchors = re.findall(r"\]\(#([^)\s]+)\)", text)
    if anchors:
        return all(a in doc.slugs for a in anchors)
    segs = []
    for g in re.findall(r"\(([^()]*)\)", text):
        segs += g.split(";")
    segs += re.findall(r"`([^`]+)`", text)
    segs += text.split(";")
    for seg in segs:
        s = re.sub(r"^\s*(see|in|per|section|§)\s+", "", seg.strip().strip(".,"), flags=re.I)
        if not s:
            continue
        if norm(s) in doc.head_norms:
            return True
        if "," in s:
            parts = [p.strip() for p in s.split(",")]
            if len(parts) >= 2 and all(norm(p) in doc.head_norms for p in parts):
                return True
    return False


def read(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


# ---------------------------------------------------------------- inputs
wire = Doc(wire_path)
risks = Doc(risks_path)
ticket_text = read(ticket_path)
const_lines = read(const_path).split("\n")
const_rows = {}
for line in const_lines:
    m = re.match(r"^\|\s*(C-\d+)\s*\|", line)
    if m:
        const_rows.setdefault(canon(m.group(1)[2:]), []).append(line)

# ---------------------------------------------------------------- AC1
ctx = re.search(r"^## Context\s*$(.*?)^## Work\s*$", ticket_text, re.M | re.S)
groups = []
if ctx:
    for line in ctx.group(1).split("\n"):
        m = re.match(r"^- (C-.*)$", line)
        if m:
            label = m.group(1).split(":")[0]
            groups.append(sorted(ids_in(label)))
if len(groups) != 4:
    fail("AC1", f"the ticket's Context lists {len(groups)} constant group(s), expected 4 (C-04 and C-137; C-10; C-17; C-20 / C-21 / C-22)")

draft_heads = [h for h in wire.heads if re.search(r"constants", h["text"], re.I)
               and re.search(r"draft", h["text"], re.I)]
if not draft_heads:
    fail("AC1", "wire.md has no heading that contains both 'constants' and 'draft': the draft list for the design owner is missing (R2)")

ranges = [wire.section_range(h) for h in draft_heads]
for lo, hi in list(ranges):
    for m in re.finditer(r"\]\(#([^)\s]+)\)", "\n".join(wire.lines[lo:hi])):
        for h in wire.heads:
            if h["slug"] == m.group(1):
                r = wire.section_range(h)
                if r not in ranges:
                    ranges.append(r)

scope_units = []
seen_lines = set()
for lo, hi in ranges:
    for u in wire.units(lo, hi):
        key = (u["kind"], u["line"], u.get("text", "")[:40])
        if key in seen_lines:
            continue
        seen_lines.add(key)
        scope_units.append(u)


def entries_of(u):
    """Draft entries carried by one unit: a row under current/proposed headers, or labelled text."""
    out = []
    if u["kind"] == "row" and u["header"]:
        hdr = u["header"]
        ci = [j for j, h in enumerate(hdr) if re.search(r"\bcurrent\b", h)]
        pi = [j for j, h in enumerate(hdr) if re.search(r"\bproposed\b", h)]
        ji = [j for j, h in enumerate(hdr) if re.search(r"justif|reference|rationale|basis|because|reason", h)]
        if ci and pi:
            cells = u["cells"]
            label_cols = set(ci + pi + ji)
            ids = set()
            for j, c in enumerate(cells):
                if j not in label_cols:
                    ids |= ids_in(c)

            def at(j):
                return cells[j] if j is not None and j < len(cells) else None

            if ids:
                out.append({"ids": ids, "cur": at(ci[0]), "prop": at(pi[0]),
                            "just": at(ji[0]) if ji else None, "line": u["line"]})
    elif u["kind"] in ("p", "li"):
        s = u["text"].replace("**", "").replace("__", "")
        ms = list(LABEL_RE.finditer(s))
        if ms:
            ids = ids_in(s[:ms[0].start()])
            if ids:
                parts = {}
                for k, m in enumerate(ms):
                    end = ms[k + 1].start() if k + 1 < len(ms) else len(s)
                    key = m.group(1).lower()
                    kind = "cur" if key == "current" else "prop" if key == "proposed" else "just"
                    parts.setdefault(kind, s[m.end():end])
                out.append({"ids": ids, "cur": parts.get("cur"), "prop": parts.get("prop"),
                            "just": parts.get("just"), "line": u["line"]})
    return out


confirmed = set()
entries = []
for u in scope_units:
    if u["kind"] in ("p", "li", "row"):
        ids = ids_in(u["text"])
        if ids and confirmed_reason(u["text"]):
            confirmed |= ids
        entries += entries_of(u)

covered = set(confirmed)
for e in entries:
    problems = []
    cur = clean_value(TAG_RE.sub("", e["cur"] or ""))
    prop = clean_value(TAG_RE.sub("", e["prop"] or ""))
    if not cur:
        problems.append("no current text")
    if not prop:
        problems.append("no proposed text")
    elif norm(prop) == norm(cur):
        problems.append("proposed text equals current text")
    just = TAG_RE.sub("", e["just"] or "").strip()
    if not just:
        problems.append("no justification reference")
    elif not reference_resolves(just, wire):
        problems.append(f"justification '{strip_links(just).strip()[:60]}' does not resolve to a heading of wire.md")
    for cid in sorted(e["ids"]):
        if NO_ROW_RE.search(cur or ""):
            if cid in const_rows:
                problems.append(f"{cid}: current text says no row exists, but 02-constants.md has one")
        elif cid not in const_rows:
            problems.append(f"{cid}: no row in 02-constants.md, and current text does not say so")
        elif not any(ws(cur) in ws(r) for r in const_rows[cid]):
            problems.append(f"{cid}: current text is not verbatim in the 02-constants.md row of {cid}")
    if problems:
        fail("AC1", f"draft entry at wire.md line {e['line']} ({', '.join(sorted(e['ids']))}): " + "; ".join(problems))
    else:
        covered |= e["ids"]

for g in groups:
    missing = [i for i in g if i not in covered]
    if missing:
        fail("AC1", f"constant group {', '.join(g)}: {', '.join(missing)} has neither a 'confirmed unchanged' "
                    "statement with a reason nor a valid draft entry in the draft section or a section it links to")


# ---------------------------------------------------------------- AC2
readme = Doc(readme_path)
rd_heads = readme.find_heads(r"open sub-questions")
rows = []
if not rd_heads:
    fail("AC2", "the epic README has no 'Open sub-questions' heading")
for h in rd_heads:
    lo, hi = readme.section_range(h)
    for u in readme.units(lo, hi):
        if u["kind"] == "row" and u["header"] and any("ticket" in c for c in u["header"]):
            col = next(j for j, c in enumerate(u["header"]) if "ticket" in c)
            cells = u["cells"]
            rows.append((cells[0], cells[col] if col < len(cells) else ""))
if rd_heads and not rows:
    fail("AC2", "the 'Open sub-questions' table of the epic README has no rows")

answer_units = [u for doc in (wire, risks) for u in doc.units() if u["kind"] in ("p", "li", "row")]
for question, ticket_cell in rows:
    tids = TICKET_ID_RE.findall(ticket_cell)
    if not tids:
        fail("AC2", f"README question '{question[:60]}' names no ticket")
    for tid in tids:
        pat = re.compile(r"(?<![\w-])" + re.escape(tid) + r"(?![\w-])")
        ok = False
        for u in answer_units:
            if pat.search(u["text"]) and (answered(u["text"]) or open_with_owner_and_falsifier(u["text"])):
                ok = True
                break
        if not ok:
            fail("AC2", f"README question '{question[:60]}' (ticket {tid}): no unit of wire.md or 05-risks-and-open.md "
                        "records an answer or states an open item with an owner and a falsifier")

# DoD: no new open item without an owner and a falsifier (every declared open item of wire.md)
for u in wire.units():
    if u["kind"] in ("p", "li") and re.match(r"^(-\s+)?Open item\b", u["text"].replace("**", "")):
        if not open_with_owner_and_falsifier(u["text"]):
            fail("DoD", f"open item at wire.md line {u['line']} names no owner or no falsifier: "
                        f"'{u['text'][:70]}'")

# ---------------------------------------------------------------- AC3
untagged = []
last_prose_tagged = False
for u in wire.units():
    if u["kind"] == "h":
        last_prose_tagged = False
    elif u["kind"] in ("p", "li"):
        if len(words(u["text"])) >= 5:
            tagged = has_tag(u["text"])
            if not tagged:
                untagged.append((u["line"], u["text"][:90]))
            last_prose_tagged = tagged
    elif u["kind"] == "table":
        if not (has_tag(u["text"]) or last_prose_tagged):
            untagged.append((u["line"], "table without an evidence tag, after an untagged paragraph"))
print(f"INFO: AC3 tag rule over wire.md: {len(untagged)} untagged claim(s) (required: 0)")
for line, text in untagged[:3]:
    print(f"INFO:   line {line}: {text}")
if untagged:
    fail("AC3", f"{len(untagged)} untagged factual claim(s) in wire.md; first at line {untagged[0][0]}")

# ---------------------------------------------------------------- R1 (DoD: design documents)
if overridden:
    notes.append("NOTE: fixture overrides set; R1 (no edit to 02-constants.md, tickets/, adr/ or reports/ "
                 "against origin/main) is skipped")
else:
    for path in ["doc/design/02-constants.md", "doc/design/tickets", "doc/design/adr", "doc/design/reports"]:
        proc = subprocess.run(["git", "diff", "--quiet", "origin/main", "--", path],
                              capture_output=True, text=True)
        if proc.returncode == 1:
            fail("R1", f"{path} differs from origin/main; the swarm does not edit it (rulings R1, R2)")
        elif proc.returncode != 0:
            fail("R1", f"git could not compare {path} with origin/main: {proc.stderr.strip()[:120]}")
        else:
            untracked = subprocess.run(["git", "status", "--porcelain", "--untracked-files=all", "--", path],
                                       capture_output=True, text=True).stdout.strip()
            if untracked:
                fail("R1", f"{path} has files that origin/main does not have: {untracked.splitlines()[0][:100]}")

# ---------------------------------------------------------------- AC4
if not os.path.isdir(poc_dir):
    notes.append(f"NOTE: {poc_dir} is absent (CI); AC4 and R5 are skipped")
else:
    def git_poc(*args):
        return subprocess.run(["git", "--no-optional-locks", "-C", poc_dir, *args],
                              capture_output=True, text=True)
    head = git_poc("rev-parse", "HEAD")
    base = git_poc("rev-parse", "poc-baseline^{commit}")
    status = git_poc("status", "--porcelain")
    if head.returncode != 0 or base.returncode != 0 or status.returncode != 0:
        fail("AC4", "git could not read the proof-of-concept repository: "
                    + (head.stderr + base.stderr + status.stderr).strip()[:160])
    else:
        if head.stdout.strip() != base.stdout.strip():
            fail("AC4", f"the proof-of-concept HEAD {head.stdout.strip()[:12]} differs from poc-baseline "
                        f"{base.stdout.strip()[:12]}")
        if status.stdout.strip():
            fail("AC4", "the proof-of-concept repository has changes: " + status.stdout.strip()[:160])

# ---------------------------------------------------------------- R6 (DoD: machine-readable tables)
if overridden:
    notes.append("NOTE: fixture overrides set; cargo wire-check and wire_tables are skipped "
                 "(they target the real repository)")
else:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-6:])
            fail("R6", f"cargo run --bin wire-check did not report checks clean:\n{tail}")
    except (OSError, subprocess.SubprocessError) as error:
        fail("R6", f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        res = re.findall(r"test result: \w+\. (\d+) passed; (\d+) failed", proc.stdout + proc.stderr)
        if not res:
            fail("DoD", "the wire_tables test run printed no 'test result:' line")
        else:
            passed = sum(int(p) for p, _ in res)
            failed = sum(int(f) for _, f in res)
            if failed != 0 or passed < 12:
                fail("DoD", f"wire_tables must report at least 12 passed and 0 failed; got {passed} passed, {failed} failed")
    except (OSError, subprocess.SubprocessError) as error:
        fail("DoD", f"could not run the wire_tables tests: {error}")

# ---------------------------------------------------------------- report
for n in notes:
    print(n)
if fails:
    for label, msg in fails:
        print(f"FAIL {label}: {msg}")
    print(f"FAIL: {len(fails)} assertion(s) failed for 004-23")
    sys.exit(1)
print("OK: 4/4 acceptance criteria for 004-23 and the DoD checks pass")
PY
