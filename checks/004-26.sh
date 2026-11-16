#!/usr/bin/env bash
# 004-26 — manifest encoding in the admission handshake.
#
# RED-phase document check. It validates the "Manifest" section of
# doc/design/spec/wire.md and the manifest tables in doc/design/spec/wire.toml
# against the acceptance criteria of the ticket, and re-runs the wire-check
# binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing
# on throwaway fixtures only: when either path is overridden the cargo checks are
# skipped with a NOTE, because they always target the real repository.
#
# Numbers are read at run time: the 1 252-byte control bound (C-17) and the
# stream bound (C-09) from doc/design/02-constants.md; the manifest envelope of
# the landed "Manifest section" (its manifest-bytes maximum, its section-count
# maximum, its manifest-length range); the Reject enumeration and the refusal
# enumeration (their names are read, not copied); the common control header of
# the Hello section. Landed sections are read, never rewritten.
#
# The author contract (headings and labels the check locates):
#
#   ## Manifest                prose: the manifest's content and its envelope
#       (004-18's Manifest section), every prose paragraph of 5+ words tagged.
#   field tables               five columns (Offset, Width, Type, Meaning, Valid
#       range); each tiles from its first offset; every counted row states its
#       fixed maximum ("W × max M"). The layout (one table or several, and which
#       is top-level) is the coder's: the maximum manifest size is the computed
#       size of a table that no counted row references, stated in a sentence
#       that also says "maximum" and "manifest" with its bytes.
#   ### <title>                one per subsection; a subsection with a table has
#       an evidence tag in its prose.
#   Invalid forms              a two-column table with a "Form" and an "Outcome"
#       header, at least five rows, one for each of the five cases of ticket
#       Work item 3; every outcome names a member of the Reject or refusal
#       enumeration.
#   Open item paragraph        starts "Open item", names an owner and a
#       falsifier; one of them records the join-delay bound (owner epic 004).
#   wire.toml                  one [[message]] per manifest field table whose
#       fields (offset and width in bits, maximum count) equal the Markdown
#       table's, layer control, dscp 40, max_size equal to the computed size and
#       rejection_reasons naming members of the Reject enumeration.
#
# What is checked, per acceptance criterion:
#   AC 1  every Context item of the ticket has a field with a width and a unit:
#         ports, content types, channel map, L in whole samples, the join-delay
#         count and each join delay in whole samples with the frequency-dependent
#         flag, dense controller initial values, the conversion delay of a
#         device-facing port, the cost bound in operations per slice, required E,
#         declared E and the unit of E.
#   AC 2  the none encoding of required E and the unknown encoding of declared E
#         are stated, in either of two forms the ticket leaves open: a sentinel
#         outside the stated E range (checked arithmetically), or a flag field
#         (its meaning stated, the E value stated as not interpreted, and a
#         sentence stating distinctness from every valid E value).
#   AC 3  the maximum manifest size, computed from the table no counted row references, at its
#         maximum counts, is stated in bytes; it fits the landed envelope (its
#         sections, at most 255 of 1 196 bytes of manifest each, the landed
#         envelope's 56 + 1 196 = 1 252 bytes agreeing with C-17). Above 1 252
#         bytes the text must report a falsified constant and propose a split.
#   AC 4  the join-delay bound is an open item with an owner and a falsifier.
#   AC 5  at least five invalid forms; the five categories of Work item 3 are
#         each covered; every outcome names an existing reason of the Reject or
#         refusal enumeration; the length row names BadLength (Work item 3 fixes it).
#   AC 6  the Markdown and TOML agree table for table (offsets, widths, maximum
#         counts, max_size, layer, dscp, reasons); when the paths are not
#         overridden, `cargo run --bin wire-check` reports "checks clean" and
#         `cargo test -p hadronyche-tools --test wire_tables` has 0 failed and at
#         least 12 passed (the count that stood before this ticket).
#   DoD   every prose paragraph of 5+ words in the section carries an evidence
#         tag; every subsection with a table has one; every open item names an
#         owner and a falsifier; the landed Hello common header and the landed
#         manifest envelope are unchanged in the way 004-18's check verifies them.
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
import math
import os
import re
import subprocess
import sys

wire_path, tables_path = sys.argv[1], sys.argv[2]
overridden = bool(os.environ.get("WIRE_MD") or os.environ.get("WIRE_TOML"))
CONSTANTS_PATH = "doc/design/02-constants.md"
RESULTS = []


def check(ac, ok, text):
    ok = bool(ok)
    RESULTS.append(ok)
    print(("PASS " if ok else "FAIL ") + ac + ": " + text)
    return ok


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


NUM = r"(?:0x[0-9a-fA-F]+|\d{1,3}(?:[   ]\d{3})+|\d+)"
TIME_UNIT = r"(?:µs|μs|\bus\b|microseconds?|\bns\b|nanoseconds?|\bms\b|milliseconds?)"
ANY_UNIT = r"(?:samples?|frames?|operations?|bits?|bytes?|Hz|seconds?|\bs\b|" + TIME_UNIT + r"|\bns\b)"


def to_int(s):
    s = s.strip()
    if s.lower().startswith("0x"):
        return int(s, 16)
    return int(re.sub(r"[   ,]", "", s))


# ---------------------------------------------------------------------------
# Markdown helpers (conventions of the 004-18 checker)
# ---------------------------------------------------------------------------
def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    cells = re.split(r"(?<!\\)\|", s)
    return [c.strip().replace("\\|", "|") for c in cells]


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


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(text):
    return bool(EVIDENCE.search(text))


