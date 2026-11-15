#!/usr/bin/env bash
# 004-19 — sync request and reply layouts.
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
# The section is located by its heading text (a heading containing
# "Sync request"), never by position. Numbers are read from
# doc/design/02-constants.md at run time, never copied into this file.
#
# What is checked, per acceptance criterion:
#
#   AC 1  Request and reply have identical byte size. The size is the SUM of
#         the field widths of each field table (markdown), the fields tile
#         from offset 0 with no gap and no overlap, and the number the
#         section states ("... N bytes" in a sentence about size) equals that
#         sum. The request reaches the size through an explicit padding or
#         reserved field of width > 0 whose required value is zero, in the
#         markdown table and in wire.toml. wire.toml's max_size of both
#         messages equals the computed size, and its fields sum to it.
#   AC 2  The reply table carries the echoed request send timestamp (64 bits,
#         "echo" in its meaning), the timeline epoch at exactly 16 bits (C-10,
#         read from the constants table) and the holder module id at exactly
#         64 bits (C-11). Both tables carry a domain id of 16 bits (C-11) whose
#         row states 0 = unconfigured.
#   AC 3  Positive: the section states the exchange is stateless and that the
#         reply echoes the request's own send timestamp (one prose paragraph
#         holds both words, "stateless" and "echo"). Negative: no table cell
#         names a field that only per-request holder state could fill (a
#         sequence, exchange id, transaction, correlation id, request
#         number, cookie, pending, handle or token). LIMIT: this is a keyword
#         detector over the table text; it cannot prove that a field's
#         meaning is free of per-request state, so the author's wording is
#         the real evidence, and the detector only catches the obvious names.
#   AC 4  The authentication-reservation decision is a prose paragraph tagged
#         [decision] that says whether sync messages carry the reservation,
#         and has a falsifier (the word "falsifier" followed by at least four
#         words, in that paragraph or an adjacent one). Either decision is
#         accepted. If the decision is "carries": both tables carry an 8-bit
#         scheme with value 0, a 64-bit message counter that is live, and a
#         128-bit tag that is zero (C-12 widths, C-115 values). If the decision
#         is "does not carry": neither table carries a scheme, counter, tag or
#         reservation field.
#   AC 5  At least 6 invalid forms in a table with an "outcome" column. The
#         forms are classified by their form cell: non-zero padding must be
#         ReservedNonZero; bad magic BadMagic; bad version VersionMismatch;
#         foreign domain ForeignDomain; wrong epoch WrongEpoch; size not equal
#         to the fixed size BadLength (each exactly that one reason, as the
#         ticket names it); a datagram shorter than the minimum that carries
#         the magic and version (too short, fewer or under N bytes or octets)
#         must be TooShort; a size that is not the fixed size must be
#         BadLength, and at least one such row must exist; reply from a
#         non-holder must name some member of
#         the closed enumeration. Every reason named in an outcome cell, and
#         every reason after "under" in the section, must be a member of the
#         closed enumeration parsed from wire.md's "Rejection reasons" table
#         (13 names); no reason is invented, and names after "never" or "not"
#         are negations, not outcomes.
#   AC 6  wire.toml declares "sync request" and "sync reply" (layer "sync",
#         dscp 48 from C-13, max_size equal to the markdown size). Their
#         [[message.field]] tables tile the markdown tables field for field
#         (offset, width and, for primitive types, type), and their
#         rejection_reasons include every exact reason named by AC 5. When the
#         paths are not overridden: `cargo run --bin wire-check` reports
#         "checks clean" and `cargo test -p hadronyche-tools --test
#         wire_tables` stays 12 passed, 0 failed, 0 ignored.
#
# Work items, as invariants:
#   W1  Header: both tables start with the magic (0x4d4e, offset 0, 16 bits)
#       and the version (0x0100, offset 2, 16 bits), and each table carries a
#       message type row (ticket Work 1).
#   W2  Timestamps: the unit (nanoseconds), the clock (monotonic) and the
#       relation to the slice header's send timestamp (004-08) are stated,
#       as the same as or explicitly different from it (Work 2).
#   W4  Behaviour restated, no fields: CS6 (48) (C-13), IPv4 unicast (C-100),
#       50 requests per second per follower (C-36) and 10 to 200 per second
#       accepted (C-112) appear in the prose, with the constants' numbers. No
#       table cell and no toml field names a DSCP or a rate (C-152, C-204).
#       Names are matched by token: split on underscores, dashes, spaces and
#       camelCase, so observed_dscp and request_rate fire while generate,
#       separate and accurate do not; a cell that negates the mention
#       ("carries no DSCP field") does not fire,
#       and the toml message's dscp is 48.
#   W5  A note slot for per-OS receive-timestamp remarks exists (epic 001),
#       the prose names C-131 and the regime stamp, and no sync table cell
#       names a receive-timestamp source (kernel, userspace, mac-hardware,
#       interrupt): the source belongs in the regime stamp.
#   DoD  Every prose paragraph and every table of the section carries an
#        evidence tag, or (tables) follows a tagged paragraph; any paragraph
#        mentioning an "open item" names an owner and a falsifier.
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
CONSTANTS_PATH = "doc/design/02-constants.md"

