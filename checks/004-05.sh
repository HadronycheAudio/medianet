#!/usr/bin/env bash
# 004-05 — reserved fields and their required values.
#
# RED-phase document check. It validates doc/design/spec/wire.md and
# doc/design/spec/wire.toml against every acceptance criterion of the ticket,
# and re-runs the 004-02 checker plus the wire_tables tests.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo, the design checker, the
# earlier document checks) are skipped with a NOTE, because they always
# target the real repository.
#
# What is checked, per acceptance criterion:
#
#   AC 1  The "Reserved fields" section states, for the copy index (2 bits,
#         C-07) and the authentication reservation (8 + 64 + 128 bits, C-12),
#         the required v1 value and the receiver outcome. For the copy index:
#         senders emit 0 and the receiver rule matches the decision below. For
#         the reservation: scheme is 0 and tag is 128 zero bits, each with the
#         ReservedNonZero outcome (drop and count); the 64-bit counter is
#         stated live. A non-zero reserved value names ReservedNonZero, is
#         dropped and is counted.
#   AC 2  The section cites C-115 and states that the 64-bit counter is live
#         because it is the liveness counter, while scheme and tag are zero.
#   AC 3  A general rule covers padding and reserved bits added by other
#         tickets: the sender zeroes them, and a receiver that sees a non-zero
#         value has a stated outcome naming ReservedNonZero (drop and count).
#   AC 4  wire.toml carries the reserved values in any compatible extension
#         (a per-field `reserved`/`reserved_value` attribute, a `[[reserved]]`
#         array, or a keyed table), agreeing with wire.md; `cargo run --bin
#         wire-check` exits 0 and the wire_tables tests report 12/0/0.
#
# The copy-index decision (ticket Work step 2) must be explicit and tagged
# [decision]: v1 senders emit only 0, and receivers either accept all four
# values (0..3, allowing duplication) or accept only 0. The dedup rule must be
# restated: by stream, position and packet index, the first valid copy wins,
# and there is no separate sequence number. The falsifier must be considered:
# the section states what happens when duplication arrives, consistently with
# the chosen decision (a major version if v1 receivers reject a needed value,
# no major version if v1 receivers already accept all four).
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


def fail(label, msg):
    fails.append((label, msg))


def warn(label, msg):
    warns.append((label, msg))


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


# ---------------------------------------------------------------------------
# Markdown helpers (same conventions as checks/004-01.sh and checks/004-03.sh)
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


def section_units(text):
    """Paragraph-like units (bullets join) plus one unit per table row."""
    units = []
    in_fence = False
    cur = []
    for line in text.splitlines():
        if re.match(r"^\s*```", line):
            in_fence = not in_fence
            if cur:
                units.append("\n".join(cur))
                cur = []
            continue
        if in_fence:
            continue
        if line.strip().startswith("|"):
            if cur:
                units.append("\n".join(cur))
                cur = []
            units.append(line.strip())
        elif line.strip():
            cur.append(line)
        elif cur:
            units.append("\n".join(cur))
            cur = []
    if cur:
        units.append("\n".join(cur))
    return units


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(block):
    return bool(EVIDENCE.search(block))


def norm(text):
    return re.sub(r"\s+", " ", str(text).replace("_", " ")).strip().lower()


# ---------------------------------------------------------------------------
# A small TOML reader (the same subset as checks/004-03.sh: Python 3.10 has no
# tomllib, and the check runs unprivileged and offline).
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
                    "n": "\n",
                    "t": "\t",
                    "r": "\r",
                    "b": "\b",
                    "f": "\f",
                    '"': '"',
                    "\\": "\\",
                    "/": "/",
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
                raise TomlError("[[%s]] is not an array of tables" % ".".join(path))
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
            raise TomlError("not a key/value statement: %r" % stmt[:60])
        key = stmt[:eq].strip()
        parser = _ValueParser(stmt[eq + 1 :])
        value = parser.parse()
        parser.skip_ws()
        if parser.i != len(parser.t):
            raise TomlError("trailing text after value: %r" % parser.t[parser.i :])
        parts = _split_dotted(key)
        target = current
        for p in parts[:-1]:
            target = target.setdefault(p, {})
        target[parts[-1]] = value
    return root


