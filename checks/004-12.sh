#!/usr/bin/env bash
# 004-12 — content messages: sparse keyed controllers and read-only transport state.
#
# RED-phase document check. It validates the two new subsections of the
# "Content messages" section of doc/design/spec/wire.md and the two new
# [[message]] tables of doc/design/spec/wire.toml against every acceptance
# criterion of the ticket, and re-runs the 004-02 checker (cargo run --bin
# wire-check and the wire_tables tests) and the design checker once each.
# It does not run the other checks/004-*.sh: the per-change suite runs every
# file in checks/, so a sibling loop here would be quadratic.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo and the design checker) are
# skipped with a NOTE, because they always target the real repository.
#
# Where the new text lives: every heading of the Content messages section
# whose title names "sparse", "keyed" or "transport" is a new subsection, and
# its whole subtree (up to the next heading of the same or a higher level) is
# the "new area". Field tables are found in the new area by content:
#
#   entry table   the five-column field table with a row naming the voice key
#   transport     the five-column field table with a row naming the tempo
#   layout        the five-column field table with a kind row at offset 0 that is
#                 not the transport table (the sparse keyed message's layout)
#
# The invalid-forms rows are those of every table in the new area whose first
# header cell is "Invalid form". The closed enumeration is read from wire.md
# at run time; nothing about it is pinned here.
#
# What is checked, per acceptance criterion:
#
#   AC 1  Sparse keyed controllers: an entry field table (voice key, field id,
#         target value, value format, start offset) that tiles from byte 0;
#         its byte size stated in prose and equal to the sum of its widths; a
#         stated maximum number of entries per message N; a linear ramp over
#         the slice; the layout table (common header, count, entries) whose
#         count width cw and entry width W give the fixed maximum
#         E = 3 + cw + N x W, stated in prose; the toml message's max_size = E,
#         its counted field's max_count = N and its fields agree with the layout.
#   AC 2  Transport state: a field table that tiles from byte 0 (common header,
#         then the body), its byte size stated and equal to the sum of widths;
#         a tempo field whose literal 120 beats per minute and whose encoding
#         (a stated value, or a stated scale) reproduces 120 and fits the
#         width; a playing/stopped row; no unset, gap or absent value in its
#         rows; the receiver-side statements: read-only (a sentence saying so)
#         and receivers never write it back (a sentence saying so); the
#         pre-arrival default (stopped, playhead at zero, 120 bpm); the gap
#         rule (held, degraded); the transport field set either as an open
#         item (a paragraph that begins "Open item", with an owner and a
#         falsifier) or stated as decided ([decision], no open item). Headings
#         and cross-references are not open items.
#   AC 3  At least 6 invalid-form rows in the new area, across the two types.
#         The outcome of each row names only closed-enumeration members, or
#         says the message is skipped, ignored, not counted, cannot occur or
#         not a rejection. Concepts: entry count above maximum (BadLength,
#         exactly); length mismatch (BadLength, exactly); unknown field id;
#         offset beyond the slice's frames; duplicate (voice, field); invalid
#         transport state value; a sparse message in an audio member stream of
#         a bundle source (C-205). Every reason a row names must be listed in
#         the rejection_reasons of one of the two toml messages.
#   AC 4  Passes the 004-02 checker: `cargo run --bin wire-check` clean and
#         `cargo test -p hadronyche-tools --test wire_tables` 12/0/0 (skipped
#         under overrides); the toml declares the two messages (names containing
#         "sparse" or "keyed", and "transport" with "state") with fields that
#         agree with the md layout tables in offset, width and type, a max_size
#         equal to the md-stated size, and rejection_reasons that are members.
#   consistency  Both messages use the common content-message header (kind at
#         +0, width 1; length at +1, width 2); each kind code is the one the
#         Kind codes table assigns to it, the codes are unique, and the toml
#         kind field states the same code; the new text does not say a kind
#         precedes the audio message; C-205 and C-181 are restated (a bundle
#         source's controllers always in their own member stream; a single-
#         stream source may share the audio packet while they fit); a sentence
#         names transport state together with the 100-byte cap or the Budget
#         (C-137); the constants C-01, C-02, C-137, C-141, C-181, C-184, C-199,
#         C-200 and C-205 are cited in the new area.
#   DoD   every prose unit of 5 or more words in the new area carries an
#         evidence tag; a [code] unit names a POC pointer; every declared open
#         item (a paragraph beginning "Open item") names an owner and a
#         falsifier.
#
# Compatibility with the siblings: the new subsection headings contain none of
# the words "header", "audio" or "dense", and the new toml message names contain
# neither "audio" nor "dense", so that checks/004-11.sh still finds its own
# subsections and messages.
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


