#!/usr/bin/env bash
# 004-18 — admission handshake messages and refusal reasons.
#
# RED-phase document check. It validates the "Admission messages" section of
# doc/design/spec/wire.md and the admission messages in doc/design/spec/wire.toml
# against the acceptance criteria of the ticket, and re-runs the wire-check
# binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either path is overridden the
# cargo checks are skipped with a NOTE, because they always target the real
# repository.
#
# Numbers are read at run time: the common control header and the admission
# type-code block from the landed Hello section, the 1 252-byte bound (C-17), the
# refusal enumeration of C-185 (its 18 names are read, not copied), the packet
# enumeration of wire.md's Rejection reasons, and the current A width of the
# Receiver report (004-20). Landed sections are read, never rewritten, so the
# Hello section is held unchanged structurally: an admission header that
# differs from it, or a type code outside the block its table gives, fails.
#
# The author contract (headings and labels the check locates):
#
#   ## Admission messages      prose: the handshake's behaviour restated as fact
#       (IPv4 unicast, C-100; CS5 (40), C-13; C-17; C-45 Pending has no timeout;
#       C-103, C-142, C-185, C-209, C-219 cited); the refusal enumeration;
#       the C-115 reservation values; "discover" beside the three names
#       VersionMismatch, NameTaken and ModuleCap.
#   ### <message title>        one per admission message; its table starts with the
#       seven common control header rows in the hello's order and widths; its
#       Message type row states a code inside the admission block; the rows tile;
#       a sentence "maximum size ... N bytes" equals the sum (counted rows at
#       their maximum count); the refusal-reason row is a u8.
#   refusal table               rows of a number and a CamelCase name, one per
#       reason of C-185 (18), each with a unique value; the three discovery-raised
#       rows say so.
#   ### Validity rules          a Form / Outcome table shared by the messages, and
#       per-message Form / Outcome tables.
#   Open item paragraph         owner 004 with epic 005 confirming, and the
#       falsifier "epic 005's admission state machine needs a message or field the
#       set lacks".
#   wire.toml                   one [[message]] per admission message (layer control,
#       dscp 40, max_size, one [[message.field]] per row, rejection_reasons naming
#       packet reasons only) and a top-level array holding the refusal enumeration
#       with the same names and values as wire.md.
#
# What is checked, per acceptance criterion:
#   AC 1  every Context item is a field in an admission message: declared rates in
#         bit/s (at least the bits of the C-219 figure of a 1 472-byte packet plus
#         66 bytes at 2 000 packets/s), packets/s, slice rate, the bundle member
#         list (maximum count 32), the reverse measurement port (16 bits), pinned
#         addresses (32 bits each, IPv4), a reference to 004-26 (manifest) and to
#         004-27 (probe pair), the downstream A (C-142); type codes distinct and
#         inside the admission block; common header rows; a refusal-reason u8.
#   AC 2  the refusal enumeration is exactly the 18 names of C-185, with unique
#         numeric values; VersionMismatch, NameTaken and ModuleCap are marked as
#         raised at discovery and never carried by an admission refusal.
#   AC 3  the downstream A has a unit (nanoseconds) and the width of the consumer's
#         current A in the receiver report.
#   AC 4  every message's maximum size equals its table sum and is at most C-17's
#         bound.
#   AC 5  no admission field is a timeout or a Pending timer; C-45 is cited.
#   AC 6  at least 8 invalid forms across the set, at least 6 per message (the
#         shared table and its own); the ticket's categories are covered; exact
#         reasons where the ticket fixes them (BadMemberList for member forms,
#         the packet reasons for the generic violations); a discovery-raised
#         reason is never a refusal outcome; no outcome names a reason outside the
#         two closed enumerations.
#   AC 7  wire.toml declares each admission message (layer control, dscp 40 by C-13,
#         max_size, field names, offsets and widths, rejection_reasons); the
#         refusal enumeration in wire.toml equals wire.md's; when the paths are not
#         overridden: `cargo run --bin wire-check` reports "checks clean" and
#         `cargo test -p hadronyche-tools --test wire_tables` has 0 failed and at
#         least 12 passed (the count that stood before 004-18).
#   DoD   every prose paragraph of 5+ words carries an evidence tag; every table's
#         subsection has one; a paragraph starting "Open item" names an owner and a
#         falsifier.
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


NUM = r"(?:\d{1,3}(?:[  ]\d{3})+|\d+)"


def to_int(s):
    return int(re.sub(r"[  ]", "", s))


# ---------------------------------------------------------------------------
# Markdown helpers (conventions of checks/004-19.sh and checks/004-20.sh)
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




# ---------------------------------------------------------------------------
# Constants (doc/design/02-constants.md wins; read at run time)
# ---------------------------------------------------------------------------
CONST_TEXT = read_text(CONSTANTS_PATH) or ""


def const_val(cid):
    for line in CONST_TEXT.splitlines():
        if line.startswith(f"| {cid} |"):
            cells = line.split("|")
            return cells[3].strip() if len(cells) > 3 else ""
    return ""


def const_int(cid, pattern):
    m = re.search(pattern, const_val(cid), re.I)
    return to_int(m.group(1)) if m else None


