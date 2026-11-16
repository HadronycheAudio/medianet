#!/usr/bin/env bash
# 004-29 — commissioning window announcement and close messages.
#
# RED-phase document check (tester). It validates the "Commissioning window messages"
# section of doc/design/spec/wire.md and the two messages' entries in
# doc/design/spec/wire.toml against the acceptance criteria of the ticket, and re-runs
# the wire-check binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing on
# throwaway fixtures only: when either path is overridden the cargo checks are skipped
# with a NOTE, because they always target the real repository.
#
# The author contract (what the checker locates; nothing else is fixed):
#   ## Commissioning window messages  the placeholder is gone; prose paragraphs carry evidence tags
#   ### <announcement>                 the first field table is the announcement: the seven rows of the
#                                      Hello common control header, then its own rows, tiling from byte 0;
#                                      a sentence "maximum size ... N bytes" equal to the table sum; a
#                                      Message type row whose valid range states the code ("48 only")
#   ### <close>                        the same for the close
#   ### Receiver rules                 prose paragraphs: the open transition (announcement, stamp), the expiry
#                                      transition (expiry, close) and the explicit close, each a paragraph of
#                                      its own; the outcome of a second announcement while a window is open
#   ### Validity rules                 an "Invalid form | Required outcome" table (any subsection may hold one)
#   "**Open item" paragraphs          owner and falsifier named
#   regime stamp, Open item 11        a status sentence naming this section (a), or the unchanged text plus a
#                                      new open item here that names Open item 11 (b)
#   wire.toml                         one [[message]] per message with no section_of, layer control, DSCP 40,
#                                      max_size the table sum, fields row for row (offset and width in bits)
#
# What is checked, per acceptance criterion:
#   AC 1  both messages: a type code in the block the Hello section gives this ticket, distinct; a field
#         table with the common control header, tiling, size stated and within 1 252 bytes; a tool identity
#         row; a window identifier stated stable and unique
#   AC 2  the expiry row: width equal to the merged commissioning window section's expiry, a unit, the clock
#         network time, absolute; the window identifier width equal to the merged stamp's, in both messages
#   AC 3  receiver rules: open on announcement, expiry closes, explicit close; the second announcement's outcome
#   AC 4  invalid forms: at least 6; the control violations name their merged reason exactly (TooShort,
#         BadMagic, VersionMismatch, ForeignDomain, OversizeControl, ReservedNonZero) and are not ignored;
#         expiry out of range, an expiry already past and a close for an unknown window each state an outcome
#   AC 5  wire.toml agrees with the tables; wire-check clean; wire_tables 0 failed, at least the merged count
#   DoD   evidence tags, owners and falsifiers, merged tables unchanged, Open item 11 decided where this text
#         decides it, reserved fields in the merged Reserved fields table, no ref to the main branch, no sibling
#         check run
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
DSCP_CONTROL = const_int("C-13", r"control CS5 \((\d+)\)")
check("AC1", CTRL_BOUND == 1252, f"the constants table gives the control bound (C-17) as 1 252 bytes (read {CTRL_BOUND})")
check("AC2", DSCP_CONTROL == 40, f"the constants table gives control CS5 as 40 (C-13) (read {DSCP_CONTROL})")

SIZE_MAX_RE = re.compile(r"maximum[^.;]{0,120}?(\d+)\s*bytes", re.I)
UNIT_RE = re.compile(r"nanosecond|microsecond|millisecond|\bns\b|µs|\bus\b|\bms\b|\bsecond", re.I)
CLOCK_RE = re.compile(r"network time", re.I)
GENERIC = [
    ("TooShort", r"too short|under 4 bytes|under four bytes|fewer than 4 bytes"),
    ("BadMagic", r"bad magic"),
    ("VersionMismatch", r"version mismatch"),
    ("ForeignDomain", r"foreign domain"),
    ("OversizeControl", r"oversize"),
    ("ReservedNonZero", r"reserved non-?zero|scheme non-?zero|tag non-?zero"),
]
NONREJ_RE = re.compile(r"not a rejection|never a rejection|\bignored\b|no reason", re.I)

