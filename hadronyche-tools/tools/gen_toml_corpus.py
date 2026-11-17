#!/usr/bin/env python3
"""Generate hadronyche-tools/data/toml_corpus.txt (lead ruling R14, M1-lib).

Every snippet below is read by each TOML reader the 004 scripts carry, taken
verbatim from the scripts themselves, and by tomllib:
  * subset: the statement-scanner reader (`_scan_statements` ... `parse_toml`)
    of 004-03, 05, 06, 08-13, 16-20 and 24-29 (at SCRIPTS_COMMIT) and of
    004-27 (at b9de402^, before its deletion). The generator runs EVERY
    script's copy and fails unless all of them give the same result on every
    snippet (exception types included), so one record stands for all;
  * line14 / line15: the line readers (`strip_toml_comment`, `parse_value`,
    `parse_toml`) of 004-14 and 004-15, which differ, so each has its record;
    a result is `[messages, others]`;
  * tomllib: `tomllib.loads`, Python 3.13 only (004-22's interpreter).
Each script reader runs on Python 3.10 (the scripts' interpreter) and 3.13;
the generator fails unless they agree.

Each result is written in the canonical form of
`hadronyche_tools::doccheck::pyvalue::canonical` (dict keys in insertion
order for the script readers, sorted for tomllib, whose order no check
reads; `n` for None), or `ERR <exception type>`. The Rust tests compare the
library's ports (`tomlsubset`, `tomlline`) and its tomllib stand-in (the
`toml` crate) to it; the crate's differences from tomllib are a committed
list in the test.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_toml_corpus.py [--py310 PY] [--py313 PY] > hadronyche-tools/data/toml_corpus.txt
The interpreters default to python3.10 and python3.13 on PATH.
"""

import argparse
import json
import shutil
import subprocess
import sys

SCRIPTS_COMMIT = "b7c71c5"
SUBSET_SCRIPTS = [(SCRIPTS_COMMIT, f"checks/004-{n}.sh") for n in
                  ("03", "05", "06", "08", "09", "10", "11", "12", "13", "16", "17", "18", "19", "20",
                   "24", "25", "26", "28", "29")] + [("b9de402^", "checks/004-27.sh")]
LINE_SCRIPTS = {"line14": (SCRIPTS_COMMIT, "checks/004-14.sh"), "line15": (SCRIPTS_COMMIT, "checks/004-15.sh")}
BIG_4300 = "1" + "0" * 4299
BIG_4301 = "1" + "0" * 4300

