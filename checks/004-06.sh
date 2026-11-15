#!/usr/bin/env bash
# 004-06 — media packet header layout.
#
# RED-phase document check. It validates doc/design/spec/wire.md and
# doc/design/spec/wire.toml against every acceptance criterion of the ticket,
# and re-runs the 004-02 checker, the wire_tables tests, the design checker
# and checks/004-01.sh.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo, the design checker, the
# earlier document check) are skipped with a NOTE, because they always target
# the real repository.
#
# What is checked, per acceptance criterion:
#
#   AC 1  The "Media packet header" section's field table contains every
#         rule-1 field with the C-10/C-11/C-07 widths: magic 16 bits and
#         version 16 bits (from 004-04: magic 0x4d4e at offset 0, version
#         0x0100 at offset 2, major in the high byte), domain 16 bits
#         (C-11), stream number 16 bits, session 32 bits, timeline epoch
#         16 bits (C-10), copy index 2 bits (C-07), and a flags field whose
#         named bits include gap and degraded.
#   AC 2  A short note confirms field by field that the header contains NO
#         module id (rule 7), NO destination address (rule 7) and NO
#         per-receiver acknowledgement (rule 8); a spot check verifies that
#         no field of the table carries a destination-specific value.
#   AC 3  The header size in bytes is stated and every field offset is
#         given; the offsets are contiguous and non-overlapping (bit-level
#         where the sub-byte fields state their bits). The 004-02 checker
#         re-verifies the machine-readable side (repository level).
#   AC 4  At least 10 invalid forms are listed, each with a rejection reason
#         that is a member of the closed 13-name enumeration — or, for
#         VersionMismatch, the named event the enumeration gives it. The
#         ticket's named forms must each appear: shorter than header
#         (TooShort), bad magic (BadMagic), version mismatch
#         (VersionMismatch + event), foreign domain (ForeignDomain),
#         domain 0 (UnconfiguredDomain), unknown stream (UnknownStream),
#         unadmitted source (UnadmittedSource), wrong epoch (WrongEpoch),
#         reserved non-zero (ReservedNonZero), gap flag with non-empty
#         payload, and invalid flags combination.
#   AC 5  The session field's meaning cites C-19 and the degraded flag's
#         meaning cites C-122.
#   AC 6  wire.toml gains the media packet [[message]] table with its
#         [[message.field]] rows, agreeing with wire.md name-by-name and
#         offset-by-offset; max_size is stated; layer is media; dscp is 46
#         (C-13); every rejection_reasons entry is a member of the closed
#         enumeration; `cargo run --bin wire-check` passes and
#         `cargo test -p hadronyche-tools --test wire_tables` stays 12/0/0.
#
# Also required by the ticket and checked here: the don't-fragment statement
# (rule 11, C-100) — an IPv4 DF-bit sender obligation, outside the header.
#
# DoD also: every paragraph of the new section carries an evidence tag
# ([code], [measured], [kernel-doc], [standard], [memory], [decision]); table
# and code blocks may inherit the tag of the paragraph that introduces them.
# The design checker must print OK, and 004-01 must still pass.
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
# Markdown helpers (same conventions as checks/004-01.sh, 004-03.sh, 004-05.sh)
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
# A small TOML reader (the 004-03 subset parser, reused: the check must run
# offline, unprivileged, on the Python the other checks use).
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
# Load wire.md, find the Media packet header section and its field table
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    for ac in range(1, 7):
        print(f"FAIL AC{ac}: {wire_path} does not exist")
    print("FAIL DF: cannot check — wire.md does not exist")
    print("FAIL DoD: cannot check — wire.md does not exist")
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.search(r"media\s+packet\s+header", title, re.I):
        sec_i = i
        break

PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)
placeholder = sec_i is None
section = ""
section_prose = ""
if sec_i is None:
    fail(1, "no 'Media packet header' section heading in wire.md")
    fail(2, "no 'Media packet header' section heading in wire.md")
    fail(3, "no 'Media packet header' section heading in wire.md")
    fail(4, "no 'Media packet header' section heading in wire.md")
    fail(5, "no 'Media packet header' section heading in wire.md")
    fail(6, "no 'Media packet header' section in wire.md to agree with wire.toml")
    fail("DF", "no 'Media packet header' section heading in wire.md")
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
        fail(1, "the Media packet header section is still the 004-06 placeholder")
        fail(2, "the Media packet header section is still the 004-06 placeholder")
        fail(3, "the Media packet header section is still the 004-06 placeholder")
        fail(4, "the Media packet header section is still the 004-06 placeholder")
        fail(5, "the Media packet header section is still the 004-06 placeholder")
        fail("DF", "the Media packet header section is still the 004-06 placeholder")


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


field_rows = []  # (row_text, offset_bytes, width_bits, stated_bits)
field_table_found = False
field_table_start = -1
for tstart, header, rows in parse_tables(section):
    if not is_field_table(header) or len(rows) < 4:
        continue
    candidate = []
    ok = True
    for row in rows:
        text = " | ".join(row)
        off = parse_int_token(row[0]) if row else None
        bits = width_to_bits(row[1]) if len(row) > 1 else None
        if off is None or bits is None:
            ok = False
            break
        candidate.append((text, off, bits, bit_positions(text)))
    if ok and len(candidate) > len(field_rows):
        field_rows = candidate
        field_table_found = True
        field_table_start = tstart
        break

if sec_i is not None and not placeholder and not field_table_found:
    fail(1, "no field table (Offset/Width/Type/Meaning/Valid range) in the Media packet header section")
    fail(3, "no field table (Offset/Width/Type/Meaning/Valid range) in the Media packet header section")

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
    if len(found) >= 13:
        MEMBERS = sorted(found)
    elif len(found) >= 10:
        MEMBERS = sorted(found)

member_set = set(MEMBERS)