def prose_blocks(text):
    """[(kind, text)]: kind 'p' prose or 't' table; headings and blank lines split blocks."""
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


def width_of(cell):
    """(bits, max_count): 'N' or 'N bytes' -> N*8; 'N bits' -> N; 'W × max M' -> (W in bits, M)."""
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


def is_field_table(header):
    h = [re.sub(r"[^a-z0-9]", "", x.lower()) for x in header]
    return len(h) >= 5 and h[0].startswith("offset") and "width" in h[1] and "type" in h[2] \
        and "meaning" in h[3] and "valid" in h[4]


def is_validity_table(header):
    h = [x.lower() for x in header]
    return len(h) >= 2 and any("form" in x for x in h[:2]) and any("outcome" in x for x in h[:2])


def model_rows(tab):
    """Rows of a field table as dicts; malformed rows are reported by the caller."""
    rows, bad = [], []
    for r in tab[2]:
        if len(r) < 5:
            bad.append(f"a row has fewer than five cells: {r[:2]}")
            continue
        if not re.fullmatch(r"\d+", r[0].strip()):
            bad.append(f"the Offset cell '{r[0]}' is not a byte offset")
            continue
        bits, maxc = width_of(r[1])
        if bits is None:
            bad.append(f"the Width cell '{r[1]}' is neither a byte count, a bit count nor 'W × max M'")
            continue
        rows.append({"off": int(r[0]), "bits": bits, "maxc": maxc, "type": r[2].strip(),
                     "meaning": r[3].strip(), "valid": r[4].strip(), "wcell": r[1].strip()})
    return rows, bad


def tile(rows):
    """Bit offset of each row, the total bits, and the tiling errors (cursor from the first offset)."""
    errors, offsets = [], []
    if not rows:
        return offsets, 0, errors
    cursor = rows[0]["off"] * 8
    start = cursor
    for r in rows:
        if r["off"] != cursor // 8:
            errors.append(f"offset {r['off']} does not follow the previous row, which ends at byte {cursor // 8}")
        if r["bits"] % 8 == 0 and cursor % 8 != 0:
            errors.append(f"a whole-byte row at offset {r['off']} is not byte-aligned")
        offsets.append(cursor)
        cursor += r["bits"] * (r["maxc"] or 1)
    if (cursor - start) % 8 != 0:
        errors.append("the table does not cover a whole number of bytes")
    return offsets, cursor - start, errors


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


def section_lines(text, pattern, level):
    lines = text.splitlines()
    hs = heading_index(lines)
    for i, (idx, lvl, title) in enumerate(hs):
        if lvl == level and re.search(pattern, title, re.I):
            end = len(lines)
            for j in range(i + 1, len(hs)):
                if hs[j][1] <= level:
                    end = hs[j][0]
                    break
            return lines[idx + 1:end]
    return None


# ---------------------------------------------------------------------------
# TOML reader (the subset of the 004-18 checker; Python 3.10 has no tomllib)
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
# Constants and landed sections (read at run time, never copied)
# ---------------------------------------------------------------------------
CONST_TEXT = read_text(CONSTANTS_PATH) or ""


def const_val(cid):
    for line in CONST_TEXT.splitlines():
        if line.startswith(f"| {cid} |"):
            cells = line.split("|")
            return cells[3].strip() if len(cells) > 3 else ""
    return ""


BOUND = None
_m = re.search(r"at\s+most\s+(" + NUM + r")\s*bytes", const_val("C-17"), re.I)
if _m:
    BOUND = to_int(_m.group(1))
check("AC3", BOUND is not None, f"the control bound (C-17) reads as {BOUND} bytes from {CONSTANTS_PATH}")
BOUND = BOUND or 0

wire = read_text(wire_path)
if wire is None:
    check("AC3", False, f"{wire_path} does not exist")
    wire = ""
wire_lines = wire.splitlines()

# Landed: the Hello common control header (the 004-18 structural reference)
HELLO_HEADER = None
_hello = section_lines(wire, r"^Hello$", 2)
if _hello is not None:
    _sub = [t for t in subsections(_hello) if re.search(r"common control header", t[0], re.I)]
    if _sub:
        _rows, _bad = model_rows(("", ["Offset (bytes)", "Width (bytes)", "Type", "Meaning", "Valid range"],
                                  [split_row(l) for l in _sub[0][1].splitlines() if l.lstrip().startswith("|")][2:]))
        HELLO_HEADER = [(r["off"], r["bits"], r["type"]) for r in _rows[:7]]
check("DoD", HELLO_HEADER is not None and len(HELLO_HEADER) == 7,
      "the Hello common control header is readable (seven rows)")

# Landed: the admission manifest envelope and its limits
ENV = {"section_max": None, "manifest_max": None, "length_max": None, "blob_bits": None, "envelope_bytes": None}
_adm = section_lines(wire, r"^Admission messages$", 2)
_env_lines = None
if _adm is not None:
    for title, body in subsections(_adm):
        if re.search(r"manifest section", title, re.I):
            _env_lines = body
