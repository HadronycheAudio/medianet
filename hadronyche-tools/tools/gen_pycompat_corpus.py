#!/usr/bin/env python3
"""Generate hadronyche-tools/data/pycompat_corpus.txt: Python's answers for
the behaviours `doccheck::pycompat`, `doccheck::rx` and `doccheck::pynum`
reproduce, proven by the Rust test `pynum::tests::the_corpus_agrees_with_python`
without running Python.

Cases (one per line, tab-separated fields; in every field `\\` `\t` `\n` `\r`
are written `\\\\` `\\t` `\\n` `\\r`; a Python `None` is written `~`):
  search   PATTERN HAY            -> the match and its groups, or `~`
  iter     PATTERN HAY            -> every match (re.finditer, group 0)
  split    PATTERN HAY            -> re.split (patterns without groups)
  sub      PATTERN REPL HAY       -> re.sub with a literal replacement
  float    TEXT                   -> repr(float(TEXT)), or `ValueError`
  fmt      SPEC REPR              -> format(float(REPR), SPEC)
  pct      SPEC REPR              -> ("%" + SPEC) % float(REPR)
  slice    TEXT START STOP        -> TEXT[START:STOP] (`~` for an omitted bound)
  truthy   KIND VALUE             -> bool(value) (kinds: str int float list)
  eval     EXPR                   -> repr(eval(EXPR)) over + - * **, or the error class
  int      TEXT                   -> str(int(TEXT)), or `ValueError`
  frac     A B                    -> str(Fraction(A, B))
  fraclim  REPR                   -> str(Fraction(float(REPR)).limit_denominator())
  nfkc     TEXT                   -> unicodedata.normalize("NFKC", TEXT)

The patterns are the scripts' own look-around patterns (004-01, 004-04,
004-09, 004-11, 004-12, 004-15, 004-29, 006-02, the 002 family) and the
review's probes. Every interpreter given must produce the same file.

Usage (repository root):
  python3 hadronyche-tools/tools/gen_pycompat_corpus.py [--python PY]... > hadronyche-tools/data/pycompat_corpus.txt
  python3 hadronyche-tools/tools/gen_pycompat_corpus.py [--python PY]... --check hadronyche-tools/data/pycompat_corpus.txt
Interpreters default to python3.10 and python3.13 on PATH.
"""

import argparse
import shutil
import subprocess
import sys

