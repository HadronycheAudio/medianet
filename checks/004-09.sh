#!/usr/bin/env bash
# 004-09 — in-band announcement layouts.
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
#   AC 1  The Announcements section defines the announcement CONTAINER and
#         the FOUR types (end-of-stream, re-alignment, timeline epoch
#         switch, transport reference change). Container: the 004-06
#         presence flag (bit 5 of the flags byte) referenced, a count field
#         with a stated maximum, a per-announcement type field and a
#         per-announcement length field, and the 004-04 minor-change rule
#         encoded as a length-prefix requirement (unknown parts are skipped
#         by length by a same-major receiver). Per type: a TYPE CODE (an
#         integer in a code table row or "type code N" beside the name), a
#         FIELD TABLE (Offset/Width/Type/Meaning/Valid range) assigned to
#         the type, and a BYTE SIZE (per type, or a single per-announcement
#         size statement that covers all four — the author's choice). Each
#         announcement names a FUTURE POSITION of 64 bits (the slice
#         header's position form, u64 / 8 bytes).
#   AC 2  The maximum NUMBER of announcements per packet and the maximum
#         ANNOUNCEMENT BYTES per packet are stated as NUMBERS. The numbers
#         must be self-consistent and fit the C-137 remainder: the section
#         must show the cap arithmetic (62 bytes remain of the 100-byte cap
#         after the 38-byte packet + slice headers), the stated maximum
#         bytes must be <= 62 minus the measurement extension's and the
#         checksum's shares when the section states them, and
#         max_count x (per-announcement bytes derived from the container
#         field table) must be <= the stated maximum bytes.
#   AC 3  At least 6 invalid forms are listed with outcomes, covering the
#         ticket's list: unknown type within the same major (SKIPPABLE by
#         length — encoded as NOT a rejection), length overrun (BadLength),
#         named position not in the future (BadLength: at or before the
#         packet's slice position), duplicate type in one packet, announcement
#         in a gap slice (if forbidden), count above maximum (BadLength),
#         announcement-present flag with a zero count (BadLength). Every
#         rejection reason named is a member of the closed 13-name
#         enumeration; no invented reason.
#   AC 4  C-43 (lead 64 slices, flat; 64 x 0.5 ms = 32 ms at the C-02
#         defaults) and C-44 (re-issued every 16 slices until the named
#         position passes, in every bundle member) restated as SENDER rules;
#         the lead binds the sender's FIRST issue of a change.
#   AC 5  The proposed re-alignment and transport-reference-change payloads
#         are recorded as OPEN ITEMS with epic 005 as the confirming owner
#         and the ticket's falsifier (a receiver cannot act on the
#         announcement from its payload alone). The epoch-switch payload
#         states whether it names the new session (C-19).
#   AC 6  The machine-readable side: the author extends the media packet
#         [[message]] (fields at offset >= 304 bits = byte 38) or adds a
#         sub-table message whose name contains "announcement" — both are
#         accepted. Required: a count field with max_count equal to the
#         stated maximum, a type field, a length field and a 64-bit
#         position field; the carrying message's rejection_reasons include
#         BadLength and name only enumeration members. `cargo run --bin
#         wire-check` passes and `cargo test -p hadronyche-tools --test
#         wire_tables` stays 12/0/0.
#
# DEFECT (004-09 amendment): the lead (C-43) is a SENDER obligation on the
# FIRST issue of a change; a re-issue (C-44) inside the lead is valid; a
# receiver rejects (BadLength) only a named position that is NOT in the
# future. The check fails while a table row, a field-table cell, a prose
# sentence or a wire.toml field string makes the lead a validity condition,
# and requires the statement that a re-issue inside the lead is valid. A
# Validity-rules row whose invalid-form cell names the lead and whose outcome
# rejects fails with no exemption; the Sender obligations text must state the
# stop rule at the named position (ruling R6: no announcement rides in a slice
# at or after the position it names).
#
# DoD also: every block of the new section carries an evidence tag ([code],
# [measured], [kernel-doc], [standard], [memory], [decision]); the design
# checker prints OK.
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
# Markdown helpers (same conventions as checks/004-01.sh … checks/004-08.sh)
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
# Load wire.md, find the Announcements section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    for ac in range(1, 7):
        print(f"FAIL AC{ac}: {wire_path} does not exist")
    print("FAIL DoD: cannot check — wire.md does not exist")
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.fullmatch(r"announcements?", title, re.I):
        sec_i = i
        break

PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)
placeholder = sec_i is None
section = ""
section_prose = ""
if sec_i is None:
    for ac in range(1, 7):
        fail(ac, "no 'Announcements' section heading in wire.md")
    fail("DoD", "no 'Announcements' section heading in wire.md")
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
        for ac in range(1, 7):
            fail(ac, "the Announcements section is still the 004-09 placeholder")
        fail("DoD", "the Announcements section is still the 004-09 placeholder")

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
# Collect the section's units: field tables, other table rows, list items
# ---------------------------------------------------------------------------
all_tables = parse_tables(section)

field_tables = []  # (rows_text, [rows], intro_text)
sec_lines = section.splitlines()
for tstart, header, rows in all_tables:
    if is_field_table(header):
        intro = "\n".join(sec_lines[max(0, tstart - 5) : tstart])
        field_tables.append(("\n".join(" | ".join(r) for r in rows), rows, intro))

units = []  # every table row and list item, as flat text
for _, _, rows in all_tables:
    for row in rows:
        units.append(" | ".join(row))
for line in section_prose.splitlines():
    s = line.strip()
    if re.match(r"^[-*]\s+", s) or re.match(r"^\d+[.)]\s+", s):
        units.append(s)


def unit_window(unit):
    """The unit plus its immediate neighbours (for arithmetic that sits one
    row or bullet away)."""
    k = units.index(unit)
    lo, hi = max(0, k - 1), min(len(units), k + 2)
    return "\n".join(units[lo:hi])