# ---------------------------------------------------------------------------
# Find the Reserved fields section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    print("FAIL AC1: %s does not exist" % wire_path)
    for label in ("AC2", "AC3", "AC4", "DEC", "DEDUP", "FALSIF"):
        print("FAIL %s: cannot check — %s does not exist" % (label, wire_path))
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.search(r"^reserved\s+fields?$", title.strip(), re.I):
        sec_i = i
        break
if sec_i is None:
    for i, (_, _, title) in enumerate(headings):
        if re.search(r"reserved[-\s]+fields?", title, re.I):
            sec_i = i
            break

if sec_i is None:
    fail(1, "no 'Reserved fields' section heading in %s" % wire_path)
    section = ""
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
units = section_units(section_prose)

PLACEHOLDER = re.compile(r"to\s+be\s+filled\s+by\s*004-05", re.I)
placeholder = sec_i is None or bool(PLACEHOLDER.search(section)) or not section.strip()


def units_matching(pattern):
    return [u for u in units if re.search(pattern, u, re.I)]


def any_unit(units_list, *patterns):
    """True when one unit satisfies every pattern (locality matters)."""
    for u in units_list:
        if all(re.search(p, u, re.I) for p in patterns):
            return True
    return False


def table_has(field_pattern, value_pattern):
    """A table row about the field whose value column matches value_pattern."""
    for _, header, rows in parse_tables(section):
        hnorm = [norm(h) for h in header]
        value_cols = [
            k
            for k, h in enumerate(hnorm)
            if re.search(r"required|value|outcome|receiver|behavio|live", h)
        ]
        if not value_cols:
            continue
        for row in rows:
            field_text = " ".join(
                c for k, c in enumerate(row) if k not in value_cols
            )
            if not re.search(field_pattern, field_text, re.I):
                continue
            for k in value_cols:
                if k < len(row) and re.search(value_pattern, row[k], re.I):
                    return True
    return False


COPY_FIELD = r"copy[\s_-]*index|\bC-07\b"
COPY_WIDTH = r"\b(?:2|two)[-\s]?bits?\b"
SENDER_ZERO = r"(?:senders?|transmit\w*|emit\w*|send\w*)\b[^.\n]{0,140}?\b(?:only\s+)?(?:0|zero)\b"
REQUIRED_ZERO = r"\b(?:required(?:\s+v1)?\s+value|value\s+in\s+v1|v1\s+value)\b[^.\n|]{0,60}?\b0\b"
COPY_IS_ZERO = (
    r"(?:copy[\s_-]*index|\bC-07\b)[^.\n]{0,80}?"
    r"(?:is|stays|remains|set\s+to|=)\s*(?:only\s+)?(?:0|zero)\b"
)
COPY_WRITTEN_ZERO = (
    r"\b(?:written|encoded|sent|transmitted|emitted|set|fixed)\s+"
    r"(?:as|to|at)\s+(?:only\s+)?(?:0|zero)\b"
)
ZERO_CELL = r"^\s*(?:0|zero)\b"

ACCEPT_ALL = (
    r"(?:receivers?|consumers?)\b[^.\n]{0,100}?\baccept\w*\b[^.\n]{0,80}?"
    r"(?:\ball\s+four\b|\bfour\s+values?\b|0\s*(?:to|\.\.|–|—|-)\s*3)"
    r"|\ball\s+four\s+(?:values?|copy[\s_-]*index\s+values?)"
    r"|\baccept\w*\b[^.\n]{0,40}?0\s*(?:to|\.\.|–|—|-)\s*3\b"
    r"|\baccept\w*\b[^.\n]{0,90}?\b0\s*,\s*1\s*,\s*2\b"
    r"|\baccept\w*\b[^.\n]{0,90}?\b(?:every|each)\s+(?:of\s+the\s+four|value)\b"
)
ONLY_ZERO = (
    r"(?:receivers?|consumers?)\b[^.\n]{0,100}?\baccept\w*\b[^.\n]{0,60}?"
    r"\bonly\s+(?:the\s+value\s+)?(?:0|zero)\b"
    r"|\bonly\s+(?:the\s+value\s+)?(?:0|zero)\b[^.\n]{0,80}?\baccept"
    r"|\breject\w*\b[^.\n]{0,60}?\bnon[\s-]?zero\s+copy"
)
DROP = r"\b(?:drop\w*|discard\w*|reject\w*)\b"
COUNT = r"\b(?:count\w*|increment\w*)\b"

