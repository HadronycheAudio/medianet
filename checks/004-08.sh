#!/usr/bin/env bash
# 004-08 — slice header and send-timestamp format.
#
# RED-phase document check. It validates doc/design/spec/wire.md and
# doc/design/spec/wire.toml against every acceptance criterion of the ticket,
# and re-runs the 004-02 checker, the wire_tables tests and the design
# checker.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo, the design checker) are
# skipped with a NOTE, because they always target the real repository.
#
# What is checked, per acceptance criterion:
#
#   AC 1  The send timestamp has a stated WIDTH, UNIT, CLOCK SOURCE and WRAP
#         PERIOD, each [decision]-tagged. The falsifier arithmetic is
#         encoded: the unit's tick must resolve one sample period at
#         96 000 Hz (10 416 2/3 ns), and the stated wrap period must be
#         longer than one reassembly span (2 slice periods, C-15 — 1 ms at
#         the C-02 defaults), with the justification naming the reassembly
#         span, the alignment span (C-110) and the 96 000 Hz / 10 416 ns
#         arithmetic.
#   AC 2  The slice header table contains position as 64 bits and all five
#         rule-2 fields (position, frames, packet index, packet count, send
#         timestamp) with offsets and valid ranges: frames >= 1 with an
#         upper bound; packet index bounded by packet count; packet count = 1
#         in the default regime. The offsets continue from the packet header
#         (byte 16, 004-06) and tile the slice header contiguously.
#   AC 3  The multi-packet regime is defined on the media side (packet count
#         > 1) and the regime-stamp values it needs are named and explicitly
#         handed to 004-25 (at least stream number, packets per slice,
#         frames per slice — or the author's documented set).
#   AC 4  At least 5 invalid forms are listed with rejection reasons that are
#         members of the closed 13-name enumeration, including the ticket's
#         named forms: packet index >= count (BadPacketIndex), packet count
#         = 0, frames = 0 or above maximum, frames inconsistent with payload
#         length (BadLength), packet count inconsistent across one slice.
#   WS 5  The receive-timestamp note exists: C-131's four source values
#         (kernel, userspace, mac-hardware, interrupt) with C-129 capability
#         states, pointing at 004-25's regime stamp, not the slice header.
#   AC 5  wire.toml's media packet [[message]] (or a slice-header field
#         group) carries the slice-header fields, agreeing with wire.md
#         offset-by-offset; the declared max_size covers the packet header
#         plus the slice header; the media packet can produce
#         BadPacketIndex; `cargo run --bin wire-check` passes and
#         `cargo test -p hadronyche-tools --test wire_tables` stays 12/0/0.
#
# DoD also: every block of the new section carries an evidence tag ([code],
# [measured], [kernel-doc], [standard], [memory], [decision]); the design
# checker prints OK; and the regime-stamp-fields-touched DoD line is covered
# (the fields handed over are stated complete and documented) or marked not
# applicable with the reason (C-125).
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
python3 - "$WIRE" "$TABLES" <<'PY'
import os
import re
import subprocess
import sys

wire_path, tables_path = sys.argv[1], sys.argv[2]
overridden = bool(os.environ.get("WIRE_MD") or os.environ.get("WIRE_TOML"))

fails = []
warns = []


def fail(ac, msg):
    fails.append((ac, msg))


def warn(ac, msg):
    warns.append((ac, msg))


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


# ---------------------------------------------------------------------------
# Markdown helpers (same conventions as checks/004-01.sh, 004-03.sh, 004-06.sh)
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
    """Return [(start_line, header_cells, data_rows)] for Markdown tables."""
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
            if (
                len(block) >= 2
                and "-" in block[1]
                and re.match(r"^\s*\|?[\s:|-]+\|?\s*$", block[1])
            ):
                tables.append((start, split_row(block[0]), [split_row(r) for r in block[2:]]))
            i = j
        else:
            i += 1
    return tables


def heading_index(lines):
    """[(line_no, level, title)] for Markdown headings outside fences."""
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
    """Blank out fenced code, keeping line numbers (used for prose)."""
    lines = text.splitlines()
    in_fence = False
    out = []
    for line in lines:
        if re.match(r"^\s*```", line):
            in_fence = not in_fence
            out.append("")
            continue
        out.append("" if in_fence else line)
    return "\n".join(out)


def blocks(text):
    """Paragraph-like blocks: runs of non-empty lines."""
    out = []
    cur = []
    for line in text.splitlines():
        if line.strip():
            cur.append(line)
        elif cur:
            out.append("\n".join(cur))
            cur = []
    if cur:
        out.append("\n".join(cur))
    return out


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(block):
    return bool(EVIDENCE.search(block))


def norm_name(s):
    return re.sub(r"[^a-z0-9]", "", s.lower())


def parse_int_token(text):
    """The first integer in the text, with thin/regular spaces removed."""
    m = re.search(r"(\d[\d _]*)", text)
    if not m:
        return None
    return int(m.group(1).replace(" ", "").replace("_", ""))


def width_to_bits(cell):
    """A Width cell: '2' or '2 bytes' -> bytes*8; '2 bits' -> bits."""
    s = cell.lower()
    m = re.search(r"(\d[\d _]*)\s*bits?", s)
    if m:
        return int(m.group(1).replace(" ", "").replace("_", ""))
    m = re.search(r"(\d[\d _]*)", s)
    if m:
        return int(m.group(1).replace(" ", "").replace("_", "")) * 8
    return None