# ---------------------------------------------------------------------------
# AC 1 — the container and the four types: codes, field tables, byte sizes
# ---------------------------------------------------------------------------
TYPES = [
    ("end-of-stream", r"end[\s_-]*of[\s_-]*stream"),
    ("re-alignment", r"re[\s_-]*alignment"),
    ("timeline epoch switch", r"epoch[\s_-]*switch"),
    ("transport reference change", r"transport[\s_-]*reference"),
]

if not placeholder:
    ac1_ok = True

    # --- the container -----------------------------------------------------
    # presence flag: the 004-06 announcement bit (bit 5 of the flags byte)
    if not re.search(r"bit\s*5|announcement\s+present", section_prose, re.I) or not re.search(
        r"004-06|flags?\s+byte", section_prose, re.I
    ):
        fail(
            1,
            "the container does not tie the presence flag to the 004-06 flags "
            "byte (announcement present, bit 5)",
        )
        ac1_ok = False

    # count, per-announcement type and length as container fields
    container_rows = []
    for rows_text, rows, intro in field_tables:
        joined = rows_text + "\n" + intro
        if re.search(r"\bcount\b", joined, re.I) and re.search(
            r"\btype\b", joined, re.I
        ) and re.search(r"\blength\b", joined, re.I):
            container_rows = rows
            break
    if not container_rows:
        fail(
            1,
            "no container field table with count, per-announcement type and "
            "per-announcement length fields",
        )
        ac1_ok = False

    # the 004-04 minor-change rule encoded as a length-prefix requirement:
    # unknown parts are skipped by length by a same-major receiver
    if not re.search(r"skip", section_prose, re.I) or not re.search(
        r"length", section_prose, re.I
    ):
        fail(
            1,
            "the section does not state the skip-by-length requirement (004-04's "
            "minor-change rule: a length prefix per announcement, so unknown "
            "types are skippable by a same-major receiver)",
        )
        ac1_ok = False
    elif not re.search(
        r"same\s+major|minor|004-04", section_prose, re.I
    ):
        fail(
            1,
            "the skip-by-length rule is not tied to the version rule (same-major "
            "receiver, 004-04's minor change)",
        )
        ac1_ok = False

    # --- the named future position, 64 bits --------------------------------
    pos_row = None
    for rows_text, rows, intro in field_tables:
        for row in rows:
            joined = " | ".join(row)
            if re.search(r"\bposition\b", joined, re.I):
                bits = width_to_bits(row[1]) if len(row) > 1 else None
                if bits == 64:
                    pos_row = joined
                    break
        if pos_row:
            break
    if pos_row is None:
        fail(
            1,
            "no announcement field table carries the named future position as "
            "64 bits (the slice header's position form, u64 / 8 bytes)",
        )
        ac1_ok = False
    if not re.search(r"future", section_prose, re.I):
        fail(
            1,
            "the section never says the named position is a FUTURE position (the "
            "ticket: each announcement names a future position, as in the slice "
            "header)",
        )
        ac1_ok = False

    # --- per type: code, field table, byte size ----------------------------
    # a global per-announcement byte size is accepted for all four types
    global_size = None
    m = re.search(
        r"each\s+announcement[^.;\n]{0,80}?(\d+)\s*bytes?"
        r"|announcement[^.;\n]{0,60}?\bis\s+(\d+)\s*bytes?"
        r"|(\d+)\s*bytes?\s+per\s+announcement",
        section_prose,
        re.I,
    )
    if m:
        global_size = int(next(g for g in m.groups() if g))

    # a global type-code table is accepted: rows mapping an integer to a type
    code_table_hits = {}
    for _, _, rows in all_tables:
        for row in rows:
            joined = " | ".join(row)
            ints = [
                int(x)
                for x in re.findall(r"(?<![\w.\-])(\d{1,3})(?![\w.\-])", joined)
                if int(x) <= 255
            ]
            if not ints:
                continue
            for label, pattern in TYPES:
                if re.search(pattern, joined, re.I):
                    code_table_hits.setdefault(label, joined)

    for label, pattern in TYPES:
        hit = None
        for u in units:
            if re.search(pattern, u, re.I):
                hit = u
                break
        if hit is None:
            fail(1, f"the section never names the {label} announcement type")
            ac1_ok = False
            continue

        # type code: an integer in a code-table row, or "type code N" beside
        # the name, or a small integer stated in the type's own row
        code_ok = label in code_table_hits
        if not code_ok:
            for u in units:
                if not re.search(pattern, u, re.I):
                    continue
                m = re.search(r"type\s+code\s*(?:is|:|=)?\s*(\d{1,3})\b", u, re.I)
                if m and int(m.group(1)) <= 255:
                    code_ok = True
                    break
                ints = [
                    int(x)
                    for x in re.findall(r"(?<![\w.\-])(\d{1,3})(?![\w.\-])", u)
                    if int(x) <= 255
                ]
                if ints and re.search(r"\bcode\b", u, re.I):
                    code_ok = True
                    break
        if not code_ok:
            fail(
                1,
                f"the {label} announcement has no stated TYPE CODE (an integer in "
                "a code table row, or 'type code N' beside the name)",
            )
            ac1_ok = False

        # field table assigned to the type (its rows or intro name the type)
        type_table = None
        for rows_text, rows, intro in field_tables:
            window = rows_text + "\n" + intro
            if re.search(pattern, window, re.I):
                type_table = (rows_text, rows, intro)
                break
        if type_table is None:
            fail(
                1,
                f"the {label} announcement has no field table (Offset/Width/Type/"
                "Meaning/Valid range) of its own",
            )
            ac1_ok = False

        # byte size: per type, or the global per-announcement size
        size_ok = global_size is not None
        size_txt = f"{global_size} bytes per announcement (global statement)" if size_ok else ""
        if not size_ok and type_table is not None:
            rows_text, rows, intro = type_table
            window = rows_text + "\n" + intro
            m = re.search(r"(\d+)\s*bytes?", window, re.I)
            if m:
                size_ok = True
                size_txt = f"{m.group(1)} bytes ({label})"
        if not size_ok:
            fail(
                1,
                f"the {label} announcement has no stated BYTE SIZE (state it in "
                "the type's table or as a per-announcement size covering all "
                "four types)",
            )
            ac1_ok = False

    if ac1_ok:
        print(
            "PASS AC1: container present (004-06 bit-5 presence flag, count, "
            "per-announcement type + length, skippable by length for same-major "
            "receivers, 004-04); four types with codes, field tables and byte "
            "sizes; named future position 64 bits"
        )

