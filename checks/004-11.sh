#!/usr/bin/env bash
# 004-11 — content messages: common header, audio and dense controllers.
#
# RED-phase document check. It validates doc/design/spec/wire.md and
# doc/design/spec/wire.toml against every acceptance criterion of the ticket,
# and re-runs the 004-02 checker, the wire_tables tests, the design checker
# and the other 004-* checks.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo, the design checker and the
# sibling 004-* checks) are skipped with a NOTE, because they always target
# the real repository.
#
# The section checked is "## Content messages" of wire.md; its level-3
# subsections are found by their headings: "header" (the common content-
# message header), "audio" and "dense". The invalid-forms table is the one
# whose first column is "Invalid form".
#
# What is checked, per acceptance criterion:
#
#   AC 1  The common content-message header is a five-column field table
#         whose fields tile it from byte 0 with no gap and no overlap; its
#         size in bytes is stated in prose and equals the sum of the widths;
#         it has a kind field and a length field, and the section says how an
#         unknown kind is skipped by its length.
#   AC 2  Audio: 192 bytes per channel per default slice (48 x 4, the value
#         is a whole-number check) and 1 152 bytes for six channels, or a
#         different per-channel figure with a stated justification; f32 and
#         Q1.31 named with their signalling in the manifest (004-26); a
#         stated maximum channel count per message; channel order = the
#         manifest's channel map in manifest order; no per-channel id byte;
#         the audio field table has a 192-byte row.
#   AC 3  Dense controllers: a field table, with the width, the format and a
#         maximum count stated, and one value per frame (or a stated
#         sub-rate). An open item, owned by epic 004, about the dense field
#         set, with the falsifier "a producer cannot express a value it must
#         send without a major change" and the word "falsifier".
#   AC 4  At least 6 rows in the invalid-forms table. Every CamelCase name in
#         an outcome cell is a member of the closed 13-name enumeration, and
#         each row names a member or says it is skipped, ignored or cannot
#         occur. The nine forms of ticket Work 4 are covered by keyword. A
#         statement that a controller second stream whose controllers would
#         have fitted the audio packet is valid (C-181, permissive) must
#         exist, and no row or validity sentence rejects that case for fitting.
#   AC 5  wire.toml has a content-header message, an audio message and a
#         dense message (messages or section messages of "media packet"),
#         each with max_size equal to the sum of its field widths, member
#         rejection reasons, and fields that agree with the wire.md table
#         field by field (offset and width in bytes, primitive type). The
#         counted fields' max_count appears in the wire.md section. Then
#         `cargo run --bin wire-check` is clean and wire_tables is 12/0/0.
#
# Budget (C-137), checked as arithmetic in the section's budget statements
# (units naming C-137, the header cap, 100 bytes or the budget): the section
# must state the header size H in bytes, and must EITHER
#   (a) raise the overflow: a budget unit naming 004-14 as owner and a
#       falsifier, OR
#   (b) show a fit: a unit that says a named optional part is omitted (the
#       measurement extension, 9 bytes, or the announcement container, 49
#       bytes) on a fallback path, with when, and H <= the omitted size, since
#       16 + 22 + 49 + 9 + 4 = 100 bytes.
# A section that states H and is silent on the budget fails.
#
# Also: every new block of the section carries an evidence tag; the 13
# constants this ticket cites are cited in the section; the design checker
# prints OK; the sibling 004-*.sh checks pass.
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
# A small TOML reader (copied from checks/004-13.sh, the same subset parser
# that checks/004-08.sh and 004-09.sh use).
# ---------------------------------------------------------------------------
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
# Constants and the closed enumeration
# ---------------------------------------------------------------------------
MEMBERS_EXPECTED = [
    "TooShort", "BadMagic", "VersionMismatch", "ForeignDomain", "UnconfiguredDomain",
    "UnknownStream", "UnadmittedSource", "BadLength", "BadChecksum", "BadPacketIndex",
    "WrongEpoch", "OversizeControl", "ReservedNonZero",
]
MEMBER_SET = set(MEMBERS_EXPECTED)
CIDS = ["C-01", "C-02", "C-03", "C-04", "C-08", "C-80", "C-134", "C-137", "C-158",
        "C-181", "C-205", "C-209", "C-219"]
PLACEHOLDER = re.compile(r"to\s+be\s+filled\s+by\s+004-11", re.I)
JUSTIFY = re.compile(r"because|since|justif|\brounds?\b|hence", re.I)
HEADER_FIXED_CAP = {"media": 16, "slice": 22, "container": 49, "extension": 9, "checksum": 4}

