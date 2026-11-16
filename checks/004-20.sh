#!/usr/bin/env bash
# 004-20 — receiver report and one-way probe layouts.
#
# RED-phase document check. It validates the "Receiver report" and
# "One-way probe" sections of doc/design/spec/wire.md and the measurement
# [[message]] tables of doc/design/spec/wire.toml against every acceptance
# criterion of the ticket, and re-runs the 004-02 checker (cargo run --bin
# wire-check and the wire_tables tests) and the design checker once each.
# It never runs the other checks/004-*.sh (a sibling loop would be quadratic
# in the per-change suite) and never depends on the main git ref.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo and the design checker) are
# skipped with a NOTE, because they always target the real repository.
#
# Sections are located by heading text ("Receiver report", "One-way probe",
# "Admission messages"), never by position. Numbers come from
# doc/design/02-constants.md at run time. Field tables are found by content:
# the receiver report has a fixed-part table and a per-member table (told
# apart by the heading or paragraph above them); the probe has a request and
# a reply table (told apart the same way).
#
# Layout convention the check reads (the author's contract):
#   - offsets are bytes from the message start in the fixed table, and bytes
#     from the member's first byte in the per-member table; widths are bytes
#     or "N bits" (a sub-byte field shares its byte, as the media header does);
#   - the maximum size of the report is fixed + 32 x per-member, computed here
#     from the tables and compared with the stated number and with wire.toml's
#     max_size (the counted member field has max_count = 32).
#
# What is checked, per acceptance criterion:
#
#   AC 1  Exactly two v1 measurement message kinds: the receiver report and
#         the one-way probe (request and reply, one or two [[message]] tables
#         in wire.toml, layer "measurement"). The report is unicast only, not
#         multicast, and no query message exists (C-47). The count of
#         measurement-layer tables is not pinned.
#   AC 2  The report's maximum size with 32 members is stated in bytes and
#         equals fixed + 32 x per-member, computed from the field tables and
#         equal to wire.toml's max_size. The member count field's valid range
#         has maximum 32 (C-08).
#   AC 3  Every content item has a field: bundle identity (64 bits, the
#         bundle source's module id, C-11); member count; per member the
#         eleven C-159 counters, each with a width and a wrap-or-saturate rule
#         (per row, or one sentence covering all counters), in the per-member
#         table; socket and interface drop counters with widths and rules;
#         observed DSCP at exactly 6 bits, valid 0 to 63, any padding zero;
#         the consumer's current A (C-198) with a width and a unit (ns,
#         samples or ms) and a valid range. The C-108 reclassification is
#         stated (counters are not monotonic per class, or the report makes
#         it unambiguous); the interrupted counter counts EVENTS (C-102, C-38);
#         Demoted is carried by the demoted-rounds counter (C-109, C-159).
#         The A field is bound to the downstream A of 004-18 (C-142): when the
#         Admission messages section is still a placeholder, an Open item
#         names 004-18 as owner with a falsifier about width or unit; when it
#         has landed, its downstream-A row must have the same width and unit.
#   AC 4  The probe's measured quantities are stated (which delays its
#         timestamps yield), tagged [decision], with a falsifier that names a
#         one-way delay. The authentication-reservation choice for BOTH
#         messages is tagged [decision] with a falsifier. If carried, both
#         messages' tables hold the C-12 widths (8 + 64 + 128 bits) with the
#         C-115 values; if not, neither table holds a scheme, counter, tag or
#         reservation field. The probe tables hold identifiers and 64-bit
#         nanosecond timestamps, stated consistent with (or explicitly
#         different from) the slice-header send timestamp (004-08) and the
#         sync timestamps (004-19). Request and reply sizes are stated and
#         equal their tables' sums.
#   AC 5  At least 6 invalid forms with outcomes, covering the ticket's list:
#         too short (TooShort), bad version (VersionMismatch), foreign domain
#         (ForeignDomain), member count above 32 (BadLength), length
#         inconsistent with member count (BadLength), unknown bundle, DSCP
#         above 63, A out of its valid range, reply without matching fields.
#         The three exact mappings are required; the others accept any member
#         of the closed enumeration, or an argued non-rejection (the words
#         "not a rejection", "never a rejection", "cannot occur", "ignored",
#         "is not counted"). Every reason named in an outcome must be a member
#         (negated mentions excepted); no reason is invented. The enumeration
#         is parsed from wire.md's "Rejection reasons" section at run time.
#   AC 6  wire.toml declares the measurement messages with layer "measurement",
#         dscp 40 for the report (CS5, C-13) and 46 for the probe (EF, C-13);
#         max_size equals the markdown computation; the fixed fields tile the
#         markdown tables field for field (offset, width and, for primitive
#         types, type); the counted member field has max_count = 32 and its
#         count names the member count field. Every reason the invalid-form
#         rows name is in the messages' rejection_reasons; each of those
#         reasons lists "measurement" in its enumeration Layers cell. When the
#         paths are not overridden: `cargo run --bin wire-check` reports
#         "checks clean" and `cargo test -p hadronyche-tools --test
#         wire_tables` stays 12 passed, 0 failed, 0 ignored.
#
# Consistency and DoD:
#   C-40 one report per bundle every 100 ms and C-41 a 2 s producer timeout
#        are restated as behaviour; no table row names a rate, interval or
#        period; IPv4 unicast on the reverse port (C-100) is restated; no
#        sentence says forward media packets carry per-receiver state; the
#        DSCP values are restated as CS5 (40) and EF (46).
#   DoD  every prose paragraph of 5+ words in the two sections carries an
#        evidence tag, and so does every table (or it follows a tagged
#        paragraph); any paragraph starting "Open item" names an owner and a
#        falsifier.
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
# Markdown helpers (the conventions of checks/004-19.sh)
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


def section_blocks(src_lines):
    """Blocks of a section: each heading is its own block; fenced code stays
    inside its block; blank lines separate the rest."""
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


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(text):
    return bool(EVIDENCE.search(text))


def norm(s):
    return re.sub(r"[^a-z]", "", s.lower())


def width_to_bits(cell):
    """A Width cell: '2' or '2 bytes' -> bits; '6 bits' -> 6; '1 byte' -> 8."""
    s = (cell or "").lower()
    m = re.search(r"(\d+)", s)
    if not m:
        return None
    n = int(m.group(1))
    if re.search(r"\bbits?\b", s) and not re.search(r"\bbytes?\b", s):
        return n
    return n * 8