# ---------------------------------------------------------------------------
# AC 1 — every rule-1 field, with the exact widths and the 004-04 values
# ---------------------------------------------------------------------------
if field_table_found:
    REQUIRED = [
        ("magic", r"\bmagic\b", 16),
        ("version", r"\bversion\b", 16),
        ("domain", r"\bdomain\b", 16),
        ("stream number", r"stream\s*number|\bstream\b", 16),
        ("session", r"\bsession\b", 32),
        ("timeline epoch", r"\bepoch\b", 16),
        ("copy index", r"copy[\s_-]*index", 2),
        ("flags", r"\bflags?\b", None),
    ]
    claimed = [False] * len(field_rows)
    ac1_ok = True
    for label, pattern, need_bits in REQUIRED:
        hit = None
        # Prefer the most specific pattern first (stream number, not any stream).
        for row_i, (text, off, bits, stated) in enumerate(field_rows):
            if claimed[row_i]:
                continue
            if re.search(pattern, text, re.I):
                hit = row_i
                break
        if hit is None:
            fail(1, f"the header table has no {label} field")
            ac1_ok = False
            continue
        claimed[hit] = True
        text, off, bits, _ = field_rows[hit]
        if need_bits is not None and bits != need_bits:
            fail(
                1,
                f"{label}: the table gives {bits} bits, the rule-1/C-10/C-11/C-07 "
                f"width is {need_bits} bits",
            )
            ac1_ok = False
        if label == "magic" and off != 0:
            fail(1, f"the magic sits at offset {off}; offsets 0 and 2 are immovable (004-04)")
            ac1_ok = False
        if label == "version" and off != 2:
            fail(1, f"the version sits at offset {off}; offsets 0 and 2 are immovable (004-04)")
            ac1_ok = False
        if label == "flags" and bits is not None and bits < 8:
            fail(1, f"the flags field is {bits} bits; a flags field is at least one byte")
            ac1_ok = False

    # The magic and version values agree with 004-04.
    if not re.search(r"0x4d4e|4d\s*4e", section, re.I):
        fail(1, "the section does not state the magic value 0x4d4e fixed by 004-04")
        ac1_ok = False
    if not re.search(r"0x0100|\bv?1\.0\b", section):
        fail(1, "the section does not state the v1 version value 0x0100 (major 1 high byte, minor 0 low byte)")
        ac1_ok = False

    # The flags field names the gap and degraded bits (rule 1, rule 10).
    flags_text = ""
    for row_i, (text, off, bits, stated) in enumerate(field_rows):
        if claimed[row_i] and re.search(r"\bflags?\b", text, re.I):
            flags_text = text
    gap_named = bool(re.search(r"\bgap\b", flags_text, re.I)) or bool(
        re.search(r"flag[^.\n]{0,80}\bgap\b|\bgap\b[^.\n]{0,80}flag", section_prose, re.I)
    )
    degraded_named = bool(re.search(r"\bdegraded\b", flags_text, re.I)) or bool(
        re.search(r"flag[^.\n]{0,80}\bdegraded\b|\bdegraded\b[^.\n]{0,80}flag", section_prose, re.I)
    )
    if not gap_named:
        fail(1, "the flags field does not name the gap bit")
        ac1_ok = False
    if not degraded_named:
        fail(1, "the flags field does not name the degraded bit")
        ac1_ok = False
    if not re.search(r"announce?ment", section_prose, re.I):
        warn(1, "the section does not name presence bits for announcements (Work step 2; the acceptance criteria do not require it)")
    if not re.search(r"measurement\s+extension", section_prose, re.I):
        warn(1, "the section does not name presence bits for the measurement extension (Work step 2; the acceptance criteria do not require it)")
    if not re.search(r"reserved", section_prose, re.I):
        warn(1, "the section does not state that unused flag bits are reserved (Work step 2; the acceptance criteria do not require it)")

    if ac1_ok:
        print(
            "PASS AC1: magic 0x4d4e @0 and version 0x0100 @2 (004-04), domain 16, "
            "stream 16, session 32, epoch 16, copy index 2 bits, flags with gap "
            "and degraded"
        )
else:
    if sec_i is not None and not placeholder:
        pass  # already failed above

# ---------------------------------------------------------------------------
# AC 2 — the excluded-kinds note, field by field
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    EXCLUDED = [
        ("module id", r"module\s*id"),
        ("destination address", r"destination|dest\b"),
        ("per-receiver acknowledgement", r"acknowledg|per[\s-]receiver|\back\b"),
    ]
    NEGATION = re.compile(
        r"\bno\b|\bnot\b|\bnever\b|\bwithout\b|\bexcluded?\b|\babsen(?:t|ce)\b"
        r"|\bnone\b|\blacks?\b|\bomitted?\b|\bretired\b|\bcarries\s+neither",
        re.I,
    )
    ac2_ok = True
    for label, pattern in EXCLUDED:
        named = False
        for block in blocks(section_prose):
            if not re.search(pattern, block, re.I):
                continue
            # The kind must be excluded: negation in the same block, or in the
            # sentence around the match.
            window = re.search(pattern, block, re.I)
            around = block[max(0, window.start() - 120) : window.end() + 160]
            if NEGATION.search(around) or NEGATION.search(block):
                named = True
                break
        if not named:
            fail(
                2,
                f"no note states the header carries no {label} (rules 7 and 8); "
                "the excluded kinds must each be named with its absence",
            )
            ac2_ok = False
    if not re.search(
        r"field\s+by\s+field|each\s+field|every\s+field|per\s+field|field\s+list"
        r"|field\s+of\s+the\s+table|each\s+entry",
        section_prose,
        re.I,
    ):
        fail(
            2,
            "the excluded-kinds note is not field by field (the ticket asks for a "
            "field-by-field confirmation: 'each field', 'field by field', or a "
            "walk over the field list)",
        )
        ac2_ok = False

    # Spot check: no field of the table is destination-specific.
    for text, off, bits, stated in field_rows:
        for pattern in (
            r"\bmodule\s*id\b",
            r"destination",
            r"\back\b|acknowledg",
            r"receiver[\s-]specific|per[\s-]receiver\s+field",
        ):
            m = re.search(pattern, text, re.I)
            if m:
                around = text[max(0, m.start() - 60) : m.end() + 80]
                if not NEGATION.search(around):
                    fail(
                        2,
                        "the header table carries a destination-specific field, "
                        f"contradicting the note: {text[:110]}",
                    )
                    ac2_ok = False
    if ac2_ok:
        print(
            "PASS AC2: the note excludes module id, destination address and "
            "per-receiver acknowledgement, field by field; no table field is "
            "destination-specific"
        )

