#!/usr/bin/env python3
"""Generate hadronyche-tools/data/family004_corpus.txt (M1-lib, the 004 family's
Markdown helpers).

The 004-family scripts each carry their own copy of a dozen Markdown helpers
(heading_index, parse_tables, strip_fences, blocks, prose_blocks, subsections,
sentence splits, name normalisers, width readers) and write their DoD rules
(evidence tags, open items) and their negation look-backs inline. The library
ports each distinct variant once, as a named policy. This generator is the
proof that each library function equals the scripts' own Python: it reads the
scripts at a commit, executes each script's own function definitions (and the
inline rules, copied below verbatim with their script:line), on a corpus of
inputs, and records a hash of every result. The Rust test
`doccheck::proofs004` recomputes every result with the library and compares.

Variants are grouped by AST (docstrings and comments ignored): copies that
differ only in text are one variant, named after the first script holding it.
Each script's copy of a helper is listed under its variant (`# variant` lines),
so a script whose copy differs from its family shows up here, by name.

Inputs: excerpts of the real documents at the commit (the first 40 lines of
sections the four packages read), deterministic line mutations of each excerpt
(stored as edits, so the file stays small), and hand-written edge cases. All
inputs are inside the repertoire (data/repertoire.tsv).

Results are compared as canonical JSON (json.dumps, ensure_ascii=False,
separators=(",", ":"); tuples as lists) hashed with 64-bit FNV-1a, of which
the low 32 bits are written: one `result` line per helper variant, one hash
per input in the order of the `input` lines.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_family004_corpus.py [--python PY]... [--commit C] > hadronyche-tools/data/family004_corpus.txt
The interpreters default to python3.10 and python3.13 on PATH; they must agree.
"""

import argparse
import ast
import json
import random
import re
import shutil
import subprocess
import sys

DEFAULT_COMMIT = "b7c71c5"
DEFAULT_INTERPRETERS = ["python3.10", "python3.13"]
SCRIPTS = ["004-01", "004-03", "004-04", "004-05", "004-06", "004-08", "004-09", "004-10", "004-11",
           "004-12", "004-13", "004-14", "004-15", "004-16", "004-17", "004-18", "004-19", "004-20",
           "004-21", "004-24", "004-25", "004-26", "004-28", "004-29"]
SECTIONS = ["Rejection reasons", "Reserved fields", "Media packet header", "Slice header", "Announcements",
            "Hello", "Election messages", "Admission messages", "Manifest", "Path-MTU probe pair",
            "Regime stamp", "Commissioning window messages", "Sync request and reply", "Receiver report",
            "One-way probe", "Validity rules"]
EXCERPT = 40
MUTATIONS = 8

