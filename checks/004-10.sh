#!/usr/bin/env bash
# 004-10 — media measurement extension layout.
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
#   AC 1  Three fields, each a row of the section's field table with a width
#         (whole bytes, matching its u8/u16/u32/u64 type), a unit and a
#         saturation rule ("saturate", never wrap) in its own row:
#         hop count (bounded by a stated maximum <= 64, C-34), accumulated
#         residence time (its unit named, ns or samples, tagged [decision]),
#         accumulated declared delay (whole samples at the domain rate:
#         C-105, C-01). The forwarding update rule (increment the hop count,
#         add the residence time, add the declared delay, saturating) is
#         stated. The open item on the widths is recorded with epic 004 as
#         owner and the ticket's falsifier (the declared-delay field
#         saturates over the maximum hop count at the largest L a manifest
#         admits).
#   AC 2  The extension's byte size is stated as a NUMBER and equals the sum
#         of its field widths, and the fields are contiguous. The C-137
#         arithmetic is shown: 100 - 38 = 62 remain after the headers, a
#         maximal announcement container takes 49 (1 + 4 x 12), 62 - 49 = 13
#         remain for the extension and the checksum together. When the
#         extension fits (<= 13 bytes), the checksum remainder the section
#         states must equal 13 - extension bytes, and the checksum share is
#         the one 004-13 states; when it does not fit, the omission on the
#         fallback path and its condition must be stated.
#   AC 3  At least 3 invalid forms with outcomes that are members of the
#         closed 13-name enumeration: the flag set with the bytes missing
#         (TooShort or BadLength), hop count beyond the maximum (BadLength),
#         reserved bits set (ReservedNonZero). No reason outside the
#         enumeration is named.
#   AC 4  A statement that no field identifies a receiver or destination
#         (rule 8), in one block.
#   AC 5  wire.toml carries the extension's three fields, agreeing with
#         wire.md field by field (width, type, offset relative to the
#         extension's first byte). Either form is accepted: the fields in the
#         media packet [[message]] inside the extension window, [container
#         end, container end + extension bits), where the container's maximum
#         end is bit 696 = byte 87 = 38 + 1 + 4 x 12 and the extension bits are
#         the sum of its widths in wire.md; fields after the window (004-13's
#         checksum, a later ticket's fields) are not extension fields and are
#         ignored. Or a [[message]] whose name contains "measurement", with
#         section_of = "media packet" and offsets from 0; its own max_size
#         equals the extension's bytes. The media packet max_size is an
#         invariant, not a snapshot: it is >= the container's maximum end plus
#         the extension's bytes (the extension fits) and <= 100, the C-137
#         header cap that later tickets fill (004-13 adds the checksum); in the
#         section form the media packet max_size gets the same bounds. The
#         carrying message's rejection_reasons are enumeration members and
#         include BadLength and ReservedNonZero. `cargo run --bin wire-check`
#         passes and `cargo test -p hadronyche-tools --test wire_tables` stays
#         12/0/0.
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
# Load wire.md, find the Measurement extension section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    for ac in range(1, 6):
        print(f"FAIL AC{ac}: {wire_path} does not exist")
    print("FAIL DoD: cannot check — wire.md does not exist")
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.fullmatch(r"measurement[\s-]+extensions?", title, re.I):
        sec_i = i
        break

PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)
placeholder = sec_i is None
section = ""
section_prose = ""
if sec_i is None:
    for ac in range(1, 5):
        fail(ac, "no 'Measurement extension' section heading in wire.md")
    fail("DoD", "no 'Measurement extension' section heading in wire.md")
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
        for ac in range(1, 5):
            fail(ac, "the Measurement extension section is still the 004-10 placeholder")
        fail("DoD", "the Measurement extension section is still the 004-10 placeholder")

blks = blocks(section_prose)
prose_blks = [b for b in blks if not b.lstrip().startswith("|")]


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
# The extension's field rows: one per field, found by keyword in the Meaning
# cell of a five-column field table (the table has no Name column).
# ---------------------------------------------------------------------------
KEYS = [
    ("hop", r"\bhop\b", "hop count"),
    ("residence", r"residence", "accumulated residence time"),
    ("delay", r"\bdelay\b", "accumulated declared delay"),
]
ext = {}
field_table_found = False
for tstart, header, rows in parse_tables(section):
    if not is_field_table(header):
        continue
    field_table_found = True
    for row in rows:
        if len(row) < 4:
            continue
        for key, pat, _ in KEYS:
            if key not in ext and re.search(pat, row[3], re.I):
                m = re.search(r"\d+", row[0])
                ext[key] = {
                    "cells": row,
                    "text": " | ".join(row),
                    "off": int(m.group(0)) if m else None,
                    "bits": width_to_bits(row[1]) if len(row) > 1 else None,
                    "type": row[2].strip().lower() if len(row) > 2 else "",
                    "valid": row[4] if len(row) > 4 else "",
                }
                break