# ---------------------------------------------------------------------------
# Merged text, read at run time (nothing copied): the Hello common control header, the
# Hello's control message type codes, the stamp's commissioning window section, the
# Rejection reasons enumeration, the Reserved fields table
# ---------------------------------------------------------------------------
HELLO_BODY = body(span_of(r"^hello$", 2))
CCH_HIT = find_sub(subsections(HELLO_BODY), r"common control header")
CCH_T = first_table(CCH_HIT[0][1], lambda h: is_field_table(h)) if len(CCH_HIT) == 1 else None
CCH = fields_of(CCH_T, "AC1", "the Hello common control header") if CCH_T else []
CCH_ROWS = [(f["off"], f["bits"]) for f in CCH]
check("DoD", CCH_ROWS == [(0, 16), (2, 16), (4, 16), (6, 8), (7, 8), (8, 64), (16, 128)],
      f"the merged common control header of the Hello section is unchanged (read {CCH_ROWS})")

HELLO_CODES_T = None
for _t in parse_tables(WIRE):
    if _t[1] and _t[1][0].lower().startswith("codes") and any(len(r) > 1 and re.search(r"hello", r[1], re.I) for r in _t[2]):
        HELLO_CODES_T = _t
CODES_FIRST = [r[0].strip() for r in HELLO_CODES_T[2]] if HELLO_CODES_T else []
check("DoD", CODES_FIRST == ["0", "1", "2 to 15", "16 to 31", "32 to 35", "36 to 47", "48 to 51", "52 to 255"],
      f"the merged control message type codes keep their code column (read {CODES_FIRST})")
CODE_LO = CODE_HI = None
for _r in (HELLO_CODES_T[2] if HELLO_CODES_T else []):
    if len(_r) >= 3 and "004-29" in _r[2]:
        _c = re.search(r"(\d+)\s+to\s+(\d+)", _r[0])
        if _c:
            CODE_LO, CODE_HI = int(_c.group(1)), int(_c.group(2))
check("AC1", CODE_LO is not None, f"the control message type codes give this ticket a block (read {CODE_LO} to {CODE_HI})")

STAMP_BODY = body(span_of(r"^regime\s+stamp$", 2))
CW_HIT = find_sub(subsections(STAMP_BODY), r"commissioning window section")
CW_T = first_table(CW_HIT[0][1], lambda h: is_field_table(h)) if len(CW_HIT) == 1 else None
CW = fields_of(CW_T, "AC2", "the merged commissioning window section") if CW_T else []
CW_WIN = [f["bits"] for f in CW if re.match(r"window identifier", f["meaning"], re.I)]
CW_EXP = [f["bits"] for f in CW if re.match(r"expiry time", f["meaning"], re.I)]
check("AC2", len(CW_WIN) == 1 and len(CW_EXP) == 1,
      f"the merged commissioning window section has one window identifier row and one expiry row (read {CW_WIN}, {CW_EXP})")
W_STAMP = CW_WIN[0] if CW_WIN else None
E_STAMP = CW_EXP[0] if CW_EXP else None

REJ_VALUE = {}
REJ_LAYERS = {}
for _t in parse_tables("\n".join(body(span_of(r"^rejection\s+reasons$", 2)))):
    for _r in _t[2]:
        if len(_r) >= 4 and re.fullmatch(r"\d+", _r[0].strip()) and re.fullmatch(r"[A-Z][A-Za-z0-9]+", _r[1].strip()):
            REJ_VALUE[_r[1].strip()] = int(_r[0].strip())
            REJ_LAYERS[_r[1].strip()] = _r[3]
CONTROL_MEMBERS = {n for n, lay in REJ_LAYERS.items() if "control" in lay}
check("DoD", len(REJ_VALUE) == 13 and {"TooShort", "BadMagic", "VersionMismatch", "ForeignDomain", "OversizeControl",
                                       "ReservedNonZero", "BadLength"} <= CONTROL_MEMBERS,
      f"the merged Rejection reasons enumeration keeps its 13 members, with the control reasons (read {len(REJ_VALUE)})")

RES_T = None
for _t in parse_tables(WIRE):
    if _t[1] and _t[1][0].lower().startswith("field") and len(_t[1]) > 1 and "location" in _t[1][1].lower():
        RES_T = _t
RES_ROWS = RES_T[2] if RES_T else []

# ---------------------------------------------------------------------------
# The section of this ticket
# ---------------------------------------------------------------------------
NEW_SPAN = span_of(r"^commissioning\s+window\s+messages$", 2)
NEW_LINES = body(NEW_SPAN)
NEW = "\n".join(NEW_LINES)
check("DoD", NEW_SPAN is not None and "to be filled" not in NEW.lower(),
      "wire.md has a Commissioning window messages section with its placeholder replaced")
