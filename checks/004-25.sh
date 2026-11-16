#!/usr/bin/env bash
# 004-25 — regime stamp section types named by the constants table.
#
# RED-phase document check (tester). It validates the seven section types that the
# "Regime stamp" section of doc/design/spec/wire.md must lay out, and their
# regime stamp messages in doc/design/spec/wire.toml, against the acceptance
# criteria of the ticket, and re-runs the wire-check binary and the wire_tables
# tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing
# on throwaway fixtures only: when either path is overridden the cargo checks are
# skipped with a NOTE, because they always target the real repository.
#
# The author contract (what the checker locates; nothing else is fixed):
#   ## Regime stamp            the seven types each have exactly one ### subsection whose
#                              heading names the type (multi-packet, health / per-connection,
#                              calibration, commissioning, injection, capability,
#                              timestamp); the placeholder is gone
#   each type subsection       its first field table (Offset | Width | Type | Meaning | Valid range) is the
#                              section table. A list row ("E × max F") is followed, in document order and
#                              depth-first, by its element table, which may itself list a nested element table;
#                              each element table sums to E bytes, its rows tile with no gap or overlap (bit rows
#                              name their bit range "bit [hi:lo] of byte N"), and the section's maximum is
#                              computed from the stated structure at the declared maxima (T3: nesting is allowed).
#                              The section table's first row starts at 0 (body-relative) or 58 (section-relative);
#                              a "type code N" sentence with N inside the block the merged section type codes give
#                              004-25; a sentence "maximum ... N bytes" equal to the computed section size; for each
#                              root list row a statement ⌈M / F⌉ = N; evidence tags on the subsection and on each
#                              prose paragraph of 5 or more words
#   capability subsection      a table of exactly the four C-129 states; control isolation
#   timestamp subsection       a table of exactly the four C-131 sources; a C-129 capability state
#   health subsection          the eleven C-159 counters, each a row with a wrap or saturation
#                              rule and a unit; the observed DSCP
#   injection subsection       module id or nonce, epoch, raw packets, and the panic hook
#   Invalid form table         rows "Invalid form | Required outcome" for the new forms
#   wire.toml                  one [[message]] per type with section_of = "regime stamp", and one entry [[message]]
#                              per element table (T1: rows, offsets, widths, counts and total size compared bit for bit)
#   wire.toml (entries)        as above, without section_of, in document order of the element tables,
#                              layer control, dscp 40, max_size equal to the table's size,
#                              fields agreeing with the table row for row
#
# What is checked, per acceptance criterion:
#   AC 1  seven types, one code each in the merged block, sizes within 1 252 bytes,
#         the Context values, the section count arithmetic, the 64-section bound (or its report)
#   AC 2  the capability-state and timestamp-source enumerations: exactly the C-129 and C-131 names
#   AC 3  the health section: the eleven C-159 counters, each with a wrap or saturation rule, and the DSCP
#   AC 4  control isolation in the capability section, the panic hook in the injection section
#   AC 5  at least five invalid forms of the new types with outcomes naming control reasons
#   AC 6  wire.toml agrees with the tables; wire-check clean; wire_tables 0 failed, at least 12 passed
#   DoD   review findings (PR 35): C1 the overflow report is complete (the total, Open item 1, exceeded and the design
#         owner, merged sender rule 7 with "no stamp", the entry threshold, the health maximum equal to the constants'
#         product); C2 a capability named with a ticket id in a sentence has a numbered row in the capability table;
#         C3 no hand-back of a Manifest open item owned by 004-25; C4 a health entry distinguishes admissions by a
#         field or a tagged sentence; T2 counters and DSCP are matched only in the entry tables
#         Declared limits: the health maximum is checked against the constants only when the report is required (sum above 64);
#         the threshold uses the other six types' computed sections; the capability vocabulary is the list in review.py
#   DoD   merged tables unchanged, reserved fields recorded, evidence tags, open items, no ref to the main branch,
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


WIRE_RAW = read_text(wire_path)
if WIRE_RAW is None:
    fail("AC1", f"{wire_path} does not exist")
    WIRE_RAW = ""
WIRE = norm(WIRE_RAW.replace("−", "-"))
lines = WIRE.splitlines()
HEADS = heading_index(lines)
TOML_TEXT = read_text(tables_path)
if TOML_TEXT is None:
    fail("AC6", f"{tables_path} does not exist")
    TOML_TEXT = ""