copy_decision = None
copy_context = ""
scheme_zero = tag_zero = counter_live = counter_liveness = False

if not placeholder:
    # ------------------------------------------------------------------
    # AC 1 — copy index
    # ------------------------------------------------------------------
    copy_units = units_matching(COPY_FIELD)
    copy_context = "\n".join(copy_units)

    if not copy_units:
        fail(1, "the section does not name the copy index (C-07)")
    else:
        if not re.search(COPY_WIDTH, copy_context, re.I):
            fail(1, "the copy index is not stated as 2 bits (C-07)")
        if not (
            any_unit(copy_units, SENDER_ZERO)
            or any_unit(copy_units, REQUIRED_ZERO)
            or any_unit(copy_units, COPY_IS_ZERO)
            or any_unit(copy_units, COPY_WRITTEN_ZERO)
            or table_has(COPY_FIELD, ZERO_CELL)
        ):
            fail(
                1,
                "the copy index has no stated required v1 value: say that v1 "
                "senders emit only 0 (or give 0 in a 'required value in v1' "
                "column)",
            )

        accept_all = any_unit(copy_units, ACCEPT_ALL)
        only_zero = any_unit(copy_units, ONLY_ZERO)
        if accept_all and only_zero:
            fail(
                1,
                "the copy-index receiver rule is ambiguous: the section states "
                "both that receivers accept all four values and that they "
                "accept only 0; choose exactly one and tag it [decision]",
            )
        elif accept_all:
            copy_decision = "accept_all"
        elif only_zero:
            copy_decision = "only_zero"
        else:
            fail(
                1,
                "the copy-index decision is absent: state whether v1 receivers "
                "accept all four values (0..3, for duplication) or only 0, and "
                "tag it [decision]",
            )

        if copy_decision == "only_zero":
            outcome = any_unit(
                copy_units,
                COPY_FIELD,
                r"\bReservedNonZero\b",
                DROP,
                COUNT,
            )
            if not outcome:
                fail(
                    1,
                    "copy index: a non-zero value has no receiver outcome; with "
                    "the only-0 decision it must be ReservedNonZero, dropped and "
                    "counted",
                )
        # With the accept-all decision no copy-index value is non-conforming;
        # the explicit acceptance statement is the receiver rule (checked by
        # the decision above).

        # ------------------------------------------------------------------
        # DEC — the decision must be tagged [decision]
        # ------------------------------------------------------------------
        decision_indexes = [
            k
            for k, u in enumerate(copy_units)
            if re.search(ACCEPT_ALL, u, re.I) or re.search(ONLY_ZERO, u, re.I)
        ]
        tagged = any(
            "[decision]" in copy_units[k]
            or (k > 0 and "[decision]" in copy_units[k - 1])
            for k in decision_indexes
        )
        if not tagged:
            fail(
                "DEC",
                "the copy-index decision is not tagged [decision] (Work step 2: "
                "decide and tag the decision)",
            )

        # ------------------------------------------------------------------
        # DEDUP — the dedup rule is restated
        # ------------------------------------------------------------------
        dedup_facts = [
            (r"\bby\s+stream\b", "by stream"),
            (r"\bposition\b", "position"),
            (r"\bpacket[\s-]+index\b", "packet index"),
            (r"\bfirst\s+valid\s+copy\b", "the first valid copy wins"),
            (
                r"\bno\s+(?:separate\s+)?sequence\s+number\b|"
                r"\bwithout\s+a\s+(?:separate\s+)?sequence\s+number\b",
                "no separate sequence number",
            ),
        ]
        missing = [label for pat, label in dedup_facts if not re.search(pat, section, re.I)]
        if missing:
            fail(
                "DEDUP",
                "the copy-index dedup rule is not fully restated: missing %s "
                "(rule: by stream, position and packet index; the first valid "
                "copy wins; there is no separate sequence number)" % ", ".join(missing),
            )

        # ------------------------------------------------------------------
        # FALSIF — what happens when duplication arrives
        # ------------------------------------------------------------------
        dup_units = units_matching(r"\bduplicat")
        if not dup_units:
            fail(
                "FALSIF",
                "the section does not consider the deferred duplication feature "
                "(the falsifier: a duplication value v1 receivers reject would "
                "force a major version); state what happens when duplication "
                "arrives",
            )
        else:
            if copy_decision == "only_zero":
                ok = any_unit(dup_units, r"\bmajor\s+version\b")
                want = "a major version (v1 receivers reject a non-zero copy index)"
            elif copy_decision == "accept_all":
                ok = any_unit(
                    dup_units,
                    r"\bminor\s+version\b|"
                    r"\bno\s+(?:new\s+)?major\s+version\b|"
                    r"\bwithout\s+a\s+major\s+version\b|"
                    r"\b(?:does|do)\s+not\s+(?:need|require)\s+a\s+major\b|"
                    r"\bno\s+(?:new\s+)?wire\s+(?:value|change)\b|"
                    r"\bexisting\s+values?\b|"
                    r"\balready\s+accept",
                )
                want = (
                    "no major version (v1 receivers already accept all four "
                    "values, so duplication arrives without a new wire value)"
                )
            else:
                ok = False
                want = "a version consequence consistent with the copy-index decision"
            if not ok:
                fail(
                    "FALSIF",
                    "the duplication statement does not say what happens to the "
                    "version when duplication arrives; expected %s" % want,
                )

    # ------------------------------------------------------------------
    # AC 1 — authentication reservation
    # ------------------------------------------------------------------
    auth_units = units_matching(
        r"\bauthentication\b|\bauth\b|\bscheme\b|\bcounter\b|\btag\b|\bC-12\b"
    )
    auth_context = "\n".join(auth_units)

    if not auth_units:
        fail(1, "the section does not describe the authentication reservation (C-12)")
    else:
        if not re.search(r"\bC-12\b", auth_context):
            fail(1, "the authentication reservation does not cite C-12")

        scheme_units = units_matching(r"\bscheme\b")
        counter_units = units_matching(r"\bcounter\b")
        tag_units = units_matching(r"\btag\b")

        # The widths may be given per field or as the ordered triple "8 + 64 +
        # 128 bits" of C-12.
        triple = any_unit(auth_units, r"\b8\b", r"\b64\b", r"\b128\b")

        if not (
            any_unit(scheme_units, r"\b8[-\s]?bits?\b")
            or table_has(r"\bscheme\b", r"\b8\b")
            or triple
        ):
            fail(1, "the authentication scheme is not stated as 8 bits (C-12)")
        if not (
            any_unit(counter_units, r"\b64[-\s]?bits?\b")
            or table_has(r"\bcounter\b", r"\b64\b")
            or triple
        ):
            fail(1, "the authentication counter is not stated as 64 bits (C-12)")
        if not (
            any_unit(tag_units, r"\b128[-\s]?bits?\b")
            or table_has(r"\btag\b", r"\b128\b")
            or triple
        ):
            fail(1, "the authentication tag is not stated as 128 bits (C-12)")

        scheme_zero = (
            any_unit(scheme_units, r"\bscheme\b[^.\n|]{0,60}?\b(?:0|zero)\b")
            or table_has(r"\bscheme\b", ZERO_CELL)
        )
        if not scheme_zero:
            fail(
                1,
                "the authentication scheme has no required v1 value: v1 senders "
                "set scheme = 0 (none)",
            )

        tag_zero = (
            any_unit(tag_units, r"\btag\b[^.\n|]{0,80}?\b(?:0|zero)\b")
            or any_unit(tag_units, r"\b128\b[^.\n|]{0,40}?\b(?:zero|0)\b")
            or table_has(r"\btag\b", r"^\s*(?:0|zero|all[-\s]?zero)")
        )
        if not tag_zero:
            fail(
                1,
                "the authentication tag has no required v1 value: the tag is 128 "
                "zero bits",
            )

        scheme_outcome = any_unit(
            scheme_units, r"\bscheme\b", r"\bReservedNonZero\b", DROP, COUNT
        )
        if not scheme_outcome:
            fail(
                1,
                "authentication scheme: a non-zero scheme has no receiver "
                "outcome; it must name ReservedNonZero, drop and count",
            )
        tag_outcome = any_unit(
            tag_units, r"\btag\b", r"\bReservedNonZero\b", DROP, COUNT
        )
        if not tag_outcome:
            fail(
                1,
                "authentication tag: a non-zero tag with scheme = 0 has no "
                "receiver outcome; it must name ReservedNonZero, drop and count",
            )

        counter_live = any_unit(
            counter_units,
            r"\bcounter\b",
            r"\blive\b|\bliveness\b",
        )
        if not counter_live:
            fail(
                1,
                "the 64-bit message counter has no stated required v1 value: it "
                "is live in v1, not a zero field (C-115)",
            )
        counter_liveness = any_unit(
            counter_units,
            r"\bcounter\b",
            r"\bliveness\b|\bincrement\w*\b|\bmonotonic\w*\b",
        )
        if not counter_liveness:
            fail(
                1,
                "the 64-bit counter is not stated as the liveness counter (C-115)",
            )
        if counter_units and not any_unit(
            counter_units,
            r"\b(?:any\s+value|not\s+(?:a\s+)?reserved(?:\s+zero)?\s+field|"
            r"not\s+(?:fixed|required|checked)\s+(?:at\s+)?zero|not\s+rejected)\b",
        ):
            warn(
                1,
                "the counter is not explicitly excluded from the zero rule; say "
                "that it is not a reserved zero field (C-115)",
            )

    # ------------------------------------------------------------------
    # AC 2 — C-115: counter live, scheme and tag zero
    # ------------------------------------------------------------------
    if not re.search(r"\bC-115\b", section):
        fail(
            2,
            "the section does not cite C-115 for the v1 authentication "
            "reservation (scheme 0, tag zero, counter live)",
        )
    if not (counter_live and counter_liveness):
        fail(
            2,
            "the section does not state that the 64-bit counter is live in v1 "
            "because it is the liveness counter (C-115)",
        )
    if not scheme_zero:
        fail(2, "C-115: the section does not state that the v1 scheme is 0")
    if not tag_zero:
        fail(2, "C-115: the section does not state that the v1 tag is zero")

    # ------------------------------------------------------------------
    # AC 3 — general padding / reserved-bit rule
    # ------------------------------------------------------------------
    general_ok = False
    general_best = None
    for u in units:
        if not re.search(r"\bpadding\b|\bpad(?:ding)?\b|\bpad\b", u, re.I):
            continue
        if not re.search(r"\breserved\b", u, re.I):
            continue
        if not re.search(r"\b(?:0|zero)\b", u, re.I):
            continue
        if not re.search(r"\bsender|writer|transmit|emit|set\b", u, re.I):
            continue
        if not re.search(r"\bReservedNonZero\b", u):
            continue
        if not (re.search(DROP, u, re.I) and re.search(COUNT, u, re.I)):
            continue
        general_ok = True
        general_best = u
        break
    if not general_ok:
        fail(
            3,
            "no general rule for padding and reserved bits added by other "
            "tickets: state that the sender zeroes them and that a non-zero "
            "value is dropped and counted as ReservedNonZero",
        )

    # ------------------------------------------------------------------
    # DoD — evidence tags on the new section
    # ------------------------------------------------------------------
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
        is_table = all(
            line.strip().startswith("|") for line in content_lines if line.strip()
        )
        is_code = content_lines[0].lstrip().startswith("```")
        if has_evidence(body):
            prev_tagged = True
            continue
        if (is_table or is_code) and prev_tagged:
            prev_tagged = True
            continue
        fail(
            "DoD",
            "this block of the Reserved fields section carries no evidence tag "
            "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
            + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
        )
        prev_tagged = False
