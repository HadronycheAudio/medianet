#!/usr/bin/env python3
r"""Generate the committed proof tables of doccheck::family002.

The epic-002 check scripts (checks/002-*.sh) are bash: GNU grep (ERE and
BRE), sed, awk (mawk on this machine), tr, cut, sort, wc and bash's own
pattern matching, run in the locale of the environment (no script sets
LC_ALL or LANG). The Rust module `doccheck::family002` reproduces those
semantics as named helpers. This generator runs the REAL tools on a corpus
and writes what they answered; the Rust unit tests
(`family002::proof`) compare the helpers with these tables without running
any tool. The tool versions and the locale are recorded in each header.

Files written (under --out, default hadronyche-tools/data/family002):

  ranges.tsv    per digit range lo-hi (and a-z, A-Z): its members under
                gnulib's DFA and under glibc's regex matcher.
  chars.tsv     per character of the repertoire (data/repertoire.tsv), minus
                newline (no tool sees a newline inside a line): its class
                memberships for grep -E (with and without -i), bash patterns,
                tr, mawk and cut. Columns are documented in the header.
  patterns.tsv  every (tool, flags, pattern) the corpus proves, one per line:
                the list a port's patterns must belong to
                (`family002::unproven_patterns`).
  corpus.tsv    the inputs and, per probe, what the tool printed:
                  I <id> <line>              an input line
                  Q <key> <ids>              grep -q: the inputs that match
                                             (ranges "0-3,7")
                  O <key> <id> <m1 US m2..>  grep -o: the matches of one input
                                             (only inputs with a match)
                  SD <key> <digests>         sed: the outputs of every input,
                                             one FNV-1a 64 digest per chunk
                                             of 64 (each output + \x1e)
                  HD <snippet> <ids> <dig,..> a transform's outputs on those
                                             inputs, digested as for SD
                  HC <snippet> <ids> <v,..>  a counter's output per input
                  F <id> <text>              a whole-file fixture
                  R <probe> <fixture> <out>  a file-level probe's output
                Strings are escaped: \\ \t \n \r and \xHH for other control
                bytes and invalid UTF-8; US (\x1f) is written \x1f.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_family002.py [--locale L] [--commit C] [--out DIR]
  python3 hadronyche-tools/tools/gen_family002.py --check   # regenerate in memory, compare

--locale defaults to the environment's (LC_ALL, else LC_CTYPE, else LANG);
the tables say which. --commit (default HEAD) is the commit whose
REPORT-0b.md lines seed the corpus. --check regenerates and compares the
rows (not the version comments) with the committed files: exit 1 on a
difference.

Adding a pattern: append it to PATTERNS (with the script it comes from),
regenerate, run `cargo test -p hadronyche-tools family002`. A pattern the
Rust translator cannot reproduce fails that test with the inputs it
disagrees on.
"""

import argparse
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
REPO = os.path.dirname(CRATE)
REPORT = "doc/design/reports/archaeology/REPORT-0b.md"