# ---------------------------------------------------------------------------
# Load wire.md, the closed enumeration and the Content messages section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    fail(1, f"{wire_path} does not exist")
    wire = ""
lines = wire.splitlines()
heads = heading_index(lines)

enum_names = None
for _, hdr, rows in parse_tables(wire):
    h = [norm_name(x) for x in hdr]
    if len(h) >= 2 and h[0].startswith("value") and h[1] == "name":
        enum_names = [r[1] for r in rows if len(r) >= 2]
        break
if enum_names is None or set(enum_names) != MEMBER_SET:
    fail(4, "wire.md's closed Rejection enumeration is not the 13 names this check knows "
         "(a new name needs a major version; wire.md, Closed enumeration)")

sec_i = None
for i, (_, _, title) in enumerate(heads):
    if re.fullmatch(r"content\s+messages?", title, re.I):
        sec_i = i
        break

placeholder = True
content_text = ""
content_prose = ""
content_units = []
subs = []
if sec_i is None:
    fail("DoD", "no 'Content messages' section heading in wire.md")
else:
    level = heads[sec_i][1]
    start = heads[sec_i][0] + 1
    end = len(lines)
    for j in range(sec_i + 1, len(heads)):
        if heads[j][1] <= level:
            end = heads[j][0]
            break
    content_text = "\n".join(lines[start:end])
    content_prose = strip_fences(content_text)
    content_units = units(content_prose)
    for j in range(sec_i + 1, len(heads)):
        s = heads[j][0]
        if s >= end:
            break
        if heads[j][1] <= level:
            break
        e = heads[j + 1][0] if j + 1 < len(heads) else len(lines)
        e = min(e, end)
        subs.append((heads[j][2], "\n".join(lines[s + 1:e])))
    placeholder = bool(PLACEHOLDER.search(content_text)) or "|" not in content_text

if placeholder:
    for ac in range(1, 6):
        fail(ac, "the Content messages section is still the 004-11 placeholder")
    fail("DoD", "the Content messages section is still the 004-11 placeholder")


def sub_text(pattern):
    for title, text in subs:
        if re.search(pattern, title, re.I):
            return title, text
    return None, None


def prose_of(text):
    return strip_fences("\n".join(ln for ln in text.splitlines() if not ln.lstrip().startswith("|")))


H = None
audio_rows = None
dense_rows = None
header_rows = None
header_text = ""
audio_text = ""
dense_text = ""

# ---------------------------------------------------------------------------
# AC 1 — the common content-message header
# ---------------------------------------------------------------------------
if not placeholder:
    _, header_text = sub_text(r"header")
    if header_text is None:
        fail(1, "no content-message header subsection (a heading naming the header)")
    else:
        header_rows = field_rows(header_text)
        if header_rows is None:
            fail(1, "the content-message header has no five-column field table "
                 "(Offset, Width, Type, Meaning, Valid range)")
        else:
            expected = 0
            tiled = True
            for r in header_rows:
                off, w = offset_of(r[0]), wbytes(r[1])
                if off is None or w is None or w <= 0 or w != int(w):
                    fail(1, f"header row '{r[3][:40]}' has no whole-byte offset and width")
                    tiled = False
                    break
                if off != expected:
                    fail(1, f"header field at offset {off} does not start at byte {expected}: "
                         "the fields must tile the header with no gap and no overlap")
                    tiled = False
                    break
                expected += int(w)
            if tiled:
                H = expected
                stated = []
                for s in re.split(r"(?<=[.;])\s+|\n", prose_of(header_text)):
                    if re.search(r"header", s, re.I) and not re.search(r"POC|retired|\bMsg\b", s):
                        stated += [int(m.group(1)) for m in re.finditer(r"(\d+)[\s-]*bytes?\b", s, re.I)]
                if not stated:
                    fail(1, f"the header's size in bytes is not stated (its fields sum to {H} bytes)")
                elif any(x != H for x in stated):
                    fail(1, f"the header states {stated} bytes but its field widths sum to {H} bytes")
            table_text = "\n".join(" | ".join(r) for r in header_rows)
            if not re.search(r"\bkind\b", table_text, re.I):
                fail(1, "the header has no kind field")
            if not re.search(r"\blength\b", table_text, re.I):
                fail(1, "the header has no length field")
            if not re.search(r"\bskip", prose_of(header_text), re.I):
                fail(1, "the header section does not say how a receiver skips an unknown kind "
                     "by its length")