# ---------------------------------------------------------------------------
# AC 2 — maximum announcements and maximum announcement bytes, as numbers,
#        self-consistent and inside the C-137 remainder
# ---------------------------------------------------------------------------
max_count = None
max_bytes = None
per_ann_bytes = None
if not placeholder:
    ac2_ok = True

    # maximum number of announcements per packet
    for m in re.finditer(
        r"at\s+most\s+(\d+)\s+announcements?"
        r"|maximum\s+(?:of\s+)?(\d+)\s+announcements?"
        r"|(\d+)\s+announcements?\s+per\s+packet",
        section_prose,
        re.I,
    ):
        v = int(next(g for g in m.groups() if g))
        if v > 0:
            max_count = v
            break
    if max_count is None:
        fail(
            2,
            "the maximum NUMBER of announcements per packet is not stated as a "
            "number (needed by 004-14)",
        )
        ac2_ok = False

    # maximum announcement bytes per packet. The alternatives are worded so
    # the C-137 remainder statement ("62 bytes for the announcements, the
    # measurement extension …") is NOT read as the stated maximum: a
    # maximum needs the announcement-bytes or per-packet wording.
    for m in re.finditer(
        r"(\d+)\s+announcement\s+bytes?"
        r"|announcement\s+bytes[^.;\n]{0,40}?(\d+)"
        r"|at\s+most\s+(\d+)\s*bytes?[^.;\n]{0,30}?announcements?\s+per\s+packet"
        r"|(\d+)\s*bytes?\s+of\s+announcements?\b",
        section_prose,
        re.I,
    ):
        v = int(next(g for g in m.groups() if g))
        if v > 0:
            max_bytes = v
            break
    if max_bytes is None:
        fail(
            2,
            "the maximum ANNOUNCEMENT BYTES per packet is not stated as a number "
            "(needed by 004-14)",
        )
        ac2_ok = False

    # the C-137 cap arithmetic must be shown: 62 bytes remain of the 100-byte
    # cap after the 38-byte packet + slice headers
    if not re.search(r"\b62\b", section_prose) and not (
        re.search(r"\b100\b", section_prose) and re.search(r"\b38\b", section_prose)
    ):
        fail(
            2,
            "the section does not show the C-137 cap arithmetic (100-byte cap, "
            "38 bytes used by the packet + slice headers, 62 remaining for "
            "announcements + measurement extension + checksum)",
        )
        ac2_ok = False

    # the shares the section itself states for the measurement extension and
    # the checksum come out of the 62 bytes first
    available = 62
    m = re.search(
        r"measurement\s+extension[^.;\n]{0,100}?(\d+)\s*bytes?", section_prose, re.I
    )
    ext_share = int(m.group(1)) if m else 0
    m = re.search(r"checksum[^.;\n]{0,100}?(\d+)\s*bytes?", section_prose, re.I)
    cks_share = int(m.group(1)) if m else 0
    available -= ext_share + cks_share

    if max_bytes is not None and max_bytes > available:
        fail(
            2,
            f"the stated maximum announcement bytes ({max_bytes}) do not fit the "
            f"C-137 remainder: 62 bytes minus the stated shares (measurement "
            f"extension {ext_share}, checksum {cks_share}) leave {available}",
        )
        ac2_ok = False

    # per-announcement bytes derived from the container field table: fixed
    # fields' widths plus the payload's stated maximum, if any. The packet
    # -level count field is not part of each announcement and is skipped.
    if container_rows:
        fixed_bits = 0
        payload_max = 0
        derivable = True
        for row in container_rows:
            joined = " | ".join(row)
            if re.search(r"\bcount\b|how\s+many", joined, re.I):
                continue
            bits = width_to_bits(row[1]) if len(row) > 1 else None
            type_cell = row[2] if len(row) > 2 else ""
            fixed_type = bool(re.fullmatch(r"u\d+|i\d+|uint|f32|q1\.31|pad", type_cell.strip().lower()))
            if bits is None:
                derivable = False
                break
            if re.search(r"payload", joined, re.I) and not fixed_type:
                m = re.search(r"(\d+)\s*bytes?", row[4] if len(row) > 4 else joined, re.I)
                if m:
                    payload_max = max(payload_max, int(m.group(1)))
                else:
                    # a counted blob: its width cell is per element and the
                    # valid range states the element maximum
                    m = re.search(r"0\s+to\s+(\d+)", row[4] if len(row) > 4 else joined)
                    if m and bits == 8:
                        payload_max = max(payload_max, int(m.group(1)))
                    else:
                        derivable = False
                        break
            else:
                fixed_bits += bits
        if derivable:
            per_ann_bytes = fixed_bits // 8 + payload_max
            if max_bytes is not None and max_count is not None:
                if max_count * per_ann_bytes > max_bytes:
                    fail(
                        2,
                        f"the stated numbers are not self-consistent with the field "
                        f"tables: {max_count} announcements x {per_ann_bytes} bytes "
                        f"(container fields + payload maximum) = "
                        f"{max_count * per_ann_bytes} bytes, above the stated "
                        f"maximum of {max_bytes}",
                    )
                    ac2_ok = False
        else:
            warn(
                2,
                "the per-announcement size is not derivable from the container "
                "field table (a payload field without a stated byte maximum); the "
                "count-times-size consistency was not verified",
            )

    if ac2_ok:
        detail = ""
        if per_ann_bytes is not None:
            detail = (
                f"; self-consistent: {max_count} x {per_ann_bytes} bytes = "
                f"{max_count * per_ann_bytes} <= {max_bytes}"
            )
        print(
            f"PASS AC2: at most {max_count} announcements and at most {max_bytes} "
            f"announcement bytes per packet, inside the C-137 remainder "
            f"({available} bytes available of 62){detail}"
        )