else:
    for label, name in (
        ("AC1", "copy index and authentication reservation values and outcomes"),
        ("AC2", "C-115 liveness"),
        ("AC3", "general padding/reserved-bit rule"),
        ("DEC", "copy-index decision"),
        ("DEDUP", "dedup rule"),
        ("FALSIF", "duplication falsifier"),
    ):
        fail(
            label,
            "the 'Reserved fields' section is still the 004-05 placeholder: %s "
            "is not stated" % name,
        )

# ---------------------------------------------------------------------------
# AC 4 — wire.toml carries the reserved values; the 004-02 checker passes
# ---------------------------------------------------------------------------
ac4 = True
tables_text = read_text(tables_path)
records = []
document = None
if tables_text is None:
    fail(4, "%s does not exist" % tables_path)
    ac4 = False
else:
    try:
        document = parse_toml(tables_text)
    except TomlError as error:
        fail(4, "%s could not be read by this check: %s" % (tables_path, error))
        ac4 = False


def collect_records(root):
    """One record per table-ish node, with its scalar keys and values."""
    out = []

    def visit(node, path):
        if isinstance(node, dict):
            ctx = list(path)
            scalars = []
            for k, v in node.items():
                if isinstance(v, bool):
                    scalars.append((str(k), "bool", v))
                    ctx.append(str(k))
                elif isinstance(v, int):
                    scalars.append((str(k), "int", v))
                    ctx.append(str(k))
                    ctx.append(str(v))
                elif isinstance(v, str):
                    scalars.append((str(k), "str", v))
                    ctx.append(str(k))
                    ctx.append(v)
                else:
                    ctx.append(str(k))
            out.append({"ctx": norm(" ".join(ctx)), "scalars": scalars})
            for k, v in node.items():
                if isinstance(v, (dict, list)):
                    visit(v, path + (str(k),))
        elif isinstance(node, list):
            for v in node:
                visit(v, path)

    visit(root, ())
    return out


