#!/usr/bin/env bash
# 004-24 — regime stamp message: sections, version and fetch request.
#
# RED-phase document check. It validates the "Regime stamp" section of
# doc/design/spec/wire.md and the stamp messages in doc/design/spec/wire.toml
# against the acceptance criteria of the ticket, and re-runs the wire-check
# binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing
# on throwaway fixtures only: when either path is overridden the cargo checks are
# skipped with a NOTE, because they always target the real repository.
#
# Numbers are read at run time: C-17 (the control bound), C-13 (CS5), C-18 (the
# refresh period, "every N s"), C-217 (the peer count), the Hello's common header
# rows and its type-code block for 004-24, and the Rejection reasons with their
# Layers cells. The Markdown helpers and the TOML reader are copied from the
# 004-27 checker (Python 3.10 here, no tomllib).
#
# The author contract (headings and labels the check locates):
#
#   ## Regime stamp            the placeholder "To be filled by" line is replaced
#   ### <title>                one table per message: the section table (the seven
#       common control header rows in the hello's order and widths, a stamp version
#       row of 32 bits at offset 32 or later, rows for section index, section count,
#       section type, section length and a typed body, a Message type row whose
#       valid range is a code in the block the hello gives 004-24, the rows tile,
#       a sentence "maximum size ... N bytes"); the request table (the seven header
#       rows, a module id row of 8 bytes, a Message type row in the same block, the
#       rows tile, a "maximum size" sentence)
#   Validity rules table      header "Invalid form | Required outcome"; one row per
#       invalid form; each outcome names members of the Rejection reasons enumeration
#       and every named member lists the control layer
#   Open item paragraph       names an owner and a falsifier
#   wire.toml                 one [[message]] with section_of naming a declared message
#       (and "stamp" in that name) per section, layer control, dscp 40, max_size at
#       most the control bound, fields agreeing with the markdown table row for row;
#       one [[message]] for the request (named with "stamp" and "request" or "fetch",
#       no section_of), with the same checks
#
# What is checked, per acceptance criterion:
#   AC 1  the section and the request tile their bytes, each at most the control bound
#         (C-17, C-18); the common control header is the landed hello header; the
#         stamp version is 32 bits and compared as the hello's version is; the section
#         carries index, count, type, length and body; both message types lie in the
#         block the hello gives 004-24; the section placeholder is gone
#   AC 2  the maximum section count, the maximum stamp size and the per-peer storage
#         are stated as numbers: the count matches its valid range, the stamp size lies
#         between the largest section and count times the control bound, and the
#         storage is at least the stamp size
#   AC 3  the sender and receiver rules: multicast on any regime change, the refresh
#         period of C-18, the version bump on a change, the unicast fetch of a request,
#         skipping unknown section types by their length (004-04), the complete-stamp
#         test, discarding an incomplete stamp when a newer version arrives; the request
#         carries the module id, and the choice of return path is tagged [decision]
#   AC 4  at least seven invalid forms with outcomes; each outcome names only members of
#         the closed Rejection reasons enumeration whose Layers include control; the
#         oversize section names OversizeControl, the section length overrun and a count
#         above its maximum name BadLength (the enumeration's own meaning of BadLength)
#   AC 5  wire.toml parses; section and request messages declared as above; their fields
#         agree with the markdown tables; the rejection reasons cover the outcomes; when
#         the paths are not overridden: `cargo run --bin wire-check` reports "checks
#         clean" and `cargo test -p hadronyche-tools --test wire_tables` has 0 failed and
#         at least 12 passed (the count that stood before this ticket)
#   DoD   every prose paragraph of 5+ words and every subsection of the section carries
#         an evidence tag; every "Open item" paragraph names an owner and a falsifier;
#         the checker reads the working tree only and runs no other check
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
# Inputs and the numbers the check reads from the constants table
# ---------------------------------------------------------------------------
WIRE_RAW = read_text(wire_path)
if WIRE_RAW is None:
    fail("AC1", f"{wire_path} does not exist")
    WIRE_RAW = ""