def const_dur(cid):
    m = re.search(r"(\d+(?:\.\d+)?)\s*(ms|s)\b", const_val(cid))
    return (m.group(1), m.group(2)) if m else None


K = {
    "bound": const_int("C-17", r"at\s+most\s+(" + NUM + r")\s*bytes"),
    "dscp_control": const_int("C-13", r"control\s+CS5\s*\((\d+)\)"),
    "peers_max": const_int("C-34", r"at\s+most\s+(\d+)"),
}
for _key, _value in K.items():
    if _value is None:
        fail("DoD", f"constant for {_key} could not be read from {CONSTANTS_PATH}")
if K["bound"] is None:
    K["bound"] = 0
DUR = {"lbc": const_dur("C-32"), "window": const_dur("C-33"), "silence": const_dur("C-37")}
for _key, _value in DUR.items():
    if _value is None:
        fail("Behaviour", f"constant for {_key} could not be read from {CONSTANTS_PATH}")

# ---------------------------------------------------------------------------
# Markdown: sections, the closed enumeration, the landed Hello section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    fail(1, f"{wire_path} does not exist")
    wire = ""
lines = wire.splitlines()
HEADS = heading_index(lines)


def section_span(pattern, level=None):
    for i, (idx, lvl, title) in enumerate(HEADS):
        if re.search(pattern, title, re.I) and (level is None or lvl == level):
            end = len(lines)
            for j in range(i + 1, len(HEADS)):
                if HEADS[j][1] <= lvl:
                    end = HEADS[j][0]
                    break
            return idx + 1, end
    return None