NAMEISH = re.compile(r"^(?:name|field|field name|message|location|section|attribute|id)$")
WIDTHISH = re.compile(r"width|bits?$|size")
OTHERISH = re.compile(
    r"^(?:outcome|receiver|behaviour|behavior|meaning|note|comment|description|"
    r"type|layer|dscp|schema|offset|offset unit|width unit|size unit)$"
)


def atoms_for(records, pattern):
    """Per-record values that belong to one field, scoped by key when possible."""
    atoms = []
    for rec in records:
        if not re.search(pattern, rec["ctx"], re.I):
            continue
        own = [
            (norm(k), kind, v)
            for k, kind, v in rec["scalars"]
            if re.search(pattern, norm(k), re.I)
        ]
        if own:
            values = own
        else:
            values = []
            for k, kind, v in rec["scalars"]:
                kn = norm(k)
                if NAMEISH.fullmatch(kn) or WIDTHISH.search(kn) or OTHERISH.fullmatch(kn):
                    continue
                values.append((kn, kind, v))
        atoms.append({"values": values, "ctx": rec["ctx"], "scalars": rec["scalars"]})
    return atoms


def atom_zero(atom):
    for _, kind, v in atom["values"]:
        if kind == "int" and v == 0:
            return True
        if kind == "str":
            s = norm(v)
            if re.match(r"^0\s*(?:to|\.\.|–|—|-)\s*\d", s):
                continue
            if re.match(r"^(?:0\b|zero\b|all[-\s]?zero)", s):
                return True
    return False


