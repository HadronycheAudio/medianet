#!/usr/bin/env bash
# 004-28 — regime stamp sections: observed DSCP of peers and the alignment group.
#
# RED-phase document check (tester). It validates the two section types that the
# "Regime stamp" section of doc/design/spec/wire.md lays out (type codes 16 to 31, the
# block the merged section type codes table gives 004-28) and their regime stamp
# messages in doc/design/spec/wire.toml against the acceptance criteria of the ticket,
# and re-runs the wire-check binary and the wire_tables tests when the paths are not
# overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing on
# throwaway fixtures only: when either path is overridden the cargo checks are skipped
# with a NOTE, because they always target the real repository.
#
# The author contract (what the checker locates; nothing else is fixed):
#   ## Regime stamp            two ### subsections: one whose heading names the observed DSCP
#                              of peers (dscp), one whose heading names the alignment group
#                              (alignment); the placeholder is gone
#   each new subsection        its first field table (Offset | Width | Type | Meaning | Valid range)
#                              is the section table; a list row ("E × max F") is followed, depth-first,
#                              by its element table; the section maximum is 58 + body bytes; a
#                              "Section type code N" sentence with N in the block of 004-28; a sentence
#                              "maximum ... N bytes" equal to that maximum; one statement ⌈M / F⌉ = N per
#                              list row; evidence tags on the subsection and on each prose paragraph of
#                              5 or more words
#   invalid forms              a table "Invalid form | Required outcome" inside each new subsection
#   wire.toml                  one [[message]] per new section type with section_of = "regime stamp", and one
#                              entry message per element table, rows agreeing bit for bit
#   budget                     the threshold and the total of sections are stated; an overflow is reported
#
# What is checked, per acceptance criterion:
#   AC 1  two section types, each with a type code in the block of 004-28 and distinct from 004-25's, sizes
#         within 1 252 bytes, the section count arithmetic, the budget (fit with arithmetic, or the complete
#         overflow report)
#   AC 2  the observed DSCP section: the 64-bit module id, the hello and sync DSCP, the not-observed encoding,
#         the dropped-request counter with its width and one wrap or saturation rule, at most 63 entries
#   AC 3  the alignment section: the health key widths, exactly the four C-160 figures with widths and units,
#         the maximum since live, the reclaimable latency bounded by A, the rolling 60 s window (C-182)
#   AC 4  at least five invalid forms with outcomes naming control reasons; the outcomes Work 3 fixes
#   AC 5  wire.toml agrees with the tables; wire-check clean; wire_tables 0 failed, at least the merged count
#   DoD   evidence tags, owners and falsifiers, merged tables unchanged, no ref to the main branch,
#         no sibling check run
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
MAIN_WORD=main
SIBLING_PATTERN="checks/[0-9]"
rc=0
if grep -nE "git.*${MAIN_WORD}" "$0" >/dev/null; then
    echo "FAIL DoD: the checker names a repository ref containing main; it must read the working tree only"
    rc=1
else
    echo "PASS DoD: the checker names no repository ref containing main"
fi
if grep -nE "${SIBLING_PATTERN}" "$0" >/dev/null; then
    echo "FAIL DoD: the checker invokes or names a sibling check; it must run no other check"
    rc=1
else
    echo "PASS DoD: the checker runs no sibling check"
fi

python3 - "$WIRE" "$TABLES" <<'PY'
import os
import re
import subprocess
import sys

wire_path, tables_path = sys.argv[1], sys.argv[2]
overridden = bool(os.environ.get("WIRE_MD") or os.environ.get("WIRE_TOML"))
CONSTANTS_PATH = "doc/design/02-constants.md"
RESULTS = []
NBSP = "  "


def check(ac, ok, text):
    RESULTS.append((ac, bool(ok)))
    print(("PASS " if ok else "FAIL ") + ac + ": " + text)
    return bool(ok)


def fail(ac, msg):
    return check(ac, False, msg)


def norm(text):
    # Digit groups written with a space or a no-break space: "1 252" reads as "1252".
    return re.sub(r"(?<=\d)[ " + NBSP + r"](?=\d{3}(?!\d))", "", text)


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


NUM = r"(?:\d{1,3}(?:[  ]\d{3})+|\d+)"


def to_int(s):
    return int(re.sub(r"[  ]", "", s))


# ---------------------------------------------------------------------------
# Markdown helpers (conventions of the earlier 004 checkers)
# ---------------------------------------------------------------------------
def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    cells = re.split(r"(?<!\\)\|", s)
    return [c.strip().replace("\\|", "|") for c in cells]


def parse_tables(text):
    """[(start_line, header_cells, data_rows)] for Markdown tables outside fences."""
    lines = text.splitlines()
    tables = []
    in_fence = False
    i = 0
    while i < len(lines):
        stripped = lines[i].lstrip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            i += 1
            continue
        if not in_fence and stripped.startswith("|"):
            start = i
            block = []
            j = i
            while j < len(lines) and lines[j].lstrip().startswith("|"):
                block.append(lines[j])
                j += 1
            if len(block) >= 2 and re.match(r"^\s*\|?[\s:|-]+\|?\s*$", block[1]) and "-" in block[1]:
                tables.append((start, split_row(block[0]), [split_row(r) for r in block[2:]]))
            i = j
        else:
            i += 1
    return tables