# ----------------------------------------------------------------- patterns
# (tool+flags, pattern, origin). Tool+flags is one of:
#   grep-E grep-Ei grep-G grep-Gi grep-F grep-Fi   (match per line: -q/-c/-n)
#   grep-Eo grep-Eio grep-Go                      (grep -o: matches per line)
#   sed-E                                          (sed -E script, per line)
PATTERNS = [
    # 002-02
    ("grep-Ei", r"^#+[[:space:]]*\(d\)", "002-02 heading_no (d)"),
    ("grep-Ei", r"^#+[[:space:]]*\(e\)", "002-02 heading_no (e)"),
    ("grep-Ei", r"^#+[[:space:]]*Open sub-questions", "002-02 DoD heading"),
    ("grep-E", r"^#+[[:space:]]", "002-02 heading line"),
    ("grep-G", r"[^[:space:]]", "002-02 AC1 non-empty"),
    ("grep-E", r"audio\.rs:[0-9]+", "002-02 pointer"),
    ("grep-Eo", r"audio\.rs:[0-9]+", "002-02 AC1 pointers"),
    ("grep-G", r".", "002-02 grep -c ."),
    ("grep-Ei", r"toleran|threshold|epsilon|eps\b", "002-02 AC2 tolerance"),
    ("grep-Ei", r"1024|1[ ,]024", "002-02 AC3 fact 1"),
    ("grep-Ei", r"window", "002-02 AC3 fact 1"),
    ("grep-Ei", r"complete", "002-02 AC3 fact 2"),
    ("grep-Ei", r"both", "002-02 AC3 fact 2"),
    ("grep-Ei", r"256|MAX_LAG_WINDOWS", "002-02 AC3 fact 3"),
    ("grep-Ei", r"2[.,]7", "002-02 AC3 fact 3"),
    ("grep-Ei", r"settle|settling", "002-02 AC3 fact 4"),
    ("grep-Ei", r"(1[ -]?s|1 second|1-second)", "002-02 AC3 fact 4"),
    ("grep-Ei", r"(3[ -]?s|3 second|3-second|SETTLE_SECS)", "002-02 AC3 fact 4"),
    ("grep-Ei", r"replica", "002-02 AC3 fact 5"),
    ("grep-Ei", r"gap", "002-02 AC3 fact 5"),
    ("grep-Ei", r"(^|[^[:alnum:]])(confirmed|corrected)([^[:alnum:]]|$)", "002-02 marker"),
    ("grep-F", r"[code]", "002-02 AC3 tag"),
    ("grep-E", r"^[[:space:]]*(-|\*|\+|>|[0-9]+[.)])[[:space:]]|^[[:space:]]*\|", "002-02 AC4 list entry"),
    ("grep-Ei", r"incomplete", "002-02 AC4 category"),
    ("grep-Ei", r"lag|MAX_LAG_WINDOWS", "002-02 AC4 category"),
    ("grep-Ei", r"sender[ -]?gap", "002-02 AC4 category"),
    ("grep-Ei", r"settle|SETTLE_SECS", "002-02 AC4 category"),
    ("grep-Ei", r"voices[ -]?flag|voices.*off|flag off|voices=false|voices = false", "002-02 AC4 category"),
    ("grep-Ei", r"because|since|reason|so that|to avoid|otherwise", "002-02 AC4 reason"),
    ("sed-E", r"s/audio\.rs:[0-9]+//g; s/[0-9]+\.[0-9]+//g; s/\be\.g\.//g; s/\bi\.e\.//g", "002-02 AC5 clean"),
    ("sed-E", r"s/\[(code|measured|kernel-doc|memory|decision)\]//g", "002-02 AC5 tags"),
    ("grep-Eo", r"[.!?]", "002-02 AC5 terminals"),
    ("grep-G", r"[[:alnum:]]", "002-02 AC5 words"),
    ("grep-Gi", r"voice comparison bit-level", "002-02 DoD row"),
    ("grep-E", r"^[[:space:]]*\|", "002-02 DoD table row"),
    ("grep-Gi", r"answered", "002-02 DoD answered"),
    ("grep-Ei", r"(^|[^[:alnum:]])(yes|no)([^[:alnum:]]|$)", "002-02 DoD yes/no"),
]

# Bash snippets proven per input line: the line is in "$line"; the output
# is what the snippet prints. Each is the script's own code.
SNIPPETS = {
    # 002-02 normalise(), verbatim
    "normalise": r'''s="$line"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s#- }"; s="${s#\* }"; s="${s#+ }"; s="${s#> }"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s#\*\*}"; s="${s%\*\*}"
  s="${s#"${s%%[![:space:]]*}"}"
  printf '%s' "$s"''',
    "tr_lower": r'''printf '%s' "$line" | tr '[:upper:]' '[:lower:]' ''',
    "rstrip_space": r'''nl="$line"; nl="${nl%"${nl##*[![:space:]]}"}"; printf '%s' "$nl"''',
    "lstrip_space": r'''n="$line"; n="${n#"${n%%[![:space:]]*}"}"; printf '%s' "$n"''',
    "hash_run_len": r'''hashes=${line%%[! #]*}; printf '%s' "${#hashes}"''',
    "tr_delete_space": r'''printf '%s' "$line" | tr -d '[:space:]' ''',
    "word_count": r'''printf '%s' "$line" | tr -s '[:space:]' '\n' | grep -c '[[:alnum:]]' ''',
    "terminal_count": r'''printf '%s' "$line" | grep -oE '[.!?]' | wc -l | tr -d ' ' ''',
    "cut_110": r'''printf '%s' "$line" | cut -c1-110''',
}