# ---------------------------------------------------------------------------
# AC 3 — the size in bytes is stated and the offsets tile the header
# ---------------------------------------------------------------------------
if field_table_found:
    # Bit intervals: exact for byte fields and for sub-byte fields that state
    # their bits; unstated sub-byte fields degrade conflicts to NOTEs, because
    # the 004-02 checker verifies the bitwise layout on the machine-readable
    # side.
    intervals = []
    unstated_subbyte = False
    for text, off, bits, stated in field_rows:
        if bits < 8:
            if stated:
                lo, hi = stated
                if bits != hi - lo + 1:
                    fail(
                        3,
                        f"sub-byte field states bits {hi}..{lo} ({hi - lo + 1} bits) "
                        f"but its width is {bits} bits: {text[:90]}",
                    )
                start = off * 8 + lo
            else:
                start = off * 8
                unstated_subbyte = True
        else:
            if bits % 8 != 0:
                fail(3, f"a field of {bits} bits is neither whole bytes nor stated bits: {text[:90]}")
            start = off * 8
        intervals.append((start, start + bits, text, bits < 8 and not stated))

    ac3_ok = True
    intervals.sort()
    cursor = 0
    for start, end, text, approx in intervals:
        if start > cursor:
            gap = start - cursor
            msg = (
                f"undeclared gap of {gap} bits before {text.split('|')[3].strip()[:60]!r}"
                if len(text.split("|")) > 3
                else f"undeclared gap of {gap} bits at bit {cursor}"
            )
            if gap < 8 and unstated_subbyte:
                warn(3, msg + " (bit positions not stated; the 004-02 checker verifies the toml side)")
            else:
                fail(3, msg + "; the offsets must be contiguous")
                ac3_ok = False
        elif start < cursor:
            if approx:
                warn(3, "possible overlap at bit " + str(start) + " (a sub-byte field does not state its bits; cannot verify bitwise here)")
            else:
                fail(3, f"overlapping fields at bit {start}: {text[:90]}")
                ac3_ok = False
        cursor = max(cursor, end)
    header_bits = cursor
    if header_bits % 8 != 0:
        if unstated_subbyte:
            warn(3, f"the fields end mid-byte at bit {header_bits} (positions not stated; cannot verify)")
        else:
            fail(3, f"the fields end at bit {header_bits}, not a whole byte boundary")
            ac3_ok = False
    header_bytes = (header_bits + 7) // 8

    stated_sizes = {int(x.replace(" ", "").replace("_", "")) for x in re.findall(r"(\d[\d _]*)\s*bytes", section_prose, re.I)}
    stated_sizes |= {int(x.replace(" ", "").replace("_", "")) for x in re.findall(r"(\d[\d _]*)\s*B\b", section_prose)}
    if header_bits % 8 == 0:
        if header_bytes not in stated_sizes:
            fail(
                3,
                f"the table's fields total {header_bytes} bytes, but the section "
                "never states that size in bytes (AC 3: the header size in bytes "
                "is stated)",
            )
            ac3_ok = False
    elif not stated_sizes:
        fail(3, "the section states no header size in bytes")
        ac3_ok = False

    if ac3_ok and header_bits % 8 == 0:
        print(
            f"PASS AC3: size {header_bytes} bytes stated and every offset given; "
            "the offsets are contiguous and non-overlapping"
        )