CONST = norm(read_text(CONSTANTS_PATH) or "")


def const_row(cid):
    for line in CONST.splitlines():
        if line.startswith(f"| {cid} |"):
            cells = line.split("|")
            return cells[3].strip() if len(cells) > 3 else ""
    return ""


_m = re.search(r"at most (\d+) bytes", const_row("C-17"))
CTRL_BOUND = int(_m.group(1)) if _m else None
C159 = [x.strip() for x in const_row("C-159").split(",") if x.strip()]
_m = re.search(r"control CS5 \((\d+)\)", const_row("C-13"))
DSCP_CONTROL = int(_m.group(1)) if _m else None
_m = re.search(r"four: ([^.]*)\.", const_row("C-129"))
CAP_STATES = [x.strip().lower() for x in _m.group(1).split(",")] if _m else []
TS_SOURCES = [x.lower() for x in re.findall(r"([a-z][a-z-]*) \(", const_row("C-131"))]
check("AC1", bool(CTRL_BOUND) and CTRL_BOUND == 1252,
      f"the constants table gives the control bound C-17 as 1 252 bytes (read {CTRL_BOUND})")
check("AC3", len(C159) == 11,
      f"the constants table gives eleven C-159 counters, read at run time (found {len(C159)}: {C159})")
check("AC2", len(CAP_STATES) == 4,
      f"the constants table gives four C-129 capability states, read at run time (found {CAP_STATES})")
check("AC2", len(TS_SOURCES) == 4,
      f"the constants table gives four C-131 receive-timestamp sources, read at run time (found {TS_SOURCES})")

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

CODE_LO = CODE_HI = None
for _t in parse_tables(WIRE):
    if _t[1] and _t[1][0].lower().startswith("codes"):
        for r in _t[2]:
            if len(r) >= 3 and "004-25" in r[2]:
                _c = re.search(r"(\d+)\s+to\s+(\d+)", r[0])
                if _c:
                    CODE_LO, CODE_HI = int(_c.group(1)), int(_c.group(2))
check("AC1", CODE_LO is not None,
      f"the merged section type codes give 004-25 a block (read {CODE_LO} to {CODE_HI})")

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

# ---------------------------------------------------------------------------
# The seven section types (Context, Work 1)
# ---------------------------------------------------------------------------
TYPE_SPECS = [
    ("multi", r"multi[- ]?packet"),
    ("health", r"health|per[- ]connection"),
    ("calibration", r"calibration"),
    ("commissioning", r"commissioning"),
    ("injection", r"injection"),
    ("capability", r"capabilit"),
    ("timestamp", r"timestamp|time stamp"),
]
SUBS = subsections(STAMP_LINES)
TYPES = {}
for key, pat in TYPE_SPECS:
    hits = find_sub(SUBS, pat)
    good = len(hits) == 1
    check("AC1", good, f"exactly one subsection of the regime stamp section names the {key} section type (found {len(hits)})")
    if good:
        TYPES[key] = {"title": hits[0][0], "text": hits[0][1]}
check("AC1", len(TYPES) == 7, f"seven section types are laid out in the regime stamp section (found {len(TYPES)})")

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