# File-level probes: "$f" is the fixture file. Output records are
# separated by \x1e.
FILE_PROBES = {
    "grep_lines": r'''grep -n '' "$f" | while IFS= read -r l || [ -n "$l" ]; do printf '%s\x1e' "$l"; done''',
    "line_count": r'''wc -l < "$f" | tr -d ' ' ''',
    "read_loop_from_2": r'''while IFS= read -r ln; do printf '%s\x1e' "$ln"; done < <(sed -n "2,\$p" "$f")''',
    "read_loop_from_1": r'''while IFS= read -r ln; do printf '%s\x1e' "$ln"; done < <(sed -n "1,\$p" "$f")''',
    "subst_range_1_3": r'''SUB=$(sed -n "1,3p" "$f"); printf '%s' "$SUB"''',
    "subst_range_2_9": r'''SUB=$(sed -n "2,9p" "$f"); printf '%s' "$SUB"''',
    "subst_line_2": r'''l=$(sed -n "2p" "$f"); printf '%s' "$l"''',
    "heredoc_lines_all": r'''SUB=$(sed -n "1,99p" "$f")
while IFS= read -r ln; do printf '%s\x1e' "$ln"; done <<EOF
$SUB
EOF''',
    "heading_no_d": r'''grep -Ein '^#+[[:space:]]*\(d\)' "$f" | awk -F: '/^[0-9]+:#/ { print $1; exit }' ''',
    "heading_no_open": r'''grep -Ein '^#+[[:space:]]*Open sub-questions' "$f" | awk -F: '/^[0-9]+:#/ { print $1; exit }' ''',
    "count_nonempty_sub": r'''SUB=$(sed -n "1,99p" "$f"); printf '%s\n' "$SUB" | grep -c '[^[:space:]]' ''',
    "ptr_distinct": r'''SUB=$(sed -n "1,99p" "$f"); ptrs=$(printf '%s\n' "$SUB" | grep -oE 'audio\.rs:[0-9]+' | sort -u); printf '%s\n' "$ptrs" | grep -c . || true''',
}

FIXTURES = [
    "",
    "\n",
    "one\n",
    "one",
    "one\ntwo",
    "one\ntwo\n",
    "one\ntwo\n\n\n",
    "a\r\nb\r\nc\r\n",
    "a\r\nb\r\nc",
    "\n\nthird\n",
    "## (d) Reading notes\n### audio.rs\nx\n",
    "```\n## (d) in a fence\n```\n## (D) real\n",
    "text (d)\n# (d)\n",
    "##\t(d)\r\n",
    "  ## (d) indented\n## (d)\n",
    "## Open sub-questions\n| q | voice comparison bit-level | answered: yes |",
    "## Open sub-questions\n| q | voice comparison bit-level | answered: yes |\n",
    "## OPEN SUB-QUESTIONS\n",
    "audio.rs:1 audio.rs:01 audio.rs:1\naudio.rs:22\n\naudio.rs:333 x audio.rs:22\n",
    "AUDIO.RS:5\naudio.rs:6\r\n",
    "line with trailing spaces   \n\t\n \n",
    "x\ny\nz\nw\n",
]


