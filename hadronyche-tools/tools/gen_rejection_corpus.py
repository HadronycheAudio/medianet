#!/usr/bin/env python3
"""Generate hadronyche-tools/data/rejection_corpus.txt.

The proof of `doccheck::rejection_members` (lead's M1-lib order): every
variant of the closed rejection-reason enumeration, and the reason helpers
around it, run as the scripts ran them, on the real
doc/design/spec/wire.md and on a committed list of edits of it.

Nothing is re-typed: the code is taken from each script at COMMIT (the
Python heredoc of checks/<id>.sh, read with `ast`): its helper functions
(`split_row`, `parse_tables`, `heading_index`, `section_span`, `span_of`,
`section_lines`, `enum_names`, `norm_name`, `named_reasons`), the
module-level statements that read the enumeration (from the first
statement that assigns the anchor name to the last statement before the
end anchor), and the compiled patterns (`NON_REJECTION`, `NONREJ_RE`). The
script's `fail` and `check` are replaced by recorders.

Each interpreter given must produce the same corpus (python3.10 and
python3.13 on PATH by default, or --python PATH, repeatable); the file
records both versions.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_rejection_corpus.py [--python PY]... [--commit C] > hadronyche-tools/data/rejection_corpus.txt
"""

import argparse
import json
import shutil
import subprocess
import sys

DEFAULT_INTERPRETERS = ["python3.10", "python3.13"]