SYNTH = [
    "",
    "plain",
    "# T #\n```\n# not a heading\n```\n## U\n### V ##\n####### seven\n#nospace\n##\tTab title",
    "  ```\nindented fence\n  ```\n~~~\ntilde is not a fence\n~~~\n```python\ncode\n```",
    "Para one `[code]`.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nPara two untagged with several words here.\n\n| c |\n|---|\n| 3 |",
    "Line one\n   \nLine two after a whitespace-only line\n\t\nLine three",
    "### Sub `[decision]`\nText one two three four five.\n### Two\n| x | y |\n| - | - |\n| 1 | 2 |\n#### Deeper\nmore words here and there\n## Up\nend",
    "| A | B |\n| C | D |\n\n|:|:|\n| E |\n|---|\n\n| Offset | Width | Type | Meaning | Valid range |\n|---|---|---|---|---|\n| 0 | 2 | u16 | Magic | 0x4D4E |\n| 2 | 1 × max 3 | u8 | Count `\\|` x | any |\n| 3 | 16 bits | u16 | x | y |\n| +4 | 2 BYTES | u16 | x | y |",
    "Open item 3: who owns it? Owner: 005. Falsifier: a test.\n\n**Open item 4** has no owner here.\n\n- open items list, owner and falsifier\n\n> Open item 5 owner, falsifier `[memory]`\n\nThe open item mentioned mid-text names the owner but not the other.",
    "First sentence. Second; third: fourth! Fifth? Sixth.  Seventh\nEighth line\n\nNinth paragraph. `[code]` tag after.\n| row | x |\n* bullet\n- dash\n# head",
    "counted under BadMagic, not under VersionMismatch | never TooShort; rather than BadLength instead of OversizeControl\nno ForeignDomain. nor UnconfiguredDomain | neither ReservedNonZero is fine but BadMagik",
    "drop and count under BadMagic, AaBBBB_ x BadMagik — not even BadLength × 2 ≤ 3",
    "The reply's maximum size is 54 bytes; the request's maximum size is 65 507 bytes. 1 252 bytes ≈ 1.2 kB.",
    "To be filled by 004-06.",
    "To be filled\n| but | a table |\n|---|---|",
    "   \n\t\n",
    "## Path-MTU probe pair\n### Path-MTU probe request\nRequest text. `[standard]`\n### Path-MTU probe reply\nReply text.\n## Regime stamp\n### Section types\ntext",
    "| Invalid form | Outcome |\n|---|---|\n| short | drop and count under TooShort |\n| bad magic | count under BadMagic |\n\n| Further rule | Result | Outcome |\n|---|---|---|\n| r1 | x | ignored |\n\n| Case | Notes |\n|---|---|\n| c | n |",
    "bits 7..6 | bit 3 | Bits 9..2 | bit 8 | BIT 0 | bits 3 .. 5 | bit12 | 2bit 7 | 99999999999999999999999999999999999999999 bit 1\n1 2_3 4 | _5 | x 0 y",
    "| C-06 | Fallback | 1 280 bytes | decided |\n| C-17 | Max | 0x1F, 1,000 | x |\n| C-13 | n | 12 34 | y |\n| x | 0x_1f | +5 | -0x2 | 0xg | 1_000 | 1__0 | ٣ |\n| C-1 | short |\n| C-02 |\n| Magic number | Protocol version | Domain id | Message type | Authentication scheme | Counter | Authentication tag | 2nd thing | x9 y |\n| `**Magic**` | > tag |",
    "Width cells: 2 | 2 bytes | 16 bits | 6 bits 2 bytes | 1 byte | bits | 1 2 | 1_000 bits | 4 × max 12 | 3 BITS",
    "Name_With-Mixed  Case\tand_underscores\n  leading and trailing  \nµs² Δ — dash – en",
    "Text with `code [decision]` span and [CODE] upper-case tag.\n\n[measured] first word tag only\n\nfive words without any tag",
    "**Purpose:** a paragraph.\n\n* item one is a list item with enough words\n* item two `[code]`\n\n```\n| fenced | table |\n|---|---|\n```\n\n| real | table |\n|---|---|",
    "Open item 7: the owners decide; falsifiers listed.\n\n**Open item 6** coowner, co-owner, falsifier.\n\n_Open_ `item` 5 owner falsifier\n\nA long lead sentence of more than forty characters, then open item, owner, falsifier.\n\n(Open item 2) owner and falsifiers\n\nOpen items: none. Open itemize owner falsifier.",
    "Open item 1: no owner yet.\n\nNext block names the owner.\n\nAnd the falsifier here.\n\n## Open item heading\n\n| open item | owner | falsifier |\n|---|---|---|",
    "Untagged prose block one two three.\n\n```\ncode after untagged\n```\n\n## Only a heading\n\n| table | after heading |\n|---|---|\n\nTagged `[memory]`.\n   \n| inherits | across a spaces line |\n\n#### H\nbody under heading untagged",
    "# heading with five words here\n| table row with five words |\n- a - b - c - d\none-two three-four five-six seven\n  four words only here\nfive words in this line\n`*_` ** __ x y z w v",
    "First. Second\n \nThird\n\n\nFourth.\n| row\n* star\n# hash\n- dash\n[code] continues\n`tick` continues. Next one; and: colon. End?",
    "x not TooShort | abcdefghijklmnopqrstuvwxy BadMagic | 123456789012345678901 no BadLength\nnorm BadMagic, No TooShort, Never BadMagic, rather  than TooShort, instead of TooShort\n× × × × × × × × × × × not × BadMagic; not; BadMagic\nNever VersionMismatch | not-UnconfiguredDomain | nor_BadMagic",
    "| Condition | Outcome / notes |\n|---|---|\n|  | empty first cell |\n| c1 | o1 |\n\n| Rule | Result |\n|---|---|\n| r | x |\n\n| Kind | Invalid input | Outcome |\n|---|---|---|\n| k | i | o |",
    "## Hello\ntext\n### Hello reply\n#### probe deep\n### Section types\n## Path-MTU probe pair\n### Rejection\n## Hello\nsecond",
    "Open item 7: the owners decide; falsifier listed.\n\nOpen item 8: owner named; falsifiers listed.\n\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx open item owner falsifier\n\nyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy open item owner falsifier\n\n**zzzzzzzzzzzzzzzzzzzzzzzzzz open** item owner falsifier",
    "x | not BadMagic . TooShort | rather than; BadLength\nnot BadMagic | VersionMismatch\n| a | b |\n|---|---|\n| c\n- dash after newline\n#x hash\nLine one. # not a heading\n| pipe line with five words here\n## heading line with five words",
]

