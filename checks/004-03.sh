#!/usr/bin/env bash
# 004-03 — rejection reason enumeration.
#
# RED-phase document check. It validates doc/design/spec/wire.md and
# doc/design/spec/wire.toml against every acceptance criterion of the ticket,
# and re-runs the earlier checks the branch must keep green (checks/004-01.sh
# and the wire_tables tests).
#
# WIRE_MD / WIRE_TOML / WIRE_TEST_SRC may override the paths. The overrides are
# for self-testing on throwaway fixtures only: when either document path is
# overridden the repository-level checks (cargo tests, the ignored-test
# removal, checks/004-01.sh) are skipped with a NOTE, because they always
# target the real repository.
#
# What is checked, per acceptance criterion:
#
#   AC 1  A "Rejection reasons" section lists at least the 12 starting
#         variants (a starting name may be replaced only when a "changes from
#         the starting shape" list maps the old name to a member of the
#         enumeration), each with a unique u8 value in 0..=255, a meaning, and
#         at least one variant for a domain-0 packet that is counted and is not
#         ForeignDomain (ticket Work step 5, C-173).
#   AC 2  VersionMismatch is specified as raising a named event in addition to
#         being counted (rule 12).
#   AC 3  The counting key (input, reason) is defined for traffic matching an
#         admitted input and for traffic matching none; the no-match key is an
#         explicit per-socket / per-interface / per-source-address choice
#         tagged [decision]; the counter set has a stated numeric fixed bound.
#   AC 4  wire.toml contains the enumeration (name + value entries, in whatever
#         table or array form the author extends the schema with), agreeing
#         name-for-name and value-for-value with wire.md; every reason named in
#         a message's rejection_reasons is a member; the previously ignored
#         test rejection_reasons_referenced_are_members_of_the_closed_enumeration
#         carries no #[ignore] and `cargo test --test wire_tables` reports
#         12 passed, 0 failed, 0 ignored.
#   AC 5  The enumeration is declared closed within a major version, with a
#         stated numeric range of reserved values for future variants (not
#         overlapping any assigned value).
#
# DoD also: every paragraph of the new section carries an evidence tag
# ([code], [measured], [kernel-doc], [standard], [memory], [decision]); table
# and code blocks may inherit the tag of the paragraph that introduces them.
# The ticket's Work step 1 also asks each row for the layers that can produce
# the reason; a missing layer statement is a NOTE, not a failure, because the
# acceptance criteria do not require it.
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
TEST_SRC="${WIRE_TEST_SRC:-hadronyche-tools/tests/wire_tables.rs}"

python3 - "$WIRE" "$TABLES" "$TEST_SRC" <<'PY'
import os
import re
import subprocess
import sys

wire_path, tables_path, test_src_path = sys.argv[1], sys.argv[2], sys.argv[3]
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
# Markdown helpers (same conventions as checks/004-01.sh)
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
    """Blank out fenced code, keeping line numbers (used for tables/prose)."""
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


# ---------------------------------------------------------------------------
# Enumeration parsing from Markdown
# ---------------------------------------------------------------------------
def parse_u8(cell):
    s = cell.strip().strip("`*_ \u00a0")
    s = re.sub(r"\s+", " ", s)
    m = re.fullmatch(r"(0[xX][0-9a-fA-F]+|0[bB][01]+|\d+)(?:\s*\([^)]*\))?", s)
    if not m:
        return None
    token = m.group(1)
    if token.lower().startswith("0x"):
        return int(token, 16)
    if token.lower().startswith("0b"):
        return int(token, 2)
    return int(token, 10)


def parse_variant_name(cell):
    s = cell.strip().strip("`*_ \u00a0")
    s = re.sub(r"^Reject\s*::\s*", "", s)
    s = s.strip("`*_ \u00a0")
    if re.fullmatch(r"[A-Z][A-Za-z0-9]*", s):
        return s
    return None


def norm_header(cell):
    return re.sub(r"[^a-z0-9]", "", cell.lower())