SUBS = subsections(NEW_LINES)
ANN_H = find_sub(SUBS, r"announc")
CLO_H = find_sub(SUBS, r"\bclose\b")
RUL_H = find_sub(SUBS, r"receiver rule")
check("AC1", len(ANN_H) == 1, f"exactly one subsection names the announcement (found {len(ANN_H)})")
check("AC1", len(CLO_H) == 1, f"exactly one subsection names the close (found {len(CLO_H)})")
check("AC3", len(RUL_H) == 1, f"exactly one subsection holds the receiver rules (found {len(RUL_H)})")
ANN_TXT = ANN_H[0][1] if len(ANN_H) == 1 else ""
CLO_TXT = CLO_H[0][1] if len(CLO_H) == 1 else ""
RUL_TXT = RUL_H[0][1] if len(RUL_H) == 1 else ""


def msg_analysis(text, title, label):
    """The message's field table: header rows, tiling, stated size, code. None when there is no table."""
    tb = first_table(text, lambda h: is_field_table(h))
    if tb is None:
        check(label, False, f"the {title} has no field table (Offset, Width, Type, Meaning, Valid range)")
        return None
    fl = fields_of(tb, label, title)
    hdr = [(f["off"], f["bits"]) for f in fl[:7]]
    check(label, len(fl) >= 7 and hdr == CCH_ROWS,
          f"the {title} starts with the seven rows of the common control header of the Hello section (read {hdr[:7]})")
    pos = 0
    tiles = True
    for f in fl:
        if f["bits"] % 8 or f["off"] != pos:
            tiles = False
        pos = f["off"] + f["bits"] // 8
    check(label, tiles, f"the {title} table tiles from byte 0 with no gap or overlap (its rows end at byte {pos})")
    stated = [int(x) for x in SIZE_MAX_RE.findall(text)]
    check(label, pos in stated and pos <= CTRL_BOUND,
          f"the {title} states its maximum size as its table sum, {pos} bytes, within the control bound (C-17) of "
          f"{CTRL_BOUND} bytes (stated {stated})")
    code = None
    for f in fl:
        if f["off"] == 6:
            _m = re.match(r"\s*(\d+)\s+only", f["valid"])
            code = int(_m.group(1)) if _m else None
    check(label, code is not None and CODE_LO is not None and CODE_LO <= code <= CODE_HI,
          f"the {title} has a type code, in its Message type row, inside the block the Hello section gives this ticket "
          f"({CODE_LO} to {CODE_HI}) (found {code})")
    return {"fl": fl, "end": pos, "code": code}


ANN = msg_analysis(ANN_TXT, "announcement", "AC1") if ANN_TXT else None
CLO = msg_analysis(CLO_TXT, "close", "AC1") if CLO_TXT else None
if ANN and CLO:
    check("AC1", ANN["code"] != CLO["code"], f"the announcement and the close have distinct type codes (found {ANN['code']} and {CLO['code']})")

# ---------------------------------------------------------------------------
# AC 2: the clock and the form of the expiry. Run-time term list from timeline.md (the clock terms it
# uses); the expiry may be an absolute instant or a relative duration, one form only, and a relative
# duration needs the instant it counts from, the late-or-repeated rule, and an updated merged section.
# ---------------------------------------------------------------------------
TL_TEXT = read_text("doc/design/spec/timeline.md") or ""
TL_STOP = {"a", "an", "the", "every", "of", "on", "its", "this", "one", "whose", "with", "from", "to", "at", "as",
           "is", "by", "each", "any", "and", "or", "in", "for", "no", "has", "keeps", "all", "such", "same", "that",
           "when", "be", "it", "only", "their", "which", "if", "but", "not", "into", "over", "under", "during",
           "after", "before", "does", "do", "must", "may", "can", "instant"}
TL_EXCL = {"time", "clock", "origin time", "exact time", "usable time", "compile time", "published time", "representable time", "pre-origin time",
           "same time", "such time", "no time", "this time", "wall time", "gap time"}


def clock_terms(text):
    t = text.lower().replace("`", "").replace("wall-clock", "wall clock")
    out = set()
    for m in re.finditer(r"\b((?:[a-z][a-z']*\s+){0,3}?(?:time|clock))\b", t):
        words = m.group(1).split()
        cut = [i for i, w in enumerate(words[:-1]) if w in TL_STOP]
        words = words[cut[-1] + 1:] if cut else words
        if len(words) > 3 or any(w == "s" or "'" in w for w in words):
            continue
        term = " ".join(words)
        if term and term not in TL_EXCL:
            out.add(term)
    return sorted(out)