FNV_OFFSET = 0xcbf29ce484222325
FNV_PRIME = 0x100000001b3


def fnv(text):
    h = FNV_OFFSET
    for b in text.encode("utf-8"):
        h ^= b
        h = (h * FNV_PRIME) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def canon(obj):
    return json.dumps(obj, ensure_ascii=False, separators=(",", ":"))


def git_show(commit, path):
    return subprocess.run(["git", "--no-optional-locks", "show", f"{commit}:{path}"],
                          capture_output=True, check=True).stdout.decode("utf-8")


# --- the scripts' own definitions -------------------------------------------

def top_defs(src):
    """{name: source} of the top-level `def`s of a script (first one wins)."""
    lines = src.split("\n")
    out, i = {}, 0
    while i < len(lines):
        m = re.match(r"^def (\w+)\(", lines[i])
        if m:
            j = i + 1
            while j < len(lines) and (lines[j] == "" or lines[j][0] in " \t"):
                j += 1
            out.setdefault(m.group(1), "\n".join(lines[i:j]).rstrip())
            i = j
        else:
            i += 1
    return out


def top_consts(src):
    """Top-level `NAME = <compile or literal>` statements, multi-line."""
    lines = src.split("\n")
    out, i = [], 0
    while i < len(lines):
        m = re.match(r"^([A-Z][A-Z0-9_]*)\s*=\s*(re\.compile\(|[rbuf]?[\"'(\[{]|\d)", lines[i])
        if m:
            chunk, depth, j = [lines[i]], 0, i + 1
            depth = sum(lines[i].count(c) for c in "([{") - sum(lines[i].count(c) for c in ")]}")
            while depth > 0 and j < len(lines):
                chunk.append(lines[j])
                depth += sum(lines[j].count(c) for c in "([{") - sum(lines[j].count(c) for c in ")]}")
                j += 1
            out.append("\n".join(chunk))
            i = j
        else:
            i += 1
    return out


def ast_key(src):
    t = ast.parse(src)
    f = t.body[0]
    if f.body and isinstance(f.body[0], ast.Expr) and isinstance(getattr(f.body[0], "value", None), ast.Constant) \
            and isinstance(f.body[0].value.value, str):
        f.body = f.body[1:]
    return ast.dump(t, annotate_fields=False)


def namespace(src):
    ns = {"re": re, "sys": sys}
    for _ in range(2):
        for c in top_consts(src):
            try:
                exec(c, ns)
            except Exception:
                pass
        for d in top_defs(src).values():
            try:
                exec(d, ns)
            except Exception:
                pass
    return ns


# --- inputs -------------------------------------------------------------------

CIDS = ["C-01", "C-06", "C-17", "C-100", "C-13", "C-999", "C-1", "C-02"]
NUM_16 = r"(?:\d{1,3}(?:[ \u00a0]\d{3})+|\d+)"
CINT = [("C-06", r"(" + NUM_16 + r")\s*bytes"), ("C-17", r"(" + NUM_16 + r")"), ("C-01", r"(\d[\d ]*)\s*hz"),
        ("C-02", r"(\d+)\s*frames"), ("C-999", r"(\d+)"), ("C-13", r"([0-9 ]+)"), ("C-01", r"(" + NUM_16 + r") Hz")]


def const_excerpts(consts):
    rows = [l for l in consts.splitlines() if l.startswith("| C-")]
    return [("constants rows 1-40", "\n".join(rows[:40])), ("constants rows 80-120", "\n".join(rows[80:120]))]


def per_item(f, items):
    out = []
    for it in items:
        try:
            out.append(f(it))
        except ValueError as e:
            out.append("raises:ValueError")
    return out


def const_cells(ns, t, var):
    ns[var] = t
    f = ns["const_val"] if "const_val" in ns else ns["const_row"]
    return [f(c) for c in CIDS]


def const_ints(ns, t, var):
    ns[var] = t
    return per_item(lambda ci: ns["const_int"](*ci), CINT)


def excerpts(wire):
    lines = wire.splitlines()
    out = []
    for name in SECTIONS:
        idx = next((k for k, l in enumerate(lines) if l == f"## {name}"), None)
        if idx is not None:
            out.append((name, "\n".join(lines[idx:idx + EXCERPT])))
    return out


TAG_RE = r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]"