def collect_enumeration(section):
    """Return (name -> list of (value, row_text), duplicates, table_count)."""
    found = {}
    duplicates = []
    table_count = 0

    def add(name, value, row_text):
        found.setdefault(name, []).append((value, row_text))

    for _, header, rows in parse_tables(section):
        vcol = ncol = None
        for k, h in enumerate(header):
            hn = norm_header(h)
            if vcol is None and (
                "value" in hn
                or hn in ("u8", "code", "id", "number", "num")
                or re.search(r"\bu8\b", h, re.I)
            ):
                vcol = k
            if ncol is None and (
                "name" in hn or "variant" in hn or hn == "reason" or "reject" in hn
            ):
                ncol = k

        if vcol is not None and ncol is not None:
            seen_here = False
            for row in rows:
                if len(row) <= max(vcol, ncol):
                    continue
                value = parse_u8(row[vcol])
                name = parse_variant_name(row[ncol])
                if value is None or name is None:
                    continue
                seen_here = True
                row_text = " | ".join(c for c in row if c.strip())
                add(name, value, row_text)
            if seen_here:
                table_count += 1
            continue

        # No recognisable header: accept a row with exactly one integer cell
        # and exactly one bare CamelCase variant cell (the value and name
        # columns in either order).
        for row in rows:
            values = [parse_u8(c) for c in row]
            names = [parse_variant_name(c) for c in row]
            ints = [v for v in values if v is not None]
            names_ok = [n for n in names if n is not None]
            if len(ints) == 1 and len(names_ok) == 1:
                row_text = " | ".join(c for c in row if c.strip())
                add(names_ok[0], ints[0], row_text)
                table_count += 1

    # Rust-style `Name = 0,` lines, but only inside a fenced code block or
    # with the name backticked, so ordinary prose numbers cannot be mistaken
    # for values.
    in_fence = False
    for line in section.splitlines():
        if re.match(r"^\s*```", line):
            in_fence = not in_fence
            continue
        m = re.match(
            r"^\s*(?:Reject\s*::\s*)?([A-Z][A-Za-z0-9]*)\s*=\s*(0[xX][0-9a-fA-F]+|\d+)\s*,?\s*(?://.*)?$",
            line,
        )
        if m and (in_fence or "`" in line):
            value = parse_u8(m.group(2))
            if value is not None:
                add(m.group(1), value, line.strip())
        else:
            m = re.match(
                r"^\s*[-*]?\s*`(?:Reject\s*::\s*)?([A-Z][A-Za-z0-9]*)`\s*=\s*(0[xX][0-9a-fA-F]+|\d+)\b",
                line,
            )
            if m:
                value = parse_u8(m.group(2))
                if value is not None:
                    add(m.group(1), value, line.strip())

    for name, entries in found.items():
        if len(entries) > 1:
            duplicates.append((name, [e[0] for e in entries]))
    return found, duplicates, table_count


# ---------------------------------------------------------------------------
# A small TOML reader.
#
# The checker must run in CI unprivileged and offline with the Python that the
# other checks use (3.10 here, which has no tomllib), so it reads the subset of
# TOML that the tables file uses: tables, arrays of tables, dotted keys, basic
# and literal strings, integers, floats, booleans, arrays and inline tables.
# A construct outside that subset is reported, never silently skipped.
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
            # Dates and other bare scalars are kept as text; the checker only
            # needs names and integers.
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
        # key = value
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


def walk_toml(node, path=()):
    if isinstance(node, dict):
        yield path, node
        for k, v in node.items():
            yield from walk_toml(v, path + (str(k),))
    elif isinstance(node, list):
        for v in node:
            yield from walk_toml(v, path)


CAMEL = re.compile(r"^[A-Z][A-Za-z0-9]*$")


def collect_toml_enumeration(root):
    """Collect (name -> value) entries in any table/array form.

    Accepted forms: a dict with `name` and `value` (`code`/`id` also accepted),
    an array of such dicts, and a table of name -> integer entries. The search
    is over the whole document so the author may extend the revision-1 schema
    with any compatible shape.
    """
    pairs = {}

    def add(name, value):
        pairs.setdefault(name, set()).add(value)

    def visit(node):
        if isinstance(node, dict):
            name = node.get("name")
            value = node.get("value", node.get("code", node.get("id")))
            if (
                isinstance(name, str)
                and CAMEL.fullmatch(name)
                and isinstance(value, int)
                and not isinstance(value, bool)
            ):
                add(name, value)
                return
            for k, v in node.items():
                if CAMEL.fullmatch(str(k)) and isinstance(v, int) and not isinstance(v, bool):
                    add(str(k), v)
                else:
                    visit(v)
        elif isinstance(node, list):
            for v in node:
                visit(v)

    visit(root)
    return pairs


# ---------------------------------------------------------------------------
# Load wire.md and find the Rejection reasons section
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    print(f"FAIL AC1: {wire_path} does not exist")
    for ac in range(2, 6):
        print(f"FAIL AC{ac}: cannot check — {wire_path} does not exist")
    sys.exit(1)

