#!/usr/bin/env bash
# 004-13 — slice checksum: algorithm, width, covered range, placement.
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
#   AC 1  The "Checksum" section states every parameter of the algorithm as
#         a table row or a "name: value" line: width, polynomial, initial
#         value, input reflection, output reflection, final XOR and the
#         check value of the "123456789" test string. The check value is
#         VERIFIED: the python bitwise CRC below, run on the stated
#         parameters, must produce it. If the section names a CRC catalogue
#         variant that this check knows, its parameters must equal the
#         catalogue's.
#   AC 2  The covered byte range is exact: the section names the packet
#         header, the slice header and the audio payload as covered, and
#         names as excluded (or otherwise treated) every mutable or optional
#         field: the measurement extension (004-10), the copy index (C-07),
#         the send timestamp, the packet index and packet count, and the
#         announcement container. The placement is decided: in the last
#         packet of a multi-packet slice, or in every packet.
#   AC 3  A worked example: a labelled input (hex bytes, with its byte count
#         when stated) and an expected checksum. The check computes the CRC
#         of the input with the stated parameters and requires the expected
#         value.
#   AC 4  The cost estimate per default slice (1 152 audio bytes at 2 000
#         slices/s, 2 304 000 bytes/s) is stated, and the task 026
#         performance check is an open item with an owner and a falsifier
#         that names a too-slow result on the reference embedded target.
#   AC 5  wire.toml's media packet message carries a checksum field whose
#         offset and width agree with wire.md's checksum table, whose type
#         is the unsigned integer of that width, whose rejection reasons
#         include BadChecksum, and whose max_size equals 96 + the checksum's
#         bytes. `cargo run --bin wire-check` passes and
#         `cargo test -p hadronyche-tools --test wire_tables` stays 12/0/0.
#
# Budget (C-137), checked as arithmetic: 16 (packet header) + 22 (slice
# header) + 49 (maximal announcement container) + 9 (measurement extension)
# = 96 bytes, so the checksum has at most 100 - 96 = 4 bytes. A wider
# checksum needs a stated fallback rule that names the omitted part and when.
#
# Validity: BadChecksum is a member of the closed enumeration and the section
# states that a mismatch is dropped and counted under it.
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
# CRC: a generic, table-free bitwise implementation parameterised by width,
# polynomial, initial value, input/output reflection and final XOR (the
# Rocksoft model). Used to verify the stated check value and worked example.
# ---------------------------------------------------------------------------
def reflect_bits(value, n):
    out = 0
    for _ in range(n):
        out = (out << 1) | (value & 1)
        value >>= 1
    return out


def crc_bitwise(data, width, poly, init, refin, refout, xorout):
    mask = (1 << width) - 1
    top = 1 << (width - 1)
    reg = init & mask
    for byte in data:
        b = reflect_bits(byte, 8) if refin else byte
        for k in range(8):
            bit = (b >> (7 - k)) & 1
            feedback = (1 if reg & top else 0) ^ bit
            reg = (reg << 1) & mask
            if feedback:
                reg ^= poly
    if refout:
        reg = reflect_bits(reg, width)
    return (reg ^ xorout) & mask