def mutations(text, seed):
    """Deterministic line edits: (op, k, text) with op ins, del or rep."""
    rnd = random.Random(seed)
    lines = text.split("\n")
    ops = []
    for _ in range(MUTATIONS):
        k = rnd.randrange(len(lines))
        kind = rnd.choice(["blankws", "tilde", "fence", "seven", "closing", "deltag", "open", "neg",
                           "del", "camel", "indent"])
        line = lines[k]
        if kind == "blankws":
            ops.append(["ins", k, rnd.choice(["   ", "\t", " "])])
        elif kind == "tilde":
            ops.append(["ins", k, "~~~"])
        elif kind == "fence":
            ops.append(["ins", k, rnd.choice(["```", "  ```", "```text"])])
        elif kind == "seven":
            ops.append(["ins", k, rnd.choice(["####### Seven", "#### Four ##", "##\tTabbed", "#x"])])
        elif kind == "closing" and line.startswith("#"):
            ops.append(["rep", k, line + " ##"])
        elif kind == "deltag":
            ops.append(["rep", k, re.sub(TAG_RE, "", line, flags=re.I)])
        elif kind == "open":
            ops.append(["ins", k, rnd.choice(["Open item 9: no owner here.", "**Open item 8** owner 005, falsifier x.",
                                              "An open item, owner and falsifier named.", "open items are listed"])])
        elif kind == "neg":
            ops.append(["rep", k, re.sub(r"\b([A-Z][a-z]+[A-Z])", r"not \1", line, count=1)])
        elif kind == "del" and len(lines) > 2:
            ops.append(["del", k])
        elif kind == "camel":
            ops.append(["rep", k, line + rnd.choice([" never BadMagic.", " rather than TooShort", "; under BadLength",
                                                     " | no VersionMismatch | x"])])
        elif kind == "indent":
            ops.append(["rep", k, "  " + line])
        else:
            continue
        lines = apply_op(lines, ops[-1])
    return ops


def apply_op(lines, op):
    lines = list(lines)
    if op[0] == "ins":
        lines.insert(op[1], op[2])
    elif op[0] == "del":
        del lines[op[1]]
    else:
        lines[op[1]] = op[2]
    return lines


# --- the inline rules, verbatim ----------------------------------------------

PATTERNS = ["probe", "^Hello$", "reply", "section types?", "Open items", "nothing-matches", "Rejection", "^Path-MTU"]


def r_every_block(ns, section):
    # checks/004-06.sh:1243 (004-03, 05, 06, 08, 09, 10, 11, 13)
    out = []
    prev_tagged = True
    for block in ns["blocks"](section):
        stripped = block.strip()
        if not stripped:
            continue
        content_lines = [
            line for line in stripped.splitlines() if not line.lstrip().startswith("#")
        ]
        if not content_lines:
            prev_tagged = True
            continue
        body = "\n".join(content_lines).strip()
        is_table = all(line.strip().startswith("|") for line in content_lines if line.strip())
        is_code = content_lines[0].lstrip().startswith("```")
        if ns["has_evidence"](body):
            prev_tagged = True
            continue
        if (is_table or is_code) and prev_tagged:
            prev_tagged = True
            continue
        out.append(body)
        prev_tagged = False
    return out


def r_lines_004_24(ns, STAMP):
    # checks/004-24.sh:1103
    _untagged = []
    for _ln in ns["strip_fences"](STAMP).splitlines():
        _t = _ln.strip()
        if not _t or _t.startswith("#") or _t.startswith("|"):
            continue
        if len(re.sub(r"[`*_|>#-]", " ", _t).split()) >= 5 and not ns["has_evidence"](_t):
            _untagged.append(_t)
    return _untagged


def r_prose_split_004_25(ns, text):
    # checks/004-25.sh:1054 (004-25, 004-28:1206, 004-29:988)
    return [x for k, x in ns["prose_blocks"](text) if k == "p"
            and len(re.sub(r"[`*_|>#-]", " ", x).split()) >= 5 and not ns["has_evidence"](x)]


def r_prose_nonspace_004_26(ns, MAN_TEXT):
    # checks/004-26.sh:1006
    out = []
    for block_kind, text in ns["prose_blocks"](MAN_TEXT):
        if block_kind != "p":
            continue
        words = len(re.findall(r"\S+", text))
        if words >= 5:
            if not ns["has_evidence"](text):
                out.append(text)
    return out


def r_subs_table_004_26(ns, MAN):
    # checks/004-26.sh:1011
    out = []
    for title, body in ns["subsections"](MAN.splitlines()):
        if any(k == "t" for k, _ in ns["prose_blocks"](body)):
            if not ns["has_evidence"](body):
                out.append(title)
    return out