if not placeholder and not field_table_found:
    fail(1, "no field table (Offset/Width/Type/Meaning/Valid range) in the Measurement extension section")

UNIT_RE = (
    r"(?<![A-Za-z])(ns|nanoseconds?|µs|us|microseconds?|ms|milliseconds?"
    r"|samples?\s+at\s+(?:the\s+)?(?:domain\s+rate|96))(?![A-Za-z])"
)

# ---------------------------------------------------------------------------
# AC 1 — three fields with width, unit and saturation; the declared delay in
#        whole samples (C-105); the forwarding rule; the open item and falsifier
# ---------------------------------------------------------------------------
if not placeholder:
    ac1_ok = True
    for key, _, label in KEYS:
        if key not in ext:
            fail(1, f"no field-table row for the {label} field (hop count, accumulated residence time, accumulated declared delay)")
            ac1_ok = False
            continue
        e = ext[key]
        bits = e["bits"]
        if bits is None or bits <= 0 or bits % 8 != 0:
            fail(1, f"the {label} field's width is not a whole number of bytes: {e['cells'][1]!r}")
            ac1_ok = False
        tm = re.fullmatch(r"u(\d+)", e["type"])
        if not tm:
            fail(1, f"the {label} field's type is not a fixed-width unsigned integer (u8, u16, u32, u64): {e['type']!r}")
            ac1_ok = False
        elif bits is not None and int(tm.group(1)) != bits:
            fail(1, f"the {label} field's width ({bits} bits) disagrees with its type {e['type']}")
            ac1_ok = False
        if not re.search(r"saturat", e["text"], re.I) or not re.search(r"wrap", e["text"], re.I):
            fail(1, f"the {label} field states no saturation rule (saturate, never wrap) in its own row")
            ac1_ok = False

    if "hop" in ext:
        nums = [int(x) for x in re.findall(r"\d+", ext["hop"]["valid"])]
        if not nums:
            fail(1, "the hop count's valid range states no maximum")
            ac1_ok = False
        else:
            max_hops = max(nums)
            if max_hops > 64:
                fail(1, f"the hop count's maximum ({max_hops}) exceeds the 64 modules of C-34")
                ac1_ok = False
            if ext["hop"]["bits"] is not None and max_hops > 2 ** ext["hop"]["bits"] - 1:
                fail(1, f"the hop count's maximum ({max_hops}) does not fit its width")
                ac1_ok = False

    if "residence" in ext:
        res_texts = [ext["residence"]["text"]] + [
            b for b in prose_blks if re.search(r"residence", b, re.I)
        ]
        if not any(re.search(UNIT_RE, t, re.I) and re.search(r"\[decision\]", t, re.I) for t in res_texts):
            fail(
                1,
                "the residence-time unit is not stated (ns or samples at the domain "
                "rate) and tagged [decision] in the same row or block",
            )
            ac1_ok = False

    if "delay" in ext:
        dl_texts = [ext["delay"]["text"]] + [
            b for b in prose_blks if re.search(r"delay", b, re.I)
        ]
        if not any(
            re.search(r"\bsamples?\b", t, re.I)
            and re.search(r"96[\s\u00a0_]?000|domain\s+rate", t, re.I)
            and re.search(r"C-105", t)
            and re.search(r"\bwhole\b|\binteger\b", t, re.I)
            for t in dl_texts
        ):
            fail(
                1,
                "the accumulated declared delay is not stated as a whole-sample sum "
                "at the domain rate (samples, 96 000 Hz, whole, C-105)",
            )
            ac1_ok = False

    if not any(
        re.search(r"forward", b, re.I)
        and re.search(r"hop", b, re.I)
        and re.search(r"residence", b, re.I)
        and re.search(r"delay", b, re.I)
        and re.search(r"saturat", b, re.I)
        and re.search(r"increment|\badds?\b|\+\s*1", b, re.I)
        for b in prose_blks
    ):
        fail(
            1,
            "no forwarding update rule (each processor increments the hop count, "
            "adds its residence time and its declared delay, saturating, never wrapping)",
        )
        ac1_ok = False

    if not any(
        re.search(r"open\s+item", b, re.I)
        and re.search(r"epic\s+004", b, re.I)
        and re.search(r"width", b, re.I)
        and re.search(r"falsif", b, re.I)
        for b in prose_blks
    ):
        fail(1, "the open item on the widths is not recorded with epic 004 as owner and a falsifier")
        ac1_ok = False

    if not any(
        re.search(r"saturat[^.;\n]{0,160}?maximum\s+hop|maximum\s+hop[^.;\n]{0,160}?saturat", b, re.I)
        and re.search(r"largest\s+L\b", b)
        and re.search(r"manifest", b, re.I)
        for b in prose_blks
    ):
        fail(
            1,
            "the falsifier is not the ticket's (the declared-delay field saturates "
            "over the maximum hop count at the largest L a manifest admits)",
        )
        ac1_ok = False

    if ac1_ok:
        print(
            "PASS AC1: hop count (u8, maximum 64, C-34), accumulated residence time "
            "(u32, unit tagged [decision]) and accumulated declared delay (u32, whole "
            "samples at 96 000 Hz, C-105) each with a width and a saturation rule; "
            "forwarding update rule and the width open item (epic 004, falsifier) stated"
        )