def bit_positions(row_text):
    """Stated bit positions of a sub-byte field: bits hi..lo (or a single bit)."""
    m = re.search(r"bits?\s*(\d+)\s*\.\.\s*(\d+)", row_text, re.I)
    if m:
        hi, lo = int(m.group(1)), int(m.group(2))
        if 0 <= lo <= hi <= 7:
            return lo, hi
    m = re.search(r"\bbit\s*(\d+)\b", row_text, re.I)
    if m:
        b = int(m.group(1))
        if 0 <= b <= 7:
            return b, b
    return None


# ---------------------------------------------------------------------------
# A small TOML reader (the 004-03/004-06 subset parser, reused: the check
# must run offline, unprivileged, on the Python the other checks use).
# ---------------------------------------------------------------------------
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
# Duration parsing, for the send-timestamp falsifier arithmetic (AC 1)
# ---------------------------------------------------------------------------
UNIT_NS = {
    "ns": 1.0, "nanosecond": 1.0, "nanoseconds": 1.0,
    "µs": 1e3, "us": 1e3, "microsecond": 1e3, "microseconds": 1e3,
    "ms": 1e6, "millisecond": 1e6, "milliseconds": 1e6,
    "s": 1e9, "sec": 1e9, "second": 1e9, "seconds": 1e9,
    "min": 6e10, "minute": 6e10, "minutes": 6e10,
    "hour": 3.6e12, "hours": 3.6e12,
    "day": 8.64e13, "days": 8.64e13,
    "week": 6.048e14, "weeks": 6.048e14,
    "year": 3.1536e16, "years": 3.1536e16,
}
UNIT_ALT = (
    r"nanoseconds?|microseconds?|milliseconds?|seconds?|minutes?|hours?|days?"
    r"|weeks?|years?|ns|µs|us|ms|sec(?:onds?)?|min"
)
GUARD = r"(?<![a-zA-Zµ])"

SAMPLE_PERIOD_NS = 1e7 / 960.0  # one sample period at 96 000 Hz: 10 416 2/3 ns


def durations_in(text):
    """[(value_ns, matched_text)] for '2^64 ns', '500 µs', '584 years', ..."""
    hits = []
    for m in re.finditer(
        r"2\^\s*(\d+)\s*" + GUARD + r"(" + UNIT_ALT + r")\b", text, re.I
    ):
        unit = m.group(2).lower()
        if unit in UNIT_NS:
            hits.append((m.start(), 2 ** int(m.group(1)) * UNIT_NS[unit], m.group(0)))
    for m in re.finditer(
        r"(\d[\d _]*(?:\.\d+)?)\s*" + GUARD + r"(" + UNIT_ALT + r")\b", text, re.I
    ):
        if text[max(0, m.start() - 1) : m.start() + 1].find("^") != -1:
            continue  # the number of a 2^x power, already counted above
        unit = m.group(2).lower()
        if unit in UNIT_NS:
            v = m.group(1).replace(" ", "").replace("_", "").replace(",", "")
            hits.append((m.start(), float(v) * UNIT_NS[unit], m.group(0)))
    hits.sort(key=lambda h: h[0])
    return [(v, t) for _, v, t in hits]


def stated_wrap_ns(region):
    """The stated wrap period: the first duration after a 'wrap period is…'
    (or 'wraps after…') statement."""
    for m in re.finditer(r"wrap(?:s)?\s+period[^.;\n]{0,140}", region, re.I):
        d = durations_in(m.group(0))
        if d:
            return d[0]
    for m in re.finditer(r"\bwraps?\s+(?:after|at|in)\b[^.;\n]{0,140}", region, re.I):
        d = durations_in(m.group(0))
        if d:
            return d[0]
    return None


def stated_slice_period_ns(prose):
    """The slice period stated in the section, for the reassembly-span
    arithmetic; falls back to the C-02 default (48 frames at 96 000 Hz)."""
    m = re.search(r"(\d[\d _]*)\s*frames?\b", prose)
    if m and re.search(r"96[\s_]?000", prose):
        frames = int(m.group(1).replace(" ", "").replace("_", ""))
        if frames > 0:
            return frames / 96000.0 * 1e9, f"{frames} frames at 96 000 Hz (C-02, C-01)"
    for m in re.finditer(
        r"slice\s+periods?\s+(?:of\s+)?(\d[\d _]*(?:\.\d+)?)\s*"
        + GUARD
        + r"("
        + UNIT_ALT
        + r")\b",
        prose,
        re.I,
    ):
        unit = m.group(2).lower()
        if unit in UNIT_NS:
            v = m.group(1).replace(" ", "").replace("_", "").replace(",", "")
            return float(v) * UNIT_NS[unit], m.group(0)
    return 48 / 96000.0 * 1e9, "the C-02 default 48 frames at 96 000 Hz"


def stated_unit_tick_ns(region):
    """The stated send-timestamp unit's tick in ns; None when unstated.
    The unit must resolve one sample period at 96 000 Hz (10 416 2/3 ns).

    Strict first: the text right after the word 'unit'. There the unit must
    be named outright — a bare 'sample period' mention is the falsifier's
    own wording, not a unit statement, so it does not count in the windows.
    Fallback over the whole region: the ns family first, then a sample unit
    qualified by its rate, then a bare sample-period statement."""
    for m in re.finditer(
        r"\bunits?\b\s*(?:is|are|=|:)?\s*(?:the\s+|in\s+|of\s+|one\s+)?[^.;\n]{0,60}",
        region,
        re.I,
    ):
        w = m.group(0)
        if re.search(
            r"samples?\s+at\s+(?:the\s+)?(?:domain\s+rate|96)"
            r"|sample\s+per\s+(?:domain\s+)?tick",
            w,
            re.I,
        ):
            return SAMPLE_PERIOD_NS, "samples at 96 000 Hz"
        if re.search(GUARD + r"nanoseconds?\b|" + GUARD + r"ns\b", w, re.I):
            return 1.0, "nanoseconds"
        if re.search(GUARD + r"microseconds?\b|µs|" + GUARD + r"us\b", w, re.I):
            return 1e3, "microseconds"
        if re.search(GUARD + r"milliseconds?\b|" + GUARD + r"ms\b", w, re.I):
            return 1e6, "milliseconds"
    if re.search(GUARD + r"nanoseconds?\b|" + GUARD + r"ns\b", region, re.I):
        return 1.0, "nanoseconds"
    if re.search(GUARD + r"microseconds?\b|µs|" + GUARD + r"us\b", region, re.I):
        return 1e3, "microseconds"
    if re.search(GUARD + r"milliseconds?\b|" + GUARD + r"ms\b", region, re.I):
        return 1e6, "milliseconds"
    if re.search(r"samples?\s+at\s+(?:the\s+)?(?:domain\s+rate|96)", region, re.I):
        return SAMPLE_PERIOD_NS, "samples at 96 000 Hz"
    if re.search(r"one\s+sample\b|sample\s+period", region, re.I):
        return SAMPLE_PERIOD_NS, "samples at 96 000 Hz"
    return None, None


