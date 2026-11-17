#!/usr/bin/env python3
"""Generate hadronyche-tools/data/wp8/c004_NN.txt: the proof of the WP8 ports'
private helpers (004-24, 004-25, 004-26, 004-28, 004-29).

Each of these ports keeps a few helpers of its own that re-implement a
function or an inline rule of its script (the library has no equivalent; the
list is in each port's `proof.rs`). This generator is their proof: it reads
the scripts at a commit, executes each script's OWN definitions (and the
inline rules, copied below verbatim with their script:line), on a corpus of
inputs, and writes every result as canonical JSON. Each port's `proof.rs`
recomputes every result with the port's helper and compares, line by line; a
second test alters one line of the file and requires the comparison to fail.

Inputs (`data/wp8/inputs.txt`, shared by the five files): the synthetic edge
cases of gen_family004_corpus.py; every block (blank-line separated) of
doc/design/spec/wire.md and doc/design/spec/timeline.md at the commit, up to
a cap; and, for blocks holding a number, the variants of checklist point 3:
the first number replaced by 0, 007, +7, 1_2, 2^63, 20 digits and 45 digits,
and `²` / `µ` set next to it. Results holding an integer beyond i128 are
written `big` (the library reports such a value as INT_LIMIT, red by rule);
results where the script's function raises are written `raises:<Type>` (the
crash paths are the ports' denials). Both are counted by the tests.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_wp8_helpers.py [--python PY]... [--commit C]
The interpreters default to python3.10 and python3.13 on PATH; they must agree.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gen_family004_corpus as fam  # noqa: E402

DEFAULT_COMMIT = "b7c71c5"
OUT = "hadronyche-tools/src/doccheck/checks"
BASE_CAP = 400
VARIANT_CAP = 100
I128 = 2 ** 127


def canon(obj):
    return json.dumps(obj, ensure_ascii=False, separators=(",", ":"))


def norm(v):
    """Tuples as lists, sets sorted; `big` when an int is beyond i128."""
    if isinstance(v, bool) or v is None or isinstance(v, str):
        return v
    if isinstance(v, int):
        if not -I128 <= v < I128:
            raise OverflowError
        return v
    if isinstance(v, (set, frozenset)):
        return [norm(x) for x in sorted(v)]
    if isinstance(v, (list, tuple)):
        return [norm(x) for x in v]
    if isinstance(v, dict):
        return {k: norm(x) for k, x in v.items()}
    raise TypeError(type(v))


def result(f, *args):
    try:
        out = canon(norm(f(*args)))
        # a number beyond i128 written into a message counts as one too
        if any(not -I128 <= int(d) < I128 for d in re.findall(r"\d{39,}", out)):
            return "big"
        return out
    except OverflowError:
        return "big"
    except Exception as e:  # the script's function raises
        return "raises:" + type(e).__name__


def rust_str(t):
    out = ['"']
    for c in t:
        if c == '"':
            out.append('\\"')
        elif c == "\\":
            out.append("\\\\")
        elif c == "\n":
            out.append("\\n")
        elif c == "\r":
            out.append("\\r")
        elif c == "\t":
            out.append("\\t")
        elif ord(c) < 0x20 or ord(c) == 0x7f or 0x80 <= ord(c) < 0xa0 or c in "\u2028\u2029":
            out.append("\\u{%x}" % ord(c))
        else:
            out.append(c)
    out.append('"')
    return "".join(out)


def blocks(text):
    return [b for b in re.split(r"\n\s*\n", text) if b.strip()]


NUM_TOKEN = re.compile(r"\d+")
POINT3 = ["0", "007", "+7", "1_2", str(2 ** 63), "1" + "0" * 19, "1" * 45]


def variants(text):
    m = NUM_TOKEN.search(text)
    if not m:
        return []
    a, b = m.span()
    out = [text[:a] + v + text[b:] for v in POINT3]
    out.append(text[:b] + "²" + text[b:])
    out.append(text[:a] + "µ" + text[a:])
    return out


def inputs(commit):
    wire = fam.git_show(commit, "doc/design/spec/wire.md")
    timeline = fam.git_show(commit, "doc/design/spec/timeline.md")
    base = list(fam.SYNTH)
    for t in (wire, timeline):
        base += blocks(t)
    base = list(dict.fromkeys(base))[:BASE_CAP]
    with_num = [b for b in base if NUM_TOKEN.search(b)][:VARIANT_CAP]
    out = list(base)
    for b in with_num:
        out += variants(b)
    return list(dict.fromkeys(out))


def ints_of(t):
    return list(dict.fromkeys(int(x) for x in re.findall(r"\d+", t) if len(x) < 30))[:3]


def field_tables(ns, t):
    return [tb for tb in ns["parse_tables"](t) if ns["is_field_table"](tb[1])]


# --- per port: (helper name, args as input indexes or literals, result) ------
# Inline rules copied verbatim from the scripts, with script:line.

def cases_24(ns, ins):
    out = []
    for i, t in enumerate(ins):
        out.append(("tokens", [i], result(ns["outcome_tokens"], t)))
        out.append(("category", [i], result(ns["category"], t)))
        # 004-24:753 SENT = [s.strip().lower() for s in re.split(r"(?<=[.;:])\s+", PROSE) if s.strip()]
        sent = [s.strip().lower() for s in re.split(r"(?<=[.;:])\s+", t) if s.strip()]
        ns["SENT"] = sent
        for n in ints_of(t):
            out.append(("stated_max_size", [i, n], result(ns["stated_max_size"], n)))
        # 004-24: per sentence, re.findall(r"(?<![\d.])(\d+)\s*bytes", s) read with int()
        out.append(("byte_figures", [i], result(lambda s: [int(x) for x in re.findall(r"(?<![\d.])(\d+)\s*bytes", s)], t)))
        # 004-24: re.findall(r"(\d+)\s*×\s*(\d+)\s*=\s*(\d+)", s) read with int()
        out.append(("products", [i], result(lambda s: [(int(a), int(b), int(c)) for a, b, c in re.findall(r"(\d+)\s*×\s*(\d+)\s*=\s*(\d+)", s)], t)))
        if field_tables(ns, t):
            ns["RESULTS"] = []
            out.append(("tile_problems", [i], result(lambda s: ns["tile_problems"](ns["fields_of"](ns["first_table"](s, ns["is_field_table"]), "AC1", "t")), t)))
    return out


def tree_25(ns, title, text):
    """004-25:840-860, the per-type tree (inline in the script's loop), copied."""
    ns["RESULTS"] = []
    tables = ns["parse_tables"](text)
    field_tables_ = [tb for tb in tables if ns["is_field_table"](tb[1])]
    ft = []
    for tb in field_tables_:
        ft.append(ns["fields_of"](tb, "AC1", title))
    if not ft or not ft[0]:
        return None
    root, unused = ns["build_tree"](ft)
    fl_main = [f for f, _ in root["rows"]]
    off0 = fl_main[0]["off"]
    ns["tile_node"](root, off0 * 8 + (ns["lo_of"](fl_main[0]) or 0), title, "AC1")
    entries = ns["entry_nodes"](root)
    rbits = ns["size_node"](root, title)
    sec_max = 58 + ns["cdiv"](rbits, 8)
    starts = [f["_start"] for f, _ in root["rows"]]
    return [sec_max, len(unused), starts, [e["size"] for e in entries], len([r for r in ns["RESULTS"] if not r[1]])]


def cases_25(ns, ins):
    out = []
    for i, t in enumerate(ins):
        for n in ints_of(t):
            out.append(("has_num", [i, n], result(ns["has_num"], t, n)))
        out.append(("capnorm", [i], result(ns["capnorm"], t)))
        out.append(("lo_of", [i], result(lambda s: ns["lo_of"]({"meaning": s}), t)))
        out.append(("statements", [i], result(lambda s: [(int(a), int(b), int(c)) for a, b, c in ns["STMT"].findall(s)], t)))
        # 004-25: _code = TYPE_CODE_RE.search(text); int([g for g in _code.groups() if g][0])
        out.append(("type_code", [i], result(lambda s: (lambda m: int([g for g in m.groups() if g][0]) if m else None)(ns["TYPE_CODE_RE"].search(s)), t)))
        out.append(("stated_sizes", [i], result(lambda s: [int(x) for x in ns["SIZE_MAX_RE"].findall(s)], t)))
        for name in [c for c in C159 if re.search(r"\b" + c.split()[0][:4], t, re.I)]:
            out.append(("counter", [i, name], result(lambda s: re.search(ns["counter_re"](name), s, re.I) is not None, t)))
        out.append(("tree", [i], result(tree_25, ns, "t", t)))
        # 004-25:1030 keys = [counter_re(c) for c in C159]; max_match(keys, texts),
        # here over the lines of the input
        keys = [ns["counter_re"](c) for c in C159]
        out.append(("max_match", [i], result(lambda s: (lambda r: [r[0], sorted(r[1].items())])(ns["max_match"](keys, s.split("\n"))), t)))
    for c in REPERTOIRE:
        out.append(("isdigit", [c], result(str.isdigit, c)))
    return out


def cases_26(ns, ins):
    out = []
    for i, t in enumerate(ins):
        for k, tb in enumerate(ns["parse_tables"](t)):
            out.append(("is_validity_table", [i, k], result(ns["is_validity_table"], tb[1])))
        tabs = field_tables(ns, t)
        if tabs:
            def rows_and_tile(_):
                rows, bad = ns["model_rows"](tabs[0])
                offs, total, errs = ns["tile"](rows)
                return [[[r["off"], r["bits"], r["maxc"]] for r in rows], bad, offs, total, errs]
            out.append(("model_rows_tile", [i], result(rows_and_tile, t)))
        for w in ["none", "unknown"]:
            out.append(("code_for", [i, w], result(ns["code_for"], t, w)))
        out.append(("numeric_range", [i], result(ns["numeric_range"], t)))
        out.append(("is_repeat_row", [i], result(ns["is_repeat_row"], t)))
        out.append(("to_int", [i], result(lambda s: [ns["to_int"](m) for m in re.findall(ns["NUM"], s)], t)))
    return out


def cases_28(ns, ins):
    out = []
    for i, t in enumerate(ins):
        for n in ints_of(t):
            out.append(("has_num2", [i, n], result(ns["has_num2"], t, n)))
            out.append(("has_num2", [i, -n], result(ns["has_num2"], t, -n)))
        out.append(("exact_product_results", [i], result(ns["exact_product_results"], t)))
        out.append(("lo_of", [i], result(lambda s: ns["lo_of"]({"meaning": s}), t)))
        out.append(("statements", [i], result(lambda s: [(int(a), int(b), int(c)) for a, b, c in ns["STMT"].findall(s)], t)))
        out.append(("type_code", [i], result(lambda s: (lambda m: int([g for g in m.groups() if g][0]) if m else None)(ns["TYPE_CODE_RE"].search(s)), t)))
        out.append(("stated_sizes", [i], result(lambda s: [int(x) for x in ns["SIZE_MAX_RE"].findall(s)], t)))

        # 004-28:1047 not_observed_codes (nested in the AC2 block), copied
        def not_observed_codes(valid):
            o = []
            for cl in re.split(r";", re.sub(r"C-\d+", "", valid)):
                if re.search(r"not observed", cl, re.I):
                    o += [int(x) for x in re.findall(r"\b\d+\b", cl)]
            return o
        out.append(("not_observed_codes", [i], result(not_observed_codes, t)))
        if field_tables(ns, t):
            def analysed(s):
                ns["RESULTS"] = []
                a = ns["analyse"]("t", s, "AC1")
                return [a["sec_max"], a["Nt"], a["stmts"], a["code"], a["ok"], len([r for r in ns["RESULTS"] if not r[1]])]
            out.append(("analyse", [i], result(analysed, t)))
    return out


def cases_29(ns, ins):
    out = []
    for i, t in enumerate(ins):
        out.append(("clock_terms", [i], result(ns["clock_terms"], t)))
        rules = ns["parse_rules"](t)
        out.append(("parse_rules", [i], result(ns["parse_rules"], t)))
        for k, r in enumerate(rules):
            out.append(("rule_parts", [i, k], result(ns["rule_parts"], r)))
        out.append(("rule_parts_text", [i], result(ns["rule_parts"], t)))
        # 004-29:754 stated = [int(x) for x in SIZE_MAX_RE.findall(text)]
        out.append(("stated_sizes", [i], result(lambda s: [int(x) for x in ns["SIZE_MAX_RE"].findall(s)], t)))
    return out


C159 = ["received", "late", "missing", "marked gap", "other-epoch", "too-early", "duplicates",
        "incomplete reassemblies", "demoted rounds", "interrupted events", "dead"]
REPERTOIRE = [chr(int(l.split("\t")[0], 16)) for l in open(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "repertoire.tsv"), encoding="utf-8")
    if l.strip() and not l.startswith("#")]