def r_subs_any_004_24(ns, STAMP):
    # checks/004-24.sh:1114 (STAMP_SUBS = subsections(STAMP_LINES))
    return [t for t, x in ns["subsections"](STAMP.splitlines()) if not ns["has_evidence"](x)]


def r_prose_alpha_004_16(ns, text):
    # checks/004-16.sh:1311 (004-17:746, 004-18:1178), one section text
    out = []
    has_tag_here = ns["has_evidence"](text)
    for kind, body in ns["prose_blocks"](text):
        if kind == "p":
            words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", body)))
            if words >= 5 and not ns["has_evidence"](body):
                out.append(["prose", body])
        elif kind == "t" and not has_tag_here:
            out.append(["table", body])
    return out


def r_blocks_alpha_004_19(ns, section):
    # checks/004-19.sh:1295 (004-20:1471)
    out = []
    last_prose_tagged = True
    for b in ns["section_blocks"](section.splitlines()):
        kind = ns["block_kind"](b)
        text = "\n".join(b)
        if kind == "heading" or kind == "code":
            continue
        if kind == "table":
            if not (ns["has_evidence"](text) or last_prose_tagged):
                out.append(["table", text])
            continue
        words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", text)))
        tagged = ns["has_evidence"](text)
        if words >= 5 and not tagged:
            out.append(["prose", text])
        last_prose_tagged = tagged or words < 5
    return out


def r_open_mention_004_24(ns, STAMP):
    # checks/004-24.sh:1117 (004-25:1058, 004-28:1213)
    _open = [t for k, t in ns["prose_blocks"](STAMP) if re.search(r"open item", t, re.I)]
    return [[t, bool(re.search(r"owner", t, re.I) and re.search(r"falsifier", t, re.I))] for t in _open]


def r_open_leading_004_29(ns, NEW):
    # checks/004-29.sh:992
    OPEN_PARAS = [x for k, x in ns["prose_blocks"](NEW) if k == "p" and re.match(r"\s*\**\s*Open item", x)]
    return [[x, bool(re.search(r"owner", x, re.I) and re.search(r"falsifier", x, re.I))] for x in OPEN_PARAS]


def r_open_head40_004_26(ns, MAN_TEXT):
    # checks/004-26.sh:896 and :1015
    PROSE = [b for b in ns["prose_blocks"](MAN_TEXT) if b[0] == "p"]
    OPEN_PARAS = [b[1] for b in PROSE if re.search(r"open item", re.sub(r"[*_`]", "", b[1]).strip()[:40], re.I)]
    return [[p, bool(re.search(r"\bowner\b", p, re.I) and re.search(r"\bfalsifier\b", p, re.I))] for p in OPEN_PARAS]


def r_open_clean_004_16(ns, text):
    # checks/004-16.sh:1318 (004-17:753, 004-18:1185)
    out = []
    for kind, body in ns["prose_blocks"](text):
        if kind == "p" and re.match(r"\s*\*{0,2}\s*open\s+items?\b", ns["clean_start"](body), re.I):
            out.append([body, bool(re.search(r"owner", body, re.I) and re.search(r"falsifier", body, re.I))])
    return out


def r_open_window3_004_19(ns, section):
    # checks/004-19.sh:1313
    flat = [("\n".join(b)) for b in ns["section_blocks"](section.splitlines())]
    out = []
    for i, text in enumerate(flat):
        if re.search(r"open\s+item", text, re.I):
            window = "\n".join(flat[i: i + 3])
            out.append([text, bool(re.search(r"owner", window, re.I) and re.search(r"falsifier", window, re.I))])
    return out


def r_open_wleading_004_20(ns, text):
    # checks/004-20.sh:1486 (Sec.prose, :745)
    blocks = ns["section_blocks"](text.splitlines())
    prose = ["\n".join(b) for b in blocks if ns["block_kind"](b) == "prose"]
    out = []
    for b in prose:
        if re.match(r"\W*open\s+items?\b", b.strip(), re.I):
            out.append([b, bool(re.search(r"\bowner\b", b, re.I) and re.search(r"falsifier", b, re.I))])
    return out


def r_form_rows_004_20(ns, text):
    # checks/004-20.sh:775 (Sec.form_rows) with FORM_KEYS (:1224) and norm
    FORM_KEYS = ("form", "invalid", "rule", "case", "condition")
    rows_out = []
    for _, header, rows in ns["parse_tables"](text):
        heads = [ns["norm"](h) for h in header]
        if len(heads) >= 2 and any(k in heads[0] for k in FORM_KEYS) and any("outcome" in h for h in heads):
            rows_out.extend(r for r in rows if len(r) >= 2 and r[0].strip())
    return rows_out


