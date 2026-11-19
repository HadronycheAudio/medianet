#!/usr/bin/env python3
"""Generate the markdown and look-around corpora (M1-lib, lead ruling R1).

The scripts' own Python functions, taken verbatim from checks/*.sh at
SOURCE_COMMIT (never from a porter's copy), are run by each interpreter
the scripts ran on (python3.10 and python3.13, which must agree) over
  * edge cases written below (fences, headings, tables, rows, numbers),
  * the design documents the ported checks read (DOCS), at SOURCE_COMMIT.
Each result is written in a canonical form (`q` below) that the Rust tests
in hadronyche-tools/src/doccheck/mdproof.rs rebuild from the library's
helpers and compare. For a whole document the line holds the FNV-1a 64
hash of the document text and of the canonical result, so the corpus stays
small; the test fails, naming the regeneration command, on a document
whose text has changed since.

Record: kind TAB id TAB input TAB expected, strings escaped by `esc`.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_markdown_corpus.py markdown > hadronyche-tools/data/markdown_corpus.txt
  python3 hadronyche-tools/tools/gen_markdown_corpus.py lookaround > hadronyche-tools/data/lookaround_corpus.txt
  [--python PY]... as for gen_repertoire.py (default python3.10 and python3.13 on PATH)
"""

import argparse
import json
import re
import shutil
import subprocess
import sys

SOURCE_COMMIT = "b7c71c5"
DOC_PREFIXES = ["doc/design/spec/", "doc/design/tickets/004/", "doc/design/tickets/006/"]
DOC_TOP = "doc/design/"  # plus every .md directly under it
# the look-around proof runs every pattern over every line of these
LOOK_DOCS = ["doc/design/01-architecture.md", "doc/design/02-constants.md",
             "doc/design/spec/timeline.md", "doc/design/spec/wire.md"]


def git(*args):
    return subprocess.run(["git", "--no-optional-locks", *args], capture_output=True, check=True).stdout


def script(name):
    return git("show", f"{SOURCE_COMMIT}:checks/{name}.sh").decode("utf-8")


def func(name, fn):
    """The top-level `def fn(` block of a script, verbatim."""
    s = script(name)
    m = re.search(r"^def " + fn + r"\(.*?(?=^\S)", s, re.S | re.M)
    if not m:
        sys.exit(f"{name}: no def {fn}")
    return m.group(0)


def assign(name, var):
    """The top-level assignment `VAR = ...` of a script, up to balanced brackets."""
    s = script(name)
    m = re.search(r"^" + var + r" = ", s, re.M)
    if not m:
        sys.exit(f"{name}: no {var}")
    out, depth = [], 0
    for line in s[m.start():].splitlines():
        out.append(line)
        depth += sum(line.count(c) for c in "([{") - sum(line.count(c) for c in ")]}")
        if depth <= 0:
            break
    return "\n".join(out) + "\n"


def lookaround_patterns():
    """Every complete pattern literal of checks/*.sh holding a look-around."""
    names = git("ls-tree", "--name-only", f"{SOURCE_COMMIT}:checks").decode().split()
    pats = set()
    for n in names:
        if not n.endswith(".sh"):
            continue
        s = git("show", f"{SOURCE_COMMIT}:checks/{n}").decode("utf-8")
        for m in re.finditer(r'''r(["'])((?:(?!\1).)*\(\?<?[=!](?:(?!\1).)*)\1''', s):
            p = m.group(2).replace("%d", "64").replace("%s", "64")
            try:
                re.compile(p)
            except re.error:
                continue  # a fragment of a longer pattern
            pats.add(p)
    return sorted(pats)