PORTS = {"24": cases_24, "25": cases_25, "26": cases_26, "28": cases_28, "29": cases_29}


TABLE_HELPERS = {"analyse", "tile_problems", "model_rows_tile", "tree"}


def has_big(t):
    return any(not -I128 <= int(d) < I128 for d in re.findall(r"\d{39,}", t))


def worker(commit):
    ins = inputs(commit)
    res = {}
    real = sys.stdout
    sys.stdout = open(os.devnull, "w")  # the scripts' check() prints PASS/FAIL lines
    for port, f in PORTS.items():
        src = fam.git_show(commit, f"checks/004-{port}.sh")
        ns = fam.namespace(src)
        # the one-line compiled patterns the inline rules use, where the
        # family generator's constant scan does not reach them
        for line in src.split("\n"):
            if re.match(r"^(STMT|SIZE_MAX_RE|TYPE_CODE_RE) = re\.compile\(.*\)$", line):
                exec(line, ns)
        ns["RESULTS"] = []
        # a table reader reads every Offset and Width cell with int(): with a
        # cell beyond i128 the library reports INT_LIMIT (red by rule)
        res[port] = [[h, a, "big" if h in TABLE_HELPERS and has_big(ins[a[0]]) else r] for h, a, r in f(ns, ins)]
    sys.stdout = real
    json.dump({"version": sys.version.split()[0], "inputs": ins, "results": res}, sys.stdout, ensure_ascii=False)