for key, t in TYPES.items():
    text = t["text"]
    tables = parse_tables(text)
    field_tables = [tb for tb in tables if is_field_table(tb[1])]
    fl_all, ft = [], []
    for tb in field_tables:
        fl = fields_of(tb, "AC1", t["title"])
        ft.append(fl)
        fl_all += fl
    t["fl_all"] = fl_all
    if not ft or not ft[0]:
        fail("AC1", f"the {key} section type has no field table of Offset, Width, Type, Meaning, Valid range")
        t["sec_max"], t["Nt"], t["root"] = None, None, None
        continue
    root, unused = build_tree(ft)
    if unused:
        fail("AC1", f"the {key} section type has {len(unused)} field table(s) that no list row of it names as its entry")
    t["root"] = root
    t["fl_main"] = [f for f, _ in root["rows"]]
    off0 = t["fl_main"][0]["off"]
    body_rel = off0 != 58
    if off0 not in (0, 58):
        fail("AC1", f"the {key} table starts at offset {off0}: a body starts at 0 and a section at 58")
    tile_node(root, off0 * 8 + (lo_of(t["fl_main"][0]) or 0), t["title"], "AC1")
    t["entries"] = entry_nodes(root)
    rbits = size_node(root, t["title"])
    sec_max = 58 + cdiv(rbits, 8)
    t["sec_max"], t["body_rel"] = sec_max, body_rel
    check("AC1", sec_max <= CTRL_BOUND,
          f"the {key} section at its maxima is {sec_max} bytes, no more than the control bound (C-17) of {CTRL_BOUND} bytes"
          + ("" if sec_max <= CTRL_BOUND else f": over the control bound by {sec_max - CTRL_BOUND}"))
    stmts = [(int(a), int(b), int(c)) for a, b, c in STMT.findall(text)]
    Nt = 1
    for f in [f for f, _ in root["rows"] if f["maxc"]]:
        F = f["maxc"]
        cand = [s for s in stmts if s[1] == F]
        if not cand:
            fail("AC1", f"the {key} list row (at most {F} entries per section) has no statement of the form ⌈M / {F}⌉ = N")
            continue
        M, _F, N = cand[0]
        if N != cdiv(M, F):
            fail("AC1", f"the {key} section count is wrong: ⌈{M} / {F}⌉ = {cdiv(M, F)}, the text states {N}")
        Nt = max(Nt, cdiv(M, F))
    t["Nt"] = Nt
    t["M_stated"] = [s[0] for s in stmts if s[1] in [f["maxc"] for f, _ in root["rows"] if f["maxc"]]]
    stated = [int(x) for x in SIZE_MAX_RE.findall(text)]
    check("AC1", sec_max in stated,
          f"a sentence of the {key} subsection states its maximum section size as {sec_max} bytes (the table's sum)"
          + ("" if sec_max in stated else f"; stated {stated}"))
    _code = TYPE_CODE_RE.search(text)
    code = int([g for g in _code.groups() if g][0]) if _code else None
    t["code"] = code
    check("AC1", code is not None and CODE_LO is not None and CODE_LO <= code <= CODE_HI,
          f"the {key} section type has a type code inside the block of 004-25 ({CODE_LO} to {CODE_HI}) (found {code})")

codes = [t["code"] for t in TYPES.values() if t.get("code") is not None]
check("AC1", len(TYPES) == 7 and len(codes) == len(set(codes)) == len(TYPES), f"the seven section type codes are distinct (found {codes})")

# Products and sums in the regime stamp section, computed (lesson 4)
_bad = []
for a, b, c in re.findall(r"(\d+)\s*(?:×|\bx\b)\s*(\d+)\s*=\s*(\d+)", STAMP):
    if int(a) * int(b) != int(c):
        _bad.append(f"{a} x {b} = {c}")
for expr, c in re.findall(r"([\d\s\u00d7*+-]*\d[\d\s\u00d7*+-]*)=\s*(\d+)", STAMP):
    if re.search(r"[+*\u00d7-]", expr.strip()) and re.search(r"\d\s*[+*\u00d7-]\s*\d", expr):
        _val = eval(re.sub(r"\s", "", expr).replace("\u00d7", "*"))
        if _val != int(c):
            _bad.append(f"{expr.strip()} = {c}")
check("AC1", not _bad, "every product and sum stated in the regime stamp section is exact"
      + (f": {_bad[:4]}" if _bad else ""))

_S = sum(t["Nt"] for t in TYPES.values() if t.get("Nt"))
_report64 = re.search(r"(64[^.]{0,100}(exceed|more than|above|over)|(exceed|more than|above|over)[^.]{0,100}64)[^.]*Open item 1|Open item 1[^.]*(64[^.]{0,100}(exceed|more than|above|over)|(exceed|more than|above|over)[^.]{0,100}64)", STAMP, re.I)
check("AC1", len(TYPES) == 7 and (_S <= (SECTION_MAX or 64) or bool(_report64)),
      f"the seven types need {_S} sections at their maxima; that fits the 64-section bound, or the text reports the excess and names Open item 1")
# ---------------------------------------------------------------------------
# Per-type content: Context items, AC 2, AC 3, AC 4
# ---------------------------------------------------------------------------
def rows_text(fl):
    return [f["meaning"] + " " + f["valid"] for f in fl]