def r_lookback_24_004_16(ns, text):
    # checks/004-16.sh:1223 named_reasons, the look-back alone (every word)
    out = []
    for line in text.splitlines():
        for m in re.finditer(r"\b[A-Za-z]+\b", line):
            before = line[max(0, m.start() - 24):m.start()]
            out.append([m.start(), bool(re.search(r"\b(not|never|nor|no|neither)\b[^|;.]*$", before, re.I))])
    return out


def r_lookback_25_004_19(ns, text):
    # checks/004-19.sh:1102
    out = []
    for line in text.splitlines():
        for m_ in re.finditer(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b", line):
            before = line[max(0, m_.start() - 25): m_.start()]
            out.append([m_.group(0), bool(re.search(r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$", before, re.I))])
    return out


def r_preceding_004_16(ns, text):
    # checks/004-16.sh:1233 (004-17, 004-18:370 CAMEL)
    out = []
    for line in text.splitlines():
        for name in ns["CAMEL"].findall(line):
            out.append([name, bool(re.search(r"\b(not|never|nor)\b[^|;.]*" + name, line))])
    return out


def r_placeholder_004_06(ns, section):
    # checks/004-06.sh:573 (004-08, 09, 10)
    return bool(not section.strip() or (ns["PLACEHOLDER"].search(section) and "|" not in section))


def per_line(f):
    return lambda ns, text: [f(ns, l) for l in text.splitlines()]


def cells(ns, text):
    out = []
    for _, h, rows in ns["parse_tables"](text):
        out.extend(h)
        for r in rows:
            out.extend(r)
    return out


def spans_16(ns, text):
    lines = text.splitlines()
    ns["lines"], ns["HEADS"] = lines, ns["heading_index"](lines)
    return [[p, lvl, ns["section_span"](p, lvl)] for p in PATTERNS for lvl in (None, 2, 3)]


def spans_19(ns, text):
    lines = text.splitlines()
    ns["lines"], ns["headings"] = lines, ns["heading_index"](lines)
    return [[p, ns["section_span"](p)] for p in PATTERNS]


def spans_24(ns, text):
    lines = text.splitlines()
    ns["lines"], ns["HEADS"] = lines, ns["heading_index"](lines)
    return [[p, lvl, ns["span_of"](p, lvl)] for p in PATTERNS for lvl in (2, 3)]


def prose_sentences_09(ns, text):
    ns["blks"] = ns["blocks"](ns["strip_fences"](text))
    return list(ns["prose_sentences"]())


def sent_004_24(ns, text):
    # checks/004-24.sh SENT, on the text itself
    return [s.strip().lower() for s in re.split(r"(?<=[.;:])\s+", text) if s.strip()]


def section_blocks_kinds(ns, text):
    return [[ns["block_kind"](b), b] for b in ns["section_blocks"](text.splitlines())]


# (helper, variant script, function of (namespace, text)); `def` helpers name the
# script function; the AST variants of each are checked below
HELPERS = [
    ("heading_index", "004-06", lambda ns, t: ns["heading_index"](t.splitlines())),
    ("parse_tables", "004-06", lambda ns, t: ns["parse_tables"](t)),
    ("parse_tables", "004-16", lambda ns, t: ns["parse_tables"](t)),
    ("strip_fences", "004-06", lambda ns, t: ns["strip_fences"](t)),
    ("strip_fences", "004-16", lambda ns, t: ns["strip_fences"](t)),
    ("blocks", "004-06", lambda ns, t: ns["blocks"](t)),
    ("prose_blocks", "004-16", lambda ns, t: ns["prose_blocks"](t)),
    ("clean_start", "004-16", per_line(lambda ns, l: ns["clean_start"](l))),
    ("subsections", "004-16", lambda ns, t: ns["subsections"](t.splitlines())),
    ("section_span", "004-16", spans_16),
    ("section_span", "004-19", spans_19),
    ("span_of", "004-24", spans_24),
    ("norm_name", "004-06", per_line(lambda ns, l: ns["norm_name"](l))),
    ("norm_name", "004-16", per_line(lambda ns, l: ns["norm_name"](l))),
    ("norm", "004-20", per_line(lambda ns, l: ns["norm"](l))),
    ("norm", "004-24", per_line(lambda ns, l: ns["norm"](l))),
    ("width_to_bits", "004-06", lambda ns, t: [ns["width_to_bits"](c) for c in cells(ns, t)]),
    ("width_to_bits", "004-20", lambda ns, t: [ns["width_to_bits"](c) for c in cells(ns, t)]),
    ("const_val", "004-16", lambda ns, t: const_cells(ns, t, "CONST_TEXT")),
    ("const_row", "004-25", lambda ns, t: const_cells(ns, t, "CONST")),
    ("const_row_line", "004-19", lambda ns, t: const_cells(ns, t, "CONST_TEXT")),
    ("const_int", "004-16", lambda ns, t: const_ints(ns, t, "CONST_TEXT")),
    ("const_int", "004-28", lambda ns, t: const_ints(ns, t, "CONST")),
    ("to_int", "004-16", lambda ns, t: per_item(ns["to_int"], cells(ns, t))),
    ("to_int", "004-26", lambda ns, t: per_item(ns["to_int"], cells(ns, t))),
    ("md_name", "004-16", lambda ns, t: [ns["md_name"](c) for c in cells(ns, t)]),
    ("key_of", "004-16", lambda ns, t: [ns["key_of"](c) for c in cells(ns, t)]),
    ("tname", "004-17", per_line(lambda ns, l: ns["tname"](l))),
    ("parse_int_token", "004-06", per_line(lambda ns, l: ns["parse_int_token"](l))),
    ("bit_positions", "004-06", per_line(lambda ns, l: ns["bit_positions"](l))),
    ("sentences", "004-20", lambda ns, t: ns["sentences"](t)),
    ("sentences", "004-25", lambda ns, t: ns["sentences"](t)),
    ("sentences_of", "004-29", lambda ns, t: ns["sentences_of"](t)),
    ("sents", "004-12", lambda ns, t: ns["sents"](t)),
    ("split_sentences", "004-04", per_line(lambda ns, l: ns["split_sentences"](l))),
    ("prose_sentences", "004-09", prose_sentences_09),
    ("SENT", "004-24", sent_004_24),
    # the inline splits (script:line of one use each)
    ("split_raw", "004-09", lambda ns, t: re.split(r"(?<=[.;])\s+", t)),  # 004-09:1606
    ("split_raw_colon", "004-26", lambda ns, t: re.split(r"(?<=[.;:])\s+", t)),  # 004-26:875
    ("split_raw_colon_newlines", "004-19", lambda ns, t: re.split(r"(?<=[.;:])\s+|\n+", t)),  # 004-19:845
    ("split_raw_newline", "004-11", lambda ns, t: re.split(r"(?<=[.;])\s+|\n", t)),  # 004-11:700
    ("split_raw_pipe", "004-21", lambda ns, t: re.split(r"(?<=[.;])\s+|\|", t)),  # 004-21:946
    ("split_raw_terminal", "004-03", lambda ns, t: re.split(r"(?<=[.!?])\s+(?=[A-Z`(\[])", t)),  # 004-03:1037
    ("first_sentence", "004-29", per_line(lambda ns, l: re.split(r"(?<=\.)\s", l, 1)[0])),  # 004-29:940
    ("form_rows", "004-17", lambda ns, t: ns["form_rows"](t)),
    ("Sec.form_rows", "004-20", r_form_rows_004_20),
    ("section_blocks", "004-19", section_blocks_kinds),
    ("is_placeholder", "004-20", lambda ns, t: ns["is_placeholder"](t)),
    ("placeholder", "004-06", r_placeholder_004_06),
    ("every_block", "004-06", r_every_block),
    ("lines_untagged", "004-24", r_lines_004_24),
    ("prose_split", "004-25", r_prose_split_004_25),
    ("prose_nonspace", "004-26", r_prose_nonspace_004_26),
    ("prose_alpha", "004-16", r_prose_alpha_004_16),
    ("blocks_alpha", "004-19", r_blocks_alpha_004_19),
    ("subs_with_table", "004-26", r_subs_table_004_26),
    ("subs_any", "004-24", r_subs_any_004_24),
    ("open_mention_blocks", "004-24", r_open_mention_004_24),
    ("open_leading", "004-29", r_open_leading_004_29),
    ("open_head40", "004-26", r_open_head40_004_26),
    ("open_clean", "004-16", r_open_clean_004_16),
    ("open_window3", "004-19", r_open_window3_004_19),
    ("open_wleading", "004-20", r_open_wleading_004_20),
    ("lookback_24", "004-16", r_lookback_24_004_16),
    ("lookback_25", "004-19", r_lookback_25_004_19),
    ("preceding", "004-16", r_preceding_004_16),
]

DEF_HELPERS = ["heading_index", "parse_tables", "split_row", "strip_fences", "blocks", "has_evidence",
               "prose_blocks", "clean_start", "subsections", "find_sub", "first_table", "section_span", "span_of",
               "body", "norm_name", "norm", "width_to_bits", "is_field_table", "width_of", "fields_of",
               "sentences", "sentences_of", "form_rows", "section_blocks", "block_kind", "is_placeholder"]


def worker(commit):
    srcs = {s: git_show(commit, f"checks/{s}.sh") for s in SCRIPTS}
    nss = {s: namespace(src) for s, src in srcs.items()}
    out = {"version": sys.version.split()[0], "variants": [], "inputs": [], "results": []}
    # which script's copy of each shared helper is which variant
    for name in DEF_HELPERS:
        groups = {}
        for s in SCRIPTS:
            d = top_defs(srcs[s])
            if name in d:
                groups.setdefault(ast_key(d[name]), []).append(s)
        for members in groups.values():
            out["variants"].append([name, members[0], members])
    wire = git_show(commit, "doc/design/spec/wire.md")
    inputs = []
    for k, (name, text) in enumerate(excerpts(wire)):
        base = f"x{k:02}"
        inputs.append([base, "excerpt", name, text])
        lines = text.split("\n")
        for i, op in enumerate(mutations(text, 1000 + k)):
            lines = apply_op(lines, op)
            inputs.append([f"{base}m{i}", "mutation", base, op, "\n".join(lines)])
    for k, (name, text) in enumerate(const_excerpts(git_show(commit, "doc/design/02-constants.md"))):
        inputs.append([f"c{k:02}", "excerpt", name, text])
    for k, text in enumerate(SYNTH):
        inputs.append([f"s{k:02}", "synthetic", text])
    for inp in inputs:
        text = inp[-1]
        out["inputs"].append(inp[:-1] if inp[1] == "mutation" else inp)
        for helper, variant, f in HELPERS:
            try:
                res = canon(f(nss[variant], text))
            except Exception as e:  # a script function that raises: recorded as such
                res = canon({"raises": type(e).__name__})
                print(f"note: {helper}@{variant} raises {type(e).__name__} on {inp[0]}", file=sys.stderr)
            out["results"].append([inp[0], helper, variant, fnv(res)])
    print(json.dumps(out, ensure_ascii=False))


def interpreters(given):
    found = []
    for name in given or DEFAULT_INTERPRETERS:
        path = shutil.which(name)
        if path is None:
            sys.exit(f"interpreter {name!r} not found on PATH; pass --python PATH (one per interpreter)")
        found.append(path)
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--python", action="append", default=[])
    ap.add_argument("--commit", default=DEFAULT_COMMIT)
    ap.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    args = ap.parse_args()
    if args.worker:
        worker(args.commit)
        return
    runs = []
    for py in interpreters(args.python):
        res = subprocess.run([py, __file__, "--worker", "--commit", args.commit], capture_output=True, text=True)
        if res.returncode != 0:
            sys.exit(f"{py} failed:\n{res.stderr}")
        runs.append(json.loads(res.stdout))
    for r in runs[1:]:
        if r["results"] != runs[0]["results"] or r["variants"] != runs[0]["variants"]:
            diff = [a for a, b in zip(runs[0]["results"], r["results"]) if a != b]
            sys.exit(f"the interpreters disagree on {diff[:10]}: record both and decide")
    run = runs[0]
    print(f"# family004 corpus: the 004 scripts' Markdown helpers and inline rules at {args.commit}")
    print("# generated by hadronyche-tools/tools/gen_family004_corpus.py with "
          + ", ".join(f"Python {r['version']}" for r in runs) + "; the interpreters agree on every result")
    print("# variant <helper> <variant> <scripts holding this AST variant>")
    for name, first, members in run["variants"]:
        print(f"variant {name} {first} {' '.join(members)}")
    print("# input <id> excerpt <section> <json text> | input <id> mutation <base> <json op>"
          " (applied to the previous input: the base or the base's previous mutation) | input <id> synthetic <json text>")
    for inp in run["inputs"]:
        if inp[1] == "excerpt":
            print(f"input {inp[0]} excerpt {canon(inp[2])} {canon(inp[3])}")
        elif inp[1] == "mutation":
            print(f"input {inp[0]} mutation {inp[2]} {canon(inp[3])}")
        else:
            print(f"input {inp[0]} synthetic {canon(inp[2])}")
    print("# result <helper> <variant> <low 32 bits of the FNV-1a 64 hash of each input's canonical JSON result, in input order>")
    by = {}
    for inp, helper, variant, h in run["results"]:
        by.setdefault((helper, variant), []).append(h[-8:])
    for (helper, variant), hs in by.items():
        print(f"result {helper} {variant} " + " ".join(hs))


if __name__ == "__main__":
    main()