else:
    if sec_i is not None and not placeholder:
        pass  # already failed above

# ---------------------------------------------------------------------------
# AC 4 — at least 10 invalid forms, each with a member reason (or the
#         VersionMismatch named event)
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    # Candidate units: table rows and bullet lines of the section. The field
    # table's own rows are excluded: its Valid range column explains what a
    # receiver accepts, and a note like "domain 0 = unconfigured" there is not
    # the list of invalid forms with their outcomes that Work step 3 asks for.
    units = []
    for tstart, _, rows in parse_tables(section):
        if field_table_found and tstart == field_table_start:
            continue
        for row in rows:
            units.append(" | ".join(row))
    for line in section_prose.splitlines():
        s = line.strip()
        if re.match(r"^[-*]\s+", s) or re.match(r"^\d+[.)]\s+", s):
            units.append(s)

    FORMS = [
        ("shorter than the header (TooShort)", r"shorter\s+than\s+the\s+header|shorter\s+than|too\s+short|fewer\s+bytes\s+than|less\s+than\s+the\s+header|below\s+the\s+header\s+size"),
        ("bad magic (BadMagic)", r"bad\s+magic|wrong\s+magic|magic\s+mismatch|magic\s+(?:is\s+)?(?:not|wrong)|incorrect\s+magic|magic\s+does\s+not\s+match|magic\s+differs"),
        ("version mismatch (VersionMismatch)", r"version\s+mismatch|wrong\s+version|version\s+(?:is\s+)?(?:not|other\s+than|differs)|different\s+major|other\s+major|major\s+differs"),
        ("foreign domain (ForeignDomain)", r"foreign\s+domain|domain[^.\n]{0,30}(?:≠|!=|is\s+not|other\s+than|differs\s+from)\s*(?:the\s+)?local|different\s+(?:non-?zero\s+)?domain|another\s+domain|other\s+than\s+the\s+local\s+domain|non-?local\s+domain"),
        ("domain 0 (UnconfiguredDomain)", r"domain\s*(?:id\s*)?(?:is\s*)?(?:=|is)?\s*0\b|domain\s+zero|unconfigured\s+domain|domain\s*=\s*0\b"),
        ("unknown stream (UnknownStream)", r"unknown\s+stream|stream\s+(?:number\s+)?(?:is\s+)?unknown|no\s+(?:such\s+)?stream|stream\s+[^.\n]{0,30}unknown|not\s+a\s+known\s+stream"),
        ("unadmitted source (UnadmittedSource)", r"unadmitted\s+source|source\s+not\s+admitted|not\s+admitted|no\s+admitted\s+connection|source\s+is\s+not\s+admitted"),
        ("wrong epoch (WrongEpoch)", r"wrong\s+epoch|epoch\s+(?:is\s+)?not\s+current|other\s+epoch|epoch\s+mismatch|stale\s+epoch|epoch\s+differs|not\s+the\s+current\s+epoch"),
        ("reserved non-zero (ReservedNonZero)", r"reserved[^.\n]{0,40}non-?zero|non-?zero[^.\n]{0,20}reserved|reserved\s+bits?[^.\n]{0,25}(?:not\s+zero|set|nonzero)|reserved[^.\n]{0,20}\bset\b"),
        ("gap flag with non-empty payload", r"gap[^.\n]{0,60}payload|payload[^.\n]{0,60}gap"),
        ("invalid flags combination", r"flag[^.\n]{0,60}combin|combin\w+[^.\n]{0,60}flag|invalid\s+flags?|flags\s+combination|inconsistent\s+flags|illegal\s+flag|forbidden\s+flag"),
    ]

    ac4_ok = True
    forms_with_reason = 0
    non_member_tokens = set()
    for label, pattern in FORMS:
        hit = None
        for unit in units:
            if re.search(pattern, unit, re.I):
                hit = unit
                break
        if hit is None:
            fail(4, f"no invalid form listed for: {label}")
            ac4_ok = False
            continue
        reasons_here = {name for name in MEMBERS if re.search(r"\b" + name + r"\b", hit)}
        event_here = re.search(r"named\s+event|\bevent\b", hit, re.I)
        if not reasons_here and not (event_here and label.startswith("version")):
            fail(
                4,
                f"the invalid form {label!r} carries no member of the closed "
                f"enumeration (expected one of {', '.join(MEMBERS)}): {hit[:110]}",
            )
            ac4_ok = False
            continue
        if reasons_here:
            forms_with_reason += 1
        if label.startswith("version mismatch"):
            if not event_here:
                fail(
                    4,
                    "VersionMismatch is listed without its named event (004-03: a "
                    "version mismatch is never a silent drop; it raises the "
                    "VersionMismatch event in addition to being counted)",
                )
                ac4_ok = False
        if label.startswith(("gap flag", "invalid flags")) and not reasons_here:
            fail(
                4,
                f"the invalid form {label!r} is named by the ticket but carries "
                f"neither a member reason nor a named event: {hit[:110]}",
            )
            ac4_ok = False

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

    if forms_with_reason < 10:
        fail(
            4,
            f"only {forms_with_reason} invalid form(s) carry a member rejection "
            "reason; at least 10 are required",
        )
        ac4_ok = False

    if ac4_ok:
        print(
            f"PASS AC4: {forms_with_reason} invalid forms with member reasons, "
            "including every form the ticket names; VersionMismatch keeps its "
            "named event"
        )