# (variant, script, anchor statement start, end anchor (exclusive) or None
# for "through the statement that contains end_incl"), outputs to record.
RUNNER = r'''
import ast, json, re, sys
job = json.loads(sys.stdin.read())

def esc(s):
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")

def heredoc(src):
    start = src.index("<<'PY'\n") + len("<<'PY'\n")
    return src[start:src.index("\nPY\n", start)]

HELPERS = ["split_row", "parse_tables", "heading_index", "section_span", "span_of", "body",
           "section_lines", "enum_names", "norm_name", "named_reasons", "named_in"]

def functions(tree, code):
    out = {}
    for node in tree.body:
        if isinstance(node, ast.FunctionDef) and node.name in HELPERS:
            out[node.name] = ast.get_source_segment(code, node)
    return out

def block(tree, code, first, last):
    """Source of the module statements from the first whose text starts
    with `first` through the first later one whose text contains `last`."""
    nodes = tree.body
    seg = lambda n: ast.get_source_segment(code, n)
    i = next(k for k, n in enumerate(nodes) if seg(n).startswith(first))
    j = next(k for k in range(i, len(nodes)) if last in seg(nodes[k]))
    return "\n".join(seg(n) for n in nodes[i:j + 1])

def assigned(tree, code, name):
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == name for t in node.targets):
            return ast.get_source_segment(code, node)
    raise KeyError(name)

out = []
def emit(case, variant, key, value):
    out.append(f"{case}\t{variant}\t{key}\t{esc(value)}")

scripts = {}
for sid, src in job["scripts"].items():
    code = heredoc(src)
    tree = ast.parse(code)
    scripts[sid] = (code, tree, functions(tree, code))

HEAD_CACHE = {}

def env(sid, wire, extra=None):
    code, tree, fns = scripts[sid]
    g = {"re": re, "FAILS": []}
    g["fail"] = lambda ac, msg: g["FAILS"].append(str(ac))
    def check(ac, ok, msg):
        if not ok:
            g["FAILS"].append(str(ac))
        return ok
    g["check"] = check
    wire = wire.replace("\r\n", "\n").replace("\r", "\n")   # open() in text mode
    g["wire"] = wire
    g["lines"] = wire.splitlines()
    for f in fns.values():
        exec(f, g)
    # the script's own heading_index, run once per (its source, input):
    # Python 3.10 is slow on its pattern, and the result is a pure function
    key = (fns["heading_index"], wire)
    if key not in HEAD_CACHE:
        HEAD_CACHE[key] = g["heading_index"](g["lines"])
    g["HEADS"] = g["headings"] = HEAD_CACHE[key]
    if extra:
        g.update(extra)
    return g

def dict_items(d):
    return " ".join(f"{k}={v}" for k, v in d.items())

ENUM = job["enum_blocks"]
for case, wire in job["cases"]:
    for variant, (sid, first, last, keys) in ENUM.items():
        code, tree, _ = scripts[sid]
        g = env(sid, wire)
        try:
            for name in job["pre"].get(variant, []):
                exec(assigned(tree, code, name), g)
            exec(block(tree, code, first, last), g)
        except Exception as e:  # recorded: the script would stop here
            emit(case, variant, "raises", type(e).__name__)
            continue
        for key in keys:
            v = g[key]
            if isinstance(v, dict):
                if all(isinstance(x, str) for x in v.values()):
                    for name, cell in v.items():
                        emit(case, variant, key, f"{name}={cell}")
                    emit(case, variant, key + "#", str(len(v)))
                else:
                    emit(case, variant, key, dict_items(v))
            elif isinstance(v, (list, set)):
                emit(case, variant, key, " ".join(sorted(v) if isinstance(v, set) else v))
            elif v is None:
                emit(case, variant, key, "None")
            else:
                emit(case, variant, key, str(v))
        emit(case, variant, "fails", " ".join(g["FAILS"]))

# 004-18's refusal set, from the value of C-185
code, tree, _ = scripts["004-18"]
ref_block = block(tree, code, "_seg = const_val(", "UNION = set(MEMBERS)")
for case, value in job["c185"]:
    g = env("004-18", "", {"const_val": lambda cid, v=value: v, "MEMBERS": {}})
    exec(assigned(tree, code, "DISCOVERY"), g)
    exec(ref_block, g)
    emit(case, "refusal-004-18", "REFUSAL_SET", " ".join(g["REFUSAL_SET"]))
    emit(case, "refusal-004-18", "fails", " ".join(g["FAILS"]))

# named reasons and look-backs over the real members
members = job["members"]
for sid in ["004-16", "004-17"]:
    g = env(sid, "", {"MEMBERS": dict.fromkeys(members, 0)})
    for k, o in enumerate(job["outcomes"]):
        emit(f"o{k}", f"named-{sid}", "found", " ".join(g["named_reasons"](o)))
union = members + job["refusals"]
g = env("004-18", "", {"UNION": set(union)})
for k, o in enumerate(job["outcomes"]):
    emit(f"o{k}", "named-004-18", "found", " ".join(g["named_in"](o)))
# the 25-character look-back of 004-19 and 004-20, written inline in each:
# its pattern is taken from the script text, the slice is the script's
for sid in ["004-19", "004-20"]:
    code = scripts[sid][0]
    pat = re.search(r're\.search\(r"(\\b\(\?:never\|not[^"]*)", before, re\.I\)', code).group(1)
    width = int(re.search(r"before = outcome\[max\(0, m_\.start\(\) - (\d+)\): m_\.start\(\)\]", code).group(1))
    for k, o in enumerate(job["outcomes"]):
        hits = []
        for m_ in re.finditer(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b", o):
            before = o[max(0, m_.start() - width): m_.start()]
            if re.search(pat, before, re.I):
                hits.append(m_.group(0))
        emit(f"o{k}", f"negated-{sid}", "found", " ".join(hits))
        emit(f"o{k}", f"negated-{sid}", "pattern", pat + f" /{width}")
# the in-unit negation guard of 004-06/08/09/10 (inline in each)
for sid in ["004-06", "004-08", "004-09", "004-10"]:
    code = scripts[sid][0]
    m = re.search(r'for m in re\.finditer\(r"(.*?)", unit\)', code) or re.search(r'for m in re\.finditer\(r"(.*?)", u\)', code)
    tok_pat = m.group(1)
    guard = re.search(r'r"(\\b\(\?:no\|not\|never\|without\)\\b\[\^\.\\n\]\{0,30\})" \+ re\.escape\(token\)', code).group(1)
    g_members = set(members)
    for k, o in enumerate(job["outcomes"]):
        toks = set()
        for mm in re.finditer(tok_pat, o):
            token = mm.group(1)
            if token not in g_members and not re.search(guard + re.escape(token), o, re.I):
                toks.add(token)
        emit(f"o{k}", f"guard-{sid}", "found", " ".join(sorted(toks)))
        emit(f"o{k}", f"guard-{sid}", "pattern", tok_pat + " " + guard)
# the non-rejection patterns, compiled from each script's own assignment
for sid, name in job["nonrej"]:
    code, tree, _ = scripts[sid]
    g = {"re": re}
    exec(assigned(tree, code, name), g)
    pat = g[name]
    for k, o in enumerate(job["outcomes"]):
        emit(f"o{k}", f"nonrej-{sid}", "match", "1" if pat.search(o) else "0")
    emit("pattern", f"nonrej-{sid}", "source", pat.pattern)

print(json.dumps({"version": sys.version.split()[0], "lines": out}))
'''