# ---------------------------------------------------------------------------
# AC 2 — byte size stated as a number, equal to the field widths; contiguous;
#        the C-137 arithmetic and the checksum remainder
# ---------------------------------------------------------------------------
ext_stated = None
if not placeholder:
    ac2_ok = True
    m = re.search(
        r"extension[^.;\n|]{0,60}?\b(\d+)\s*-?\s*bytes?\b"
        r"|\b(\d+)\s*-?\s*bytes?\s+(?:measurement\s+)?extension\b",
        section_prose,
        re.I,
    )
    if m:
        ext_stated = int(next(g for g in m.groups() if g))
    else:
        fail(2, "the extension's byte size is not stated as a number")
        ac2_ok = False

    if all(k in ext and ext[k]["bits"] for k in ("hop", "residence", "delay")):
        widths_sum = sum(ext[k]["bits"] for k in ("hop", "residence", "delay")) // 8
        if ext_stated is not None and ext_stated != widths_sum:
            fail(
                2,
                f"the stated extension size ({ext_stated} bytes) does not equal the sum "
                f"of its field widths ({widths_sum} bytes)",
            )
            ac2_ok = False
        ordered = sorted(
            ("hop", "residence", "delay"), key=lambda k: (ext[k]["off"] is None, ext[k]["off"] or 0)
        )
        if any(ext[k]["off"] is None for k in ordered):
            fail(2, "an extension field row states no offset")
            ac2_ok = False
        else:
            base = ext[ordered[0]]["off"]
            cum = 0
            for k in ordered:
                if ext[k]["off"] - base != cum:
                    fail(
                        2,
                        f"the extension fields are not contiguous: {k} at offset "
                        f"{ext[k]['off']} where {cum} bytes of the extension precede it",
                    )
                    ac2_ok = False
                    break
                cum += ext[k]["bits"] // 8
    else:
        ac2_ok = False

    budget_nums_ok = all(re.search(r"\b" + n + r"\b", section_prose) for n in ("100", "38", "62", "49", "13"))
    if not budget_nums_ok or not re.search(r"004-13", section_prose):
        fail(
            2,
            "the C-137 arithmetic is not shown (100-byte cap, 38 for the headers, 62 "
            "remaining, 49 for a maximal announcement container, 13 for the extension "
            "and the checksum together, the checksum share from 004-13)",
        )
        ac2_ok = False

    if ext_stated is not None:
        rem = 13 - ext_stated
        if ext_stated <= 13:
            if not any(
                re.search(r"checksum", b, re.I)
                and re.search(r"004-13", b)
                and re.search(r"\b" + str(rem) + r"\s*bytes?\b", b, re.I)
                for b in prose_blks
            ):
                fail(
                    2,
                    f"the checksum remainder is not stated: 13 - {ext_stated} = {rem} "
                    "bytes are left for the checksum (004-13)",
                )
                ac2_ok = False
            if not re.search(r"\bfits?\b|\bfitting\b", section_prose, re.I):
                fail(2, "the section does not say whether the extension fits beside a maximal announcement container")
                ac2_ok = False
        else:
            if not (
                re.search(r"fallback", section_prose, re.I)
                and re.search(r"omit", section_prose, re.I)
                and re.search(r"\bwhen\b|\bif\b", section_prose, re.I)
            ):
                fail(
                    2,
                    f"the extension ({ext_stated} bytes) does not fit beside a maximal "
                    "announcement container (13 bytes), and no omission on a fallback "
                    "path is stated, with its condition",
                )
                ac2_ok = False

    if ac2_ok:
        print(
            f"PASS AC2: extension stated as {ext_stated} bytes, equal to its field "
            f"widths and contiguous; C-137 arithmetic 100 - 38 = 62, 62 - 49 = 13 for "
            f"the extension and the checksum, checksum remainder {13 - (ext_stated or 0)} bytes"
        )