fails = []


def fail(ac, msg):
    fails.append((ac, msg))


def label(ac):
    return ac if isinstance(ac, str) else f"AC{ac}"


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


# ---------------------------------------------------------------------------
# Markdown helpers (same conventions as checks/004-08.sh and 004-09.sh)
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


# ---------------------------------------------------------------------------
# A small TOML reader (the subset parser of checks/004-08.sh and 004-09.sh:
# the check must run offline, unprivileged, on the Python the other checks use).
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
# Constants: read from doc/design/02-constants.md (that file wins)
# ---------------------------------------------------------------------------
CONST_TEXT = read_text(CONSTANTS_PATH) or ""


def const_row(cid):
    for line in CONST_TEXT.splitlines():
        if line.startswith(f"| {cid} |"):
            return line
    return ""


def first_int(pattern, text):
    m = re.search(pattern, text, re.I)
    return int(m.group(1)) if m else None


K = {
    "dscp": first_int(r"CS6\s*\((\d+)\)", const_row("C-13")),
    "req_rate": first_int(r"(\d+)\s+per\s+second", const_row("C-36")),
    "acc_lo": None,
    "acc_hi": None,
    "epoch_bits": None,
    "module_bits": None,
    "domain_bits": None,
    "scheme_bits": first_int(r"scheme\s+(\d+)\s+bits", const_row("C-12")),
    "counter_bits": first_int(r"counter\s+(\d+)\s+bits", const_row("C-12")),
    "tag_bits": first_int(r"tag\s+(\d+)\s+bits", const_row("C-12")),
}
m = re.search(r"(\d+)\s+to\s+(\d+)\s+per\s+second", const_row("C-112"), re.I)
if m:
    K["acc_lo"], K["acc_hi"] = int(m.group(1)), int(m.group(2))
m = re.search(r"(\d+)\s*/\s*(\d+)\s*/\s*(\d+)\s*bits", const_row("C-10"))
if m:
    K["epoch_bits"] = int(m.group(3))
m = re.search(r"(\d+)\s*/\s*(\d+)\s*/\s*(\d+)\s*bits", const_row("C-11"))
if m:
    K["module_bits"] = int(m.group(1))
    K["domain_bits"] = int(m.group(3))
for key, value in K.items():
    if value is None:
        fail("DoD", f"constants row for {key} could not be read from {CONSTANTS_PATH}")

# ---------------------------------------------------------------------------
# The closed enumeration, parsed from wire.md's "Rejection reasons" section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    fail(1, f"{wire_path} does not exist")
    for ac in (2, 3, 4, 5):
        fail(ac, f"{wire_path} does not exist")
    fail("DoD", f"{wire_path} does not exist")
    wire = ""

lines = wire.splitlines()
headings = heading_index(lines)


def section_span(pattern):
    """(start, end) line span of the first heading matching pattern, or None."""
    for i, (_, level, title) in enumerate(headings):
        if re.search(pattern, title, re.I):
            start = headings[i][0] + 1
            end = len(lines)
            for j in range(i + 1, len(headings)):
                if headings[j][1] <= level:
                    end = headings[j][0]
                    break
            return start, end
    return None


MEMBERS = {}
span = section_span(r"rejection[-\s]+reasons?")
if span is None:
    fail(5, "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read")
else:
    for _, _, rows in parse_tables("\n".join(lines[span[0]:span[1]])):
        for row in rows:
            if len(row) >= 2 and re.fullmatch(r"\d+", row[0].strip()):
                name = row[1].strip().strip("`*_ ")
                if re.fullmatch(r"[A-Z][A-Za-z0-9]+", name):
                    MEMBERS[name] = int(row[0].strip())
    if len(MEMBERS) != 13:
        fail(5, f"the closed enumeration parsed from wire.md has {len(MEMBERS)} names, not 13")
MEMBER_SET = set(MEMBERS)

# ---------------------------------------------------------------------------
# The sync section, located by heading text
# ---------------------------------------------------------------------------
PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.search(r"sync\s+requests?\b", title, re.I):
        sec_i = i
        break

placeholder = sec_i is None
section = ""
sec_lines = []
if sec_i is None:
    for ac in (1, 2, 3, 4, 5, 6, "W1", "W2", "W4", "W5", "DoD"):
        if ac == 6:
            continue  # the toml side is checked on its own below
        fail(ac, "no 'Sync request' section heading in wire.md")
else:
    level = headings[sec_i][1]
    start = headings[sec_i][0] + 1
    end = len(lines)
    for j in range(sec_i + 1, len(headings)):
        if headings[j][1] <= level:
            end = headings[j][0]
            break
    section = "\n".join(lines[start:end])
    sec_lines = section.splitlines()
    if not section.strip() or (PLACEHOLDER.search(section) and "|" not in section):
        placeholder = True
        for ac in (1, 2, 3, 4, 5, "W1", "W2", "W4", "W5", "DoD"):
            fail(ac, "the Sync request section is still the 004-19 placeholder")

prose_text = strip_fences(section)