def atom_live(atom):
    for key, kind, v in atom["values"]:
        if kind == "bool" and v and key == "live":
            return True
        if kind == "str" and re.search(r"\blive\b|\bliveness\b", norm(v)):
            return True
    # A record may carry liveness in its meaning rather than its value.
    return bool(re.search(r"\blive\b|\bliveness\b", atom["ctx"]))


def atom_widths(atom):
    out = []
    for k, kind, v in atom["scalars"]:
        if kind == "int" and re.search(r"width|bits", norm(k)):
            out.append(v)
    for _, kind, v in atom["values"]:
        if kind == "str":
            for m in re.finditer(r"\b(\d+)\s*bits?\b", norm(v)):
                out.append(int(m.group(1)))
    return out


def reserved_records(records):
    return [
        r
        for r in records
        if re.search(
            COPY_FIELD
            + r"|\bscheme\b|\bcounter\b|\btag\b|\bauthentication\b|\breserved\b",
            r["ctx"],
            re.I,
        )
    ]


if tables_text is not None:
    records = collect_records(document) if document is not None else []
    reserved = reserved_records(records)
    if not reserved:
        fail(
            4,
            "%s carries no reserved-value entry: add the reserved values in any "
            "compatible extension (a per-field `reserved`/`reserved_value` "
            "attribute, a `[[reserved]]` array, or a keyed table)" % tables_path,
        )
        ac4 = False
    else:
        copy_atoms = atoms_for(records, COPY_FIELD)
        if not copy_atoms:
            fail(4, "%s carries no entry for the copy index (C-07)" % tables_path)
            ac4 = False
        else:
            if not any(atom_zero(a) for a in copy_atoms):
                fail(
                    4,
                    "%s: the copy index carries no reserved value 0" % tables_path,
                )
                ac4 = False
            for a in copy_atoms:
                widths = atom_widths(a)
                if widths and 2 not in widths:
                    fail(
                        4,
                        "%s: the copy index width is %s, but wire.md fixes 2 bits"
                        % (tables_path, widths),
                    )
                    ac4 = False

        auth_atoms = atoms_for(records, r"\bscheme\b")
        if not auth_atoms:
            fail(4, "%s carries no entry for the authentication reservation" % tables_path)
            ac4 = False
        else:
            if not any(atom_zero(a) for a in auth_atoms):
                fail(4, "%s: the authentication scheme carries no reserved value 0" % tables_path)
                ac4 = False
            for a in auth_atoms:
                widths = atom_widths(a)
                if widths and 8 not in widths:
                    fail(
                        4,
                        "%s: the authentication scheme width is %s, but wire.md "
                        "fixes 8 bits" % (tables_path, widths),
                    )
                    ac4 = False

        tag_atoms = atoms_for(records, r"\btag\b")
        if not tag_atoms:
            fail(4, "%s carries no entry for the authentication tag (C-12)" % tables_path)
            ac4 = False
        else:
            if not any(atom_zero(a) for a in tag_atoms):
                fail(4, "%s: the authentication tag carries no reserved value 0" % tables_path)
                ac4 = False
            for a in tag_atoms:
                widths = atom_widths(a)
                if widths and 128 not in widths:
                    fail(
                        4,
                        "%s: the authentication tag width is %s, but wire.md "
                        "fixes 128 bits" % (tables_path, widths),
                    )
                    ac4 = False

        counter_atoms = atoms_for(records, r"\bcounter\b")
        if not counter_atoms:
            fail(4, "%s carries no entry for the 64-bit message counter (C-12)" % tables_path)
            ac4 = False
        else:
            if not any(atom_live(a) for a in counter_atoms):
                fail(
                    4,
                    "%s: the message counter carries no live value (C-115: it is "
                    "live in v1, not zero)" % tables_path,
                )
                ac4 = False
            for a in counter_atoms:
                if atom_zero(a):
                    fail(
                        4,
                        "%s: the message counter is given a zero value, but C-115 "
                        "makes it live" % tables_path,
                    )
                    ac4 = False
                widths = atom_widths(a)
                if widths and 64 not in widths:
                    fail(
                        4,
                        "%s: the message counter width is %s, but wire.md fixes "
                        "64 bits" % (tables_path, widths),
                    )
                    ac4 = False

