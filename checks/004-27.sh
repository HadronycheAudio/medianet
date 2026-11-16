#!/usr/bin/env bash
# 004-27 — path-MTU probe pair: the probe request and reply, their exemption and their validity rules.
#
# RED-phase document check. It validates the "Path-MTU probe pair" section of
# doc/design/spec/wire.md and the two probe messages in doc/design/spec/wire.toml
# against the acceptance criteria of the ticket, and re-runs the wire-check
# binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing
# on throwaway fixtures only: when either path is overridden the cargo checks are
# skipped with a NOTE, because they always target the real repository.
#
# Numbers are read at run time: C-06 (the fallback MTU), C-17 (the control bound),
# C-13 (CS5), the Hello's common header and its type-code block, the admission
# request's intended packet size row, the refusal enumeration, and the Rejection
# reasons with their Layers cells. The Markdown helpers and the TOML reader are
# the ones of the 004-18 checker (Python 3.10 here, no tomllib).
#
# The author contract (headings and labels the check locates):
#
#   ## Path-MTU probe pair       prose: the size rule (the intended media packet size,
#       C-116), the DF requirement (rule 11, IPv4 only, C-100), CS5 (40) (C-13), the
#       fallback sequence (intended size, then the 1 280-byte fallback, C-06, then
#       PathMtuTooSmall, C-173), "no reply", the exemption (C-116, the only one)
#   ### <title with probe and request, or probe and reply>   one per probe message;
#       the seven common control header rows in the hello's order and widths; a
#       Message type row with a code in the block the hello gives the probe pair
#       (32 to 35); a padding row whose valid range is zero; the rows tile; a
#       sentence "maximum size ... N bytes" equal to the sum; the request's intended
#       packet size at the admission request's width and a probe identifier or
#       request id; the reply echoes an identifier
#   ### Matching ...           how a reply identifies its probe: names the reply and
#       the request id or the probe identifier
#   ### Validity rules         an Invalid form | Required outcome table
#   Open item paragraph        names an owner (a ticket or an epic) and a falsifier
#   wire.toml                  one [[message]] per probe message, named with "probe",
#       layer control, dscp 40, max_size above the control bound, exempt_from_control_bound
#       "C-116", fields tiling from bit 0 to max_size, rows agreeing with the markdown
#       table row for row; no other message carries the exemption
#
# What is checked, per acceptance criterion:
#   AC 1  the probe pair is specified with sizes, DF requirement and matching rule, and
#         the fallback sequence ends in PathMtuTooSmall; the message tables (header rows,
#         code in the block, tiling, maximum size, padding, the intended size row) and the
#         rule sentences; the toml code field inside the block
#   AC 2  the exemption cites C-116 and is carried by exactly the probe messages of the
#         toml file; every sentence of the section that says "exempt" cites C-116 only
#         (C-17 is allowed as the bound it exempts from)
#   AC 3  at least 4 invalid forms with outcomes; the probe-size, non-zero padding, reply
#         matching and generic control rows each have their outcome; every outcome names
#         an enumerated reason, and every packet reason named lists the control layer
#   AC 4  wire.toml parses; each probe message: layer, dscp 40, a max_size above the bound,
#         tiling and sum, rejection reasons that are members of the closed enumeration;
#         the markdown and toml field rows agree; the refusal enumeration of wire.toml
#         equals wire.md's; when the paths are not overridden: `cargo run --bin wire-check`
#         reports "checks clean" and `cargo test -p hadronyche-tools --test wire_tables`
#         has 0 failed and at least 12 passed (the count that stood before this ticket)
#   DoD   every prose paragraph of 5+ words carries an evidence tag, every subsection has
#         one; every "Open item" paragraph names an owner and a falsifier; the checker
#         reads the working tree only and runs no other check
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
import os
import re
import subprocess
import sys

wire_path, tables_path = sys.argv[1], sys.argv[2]
overridden = bool(os.environ.get("WIRE_MD") or os.environ.get("WIRE_TOML"))
CONSTANTS_PATH = "doc/design/02-constants.md"
RESULTS = []