ENUM_BLOCKS = {
    # variant: (script, first statement, last statement contains, keys)
    "enum-004-16": ("004-16", "MEMBERS = {}", 'not 13")', ["MEMBERS"]),
    "enum-004-17": ("004-17", "MEMBERS = {}", 'not 13")', ["MEMBERS"]),
    "enum-004-18": ("004-18", "MEMBERS = {}", 'not 13")', ["MEMBERS"]),
    "enum-004-19": ("004-19", "MEMBERS = {}", 'not 13")', ["MEMBERS"]),
    "enum-004-20": ("004-20", "MEMBERS = {}", 'not 13")', ["MEMBERS", "LAYERS"]),
    "enum-004-24": ("004-24", "REJ_SPAN = span_of", "found {len(MEMBERS)})", ["MEMBERS", "MEMBER_LAYERS"]),
    "enum-004-25": ("004-25", "REJ_FIXED = [", "CONTROL_MEMBERS = {", ["REJ_VALUE", "REJ_LAYERS", "CONTROL_MEMBERS"]),
    "enum-004-28": ("004-28", "REJ_FIXED = [", "CONTROL_MEMBERS = {", ["REJ_VALUE", "REJ_LAYERS", "CONTROL_MEMBERS"]),
    "enum-004-29": ("004-29", "REJ_VALUE = {}", "read {len(REJ_VALUE)})", ["REJ_VALUE", "REJ_LAYERS", "CONTROL_MEMBERS"]),
    "enum-004-26": ("004-26", "_rej = section_lines(", "ALL_REASONS =", ["REJECT", "REFUSAL"]),
    "tolerant-004-06": ("004-06", "MEMBERS = [", "member_set = set(MEMBERS)", ["MEMBERS"]),
    "tolerant-004-08": ("004-08", "MEMBERS = [", "member_set = set(MEMBERS)", ["MEMBERS"]),
    "tolerant-004-09": ("004-09", "MEMBERS = [", "member_set = set(MEMBERS)", ["MEMBERS"]),
    "tolerant-004-10": ("004-10", "MEMBERS = [", "member_set = set(MEMBERS)", ["MEMBERS"]),
    "table-004-11": ("004-11", "enum_names = None", "Closed enumeration)", ["enum_names"]),
}

NONREJ = [("004-16", "NON_REJECTION"), ("004-17", "NON_REJECTION"), ("004-18", "NON_REJECTION"),
          ("004-20", "NON_REJECTION"), ("004-24", "NONREJ_RE"), ("004-25", "NONREJ_RE"),
          ("004-28", "NONREJ_RE"), ("004-29", "NONREJ_RE")]

SCRIPTS = sorted({v[0] for v in ENUM_BLOCKS.values()} | {s for s, _ in NONREJ})