MEMBERS = {}
_span = section_span(r"rejection[-\s]+reasons?")
if _span is None:
    fail("AC4", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read")
else:
    for _, _, _rows in parse_tables("\n".join(lines[_span[0]:_span[1]])):
        for _row in _rows:
            if len(_row) >= 2 and re.fullmatch(r"\d+", _row[0].strip()):
                _name = _row[1].strip().strip("`*_ ")
                if re.fullmatch(r"[A-Z][A-Za-z0-9]+", _name):
                    MEMBERS[_name] = int(_row[0].strip())
    if len(MEMBERS) != 13:
        fail("AC4", f"the closed enumeration parsed from wire.md has {len(MEMBERS)} names, not 13")
CAMEL = re.compile(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b")

KEY_RULES = [("magic", r"^magic"), ("version", r"^(protocol\s+)?version"), ("domain", r"^domain"),
             ("type", r"^message\s+type"), ("scheme", r"^(authentication\s+)?scheme"),
             ("counter", r"^(message\s+)?counter"), ("tag", r"^(authentication\s+)?tag")]


def key_of(meaning):
    m = clean_start(meaning).lower()
    for k, rx in KEY_RULES:
        if re.search(rx, m):
            return k
    return None


def md_name(meaning):
    m = re.match(r"\s*([A-Za-z][A-Za-z ]*[A-Za-z]|[A-Za-z])", meaning)
    return m.group(1) if m else meaning


def norm_name(text):
    return re.sub(r"[\s_]+", " ", text.replace("-", " ").lower()).strip()


def row_in(fl, pat, bits=None):
    for f in fl:
        if re.match(r"\s*" + pat, f["meaning"], re.I) and (bits is None or f["bits"] == bits):
            return f
    return None


_hs = section_span(r"^hello$", level=2)
if _hs is None:
    fail(1, "wire.md has no '## Hello' section")
HELLO_LINES = lines[_hs[0]:_hs[1]] if _hs else []
HELLO_SUBS = subsections(HELLO_LINES)
_hdr_sub = find_sub(HELLO_SUBS, r"common\s+control\s+header")
HDR = fields_of(first_table(_hdr_sub[0][1], is_field_table) if _hdr_sub else None, "H",
                "the hello's common control header")
HDR_KEYS = [key_of(f["meaning"]) for f in HDR]
_lay_sub = find_sub(HELLO_SUBS, r"layout")
LAY = fields_of(first_table(_lay_sub[0][1], is_field_table) if _lay_sub else None, "H",
                "the hello layout")
HELLO_TYPE = None
HELLO_EPOCH_BITS = None
for _f in LAY:
    if key_of(_f["meaning"]) == "type" and HELLO_TYPE is None:
        _m = re.search(r"\d+", _f["valid"])
        HELLO_TYPE = int(_m.group(0)) if _m else None
    if re.match(r"\s*election\s+epoch\b", _f["meaning"], re.I):
        HELLO_EPOCH_BITS = _f["bits"]
if HELLO_EPOCH_BITS is None:
    fail(3, "the hello layout has no election epoch row to compare the election messages' width with")

# A small TOML reader (copied from checks/004-20.sh: offline, Python 3.10, no tomllib).
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
# 004-18 additions: the Admission messages section (helpers copied from
# checks/004-17.sh where noted; the refusal enumeration is read from C-185)
# ---------------------------------------------------------------------------
TIMER = re.compile(r"timer|interval|timeout|time[\s-]*out|listen|window|silence|period|expir|deadline|"
                   r"backoff|delay|pending|\bms\b|seconds?\b", re.I)
NON_REJECTION = re.compile(r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|"
                           r"cannot\s+occur|ignored|never\s+a\s+miss|not\s+a\s+refusal", re.I)
DISCOVERY = ["VersionMismatch", "NameTaken", "ModuleCap"]

# The packet enumeration (MEMBERS, 13 rows of Rejection reasons) is read above.
# The refusal enumeration is the closed set of C-185, read from the constants:
# its names are the CamelCase words of the list before the first sentence end,
# with the parenthetical glosses removed.
_seg = const_val("C-185").split(". ")[0]
_seg = re.sub(r"\([^)]*\)", "", _seg)
CAMEL = re.compile(r"\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b")
REFUSAL_SET = []
for _n in CAMEL.findall(_seg):
    if _n not in REFUSAL_SET:
        REFUSAL_SET.append(_n)
if len(REFUSAL_SET) != 18:
    fail("AC2", f"C-185 names {len(REFUSAL_SET)} refusal reasons; the closed set is 18")
for _n in DISCOVERY:
    if _n not in REFUSAL_SET:
        fail("AC2", f"discovery-raised reason {_n} is not in the C-185 set")
UNION = set(MEMBERS) | set(REFUSAL_SET)

# Locate the section, its prose, its subsections
_as = section_span(r"^admission\s+messages?$", level=2)
if _as is None:
    fail(1, "wire.md has no '## Admission messages' section")
AD_LINES = lines[_as[0]:_as[1]] if _as else []
AD = "\n".join(AD_LINES)
AD_PLAIN = strip_fences(AD)
AD_SUBS = subsections(AD_LINES)
_adtop = []
for _ln in AD_LINES:
    if re.match(r"^###\s", _ln):
        break
    _adtop.append(_ln)
AD_TOP = "\n".join(_adtop)
AD_PARAS = [t for k, t in prose_blocks(AD) if k == "p"]


def ad_para(*pats):
    for t in AD_PARAS:
        if all(re.search(p, t, re.I) for p in pats):
            return t
    return None


# AC 2: the numbered refusal enumeration (any table of the section whose rows
# are a number and a CamelCase name)
REF = {}
REF_ROW = {}
for _, _hdr, _rows in parse_tables(AD_PLAIN):
    for _row in _rows:
        if len(_row) >= 2 and re.fullmatch(r"\d+", _row[0].strip()):
            _nm = _row[1].strip().strip("`*_ ")
            if re.fullmatch(r"[A-Z][A-Za-z0-9]+", _nm):
                if _nm in REF:
                    fail(2, f"refusal reason {_nm} is listed twice")
                REF[_nm] = int(_row[0].strip())
                REF_ROW[_nm] = " | ".join(_row)
if not REF:
    fail(2, "the Admission messages section has no numbered refusal enumeration (value, name rows)")
if set(REF) != set(REFUSAL_SET):
    fail(2, f"the refusal enumeration names differ from C-185: missing {sorted(set(REFUSAL_SET) - set(REF))}, "
         f"extra {sorted(set(REF) - set(REFUSAL_SET))}")
if len(set(REF.values())) != len(REF):
    fail(2, "the refusal reason values are not unique")
for _n, _v in REF.items():
    if not 0 <= _v <= 255:
        fail(2, f"refusal reason {_n} has value {_v}, outside a u8")
for _n in DISCOVERY:
    if _n in REF and not re.search(r"discover", REF_ROW[_n], re.I):
        fail(2, f"refusal reason {_n} is not marked as raised at discovery in its row")
for _n in REF:
    if _n not in DISCOVERY and re.search(r"discover", REF_ROW[_n], re.I):
        fail(2, f"refusal reason {_n} is marked as discovery-raised; only {DISCOVERY} are (C-185)")
if not any(all(re.search(re.escape(n), t) for n in DISCOVERY) and re.search(r"discover", t, re.I)
           for t in AD_PARAS):
    fail(2, "no sentence in the Admission messages section says that VersionMismatch, NameTaken and ModuleCap "
         "are raised at discovery and never carried by an admission refusal (C-185)")
if not ad_para(r"C-185"):
    fail(2, "no sentence in the Admission messages section cites C-185 for the refusal enumeration")

# AC 2 (review finding C3): the refusal values are 0..17 in the order of the closed set of C-185
# (read from the constants file; the list is not copied here)
for _i, _n in enumerate(REFUSAL_SET):
    if REF.get(_n) != _i:
        fail(2, f"wire.md gives refusal reason {_n} the value {REF.get(_n)}; C-185's order gives {_i} "
             "(the refusal values are 0..17 in the order of the closed set)")

# Admission messages: field tables and invalid-form tables
ADM_MSGS = []
ADM_SHARED = []


def form_rows(text):
    out = []
    for _, header, rows in parse_tables(text):
        h = [c.lower() for c in header]
        if any("form" in c or "invalid" in c for c in h) and any("outcome" in c for c in h):
            fi = next(i for i, c in enumerate(h) if "form" in c or "invalid" in c)
            oi = next(i for i, c in enumerate(h) if "outcome" in c)
            for r in rows:
                if len(r) > max(fi, oi):
                    out.append((r[fi], r[oi]))
    return out


for _title, _text in AD_SUBS:
    _tb = first_table(_text, is_field_table)
    if _tb is not None and not re.search(r"header", _title, re.I):
        ADM_MSGS.append({"title": _title, "text": _text, "fields": fields_of(_tb, 2, f"'{_title}'"),
                         "forms": form_rows(_text), "total": 0, "code": None})
    else:
        ADM_SHARED.extend(form_rows(_text))
TITLES = [m["title"] for m in ADM_MSGS]
if len(ADM_MSGS) < 2:
    fail(1, "the Admission messages section needs at least two messages with a field table (a request and a refusal)")

# AC 1: type codes inside the admission block the Hello section gives
ADM_LO, ADM_HI = None, None
for _title, _text in find_sub(HELLO_SUBS, r"type\s+codes"):
    for _, _, _rows in parse_tables(_text):
        for _row in _rows:
            if _row and len(_row) > 1 and re.search(r"admission", _row[1], re.I):
                _nums = [int(x) for x in re.findall(r"\d+", _row[0])]
                if _nums:
                    ADM_LO, ADM_HI = _nums[0], _nums[-1]
if ADM_LO is None:
    fail(1, "the Hello section's Control message type codes table does not give the admission block")
    ADM_LO, ADM_HI = 0, -1

# AC 1 and AC 4 per message: header rows, tiling, maximum size, code; AC 2 per message: C-115
ADM_CODES = {}
for m in ADM_MSGS:
    t = m["title"]
    fl = m["fields"]
    if len(fl) < 7 or [key_of(f["meaning"]) for f in fl[:7]] != HDR_KEYS:
        fail(1, f"'{t}': its table does not start with the common control header rows in the hello's order {HDR_KEYS}")
    else:
        for a, b in zip(fl[:7], HDR):
            if (a["off"], a["bits"], a["type"]) != (b["off"], b["bits"], b["type"]):
                fail(1, f"'{t}': header row '{b['meaning'][:30]}' differs from the common control header in offset, width or type")
    cur = 0
    for f in fl:
        if f["off"] != cur:
            fail(1, f"'{t}': '{f['meaning'][:40]}' at byte {f['off']}, expected {cur} (rows must tile)")
        if size_bits(f) % 8:
            fail(1, f"'{t}': '{f['meaning'][:40]}' is not a whole number of bytes")
        cur = f["off"] + size_bits(f) // 8
    m["total"] = cur
    _ms = re.search(r"maximum\s+size[^.;\n]*?(" + NUM + r")\s*bytes", strip_fences(m["text"]), re.I)
    if _ms is None:
        fail(4, f"'{t}': no sentence states its maximum size in bytes ('maximum size ... N bytes')")
    elif to_int(_ms.group(1)) != cur:
        fail(4, f"'{t}': states a maximum size of {to_int(_ms.group(1))} bytes; its table sums to {cur} "
             "(counted rows at their maximum counts)")
    if K["bound"] and cur > K["bound"]:
        fail(4, f"'{t}': its maximum size {cur} bytes is above the {K['bound']}-byte control bound (C-17)")
    _tr = [f for f in fl if key_of(f["meaning"]) == "type"]
    code = None
    if _tr:
        _mc = re.search(r"\d+", _tr[0]["valid"])
        code = int(_mc.group(0)) if _mc else None
    if code is None:
        fail(1, f"'{t}': the Message type row's valid range does not state the message's type code")
    else:
        if not (ADM_LO <= code <= ADM_HI) or code == HELLO_TYPE:
            fail(1, f"'{t}': type code {code} is outside the admission block ({ADM_LO} to {ADM_HI}) "
                 "the Hello section gives, or is the hello's code")
        if code in ADM_CODES:
            fail(1, f"'{t}' and '{ADM_CODES[code]}' share type code {code}; the codes must be distinct")
        ADM_CODES[code] = t
    m["code"] = code
    if len(fl) >= 7:
        if not (re.search(r"\b0\b", fl[4]["valid"]) and re.search(r"only|zero|=\s*0", fl[4]["valid"], re.I)):
            fail(2, f"'{t}': the authentication scheme row does not fix the v1 value 0 (C-115)")
        if not (re.search(r"\b0\b", fl[6]["valid"]) and re.search(r"only|zero|=\s*0", fl[6]["valid"], re.I)):
            fail(2, f"'{t}': the authentication tag row does not fix the v1 value of 128 zero bits (C-115)")
        if not re.search(r"\blive\b|liveness", fl[5]["meaning"], re.I):
            fail(2, f"'{t}': the message counter row does not say it is live (C-115)")
if ADM_MSGS and not ad_para(r"C-115", r"scheme|tag|reservation"):
    fail(2, "no sentence in the Admission messages section cites C-115 for the v1 values of the reservation")

# AC 2 and AC 1 (refusal message): a message carries the refusal reason when refused, as a u8 whose
# valid range does not carry a discovery-raised name
REF_FIELDS = [(m, f) for m in ADM_MSGS for f in m["fields"] if re.search(r"refusal\s+reason", f["meaning"], re.I)]
if not REF_FIELDS:
    fail(1, "no admission message carries a refusal reason field")
for m, f in REF_FIELDS:
    if f["bits"] != 8:
        fail(1, f"'{m['title']}': the refusal reason is {f['bits']} bits; a u8 value of the enumeration")
    for n in DISCOVERY:
        cell = f["valid"]
        if re.search(r"\b%s\b" % n, cell) and not re.search(r"\b(not|never|except|discover\w*|raised)\b", cell, re.I):
            fail(2, f"'{m['title']}': the refusal reason's valid range names {n}, which is never carried by an admission refusal")

# AC 1: every item of the Context list is a field in at least one admission message
ALL_ROWS = [(m, f) for m in ADM_MSGS for f in m["fields"]]


def rows_of(pred):
    return [(m, f) for m, f in ALL_ROWS if pred(f)]


def both(f):
    return f["meaning"] + " " + f["valid"]


# declared rates: bit/s wide enough for the C-219 figure of a controller stream and an audio stream (004-15)
NEED_BPS = (1472 + 66) * 8 * 2000
BPS_BITS = NEED_BPS.bit_length()
RATE_BPS = rows_of(lambda f: re.search(r"bit\s*/\s*s|bits?\s+per\s+second", both(f), re.I)
                   and re.search(r"declared|rate", f["meaning"], re.I))
if not RATE_BPS:
    fail(1, "no admission field carries a declared rate in bit/s (Context: declared rates)")
for m, f in RATE_BPS:
    if f["bits"] < BPS_BITS:
        fail(1, f"'{m['title']}': the declared rate in bit/s is {f['bits']} bits; the C-219 figure of a 1 472-byte "
             f"packet plus 66 bytes at 2 000 packets/s ({NEED_BPS} bit/s) needs {BPS_BITS}")
RATE_PPS = rows_of(lambda f: re.search(r"packets?\s*/\s*s|packets?\s+per\s+second", both(f), re.I))
if not RATE_PPS:
    fail(1, "no admission field carries a declared rate in packets/s")
for m, f in RATE_PPS:
    if f["bits"] < 16:
        fail(1, f"'{m['title']}': the packets/s rate is {f['bits']} bits; at least 16 are needed for 2 000 packets/s")
RATE_SLICE = rows_of(lambda f: re.search(r"slice\s+rate", f["meaning"], re.I))
if not RATE_SLICE:
    fail(1, "no admission field carries a slice rate (Context: declared rates)")
for m, f in RATE_SLICE:
    if f["bits"] < 16:
        fail(1, f"'{m['title']}': the slice rate is {f['bits']} bits; at least 16 are needed for 2 000 slices/s (C-02)")
if not ad_para(r"C-219"):
    fail(1, "no sentence in the Admission messages section cites C-219 for the declared rates")

# bundle member list with a maximum count of 32 (C-08, C-209)
if not rows_of(lambda f: re.search(r"member", f["meaning"], re.I) and f["maxc"] == 32):
    fail(1, "no admission field lists bundle members with a maximum count of 32 (C-08, C-209)")

# reverse measurement port: a 16-bit port
if not rows_of(lambda f: re.search(r"reverse", f["meaning"], re.I) and re.search(r"port", f["meaning"], re.I)
               and f["bits"] == 16):
    fail(1, "no admission field carries the reverse measurement port as a 16-bit port")

# pinned addresses: IPv4 only (C-100), 4 bytes each
ADDR = rows_of(lambda f: re.search(r"address", f["meaning"], re.I))
if not ADDR:
    fail(1, "no admission field carries a pinned address")
for m, f in ADDR:
    if f["bits"] != 32:
        fail(1, f"'{m['title']}': the pinned address is {f['bits']} bits; an IPv4 address is 4 bytes (C-100)")
if not (re.search(r"IPv4", AD_PLAIN) and "C-100" in AD_PLAIN):
    fail("Behaviour", "unicast over IPv4 (C-100) is not restated in the Admission messages section")
if not (re.search(r"CS5\s*\(40\)", AD_PLAIN) and "C-13" in AD_PLAIN):
    fail("Behaviour", "DSCP CS5 (40) with C-13 is not restated in the Admission messages section")

# the manifest (004-26) and the path-MTU probe pair (004-27), by reference
if not rows_of(lambda f: re.search(r"manifest", both(f), re.I) and re.search(r"004-26", both(f))):
    fail(1, "no admission field refers to the manifest laid out in 004-26")
if not rows_of(lambda f: re.search(r"probe", both(f), re.I) and re.search(r"004-27", both(f))):
    fail(1, "no admission field refers to the path-MTU probe pair laid out in 004-27")

# downstream A (C-142): width and unit, and the width of the consumer's current A (004-20)
DA = rows_of(lambda f: re.search(r"downstream", f["meaning"], re.I) and re.search(r"\bA\b", f["meaning"]))
if not DA:
    fail(1, "no admission field carries the downstream A of C-142")
_rr = section_span(r"receiver\s+report")
CUR_A_BITS = None
if _rr is None:
    fail(1, "wire.md has no receiver report section: the downstream A width cannot be tied to the consumer's current A")
else:
    for _ti in parse_tables("\n".join(lines[_rr[0]:_rr[1]])):
        if not is_field_table(_ti[1]):
            continue
        for _f in fields_of(_ti, 1, "receiver report"):
            if re.search(r"current\s+a\b", _f["meaning"], re.I):
                CUR_A_BITS = _f["bits"]
if CUR_A_BITS is None and _rr is not None:
    fail(1, "the receiver report has no current A row (004-20) to tie the downstream A width to")
for m, f in DA:
    if not re.search(r"\bns\b|nanosec", both(f), re.I):
        fail(1, f"'{m['title']}': the downstream A has no unit (nanoseconds)")
    if CUR_A_BITS is not None and f["bits"] != CUR_A_BITS:
        fail(1, f"'{m['title']}': the downstream A is {f['bits']} bits; the consumer's current A (004-20) is {CUR_A_BITS}")
    if f["bits"] <= 0:
        fail(1, f"'{m['title']}': the downstream A has no width")

# AC 5: no admission field is a timeout or Pending timer; C-45 is cited
for m in ADM_MSGS:
    for f in m["fields"]:
        if TIMER.search(md_name(f["meaning"])):
            fail(5, f"'{m['title']}': the field '{md_name(f['meaning'])}' is a timer; C-45 gives Pending no timeout")
if "C-45" not in AD_PLAIN:
    fail(5, "C-45 (Pending has no timeout) is not cited in the Admission messages section")

# Rulings the section must cite (settled rulings of the ticket)
for _c in ("C-17", "C-103", "C-142", "C-185", "C-209", "C-219"):
    if _c not in AD_PLAIN:
        fail("Settled", f"{_c} is not cited in the Admission messages section")

# C1 (review finding): in the request's own invalid-form table, every row whose outcome is a drop
# (decided from the datagram's own bytes) comes before every row whose outcome is a refusal (which
# depends on receiver state); the order among refusals is handed to 005-14.
_req = [m for m in ADM_MSGS if re.search(r"request", m["title"], re.I)]
if not _req:
    fail(6, "no admission request message: its invalid-form table cannot be checked for drops before refusals")
else:
    _kinds = []
    for f_, o_ in _req[0]["forms"]:
        if re.search(r"\bdrop\b|\bignore\b", o_, re.I):
            _kinds.append(("drop", f_))
        elif re.search(r"refus", o_, re.I):
            _kinds.append(("refusal", f_))
    if any(k == "drop" for k, _ in _kinds) and any(k == "refusal" for k, _ in _kinds):
        _last_drop = max(i for i, (k, _) in enumerate(_kinds) if k == "drop")
        _first_ref = min(i for i, (k, _) in enumerate(_kinds) if k == "refusal")
        if _last_drop > _first_ref:
            fail(6, "the admission request's invalid-form table lists a refusal before a drop: every drop "
                 "(decided from the datagram's bytes) must come before every refusal (receiver state, C-164)")
if not ad_para(r"005-14", r"order"):
    fail(6, "no sentence in the Admission messages section hands the order among refusals to 005-14")

# C6 (review finding): the ForeignDomain refusal row, or a prose sentence (not an Open item paragraph),
# cites an open item by number or singular 'Open item'; a plural 'Open items' section is not a citation.
_OPEN_ITEM = re.compile(r"open\s+item(?!s)", re.I)
_fd_cited = False
for _, _h, _rows in parse_tables(AD_PLAIN):
    for _row in _rows:
        if any(c.strip().strip("`*_ ") == "ForeignDomain" for c in _row) and _OPEN_ITEM.search(" | ".join(_row)):
            _fd_cited = True
for _p in AD_PARAS:
    if re.search(r"\bForeignDomain\b", _p) and _OPEN_ITEM.search(_p) \
            and not re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(_p), re.I):
        _fd_cited = True
if not _fd_cited:
    fail(6, "the ForeignDomain refusal row (or a sentence naming it) does not cite an open item by number "
         "('Open item N'): its 'where raised' cell must say that discovery is epic 005's and admission is per 012-06, "
         "with the open item that settles the conflict")

# Hand-off (review C3/C4): the section names 012-01, which fixes the refusal discriminants
if not re.search(r"012-01", AD_PLAIN):
    fail("Hand-off", "the Admission messages section does not mention 012-01, which fixes RefuseReason's discriminants "
         "(0..=17 in the order of C-185)")

# AC 6: invalid forms with outcomes
ADM_EXACT = {"short": "TooShort", "magic": "BadMagic", "version": "VersionMismatch", "domain0": "UnconfiguredDomain",
             "foreign": "ForeignDomain", "oversize": "OversizeControl", "length": "BadLength",
             "reserved": "ReservedNonZero"}


def classify(form):
    f = form.lower()
    if re.search(r"refus", f) and re.search(r"VersionMismatch|NameTaken|ModuleCap|version\s+mismatch|name\s+taken|"
                                            r"module\s+cap", form, re.I):
        return "discovery"
    if re.search(r"refus", f) and re.search(r"unknown|outside|not\s+in|not\s+a\s+(known|member|valid)", f):
        return "unknown_reason"
    if "downstream" in f:
        return "downstream"
    if "address" in f:
        return "address"
    if "member" in f or "duplicate" in f:
        return "member"
    if re.search(r"\brate", f):
        return "rate"
    if re.search(r"too\s+short|shorter|fewer\s+than", f):
        return "short"
    if "magic" in f:
        return "magic"
    if "version" in f:
        return "version"
    if re.search(r"domain\s*(id\s*)?0\b|unconfigured", f):
        return "domain0"
    if re.search(r"foreign|different\s+non-?zero", f):
        return "foreign"
    if re.search(r"oversiz|larger\s+than", f):
        return "oversize"
    if re.search(r"\bsize\b|length", f):
        return "length"
    if re.search(r"reserved|non-?zero|scheme|\btag\b", f):
        return "reserved"
    return None


def named_in(outcome):
    found = []
    for m_ in re.finditer(r"\b[A-Za-z]+\b", outcome):
        w = m_.group(0)
        if w in UNION:
            before = outcome[max(0, m_.start() - 24):m_.start()]
            if re.search(r"\b(not|never|nor|no|neither)\b[^|;.]*$", before, re.I):
                continue
            found.append(w)
    return found


CATS_SEEN = set()


def check_form(form, outcome, where):
    for mm in re.findall(r"count\s+under\s+`?([A-Za-z]+)`?", outcome):
        if mm not in UNION:
            fail(6, f"{where}: the form '{form[:50]}' counts under '{mm}', which is in neither closed enumeration")
    for name in CAMEL.findall(outcome):
        if name not in UNION and not re.search(r"\b(not|never|nor|no)\b[^|;.]*" + re.escape(name), outcome):
            fail(6, f"{where}: the outcome names '{name}', which is in neither enumeration: no invented reason")
    cat = classify(form)
    if cat:
        CATS_SEEN.add(cat)
    named = named_in(outcome)
    if cat in ADM_EXACT:
        if not re.search(r"\b" + ADM_EXACT[cat] + r"\b", outcome):
            fail(6, f"{where}: the form '{form[:50]}' must be counted under {ADM_EXACT[cat]}: {outcome[:70]}")
    if cat == "member" and re.search(r"32|unknown|duplicate", form, re.I):
        if "BadMemberList" not in outcome:
            fail(6, f"{where}: the member form '{form[:50]}' must be refused with BadMemberList (C-173, C-185)")
    if cat == "discovery" and any(n in named for n in DISCOVERY):
        fail(6, f"{where}: the form '{form[:50]}' is refused with a discovery-raised reason: "
             "VersionMismatch, NameTaken and ModuleCap are never an admission refusal")
    if not (named or NON_REJECTION.search(outcome)):
        fail(6, f"{where}: the form '{form[:50]}' names no member of the closed enumerations and argues no non-rejection")


for m in ADM_MSGS:
    total_forms = len(ADM_SHARED) + len(m["forms"])
    if total_forms < 6:
        fail(6, f"'{m['title']}' has {total_forms} invalid forms with outcomes (the shared table and its own); at least 6")
    for f_, o_ in m["forms"]:
        check_form(f_, o_, f"'{m['title']}'")
for f_, o_ in ADM_SHARED:
    check_form(f_, o_, "the shared Validity rules")
ALL_FORMS = ADM_SHARED + [x for m in ADM_MSGS for x in m["forms"]]
if len(ALL_FORMS) < 8:
    fail(6, f"the admission set has {len(ALL_FORMS)} invalid forms with outcomes; at least 8 are required (AC 6)")
for _c in ("short", "magic", "version", "foreign", "length", "reserved", "oversize", "domain0", "rate", "member",
           "address", "unknown_reason", "discovery", "downstream"):
    if _c not in CATS_SEEN:
        fail(6, f"no invalid form covers the '{_c}' case of the ticket's list")

# Open item: owner (004, confirmed by 005) and the falsifier
_opens = [t for t in AD_PARAS if re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(t), re.I)]
if not any(re.search(r"owner", t, re.I) and re.search(r"\b004\b", t) and re.search(r"\b005\b", t)
           and re.search(r"falsifier", t, re.I) and re.search(r"needs\s+a\s+message\s+or\s+field", t, re.I)
           for t in _opens):
    fail("DoD", "no paragraph starting 'Open item' in the Admission messages section names owner 004, epic 005 as "
         "the confirming owner, and the falsifier 'epic 005's admission state machine needs a message or field the set lacks'")