# Catalogue entries this check recognises by name (values from the CRC
# catalogue; the check value is that of the "123456789" test string). A name
# that is not listed here is accepted with its stated parameters, verified
# only by the check value.
CATALOGUE = {
    "CRC-32/ISO-HDLC": dict(width=32, poly=0x04C11DB7, init=0xFFFFFFFF, refin=True,
                            refout=True, xorout=0xFFFFFFFF, check=0xCBF43926),
    "CRC-32C": dict(width=32, poly=0x1EDC6F41, init=0xFFFFFFFF, refin=True,
                    refout=True, xorout=0xFFFFFFFF, check=0xE3069283,
                    aliases=["CRC-32/ISCSI"]),
    "CRC-32/BZIP2": dict(width=32, poly=0x04C11DB7, init=0xFFFFFFFF, refin=False,
                         refout=False, xorout=0xFFFFFFFF, check=0xFC891918),
    "CRC-32/MPEG-2": dict(width=32, poly=0x04C11DB7, init=0xFFFFFFFF, refin=False,
                          refout=False, xorout=0x00000000, check=0x0376E6E7),
    "CRC-16/CCITT-FALSE": dict(width=16, poly=0x1021, init=0xFFFF, refin=False,
                               refout=False, xorout=0x0000, check=0x29B1),
    "CRC-16/ARC": dict(width=16, poly=0x8005, init=0x0000, refin=True,
                       refout=True, xorout=0x0000, check=0xBB3D),
    "CRC-16/XMODEM": dict(width=16, poly=0x1021, init=0x0000, refin=False,
                          refout=False, xorout=0x0000, check=0x31C3),
}
CHECK_INPUT = b"123456789"

# ---------------------------------------------------------------------------
# Load wire.md, find the Checksum section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    fail(1, f"{wire_path} does not exist")
    fail("DoD", "cannot check — wire.md does not exist")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.fullmatch(r"(slice\s+)?checksums?", title, re.I):
        sec_i = i
        break

PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)
placeholder = sec_i is None
section = ""
section_prose = ""
if sec_i is None:
    for ac in range(1, 6):
        fail(ac, "no 'Checksum' section heading in wire.md")
    fail("DoD", "no 'Checksum' section heading in wire.md")
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
            fail(ac, "the Checksum section is still the 004-13 placeholder")
        fail("DoD", "the Checksum section is still the 004-13 placeholder")

blks = blocks(section_prose)
EXCL = re.compile(
    r"exclud|not\s+covered|outside\s+(?:the\s+)?covered|omitted\s+from|"
    r"not\s+(?:part\s+of|included|in\s+the\s+covered)|left\s+out|"
    r"treated|zero(?:ed)?\s+(?:for|in)\s+the\s+(?:computation|checksum)",
    re.I,
)

# The field-level units of the section: table rows and prose blocks.
section_units = []
for _, _, rows in parse_tables(section):
    for row in rows:
        section_units.append(" | ".join(row))
section_units += blks


def hex_or_none(text):
    if text is None:
        return None
    m = re.search(r"0x([0-9a-fA-F]+)", text)
    if m:
        return int(m.group(1), 16)
    m = re.search(r"\b(\d+)\b", text)
    return int(m.group(1)) if m else None


def bool_or_none(text):
    if text is None:
        return None
    m = re.search(r"\b(true|false|yes|no)\b", text, re.I)
    if not m:
        return None
    return m.group(1).lower() in ("true", "yes")


PARAM_NAMES = {
    "width": r"^(crc)?width(bits?)?$",
    "poly": r"^(generator)?poly(nomial)?$",
    "init": r"^init(ial)?(value)?$",
    "refin": r"^(refin|reflectin|reflectinput|inputreflect(ed|ion)?|refinput)$",
    "refout": r"^(refout|reflectout|reflectoutput|outputreflect(ed|ion)?|refoutput)$",
    "xorout": r"^(xorout|finalxor|xoroutput)$",
    "check": r"^check(value)?(of)?(123456789)?$",
}


def find_param(key):
    pat = re.compile(PARAM_NAMES[key], re.I)
    if not isinstance(section, str) or not section:
        return None
    for _, _, rows in parse_tables(section):
        for row in rows:
            if len(row) >= 2 and pat.search(norm_name(row[0])):
                return row[1]
    for line in section_prose.splitlines():
        m = re.match(r"^\s*(?:[-*]\s*)?\**([^:=|]+?)\**\s*[:=]\s*(.+?)\s*$", line)
        if m and pat.search(norm_name(m.group(1))):
            return m.group(2)
    return None