# ---------------------------------------------------------------------------
# AC 2 — the audio message
# ---------------------------------------------------------------------------
if not placeholder:
    _, audio_text = sub_text(r"audio")
    if audio_text is None:
        fail(2, "no audio subsection")
    else:
        vals = set()
        for m in re.finditer(r"(\d[\d ,\u00a0\u202f]*\d|\d)\s*(?:-\s*)?bytes?\s+(?:per|each|for\s+each)\s+channel",
                             audio_text, re.I):
            vals.add(num(m.group(1)))
        for _m in re.finditer(r"48\s*[x\u00d7*]\s*4\b", audio_text):
            vals.add(192)
        for m in re.finditer(r"per\s+channel[^.;\n]{0,40}?(\d[\d ,\u00a0\u202f]*\d|\d)\s*bytes?",
                             audio_text, re.I):
            vals.add(num(m.group(1)))
        if 192 not in vals:
            fail(2, "the audio layout does not state 192 bytes per channel per default slice (48 x 4)")
        for v in sorted(vals - {192}):
            if not JUSTIFY.search(audio_text):
                fail(2, f"the audio layout states {v} bytes per channel, not 192 (48 x 4), "
                     "with no justification")
        big = set()
        for m in re.finditer(r"(\d[\d ,\u00a0\u202f]*\d|\d)\s*bytes?\s+of\s+audio", audio_text, re.I):
            big.add(num(m.group(1)))
        if not re.search(r"\b1[ \u00a0\u202f,]?152\b", audio_text):
            fail(2, "1 152 bytes of audio for six channels is not stated (C-04; Work 6 confirmation)")
        for v in sorted(big - {1152}):
            if not JUSTIFY.search(audio_text):
                fail(2, f"the audio states {v} bytes of audio where six channels give 1 152, "
                     "with no justification")
        if not re.search(r"\b(6|six)\b", audio_text, re.I):
            fail(2, "six channels per stream at the defaults (C-04) are not stated")
        if not (re.search(r"\bf32\b", audio_text, re.I) and re.search(r"Q1\.31", audio_text, re.I)):
            fail(2, "the sample format (f32 or Q1.31, C-03) is not stated")
        sents = [s for s in re.split(r"(?<=[.;])\s+|\n", audio_text) if s.strip()]
        if not any(re.search(r"f32|Q1\.31", s, re.I) and re.search(r"manifest|004-26", s, re.I)
                   for s in sents):
            fail(2, "the sample-format signalling is not stated as declared in the manifest (004-26)")
        if not any(re.search(r"channel", s, re.I)
                   and re.search(r"\b(max\w*|at most|up to|no more than)\b", s, re.I)
                   and re.search(r"\d", s) for s in sents):
            fail(2, "the maximum channel count per audio message is not stated "
                 "(a sentence with 'maximum' or 'at most', a number and 'channel')")
        if not any(re.search(r"(?:channel[- ]map|manifest)\W+(?:\w+\W+){0,4}?order\b", s, re.I)
                   for s in sents):
            fail(2, "the channel order is not stated as the manifest's channel map, in manifest order")
        if not re.search(r"no\s+(?:per-?channel\s+|channel\s+)?id(?:entifier)?s?\b"
                         r"|without\s+(?:a\s+|any\s+)?(?:per-?channel\s+)?id(?:entifier)?\b",
                         audio_text, re.I):
            fail(2, "the audio layout does not say there is no per-channel id byte (C-04)")
        audio_rows = field_rows(audio_text)
        if audio_rows is None:
            fail(2, "the audio layout has no five-column field table")
        elif not any(re.search(r"\b192\b", " | ".join(r)) for r in audio_rows):
            fail(2, "the audio field table has no row of 192 bytes per channel per slice")