def check(ac, ok, text):
    RESULTS.append((ac, bool(ok)))
    print(("PASS " if ok else "FAIL ") + ac + ": " + text)
    return bool(ok)


def fail(ac, msg):
    return check(ac, False, msg)


def norm(text):
    # Digit groups written with a space or a no-break space: "1 280" reads as "1280".
    return re.sub(r"(?<=\d)[   ](?=\d{3}(?!\d))", "", text)


# ---------------------------------------------------------------------------
# Markdown helpers and the TOML reader: copied from the 004-18 checker.
# ---------------------------------------------------------------------------
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
# Markdown helpers (conventions of the earlier 004 checkers)
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




# A small TOML reader (copied from the earlier 004 checkers: offline, Python 3.10, no tomllib).
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
# Inputs
# ---------------------------------------------------------------------------
WIRE_RAW = read_text(wire_path)
if WIRE_RAW is None:
    fail("AC1", f"{wire_path} does not exist")
    WIRE_RAW = ""
WIRE = norm(WIRE_RAW)
lines = WIRE.splitlines()
HEADS = heading_index(lines)
TOML_TEXT = read_text(tables_path)
if TOML_TEXT is None:
    fail("AC4", f"{tables_path} does not exist")
    TOML_TEXT = ""
CONST = norm(read_text(CONSTANTS_PATH) or "")


def const_val(cid):
    for line in CONST.splitlines():
        if line.startswith(f"| {cid} |"):
            cells = line.split("|")
            return cells[3].strip() if len(cells) > 3 else ""
    return ""


def first_int(s):
    m = re.search(r"\d+", s)
    return int(m.group(0)) if m else None


FALLBACK_MTU = first_int(const_val("C-06"))
_m = re.search(r"at most (\d+) bytes", const_val("C-17"))
CTRL_BOUND = int(_m.group(1)) if _m else None
_m = re.search(r"control CS5 \((\d+)\)", const_val("C-13"))
DSCP_CONTROL = int(_m.group(1)) if _m else None
IPV4_UDP_HEADERS = 28  # 20-byte IPv4 header (RFC 791) plus 8-byte UDP header (RFC 768) [standard]
check("AC1", bool(FALLBACK_MTU and CTRL_BOUND) and FALLBACK_MTU - IPV4_UDP_HEADERS == CTRL_BOUND,
      f"the fallback arithmetic holds: C-06's {FALLBACK_MTU} bytes less {IPV4_UDP_HEADERS} bytes of IPv4 and UDP "
      f"headers is C-17's {CTRL_BOUND} bytes of UDP payload")


def span_of(pattern, level):
    for i, (idx, lvl, title) in enumerate(HEADS):
        if lvl == level and re.search(pattern, title, re.I):
            end = len(lines)
            for j in range(i + 1, len(HEADS)):
                if HEADS[j][1] <= lvl:
                    end = HEADS[j][0]
                    break
            return idx, end
    return None


def body(span):
    return lines[span[0]:span[1]] if span else []


# ---------------------------------------------------------------------------
# Landed sections the probe pair reads from (never rewritten)
# ---------------------------------------------------------------------------
HELLO_SPAN = span_of(r"^hello$", 2)
HSUBS = subsections(body(HELLO_SPAN))
_hdr = find_sub(HSUBS, r"common\s+control\s+header")
HDR = fields_of(first_table(_hdr[0][1], is_field_table) if _hdr else None, "AC1",
                "the hello's common control header")