def esc(b):
    """Escape bytes for a table field."""
    out = []
    try:
        s = b.decode("utf-8")
    except UnicodeDecodeError:
        return "".join(f"\\x{x:02x}" if (x < 0x20 or x >= 0x7F or x == 0x5C) and x not in (0x20,) else chr(x) for x in b)
    for c in s:
        o = ord(c)
        if c == "\\":
            out.append("\\\\")
        elif c == "\t":
            out.append("\\t")
        elif c == "\n":
            out.append("\\n")
        elif c == "\r":
            out.append("\\r")
        elif o < 0x20 or o == 0x7F:
            out.append(f"\\x{o:02x}")
        else:
            out.append(c)
    return "".join(out)


def ranges(ids):
    ids = sorted(ids)
    out, i = [], 0
    while i < len(ids):
        j = i
        while j + 1 < len(ids) and ids[j + 1] == ids[j] + 1:
            j += 1
        out.append(str(ids[i]) if i == j else f"{ids[i]}-{ids[j]}")
        i = j + 1
    return ",".join(out) or "-"


class Tools:
    def __init__(self, locale):
        self.env = dict(os.environ)
        self.env["LC_ALL"] = locale
        self.locale = locale
        self.bash = shutil.which("bash")

    def run(self, argv, data=b"", check=False):
        r = subprocess.run(argv, input=data, capture_output=True, env=self.env)
        if check and r.returncode not in (0, 1):
            sys.exit(f"{argv}: exit {r.returncode}: {r.stderr.decode(errors='replace')}")
        return r

    def versions(self):
        def first(argv):
            r = self.run(argv)
            return (r.stdout or r.stderr).decode(errors="replace").splitlines()[0].strip()

        awk = os.path.realpath(shutil.which("awk"))
        return [
            first(["grep", "--version"]),
            first(["sed", "--version"]),
            first([awk, "-W", "version"]) + f" ({awk})",
            first(["tr", "--version"]),
            first(["cut", "--version"]),
            first(["sort", "--version"]),
            first(["wc", "--version"]),
            first([self.bash, "--version"]),
        ]


def repertoire():
    rows = []
    with open(os.path.join(CRATE, "data", "repertoire.tsv"), encoding="utf-8") as f:
        for line in f:
            if line.startswith("#"):
                continue
            rows.append(chr(int(line.split("\t")[0], 16)))
    return [c for c in rows if c != "\n"]


CLASSES = ["alnum", "alpha", "digit", "space", "upper", "lower", "punct", "blank", "print", "graph", "cntrl", "xdigit"]
EXTRA = [("w", r"\w"), ("s", r"\s"), ("dot", "."), ("09", "[0-9]"), ("az", "[a-z]"), ("AZ", "[A-Z]")]