# ---------------------------------------------------------------------------
# AC 1 — the algorithm parameters, and the check value verified by computing
# ---------------------------------------------------------------------------
params = {}
stated_check = None
if not placeholder:
    ac1_ok = True
    raw = {k: find_param(k) for k in PARAM_NAMES}
    missing = [k for k in PARAM_NAMES if raw[k] is None]
    if missing:
        fail(1, "parameter(s) not stated: " + ", ".join(missing)
             + " (width, polynomial, initial value, input and output reflection,"
             " final XOR and the check value of \"123456789\" are all required)")
        ac1_ok = False
    else:
        params["width"] = hex_or_none(raw["width"])
        params["poly"] = hex_or_none(raw["poly"])
        params["init"] = hex_or_none(raw["init"])
        params["refin"] = bool_or_none(raw["refin"])
        params["refout"] = bool_or_none(raw["refout"])
        params["xorout"] = hex_or_none(raw["xorout"])
        stated_check = hex_or_none(raw["check"])
        bad = [k for k in ("width", "poly", "init", "xorout", "check") if (
            params.get(k) if k != "check" else stated_check) is None]
        if params["refin"] is None or params["refout"] is None:
            bad.append("refin/refout (true or false)")
        if bad:
            fail(1, "parameter value(s) unreadable: " + ", ".join(bad))
            ac1_ok = False
        else:
            w = params["width"]
            if not 1 <= w <= 128:
                fail(1, f"width {w} bits is outside 1..128")
                ac1_ok = False
            mask = (1 << w) - 1 if 1 <= w <= 128 else 0
            for k in ("poly", "init", "xorout"):
                if params[k] & ~mask:
                    fail(1, f"{k} 0x{params[k]:X} does not fit in {w} bits")
                    ac1_ok = False
            if stated_check & ~mask:
                fail(1, f"check value 0x{stated_check:X} does not fit in {w} bits")
                ac1_ok = False

    if ac1_ok:
        computed = crc_bitwise(CHECK_INPUT, params["width"], params["poly"],
                               params["init"], params["refin"], params["refout"],
                               params["xorout"])
        if computed != stated_check:
            fail(1, f"the stated check value 0x{stated_check:X} is not what the stated "
                 f"parameters give for \"123456789\": computed 0x{computed:X}")
            ac1_ok = False
        for name, entry in CATALOGUE.items():
            names = [name] + entry.get("aliases", [])
            hit = any(
                re.search(r"(?<![A-Za-z0-9-])" + re.escape(n) + r"(?![A-Za-z0-9])",
                          section_prose)
                for n in names
            )
            if not hit:
                continue
            for k in ("width", "poly", "init", "refin", "refout", "xorout", "check"):
                want = entry[k]
                got = stated_check if k == "check" else params[k]
                if got != want:
                    fail(1, f"the section names {name} but states {k} "
                         f"{got!r}; the catalogue value is {want!r}")
                    ac1_ok = False
        if ac1_ok:
            print(
                f"PASS AC1: width {params['width']} bits, poly 0x{params['poly']:X}, "
                f"init 0x{params['init']:X}, refin {params['refin']}, refout "
                f"{params['refout']}, xorout 0x{params['xorout']:X}; check value "
                f"0x{stated_check:08X} verified by computation"
            )
    AC1_OK = ac1_ok
else:
    AC1_OK = False