def interpreters(given):
    found = given or [p for p in ["python3.10", "python3.13"] if shutil.which(p)]
    if not found:
        sys.exit("no python3.10 or python3.13 on PATH; pass --python")
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
        r = subprocess.run([py, __file__, "--worker", "--commit", args.commit], capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"{py} failed:\n{r.stderr}")
        runs.append(json.loads(r.stdout))
    for r in runs[1:]:
        if r["results"] != runs[0]["results"] or r["inputs"] != runs[0]["inputs"]:
            sys.exit("the interpreters disagree: record both and decide")
    run = runs[0]
    who = ", ".join(f"Python {r['version']}" for r in runs)
    head = (f"// @generated by hadronyche-tools/tools/gen_wp8_helpers.py at {args.commit} ({who}).\n"
            "// Do not edit: rerun the generator. A check module may not read files\n"
            "// (include_str! is refused), so the proof corpus is Rust data.\n\n")
    for port, rows in run["results"].items():
        used = sorted({a[0] for h, a, r in rows if h != "isdigit"})
        with open(f"{OUT}/c004_{port}/proof_data.rs", "w", encoding="utf-8") as f:
            f.write(head)
            f.write(f"//! The proof corpus of 004-{port}'s private helpers: the inputs, then one\n"
                    "//! (helper, JSON args, script result) per case; a result is canonical JSON,\n"
                    "//! `big` (an integer beyond i128: INT_LIMIT by rule) or `raises:<Type>`.\n\n")
            f.write("/// The inputs (an argument `n` of a case names `INPUTS[n]`).\n")
            f.write("pub(super) const INPUTS: &[&str] = &[\n")
            for k, t in enumerate(run["inputs"]):
                f.write(f"    {rust_str(t if k in used else '')},\n")
            f.write("];\n\n/// The cases.\npub(super) const CASES: &[(&str, &str, &str)] = &[\n")
            for h, a, r in rows:
                f.write(f"    ({rust_str(h)}, {rust_str(canon(a))}, {rust_str(r)}),\n")
            f.write("];\n")

if __name__ == "__main__":
    main()