WIRE = norm(WIRE_RAW)
lines = WIRE.splitlines()
HEADS = heading_index(lines)
TOML_TEXT = read_text(tables_path)
if TOML_TEXT is None:
    fail("AC5", f"{tables_path} does not exist")
    TOML_TEXT = ""
CONST = norm(read_text(CONSTANTS_PATH) or "")


def const_val(cid):
    for line in CONST.splitlines():
        if line.startswith(f"| {cid} |"):
            cells = line.split("|")
            return cells[3].strip() if len(cells) > 3 else ""
    return ""


_m = re.search(r"at most (\d+) bytes", const_val("C-17"))
CTRL_BOUND = int(_m.group(1)) if _m else None
_m = re.search(r"control CS5 \((\d+)\)", const_val("C-13"))
DSCP_CONTROL = int(_m.group(1)) if _m else None
_m = re.search(r"every (\d+) s\b", const_val("C-18"))
REFRESH_S = int(_m.group(1)) if _m else None
check("AC1", bool(CTRL_BOUND) and bool(DSCP_CONTROL) and bool(REFRESH_S),
      f"the constants read: the control bound is {CTRL_BOUND} bytes (C-17), control DSCP {DSCP_CONTROL} (C-13), "
      f"the refresh period {REFRESH_S} s (C-18)")


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


def size_of_field(f):
    return f["bits"] * (f["maxc"] if f["maxc"] else 1)


def tile_problems(fl):
    cur, probs = 0, []
    for f in fl:
        if f["off"] != cur:
            probs.append(f"'{f['meaning'][:30]}' starts at byte {f['off']}, expected {cur}")
        if size_of_field(f) % 8:
            probs.append(f"'{f['meaning'][:30]}' is not a whole number of bytes")
        cur = f["off"] + size_of_field(f) // 8
    return cur, probs


# ---------------------------------------------------------------------------
# Landed sections the stamp reads from (never rewritten)
# ---------------------------------------------------------------------------
HELLO_SPAN = span_of(r"^hello$", 2)
HSUBS = subsections(body(HELLO_SPAN))
_hdr = find_sub(HSUBS, r"common\s+control\s+header")
HDR = fields_of(first_table(_hdr[0][1], is_field_table) if _hdr else None, "AC1",
                "the hello's common control header")
