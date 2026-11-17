#!/usr/bin/env python3
"""Generate hadronyche-tools/data/toml_corpus.txt (lead ruling R14).

Every snippet below is read by:
  * the 004 family's hand-written TOML reader (`_scan_statements` ...
    `parse_toml`), taken verbatim from checks/004-27.sh as it stood before
    its deletion (b9de402^; byte-identical to 004-18's), run with the Python
    the script ran on (3.10, no tomllib);
  * `tomllib.loads`, run with Python 3.13 (004-22's interpreter).
Each result is written in the canonical form of
`hadronyche_tools::doccheck::pyvalue::canonical` (dict keys in insertion
order for the subset reader, sorted for tomllib, whose order no check
reads), or `ERR <exception type>`. The Rust tests read the file and compare
the library's port of the reader (`tomlsubset`) and its tomllib stand-in
(the `toml` crate) to it; the crate's differences from tomllib are a
committed list in the test.

Usage: python3 hadronyche-tools/tools/gen_toml_corpus.py > hadronyche-tools/data/toml_corpus.txt
"""

import json
import subprocess
import sys

PY_SUBSET = "/home/dorian/.pyenv/versions/3.10.19/bin/python3"
PY_TOMLLIB = "/usr/bin/python3.13"
SOURCE = ("b9de402^", "checks/004-27.sh")

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
]

FACTS = r'''
import json, math, re, struct, sys
def canon(v, sort):
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
if mode == "subset":
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


def reader_source():
    text = subprocess.run(["git", "--no-optional-locks", "show", f"{SOURCE[0]}:{SOURCE[1]}"],
                          capture_output=True, text=True, check=True).stdout
    start = text.index("class TomlError(Exception):")
    end = text.index("    return root\n", start) + len("    return root\n")
    return text[start:end]


def run(py, mode, extra=()):
    srcs = [s for _, s in SNIPPETS]
    res = subprocess.run([py, "-c", FACTS, mode, *extra], input=json.dumps(srcs),
                         capture_output=True, text=True, check=True)
    return json.loads(res.stdout)


def main():
    subset = run(PY_SUBSET, "subset", [reader_source()])
    tomllib = run(PY_TOMLLIB, "tomllib")
    print(f"# TOML corpus: {len(SNIPPETS)} snippets. subset = the reader of {SOURCE[1]} at {SOURCE[0]} "
          f"run with Python {subset['version']}; tomllib = Python {tomllib['version']}.")
    print("# generated by hadronyche-tools/tools/gen_toml_corpus.py; one record per snippet:")
    print("# === name / src s\"...\" / subset <canonical|ERR type> / tomllib <canonical|ERR type>")
    for (name, src), a, b in zip(SNIPPETS, subset["out"], tomllib["out"]):
        print(f"=== {name}")
        print(f"src {enc(src)}")
        print(f"subset {a}")
        print(f"tomllib {b}")


if __name__ == "__main__":
    sys.exit(main())