def max_match(keys, texts):
    cand = [[j for j, t in enumerate(texts) if re.search(k, t, re.I)] for k in keys]
    owner = {}

    def aug(i, seen):
        for j in cand[i]:
            if j in seen:
                continue
            seen.add(j)
            if j not in owner or aug(owner[j], seen):
                owner[j] = i
                return True
        return False

    matched = sum(1 for i in range(len(keys)) if aug(i, set()))
    return matched, {i: j for j, i in owner.items()}


def counter_re(n):
    special = {"marked gap": r"marked[\s-]+gap", "other-epoch": r"other[\s-]+epoch",
               "too-early": r"too[\s-]+early", "incomplete reassemblies": r"incomplete[\s-]+reassembl",
               "demoted rounds": r"demoted", "interrupted events": r"interrupted",
               "duplicates": r"duplicat", "dead": r"\bdead\b", "received": r"\breceiv",
               "late": r"\blate\b", "missing": r"\bmissing\b"}
    return special.get(n, re.escape(n))


# AC 1 (Context): each type carries the values the Context names
if "multi" in TYPES:
    mf = TYPES["multi"]["fl_all"]
    mt = rows_text(mf)
    for label, pat in [("the stream number", r"stream"), ("the packets per slice", r"packet"),
                       ("the frames per slice or its equivalent", r"frame|slice length|slice")]:
        check("AC1", any(re.search(pat, t, re.I) for t in mt),
              f"the multi-packet section carries {label} (Context, the values 004-08 hands over)")
if "commissioning" in TYPES:
    check("AC1", any(re.search(r"expir", f["meaning"], re.I) for f in TYPES["commissioning"]["fl_all"]),
          "the commissioning window section carries the expiry time of the open window (Context, C-153)")
if "calibration" in TYPES:
    check("AC1", bool(re.search(r"calibration factor", TYPES["calibration"]["text"], re.I))
          and any(re.search(r"operation", f["meaning"], re.I) for f in TYPES["calibration"]["fl_all"]),
          "the calibration section carries the calibration factor, time per counted operation (Context, C-106)")
if "injection" in TYPES:
    _it = TYPES["injection"]["text"]
    for label, pat, ac in [("a forced module id or nonce", r"module id|nonce", "AC1"),
                           ("a stream forced onto a timeline epoch", r"epoch", "AC1"),
                           ("raw packets on a media or control socket", r"\braw\b", "AC1"),
                           ("a panic injected into a thread (AC 4: the panic hook)", r"panic", "AC4")]:
        check(ac, bool(re.search(pat, _it, re.I)), f"the injection section names {label} (Context, C-120, C-145)")
if "capability" in TYPES:
    check("AC4", bool(re.search(r"isolat", TYPES["capability"]["text"], re.I)),
          "the capability section includes the control-isolation capability (AC 4, C-161)")
if "timestamp" in TYPES:
    check("AC1", bool(re.search(r"capabilit\w*\s+state|C-129", TYPES["timestamp"]["text"], re.I)),
          "the receive-timestamp section reports its source with a C-129 capability state (Context, C-131)")

# AC 2: capability states and timestamp sources, closed enumerations of exactly four values
def closed_enum(text, names):
    best = None
    for tb in parse_tables(text):
        matched = [r for r in tb[2] if any(c.strip().strip("`*").strip().lower() in names for c in r)]
        if best is None or len(matched) > len(best[1]):
            best = (tb, matched, tb[2])
    return best


if "capability" in TYPES and CAP_STATES:
    best = closed_enum(TYPES["capability"]["text"], set(CAP_STATES))
    ok = best is not None and len(best[2]) == 4 and len(best[1]) == 4 and {
        c.strip().strip("`*").strip().lower() for r in best[1] for c in r} >= set(CAP_STATES)
    check("AC2", ok, f"the capability-state enumeration is a table of exactly the four C-129 states "
          f"(table rows {len(best[2]) if best else 0}, matched {len(best[1]) if best else 0})")
if "timestamp" in TYPES and TS_SOURCES:
    best = closed_enum(TYPES["timestamp"]["text"], set(TS_SOURCES))
    ok = best is not None and len(best[2]) == 4 and len(best[1]) == 4 and {
        c.strip().strip("`*").strip().lower() for r in best[1] for c in r} >= set(TS_SOURCES)
    check("AC2", ok, f"the receive-timestamp enumeration is a table of exactly the four C-131 sources "
          f"(table rows {len(best[2]) if best else 0}, matched {len(best[1]) if best else 0})")