TL_TERMS = clock_terms(TL_TEXT)
TL_SHARED = [t for t in TL_TERMS if "network time" in t]
EXTRA_CLOCKS = ["wall clock", "utc", "tool's clock"]
print(f"NOTE clock terms extracted from timeline.md: {TL_TERMS}; shared (evaluable by every module): {TL_SHARED}")
EVAL_RE = re.compile(r"evaluat|convert|does not share|do not share|without the holder|on its own clock|maps? it", re.I)
EVAL_PARAS = [x for k, x in prose_blocks(ANN_TXT) if k == "p" and has_evidence(x) and EVAL_RE.search(x)]
ANN_PARAS_TAGGED = [x for k, x in prose_blocks(ANN_TXT) if k == "p" and has_evidence(x)]

REL_FORM = [False]


def sentences_of(text):
    return [x for x in re.split(r"(?<=[.;])\s+|\n\s*\n", text) if x.strip()]


if ANN:
    fl = ANN["fl"]
    wid = [f for f in fl if re.match(r"window identifier", f["meaning"], re.I)]
    exp = [f for f in fl if re.match(r"expiry", f["meaning"], re.I)]
    tool = [f for f in fl if re.search(r"tool", f["meaning"], re.I) and re.search(r"identit", f["meaning"], re.I)
            and not re.match(r"window identifier", f["meaning"], re.I)]
    check("AC1", len(wid) == 1, f"the announcement has one window identifier row (found {len(wid)})")
    check("AC1", len(tool) == 1 and tool[0]["bits"] > 0,
          f"the announcement has one row for the announcing tool's identity, with a width (found {len(tool)})")
    check("AC1", bool(re.search(r"unique", ANN_TXT, re.I)) and bool(re.search(r"stable", ANN_TXT, re.I)),
          "the announcement's text states the window identifier stable and unique, so results can reference it (C-178, Work 1)")
    check("AC2", len(wid) == 1 and W_STAMP is not None and wid[0]["bits"] == W_STAMP,
          f"the announcement's window identifier is as wide as the merged stamp's, {W_STAMP} bits "
          f"(read {[f['bits'] for f in wid]})")
    check("AC2", len(exp) == 1 and E_STAMP is not None and exp[0]["bits"] == E_STAMP,
          f"the announcement's expiry is as wide as the merged stamp's expiry, {E_STAMP} bits, so the stamp can copy it "
          f"(read {[f['bits'] for f in exp]})")
    # the clock: a term timeline.md uses, named in the expiry row or in a tagged sentence of the announcement
    clock_src = " ".join([f["meaning"] + " " + f["valid"] for f in fl] + [x for k, x in prose_blocks(ANN_TXT)
                                                                       if k == "p" and has_evidence(x)])
    named = [t for t in TL_TERMS if re.search(r"\b" + re.escape(t) + r"\b", clock_src.lower())]
    named_extra = [t for t in EXTRA_CLOCKS if re.search(r"\b" + re.escape(t) + r"\b", clock_src.lower())]
    named_shared = [t for t in named if t in TL_SHARED]
    named_local = [t for t in named if t not in TL_SHARED]
    check("AC2", bool(named_shared) or ((named_local or named_extra) and bool(EVAL_PARAS)),
          "the announcement names its clock: a clock timeline.md defines (network time), or a holder-local or other "
          "clock with a tagged sentence on how a module that does not share it evaluates the expiry (Work 1, C-153) "
          f"(named {named + named_extra})")
    if len(exp) == 1:
        em = exp[0]["meaning"]
        ev = em + " " + exp[0]["valid"]
        check("AC2", bool(UNIT_RE.search(ev)), "the expiry row states its unit (Work 1; C-153)")
        row_all = em + " " + exp[0]["valid"]
        rel = bool(re.search(r"relative|duration", row_all, re.I))
        absf = bool(re.search(r"\babsolute\b", row_all, re.I))
        check("AC2", rel != absf, "the expiry row states one form: an absolute instant or a relative duration (Open item 11)")
        form_word = "relative" if rel else "absolute"
        check("AC2", any(re.search(r"expir", x, re.I) and re.search(form_word, x, re.I) for x in ANN_PARAS_TAGGED),
              f"a tagged sentence of the announcement states the expiry form ({form_word}) (review T2)")
        if rel:
            check("AC2", bool(re.search(r"counted from|counts from|from the issue time|from the instant", ev + " " + ANN_TXT, re.I)),
                  "the relative expiry states the instant it counts from (Open item 11, branch b)")
            check("AC2", bool(re.search(r"\blate\b|twice", ANN_TXT, re.I)) and bool(re.search(r"same expiry|derives? the same", ANN_TXT, re.I)),
                  "the relative expiry states how a late or repeated announcement derives the same expiry (Open item 11, branch b)")
            cw_exp_row = [f for f in CW if re.match(r"expiry time", f["meaning"], re.I)]
            merged_ok = bool(cw_exp_row) and "copied from the announcement" not in cw_exp_row[0]["valid"] \
                and bool(re.search(r"deriv", cw_exp_row[0]["meaning"] + " " + cw_exp_row[0]["valid"], re.I))
            check("AC2", merged_ok, "the merged commissioning window section no longer says the stamped expiry is copied "
                                    "from the announcement: it says the stamped value is derived (Open item 11, branch b)")
            REL_FORM[0] = True