if _env_lines is not None:
    _tabs = [t for t in parse_tables(_env_lines) if is_field_table(t[1])]
    if _tabs:
        _rows, _bad = model_rows(_tabs[0])
        _offs, _total, _errs = tile(_rows)
        ENV["envelope_bytes"] = _total // 8
        for r in _rows:
            if r["meaning"].lower().startswith("section count"):
                _m = re.search(r"1 to (" + NUM + r")", r["valid"])
                ENV["section_max"] = to_int(_m.group(1)) if _m else None
            if r["meaning"].lower().startswith("manifest length"):
                _m = re.search(r"1 to (" + NUM + r")", r["valid"])
                ENV["length_max"] = to_int(_m.group(1)) if _m else None
            if r["meaning"].lower().startswith("manifest bytes"):
                ENV["manifest_max"] = r["maxc"]
                ENV["blob_bits"] = r["bits"]
check("AC3", ENV["section_max"] is not None and ENV["length_max"] is not None and ENV["manifest_max"] is not None,
      f"the landed manifest section reads: section count max {ENV['section_max']}, manifest length max "
      f"{ENV['length_max']}, manifest bytes max {ENV['manifest_max']}")
if ENV["manifest_max"] is not None and ENV["length_max"] is not None:
    check("AC3", ENV["manifest_max"] == ENV["length_max"],
          "the landed envelope's manifest bytes maximum equals its manifest length maximum")
    check("AC3", ENV["envelope_bytes"] == BOUND,
          f"the landed manifest section computes to {ENV['envelope_bytes']} bytes at its maximum, "
          f"the control bound {BOUND} (C-17)")

# Landed enumerations: the Reject names and the refusal names, read from their tables
def enum_names(lines):
    names = {}
    for _, header, rows in parse_tables("\n".join(lines)):
        for row in rows:
            if len(row) >= 2 and re.fullmatch(r"\d+", row[0].strip()):
                nm = row[1].strip().strip("`*_ ")
                if re.fullmatch(r"[A-Z][A-Za-z0-9]+", nm):
                    names[nm] = int(row[0].strip())
    return names


_rej = section_lines(wire, r"rejection[-\s]+reasons?", 2)
REJECT = enum_names(_rej or [])
_ref = section_lines(wire, r"refusal[-\s]+reasons?", 3)
REFUSAL = enum_names(_ref or [])
check("DoD", len(REJECT) == 13 and len(REFUSAL) == 18,
      f"the landed enumerations read as {len(REJECT)} Reject names and {len(REFUSAL)} refusal names (13 and 18)")
ALL_REASONS = set(REJECT) | set(REFUSAL)

# ---------------------------------------------------------------------------
# The Manifest section
# ---------------------------------------------------------------------------
MAN = section_lines(wire, r"^Manifest$", 2)
check("AC1", MAN is not None, "wire.md has a '## Manifest' section")
MAN = MAN or []
MAN_TEXT = "\n".join(MAN)
FLAT = re.sub(r"\s+", " ", strip_fences(MAN_TEXT))
FIELD_TABS = [t for t in parse_tables(MAN_TEXT) if is_field_table(t[1])]
VALID_TABS = [t for t in parse_tables(MAN_TEXT) if is_validity_table(t[1]) and not is_field_table(t[1])]
check("AC1", len(FIELD_TABS) >= 1, f"the Manifest section has field tables ({len(FIELD_TABS)} found)")

MODELS = []  # per field table: (rows, offsets, total_bits, errors, where)
for k, tab in enumerate(FIELD_TABS):
    rows, bad = model_rows(tab)
    where = f"Manifest field table {k + 1}"
    for b in bad:
        check("AC1", False, f"{where}: {b}")
    offs, total, errs = tile(rows)
    for e in errs:
        check("AC1", False, f"{where}: {e}")
    MODELS.append((rows, offs, total, errs, where))

ALL_ROWS = [r for m in MODELS for r in m[0]]


def txt(r):
    return (r["meaning"] + " " + r["valid"]).lower()


def rows_where(pred):
    return [r for r in ALL_ROWS if pred(r)]


# --- AC1: every Context item has a field, with width and unit ------------------
has_ports = rows_where(lambda r: "port" in r["meaning"].lower() or "port" in r["valid"].lower())
has_content = rows_where(lambda r: "content type" in txt(r))
check("AC1", has_ports and has_content, "ports and content types have fields")
check("AC1", rows_where(lambda r: "channel map" in txt(r)), "the channel map has a field")

L_rows = rows_where(lambda r: (re.search(r"\bL\b", r["meaning"]) or "latency" in r["meaning"].lower())
                    and "sample" in txt(r))
check("AC1", L_rows, "L has a field whose valid range is in whole samples (unit stated)")

join_rows = rows_where(lambda r: "join" in txt(r))
join_counted = [r for r in join_rows if r["maxc"] is not None]
check("AC1", join_counted, "the join-delay count is a counted row with a fixed maximum")
check("AC1", any("sample" in txt(r) for r in join_rows),
      "each inserted join delay is a whole number of samples")
check("AC1", any("frequency" in txt(r) for r in join_rows),
      "the frequency-dependent-delay flag of each join (C-104) has a field")

check("AC1", rows_where(lambda r: "initial value" in txt(r)),
      "dense controller initial values (C-158) have a field")

CONV = rows_where(lambda r: "conversion delay" in txt(r) and "device" in txt(r))
check("AC1", CONV, "the conversion delay on a device-facing port (C-207) has a field")
check("AC1", any(re.search(ANY_UNIT, r["meaning"] + " " + r["valid"] + " " + r["wcell"], re.I) for r in CONV),
      "the conversion delay states its unit")

COST = rows_where(lambda r: "cost bound" in txt(r) and "operation" in txt(r))
check("AC1", COST, "the cost bound (C-106) has a field in operations")
check("AC1", any("slice" in txt(r) for r in COST), "the cost bound is per slice (C-106)")