def heading_index(lines):
    """[(line_no, level, title)] for headings outside fences."""
    out = []
    in_fence = False
    for idx, line in enumerate(lines):
        if re.match(r"^\s*```", line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
        if m:
            out.append((idx, len(m.group(1)), m.group(2).strip()))
    return out


def strip_fences(text):
    out = []
    in_fence = False
    for line in text.splitlines():
        if re.match(r"^\s*```", line):
            in_fence = not in_fence
            out.append("")
            continue
        out.append("" if in_fence else line)
    return "\n".join(out)


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(text):
    return bool(EVIDENCE.search(text))


def prose_blocks(text):
    """[(kind, text)] with kind 'p' (prose) or 't' (table); headings and blank lines split blocks."""
    out = []
    cur = []
    kind = None
    for line in strip_fences(text).splitlines():
        if re.match(r"^#{1,6}\s", line) or not line.strip():
            if cur:
                out.append((kind, "\n".join(cur)))
            cur, kind = [], None
            continue
        k = "t" if line.lstrip().startswith("|") else "p"
        if kind is not None and k != kind:
            out.append((kind, "\n".join(cur)))
            cur = []
        kind = k
        cur.append(line)
    if cur:
        out.append((kind, "\n".join(cur)))
    return out


def clean_start(text):
    return re.sub(r"^[\s*_`>-]+", "", text)


def is_field_table(header):
    h = [re.sub(r"[^a-z0-9]", "", x.lower()) for x in header]
    return len(h) >= 5 and h[0].startswith("offset") and "width" in h[1] and "type" in h[2] \
        and "meaning" in h[3] and "valid" in h[4]


def width_of(cell):
    """(bits, max_count): 'N' or 'N bytes' -> N*8; 'N bits' -> N; 'E × max M' -> (E, M)."""
    s = cell.strip().replace("`", "").replace("*", "").replace("X", "x")
    m = re.fullmatch(r"(\d+)\s*(bytes?|bits?)?\s*(?:x|×)\s*max\s*(\d+)", s, re.I)
    if m:
        unit = (m.group(2) or "bytes").lower()
        return int(m.group(1)) * (1 if unit.startswith("bit") else 8), int(m.group(3))
    m = re.fullmatch(r"(\d+)\s*(bytes?|bits?)?", s, re.I)
    if m:
        unit = (m.group(2) or "bytes").lower()
        return int(m.group(1)) * (1 if unit.startswith("bit") else 8), None
    return None, None


def size_bits(f):
    return f["bits"] * (f["maxc"] if f["maxc"] else 1)


def fields_of(table, ac, where):
    """Field rows of a five-column field table: dicts with off (bytes), bits, maxc, type, meaning, valid."""
    out = []
    if table is None:
        return out
    _, header, rows = table
    if not is_field_table(header):
        fail(ac, f"{where}: the table does not have the five columns Offset, Width, Type, Meaning, Valid range")
        return out
    for r in rows:
        if len(r) < 5:
            fail(ac, f"{where}: a row has fewer than five cells: {r[:2]}")
            continue
        if not re.fullmatch(r"\d+", r[0].strip()):
            fail(ac, f"{where}: the Offset cell '{r[0]}' is not a byte offset")
            continue
        bits, maxc = width_of(r[1])
        if bits is None:
            fail(ac, f"{where}: the Width cell '{r[1]}' is neither a byte count nor 'E × max M'")
            continue
        out.append({"off": int(r[0]), "bits": bits, "maxc": maxc, "type": r[2].strip(),
                    "meaning": r[3].strip(), "valid": r[4].strip()})
    return out


def subsections(sec_lines):
    hs = heading_index(sec_lines)
    out = []
    for k, (idx, lvl, title) in enumerate(hs):
        if lvl != 3:
            continue
        end = len(sec_lines)
        for j, l2, _ in hs[k + 1:]:
            if l2 <= 3:
                end = j
                break
        out.append((title, "\n".join(sec_lines[idx + 1:end])))
    return out


def find_sub(subs, pattern):
    return [(t, x) for t, x in subs if re.search(pattern, t, re.I)]


def first_table(text, pred=lambda h: True):
    for t in parse_tables(text):
        if pred(t[1]):
            return t
    return None




# A small TOML reader (copied from the earlier 004 checkers: offline, Python 3.10, no tomllib).
class TomlError(Exception):
    pass


def _scan_statements(src):
    stmts = []
    buf = []
    i = 0
    n = len(src)
    in_str = None
    depth = 0
    while i < n:
        c = src[i]
        if in_str:
            if in_str in ('"', '"""') and c == "\\":
                buf.append(src[i : i + 2])
                i += 2
                continue
            if src.startswith(in_str, i):
                buf.append(src[i : i + len(in_str)])
                i += len(in_str)
                in_str = None
                continue
            buf.append(c)
            i += 1
            continue
        if c == "#":
            j = src.find("\n", i)
            if j == -1:
                break
            i = j
            continue
        if c in "\"'":
            if src.startswith(c * 3, i):
                in_str = c * 3
                buf.append(c * 3)
                i += 3
                continue
            in_str = c
            buf.append(c)
            i += 1
            continue
        if c in "[{":
            depth += 1
        elif c in "]}":
            depth = max(0, depth - 1)
        if c == "\n":
            if depth == 0:
                s = "".join(buf).strip()
                if s:
                    stmts.append(s)
                buf = []
            else:
                buf.append(" ")
            i += 1
            continue
        buf.append(c)
        i += 1
    s = "".join(buf).strip()
    if s:
        stmts.append(s)
    if in_str:
        raise TomlError("unterminated string")
    return stmts


def _split_dotted(key):
    parts = []
    cur = []
    quote = None
    for c in key:
        if quote:
            if c == quote:
                quote = None
            cur.append(c)
        elif c in "\"'":
            quote = c
            cur.append(c)
        elif c == ".":
            parts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
    parts.append("".join(cur).strip())
    out = []
    for p in parts:
        if len(p) >= 2 and p[0] == p[-1] and p[0] in "\"'":
            p = p[1:-1]
        if not p:
            raise TomlError("empty key component")
        out.append(p)
    return out


class _ValueParser:
    def __init__(self, text):
        self.t = text
        self.i = 0

    def skip_ws(self):
        while self.i < len(self.t) and self.t[self.i] in " \t\r\n":
            self.i += 1

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else ""

    def parse(self):
        self.skip_ws()
        c = self.peek()
        if c == "":
            raise TomlError("missing value")
        if c == '"':
            return self.basic_string()
        if c == "'":
            return self.literal_string()
        if c == "[":
            return self.array()
        if c == "{":
            return self.inline_table()
        return self.bare()

    def basic_string(self):
        self.i += 1
        out = []
        while self.i < len(self.t):
            c = self.t[self.i]
            self.i += 1
            if c == '"':
                return "".join(out)
            if c == "\\":
                if self.i >= len(self.t):
                    break
                e = self.t[self.i]
                self.i += 1
                simple = {
                    "n": "\n", "t": "\t", "r": "\r", "b": "\b", "f": "\f",
                    '"': '"', "\\": "\\", "/": "/",
                }
                if e == "u":
                    out.append(chr(int(self.t[self.i : self.i + 4], 16)))
                    self.i += 4
                elif e == "U":
                    out.append(chr(int(self.t[self.i : self.i + 8], 16)))
                    self.i += 8
                else:
                    out.append(simple.get(e, e))
            else:
                out.append(c)
        raise TomlError("unterminated basic string")

    def literal_string(self):
        self.i += 1
        j = self.t.find("'", self.i)
        if j == -1:
            raise TomlError("unterminated literal string")
        s = self.t[self.i : j]
        self.i = j + 1
        return s

    def array(self):
        self.i += 1
        out = []
        while True:
            self.skip_ws()
            if self.peek() == "]":
                self.i += 1
                return out
            if self.peek() == "":
                raise TomlError("unterminated array")
            out.append(self.parse())
            self.skip_ws()
            if self.peek() == ",":
                self.i += 1
                continue
            if self.peek() == "]":
                self.i += 1
                return out
            raise TomlError("expected ',' or ']' in array")

    def inline_table(self):
        self.i += 1
        out = {}
        while True:
            self.skip_ws()
            if self.peek() == "}":
                self.i += 1
                return out
            start = self.i
            quote = None
            while self.i < len(self.t):
                c = self.t[self.i]
                if quote:
                    if c == quote:
                        quote = None
                elif c in "\"'":
                    quote = c
                elif c == "=":
                    break
                self.i += 1
            if self.peek() != "=":
                raise TomlError("expected '=' in inline table")
            key = self.t[start : self.i].strip()
            self.i += 1
            value = self.parse()
            parts = _split_dotted(key)
            target = out
            for p in parts[:-1]:
                target = target.setdefault(p, {})
            target[parts[-1]] = value
            self.skip_ws()
            if self.peek() == ",":
                self.i += 1
                continue
            if self.peek() == "}":
                self.i += 1
                return out
            raise TomlError("expected ',' or '}' in inline table")

    def bare(self):
        start = self.i
        while self.i < len(self.t) and self.t[self.i] not in ",]}\n":
            self.i += 1
        tok = self.t[start : self.i].strip()
        if not tok:
            raise TomlError("empty value")
        low = tok.lower()
        if low == "true":
            return True
        if low == "false":
            return False
        t = tok.replace("_", "")
        try:
            if t.lower().startswith("0x"):
                return int(t, 16)
            if t.lower().startswith("0o"):
                return int(t, 8)
            if t.lower().startswith("0b"):
                return int(t, 2)
            if re.fullmatch(r"[+-]?\d+", t):
                return int(t)
            return float(t)
        except ValueError:
            return tok


def parse_toml(src):
    root = {}
    current = root
    for stmt in _scan_statements(src):
        if stmt.startswith("[[") and stmt.endswith("]]"):
            path = _split_dotted(stmt[2:-2])
            target = root
            for p in path[:-1]:
                nxt = target.setdefault(p, {})
                if isinstance(nxt, list):
                    nxt = nxt[-1]
                target = nxt
            last = path[-1]
            arr = target.setdefault(last, [])
            if not isinstance(arr, list):
                raise TomlError(f"[[{'.'.join(path)}]] is not an array of tables")
            entry = {}
            arr.append(entry)
            current = entry
            continue
        if stmt.startswith("[") and stmt.endswith("]"):
            path = _split_dotted(stmt[1:-1])
            target = root
            for p in path:
                nxt = target.setdefault(p, {})
                if isinstance(nxt, list):
                    nxt = nxt[-1]
                target = nxt
            current = target
            continue
        quote = None
        eq = -1
        for k, c in enumerate(stmt):
            if quote:
                if c == quote:
                    quote = None
            elif c in "\"'":
                quote = c
            elif c == "=":
                eq = k
                break
        if eq == -1:
            raise TomlError(f"not a key/value statement: {stmt[:60]!r}")
        key = stmt[:eq].strip()
        parser = _ValueParser(stmt[eq + 1 :])
        value = parser.parse()
        parser.skip_ws()
        if parser.i != len(parser.t):
            raise TomlError(f"trailing text after value: {parser.t[parser.i:]!r}")
        parts = _split_dotted(key)
        target = current
        for p in parts[:-1]:
            target = target.setdefault(p, {})
        target[parts[-1]] = value
    return root

# ---------------------------------------------------------------------------
# Inputs and the numbers the constants table gives (read at run time)
# ---------------------------------------------------------------------------
def span_of(pattern, level):
    for i, (idx, lvl, title) in enumerate(HEADS):
        if lvl == level and re.search(pattern, title, re.I):
            end = len(lines)
            for j in range(i + 1, len(HEADS)):
                if HEADS[j][1] <= lvl:
                    end = HEADS[j][0]
                    break
            return idx, end
    return None


def body(span):
    return lines[span[0]:span[1]] if span else []


def cdiv(a, b):
    return -(-a // b)


# ---------------------------------------------------------------------------
# Inputs: the wire file, the machine-readable tables and the constants (read at run time)
# ---------------------------------------------------------------------------
WIRE_RAW = read_text(wire_path)
if WIRE_RAW is None:
    fail("AC1", f"{wire_path} does not exist")
    WIRE_RAW = ""
WIRE = norm(WIRE_RAW.replace("−", "-"))
lines = WIRE.splitlines()
HEADS = heading_index(lines)
TOML_TEXT = read_text(tables_path)
if TOML_TEXT is None:
    fail("AC5", f"{tables_path} does not exist")
    TOML_TEXT = ""
CONST = norm(read_text(CONSTANTS_PATH) or "")


def const_row(cid):
    for line in CONST.splitlines():
        if line.startswith(f"| {cid} |"):
            cells = line.split("|")
            return cells[3].strip() if len(cells) > 3 else ""
    return ""


def const_int(cid, pattern, group=1):
    m = re.search(pattern, const_row(cid))
    return int(m.group(group)) if m else None


CTRL_BOUND = const_int("C-17", r"at most (\d+) bytes")
MAX_MODULES = const_int("C-34", r"at most (\d+)")
MAX_FOLLOWERS = const_int("C-114", r"holder has at most (\d+) real followers")
MAX_PEERS = const_int("C-217", r"(\d+) peers")
DSCP_CONTROL = const_int("C-13", r"control CS5 \((\d+)\)")
ACCEPT_HI = const_int("C-112", r"(\d+) to (\d+) per second", 2)
WINDOW_S = const_int("C-182", r"rolling (\d+) s window")
MAX_STREAMS = const_int("C-09", r"at most (\d+)")
MAX_MEMBERS = const_int("C-08", r"at most (\d+)")
PEER_MAX = MAX_MODULES - 1 if MAX_MODULES is not None else None
check("AC1", CTRL_BOUND == 1252, f"the constants table gives the control bound C-17 as 1 252 bytes (read {CTRL_BOUND})")
check("AC2", PEER_MAX is not None and PEER_MAX == MAX_PEERS == MAX_FOLLOWERS,
      f"the constants give 63 peers and 63 followers: 64 modules (C-34) less the module itself (C-217), "
      f"and at most 63 real followers (C-114) (read {MAX_MODULES}, {MAX_PEERS}, {MAX_FOLLOWERS})")
check("AC2", DSCP_CONTROL == 40, f"the constants give control CS5 as 40 (C-13) (read {DSCP_CONTROL})")
check("AC3", WINDOW_S == 60, f"the constants give the lateness window as a rolling 60 s window (C-182) (read {WINDOW_S})")
check("AC2", ACCEPT_HI == 200, f"the constants give the accepted sync request rate as 10 to 200 per second (C-112) (read {ACCEPT_HI})")

# ---------------------------------------------------------------------------
# Merged text that must not change (Regime stamp, Rejection reasons, Reserved fields)
# ---------------------------------------------------------------------------
STAMP_SPAN = span_of(r"^regime\s+stamp$", 2)
STAMP_LINES = body(STAMP_SPAN)
STAMP = "\n".join(STAMP_LINES)
check("AC1", STAMP_SPAN is not None and "to be filled" not in STAMP.lower(),
      "wire.md has a Regime stamp section with its placeholder replaced")

HDR_STAMP = [(0, 2), (2, 2), (4, 2), (6, 1), (7, 1), (8, 8), (16, 16), (32, 8), (40, 8),
             (48, 4), (52, 1), (53, 1), (54, 2), (56, 2)]
SEC_T = None
for _t in parse_tables(STAMP):
    if any(len(r) > 3 and r[3].startswith("Section length") for r in _t[2]):
        SEC_T = _t
        break
_sec_rows = SEC_T[2] if SEC_T else []
_sig_ok = len(_sec_rows) >= 15 and all(
    re.fullmatch(r"\d+", r[0].strip()) and int(r[0]) == off and (width_of(r[1])[0] == bits)
    for r, (off, bits) in zip(_sec_rows[:14], [(o, b * 8) for o, b in HDR_STAMP]))
check("DoD", _sig_ok, "the merged stamp section header rows (bytes 0 to 57) are unchanged")
_secmax = [r for r in _sec_rows if len(r) > 3 and r[3].startswith("Section count")]
SECTION_MAX = int(re.search(r"1 to (\d+)", _secmax[0][4]).group(1)) if _secmax and re.search(r"1 to (\d+)", _secmax[0][4]) else None
check("AC1", SECTION_MAX == 64, f"the merged stamp allows 1 to 64 sections per stamp (read {SECTION_MAX})")

REJ_FIXED = [("TooShort", 0), ("BadMagic", 1), ("VersionMismatch", 2), ("ForeignDomain", 3),
             ("UnconfiguredDomain", 4), ("UnknownStream", 5), ("UnadmittedSource", 6), ("BadLength", 7),
             ("BadChecksum", 8), ("BadPacketIndex", 9), ("WrongEpoch", 10), ("OversizeControl", 11),
             ("ReservedNonZero", 12)]
REJ_LAYERS = {}
REJ_VALUE = {}
_rej_span = span_of(r"^rejection\s+reasons$", 2)
for _t in parse_tables("\n".join(body(_rej_span))):
    for _r in _t[2]:
        if len(_r) >= 4 and re.fullmatch(r"\d+", _r[0].strip()) and re.fullmatch(r"[A-Z][A-Za-z0-9]+", _r[1].strip()):
            REJ_VALUE[_r[1].strip()] = int(_r[0])
            REJ_LAYERS[_r[1].strip()] = _r[3]
check("DoD", [(n, v) for n, v in REJ_VALUE.items()] and sorted(REJ_VALUE.items(), key=lambda x: x[1]) == REJ_FIXED,
      "the merged Rejection reasons enumeration is unchanged (13 members, names and values)")
CONTROL_MEMBERS = {n for n, layers in REJ_LAYERS.items() if "control" in layers}

_reserved = None
for _t in parse_tables(WIRE):
    if _t[1] and _t[1][0].lower().startswith("field") and len(_t[1]) > 1 and "location" in _t[1][1].lower():
        _reserved = _t
RES_ROWS = _reserved[2] if _reserved else []
check("DoD", any("section type codes" in r[0].lower() for r in RES_ROWS),
      "the merged Reserved fields table still records the section type codes row")

# The regime stamp reserved rows of the merged Reserved fields table (4 at the merged text)
MERGED_RES_STAMP_ROWS = 4
check("DoD", len([r for r in RES_ROWS if len(r) > 1 and "regime stamp" in r[1].lower()]) >= MERGED_RES_STAMP_ROWS,
      "the merged Reserved fields table keeps its four regime stamp rows (the new types add rows, none is removed)")

# Code block of this ticket (the merged Section type codes table gives it to 004-28)
CODE_LO2 = CODE_HI2 = None
for _t in parse_tables(WIRE):
    if _t[1] and _t[1][0].lower().startswith("codes"):
        for r in _t[2]:
            if len(r) >= 3 and "004-28" in r[2]:
                _c = re.search(r"(\d+)\s+to\s+(\d+)", r[0])
                if _c:
                    CODE_LO2, CODE_HI2 = int(_c.group(1)), int(_c.group(2))
check("AC1", CODE_LO2 is not None,
      f"the merged section type codes give 004-28 a block (read {CODE_LO2} to {CODE_HI2})")

STMT = re.compile(r"(?:⌈|ceil\s*\()\s*(\d+)\s*/\s*(\d+)\s*(?:⌉|\))\s*=\s*(\d+)", re.I)
SIZE_MAX_RE = re.compile(r"maximum[^.;]{0,120}?(\d+)\s*bytes", re.I)
TYPE_CODE_RE = re.compile(r"type code\D{0,20}?(\d+)|\bcode (\d+)\b|section type\D{0,20}?(\d+)\b", re.I)

def lo_of(f):
    m = re.search(r"bits?\s*\[(\d+):(\d+)\]", f["meaning"])
    return min(int(m.group(1)), int(m.group(2))) if m else None


def build_tree(ft):
    """Each list row consumes the next field table in document order as its element (depth-first)."""
    ptr = [1]
    used = {0}

    def node(i):
        rows = []
        for f in ft[i]:
            child = None
            if f["maxc"] and ptr[0] < len(ft):
                c = ptr[0]
                ptr[0] += 1
                used.add(c)
                child = node(c)
            rows.append((f, child))
        return {"rows": rows}

    root = node(0)
    return root, [k for k in range(len(ft)) if k not in used]


def tile_node(node, start_bits, title, label):
    """Rows tile with no gap or overlap; returns the end position in bits."""
    pos = start_bits
    for f, child in node["rows"]:
        lo = lo_of(f)
        if lo is not None:
            s = f["off"] * 8 + lo
        elif f["bits"] % 8 != 0:
            s = pos
        else:
            s = f["off"] * 8
        if s != pos:
            fail(label, f"the {title} table does not tile: a row at bit {s} follows bit {pos}")
            pos = s
        f["_start"] = s
        pos = s + f["bits"] * (f["maxc"] or 1)
        if child is not None:
            tile_node(child, 0, title + " (entry)", label)
    return pos


def size_node(node, title):
    total = 0
    for f, child in node["rows"]:
        if f["maxc"]:
            if child is not None:
                e = size_node(child, title)
                child["size"] = e
                if e != f["bits"]:
                    fail("AC1", f"the {title} entry table sums to {e} bits, but its list row declares {f['bits']} bits per entry")
            total += f["bits"] * f["maxc"]
        else:
            total += f["bits"]
    node["size"] = total
    return total


def entry_nodes(node, out=None):
    out = [] if out is None else out
    for f, child in node["rows"]:
        if child is not None:
            out.append(child)
            entry_nodes(child, out)
    return out


def exact_product_results(text):
    """Results c of every chain product 'a × b × ... = c' in text whose factors multiply exactly to c (chain parser)."""
    out = set()
    for expr, c in re.findall(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)", text):
        prod = 1
        for fac in re.split(r"\s*(?:×|\bx\b|\*)\s*", expr.strip()):
            prod *= int(fac)
        if prod == int(c):
            out.add(int(c))
    return out


def has_num2(text, n):
    """A number as a whole token: 17 does not match C-17, and -324 matches only as a negative."""
    s = str(n)
    if n < 0:
        return re.search(r"(?<![\w])" + re.escape(s) + r"(?!\d)", text) is not None
    return re.search(r"(?<![\w\-.])" + s + r"(?!\d|\.\d)", text) is not None


def sentences(text):
    pieces = [s for s in re.split(r"(?<=[.;])\s+|\n\s*\n|\n(?=[|*#-])", text) if s.strip()]
    out = []
    for s in pieces:
        if out and re.match(r"\s*[\[`]", s):
            out[-1] = out[-1] + " " + s
        else:
            out.append(s)
    return out


def analyse(title, text, label):
    """Field tables of one subsection: the section maximum, the list rows and their entry tables."""
    out = {"sec_max": None, "Nt": None, "root": None, "entries": [], "body_rel": None,
           "fl_main": [], "stmts": [], "fl_all": [], "code": None, "ok": False}
    ft = []
    for tb in parse_tables(text):
        if is_field_table(tb[1]):
            fl = fields_of(tb, label, title)
            ft.append(fl)
            out["fl_all"] += fl
    if not ft or not ft[0]:
        fail(label, f"the {title} has no field table of Offset, Width, Type, Meaning, Valid range")
        return out
    root, unused = build_tree(ft)
    if unused:
        fail(label, f"the {title} has {len(unused)} field table(s) that no list row names as its entry")
    out["root"] = root
    out["fl_main"] = [f for f, _ in root["rows"]]
    if not out["fl_main"]:
        return out
    off0 = out["fl_main"][0]["off"]
    out["body_rel"] = off0 != 58
    if off0 not in (0, 58):
        fail(label, f"the {title} table starts at offset {off0}: a body starts at 0 and a section at 58")
    tile_node(root, off0 * 8 + (lo_of(out["fl_main"][0]) or 0), title, label)
    out["entries"] = entry_nodes(root)
    rbits = size_node(root, title)
    out["sec_max"] = 58 + cdiv(rbits, 8)
    out["stmts"] = [(int(a), int(b), int(c)) for a, b, c in STMT.findall(text)]
    Nt = 1
    for f in [f for f, _ in root["rows"] if f["maxc"]]:
        cand = [s for s in out["stmts"] if s[1] == f["maxc"]]
        if cand:
            Nt = max(Nt, cdiv(cand[0][0], f["maxc"]))
    out["Nt"] = Nt
    _code = TYPE_CODE_RE.search(text)
    out["code"] = int([g for g in _code.groups() if g][0]) if _code else None
    out["ok"] = True
    return out


# The seven section types of 004-25 (merged): the six other types need the sections that the budget counts
MERGED_SPECS = [("multi", r"multi[- ]?packet"), ("health", r"health|per[- ]connection"),
                ("calibration", r"calibration"), ("commissioning", r"commissioning"),
                ("injection", r"injection"), ("capability", r"capabilit"), ("timestamp", r"timestamp|time stamp")]
SUBS = subsections(STAMP_LINES)
MERGED = {}
for key, pat in MERGED_SPECS:
    hits = find_sub(SUBS, pat)
    if len(hits) == 1:
        MERGED[key] = analyse(hits[0][0], hits[0][1], "AC1")
check("DoD", len(MERGED) == 7, f"the seven merged section types of 004-25 are found once each (found {len(MERGED)})")
MERGED_CODES = {k: v["code"] for k, v in MERGED.items() if v.get("code") is not None}
S_OTHER = sum(v["Nt"] for k, v in MERGED.items() if k != "health" and v.get("Nt"))
check("AC1", S_OTHER == 7, f"the six other merged section types need 7 sections at their maxima (computed {S_OTHER})")
HEALTH = MERGED.get("health")
F_H = [f["maxc"] for f, _ in HEALTH["root"]["rows"] if f["maxc"]][0] if HEALTH and HEALTH.get("root") else None
check("DoD", HEALTH is not None and HEALTH["entries"] and cdiv(HEALTH["entries"][0]["size"], 8) == 65 and F_H == 18,
      f"the merged health entry is 65 bytes and 18 entries fit a section (read {F_H})")

# ---------------------------------------------------------------------------
# The two section types of this ticket (Work 1 and Work 2)
# ---------------------------------------------------------------------------
NEW = {}
for key, pat, name in [("dscp", r"dscp", "observed DSCP of peers"), ("alignment", r"alignment", "alignment group")]:
    hits = find_sub(SUBS, pat)
    check("AC1", len(hits) == 1, f"exactly one subsection of the regime stamp section names the {name} section type (found {len(hits)})")
    if len(hits) != 1:
        continue
    a = analyse(hits[0][0], hits[0][1], "AC1")
    a["title"], a["text"] = hits[0][0], hits[0][1]
    NEW[key] = a

for key, a in NEW.items():
    text = a["text"]
    if not a["ok"]:
        fail("AC1", f"the {key} section type has no usable field table")
        continue
    check("AC1", a["sec_max"] <= CTRL_BOUND,
          f"the {key} section at its maxima is {a['sec_max']} bytes, no more than the control bound (C-17) of {CTRL_BOUND} bytes"
          + ("" if a["sec_max"] <= CTRL_BOUND else f": over the control bound by {a['sec_max'] - CTRL_BOUND}"))
    stated = [int(x) for x in SIZE_MAX_RE.findall(text)]
    check("AC1", a["sec_max"] in stated,
          f"a sentence of the {key} subsection states its maximum section size as {a['sec_max']} bytes (the table's sum)"
          + ("" if a["sec_max"] in stated else f"; stated {stated}"))
    code = a["code"]
    check("AC1", code is not None and CODE_LO2 is not None and CODE_LO2 <= code <= CODE_HI2 and code not in MERGED_CODES.values(),
          f"the {key} section type has a type code inside the block the merged table gives 004-28 ({CODE_LO2} to {CODE_HI2}) and not a code of 004-25 (found {code})")
    for f in [f for f, _ in a["root"]["rows"] if f["maxc"]]:
        cand = [s for s in a["stmts"] if s[1] == f["maxc"]]
        if not cand:
            fail("AC1", f"the {key} list row (at most {f['maxc']} entries per section) has no statement of the form ⌈M / {f['maxc']}⌉ = N")
            continue
        M, F, N = cand[0]
        check("AC1", N == cdiv(M, F), f"the {key} section count is ⌈{M} / {F}⌉ = {cdiv(M, F)}, the text states {N}")
    bad = []
    for expr, c in re.findall(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)", text):
        prod = 1
        for fac in re.split(r"\s*(?:×|\bx\b|\*)\s*", expr.strip()):
            prod *= int(fac)
        if prod != int(c):
            bad.append(f"{expr.strip()} = {c}")
    for expr, c in re.findall(r"([\d\s×*+-]*\d[\d\s×*+-]*)=\s*(\d+)", text):
        if re.search(r"\d\s*(\+|-)\s*\d", expr):
            try:
                val = eval(re.sub(r"\s", "", expr).replace("×", "*"))
            except Exception:
                bad.append(f"{expr.strip()} = {c}")
                continue
            if val != int(c):
                bad.append(f"{expr.strip()} = {c}")
    check("AC1", not bad, f"every product and sum stated in the {key} subsection is exact" + (f": {bad[:4]}" if bad else ""))

S_NEW = sum(a["Nt"] for a in NEW.values() if a.get("ok") and a.get("Nt"))
S_TOTAL = S_OTHER + S_NEW
THRESH = ((SECTION_MAX or 64) - S_OTHER - S_NEW) * (F_H or 0)
PARAS = [x for k, x in prose_blocks(STAMP) if k == "p"]

# AC 1, the budget: the sections fit with their arithmetic stated, or the overflow is reported completely
RULE7 = "a stamp that needs more cannot be sent"
if len(NEW) != 2:
    check("AC1", False, f"the budget cannot be computed: both new section types must exist (found {sorted(NEW)})")
elif S_TOTAL <= (SECTION_MAX or 64):
    fit = [x for x in PARAS if has_num2(x, S_TOTAL) and has_num2(x, THRESH) and re.search(r"\bfit", x, re.I) and has_evidence(x)]
    check("AC1", bool(fit),
          f"the budget fits with its arithmetic stated: the six other types and the two new types need {S_TOTAL} sections of 64, "
          f"and the threshold (64 − {S_OTHER} − {S_NEW}) × {F_H} = {THRESH} entries is stated in one tagged paragraph")
else:
    def score(p):
        return sum([has_num2(p, S_TOTAL), bool(re.search(r"exceed|falsif", p, re.I)), "design owner" in p.lower(),
                    RULE7 in p.lower() and "cannot be sent" in p.lower(), bool(re.search(r"no stamp", p, re.I)),
                    has_num2(p, THRESH) and bool(re.search(r"threshold|fit", p, re.I)), has_evidence(p),
                    bool(re.search(r"\bcap", p, re.I)), "open item" in p.lower()])
    cands = [x for x in PARAS if "open item" in x.lower() or has_num2(x, S_TOTAL)]
    best = max(cands, key=score) if cands else ""
    check("AC1", has_num2(best, S_TOTAL) and "open item" in best.lower(),
          f"the overflow report names an Open item and states the total {S_TOTAL} sections against the bound {SECTION_MAX}")
    check("AC1", bool(re.search(r"exceed|falsif", best, re.I)) and "design owner" in best.lower(),
          "the overflow report says the bound is exceeded or falsified, and that it goes to the design owner")
    check("AC1", RULE7 in best.lower() and "cannot be sent" in best.lower(),
          "the overflow report cites merged sender rule 7 (a stamp that needs more cannot be sent)")
    check("AC1", bool(re.search(r"no stamp", best, re.I)),
          "the overflow report says that above the bound a module sends no stamp at all")
    check("AC1", has_num2(best, THRESH) and bool(re.search(r"threshold|fit", best, re.I)),
          f"the overflow report gives the threshold in entries, (64 − {S_OTHER} − {S_NEW}) × {F_H} = {THRESH}")
    check("AC1", bool(re.search(r"\bcap", best, re.I)), "the overflow report names the connection cap that the design owner sets")
    check("DoD", has_evidence(best), "the overflow report paragraph carries an evidence tag")
    # C2 (review): the closed-form cap condition states the width it assumes, the limit for its own entry size and for an
    # alternative entry size as exact arithmetic, per-input entries (Open item 17), and the section types not yet laid out.
    CAP_PARAS = [x for x in PARAS if "18⌉" in x and re.search(r"\bcap\b", x)]
    union = "\n".join(CAP_PARAS)
    check("AC1", bool(CAP_PARAS), "the cap condition is stated in closed form over ⌈ ⌉ of 18 member entries per section")
    check("AC1", any(re.search(r"assum", x, re.I) and re.search(r"bit|bytes", x, re.I) and has_evidence(x)
                     for x in sentences(union)),
          "the cap condition states, in a tagged sentence, the figure width it assumes (review C2 (a))")
    check("AC1", bool(re.search(r"per[- ]input", union, re.I)) and "Open item 17" in union,
          "the cap condition assumes per-input alignment entries and names Open item 17 (review C2 (b))")
    for tok, what in [(r"path[ -]MTU", "the path MTU"), (r"013-08", "013-08"), (r"006-19", "006-19")]:
        check("AC1", bool(re.search(tok, union, re.I)),
              f"the cap condition names {what}, a section type not yet laid out that lowers the bound (review C2 (c))")
    if NEW.get("alignment", {}).get("ok") and NEW.get("dscp", {}).get("ok"):
        own_F = [f["maxc"] for f, _ in NEW["alignment"]["root"]["rows"] if f["maxc"]][0]
        own_size = NEW["alignment"]["entries"][0]["size"] // 8
        budget = (SECTION_MAX or 64) - S_OTHER - NEW["dscp"]["Nt"]

        def limit(Fa):
            e = 0
            while cdiv(e + 1, F_H) + cdiv(e + 1, Fa) <= budget:
                e += 1
            return e

        L_own = limit(own_F)
        alt = [(int(x), int(y)) for x, y in re.findall(r"⌊\s*1193\s*/\s*(\d+)\s*⌋\s*=\s*(\d+)", union)
               if int(x) != own_size and int(y) != own_F]
        prods = exact_product_results(union)
        check("AC1", L_own in prods,
              f"the cap condition gives the limit for its own entry size as exact arithmetic k × n = {L_own} (review C2 (a))")
        alt_L = limit(alt[0][1]) if alt else None
        check("AC1", alt_L is not None and alt_L in prods,
              f"the cap condition gives the limit for one alternative entry size as exact arithmetic k × n = "
              f"{alt_L if alt_L is not None else 'unstated'}, with ⌊1 193 / X⌋ = Y stating the alternative (review C2 (a))")

# ---------------------------------------------------------------------------
# AC 2, Work 1: the observed DSCP section
# ---------------------------------------------------------------------------
DSCP_WIDE_FLAG = [False]
if not ("dscp" in NEW and NEW["dscp"]["ok"]):
    check("AC2", False, "the observed DSCP section is missing: the peer id, the hello and sync DSCP, the not-observed encoding "
          "and the dropped-request counter cannot be checked")
else:
    d = NEW["dscp"]
    ent = [f for n in d["entries"] for f, _ in n["rows"]]
    rows_like = lambda pat: [f for f in ent if re.search(pat, f["meaning"], re.I)]
    ids = rows_like(r"module id")
    check("AC2", any(f["bits"] == 64 for f in ids), "the observed DSCP entry names its peer by a 64-bit module id (Work 1, C-11)")
    hello = rows_like(r"hello")
    check("AC2", bool(hello) and any(re.search(r"\b63\b", f["valid"]) for f in hello),
          "the observed DSCP entry carries the DSCP observed on the peer's hellos, valid 0 to 63 (C-152, every module)")
    sync = rows_like(r"sync")
    check("AC2", any(re.search(r"holder", f["meaning"], re.I) for f in sync),
          "the observed DSCP entry carries the DSCP observed on sync requests, holder only (C-152)")
    dropped = rows_like(r"drop")
    check("AC2", any(re.search(rf"\b{ACCEPT_HI}\b|C-204", f["meaning"] + " " + f["valid"]) for f in dropped),
          f"the dropped-request counter names the accepted rate of {ACCEPT_HI} per second (C-204, C-112)")
    check("AC2", any(f["bits"] for f in dropped), "the dropped-request counter has a width (C-204)")
    check("AC2", any(re.search(r"request|per second", f["meaning"], re.I) for f in dropped),
          "the dropped-request counter has a unit: requests (C-204)")
    check("AC2", any(len(set(x.lower()[:4] for x in re.findall(r"wrap|saturat", f["meaning"] + " " + f["valid"], re.I))) == 1
                     for f in dropped),
          "the dropped-request counter states exactly one wrap or saturation rule (Work 1)")
    # Work 1 asks for "a distinct 'not observed' encoding" and fixes no shape. Any stated encoding passes if it is told
    # from an observed DSCP 0 to 63: (a) a sentinel code whose not-observed clause holds no value in 0 to 63,
    # (b) a one-bit flag, (c) a C-129 state field whose four states are the only states, with the DSCP field
    # counted only when the state says present (zero otherwise).
    notobs = [f for f in ent if re.search(r"not observed", f["meaning"] + " " + f["valid"], re.I)]

    def not_observed_codes(valid):
        out = []
        for cl in re.split(r";", re.sub(r"C-\d+", "", valid)):
            if re.search(r"not observed", cl, re.I):
                out += [int(x) for x in re.findall(r"\b\d+\b", cl)]
        return out

    sentinel = [f for f in notobs if not_observed_codes(f["valid"]) and all(x > 63 for x in not_observed_codes(f["valid"]))]
    collide = [f for f in notobs if any(x <= 63 for x in not_observed_codes(f["valid"]))]
    flag = [f for f in ent if re.search(r"observed", f["meaning"], re.I) and f["bits"] == 1
            and re.search(r"\b0 or 1\b|\b0 to 1\b|\b0/1\b", f["valid"])]
    state = [f for f in ent if re.search(r"C-129", f["meaning"]) and re.search(r"\bstate", f["meaning"], re.I)]
    for f in state:
        counts = [int(b) - int(a) + 1 for a, b in re.findall(r"(\d+) to (\d+)", f["valid"]) if int(a) >= 1]
        check("AC2", counts == [4], f"the C-129 state field enumerates four states (C-129), found {counts}")
    state_dscp = [f for f in ent if re.search(r"DSCP", f["meaning"]) and re.search(r"zero|present", f["meaning"] + " " + f["valid"], re.I)]
    state_ok = bool(state) and bool(state_dscp)
    check("AC2", not collide and (bool(sentinel) or bool(flag) or state_ok),
          "the not-observed encoding is stated and told from an observed DSCP 0 to 63 (a sentinel, a one-bit flag, or a C-129 state field)")
    check("AC2", any(re.search(r"DSCP", f["meaning"]) and re.search(r"0 to 63", f["valid"]) for f in ent),
          "the DSCP proper is 0 to 63, six bits (C-13), in the observed DSCP entry")
    DSCP_WIDE_FLAG[0] = any(re.search(r"DSCP", f["meaning"]) and f["bits"] >= 7 for f in ent)
    check("AC2", bool(re.search(r"order", d["text"], re.I)), "the entry order of the observed DSCP section is stated")
    check("AC2", bool(re.search(r"C-34|C-114|C-217", d["text"])), "the observed DSCP entry count derives from the constants (C-34, C-114)")
    dlist = [f for f, _ in d["root"]["rows"] if f["maxc"]]
    if dlist:
        cand = [s for s in d["stmts"] if s[1] == dlist[0]["maxc"]]
        if cand:
            check("AC2", cand[0][0] == MAX_PEERS,
                  f"the observed DSCP section holds at most {MAX_PEERS} entries, one per peer (C-34, C-114): the text states {cand[0][0]}")
    check("DoD", "sender rule 3" in d["text"],
          "hand-off 004-28: the dropped-request counter falls under sender rule 3 (any byte change is a new version)")

# ---------------------------------------------------------------------------
# AC 3, Work 2: the alignment group
# ---------------------------------------------------------------------------
if not ("alignment" in NEW and NEW["alignment"]["ok"] and HEALTH and HEALTH.get("entries")):
    check("AC3", False, "the alignment group section is missing: the input identifier, the four lateness figures, the since-live "
          "maximum and the reclaimable latency cannot be checked")
else:
    g = NEW["alignment"]
    ent = [f for n in g["entries"] for f, _ in n["rows"]]
    H = [f for n in HEALTH["entries"] for f, _ in n["rows"]]
    for pat, label in [(r"module id", "peer module id"), (r"stream number", "stream number"),
                       (r"request id", "admission request id"), (r"\brole\b", "role")]:
        hr = [f for f in H if re.search(pat, f["meaning"], re.I)]
        gr = [f for f in ent if re.search(pat, f["meaning"], re.I)]
        check("AC3", bool(hr) and bool(gr) and gr[0]["bits"] == hr[0]["bits"],
              f"the alignment input identifier carries the health key's {label} at the health width (004-25; hand-off 004-28)")
    UNIT = re.compile(r"nanosecond|microsecond|millisecond|\bns\b|µs|\bus\b|\bms\b|slice|sample|packet|second", re.I)

    def lat_cat(m):
        if re.search(r"99\.9th", m):
            return "99.9th"
        if re.search(r"\b99th\b", m):
            return "99th"
        if re.search(r"\b50th\b", m):
            return "50th"
        if re.search(r"maximum", m, re.I):
            return "maximum"
        return "other"

    lat = [f for f in ent if not re.search(r"since|reclaimable", f["meaning"], re.I)
           and re.search(r"50th|99th|99\.9th|maximum|percentile|lateness", f["meaning"], re.I)]
    cats = sorted(lat_cat(f["meaning"]) for f in lat)
    check("AC3", cats == ["50th", "99.9th", "99th", "maximum"],
          f"the alignment group carries exactly the four lateness figures of C-160: 50th, 99th, 99.9th and the maximum (found {cats})")
    check("AC3", bool(lat) and all(f["bits"] and UNIT.search(f["meaning"]) for f in lat),
          "each lateness figure has a width and a unit (Work 2)")
    check("AC3", bool(re.search(rf"rolling\s+{WINDOW_S}\s*s\b", g["text"])),
          f"the alignment group states its window as a rolling {WINDOW_S} s window in wire.md (C-182)")
    since = [f for f in ent if re.search(r"since", f["meaning"], re.I)]
    check("AC3", len(since) == 1 and bool(since[0]["bits"]) and bool(UNIT.search(since[0]["meaning"])),
          "the maximum since the connection went live is one figure with a width and a unit (C-182)")
    recl = [f for f in ent if re.search(r"reclaimable", f["meaning"], re.I)]
    check("AC3", len(recl) == 1 and bool(recl[0]["bits"]) and bool(UNIT.search(recl[0]["meaning"])),
          "the reclaimable latency is one figure with a width and a unit (C-157)")
    check("AC3", len(recl) == 1 and bool(re.search(r"\bA\b|audition", recl[0]["valid"] + " " + recl[0]["meaning"])),
          "the reclaimable latency's valid range is bounded by A (C-53, C-96)")
    glist = [f for f, _ in g["root"]["rows"] if f["maxc"]]
    if glist:
        cand = [s for s in g["stmts"] if s[1] == glist[0]["maxc"]]
        if cand:
            sent = [s for s in sentences(g["text"]) if STMT.search(s)]
            check("AC3", any(re.search(r"C-\d+", s) for s in sent),
                  "the maximum alignment entry count is stated with the constant it derives from (C-nn in the same sentence)")
    check("AC3", bool(re.search(r"split|several sections|each section", g["text"], re.I)),
          "the split of the alignment type over several sections is stated (Work 2)")
    check("AC3", bool(re.search(r"order", g["text"], re.I)), "the entry order of the alignment group is stated")

# ---------------------------------------------------------------------------
# AC 4, Work 3: invalid forms with outcomes, in the two new subsections
# ---------------------------------------------------------------------------
ROWS = []
for key in NEW:
    for tb in parse_tables(NEW[key]["text"]):
        if tb[1] and tb[1][0].lower().startswith("invalid form") and len(tb[1]) > 1 and "required outcome" in tb[1][1].lower():
            ROWS += [(key, r) for r in tb[2] if len(r) >= 2]
check("AC4", len(ROWS) >= 5, f"at least 5 invalid forms with outcomes in the new subsections (found {len(ROWS)})")
NONREJ_RE = re.compile(r"not a rejection|never a rejection|\bignored\b|\bvalid\b|no reason", re.I)
NEUTRAL_RE = re.compile(r"repeat|unknown (section )?(type|code)|older|skipped", re.I)
for key, r in ROWS:
    form, outcome = r[0], r[1]
    names = [t for t in re.findall(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b", outcome) if t in REJ_VALUE]
    bad = [n for n in names if n not in CONTROL_MEMBERS]
    if bad:
        fail("AC4", f"an invalid form of the {key} type names a reason outside the control layer {bad}: {form[:50]}")
        continue
    if not names and not (NONREJ_RE.search(outcome) and NEUTRAL_RE.search(form)):
        fail("AC4", f"a malformed form of the {key} type names no Reject reason (only repeats, unknown codes and older versions may be silent): {form[:60]}")
        continue
    fl_ = form.lower()
    if re.search(r"reserved|non-?zero", fl_) and "ReservedNonZero" not in names:
        fail("AC4", f"a reserved-bit form must name ReservedNonZero: {form[:60]}")
    if re.search(r"length", fl_) and "BadLength" not in names:
        fail("AC4", f"a length form must name BadLength: {form[:60]}")
    if re.search(r"count", fl_) and re.search(r"above|exceed|more than|maximum", fl_) and "BadLength" not in names:
        fail("AC4", f"a count-above-maximum form must name BadLength (the merged BadLength row): {form[:60]}")
for cat, pat in [("a DSCP value above 63", r"DSCP"), ("a count above its maximum", r"count|above|exceed|more than|maximum"),
                 ("a length that disagrees", r"length"), ("percentiles out of non-decreasing order", r"non-decreasing|percentile|50th|99th"),
                 ("reserved bits non-zero", r"reserved|non-?zero")]:
    if cat == "a DSCP value above 63" and not DSCP_WIDE_FLAG[0]:
        continue  # a six-bit DSCP field holds no value above 63, so no such form is needed
    check("AC4", any(re.search(pat, r[0], re.I) for _, r in ROWS), f"an invalid form covers: {cat} (Work 3)")

# C1 (review): the entry is listed from the moment its connection first goes live (coder text, Alignment group section),
# and 009-13 says "The maximum since live starts when `finish_aligning` switches to `Live`". So an entry can hold a
# since-live maximum with no sample yet: its no-sample value must be stated and must not be rejected or excluded.
if "alignment" in NEW and NEW["alignment"]["ok"]:
    ga = NEW["alignment"]
    gent = [f for n in ga["entries"] for f, _ in n["rows"]]
    nos = [f for f in gent if re.search(r"since", f["meaning"], re.I)
           and re.search(r"no (live |lateness )?sample|first live sample", f["meaning"] + " " + f["valid"], re.I)]
    check("AC3", bool(nos), "the maximum since live states the value it carries when it has no sample (C-182; 009-13: it starts when finish_aligning switches to Live)")
    vals = set()
    for f in nos:
        for cl in re.split(r";", f["valid"]):
            if re.search(r"sample", cl, re.I):
                vals |= set(re.findall(r"-2\^63|-?\d+", cl))
    excluded = any(re.search(r"other than\s*" + re.escape(v) + r"(?![\w^])", f["valid"]) for f in nos for v in vals)
    listed_first = bool(re.search(r"first live sample", ga["text"], re.I))
    check("AC3", bool(nos) and ((bool(vals) and not excluded) or listed_first),
          "the no-sample value of the maximum since live is admitted by its own valid range, or entries are listed only from the first live sample (C1)")
    pct = [x for x in sentences(ga["text"]) if re.search(r"non-decreasing", x, re.I)]
    check("AC3", any(re.search(r"exempt|holds? for|no (live |lateness )?sample|no-sample|first live sample|-2\^63", x, re.I) for x in pct),
          "the percentile order rule is stated to hold or be exempt for the no-sample encoding (C1)")
    for key, r in ROWS:
        if key != "alignment" or not re.search(r"since", r[0], re.I):
            continue
        for v in vals:
            if re.search(r"(?<![\w^-])" + re.escape(v) + r"(?![\w^])", r[0]):
                fail("AC4", f"an invalid form rejects the since-live maximum that holds its no-sample value {v}: {r[0][:70]}")

# ---------------------------------------------------------------------------
# DoD: evidence tags, open items, reserved fields
# ---------------------------------------------------------------------------
for key in NEW:
    text = NEW[key]["text"]
    check("DoD", has_evidence(text), f"the {key} subsection carries an evidence tag")
    untagged = [x[:50] for k, x in prose_blocks(text) if k == "p"
                and len(re.sub(r"[`*_|>#-]", " ", x).split()) >= 5 and not has_evidence(x)]
    check("DoD", not untagged, f"every prose paragraph of 5 or more words in the {key} subsection carries an evidence tag"
          + (f"; untagged: {untagged[:2]}" if untagged else ""))
share = [x for x in sentences(STAMP) if re.search(r"share", x) and re.search(r"address", x) and re.search(r"host", x)]
check("DoD", all(re.search(r"Admission|Receiver report", x) for x in share),
      "review C4: the sentence on two modules on one host sharing an address cites the merged wire text (Admission or Receiver report), not 01-architecture § 2.6 alone")
_open = [x for k, x in prose_blocks(STAMP) if re.search(r"open item", x, re.I)]
check("DoD", all(re.search(r"owner", x, re.I) and re.search(r"falsifier", x, re.I) for x in _open),
      f"every Open item paragraph of the regime stamp section names an owner and a falsifier ({len(_open)} found)")
_need_res = any(re.search(r"\breserved\b|\bunused\b|accept[- ]any", f["meaning"] + " " + f["valid"], re.I)
                for a in NEW.values() for f in a.get("fl_all", []))
_res_stamp = [r for r in RES_ROWS if len(r) > 1 and "regime stamp" in r[1].lower()]
N_RES_NEW = sum(1 for a in NEW.values() for f in a.get("fl_all", []) if re.search(r"\breserved\b", f["meaning"], re.I))
check("DoD", len(_res_stamp) >= MERGED_RES_STAMP_ROWS + N_RES_NEW,
      f"every reserved field of the new types ({N_RES_NEW} found) is recorded in the merged Reserved fields table, one row each "
      f"(the table has {len(_res_stamp)} regime stamp rows, the merged four included)")

# ---------------------------------------------------------------------------
# AC 5, Work 4: wire.toml declares the two section types and their entry tables, row for row
# ---------------------------------------------------------------------------
try:
    TOML = parse_toml(TOML_TEXT)
    TOML_OK = True
except Exception as error:
    fail("AC5", f"wire.toml does not parse: {error}")
    TOML = {}
    TOML_OK = False
msgs = TOML.get("message", []) if TOML_OK else []
msgs = msgs if isinstance(msgs, list) else []
for key, pat in [("dscp", r"dscp"), ("alignment", r"alignment")]:
    cand = [m for m in msgs if re.search(pat, str(m.get("name", "")), re.I) and m.get("section_of") == "regime stamp"]
    check("AC5", len(cand) == 1, f"wire.toml declares exactly one regime stamp message for the {key} section type (found {len(cand)})")
    if len(cand) != 1 or key not in NEW or not NEW[key]["ok"]:
        continue
    a = NEW[key]
    m = cand[0]
    check("AC5", m.get("layer") == "control" and int(m.get("dscp", -1)) == DSCP_CONTROL and int(m.get("max_size", -1)) == a["sec_max"],
          f"the {key} message of wire.toml is control, DSCP {DSCP_CONTROL}, and its max_size is the table's {a['sec_max']} bytes")
    reasons = set(m.get("rejection_reasons", []))
    check("AC5", bool(reasons) and reasons <= CONTROL_MEMBERS, f"the {key} message names only control reasons (found {sorted(reasons)})")
    tf = [f for f in m.get("field", []) if int(f.get("offset", 0)) >= 464]
    md = [f for f, _ in a["root"]["rows"]]
    agree = len(tf) == len(md)
    for f, g_ in zip(tf, md):
        if int(f["offset"]) != g_["_start"] + (464 if a["body_rel"] else 0) or int(f["width"]) != g_["bits"]:
            agree = False
        if bool(g_["maxc"]) != ("count" in f) or (g_["maxc"] and int(f.get("max_count", -1)) != g_["maxc"]):
            agree = False
    check("AC5", agree, f"the {key} fields of wire.toml agree with its table row for row (offset, width, count)")
    ent_msgs = [x for x in msgs if re.search(pat, str(x.get("name", "")), re.I)
                and "entry" in str(x.get("name", "")).lower() and "section_of" not in x]
    check("AC5", len(ent_msgs) == len(a["entries"]),
          f"wire.toml declares one entry message per entry table of the {key} section (tables {len(a['entries'])}, messages {len(ent_msgs)})")
    for n, em in zip(a["entries"], ent_msgs):
        efs = em.get("field", [])
        ok = len(efs) == len(n["rows"]) and int(em.get("max_size", -1)) == cdiv(n["size"], 8)
        for (f, child), g_ in zip(n["rows"], efs):
            if int(g_.get("offset", -1)) != f["_start"] or int(g_.get("width", -1)) != f["bits"]:
                ok = False
            if bool(f["maxc"]) != ("count" in g_) or (f["maxc"] and int(g_.get("max_count", -1)) != f["maxc"]):
                ok = False
        ends = [int(g_["offset"]) + int(g_["width"]) * (int(g_.get("max_count", 1)) if "count" in g_ else 1) for g_ in efs]
        if ends and ends[-1] != n["size"]:
            ok = False
        check("AC5", ok, f"the entry table of {em.get('name')} agrees with wire.toml row for row and tiles to its size ({n['size']} bits)")

# AC 5: the 004-02 checker and the wire_tables tests, at the repository level only
MIN_WIRE_TABLES = 12  # the count at the merged text: 12 passed, 0 failed
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"], capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        check("AC5", proc.returncode == 0 and "checks clean" in out,
              "cargo run --bin wire-check reports checks clean"
              + ("" if proc.returncode == 0 and "checks clean" in out else ": " + " | ".join(out.strip().splitlines()[-4:])))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC5", f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"], capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result[-1]) if result else None
        check("AC5", proc.returncode == 0 and bool(mm) and int(mm.group(2)) == 0 and int(mm.group(1)) >= MIN_WIRE_TABLES,
              f"cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least {MIN_WIRE_TABLES} passed, the count at the merged text"
              + (f": {result[-1]}" if result else f": no result line ({out.strip()[-200:]})"))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC5", f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC5: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

fails = [ac for ac, ok in RESULTS if not ok]
print(f"{len(RESULTS) - len(fails)} passed, {len(fails)} failed")
sys.exit(1 if fails else 0)

PY
pyrc=$?
if [ "$pyrc" -ne 0 ]; then
    rc=1
fi
exit "$rc"