# ---------------------------------------------------------------------------
# AC 3 — dense controllers and their open item
# ---------------------------------------------------------------------------
if not placeholder:
    _, dense_text = sub_text(r"dense")
    if dense_text is None:
        fail(3, "no dense-controller subsection")
    else:
        dense_rows = field_rows(dense_text)
        if dense_rows is None:
            fail(3, "the dense controller message has no five-column field table")
        if not re.search(r"\bwidth\b", dense_text, re.I):
            fail(3, "the dense controller layout does not state its width")
        if not re.search(r"\bformat\b", dense_text, re.I):
            fail(3, "the dense controller layout does not state its format")
        if not re.search(r"\b(?:max\w*|at most|up to)\b[^.;\n]{0,40}\d|\d[^.;\n]{0,40}\bmax\w*\b",
                         dense_text, re.I):
            fail(3, "the dense controller maximum count is not stated")
        if not re.search(r"per\s+frame|sub-?rate|per\s+slice", dense_text, re.I):
            fail(3, "the dense controller sampling (one value per frame, or a stated sub-rate) "
                 "is not stated")
    cand = [u for u in content_units if re.search(r"open\s+item", u, re.I)
            and re.search(r"dense", u, re.I)]
    joined = "\n".join(cand)
    if not cand:
        fail(3, "no open item about the dense-controller field set")
    else:
        if not re.search(r"\bfield", joined, re.I):
            fail(3, "the dense-controller open item does not name the field set")
        if not re.search(r"epic\s+004", joined, re.I):
            fail(3, "the dense-controller open item has no owner, epic 004")
        if not re.search(r"falsif", joined, re.I):
            fail(3, "the dense-controller open item has no falsifier")
        if not re.search(r"cannot\s+express\s+a\s+value\s+it\s+must\s+send\s+without\s+a\s+major",
                         joined, re.I):
            fail(3, "the dense-controller falsifier is not the stated one: 'a producer cannot "
                 "express a value it must send without a major change'")