def fail(ac, msg):
    fails.append((ac, msg))


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


# ---------------------------------------------------------------------------
# Markdown helpers (same conventions as checks/004-13.sh)
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


def units(text):
    """Claim units: paragraphs, split again at list items and headings."""
    out = []
    cur = []
    in_fence = False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if not s:
            if cur:
                out.append("\n".join(cur))
                cur = []
            continue
        if cur and re.match(r"^([-*+]\s|\d+\.\s|#)", s):
            out.append("\n".join(cur))
            cur = []
        cur.append(line)
    if cur:
        out.append("\n".join(cur))
    return out


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(block):
    return bool(EVIDENCE.search(block))


def norm_name(s):
    return re.sub(r"[^a-z0-9]", "", s.lower())


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


def num(text):
    return int(re.sub(r"[\s,\u00a0\u202f]", "", text))


def wbytes(cell):
    """Width cell -> bytes (first integer; 'bits' divides by 8)."""
    m = re.search(r"\d+", cell or "")
    if not m:
        return None
    n = int(m.group(0))
    if re.search(r"\bbits?\b", cell, re.I) and not re.search(r"\bbytes?\b", cell, re.I):
        return n / 8
    return n


def offset_of(cell):
    m = re.search(r"\d+", cell or "")
    return int(m.group(0)) if m else None


def field_rows(text):
    """Data rows of the first five-column field table in text, or None."""
    for _, hdr, rows in parse_tables(text):
        if is_field_table(hdr):
            return [r + [""] * (5 - len(r)) if len(r) < 5 else r for r in rows]
    return None


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

NUMPAT = r"\d{1,3}(?:[   ]\d{3})+|\d+"


def nums(s):
    return [int(re.sub(r"[   ]", "", m.group(0))) for m in re.finditer(NUMPAT, s)]


def sents(text):
    return [s.strip() for s in re.split(r"(?<=[.;])\s+|\n", text) if s.strip()]


def rtext(r):
    return " | ".join(r)


def tile(rows, label, ac):
    """Rows must tile from byte 0 with no gap or overlap; returns the end byte."""
    exp = 0
    for r in rows:
        off, w = offset_of(r[0]), wbytes(r[1])
        if off is None or w is None or w <= 0 or w != int(w):
            fail(ac, f"{label}: row '{r[3][:40]}' has no whole-byte offset and width")
            return None
        if off != exp:
            fail(ac, f"{label}: field at offset {off} does not start at byte {exp}: "
                 "the fields must tile with no gap and no overlap")
            return None
        exp += int(w)
    return exp


# ---------------------------------------------------------------------------
# Load wire.md; the closed enumeration and the Kind codes are read at run time
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    fail("AC 1", f"{wire_path} does not exist")
    wire = ""
lines = wire.splitlines()
heads = heading_index(lines)

enum_names = []
for _, hdr, rows in parse_tables(wire):
    h = [norm_name(x) for x in hdr]
    if len(h) >= 2 and h[0].startswith("value") and h[1] == "name":
        enum_names = [r[1] for r in rows if len(r) >= 2 and r[1]]
        break
MEMBER_SET = set(enum_names)
if not MEMBER_SET:
    fail("DoD", "wire.md has no closed Rejection enumeration table (Value (u8) | Name | ...)")

sec_lo = sec_hi = 0
sec_level = 2
sec_i = None
for i, (_, _, title) in enumerate(heads):
    if re.fullmatch(r"content\s+messages?", title, re.I):
        sec_i = i
        break
if sec_i is None:
    fail("DoD", "no 'Content messages' section heading in wire.md")
else:
    sec_level = heads[sec_i][1]
    sec_lo = heads[sec_i][0] + 1
    sec_hi = len(lines)
    for j in range(sec_i + 1, len(heads)):
        if heads[j][1] <= sec_level:
            sec_hi = heads[j][0]
            break
content_text = "\n".join(lines[sec_lo:sec_hi])


def subtree_indices(pattern):
    """Line indices of every heading in the section matching pattern, with its subtree."""
    out = set()
    for j in range(sec_i + 1 if sec_i is not None else 0, len(heads)):
        s, lvl, title = heads[j]
        if s < sec_lo or s >= sec_hi:
            continue
        if not re.search(pattern, title, re.I):
            continue
        e = sec_hi
        for k in range(j + 1, len(heads)):
            if heads[k][1] <= lvl:
                e = heads[k][0]
                break
        e = min(e, sec_hi)
        out.update(range(s + 1, e))
    return sorted(out)