lines = wire.splitlines()
headings = heading_index(lines)
sec_i = None
for i, (_, _, title) in enumerate(headings):
    if re.search(r"rejection[-\s]+reasons?", title, re.I):
        sec_i = i
        break

if sec_i is None:
    fail(1, "no 'Rejection reasons' section heading in wire.md")
    section = ""
    full_section = ""
else:
    start = headings[sec_i][0] + 1
    level = headings[sec_i][1]
    end = len(lines)
    for j in range(sec_i + 1, len(headings)):
        if headings[j][1] <= level:
            end = headings[j][0]
            break
    section = "\n".join(lines[start:end])

section_for_prose = strip_fences(section)
enum, duplicates, enum_tables = collect_enumeration(section)

# ---------------------------------------------------------------------------
# AC 1 — at least the 12 starting variants, documented renames, unique u8
# ---------------------------------------------------------------------------
STARTING = [
    "TooShort",
    "BadMagic",
    "VersionMismatch",
    "ForeignDomain",
    "UnknownStream",
    "UnadmittedSource",
    "BadLength",
    "BadChecksum",
    "BadPacketIndex",
    "WrongEpoch",
    "OversizeControl",
    "ReservedNonZero",
]

ac1 = True

if not enum:
    fail(
        1,
        "the Rejection reasons section lists no variant with a numeric value "
        "(it is still the 004-03 placeholder, or the table does not carry a "
        "name column and a value column)",
    )
    ac1 = False

enum_map = {}
for name, entries in enum.items():
    values = {v for v, _ in entries}
    if len(values) > 1:
        fail(1, f"{name} is listed with more than one value: {sorted(values)}")
        ac1 = False
        continue
    enum_map[name] = next(iter(values))

for name, entries in duplicates:
    warn(
        1,
        f"{name} is listed {len(entries)} times; a repeated entry is accepted "
        "only because its values agree (the unique-value check is below)",
    )

seen_values = {}
for name, value in enum_map.items():
    if not (0 <= value <= 255):
        fail(1, f"{name} = {value} is outside the u8 range 0..=255")
        ac1 = False
    elif value in seen_values and seen_values[value] != name:
        fail(
            1,
            f"duplicate u8 value {value}: used by both {seen_values[value]} and {name}",
        )
        ac1 = False
    else:
        seen_values[value] = name

missing = [n for n in STARTING if n not in enum_map]
if missing:
    changes_region = None
    m = re.search(r"changes?\s+from\s+the\s+starting\s+shape", section, re.I)
    if m:
        changes_region = section[m.start() :]
    if changes_region is None:
        fail(
            1,
            "starting variant(s) missing with no 'changes from the starting shape' "
            f"list: {', '.join(missing)}",
        )
        ac1 = False
    else:
        # A "changes" table with a starting-name column and a final-name column
        # is the clearest form: | Starting name | Final name | Reason |.
        table_renames = {}
        for _, header, rows in parse_tables(changes_region):
            hnorm = [norm_header(h) for h in header]
            oldcol = next(
                (
                    k
                    for k, h in enumerate(hnorm)
                    if h in ("old", "oldname", "was", "before", "from")
                    or "starting" in h
                ),
                None,
            )
            newcol = next(
                (
                    k
                    for k, h in enumerate(hnorm)
                    if h in ("new", "newname", "final", "replacement", "now", "after", "to")
                    or "final" in h
                ),
                None,
            )
            if oldcol is None or newcol is None:
                continue
            for row in rows:
                if len(row) <= max(oldcol, newcol):
                    continue
                old = parse_variant_name(row[oldcol]) or row[oldcol].strip()
                new = parse_variant_name(row[newcol]) or row[newcol].strip()
                if new in enum_map:
                    table_renames[old] = new

        for old in missing:
            mapped = table_renames.get(old)
            if mapped is None:
                for line in changes_region.splitlines():
                    if not re.search(r"\b" + re.escape(old) + r"\b", line):
                        continue
                    if not re.search(
                        r"renam|replac|becom|now\s+called|moved\s+to|→|->", line, re.I
                    ):
                        continue
                    targets = [
                        n
                        for n in enum_map
                        if n != old and re.search(r"\b" + re.escape(n) + r"\b", line)
                    ]
                    if targets:
                        mapped = targets[0]
                        break
            if mapped is None:
                fail(
                    1,
                    f"starting variant {old} is absent and the 'changes from the "
                    "starting shape' list does not map it to a variant of the "
                    "enumeration (old name -> new name, in a table or with rename "
                    "language)",
                )
                ac1 = False