def git(*args):
    return subprocess.run(["git", "--no-optional-locks", *args], capture_output=True, text=True, check=True).stdout


def section_edit(wire, old, new, count=1):
    start = wire.index("## Rejection reasons")
    i = wire.index(old, start)
    return wire[:i] + new + wire[i + len(old):] if count == 1 else wire.replace(old, new)


def row(wire, name):
    """The full line of the member row `name` (with its newline)."""
    start = wire.index("## Rejection reasons")
    i = wire.index(f" | {name} | ", start)
    a = wire.rindex("\n", 0, i) + 1
    b = wire.index("\n", i) + 1
    return wire[a:b]


def excerpt(wire):
    """wire.md up to the end of its Rejection reasons section, then its
    Refusal reasons subsection under a level-2 heading: the edits below are
    applied to this (the whole file is the case "base"), which keeps the
    Rust proof fast."""
    start = wire.index("\n## Rejection reasons\n")
    end = wire.index("\n## ", start + 1)
    r0 = wire.index("\n### Refusal reasons\n")
    r1 = min(wire.index(h, r0 + 1) for h in ("\n### ", "\n## ") if h in wire[r0 + 1:])
    return wire[:end] + "\n\n## Admission messages\n" + wire[r0:r1] + "\n"


def cases(full):
    wire = excerpt(full)
    r = lambda n: row(wire, n)
    out = [("base", full), ("excerpt", wire)]
    def edit(name, old, new):
        out.append((name, section_edit(wire, old, new)))
    def _without(names):
        t = wire
        for n in names:
            t = t.replace(row(wire, n), "", 1)
        return t
    out.append(("delete-1", _without(["UnconfiguredDomain"])))
    out.append(("delete-3-leaves-10", _without(["UnconfiguredDomain", "BadChecksum", "WrongEpoch"])))
    out.append(("delete-4-leaves-9", _without(["UnconfiguredDomain", "BadChecksum", "WrongEpoch", "BadMagic"])))
    edit("duplicate-name-new-value", r("ReservedNonZero"), r("ReservedNonZero") + "| 13 | TooShort | again | control |\n")
    edit("value-tie-11", "| 12 | ReservedNonZero |", "| 11 | ReservedNonZero |")
    edit("values-swapped", "| 0 | TooShort |", "| 1 | TooShort |")
    edit("values-swapped-2", "| 1 | BadMagic |", "| 0 | BadMagic |")
    edit("markup-bold", "| 1 | BadMagic |", "| 1 | **BadMagic** |")
    edit("markup-code", "| 0 | TooShort |", "| 0 | `TooShort` |")
    edit("leading-zeros", "| 7 | BadLength |", "| 007 | BadLength |")
    edit("value-beyond-i128", "| 7 | BadLength |", "| " + "9" * 45 + " | BadLength |")
    edit("value-beyond-i64-on-the-zero-member", "| 0 | TooShort |", "| 99999999999999999999 | TooShort |")
    edit("three-cells", r("BadPacketIndex"), "| 9 | BadPacketIndex | the packet index |\n")
    edit("lowercase-name", "| 8 | BadChecksum |", "| 8 | badChecksum |")
    edit("hyphen-name", "| 8 | BadChecksum |", "| 8 | Bad-Checksum |")
    edit("one-letter-name", "| 8 | BadChecksum |", "| 8 | B |")
    edit("value-with-superscript", "| 8 | BadChecksum |", "| 8² | BadChecksum |")
    edit("value-text", "| 8 | BadChecksum |", "| eight | BadChecksum |")
    edit("tabs-in-cells", "| 3 | ForeignDomain |", "|\t3\t|\tForeignDomain\t|")
    edit("escaped-pipe-in-meaning", "fewer bytes than", "fewer \\| bytes than")
    edit("no-layers-control", "dropped before any field is read | control, sync, media, measurement |",
         "dropped before any field is read | sync, media, measurement |")
    edit("layers-uppercase", "a control message exceeds the 1 252-byte UDP-payload bound (C-17) | control |",
         "a control message exceeds the 1 252-byte UDP-payload bound (C-17) | Control |")
    edit("separator-removed", "|---|---|---|---|\n| 0 | TooShort", "| 0 | TooShort")
    edit("separator-aligned", "|---|---|---|---|\n| 0 | TooShort", "|:--|--:|:-:|---|\n| 0 | TooShort")
    edit("separator-no-dash", "|---|---|---|---|\n| 0 | TooShort", "|:::|:::|:::|:::|\n| 0 | TooShort")
    edit("table-fenced", "| Value (u8) | Name |", "```\n| Value (u8) | Name |")
    edit("extra-row-in-subsection", "### Counting model\n",
         "### Counting model\n\n| V | N | M | L |\n|---|---|---|---|\n| 13 | Extra | x | control |\n\n")
    edit("header-renamed-code", "| Value (u8) | Name |", "| Code | Name |")
    edit("header-name-first", "| Value (u8) | Name |", "| Value | Label |")
    edit("trailing-space-name", "| 5 | UnknownStream |", "| 5 | UnknownStream  |")
    out.append(("heading-level-3", wire.replace("## Rejection reasons\n", "### Rejection reasons\n", 1)))
    out.append(("heading-codes", wire.replace("## Rejection reasons\n", "## Rejection reason codes\n", 1)))
    out.append(("heading-hyphen", wire.replace("## Rejection reasons\n", "## Rejection-reasons\n", 1)))
    out.append(("heading-closing-hashes", wire.replace("## Rejection reasons\n", "## Rejection reasons ##\n", 1)))
    out.append(("heading-upper", wire.replace("## Rejection reasons\n", "## REJECTION REASONS\n", 1)))
    out.append(("heading-tab", wire.replace("## Rejection reasons\n", "##\tRejection\treasons\n", 1)))
    out.append(("heading-seven-hashes", wire.replace("## Rejection reasons\n", "####### Rejection reasons\n", 1)))
    out.append(("heading-fenced", wire.replace("## Rejection reasons\n", "```\n## Rejection reasons\n```\n", 1)))
    first = wire.index("\n## ")
    out.append(("earlier-level-3-match", wire[:first] + "\n### Rejection-reason codes\n\n| V | N | M | L |\n|---|---|---|---|\n| 1 | Early | x | control |\n" + wire[first:]))
    out.append(("section-removed", wire.replace("## Rejection reasons\n", "## Reasons\n", 1)))
    out.append(("crlf", wire.replace("\n", "\r\n")))
    out.append(("lone-cr", wire.replace("\n", "\r")))
    return out