# ---------------------------------------------------------------------------
# Load wire.md, find the Slice header section and its field table
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    for ac in range(1, 6):
        print(f"FAIL AC{ac}: {wire_path} does not exist")
    print("FAIL WS5: cannot check — wire.md does not exist")
    print("FAIL DoD: cannot check — wire.md does not exist")
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.search(r"slice\s+header", title, re.I):
        sec_i = i
        break

PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)
placeholder = sec_i is None
section = ""
section_prose = ""
if sec_i is None:
    for ac in range(1, 6):
        fail(ac, "no 'Slice header' section heading in wire.md")
    fail("WS5", "no 'Slice header' section heading in wire.md")
    fail("DoD", "no 'Slice header' section heading in wire.md")
else:
    start = headings[sec_i][0] + 1
    level = headings[sec_i][1]
    end = len(lines)
    for j in range(sec_i + 1, len(headings)):
        if headings[j][1] <= level:
            end = headings[j][0]
            break
    section = "\n".join(lines[start:end])
    section_prose = strip_fences(section)
    if not section.strip() or (PLACEHOLDER.search(section) and "|" not in section):
        placeholder = True
        for ac in range(1, 6):
            fail(ac, "the Slice header section is still the 004-08 placeholder")
        fail("WS5", "the Slice header section is still the 004-08 placeholder")
        fail("DoD", "the Slice header section is still the 004-08 placeholder")

blks = blocks(section_prose)


def find_block(pattern):
    """(index, block) of the first prose block matching the pattern."""
    for k, b in enumerate(blks):
        if re.search(pattern, b, re.I):
            return k, b
    return None, None


def decision_tagged(pattern):
    """The first block matching the pattern carries [decision], or the block
    before or after it does (the introducing paragraph's tag counts)."""
    k, b = find_block(pattern)
    if k is None:
        return False, None
    for c in (k - 1, k, k + 1):
        if 0 <= c < len(blks) and re.search(r"\[decision\]", blks[c], re.I):
            return True, b
    return False, b


def is_field_table(header):
    h = [re.sub(r"[^a-z0-9]", "", x.lower()) for x in header]
    return (
        len(h) >= 5
        and h[0].startswith("offset")
        and "width" in h[1]
        and "type" in h[2]
        and "meaning" in h[3]
        and "valid" in h[4]
    )


field_rows = []  # (cells, row_text, offset_bytes, width_bits, stated_bits)
field_table_found = False
for tstart, header, rows in parse_tables(section):
    if not is_field_table(header) or len(rows) < 4:
        continue
    candidate = []
    ok = True
    for row in rows:
        off = parse_int_token(row[0]) if row else None
        bits = width_to_bits(row[1]) if len(row) > 1 else None
        if off is None or bits is None:
            ok = False
            break
        candidate.append((row, " | ".join(row), off, bits, bit_positions(" | ".join(row))))
    if ok and len(candidate) > len(field_rows):
        field_rows = candidate
        field_table_found = True
        break

if sec_i is not None and not placeholder and not field_table_found:
    fail(2, "no field table (Offset/Width/Type/Meaning/Valid range) in the Slice header section")

# The closed enumeration, parsed from wire.md's Rejection reasons section
# (the landed 004-03 set of 13 names).
MEMBERS = [
    "TooShort", "BadMagic", "VersionMismatch", "ForeignDomain", "UnconfiguredDomain",
    "UnknownStream", "UnadmittedSource", "BadLength", "BadChecksum", "BadPacketIndex",
    "WrongEpoch", "OversizeControl", "ReservedNonZero",
]
rej_i = None
for i, (_, _, title) in enumerate(headings):
    if re.search(r"rejection[-\s]+reasons?", title, re.I):
        rej_i = i
        break
if rej_i is not None:
    rstart = headings[rej_i][0] + 1
    rlevel = headings[rej_i][1]
    rend = len(lines)
    for j in range(rej_i + 1, len(headings)):
        if headings[j][1] <= rlevel:
            rend = headings[j][0]
            break
    rej_section = "\n".join(lines[rstart:rend])
    found = set()
    for _, _, rows in parse_tables(rej_section):
        for row in rows:
            for cell in row:
                m = re.fullmatch(r"[`*_\s]*([A-Z][A-Za-z0-9]+)[`*_\s]*", cell.strip())
                if m and m.group(1) in MEMBERS:
                    found.add(m.group(1))
    if len(found) >= 10:
        MEMBERS = sorted(found)

member_set = set(MEMBERS)