SNIPPETS = [
    ("table header", '[a]\nx = 1\n'),
    ("array of tables", '[[m]]\nname = "a"\n[[m]]\nname = "b"\n'),
    ("nested array of tables", '[[m]]\nname = "a"\n[[m.field]]\nname = "f"\nwidth = 8\n[[m.field]]\nname = "g"\n'),
    ("dotted header", '[a.b.c]\nx = 1\n'),
    ("header into array", '[[m]]\n[m.sub]\nx = 1\n'),
    ("dotted key", 'meta.note = "x"\n'),
    ("dotted key quoted", '"a.b".c = 1\n'),
    ("quoted key", '"key with space" = 1\n'),
    ("single quoted key", "'k' = 1\n"),
    ("bare key with space", 'not exempt = 1\n'),
    ("empty key component", 'a..b = 1\n'),
    ("integer", 'a = 42\n'),
    ("negative integer", 'a = -17\n'),
    ("plus integer", 'a = +17\n'),
    ("leading zero", 'a = 040\n'),
    ("underscores", 'a = 1_000\n'),
    ("hex", 'a = 0x28\n'),
    ("hex upper", 'a = 0X2A\n'),
    ("hex underscore", 'a = 0xdead_beef\n'),
    ("octal", 'a = 0o50\n'),
    ("binary", 'a = 0b101\n'),
    ("bad hex", 'a = 0xZZ\n'),
    ("negative hex", 'a = -0x10\n'),
    ("big integer", 'a = 123456789012345678901234567890123456789012\n'),
    ("int64 max", 'a = 9223372036854775807\n'),
    ("int64 overflow", 'a = 9223372036854775808\n'),
    ("float", 'a = 40.0\n'),
    ("float exponent", 'a = 1e5\n'),
    ("float exponent sign", 'a = 6.626e-34\n'),
    ("float trailing dot", 'a = 5.\n'),
    ("float leading dot", 'a = .5\n'),
    ("float underscore", 'a = 1_000.5\n'),
    ("inf", 'a = inf\n'),
    ("negative inf", 'a = -inf\n'),
    ("nan", 'a = nan\n'),
    ("bool true", 'a = true\n'),
    ("bool false", 'a = false\n'),
    ("bool capital", 'a = True\n'),
    ("date", 'a = 1979-05-27\n'),
    ("datetime", 'a = 1979-05-27T07:32:00Z\n'),
    ("local time", 'a = 07:32:00\n'),
    ("bare word", 'a = control\n'),
    ("bare value with dash", 'exempt_from_control_bound = C-116\n'),
    ("basic string", 'a = "control"\n'),
    ("empty string", 'a = ""\n'),
    ("escapes", 'a = "tab\\tnl\\nq\\"bs\\\\"\n'),
    ("unicode escape", 'a = "contro\\u006C"\n'),
    ("long unicode escape", 'a = "\\U0001F600"\n'),
    ("bad unicode escape", 'a = "\\uZZZZ"\n'),
    ("unknown escape", 'a = "\\q"\n'),
    ("literal string", "a = 'C:\\\\path'\n"),
    ("triple double quoted", 'note = """valid TOML"""\n'),
    ("triple single quoted", "note = '''valid TOML'''\n"),
    ("multi-line triple string", 'note = """\nline\n"""\n'),
    ("string with hash", 'a = "x # not a comment"\n'),
    ("string with equals", 'a = "b = c"\n'),
    ("unterminated string", 'a = "open\n'),
    ("unterminated literal", "a = 'open\n"),
    ("comment line", '# comment\na = 1\n'),
    ("trailing comment", 'a = 1 # comment\n'),
    ("comment at end without newline", 'a = 1\n# end'),
    ("comment inside array", 'a = [ # first\n "x", # second\n "y" ]\n'),
    ("array", 'a = ["TooShort", "BadMagic"]\n'),
    ("empty array", 'a = []\n'),
    ("mixed array", 'a = [3, "TooShort"]\n'),
    ("nested array", 'a = [[1, 2], [3]]\n'),
    ("trailing comma array", 'a = [1, 2,]\n'),
    ("multi-line array", 'a = [\n  1,\n  2\n]\n'),
    ("unterminated array", 'a = [1, 2\n'),
    ("array missing comma", 'a = [1 2]\n'),
    ("inline table", 'a = { BadLength = 1 }\n'),
    ("inline table dotted", 'a = { x.y = 1, z = "w" }\n'),
    ("empty inline table", 'a = {}\n'),
    ("inline table missing equals", 'a = { x }\n'),
    ("duplicate key", 'dscp = 46\ndscp = 40\n'),
    ("duplicate key in field", '[[f]]\nwidth = 16\nwidth = 16\n'),
    ("duplicate table", '[a]\nx = 1\n[a]\ny = 2\n'),
    ("key redefined as table", 'a = 1\n[a.b]\nx = 1\n'),
    ("scalar then array of tables", 'a = 1\n[[a]]\n'),
    ("table then array of tables", '[a]\n[[a]]\n'),
    ("header into empty array", 'a = []\n[a.b]\n'),
    ("dotted key through scalar", 'a = 1\na.b = 2\n'),
    ("missing value", 'a =\n'),
    ("trailing text", 'a = 1 2\n'),
    ("no equals", 'just text\n'),
    ("crlf line ends", 'a = 1\r\nb = 2\r\n'),
    ("tab indentation", '\ta = 1\n'),
    ("whitespace around equals", 'a=1\nb   =   2\n'),
    ("message like wire.toml", '[[message]]\nname = "path-MTU probe request"\nlayer = "control"\ndscp = 40\nmax_size = 65507\nexempt_from_control_bound = "C-116"\nrejection_reasons = ["TooShort", "BadMagic"]\n\n[[message.field]]\nname = "magic"\noffset = 0\nwidth = 16\ntype = "u16"\nvalid = "0x4d4e only"\n'),
    ("refusal entries", '[[refusal]]\nname = "Busy"\nvalue = 0\n[[refusal]]\nname = "PathMtuTooSmall"\nvalue = 10\n'),
    ("rejection_reasons string", 'rejection_reasons = "BadLength"\n'),
    ("rejection_reasons integer", 'rejection_reasons = 5\n'),
    ("offset string", 'offset = " +384 "\n'),
    ("offset string underscore", 'offset = "3_84"\n'),
    ("max_size float", 'max_size = 65507.0\n'),
    # aimed at the line readers of 004-14 and 004-15 (M1-lib)
    ("line: message with fields", '[[message]]\nname = "a"\nlayer = "control"\n[[message.field]]\nname = "f"\noffset = 0\nwidth = 16\n[[message.field]]\nname = "g"\n'),
    ("line: message without name", '[[message]]\nlayer = "sync"\n'),
    ("line: field before any message", '[[message.field]]\nname = "f"\n'),
    ("line: key before any table", 'a = 040x\n[t]\nb = 1\n'),
    ("line: array of other tables", '[[reject]]\nname = "TooShort"\nvalue = 1\n[[reject]]\nname = "BadMagic"\nvalue = 2\n'),
    ("line: table then array of the same name", '[r]\nx = 1\n[[r]]\ny = 2\n'),
    ("line: array then table of the same name", '[[r]]\nx = 1\n[r]\ny = 2\n'),
    ("line: fields reassigned then a field", '[[message]]\nfields = 3\n[[message.field]]\n'),
    ("line: fields reassigned to a list", '[[message]]\nfields = [1]\n[[message.field]]\nname = "f"\n'),
    ("line: spaces inside the brackets", '[[ message ]]\nname = "a"\n[ meta ]\nx = 1\n'),
    ("line: text after the brackets", '[[message]] trailing\nname = "a"\n'),
    ("line: unterminated array header", '[[message\nname = "a"\n'),
    ("line: unterminated table header", '[meta\nx = 1\n'),
    ("line: empty header", '[[]]\nx = 1\n[]\ny = 2\n'),
    ("line: escaped quote in a string", '[t]\na = "say \\"hi\\""\n'),
    ("line: escaped backslash in a string", '[t]\na = "C:\\\\x"\n'),
    ("line: escaped quote then hash", '[t]\na = "x\\"#y" # comment\n'),
    ("line: unterminated string value", '[t]\na = "open\n'),
    ("line: text after a string", '[t]\na = "x" trailing\n'),
    ("line: hash in a string", '[t]\na = "# not a comment"\n'),
    ("line: array of ints and hex", '[t]\na = [0x1f, -3, 1_0, -0x10]\n'),
    ("line: array with a leading zero", '[t]\na = [040]\n'),
    ("line: array with a float and a bool", '[t]\na = [1.5, true, "s"]\n'),
    ("line: array with escaped string", '[t]\na = ["a\\"b", "c"]\n'),
    ("line: array then more text", '[t]\na = [1] 2\n'),
    ("line: array with zero forms", '[t]\na = [0, 00, -0, 0_1]\n'),
    ("line: scalar forms", '[t]\na = 0o17\nb = 0B11\nc = 0X1F\nd = -040\ne = +5\nf = 00\ng = 0x\nh = 1e3\n'),
    ("line: scalar bare and quoted", "[t]\na = C-116\nb = 'lit'\nc = 1.5\nd = True\n"),
    ("line: two equals", '[t]\na = 1 = 2\n'),
    ("line: empty key and value", '[t]\n= 5\nb =\n'),
    ("line: no equals", '[t]\njust text\n'),
    ("line: underscores", '[t]\na = _5_\nb = 1__000\nc = 0x_ff\n'),
    ("line: big integer", '[t]\na = 123456789012345678901234567890123456789012\nb = [123456789012345678901234567890123456789012]\n'),
    ("line: 4300 digits", '[t]\na = ' + BIG_4300 + '\n'),
    ("line: 4301 digits", '[t]\na = ' + BIG_4301 + '\n'),
    ("line: 4301 digits in an array", '[t]\na = [' + BIG_4301 + ']\n'),
    ("line: crlf and lone cr", '[t]\r\na = 1\rb = 2\r\n'),
    ("line: form feed splits a line", '[t]\na = 1\x0cb = 2\n'),
    ("line: tabs and duplicate keys", '[t]\n\ta\t=\t1\na = 2\n'),
    ("line: unicode key", '[t]\n\u00b5 = "\u00b5s"\n'),
    ("line: duplicate table replaced", '[t]\na = 1\n[t]\nb = 2\n'),
    ("line: single-bracket message", '[message]\nname = "x"\n'),
]