C185 = [
    ("base", None),
    ("duplicate-name", "PortCeiling, PacketRate, PortCeiling, SliceRate"),
    ("parenthetical-kept-camel", "PortCeiling (see PacketRate), SliceRate"),
    ("sentence-break", "PortCeiling, PacketRate. SliceRate, InputCount"),
    ("lowercase", "portCeiling, PacketRate, Slicerate"),
    ("caps-run", "PortCE, PacketR"),
    ("nineteen", None),
    ("no-period-space", "PortCeiling.PacketRate, SliceRate"),
    ("nested-parens", "PortCeiling (a (PacketRate) b) SliceRate"),
]

OUTCOMES = [
    "drop and count under BadMagic",
    "not BadMagic but TooShort",
    "never counted under BadLength; counted under WrongEpoch",
    "neither TooShort nor BadMagic",
    "rather than BadMagic, count UnknownStream",
    "instead of TooShort | BadLength",
    "no BadChecksum. BadLength",
    "not a rejection: TooShort",
    "not, the packet is dropped and the reason is BadMagic",
    "NOT BadMagic",
    "knot BadMagic",
    "no: BadMagic",
    "the reason is BadMagik, not BadMagic",
    "count under FooBar and BazQux",
    "without FooBar the packet is ignored",
    "no reason; never a miss",
    "not a parse rejection",
    "is not counted, cannot occur",
    "valid frames pass; invalid frames count under BadLength",
    "IGNORED",
    "not a refusal: PortCeiling",
    "counted under NameTaken and PortCeiling",
    "not counted",
    "Aa1Bb FooBar2 Foo2Bar",
    "drop; never FooBar. Then FooBar",
    "nor\tBadMagic",
    "x" * 30 + " not " + "y" * 20 + " BadMagic",
    "TooShort, BadMagic and VersionMismatch",
    "",
]