def offset_bytes(cell):
    m = re.fullmatch(r"\s*\+?\s*(\d+)\s*", cell or "")
    return int(m.group(1)) if m else None


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


def cell_label(meaning):
    """The label of a Meaning cell: the text before its first colon, or its
    first four words. 'Received: slices ...' -> 'Received'."""
    s = re.sub(r"[*_`]", "", meaning).strip()
    if ":" in s[:80]:
        return s.split(":")[0].strip()
    return " ".join(s.split()[:4])


def label_tokens(text):
    return [t.lower() for t in re.split(r"[^A-Za-z0-9]+", text) if t]


def sentences(text):
    return [s for s in re.split(r"(?<=[.;])\s+|\n+", text) if s.strip()]


def stated_bytes(sentence):
    out = []
    for m in re.finditer(r"(\d[\d ,  ]*\d|\d)\s*bytes?\b", sentence):
        out.append(int(re.sub(r"[ ,  ]", "", m.group(1))))
    return out


def max_int(text):
    nums = [int(re.sub(r"[ ,  ]", "", m)) for m in re.findall(r"\d[\d ,  ]*\d|\d", text or "")]
    return max(nums) if nums else None


UNIT = re.compile(r"\b(ns|nanoseconds?|samples?|ms|milliseconds?|µs|us|microseconds?)\b", re.I)
WRAP = re.compile(r"\b(wraps?|modulo|saturates?)\b", re.I)
RECLASS_WORDS = re.compile(
    r"decreas|not\s+monotonic|non-?monotonic|unambiguous|[−-]\s*1\b|minus\s+one", re.I
)
NEG_WORD = re.compile(r"\b(?:no|not|never|without|cannot|nor)\b", re.I)


# ---------------------------------------------------------------------------
# A small TOML reader (the subset parser of checks/004-19.sh: the check must
# run offline, unprivileged, on the Python the other checks use).
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
    "members_max": first_int(r"at\s+most\s+(\d+)", const_row("C-08")),
    "dscp_report": first_int(r"control\s+CS5\s*\((\d+)\)", const_row("C-13")),
    "dscp_probe": first_int(r"EF\s*\((\d+)\)", const_row("C-13")),
    "report_ms": first_int(r"every\s+(\d+)\s*ms", const_row("C-40")),
    "timeout_s": first_int(r"(\d+)\s*s\s+without", const_row("C-41")),
    "scheme_bits": first_int(r"scheme\s+(\d+)\s+bits", const_row("C-12")),
    "counter_bits": first_int(r"counter\s+(\d+)\s+bits", const_row("C-12")),
    "tag_bits": first_int(r"tag\s+(\d+)\s+bits", const_row("C-12")),
}
for key, value in K.items():
    if value is None:
        fail("DoD", f"constants row for {key} could not be read from {CONSTANTS_PATH}")