# ---------------------------------------------------------------------------
# AC 4 — the invalid forms, the enumeration and the C-181 permissive rule
# ---------------------------------------------------------------------------
CONCEPTS = {
    "unknown kind": r"\bkind\b",
    "length": r"\blength\b|\bBadLength\b|frames\s*[x\u00d7*]",
    "channel count": r"channel",
    "sample format": r"sample|format|Q1\.31|f32",
    "overrunning the slice": r"overrun|past the end|runs? past|beyond the slice|exceed\w*\s+(?:the\s+|its\s+)?slice",
    "audio in a gap slice": r"\bgap\b",
    "bundle member streams (C-205)": r"bundle|C-205|member stream",
    "packet over the cap (C-181, C-137)": r"C-137|C-181|100[- ]byte|\bcap\b",
    "audio in a controller second stream": r"second stream|controller stream",
}
if not placeholder:
    inv_rows = None
    for _, hdr, rows in parse_tables(content_text):
        if hdr and re.search(r"invalid", hdr[0], re.I):
            inv_rows = [r + [""] * (2 - len(r)) if len(r) < 2 else r for r in rows
                        if any(c.strip() for c in r)]
            break
    if inv_rows is None:
        fail(4, "no invalid-forms table (first column 'Invalid form')")
    else:
        if len(inv_rows) < 6:
            fail(4, f"only {len(inv_rows)} invalid form(s); at least 6 are required")
        for r in inv_rows:
            form, outcome = r[0], r[1]
            names = re.findall(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b", outcome)
            nonmember = [n for n in names if n not in MEMBER_SET]
            if nonmember:
                fail(4, f"row '{form[:50]}' names non-member outcome(s) {nonmember}: the "
                     "enumeration is closed within a major version (wire.md, Closed enumeration)")
            here = [n for n in names if n in MEMBER_SET]
            if not here and not re.search(r"skip|ignor|cannot\s+occur|not\s+a\s+rejection",
                                          outcome, re.I):
                fail(4, f"row '{form[:50]}' gives no member outcome and is not skipped or ignored")
            if re.search(r"second stream|controller", form, re.I) and re.search(r"\bfit", form, re.I) \
                    and here:
                fail(4, f"row '{form[:50]}' rejects a controller second stream for fitting: "
                     "a controller second stream whose controllers would have fitted the audio "
                     "packet is valid (C-181, permissive)")
        for concept, pat in CONCEPTS.items():
            if not any(re.search(pat, " | ".join(r), re.I) for r in inv_rows):
                fail(4, f"no invalid form covers: {concept} (ticket Work 4)")
    permissive = False
    for u in content_units:
        if re.search(r"second\s+stream|controller\s+stream", u, re.I) and re.search(r"\bfit", u, re.I) \
                and re.search(r"\b(never|valid|permissive|may|accepted|not\s+rejected)\b", u, re.I) \
                and not re.search(r"^\s*\|", u):
            permissive = True
    for u in content_units:
        if re.search(r"^\s*\|", u):
            continue
        for s in re.split(r"(?<=[.;])\s+", u):
            if re.search(r"second\s+stream|controller\s+stream", s, re.I) and re.search(r"\bfit", s, re.I) \
                    and re.search(r"\b(reject\w*|drop\w*|BadLength)\b", s, re.I) \
                    and not re.search(r"\b(never|not|no|valid|permissive|may|accepted)\b", s, re.I):
                fail(4, "a validity sentence rejects a controller second stream for fitting: "
                     + s.strip()[:90])
    if not permissive:
        fail(4, "no statement that a controller second stream whose controllers would have fitted "
             "the audio packet is valid (C-181, permissive)")

# ---------------------------------------------------------------------------
# C-137 — the header budget, as arithmetic and as a stated decision
# ---------------------------------------------------------------------------
if not placeholder:
    budget = [u for u in content_units
              if re.search(r"C-137|header cap|100[- ]byte|\b100 bytes|budget", u, re.I)]
    if not budget:
        fail("C-137", "the budget is silent: the content-message header is never placed against "
             "the 100-byte cap (16 + 22 + 49 + 9 + 4 = 100 already fills it; C-137)")
    elif H is not None and not any(re.search(r"(?<![\d.])%d(?![\d.])" % H, u) for u in budget):
        fail("C-137", f"the budget statement does not state the header size of {H} bytes")
    if H is not None and budget:
        overflow = any(re.search(r"004-14", u) and re.search(r"owner", u, re.I)
                       and re.search(r"falsif", u, re.I) for u in budget)
        fits = False
        for u in budget:
            m = re.search(r"\bomit\w*", u, re.I)
            if not m:
                continue
            clause = u[m.start():m.start() + 160]
            omitted = 0
            if re.search(r"extension|measurement", clause, re.I):
                omitted += HEADER_FIXED_CAP["extension"]
            if re.search(r"container|announcement", clause, re.I):
                omitted += HEADER_FIXED_CAP["container"]
            if omitted and H <= omitted and re.search(r"\bwhen\b|\bif\b|fallback", u, re.I) \
                    and re.search(r"\b100\b|cap", u, re.I):
                fits = True
        if not (overflow or fits):
            fail("C-137", f"the {H}-byte header cannot sit beside the 49-byte container and the "
                 "9-byte extension (16 + 22 + 49 + 9 + 4 = 100 bytes, C-137): either name the "
                 "fallback part that is omitted and when, with the arithmetic, or raise the "
                 "overflow as an open item owned by 004-14 with a falsifier")

# ---------------------------------------------------------------------------
# DoD — tags, constants
# ---------------------------------------------------------------------------
if not placeholder:
    prev_tagged = True
    for block in blocks(content_text):
        stripped = block.strip()
        if not stripped:
            continue
        content_lines = [ln for ln in stripped.splitlines() if not ln.lstrip().startswith("#")]
        if not content_lines:
            prev_tagged = True
            continue
        body = "\n".join(content_lines).strip()
        is_table = all(ln.strip().startswith("|") for ln in content_lines if ln.strip())
        is_code = content_lines[0].lstrip().startswith("```")
        if has_evidence(body):
            prev_tagged = True
            continue
        if (is_table or is_code) and prev_tagged:
            prev_tagged = True
            continue
        fail("DoD", "this block of the Content messages section carries no evidence tag "
             "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
             + body.splitlines()[0][:90])
        prev_tagged = False
    for cid in CIDS:
        if not re.search(r"\b%s\b" % re.escape(cid), content_text):
            fail("DoD", f"the Content messages section does not cite {cid}")

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

msgs = []
if isinstance(document, dict):
    msgs = [m for m in (document.get("message", []) or []) if isinstance(m, dict)]


def find_msg(*words):
    for m in msgs:
        n = str(m.get("name", "")).lower()
        if all(w in n for w in words):
            return m
    return None


def md_check(label, msg, md_rows, md_text, ac_word):
    if msg is None:
        fail(5, f"{tables_path} has no {label} message (a message or a section of \"media packet\")")
        return None
    if msg.get("layer") != "media":
        fail(5, f"{label}: layer must be \"media\", not {msg.get('layer')!r}")
    if msg.get("section_of") not in (None, "media packet"):
        fail(5, f"{label}: section_of must be \"media packet\" or absent")
    fields = [f for f in (msg.get("field", []) or []) if isinstance(f, dict)]
    if not fields:
        fail(5, f"{label}: declares no fields")
        return None
    end_bits = 0
    for f in fields:
        off, wid = f.get("offset"), f.get("width")
        if not isinstance(off, int) or not isinstance(wid, int) or off % 8 or wid % 8:
            fail(5, f"{label}: field {f.get('name')!r} is not whole bytes")
            return None
        n = 1
        if "count" in f:
            n = f.get("max_count")
            if not isinstance(n, int):
                fail(5, f"{label}: counted field {f.get('name')!r} has no integer max_count")
                return None
            if md_text is not None and not re.search(r"(?<![\d.])%d(?![\d.])" % n, md_text):
                fail(5, f"{label}: max_count {n} of {f.get('name')!r} is not stated in wire.md's section")
        end_bits = max(end_bits, off + wid * n)
    computed = end_bits // 8
    if msg.get("max_size") != computed:
        fail(5, f"{label}: max_size {msg.get('max_size')} is not the size its fields compute to "
             f"({computed} bytes)")
    rr = msg.get("rejection_reasons")
    if not isinstance(rr, list):
        fail(5, f"{label}: rejection_reasons is missing")
    else:
        bad = [r for r in rr if r not in MEMBER_SET]
        if bad:
            fail(5, f"{label}: rejection_reasons name non-member(s) {bad}")
    if md_rows is not None:
        tp_all = sorted((f["offset"] // 8, f["width"] // 8, str(f.get("type", ""))) for f in fields)
        mp = sorted((offset_of(r[0]), int(wbytes(r[1]) or 0), r[2]) for r in md_rows)
        if [(a, b) for a, b, _ in tp_all] != [(a, b) for a, b, _ in mp]:
            fail(5, f"{label}: wire.toml's (offset, width) in bytes {[(a, b) for a, b, _ in tp_all]} "
                 f"do not agree with wire.md's table {[(a, b) for a, b, _ in mp]}")
        else:
            for (a, b, t), (_, _, mt) in zip(tp_all, mp):
                if re.fullmatch(r"u(?:8|16|32|64|128)|i(?:8|16|32|64|128)|f32|Q1\.31", mt.strip(), re.I) \
                        and t.lower() != mt.strip().lower():
                    fail(5, f"{label}: field at byte {a} is {t!r} in wire.toml but {mt!r} in wire.md")
    return computed


if not placeholder:
    _, hdr_md_text = sub_text(r"header")
    _, aud_md_text = sub_text(r"audio")
    _, den_md_text = sub_text(r"dense")
    hdr_rows = field_rows(hdr_md_text) if hdr_md_text else None
    aud_rows = field_rows(aud_md_text) if aud_md_text else None
    den_rows = field_rows(den_md_text) if den_md_text else None
else:
    hdr_md_text = aud_md_text = den_md_text = None
    hdr_rows = aud_rows = den_rows = None

hdr_m = find_msg("content", "header")
aud_m = find_msg("audio")
den_m = find_msg("dense")
hdr_size = md_check("content-message header", hdr_m, hdr_rows, hdr_md_text, "header")
if hdr_size is not None and H is not None and hdr_size != H and hdr_m is not None:
    fail(5, f"wire.toml's content header is {hdr_size} bytes, wire.md's is {H}")
md_check("audio", aud_m, aud_rows, aud_md_text, "audio")
md_check("dense controller", den_m, den_rows, den_md_text, "dense")

# ---------------------------------------------------------------------------
# Repository-level gates (skipped under overrides, as in checks/004-13.sh)
# ---------------------------------------------------------------------------
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-8:])
            fail(5, f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(5, f"could not run cargo run --bin wire-check: {error}")

    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
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
    print("NOTE DoD: document paths overridden; repository-level checks skipped")

# ---------------------------------------------------------------------------
if fails:
    for ac, msg in fails:
        label = ac if isinstance(ac, str) else f"AC{ac}"
        print(f"FAIL {label}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 5/5 acceptance criteria for 004-11, the C-137 budget, the invalid forms and the "
      "C-181 rule, DoD tags present")
PY
PY_RC=$?

SIB_RC=0
if [ -z "${WIRE_MD:-}" ] && [ -z "${WIRE_TOML:-}" ]; then
    for f in checks/004-*.sh; do
        [ "$(basename "$f")" = "004-11.sh" ] && continue
        if ! out=$(bash "$f" 2>&1); then
            echo "FAIL sibling $(basename "$f"):"
            printf '%s\n' "$out" | tail -n 4
            SIB_RC=1
        fi
    done
else
    echo "NOTE siblings: document paths overridden; sibling 004-* checks skipped"
fi

if [ "$PY_RC" -ne 0 ] || [ "$SIB_RC" -ne 0 ]; then
    exit 1
fi
exit 0