CASES = r'''
import re, sys, unicodedata
from fractions import Fraction

def enc(v):
    if v is None:
        return "~"
    return str(v).replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")

out = []
def emit(op, args, res):
    out.append("\t".join([op] + [enc(a) for a in args] + [enc(r) for r in res]))

LOOK = [
    r"(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)",
    r"(?<![\w.\-])(\d+)\.(\d+)(?!\w)",
    r"(?<![\w.])(\d+)(?:\.(\d+))?\s*%",
    r"(?<![\w.])(\d+)(?:\.(\d+))?\s*Mbit/s",
    r"(?<![\w\-])([−+-])\s*(\d+)(?:\.(\d+))?\s*%",
    r"(?<![\w.\-−+])(\d+)(?:\.(\d+))?\s*%",
    r"(?<![\w.])5 ?%",
    r"(?<![\d.])28 ?\+ ?38(?![\d])",
    r"(?<![\d.])66(?![\d.])",
    r"(?<![\w.\-])(\d{1,3})(?![\w.\-])",
    r"(?<![\w.])(\d[\d_]*)(?![\d.]|\.\d)",
    r"(?<![\d.])(\d+)\s*\.\s*(\d+)(?![\d.])",
    r"(?<![\w.])(\d+)(?![\d.])",
    r"(?<![\d.])(?:1|2|4|8|16)(?![\d.])",
    r"(?<![\w.])\d+(?:[ ]\d{3})*(?:[.]\d+)?(?![\w])",
    r"(?<!not\s)(?<!never\s)(?<!no\s)\b(?:raise|raises|raised|emit|emits)\b",
    r"(?<![\d.,])(10|50|200|1000)(?![\d.,])",
    r"(?<![\w.,/-])(?:37|77)(?:\s*/\s*s\b|/s\b)",
    r"(?<![A-Za-z0-9-])C-17(?![A-Za-z0-9])",
    r"(?<![\w-])`?frames`?(?![\w-])",
    r"(?<![\w.])(\d(?:[\d ]*\d)?(?:\s*[+\-−×x*]\s*\d(?:[\d ]*\d)?)+)\s*=\s*(\d(?:[\d ]*\d)?)(?:\.(\d+))?",
    r"(?<=\d),(?=\d{3}(?!\d))",
    r"(?<=\d)[ ](?=\d{3}(?!\d))",
    r"(?=\w)x\d",
    r"a(?!b)",
    r"(?<=[.;])\s+",
    r"x*",
    r"\s*",
    r",?",
    r"a$",
    r"(?m)a$",
    r"^\w+$",
    r"\bv1\b[^.;]{0,50}?(?:=|:|is)\s*(?<![\w.])(\d+)(?![\d.]|\.\d)",
]
HAYS = [
    "", "a", "a\n", "ab", "abxd", "a b", "a,b", "a\nb\n",
    "12", "a 12 b", "a-12 b", "1.12 x", "x12", "12.5", "12.5.3", "v1.2", "the 1 280-byte fallback",
    "65 507 bytes and 1 280", "3.5 Mbit/s per channel, 21.1 %", "share: -2.5 % and +3 %", "−4 %",
    "5 % or 5%", "15 %", "28 + 38 = 66", "H + 28 + 38 = 66 bytes", "66.0", "a.66", "28+38",
    "1 2 3 1000 10 50.5 200", "37/s and 77 / s", "x/37/s", "C-17 and C-170 and XC-17",
    "`frames` and frames-x and -frames", "it does not raise; it raises", "never raise; no emit; emits",
    "1,000,000 and 12,34", "1 000 000 and 12 34", "x1 _x2 ²x3", "v1 is 7; v1 = 8.5",
    "²³ 1²", "µs 12", "a=1. b=2; c", "sizes: 1 2 4 8 16 32", "8.16 4", "12_000 and 1_2.5",
]
for p in LOOK:
    r = re.compile(p)
    for h in HAYS:
        m = r.search(h)
        emit("search", [p, h], ["~"] if m is None else [m.group(0)] + list(m.groups()))
        emit("iter", [p, h], [m.group(0) for m in r.finditer(h)])
        if r.groups == 0:
            emit("split", [p, h], re.split(p, h))
            emit("sub", [p, "<>", h], [re.sub(p, "<>", h)])

# the number readers of 004-15 and 004-24 on seeded random text (with ², µ,
# minus signs and units), where private emulations of their look-behinds
# were found to differ
import random
rnd = random.Random(7)
ALPHA = list("0123456789 .-_aZ²µ%,;x") + [" Mbit/s", " bytes", "bytes", "−", "+"]
NUM = [r"(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)", r"(?<![\w.\-])(\d+)\.(\d+)(?!\w)",
       r"(?<![\w.])(\d+)(?:\.(\d+))?\s*%", r"(?<![\w.])(\d+)(?:\.(\d+))?\s*Mbit/s", r"(?<![\d.])(\d+)\s*bytes"]
for _ in range(400):
    h = "".join(rnd.choice(ALPHA) for _ in range(rnd.randint(1, 14)))
    for p in NUM:
        emit("iter", [p, h], [m.group(0) for m in re.finditer(p, h)])

for t in ["1", "1.5", " 2.50 ", "1e3", "1E-7", "inf", "-inf", "nan", "1_0.5", "0.1", "1.", ".5", "-0",
          "1e400", "x", "", "1,5", "12345678901234567890.5", "3.5", "1.1", "2.675", "0.000001", "1e16", "١٢"]:
    try:
        emit("float", [t], [repr(float(t))])
    except ValueError:
        emit("float", [t], ["ValueError"])

VALS = [0.0, -0.0, 1.0, 0.5, 2.675, 1e16, 1.5e-7, 123456789.0, 3.5, 21.1, 0.1 + 0.2, 1 / 3, 2 / 3,
        65536 * 0.4, 1e22, 1e-5, 100.0, 12.0, 0.05, 2.5, 3.0005, 7.25e-05, 31250 / 3, 10.0 / 4]
for v in VALS:
    for spec in ["g", ".4g", ".1f", ".2f", ".3f", "+.2f", "+.3f", ".0f", "e", ".3e", ""]:
        emit("fmt", [spec, repr(v)], [format(v, spec)])
    for spec in [".4g", ".2f", "d", "s", "g"]:
        emit("pct", [spec, repr(v)], [("%" + spec) % v])
    emit("fmt", ["repr", repr(v)], [repr(v)])
    emit("fmt", ["str", repr(v)], [str(v)])

S = ["", "a", "héllo²", "µs Δ ≤ 3", "abcdef"]
B = [None, 0, 1, 2, -1, -2, 3, 10, -10]
for s in S:
    for a in B:
        for b in B:
            emit("slice", [s, a, b], [s[a:b]])

for kind, v in [("str", ""), ("str", " "), ("str", "0"), ("int", "0"), ("int", "-1"), ("float", "0.0"),
                ("float", "-0.0"), ("float", "nan"), ("float", "0.5"), ("list", "0"), ("list", "2")]:
    val = {"str": lambda x: x, "int": int, "float": float, "list": lambda x: [0] * int(x)}[kind](v)
    emit("truthy", [kind, v], [str(bool(val))])

for e in ["1+2", "2*3+4", "2**3**2", "-2**2", "2**-1", "1--1", "1++1", "10-3-2", "2*-3", "00", "01", "0+00",
          "1+", "**2", "2**", "4**0.5", "7-7", "2**127", "2**128", "-2**127", "170141183460469231731687303715884105727+1",
          "9999999999*9999999999*9999999999*9999999999", "3*4*5-60", "2+2*2**2", "1 + 2", "100-0", "0**0", "2*", "+5", "-0",
          # review round 5: parentheses, unary minus on a group, underscores, based literals
          "-(2)", "(1+2)*3", "1_000+1", "0x10+1", "0o17*2", "0b101-1", "0x_1f", "1__0", "1_", "0_0",
          "((2))", "(2", "2)", "-(2**2)", "(-2)**2", "2**(1+1)", "1e3+1", "1.5e-1*2", ".5+.5", "1.", "0x1e-1",
          "1e", "0xg", "00_0", "0b2", "2**-(1)", "170141183460469231731687303715884105727+(1)",
          "0x7fffffffffffffffffffffffffffffff+1"]:
    try:
        emit("eval", [e], [repr(eval(e))])
    except Exception as ex:
        emit("eval", [e], [type(ex).__name__])

for t in ["0", "12", "-12", "+7", " 42 ", "1_000", "1__0", "_1", "0012", "١٢", "²", "1.0", "", "x",
          "170141183460469231731687303715884105727", "170141183460469231731687303715884105728",
          "-170141183460469231731687303715884105728", "-170141183460469231731687303715884105729", "9" * 60]:
    try:
        emit("int", [t], [str(int(t))])
    except ValueError:
        emit("int", [t], ["ValueError"])

for a, b in [(1, 2), (2, 4), (-3, 6), (3, -6), (0, 5), (48000, 1000000000), (12, 125000), (1000000000, 48000),
             (31250, 3), (7, 1)]:
    emit("frac", [a, b], [str(Fraction(a, b))])
for v in [0.5, 0.1, 1 / 3, 2.675, 31250 / 3, 0.000123, 3.14159265358979, 1e-7, 12.0, -0.75, 1e10 + 0.5]:
    emit("fraclim", [repr(v)], [str(Fraction(v).limit_denominator())])

for t in ["µs", "x²+y³", "¹⁴⁵⁶⁹⁻₁₂₃", "⅓ ⅔", "…", "a – b — c − d", "§ ± · × ÷ ‖ ‰", "Δ Σ α π τ",
          "→ ↔ ⇒ ∈ ∞ ≈ ≠ ≡ ≤ ≥ ⌈ ⌉ ⌊ ⌋", "plain ASCII 123"]:
    emit("nfkc", [t], [unicodedata.normalize("NFKC", t)])

print(sys.version.split()[0])
print("\n".join(out))
'''