REQ_E = rows_where(lambda r: "required" in txt(r) and re.search(r"\bE\b", r["meaning"] + " " + r["valid"]))
DEC_E = rows_where(lambda r: "declared" in txt(r) and re.search(r"\bE\b", r["meaning"] + " " + r["valid"]))
check("AC1", REQ_E, "required E per presentation group has a field")
check("AC1", DEC_E, "declared E per sink has a field")
check("AC1", any(re.search(TIME_UNIT, txt(r), re.I) for r in REQ_E + DEC_E),
      "the unit of E is stated")

# --- AC2: the none and unknown encodings are stated, distinct, and kept apart from every valid E value ---
# The ticket fixes the property, not the representation, so two forms are accepted:
#   (a) a sentinel: a numeric code in the E row's valid range, outside the stated "lo to hi" range
#       of that row, and within the field's width;
#   (b) a flag: a separate field (a row without a count) whose valid range maps a number to none or
#       to unknown, with the E value stated as not interpreted when the flag says so, and a sentence
#       stating that none and unknown are distinct from every valid E value.
HEXNUM = r"0x[0-9a-fA-F]+"
NOT_INTERP = r"not (?:interpreted|read|used)|ignored|disregard|does not apply|no meaning"
DISTINCT_TXT = r"distinct|never|cannot|not a valid|no valid|differ"


def code_for(text, word):
    m = re.search(r"(" + HEXNUM + "|" + NUM + r")[^;.,]{0,40}?\b" + word + r"\b", text, re.I)
    if not m:
        m = re.search(r"\b" + word + r"\b[^;.,]{0,40}?(" + HEXNUM + "|" + NUM + ")", text, re.I)
    return to_int(m.group(1)) if m else None


def numeric_range(text):
    m = re.search(r"(" + HEXNUM + "|" + NUM + r")\s*(?:to|\.\.|–|—|-)\s*(" + HEXNUM + "|" + NUM + ")", text)
    if not m:
        return None
    return to_int(m.group(1)), to_int(m.group(2))


def encoding_of(kind, E_rows):
    """('sentinel' | 'flag', code, row) for the kind word, or None."""
    for r in E_rows:
        code = code_for(r["valid"], kind)
        rng = numeric_range(r["valid"])
        if code is not None and rng is not None and not (rng[0] <= code <= rng[1]) and code < 2 ** r["bits"]:
            return ("sentinel", code, r)
    for r in ALL_ROWS:
        if r["maxc"] is None and numeric_range(r["valid"]) is None:
            code = code_for(r["meaning"] + " " + r["valid"], kind)
            if code is not None:
                return ("flag", code, r)
    return None


for label_, kind_, E_rows in (("none", "none", REQ_E), ("unknown", "unknown", DEC_E)):
    for r in E_rows:
        c_ = code_for(r["valid"], kind_)
        g_ = numeric_range(r["valid"])
        if c_ is not None and g_ is not None:
            check("AC2", not (g_[0] <= c_ <= g_[1]),
                  f"the {label_} code {c_} lies outside the E range {g_[0]} to {g_[1]} that the same row states")
ENC_NONE = encoding_of("none", REQ_E)
ENC_UNKNOWN = encoding_of("unknown", DEC_E)
check("AC2", ENC_NONE is not None,
      "the none encoding of required E is stated (a sentinel outside the E range, or a flag field)")
check("AC2", ENC_UNKNOWN is not None,
      "the unknown encoding of declared E is stated (a sentinel outside the E range, or a flag field)")
for label_, enc, E_rows in (("none", ENC_NONE, REQ_E), ("unknown", ENC_UNKNOWN, DEC_E)):
    if enc is not None and enc[0] == "sentinel":
        check("AC2", enc[1] < 2 ** enc[2]["bits"],
              f"the {label_} sentinel {enc[1]} fits the {enc[2]['bits']}-bit E field")
    if enc is not None and enc[0] == "flag":
        check("AC2", any(re.search(NOT_INTERP, r["valid"] + " " + r["meaning"], re.I) for r in E_rows),
              f"the E value is stated as not interpreted when the flag says {label_}")
        check("AC2", any(label_ in sent.lower() and "valid" in sent.lower() and re.search(DISTINCT_TXT, sent, re.I)
                         for sent in re.split(r"(?<=[.;:])\s+", FLAT)),
              f"a sentence states that {label_} is distinct from every valid E value")
if ENC_NONE is not None and ENC_UNKNOWN is not None:
    if ENC_NONE[0] == "sentinel" and ENC_UNKNOWN[0] == "sentinel":
        check("AC2", ENC_NONE[1] != ENC_UNKNOWN[1], "the none and unknown sentinels are distinct numbers")
        for label_, enc in (("none", ENC_NONE), ("unknown", ENC_UNKNOWN)):
            rng = numeric_range(enc[2]["valid"])
            check("AC2", not (rng[0] <= enc[1] <= rng[1]),
                  f"the {label_} sentinel {enc[1]} lies outside its stated E range {rng[0]} to {rng[1]}")
    else:
        check("AC2", True, "the none and unknown encodings take different forms or values; distinctness is stated")

# --- AC3: the maximum manifest size, computed and stated ---------------------
# The ticket fixes the size, not the table layout. The maximum is the computed size of a manifest table
# that no counted row of the section references (its size is not the per-element width of another row).
REFERENCED_BITS = set()
for k, (rows, offs, total, errs, where) in enumerate(MODELS):
    for r in rows:
        if r["maxc"] is not None:
            REFERENCED_BITS.add(r["bits"])