# ---------------------------------------------------------------------------
# AC 3 — at least 3 invalid forms with member outcomes
# ---------------------------------------------------------------------------
if not placeholder:
    ac3_ok = True
    vunits = []
    for tstart, header, rows in parse_tables(section):
        if is_field_table(header):
            continue
        for row in rows:
            vunits.append(" | ".join(row))
    for line in section_prose.splitlines():
        s = line.strip()
        if re.match(r"^[-*]\s+", s) or re.match(r"^\d+[.)]\s+", s):
            vunits.append(s)

    FORMS = [
        (
            "flag set but the extension's bytes missing",
            r"flag|presence|present|bit\s*4",
            r"missing|short|fewer|truncat|absent|runs?\s+past|not\s+fill|do(?:es)?\s+not\s+fit",
            {"TooShort", "BadLength"},
        ),
        (
            "hop count beyond the maximum",
            r"\bhop\b",
            r"beyond|above|exceed|greater|more\s+than|over\s+the|larger",
            {"BadLength"},
        ),
        (
            "reserved bits set",
            r"reserved",
            None,
            {"ReservedNonZero"},
        ),
    ]
    forms_with_reason = 0
    for label, pattern, extra, allowed in FORMS:
        hit = None
        for u in vunits:
            if re.search(pattern, u, re.I) and (extra is None or re.search(extra, u, re.I)):
                hit = u
                break
        if hit is None:
            fail(3, f"no invalid form listed for: {label}")
            ac3_ok = False
            continue
        reasons_here = {name for name in MEMBERS if re.search(r"\b" + name + r"\b", hit)}
        if not reasons_here:
            fail(3, f"the invalid form {label!r} carries no member of the closed enumeration: {hit[:110]}")
            ac3_ok = False
            continue
        if not (reasons_here & allowed):
            fail(
                3,
                f"the invalid form {label!r} must be rejected under "
                f"{' or '.join(sorted(allowed))}: {hit[:110]}",
            )
            ac3_ok = False
            continue
        forms_with_reason += 1

    non_member_tokens = set()
    for u in vunits + prose_blks:
        for m in re.finditer(r"\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b", u):
            token = m.group(1)
            if token not in member_set and not re.search(
                r"\b(?:no|not|never|without)\b[^.\n]{0,30}" + re.escape(token), u, re.I
            ):
                non_member_tokens.add(token)
    if non_member_tokens:
        fail(
            3,
            "the section names rejection reason(s) outside the closed enumeration: "
            + ", ".join(sorted(non_member_tokens)),
        )
        ac3_ok = False

    if forms_with_reason < 3:
        fail(3, f"only {forms_with_reason} invalid form(s) carry a member outcome; at least 3 are required")
        ac3_ok = False

    if ac3_ok:
        print(
            f"PASS AC3: {forms_with_reason} invalid forms with member outcomes (flag "
            "set with bytes missing, hop count beyond the maximum, reserved bits set)"
        )