def sources():
    fam = func("004-18", "split_row")
    return {
        "row_family": fam,
        "row_004_14": func("004-14", "split_row"),
        "row_006": func("006-01", "split_row"),
        "tables_a": fam + func("004-09", "parse_tables"),
        "tables_b": fam + func("004-18", "parse_tables"),
        "t006_01": func("006-01", "split_row") + func("006-01", "is_separator")
        + func("006-01", "scan_fences_and_headings") + func("006-01", "parse_tables"),
        "t006_02": func("006-02", "split_row") + func("006-02", "is_separator")
        + func("006-02", "visible_lines") + func("006-02", "parse_tables"),
        "find_006_02": "HEADING = None\n" + func("006-02", "find_section"),
        "heads_a": func("004-09", "heading_index"),
        "heads_b": func("004-18", "heading_index"),
        "heads_c": func("004-20", "heading_index"),
        "fence_004_14": func("004-14", "fence_mask"),
        "caption_004_14": func("004-14", "split_row") + func("004-14", "is_table_line")
        + func("004-14", "caption_above") + func("004-14", "section_tables")
        + func("004-14", "fence_mask"),
        "caption_004_15": func("004-15", "caption_above"),
        "stem_006_01": assign("006-01", "STOPWORDS") + func("006-01", "norm_text")
        + func("006-01", "heading_tokens"),
        "norm_006_01": func("006-01", "norm_text"),
        "norm_006_02": func("006-02", "norm_text"),
        "cell_004_14": assign("004-14", "NUM_RE") + assign("004-14", "DASHES")
        + func("004-14", "to_int") + func("004-14", "first_int") + func("004-14", "label_of")
        + func("004-14", "parse_num"),
    }


ROW_CASES = [
    "| a | b |", "|a|b|", "| a | b \\| c |", "| a \\|", "| a \\\\| b |", "|", "||", "| |", "a | b",
    "  | a | b |  ", "| a | b", "| `x|y` | z |", "|\t a\t|b |", "| a | | c |", "| \\| |", "|a|\\|",
    "| a |b\\|c| d |", "no pipe", "| µs | ² |", "|---|:-:|", "| a || b |", "\\| a | b |",
]

DOC_CASES = [
    "## A\n| h | i |\n|---|---|\n| 1 | 2 |\n",
    "| h | i |\n|---|---|\n",
    "| h | i |\n| 1 | 2 |\n",
    "| h |\n|:-:|\n| a |\n| b \\| c |\n\ntext\n| x |\n| --- |\n| y |\n",
    "```\n| h | i |\n|---|---|\n```\n| a | b |\n|--|--|\n| 1 | 2 |\n",
    "~~~\n## in tilde\n| h |\n|---|\n| 1 |\n~~~\n## after\n",
    "````\n```\n## inner\n````\n## out\n",
    "```python\n## code\n```\n## text\n",
    "```\n## code\n``` trailing\n## still code?\n```\n## out\n",
    "  ```\n## indented fence\n  ```\n## out\n",
    "\t```\n## tab fence\n\t```\n## out\n",
    "~~~~\n~~~\n## inside long tilde\n~~~~\n## out\n",
    "```\n~~~\n## mixed\n~~~\n```\n## out\n",
    "## a\n```\nnever closed\n", "~~~~\n~~~\nstill open\n", "```\n``` tail\nopen\n", "  ````\n```\n",
    "# One #\n## Two ##   \n### Three#\n####### Seven\n#NoSpace\n##  Spaced  title  \n## #\n##\t Tab\n ## Indented\n",
    "## Size budget\n\nThe budget.\n\n**Table 1: default case**\n\n| Item | Bytes |\n|---|---|\n| packet header | 28 |\n| total | 28 |\n\n```\n| fenced | 1 |\n|---|---|\n```\nA caption\n\n\n\n\n\n| far | x |\n|---|---|\n| a | 1 |\n",
    "Caption one\nCaption two\nCaption three\n| a | b |\n| - | - |\n| 1 | 2 |\n",
    "# T\n| a |\n| --- | |\n| 1 |\n| b |\n|-|\n| 2 |\n| c |\n| -- |\n| 3 |\n| d |\n| : |\n",
    "| a | b |\n|---|---|\n| 1 | 2 |\n|---|---|\n| 3 | 4 |\n",
    "## Domain line and position map\ntext\n```\n## Domain line and position map\n```\n### Sub\n## Next\n",
    "## Domain line and position map ##\n### x\n# Top\n",
    "| a |\n|---|\n```\n| b |\n```\n",
    "  | a | b |\n  |---|---|\n  | 1 | 2 |\n",
    "|a|\n|-|\n|b|\n\n|c|\n|:--|\n",
    "",
    "\n\n",
    "## Heading with `code` and [link](#x)\n### Fences of the 006 family\n```` js\n## no\n````\n",
]