# AC 3: the health section has the eleven C-159 counters and the observed DSCP
if "health" in TYPES and len(C159) == 11:
    hf = [f for n in TYPES["health"]["entries"] for f, _ in n["rows"]]
    ht = [f["meaning"] for f in hf]
    n, assign = max_match([counter_re(c) for c in C159], ht)
    check("AC3", n == 11, f"each of the eleven C-159 counters has its own row of the health section (matched {n})")
    for i, c in enumerate(C159):
        if i not in assign:
            continue
        f = hf[assign[i]]
        check("AC3", bool(re.search(r"wrap|saturat", f["meaning"] + " " + f["valid"], re.I)),
              f"the counter {c} states its wrap or saturation rule (Work 1)")
        check("AC3", bool(re.search(r"packet|slice|event|round|report|connection|stream", f["meaning"] + " " + f["valid"], re.I)),
              f"the counter {c} states its unit (Work 1)")
    check("AC3", any(re.search(r"DSCP", f["meaning"]) and re.search(r"observ", f["meaning"], re.I) for f in hf),
          "the health section carries the observed DSCP of the connection (C-47, Context)")

# AC 4: the injection section includes the panic hook (checked above) and the capability section control isolation
# AC 5: invalid forms with outcomes
VALID_TABLES = []
for tb in parse_tables(STAMP):
    if tb[1] and tb[1][0].lower().startswith("invalid form") and len(tb[1]) > 1 and "required outcome" in tb[1][1].lower():
        VALID_TABLES.append(tb)
merged_rows = set()
NEW_ROWS = []
for tb in VALID_TABLES:
    if any("too short" in r[0].lower() for r in tb[2] if r):
        merged_rows = {id(r) for r in tb[2]}
        continue
    NEW_ROWS += [r for r in tb[2] if len(r) >= 2]