# ---------------------------------------------------------------------------
# AC 4 — no field identifies a receiver or destination (rule 8)
# ---------------------------------------------------------------------------
if not placeholder:
    ok4 = False
    for k, b in enumerate(prose_blks):
        window = b + "\n" + (prose_blks[k + 1] if k + 1 < len(prose_blks) else "")
        if (
            re.search(r"no\s+field|none\s+of\s+the\s+fields|no\s+field\s+of", window, re.I)
            and re.search(r"receiver|destination", window, re.I)
            and re.search(r"rule\s+8", window, re.I)
        ):
            ok4 = True
            break
    if not ok4:
        fail(4, "no statement that no field identifies a receiver or destination (rule 8)")
    else:
        print("PASS AC4: no field of the extension identifies a receiver or destination (rule 8)")

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
# AC 5 — the machine-readable side
# ---------------------------------------------------------------------------
messages = [m for m in (document.get("message", []) if isinstance(document, dict) else []) if isinstance(m, dict)]
media_msg = next((m for m in messages if "mediapacket" in norm_name(str(m.get("name", "")))), None)
meas_msg = next((m for m in messages if "measurement" in norm_name(str(m.get("name", "")))), None)

# The container's maximum end: the counted announcement region as declared
# (offset + width x max_count), 87 bytes in v1 = 38 + 1 + 4 x 12.
container_end_bits = None
if media_msg is not None:
    for f in media_msg.get("field", []) or []:
        if isinstance(f, dict) and isinstance(f.get("max_count"), int) and isinstance(f.get("offset"), int) and isinstance(f.get("width"), int):
            end_bits = f["offset"] + f["width"] * f["max_count"]
            if container_end_bits is None or end_bits > container_end_bits:
                container_end_bits = end_bits

ac5_ok = True
toml_ext = {}
form = None
def toml_key(name):
    n = norm_name(str(name))
    for key in ("hop", "residence", "delay"):
        if key in n:
            return key
    return None

# The extension's window in wire.toml: from the container's maximum end for
# the byte size wire.md states (the sum of its field widths). Fields after the
# window (004-13's checksum, a later ticket's fields) are not extension fields.
ext_window_bits = None
if all(k in ext and ext[k]["bits"] for k in ("hop", "residence", "delay")) and container_end_bits is not None:
    ext_window_bits = sum(ext[k]["bits"] for k in ("hop", "residence", "delay"))
if media_msg is not None and container_end_bits is not None and ext_window_bits is not None:
    for f in media_msg.get("field", []) or []:
        if not isinstance(f, dict) or not isinstance(f.get("offset"), int):
            continue
        key = toml_key(f.get("name", ""))
        in_window = container_end_bits <= f["offset"] < container_end_bits + ext_window_bits
        if key is not None and in_window and key not in toml_ext:
            toml_ext[key] = (media_msg, f)
            form = "media"
if meas_msg is not None and not toml_ext:
    for f in meas_msg.get("field", []) or []:
        if not isinstance(f, dict) or not isinstance(f.get("offset"), int):
            continue
        key = toml_key(f.get("name", ""))
        if key is not None and key not in toml_ext:
            toml_ext[key] = (meas_msg, f)
            form = "section"

if container_end_bits is None:
    fail(5, "wire.toml's media packet declares no counted announcement region to place the extension after")
    ac5_ok = False
if not toml_ext:
    fail(
        5,
        f"{tables_path} declares no measurement-extension fields (hop count, residence "
        "time, declared delay): neither media packet fields after the announcement "
        "container nor a measurement section message",
    )
    ac5_ok = False