def run(py):
    path = shutil.which(py)
    if path is None:
        sys.exit(f"interpreter {py!r} not found on PATH; pass --python PATH")
    res = subprocess.run([path, "-c", CASES], capture_output=True, text=True, check=True)
    version, _, body = res.stdout.partition("\n")
    return version, body.rstrip("\n").split("\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--python", action="append", default=[])
    ap.add_argument("--check", metavar="FILE")
    args = ap.parse_args()
    runs = [run(py) for py in (args.python or ["python3.10", "python3.13"])]
    for v, body in runs[1:]:
        if body != runs[0][1]:
            diff = [a for a, b in zip(runs[0][1], body) if a != b][:5]
            sys.exit(f"Python {runs[0][0]} and {v} disagree: {diff}")
    lines = [
        "# Python's answers for doccheck::pycompat / rx / pynum, generated by hadronyche-tools/tools/gen_pycompat_corpus.py",
        "# with " + ", ".join(f"Python {v}" for v, _ in runs) + "; the interpreters agree on every case",
    ] + runs[0][1]
    if args.check is None:
        print("\n".join(lines))
        return
    with open(args.check, encoding="utf-8") as fh:
        have = [l for l in fh.read().splitlines() if not l.startswith("#")]
    want = [l for l in lines if not l.startswith("#")]
    if have != want:
        sys.exit(f"{args.check} differs from the generated corpus ({len(set(want) - set(have))} cases changed)")
    print(f"{args.check}: {len(want)} cases agree")


if __name__ == "__main__":
    main()