NORM_CASES = [
    "", " ", "a  b", " a\tb\n", "a\u00a0b", "a\u2007b\u202fc\u2009d\u200ae", "a\u200bb\ufeffc",
    "x \u2010 \u2011 \u2012 \u2013 \u2014 \u2015 \u2212 y", "\u2018q\u2019 \u201cqq\u201d",
    "10\u207b\u00b3 \u00b5s \u00b2 \u2074\u2075\u2076\u2079 \u2081\u2082\u2083 \u2153 \u2154",
    "a\x1cb\x1dc\x1ed\x1ff", "a\x0bb\x0cc\rd", "C-17 \u2013 C-22", "**bold** `code` __u__",
    "\u00a7 3.5 \u00b1 \u00d7 \u00f7 \u2192 \u2264 \u2265 \u2248 \u2260",
]

STEM_CASES = [
    "Sequence diagrams", "Scenarios", "Regime stamp epoch field", "Open items", "The timing of the stamps",
    "Domain lines and position maps", "Copies and replies", "Settled rules: the 96 kHz case",
    "Lossless glasses", "Encoded framing", "Studies of µs timing²", "ADDRESSING and Routing",
    "a an the of", "Processing, buffering, and caching", "Status (C-123)", "x" * 3 + "ies",
]

CELL_CASES = [
    "28", "1 280", "1\u00a0280", "**42**", "`8`", "—", "–", "-", "none", "Omitted", "n/a", "not sent", "",
    "12 = 4 + 8", "− 4", "-4", "x-4", "deficit 12", "(note 4) 9", "a = b = 7", "*1 472*", "64 (8 B)",
    "123456789012345678901234567890", "1" * 45, "1,280", "5-6", "_x_", "spare: -12", "−12 bytes",
]

NUM_CASES = [
    "1 280 a1 2.5 -7 65 507x 12 345", "21.1% of 21.1²", "about 64 bytes, 64.0 and 64x", "1.25a 1.2.3 .5 5.",
    "-12 Mbit/s +3.5 Mbit/s −4 % +7%", "not fits; it fits. never raises", "a. b; c d.", "x = 12 + 3 = 15",
    "v1 is 64 and 65", "1,234 1 234 12 3456", "µs 4.1 s and 4.1s", "28 + 38 = 66", "`64` and -64-",
    "(64) [64] 64.", "64\u00a0000 and 64 000", "C-64, C-640", "a\nb\n\nc", "'quoted' word 'x'.",
]

WINDOW_CASES = [
    ("µµµ 1280 µµ", 4, 8, 2, 2), ("abc", 0, 1, 80, 80), ("abcdef", 3, 4, 1, 0), ("abc", 3, 3, 0, 5),
]