else:
    for key, _, label in KEYS:
        if key not in toml_ext:
            fail(5, f"{tables_path} has no field for the {label}")
            ac5_ok = False
    if form == "section" and meas_msg.get("section_of") != "media packet":
        fail(5, "the measurement section message does not declare section_of = \"media packet\"")
        ac5_ok = False

    toml_bytes = 0
    for key in ("hop", "residence", "delay"):
        if key not in toml_ext:
            continue
        msg, f = toml_ext[key]
        if not isinstance(f.get("width"), int):
            fail(5, f"field {f.get('name')!r}: width is not an integer")
            ac5_ok = False
            continue
        toml_bytes += f["width"] // 8
        if f["width"] % 8:
            fail(5, f"field {f.get('name')!r}: width {f['width']} bits is not whole bytes")
            ac5_ok = False

    # agreement with wire.md: width, type and the offset relative to the
    # extension's first byte
    if all(k in ext for k in ("hop", "residence", "delay")) and all(k in toml_ext for k in ("hop", "residence", "delay")):
        base_off = min(toml_ext[k][1]["offset"] for k in toml_ext)
        if form == "media":
            base_off = container_end_bits
        md_base = min(ext[k]["off"] for k in ext if ext[k]["off"] is not None)
        for key, _, label in KEYS:
            msg, f = toml_ext[key]
            md_rel = (ext[key]["off"] - md_base) * 8
            toml_rel = f["offset"] - base_off
            if toml_rel != md_rel:
                fail(
                    5,
                    f"field {f.get('name')!r}: wire.toml places it {toml_rel} bits into the "
                    f"extension, wire.md {md_rel} bits",
                )
                ac5_ok = False
            if ext[key]["bits"] != f.get("width"):
                fail(
                    5,
                    f"field {f.get('name')!r}: wire.toml width {f.get('width')} bits vs "
                    f"wire.md width {ext[key]['bits']} bits",
                )
                ac5_ok = False
            if ext[key]["type"] != str(f.get("type", "")).lower():
                fail(
                    5,
                    f"field {f.get('name')!r}: wire.toml type {f.get('type')!r} vs "
                    f"wire.md type {ext[key]['type']!r}",
                )
                ac5_ok = False

    if ext_stated is not None and toml_bytes != ext_stated:
        fail(5, f"wire.toml's extension fields total {toml_bytes} bytes; wire.md states {ext_stated}")
        ac5_ok = False

    carrying = media_msg if form == "media" else meas_msg
    if carrying is not None and container_end_bits is not None:
        base_bytes = container_end_bits // 8
        max_size = carrying.get("max_size")
        # The invariant, not a snapshot: the media packet's declared maximum
        # holds the container's maximum end and the extension (it fits), and
        # stays inside the C-137 header cap of 100 bytes, which later tickets
        # fill with the checksum (004-13) and nothing else.
        HEADER_CAP = 100
        if not isinstance(max_size, int) or isinstance(max_size, bool):
            fail(5, f"message {carrying.get('name')!r}: max_size is not an integer")
            ac5_ok = False
        elif form == "section" and max_size != toml_bytes:
            fail(5, f"message {carrying.get('name')!r}: max_size {max_size} != extension bytes {toml_bytes}")
            ac5_ok = False
        elif form == "media" and max_size < base_bytes + toml_bytes:
            fail(
                5,
                f"media packet max_size {max_size} does not hold the container's maximum "
                f"end ({base_bytes} bytes) plus the extension ({toml_bytes} bytes)",
            )
            ac5_ok = False
        if form == "section":
            media_max = media_msg.get("max_size") if media_msg else None
            if not isinstance(media_max, int) or isinstance(media_max, bool) or media_max < base_bytes + toml_bytes:
                fail(
                    5,
                    f"media packet max_size {media_max} does not hold the container's "
                    f"maximum end plus the extension ({base_bytes + toml_bytes} bytes)",
                )
                ac5_ok = False
        if isinstance(max_size, int) and not isinstance(max_size, bool) and max_size > HEADER_CAP:
            fail(5, f"message {carrying.get('name')!r}: max_size {max_size} exceeds the C-137 header cap of {HEADER_CAP} bytes")
            ac5_ok = False
        if form == "section" and media_msg is not None:
            media_max = media_msg.get("max_size")
            if isinstance(media_max, int) and not isinstance(media_max, bool) and media_max > HEADER_CAP:
                fail(5, f"media packet max_size {media_max} exceeds the C-137 header cap of {HEADER_CAP} bytes")
                ac5_ok = False

    reasons = carrying.get("rejection_reasons") if carrying is not None else None
    if not isinstance(reasons, list) or not reasons:
        fail(5, "the extension-carrying message declares no rejection_reasons")
        ac5_ok = False
    else:
        for r in reasons:
            if r not in member_set:
                fail(5, f"rejection reason {r!r} is not a member of the closed enumeration")
                ac5_ok = False
        for need in ("BadLength", "ReservedNonZero"):
            if need not in reasons:
                fail(5, f"the extension-carrying message's rejection_reasons do not include {need}")
                ac5_ok = False

if ac5_ok:
    print(
        f"PASS AC5: wire.toml carries the extension's three fields ({form} form), "
        "agreeing with wire.md on offset, width and type; media max_size >= container "
        "end + extension bytes and <= 100 (C-137 cap); reasons are enumeration members"
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
            "this block of the Measurement extension section carries no evidence tag "
            "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
            + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
        )
        prev_tagged = False

if not overridden:
    # 1. The 004-02 checker, from the command line.
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

print("OK: 5/5 acceptance criteria for 004-10, extension sized and budgeted, open items owned, DoD tags present")
PY