# A domain-0 packet is counted but is not ForeignDomain (C-173, C-11, Work step 5).
DOMAIN0 = re.compile(
    r"domain\s*(?:id\s*)?(?:=|:|is|of)?\s*(?:0|zero)\b"
    r"|\bzero[-\s]*domain\b|\bdomain[-\s]*zero\b"
    r"|\bunconfigured\s+domain\b"
    r"|\bdomain\s*(?:id\s*)?(?:is\s*)?unconfigured\b"
    r"|\bno\s+domain\b",
    re.I,
)
domain0 = []
for name, entries in enum.items():
    if name == "ForeignDomain":
        continue
    text = name + " " + " ".join(row for _, row in entries)
    if DOMAIN0.search(text):
        domain0.append(name)
if enum and not domain0:
    fail(
        1,
        "no variant counts a domain-0 packet (C-173: it is counted but is not "
        "ForeignDomain; add a variant whose name or meaning says domain 0 / "
        "unconfigured)",
    )
    ac1 = False

# Work step 1: one row per variant with a meaning. Accept a meaning column, or
# any row text beyond the name and value.
if enum and enum_tables:
    for _, header, rows in parse_tables(section):
        hnorm = [norm_header(h) for h in header]
        if not any("meaning" in h or "description" in h for h in hnorm):
            continue
        mcol = next(
            k for k, h in enumerate(hnorm) if "meaning" in h or "description" in h
        )
        ncol = next((k for k, h in enumerate(hnorm) if "name" in h or "variant" in h), None)
        if ncol is None:
            continue
        for row in rows:
            if len(row) <= max(mcol, ncol):
                continue
            if parse_variant_name(row[ncol]) is None:
                continue
            if not row[mcol].strip():
                fail(1, f"{row[ncol].strip()}: the meaning column is empty (Work step 1)")
                ac1 = False

# Work step 1 also asks which layers can produce each reason. A per-row layer
# column or one global layer statement is accepted; absence is a NOTE.
layer_column = False
for _, header, rows in parse_tables(section):
    hnorm = [norm_header(h) for h in header]
    if not any("layer" in h or "producer" in h or "produced" in h for h in hnorm):
        continue
    lcol = next(
        k for k, h in enumerate(hnorm) if "layer" in h or "producer" in h or "produced" in h
    )
    if any(
        len(row) > lcol and re.search(r"control|sync|media|measurement", row[lcol], re.I)
        for row in rows
    ):
        layer_column = True
if not layer_column:
    layer_words = sum(
        1
        for w in ("control", "sync", "media", "measurement")
        if re.search(r"\b" + w + r"\b", section_for_prose, re.I)
    )
    if layer_words < 3 and not re.search(r"\blayers?\b", section_for_prose, re.I):
        warn(
            1,
            "the section does not state which layers can produce each reason "
            "(Work step 1 asks for it; the acceptance criteria do not)",
        )

if ac1:
    print(
        "PASS AC1: "
        f"{len(enum_map)} variants, unique u8 values, all 12 starting variants "
        f"accounted for, domain-0 reason {domain0[0] if domain0 else '-'}"
    )

# ---------------------------------------------------------------------------
# AC 2 — VersionMismatch raises a named event in addition to being counted
# ---------------------------------------------------------------------------
ac2 = True
EVENT = re.compile(r"\bevents?\b", re.I)
RAISE = re.compile(r"\brais\w*|\bemit\w*|\bsignal\w*|\bproduc\w*|\breport\w*\s+an\s+event", re.I)
COUNT = re.compile(r"\bcount\w*|\bincrement\w*", re.I)

version_texts = []
for name, entries in enum.items():
    if name == "VersionMismatch":
        version_texts.extend(row for _, row in entries)
for block in blocks(section_for_prose):
    if re.search(r"\bVersionMismatch\b", block):
        version_texts.append(block)

event_ok = False
counted_ok = False
for text in version_texts:
    if EVENT.search(text) and RAISE.search(text):
        event_ok = True
        if COUNT.search(text):
            counted_ok = True

# The counting of every rejection may be stated once, globally (AC 3), so a
# row that names the event inherits "counted" from that statement.
global_counting = re.search(
    r"\b(?:every|each|all)\s+(?:rejection|rejected\s+(?:packet|input)|reject\w*)\b"
    r"[^.\n]{0,120}?\b(?:increments?|counted|counter)\b"
    r"|\b(?:increments?|counted)\b[^.\n]{0,120}?\b(?:every|each)\s+(?:rejection|reason)\b",
    section_for_prose,
    re.I,
)
if not enum_map:
    fail(2, "no enumeration to find VersionMismatch in")
    ac2 = False