FACTS = r'''
import json, math, re, struct, sys
def canon(v, sort):
    if v is None:
        return "n"
    if isinstance(v, tuple):
        v = list(v)
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return "i" + str(v)
    if isinstance(v, float):
        return "f" + struct.pack(">d", v).hex()
    if isinstance(v, str):
        return 's"' + "".join(c if (" " <= c <= "~" and c not in '"\\') else "\\u{%x}" % ord(c) for c in v) + '"'
    if isinstance(v, list):
        return "[" + ",".join(canon(x, sort) for x in v) + "]"
    if isinstance(v, dict):
        items = [(k, canon(x, sort)) for k, x in v.items()]
        if sort:
            items.sort()
        return "{" + ",".join(canon(k, sort) + ":" + x for k, x in items) + "}"
    return "d"
snippets = json.loads(sys.stdin.read())
mode = sys.argv[1]
out = []
if mode == "reader":
    ns = {"re": re}
    exec(sys.argv[2], ns)
    for src in snippets:
        try:
            out.append(canon(ns["parse_toml"](src), False))
        except Exception as e:
            out.append("ERR " + type(e).__name__)
else:
    import tomllib
    for src in snippets:
        try:
            out.append(canon(tomllib.loads(src), True))
        except Exception as e:
            out.append("ERR " + type(e).__name__)
print(json.dumps({"version": sys.version.split()[0], "out": out}))
'''