def char_facts(t, chars, tmp):
    path = os.path.join(tmp, "chars.txt")
    with open(path, "wb") as f:
        f.write(b"".join(c.encode() + b"\n" for c in chars))

    def members(flags, pat):
        r = t.run(["grep", "-n", flags, "-e", pat, path], check=True)
        return {int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l}

    cols = {}
    for k in CLASSES:
        cols["g_" + k] = members("-Ex", f"[[:{k}:]]")
        cols["gi_" + k] = members("-Exi", f"[[:{k}:]]")
    for name, pat in EXTRA:
        cols["g_" + name] = members("-Ex", pat)
        cols["gi_" + name] = members("-Exi", pat)
    fold = []
    for i, c in enumerate(chars):
        r = t.run(["grep", "-nixF", "-e", c, path], check=True)
        fold.append(sorted(int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l if int(l.split(b":")[0]) - 1 != i))
    script = r'''while IFS= read -r c || [ -n "$c" ]; do
  o=""
  for k in alnum alpha digit space upper lower punct blank; do
    case "$c" in [[:$k:]]) o="$o$k,";; esac
  done
  printf '%s\037%s\n' "$o" "${#c}"
done'''
    bash_out = t.run([t.bash, "-c", script], data=open(path, "rb").read()).stdout.split(b"\n")
    rows = []
    for i, c in enumerate(chars):
        b = c.encode()
        lower = t.run(["tr", "[:upper:]", "[:lower:]"], data=b).stdout
        deleted = t.run(["tr", "-d", "[:space:]"], data=b).stdout
        cut1 = t.run(["cut", "-c1-1"], data=b + b"\n").stdout[:-1]
        awk = t.run(["awk", '{ printf "%s\037%s\037%d\037%d\037%d\037%d", tolower($0), toupper($0), length($0), ($0 ~ /^[[:alnum:]]$/), ($0 ~ /^[[:space:]]$/), ($0 ~ /^.$/) }'], data=b + b"\n").stdout
        al, au, alen, aaln, asp, adot = awk.split(b"\x1f")
        bclasses, blen = bash_out[i].split(b"\x1f")
        rows.append(
            "\t".join(
                [
                    f"{ord(c):04X}",
                    "".join("1" if i in cols["g_" + k] else "0" for k in CLASSES + [n for n, _ in EXTRA]),
                    "".join("1" if i in cols["gi_" + k] else "0" for k in CLASSES + [n for n, _ in EXTRA]),
                    " ".join(f"{ord(chars[j]):04X}" for j in fold[i]) or "-",
                    bclasses.decode().rstrip(",") or "-",
                    blen.decode(),
                    esc(lower),
                    "1" if deleted == b"" else "0",
                    esc(cut1),
                    esc(al),
                    esc(au),
                    alen.decode(),
                    aaln.decode() + asp.decode() + adot.decode(),
                ]
            )
        )
    return rows


def range_facts(t, chars, tmp):
    """Per digit range lo-hi: the repertoire characters gnulib's DFA puts in
    it (the range alone, which the DFA supports) and those glibc's regex
    matcher puts in it: an alternative with a negated class makes the DFA
    unsupported and the superset DFA accept every line (the class becomes
    "any text"), so the regex matcher decides; the negated class itself
    matches no repertoire character."""
    path = os.path.join(tmp, "chars.txt")
    rows = []
    for lo in "0123456789":
        for hi in "0123456789":
            if hi < lo:
                continue
            out = []
            for pat in (f"[{lo}-{hi}]", f"[{lo}-{hi}]|[^[:print:][:cntrl:]]"):
                r = t.run(["grep", "-nEx", "-e", pat, path], check=True)
                ids = [int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l]
                out.append(" ".join(f"{ord(chars[i]):04X}" for i in ids) or "-")
            rows.append(f"{lo}-{hi}\t{out[0]}\t{out[1]}")
    for lo, hi in (("a", "z"), ("A", "Z")):
        out = []
        for pat in (f"[{lo}-{hi}]x|[{lo}-{hi}]", f"[{lo}-{hi}]|[^[:print:][:cntrl:]]"):
            r = t.run(["grep", "-nEx", "-e", pat, path], check=True)
            ids = [int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l]
            out.append(" ".join(f"{ord(chars[i]):04X}" for i in ids) or "-")
        rows.append(f"{lo}-{hi}\t{out[0]}\t{out[1]}")
    return rows