WORKER = r'''
import json, re, sys, unicodedata
data = json.loads(sys.stdin.read())
src, docs, mode = data["src"], data["docs"], data["mode"]

def esc(s):
    out = []
    for c in s:
        if c == "\\": out.append("\\\\")
        elif c == '"': out.append('\\"')
        elif c == "\n": out.append("\\n")
        elif c == "\t": out.append("\\t")
        elif c == "\r": out.append("\\r")
        elif ord(c) < 0x20 or ord(c) == 0x7f: out.append("\\x{%02x}" % ord(c))
        else: out.append(c)
    return "".join(out)

def q(x):
    if x is None: return "None"
    if isinstance(x, bool): return "True" if x else "False"
    if isinstance(x, int): return str(x)
    if isinstance(x, str): return '"' + esc(x) + '"'
    if isinstance(x, (list, tuple)): return "[" + ", ".join(q(i) for i in x) + "]"
    raise TypeError(type(x))

def fnv(s):
    h = 0xcbf29ce484222325
    for b in s.encode("utf-8"):
        h = ((h ^ b) * 0x100000001b3) & 0xffffffffffffffff
    return "%016x" % h

def ns(key, extra=None):
    g = {"re": re, "unicodedata": unicodedata}
    if extra: g.update(extra)
    exec(src[key], g)
    return g

out = []
def rec(kind, ident, inp, exp):
    out.append([kind, ident, inp, exp])

def doc_lines(text):
    return text.splitlines()

if mode == "markdown":
    fns = {k: ns(k) for k in src if k not in ("caption_004_15", "lookaround")}
    cap15 = ns("caption_004_14")
    exec(src["caption_004_15"], cap15)
    def canon(key, text):
        g = fns.get(key)
        lines = text.splitlines()
        if key in ("tables_a", "tables_b"):
            return q(g["parse_tables"](text))
        if key == "t006_01":
            _, _, vis = g["scan_fences_and_headings"](lines)
            return q(g["parse_tables"](lines, vis))
        if key == "t006_02":
            return q(g["parse_tables"](lines, g["visible_lines"](lines)))
        if key in ("heads_a", "heads_b", "heads_c"):
            return q(g["heading_index"](lines))
        if key == "fence_end_006_01":
            _, errs, _ = fns["t006_01"]["scan_fences_and_headings"](lines)
            return q([e for e in errs if "unclosed code fence" in e])
        if key in ("norm_006_01", "norm_006_02"):
            return q([g["norm_text"](l) for l in lines])
        if key == "fence_004_14":
            return "".join("1" if f else "0" for f in g["fence_mask"](lines))
        if key == "fence_006_01":
            _, _, vis = fns["t006_01"]["scan_fences_and_headings"](lines)
            return "".join("0" if v else "1" for v in vis)
        if key == "fence_006_02":
            return "".join("0" if v else "1" for v in fns["t006_02"]["visible_lines"](lines))
        if key == "heads_006_01":
            heads, _, _ = fns["t006_01"]["scan_fences_and_headings"](lines)
            return q(heads)
        if key == "find_006_02":
            titles = []
            for line in lines:
                m = re.match(r"^(#{1,6})\s+(.*?)\s*$", line)
                if m:
                    t = m.group(2).strip().rstrip("#").strip()
                    if t not in titles: titles.append(t)
            res = []
            for t in titles:
                g["HEADING"] = t
                res.append([t, g["find_section"](lines)])
            return q(res)
        if key in ("caption_004_14", "caption_004_15"):
            h = fns["caption_004_14"] if key == "caption_004_14" else cap15
            fm = h["fence_mask"](lines)
            return q([[t["start"], t["end"], t["header"], t["rows"], t["caption"]] for t in h["section_tables"](lines, fm)])
        raise KeyError(key)
    doc_kinds = ["tables_a", "tables_b", "t006_01", "t006_02", "heads_a", "heads_b", "heads_c", "fence_004_14",
                 "fence_006_01", "fence_006_02", "heads_006_01", "find_006_02", "caption_004_14", "caption_004_15",
                 "norm_006_01", "norm_006_02", "fence_end_006_01"]
    for line in data["rows"]:
        for key in ("row_family", "row_004_14", "row_006"):
            rec(key, "-", esc(line), q(fns[key]["split_row"](line)))
    for i, text in enumerate(data["doc_cases"]):
        for key in doc_kinds:
            rec(key, "case%d" % i, esc(text), canon(key, text))
    for t in data["norms"]:
        for key in ("norm_006_01", "norm_006_02"):
            rec(key, "-", esc(t), q(fns[key]["norm_text"](t)))
    for t in data["stems"]:
        g = fns["stem_006_01"]
        n = g["norm_text"](t).lower()
        rec("stem_006_01", "-", esc(n), q(sorted(g["heading_tokens"](t))))
    for c in data["cells"]:
        g = fns["cell_004_14"]
        try:
            pn = g["parse_num"](c)
        except Exception as e:
            pn = "ERR " + type(e).__name__
        rec("cell_004_14", "-", esc(c), q([g["label_of"](c), pn, g["first_int"](c)]))
    for (t, s, e, b, a) in data["windows"]:
        rec("window", "-", esc("%s\x1f%d\x1f%d\x1f%d\x1f%d" % (t, s, e, b, a)), q(t[max(0, s - b):e + a]))
    for path, text in docs:
        for key in doc_kinds:
            rec("doc:" + key, path, "fnv:" + fnv(text), "fnv:" + fnv(canon(key, text)))
else:
    def matches(p, s):
        res = []
        for m in re.finditer(p, s):
            res.append([m.start(), m.end(), list(m.groups())])
        return q(res)
    for p in data["patterns"]:
        for s in data["nums"]:
            rec("look", esc(p), esc(s), matches(p, s))
        for s in data["nums"][:6]:
            rec("split", esc(p), esc(s), q(re.split(p, s)))
        if re.search(p, "") is not None:
            continue  # a fragment that matches the empty string: edge cases only
        doc_lines_all = [l for _, text in docs for l in text.splitlines()]
        body = "\n".join(matches(p, l) for l in doc_lines_all)
        rec("doc:look", esc(p), "fnv:" + fnv("\n".join(doc_lines_all)), "fnv:" + fnv(body))
print(json.dumps({"version": sys.version.split()[0], "records": out}))
'''