HDR_SIG = [(f["off"], f["bits"], f["type"]) for f in HDR[:7]]
check("AC1", len(HDR) == 7 and sum(size_of_field(f) for f in HDR) == 256 and all(
    f["off"] == sum(size_of_field(g) for g in HDR[:i]) // 8 for i, f in enumerate(HDR)),
      f"the hello's common control header is seven rows tiling 32 bytes (found {len(HDR)} rows)")

HELLO_TEXT = "\n".join(body(HELLO_SPAN))


def hello_width(pattern):
    """Bit width of the first fixed-width row of the hello's layout whose meaning matches the pattern."""
    for _s, _hd, _rows in parse_tables(HELLO_TEXT):
        for _r in _rows:
            if len(_r) >= 5 and re.fullmatch(r"\d+", _r[0].strip()) and re.search(pattern, _r[3], re.I):
                _bits, _mc = width_of(_r[1])
                if _bits is not None and not _mc:
                    return _bits
    return None


VER_BITS = hello_width(r"regime\s+stamp\s+version\s+number")
MODULE_BITS = hello_width(r"module\s+id\s+\(C-11\)")
check("AC1", VER_BITS is not None and MODULE_BITS is not None,
      f"the hello's layout gives the widths the stamp reuses: the stamp version number {VER_BITS} bits, "
      f"the module id {MODULE_BITS} bits (C-11)")

BLOCK = None
_cds = find_sub(HSUBS, r"control\s+message\s+type\s+codes")
if _cds:
    for _s, _hd, _rows in parse_tables(_cds[0][1]):
        for _r in _rows:
            if len(_r) >= 3 and "004-24" in _r[2]:
                _n = [int(x) for x in re.findall(r"\d+", _r[0])]
                if _n:
                    BLOCK = range(_n[0], _n[-1] + 1)
check("AC1", BLOCK is not None,
      "the hello's type-code table gives the regime stamp messages a block assigned to 004-24"
      + (f" (codes {BLOCK.start} to {BLOCK.stop - 1})" if BLOCK is not None else ""))

REJ_SPAN = span_of(r"rejection[-\s]+reasons?", 2)
MEMBERS = {}
MEMBER_LAYERS = {}
for _s, _hd, _rows in parse_tables("\n".join(body(REJ_SPAN))):
    for _r in _rows:
        if len(_r) >= 4 and re.fullmatch(r"\d+", _r[0].strip()) and re.fullmatch(r"[A-Z][A-Za-z0-9]+", _r[1].strip()):
            MEMBERS[_r[1].strip()] = int(_r[0])
            MEMBER_LAYERS[_r[1].strip()] = _r[3]
check("AC4", len(MEMBERS) == 13, f"the Rejection reasons enumeration of wire.md has 13 members (found {len(MEMBERS)})")


def outcome_tokens(cell):
    return re.findall(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b", cell)


# ---------------------------------------------------------------------------
# The regime stamp section
# ---------------------------------------------------------------------------
STAMP_SPAN = span_of(r"^regime\s+stamp$", 2)
STAMP_LINES = body(STAMP_SPAN)
STAMP = "\n".join(STAMP_LINES)
check("AC1", STAMP_SPAN is not None and "to be filled" not in STAMP.lower(),
      "wire.md has a '## Regime stamp' section with its own text, not the placeholder")
STAMP_TABLES = parse_tables(STAMP)
STAMP_PROSE = prose_blocks(STAMP)
PROSE = "\n".join(t for k, t in STAMP_PROSE if k == "p")
SENT = [s.strip().lower() for s in re.split(r"(?<=[.;:])\s+", PROSE) if s.strip()]
STAMP_SUBS = subsections(STAMP_LINES)


def any_sent(*pats):
    return any(all(re.search(p, s) for p in pats) for s in SENT)


def stated_max_size(n):
    return any(re.search(r"maximum|at most|largest|fixed size", s) and re.search(rf"(?<!\d){n}\s*bytes", s) for s in SENT)


def rows_matching(fl, pat, minoff=0):
    return [f for f in fl if f["off"] >= minoff and re.search(pat, f["meaning"], re.I)]


FIELD_TABS = [t for t in STAMP_TABLES if is_field_table(t[1])]
SEC_FL = None
REQ_FL = None
for _t in FIELD_TABS:
    _fl = fields_of(_t, "AC1", "a regime stamp field table")
    if SEC_FL is None and any(re.search(r"section\W+(index|number)", f["meaning"], re.I) for f in _fl):
        SEC_FL = _fl
    elif REQ_FL is None and any(re.search(r"module\s+id", f["meaning"], re.I) for f in _fl):
        REQ_FL = _fl

# --- the section table (AC 1, AC 2) ---
if SEC_FL is None:
    fail("AC1", "the regime stamp has no section table: no field table with a 'section index' row")
    SEC_FL = []
check("AC1", len(SEC_FL) >= 8 and [(f["off"], f["bits"], f["type"]) for f in SEC_FL[:7]] == HDR_SIG,
      "the section table opens with the seven common control header rows of the hello, unchanged (Hello, Common control header)")
_ver = [f for f in rows_matching(SEC_FL, r"version", 32) if not re.search(r"section\W+(count|index|type|length)|number of sections", f["meaning"], re.I)]
check("AC1", len(_ver) == 1 and VER_BITS is not None and _ver[0]["bits"] == VER_BITS and not _ver[0]["maxc"],
      f"the section table carries one stamp version number of the hello's width, {VER_BITS} bits (Hello, Regime stamp version number)")
check("AC1", any(re.search(r"version", s) and re.search(r"serial|half the range|modulo|2\^31|hello", s) for s in SENT),
      "the stamp states the version comparison of the hello (serial comparison within one instance nonce)")
for _label, _pat in [("section index", r"section\W+(index|number)"), ("section count", r"section\W+count"),
                     ("section type", r"section\W+type"), ("section length", r"section\W+length"),
                     ("typed body", r"\bbody\b")]:
    check("AC1", len(rows_matching(SEC_FL, _pat, 32)) >= 1, f"the section table carries a row for the {_label} (Work item 1)")
SEC_TILE, SEC_PROBS = tile_problems(SEC_FL)
check("AC1", not SEC_PROBS and 0 < SEC_TILE <= CTRL_BOUND,
      f"the section table tiles its bytes with no gap and totals {SEC_TILE} bytes, at most the control bound {CTRL_BOUND} (C-17, C-18)"
      + (f": {SEC_PROBS[:3]}" if SEC_PROBS else ""))
check("AC1", stated_max_size(SEC_TILE), f"a sentence states the maximum size of a section as {SEC_TILE} bytes (the table's sum)")

# --- the request table (AC 1, AC 3) ---
if REQ_FL is None:
    fail("AC3", "the regime stamp has no request table: no field table with a 'module id' row")
    REQ_FL = []
check("AC1", len(REQ_FL) >= 8 and [(f["off"], f["bits"], f["type"]) for f in REQ_FL[:7]] == HDR_SIG,
      "the stamp request opens with the seven common control header rows of the hello, unchanged")
_mod = [f for f in REQ_FL if re.search(r"module\s+id", f["meaning"], re.I)]
check("AC3", len(_mod) == 1 and MODULE_BITS is not None and _mod[0]["bits"] == MODULE_BITS and not _mod[0]["maxc"],
      f"the stamp request names the requested module by its {MODULE_BITS}-bit module id (C-11, the hello's width)")
REQ_TILE, REQ_PROBS = tile_problems(REQ_FL)
check("AC1", not REQ_PROBS and 0 < REQ_TILE <= CTRL_BOUND,
      f"the request table tiles its bytes and totals {REQ_TILE} bytes, at most the control bound (C-17)"
      + (f": {REQ_PROBS[:3]}" if REQ_PROBS else ""))
check("AC1", stated_max_size(REQ_TILE), f"a sentence states the maximum size of the request as {REQ_TILE} bytes (the table's sum)")


def type_row(fl):
    for f in fl:
        if re.search(r"message\s+type", f["meaning"], re.I):
            return f
    return None


for _label, _fl in [("section", SEC_FL), ("request", REQ_FL)]:
    _tr = type_row(_fl)
    _codes = [int(x) for x in re.findall(r"\d+", _tr["valid"])] if _tr else []
    check("AC1", BLOCK is not None and bool(_codes) and all(c in BLOCK for c in _codes),
          f"the {_label} message type is a code inside the block the hello gives 004-24 (Hello, Control message type codes)")

# --- AC 2: the numbers ---
_cnt = rows_matching(SEC_FL, r"section\W+count")
M = None
if len(_cnt) == 1:
    _nums = [int(x) for x in re.findall(r"\d+", _cnt[0]["valid"])]
    M = _nums[-1] if _nums else None
check("AC2", M is not None and M >= 1,
      f"the section count row states a maximum section count in its valid range (found {M})")
check("AC2", M is not None and any(re.search(r"section", s) and re.search(r"maximum|at most|fixed|up to", s)
                                    and re.search(rf"(?<!\d){M}(?!\d)", s) for s in SENT),
      f"a sentence states the maximum section count {M} as a number")
_s_cands = []
for s in SENT:
    if "stamp" in s and "storage" not in s and re.search(r"maximum|at most|largest|fixed", s):
        _s_cands += [int(x) for x in re.findall(r"(?<![\d.])(\d+)\s*bytes", s)]
_s_ok = [x for x in _s_cands if M and SEC_TILE <= x <= M * CTRL_BOUND]
S_MAX = max(_s_ok) if _s_ok else None
check("AC2", S_MAX is not None,
      f"a sentence states the maximum stamp size in bytes, between one largest section ({SEC_TILE}) and "
      f"the section count times the control bound ({M} x {CTRL_BOUND})")
_n_cands = []
for s in SENT:
    if "storage" in s:
        _n_cands += [int(x) for x in re.findall(r"(?<![\d.])(\d+)\s*bytes", s)]
check("AC2", S_MAX is not None and any(x >= S_MAX for x in _n_cands),
      f"a sentence states the per-peer storage a receiver needs in bytes, at least the maximum stamp size ({S_MAX})")

# --- AC 3: sender and receiver rules ---
check("AC3", any_sent(r"multicast", r"change"),
      "a sender rule says the stamp is multicast to the control group on any regime change (C-18)")
check("AC3", any_sent(rf"(?<![\d.]){REFRESH_S} ?(s|seconds)\b", r"refresh|every|period|interval"),
      f"a sender rule says the stamp is refreshed every {REFRESH_S} s (C-18)")
check("AC3", any_sent(r"unicast", r"request|fetch"),
      "a request is unicast and gives an immediate fetch of the stamp (C-18)")
check("AC3", any_sent(r"version", r"bump|increment|advance|change"),
      "the sender bumps the stamp version on any change of its stamp")
check("AC3", any_sent(r"unknown|not know|does not know|unrecogni", r"skip|ignore", r"length"),
      "a receiver skips an unknown section type of the same major by its length (004-04)")
check("AC3", any_sent(r"incomplete", r"discard|drop|replace|abandon", r"newer|later|higher|new version|greater"),
      "a receiver discards an incomplete stamp when a newer version arrives")
check("AC3", any_sent(r"complete", r"version"),
      "a receiver can tell that it holds a complete stamp of one version (Work item 1)")
check("AC3", any(re.search(r"send", t, re.I) for t, _x in STAMP_SUBS)
      and any(re.search(r"receiv", t, re.I) for t, _x in STAMP_SUBS),
      "the stamp section has a sender part and a receiver part (Work items 4 and 5)")
_ret = [t for k, t in STAMP_PROSE if k == "p" and re.search(r"unicast|multicast", t, re.I)
        and re.search(r"request|fetch", t, re.I) and re.search(r"return|reply|answer|sent back|respon", t, re.I)]
check("AC3", any("[decision]" in t for t in _ret),
      "a paragraph says how the stamp is returned to the requester (unicast or multicast) and is tagged [decision] (Work item 3)")

# --- AC 4: the invalid forms ---
VALID_T = None
for _t in STAMP_TABLES:
    _h = _t[1]
    if len(_h) >= 2 and re.search(r"invalid", _h[0], re.I) and re.search(r"outcome", _h[1], re.I):
        VALID_T = _t
        break
check("AC4", VALID_T is not None, "the stamp section has a Validity rules table of 'Invalid form | Required outcome'")
ROWS = [r for r in (VALID_T[2] if VALID_T else []) if len(r) >= 2]
check("AC4", len(ROWS) >= 7, f"the stamp section lists at least 7 invalid forms with outcomes (found {len(ROWS)})")


def category(form):
    f = form.lower()
    if re.search(r"oversize|over-size|1252|control bound", f):
        return "oversize"
    if re.search(r"index", f) and re.search(r"count", f):
        return "index_vs_count"
    if re.search(r"count", f) and re.search(r"above|exceed|greater|more than|beyond|maximum|\bmax\b", f):
        return "count_above_max"
    if re.search(r"count", f) and re.search(r"differ|disagree|mismatch|inconsisten|between|not the same", f):
        return "count_differs"
    if re.search(r"duplicate|twice|repeat|same section", f):
        return "duplicate"
    if re.search(r"length|overrun|overflow|body", f):
        return "length_overrun"
    if re.search(r"magic|version|domain|authentication|scheme|tag|datagram|too short|control header|counter|generic", f):
        return "generic"
    return None


_cats = {}
for row in ROWS:
    c = category(row[0])
    if c and c not in _cats:
        _cats[c] = row
for c, label in [("oversize", "an oversize section"), ("index_vs_count", "a section index not below the section count"),
                 ("count_above_max", "a section count above its maximum"),
                 ("count_differs", "a section count differing between sections of one version"),
                 ("duplicate", "a duplicate section"), ("length_overrun", "a section length overrun"),
                 ("generic", "a generic control violation")]:
    check("AC4", c in _cats, f"the invalid forms include {label}")
for row in ROWS:
    toks = outcome_tokens(row[1])
    mem = [t for t in toks if t in MEMBERS]
    unknown = [t for t in toks if t not in MEMBERS]
    ok = bool(mem) and not unknown and all("control" in MEMBER_LAYERS[t] for t in mem)
    check("AC4", ok, f"the outcome of '{row[0][:48]}' names only members of the Rejection enumeration whose "
                     f"Layers include control" + (f"; names {mem} and {unknown}" if not ok else ""))
for c, need, why in [("oversize", "OversizeControl", "the oversize section (C-17)"),
                     ("count_above_max", "BadLength", "a count above its fixed maximum (Rejection reasons)"),
                     ("length_overrun", "BadLength", "a section length overrun (the ticket's outcome)")]:
    if c in _cats:
        check("AC4", need in outcome_tokens(_cats[c][1]), f"{why} is outcome {need}")
_reasons_md = set(t for row in ROWS for t in outcome_tokens(row[1]) if t in MEMBERS)

# --- AC 5: wire.toml ---
try:
    TOML = parse_toml(TOML_TEXT)
except Exception as error:  # the reader raises TomlError or ValueError on a malformed file
    TOML = {}
    fail("AC5", f"wire.toml does not parse with the subset reader: {error}")
TMSGS = [m for m in TOML.get("message", []) if isinstance(m, dict)]
NAMES = {m.get("name") for m in TMSGS}
def toml_fields(m):
    return [f for f in m.get("field", []) if isinstance(f, dict)]


def toml_sig(fl):
    return [(int(f.get("offset", -1)), int(f.get("width", 0)), int(f.get("max_count", 1))) for f in fl]


def md_sig(fl):
    return [(f["off"] * 8, f["bits"], f["maxc"] or 1) for f in fl]


def _md_matches(m, fl):
    return bool(fl) and md_sig(fl) == toml_sig(toml_fields(m))


SEC_TOML = [m for m in TMSGS if m.get("section_of") and _md_matches(m, SEC_FL)]
REQ_TOML = [m for m in TMSGS if not m.get("section_of") and _md_matches(m, REQ_FL)]
check("AC5", len(SEC_TOML) >= 1,
      "a wire.toml message with section_of (the 004-02 attribute) carries the markdown section table row for row")
check("AC5", len(REQ_TOML) >= 1,
      "a wire.toml message without section_of carries the markdown request table row for row")
for m in SEC_TOML:
    check("AC5", m.get("section_of") in NAMES,
          f"the section_of of '{m.get('name')}' names a message declared in wire.toml ({m.get('section_of')})")


def check_toml_message(m, md_fl, md_total, ac_label):
    fl = toml_fields(m)
    ms = m.get("max_size")
    cur, probs = 0, []
    for f in fl:
        mult = int(f.get("max_count", 1))
        if int(f.get("offset", -1)) != cur:
            probs.append(f"'{f.get('name')}' at bit {f.get('offset')}, expected {cur}")
        cur = int(f.get("offset", 0)) + int(f.get("width", 0)) * mult
    check(ac_label, isinstance(ms, int) and ms <= CTRL_BOUND and not probs and cur == ms * 8,
          f"toml '{m.get('name')}' tiles from bit 0 to max_size, and max_size is at most the control bound "
          f"({cur} bits against {ms} bytes)" + (f": {probs[:2]}" if probs else ""))
    check(ac_label, m.get("layer") == "control" and m.get("dscp") == DSCP_CONTROL,
          f"toml '{m.get('name')}' is layer control at dscp {DSCP_CONTROL} (C-13)")
    check(ac_label, "exempt_from_control_bound" not in m,
          f"toml '{m.get('name')}' carries no exemption from the control bound (C-116 is the probe pair's alone)")
    rs = m.get("rejection_reasons", [])
    unknown = [r for r in rs if r not in MEMBERS]
    check("AC4", not unknown and all("control" in MEMBER_LAYERS.get(r, "") for r in rs),
          f"toml '{m.get('name')}' rejection_reasons are members whose Layers include control"
          + (f"; not members: {unknown}" if unknown else ""))
    rows_ok = bool(md_fl) and md_sig(md_fl) == toml_sig(fl) and md_total == ms
    check(ac_label, rows_ok,
          f"the markdown table and toml '{m.get('name')}' agree row for row in offset, width and count, and in size")
    return rows_ok, rs


SEC_AGREE = False
for m in SEC_TOML:
    ok, _rs = check_toml_message(m, SEC_FL, SEC_TILE, "AC5")
    SEC_AGREE = SEC_AGREE or ok
check("AC5", SEC_AGREE or not SEC_TOML, "a stamp section message of wire.toml is the markdown section table") if SEC_TOML else None
_toml_reasons = set()
for m in SEC_TOML + REQ_TOML:
    _toml_reasons |= set(m.get("rejection_reasons", []))
for m in REQ_TOML:
    check_toml_message(m, REQ_FL, REQ_TILE, "AC5")
check("AC4", bool(_reasons_md) and _reasons_md <= _toml_reasons,
      f"every reason the stamp's validity table names is in the rejection_reasons of a stamp message of wire.toml"
      + (f" (missing {sorted(_reasons_md - _toml_reasons)})" if _reasons_md - _toml_reasons else ""))

# AC 5 (repository level): the wire-check binary and the wire_tables tests
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        check("AC5", proc.returncode == 0 and "checks clean" in out,
              "cargo run --bin wire-check reports checks clean"
              + ("" if proc.returncode == 0 and "checks clean" in out else ": " + " | ".join(out.strip().splitlines()[-4:])))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC5", f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result[-1]) if result else None
        check("AC5", proc.returncode == 0 and bool(mm) and int(mm.group(2)) == 0 and int(mm.group(1)) >= 12,
              "cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least the 12 passed that stood before"
              + (f": {result[-1]}" if result else f": no result line ({out.strip()[-200:]})"))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC5", f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC5: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

# ---------------------------------------------------------------------------
# DoD: evidence tags, open items
# ---------------------------------------------------------------------------
_untagged = []
for _ln in strip_fences(STAMP).splitlines():
    _t = _ln.strip()
    if not _t or _t.startswith("#") or _t.startswith("|"):
        continue
    if len(re.sub(r"[`*_|>#-]", " ", _t).split()) >= 5 and not has_evidence(_t):
        _untagged.append(_t[:60])
check("DoD", not _untagged,
      "every prose paragraph of 5 or more words in the regime stamp section carries an evidence tag"
      + (f"; untagged: {_untagged[:3]}" if _untagged else ""))
_untagged_subs = [t for t, x in STAMP_SUBS if not has_evidence(x)]
check("DoD", not _untagged_subs,
      "every subsection of the regime stamp section carries an evidence tag"
      + (f"; untagged: {_untagged_subs}" if _untagged_subs else ""))
_open = [t for k, t in STAMP_PROSE if re.search(r"open item", t, re.I)]
check("DoD", all(re.search(r"owner", t, re.I) and re.search(r"falsifier", t, re.I) for t in _open),
      f"every Open item paragraph of the regime stamp section names an owner and a falsifier ({len(_open)} found)")

fails = [ac for ac, ok in RESULTS if not ok]
print(f"{len(RESULTS) - len(fails)} passed, {len(fails)} failed")
sys.exit(1 if fails else 0)
PY
pyrc=$?
if [ "$pyrc" -ne 0 ]; then
    rc=1
fi
exit "$rc"