# ---------------------------------------------------------------------------
# AC 2 — covered byte range, excluded mutable fields, placement
# ---------------------------------------------------------------------------
if not placeholder:
    ac2_ok = True

    if not re.search(r"\bcover", section_prose, re.I):
        fail(2, "the section does not state the covered byte range")
        ac2_ok = False
    for name, pattern in (
        ("packet header", r"packet\s+header"),
        ("slice header", r"slice\s+header"),
        ("audio payload", r"audio|payload"),
    ):
        if not re.search(pattern, section_prose, re.I):
            fail(2, f"the covered range does not mention the {name}")
            ac2_ok = False

    REQUIRED_EXCLUDED = (
        ("measurement extension (004-10, mutated by every processor)", r"measurement[\s_-]*extension"),
        ("copy index (C-07)", r"copy[\s_-]*index"),
        ("send timestamp (per packet)", r"send[\s_-]*timestamp"),
        ("packet index / packet count (per packet)", r"packet[\s_-]*(?:index|count)"),
        ("announcement container (optional, omittable on a fallback path)", r"announcements?"),
    )
    for label, pattern in REQUIRED_EXCLUDED:
        if not any(re.search(pattern, u, re.I) and EXCL.search(u)
                   for u in section_units):
            fail(2, f"the covered range does not name {label} as excluded "
                 "(or otherwise treated) in the same statement")
            ac2_ok = False

    if not re.search(r"placed|placement|placing", section_prose, re.I) or not re.search(
        r"last\s+packet|every\s+packet|each\s+packet|per[\s-]packet", section_prose, re.I
    ):
        fail(2, "the placement is not decided (in the last packet of a multi-packet "
             "slice, or in every packet)")
        ac2_ok = False

    if ac2_ok:
        print("PASS AC2: covered range names the packet header, slice header and audio; "
              "measurement extension, copy index, send timestamp, packet index/count and "
              "announcements are excluded or treated; placement decided")
    AC2_OK = ac2_ok
else:
    AC2_OK = False

# ---------------------------------------------------------------------------
# AC 3 — worked example: hex input, expected value, computed
# ---------------------------------------------------------------------------
if not placeholder:
    ac3_ok = True
    idx = section.lower().find("worked example")
    if idx < 0:
        fail(3, "no 'worked example' in the Checksum section")
        ac3_ok = False
    else:
        tail = section[idx:]
        m_in = re.search(r"input", tail, re.I)
        after = tail[m_in.start():] if m_in else ""
        m_exp2 = re.search(r"expected", after, re.I)
        if m_in is None:
            fail(3, "the worked example states no input")
            ac3_ok = False
        elif m_exp2 is None:
            fail(3, "the worked example states no expected checksum after its input")
            ac3_ok = False
        else:
            span = after[: m_exp2.start()]
            hexbytes = []
            for ln in span.splitlines():
                s = ln.strip().strip("`").strip()
                if not s or s.startswith("```"):
                    continue
                if ":" in s and re.search(r"input", s, re.I):
                    s = s.split(":", 1)[1].strip()
                tokens = [t for t in re.split(r"[\s,]+", s) if t]
                if not tokens:
                    continue
                if all(re.fullmatch(r"(0x)?[0-9a-fA-F]{2}", t) for t in tokens):
                    hexbytes.extend(int(t[-2:], 16) for t in tokens)
            if not hexbytes:
                fail(3, "no input bytes (hex pairs) found after the Input label")
                ac3_ok = False
            m_n = re.search(r"input[^\n:]{0,40}?\(\s*(\d+)\s*bytes?", after, re.I)
            if m_n and int(m_n.group(1)) != len(hexbytes):
                fail(3, f"the input is labelled {m_n.group(1)} bytes but holds "
                     f"{len(hexbytes)}")
                ac3_ok = False
            m_val = re.search(r"0x([0-9a-fA-F]+)", after[m_exp2.start(): m_exp2.start() + 200])
            if m_val is None:
                fail(3, "the expected checksum has no 0x value after the word 'expected'")
                ac3_ok = False
            elif AC1_OK and hexbytes:
                expected = int(m_val.group(1), 16)
                got = crc_bitwise(bytes(hexbytes), params["width"], params["poly"],
                                  params["init"], params["refin"], params["refout"],
                                  params["xorout"])
                if got != expected:
                    fail(3, f"the stated input ({len(hexbytes)} bytes) with the stated "
                         f"parameters gives 0x{got:X}, not the stated 0x{expected:X}")
                    ac3_ok = False
                else:
                    print(f"PASS AC3: worked example of {len(hexbytes)} bytes gives "
                          f"0x{got:X} under the stated parameters")
    AC3_OK = ac3_ok