if overridden:
    print("NOTE AC4: document paths overridden; repository-level checks skipped")
elif ac4:
    # The 004-02 command-line checker must accept the file.
    try:
        proc = subprocess.run(
            ["cargo", "run", "--quiet", "--bin", "wire-check"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail(4, "could not run cargo run --bin wire-check: %s" % error)
        ac4 = False
    if proc is not None:
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail(4, "cargo run --bin wire-check did not report a clean check:\n%s" % tail)
            ac4 = False

    # The wire_tables tests must stay at 12 passed, 0 failed, 0 ignored.
    try:
        proc_tests = subprocess.run(
            ["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc_tests = None
        fail(4, "could not run the wire_tables tests: %s" % error)
        ac4 = False
    if proc_tests is not None:
        out = proc_tests.stdout + proc_tests.stderr
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if proc_tests.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail(4, "cargo test -p hadronyche-tools --test wire_tables failed:\n%s" % tail)
            ac4 = False
        if result is None:
            fail(4, "the wire_tables test run printed no 'test result:' line")
            ac4 = False
        else:
            m = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result)
            if not m:
                fail(4, "could not read the test counts from: %s" % result)
                ac4 = False
            elif (int(m.group(1)), int(m.group(2)), int(m.group(3))) != (12, 0, 0):
                fail(
                    4,
                    "wire_tables must report 12 passed, 0 failed, 0 ignored; got: %s"
                    % result,
                )
                ac4 = False

# ---------------------------------------------------------------------------
# DoD — the design checker prints OK, and 004-01 still passes
# ---------------------------------------------------------------------------
if not overridden:
    try:
        design = subprocess.run(
            ["python3", "doc/kickoff/check-design.py", "doc/design"],
            capture_output=True,
            text=True,
            timeout=300,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        design = None
        fail("DoD", "could not run the design checker: %s" % error)
    if design is not None:
        out = (design.stdout + design.stderr).strip().splitlines()
        last = out[-1].strip() if out else ""
        if design.returncode != 0 or last != "OK":
            tail = "\n".join(out[-8:])
            fail("DoD", "the design checker does not print OK:\n%s" % tail)

    if os.path.exists("checks/004-01.sh"):
        try:
            prev = subprocess.run(
                ["bash", "checks/004-01.sh"], capture_output=True, text=True, timeout=600
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            prev = None
            fail("DoD", "could not run checks/004-01.sh: %s" % error)
        if prev is not None and prev.returncode != 0:
            tail = "\n".join((prev.stdout + prev.stderr).strip().splitlines()[-8:])
            fail("DoD", "checks/004-01.sh no longer passes:\n%s" % tail)

# ---------------------------------------------------------------------------
for label, msg in warns:
    print("NOTE %s: %s" % (label, msg))

if not fails and not placeholder:
    print(
        "PASS AC1: copy index (2 bits, C-07) and authentication reservation "
        "(8+64+128 bits, C-12) carry required v1 values and receiver outcomes"
    )
    print(
        "PASS AC2: C-115 cited; 64-bit counter live as the liveness counter, "
        "scheme and tag zero"
    )
    print("PASS AC3: general padding/reserved-bit rule names ReservedNonZero")
    if overridden:
        print(
            "PASS AC4: wire.toml carries the reserved values and agrees with "
            "wire.md (repository-level checks skipped by the path override)"
        )
    else:
        print(
            "PASS AC4: wire.toml carries the reserved values and the 004-02 "
            "checker passes (wire-check clean, wire_tables 12/0/0)"
        )
    print(
        "PASS DEC: copy-index decision explicit and tagged [decision] (%s)"
        % (copy_decision or "-")
    )
    print("PASS DEDUP: dedup rule restated (stream, position, packet index, first valid copy, no sequence number)")
    print("PASS FALSIF: the duplication arrival path is stated consistently with the decision")

if fails:
    for label, msg in fails:
        shown = "AC%d" % label if isinstance(label, int) else label
        print("FAIL %s: %s" % (shown, msg))
    print("%d failure(s)" % len(fails))
    sys.exit(1)

print("OK: 004-05 acceptance criteria met")
PY