elif not event_ok:
    fail(
        2,
        "VersionMismatch is not specified as raising a named event (rule 12: a "
        "version mismatch is a named event, never a silent drop); expected "
        "event language (raise/emit) in its row or in a sentence naming it",
    )
    ac2 = False
elif not counted_ok and not global_counting:
    fail(
        2,
        "VersionMismatch names an event but is not stated as also counted; add "
        "'counted' to its row or state that every rejection is counted",
    )
    ac2 = False

if ac2 and enum_map:
    print("PASS AC2: VersionMismatch raises a named event and is counted")

# ---------------------------------------------------------------------------
# AC 3 — counting key (input, reason) for admitted and non-matching traffic,
#        [decision]-tagged no-match key, numeric fixed bound on the counters
# ---------------------------------------------------------------------------
ac3 = True
prose_blocks = blocks(section_for_prose)

KEY = re.compile(
    r"\(\s*input\s*,\s*reason\s*\)"
    r"|\bkeyed\s+by\b[^.\n]{0,100}\binput\b[^.\n]{0,100}\breason\b"
    r"|\bper\s+input\b[^.\n]{0,80}\bper\s+reason\b"
    r"|\bper\s+reason\b[^.\n]{0,80}\bper\s+input\b"
    r"|\binput\b[^.\n]{0,40}\breason\b[^.\n]{0,20}\bkey\b",
    re.I,
)
ADMITTED_INPUT = re.compile(
    r"\badmitted\s+(?:stream|input)\b|\binput\b[^.\n]{0,80}\badmitted\b|\badmitted\b[^.\n]{0,80}\binput\b",
    re.I,
)
NOMATCH = re.compile(
    r"matches?\s+no\s+(?:admitted\s+)?input"
    r"|\bmatching\s+none\b"
    r"|\bno\s+admitted\s+input\b"
    r"|\bdoes\s+not\s+match\s+(?:an|any)\s+(?:admitted\s+)?input\b"
    r"|\bmatches?\s+none\s+of\s+the\s+admitted\s+inputs\b"
    r"|\bno\s+input\s+matches?\b"
    r"|\bunmatched\s+(?:traffic|packets?|input)\b"
    r"|\btraffic\s+that\s+matches?\s+none\b",
    re.I,
)
NOMATCH_KEY = re.compile(
    r"\bper[\s-]+(?:socket|interface|source(?:\s+address)?|src)\b"
    r"|\bby\s+(?:the\s+)?source\s+address\b"
    r"|\bsource[-\s]+address[-\s]+key\b",
    re.I,
)
BOUND_WORDS = re.compile(
    r"\b(?:at\s+most|no\s+more\s+than|maximum\s+(?:of\s+)?|bounded\s+by|"
    r"bound\s+of|fixed\s+bound\s+of|fixed\s+at|set\s+to|limit(?:ed)?\s+(?:to|of)|"
    r"cap(?:ped)?\s+(?:at|of)|up\s+to|≤|<=)\s*"
    r"(?:~|about\s+)?(\d[\d _]*)",
    re.I,
)
COUNTER_WORD = re.compile(r"\bcounters?\b", re.I)

admitted_defined = bool(ADMITTED_INPUT.search(section_for_prose))
key_defined = bool(KEY.search(section_for_prose))
if not admitted_defined:
    fail(
        3,
        "the section does not state the design's meaning of input (the admitted "
        "stream at a consumer, C-16/C-109) for the counting key",
    )
    ac3 = False
if not key_defined:
    fail(
        3,
        "the counting key (input, reason) is not defined for traffic matching "
        "an admitted input",
    )
    ac3 = False

nomatch_ok = False
nomatch_decision_ok = False
for idx, block in enumerate(prose_blocks):
    if not NOMATCH.search(block):
        continue
    sentences = re.split(r"(?<=[.!?])\s+(?=[A-Z`(\[])", block)
    candidates = []
    for k, sentence in enumerate(sentences):
        if NOMATCH.search(sentence):
            candidates.append(sentence)
            if k + 1 < len(sentences) and re.search(r"\bkey(?:ed|s)?\b", sentences[k + 1], re.I):
                candidates.append(sentences[k + 1])
    if not any(NOMATCH_KEY.search(s) for s in candidates):
        continue
    nomatch_ok = True
    context = block
    if idx > 0:
        context += "\n" + prose_blocks[idx - 1]
    if idx + 1 < len(prose_blocks):
        context += "\n" + prose_blocks[idx + 1]
    if "[decision]" in context:
        nomatch_decision_ok = True
    break