# ---------------------------------------------------------------------------
# The closed enumeration and its Layers cells, parsed from wire.md
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    fail(1, f"{wire_path} does not exist")
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
LAYERS = {}
span = section_span(r"rejection[-\s]+reasons?")
if span is None:
    fail("AC5", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read")
else:
    for _, _, rows in parse_tables("\n".join(lines[span[0]:span[1]])):
        for row in rows:
            if len(row) >= 2 and re.fullmatch(r"\d+", row[0].strip()):
                name = row[1].strip().strip("`*_ ")
                if re.fullmatch(r"[A-Z][A-Za-z0-9]+", name):
                    MEMBERS[name] = int(row[0].strip())
                    LAYERS[name] = row[3] if len(row) >= 4 else ""
    if len(MEMBERS) != 13:
        fail("AC5", f"the closed enumeration parsed from wire.md has {len(MEMBERS)} names, not 13")
MEMBER_SET = set(MEMBERS)
NAME_RE = re.compile(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b")

# ---------------------------------------------------------------------------
# The sections, located by heading text
# ---------------------------------------------------------------------------
def span_text(pattern):
    sp = section_span(pattern)
    if sp is None:
        return None
    return "\n".join(lines[sp[0]:sp[1]])


REP_TEXT = span_text(r"^receiver\s+report")
PRB_TEXT = span_text(r"^one-way\s+probe")
ADM_TEXT = span_text(r"^admission")
PLACEHOLDER = re.compile(r"to\s+be\s+filled", re.I)


def is_placeholder(text):
    return text is None or not text.strip() or (PLACEHOLDER.search(text) is not None and "|" not in text)


REP_PLACE = is_placeholder(REP_TEXT)
PRB_PLACE = is_placeholder(PRB_TEXT)
if REP_PLACE:
    for ac in (1, 2, 3, 4, 5, 6, "DoD"):
        fail(ac, "the Receiver report section is missing or still the 004-20 placeholder")
if PRB_PLACE:
    for ac in (1, 4, 5, 6, "DoD"):
        fail(ac, "the One-way probe section is missing or still the 004-20 placeholder")


class Sec:
    """One section: its text, lines, prose, blocks and field tables."""

    def __init__(self, text):
        self.text = text or ""
        self.lines = self.text.splitlines()
        self.prose_text = strip_fences(self.text)
        self.blocks = section_blocks(self.lines)
        self.prose = ["\n".join(b) for b in self.blocks if block_kind(b) == "prose"]
        self.prose_all = "\n".join(self.prose)
        self.sents = sentences(self.prose_all)

    def table_label(self, start):
        heading = None
        para = None
        for k in range(start - 1, -1, -1):
            ln = self.lines[k].strip()
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

    def field_tables(self):
        out = []
        for start, header, rows in parse_tables(self.text):
            if not is_field_table(header):
                continue
            heading, para = self.table_label(start)
            out.append({"start": start, "label": heading or para or "", "rows": rows})
        return out

    def form_rows(self):
        """Rows of every invalid-form table of this section: a table whose
        first header cell names the form (Invalid form, Further rule, ...) and
        whose second names the outcome. One table or several, the rows count."""
        rows_out = []
        for _, header, rows in parse_tables(self.text):
            heads = [norm(h) for h in header]
            if len(heads) >= 2 and any(k in heads[0] for k in FORM_KEYS) and any("outcome" in h for h in heads):
                rows_out.extend(r for r in rows if len(r) >= 2 and r[0].strip())
        return rows_out


REP = Sec(REP_TEXT)
PRB = Sec(PRB_TEXT)
ADM = Sec(ADM_TEXT)
ADM_PLACE = is_placeholder(ADM_TEXT)


def layout(rows, name, ac):
    """Tile a field table: returns (fields, bits). A sub-byte row shares its
    byte with the rows before it; any other gap or overlap is a failure."""
    fields = []
    pos = 0
    for row in rows:
        if len(row) < 5:
            continue
        off = offset_bytes(row[0])
        if off is None:
            continue
        w = width_to_bits(row[1])
        if w is None:
            fail(ac, f"{name}: the width of '{cell_label(row[3])}' is not stated")
            continue
        declared = off * 8
        sub = w < 8
        if declared == pos or (sub and declared // 8 == pos // 8 and pos % 8 + w <= 8):
            start = pos
        else:
            fail(ac, f"{name}: the row at byte {off} ('{cell_label(row[3])}') does not start where "
                 f"the previous row ends (bit {pos}): a gap or an overlap")
            start = pos
        fields.append({
            "off": start, "w": w, "label": cell_label(row[3]), "lab": norm(cell_label(row[3])),
            "type": re.sub(r"[`*_ ]", "", row[2]).lower(), "meaning": row[3], "valid": row[4],
            "text": " | ".join(row), "row": row,
        })
        pos = start + w
    if pos % 8:
        fail(ac, f"{name}: the fields sum to {pos} bits, not whole bytes")
    return fields, pos


def find_lab(fields, key):
    return [f for f in fields if key in f["lab"]]


# ---------------------------------------------------------------------------
# The receiver report tables: fixed part and per-member part
# ---------------------------------------------------------------------------
REP_FIXED = REP_MEMBER = None
REP_FIX_F = REP_MEM_F = []
REP_FIX_BITS = REP_MEM_BITS = None
if not REP_PLACE:
    rep_tabs = REP.field_tables()
    member_tabs = [t for t in rep_tabs if re.search(r"per[- ]member|each member|member block", t["label"], re.I)]
    fixed_tabs = [t for t in rep_tabs if t not in member_tabs]
    if len(fixed_tabs) != 1 or len(member_tabs) != 1:
        fail(1, "the receiver report needs one fixed-part field table and one per-member field table "
             "(name them by their heading, e.g. '### Fixed part' and '### Per-member part')")
    else:
        REP_FIX_F, REP_FIX_BITS = layout(fixed_tabs[0]["rows"], "receiver report fixed part", 1)
        REP_MEM_F, REP_MEM_BITS = layout(member_tabs[0]["rows"], "receiver report per-member part", 1)
        REP_FIXED = fixed_tabs[0]
        REP_MEMBER = member_tabs[0]

REP_OK = REP_FIXED is not None and REP_MEMBER is not None
MAX_REP = None
NAMED = set()
FORM_ROWS = []
PRIMITIVE = {"uint", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "q1.31", "pad"}
if REP_OK and REP_FIX_BITS is not None and REP_MEM_BITS is not None and (REP_FIX_BITS % 8 == 0) and (REP_MEM_BITS % 8 == 0):
    MAX_REP = (REP_FIX_BITS + K["members_max"] * REP_MEM_BITS) // 8 if K["members_max"] else None
FIX_BYTES = REP_FIX_BITS // 8 if REP_OK and REP_FIX_BITS is not None and REP_FIX_BITS % 8 == 0 else None
MEM_BYTES = REP_MEM_BITS // 8 if REP_OK and REP_MEM_BITS is not None and REP_MEM_BITS % 8 == 0 else None

# ---------------------------------------------------------------------------
# The probe tables: request and reply
# ---------------------------------------------------------------------------
PRB_REQ = PRB_REP = None
PRB_REQ_F = PRB_REP_F = []
PRB_REQ_BITS = PRB_REP_BITS = None
if not PRB_PLACE:
    prb_tabs = PRB.field_tables()
    cls = {}
    for t in prb_tabs:
        lab = t["label"].lower()
        has_req = "request" in lab
        has_rep = re.search(r"\brepl(y|ies)\b", lab) is not None
        if has_rep and not has_req:
            cls.setdefault("reply", []).append(t)
        elif has_req and not has_rep:
            cls.setdefault("request", []).append(t)
    if len(prb_tabs) != 2 or len(cls.get("request", [])) != 1 or len(cls.get("reply", [])) != 1:
        fail(1, "the one-way probe needs exactly one request field table and one reply field table "
             "(name them by their heading, e.g. '### Request' and '### Reply')")
    else:
        PRB_REQ_F, PRB_REQ_BITS = layout(cls["request"][0]["rows"], "probe request", 1)
        PRB_REP_F, PRB_REP_BITS = layout(cls["reply"][0]["rows"], "probe reply", 1)
        PRB_REQ = cls["request"][0]
        PRB_REP = cls["reply"][0]
REQ_BYTES = PRB_REQ_BITS // 8 if PRB_REQ_BITS is not None and PRB_REQ_BITS % 8 == 0 else None
REP_BYTES = PRB_REP_BITS // 8 if PRB_REP_BITS is not None and PRB_REP_BITS % 8 == 0 else None

# ---------------------------------------------------------------------------
# AC 1 — two measurement message kinds; unicast only; no query message
# ---------------------------------------------------------------------------
if not REP_PLACE:
    ac1 = True
    if not re.search(r"\bunicast\b", REP.prose_all, re.I) or not re.search(
            r"\b(?:not|never|no)\b[^.;]{0,20}\bmulticast\b|\bunicast\s+only\b", REP.prose_all, re.I):
        fail(1, "the receiver report section does not say the report is unicast only, not multicast (C-47)")
        ac1 = False
    if not re.search(r"\b(?:no|not|never|without)\b[^.;]{0,40}\bquer(?:y|ies)\b", REP.prose_all, re.I):
        fail(1, "the receiver report section does not state that there is no query message (C-47)")
        ac1 = False
    if ac1 and REP_OK and PRB_REQ is not None:
        print("PASS AC1: receiver report and one-way probe (request, reply) defined; report unicast only; "
              "no query message")

# ---------------------------------------------------------------------------
# AC 2 — maximum size with 32 members, in bytes; member count maximum 32
# ---------------------------------------------------------------------------
if REP_OK and MAX_REP is not None:
    ac2 = True
    allowed = {MAX_REP, FIX_BYTES, MEM_BYTES, K["members_max"] * MEM_BYTES if MEM_BYTES else None}
    size_sents = [s for s in REP.sents
                  if re.search(r"maximum|at most|largest|size|long", s, re.I) and re.search(r"\b32\b", s)]
    stated = [n for s in size_sents for n in stated_bytes(s)]
    if MAX_REP not in stated:
        fail(2, f"the receiver report's maximum size with 32 members is not stated in bytes as "
             f"fixed + 32 x per-member = {MAX_REP} bytes")
        ac2 = False
    for s in size_sents:
        for n in stated_bytes(s):
            if n not in allowed:
                fail(2, f"a size sentence states {n} bytes, which is not the computed maximum {MAX_REP} "
                     f"(or its parts): {s[:90]}")
                ac2 = False
                break
    cnt = [f for f in REP_FIX_F if "member" in f["lab"] and "count" in f["lab"]]
    if not cnt:
        fail(2, "the fixed part has no member count field")
        ac2 = False
    else:
        vmax = max_int(cnt[0]["valid"])
        if vmax != K["members_max"]:
            fail(2, f"the member count's valid range has maximum {vmax}, not {K['members_max']} (C-08): "
                 f"{cnt[0]['valid'][:60]}")
            ac2 = False
    if ac2:
        print(f"PASS AC2: receiver report maximum with 32 members is {MAX_REP} bytes "
              f"({FIX_BYTES} fixed + 32 x {MEM_BYTES}); member count maximum 32")

# ---------------------------------------------------------------------------
# AC 3 — every content item has a field, with widths, rules and bindings
# ---------------------------------------------------------------------------
COUNTER_KEYS = [
    ("received", "received"), ("late", "late"), ("missing", "missing"),
    ("markedgap", "marked gap"), ("otherepoch", "other-epoch"), ("tooearly", "too-early"),
    ("duplicate", "duplicates"), ("incompletereassembl", "incomplete reassemblies"),
    ("demoted", "demoted rounds"), ("interrupted", "interrupted events"), ("dead", "dead"),
]
if REP_OK:
    ac3 = True
    rep_all_fields = REP_FIX_F + REP_MEM_F
    # bundle identity
    bund = find_lab(REP_FIX_F, "bundle")
    if not bund:
        fail(3, "the fixed part has no bundle identity field")
        ac3 = False
    elif not (bund[0]["w"] == 64 and re.search(r"module\s+id", bund[0]["meaning"], re.I)):
        fail(3, "the bundle identity must be the bundle source's module id, 64 bits (C-11): "
             f"{bund[0]['meaning'][:80]}")
        ac3 = False
    # member count presence checked in AC 2; per-member counters
    cover_all = any(re.search(r"\bcounters?\b", s, re.I) and WRAP.search(s) for s in REP.sents)
    for key, shown in COUNTER_KEYS:
        rows = find_lab(REP_MEM_F, key)
        if not rows:
            fail(3, f"the per-member table has no field for the C-159 counter '{shown}'")
            ac3 = False
            continue
        r0 = rows[0]
        if r0["w"] <= 0:
            fail(3, f"the counter '{shown}' has no width")
            ac3 = False
        is_flag = key == "dead" and re.search(r"\bflag\b", r0["meaning"] + " " + r0["valid"], re.I)
        if not (cover_all or is_flag or WRAP.search(r0["meaning"] + " " + r0["valid"])):
            fail(3, f"the counter '{shown}' states no wrap-or-saturate rule (wrap, modulo or saturate)")
            ac3 = False
    # socket and interface drop counters, anywhere in the report
    for key, shown in (("socket", "socket drop"), ("interface", "interface drop")):
        rows = [f for f in rep_all_fields if key in f["lab"] and "drop" in f["lab"]]
        if not rows:
            fail(3, f"the report has no {shown} counter field")
            ac3 = False
        elif rows[0]["w"] <= 0 or not (cover_all or WRAP.search(rows[0]["meaning"] + " " + rows[0]["valid"])):
            fail(3, f"the {shown} counter needs a width and a wrap-or-saturate rule")
            ac3 = False
    # observed DSCP: exactly 6 bits, valid 0 to 63, padding zero
    dscp = [f for f in rep_all_fields if "dscp" in f["lab"]]
    if not dscp:
        fail(3, "the report has no observed DSCP field")
        ac3 = False
    else:
        d = dscp[0]
        if d["w"] != 6:
            fail(3, f"the observed DSCP must be exactly 6 bits, not {d['w']} (the DSCP is 0 to 63)")
            ac3 = False
        if max_int(d["valid"]) != 63:
            fail(3, f"the observed DSCP's valid range must end at 63: {d['valid'][:60]}")
            ac3 = False
    # the consumer's current A (C-198): width, unit, valid range
    a_rows = [f for f in rep_all_fields if re.search(r"\bA\b", f["label"]) or "C-198" in f["meaning"]]
    if not a_rows:
        fail(3, "the report has no field for the consumer's current A (C-198)")
        ac3 = False
        A_ROW = None
    else:
        A_ROW = a_rows[0]
        if A_ROW["w"] <= 0:
            fail(3, "the current A has no stated width")
            ac3 = False
        um = UNIT.search(A_ROW["meaning"] + " " + A_ROW["valid"])
        if not um:
            fail(3, "the current A has no stated unit (ns, samples or ms)")
            ac3 = False
        if not re.search(r"\d", A_ROW["valid"]):
            fail(3, "the current A has no stated valid range")
            ac3 = False
    # reclassification, interrupted events, demoted rounds: sentences
    if not any(re.search(r"C-108|reclassif", s, re.I) and RECLASS_WORDS.search(s) for s in REP.sents):
        fail(3, "the report does not state how the C-108 reclassification (missing -1, late +1) is "
             "reflected: say the counters are not monotonic per class (missing can decrease), or that "
             "the report makes the reclassification unambiguous")
        ac3 = False
    if not any(re.search(r"\binterrupted\b", s, re.I) and re.search(r"\bevents?\b", s, re.I) for s in REP.sents):
        fail(3, "the report does not say the interrupted counter counts interrupted EVENTS (C-102, C-38)")
        ac3 = False
    if not any(re.search(r"\bdemoted\b", s, re.I) and re.search(r"\brounds?\b", s, re.I)
               and re.search(r"\bcounter", s, re.I) for s in REP.sents):
        fail(3, "the report does not say Demoted is carried by the demoted-rounds counter (C-109, C-159)")
        ac3 = False
    # the A binding to 004-18 (C-142): stated as the same, in either branch
    if not any(re.search(r"004-18", s) and re.search(r"\bsame\b|\bbind|\bequal|\bmatch", s, re.I)
               for s in REP.sents):
        fail(3, "the report does not say its A has the width and unit of the downstream A of 004-18 (C-142)")
        ac3 = False
    # header rows (Work 1): magic, version and domain at 0, 2, 4; a message type row
    for name, fields in (("receiver report fixed part", REP_FIX_F), ("probe request", PRB_REQ_F),
                         ("probe reply", PRB_REP_F)):
        if name != "receiver report fixed part" and PRB_PLACE:
            continue
        if len(fields) < 4 or not (re.search(r"magic", fields[0]["label"], re.I) and fields[0]["off"] == 0
                                   and fields[0]["w"] == 16 and re.search(r"4d4e", fields[0]["text"], re.I)):
            fail("W1", f"the {name} does not start with the 2-byte magic 0x4d4e at offset 0")
            ac3 = False
        if len(fields) < 4 or not (re.search(r"version", fields[1]["label"], re.I) and fields[1]["off"] == 16
                                   and fields[1]["w"] == 16):
            fail("W1", f"the {name} has no 2-byte version field at offset 2")
            ac3 = False
        if len(fields) < 4 or not (re.search(r"domain", fields[2]["label"], re.I) and fields[2]["off"] == 32
                                   and fields[2]["w"] == 16):
            fail("W1", f"the {name} has no 2-byte domain id field at offset 4")
            ac3 = False
        if not any(re.search(r"message\s+type", f["label"] + " " + f["meaning"], re.I) for f in fields):
            fail("W1", f"the {name} has no message type row")
            ac3 = False
    if ADM_PLACE:
        ok_open = False
        for b in REP.prose:
            if re.match(r"\W*open\s+items?\b", b.strip(), re.I) and re.search(r"004-18", b) \
                    and re.search(r"owner", b, re.I) and re.search(r"falsifier", b, re.I) \
                    and re.search(r"width|unit", b, re.I):
                ok_open = True
        if not ok_open:
            fail(3, "the A field is not bound to the downstream A of 004-18 (C-142): the Admission messages "
                 "section is a placeholder, so an Open item must name 004-18 as owner, with a falsifier "
                 "that says 004-18 lays out a different width or unit")
            ac3 = False
    else:
        adm_fields = []
        for t in parse_tables(ADM.text):
            if is_field_table(t[1]):
                adm_fields.extend(layout(t[2], "admission", 3)[0])
        down = [f for f in adm_fields if re.search(r"\bA\b", f["label"]) or re.search(r"downstream", f["meaning"], re.I)]
        if not down:
            fail(3, "the Admission messages section has no downstream A field to compare with (004-18)")
            ac3 = False
        elif A_ROW is not None and (down[0]["w"] != A_ROW["w"] or
                                    not UNIT.search(down[0]["meaning"] + " " + down[0]["valid"])):
            fail(3, f"the current A ({A_ROW['w']} bits) does not have the width and unit of the downstream "
                 f"A of 004-18 ({down[0]['w']} bits)")
            ac3 = False
    if ac3:
        print("PASS AC3: bundle identity, member count, eleven C-159 counters with widths and wrap rules, "
              "socket and interface drops, observed DSCP (6 bits), current A with width and unit, "
              "C-108, interrupted events and demoted rounds stated")

# ---------------------------------------------------------------------------
# AC 4 — probe quantities, authentication decision, probe identifiers and
#         timestamps, probe sizes
# ---------------------------------------------------------------------------
def decision_blocks(sec):
    """(index, text) of [decision]-tagged prose blocks of a section."""
    return [(i, b) for i, b in enumerate(sec.prose) if re.search(r"\[decision\]", b, re.I)]


def falsifier_near(sec, idx):
    window = "\n".join(sec.prose[max(0, idx - 1): idx + 2])
    fm = re.search(r"falsifier\s*[:\-—]?\s*(.*)", window, re.I | re.S)
    return fm.group(1) if fm else None


if not PRB_PLACE and PRB_REQ is not None and PRB_REP is not None:
    ac4 = True
    # measured quantities
    q_hit = None
    for i, b in decision_blocks(PRB):
        if re.search(r"round[\s-]+trip|one-way", b, re.I) and re.search(r"delay", b, re.I) \
                and re.search(r"timestamp", b, re.I):
            q_hit = i
            break
    if q_hit is None:
        fail(4, "no [decision]-tagged probe paragraph states which delays its timestamps yield "
             "(one-way in each direction under network time, round trip, or both)")
        ac4 = False
    else:
        fal = falsifier_near(PRB, q_hit)
        if not fal or len(fal.split()) < 4 or not re.search(r"delay", fal, re.I):
            fail(4, "the probe-quantities decision has no falsifier naming the one-way delay a consumer "
                 "could need and the timestamps cannot yield")
            ac4 = False
    # authentication decision for BOTH messages
    auth_hit = None
    for sec in (REP, PRB):
        for i, b in decision_blocks(sec):
            if re.search(r"authentication|reservation", b, re.I):
                auth_hit = (sec, i, b)
                break
        if auth_hit:
            break
    if auth_hit is None:
        fail(4, "no [decision]-tagged paragraph states the authentication-reservation choice")
        ac4 = False
    else:
        sec, idx, btext = auth_hit
        if not (re.search(r"\breport", btext, re.I) and re.search(r"\bprobe", btext, re.I)
                or re.search(r"\bboth\b", btext, re.I)):
            fail(4, "the authentication decision does not say it covers both the report and the probe")
            ac4 = False
        decision = None
        for sentence in re.split(r"(?<=[.;])\s+", btext):
            if not re.search(r"authentication|reservation", sentence, re.I):
                continue
            if re.search(r"\b(?:not|no|without|never)\b|n't", sentence, re.I):
                decision = "not carried"
                break
            if re.search(r"\b(?:carr(?:y|ies|ied)|includ\w*|bears?|has)\b", sentence, re.I):
                decision = "carried"
                break
        if decision is None:
            fail(4, "the authentication decision does not say whether the messages carry the reservation")
            ac4 = False
        fal = falsifier_near(sec, idx)
        if not fal or len(fal.split()) < 4:
            fail(4, "the authentication decision has no falsifier (the word 'falsifier' followed by the "
                 "condition that would overturn it)")
            ac4 = False
        tables = [("report fixed part", REP_FIX_F), ("report per-member part", REP_MEM_F),
                  ("probe request", PRB_REQ_F), ("probe reply", PRB_REP_F)]
        if decision == "carried":
            for name, fields in tables[0:1] + tables[2:]:
                sch = [f for f in fields if re.search(r"authentication|scheme", f["label"], re.I)
                       and f["w"] == K["scheme_bits"] and re.search(r"\b0\b|none|zero", f["valid"], re.I)]
                cnt_ = [f for f in fields if re.search(r"counter", f["label"], re.I) and f["w"] == K["counter_bits"]
                        and re.search(r"live", f["meaning"] + " " + f["valid"], re.I)]
                tag = [f for f in fields if re.search(r"\btag\b", f["label"], re.I) and f["w"] == K["tag_bits"]
                       and re.search(r"zero|\b0\b", f["valid"] + " " + f["meaning"], re.I)]
                if not sch or not cnt_ or not tag:
                    fail(4, f"the decision carries the reservation but the {name} table lacks the C-12 widths "
                         f"({K['scheme_bits']} + {K['counter_bits']} + {K['tag_bits']} bits) with the C-115 "
                         "values (scheme 0, live counter, zero tag)")
                    ac4 = False
        elif decision == "not carried":
            for name, fields in tables:
                for f in fields:
                    if re.search(r"authentication|scheme|reservation", f["label"], re.I) or \
                            re.search(r"\btag\b|message counter", f["label"], re.I):
                        fail(4, f"the decision says the messages do not carry the reservation, but the {name} "
                             f"table has the field '{f['label']}'")
                        ac4 = False
    # probe identifiers and timestamps
    ident = [f for f in PRB_REQ_F if re.search(r"identif", f["label"] + " " + f["meaning"], re.I)]
    echo = [f for f in PRB_REP_F if re.search(r"\becho", f["meaning"], re.I)]
    if not ident:
        fail(4, "the probe request has no identifier field closing the round trip")
        ac4 = False
    if not echo or (ident and not any(e["w"] == ident[0]["w"] for e in echo)):
        fail(4, "the probe reply does not echo the request's identifier at the same width (the reply "
             "without matching fields is an invalid form)")
        ac4 = False
    for name, fields in (("request", PRB_REQ_F), ("reply", PRB_REP_F)):
        if not any(re.search(r"timestamp", f["label"] + " " + f["meaning"], re.I) and f["w"] == 64 for f in fields):
            fail(4, f"the probe {name} has no 64-bit timestamp field")
            ac4 = False
    if not re.search(r"slice[\s-]header", PRB.prose_all, re.I) or \
            not re.search(r"same|consistent|different|differs", PRB.prose_all, re.I):
        fail(4, "the probe does not state its timestamps are consistent with (or differ from) the slice-header "
             "send timestamp (004-08) and the sync timestamps (004-19)")
        ac4 = False
    if not re.search(r"nanosecond|\bns\b", PRB.prose_all, re.I) or not re.search(r"monotonic", PRB.prose_all, re.I):
        fail(4, "the probe timestamps' unit (nanoseconds) and clock (monotonic) are not stated")
        ac4 = False
    # request and reply sizes
    if REQ_BYTES is None or REP_BYTES is None:
        fail(4, "the probe request or reply has no whole-byte size")
        ac4 = False
    else:
        size_sents = [s for s in PRB.sents if re.search(r"size|long|bytes", s, re.I) and stated_bytes(s)]
        stated_all = {n for s in size_sents for n in stated_bytes(s)}
        if REQ_BYTES not in stated_all or REP_BYTES not in stated_all:
            fail(4, f"the probe sizes are not stated as the sums of their tables (request {REQ_BYTES} bytes, "
                 f"reply {REP_BYTES} bytes)")
            ac4 = False
        for s in size_sents:
            for n in stated_bytes(s):
                if n not in (REQ_BYTES, REP_BYTES):
                    fail(4, f"a probe size sentence states {n} bytes, not a table sum: {s[:90]}")
                    ac4 = False
                    break
    if ac4:
        print("PASS AC4: probe quantities [decision] with falsifier; authentication decision for both messages "
              f"with falsifier ({decision}); probe identifiers, timestamps and sizes")

# ---------------------------------------------------------------------------
# AC 5 — invalid forms with outcomes
# ---------------------------------------------------------------------------
SHORT_FORM = re.compile(r"too\s+short|shorter\s+than|fewer\s+than|less\s+than|under\s+\d+\s+(?:bytes?|octets?)", re.I)
FORM_KEYS = ("form", "invalid", "rule", "case", "condition")
NON_REJECTION = re.compile(
    r"not\s+a\s+rejection|never\s+a\s+rejection|cannot\s+occur|ignored|is\s+not\s+counted|not\s+counted",
    re.I,
)
if not REP_PLACE:
    FORM_ROWS += REP.form_rows()
if not PRB_PLACE:
    FORM_ROWS += PRB.form_rows()
if not REP_PLACE or not PRB_PLACE:
    ac5 = True
    if len(FORM_ROWS) < 6:
        fail(5, f"only {len(FORM_ROWS)} invalid form(s) with outcomes across the two messages; at least 6 required")
        ac5 = False

    def category(form):
        if re.search(r"dscp|codepoint", form, re.I):
            return "dscp"
        if SHORT_FORM.search(form):
            return "short"
        if re.search(r"\bversion\b", form, re.I):
            return "version"
        if re.search(r"foreign", form, re.I):
            return "foreign"
        if re.search(r"member", form, re.I) and re.search(r"above|more\s+than|exceed|beyond|greater", form, re.I):
            return "members"
        if re.search(r"\blength\b|\bsize\b|inconsistent|disagree", form, re.I):
            return "length"
        if re.search(r"reply|match|echo", form, re.I):
            return "reply"
        if re.search(r"bundle", form, re.I):
            return "bundle"
        if re.search(r"\bA\b", form):
            return "a"
        return None

    EXACT = {"version": "VersionMismatch", "foreign": "ForeignDomain", "short": "TooShort",
             "members": "BadLength", "length": "BadLength"}
    seen = {}
    for r in FORM_ROWS:
        form = r[0]
        outcome = " | ".join(r[1:])
        cat = category(form)
        if cat is None:
            # a row outside the ticket's nine: accepted when its outcome names a
            # member of the closed enumeration or argues a non-rejection; an
            # invented reason fails in the scan below, wherever the row sits
            if not (any(n in MEMBER_SET and re.search(r"\b" + n + r"\b", outcome) for n in MEMBERS)
                    or NON_REJECTION.search(outcome)):
                fail(5, f"the invalid-form row '{form[:60]}' is outside the ticket's list and its outcome "
                     f"names no member of the closed enumeration nor argues a non-rejection: {outcome[:90]}")
                ac5 = False
            continue
        seen.setdefault(cat, []).append(outcome)
        if cat in EXACT and not re.search(r"\b" + EXACT[cat] + r"\b", outcome):
            fail(5, f"the invalid form '{form[:60]}' must be counted under {EXACT[cat]} (the ticket names it): "
                 f"{outcome[:90]}")
            ac5 = False
        if cat in ("bundle", "a", "reply"):
            if not (any(n in MEMBER_SET and re.search(r"\b" + n + r"\b", outcome) for n in MEMBERS)
                    or NON_REJECTION.search(outcome)):
                fail(5, f"the invalid form '{form[:60]}' needs a member reason or an argued non-rejection: "
                     f"{outcome[:90]}")
                ac5 = False
        if cat == "dscp" and not (NON_REJECTION.search(outcome) or re.search(r"ReservedNonZero", outcome)):
            fail(5, f"the DSCP-above-63 form must say it cannot occur (6-bit field) or name the padding "
                 f"reason ReservedNonZero: {outcome[:90]}")
            ac5 = False
    for cat, what in (("version", "bad version"), ("foreign", "foreign domain"), ("short", "too short"),
                      ("members", "member count above 32"), ("length", "length inconsistent with member count"),
                      ("bundle", "unknown bundle"), ("dscp", "DSCP above 63"), ("a", "A out of its valid range"),
                      ("reply", "reply without matching fields")):
        if cat not in seen:
            fail(5, f"no invalid form for {what} (the ticket's list)")
            ac5 = False
    # invented reasons: every CamelCase outcome token must be a member, unless negated
    for r in FORM_ROWS:
        outcome = " | ".join(r[1:])
        for m_ in NAME_RE.finditer(outcome):
            name = m_.group(0)
            before = outcome[max(0, m_.start() - 25): m_.start()]
            if name in MEMBER_SET:
                if not re.search(r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$", before, re.I):
                    NAMED.add(name)
                continue
            if not re.search(r"\b(?:never|not|nor|rather than|instead of)\b[^|.;]*$", before, re.I):
                fail(5, f"the outcome names '{name}', which is not in the closed enumeration")
                ac5 = False
    for m_ in re.finditer(r"\bunder\s+`?([A-Z][A-Za-z]+)`?", REP.text + "\n" + PRB.text):
        if m_.group(1) not in MEMBER_SET:
            fail(5, f"the section counts a rejection under '{m_.group(1)}', not a member of the closed enumeration")
            ac5 = False
    if ac5:
        print(f"PASS AC5: {len(FORM_ROWS)} invalid forms with outcomes; exact reasons where the ticket names them; "
              "every named reason a member")

# ---------------------------------------------------------------------------
# AC 6 (markdown-to-toml agreement and the toml declarations)
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
MEAS = [m for m in msgs if isinstance(m, dict) and m.get("layer") == "measurement"]
REP_T = [m for m in msgs if isinstance(m, dict) and re.search(r"receiver[\s_-]*report", str(m.get("name", "")), re.I)]
# the report is the table that owns the counted member field; a separate
# member-row message (the sparse entry precedent) is not the report itself
T_REP = next((m for m in REP_T if any(isinstance(f, dict) and "count" in f for f in (m.get("field") or []))),
             REP_T[0] if REP_T else None)
PROBE_T = [m for m in msgs if isinstance(m, dict) and re.search(r"probe", str(m.get("name", "")), re.I)]
if T_REP is None:
    fail(1, "wire.toml declares no [[message]] named 'receiver report'")
if not PROBE_T:
    fail(1, "wire.toml declares no [[message]] for the one-way probe")
T_PREQ = next((m for m in PROBE_T if re.search(r"request", str(m.get("name", "")), re.I)), None)
T_PREP = next((m for m in PROBE_T if re.search(r"repl", str(m.get("name", "")), re.I)), None)
if len(PROBE_T) == 1 and T_PREQ is None and T_PREP is None:
    T_PREQ = PROBE_T[0]

ALL_TOML_REASONS = set()


def check_tile(name, tm, dscp_want):
    if tm.get("layer") != "measurement":
        fail(6, f"{name} in wire.toml must have layer = \"measurement\"")
    if tm.get("dscp") != dscp_want:
        fail(6, f"{name} in wire.toml must have dscp = {dscp_want} (C-13)")
    reasons = tm.get("rejection_reasons")
    if not isinstance(reasons, list):
        fail(6, f"{name} in wire.toml has no rejection_reasons list")
        reasons = []
    for reason in reasons:
        if reason not in MEMBER_SET:
            fail(6, f"{name} names '{reason}', which is not a member of the closed enumeration")
        ALL_TOML_REASONS.add(reason)


def tiles_match(name, tfields, md_fields):
    if len(tfields) != len(md_fields):
        fail(6, f"{name}: the markdown table has {len(md_fields)} fields, wire.toml {len(tfields)}")
        return
    for tf, mf in zip(tfields, md_fields):
        if not isinstance(tf, dict):
            continue
        if (tf.get("offset"), tf.get("width")) != (mf["off"], mf["w"]):
            fail(6, f"{name}: markdown field '{mf['label']}' at bit {mf['off']} ({mf['w']} bits) does not match "
                 f"wire.toml field '{tf.get('name')}' at bit {tf.get('offset')} ({tf.get('width')} bits)")
            return
        if mf["type"] in PRIMITIVE and mf["type"] != str(tf.get("type", "")).lower():
            fail(6, f"{name}: markdown type '{mf['type']}' of '{mf['label']}' does not match wire.toml type "
                 f"'{tf.get('type')}'")
            return


if not REP_PLACE and REP_OK and T_REP is not None:
    check_tile("receiver report", T_REP, K["dscp_report"])
    tfields = [f for f in (T_REP.get("field") or []) if isinstance(f, dict)]
    if len(tfields) != len(REP_FIX_F) + 1:
        fail(6, f"receiver report in wire.toml: {len(tfields)} fields, expected the {len(REP_FIX_F)} fixed "
             "fields plus one counted member field")
    else:
        tiles_match("receiver report", tfields[:-1], REP_FIX_F)
        el = tfields[-1]
        if el.get("width") != REP_MEM_BITS or el.get("max_count") != K["members_max"]:
            fail(6, f"receiver report in wire.toml: the counted member field must have width "
                 f"{REP_MEM_BITS} bits per element and max_count {K['members_max']}")
        cnt_idx = next((i for i, f in enumerate(REP_FIX_F) if "member" in f["lab"] and "count" in f["lab"]), None)
        if cnt_idx is None or el.get("count") != tfields[cnt_idx].get("name"):
            fail(6, "receiver report in wire.toml: the counted member field's count must name the member count field")
    if MAX_REP is not None and T_REP.get("max_size") != MAX_REP:
        fail(6, f"receiver report in wire.toml: max_size {T_REP.get('max_size')} differs from the markdown "
             f"maximum {MAX_REP}")

for tm, md_fields, md_bits, want, label_ in ((T_PREQ, PRB_REQ_F, PRB_REQ_BITS, K["dscp_probe"], "probe request"),
                                             (T_PREP, PRB_REP_F, PRB_REP_BITS, K["dscp_probe"], "probe reply")):
    if tm is None and label_ == "probe reply" and len(PROBE_T) == 1:
        continue  # one toml table declares the whole probe; checked as the request
    if tm is None or PRB_PLACE or PRB_REQ is None:
        if tm is None and not PRB_PLACE and PRB_REQ is not None:
            fail(6, f"wire.toml declares no [[message]] for the {label_}")
        continue
    if label_ == "probe reply" and not PRB_REP_F:
        continue
    check_tile(label_, tm, want)
    tfields = [f for f in (tm.get("field") or []) if isinstance(f, dict)]
    tiles_match(label_, tfields, md_fields)
    if md_bits is not None and tm.get("max_size") != md_bits // 8:
        fail(6, f"{label_} in wire.toml: max_size {tm.get('max_size')} differs from the markdown size "
             f"{md_bits // 8}")

# Reasons named by the invalid-form rows must be declared by a measurement message
if FORM_ROWS:
    for name in sorted(NAMED):
        if name not in ALL_TOML_REASONS:
            fail(6, f"the measurement messages' rejection_reasons in wire.toml miss '{name}', which an "
                 "invalid-form row names")
    for reason in sorted(ALL_TOML_REASONS):
        if reason in MEMBER_SET and "measurement" not in LAYERS.get(reason, ""):
            fail(6, f"the enumeration row for '{reason}' does not list the measurement layer in its Layers cell, "
                 "but a measurement message can produce it")

# ---------------------------------------------------------------------------
# Consistency: C-40, C-41 as behaviour with no rate field; IPv4 unicast on the
# reverse port (C-100); rule 8; DSCP values; no per-receiver forward state
# ---------------------------------------------------------------------------
BOTH = "\n".join(t for t in (REP.prose_all, PRB.prose_all))
if not REP_PLACE:
    if K["report_ms"] is not None and not re.search(r"\b%d\s*(?:ms|milliseconds)\b" % K["report_ms"], REP.prose_all, re.I):
        fail("C-40", f"the report section does not restate one report per bundle every {K['report_ms']} ms (C-40)")
    if not re.search(r"\bbundle\b", REP.prose_all, re.I):
        fail("C-40", "the report section does not say the report is per bundle (C-40)")
    if K["timeout_s"] is not None and not re.search(r"\b%d\s*(?:s|seconds?)\b" % K["timeout_s"], REP.prose_all, re.I):
        fail("C-41", f"the report section does not restate the producer's {K['timeout_s']} s timeout without "
             "receiver reports (C-41)")
    if K["dscp_report"] is not None and not re.search(r"CS5\s*\(%d\)" % K["dscp_report"], REP.prose_all):
        fail("C-13", f"the report section does not restate its DSCP as CS5 ({K['dscp_report']}) (C-13)")
    for f in REP_FIX_F + REP_MEM_F:
        tokens = set(label_tokens(f["label"]))
        if tokens & {"rate", "interval", "period", "frequency"} or re.search(r"per\s+second", f["meaning"], re.I) \
                and not NEG_WORD.search(f["meaning"]):
            fail("C-40", f"the report table names a rate or interval field ('{f['label']}'); C-40 is behaviour, "
                 "not a field")
    for s in sentences(REP.prose_all) + sentences(PRB.prose_all):
        if re.search(r"(forward|media)\s+(?:media\s+)?(?:packets?|headers?)", s, re.I) \
                and re.search(r"receiver|per[- ]receiver", s, re.I) and not NEG_WORD.search(s):
            fail(1, f"a sentence says forward media packets carry receiver state, which rule 8 forbids: {s[:90]}")
if not PRB_PLACE:
    if K["dscp_probe"] is not None and not re.search(r"EF\s*\(%d\)" % K["dscp_probe"], PRB.prose_all):
        fail("C-13", f"the probe section does not restate its DSCP as EF ({K['dscp_probe']}) (C-13)")
if not (REP_PLACE and PRB_PLACE):
    if not re.search(r"IPv4", BOTH) or not re.search(r"unicast", BOTH, re.I) or not re.search(r"reverse", BOTH, re.I):
        fail("C-100", "the sections do not restate IPv4 unicast on the reverse port (C-100)")

# ---------------------------------------------------------------------------
# DoD: evidence tags and open items
# ---------------------------------------------------------------------------
for name, sec in (("Receiver report", REP), ("One-way probe", PRB)):
    if (name == "Receiver report" and REP_PLACE) or (name == "One-way probe" and PRB_PLACE):
        continue
    last_prose_tagged = True
    for b in sec.blocks:
        kind = block_kind(b)
        text = "\n".join(b)
        if kind in ("heading", "code"):
            continue
        if kind == "table":
            if not (has_evidence(text) or last_prose_tagged):
                fail("DoD", f"a table of the {name} section carries no evidence tag and follows an untagged paragraph")
            continue
        words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", text)))
        tagged = has_evidence(text)
        if words >= 5 and not tagged:
            fail("DoD", f"a paragraph of the {name} section carries no evidence tag: " + text.splitlines()[0][:90])
        last_prose_tagged = tagged or words < 5
    for i, b in enumerate(sec.prose):
        if re.match(r"\W*open\s+items?\b", b.strip(), re.I):
            if not re.search(r"\bowner\b", b, re.I) or not re.search(r"falsifier", b, re.I):
                fail("DoD", f"an open item of the {name} section names no owner or no falsifier: "
                     + b.splitlines()[0][:90])

# ---------------------------------------------------------------------------
# The 004-02 checker and the design checker (repository-level, not overridden)
# ---------------------------------------------------------------------------
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-8:])
            fail(6, f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(6, f"could not run cargo run --bin wire-check: {error}")

    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
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
        design = subprocess.run(["python3", "doc/kickoff/check-design.py", "doc/design"],
                                capture_output=True, text=True, timeout=300)
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

print("OK: 6/6 acceptance criteria for 004-20, DoD tags and open items present, consistency checked")
PY