# ---------------------------------------------------------------------------
# AC 5 — the session meaning cites C-19, the degraded flag cites C-122
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder and field_table_found:
    ac5_ok = True
    session_row = None
    for text, off, bits, stated in field_rows:
        if re.search(r"\bsession\b", text, re.I):
            session_row = text
            break
    if session_row is None or not re.search(r"C-19", session_row):
        fail(5, "the session field's meaning does not cite C-19")
        ac5_ok = False
    degraded_c122 = any(
        re.search(r"degraded", text, re.I) and re.search(r"C-122", text)
        for text, _, _, _ in field_rows
    ) or bool(re.search(r"degraded[^.\n]{0,200}C-122|C-122[^.\n]{0,200}degraded", section_prose, re.I))
    if not degraded_c122:
        fail(5, "the degraded flag's meaning does not cite C-122")
        ac5_ok = False
    if ac5_ok:
        print("PASS AC5: the session meaning cites C-19 and the degraded flag cites C-122")

# ---------------------------------------------------------------------------
# Load wire.toml
# ---------------------------------------------------------------------------
document = None
toml_error = None
tables_text = read_text(tables_path)
if tables_text is None:
    fail(6, f"{tables_path} does not exist")
else:
    try:
        document = parse_toml(tables_text)
    except TomlError as error:
        toml_error = error
        fail(6, f"{tables_path} could not be read by this check: {error}")

# The [[versioning]] values of 004-04, for the AC 1 cross-check.
versioning = (document or {}).get("versioning", {}) if isinstance(document, dict) else {}
if isinstance(versioning, dict) and versioning:
    if versioning.get("magic") != 0x4D4E:
        fail(1, f"{tables_path} [versioning] magic is {versioning.get('magic')!r}, not 0x4d4e (004-04)")
    if versioning.get("version") != 0x0100:
        fail(1, f"{tables_path} [versioning] version is {versioning.get('version')!r}, not 0x0100 (004-04)")

# The enumeration as declared in wire.toml (the checker's authority for
# membership on the machine-readable side).
toml_members = set()
if isinstance(document, dict):
    for entry in document.get("reject", []) or []:
        if isinstance(entry, dict) and isinstance(entry.get("name"), str):
            toml_members.add(entry["name"])