# ---------------------------------------------------------------------------
# AC 1 — send timestamp: width, unit, clock source, wrap period, [decision]
#         tags, and the falsifier arithmetic encoded
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    # The region: the send-timestamp blocks and their neighbours.
    region_idx = set()
    for k, b in enumerate(blks):
        if re.search(r"send[\s_-]*timestamp", b, re.I):
            region_idx.update((k - 1, k, k + 1))
    region_idx = {k for k in region_idx if 0 <= k < len(blks)}
    region = "\n".join(blks[k] for k in sorted(region_idx))

    if not region.strip():
        fail(1, "no send-timestamp decision in the Slice header section (the open sub-question's answer)")
    else:
        ac1_ok = True

        # Width, stated in bits.
        m = re.search(r"(\d[\d _]*)\s*[\s_-]*bits?\b", region, re.I)
        if not m:
            fail(1, "the send timestamp's WIDTH is not stated in bits")
            ac1_ok = False
            width_bits = None
        else:
            width_bits = int(m.group(1).replace(" ", "").replace("_", ""))

        # Unit, and the resolution falsifier: one sample period at 96 kHz.
        tick_ns, unit_name = stated_unit_tick_ns(region)
        if tick_ns is None:
            fail(
                1,
                "the send timestamp's UNIT is not stated (state it in ns, µs or "
                "samples with its rate, so its resolution can be checked)",
            )
            ac1_ok = False
        elif tick_ns > SAMPLE_PERIOD_NS + 0.01:
            fail(
                1,
                f"the send timestamp's unit ({unit_name}, one tick = {tick_ns:g} ns) "
                "cannot resolve one sample period at 96 000 Hz (10 416 2/3 ns); the "
                "falsifier bites on the unit",
            )
            ac1_ok = False

        # Clock source.
        if not re.search(r"\bclock\b", region, re.I):
            fail(1, "the send timestamp's CLOCK SOURCE is not stated (the clock it is read from)")
            ac1_ok = False

        # Wrap period, and the span falsifier: no wrap inside one reassembly
        # or alignment span. The reassembly span is 2 slice periods (C-15).
        wrap = stated_wrap_ns(region)
        if wrap is None:
            fail(1, "the send timestamp's WRAP PERIOD is not stated")
            ac1_ok = False
            wrap_ns = None
        else:
            wrap_ns, wrap_text = wrap
            slice_ns, slice_src = stated_slice_period_ns(section_prose)
            span_ns = 2 * slice_ns
            if wrap_ns <= span_ns:
                fail(
                    1,
                    f"the stated wrap period ({wrap_text} = {wrap_ns:g} ns) wraps within "
                    f"one reassembly span (2 slice periods = {span_ns:g} ns, from "
                    f"{slice_src}; C-15) — the falsifier bites on the width",
                )
                ac1_ok = False
            if not re.search(r"reassembly", region, re.I):
                fail(
                    1,
                    "the wrap-period justification does not name the reassembly span "
                    "(C-15: 2 slice periods) it is checked against",
                )
                ac1_ok = False
            if not re.search(r"align", region, re.I):
                fail(
                    1,
                    "the wrap-period justification does not name the alignment span "
                    "(C-110: the consumer's own deadline) it is checked against",
                )
                ac1_ok = False

        # The resolution arithmetic must appear: 96 000 Hz or 10 416 2/3 ns.
        if not re.search(r"96[\s_]?000|10[\s_]?416", region):
            fail(
                1,
                "the send-timestamp decision does not show the resolution arithmetic "
                "(one sample period at 96 000 Hz = 10 416 2/3 ns)",
            )
            ac1_ok = False

        # All four items [decision]-tagged.
        for label, pattern in (
            ("width", r"\bbits?\b"),
            ("unit", r"\bunits?\b|" + GUARD + r"ns\b|samples?\s+at"),
            ("clock source", r"\bclock\b"),
            ("wrap period", r"\bwrap"),
        ):
            ok, b = decision_tagged(pattern)
            if ok is False:
                where = (b or "").strip().splitlines()[0][:80] if b else "?"
                fail(
                    1,
                    f"the send timestamp's {label} statement carries no [decision] tag: "
                    f"{where}…",
                )
                ac1_ok = False

        if ac1_ok:
            unit_txt = f"{unit_name} (tick {tick_ns:g} ns)" if tick_ns is not None else "?"
            wrap_txt = f"{wrap_ns:g} ns" if wrap_ns is not None else "?"
            print(
                f"PASS AC1: send timestamp width {width_bits} bits, unit {unit_txt}, "
                f"clock source stated, wrap period {wrap_txt} — resolves one sample "
                "period (10 416 2/3 ns at 96 000 Hz) and outlasts one reassembly span "
                "(2 slice periods, C-15) and the alignment span (C-110)"
            )