if not nomatch_ok:
    fail(
        3,
        "the key for traffic matching no admitted input is not defined; name an "
        "explicit choice: per socket, per interface or per source address",
    )
    ac3 = False
elif not nomatch_decision_ok:
    fail(
        3,
        "the no-match counting key is not tagged [decision] (the ticket says "
        "decide it and tag the decision)",
    )
    ac3 = False

bound_ok = False
for block in prose_blocks:
    if not COUNTER_WORD.search(block):
        continue
    m = BOUND_WORDS.search(block)
    if m:
        number = int(m.group(1).replace(" ", "").replace("_", ""))
        if number > 0:
            bound_ok = True
            break
    m = re.search(
        r"\b(\d[\d _]*)\s+(?:distinct\s+|separate\s+|rejection\s+|reason\s+|input\s+)?counters?\b",
        block,
        re.I,
    )
    if m:
        number = int(m.group(1).replace(" ", "").replace("_", ""))
        if number > 0:
            bound_ok = True
            break
if not bound_ok:
    fail(
        3,
        "the counter set has no stated fixed numeric bound; 'bounded' alone is "
        "not enough, give the number (e.g. 'at most 1 024 counters')",
    )
    ac3 = False

if ac3:
    print(
        "PASS AC3: counting key defined for admitted and non-matching traffic, "
        "no-match key tagged [decision], counter bound is a number"
    )

# ---------------------------------------------------------------------------
# AC 4 — wire.toml holds the enumeration; the reason-reference test is enabled
#         and the wire_tables tests pass
# ---------------------------------------------------------------------------
ac4 = True
tables_text = read_text(tables_path)
toml_pairs = {}
toml_error = None
if tables_text is None:
    fail(4, f"{tables_path} does not exist")
    ac4 = False
else:
    try:
        document = parse_toml(tables_text)
        # The packet Reject enumeration ([[reject]]) is compared with wire.md. A
        # refusal enumeration of another kind (admission refusals, separate numeric
        # values per wire.md's Rejection reasons section) may name the same
        # variant with its own value, so the one-name-one-value rule is applied
        # within each top-level enumeration, not across them.
        groups = {}
        for key, sub in document.items():
            groups[key] = collect_toml_enumeration({key: sub})
        for key, pairs in groups.items():
            if key == "reject":
                continue
            for name, values in pairs.items():
                if len(values) > 1:
                    fail(4, f"{tables_path}: {name} is listed with more than one value within '{key}': {sorted(values)}")
                    ac4 = False
        if "reject" in groups:
            toml_pairs = groups["reject"]
        else:
            toml_pairs = collect_toml_enumeration(document)
    except TomlError as error:
        toml_error = error
        fail(4, f"{tables_path} could not be read by this check: {error}")
        ac4 = False

if toml_error:
    pass
elif not toml_pairs and tables_text is not None:
    fail(
        4,
        f"{tables_path} carries no enumeration entry (no table or array whose "
        "entries are a variant name plus a numeric value)",
    )
    ac4 = False
else:
    flat = {}
    for name, values in toml_pairs.items():
        if len(values) > 1:
            fail(4, f"{tables_path}: {name} is listed with more than one value: {sorted(values)}")
            ac4 = False
            continue
        flat[name] = next(iter(values))

    if len(flat) < 12:
        fail(
            4,
            f"{tables_path} carries {len(flat)} enumeration entr(y/ies); the "
            "closed Reject enumeration needs at least the 12 starting variants",
        )
        ac4 = False

    if enum_map:
        missing_in_toml = [n for n in enum_map if n not in flat]
        for name in missing_in_toml:
            fail(4, f"wire.md lists {name} but {tables_path} has no entry for it")
            ac4 = False
        for name, value in enum_map.items():
            if name in flat and flat[name] != value:
                fail(
                    4,
                    f"{name}: wire.md says {value} but {tables_path} says {flat[name]}; "
                    "the two files must agree name-for-name and value-for-value",
                )
                ac4 = False
        extra = sorted(set(flat) - set(enum_map))
        if extra:
            warn(
                4,
                f"{tables_path} has enumeration entries wire.md does not list: "
                f"{', '.join(extra)} (if they are Reject variants, wire.md must list them)",
            )

    # Every reason named in a message's rejection_reasons must be a member; this
    # is the property the enabled test and the 004-02 checker are about.
    for path, node in walk_toml(document):
        if not isinstance(node, dict):
            continue
        reasons = node.get("rejection_reasons")
        if reasons is None:
            continue
        if not isinstance(reasons, list):
            fail(4, f"{'.'.join(path) or 'file'}: rejection_reasons must be an array")
            ac4 = False
            continue
        for reason in reasons:
            if not isinstance(reason, str):
                continue
            if reason not in flat:
                fail(
                    4,
                    f"message {node.get('name', '?')}: rejection reason {reason!r} "
                    "is not a member of the enumeration",
                )
                ac4 = False