if toml_members and toml_members != member_set:
    fail(6, "the [[reject]] enumeration in wire.toml disagrees with wire.md's Rejection reasons section")

# ---------------------------------------------------------------------------
# AC 6 — the media packet [[message]] table
# ---------------------------------------------------------------------------
media_message = None
media_fields = []
if isinstance(document, dict):
    for msg in document.get("message", []) or []:
        if isinstance(msg, dict) and "mediapacket" in norm_name(str(msg.get("name", ""))):
            media_message = msg
            media_fields = msg.get("field", []) or []
            break

if tables_text is not None and media_message is None:
    fail(
        6,
        f"{tables_path} has no [[message]] for the media packet (a message whose "
        "name normalises to something containing 'media packet')",
    )

if media_message is not None:
    ac6_ok = True
    name = media_message.get("name")
    if media_message.get("layer") != "media":
        fail(6, f"message {name!r}: layer is {media_message.get('layer')!r}, not \"media\"")
        ac6_ok = False
    if media_message.get("dscp") != 46:
        fail(6, f"message {name!r}: dscp is {media_message.get('dscp')!r}, not 46 (EF, C-13)")
        ac6_ok = False
    max_size = media_message.get("max_size")
    if not isinstance(max_size, int) or isinstance(max_size, bool):
        fail(6, f"message {name!r}: max_size is not stated as an integer")
        ac6_ok = False
    elif field_table_found and header_bits % 8 == 0 and max_size < header_bytes:
        fail(
            6,
            f"message {name!r}: max_size {max_size} is smaller than the header's "
            f"{header_bytes} bytes",
        )
        ac6_ok = False

    reasons = media_message.get("rejection_reasons")
    if not isinstance(reasons, list) or not reasons:
        fail(6, f"message {name!r}: rejection_reasons is missing or empty")
        ac6_ok = False
    else:
        for reason in reasons:
            if reason not in member_set:
                fail(
                    6,
                    f"message {name!r}: rejection reason {reason!r} is not a member "
                    "of the closed enumeration",
                )
                ac6_ok = False

    # Field rows agree with wire.md name-by-name, offset-by-offset.
    if field_table_found:
        # Field names: in wire.md's five-column format the name lives in the
        # Meaning cell; in wire.toml it is the field's `name`. A row and a
        # field agree when the normalised toml name occurs in the normalised
        # row text, or — as a fallback — when offset and width match.
        by_name = {}
        for f in media_fields:
            if isinstance(f, dict) and isinstance(f.get("name"), str):
                by_name[norm_name(f["name"])] = f
        matched_names = set()
        for text, off, bits, stated in field_rows:
            cells = [c for c in text.split("|") if c.strip()]
            md_label = cells[3][:60] if len(cells) > 3 else text[:60]
            row_norm = norm_name(text)
            match = None
            for fname, f in by_name.items():
                if len(fname) >= 3 and fname in row_norm and fname not in matched_names:
                    match = f
                    break
            if match is None:
                for f in media_fields:
                    if not isinstance(f, dict):
                        continue
                    fname = norm_name(str(f.get("name", "")))
                    if fname in matched_names:
                        continue
                    toff = f.get("offset")
                    tw = f.get("width")
                    if not isinstance(toff, int) or not isinstance(tw, int):
                        continue
                    if bits < 8:
                        if off * 8 <= toff < off * 8 + 8 and tw == bits:
                            match = f
                            break
                    elif toff == off * 8 and tw == bits:
                        match = f
                        break
            if match is None:
                fail(
                    6,
                    f"no [[message.field]] row agrees with the wire.md field "
                    f"{md_label!r} (offset {off}, width {bits} bits)",
                )
                ac6_ok = False
                continue
            matched_names.add(norm_name(str(match.get("name", ""))))
            toff = match.get("offset")
            tw = match.get("width")
            if not isinstance(toff, int) or not isinstance(tw, int):
                fail(6, f"field {match.get('name')!r}: offset/width are not integers")
                ac6_ok = False
                continue
            if bits < 8:
                if not (off * 8 <= toff < off * 8 + 8):
                    fail(
                        6,
                        f"field {match.get('name')!r}: offset {toff} is not inside "
                        f"byte {off} where wire.md places it",
                    )
                    ac6_ok = False
            elif toff != off * 8:
                fail(
                    6,
                    f"field {match.get('name')!r}: toml offset {toff} bits vs "
                    f"wire.md offset {off} bytes",
                )
                ac6_ok = False
            if tw != bits:
                fail(
                    6,
                    f"field {match.get('name')!r}: toml width {tw} bits vs wire.md "
                    f"width {bits} bits",
                )
                ac6_ok = False
        if len(media_fields) > len(field_rows):
            extra = [f.get("name") for f in media_fields if isinstance(f, dict)][len(field_rows):]
            warn(6, f"wire.toml has more fields than the wire.md table lists: {extra}")

    if ac6_ok:
        print(
            f"PASS AC6: wire.toml message {name!r}: layer media, dscp 46 (C-13), "
            "max_size stated, members-only reasons, fields agreeing with wire.md"
        )