# ---------------------------------------------------------------------------
# AC 2 — the field table: position 64 bits, all rule-2 fields, offsets from
#         byte 16, contiguous tiling, valid ranges
# ---------------------------------------------------------------------------
slice_bytes = None
if sec_i is not None and not placeholder and field_table_found:
    ac2_ok = True
    REQUIRED = [
        ("position", r"\bposition\b", 64),
        ("frames", r"\bframes?\b", None),
        ("packet index", r"packet\s+index", None),
        ("packet count", r"packet\s+count", None),
        ("send timestamp", r"send[\s_-]*timestamp", None),
    ]
    claimed = [False] * len(field_rows)
    row_by_label = {}
    for label, pattern, need_bits in REQUIRED:
        hit = None
        for row_i, (cells, text, off, bits, stated) in enumerate(field_rows):
            if claimed[row_i]:
                continue
            if re.search(pattern, text, re.I):
                hit = row_i
                break
        if hit is None:
            fail(2, f"the slice header table has no {label} field (rule 2)")
            ac2_ok = False
            continue
        claimed[hit] = True
        cells, text, off, bits, stated = field_rows[hit]
        row_by_label[label] = cells
        if need_bits is not None and bits != need_bits:
            fail(
                2,
                f"{label}: the table gives {bits} bits; rule 2 fixes position at "
                "64 bits",
            )
            ac2_ok = False

    # The offsets continue from the packet header: byte 16 (004-06).
    first_off = min(off for _, _, off, _, _ in field_rows)
    if first_off != 16:
        fail(
            2,
            f"the slice header table starts at byte {first_off}; the packet header "
            "occupies bytes 0 to 15 (004-06), so the slice-header offsets continue "
            "at byte 16",
        )
        ac2_ok = False

    # Contiguous tiling (bit-level where sub-byte fields state their bits).
    intervals = []
    unstated_subbyte = False
    for cells, text, off, bits, stated in field_rows:
        if bits < 8:
            if stated:
                lo, hi = stated
                if bits != hi - lo + 1:
                    fail(
                        2,
                        f"sub-byte field states bits {hi}..{lo} ({hi - lo + 1} bits) "
                        f"but its width is {bits} bits: {text[:90]}",
                    )
                start = off * 8 + lo
            else:
                start = off * 8
                unstated_subbyte = True
        else:
            if bits % 8 != 0:
                fail(2, f"a field of {bits} bits is neither whole bytes nor stated bits: {text[:90]}")
            start = off * 8
        intervals.append((start, start + bits, text, bits < 8 and not stated))

    intervals.sort()
    cursor = 16 * 8
    for start, end, text, approx in intervals:
        if start > cursor:
            gap = start - cursor
            if gap < 8 and unstated_subbyte:
                warn(2, f"undeclared gap of {gap} bits at bit {cursor} (bit positions not stated; cannot verify bitwise here)")
            else:
                fail(
                    2,
                    f"undeclared gap of {gap} bits before {text.split('|')[3].strip()[:60]!r}; "
                    "the tiling must be contiguous",
                )
                ac2_ok = False
        elif start < cursor:
            if approx:
                warn(2, f"possible overlap at bit {start} (a sub-byte field does not state its bits)")
            else:
                fail(2, f"overlapping fields at bit {start}: {text[:90]}")
                ac2_ok = False
        cursor = max(cursor, end)
    if cursor % 8 != 0 and not unstated_subbyte:
        fail(2, f"the fields end at bit {cursor}, not a whole byte boundary")
        ac2_ok = False
    slice_bytes = (cursor - 16 * 8 + 7) // 8 if cursor >= 16 * 8 else 0

    # Valid ranges.
    frames_cells = row_by_label.get("frames")
    if frames_cells is not None and len(frames_cells) > 4:
        cell = frames_cells[4]
        lower = re.search(
            r"(?:^|[^\d])1\s*(?:to|–|—|\.\.\.?|through|-)\s*\d"
            r"|≥\s*1\b|>=\s*1\b|at\s+least\s+1\b",
            cell,
        )
        upper = any(
            int(x.replace(" ", "").replace("_", "")) > 1
            for x in re.findall(r"(\d[\d _]*)", cell)
        ) or bool(re.search(r"at\s+most\s+\d+", cell))
        if not lower:
            fail(
                2,
                f"the frames field's valid range does not state the lower bound 1 "
                f"(frames >= 1): {cell!r}",
            )
            ac2_ok = False
        if not upper:
            fail(
                2,
                f"the frames field's valid range states no upper bound (a fixed "
                f"maximum): {cell!r}",
            )
            ac2_ok = False
    index_cells = row_by_label.get("packet index")
    if index_cells is not None and len(index_cells) > 4:
        cell = index_cells[4]
        if not re.search(
            r"packet\s+count|count\s*[−–-]\s*1|index\s*<|<\s*(?:the\s+)?packet"
            r"|below\s+(?:the\s+)?packet\s+count",
            cell,
            re.I,
        ):
            fail(
                2,
                f"the packet index's valid range does not bound it by the packet "
                f"count (packet index < packet count): {cell!r}",
            )
            ac2_ok = False
    count_cells = row_by_label.get("packet count")
    if count_cells is not None and len(count_cells) > 4:
        cell = count_cells[4]
        if not re.search(r"\b1\b", cell) or not re.search(r"default", cell, re.I):
            fail(
                2,
                f"the packet count's valid range does not state the default regime's "
                f"value 1: {cell!r}",
            )
            ac2_ok = False

    if slice_bytes is not None and not re.search(
        r"\b" + str(16 + slice_bytes) + r"\b|\b" + str(slice_bytes) + r"\b\s*bytes",
        section_prose,
    ):
        warn(
            2,
            f"the section does not state the slice header's size in bytes "
            f"({16 + slice_bytes} bytes including the packet header, {slice_bytes} "
            "for the slice header itself; the 004-06 field-table convention states it)",
        )

    if ac2_ok:
        print(
            "PASS AC2: position 64 bits and all five rule-2 fields with offsets "
            "from byte 16 (after the 004-06 packet header), contiguous tiling, "
            "frames >= 1 with an upper bound, packet index < packet count, "
            "packet count = 1 in the default regime"
        )

