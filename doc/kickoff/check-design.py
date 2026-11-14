#!/usr/bin/env python3
"""Self-check of the design set: tree, links, anchors, headings, mermaid lint, banned phrases."""
import os, re, sys

ROOT = sys.argv[1]
problems = []

def slug(h):
    s = h.strip().lower()
    s = re.sub(r"[`*_]", "", s)
    s = re.sub(r"[^\w\- ]", "", s, flags=re.UNICODE)
    return s.replace(" ", "-")

files = []
for d, _, fs in os.walk(ROOT):
    for f in fs:
        if f.endswith(".md"):
            files.append(os.path.join(d, f))
files.sort()

anchors = {}
texts = {}
for p in files:
    t = open(p, encoding="utf-8").read()
    texts[p] = t
    a = set()
    in_code = False
    for line in t.splitlines():
        if line.startswith("```"):
            in_code = not in_code
        if not in_code and line.startswith("#"):
            a.add(slug(line.lstrip("#")))
    anchors[p] = a

# 1. tree
top = ["README.md", "00-context.md", "01-architecture.md", "02-constants.md",
       "03-acceptance-criteria.md", "04-definition-of-done.md", "05-risks-and-open.md",
       "06-evidence-appendix.md", "roadmap.md"]
for f in top:
    if not os.path.exists(os.path.join(ROOT, f)):
        problems.append(f"missing top-level file {f}")
extra = [f for f in os.listdir(ROOT) if f not in top + ["adr", "implementation", "tickets"]]
if extra:
    problems.append(f"unexpected entries at root: {extra}")
adr = sorted(os.listdir(os.path.join(ROOT, "adr"))) if os.path.isdir(os.path.join(ROOT, "adr")) else []
imp = sorted(os.listdir(os.path.join(ROOT, "implementation"))) if os.path.isdir(os.path.join(ROOT, "implementation")) else []
if [f[:2] for f in adr] != [f"{i:02d}" for i in range(1, 25)]:
    problems.append(f"adr numbering not 01..24: {[f[:2] for f in adr]}")
if sorted(set(f[:3] for f in imp)) != [f"{i:03d}" for i in range(1, 30)]:
    problems.append(f"implementation numbering does not cover 001..029: {sorted(set(f[:3] for f in imp))}")

# 2. links and anchors
link = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
for p, t in texts.items():
    body = re.sub(r"```.*?```", "", t, flags=re.S)
    for m in link.finditer(body):
        target = m.group(1)
        if target.startswith(("http://", "https://", "mailto:")):
            continue
        path, _, frag = target.partition("#")
        dest = p if path == "" else os.path.normpath(os.path.join(os.path.dirname(p), path))
        if not os.path.exists(dest):
            problems.append(f"{os.path.relpath(p, ROOT)}: broken link {target}")
            continue
        if frag and dest in anchors and frag not in anchors[dest]:
            problems.append(f"{os.path.relpath(p, ROOT)}: missing anchor {target}")

# 3. headings
adr_heads = ["## Status", "## Context", "## Decision", "## Alternatives considered",
             "## Consequences", "## Falsifier", "## Links"]
imp_heads = ["## Goal", "## Background", "## Deliverables", "## Dependencies",
             "## Acceptance criteria", "## Definition of done", "## Open sub-questions"]
dod = ["Tests green in CI, unprivileged", "Every parser touched is fuzzed",
       "Regime stamp fields touched are complete and documented",
       "Design documents updated where behaviour differs, constants table first",
       "No new open item without an owner and a falsifier",
       "The frozen POC is not modified (task 003 is the only exception)",
       "Every new factual claim carries an evidence tag"]
for f in adr:
    t = texts[os.path.join(ROOT, "adr", f)]
    pos = [t.find("\n" + h) for h in adr_heads]
    if -1 in pos or pos != sorted(pos):
        problems.append(f"adr/{f}: headings missing or out of order")
for f in imp:
    t = texts[os.path.join(ROOT, "implementation", f)]
    pos = [t.find("\n" + h) for h in imp_heads]
    if -1 in pos or pos != sorted(pos):
        problems.append(f"implementation/{f}: headings missing or out of order")
    for line in dod:
        if line not in t:
            problems.append(f"implementation/{f}: DoD line missing: {line[:40]}")
    if "**Phase:**" not in t or "**Scope:**" not in t:
        problems.append(f"implementation/{f}: phase/scope line missing")

# 4. banned phrases
banned = [r"\bthe conversation\b", r"\bthis conversation\b", r"\bthe discussion\b", r"\bthe user\b",
          r"\bas discussed\b", r"\bsee above\b", r"\bearlier round\b"]
for p, t in texts.items():
    rel = os.path.relpath(p, ROOT)
    if rel.startswith(("adr", "implementation")):
        for b in banned:
            for m in re.finditer(b, t, flags=re.I):
                line = t.count("\n", 0, m.start()) + 1
                problems.append(f"{rel}:{line}: banned phrase '{m.group(0)}'")

# 5. mermaid lint
for p, t in texts.items():
    rel = os.path.relpath(p, ROOT)
    for m in re.finditer(r"```mermaid\n(.*?)```", t, flags=re.S):
        block = m.group(1)
        for n, line in enumerate(block.splitlines(), 1):
            if line.count('"') % 2:
                problems.append(f"{rel}: mermaid line with odd quote count: {line.strip()[:60]}")
            if "'" in line and '"' not in line:
                problems.append(f"{rel}: mermaid apostrophe outside quotes: {line.strip()[:60]}")
        if block.count("[") != block.count("]") or block.count("{") != block.count("}"):
            problems.append(f"{rel}: mermaid unbalanced brackets")

# 6. C-id and AC-id references resolve
cids = set(re.findall(r"\| (C-\d+) \|", texts[os.path.join(ROOT, "02-constants.md")]))
acids = set(re.findall(r"\| (AC-\d+) \|", texts[os.path.join(ROOT, "03-acceptance-criteria.md")]))
for p, t in texts.items():
    rel = os.path.relpath(p, ROOT)
    for c in set(re.findall(r"\bC-\d+\b", t)):
        if c not in cids:
            problems.append(f"{rel}: unknown constant id {c}")
    for a in set(re.findall(r"\bAC-(?!0)\d+\b(?![.\w-])", t)):
        if a not in acids:
            problems.append(f"{rel}: unknown criterion id {a}")

print(f"{len(files)} markdown files, {len(adr)} ADRs, {len(imp)} tasks, "
      f"{len(cids)} constants, {len(acids)} criteria")
for pr in problems:
    print("PROBLEM:", pr)
print("OK" if not problems else f"{len(problems)} problems")