# ---------------------------------------------------------------------------
# AC 3 — at least 6 invalid forms with outcomes; every reason a member;
#        unknown type within the same major is skippable, NOT a rejection
# ---------------------------------------------------------------------------
if not placeholder:
    ac3_ok = True

    FORMS = [
        (
            "unknown type within the same major (skippable, not a rejection)",
            r"unknown\s+type",
            "SKIP",
        ),
        (
            "length overrun",
            r"length[^.;\n]{0,60}(?:overrun|exceed|beyond|past|over\s+the)|overrun",
            "BadLength",
        ),
        (
            "named position not in the future",
            r"position[^.;\n]{0,80}(?:not\s+(?:in\s+the\s+)?future|in\s+the\s+past|already\s+passed|past)",
            "BadLength",
        ),
        ("duplicate type in one packet", r"duplicate", "MEMBER"),
        ("announcement in a gap slice (if forbidden)", r"gap", "GAP"),
        (
            "count above maximum",
            r"count[^.;\n]{0,60}(?:above|exceed|over|greater|more\s+than|larger)",
            "BadLength",
        ),
        (
            "announcement-present flag with a zero count",
            r"zero[\s-]+count|flag[^.;\n]{0,80}\bzero\b",
            "BadLength",
        ),
    ]

    forms_with_reason = 0
    for label, pattern, need in FORMS:
        hit = None
        hit_k = None
        for k, u in enumerate(units):
            if re.search(pattern, u, re.I):
                hit = u
                hit_k = k
                break
        if hit is None:
            fail(3, f"no invalid form listed for: {label}")
            ac3_ok = False
            continue
        window = "\n".join(units[max(0, hit_k - 1) : hit_k + 2])
        reasons_here = {
            name for name in MEMBERS if re.search(r"\b" + name + r"\b", hit)
        }

        if need == "SKIP":
            # encoded as what the ticket says: skippable by length, NOT a
            # rejection — no member reason may name it the outcome
            if not re.search(r"skip", hit, re.I):
                fail(
                    3,
                    "the unknown-type-within-same-major form does not state the "
                    "skippable outcome (skip by length): " + hit[:110],
                )
                ac3_ok = False
                continue
            if reasons_here:
                fail(
                    3,
                    "the unknown-type-within-same-major form is given a rejection "
                    "outcome (" + ", ".join(sorted(reasons_here)) + "); within the "
                    "same major an unknown type is SKIPPED by length, not rejected "
                    "(004-04's minor-change rule): " + hit[:110],
                )
                ac3_ok = False
                continue
            continue

        if need == "GAP":
            # the ticket says "(if forbidden)": either an outcome from the
            # enumeration, or an explicit statement that the case cannot occur
            if not reasons_here and not re.search(
                r"cannot|forbidden|impossible|never\s+carries|not\s+possible|no\s+announcement",
                hit + "\n" + window,
                re.I,
            ):
                fail(
                    3,
                    "the gap-slice form states no outcome (a member reason, or an "
                    "explicit statement that the case cannot occur): " + hit[:110],
                )
                ac3_ok = False
                continue
            if reasons_here:
                forms_with_reason += 1
            continue

        need_name = None if need == "MEMBER" else need
        if need_name is not None and need_name not in reasons_here:
            fail(
                3,
                f"the invalid form {label!r} must be rejected under {need_name}: "
                + hit[:110],
            )
            ac3_ok = False
            continue
        if not reasons_here:
            fail(
                3,
                f"the invalid form {label!r} carries no member of the closed "
                f"enumeration (expected one of {', '.join(MEMBERS)}): " + hit[:110],
            )
            ac3_ok = False
            continue
        forms_with_reason += 1

    # every reason-looking token in the section must be a member
    non_member_tokens = set()
    for u in units + blks:
        for m in re.finditer(r"\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b", u):
            token = m.group(1)
            if token not in member_set and not re.search(
                r"\b(?:no|not|never|without)\b[^.\n]{0,30}" + re.escape(token), u, re.I
            ):
                non_member_tokens.add(token)
    if non_member_tokens:
        fail(
            3,
            "the section names rejection reason(s) outside the closed "
            "enumeration: " + ", ".join(sorted(non_member_tokens)),
        )
        ac3_ok = False

    if forms_with_reason < 6:
        fail(
            3,
            f"only {forms_with_reason} invalid form(s) carry a member rejection "
            "reason or a stated resolution; at least 6 are required",
        )
        ac3_ok = False

    if ac3_ok:
        print(
            f"PASS AC3: {forms_with_reason} invalid forms with member reasons, "
            "unknown type within the same major encoded as skippable (not a "
            "rejection), no reason outside the closed enumeration"
        )