# ---------------------------------------------------------------------------
# AC 3 — the multi-packet regime on the media side, and the hand-off to 004-25
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    ac3_ok = True
    if not re.search(
        r"packet\s+count\s*>\s*1|more\s+than\s+one\s+packet|multi-?packet",
        section_prose,
        re.I,
    ) or not re.search(r"regime", section_prose, re.I):
        fail(
            3,
            "the multi-packet regime is not defined on the media side (packet "
            "count > 1 in the slice header, C-05)",
        )
        ac3_ok = False

    hand_ok = False
    hand_seen = None
    for k, b in enumerate(blks):
        if not re.search(r"004-25", b):
            continue
        window = "\n".join(blks[max(0, k - 1) : k + 2])
        if not re.search(r"regime\s+stamp", window, re.I):
            hand_seen = window
            continue
        if not re.search(r"stream\s+number", window, re.I):
            hand_seen = window
            continue
        if not re.search(r"packets?\s+per\s+slice|packet\s+count", window, re.I):
            hand_seen = window
            continue
        if not re.search(r"frames?\s+per\s+slice|slice\s+length", window, re.I):
            hand_seen = window
            continue
        hand_ok = True
        break
    if not hand_ok:
        fail(
            3,
            "no hand-off to 004-25 naming the regime-stamp values the multi-packet "
            "regime needs (at least stream number, packets per slice, frames per "
            "slice — or the author's documented set)"
            + (
                "; the 004-25 mention found does not name them: "
                + hand_seen.strip().splitlines()[0][:80]
                if hand_seen
                else ""
            ),
        )
        ac3_ok = False

    if ac3_ok:
        print(
            "PASS AC3: packet count > 1 defines the multi-packet regime on the "
            "media side; the regime-stamp values (stream number, packets per "
            "slice, frames per slice) are named and handed to 004-25"
        )

# ---------------------------------------------------------------------------
# WS 5 — the receive-timestamp note (C-131's sources, C-129's capability
#         states, pointing at 004-25's regime stamp, not the slice header)
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    ws5_ok = True
    note_blocks = [b for b in blks if re.search(r"receive[\s_-]*timestamp", b, re.I)]
    note = "\n".join(note_blocks)
    if not note.strip():
        fail("WS5", "no receive-timestamp note in the Slice header section (Work step 5)")
        ws5_ok = False
    else:
        for source in ("kernel", "userspace", r"mac[\s_-]*hardware", "interrupt"):
            if not re.search(source, note, re.I):
                fail(
                    "WS5",
                    f"the receive-timestamp note does not name the C-131 source "
                    f"{source.strip(chr(92)).replace('[_-]*', '-')!r}",
                )
                ws5_ok = False
        if not re.search(r"C-129", note) or not re.search(r"capabilit", note, re.I):
            fail(
                "WS5",
                "the receive-timestamp note does not tie the sources to C-129's "
                "capability states",
            )
            ws5_ok = False
        if not re.search(r"004-25", note) or not re.search(r"regime\s+stamp", note, re.I):
            fail(
                "WS5",
                "the receive-timestamp note does not point at the regime stamp "
                "(004-25) as the place the source and its capability state travel",
            )
            ws5_ok = False
        if not re.search(
            r"not\s+(?:in|carried\s+in|part\s+of|the)\s+(?:the\s+)?slice\s+header"
            r"|not\s+in\s+the\s+slice\s+header"
            r"|never\s+in\s+the\s+slice\s+header"
            r"|(?:not|never)[^.;\n]{0,60}slice\s+header",
            note,
            re.I,
        ):
            fail(
                "WS5",
                "the receive-timestamp note does not state that the source is NOT "
                "in the slice header (it travels in the regime stamp, 004-25)",
            )
            ws5_ok = False
        states = sum(
            1 for w in ("present", "absent", "unverified", "not requested")
            if re.search(w, note, re.I)
        )
        if states < 4:
            warn(
                "WS5",
                "the note does not spell out all four C-129 capability states "
                "(present, absent, unverified, not requested)",
            )
    if ws5_ok:
        print(
            "PASS WS5: receive-timestamp note present — C-131's four sources with "
            "C-129 capability states, stamped in the regime stamp (004-25), not "
            "in the slice header"
        )