HDR_SIG = [(f["off"], f["bits"], f["type"]) for f in HDR[:7]]
hdr_bytes = sum(size_bits(f) for f in HDR) // 8
check("AC1", len(HDR) == 7 and hdr_bytes == 32 and all(
    f["off"] == sum(size_bits(g) for g in HDR[:i]) // 8 for i, f in enumerate(HDR)),
      f"the hello's common control header is seven rows tiling 32 bytes (found {len(HDR)} rows, {hdr_bytes} bytes)")

PROBE_BLOCK = None
PROBE_ASSIGNED = ""
_cds = find_sub(HSUBS, r"control\s+message\s+type\s+codes")
if _cds:
    for _s, _hd, _rows in parse_tables(_cds[0][1]):
        for _r in _rows:
            if len(_r) >= 3 and re.search(r"probe", _r[1], re.I):
                _n = [int(x) for x in re.findall(r"\d+", _r[0])]
                if _n:
                    PROBE_BLOCK = range(_n[0], _n[-1] + 1)
                    PROBE_ASSIGNED = _r[2]
check("AC1", PROBE_BLOCK is not None and "004-27" in PROBE_ASSIGNED,
      "the hello's type-code table gives the path-MTU probe pair a block assigned to 004-27 "
      f"(block {PROBE_BLOCK.start if PROBE_BLOCK else '?'} to {PROBE_BLOCK.stop - 1 if PROBE_BLOCK else '?'})")

ADM_SPAN = span_of(r"^admission\s+messages$", 2)
ASUBS = subsections(body(ADM_SPAN))
_req = find_sub(ASUBS, r"^admission\s+request$")
REQ_FIELDS = fields_of(first_table(_req[0][1], is_field_table) if _req else None, "AC1",
                       "the admission request")
REQ_INTENDED = [f for f in REQ_FIELDS if re.match(r"\s*intended\s+packet\s+size", f["meaning"], re.I)]
check("AC1", len(REQ_INTENDED) == 1 and REQ_INTENDED[0]["bits"] == 16 and "65507" in REQ_INTENDED[0]["valid"],
      "the admission request still carries the intended packet size as a u16 with a maximum of 65 507 "
      "(the size the probe pair takes)")

REF = {}
_rf = find_sub(ASUBS, r"refusal\s+reasons")
if _rf:
    for _s, _hd, _rows in parse_tables(_rf[0][1]):
        for _r in _rows:
            if len(_r) >= 2 and re.fullmatch(r"\d+", _r[0].strip()) and re.fullmatch(r"[A-Z][A-Za-z0-9]+", _r[1].strip()):
                REF[_r[1].strip()] = int(_r[0])

MEMBERS = {}
MEMBER_LAYERS = {}
_rj = span_of(r"rejection[-\s]+reasons?", 2)
for _s, _hd, _rows in parse_tables("\n".join(body(_rj))):
    for _r in _rows:
        if len(_r) >= 4 and re.fullmatch(r"\d+", _r[0].strip()) and re.fullmatch(r"[A-Z][A-Za-z0-9]+", _r[1].strip()):
            MEMBERS[_r[1].strip()] = int(_r[0])
            MEMBER_LAYERS[_r[1].strip()] = _r[3]
check("AC3", len(MEMBERS) == 13, f"the Rejection reasons enumeration of wire.md has 13 members (found {len(MEMBERS)})")

# ---------------------------------------------------------------------------
# wire.toml
# ---------------------------------------------------------------------------
try:
    TOML = parse_toml(TOML_TEXT)
except Exception as error:  # the reader raises TomlError or ValueError on a malformed file
    TOML = {}
    fail("AC4", f"wire.toml does not parse with the subset reader: {error}")
TMSGS = [m for m in TOML.get("message", []) if isinstance(m, dict)]
TREF = {r["name"]: r["value"] for r in TOML.get("refusal", [])
        if isinstance(r, dict) and "name" in r and "value" in r}
check("AC4", bool(REF) and len(REF) == 18 and TREF == REF,
      "the refusal enumeration of wire.toml equals the 18 names and values of wire.md's Refusal reasons "
      f"(md {len(REF)}, toml {len(TREF)})")

# ---------------------------------------------------------------------------
# The probe section
# ---------------------------------------------------------------------------
PROBE_SPAN = span_of(r"^path[-\s]*mtu\s+probe\s+pair$", 2)
PROBE_LINES = body(PROBE_SPAN)
PROBE = "\n".join(PROBE_LINES)
check("AC1", PROBE_SPAN is not None and "to be filled" not in PROBE.lower(),
      "wire.md has a '## Path-MTU probe pair' section with its own text, not the placeholder")

PSUBS = subsections(PROBE_LINES)


def is_msg(title):
    tl = title.lower()
    return "probe" in tl and ("request" in tl or "reply" in tl) and not re.search(r"match|validity|identif|binding", tl)


MSG = [(t, x) for t, x in PSUBS if is_msg(t)]
REQ_TEXT = [x for t, x in MSG if "request" in t.lower()]
REP_TEXT = [x for t, x in MSG if "reply" in t.lower()]
check("AC1", len(REQ_TEXT) == 1 and len(REP_TEXT) == 1,
      f"the section has one probe request and one probe reply, each a '###' subsection (found {len(MSG)})")
req_text = REQ_TEXT[0] if len(REQ_TEXT) == 1 else ""
rep_text = REP_TEXT[0] if len(REP_TEXT) == 1 else ""
REQ_FL = fields_of(first_table(req_text, is_field_table), "AC1", "the probe request table") if req_text else []
REP_FL = fields_of(first_table(rep_text, is_field_table), "AC1", "the probe reply table") if rep_text else []
MSG_FL = [("request", REQ_FL, req_text), ("reply", REP_FL, rep_text)]


def tile_problems(fl):
    cur, probs = 0, []
    for f in fl:
        if f["off"] != cur:
            probs.append(f"'{f['meaning'][:30]}' starts at byte {f['off']}, expected {cur}")
        if size_bits(f) % 8:
            probs.append(f"'{f['meaning'][:30]}' is not a whole number of bytes")
        cur = f["off"] + size_bits(f) // 8
    return cur, probs


def type_field(fl):
    for f in fl:
        if re.match(r"\s*message\s+type", f["meaning"], re.I):
            return f
    return None


hdr_bad, tile_bad, size_bad, code_bad, pad_bad, id_bad = [], [], [], [], [], []
codes_seen = {}
for kind, fl, text in MSG_FL:
    if len(fl) < 7 or [(f["off"], f["bits"], f["type"]) for f in fl[:7]] != HDR_SIG:
        hdr_bad.append(kind)
    total, probs = tile_problems(fl)
    tile_bad += [f"{kind}: {p}" for p in probs]
    m = re.search(r"maximum\s+size[^.]*?(\d+)\s*bytes", text, re.I)
    if not m or int(m.group(1)) != total:
        size_bad.append(f"{kind}: states {m.group(1) if m else 'no maximum size'}, rows sum to {total}")
    tf = type_field(fl)
    code = first_int(tf["valid"]) if tf else None
    if code is None or PROBE_BLOCK is None or code not in PROBE_BLOCK:
        code_bad.append(f"{kind}: message type {code} is outside the probe block")
    else:
        codes_seen[kind] = code
    if kind == "request":
        if not any(re.search(r"pad", f["meaning"] + " " + f["type"], re.I) and re.search(r"\b0\b|zero", f["valid"], re.I)
                   for f in fl):
            pad_bad.append("the request has no zero-valued padding row")
    elif any(re.search(r"pad", f["meaning"] + " " + f["type"], re.I) for f in fl):
        pad_bad.append("the reply has a padding row")
    if kind == "request":
        if not any(re.search(r"probe\s+identifier|request\s+id", f["meaning"], re.I) for f in fl):
            id_bad.append("request has no probe identifier or request id row")
        if not any(re.match(r"\s*intended\s+packet\s+size", f["meaning"], re.I) and REQ_INTENDED
                   and f["bits"] == REQ_INTENDED[0]["bits"] for f in fl):
            id_bad.append("request has no intended packet size row at the admission request's width")
    else:
        if not any(re.search(r"request\s+id", f["meaning"], re.I) for f in fl):
            id_bad.append("reply has no request id row")
        if not any(re.search(r"echo", f["meaning"], re.I) and re.search(r"size", f["meaning"], re.I)
                   and REQ_INTENDED and f["bits"] == REQ_INTENDED[0]["bits"] for f in fl):
            id_bad.append("reply has no echoed size row at the request's intended packet size width")
check("AC1", bool(REQ_FL) and bool(REP_FL) and not hdr_bad,
      "each probe message starts with the seven common control header rows in the hello's order, offset, width and type"
      + (f" (failing: {hdr_bad})" if hdr_bad else ""))
check("AC1", bool(REQ_FL) and bool(REP_FL) and not tile_bad,
      "each probe message's rows tile from byte 0 with no gap and no overlap" + (f": {tile_bad[:3]}" if tile_bad else ""))
check("AC1", bool(REQ_FL) and bool(REP_FL) and not size_bad,
      "each probe message states a maximum size equal to the sum of its rows at their maximum count"
      + (f": {size_bad}" if size_bad else ""))
_rsz = re.search(r"maximum\s+size[^.]*?(\d+)\s*bytes", rep_text, re.I)
check("AC1", bool(_rsz) and CTRL_BOUND is not None and int(_rsz.group(1)) <= CTRL_BOUND,
      f"the probe reply states a size of at most the {CTRL_BOUND}-byte control bound (C-17)")
check("AC1", bool(REQ_FL) and bool(REP_FL) and not code_bad and len(set(codes_seen.values())) == 2,
      "each probe message's Message type is a code inside the block the hello gives the probe pair, and the two differ"
      + (f": {code_bad}" if code_bad else ""))
check("AC1", bool(REQ_FL) and bool(REP_FL) and not pad_bad and not any(f["maxc"] for f in REP_FL),
      "the probe request has a zero-valued padding row; the probe reply has no padding row and its rows are of fixed size "
      "(no counted row)" + (f": {pad_bad}" if pad_bad else ""))
check("AC1", bool(REQ_FL) and bool(REP_FL) and not id_bad,
      "the request carries the intended packet size and an identifier of the probe; the reply carries the request id "
      "and echoes the intended packet size"
      + (f": {id_bad}" if id_bad else ""))

MATCH_TEXT = [x for t, x in PSUBS if re.search(r"match|binding|identif", t, re.I)]
check("AC1", bool(MATCH_TEXT) and any(re.search(r"reply", x, re.I) and re.search(r"request\s+id|probe\s+identifier|intended\s+packet\s+size", x, re.I)
                                       for x in MATCH_TEXT),
      "a matching subsection says how a reply identifies its probe, naming the reply and an identifier of the probe "
      "(request id, probe identifier or intended packet size)")

PARAS = prose_blocks(PROBE)
check("AC1", any(k == "p" and re.search(r"intended", t, re.I) and re.search(r"probe", t, re.I)
                 and re.search(r"packet\s+size", t, re.I) for k, t in PARAS),
      "a prose paragraph states the probe size rule: the probe is sent at the intended media packet size")
check("AC1", bool(re.search(r"don\W{0,2}t[\s-]+fragment|\bDF\b", PROBE, re.I)) and bool(re.search(r"rule\s+11", PROBE, re.I))
      and bool(re.search(r"fragmentation", PROBE, re.I)),
      "the DF requirement is stated: every probe is sent with don't-fragment set and fragmentation is forbidden (rule 11)")
check("AC1", "IPv4" in PROBE and "C-100" in PROBE and bool(re.search(r"CS5\s*\(40\)", PROBE)) and "C-13" in PROBE,
      "the probe pair is IPv4 only (C-100) and travels at CS5 (40) (C-13)")
_dec = [t for k, t in PARAS if k == "p" and re.search(r"small\s+acknowledg", t, re.I)
        and re.search(r"only\s+the\s+probe\s+is\s+padded|the\s+probe\s+alone\s+is\s+padded", t, re.I)
        and re.search(r"exempt", t, re.I) and "[decision]" in t]
check("AC1", bool(_dec), "a [decision] paragraph states the owner's ruling: the reply is a small acknowledgement, and only "
      "the probe is padded and exempt from C-17")
check("AC1", bool(rep_text) and not re.search(r"\bis\s+padded\b|as\s+long\s+as\s+the\s+probe", rep_text, re.I),
      "the probe reply subsection does not say the reply is padded or as long as the probe")
check("AC1", bool(re.search(r"no\s+reply|does\s+not\s+arrive|no\s+answer|no\s+response|not\s+answered|never\s+arrives|silence", PROBE, re.I)),
      "the outcome when no reply arrives is stated")
p_int = re.search(r"intended", PROBE, re.I)
p_fb = re.search(r"\b%d\b" % (FALLBACK_MTU or 0), PROBE)
p_ref = re.search(r"PathMtuTooSmall", PROBE)
check("AC1", bool(p_int and p_fb and p_ref) and p_int.start() < p_fb.start() < p_ref.start(),
      f"the fallback sequence is stated in order: the intended size, then the {FALLBACK_MTU}-byte fallback, "
      "then the refusal PathMtuTooSmall (C-173)")
check("AC1", bool(re.search(r"\b%d\b" % (CTRL_BOUND or 0), PROBE)) and "C-06" in PROBE and "C-173" in PROBE,
      f"the section cites C-06 and C-173 and states the {CTRL_BOUND}-byte UDP payload of the fallback")

# AC 2: the exemption
PROBE_TOML = [m for m in TMSGS if "probe" in str(m.get("name", "")).lower()
              and not re.search(r"one[\s_-]*way", str(m.get("name", "")), re.I)]
EXEMPT_ALL = [m.get("name") for m in TMSGS if "exempt_from_control_bound" in m]
REQ_TOML = [m for m in PROBE_TOML if "request" in str(m.get("name", "")).lower()]
REP_TOML = [m for m in PROBE_TOML if "reply" in str(m.get("name", "")).lower()]
check("AC2", len(REQ_TOML) == 1 and EXEMPT_ALL == [REQ_TOML[0].get("name")],
      f"exactly one message of wire.toml carries the exemption from C-17, the probe request (exempt messages {EXEMPT_ALL})")
check("AC2", len(REQ_TOML) == 1 and REQ_TOML[0].get("exempt_from_control_bound") == "C-116"
      and len(REP_TOML) == 1 and "exempt_from_control_bound" not in REP_TOML[0],
      "the probe request's exemption cites C-116, and the probe reply carries no exemption")
_sents = re.split(r"(?<=[.;])\s+", re.sub(r"```.*?```", "", PROBE, flags=re.S))
_ex = [s for s in _sents if re.search(r"exempt", s, re.I)]
check("AC2", PROBE_SPAN is not None and "to be filled" not in PROBE.lower() and bool(_ex) and all("C-116" in s and set(re.findall(r"C-\d+", s)) <= {"C-116", "C-17"} for s in _ex),
      "every sentence of the section that says exempt cites C-116 and no other exemption")

# AC 3: invalid forms with outcomes
VAL_TEXTS = [x for t, x in PSUBS if re.search(r"validity", t, re.I)]
VROWS = []
for _v in VAL_TEXTS:
    for _s, _hd, _rows in parse_tables(_v):
        if len(_hd) >= 2 and re.search(r"form|invalid", _hd[0], re.I):
            for _r in _rows:
                if len(_r) >= 2 and _r[0].strip():
                    VROWS.append((_r[0].strip(), _r[1].strip()))
check("AC3", len(VROWS) >= 4 and all(o for _f, o in VROWS),
      f"the probe's validity rules give at least 4 invalid forms, each with an outcome (found {len(VROWS)})")


def tokens(s):
    return re.findall(r"\b[A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)+\b", s)


def ctrl_reason(outcome):
    return [t for t in tokens(outcome) if t in MEMBERS and "control" in MEMBER_LAYERS[t]]


bad_names, bad_layer = [], []
for _f, outc in VROWS:
    for t in tokens(outc):
        if t in MEMBERS:
            if "control" not in MEMBER_LAYERS[t]:
                bad_layer.append(f"{t} (Layers: {MEMBER_LAYERS[t]})")
        elif t not in REF:
            bad_names.append(t)
check("AC3", bool(VROWS) and not bad_names, "every outcome names a member of the Rejection reasons or the refusal enumeration"
      + (f"; not members: {sorted(set(bad_names))}" if bad_names else ""))
check("AC3", bool(VROWS) and not bad_layer, "every packet reason named by an outcome lists the control layer in its Layers cell"
      + (f"; {bad_layer}" if bad_layer else ""))

SIZE_ROWS = [(f, o) for f, o in VROWS if re.search(r"intended", f, re.I) and re.search(r"size", f, re.I)
             and not re.search(r"pad", f, re.I)]
PAD_ROWS = [(f, o) for f, o in VROWS if re.search(r"pad", f, re.I)
            and re.search(r"non-?\s?zero|not\s+zero|other\s+than\s+zero|\bnonzero\b", f, re.I)]
REPLY_ROWS = [(f, o) for f, o in VROWS if re.search(r"reply", f, re.I)
              and re.search(r"outstanding|match|no\s+probe|unmatched", f, re.I)]
GENERIC_ROWS = [(f, o) for f, o in VROWS if re.search(
    r"magic|message\s+type|domain|scheme|\btag\b|oversize|too\s+short|version|counter|datagram|unassigned", f, re.I)
    and (f, o) not in SIZE_ROWS and (f, o) not in PAD_ROWS and (f, o) not in REPLY_ROWS]
check("AC3", bool(SIZE_ROWS) and all(ctrl_reason(o) for _f, o in SIZE_ROWS),
      "a probe size different from its declared intended size has an outcome naming a control-layer reason")
check("AC3", bool(PAD_ROWS) and all("ReservedNonZero" in o for _f, o in PAD_ROWS),
      "non-zero padding is dropped and counted under ReservedNonZero")
check("AC3", bool(REPLY_ROWS) and all(
    ctrl_reason(o) or (re.search(r"not a parse rejection|not a rejection", o, re.I) and re.search(r"\d{3}-\d{2}|epic", o, re.I))
    for _f, o in REPLY_ROWS),
      "a reply that matches no outstanding probe has an outcome: a control-layer reason, or not a rejection with its owner")
_len_row = any(re.search(r"reply", f, re.I) and re.search(r"size|length|datagram|bytes", f, re.I) and "BadLength" in o
               and re.search(r"other\s+than|differs|not\s+the|does\s+not", f, re.I) for f, o in VROWS)
_len_sent = any(re.search(r"reply", x, re.I) and "BadLength" in x and re.search(r"fixed\s+size|length", x, re.I)
                and re.search(r"shared|Validity\s+rules|Hello|Admission\s+messages|Rejection\s+reasons", x)
                for x in re.split(r"(?<=[.;])\s+|\n", PROBE))
check("AC3", _len_row or _len_sent, "a reply whose length differs from its fixed size is dropped and counted under BadLength, "
      "by a row or by the shared rule the section cites")
check("AC3", bool(GENERIC_ROWS) and all(ctrl_reason(o) for _f, o in GENERIC_ROWS),
      "the generic control violations (magic, message type, domain, scheme or tag, oversize, too short) have control-layer outcomes")

# DoD: evidence tags, owners and falsifiers
UNTAGGED = []
for kind, t in PARAS:
    if kind == "p" and len(re.sub(r"[`*_|>#]", " ", t).split()) >= 5 and not has_evidence(t):
        UNTAGGED.append(clean_start(t)[:60])
check("DoD", PROBE_SPAN is not None and not UNTAGGED,
      "every prose paragraph of 5 or more words in the probe section carries an evidence tag"
      + (f"; untagged: {UNTAGGED[:3]}" if UNTAGGED else ""))
UNTAGGED_SUBS = [t for t, x in PSUBS if not has_evidence(x)]
check("DoD", bool(PSUBS) and not UNTAGGED_SUBS,
      "every subsection of the probe section carries an evidence tag" + (f"; untagged: {UNTAGGED_SUBS}" if UNTAGGED_SUBS else ""))
OPEN = [clean_start(t) for k, t in PARAS if k == "p" and re.match(r"^\W*open\s+item", clean_start(t), re.I)]
BAD_OPEN = [t[:50] for t in OPEN
            if not (re.search(r"owner", t, re.I) and re.search(r"\d{3}-\d{2}|\bepic\b", t, re.I)
                    and re.search(r"falsifier", t, re.I))]
check("DoD", not BAD_OPEN, f"every open item of the probe section names an owner and a falsifier ({len(OPEN)} found)"
      + (f"; without: {BAD_OPEN}" if BAD_OPEN else ""))

# AC 4: the toml probe messages and their agreement with the markdown tables
TOML_OK = len(PROBE_TOML) >= 2
for kind, fl, _text in MSG_FL:
    tm = next((m for m in PROBE_TOML if kind in str(m.get("name", "")).lower()), None)
    if tm is None:
        TOML_OK = False
        fail("AC4", f"wire.toml has no probe {kind} message named with '{kind}'")
        continue
    check("AC4", tm.get("layer") == "control" and tm.get("dscp") == DSCP_CONTROL,
          f"toml '{tm.get('name')}' is layer control at dscp {DSCP_CONTROL} (C-13)")
    ms = tm.get("max_size")
    if kind == "request":
        check("AC4", isinstance(ms, int) and CTRL_BOUND is not None and ms > CTRL_BOUND,
              f"toml '{tm.get('name')}' declares a max_size above the {CTRL_BOUND}-byte bound, so its exemption is not vacuous")
    else:
        check("AC4", isinstance(ms, int) and CTRL_BOUND is not None and ms <= CTRL_BOUND,
              f"toml '{tm.get('name')}' declares a max_size at most the {CTRL_BOUND}-byte bound: it is not exempt")
        check("AC4", not any("max_count" in f or "count" in f or f.get("type") == "pad"
                             for f in tm.get("field", []) if isinstance(f, dict)),
              f"toml '{tm.get('name')}' has no padding and no counted field")
    tfields = [f for f in tm.get("field", []) if isinstance(f, dict)]
    cur, probs = 0, []
    for f in tfields:
        mult = int(f.get("max_count", 1)) if "max_count" in f else 1
        if int(f.get("offset", -1)) != cur:
            probs.append(f"'{f.get('name')}' at bit {f.get('offset')}, expected {cur}")
        cur = int(f.get("offset", 0)) + int(f.get("width", 0)) * mult
    check("AC4", not probs and isinstance(ms, int) and cur == ms * 8,
          f"toml '{tm.get('name')}' fields tile from bit 0 to max_size ({cur} bits against {ms} bytes)"
          + (f": {probs[:3]}" if probs else ""))
    unknown = [r for r in tm.get("rejection_reasons", []) if r not in MEMBERS]
    check("AC4", not unknown, f"toml '{tm.get('name')}' rejection_reasons are members of the closed Reject enumeration"
          + (f"; not members: {unknown}" if unknown else ""))
    md_sig = [(f["off"] * 8, size_bits(f)) for f in fl]
    toml_sig = [(int(f.get("offset", -1)), int(f.get("width", 0)) * (int(f.get("max_count", 1)) if "max_count" in f else 1))
                for f in tfields]
    check("AC4", bool(fl) and md_sig == toml_sig,
          f"the {kind} rows of wire.md and of toml '{tm.get('name')}' agree row for row in offset and size")
    tt = next((f for f in tfields if re.search(r"message[_\s]type", f"{f.get('name', '')} {f.get('meaning', '')}", re.I)), None)
    tcode = first_int(str(tt.get("valid", ""))) if tt else None
    check("AC1", PROBE_BLOCK is not None and tcode is not None and tcode in PROBE_BLOCK,
          f"toml '{tm.get('name')}' carries a message type inside the probe block")
if not TOML_OK:
    fail("AC4", "wire.toml does not declare one probe request and one probe reply")

# AC 4 (repository level): the wire-check binary and the wire_tables tests
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        check("AC4", proc.returncode == 0 and "checks clean" in out,
              "cargo run --bin wire-check reports checks clean"
              + ("" if proc.returncode == 0 and "checks clean" in out else ": " + " | ".join(out.strip().splitlines()[-4:])))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC4", f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result[-1]) if result else None
        check("AC4", proc.returncode == 0 and bool(mm) and int(mm.group(2)) == 0 and int(mm.group(1)) >= 12,
              "cargo test -p hadronyche-tools --test wire_tables has 0 failed and at least the 12 passed that stood before"
              + (f": {result[-1]}" if result else f": no result line ({out.strip()[-200:]})"))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC4", f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC4: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

fails = [ac for ac, ok in RESULTS if not ok]
print(f"{len(RESULTS) - len(fails)} passed, {len(fails)} failed")
sys.exit(1 if fails else 0)
PY
pyrc=$?
if [ "$pyrc" -ne 0 ]; then
    rc=1
fi
exit "$rc"