new_lines = [lines[i] for i in subtree_indices(r"sparse|keyed|transport")]
new_text = "\n".join(new_lines)
prose_lines = [ln for ln in new_lines if not ln.lstrip().startswith(("|", "#"))]
prose = strip_fences("\n".join(prose_lines))
prose_sents = sents(prose)
if not new_text.strip():
    fail("AC 1", "no subsection of the Content messages section names sparse keyed controllers "
         "or transport state")

# Field tables of the new area, padded to five cells.
ftabs = []
inv_rows = []
for _, hdr, rows in parse_tables(new_text):
    if is_field_table(hdr):
        ftabs.append([r + [""] * (5 - len(r)) if len(r) < 5 else r for r in rows])
    elif hdr and re.search(r"invalid", hdr[0], re.I):
        inv_rows += [r + [""] * (2 - len(r)) if len(r) < 2 else r for r in rows
                     if any(c.strip() for c in r)]


def declared_units(text):
    """Units that DECLARE an open item: their first prose line begins 'Open item'."""
    out = []
    for u in units(text):
        body = [ln.strip().lstrip("*_ ").strip() for ln in u.splitlines()
                if ln.strip() and not ln.lstrip().startswith(("#", "|"))]
        if body and re.match(r"open\s+item\b", body[0], re.I):
            out.append(u)
    return out


def has_row(t, pat):
    return any(re.search(pat, rtext(r), re.I) for r in t)


entry = next((t for t in ftabs if has_row(t, r"\bvoice\b") and not has_row(t, r"\btempo\b")), None)
transport = next((t for t in ftabs if has_row(t, r"\btempo\b")), None)
sparse_layout = next((t for t in ftabs if t is not transport and t is not entry
                      and any(offset_of(r[0]) == 0 and re.search(r"\bkind\b", rtext(r), re.I)
                              for r in t)), None)

# ---------------------------------------------------------------------------
# AC 1 — sparse keyed controllers
# ---------------------------------------------------------------------------
W = None
N = None
E = None
cw = None
if entry is None:
    fail("AC 1", "no sparse keyed entry field table (five columns, a row for the voice key)")
else:
    W = tile(entry, "sparse entry table", "AC 1")
    for label, pat in [("voice key", r"\bvoice\b"), ("field id", r"\bfield\b"),
                       ("target value", r"\btarget\b"),
                       ("value format", r"\bformat\b|\bf32\b|Q1\.31"),
                       ("start offset", r"\boffset\b|\bstart\b")]:
        if not has_row(entry, pat):
            fail("AC 1", f"the sparse entry table has no row for {label}")
    if W is not None:
        stated = set()
        for s in prose_sents:
            if re.search(r"\bentr(?:y|ies)\b", s, re.I) and re.search(r"\bbytes?\b", s, re.I):
                stated.update(int(re.sub(r"[   ]", "", m.group(1)))
                              for m in re.finditer(r"(" + NUMPAT + r")\s*-?\s*bytes?\b", s, re.I))
        if W not in stated:
            fail("AC 1", f"the sparse entry's byte size is not stated as the sum of its widths, "
                 f"{W} bytes")
        for s in prose_sents:
            for m in re.finditer(r"\bentr(?:y|ies)\b[^.;]{0,30}?\b(?:is|are|=|takes|occupies|measures)\b\s*(" + NUMPAT
                                 + r")\s*-?\s*bytes?\b|(" + NUMPAT + r")\s*-?\s*bytes?\s+(?:per|each)\s+entry\b"
                                 + r"|\b(?:each|per)\s+entry\b[^.;]{0,20}?\b(?:is|=)\s*(" + NUMPAT + r")\s*-?\s*bytes?\b",
                                 s, re.I):
                val = next(g for g in m.groups() if g)
                v = int(re.sub(r"[ \u00a0\u202f]", "", val))
                if v != W:
                    fail("AC 1", f"a sentence states the entry as {v} bytes, not the sum of its widths, "
                         f"{W}: " + s[:80])

# The maximum for ENTRIES per message: a keyword (at most, up to, no more
# than, maximum, max) directly before the number and the word 'entries', or
# 'entries' followed by the keyword and the number. A sentence that gives the
# maximum of a size (the fixed maximum size in bytes) does not match.
MAX_ENTRIES = [
    re.compile(r"\b(?:at most|up to|no more than|maximum(?: of)?|max(?:imum)?)\s+(" + NUMPAT + r")\s+entr", re.I),
    re.compile(r"\bentr\w*[^.;]{0,30}?\b(?:at most|up to|no more than|maximum(?: of)?)\s+(" + NUMPAT + r")\b", re.I),
]
maxima = set()
for s in prose_sents:
    for rx in MAX_ENTRIES:
        for m in rx.finditer(s):
            maxima.add(int(re.sub(r"[ \u00a0\u202f]", "", m.group(1))))