# ---------------------------------------------------------------------------
# AC 4 — C-43 and C-44 restated as sender rules
# ---------------------------------------------------------------------------
if not placeholder:
    ac4_ok = True

    c43_k = None
    for k, b in enumerate(blks):
        if re.search(r"C-43", b) and re.search(r"\b64\b", b):
            c43_k = k
            break
    if c43_k is None:
        fail(4, "C-43 is not restated (the lead: 64 slices, flat)")
        ac4_ok = False
    else:
        window = "\n".join(blks[max(0, c43_k - 1) : c43_k + 2])
        if not re.search(r"\b32\b", window) or not re.search(r"ms|millisecond", window, re.I):
            fail(
                4,
                "the C-43 restatement does not show the arithmetic (64 slices x "
                "0.5 ms = 32 ms at the C-02 defaults)",
            )
            ac4_ok = False
        if not re.search(r"sender|senders|sender's", window, re.I):
            fail(
                4,
                "the C-43 restatement is not phrased as a sender rule (obligation "
                "of the sender)",
            )
            ac4_ok = False

    c44_k = None
    for k, b in enumerate(blks):
        if re.search(r"C-44", b) and re.search(r"\b16\b", b):
            c44_k = k
            break
    if c44_k is None:
        fail(4, "C-44 is not restated (re-issued every 16 slices until the position passes)")
        ac4_ok = False
    else:
        window = "\n".join(blks[max(0, c44_k - 1) : c44_k + 2])
        if not re.search(r"member", window, re.I):
            fail(
                4,
                "the C-44 restatement does not name the every-member rule (re-issued "
                "in every bundle member, C-08)",
            )
            ac4_ok = False
        if not re.search(r"until|passes", window, re.I):
            fail(
                4,
                "the C-44 restatement does not state the duration (until the named "
                "position passes)",
            )
            ac4_ok = False
        if not re.search(r"sender|senders|sender's", window, re.I):
            fail(
                4,
                "the C-44 restatement is not phrased as a sender rule (obligation "
                "of the sender)",
            )
            ac4_ok = False

    if ac4_ok:
        print(
            "PASS AC4: C-43 (lead 64 slices, flat, 32 ms) and C-44 (every 16 "
            "slices, every bundle member, until the position passes) restated "
            "as sender rules"
        )

# ---------------------------------------------------------------------------
# AC 5 — open items with epic 005 as confirming owner and the falsifier;
#        the epoch-switch payload states whether it names the new session
# ---------------------------------------------------------------------------
if not placeholder:
    ac5_ok = True

    for label, pattern in (
        ("re-alignment", r"re[\s_-]*alignment"),
        ("transport reference change", r"transport[\s_-]*reference"),
    ):
        ok = False
        for k, b in enumerate(blks):
            if not re.search(pattern, b, re.I):
                continue
            window = "\n".join(blks[max(0, k - 1) : k + 2])
            if re.search(r"open\s+item", b + "\n" + window, re.I) and re.search(
                r"005", window
            ) and re.search(r"owner|confirm", window, re.I):
                ok = True
                break
        if not ok:
            fail(
                5,
                f"the proposed {label} payload is not recorded as an open item "
                "with epic 005 as the confirming owner",
            )
            ac5_ok = False

    if not re.search(r"falsif", section_prose, re.I):
        fail(5, "the open items state no falsifier")
        ac5_ok = False
    elif not re.search(
        r"cannot\s+act[^.;\n]{0,80}payload|payload\s+alone", section_prose, re.I
    ):
        fail(
            5,
            "the falsifier is not the ticket's (a receiver cannot act on the "
            "announcement from its payload alone)",
        )
        ac5_ok = False

    # epoch switch: whether the payload names the new session (C-19)
    epoch_ok = False
    for k, b in enumerate(blks):
        if not re.search(r"epoch[\s_-]*switch|epoch", b, re.I):
            continue
        window = b + "\n" + (blks[k + 1] if k + 1 < len(blks) else "")
        if re.search(r"C-19", window) and re.search(r"session", window, re.I) and re.search(
            r"names?\s+the\s+new\s+session|does\s+not\s+name|also\s+bumps?|bumps?\s+the\s+session",
            window,
            re.I,
        ):
            epoch_ok = True
            break
    if not epoch_ok:
        fail(
            5,
            "the epoch-switch payload does not state whether it names the new "
            "session (C-19: a timeline epoch change also bumps the session of "
            "every stream)",
        )
        ac5_ok = False

    if ac5_ok:
        print(
            "PASS AC5: re-alignment and transport-reference-change payloads are "
            "open items owned/confirmed by epic 005 with the payload-alone "
            "falsifier; the epoch-switch payload states its C-19 session rule"
        )

# ---------------------------------------------------------------------------
# Load wire.toml
# ---------------------------------------------------------------------------
document = None
tables_text = read_text(tables_path)
if tables_text is None:
    fail(6, f"{tables_path} does not exist")
else:
    try:
        document = parse_toml(tables_text)
    except TomlError as error:
        fail(6, f"{tables_path} could not be read by this check: {error}")
        document = None

# ---------------------------------------------------------------------------
# AC 6 — the machine-readable side. Accepted forms (the author chooses, this
# check documents what it accepts): the media packet [[message]] extended
# with announcement fields at offset >= 304 bits (byte 38), OR a separate
# [[message]] whose name contains "announcement" (a sub-table). Required in
# either: a count field with max_count equal to the stated maximum, a type
# field, a length field, a 64-bit position field; the carrying message's
# rejection_reasons include BadLength and name only enumeration members.
# ---------------------------------------------------------------------------
ann_fields = []  # (message, field)
carrying = []
if isinstance(document, dict):
    for msg in document.get("message", []) or []:
        if not isinstance(msg, dict):
            continue
        nm = norm_name(str(msg.get("name", "")))
        is_media = "mediapacket" in nm
        is_ann_msg = "announcement" in nm
        for f in msg.get("field", []) or []:
            if not isinstance(f, dict):
                continue
            off = f.get("offset")
            if not isinstance(off, int):
                continue
            if (is_media and off >= 38 * 8) or (is_ann_msg and off >= 0):
                ann_fields.append((msg, f))
                if id(msg) not in [id(m) for m in carrying]:
                    carrying.append(msg)