TOP_CANDIDATES = [m for k, m in enumerate(MODELS) if m[2] not in {b for b in REFERENCED_BITS}] or list(MODELS)
claims = []
for sent in re.split(r"(?<=[.;:])\s+", FLAT):
    if re.search(r"maximum", sent, re.I) and re.search(r"manifest", sent, re.I):
        for m in re.finditer(r"(" + NUM + r")\s*bytes", sent):
            claims.append(to_int(m.group(1)))
matched = [m[2] // 8 for m in TOP_CANDIDATES if m[2] // 8 in claims]
TOP_BYTES = max(matched) if matched else None
check("AC3", TOP_BYTES is not None,
      f"the maximum manifest size is stated in bytes and equals the computed size of a manifest table that no "
      f"counted row references (candidates {[m[2] // 8 for m in TOP_CANDIDATES]} bytes; claimed {sorted(set(claims))})")
if TOP_BYTES is not None and ENV["length_max"] and ENV["section_max"]:
    sections = math.ceil(TOP_BYTES / ENV["length_max"])
    check("AC3", sections <= ENV["section_max"],
          f"the manifest needs {sections} section(s) of {ENV['length_max']} bytes, within the landed count "
          f"maximum {ENV['section_max']}")
if TOP_BYTES is not None:
    if TOP_BYTES <= BOUND:
        check("AC3", True, f"the maximum manifest size, {TOP_BYTES} bytes, fits the {BOUND}-byte control bound")
    else:
        pass  # the falsification report is checked in T1 below

# --- AC4: join-delay bound recorded as an open item with an owner and a falsifier ---
PROSE = [b for b in prose_blocks(MAN_TEXT) if b[0] == "p"]
OPEN_PARAS = [b[1] for b in PROSE if re.search(r"open item", re.sub(r"[*_`]", "", b[1]).strip()[:40], re.I)]
join_open = [p for p in OPEN_PARAS if re.search(r"\bjoin", p, re.I) and re.search(r"bound|maximum|\bmax\b", p, re.I)]
check("AC4", join_open, "an open item records the join-delay bound")
check("AC4", any(re.search(r"\b(epic\s*)?004\b", p, re.I) for p in join_open),
      "the join-delay open item is owned by epic 004")

# --- AC5: invalid forms with outcomes -----------------------------------------
FORMS = []
for _, header, rows in VALID_TABS:
    for r in rows:
        if len(r) >= 2:
            FORMS.append((r[0], r[1]))
check("AC5", len(FORMS) >= 5, f"at least five invalid forms are tabled ({len(FORMS)} found)")
for form, outcome in FORMS:
    names = [n for n in ALL_REASONS if re.search(r"\b" + n + r"\b", outcome)]
    check("AC5", bool(names), f"the invalid form '{form[:50]}' names its outcome as a member of the Reject or refusal enumeration")


def cat(pred):
    return [(f, o) for f, o in FORMS if pred(f, o)]


check("AC5", cat(lambda f, o: re.search(r"count", f, re.I) and re.search(r"above|more than|exceed|maximum|greater", f, re.I)),
      "a count above its maximum is an invalid form")
check("AC5", cat(lambda f, o: re.search(r"length", f, re.I) and re.search(r"inconsistent|does not fill|not fill", f, re.I) and "BadLength" in o),
      "a length inconsistent with the sections is an invalid form with outcome BadLength")
check("AC5", cat(lambda f, o: re.search(r"channel[- ]map", f, re.I) and re.search(r"range|duplicat|distinct|twice", f, re.I)),
      "a channel-map entry out of range or duplicated is an invalid form")
check("AC5", cat(lambda f, o: re.search(r"device", f, re.I) and re.search(r"conversion", f, re.I)),
      "a conversion delay on a port that is not device-facing is an invalid form")
check("AC5", cat(lambda f, o: re.search(r"reserved", f, re.I)),
      "a non-zero reserved field is an invalid form (Work item 3)")

# --- T1: a falsified criterion is reported as falsified, never denied (review finding) ------------
PARAS_FLAT = [re.sub(r"\s+", " ", strip_fences(b[1])) for b in prose_blocks(MAN_TEXT) if b[0] == "p"]
DENIAL = re.compile(r"no\s+(constant|statement|criterion|design statement)[^.]*?falsif|not\s+falsif", re.I)
AFFIRM = re.compile(r"\b(is|are)\s+falsified\b|\bfalsifies\b", re.I)
AC3_NAME = re.compile(r"\bAC[- ]3\b")
DENIAL_HITS = [p for p in PARAS_FLAT if DENIAL.search(p) and not (AFFIRM.search(p) and AC3_NAME.search(p))]
for p in DENIAL_HITS:
    check("AC3", False, f"a paragraph denies a falsification without reporting one as falsified: "
                        f"'{DENIAL.search(p).group(0)}'")
check("AC3", not DENIAL_HITS, "no paragraph denies a falsification without the affirmative report of AC-3")
if TOP_BYTES is not None and TOP_BYTES > BOUND:
    REPORT = [p for p in PARAS_FLAT if AFFIRM.search(p) and AC3_NAME.search(p) and "012-08" in p]
    check("AC3", bool(REPORT),
          "a paragraph reports as falsified (is/are falsified) the epic's AC-3 and 012-08 AC 2, which assume a "
          "one-message manifest")
    if REPORT:
        R = REPORT[0]
        nums = {to_int(x) for x in re.findall(NUM, R)}
        check("AC3", TOP_BYTES in nums and BOUND in nums and re.search(r"exceed", R, re.I) is not None,
              f"the same paragraph states the computed maximum, {TOP_BYTES} bytes, and that it exceeds {BOUND} bytes")
        check("AC3", re.search(r"design owner", R, re.I) is not None,
              "the same paragraph says the report goes to the design owner")
        cap = ENV["manifest_max"] or 0
        need = math.ceil(TOP_BYTES / cap) if cap else None
        claimed = {to_int(x) for x in re.findall(r"(" + NUM + r")\s+(?:manifest\s+)?sections?", R, re.I)}
        check("AC3", need is not None and need in claimed and need <= (ENV["section_max"] or 0) and
              re.search(r"split", R, re.I) is not None,
              f"the same paragraph proposes the split: ceil({TOP_BYTES} / {cap}) = {need} sections, within the "
              f"landed count maximum {ENV['section_max']} (claimed section counts {sorted(claimed)})")
    join_clause = re.compile(r"(already|is met|are met)[^.]{0,80}1[   ]?252|1[   ]?252[^.]{0,80}(already|is met)", re.I)
    check("AC4", any(join_clause.search(p) for p in join_open),
          "the join-delay open item says the second clause of the ticket's falsifier is already met at 1 252 bytes")

# AC 5 reassembly (T3 e): one outcome for a duplicate section index, one for disagreeing sections
dup_rows = [(f, o) for f, o in FORMS if re.search(r"section index", f, re.I) and re.search(r"duplicate|twice|same|two sections", f, re.I)]
check("AC5", bool(dup_rows), "a duplicate section index has an invalid-form row")
for f, o in dup_rows:
    check("AC5", len([n for n in ALL_REASONS if re.search(r"\b" + n + r"\b", o)]) == 1,
          "the duplicate section index names exactly one outcome")
dis_rows = [(f, o) for f, o in FORMS if re.search(r"disagree|differ|different|conflict", f, re.I) and re.search(r"section count|manifest length", f, re.I)]
check("AC5", any(re.search(r"manifest length", f, re.I) for f, o in dis_rows),
      "sections that disagree on the manifest length have an invalid-form row")
check("AC5", any(re.search(r"section count", f, re.I) for f, o in dis_rows),
      "sections that disagree on the section count have an invalid-form row")
for f, o in dis_rows:
    check("AC5", len([n for n in ALL_REASONS if re.search(r"\b" + n + r"\b", o)]) == 1,
          f"the disagreement row '{f[:40]}' names exactly one outcome")

# --- AC6 and DoD: evidence tags, subsections, and the Markdown/TOML agreement ----------
for block_kind, text in prose_blocks(MAN_TEXT):
    if block_kind != "p":
        continue
    words = len(re.findall(r"\S+", text))
    if words >= 5:
        check("DoD", has_evidence(text), f"prose paragraph '{text.strip()[:50]}...' carries an evidence tag")
for title, body in subsections(MAN):
    if any(k == "t" for k, _ in prose_blocks(body)):
        check("DoD", has_evidence(body), f"subsection '{title}' with a table carries an evidence tag")
for p in OPEN_PARAS:
    check("DoD", re.search(r"\bowner\b", p, re.I) and re.search(r"\bfalsifier\b", p, re.I),
          f"open item '{p.strip()[:50]}...' names an owner and a falsifier")

toml_text = read_text(tables_path)
toml_root = None
if toml_text is None:
    check("AC6", False, f"{tables_path} does not exist")
else:
    try:
        toml_root = parse_toml(toml_text)
        check("AC6", True, "wire.toml parses with the TOML reader")
    except TomlError as error:
        check("AC6", False, f"wire.toml does not parse: {error}")
TOML_MSGS = (toml_root or {}).get("message", []) if isinstance(toml_root, dict) else []
REJECT_SET = set(REJECT)

# T2: the whole manifest above the bound needs a stated reason and an owner for the tooling gap
WHOLE_OK = any(re.search(r"schema|wire-check|checker", sent, re.I) and
               re.search(r"cannot|can not|does not|not expressible|no single|not declare", sent, re.I) and
               re.search(r"\b0\d\d-\d\d\b|\b004-02\b", sent)
               for p in PARAS_FLAT for sent in re.split(r"(?<=[.;:])\s+", p))
for k, (rows, offs, total, errs, where) in enumerate(MODELS):
    top = k == 0
    if total // 8 > BOUND:
        check("AC6", WHOLE_OK, f"{where} ({total // 8} bytes) is above the control bound: wire.toml cannot declare it "
                               "as one message, and the section says so with an owner for the tooling gap")
        continue
    sig_md = [(offs[i], rows[i]["bits"], rows[i]["maxc"]) for i in range(len(rows))]
    matches = []
    for m in TOML_MSGS:
        fields = m.get("field", []) if isinstance(m, dict) else []
        sig_t = [(f.get("offset"), f.get("width"), f.get("max_count")) for f in fields]
        if sig_t == sig_md:
            matches.append(m)
    check("AC6", len(matches) >= 1, f"{where} has a [[message]] in wire.toml with the same fields "
                                    f"(offsets, widths, maximum counts) ({len(matches)} found)")
    size = total // 8
    # A signature may also match a landed message; the declaration is the matching one whose
    # properties agree, and the first such one is the declaration of this table.
    good = [m for m in matches if m.get("layer") == "control" and m.get("dscp") == 40
            and m.get("max_size") == size
            and all(r in REJECT_SET for r in m.get("rejection_reasons", []))
            and all((f.get("max_count") is not None) == ("count" in f) for f in m.get("field", []))]
    check("AC6", bool(good), f"{where}: a matching toml message is layer control, dscp 40 (C-13), its max_size "
                             f"equals the computed {size} bytes, its reasons are Reject members, and each counted "
                             f"field carries both count and max_count")

# --- T2: the fixed part of the top-level table is declared in wire.toml, row for row ----------------
TOP_MODEL = None
if TOP_BYTES is not None:
    TOP_MODEL = next((m for m in TOP_CANDIDATES if m[2] // 8 == TOP_BYTES), None)
FIXED_MD = []
if TOP_MODEL is not None:
    for r in TOP_MODEL[0]:
        if r["maxc"] is not None:
            break
        FIXED_MD.append(r)
fixed_offs = tile(FIXED_MD)[0] if FIXED_MD else []
fixed_bytes = sum(r["bits"] for r in FIXED_MD) // 8
fixed_sig = [(fixed_offs[i], FIXED_MD[i]["bits"], None) for i in range(len(FIXED_MD))]
fixed_msgs = []
for m in TOML_MSGS:
    fields = m.get("field", []) if isinstance(m, dict) else []
    if [(f.get("offset"), f.get("width"), f.get("max_count")) for f in fields] == fixed_sig and fixed_sig:
        fixed_msgs.append(m)
check("AC6", len(fixed_msgs) >= 1,
      f"wire.toml declares the manifest fixed part as a [[message]] whose rows agree with the markdown fixed part "
      f"row for row ({len(fixed_msgs)} found)")
check("AC6", any(m.get("max_size") == fixed_bytes and m.get("layer") == "control" and m.get("dscp") == 40
                 for m in fixed_msgs),
      f"the fixed-part message has max_size equal to the markdown fixed-part sum ({fixed_bytes} bytes), layer control, dscp 40")

# --- T3 (a): the L range keeps 64 processors' declared delay inside the landed u32 ----------------------
L_ROWS = [r for r in ALL_ROWS if r["meaning"].lower().startswith("structural latency")]
hop_m = re.search(r"maximum hop count of (" + NUM + ")", wire)
sat_m = re.search(r"Saturates at (" + NUM + r") samples", wire)
L_hi = numeric_range(L_ROWS[0]["valid"])[1] if L_ROWS and numeric_range(L_ROWS[0]["valid"]) else None
if L_hi is not None and hop_m and sat_m:
    hop, sat = to_int(hop_m.group(1)), to_int(sat_m.group(1))
    check("AC1", hop * L_hi <= sat,
          f"the largest L ({L_hi} samples) times the landed maximum hop count ({hop}) is within the landed declared-delay "
          f"saturation value {sat}, so the sum cannot saturate (Measurement extension, Open item)")
else:
    check("AC1", False, "the L range, the landed maximum hop count and the landed saturation value are all readable")

# --- T3 (b): every sentinel stated in prose equals the table row and the toml field -------------------
E_SENT = {"none": REQ_E, "unknown": DEC_E}
TABLE_CODE = {}
for kind_, rows_ in E_SENT.items():
    for r in rows_:
        c_ = code_for(r["valid"], kind_)
        if c_ is not None:
            TABLE_CODE[kind_] = c_
            break
for kind_, other_ in (("none", "unknown"), ("unknown", "none")):
    if kind_ not in TABLE_CODE:
        continue
    prose_hex = set()
    for sent in re.split(r"(?<=[.;:])\s+", " ".join(PARAS_FLAT)):
        if re.search(r"\b" + kind_ + r"\b", sent, re.I) and re.search(r"\bE\b", sent):
            prose_hex |= {int(h, 16) for h in re.findall(r"0x[0-9a-fA-F]+", sent)}
    stray = prose_hex - {TABLE_CODE[kind_], TABLE_CODE.get(other_)}
    if prose_hex:
        check("AC2", TABLE_CODE[kind_] in prose_hex and not stray,
              f"the prose sentinel for {kind_} agrees with the table row ({hex(TABLE_CODE[kind_])}); "
              f"prose hex values {sorted(hex(x) for x in prose_hex)}")
    fixed_fields = [f for m in fixed_msgs for f in m.get("field", [])]
    row_bits = [r["bits"] for r in FIXED_MD]
    if fixed_msgs and FIXED_MD:
        for r, off in zip(FIXED_MD, fixed_offs):
            meaning_ = r["meaning"].lower()
            if (kind_ == "none" and meaning_.startswith("required e")) or (kind_ == "unknown" and meaning_.startswith("declared e")):
                tf = [f for f in fixed_msgs[0].get("field", []) if f.get("offset") == off]
                tcode = code_for(tf[0].get("valid", ""), kind_) if tf else None
                check("AC2", tcode == TABLE_CODE[kind_],
                      f"wire.toml's valid range for the {kind_} field states {hex(TABLE_CODE[kind_])} as the markdown does "
                      f"(read {tcode if tcode is None else hex(tcode)})")

# --- T3 (c): a stated maximum is never [measured]; a chosen maximum is tagged [decision] -----------------
MAXW = r"maximum|at[ -]most|\bmax\b|no more than|\bup to\b|\bto\s+\d"
bad_maxima = [s_ for s_ in re.split(r"(?<=[.;:])\s+", " ".join(PARAS_FLAT))
              if "[measured]" in s_ and re.search(MAXW, s_, re.I)]
bad_rows = [r for r in ALL_ROWS if "[measured]" in (r["meaning"] + " " + r["valid"]) and re.search(MAXW, r["meaning"] + " " + r["valid"], re.I)]
check("DoD", not bad_maxima and not bad_rows, "no stated maximum carries the [measured] tag")
chosen = [s_ for s_ in re.split(r"(?<=[.;:])\s+", " ".join(PARAS_FLAT))
          if re.search(r"\bchosen\b|not derived", s_, re.I) and re.search(MAXW, s_, re.I) and re.search(r"\d", s_)]
check("DoD", all("[decision]" in s_ for s_ in chosen),
      f"every sentence that calls a number chosen or not derived is tagged [decision] ({len(chosen)} found)")

# --- T3 (d): every reserved or accept-any manifest field is recorded in the landed Reserved fields table -----
RES_SEC = section_lines(wire, r"^Reserved fields$", 2) or []
RES_ROWS = []
for _, hdr_, rows_ in parse_tables("\n".join(RES_SEC)):
    if hdr_ and hdr_[0].lower().startswith("field"):
        RES_ROWS += rows_
TITLE_OF = {}
for title_, body_ in subsections(MAN):
    for _, hdr_, rows_ in parse_tables(body_):
        if is_field_table(hdr_):
            for rr in rows_:
                if len(rr) >= 5:
                    TITLE_OF[rr[3].strip()] = title_.lower()
RESERVED_MD = [r for r in ALL_ROWS if r["meaning"].startswith("Reserved") or r["valid"].strip().lower() == "0 only"
               or re.search(r"must be zero|accept any|ignores? any other", r["valid"], re.I)]
for r in RESERVED_MD:
    title_ = TITLE_OF.get(r["meaning"], "")
    key_ = title_.split()[0] if title_.split() else ""
    words = ["manifest", key_] + (["content"] if re.search(r"content", r["meaning"] + r["valid"], re.I) else [])
    found = any(all(w and w in (rr[0] + " " + rr[1]).lower() for w in words) for rr in RES_ROWS if len(rr) >= 2)
    check("DoD", found, f"the reserved or accept-any manifest field '{r['meaning'][:50]}' has a row in the landed "
                        f"Reserved fields table naming {words}")

# --- T3 (f): the audio-length bound on a stream's channel count -------------------------------------
AUD_M = re.search(r"frames × channels × 4, the samples that follow \|[^\n]*?at most (" + NUM + ")", wire)
AUD_MAX = to_int(AUD_M.group(1)) if AUD_M else None
aud_stated = AUD_MAX is not None and (
    any(re.search(r"channel", sent, re.I) and AUD_MAX in {to_int(x) for x in re.findall(NUM, sent)}
        for sent in re.split(r"(?<=[.;:])\s+", FLAT)) or
    any(AUD_MAX in {to_int(x) for x in re.findall(NUM, f + " " + o)} and re.search(r"channel|frames", f, re.I)
        for f, o in FORMS))
check("AC1", aud_stated, f"the manifest states the landed audio length bound ({AUD_MAX}) for frames × channels × 4, "
                         "or an invalid form for exceeding it")

# --- T3 (g): a sentence that waits for every section names what does not wait ------------------------
WAIT = re.compile(r"(judges?|wait|waits|waiting|decide|decides|hold|holds)[^.]{0,120}(every section|complete manifest)", re.I)
EXCEPT = re.compile(r"\bexcept\b|other refusals|without (the|a|its) manifest|(does|do|need) not wait", re.I)
for p in PARAS_FLAT:
    if WAIT.search(p):
        check("DoD", re.search(r"BadMemberList", p) is not None and EXCEPT.search(p) is not None,
              f"the paragraph that waits for every section names BadMemberList as waiting and what does not wait "
              f"('{p[:50]}...')")

# Landed tables unchanged, in the way 004-18's check holds them: the admission headers equal the Hello header
if HELLO_HEADER is not None and _adm is not None:
    for want in ("Admission request", "Admission accept", "Admission refusal", "Manifest section"):
        body = [b for t, b in subsections(_adm) if re.search(want, t, re.I)]
        if not body:
            check("DoD", False, f"landed '{want}' table is present")
            continue
        tabs = [t for t in parse_tables(body[0]) if is_field_table(t[1])]
        if not tabs:
            check("DoD", False, f"landed '{want}' has a field table")
            continue
        rows, _ = model_rows(tabs[0])
        head = [(r["off"], r["bits"], r["type"]) for r in rows[:7]]
        check("DoD", head == HELLO_HEADER, f"landed '{want}' header rows equal the Hello common control header")

# AC6: the repository-level checks, when the paths are not overridden
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        check("AC6", proc.returncode == 0 and "checks clean" in out,
              "cargo run --bin wire-check reports 'checks clean'")
    except (OSError, subprocess.TimeoutExpired) as error:
        check("AC6", False, f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result[-1]) if result else None
        ok = proc.returncode == 0 and mm is not None and int(mm.group(2)) == 0 and int(mm.group(1)) >= 12
        check("AC6", ok, "wire_tables has 0 failed and at least the 12 tests that stood before this ticket"
              + (f" (got: {result[-1]})" if result else ""))
    except (OSError, subprocess.TimeoutExpired) as error:
        check("AC6", False, f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC6: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

fails = RESULTS.count(False)
print(f"{fails} failure(s), {RESULTS.count(True)} pass(es)")
sys.exit(1 if fails else 0)
PY
rc=$((rc | $?))
exit $rc