N = None
if not maxima:
    fail("AC 1", "no stated maximum number of sparse entries per message "
         "(a sentence with 'at most' or 'maximum', a number and 'entries')")
elif len(maxima) > 1:
    fail("AC 1", f"the sentences state conflicting maxima for entries per message: {sorted(maxima)}")
else:
    N = maxima.pop()

if not any(re.search(r"\blinear", s, re.I) and re.search(r"\bramp", s, re.I) for s in prose_sents):
    fail("AC 1", "the linear ramp of a sparse entry over the slice is not stated")

if sparse_layout is None:
    fail("AC 1", "no sparse keyed message layout table (kind at offset 0, count, entries)")
else:
    hdr_ok = (len(sparse_layout) >= 3 and offset_of(sparse_layout[0][0]) == 0
              and offset_of(sparse_layout[1][0]) == 1
              and wbytes(sparse_layout[0][1]) == 1 and wbytes(sparse_layout[1][1]) == 2)
    if not hdr_ok:
        fail("AC 1", "the sparse message layout does not start with the common content-message "
             "header (kind at +0, width 1; length at +1, width 2)")
    for r in sparse_layout[2:]:
        if re.search(r"\bcount\b", rtext(r), re.I) and offset_of(r[0]) is not None:
            cw = wbytes(r[1])
            break
    if cw is None:
        fail("AC 1", "the sparse message layout has no count field with a stated width")
    ent_rows = [r for r in sparse_layout if re.search(r"\bentr", rtext(r), re.I)]
    if W is not None and not any(wbytes(r[1]) == W for r in ent_rows):
        fail("AC 1", f"the sparse message layout's entries row does not have the entry width {W}")
    if cw is not None and hdr_ok:
        end = tile(sparse_layout, "sparse message layout", "AC 1")
        if N is not None and W is not None and end is not None:
            E = 3 + int(cw) + N * W
            cands = []
            for s in prose_sents:
                if re.search(r"\bbytes?\b", s, re.I) and re.search(r"\b(?:max\w*|fixed)\b", s, re.I) \
                        and re.search(r"sparse|keyed|message", s, re.I):
                    cands += nums(s)
            if E not in cands:
                fail("AC 1", f"the sparse message's fixed maximum size is not stated as "
                     f"3 + count + N x entry = {E} bytes (3 + {cw} + {N} x {W})")

# ---------------------------------------------------------------------------
# AC 2 — read-only transport state
# ---------------------------------------------------------------------------
T = None
if transport is None:
    fail("AC 2", "no transport state field table with a tempo field (C-199)")