def inputs(commit):
    text = subprocess.run(["git", "show", f"{commit}:{REPORT}"], cwd=REPO, capture_output=True, check=True).stdout.decode()
    seen, out = set(), []

    def add(s):
        if s not in seen and "\n" not in s:
            seen.add(s)
            out.append(s)

    real = text.split("\n")
    for line in real:
        add(line[:140])
        if len(line) > 140:
            add(line[-90:])
    # variants of the lines the patterns aim at
    for line in real:
        if not line.strip():
            continue
        short = line[:160]
        if line.startswith("#") or line.lstrip().startswith(("-", "*", "|", ">", "1", "2", "3")):
            add(short.upper())
            add(short + "\r")
            add(short.replace(" ", "\t", 2))
            add("  " + short + "  ")
    # digits next to their superscript, subscript and fraction variants,
    # where the DFA and the regex matcher disagree on [0-9]
    variant = {"1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "9": "⁹"}
    sub = {"1": "₁", "2": "₂", "3": "₃"}
    digit_lines = [l[:120] for l in real if any(ch.isdigit() for ch in l)][:120]
    for line in digit_lines:
        add("".join(variant.get(ch, ch) for ch in line))
        add("".join(sub.get(ch, ch) for ch in line))
        add("".join(ch + variant.get(ch, "") for ch in line))
    for a in ["1", "6", "²", "⁹", "₃", "⅔", "⅓"]:
        for b in ["", "²", "⁴", "⁹", "₁", "⅓", "7"]:
            add(f"audio.rs:{a}{b}")
            add(f"see audio.rs:604 and audio.rs:{a}{b}")
            add(f"{a}.{b}5 1.5")
            add(f"{a}{b}.5")
            add(f"x {a}.5{b} and 2.7 s")
    keywords = set()
    for _, pat, _ in PATTERNS:
        for word in pat.replace("\\", " ").replace("|", " ").replace("(", " ").replace(")", " ").replace("^", " ").replace("$", " ").split():
            w = "".join(ch for ch in word if ch.isalnum() or ch in "._:-[]? ")
            if len(w) >= 2:
                keywords.add(w)
    keywords |= {"confirmed", "corrected", "yes", "no", "eps", "e.g.", "i.e.", "audio.rs:604", "audio.rs:", "[code]", "[measured]",
                 "Verdict statement:", "Voice comparison bit-level: yes", "voice comparison bit-level", "answered", "1.5", "2.7 s",
                 "1 s", "3 s", "1-second", "3-second", "SETTLE_SECS", "MAX_LAG_WINDOWS", "1 024", "1,024", "voices=false"}
    contexts = ["", " ", "_", "x", "X", "0", ".", "-", "|", "\t", "\r", "µ", "²", "Δ", "—"]
    for k in sorted(keywords):
        for c in contexts:
            add(c + k + " end")
            add("start " + k + c)
        add(k.upper())
        add(k.capitalize())
    hand = [
        "", " ", "\t", "\r", " \r", "#", "##", "# (d)", "## (d)", "##(d)", "## (D) x", "#\t(d)", " ## (d)", "## (d)\r", "x ## (d)",
        "## Open sub-questions", "## open SUB-QUESTIONS", "- item", "-item", "* item", "+ item", "> quote", "1. one", "12) two",
        "1.two", " | row |", "|", "\t- tab item", "-\titem", "**bold**", "** bold **", "- **Verdict statement:** x", "> - x",
        "*  x", "- - x", "-  - x", "Verdict statement: one two three four five six.", "Verdict statement:", "Verdict statement:   ",
        "a.b.c", "e.g. i.e. eg ie", "x e.g.", "xe.g.", "_e.g.", "µe.g.", "1.2.3", "3.14 and 2.71", "audio.rs:12.5",
        "[code][measured] [kernel-doc] [memory] [decision] [standard] [CODE]", "Done! Really? Yes.", "...", "?!",
        "yes", "no", "No.", "yes/no", "noyes", "eyes", "know", "no_", "_no", "µno", "²no", "Δno", "no²", "noµ",
        "confirmedly", "unconfirmed", "(confirmed)", "Confirmed.", "CORRECTED", "µconfirmed", "²confirmed", "confirmed²",
        "eps", "epsilon", "eps_", "epsµ", "eps²", "eps-", "Eps.", "EPS", "steps", "eps1",
        "1024", "1 024", "1,024", "1.024", "10240", "2.7", "2,7", "2;7", "256", "1s", "1 s", "1-s", "1  s", "3s", "13 s",
        "voices flag", "voices-flag", "voicesflag", "voices are off", "flag off", "voices = false", "voices=false", "VOICES OFF",
        "sender gap", "sender-gap", "sendergap", "sender  gap", "lag", "flag", "MAX_LAG_WINDOWS", "settle", "Settling",
        "because", "since", "reason", "so that", "to avoid", "otherwise", "so  that",
        "voice comparison bit-level", "Voice Comparison Bit-Level", "voice comparison bit level", "answered", "ANSWERED",
        "x\ty", "a  b", "trailing   ", "\ttab lead", "§ ± × ÷ – — ‖ … ‰ ⅓ ⅔ → ↔ ⇒ ∈ − ∞ ≈ ≠ ≡ ≤ ≥ ⌈ ⌉ ⌊ ⌋ · ⁻ ₁₂₃ ⁴⁵⁶⁹ ²³¹",
        "µs Δt Σx α π τ", "ΔΣαπτµ", "a²b", "word² word", "² ³ ¹", "x — y", "→", "Ünïcödé-free", "back\\slash", "a\\b",
        "\\.", "[.]", "a{b", "{", "}", "$", "^", "a$b", "a^b", "( ) [ ] { } * + ? | \\ / . ^ $",
    ]
    for h in hand:
        add(h)
    return out


def grep_flags(tool):
    kind = tool.split("-")[1]
    flags = ["-" + kind[0]]
    if "i" in kind[1:]:
        flags.append("-i")
    if "o" in kind[1:]:
        flags.append("-o")
    return flags


def corpus(t, lines, tmp):
    path = os.path.join(tmp, "inputs.txt")
    with open(path, "wb") as f:
        f.write(b"".join(l.encode() + b"\n" for l in lines))
    out = [f"I\t{i}\t{esc(l.encode())}" for i, l in enumerate(lines)]
    for tool, pat, _ in PATTERNS:
        key = f"{tool}\t{esc(pat.encode())}"
        if tool.startswith("grep") and "o" not in tool.split("-")[1][1:]:
            r = t.run(["grep", "-n"] + grep_flags(tool) + ["-e", pat, path], check=True)
            ids = {int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l}
            out.append(f"Q\t{key}\t{ranges(ids)}")
        elif tool.startswith("grep"):
            r = t.run(["grep", "-n"] + grep_flags(tool) + ["-e", pat, path], check=True)
            per = {}
            for l in r.stdout.split(b"\n"):
                if not l:
                    continue
                n, _, m = l.partition(b":")
                per.setdefault(int(n) - 1, []).append(m)
            for i in sorted(per):
                out.append(f"O\t{key}\t{i}\t" + "\\x1f".join(esc(m) for m in per[i]))
        elif tool == "sed-E":
            r = t.run(["sed", "-E", pat, path], check=True)
            got = r.stdout.split(b"\n")
            if got and got[-1] == b"":
                got.pop()
            assert len(got) == len(lines), (pat, len(got), len(lines))
            out.append(f"SD\t{key}\t{digests(got)}")
        else:
            sys.exit(f"unknown tool {tool}")
    return out


COUNTERS = {"hash_run_len", "word_count", "terminal_count"}


def snippets(t, lines, tmp):
    """Transforms: an H row where the output differs from the input, and one
    HS row naming the inputs run. Counters: one HC row with every output."""
    path = os.path.join(tmp, "snip.txt")
    ids = [i for i, l in enumerate(lines) if len(l) <= 200]
    with open(path, "wb") as f:
        f.write(b"".join(lines[i].encode() + b"\n" for i in ids))
    out = []
    for name, body in SNIPPETS.items():
        script = f'while IFS= read -r line; do\n{body}\nprintf "\\x1e\\n"\ndone < "$1"\n'
        r = t.run([t.bash, "-c", script, "snip", path], check=False)
        got = r.stdout.split(b"\x1e\n")
        if got and got[-1] == b"":
            got.pop()
        assert len(got) == len(ids), (name, len(got), len(ids), r.stderr[:300])
        if name in COUNTERS:
            out.append(f"HC\t{name}\t{ranges(ids)}\t" + ",".join(g.decode().strip() for g in got))
            continue
        out.append(f"HD\t{name}\t{ranges(ids)}\t" + digests(got))
    return out


def fnv(data):
    """FNV-1a, 64 bits."""
    h = 0xCBF29CE484222325
    for b in data:
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def digests(outputs):
    """One FNV-1a digest per chunk of CHUNK outputs, each output followed by
    \x1e: a difference is located to its chunk without committing every
    output."""
    out = []
    for k in range(0, len(outputs), CHUNK):
        chunk = b"".join(o + RS for o in outputs[k : k + CHUNK])
        out.append(f"{fnv(chunk):016x}")
    return ",".join(out)


CHUNK = 64
RS = b"\x1e"


def files(t, tmp):
    out = [f"F\t{i}\t{esc(x.encode())}" for i, x in enumerate(FIXTURES)]
    for i, text in enumerate(FIXTURES):
        f = os.path.join(tmp, f"fixture{i}.md")
        with open(f, "wb") as h:
            h.write(text.encode())
        for name, body in FILE_PROBES.items():
            r = t.run([t.bash, "-c", body.replace('"$f"', '"$1"'), "probe", f])
            out.append(f"R\t{name}\t{i}\t{esc(r.stdout)}")
    return out


def generate(locale, commit):
    t = Tools(locale)
    with tempfile.TemporaryDirectory(prefix="gen-family002-") as tmp:
        chars = repertoire()
        crows = char_facts(t, chars, tmp)
        rrows = range_facts(t, chars, tmp)
        lines = inputs(commit)
        rows = corpus(t, lines, tmp) + snippets(t, lines, tmp) + files(t, tmp)
    head = [f"# locale {locale}; inputs seeded from {REPORT} at {commit}"] + [f"# tool: {v}" for v in t.versions()]
    chars_tsv = head + [
        "# cp\tgrep -E classes\tgrep -Ei classes\tgrep -iF partners\tbash classes\tbash ${#c}\ttr upper->lower\ttr -d [:space:] deletes\tcut -c1-1\tawk tolower\tawk toupper\tawk length\tawk alnum,space,dot",
        "# class columns, one digit each: " + " ".join(CLASSES + [n for n, _ in EXTRA]),
    ] + crows
    patterns_tsv = head + ["# tool\tpattern\torigin"] + [f"{tool}\t{esc(p.encode())}\t{o}" for tool, p, o in PATTERNS]
    corpus_tsv = head + rows
    ranges_tsv = head + ["# range\tgnulib DFA members\tglibc regex members (code points)"] + rrows
    return {"chars.tsv": chars_tsv, "patterns.tsv": patterns_tsv, "corpus.tsv": corpus_tsv, "ranges.tsv": ranges_tsv}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    env_locale = os.environ.get("LC_ALL") or os.environ.get("LC_CTYPE") or os.environ.get("LANG") or "C"
    ap.add_argument("--locale", default=env_locale)
    ap.add_argument("--commit", default="HEAD")
    ap.add_argument("--out", default=os.path.join(CRATE, "data", "family002"))
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    if a.check and a.commit == "HEAD":
        with open(os.path.join(a.out, "corpus.tsv"), encoding="utf-8") as f:
            a.commit = f.readline().split(" at ")[1].strip()
    commit = subprocess.run(["git", "rev-parse", "--short=12", a.commit], cwd=REPO, capture_output=True, check=True, text=True).stdout.strip()
    tables = generate(a.locale, commit)
    if a.check:
        bad = 0
        for name, rows in tables.items():
            with open(os.path.join(a.out, name), encoding="utf-8") as f:
                have = [l.rstrip("\n") for l in f if not l.startswith("#")]
            want = [r for r in rows if not r.startswith("#")]
            if have != want:
                bad = 1
                print(f"{name}: differs ({len(have)} rows committed, {len(want)} regenerated)")
        sys.exit(bad)
    os.makedirs(a.out, exist_ok=True)
    for name, rows in tables.items():
        with open(os.path.join(a.out, name), "w", encoding="utf-8", newline="\n") as f:
            f.write("\n".join(rows) + "\n")
        print(f"{name}: {len(rows)} lines", file=sys.stderr)


if __name__ == "__main__":
    main()