def interpreters(given):
    found = []
    for name in given or DEFAULT_INTERPRETERS:
        path = shutil.which(name)
        if path is None:
            sys.exit(f"interpreter {name!r} not found on PATH; pass --python PATH (one per interpreter)")
        found.append(path)
    return found



def escape_field(s):
    """One record field on one line: backslash, tab, newline and carriage
    return escaped (the Rust reader unescapes all four)."""
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--python", action="append", default=[])
    ap.add_argument("--commit", default="HEAD")
    args = ap.parse_args()
    commit = git("rev-parse", "--short", args.commit).strip()
    wire = git("show", f"{commit}:doc/design/spec/wire.md")
    consts = git("show", f"{commit}:doc/design/02-constants.md")
    c185 = next(l for l in consts.splitlines() if l.startswith("| C-185 |")).split("|")[3].strip()
    c185_cases = []
    for name, v in C185:
        if name == "base":
            v = c185
        elif name == "nineteen":
            v = c185.replace("PortCeiling,", "PortCeiling, ExtraReason,", 1)
        c185_cases.append((name, v))
    job = {
        "scripts": {s: git("show", f"{commit}:checks/{s}.sh") for s in SCRIPTS},
        "cases": cases(wire),
        "enum_blocks": ENUM_BLOCKS,
        "c185": c185_cases,
        "members": ["TooShort", "BadMagic", "VersionMismatch", "ForeignDomain", "UnconfiguredDomain",
                    "UnknownStream", "UnadmittedSource", "BadLength", "BadChecksum", "BadPacketIndex",
                    "WrongEpoch", "OversizeControl", "ReservedNonZero"],
        "refusals": ["PortCeiling", "NameTaken"],
        "outcomes": OUTCOMES,
        "nonrej": NONREJ,
        "pre": {"table-004-11": ["MEMBERS_EXPECTED", "MEMBER_SET"]},
    }
    runs = []
    for py in interpreters(args.python):
        res = subprocess.run([py, "-c", RUNNER], input=json.dumps(job), capture_output=True, text=True)
        if res.returncode != 0:
            sys.exit(f"{py} failed:\n{res.stderr}")
        runs.append(json.loads(res.stdout))
    for r in runs[1:]:
        if r["lines"] != runs[0]["lines"]:
            diff = [a for a, b in zip(r["lines"], runs[0]["lines"]) if a != b][:5]
            sys.exit(f"the interpreters disagree: {diff}")
    print(f"# the rejection-reason enumeration and its helpers, run from the scripts at {commit} (checks/<id>.sh, read with ast)")
    print("# generated by hadronyche-tools/tools/gen_rejection_corpus.py with " +
          ", ".join(f"Python {r['version']}" for r in runs) + "; the interpreters agree on every line")
    print("# case\tvariant\tkey\tvalue (\\\\, \\t, \\n, \\r escaped)")
    for i, (case, wire_text) in enumerate(job["cases"]):
        print(f"# input {case}: {len(wire_text)} chars")
    for k, o in enumerate(OUTCOMES):
        print(f"outcome\to{k}\ttext\t" + escape_field(o))
    for case, v in c185_cases:
        print(f"c185\t{case}\ttext\t" + escape_field(v))
    print("\n".join(runs[0]["lines"]))


if __name__ == "__main__":
    main()