else:
    AC3_OK = False

# ---------------------------------------------------------------------------
# AC 4 — cost per default slice, and the task 026 open item
# ---------------------------------------------------------------------------
if not placeholder:
    ac4_ok = True
    SP = r"[\s,.\u00a0\u202f]?"
    if not re.search(r"1" + SP + r"152", section_prose):
        fail(4, "the cost estimate does not state the 1 152 audio bytes per default slice")
        ac4_ok = False
    if not re.search(r"2" + SP + r"000\s*slices?", section_prose):
        fail(4, "the cost estimate does not state the 2 000 slices/s rate")
        ac4_ok = False
    if not (re.search(r"2" + SP + r"304" + SP + r"000", section_prose)
            or re.search(r"2\.3\d*\s*M", section_prose)
            or re.search(r"18\.4\d*\s*Mbit", section_prose)):
        fail(4, "the cost estimate does not state the resulting byte rate "
             "(2 304 000 bytes/s, about 2.3 MB/s or about 18.4 Mbit/s)")
        ac4_ok = False

    perf_ok = False
    for k, b in enumerate(blks):
        if not re.search(r"\b026\b", b):
            continue
        window = "\n".join(blks[max(0, k - 1): k + 2])
        if (re.search(r"owner", window, re.I) and re.search(r"falsif", window, re.I)
                and re.search(r"slow", window, re.I)):
            perf_ok = True
            break
    if not perf_ok:
        fail(4, "the task 026 performance check is not an open item with an owner "
             "and a falsifier that names a too-slow result on the reference embedded target")
        ac4_ok = False

    if ac4_ok:
        print("PASS AC4: cost per default slice stated (1 152 bytes at 2 000 slices/s); "
              "task 026 open item with owner and falsifier")
    AC4_OK = ac4_ok
else:
    AC4_OK = False

# ---------------------------------------------------------------------------
# Width and the C-137 budget (arithmetic, not an AC of the ticket)
# ---------------------------------------------------------------------------
CKSUM_BYTES = None
if AC1_OK:
    w = params["width"]
    if w % 8 != 0:
        fail("C-137", f"the checksum width {w} bits is not a whole number of bytes")
    else:
        CKSUM_BYTES = w // 8
        total = 16 + 22 + 49 + 9 + CKSUM_BYTES
        if not placeholder:
            if not (re.search(r"\b96\b", section_prose) and re.search(r"\b100\b", section_prose)):
                fail("C-137", "the section does not show the C-137 arithmetic (96 bytes of "
                     "headers and extensions, 100-byte cap)")
            if total > 100:
                fallback_ok = False
                for b in blks:
                    if (re.search(r"fallback", b, re.I) and re.search(r"omit", b, re.I)
                            and re.search(r"\b(when|if|only)\b", b, re.I)):
                        fallback_ok = True
                        break
                if not fallback_ok:
                    fail("C-137", f"the checksum is {w} bits: 16 + 22 + 49 + 9 + "
                         f"{CKSUM_BYTES} = {total} bytes, above the 100-byte header cap, "
                         "and the section states no fallback rule that omits a named part "
                         "and when")
                else:
                    print(f"NOTE C-137: {total} bytes exceed the cap; the fallback rule "
                          "governs the omission")
            else:
                print(f"PASS C-137: 16 + 22 + 49 + 9 + {CKSUM_BYTES} = {total} <= 100")

# ---------------------------------------------------------------------------
# Validity rule and the closed enumeration
# ---------------------------------------------------------------------------
enum_row = re.search(r"\|\s*8\s*\|\s*BadChecksum\s*\|", wire)
if enum_row is None:
    fail("VALIDITY", "BadChecksum is not a member of the closed enumeration in wire.md")