# DoD: evidence tags, and owners and falsifiers on open items
for title, text in [("(section)", AD_TOP)] + AD_SUBS:
    has_tag_here = has_evidence(text)
    for kind, body in prose_blocks(text):
        if kind == "p":
            words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", body)))
            if words >= 5 and not has_evidence(body):
                fail("DoD", f"a prose paragraph of '{title}' carries no evidence tag: {body.splitlines()[0][:80]}")
            if re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), re.I):
                if not re.search(r"owner", body, re.I) or not re.search(r"falsifier", body, re.I):
                    fail("DoD", f"an open item in '{title}' names no owner or no falsifier: {body.splitlines()[0][:80]}")
        elif kind == "t" and not has_tag_here:
            fail("DoD", f"a table of '{title}' follows no evidence-tagged paragraph")

# AC 7 / wire.toml: each admission message declared, and the refusal enumeration agreeing with wire.md
def tname(text):
    return re.sub(r"\s+messages?$", "", norm_name(text))


PRIM = {"u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"}
tables_text = read_text(tables_path)
toml = None
if tables_text is None:
    fail(7, f"{tables_path} does not exist")
else:
    try:
        toml = parse_toml(tables_text)
    except TomlError as error:
        fail(7, f"{tables_path} does not parse: {error}")