else:
    hdr_ok = (len(transport) >= 3 and offset_of(transport[0][0]) == 0
              and offset_of(transport[1][0]) == 1
              and wbytes(transport[0][1]) == 1 and wbytes(transport[1][1]) == 2)
    if not hdr_ok:
        fail("AC 2", "the transport state table does not start with the common content-message "
             "header (kind at +0, width 1; length at +1, width 2)")
    T = tile(transport, "transport state table", "AC 2")
    if T is not None:
        stated = set()
        for s in prose_sents:
            if re.search(r"transport", s, re.I) and re.search(r"\bbytes?\b", s, re.I):
                stated.update(int(re.sub(r"[   ]", "", m.group(1)))
                              for m in re.finditer(r"(" + NUMPAT + r")\s*-?\s*bytes?\b", s, re.I))
        if T not in stated:
            fail("AC 2", f"the transport state's byte size is not stated as the sum of its widths, "
                 f"{T} bytes")
    if has_row(transport, r"\bunset\b|\bgap\b|\babsent\b|not\s+set|no\s+state|\bnone\b"):
        fail("AC 2", "the transport state table has an unset, gap, absent or none value; "
             "a receiver assumes the default (C-184) and needs no such value")
    if not has_row(transport, r"playing|stopped|running"):
        fail("AC 2", "the transport state table has no playing/stopped field")
    tempo_rows = [r for r in transport if re.search(r"\btempo\b", rtext(r), re.I)]
    if tempo_rows:
        tempo_text = rtext(tempo_rows[0])
    else:
        tempo_text = ""
    tempo_scope = tempo_text + "\n" + "\n".join(s for s in prose_sents
                                               if re.search(r"tempo|bpm|beats?", s, re.I))
    if not re.search(r"\b120\s*(?:beats?\s*per\s*minute|beats?|bpm)\b", tempo_scope, re.I):
        fail("AC 2", "the tempo is not stated as 120 beats per minute (C-199)")
    tw = None
    if tempo_rows:
        tw = wbytes(tempo_rows[0][1])
    enc = None
    S = None
    m = re.search(r"(" + NUMPAT + r")\s*(?:=|means|is|equals|encodes|stands for|->|→)\s*120\s*(?:beats?|bpm)",
                  tempo_scope, re.I)
    if m:
        enc = int(re.sub(r"[   ]", "", m.group(1)))
    if enc is None:
        m = re.search(r"120\s*(?:beats?(?:\s*per\s*minute)?|bpm)\s*(?:=|is|are|means|encoded as|->|→)\s*(" + NUMPAT + ")",
                      tempo_scope, re.I)
        if m:
            enc = int(re.sub(r"[   ]", "", m.group(1)))
    if enc is None:
        m = re.search(r"(" + NUMPAT + r")\s*(?:units?|steps?|ticks?)\s*(?:per|to)\s*(?:bpm|beats?)", tempo_scope, re.I)
        if m:
            S = int(re.sub(r"[   ]", "", m.group(1)))
    if enc is None and S is None:
        m = re.search(r"1\s*(?:units?|steps?|ticks?)\s*(?:=|is)\s*(?:(\d+)\s*/\s*(\d+)|(0?\.\d+))\s*(?:bpm|beats?)",
                      tempo_scope, re.I)
        if m:
            S = int(m.group(2)) // int(m.group(1)) if m.group(1) else round(1 / float(m.group(3)))
    if enc is None and S is not None:
        enc = 120 * S
    if enc is None:
        fail("AC 2", "the encoding of 120 beats per minute is not stated (a value that means 120 bpm, "
             "or a scale of units per bpm)")
    elif tw is not None and not (0 <= enc < 2 ** (8 * int(tw))):
        fail("AC 2", f"the tempo encoding {enc} does not fit the tempo field's {int(tw)} bytes")

    for s in prose_sents:
        if re.search(r"stopped", s, re.I) and re.search(r"\bzero\b|\b0\b", s, re.I) \
                and re.search(r"\b120\b", s) and re.search(r"arriv|before|until|no transport state|not yet|default", s, re.I):
            break
    else:
        fail("AC 2", "the pre-arrival default (stopped, playhead at zero, tempo 120 bpm; C-184, C-199) "
             "is not restated as receiver behaviour")
    if not any(re.search(r"\bgap\b", s, re.I) and re.search(r"\bheld\b|\bholds?\b", s, re.I)
               and re.search(r"degraded", s, re.I) for s in prose_sents):
        fail("AC 2", "the gap rule (hold the last transport state, mark the output degraded; C-141, C-200) "
             "is not restated as receiver behaviour")
    if not any(re.search(r"read[- ]only", s, re.I) for s in prose_sents):
        fail("AC 2", "the transport state is not stated to be read-only")
    if not any(re.search(r"\bback\b", s, re.I)
               and re.search(r"\b(?:never|must not|does not|do not|cannot)\b", s, re.I)
               and re.search(r"receiver", s, re.I) for s in prose_sents):
        fail("AC 2", "the section does not say that receivers never write the transport state back")

# The transport field set is either an open item (a declared unit, owner and
# falsifier) or stated as decided ([decision], no open item). Either is valid;
# a cross-reference to someone else's open item is not a declaration.
ts_items = [u for u in declared_units(new_text)
            if re.search(r"transport", u, re.I) and re.search(r"\bfield", u, re.I)]
if ts_items:
    joined = "\n".join(ts_items)
    if not re.search(r"\bowner\b|\bowned\b|\bepic\s+\d+", joined, re.I):
        fail("AC 2", "the transport field set's open item names no owner")
    if not re.search(r"falsif", joined, re.I):
        fail("AC 2", "the transport field set's open item has no falsifier")
    if not (re.search(r"cannot\s+express", joined, re.I) and re.search(r"\bmajor\b", joined, re.I)):
        fail("AC 2", "the transport falsifier is not stated: a producer cannot express a value it "
             "must send without a major change")
else:
    decided = [u for u in units(new_text)
               if not u.lstrip().startswith(("#", "|")) and u not in declared_units(new_text)
               and re.search(r"transport", u, re.I) and re.search(r"\bfield", u, re.I)
               and re.search(r"\[decision\]", u)]
    if not decided:
        fail("AC 2", "the transport state field set is neither an open item (owner and falsifier) "
             "nor stated as decided ([decision])")