def section_blocks(src_lines):
    """Blocks of the section: each heading is its own block; fenced code is
    kept inside its block; blank lines separate the rest."""
    out = []
    cur = []
    in_fence = False
    for ln in src_lines:
        if re.match(r"^\s*```", ln):
            in_fence = not in_fence
            cur.append(ln)
            continue
        if in_fence:
            cur.append(ln)
            continue
        if re.match(r"^#{1,6}\s", ln.lstrip()):
            if cur:
                out.append(cur)
                cur = []
            out.append([ln])
            continue
        if not ln.strip():
            if cur:
                out.append(cur)
                cur = []
            continue
        cur.append(ln)
    if cur:
        out.append(cur)
    return out


def block_kind(block):
    if len(block) == 1 and re.match(r"^#{1,6}\s", block[0].lstrip()):
        return "heading"
    if block and block[0].lstrip().startswith("```"):
        return "code"
    if all(ln.lstrip().startswith("|") for ln in block):
        return "table"
    return "prose"


SEC_BLOCKS = section_blocks(sec_lines)
PROSE = [
    "\n".join(b) for b in SEC_BLOCKS if block_kind(b) == "prose"
]
PROSE_ALL = "\n".join(PROSE)


# ---------------------------------------------------------------------------
# Field tables: classified request or reply by the nearest heading or the
# nearest paragraph above them.
# ---------------------------------------------------------------------------
def classify(text):
    t = text.lower()
    has_req = "request" in t
    has_rep = re.search(r"\brepl(y|ies)\b", t) is not None
    if has_rep and not has_req:
        return "reply"
    if has_req and not has_rep:
        return "request"
    return None


def table_label(start):
    heading = None
    para = None
    for k in range(start - 1, -1, -1):
        ln = sec_lines[k].strip()
        if not ln:
            continue
        if ln.startswith("#"):
            if heading is None:
                heading = ln
            break
        if ln.startswith("|"):
            break
        if para is None:
            para = ln
    return heading, para


FIELD_TABLES = []  # (cls, header, rows, start)
for start, header, rows in parse_tables(section):
    if not is_field_table(header):
        continue
    heading, para = table_label(start)
    cls = classify(heading or "") or classify(para or "")
    FIELD_TABLES.append((cls, header, rows, start))
if len(FIELD_TABLES) == 2 and all(t[0] is None for t in FIELD_TABLES):
    FIELD_TABLES = [("request", FIELD_TABLES[0][1], FIELD_TABLES[0][2], FIELD_TABLES[0][3]),
                    ("reply", FIELD_TABLES[1][1], FIELD_TABLES[1][2], FIELD_TABLES[1][3])]

REQ = next((t for t in FIELD_TABLES if t[0] == "request"), None)
REP = next((t for t in FIELD_TABLES if t[0] == "reply"), None)
if not placeholder and (REQ is None or REP is None):
    fail(1, "the section needs a request field table and a reply field table (name them "
         "by their heading, e.g. '### Request' and '### Reply')")


def table_fields(tab):
    out = []
    if tab is None:
        return out
    for row in tab[2]:
        if len(row) < 5:
            continue
        off = re.match(r"\s*(\d+)\s*$", row[0])
        width = width_to_bits(row[1])
        if not off or width is None:
            continue
        out.append({
            "off": int(off.group(1)) * 8,
            "w": width,
            "type": re.sub(r"[`*_ ]", "", row[2]).lower(),
            "meaning": row[3],
            "valid": row[4],
            "text": " | ".join(row),
        })
    return out


REQ_F = table_fields(REQ)
REP_F = table_fields(REP)


def size_of(fields, name):
    pos = 0
    ok = True
    for f in fields:
        if f["off"] != pos:
            fail(1, f"{name} table: field at byte {f['off'] // 8} does not start where the "
                 f"previous field ends (byte {pos // 8}); the fields must tile with no gap "
                 "and no overlap")
            ok = False
        pos = f["off"] + f["w"]
    if pos % 8:
        fail(1, f"{name} table: the fields sum to {pos} bits, not whole bytes")
    return pos // 8 if ok else None


REQ_SIZE = size_of(REQ_F, "request") if REQ_F else None
REP_SIZE = size_of(REP_F, "reply") if REP_F else None
PRIMITIVE = {"u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
             "f32", "q1.31", "uint", "pad"}