# ---------------------------------------------------------------------------
# The don't-fragment statement (rule 11, C-100)
# ---------------------------------------------------------------------------
if sec_i is not None and not placeholder:
    df_ok = True
    if not re.search(r"don'?t-?fragment|do\s+not\s+fragment|\bDF\b", section_prose, re.I):
        fail("DF", "the section does not state the don't-fragment requirement")
        df_ok = False
    if not re.search(r"IPv4", section_prose):
        fail("DF", "the don't-fragment statement does not name IPv4 (C-100: IPv4 only in v1)")
        df_ok = False
    if not re.search(r"sender", section_prose, re.I) or not re.search(
        r"obligat|must|sets?|require|duty", section_prose, re.I
    ):
        fail("DF", "the don't-fragment statement does not name it as a sender obligation")
        df_ok = False
    if not re.search(
        r"outside\s+the\s+header|not\s+(?:a\s+)?(?:part\s+of\s+the\s+)?header\s+field"
        r"|not\s+in\s+the\s+header|beyond\s+the\s+header|header\s+does\s+not\s+carry"
        r"|not\s+part\s+of\s+the\s+header|outside\s+this\s+header",
        section_prose,
        re.I,
    ):
        fail("DF", "the don't-fragment statement does not place the DF bit outside the header")
        df_ok = False
    if not re.search(r"C-100", section_prose):
        fail("DF", "the don't-fragment statement does not cite C-100")
        df_ok = False
    if df_ok:
        print("PASS DF: don't-fragment stated as an IPv4 DF-bit sender obligation outside the header (C-100, rule 11)")

# ---------------------------------------------------------------------------
# DoD — evidence tags on the new section; the 004-02 checker and friends
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
            "this block of the Media packet header section carries no evidence tag "
            "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
            + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
        )
        prev_tagged = False

if not overridden:
    # 1. The 004-02 checker, from the command line: the machine-readable side
    #    of AC 3's contiguous, non-overlapping layout and AC 6.
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
                fail(
                    6,
                    "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: "
                    + result,
                )

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

    # 4. 004-01's checks still pass on the branch.
    if os.path.exists("checks/004-01.sh"):
        try:
            prev = subprocess.run(
                ["bash", "checks/004-01.sh"], capture_output=True, text=True, timeout=600
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            prev = None
            fail("DoD", f"could not run checks/004-01.sh: {error}")
        if prev is not None and prev.returncode != 0:
            tail = "\n".join((prev.stdout + prev.stderr).strip().splitlines()[-8:])
            fail("DoD", f"checks/004-01.sh no longer passes:\n{tail}")
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

print("OK: 6/6 acceptance criteria for 004-06, don't-fragment stated, DoD tags present")
PY