# ---------------------------------------------------------------------------
# AC 3 — invalid forms and outcomes
# ---------------------------------------------------------------------------
NONRJ = re.compile(r"skip|ignor|not\s+counted|not\s+a\s+rejection|cannot\s+occur", re.I)
if len(inv_rows) < 6:
    fail("AC 3", f"only {len(inv_rows)} invalid form(s) in the new area; at least 6 are required "
         "across the two types")
named_all = set()
for r in inv_rows:
    form, outcome = r[0], " | ".join(r[1:])
    names = re.findall(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b", outcome)
    nonmember = [n for n in names if n not in MEMBER_SET]
    if nonmember:
        fail("AC 3", f"row '{form[:50]}' names non-member outcome(s) {nonmember}: the enumeration "
             "is closed within a major version")
    here = [n for n in names if n in MEMBER_SET]
    named_all.update(here)
    if not here and not NONRJ.search(outcome):
        fail("AC 3", f"row '{form[:50]}' gives no member outcome and is not skipped, ignored or "
             "not counted")
concepts = [
    ("entry count above maximum", r"\b(?:entr\w*|count)\b.*\b(?:above|exceed\w*|more than|over|beyond|too many)\b|\b(?:above|exceed\w*|more than|too many)\b.*\b(?:entr\w*|count)\b", "BadLength"),
    ("length mismatch", r"\blength\b.*\b(?:mismatch\w*|disagree\w*|differ\w*|not\s+(?:equal|match\w*|agree\w*)|does\s+not\s+match)\b", "BadLength"),
    ("unknown field id", r"\b(?:unknown|undefined|undeclared|not\s+(?:declared|in))\b.*\bfield\b|\bfield\b.*\b(?:unknown|undefined|undeclared)\b", None),
    ("offset beyond slice frames", r"\boffset\b.*\b(?:beyond|past|exceed\w*|above|after|greater|larger)\b.*\bframes?\b", None),
    ("duplicate (voice, field)", r"\b(?:duplicate\w*|twice|more than once|repeated)\b", None),
    ("invalid transport state value", r"\btransport\b.*\b(?:invalid|out of range|outside|not in|unknown|bad|reserved)\b|\b(?:invalid|out of range)\b.*\btransport\b", None),
    ("sparse message in an audio member stream (C-205)", r"(?:sparse|keyed)\b.*\baudio\b|\baudio\b.*(?:sparse|keyed)\b", None),
]
for label, pat, exact in concepts:
    rows_c = [r for r in inv_rows if re.search(pat, r[0], re.I)]
    if not rows_c:
        fail("AC 3", f"no invalid form covers: {label}")
        continue
    if exact is not None and not all(exact in " | ".join(r[1:]) for r in rows_c):
        fail("AC 3", f"the {label} form does not give {exact} (the enumeration's count / "
             "declared-shape member)")

# ---------------------------------------------------------------------------
# Toml agreement (AC 1, 2, 4)
# ---------------------------------------------------------------------------
document = None
tables_text = read_text(tables_path)
if tables_text is None:
    fail("AC 4", f"{tables_path} does not exist")
else:
    try:
        document = parse_toml(tables_text)
    except TomlError as error:
        fail("AC 4", f"{tables_path} could not be read by this check: {error}")
msgs = []
if isinstance(document, dict):
    msgs = [m for m in (document.get("message", []) or []) if isinstance(m, dict)]


def find_toml(pattern):
    for m in msgs:
        if re.search(pattern, str(m.get("name", "")), re.I):
            return m
    return None


def toml_check(label, msg, md_rows, want_size, want_max_count, ac):
    if msg is None:
        fail(ac, f"{tables_path} has no {label} message")
        return None
    if msg.get("layer") != "media":
        fail(ac, f"{label}: layer must be \"media\", not {msg.get('layer')!r}")
    fields = [f for f in (msg.get("field", []) or []) if isinstance(f, dict)]
    if not fields:
        fail(ac, f"{label}: declares no fields")
        return None
    end_bits = 0
    for f in fields:
        off, wid = f.get("offset"), f.get("width")
        if not isinstance(off, int) or not isinstance(wid, int) or off % 8 or wid % 8:
            fail(ac, f"{label}: field {f.get('name')!r} is not whole bytes")
            return None
        n = 1
        if "count" in f:
            n = f.get("max_count")
            if not isinstance(n, int):
                fail(ac, f"{label}: counted field {f.get('name')!r} has no integer max_count")
                return None
            if want_max_count is not None and n != want_max_count:
                fail(ac, f"{label}: max_count {n} of {f.get('name')!r} is not the stated maximum "
                     f"of {want_max_count} entries")
        end_bits = max(end_bits, off + wid * n)
    computed = end_bits // 8
    if msg.get("max_size") != computed:
        fail(ac, f"{label}: max_size {msg.get('max_size')} is not the size its fields compute to "
             f"({computed} bytes)")
    if want_size is not None and computed != want_size:
        fail(ac, f"{label}: the fields compute to {computed} bytes but wire.md states {want_size}")
    rr = msg.get("rejection_reasons")
    if not isinstance(rr, list):
        fail(ac, f"{label}: rejection_reasons is missing")
    else:
        bad = [r for r in rr if r not in MEMBER_SET]
        if bad:
            fail(ac, f"{label}: rejection_reasons name non-member(s) {bad}")
    if md_rows is not None:
        tp = sorted((f["offset"] // 8, f["width"] // 8, str(f.get("type", ""))) for f in fields)
        mp = sorted(((offset_of(r[0]) if offset_of(r[0]) is not None else -1),
                     int(wbytes(r[1]) or 0), r[2].strip()) for r in md_rows)
        if [(a, b) for a, b, _ in tp] != [(a, b) for a, b, _ in mp]:
            fail(ac, f"{label}: wire.toml's (offset, width) in bytes {[(a, b) for a, b, _ in tp]} "
                 f"do not agree with wire.md's table {[(a, b) for a, b, _ in mp]}")
        else:
            for (a, b, t), (_, _, mt) in zip(tp, mp):
                if re.fullmatch(r"u(?:8|16|32|64|128)|i(?:8|16|32|64|128)|f32|Q1\.31", mt, re.I) \
                        and t.lower() != mt.lower():
                    fail(ac, f"{label}: field at byte {a} is {t!r} in wire.toml but {mt!r} in wire.md")
    return msg


sparse_msg = find_toml(r"sparse|keyed")
transport_msg = find_toml(r"transport.*state|state.*transport")
if sparse_msg is not None and sparse_msg is transport_msg:
    sparse_msg = None
if sparse_layout is not None and cw is not None and W is not None and N is not None:
    toml_check("sparse keyed controller message", sparse_msg, sparse_layout, E, N, "AC 1")
elif sparse_msg is None:
    fail("AC 1", f"{tables_path} has no sparse keyed controller message (a name containing "
         "\"sparse\" or \"keyed\")")
if transport is not None and T is not None:
    toml_check("transport state message", transport_msg, transport, T, None, "AC 2")
elif transport_msg is None:
    fail("AC 2", f"{tables_path} has no transport state message (a name containing \"transport\" "
         "and \"state\")")

# Rejection reasons of the rows must be declared by the messages.
declared = set()
for m in (sparse_msg, transport_msg):
    if m is not None and isinstance(m.get("rejection_reasons"), list):
        declared |= set(m["rejection_reasons"])
for name in sorted(named_all - declared):
    fail("AC 4", f"the invalid-forms rows name {name}, which no new toml message lists in "
         "rejection_reasons")

# ---------------------------------------------------------------------------
# Consistency with 004-11 and the kind codes
# ---------------------------------------------------------------------------
codes = {}
code_rows = []
for _, hdr, rows in parse_tables(content_text):
    h = [norm_name(x) for x in hdr]
    if len(h) >= 2 and h[0] == "code" and h[1] == "kind":
        for r in rows:
            if len(r) >= 2 and re.match(r"^\d+$", r[0].strip()):
                code_rows.append((int(r[0]), r[1]))
        break
seen_codes = [c for c, _ in code_rows]
if len(set(seen_codes)) != len(seen_codes):
    fail("consistency", f"the Kind codes table assigns a code twice: {seen_codes}")
sparse_code = next((c for c, k in code_rows if re.search(r"sparse|keyed", k, re.I)), None)
transport_code = next((c for c, k in code_rows if re.search(r"transport", k, re.I)), None)
if sparse_code is None:
    fail("consistency", "the Kind codes table has no row for sparse keyed controllers")
if transport_code is None:
    fail("consistency", "the Kind codes table has no row for transport state")
if sparse_layout is not None and sparse_code is not None:
    kr = sparse_layout[0]
    kv = re.search(r"\d+", kr[4] or "")
    if not kv or int(kv.group(0)) != sparse_code:
        fail("consistency", f"the sparse layout's kind code is {kv.group(0) if kv else None}, "
             f"but the Kind codes table assigns {sparse_code} to sparse keyed controllers")
if transport is not None and transport_code is not None:
    kr = transport[0]
    kv = re.search(r"\d+", kr[4] or "")
    if not kv or int(kv.group(0)) != transport_code:
        fail("consistency", f"the transport layout's kind code is {kv.group(0) if kv else None}, "
             f"but the Kind codes table assigns {transport_code} to transport state")
for msg, code, label in [(sparse_msg, sparse_code, "sparse"), (transport_msg, transport_code, "transport")]:
    if msg is None or code is None:
        continue
    kf = next((f for f in (msg.get("field", []) or []) if isinstance(f, dict) and f.get("name") == "kind"), None)
    kv = re.search(r"\d+", str(kf.get("valid", "")) if kf else "")
    if not kv or int(kv.group(0)) != code:
        fail("consistency", f"the {label} message's toml kind field does not state code {code}")

for s in prose_sents:
    if re.search(r"(?:sparse|keyed|transport)[^.;]*\b(?:precedes?|before|ahead of)\b[^.;]*\baudio", s, re.I):
        fail("consistency", "a new sentence says a kind precedes the audio message: " + s[:90])
if not re.search(r"first content message|no kind precedes", content_text, re.I):
    fail("consistency", "the audio-first rule (the first content message; no kind precedes it) is "
         "not stated in the section")
if not any(re.search(r"C-205", s) and re.search(r"member\s+stream", s, re.I) for s in prose_sents):
    fail("consistency", "C-205 is not restated: a bundle source's controllers always ride in their own "
         "member stream")
if not any(re.search(r"C-181", s) and re.search(r"\bshare[sd]?\b", s, re.I) and re.search(r"\bfit", s, re.I)
           for s in prose_sents):
    fail("consistency", "C-181 is not restated: a single-stream source may share the audio packet "
         "while the controllers fit")
if not any(re.search(r"transport", s, re.I) and re.search(r"\bcap\b|budget|C-137|100[- ]byte", s, re.I)
           and re.search(r"count|controller", s, re.I) for s in prose_sents):
    fail("consistency", "no sentence names transport state together with the 100-byte cap or the "
         "Budget (C-137): say whether it counts as a controller message")
for cid in ["C-01", "C-02", "C-137", "C-141", "C-181", "C-184", "C-199", "C-200", "C-205"]:
    if not re.search(r"\b%s\b" % re.escape(cid), new_text):
        fail("consistency", f"the new area does not cite {cid}")

# ---------------------------------------------------------------------------
# DoD — evidence tags on every prose unit of 5+ words; [code] names the POC;
# every open item names an owner and a falsifier
# ---------------------------------------------------------------------------
for u in units(new_text):
    prose_body = "\n".join(ln for ln in u.splitlines()
                           if ln.strip() and not ln.lstrip().startswith(("#", "|")))
    if not prose_body.strip():
        continue
    if len(re.findall(r"[A-Za-z0-9][\w'-]*", prose_body)) < 5:
        continue
    if not has_evidence(u):
        fail("DoD", "a prose unit of the new area carries no evidence tag "
             "([code] [measured] [kernel-doc] [standard] [memory] [decision]): " + prose_body.splitlines()[0][:90])
    if re.search(r"\[code\]", u, re.I) and not re.search(r"\bPOC\b|poc/|\bPart\b", u):
        fail("DoD", "a [code] unit of the new area names no POC pointer: " + prose_body.splitlines()[0][:90])
for u in declared_units(new_text):
    if not re.search(r"owner|epic\s+004|004-\d+", u, re.I):
        fail("DoD", "an open item of the new area names no owner: " + u.splitlines()[0][:90])
    if not re.search(r"falsif", u, re.I):
        fail("DoD", "an open item of the new area names no falsifier: " + u.splitlines()[0][:90])

# ---------------------------------------------------------------------------
# Repository-level gates (skipped under overrides, as in checks/004-11.sh)
# ---------------------------------------------------------------------------
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-8:])
            fail("AC 4", f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC 4", f"could not run cargo run --bin wire-check: {error}")

    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail("AC 4", f"could not run the wire_tables tests: {error}")
    if proc is not None:
        out = proc.stdout + proc.stderr
        if proc.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail("AC 4", f"cargo test -p hadronyche-tools --test wire_tables failed:\n{tail}")
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if result is None:
            fail("AC 4", "the wire_tables test run printed no 'test result:' line")
        else:
            m = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result)
            if not m:
                fail("AC 4", f"could not read the test counts from: {result}")
            elif (int(m.group(1)), int(m.group(2)), int(m.group(3))) != (12, 0, 0):
                fail("AC 4", "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: " + result)

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
    print("NOTE AC 4: document paths overridden; cargo and the design checker skipped")

# ---------------------------------------------------------------------------
if fails:
    for ac, msg in fails:
        print(f"FAIL {ac}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 4/4 acceptance criteria for 004-12, the consistency rules and the DoD tags")
PY
PY_RC=$?

if [ "$PY_RC" -ne 0 ]; then
    exit 1
fi
exit 0