check("AC5", len(NEW_ROWS) >= 5, f"at least 5 invalid forms of the section types have a required outcome (found {len(NEW_ROWS)})")
NONREJ_RE = re.compile(r"not a rejection|never a rejection|\bignored\b|\bvalid\b|no reason", re.I)
NEUTRAL_RE = re.compile(r"repeat|unknown (section )?(type|code)|older|skipped", re.I)
for r in NEW_ROWS:
    form, outcome = r[0], r[1]
    names = [t for t in re.findall(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b", outcome) if t in REJ_VALUE]
    bad = [n for n in names if n not in CONTROL_MEMBERS]
    if bad:
        fail("AC5", f"an invalid form names a reason outside the control layer {bad}: {form[:50]}")
        continue
    if not names and not (NONREJ_RE.search(outcome) and NEUTRAL_RE.search(form)):
        fail("AC5", f"a malformed form names no Reject reason (only repeats, unknown codes and older versions may be silent): {form[:60]}")
        continue
    fl_ = form.lower()
    if re.search(r"reserved|non-?zero", fl_) and "ReservedNonZero" not in names:
        fail("AC5", f"a reserved-bit form must name ReservedNonZero: {form[:60]}")
    if re.search(r"length", fl_) and "BadLength" not in names:
        fail("AC5", f"a length form must name BadLength (Work 5): {form[:60]}")
    if re.search(r"count", fl_) and re.search(r"above|exceed|more than|maximum", fl_) and "BadLength" not in names:
        fail("AC5", f"a count-above-maximum form must name BadLength: {form[:60]}")
for cat, pat in [("unknown enumeration value", r"unknown|not one of|outside|not assigned|undefined"),
                 ("count above its maximum", r"count|above|exceed|more than|maximum"),
                 ("length inconsistent", r"length"),
                 ("reserved bits non-zero", r"reserved|non-?zero")]:
    check("AC5", any(re.search(pat, r[0], re.I) for r in NEW_ROWS), f"an invalid form covers: {cat} (Work 5)")

# DoD: reserved and accept-any fields of the new types are recorded in the Reserved fields table
_need_res = any(re.search(r"\breserved\b|\bunused\b|accept[- ]any", f["meaning"] + " " + f["valid"], re.I)
                for t in TYPES.values() for f in t.get("fl_all", []))
_res_stamp = [r for r in RES_ROWS if len(r) > 1 and "regime stamp" in r[1].lower()]
check("DoD", (not _need_res) or len(_res_stamp) >= 2,
      "every reserved or accept-any field of the section types is recorded in the Reserved fields table"
      + ("" if (not _need_res) or len(_res_stamp) >= 2 else " (none recorded)"))

# DoD: evidence tags on every subsection and every prose paragraph of 5 or more words; open items
for key, t in TYPES.items():
    check("DoD", has_evidence(t["text"]), f"the {key} subsection carries an evidence tag")
    untagged = [x[:50] for k, x in prose_blocks(t["text"]) if k == "p"
                and len(re.sub(r"[`*_|>#-]", " ", x).split()) >= 5 and not has_evidence(x)]
    check("DoD", not untagged, f"every prose paragraph of 5 or more words in the {key} subsection carries an evidence tag"
          + (f"; untagged: {untagged[:2]}" if untagged else ""))
_open = [x for k, x in prose_blocks(STAMP) if re.search(r"open item", x, re.I)]
check("DoD", all(re.search(r"owner", x, re.I) and re.search(r"falsifier", x, re.I) for x in _open),
      f"every Open item paragraph of the regime stamp section names an owner and a falsifier ({len(_open)} found)")

# AC 6: wire.toml declares each section type as a regime stamp section that agrees with its table row for row
try:
    TOML = parse_toml(TOML_TEXT)
    TOML_OK = True
except Exception as error:
    fail("AC6", f"wire.toml does not parse: {error}")
    TOML = {}
    TOML_OK = False
msgs = TOML.get("message", []) if TOML_OK else []
msgs = msgs if isinstance(msgs, list) else []
_legacy = [m for m in msgs if m.get("name") == "regime stamp section"]
if _legacy:
    _tf = [f for f in _legacy[0].get("field", []) if int(f.get("offset", 0)) < 464]
    check("AC6", len(_tf) == 14 and all(int(f["offset"]) == o * 8 and int(f["width"]) == b * 8
                                         for f, (o, b) in zip(_tf, HDR_STAMP)),
          "the merged regime stamp section message of wire.toml is unchanged (header fields)")
for key, pat in TYPE_SPECS:
    if key not in TYPES:
        continue
    t = TYPES[key]
    cand = [m for m in msgs if re.search(pat, str(m.get("name", "")), re.I) and m.get("section_of") == "regime stamp"]
    check("AC6", len(cand) == 1, f"wire.toml declares exactly one regime stamp message for the {key} section type (found {len(cand)})")
    if len(cand) != 1 or not t.get("fl_main"):
        continue
    m = cand[0]
    check("AC6", m.get("layer") == "control" and int(m.get("dscp", -1)) == DSCP_CONTROL and int(m.get("max_size", -1)) == t["sec_max"],
          f"the {key} message of wire.toml is control, DSCP {DSCP_CONTROL} (C-13), and its max_size is the table's {t['sec_max']} bytes")
    reasons = set(m.get("rejection_reasons", []))
    check("AC6", reasons <= CONTROL_MEMBERS, f"the {key} message names only control reasons (found {sorted(reasons)})")
    body_rel = t["body_rel"]
    tf = [f for f in m.get("field", []) if int(f.get("offset", 0)) >= 464]
    md = [f for f, _ in t["root"]["rows"]]
    agree = len(tf) == len(md)
    for f, g in zip(tf, md):
        exp_off = g["_start"] + (464 if body_rel else 0)
        if int(f["offset"]) != exp_off or int(f["width"]) != g["bits"]:
            agree = False
        if bool(g["maxc"]) != ("count" in f) or (g["maxc"] and int(f.get("max_count", -1)) != g["maxc"]):
            agree = False
    check("AC6", agree, f"the {key} fields of wire.toml agree with its table row for row (offset, width, count)")
    # T1: every entry table (any depth) against its wire.toml entry message, row for row, bit for bit
    ent_msgs = [x for x in msgs if re.search(pat, str(x.get("name", "")), re.I) and "entry" in str(x.get("name", "")).lower() and "section_of" not in x]
    ents = t.get("entries", [])
    check("AC6", len(ent_msgs) == len(ents), f"wire.toml declares one entry message per entry table of the {key} section (tables {len(ents)}, messages {len(ent_msgs)})")
    for n, em in zip(ents, ent_msgs):
        efs = em.get("field", [])
        ok = len(efs) == len(n["rows"]) and int(em.get("max_size", -1)) == cdiv(n["size"], 8)
        pos = 0
        for (f, child), g in zip(n["rows"], efs):
            if int(g.get("offset", -1)) != f["_start"] or int(g.get("width", -1)) != f["bits"]:
                ok = False
            if bool(f["maxc"]) != ("count" in g) or (f["maxc"] and int(g.get("max_count", -1)) != f["maxc"]):
                ok = False
            pos = f["_start"] + f["bits"] * (f["maxc"] or 1)
        ends = [int(g["offset"]) + int(g["width"]) * (int(g.get("max_count", 1)) if "count" in g else 1) for g in efs]
        if ends and ends[-1] != n["size"]:
            ok = False
        check("AC6", ok, f"the entry table of {em.get('name')} agrees with wire.toml row for row (offset, width, count) and tiles to its size ({n['size']} bits)")

# ---------------------------------------------------------------------------
# Review findings (PR 35, round 1). They are labelled DoD with the finding named.
# ---------------------------------------------------------------------------
def has_num(text, n):
    return re.search(rf"(?<!\d){n}(?!\d)", text) is not None


def sentences(text):
    pieces = [s for s in re.split(r"(?<=[.;])\s+|\n\s*\n|\n(?=[|*#-])", text) if s.strip()]
    out = []
    for s in pieces:
        if out and re.match(r"\s*[\[`]", s):
            out[-1] = out[-1] + " " + s
        else:
            out.append(s)
    return out


# C1: the overflow report is complete. The merged sender rule 7 reads: "a stamp that needs more cannot be sent".
RULE7 = "a stamp that needs more cannot be sent"
check("DoD", RULE7 in STAMP, "review C1: the merged sender rule 7 is quoted by the report: \"" + RULE7 + "\"")
HEALTH = TYPES.get("health")
C09_MAX = int(re.search(r"at most (\d+)", const_row("C-09")).group(1)) if re.search(r"at most (\d+)", const_row("C-09")) else None
PEERS = int(re.search(r"(\d+) peers", const_row("C-217")).group(1)) if re.search(r"(\d+) peers", const_row("C-217")) else None
if _S > (SECTION_MAX or 64):
    if HEALTH is not None and HEALTH.get("entries"):
        F_H = [f["maxc"] for f, _ in HEALTH["root"]["rows"] if f["maxc"]][0]
        PRODUCT = 2 * C09_MAX * PEERS if (C09_MAX and PEERS) else None
        M_H = HEALTH["M_stated"][0] if HEALTH.get("M_stated") else None
        check("DoD", PRODUCT is not None and M_H == PRODUCT,
              f"review C1: the health maximum equals the constants' product: {C09_MAX} streams (C-09) × {PEERS} peers (C-217) "
              f"× 2 directions = {PRODUCT} (stated {M_H})")
        htext = HEALTH["text"]
        check("DoD", PRODUCT is not None and has_num(norm(htext), PRODUCT) and "C-09" in htext and "C-217" in htext,
              "review C1: the health subsection states the product with its constants C-09 and C-217")
        S_other = _S - HEALTH["Nt"]
        T = ((SECTION_MAX or 64) - S_other) * F_H
        paras = [x for k, x in prose_blocks(STAMP) if k == "p" and re.search(r"Open item 1(?!\d)", x)]
        def score(p):
            return sum([has_num(p, _S), bool(re.search(r"exceed|falsif", p, re.I)), "design owner" in p.lower(),
                        RULE7 in p.lower() and "cannot be sent" in p.lower(), bool(re.search(r"no stamp", p, re.I)),
                        has_num(p, T) and bool(re.search(r"threshold|fit", p, re.I)), has_evidence(p)])
        best = max(paras, key=score) if paras else ""
        check("DoD", has_num(best, _S) and bool(paras),
              f"review C1: one tagged report paragraph names Open item 1 and states the total {_S} sections against the bound 64")
        check("DoD", bool(re.search(r"exceed|falsif", best, re.I)) and "design owner" in best.lower(),
              "review C1: the report says the bound is exceeded or falsified, and that it goes to the design owner")
        check("DoD", RULE7 in best.lower() and "cannot be sent" in best.lower(),
              "review C1: the report cites merged sender rule 7 (a stamp that needs more cannot be sent)")
        check("DoD", bool(re.search(r"no stamp", best, re.I)),
              "review C1: the report says that above the bound a module sends no stamp at all")
        check("DoD", has_num(best, T) and bool(re.search(r"threshold|fit", best, re.I)),
              f"review C1: the report gives the threshold in entries, (64 − {S_other}) × {F_H} = {T}, below which a stamp still fits")
        check("DoD", has_evidence(best), "review C1: the report paragraph carries an evidence tag")

# C2: a capability the text names as needed by a ticket has an id in the capability table.
CAP_VOCAB = ["drop accounting", "dscp marking", "fault injection", "holder-eligible", "holder eligible",
             "real-time scheduling", "memory lock", "control isolation", "idle states"]
TICKET_ID = re.compile(r"\b\d{3}[a-z]?-\d{2}\b")
CAP_T = TYPES.get("capability")
if CAP_T is not None:
    cap_rows = []
    for tb in parse_tables(CAP_T["text"]):
        for r in tb[2]:
            if r and re.fullmatch(r"\d+", r[0].strip()):
                cap_rows.append(" ".join(r).lower())
    needs = set()
    for s in sentences(STAMP):
        sl = s.lower()
        if TICKET_ID.search(s):
            for term in CAP_VOCAB:
                if term in sl:
                    needs.add(term)
    for term in sorted(needs):
        check("DoD", any(term in row for row in cap_rows),
              f"review C2: the capability \"{term}\" named as needed by a ticket has an id (a numbered row) in the capability table")

# C3: no circular hand-back of a Manifest open item whose owner is 004-25.
MAN = span_of(r"^manifest$", 2)
man_text = "\n".join(body(MAN))
owned = set()
for k, x in prose_blocks(man_text):
    if k == "p" and re.search(r"Owner:[^.]*\b004-25\b", x):
        mo = re.search(r"Open item (\d+)", x)
        if mo:
            owned.add(mo.group(1))
bad_handback = []
for s in sentences(STAMP):
    for mo in re.finditer(r"Manifest,? Open item (\d+)", s):
        if mo.group(1) in owned and re.search(r"owner|hands|left to|belongs to", s, re.I):
            bad_handback.append(f"Manifest, Open item {mo.group(1)}")
check("DoD", not bad_handback,
      f"review C3: the regime stamp section does not hand a Manifest open item owned by 004-25 back to it undecided"
      + (f": {bad_handback}" if bad_handback else f" (owned by 004-25: {sorted(owned)})"))

# C4: a health entry distinguishes one admission of a connection from the next, by a field or a tagged sentence.
if HEALTH is not None:
    ent_meanings = [f["meaning"] for n in HEALTH["entries"] for f, _ in n["rows"]]
    has_field = any(re.search(r"request id|admission|generation|epoch of the connection", x, re.I) for x in ent_meanings)
    has_sentence = any(re.search(r"reset", s, re.I) and re.search(r"wrap", s, re.I) and has_evidence(s)
                       for s in sentences(HEALTH["text"]))
    check("DoD", has_field or has_sentence,
          "review C4: a health entry has a field that tells one admission from the next, or a tagged sentence tells a counter reset on re-admission from a wrap")


# AC 6: the 004-02 checker, the repository level
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"], capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        check("AC6", proc.returncode == 0 and "checks clean" in out,
              "cargo run --bin wire-check reports checks clean"
              + ("" if proc.returncode == 0 and "checks clean" in out else ": " + " | ".join(out.strip().splitlines()[-4:])))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC6", f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"], capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result[-1]) if result else None
        check("AC6", proc.returncode == 0 and bool(mm) and int(mm.group(2)) == 0 and int(mm.group(1)) >= 12,
              "cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least the 12 passed that stood before"
              + (f": {result[-1]}" if result else f": no result line ({out.strip()[-200:]})"))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC6", f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC6: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

fails = [ac for ac, ok in RESULTS if not ok]
print(f"{len(RESULTS) - len(fails)} passed, {len(fails)} failed")
sys.exit(1 if fails else 0)
PY
pyrc=$?
if [ "$pyrc" -ne 0 ]; then
    rc=1
fi
exit "$rc"