if not placeholder:
    ac6_ok = True
    if not ann_fields:
        fail(
            6,
            f"{tables_path} declares no announcement fields (neither media-packet "
            "fields at offset >= 304 bits nor an announcement sub-table message)",
        )
        ac6_ok = False
    else:
        by_kind = {}
        for msg, f in ann_fields:
            fname = str(f.get("name", "")).lower()
            w = f.get("width")
            if isinstance(w, int):
                if re.search(r"position", fname):
                    by_kind.setdefault("position", (msg, f))
                elif re.search(r"length", fname):
                    by_kind.setdefault("length", (msg, f))
                elif re.search(r"type", fname) and not re.search(r"type.?code.?table", fname):
                    by_kind.setdefault("type", (msg, f))
                elif re.search(r"count", fname):
                    by_kind.setdefault("count", (msg, f))

        for kind in ("type", "length", "position", "count"):
            if kind not in by_kind:
                fail(6, f"the announcement fields in {tables_path} have no {kind} field")
                ac6_ok = False

        if "position" in by_kind:
            msg, f = by_kind["position"]
            if f.get("width") != 64 or f.get("type") != "u64":
                fail(
                    6,
                    "the announcement position field in wire.toml is not a 64-bit "
                    "u64 (the slice header's position form)",
                )
                ac6_ok = False

        if "count" in by_kind:
            msg, f = by_kind["count"]
            # the fixed maximum may sit on the count field itself (a scalar
            # count with a stated maximum) or on the fields it counts
            mc = f.get("max_count")
            if not isinstance(mc, int) or isinstance(mc, bool):
                counted = [
                    f2.get("max_count")
                    for _, f2 in ann_fields
                    if f2.get("count") == f.get("name")
                    and isinstance(f2.get("max_count"), int)
                    and not isinstance(f2.get("max_count"), bool)
                ]
                mc = counted[0] if counted else None
            if mc is None:
                fail(
                    6,
                    "the announcement count field has no fixed maximum (every "
                    "counted list states one; this is how the maximum size is "
                    "computed)",
                )
                ac6_ok = False
            elif max_count is not None and mc != max_count:
                fail(
                    6,
                    f"the count field's maximum ({mc}) disagrees with the stated "
                    f"maximum number of announcements per packet ({max_count})",
                )
                ac6_ok = False

    for msg in carrying:
        reasons = msg.get("rejection_reasons")
        if not isinstance(reasons, list) or not reasons:
            fail(
                6,
                f"message {msg.get('name')!r}: no rejection_reasons declared for the "
                "announcement-carrying message",
            )
            ac6_ok = False
            continue
        if "BadLength" not in reasons:
            fail(
                6,
                f"message {msg.get('name')!r}: rejection_reasons do not include "
                "BadLength (length overrun and count-above-maximum are "
                "announcement rejections)",
            )
            ac6_ok = False
        for r in reasons:
            if r not in member_set:
                fail(
                    6,
                    f"message {msg.get('name')!r}: rejection reason {r!r} is not a "
                    "member of the closed enumeration",
                )
                ac6_ok = False
        max_size = msg.get("max_size")
        if not isinstance(max_size, int) or isinstance(max_size, bool):
            fail(6, f"message {msg.get('name')!r}: max_size is not an integer")
            ac6_ok = False
        elif "mediapacket" in norm_name(str(msg.get("name", ""))) and max_size <= 38:
            fail(
                6,
                f"message {msg.get('name')!r}: max_size {max_size} does not grow past "
                "the 38-byte packet + slice headers the announcements extend",
            )
            ac6_ok = False

    if ac6_ok:
        names = ", ".join(sorted({str(m.get("name")) for m in carrying}))
        print(
            f"PASS AC6: announcement fields declared in wire.toml (messages: "
            f"{names}), count/type/length/position present, reasons are "
            "enumeration members incl. BadLength"
        )