# ---------------------------------------------------------------------------
# AC 4 — at least 5 invalid forms with member rejection reasons
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    units = []
    for tstart, _, rows in parse_tables(section):
        if field_table_found:
            # exclude the field table: its Valid range column explains what a
            # receiver accepts, not the invalid forms with their outcomes
            hdr = None
            for t2, header, _ in parse_tables(section):
                if t2 == tstart:
                    hdr = header
            if hdr and is_field_table(hdr):
                continue
        for row in rows:
            units.append(" | ".join(row))
    for line in section_prose.splitlines():
        s = line.strip()
        if re.match(r"^[-*]\s+", s) or re.match(r"^\d+[.)]\s+", s):
            units.append(s)

    FORMS = [
        ("packet index >= packet count (BadPacketIndex)",
         r"packet\s+index[^.\n]{0,80}(?:≥|>=|outside|beyond|not\s+less\s+than|at\s+least|exceed)",
         "BadPacketIndex"),
        ("packet count = 0",
         r"packet\s+count[^.\n]{0,40}(?:=\s*0\b|\bis\s+0\b|of\s+0\b|zero\b)",
         None),
        ("frames = 0 or above maximum",
         r"frames?[^.\n]{0,80}(?:\b0\b|zero|above|maximum|over\s+the|exceed)",
         None),
        ("frames inconsistent with payload length (BadLength)",
         r"frames?[^.\n]{0,120}(?:payload|length|inconsist)",
         "BadLength"),
        ("packet count inconsistent across one slice",
         r"(?:packet\s+count|count)[^.\n]{0,100}(?:inconsist|disagree|differ|across\s+the\s+packets)",
         None),
        ("position not a multiple, when a rule applies (optional)",
         r"position[^.\n]{0,80}(?:multiple|align)",
         None),
    ]

    ac4_ok = True
    forms_with_reason = 0
    non_member_tokens = set()
    for label, pattern, need_reason in FORMS:
        optional = label.endswith("(optional)")
        hit = None
        for unit in units:
            if re.search(pattern, unit, re.I):
                hit = unit
                break
        if hit is None:
            if optional:
                warn(4, f"no invalid form listed for: {label.rsplit(' (', 1)[0]} (the ticket says 'if a rule applies')")
                continue
            fail(4, f"no invalid form listed for: {label.rsplit(' (', 1)[0]}")
            ac4_ok = False
            continue
        reasons_here = {name for name in MEMBERS if re.search(r"\b" + name + r"\b", hit)}
        if need_reason and need_reason not in reasons_here:
            fail(
                4,
                f"the invalid form {label.rsplit(' (', 1)[0]!r} must be rejected "
                f"under {need_reason}: {hit[:110]}",
            )
            ac4_ok = False
            continue
        if not reasons_here:
            fail(
                4,
                f"the invalid form {label.rsplit(' (', 1)[0]!r} carries no member of "
                f"the closed enumeration (expected one of {', '.join(MEMBERS)}): "
                f"{hit[:110]}",
            )
            ac4_ok = False
            continue
        forms_with_reason += 1

    # Every reason-looking token in the validity units must be a member.
    for unit in units:
        for m in re.finditer(r"\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b", unit):
            token = m.group(1)
            if token not in member_set and not re.search(
                r"\b(?:no|not|never|without)\b[^.\n]{0,30}" + re.escape(token), unit, re.I
            ):
                non_member_tokens.add(token)
    if non_member_tokens:
        fail(
            4,
            "the validity rules name reason(s) outside the closed enumeration: "
            + ", ".join(sorted(non_member_tokens)),
        )
        ac4_ok = False

    if forms_with_reason < 5:
        fail(
            4,
            f"only {forms_with_reason} invalid form(s) carry a member rejection "
            "reason; at least 5 are required",
        )
        ac4_ok = False

    if ac4_ok:
        print(
            f"PASS AC4: {forms_with_reason} invalid forms with member reasons, "
            "including every form the ticket names"
        )

# ---------------------------------------------------------------------------
# Load wire.toml
# ---------------------------------------------------------------------------
document = None
tables_text = read_text(tables_path)
if tables_text is None:
    fail(5, f"{tables_path} does not exist")
else:
    try:
        document = parse_toml(tables_text)
    except TomlError as error:
        fail(5, f"{tables_path} could not be read by this check: {error}")
        document = None

# ---------------------------------------------------------------------------
# AC 5 — the machine-readable side: slice-header fields agreeing with
#         wire.md, max_size covering packet header + slice header,
#         BadPacketIndex produced; the 004-02 checker re-verifies the file
# ---------------------------------------------------------------------------
media_message = None
if isinstance(document, dict):
    for msg in document.get("message", []) or []:
        if isinstance(msg, dict) and "mediapacket" in norm_name(str(msg.get("name", ""))):
            media_message = msg
            break

slice_toml_fields = []  # (message, field)
if isinstance(document, dict):
    for msg in document.get("message", []) or []:
        if not isinstance(msg, dict):
            continue
        nm = norm_name(str(msg.get("name", "")))
        if "mediapacket" in nm or "sliceheader" in nm:
            for f in msg.get("field", []) or []:
                if (
                    isinstance(f, dict)
                    and isinstance(f.get("offset"), int)
                    and f["offset"] >= 16 * 8
                ):
                    slice_toml_fields.append((msg, f))

if sec_i is not None and not placeholder:
    ac5_ok = True
    if media_message is None and not slice_toml_fields:
        fail(
            5,
            f"{tables_path} has no [[message]] carrying the slice header (neither "
            "the media packet message nor a slice-header field group)",
        )
        ac5_ok = False

    if field_table_found:
        if not slice_toml_fields:
            fail(
                5,
                f"{tables_path} declares no slice-header fields (no [[message.field]] "
                "at offset >= 128 bits in the media packet or a slice-header group)",
            )
            ac5_ok = False
        else:
            by_name = {}
            for msg, f in slice_toml_fields:
                if isinstance(f.get("name"), str):
                    by_name[norm_name(f["name"])] = (msg, f)
            matched_names = set()
            carrying = {}
            for cells, text, off, bits, stated in field_rows:
                md_label = cells[3][:60] if len(cells) > 3 else text[:60]
                row_norm = norm_name(text)
                match = None
                for fname, entry in by_name.items():
                    if len(fname) >= 3 and fname in row_norm and fname not in matched_names:
                        match = entry
                        break
                if match is None:
                    for msg, f in slice_toml_fields:
                        fname = norm_name(str(f.get("name", "")))
                        if fname in matched_names:
                            continue
                        toff = f.get("offset")
                        tw = f.get("width")
                        if not isinstance(toff, int) or not isinstance(tw, int):
                            continue
                        if bits < 8:
                            if off * 8 <= toff < off * 8 + 8 and tw == bits:
                                match = (msg, f)
                                break
                        elif toff == off * 8 and tw == bits:
                            match = (msg, f)
                            break
                if match is None:
                    fail(
                        5,
                        f"no [[message.field]] row agrees with the wire.md slice-header "
                        f"field {md_label!r} (offset {off}, width {bits} bits)",
                    )
                    ac5_ok = False
                    continue
                msg, f = match
                matched_names.add(norm_name(str(f.get("name", ""))))
                carrying[id(msg)] = msg
                toff = f.get("offset")
                tw = f.get("width")
                if not isinstance(toff, int) or not isinstance(tw, int):
                    fail(5, f"field {f.get('name')!r}: offset/width are not integers")
                    ac5_ok = False
                    continue
                if bits < 8:
                    if not (off * 8 <= toff < off * 8 + 8):
                        fail(
                            5,
                            f"field {f.get('name')!r}: offset {toff} is not inside "
                            f"byte {off} where wire.md places it",
                        )
                        ac5_ok = False
                elif toff != off * 8:
                    fail(
                        5,
                        f"field {f.get('name')!r}: toml offset {toff} bits vs wire.md "
                        f"offset {off} bytes",
                    )
                    ac5_ok = False
                if tw != bits:
                    fail(
                        5,
                        f"field {f.get('name')!r}: toml width {tw} bits vs wire.md "
                        f"width {bits} bits",
                    )
                    ac5_ok = False

            # The declared max_size must cover packet header + slice header.
            if slice_bytes is not None:
                need = 16 + slice_bytes
                for msg in carrying.values():
                    max_size = msg.get("max_size")
                    name = msg.get("name")
                    if not isinstance(max_size, int) or isinstance(max_size, bool):
                        fail(5, f"message {name!r}: max_size is not stated as an integer")
                        ac5_ok = False
                    elif max_size < need:
                        fail(
                            5,
                            f"message {name!r}: max_size {max_size} does not cover the "
                            f"packet header plus the slice header ({need} bytes)",
                        )
                        ac5_ok = False

    if media_message is not None:
        reasons = media_message.get("rejection_reasons")
        if not isinstance(reasons, list) or "BadPacketIndex" not in reasons:
            fail(
                5,
                "the media packet message's rejection_reasons do not include "
                "BadPacketIndex (the slice header adds it, per the wire.toml note)",
            )
            ac5_ok = False

    if ac5_ok:
        print(
            "PASS AC5: wire.toml's slice-header fields agree with wire.md "
            "offset-by-offset, max_size covers packet header + slice header, and "
            "the media packet can produce BadPacketIndex"
        )