msgs_t = toml.get("message", []) if isinstance(toml, dict) else []
if not isinstance(msgs_t, list):
    msgs_t = []
for m in ADM_MSGS:
    t = m["title"]
    cands = [x for x in msgs_t if isinstance(x, dict) and tname(str(x.get("name", ""))) == tname(t)]
    if not cands:
        fail(7, f"wire.toml declares no [[message]] for the admission message '{t}'")
        continue
    T = cands[0]
    if T.get("layer") != "control":
        fail(7, f"toml message '{T.get('name')}' has layer '{T.get('layer')}', not 'control'")
    if K["dscp_control"] is not None and T.get("dscp") != K["dscp_control"]:
        fail(7, f"toml message '{T.get('name')}' has dscp {T.get('dscp')}; C-13 fixes CS5 ({K['dscp_control']})")
    if T.get("max_size") != m["total"]:
        fail(7, f"toml message '{T.get('name')}' has max_size {T.get('max_size')}; the markdown table sums to {m['total']}")
    tf = [x for x in (T.get("field") or []) if isinstance(x, dict)]
    if len(tf) != len(m["fields"]):
        fail(7, f"toml message '{T.get('name')}' has {len(tf)} fields; the markdown table has {len(m['fields'])} rows")
    total_bits = 0
    for mf, tfl in zip(m["fields"], tf):
        nm = md_name(mf["meaning"])
        if norm_name(nm) != norm_name(str(tfl.get("name", ""))):
            fail(7, f"toml field '{tfl.get('name')}' is not named after the markdown row '{nm}'")
        if tfl.get("offset") != mf["off"] * 8:
            fail(7, f"toml field {tfl.get('name')}: offset {tfl.get('offset')} bits, markdown byte {mf['off']}")
        if tfl.get("width") != mf["bits"]:
            fail(7, f"toml field {tfl.get('name')}: width {tfl.get('width')} bits; markdown {mf['bits']} bits")
        if mf["maxc"] is not None and tfl.get("max_count") != mf["maxc"]:
            fail(7, f"toml field {tfl.get('name')}: max_count {tfl.get('max_count')}; markdown says max {mf['maxc']}")
        if mf["type"] in PRIM and tfl.get("type") != mf["type"]:
            fail(7, f"toml field {tfl.get('name')}: type {tfl.get('type')}; markdown {mf['type']}")
        if TIMER.search(str(tfl.get("name", ""))):
            fail("DoD", f"toml field '{tfl.get('name')}' is named like a timer")
        total_bits += (tfl.get("width", 0) or 0) * (tfl.get("max_count") or 1)
    if isinstance(T.get("max_size"), int) and total_bits != T["max_size"] * 8:
        fail(7, f"the toml fields of '{T.get('name')}' compute {total_bits // 8} bytes; max_size is {T.get('max_size')}")
    need = set()
    for _, o_ in ADM_SHARED + m["forms"]:
        need.update(w for w in named_in(o_) if w in MEMBERS)
    have = set(T.get("rejection_reasons") or [])
    for r in sorted(need - have):
        fail(7, f"toml message '{T.get('name')}' rejection_reasons does not list '{r}', which its invalid-form rows name")
    for r in sorted(have - set(MEMBERS)):
        fail(7, f"toml message '{T.get('name')}' rejection_reasons names '{r}', not a member of the closed Reject "
             "enumeration (refusal reasons are not packet rejections)")