if overridden:
    print("NOTE AC4: document paths overridden; repository-level checks skipped")
else:
    # 1. The previously ignored test carries no #[ignore].
    test_src = read_text(test_src_path)
    TEST_NAME = "rejection_reasons_referenced_are_members_of_the_closed_enumeration"
    if test_src is None:
        fail(4, f"{test_src_path} does not exist")
        ac4 = False
    else:
        src_lines = test_src.splitlines()
        fn_line = None
        for k, line in enumerate(src_lines):
            if re.search(r"\bfn\s+" + re.escape(TEST_NAME) + r"\b", line):
                fn_line = k
                break
        if fn_line is None:
            fail(4, f"the test {TEST_NAME} is missing from {test_src_path}")
            ac4 = False
        else:
            block_start = fn_line
            k = fn_line - 1
            while k >= 0:
                s = src_lines[k].strip()
                if s.startswith("#[") or s.startswith("///") or s.startswith("//!") or s.startswith("//"):
                    block_start = k
                    k -= 1
                    continue
                break
            attrs = "\n".join(src_lines[block_start:fn_line])
            if re.search(r"#\[\s*ignore", attrs):
                fail(
                    4,
                    f"{TEST_NAME} still carries #[ignore]; 004-03 enables it by "
                    "removing the attribute (the test body must not change)",
                )
                ac4 = False

    # 2. The wire_tables tests: 12 passed, 0 failed, 0 ignored.
    try:
        proc = subprocess.run(
            ["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail(4, f"could not run the wire_tables tests: {error}")
        ac4 = False

    if proc is not None:
        out = proc.stdout + proc.stderr
        if proc.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail(4, f"cargo test -p hadronyche-tools --test wire_tables failed:\n{tail}")
            ac4 = False
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if result is None:
            fail(4, "the wire_tables test run printed no 'test result:' line")
            ac4 = False
        else:
            m = re.search(
                r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result
            )
            if not m:
                fail(4, f"could not read the test counts from: {result}")
                ac4 = False
            elif (int(m.group(1)), int(m.group(2)), int(m.group(3))) != (12, 0, 0):
                fail(
                    4,
                    "wire_tables must report 12 passed, 0 failed, 0 ignored (the "
                    f"ignored reason-reference test enabled); got: {result}",
                )
                ac4 = False
            if TEST_NAME not in out:
                warn(
                    4,
                    f"the test runner output does not name {TEST_NAME} (it may be "
                    "there without its name on a passing line)",
                )

    # 3. The whole crate's tests still pass.
    try:
        proc_all = subprocess.run(
            ["cargo", "test", "-p", "hadronyche-tools"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc_all = None
        fail(4, f"could not run cargo test -p hadronyche-tools: {error}")
        ac4 = False
    if proc_all is not None and proc_all.returncode != 0:
        out = proc_all.stdout + proc_all.stderr
        tail = "\n".join(out.strip().splitlines()[-12:])
        fail(4, f"cargo test -p hadronyche-tools failed:\n{tail}")
        ac4 = False

    # 4. 004-01's checks still pass on the branch.
    if os.path.exists("checks/004-01.sh"):
        try:
            proc_prev = subprocess.run(
                ["bash", "checks/004-01.sh"], capture_output=True, text=True, timeout=600
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            proc_prev = None
            fail(4, f"could not run checks/004-01.sh: {error}")
            ac4 = False
        if proc_prev is not None and proc_prev.returncode != 0:
            tail = "\n".join((proc_prev.stdout + proc_prev.stderr).strip().splitlines()[-8:])
            fail(4, f"checks/004-01.sh no longer passes:\n{tail}")
            ac4 = False

if ac4:
    if overridden:
        print(
            "PASS AC4: wire.toml enumeration agrees with wire.md and every "
            "referenced reason is a member (repository-level checks skipped by "
            "the path override)"
        )
    else:
        print(
            "PASS AC4: wire.toml enumeration agrees with wire.md, every referenced "
            "reason is a member, the reason-reference test is enabled, wire_tables "
            "reports 12/0/0 and 004-01 still passes"
        )

# ---------------------------------------------------------------------------
# AC 5 — closed within a major version, reserved numeric range, no overlap
# ---------------------------------------------------------------------------
ac5 = True
closed_ok = any(
    re.search(r"\bclosed\b", block, re.I) and re.search(r"\bmajor[-\s]+version\b", block, re.I)
    for block in prose_blocks
)
if not closed_ok:
    fail(
        5,
        "the section does not declare the enumeration closed within a major "
        "version (both 'closed' and 'major version' in one statement)",
    )
    ac5 = False

RANGE1 = re.compile(
    r"\breserved\b[^.\n]{0,50}?"
    r"(?:values?|range|ids?|codes?|variants?|from)\b[^.\n]{0,15}?[:(]?\s*"
    r"(\d+|0[xX][0-9a-fA-F]+)\s*(?:\.\.=|\.\.|–|—|-|to|through)\s*"
    r"(\d+|0[xX][0-9a-fA-F]+)",
    re.I,
)
RANGE2 = re.compile(
    r"(\d+|0[xX][0-9a-fA-F]+)\s*(?:\.\.=|\.\.|–|—|-|to|through)\s*"
    r"(\d+|0[xX][0-9a-fA-F]+)[^.\n\d]{0,50}?\breserved\b",
    re.I,
)
RANGE3 = re.compile(
    r"\breserved\b(?:[^.\n\d]{0,30}?[:(–—-])?\s*(\d+|0[xX][0-9a-fA-F]+)\s*"
    r"(?:\.\.=|\.\.|–|—|-|to|through)\s*(\d+|0[xX][0-9a-fA-F]+)",
    re.I,
)


def to_int(token):
    token = token.strip()
    if token.lower().startswith("0x"):
        return int(token, 16)
    return int(token, 10)


reserved_ranges = []
for block in prose_blocks:
    for pattern in (RANGE1, RANGE2, RANGE3):
        for m in pattern.finditer(block):
            try:
                lo, hi = to_int(m.group(1)), to_int(m.group(2))
            except ValueError:
                continue
            if lo <= hi:
                reserved_ranges.append((lo, hi, m.group(0)))
# Deduplicate ranges quoted in more than one way.
seen_ranges = set()
unique_ranges = []
for lo, hi, text in reserved_ranges:
    if (lo, hi) in seen_ranges:
        continue
    seen_ranges.add((lo, hi))
    unique_ranges.append((lo, hi, text))
reserved_ranges = unique_ranges
if not reserved_ranges:
    fail(
        5,
        "no reserved value range for future variants; state a numeric range, "
        "e.g. 'values 240..=250 are reserved'",
    )
    ac5 = False
else:
    in_range = False
    for lo, hi, text in reserved_ranges:
        if 0 <= lo <= hi <= 255:
            in_range = True
        else:
            fail(5, f"reserved range {text!r} is not inside the u8 range 0..=255")
            ac5 = False
    if not in_range:
        ac5 = False
    for name, value in enum_map.items():
        for lo, hi, text in reserved_ranges:
            if 0 <= lo <= value <= hi <= 255:
                fail(
                    5,
                    f"assigned value {value} ({name}) falls inside the reserved "
                    f"range {text!r}",
                )
                ac5 = False

if ac5:
    print(
        "PASS AC5: closed within a major version, reserved range "
        + ", ".join(f"{lo}..={hi}" for lo, hi, _ in reserved_ranges)
        + ", no assigned value inside it"
    )

# ---------------------------------------------------------------------------
# Definition of done — evidence tags on the new section
# ---------------------------------------------------------------------------
prev_tagged = True
for block in blocks(section):
    stripped = block.strip()
    if not stripped:
        continue
    # A block may start with a heading line ("### Counting model") followed
    # immediately by prose; check the prose, not the heading.
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
        # A table or code block may inherit the tag of the paragraph that
        # introduces it.
        prev_tagged = True
        continue
    fail(
        -1,
        "this block of the Rejection reasons section carries no evidence tag "
        "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
        + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
    )
    prev_tagged = False

# ---------------------------------------------------------------------------
for ac, msg in warns:
    print(f"NOTE AC{ac}: {msg}")

if fails:
    for ac, msg in fails:
        label = "DoD" if ac == -1 else f"AC{ac}"
        print(f"FAIL {label}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 5/5 acceptance criteria for 004-03")
PY