# ---------------------------------------------------------------------------
# AC 1 — identical byte size, stated as a number, equal to the tables' sum;
# explicit zero padding in the request
# ---------------------------------------------------------------------------
if not placeholder and REQ_SIZE is not None and REP_SIZE is not None:
    ac1 = True
    if REQ_SIZE != REP_SIZE:
        fail(1, f"request ({REQ_SIZE} bytes) and reply ({REP_SIZE} bytes) do not have the "
             "same byte size, by the sums of their field tables")
        ac1 = False
    stated = set()
    for sentence in re.split(r"(?<=[.;:])\s+|\n+", PROSE_ALL):
        if not re.search(r"size|long|length", sentence, re.I):
            continue
        if not re.search(r"request|reply|both|each|fixed|every|sync", sentence, re.I):
            continue
        for num in re.findall(r"(\d[\d ,_  ]*\d|\d)\s*-?\s*bytes?\b", sentence):
            stated.add(int(re.sub(r"[ ,_  ]", "", num)))
    if not stated:
        fail(1, "the section states no byte size for the request and reply as a number "
             "(a sentence with 'size' or 'long', the number and 'bytes')")
        ac1 = False
    elif stated != {REQ_SIZE}:
        fail(1, f"the stated byte size {sorted(stated)} does not equal the sum of the field "
             f"widths of each table, {REQ_SIZE} bytes")
        ac1 = False
    pad_rows = [f for f in REQ_F if f["w"] > 0 and (
        f["type"] == "pad" or re.search(r"\bpad|reserved", f["meaning"] + " " + f["valid"], re.I))]
    if not pad_rows:
        fail(1, "the request table has no explicit padding or reserved field that reaches the "
             "fixed size")
        ac1 = False
    elif not any(re.search(r"\b0\b|zero", f["valid"] + " " + f["meaning"], re.I) for f in pad_rows):
        fail(1, "the request padding has no required value of zero in its valid range")
        ac1 = False
    if ac1:
        print(f"PASS AC1: request and reply are {REQ_SIZE} bytes each, by the field-table sums, "
              "padding explicit in the request")

# ---------------------------------------------------------------------------
# AC 2 — echoed request send timestamp, epoch at 16 bits, holder module id at
# 64 bits, domain id at 16 bits with 0 = unconfigured
# ---------------------------------------------------------------------------
def find_rows(fields, meaning_pat):
    return [f for f in fields if re.search(meaning_pat, f["meaning"], re.I)]


if not placeholder and REP_SIZE is not None:
    ac2 = True
    echo = [f for f in find_rows(REP_F, r"send\s+timestamp") if re.search(r"echo", f["text"], re.I)]
    if not echo:
        fail(2, "the reply table has no row that echoes the request's send timestamp")
        ac2 = False
    elif not any(f["w"] == 64 for f in echo):
        fail(2, "the echoed send timestamp must be 64 bits, the width of the request's send timestamp")
        ac2 = False
    epoch = find_rows(REP_F, r"epoch")
    if not epoch:
        fail(2, "the reply table has no timeline epoch row")
        ac2 = False
    elif not any(f["w"] == K["epoch_bits"] for f in epoch):
        fail(2, f"the reply's timeline epoch must be {K['epoch_bits']} bits (C-10)")
        ac2 = False
    modid = find_rows(REP_F, r"module\s+id")
    if not modid:
        fail(2, "the reply table has no holder module id row")
        ac2 = False
    elif not any(f["w"] == K["module_bits"] for f in modid):
        fail(2, f"the holder module id must be {K['module_bits']} bits (C-11)")
        ac2 = False
    for name, tab in (("request", REQ_F), ("reply", REP_F)):
        dom = find_rows(tab, r"domain")
        if not dom:
            fail(2, f"the {name} table has no domain id row")
            ac2 = False
        elif not any(f["w"] == K["domain_bits"] for f in dom):
            fail(2, f"the {name} domain id must be {K['domain_bits']} bits (C-11)")
            ac2 = False
        elif not any(re.search(r"unconfigured|0\s*=|zero", f["text"], re.I) for f in dom):
            fail(2, f"the {name} domain id row does not state that 0 is the unconfigured domain")
            ac2 = False
    if ac2:
        print(f"PASS AC2: reply carries the echoed send timestamp (64 bits), the epoch "
              f"({K['epoch_bits']} bits) and the holder module id ({K['module_bits']} bits); "
              f"domain id {K['domain_bits']} bits, 0 = unconfigured")

# ---------------------------------------------------------------------------
# AC 3 — stateless, positive statement; negative keyword detector on tables
# ---------------------------------------------------------------------------
if not placeholder:
    ac3 = True
    if not any(re.search(r"stateless", b, re.I) for b in PROSE):
        fail(3, "the section does not state that the exchange is stateless")
        ac3 = False
    if not any(re.search(r"\becho", b, re.I) for b in PROSE):
        fail(3, "the section does not state that the reply echoes the request's own send timestamp")
        ac3 = False
    NEG = re.compile(
        r"\bsequence\b|\bseq\b|exchange\s+(?:id|number|identifier)|transaction|correlat|"
        r"cookie|per[- ]request|request\s+(?:id|number|identifier|counter)|\bpending\b|"
        r"\bhandle\b|\btoken\b",
        re.I,
    )
    for name, tab in (("request", REQ_F), ("reply", REP_F)):
        for f in tab:
            hit = NEG.search(f["meaning"] + " " + f["valid"])
            if hit:
                fail(3, f"the {name} table has a field that only per-request holder state could "
                     f"fill ('{hit.group(0)}'): {f['meaning'][:90]}")
                ac3 = False
    if ac3:
        print("PASS AC3: stateless, the reply echoes the request's send timestamp, no "
              "per-request field in either table")