# ---------------------------------------------------------------------------
# DEFECT — the lead is a sender obligation on the FIRST issue (C-43); a
# re-issue (C-44) inside the lead is valid; a receiver rejects only a position
# that is not in the future. Keyword-level detectors over the section's rows,
# valid-range cells, prose sentences and wire.toml field strings.
# ---------------------------------------------------------------------------
LEAD_RE = re.compile(r"\blead|C-43|closer|nearer|inside|within|less\s+than\s+a\s+lead", re.I)
EXEMPT_RE = re.compile(r"\bfirst\b|re-?issu|C-44", re.I)
NEG_RE = re.compile(r"\b(?:not|never|no|nor|cannot|neither|none)\b|n't", re.I)
REJECT_RE = re.compile(
    r"BadLength|\bdrops?\b|\breject|\bdisagree|\binvalid|\bmalformed|\bcount\s+under", re.I
)
VALID_RE = re.compile(r"\bvalid\b", re.I)
AT_LEAST_RE = re.compile(r"\bat\s+least\b", re.I)
# the lead named in an invalid-form cell (the Validity-rules first column)
LEAD_FORM_RE = re.compile(
    r"\blead\b|C-43|\bcloser\b|\bnearer\b|64\s+slices|within\s+the\s+lead"
    r"|inside\s+the\s+lead|less\s+than\s+a\s+lead",
    re.I,
)
EVERY_ISSUE_RE_1 = re.compile(
    r"\b(?:lead|C-43)\b[^.;]{0,80}\bbinds?\b[^.;]{0,40}\b(?:every|each|all|any)\s+(?:re-?)?issues?\b",
    re.I,
)
EVERY_ISSUE_RE_2 = re.compile(
    r"\b(?:every|each|all|any)\s+(?:re-?)?issues?\b[^.;]{0,40}\b(?:bind|binds|bound|applies|apply|governs?)\b"
    r"[^.;]{0,40}\b(?:lead|C-43)\b",
    re.I,
)
# R6 stop rule: the position passes once the sender's slice position reaches it
R6_STOP_RE = re.compile(
    r"\b(?:no|nothing|none)\b[^.;]{0,80}\b(?:rides?|carr(?:y|ies|ied)|re-?issued?|sent|issued|goes?\s+out|travels?)\b"
    r"[^.;]{0,80}\bslices?\b[^.;]{0,60}\b(?:at\s+or\s+after|at\s+or\s+beyond|on\s+or\s+after)\b",
    re.I,
)
R6_REACH_RE = re.compile(
    r"\bslice\s+position\s+reach(?:es)?\b|\breach(?:es)?\s+(?:it|the\s+named\s+position|the\s+position)\b",
    re.I,
)
R6_PASS_RE = re.compile(r"\bpass(?:es|ed)?\b", re.I)
# a sender sentence that puts an announcement in the slice at the named position
IN_SLICE_AT_P_RE = re.compile(
    r"\bslices?\b[^.;]{0,40}?\b(?:whose\s+slice\s+position\s+(?:is|equals)|at|equal\s+to|equals)"
    r"\s+(?:the\s+)?(?:named\s+position|position\s+it\s+names)\b",
    re.I,
)
# a re-issue inside THE LEAD is valid/accepted, in one sentence (both orders)
_LEAD_SPAN = (
    r"(?:(?:inside|within)\s+(?:the|a|one|its)\s+lead\b"
    r"|(?:less\s+than\s+(?:64\s+slices|(?:one|a|the)\s+lead)|closer\s+than\s+(?:the|a|one)\s+lead)\b)"
)
REISSUE_VALID_RES = [
    re.compile(r"re-?issu\w*[^.;]{0,80}?\b" + _LEAD_SPAN + r"[^.;]{0,60}?\b(?:valid|accepted)\b", re.I),
    re.compile(r"re-?issu\w*[^.;]{0,60}?\b(?:valid|accepted)\b[^.;]{0,60}?" + _LEAD_SPAN, re.I),
]
RECEIVER_RE = re.compile(r"\breceivers?\b|\bconsumers?\b", re.I)
# receiver rule (any order, any distance within the sentence): a receiver
# sentence fails when it has an UNNEGATED reject-type verb clause AND an
# UNNEGATED lead phrase clause. Active or passive, no first/re-issue exemption.
REJECT_VERB_RES = [
    re.compile(r"\breject\w*", re.I),
    re.compile(r"\bdrop\w*", re.I),
    re.compile(r"\bdiscard\w*", re.I),
    re.compile(r"\bcount(?:s|ed|ing)?\s+(?:\S+\s+){0,3}?under\b", re.I),
    re.compile(r"\bBadLength\b"),
    re.compile(r"\bpolic\w*", re.I),
    re.compile(r"\benforc\w*", re.I),
    re.compile(r"\bcheck(?:s|ed|ing)?\b", re.I),
    re.compile(r"\bverif\w*", re.I),
    re.compile(r"\brequir(?:e|es|ed|ing)\b", re.I),
]
LEAD_PHRASE_RES = [
    re.compile(r"\blead\b", re.I),
    re.compile(r"\bcloser\s+than\b", re.I),
    re.compile(r"\bless\s+than\s+(?:one\s+lead|a\s+lead|the\s+lead|64\s+slices)\b", re.I),
    re.compile(r"\bC-43\b"),
]


def clause_around(s, a, b):
    """The clause (text between , ; :) containing the span a..b."""
    lo = max(s.rfind(c, 0, a) for c in ",;:") + 1
    ends = [i for i in (s.find(c, b) for c in ",;:") if i != -1]
    return s[lo : (min(ends) if ends else len(s))]


def every_issue_violation(s):
    """True when the lead is said to bind every/each/all issues, unnegated."""
    for rx in (EVERY_ISSUE_RE_1, EVERY_ISSUE_RE_2):
        for m in rx.finditer(s):
            q = re.search(r"\b(?:every|each|all|any)\b", m.group(0), re.I)
            if q is None:
                return True
            qa = m.start() + q.start()
            if not NEG_RE.search(clause_around(s, qa, qa + 1)):
                return True
    return False


def _unnegated_hit(s, rxs):
    for rx in rxs:
        for m in rx.finditer(s):
            if not NEG_RE.search(clause_around(s, m.start(), m.end())):
                return True
    return False


def receiver_polices_lead(s):
    """True when a receiver/consumer sentence has an unnegated reject-type verb
    (reject, drop, discard, count under, BadLength, police, enforce, check,
    verify, require) AND an unnegated lead phrase, anywhere in the sentence."""
    if not RECEIVER_RE.search(s):
        return False
    return _unnegated_hit(s, REJECT_VERB_RES) and _unnegated_hit(s, LEAD_PHRASE_RES)


def prose_sentences(block_pattern=None):
    """Sentences of the section's prose blocks (tables skipped), optionally only
    blocks matching block_pattern."""
    for b in blks:
        if b.lstrip().startswith("|"):
            continue
        if block_pattern and not re.search(block_pattern, b, re.I):
            continue
        for sent in re.split(r"(?<=[.;])\s+", b):
            yield sent.strip()


def lead_as_rejection(seg, cell=False):
    """True when a segment makes the lead a validity condition: it names the
    lead, is neither about the first issue or a re-issue nor negated, and
    either rejects, or calls the lead a condition of a valid announcement (a
    valid-range cell states the condition on the position directly)."""
    if not LEAD_RE.search(seg) or EXEMPT_RE.search(seg) or NEG_RE.search(seg):
        return False
    if REJECT_RE.search(seg):
        return True
    if cell:
        return bool(VALID_RE.search(seg) or AT_LEAST_RE.search(seg))
    return bool(VALID_RE.search(seg) and AT_LEAST_RE.search(seg))