TOML_REF = {}
if isinstance(toml, dict):
    for _key, _val in toml.items():
        if _key in ("message", "reject", "reserved") or not isinstance(_val, list):
            continue
        for _d in _val:
            if isinstance(_d, dict) and isinstance(_d.get("name"), str) and isinstance(_d.get("value"), int):
                TOML_REF[_d["name"]] = _d["value"]
if REF and TOML_REF != REF:
    fail(7, "wire.toml declares no refusal enumeration equal to the one in wire.md's Admission messages section "
         f"(toml names and values: {sorted(TOML_REF.items())[:3]} ...; wire.md has {len(REF)})")

# AC 2 (review finding C3), wire.toml: the refusal values are 0..17 in the order of the closed set of C-185
for _i, _n in enumerate(REFUSAL_SET):
    if TOML_REF.get(_n) != _i:
        fail(7, f"wire.toml gives refusal reason {_n} the value {TOML_REF.get(_n)}; C-185's order gives {_i}")

# AC 7 (repository level): the wire-check binary and the wire_tables tests
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            fail(7, "cargo run --bin wire-check did not pass:\n" + "\n".join(out.strip().splitlines()[-8:]))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(7, f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        if proc.returncode != 0 or not result:
            fail(7, "cargo test -p hadronyche-tools --test wire_tables failed:\n" + "\n".join(out.strip().splitlines()[-12:]))
        else:
            mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result[-1])
            if not mm or int(mm.group(2)) != 0 or int(mm.group(1)) < 12:
                fail(7, "wire_tables must stay green with 0 failed and at least the 12 tests that stood before 004-18; got: " + result[-1])
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(7, f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC7: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

# ---------------------------------------------------------------------------
if fails:
    for ac, msg in fails:
        print(f"FAIL {label(ac)}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: admission messages with type codes in the hello's admission block, common headers, maximum sizes, "
      "the 18-name refusal enumeration, declared rates and references, downstream A, invalid forms, open item, "
      "toml agreement and DoD tags present")
PY