if CLO:
    cwid = [f for f in CLO["fl"] if re.match(r"window identifier", f["meaning"], re.I)]
    check("AC1", len(cwid) == 1, f"the close names the window it closes by its window identifier row (found {len(cwid)})")
    check("AC2", len(cwid) == 1 and W_STAMP is not None and cwid[0]["bits"] == W_STAMP,
          f"the close's window identifier is as wide as the merged stamp's, {W_STAMP} bits (read {[f['bits'] for f in cwid]})")
    check("AC1", len([f for f in CLO["fl"] if f["off"] >= 32]) == 1,
          "the close lays out only the identifier after the common control header (Work 2)")

# Invalid forms of the section: every "Invalid form | Required outcome" table, in any subsection
ROWS = []
for _t in parse_tables(NEW):
    if _t[1] and _t[1][0].lower().startswith("invalid form") and len(_t[1]) > 1 and "required outcome" in _t[1][1].lower():
        ROWS += [r for r in _t[2] if len(r) >= 2 and r[0].strip()]
check("AC4", len(ROWS) >= 6, f"at least 6 invalid forms with outcomes across the two messages (found {len(ROWS)})")
for r in ROWS:
    form, outcome = r[0], r[1]
    names = [t for t in re.findall(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b", outcome) if t in REJ_VALUE]
    outside = [n for n in names if n not in CONTROL_MEMBERS]
    if outside:
        fail("AC4", f"an invalid form names a reason outside the control layer {outside}: {form[:60]}")
        continue
    gen = [name for name, pat in GENERIC if re.search(pat, form, re.I)]
    for name in gen:
        if name not in names:
            fail("AC4", f"the control violation '{form[:60]}' must name {name}, as the merged shared rows do: {outcome[:60]}")
        if NONREJ_RE.search(outcome):
            fail("AC4", f"the control violation '{form[:60]}' is marked ignored or not a rejection; it is a counted rejection")
        if name == "VersionMismatch" and not re.search(r"event", outcome, re.I):
            fail("AC4", "the version mismatch form does not raise the named VersionMismatch event")
    if not gen and not names and not NONREJ_RE.search(outcome):
        fail("AC4", f"the form '{form[:60]}' states no outcome: no Reject reason and no non-rejection")
PRESENT = [
    ("the expiry out of range", r"expir.*range|range.*expir|expir.*too (large|high)"),
    ("an expiry already past", r"expir.*(past|already|elapsed|before)|(past|already|elapsed).*expir"),
    ("a close for an unknown window", r"clos.*(unknown|not held|does not hold|no open|names no|not open)|"
                                      r"(unknown|not held|does not hold|no open|names no).*(window|clos)"),
]
for label, pat in PRESENT:
    hit = [r for r in ROWS if re.search(pat, r[0], re.I)]
    check("AC4", bool(hit) and any(re.search(r"\S", r[1]) for r in hit),
          f"an invalid form covers {label}, with its outcome stated (ticket item 4)")

# Second announcement: a stated outcome, in the receiver rules or an invalid form
RUL_PARAS = [x for k, x in prose_blocks(RUL_TXT) if k == "p"]
SECOND_RE = r"second|another announcement|already open|while (a |the )?window is open"
VERB_RE = r"replac|ignor|extend|keep|discard|stays|unchanged|restart"
second_hit = [x for x in RUL_PARAS + [" ".join(r) for r in ROWS]
              if re.search(SECOND_RE, x, re.I) and re.search(VERB_RE, x, re.I)]
check("AC3", bool(second_hit), "a second announcement while a window is open has a stated outcome (replace, ignore or extend)")

# AC 3 (review T1): the three C-153 transitions are anchored to the receiver rules themselves: one numbered
# (or "Rule N") item for the open transition, one for the expiry, one for the explicit close, all different.
# Each rule's bold label names its transition; its first sentence carries the verb of the transition.
def parse_rules(text):
    rules, cur = [], None
    for line in text.splitlines():
        if re.match(r"^\s*(\d+\.\s|\*\*Rule \d+)", line):
            cur = [line]
            rules.append(cur)
        elif not line.strip():
            cur = None
        elif cur is not None:
            cur.append(line)
    return [" ".join(r) for r in rules]


def rule_parts(rule):
    lab = re.search(r"\*\*([^*]+)\*\*", rule)
    label = lab.group(1).strip().rstrip(".").lower() if lab else ""
    rest = re.sub(r"^\s*(\d+\.|\*\*Rule \d+\*\*)\s*", "", rule)
    rest = re.sub(r"\*\*[^*]+\*\*", "", rest, count=1).strip()
    return label, re.split(r"(?<=\.)\s", rest, 1)[0].lower()


RULES = parse_rules(RUL_TXT)
RULE_INFO = [rule_parts(r) for r in RULES]


def rule_cands(pos_label, neg_label, first_req):
    return [i for i, (lab, first) in enumerate(RULE_INFO)
            if re.search(pos_label, lab) and not re.search(neg_label, lab) and re.search(first_req, first)]


# Review round 2: a transition rule's first sentence applies a present-tense closing verb to the window, and
# the rule is not about what happens after a close, about memory of a closed window, or about a past expiry.
CLOSE_VERB = r"closes (?:its |the |it\b)|window closes|closes it\b|ends the window"
NEG_STATE = r"stays|remember|memor|already|past|\bafter\b|closed window|ignores"


def transition_cands(label_pos, label_neg, first_pos, need_close_verb):
    out = []
    for i, (lab, first) in enumerate(RULE_INFO):
        text = lab + " " + first
        if not re.search(label_pos, lab) or re.search(label_neg, lab) or re.search(NEG_STATE, text):
            continue
        if not re.search(first_pos, first):
            continue
        if need_close_verb and not re.search(CLOSE_VERB, first):
            continue
        out.append(i)
    return out


OPEN_C = transition_cands(r"\bopen", r"^$", r"\bopens\b", False)
EXP_C = transition_cands(r"expir", r"unknown|late|copy|repeat|after|second", r"expir|reached", True)
EXPL_C = transition_cands(r"explicit|close", r"unknown|late|copy|repeat|after|already|second",
                          r"explicit close|close message|a close|valid close", True)
check("AC3", bool(OPEN_C), f"the receiver rules hold a numbered rule for the open transition (found rules {OPEN_C})")
check("AC3", bool(EXP_C), f"the receiver rules hold a numbered rule for the expiry transition (found rules {EXP_C})")
check("AC3", bool(EXPL_C), f"the receiver rules hold a numbered rule for the explicit close (found rules {EXPL_C})")
check("AC3", any(len({o, e, x}) == 3 for o in OPEN_C for e in EXP_C for x in EXPL_C),
      "the open, expiry and explicit-close transitions are three different receiver rules (C-153: announced, "
      "modules stamp it while open, closes at expiry or on explicit close)")

# ---------------------------------------------------------------------------
# DoD: evidence tags, owners and falsifiers, reserved fields, Open item 11
# ---------------------------------------------------------------------------
for title, txt in [("announcement", ANN_TXT), ("close", CLO_TXT), ("receiver rules", RUL_TXT)]:
    check("DoD", bool(txt) and has_evidence(txt), f"the {title} subsection carries an evidence tag")
untagged = [x[:50] for k, x in prose_blocks(NEW) if k == "p"
            and len(re.sub(r"[`*_|>#-]", " ", x).split()) >= 5 and not has_evidence(x)]
check("DoD", not untagged, "every prose paragraph of 5 or more words in the section carries an evidence tag"
      + (f"; untagged: {untagged[:2]}" if untagged else ""))
OPEN_PARAS = [x for k, x in prose_blocks(NEW) if k == "p" and re.match(r"\s*\**\s*Open item", x)]
check("DoD", all(re.search(r"owner", x, re.I) and re.search(r"falsifier", x, re.I) for x in OPEN_PARAS),
      f"every open item of the section names an owner and a falsifier ({len(OPEN_PARAS)} found)")

NEW_FIELDS = (ANN["fl"] if ANN else []) + (CLO["fl"] if CLO else [])
NEEDS_RES = [f for f in NEW_FIELDS if f["off"] >= 32
             and re.search(r"reserved|unused|accept[- ]any|ignored|0 only", f["meaning"] + " " + f["valid"], re.I)]
RES_NEW = [r for r in RES_ROWS if len(r) > 1 and re.search(r"commissioning|004-29", r[1], re.I)]
check("DoD", not NEEDS_RES or bool(RES_NEW),
      f"every reserved or accept-any field of the new messages ({len(NEEDS_RES)} found) has a row in the merged "
      f"Reserved fields table ({len(RES_NEW)} rows name this ticket)")

STAMP_SPAN = span_of(r"^regime\s+stamp$", 2)
OI11 = [x for k, x in prose_blocks("\n".join(body(STAMP_SPAN))) if k == "p" and re.match(r"\s*\**\s*Open item 11\b", x)]
OI11 = OI11[0] if OI11 else ""
oi11_status = (bool(re.search(r"status", OI11, re.I)) and bool(re.search(r"commissioning window messages|004-29", OI11, re.I))
               and bool(re.search(r"64|width", OI11)))
oi11_open = ("ticket 004-29 (commissioning window announcement and close messages)" in OI11
             and any(re.search(r"Open item 11", x) for x in OPEN_PARAS))
check("DoD", bool(OI11) and (oi11_status or oi11_open),
      "Open item 11 of the regime stamp is no longer open where this text decides it: a status sentence names this "
      "section (a), or the item is unchanged and a new open item here names it (b)")

# ---------------------------------------------------------------------------
# Review findings of PR #37 (F1, F2, F3, F7): stated outcomes and a counter denial kept narrow
# ---------------------------------------------------------------------------
ALL_PARAS = [x for k, x in prose_blocks(NEW) if k == "p"] + [" ".join(r) for _t in parse_tables(NEW) for r in _t[2]]

# F3: a late or repeated announcement of a window already closed by an explicit close has one stated outcome.
# Source: Work 3 "stated outcomes" (004-29) and C-153 "it closes ... on explicit close".
AFTER_CLOSE_RE = (r"after (?:the |an |its |a )?(?:window'?s? )?(?:explicit )?close\b|already closed|closed window|"
                  r"closed by|had closed|explicitly closed")
after_close = [x for x in ALL_PARAS if re.search(AFTER_CLOSE_RE, x, re.I)
               and re.search(r"announc|copy|repeat", x, re.I) and re.search(r"ignored|reopen|remember|refus", x, re.I)]
check("AC4", bool(after_close), "a late or repeated announcement of a window the module closed by an explicit close has a "
      "stated outcome: ignored and remembered, or reopens (Work 3 stated outcomes; C-153 closes on explicit close) (review F3)")
if any(re.search(r"\breopen", x, re.I) and not re.search(r"not reopen|does not reopen|do not reopen|never reopen", x, re.I)
       for x in after_close):
    check("DoD", any(re.search(r"clos", x, re.I) and re.search(r"reopen|late|after|repeat", x, re.I)
                     and re.search(r"owner", x, re.I) and re.search(r"falsifier", x, re.I) for x in OPEN_PARAS),
          "the reopening outcome after a close is an open item with an owner and a falsifier (review F3)")

# F2: the section states whether two modules can hold different windows at once. Any stated answer passes.
# Source: Work 3 "stated outcomes" (004-29) and C-153 (one tool's window, open until expiry or explicit close).
TWO_RE = lambda x: re.search(r"\btwo\b", x, re.I) and re.search(r"windows?\b", x, re.I) \
    and re.search(r"at once|different|same time", x, re.I) and re.search(r"\bmodules?\b", x, re.I) \
    and not re.search(r"falsifier:|owner:", x, re.I)
two_hits = [x for x in RUL_PARAS + OPEN_PARAS if TWO_RE(x) and has_evidence(x)]
check("AC3", bool(two_hits), "the section states whether two modules can hold different windows at once "
      "(Work 3 stated outcomes; C-153) (review F2)")

# F7: a module whose time is unusable and whose close was lost keeps the window with no bound: stated.
# Source: Work 3 "stated outcomes" (004-29) and C-153 "it closes ... on explicit close".
unbounded = [x for k, x in prose_blocks(NEW) if k == "p" and re.search(r"time state|unusable", x, re.I)
             and re.search(r"\blost\b|\bnever\b", x, re.I) and re.search(r"\bclose", x, re.I)]
check("AC3", bool(unbounded), "the case of a module whose time is unusable and whose close was lost is stated "
      "(Work 3 stated outcomes; C-153 closes on explicit close) (review F7)")

# F1: the message counter. Merged text: Hello, Common control header, offset 8: "in every control message it is the
# sender's liveness counter (Liveness counter)"; C-115: "the 64-bit counter is live because it is the liveness
# counter". A denial of that is allowed only as a reported contradiction or an open item naming Hello and C-115.
DENY_RE = r"cannot be a (?:data-plane )?liveness counter|not a (?:data-plane )?liveness counter|cannot be the liveness counter"
for x in [x for x in ALL_PARAS if re.search(DENY_RE, x, re.I)]:
    ok = (bool(re.search(r"reported contradiction|present disagreement", x, re.I)) or bool(re.match(r"\s*\**\s*Open item", x))) \
        and "Hello" in x and "C-115" in x and bool(re.search(r"owner", x, re.I)) and bool(re.search(r"falsifier", x, re.I))
    check("DoD", ok, "a denial that the tool's message counter is a liveness counter is a reported contradiction or an open "
          "item, naming the merged Hello row and C-115, with an owner and a falsifier (review F1; Hello, Common control "
          "header, offset 8; C-115)")

if REL_FORM[0]:
    check("DoD", oi11_status, "Open item 11 of the regime stamp carries a status sentence for the relative form (branch b)")

# ---------------------------------------------------------------------------
# AC 5: wire.toml declares the two messages, row for row with the tables
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
# DoD (tightened): a new regime stamp section type restates the cap threshold, 18 entries per section
NEW_SECTION_TYPE = re.search(r"section type (?:code )?(?:[1-9]|[12]\d|3[01])\b", NEW, re.I)
NEW_STAMP_MSG = [m for m in msgs if "section_of" in m and re.search(r"commission", str(m.get("name", "")), re.I)
                 and re.search(r"announc|close", str(m.get("name", "")), re.I)] \
    if TOML_OK else []
if NEW_SECTION_TYPE or NEW_STAMP_MSG:
    check("DoD", any(re.search(r"18 entries", x) for x in sentences_of(NEW)),
          "a regime stamp section type added here restates the connection-cap threshold: a sentence with 18 entries "
          "per section (Open item 1, connection cap; hand-off from status.md)")

for label, md, pat in [("announcement", ANN, r"announc"), ("close", CLO, r"close")]:
    cand = [m for m in msgs if re.search(r"commission", str(m.get("name", "")), re.I)
            and re.search(pat, str(m.get("name", "")), re.I) and "section_of" not in m]
    check("AC5", len(cand) == 1, f"wire.toml declares exactly one commissioning window {label} message (found {len(cand)})")
    if len(cand) != 1 or md is None:
        continue
    m = cand[0]
    check("AC5", m.get("layer") == "control" and int(m.get("dscp", -1)) == DSCP_CONTROL and int(m.get("max_size", -1)) == md["end"],
          f"the {label} message of wire.toml is control, DSCP {DSCP_CONTROL}, with max_size the table's {md['end']} bytes")
    reasons = set(m.get("rejection_reasons", []))
    check("AC5", bool(reasons) and reasons <= CONTROL_MEMBERS,
          f"the {label} message names only control reasons (found {sorted(reasons)})")
    tf = m.get("field", [])
    agree = len(tf) == len(md["fl"])
    for g_, f in zip(tf, md["fl"]):
        if int(g_.get("offset", -1)) != f["off"] * 8 or int(g_.get("width", -1)) != f["bits"]:
            agree = False
    check("AC5", agree, f"the {label} fields of wire.toml agree with its table row for row (offset and width in bits)")

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
              f"cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least {MIN_WIRE_TABLES} passed, "
              f"the count at the merged text"
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