# ---------------------------------------------------------------------------
# AC 4 — authentication-reservation decision, [decision], falsifier; C-12 and
# C-115 values when it is carried, no such field when it is not
# ---------------------------------------------------------------------------
if not placeholder:
    ac4 = True
    dec_i = None
    for i, b in enumerate(PROSE):
        if re.search(r"authentication|reservation", b, re.I) and re.search(r"\[decision\]", b, re.I):
            dec_i = i
            break
    if dec_i is None:
        fail(4, "no [decision]-tagged paragraph states the authentication-reservation choice")
        ac4 = False
    else:
        decision = None
        for sentence in re.split(r"(?<=[.;])\s+", PROSE[dec_i]):
            if not re.search(r"authentication|reservation", sentence, re.I):
                continue
            if re.search(r"\b(?:not|no|without|never)\b|n't", sentence, re.I):
                decision = "not carried"
                break
            if re.search(r"\b(?:carr(?:y|ies)|includ\w*|bears?|has)\b", sentence, re.I):
                decision = "carried"
                break
        if decision is None:
            fail(4, "the decision paragraph does not say whether sync messages carry the "
                 "authentication reservation")
            ac4 = False
        window = "\n".join(PROSE[max(0, dec_i - 1): dec_i + 2])
        fm = re.search(r"falsifier\s*[:\-—]?\s*(.*)", window, re.I | re.S)
        if not fm or len(fm.group(1).split()) < 4:
            fail(4, "the authentication decision has no falsifier (the word 'falsifier' followed by "
                 "the condition that would overturn it)")
            ac4 = False
        if decision == "carried":
            for name, fields in (("request", REQ_F), ("reply", REP_F)):
                if not any(f["w"] == K["scheme_bits"] and re.search(r"scheme|authentication", f["meaning"], re.I)
                           and re.search(r"\b0\b|none|zero", f["valid"] + f["meaning"], re.I)
                           for f in fields):
                    fail(4, f"the {name} table carries the reservation but has no "
                         f"{K['scheme_bits']}-bit scheme field with value 0 (C-115)")
                    ac4 = False
                if not any(f["w"] == K["counter_bits"] and re.search(r"counter", f["meaning"], re.I)
                           and re.search(r"live", f["text"], re.I) for f in fields):
                    fail(4, f"the {name} table carries the reservation but has no "
                         f"{K['counter_bits']}-bit live message counter (C-115)")
                    ac4 = False
                if not any(f["w"] == K["tag_bits"] and re.search(r"\btag\b", f["meaning"], re.I)
                           and re.search(r"zero|\b0\b", f["valid"] + f["meaning"], re.I)
                           for f in fields):
                    fail(4, f"the {name} table carries the reservation but has no "
                         f"{K['tag_bits']}-bit zero tag field (C-115)")
                    ac4 = False
        elif decision == "not carried":
            for name, fields in (("request", REQ_F), ("reply", REP_F)):
                for f in fields:
                    if re.search(r"authentication|scheme|\btag\b|counter|reservation", f["meaning"], re.I):
                        fail(4, f"the decision says sync messages do not carry the reservation, "
                             f"but the {name} table has the field: {f['meaning'][:80]}")
                        ac4 = False
    if ac4 and dec_i is not None:
        print(f"PASS AC4: authentication decision [decision] with falsifier ({decision})")

# ---------------------------------------------------------------------------
# AC 5 — at least 6 invalid forms with outcomes; exact reasons where the ticket
# names one; every named reason a member of the closed enumeration
# ---------------------------------------------------------------------------
SHORT_FORM = (r"\btoo\s+short\b|\bshorter\s+than\b|\bfewer\s+than\b|\bless\s+than\b|"
              r"\bunder\s+\d+\s+(?:bytes?|octets?)\b|\bsmaller\s+than\b")
FIXED_WORD = r"fixed|not\s+equal|equal\s+to|other\s+than"
SIZE_FORM = r"\bsize\b|\bbytes?\b|\boctets?\b|\blength\b"
FORM_TABLE = None
for _, header, rows in parse_tables(section):
    heads = [re.sub(r"[^a-z]", "", h.lower()) for h in header]
    if any("form" in h or "invalid" in h for h in heads) and any("outcome" in h for h in heads):
        FORM_TABLE = rows
        break