def enc(s):
    return 's"' + "".join(c if (" " <= c <= "~" and c not in '"\\') else "\\u{%x}" % ord(c) for c in s) + '"'


def script_text(commit, path):
    return subprocess.run(["git", "--no-optional-locks", "show", f"{commit}:{path}"],
                          capture_output=True, text=True, check=True).stdout


def block(text, start, end):
    i = text.index(start)
    j = text.index(end, i) + len(end)
    return text[i:j]


def subset_source(commit, path):
    return block(script_text(commit, path), "class TomlError(Exception):", "    return root\n")


def line_source(commit, path):
    return block(script_text(commit, path), "def strip_toml_comment(line):", "    return messages, others\n")


def run(py, mode, extra=()):
    srcs = [s for _, s in SNIPPETS]
    res = subprocess.run([py, "-c", FACTS, mode, *extra], input=json.dumps(srcs),
                         capture_output=True, text=True, check=True)
    return json.loads(res.stdout)


def interpreter(given, default):
    path = shutil.which(given or default)
    if path is None:
        sys.exit(f"interpreter {given or default!r} not found; pass --py310/--py313")
    return path


def agreed(pys, mode, source, what):
    runs = [run(py, mode, [source]) for py in pys]
    for r in runs[1:]:
        if r["out"] != runs[0]["out"]:
            bad = [SNIPPETS[k][0] for k, (a, b) in enumerate(zip(runs[0]["out"], r["out"])) if a != b]
            sys.exit(f"{what}: Python {runs[0]['version']} and {r['version']} disagree on {bad}")
    return runs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--py310")
    ap.add_argument("--py313")
    args = ap.parse_args()
    pys = [interpreter(args.py310, "python3.10"), interpreter(args.py313, "python3.13")]
    reference = None
    for commit, path in SUBSET_SCRIPTS:
        runs = agreed(pys, "reader", subset_source(commit, path), f"{path} at {commit}")
        if reference is None:
            reference = runs
        elif runs[0]["out"] != reference[0]["out"]:
            bad = [SNIPPETS[k][0] for k, (a, b) in enumerate(zip(reference[0]["out"], runs[0]["out"])) if a != b]
            sys.exit(f"{path} at {commit}: its reader differs from {SUBSET_SCRIPTS[0][1]}'s on {bad}: name a variant")
    lines = {k: agreed(pys, "reader", line_source(*v), v[1]) for k, v in LINE_SCRIPTS.items()}
    tomllib = run(pys[1], "tomllib")
    versions = " and ".join(f"Python {r['version']}" for r in reference)
    print(f"# TOML corpus: {len(SNIPPETS)} snippets. subset = the reader of every 004 statement-scanner script "
          f"(004-03, 05, 06, 08-13, 16-20, 24-29 at {SCRIPTS_COMMIT}, 004-27 at b9de402^; identical on every snippet); "
          f"line14 / line15 = the readers of checks/004-14.sh / 004-15.sh at {SCRIPTS_COMMIT}; "
          f"each run with {versions}, which agree; tomllib = Python {tomllib['version']}.")
    print("# generated by hadronyche-tools/tools/gen_toml_corpus.py; one record per snippet:")
    print("# === name / src s\"...\" / subset / line14 / line15 / tomllib, each <canonical|ERR type>")
    for k, (name, src) in enumerate(SNIPPETS):
        print(f"=== {name}")
        print(f"src {enc(src)}")
        print(f"subset {reference[0]['out'][k]}")
        print(f"line14 {lines['line14'][0]['out'][k]}")
        print(f"line15 {lines['line15'][0]['out'][k]}")
        print(f"tomllib {tomllib['out'][k]}")


if __name__ == "__main__":
    sys.exit(main())