if not placeholder:
    vr_ok = False
    for b in blks:
        if re.search(r"BadChecksum", b) and re.search(r"drop", b, re.I) and re.search(
            r"count", b, re.I
        ):
            vr_ok = True
            break
    if not vr_ok:
        fail("VALIDITY", "the validity rule does not state that a checksum mismatch is "
             "dropped and counted under BadChecksum")

# ---------------------------------------------------------------------------
# AC 5 — the machine-readable side (runs with or without the section, so the
# RED run also shows the table gap)
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

if isinstance(document, dict):
    media = None
    for msg in document.get("message", []) or []:
        if isinstance(msg, dict) and norm_name(str(msg.get("name", ""))) == "mediapacket":
            media = msg
            break
    if media is None:
        fail(5, f"{tables_path} has no media packet message")
    else:
        cks = [f for f in media.get("field", []) or []
               if isinstance(f, dict) and "checksum" in norm_name(str(f.get("name", "")))]
        if not cks:
            fail(5, "wire.toml's media packet message declares no checksum field")
        else:
            f = cks[0]
            off = f.get("offset")
            wid = f.get("width")
            typ = f.get("type")
            if off != 768:
                fail(5, f"the checksum field's offset is {off} bits; it follows the "
                     "measurement extension at the maximum end of the container, bit 768 "
                     "(byte 96)")
            if not isinstance(wid, int) or wid % 8 != 0:
                fail(5, f"the checksum field's width {wid} is not whole bytes")
            else:
                uint_of = {8: "u8", 16: "u16", 32: "u32", 64: "u64", 128: "u128"}
                if uint_of.get(wid) != typ:
                    fail(5, f"the checksum field of {wid} bits must be type "
                         f"{uint_of.get(wid)}, not {typ!r}")
                if CKSUM_BYTES is not None and wid != CKSUM_BYTES * 8:
                    fail(5, f"wire.toml's checksum is {wid} bits; wire.md's is "
                         f"{CKSUM_BYTES * 8} bits")
                want_max = 96 + wid // 8
                if media.get("max_size") != want_max:
                    fail(5, f"media packet max_size is {media.get('max_size')}, expected "
                         f"{want_max} (96 + the checksum's {wid // 8} bytes)")
        reasons = media.get("rejection_reasons")
        if not isinstance(reasons, list) or "BadChecksum" not in reasons:
            fail(5, "the media packet's rejection_reasons do not include BadChecksum")
        elif any(r not in ("TooShort", "BadMagic", "VersionMismatch", "ForeignDomain",
                           "UnconfiguredDomain", "UnknownStream", "UnadmittedSource",
                           "BadLength", "BadChecksum", "BadPacketIndex", "WrongEpoch",
                           "OversizeControl", "ReservedNonZero") for r in reasons):
            fail(5, "the media packet's rejection_reasons name a non-member")

if not placeholder and CKSUM_BYTES is not None:
    cks_row_ok = False
    for _, header, rows in parse_tables(section):
        if not is_field_table(header):
            continue
        for row in rows:
            if len(row) < 2:
                continue
            m_off = re.search(r"\d+", row[0])
            if m_off and int(m_off.group(0)) == 96 and width_to_bits(row[1]) == CKSUM_BYTES * 8:
                cks_row_ok = True
    if not cks_row_ok:
        fail(5, "wire.md's checksum field table has no row at offset 96 with the checksum's width")
    else:
        print("PASS AC5 (md): wire.md's checksum row agrees with the derived width and offset")

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
        fail("DoD", "this block of the Checksum section carries no evidence tag "
             "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
             + body.splitlines()[0][:90])
        prev_tagged = False

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
            fail(5, f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(5, f"could not run cargo run --bin wire-check: {error}")

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
if fails:
    for ac, msg in fails:
        label = ac if isinstance(ac, str) else f"AC{ac}"
        print(f"FAIL {label}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 5/5 acceptance criteria for 004-13, budget arithmetic, validity rule, DoD tags present")
PY