def documents():
    names = git("ls-tree", "-r", "--name-only", SOURCE_COMMIT, "doc/design").decode().split("\n")
    keep = [n for n in names if n.endswith(".md") and (
        any(n.startswith(p) for p in DOC_PREFIXES) or (n.startswith(DOC_TOP) and "/" not in n[len(DOC_TOP):]))]
    out = []
    for n in sorted(keep):
        raw = git("show", f"{SOURCE_COMMIT}:{n}").decode("utf-8")
        out.append([n, raw.replace("\r\n", "\n").replace("\r", "\n")])  # read_text: universal newlines
    return out


def esc(s):
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", choices=["markdown", "lookaround"])
    ap.add_argument("--python", action="append", default=[])
    args = ap.parse_args()
    pythons = []
    for name in args.python or ["python3.10", "python3.13"]:
        path = shutil.which(name)
        if path is None:
            sys.exit(f"interpreter {name!r} not found; pass --python PATH")
        pythons.append(path)
    docs = documents()
    if args.mode == "lookaround":
        docs = [d for d in docs if d[0] in LOOK_DOCS]
    payload = {"mode": args.mode, "docs": docs, "rows": ROW_CASES, "doc_cases": DOC_CASES,
               "stems": STEM_CASES, "norms": NORM_CASES, "cells": CELL_CASES, "windows": WINDOW_CASES, "nums": NUM_CASES,
               "patterns": lookaround_patterns() if args.mode == "lookaround" else [],
               "src": sources()}
    runs = []
    for py in pythons:
        res = subprocess.run([py, "-c", WORKER], input=json.dumps(payload), capture_output=True, text=True)
        if res.returncode != 0:
            sys.exit(f"{py} failed:\n{res.stderr}")
        runs.append(json.loads(res.stdout))
    for r in runs[1:]:
        if r["records"] != runs[0]["records"]:
            diff = [a for a, b in zip(runs[0]["records"], r["records"]) if a != b]
            sys.exit(f"the interpreters disagree on {len(diff)} records, first {diff[:2]}")
    print(f"# {args.mode} corpus: the scripts' own functions at {SOURCE_COMMIT}, run by "
          + ", ".join("Python " + r["version"] for r in runs) + " (identical results)")
    print("# kind\tid\tinput\texpected; generated by hadronyche-tools/tools/gen_markdown_corpus.py")
    for kind, ident, inp, exp in runs[0]["records"]:
        print("\t".join([kind, ident, inp, exp]))


if __name__ == "__main__":
    main()