if not placeholder:
    defect_ok = True

    for _, header, rows in all_tables:
        for row in rows:
            if is_field_table(header):
                for idx, cell in enumerate(row):
                    if lead_as_rejection(cell, cell=(idx == 4)):
                        fail("defect", "a field-table cell makes the lead a validity condition: " + cell[:110])
                        defect_ok = False
            else:
                # no exemption: a row whose invalid-form cell names the lead
                # and whose outcome rejects is a validity rule, whatever else
                # the row says (first issue, re-issue, C-44, negation)
                if (
                    len(row) >= 2
                    and REJECT_RE.search(" | ".join(row[1:]))
                    and any(
                        not NEG_RE.search(clause_around(row[0], m.start(), m.end()))
                        for m in LEAD_FORM_RE.finditer(row[0])
                    )
                ):
                    fail(
                        "defect",
                        "a Validity-rules row rejects an announcement for sitting inside "
                        "the lead (no exemption): " + " | ".join(row)[:110],
                    )
                    defect_ok = False
                if lead_as_rejection(" | ".join(row)):
                    fail("defect", "a table row rejects or invalidates a position for the lead: " + " | ".join(row)[:110])
                    defect_ok = False

    for b in blks:
        if b.lstrip().startswith("|"):
            continue
        for sent in re.split(r"(?<=[.;])\s+", b):
            if lead_as_rejection(sent):
                fail("defect", "prose makes the lead a validity condition: " + sent.strip()[:110])
                defect_ok = False
    for line in section_prose.splitlines():
        s = line.strip()
        if re.match(r"^[-*]\s+", s) or re.match(r"^\d+[.)]\s+", s):
            if lead_as_rejection(s):
                fail("defect", "a list item makes the lead a validity condition: " + s[:110])
                defect_ok = False

    for msg, f in ann_fields:
        for key in ("valid", "meaning"):
            txt = f.get(key)
            if isinstance(txt, str) and lead_as_rejection(txt, cell=(key == "valid")):
                fail(
                    "defect",
                    f"wire.toml field {f.get('name')!r} ({key}) makes the lead a "
                    "validity condition: " + txt[:110],
                )
                defect_ok = False

    # one sentence must say that a re-issue inside THE LEAD is valid or
    # accepted: the sentence names the lead itself (inside/within the lead,
    # less than 64 slices or one lead, closer than the lead), and valid or
    # accepted governs the re-issue. Bare "inside" or "C-43" does not count.
    if not any(
        any(rx.search(s) for rx in REISSUE_VALID_RES) for s in prose_sentences()
    ):
        fail(
            "defect",
            "no statement that a re-issue (C-44) inside the lead is valid (the "
            "sentence must name the lead itself); the receiver rejects only a "
            "position that is not in the future",
        )
        defect_ok = False

    # the lead binds the FIRST issue only: no sentence may say it binds every,
    # each or all issues (or re-issues), in either word order. A negation inside
    # the matched span ("binds the first issue only and never each re-issue")
    # exempts it.
    for s in prose_sentences():
        if every_issue_violation(s):
            fail(
                "defect",
                "the lead is stated to bind every issue of a change (C-43 binds the "
                "sender's FIRST issue only): " + s[:110],
            )
            defect_ok = False
            break

    # a receiver never polices the lead: no receiver sentence may say that it
    # polices, enforces, checks, verifies or requires the lead without a negation
    for s in prose_sentences():
        if receiver_polices_lead(s):
            fail(
                "defect",
                "a receiver is stated to police or reject for the lead (receivers never "
                "do; they reject only a position that is not in the future): " + s[:110],
            )
            defect_ok = False
            break

    # a sender sentence may not place an announcement, or the last re-issue, in
    # the slice at the named position (the position is passed by then)
    for s in prose_sentences(r"^\s*\**Sender obligations"):
        if any(
            not NEG_RE.search(clause_around(s, m.start(), m.end()))
            for m in IN_SLICE_AT_P_RE.finditer(s)
        ):
            fail(
                "defect R6",
                "a sender sentence places an announcement in the slice at the named "
                "position (no announcement rides in a slice at or after it): " + s[:110],
            )
            defect_ok = False
            break

    # R6 (ruling): the position has passed once the sender's slice position
    # reaches it; no announcement rides in a slice at or after the position it
    # names. Must be a sentence of the Sender obligations text.
    r6_ok = False
    for s in prose_sentences(r"^\s*\**Sender obligations"):
        if not re.search(r"sender|re-?issu|carr|ride", s, re.I):
            continue
        if R6_STOP_RE.search(s) or (R6_REACH_RE.search(s) and R6_PASS_RE.search(s)):
            r6_ok = True
            break
    if not r6_ok:
        fail(
            "defect R6",
            "the Sender obligations text does not state the stop rule at the named "
            "position: the position passes once the sender's slice position reaches "
            "it, so no announcement rides in a slice at or after the position it names",
        )
        defect_ok = False

    if not any(
        re.search(r"\bfirst\b", s, re.I)
        and re.search(r"\blead|C-43|64\s+slices", s, re.I)
        and re.search(r"sender", s, re.I)
        for b in blks
        if not b.lstrip().startswith("|")
        for s in re.split(r"(?<=[.;])\s+", b)
    ):
        fail(
            "defect",
            "the lead (C-43) is not stated as the sender's obligation on the FIRST issue",
        )
        defect_ok = False

    if defect_ok:
        print(
            "PASS defect: the lead binds the sender's first issue (C-43); a re-issue "
            "inside the lead is valid (C-44); no row, cell, sentence or wire.toml "
            "string rejects a position for being inside the lead"
        )

# ---------------------------------------------------------------------------
# DoD — evidence tags on the new section; the 004-02 checker and friends
# ---------------------------------------------------------------------------
if not placeholder:
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
            "this block of the Announcements section carries no evidence tag "
            "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
            + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
        )
        prev_tagged = False

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
            fail(6, f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(6, f"could not run cargo run --bin wire-check: {error}")

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
            m = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result)
            if not m:
                fail(6, f"could not read the test counts from: {result}")
            elif (int(m.group(1)), int(m.group(2)), int(m.group(3))) != (12, 0, 0):
                fail(6, "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: " + result)

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

print("OK: 6/6 acceptance criteria for 004-09, sender rules restated, open items owned, DoD tags present")
PY