if not placeholder:
    ac5 = True
    if FORM_TABLE is None:
        fail(5, "no table of invalid forms with an outcome column")
        ac5 = False
    else:
        rows = [r for r in FORM_TABLE if len(r) >= 2 and r[0].strip() and " ".join(r[1:]).strip()]
        if len(rows) < 6:
            fail(5, f"only {len(rows)} invalid form(s) with outcomes; at least 6 are required")
            ac5 = False
        # categories, checked in this order against the form cell. Size rows
        # are not a category: they are classified by their outcome, below.
        CATS = [
            ("ReservedNonZero", r"\bpad|reserved", "exact"),
            ("BadMagic", r"magic", "exact"),
            ("VersionMismatch", r"version", "exact"),
            ("ForeignDomain", r"foreign", "exact"),
            ("WrongEpoch", r"epoch", "exact"),
            ("holder", r"holder", "member"),
        ]
        seen = {}
        short_rows = []
        fixed_rows = []
        for r in rows:
            form = r[0]
            outcome = " | ".join(r[1:])
            cat = next((c for c in CATS if re.search(c[1], form, re.I)), None)
            if cat is None:
                if re.search(SHORT_FORM, form, re.I) and not re.search(FIXED_WORD, form, re.I):
                    short_rows.append((form, outcome))
                elif re.search(SIZE_FORM, form, re.I):
                    fixed_rows.append((form, outcome))
                continue
            if cat[0] in seen:
                continue
            seen[cat[0]] = outcome
            if cat[2] == "exact":
                if not re.search(r"\b" + cat[0] + r"\b", outcome):
                    fail(5, f"the invalid form '{form[:60]}' must be counted under {cat[0]} "
                         f"(the ticket names it): {outcome[:90]}")
                    ac5 = False
            else:
                names = [n for n in MEMBERS if re.search(r"\b" + n + r"\b", outcome)]
                if not names:
                    fail(5, f"the invalid form '{form[:60]}' has no member reason in its outcome")
                    ac5 = False
        for cat in CATS:
            if cat[0] not in seen and cat[0] != "holder":
                fail(5, f"no invalid form for {cat[0]} (ticket list: size, bad magic, bad version, "
                     "foreign domain, request padding non-zero, reply from a non-holder, wrong epoch)")
                ac5 = False
            if cat[0] == "holder" and "holder" not in seen:
                fail(5, "no invalid form for a reply from a non-holder")
                ac5 = False
        # size rows, by outcome: a datagram shorter than the minimum that
        # carries the magic and the version is TooShort; a size that is not
        # the fixed size is BadLength, and the ticket names that outcome
        for form, outcome in short_rows:
            if not re.search(r"\bTooShort\b", outcome) or re.search(r"\bBadLength\b", outcome):
                fail(5, f"the invalid form '{form[:60]}' is shorter than the minimum that carries the "
                     "magic and the version: it must be counted under TooShort: " + outcome[:90])
                ac5 = False
        for form, outcome in fixed_rows:
            if re.search(r"\bTooShort\b", outcome):
                fail(5, f"the invalid form '{form[:60]}' is a size that is not the fixed size, but it "
                     "is mapped to TooShort; it must be counted under BadLength (the ticket names it)")
                ac5 = False
        if not any(re.search(r"\bBadLength\b", o) and not re.search(r"\bTooShort\b", o)
                   for _, o in fixed_rows):
            fail(5, "no invalid form for a size that is not the fixed size counted under BadLength "
                 "(the ticket names it)")
            ac5 = False
        # every outcome token must be a member; negations are not outcomes
        for r in rows:
            outcome = " | ".join(r[1:])
            for m_ in re.finditer(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b", outcome):
                name = m_.group(0)
                before = outcome[max(0, m_.start() - 25): m_.start()]
                if name not in MEMBER_SET and not re.search(r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$", before, re.I):
                    fail(5, f"the outcome names '{name}', which is not in the closed enumeration")
                    ac5 = False
    # every reason named "under X" in the section must be a member
    for m_ in re.finditer(r"\bunder\s+`?([A-Z][A-Za-z]+)`?", section):
        if m_.group(1) not in MEMBER_SET:
            fail(5, f"the section counts a rejection under '{m_.group(1)}', which is not a member "
                 "of the closed enumeration")
            ac5 = False
    if ac5:
        print("PASS AC5: invalid forms with outcomes, exact reasons where the ticket names them, "
              "every reason a member")

# ---------------------------------------------------------------------------
# AC 6 (markdown-to-toml tiling and the toml declaration)
# ---------------------------------------------------------------------------
tables_text = read_text(tables_path)
toml = None
if tables_text is None:
    fail(6, f"{tables_path} does not exist")
else:
    try:
        toml = parse_toml(tables_text)
    except TomlError as error:
        fail(6, f"{tables_path} does not parse: {error}")

msgs = toml.get("message", []) if isinstance(toml, dict) else []
if not isinstance(msgs, list):
    msgs = []


def name_tokens(text):
    """Tokens of an identifier or a cell: split on non-alphanumerics and on
    camelCase boundaries, lower-cased. 'observed_dscp' -> observed, dscp."""
    spaced = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", text)
    spaced = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1 \2", spaced)
    return [t.lower() for t in re.split(r"[^A-Za-z0-9]+", spaced) if t]


DSCP_RATE_TOKENS = {"dscp", "rate", "codepoint"}


def is_dscp_or_rate(text):
    return bool(DSCP_RATE_TOKENS & set(name_tokens(text)))


# a cell that negates the mention ("carries no DSCP field") is not a field
NEG_MENTION = re.compile(r"\b(?:no|not|without|never)\b[^|.;]{0,30}?\b(?:dscp|codepoint|rate)\b", re.I)


def toml_message(pattern):
    for mm in msgs:
        if isinstance(mm, dict) and re.fullmatch(pattern, str(mm.get("name", "")), re.I):
            return mm
    return None


T_REQ = toml_message(r"sync[ _-]request")
T_REP = toml_message(r"sync[ _-]reply")
if T_REQ is None:
    fail(6, "wire.toml declares no [[message]] named 'sync request'")
if T_REP is None:
    fail(6, "wire.toml declares no [[message]] named 'sync reply'")

ALL_TOML_REASONS = set()
for name, tm, fields_md, size_md in (("sync request", T_REQ, REQ_F, REQ_SIZE),
                                      ("sync reply", T_REP, REP_F, REP_SIZE)):
    if tm is None:
        continue
    if tm.get("layer") != "sync":
        fail(6, f"{name} in wire.toml must have layer = \"sync\"")
    if tm.get("dscp") != K["dscp"]:
        fail(6, f"{name} in wire.toml must have dscp = {K['dscp']} (C-13, sync CS6)")
    ms = tm.get("max_size")
    if not isinstance(ms, int):
        fail(1, f"{name} in wire.toml has no integer max_size")
        continue
    tfields = tm.get("field", [])
    if not isinstance(tfields, list):
        tfields = []
    pos = 0
    tlist = []
    for f in sorted(tfields, key=lambda x: x.get("offset", 0) if isinstance(x, dict) else 0):
        if not isinstance(f, dict):
            continue
        tlist.append((f.get("offset"), f.get("width"), str(f.get("type", "")).lower()))
        if f.get("offset") != pos:
            fail(1, f"{name} in wire.toml: field '{f.get('name')}' is not contiguous (bit {pos} expected)")
        pos = (f.get("offset") or 0) + (f.get("width") or 0)
    if pos != ms * 8:
        fail(1, f"{name} in wire.toml: fields sum to {pos} bits, max_size is {ms} bytes")
    if size_md is not None and ms != size_md:
        fail(1, f"{name}: wire.toml max_size {ms} differs from the markdown size {size_md}")
    ALL_TOML_REASONS.update(tm.get("rejection_reasons", []) if isinstance(tm.get("rejection_reasons"), list) else [])
    for reason in tm.get("rejection_reasons", []) if isinstance(tm.get("rejection_reasons"), list) else []:
        if reason not in MEMBER_SET:
            fail(6, f"{name} names '{reason}', which is not a member of the closed enumeration")
    if fields_md:
        mlist = [(f["off"], f["w"], f["type"]) for f in fields_md]
        if len(mlist) != len(tlist):
            fail(6, f"{name}: the markdown table has {len(mlist)} fields, wire.toml {len(tlist)}")
        else:
            for (mo, mw, mt), (to, tw, tt) in zip(mlist, tlist):
                if (mo, mw) != (to, tw):
                    fail(6, f"{name}: markdown field at bit {mo} ({mw} bits) does not match "
                         f"wire.toml field at bit {to} ({tw} bits)")
                    break
                if mt in PRIMITIVE and mt != tt:
                    fail(6, f"{name}: markdown type '{mt}' at bit {mo} does not match wire.toml type '{tt}'")
                    break
    if tm is T_REQ and tm is not None:
        if not any(isinstance(f, dict) and f.get("type") == "pad" and (f.get("width") or 0) > 0
                   for f in tfields):
            fail(1, "sync request in wire.toml has no pad field of width > 0 (the explicit padding)")
    for f in tfields if isinstance(tfields, list) else []:
        if isinstance(f, dict) and is_dscp_or_rate(str(f.get("name", ""))):
            fail("W4", f"{name} in wire.toml names a DSCP or rate field ('{f.get('name')}'); "
                 "C-152 and C-204 keep both out of sync messages")

if not placeholder and REP_F and REQ_F:
    exact_named = set()
    if FORM_TABLE is not None:
        for r in FORM_TABLE:
            for name in MEMBERS:
                if re.search(r"\b" + name + r"\b", " | ".join(r[1:])) and name in (
                        "BadLength", "BadMagic", "VersionMismatch", "ForeignDomain",
                        "ReservedNonZero", "WrongEpoch"):
                    exact_named.add(name)
    missing = sorted(exact_named - ALL_TOML_REASONS)
    if missing and T_REQ is not None and T_REP is not None:
        fail(6, "the sync messages' rejection_reasons in wire.toml miss: " + ", ".join(missing))

# W1, W2, W4, W5 and DoD read the markdown only
if not placeholder and REQ_F and REP_F:
    # --- W1: header magic and version, message type row --------------------
    for name, fields in (("request", REQ_F), ("reply", REP_F)):
        if len(fields) < 3:
            fail("W1", f"the {name} table has fewer than three header fields")
            continue
        if not (fields[0]["off"] == 0 and fields[0]["w"] == 16 and re.search(r"4d4e", fields[0]["text"], re.I)):
            fail("W1", f"the {name} table does not start with the 2-byte magic 0x4d4e at offset 0")
        if not (fields[1]["off"] == 16 and fields[1]["w"] == 16 and re.search(r"0x0100|\b1\.0\b", fields[1]["text"], re.I)):
            fail("W1", f"the {name} table's version is not the 2-byte v1 version 0x0100 at offset 2")
        if not any(re.search(r"message\s+type|type\s+code|message\s+kind", f["meaning"], re.I) for f in fields):
            fail("W1", f"the {name} table has no message type row (ticket Work 1)")

    # --- W2: timestamp unit, clock and relation to the slice header ---------
    if not re.search(r"nanosecond|\bns\b", PROSE_ALL, re.I):
        fail("W2", "the sync timestamps' unit (nanoseconds) is not stated")
    if not re.search(r"monotonic", PROSE_ALL, re.I):
        fail("W2", "the sync timestamps' clock (monotonic) is not stated")
    if not re.search(r"slice[\s-]header", PROSE_ALL, re.I):
        fail("W2", "the section does not relate its timestamps to the slice-header send timestamp")
    elif not re.search(r"same\s+(?:unit|clock|basis|as)|consistent\s+with|identical\s+to|"
                       r"explicitly\s+different|differs?\s+from|different\s+from", PROSE_ALL, re.I):
        fail("W2", "the timestamps are neither stated as consistent with nor explicitly different "
             "from the slice-header send timestamp")

    # --- W4: behaviour restated with the constants' numbers -----------------
    if K["dscp"] is not None and not re.search(r"CS6\s*\(\s*%d\s*\)" % K["dscp"], PROSE_ALL):
        fail("W4", f"the section does not restate the DSCP as CS6 ({K['dscp']}) (C-13)")
    if not (re.search(r"IPv4", PROSE_ALL) and re.search(r"unicast", PROSE_ALL, re.I)):
        fail("W4", "the section does not restate IPv4 unicast (C-100) as behaviour")
    if K["req_rate"] is not None and not re.search(r"\b%d\b[^.\n]{0,40}per\s+second" % K["req_rate"], PROSE_ALL, re.I):
        fail("W4", f"the section does not restate {K['req_rate']} requests per second per follower (C-36)")
    if K["acc_lo"] is not None and not re.search(
            r"\b%d\b[^.\n]{0,40}\b%d\b[^.\n]{0,40}per\s+second" % (K["acc_lo"], K["acc_hi"]), PROSE_ALL, re.I):
        fail("W4", f"the section does not restate the accepted {K['acc_lo']} to {K['acc_hi']} "
             "requests per second (C-112)")
    for name, fields in (("request", REQ_F), ("reply", REP_F)):
        for f in fields:
            cell = NEG_MENTION.sub("", f["text"])
            if is_dscp_or_rate(cell) or re.search(r"per\s+second|per-second", cell, re.I):
                fail("W4", f"the {name} table names a DSCP or a rate (C-152, C-204): {f['meaning'][:80]}")

    # --- W5: receive-timestamp note slot and the regime-stamp home ----------
    if not re.search(r"per[- ]OS", PROSE_ALL, re.I):
        fail("W5", "no note slot for per-OS receive-timestamp remarks (epic 001) in the section")
    if not re.search(r"C-131", PROSE_ALL):
        fail("W5", "the section does not name C-131 for the receive-timestamp source")
    if not re.search(r"regime\s+stamp", PROSE_ALL, re.I):
        fail("W5", "the section does not say the receive-timestamp source lives in the regime stamp")
    for f in REP_F + REQ_F:
        if re.search(r"\b(?:kernel|userspace|mac-hardware|interrupt)\b", f["meaning"], re.I):
            fail("W5", f"a sync table names a receive-timestamp source: {f['meaning'][:80]}")
            break

    # --- DoD: evidence tags ------------------------------------------------
    last_prose_tagged = True
    for b in SEC_BLOCKS:
        kind = block_kind(b)
        text = "\n".join(b)
        if kind == "heading" or kind == "code":
            continue
        if kind == "table":
            if not (has_evidence(text) or last_prose_tagged):
                fail("DoD", "a table of the Sync request section carries no evidence tag and "
                     "follows an untagged paragraph")
            continue
        words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", text)))
        tagged = has_evidence(text)
        if words >= 5 and not tagged:
            fail("DoD", "a paragraph of the Sync request section carries no evidence tag "
                 "([code] [measured] [kernel-doc] [standard] [memory] [decision]): " + text.splitlines()[0][:90])
        last_prose_tagged = tagged or words < 5

    flat = [("\n".join(b)) for b in SEC_BLOCKS]
    for i, text in enumerate(flat):
        if re.search(r"open\s+item", text, re.I):
            window = "\n".join(flat[i: i + 3])
            if not re.search(r"owner", window, re.I) or not re.search(r"falsifier", window, re.I):
                fail("DoD", "an open item of the Sync request section names no owner or no falsifier: "
                     + text.splitlines()[0][:90])

# ---------------------------------------------------------------------------
# The 004-02 checker and the design checker (repository-level, not overridden)
# ---------------------------------------------------------------------------
if not overridden:
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
            fail(6, f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(6, f"could not run cargo run --bin wire-check: {error}")

    try:
        proc = subprocess.run(
            ["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail(6, f"could not run the wire_tables tests: {error}")
    if proc is not None:
        out = proc.stdout + proc.stderr
        if proc.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail(6, f"cargo test -p hadronyche-tools --test wire_tables failed:\n{tail}")
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if result is None:
            fail(6, "the wire_tables test run printed no 'test result:' line")
        else:
            mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result)
            if not mm:
                fail(6, f"could not read the test counts from: {result}")
            elif (int(mm.group(1)), int(mm.group(2)), int(mm.group(3))) != (12, 0, 0):
                fail(6, "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: " + result)

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
    print("NOTE AC6: document paths overridden; repository-level checks skipped")

# ---------------------------------------------------------------------------
if fails:
    for ac, msg in fails:
        print(f"FAIL {label(ac)}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 6/6 acceptance criteria for 004-19, work items W1 W2 W4 W5, DoD tags present")
PY