# ---------------------------------------------------------------------------
# DoD — evidence tags on the new section; the regime-stamp-fields line;
#       the 004-02 checker and friends
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    prev_tagged = True
    for block in blocks(section):
        stripped = block.strip()
        if not stripped:
            continue
        content_lines = [
            line for line in stripped.splitlines() if not line.lstrip().startswith("#")
        ]
        if not content_lines:
            prev_tagged = True
            continue
        body = "\n".join(content_lines).strip()
        is_table = all(line.strip().startswith("|") for line in content_lines if line.strip())
        is_code = content_lines[0].lstrip().startswith("```")
        if has_evidence(body):
            prev_tagged = True
            continue
        if (is_table or is_code) and prev_tagged:
            prev_tagged = True
            continue
        fail(
            "DoD",
            "this block of the Slice header section carries no evidence tag "
            "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
            + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
        )
        prev_tagged = False

    # The ticket's DoD line "regime stamp fields touched are complete and
    # documented": covered by the hand-off, or marked N/A with the reason
    # (C-125).
    rs_ok = False
    for k, b in enumerate(blks):
        if not re.search(r"regime\s+stamp", b, re.I):
            continue
        window = b + ("\n" + blks[k + 1] if k + 1 < len(blks) else "")
        if re.search(r"complet|documented|hand(?:ed|over|ing)|suppli", window, re.I):
            rs_ok = True
            break
    if not rs_ok:
        rs_ok = any(
            re.search(r"not\s+applicable|\bN/?A\b", b, re.I) and re.search(r"C-125", b)
            for b in blks
        )
    if not rs_ok:
        fail(
            "DoD",
            "the regime-stamp-fields-touched line is neither covered (the handed "
            "fields stated complete and documented) nor marked not applicable "
            "with the reason (C-125)",
        )

if not overridden:
    # 1. The 004-02 checker, from the command line: the machine-readable side
    #    of the layout, the declared-vs-computed max_size and the bound.
    try:
        proc = subprocess.run(
            ["cargo", "run", "--quiet", "--bin", "wire-check"],
            capture_output=True,
            text=True,
            timeout=900,
        )
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-8:])
            fail(5, f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(5, f"could not run cargo run --bin wire-check: {error}")

    # 2. The wire_tables tests stay 12 passed, 0 failed, 0 ignored.
    try:
        proc = subprocess.run(
            ["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail(5, f"could not run the wire_tables tests: {error}")
    if proc is not None:
        out = proc.stdout + proc.stderr
        if proc.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail(5, f"cargo test -p hadronyche-tools --test wire_tables failed:\n{tail}")
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if result is None:
            fail(5, "the wire_tables test run printed no 'test result:' line")
        else:
            m = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result)
            if not m:
                fail(5, f"could not read the test counts from: {result}")
            elif (int(m.group(1)), int(m.group(2)), int(m.group(3))) != (12, 0, 0):
                fail(5, "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: " + result)

    # 3. The design checker prints OK.
    try:
        design = subprocess.run(
            ["python3", "doc/kickoff/check-design.py", "doc/design"],
            capture_output=True,
            text=True,
            timeout=300,
        )
        out = (design.stdout + design.stderr).strip().splitlines()
        last = out[-1].strip() if out else ""
        if design.returncode != 0 or last != "OK":
            tail = "\n".join(out[-8:])
            fail("DoD", f"the design checker does not print OK:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("DoD", f"could not run the design checker: {error}")
else:
    print("NOTE DoD: document paths overridden; repository-level checks skipped")

# ---------------------------------------------------------------------------
for ac, msg in warns:
    print(f"NOTE {ac if isinstance(ac, str) else 'AC' + str(ac)}: {msg}")

if fails:
    for ac, msg in fails:
        label = ac if isinstance(ac, str) else f"AC{ac}"
        print(f"FAIL {label}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 5/5 acceptance criteria for 004-08, receive-timestamp note present, DoD tags present")
PY
