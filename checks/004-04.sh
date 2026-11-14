#!/usr/bin/env bash
# 004-04 — versioning rules, magic and version fields.
#
# RED-phase document check. It validates the Versioning section of
# doc/design/spec/wire.md and the machine-readable doc/design/spec/wire.toml
# against every acceptance criterion of the ticket, and re-runs the earlier
# checks the branch must keep green (the 004-02 wire checker and the
# wire_tables tests).
#
# WIRE_MD / WIRE_TOML may override the document paths. The overrides are for
# self-testing on throwaway fixtures only: when either path is overridden the
# repository-level checks (cargo run --bin wire-check, the wire_tables tests
# and the design checker) are skipped with a NOTE, because they always target
# the real repository.
#
# What is checked, per acceptance criterion:
#
#   AC 1  The Versioning section states the magic value and width, the value is
#         not the retired POC's 0x4844, and the section makes a packet carrying
#         the POC magic distinguishable (when the chosen width makes 0x4844 a
#         prefix, the section must say how). It states the version field width,
#         the v1 version value, and a major/minor split (bit positions, high/low
#         halves, or explicit major/minor values). The ticket's falsifier is
#         satisfied either by an explicit sentence saying a v1 receiver can tell
#         a POC packet or a future major from a v1 packet before reading any
#         other field, or by the mechanical properties the section must have
#         anyway (a magic that differs from 0x4844 and a version whose major is
#         a distinct part of the field).
#
#         Encoding used by this check: the magic is a hexadecimal literal
#         stated together with the word "magic" and its width ("2-byte",
#         "u16", "width (bytes): 2", ...). The version width is the width
#         stated closest to the word "version" in the same statement. A
#         statement is a sentence of prose, one bullet line, or one table row
#         rendered as "Header: cell; Header: cell; ...". A POC / retired /
#         formerly clause is never taken as the chosen value, but a sentence
#         that mentions both the chosen value and the retired one is accepted.
#
#   AC 2  The section defines a minor change and a major change, each with at
#         least 3 examples (bullet list, table rows, or an inline
#         "adds/for example/such as" list), and "a new regime stamp section
#         type" appears among the minor examples.
#
#   AC 3  Receiver behaviour is stated for the 5 cases — same version, same
#         major/higher minor, same major/lower minor, different major, bad
#         magic — each with an outcome. An outcome is accept, or drop and count
#         under a member of the 004-03 Reject enumeration, or a named event.
#         VersionMismatch raises the named event (rule 12) and is always
#         counted; different major drops/counts VersionMismatch and raises the
#         event; a lower minor may accept or drop, but a VersionMismatch outcome
#         raises the event; bad magic drops/counts a member of the enumeration.
#         The section also states that the hello raises the event before a
#         connection exists.
#
#   AC 4  The magic and the version are at fixed offsets, the magic at 0 and
#         the version immediately after it, and an explicit immovability
#         statement names the offsets ("no future version may move ... at
#         offsets 0 and 2"). Work step 5's "checked before any other field" is
#         a NOTE when absent (the acceptance criteria do not require it).
#
#   AC 5  wire.toml carries a magic entry and a version entry (any compatible
#         schema extension: a [versioning] table, direct magic/version keys, a
#         [[constant]] entry, or a field definition named magic/version). Where
#         both files state a value, they agree: the magic literally, the
#         version as a raw value or as the packed major<<minor of the stated
#         split. `cargo run --bin wire-check` passes and `cargo test -p
#         hadronyche-tools --test wire_tables` reports 12 passed, 0 failed,
#         0 ignored.
#
# DoD: every prose block of the Versioning section carries an evidence tag
# ([code] [measured] [kernel-doc] [standard] [memory] [decision]); each of the
# six decisions (magic, version scheme, v1 value, minor definition, major
# definition, receiver behaviour, offsets) carries [decision]; an open item, if
# any, names an owner and a falsifier; `python3 doc/kickoff/check-design.py
# doc/design` prints OK.
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
# Markdown helpers (same conventions as checks/004-01.sh and 004-03.sh)
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


def section_text(text, pattern):
    """Content of the first heading whose title matches pattern."""
    lines = text.splitlines()
    headings = heading_index(lines)
    for i, (line_no, level, title) in enumerate(headings):
        if re.search(pattern, title, re.I):
            start = line_no + 1
            end = len(lines)
            for j in range(i + 1, len(headings)):
                if headings[j][1] <= level:
                    end = headings[j][0]
                    break
            return "\n".join(lines[start:end])
    return None


EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)
DECISION = re.compile(r"\[decision\]", re.I)


def has_evidence(text):
    return bool(EVIDENCE.search(text))


def norm_key(cell):
    return re.sub(r"[^a-z0-9]", "", cell.lower())


def split_sentences(line):
    return [
        s.strip()
        for s in re.split(r"(?<=[.!?])\s+(?=[A-Z`(\[])", line)
        if s.strip()
    ]


def clamp_int(token):
    token = token.strip()
    if token.lower().startswith("0x"):
        return int(token, 16)
    if token.lower().startswith("0b"):
        return int(token, 2)
    return int(token, 10)


# ---------------------------------------------------------------------------
# Block and statement model.
#
# A block is a paragraph, a table or a fenced code block. A statement is a
# sentence of prose, one bullet line, one table row rendered with its header
# names ("Header: cell; ..."), or a code block. Statements carry the index of
# the block they came from so the [decision] checks can look at the paragraph
# that introduces them.
# ---------------------------------------------------------------------------
def iter_blocks(text):
    lines = text.splitlines()
    blocks = []
    i = 0
    n = len(lines)
    while i < n:
        if not lines[i].strip():
            i += 1
            continue
        stripped = lines[i].lstrip()
        if stripped.startswith("```"):
            j = i + 1
            while j < n and not lines[j].lstrip().startswith("```"):
                j += 1
            blocks.append({"kind": "code", "text": "\n".join(lines[i : j + 1]), "line": i})
            i = j + 1
            continue
        if stripped.startswith("|"):
            j = i
            while j < n and lines[j].lstrip().startswith("|"):
                j += 1
            blocks.append({"kind": "table", "text": "\n".join(lines[i:j]), "line": i})
            i = j
            continue
        j = i
        while (
            j < n
            and lines[j].strip()
            and not lines[j].lstrip().startswith("|")
            and not lines[j].lstrip().startswith("```")
        ):
            j += 1
        blocks.append({"kind": "prose", "text": "\n".join(lines[i:j]), "line": i})
        i = j
    return blocks


def statements(blocks):
    out = []
    for bi, block in enumerate(blocks):
        if block["kind"] == "table":
            for _, header, rows in parse_tables(block["text"]):
                for row in rows:
                    parts = []
                    for k in range(min(len(header), len(row))):
                        if row[k].strip():
                            parts.append(f"{header[k]}: {row[k]}")
                    if parts:
                        out.append({"text": " ; ".join(parts), "block": bi})
        elif block["kind"] == "prose":
            for line in block["text"].splitlines():
                s = line.strip()
                if not s:
                    continue
                if re.match(r"^#{1,6}\s+", s):
                    out.append({"text": s, "block": bi, "heading": True})
                    continue
                s = re.sub(r"^(?:[-*+]|\d+[.)])\s+", "", s)
                for sentence in split_sentences(s):
                    out.append({"text": sentence, "block": bi})
        else:
            out.append({"text": block["text"], "block": bi})
    return out


def clauses(statement):
    """Split a statement into clauses at semicolons, commas and full stops."""
    return [c.strip() for c in re.split(r"[;,]|(?<=[A-Za-z)\]])\s*\.\s+", statement) if c.strip()]


RETIRED = re.compile(r"\b(?:POC|retired|formerly|old)\b", re.I)


def is_retired(clause):
    return bool(RETIRED.search(clause))


def nearest_index(text, word_re):
    positions = [m.start() for m in re.finditer(word_re, text, re.I)]
    return positions


def distance_to_nearest(pos, positions):
    if not positions:
        return 10 ** 9
    return min(abs(pos - p) for p in positions)


# ---------------------------------------------------------------------------
# Width extraction: returns (width_in_bits, position) candidates.
# ---------------------------------------------------------------------------
WORD_NUMBERS = {
    "one": 1,
    "two": 2,
    "single": 1,
    "three": 3,
    "four": 4,
    "six": 6,
    "eight": 8,
    "sixteen": 16,
}


def width_candidates(text):
    found = []
    for m in re.finditer(r"(\d+)\s*[-\s]*(?:bytes?|octets?)\b", text, re.I):
        found.append((int(m.group(1)) * 8, m.start()))
    for word, value in WORD_NUMBERS.items():
        for m in re.finditer(rf"\b{word}[-\s]+(?:bytes?|octets?)\b", text, re.I):
            found.append((value * 8, m.start()))
    for m in re.finditer(r"\bu(8|16|32|64|128)\b", text, re.I):
        found.append((int(m.group(1)), m.start()))
    for m in re.finditer(r"(\d+)\s*[-\s]*bits?\b", text, re.I):
        found.append((int(m.group(1)), m.start()))
    for m in re.finditer(
        r"\bwidth\s*(?:\([^)]*\))?\s*(?:=|:|is|of)?\s*(\d+)\s*(bits?|bytes?|octets?)?",
        text,
        re.I,
    ):
        n = int(m.group(1))
        unit = (m.group(2) or "").lower()
        found.append((n if unit.startswith("bit") else n * 8, m.start()))
    # De-duplicate exact duplicates.
    seen = set()
    out = []
    for width, pos in found:
        if width <= 0 or (width, pos) in seen:
            continue
        seen.add((width, pos))
        out.append((width, pos))
    return out


def clause_spans(text):
    """(start, end) spans at semicolons, commas and full stops."""
    cuts = [0]
    for m in re.finditer(r"[;,]|(?<=[A-Za-z)\]])\s*\.\s+", text):
        cuts.append(m.start())
        cuts.append(m.end())
    cuts.append(len(text))
    spans = []
    for i in range(0, len(cuts) - 1, 2):
        if cuts[i + 1] > cuts[i]:
            spans.append((cuts[i], cuts[i + 1]))
    return spans


def field_width(statement, word_re):
    """The width stated closest to `word_re`, ignoring a retired/POC clause."""
    anchors = nearest_index(statement, word_re)
    candidates = width_candidates(statement)
    if not candidates or not anchors:
        return None
    live_spans = [
        (start, end)
        for start, end in clause_spans(statement)
        if re.search(word_re, statement[start:end], re.I)
        and not is_retired(statement[start:end])
    ]
    if live_spans:
        live_anchors = []
        for start, end in live_spans:
            for m in re.finditer(word_re, statement[start:end], re.I):
                live_anchors.append(start + m.start())
        live_candidates = [
            (width, pos)
            for width, pos in candidates
            if any(start <= pos < end for start, end in live_spans)
        ]
        if live_candidates and live_anchors:
            anchors = live_anchors
            candidates = live_candidates
    candidates.sort(key=lambda c: (distance_to_nearest(c[1], anchors), -c[0]))
    return candidates[0][0]


def hex_candidates(statement):
    return [(int(m.group(1), 16), m.start()) for m in re.finditer(r"0[xX]([0-9a-fA-F]+)", statement)]


def chosen_numeric(statement, word_re, kind):
    """The value closest to `word_re`, skipping a retired/POC clause."""
    anchors = nearest_index(statement, word_re)
    if kind == "hex":
        candidates = hex_candidates(statement)
    else:
        candidates = [
            (clamp_int(m.group(1)), m.start())
            for m in re.finditer(r"(?<![\w.])(\d[\d_]*)(?![\d.]|\.\d)", statement)
        ]
    if not candidates:
        return None
    # Drop candidates that sit in a clause naming the retired POC.
    bounds = [0]
    for m in re.finditer(r"[;,]|(?<=[A-Za-z)\]])\s*\.\s+", statement):
        bounds.append(m.start())
        bounds.append(m.end())
    bounds.append(len(statement))

    def retired_at(pos):
        start = max(b for b in bounds if b <= pos)
        end = min((b for b in bounds if b > pos), default=len(statement))
        return is_retired(statement[start:end])

    live = [c for c in candidates if not retired_at(c[1])]
    if not live:
        return None
    # Prefer the candidate after the anchor word, closest to it.
    after = [c for c in live if anchors and c[1] >= min(anchors)]
    pool = after or live
    pool.sort(key=lambda c: distance_to_nearest(c[1], anchors))
    return pool[0][0]


# ---------------------------------------------------------------------------
# Load wire.md and locate the Versioning and Rejection reasons sections.
# ---------------------------------------------------------------------------
wire = read_text(wire_path)
if wire is None:
    print(f"FAIL AC1: {wire_path} does not exist")
    for ac in range(2, 6):
        print(f"FAIL AC{ac}: cannot check — {wire_path} does not exist")
    print("FAIL DoD: cannot check evidence tags — wire.md does not exist")
    sys.exit(1)

versioning = section_text(wire, r"versioning")
if versioning is None:
    for ac in range(1, 5):
        print(f"FAIL AC{ac}: no 'Versioning' section heading in {wire_path}")
    versioning = ""

blocks = iter_blocks(versioning)
stmts = statements(blocks)

rejection_section = section_text(wire, r"rejection[-\s]+reasons?") or ""

tables_text = read_text(tables_path)
if tables_text is None:
    fail(5, f"{tables_path} does not exist")
    tables_text = ""

# ---------------------------------------------------------------------------
# The 004-03 Reject enumeration: names from wire.md's table, unioned with the
# [[reject]] tables of wire.toml. A reason named by the Versioning section must
# be a member of this closed set.
# ---------------------------------------------------------------------------
def reject_names_from_markdown(text):
    names = set()
    for _, header, rows in parse_tables(text):
        hnorm = [norm_key(h) for h in header]
        ncol = next((k for k, h in enumerate(hnorm) if "name" in h or "reason" in h), None)
        vcol = next(
            (
                k
                for k, h in enumerate(hnorm)
                if "value" in h or "u8" in h or h in ("id", "code", "number")
            ),
            None,
        )
        if ncol is None or vcol is None:
            continue
        for row in rows:
            if len(row) <= max(ncol, vcol):
                continue
            name = row[ncol].strip().strip("`*_ ")
            name = re.sub(r"^Reject\s*::\s*", "", name)
            if re.fullmatch(r"[A-Z][A-Za-z0-9]*", name):
                try:
                    clamp_int(row[vcol].strip().strip("`*_ "))
                except ValueError:
                    continue
                names.add(name)
    return names


def strip_comment(line):
    out = []
    quote = None
    for i, c in enumerate(line):
        if quote:
            out.append(c)
            if c == quote and (i == 0 or line[i - 1] != "\\"):
                quote = None
        elif c in "\"'":
            quote = c
            out.append(c)
        elif c == "#":
            break
        else:
            out.append(c)
    return "".join(out)


def toml_blocks(text):
    """[(header, {key: raw_value})] for the subset of TOML this file uses."""
    blocks = []
    header = None
    current = {}
    for raw in text.splitlines():
        line = strip_comment(raw).strip()
        if not line:
            continue
        if line.startswith("[[") and line.endswith("]]"):
            if current or header is not None:
                blocks.append((header, current))
            header = line[2:-2].strip()
            current = {}
            continue
        if line.startswith("[") and line.endswith("]"):
            if current or header is not None:
                blocks.append((header, current))
            header = line[1:-1].strip()
            current = {}
            continue
        m = re.match(r"([A-Za-z0-9_.-]+)\s*=\s*(.*)$", line)
        if m:
            current[m.group(1)] = m.group(2).strip()
    if current or header is not None:
        blocks.append((header, current))
    return blocks


toml_blocks_parsed = toml_blocks(tables_text)


def reject_names_from_toml(blocks):
    names = set()
    for header, kv in blocks:
        if header is not None and header.strip() == "reject" and "name" in kv and "value" in kv:
            name = kv["name"].strip().strip("\"'")
            if re.fullmatch(r"[A-Z][A-Za-z0-9]*", name):
                names.add(name)
    return names


enum_names = reject_names_from_markdown(rejection_section) | reject_names_from_toml(toml_blocks_parsed)
if not enum_names:
    warn(-1, "could not read the 004-03 Reject enumeration; named reasons cannot be validated")

# ---------------------------------------------------------------------------
# AC 1 — magic value and width, v1 version value and width, major/minor scheme
# ---------------------------------------------------------------------------
ac1 = True
POC_MAGIC = 0x4844

magic_statements = [s for s in stmts if re.search(r"\bmagic\b", s["text"], re.I)]
magic_choices = []
for s in magic_statements:
    text = s["text"]
    value = chosen_numeric(text, r"\bmagic\b", "hex")
    if value is None:
        continue
    width = field_width(text, r"\bmagic\b")
    if width is None:
        continue
    magic_choices.append({"value": value, "width": width, "stmt": s})

# Keep the chosen value only if it is consistently stated. A sentence naming
# the retired value alone yields no choice at all (chosen_numeric skips the
# retired clause).
distinct_magic = {}
for choice in magic_choices:
    distinct_magic.setdefault((choice["value"], choice["width"]), []).append(choice)

if not distinct_magic:
    fail(
        1,
        "the Versioning section states no magic value and width (expected a "
        "hexadecimal literal stated with the word magic and its width, e.g. "
        "'the magic is the 2-byte value 0x4d4e')",
    )
    ac1 = False
    magic_value = magic_width = None
elif len(distinct_magic) > 1:
    fail(
        1,
        "the magic is stated with more than one value/width: "
        + ", ".join(f"0x{v:04x} ({w} bits)" for v, w in sorted(distinct_magic)),
    )
    ac1 = False
    magic_value = magic_width = None
else:
    (magic_value, magic_width), choices = next(iter(distinct_magic.items()))
    if magic_width % 8 != 0 or magic_width == 0:
        fail(1, f"the magic width {magic_width} bits is not a whole number of bytes")
        ac1 = False
    elif magic_value == POC_MAGIC:
        fail(
            1,
            "the magic is the retired POC's 0x4844; the ticket requires a value "
            "distinguishable from it",
        )
        ac1 = False
    else:
        magic_bytes = magic_value.to_bytes(magic_width // 8, "little")
        poc_bytes = b"\x44\x48"
        overlap = magic_bytes[: min(len(magic_bytes), 2)] == poc_bytes[: min(len(magic_bytes), 2)]
        if overlap:
            # The chosen magic's leading bytes coincide with the POC magic: the
            # section must say how the two are told apart.
            distinguishing = [
                s["text"]
                for s in stmts
                if re.search(r"\b(?:distinguish|differ|tell|apart|confus|mistak|prefix)\w*\b", s["text"], re.I)
                and re.search(r"0[xX]4844|\bPOC\b", s["text"], re.I)
            ]
            if not distinguishing:
                fail(
                    1,
                    f"the magic 0x{magic_value:x} shares its leading bytes with the "
                    "POC magic 0x4844; the section must state how the two are "
                    "distinguished (a sentence naming 0x4844/POC and distinguish/"
                    "differ/prefix)",
                )
                ac1 = False

        if ac1:
            print(
                f"PASS AC1 (magic): {magic_width}-bit magic 0x{magic_value:0{magic_width // 4}x}, "
                f"distinct from the POC's 0x4844"
            )

# The version field width.
version_statements = [
    s
    for s in stmts
    if re.search(r"\bversion\b", s["text"], re.I)
    and field_width(s["text"], r"\bversion\b") is not None
]
version_widths = {}
for s in version_statements:
    w = field_width(s["text"], r"\bversion\b")
    version_widths.setdefault(w, []).append(s)

version_width = None
if not version_widths:
    fail(
        1,
        "the version field width is not stated (expected 'the version field is "
        "a u16 / 2-byte field ...')",
    )
    ac1 = False
elif len(version_widths) > 1:
    fail(
        1,
        "the version field width is stated inconsistently: "
        + ", ".join(f"{w} bits" for w in sorted(version_widths)),
    )
    ac1 = False
else:
    version_width = next(iter(version_widths))
    if version_width % 8 != 0 or version_width < 8:
        fail(1, f"the version width {version_width} bits is not a whole number of bytes >= 1")
        ac1 = False
        version_width = None

# The major/minor scheme (or an explicitly named alternative scheme).
layout = None
if version_width is not None:
    for s in stmts:
        text = s["text"]
        if not re.search(r"\bmajor\b", text, re.I) or not re.search(r"\bminor\b", text, re.I):
            continue
        if is_retired(text):
            continue
        major_bits = minor_bits = None
        mj = re.search(
            r"\bmajor\b[^.;]{0,80}?bits?\s*(\d+)\s*(?:\.\.|…|to|through|[-–—])\s*(\d+)",
            text,
            re.I,
        )
        mn = re.search(
            r"\bminor\b[^.;]{0,80}?bits?\s*(\d+)\s*(?:\.\.|…|to|through|[-–—])\s*(\d+)",
            text,
            re.I,
        )
        if mj and mn:
            major_bits = abs(int(mj.group(1)) - int(mj.group(2))) + 1
            minor_bits = abs(int(mn.group(1)) - int(mn.group(2))) + 1
        else:
            high = re.search(
                r"\bmajor\b[^.;]{0,80}?(?:high|upper|most[-\s]significant)\s*(\d+)?\s*(?:bits?|bytes?)",
                text,
                re.I,
            )
            low = re.search(
                r"\bminor\b[^.;]{0,80}?(?:low|lower|least[-\s]significant)\s*(\d+)?\s*(?:bits?|bytes?)",
                text,
                re.I,
            )
            if high and low:
                major_bits = int(high.group(1)) if high.group(1) else None
                minor_bits = int(low.group(1)) if low.group(1) else None
                if "byte" in high.group(0).lower() and major_bits is None:
                    major_bits = 8
                if "byte" in low.group(0).lower() and minor_bits is None:
                    minor_bits = 8
        if major_bits is None and minor_bits is None:
            mjv = re.search(r"\bmajor\s*(?:=|:|is)\s*(\d+)", text, re.I)
            mnv = re.search(r"\bminor\s*(?:=|:|is)\s*(\d+)", text, re.I)
            if mjv and mnv:
                # Explicit values state the scheme; assume the natural half
                # split for the packed-value agreement below.
                major_bits = minor_bits = version_width // 2
        if major_bits is None and minor_bits is None:
            continue
        if major_bits is None:
            major_bits = version_width - minor_bits
        if minor_bits is None:
            minor_bits = version_width - major_bits
        if major_bits <= 0 or minor_bits <= 0 or major_bits + minor_bits > version_width:
            fail(
                1,
                f"the major/minor split ({major_bits}+{minor_bits} bits) does not fit "
                f"the {version_width}-bit version field: {text[:100]!r}",
            )
            ac1 = False
            layout = None
            break
        layout = {
            "major_bits": major_bits,
            "minor_bits": minor_bits,
            "block": s["block"],
            "text": text,
        }
        break
if version_width is not None and layout is None:
    fail(
        1,
        "the section does not state how the version field splits into major and "
        "minor (bit ranges, high/low halves, or explicit major/minor values)",
    )
    ac1 = False

# The v1 version value.
v1_candidates = []
for s in stmts:
    text = s["text"]
    if not re.search(r"\bv1\b", text, re.I):
        continue
    if is_retired(text):
        continue
    raw = None
    pair = None
    for clause in clauses(text):
        if not re.search(r"\bv1\b", clause, re.I):
            continue
        # major.minor pair
        pm = re.search(r"(?<![\d.])(\d+)\s*\.\s*(\d+)(?![\d.])", clause)
        if pm:
            pair = (int(pm.group(1)), int(pm.group(2)))
        # explicit major/minor assignments
        if pair is None:
            mjv = re.search(r"\bmajor\s*(?:=|:|is)?\s*(\d+)", clause, re.I)
            mnv = re.search(r"\bminor\s*(?:=|:|is)?\s*(\d+)", clause, re.I)
            if mjv and mnv:
                pair = (int(mjv.group(1)), int(mnv.group(1)))
        # raw value: hex, or a decimal after a value cue
        hx = hex_candidates(clause)
        if hx:
            raw = hx[0][0]
        else:
            vm = re.search(
                r"(?:\bvalue\b|\bencoded\b|\bnumber\b)[^.;]{0,30}?"
                r"(?:=|:|is|of)?\s*(?<![\w.])(\d+)(?![\d.]|\.\d)",
                clause,
                re.I,
            )
            if not vm:
                vm = re.search(
                    r"\bv1\b[^.;]{0,50}?(?:=|:|is)\s*(?<![\w.])(\d+)(?![\d.]|\.\d)",
                    clause,
                    re.I,
                )
            if vm:
                raw = int(vm.group(1))
    if raw is not None or pair is not None:
        v1_candidates.append({"raw": raw, "pair": pair, "stmt": s})

v1_raw = None
v1_pair = None
if not v1_candidates:
    fail(
        1,
        "the v1 version value is not stated (expected 'v1 is 1.0 = 0x0100' or "
        "'v1 = 1', with the major/minor values)",
    )
    ac1 = False
else:
    raws = {c["raw"] for c in v1_candidates if c["raw"] is not None}
    pairs = {c["pair"] for c in v1_candidates if c["pair"] is not None}
    if len(raws) > 1 or len(pairs) > 1:
        fail(
            1,
            "the v1 version value is stated inconsistently: raw="
            f"{sorted(raws)}, major.minor={sorted(pairs)}",
        )
        ac1 = False
    else:
        v1_raw = next(iter(raws)) if raws else None
        v1_pair = next(iter(pairs)) if pairs else None
        # If both forms appear, they must agree under the stated split.
        if v1_raw is not None and v1_pair is not None and layout is not None:
            expected = (v1_pair[0] << layout["minor_bits"]) | v1_pair[1]
            if v1_raw != expected:
                fail(
                    1,
                    f"the v1 raw value {v1_raw} (0x{v1_raw:x}) disagrees with "
                    f"major.minor {v1_pair[0]}.{v1_pair[1]} packed as "
                    f"0x{expected:x} with a {layout['minor_bits']}-bit minor",
                )
                ac1 = False
        if v1_pair is not None:
            limit = (1 << layout["major_bits"]) if layout else 1 << 16
            if v1_pair[0] >= limit:
                fail(
                    1,
                    f"the v1 major {v1_pair[0]} does not fit the stated "
                    f"{layout['major_bits'] if layout else '?'}-bit major field",
                )
                ac1 = False
        if ac1:
            shown = (
                f"{v1_pair[0]}.{v1_pair[1]}"
                if v1_pair is not None
                else f"0x{v1_raw:x}"
            )
            print(f"PASS AC1 (version): {version_width}-bit version field, v1 = {shown}")

# The ticket's falsifier: either stated explicitly, or guaranteed by the magic
# differing from the POC and the major being a distinct part of the version.
falsifier_stated = any(
    re.search(r"\b(?:tell|distinguish|differentiate|identify|separate)\w*\b", s["text"], re.I)
    and re.search(r"\bPOC\b", s["text"], re.I)
    and re.search(r"\bbefore\b", s["text"], re.I)
    and re.search(r"\b(?:field|byte|octet|reading|parse)\w*\b", s["text"], re.I)
    for s in stmts
)
if ac1:
    if falsifier_stated:
        print("PASS AC1 (falsifier): the section states the before-any-other-field property")
    elif magic_value is not None and layout is not None and (v1_pair is not None or v1_raw is not None):
        print(
            "PASS AC1 (falsifier): magic differs from 0x4844 and the version "
            "carries a distinct major, so a v1 receiver separates a POC packet "
            "and a future major from a v1 packet on the magic and version alone"
        )
    else:
        fail(
            1,
            "the ticket's falsifier is not satisfied: state that a v1 receiver "
            "can tell a POC packet or a future major version from a v1 packet "
            "before reading any other field, or state a magic distinct from "
            "0x4844 and a version with a distinct major",
        )
        ac1 = False

# ---------------------------------------------------------------------------
# AC 2 — minor and major change definitions, >= 3 examples each, regime stamp
# ---------------------------------------------------------------------------
ac2 = True


def split_at_headings(text):
    lines = text.splitlines()
    idx = [i for i, line in enumerate(lines) if re.match(r"^\s*#{1,6}\s+", line)]
    if not idx:
        return [{"title": None, "text": text}]
    out = []
    if idx[0] > 0:
        out.append({"title": None, "text": "\n".join(lines[: idx[0]])})
    for k, i in enumerate(idx):
        end = idx[k + 1] if k + 1 < len(idx) else len(lines)
        out.append({"title": lines[i].strip().lstrip("#").strip(), "text": "\n".join(lines[i:end])})
    return out


def examples_in_text(text, term_re, other_re):
    """Examples in a sub-block, attributed to the given term."""
    examples = []
    marks = sorted(
        [(m.start(), True) for m in re.finditer(term_re, text, re.I)]
        + [(m.start(), False) for m in re.finditer(other_re, text, re.I)]
    )
    for k, (start, mine) in enumerate(marks):
        if not mine:
            continue
        end = marks[k + 1][0] if k + 1 < len(marks) else len(text)
        segment = text[start:end]
        found_structured = False
        for line in segment.splitlines():
            if re.match(r"^\s*#{1,6}\s+", line):
                continue
            m = re.match(r"^\s*(?:[-*+]|\d+[.)])\s+(.+)$", line)
            if m and m.group(1).strip():
                examples.append(m.group(1).strip())
                found_structured = True
        for _, _, rows in parse_tables(segment):
            for row in rows:
                txt = " | ".join(c for c in row if c.strip()).strip()
                if txt:
                    examples.append(txt)
                    found_structured = True
        if found_structured:
            continue
        body = re.sub(r"^\s*#{1,6}\s+.*$", "", segment, flags=re.M)
        # Stay inside the paragraph that carries the example cue: the next
        # paragraph is a different topic, not more examples.
        for paragraph in re.split(r"\n\s*\n", body):
            cue = re.search(
                r"\b(?:adds?|introduces?|includes?|extends?|covers?|for\s+example|"
                r"e\.g\.|such\s+as|examples?\s+(?:are|include))\b(.*)",
                paragraph,
                re.I | re.S,
            )
            if not cue:
                continue
            tail = cue.group(1)
            fragments = re.split(r";|,|\bor\b|\band\b", tail)
            for fragment in fragments:
                fragment = re.sub(r"[`*_]", "", fragment).strip(" .:—-")
                fragment = re.sub(r"\s+", " ", fragment)
                if len(fragment.split()) >= 2:
                    examples.append(fragment)
    # De-duplicate while preserving order.
    seen = set()
    unique = []
    for e in examples:
        key = re.sub(r"[^a-z0-9 ]", "", e.lower())
        if key and key not in seen:
            seen.add(key)
            unique.append(e)
    return unique


sub_blocks = split_at_headings(versioning)
minor_examples = []
major_examples = []
for sub in sub_blocks:
    minor_examples.extend(examples_in_text(sub["text"], r"\bminor\s+changes?\b", r"\bmajor\s+changes?\b"))
    major_examples.extend(examples_in_text(sub["text"], r"\bmajor\s+changes?\b", r"\bminor\s+changes?\b"))

# De-duplicate across sub-blocks.
def dedupe(items):
    seen = set()
    out = []
    for e in items:
        key = re.sub(r"[^a-z0-9 ]", "", e.lower())
        if key and key not in seen:
            seen.add(key)
            out.append(e)
    return out


minor_examples = dedupe(minor_examples)
major_examples = dedupe(major_examples)

minor_def = re.search(
    r"\bminor\s+changes?\b[^.]{0,240}?\b(?:is|are|means|adds?|introduces?|extends?|"
    r"never\s+changes?|does\s+not\s+change)\b",
    versioning,
    re.I | re.S,
)
major_def = re.search(
    r"\bmajor\s+changes?\b[^.]{0,240}?\b(?:is|are|means|changes?|alters?|breaks?|"
    r"modifies?|any\s+change)\b",
    versioning,
    re.I | re.S,
)
if not minor_def:
    fail(2, "the section does not define a minor change (a sentence 'a minor change is/adds ...')")
    ac2 = False
if not major_def:
    fail(
        2,
        "the section does not define a major change (a sentence 'a major change "
        "is/changes ...')",
    )
    ac2 = False
if len(minor_examples) < 3:
    fail(
        2,
        f"the minor change definition has {len(minor_examples)} example(s); at least 3 "
        "are required (bullets, table rows, or an inline adds/for-example list)",
    )
    ac2 = False
if len(major_examples) < 3:
    fail(
        2,
        f"the major change definition has {len(major_examples)} example(s); at least 3 "
        "are required",
    )
    ac2 = False
regime_minor = [
    e
    for e in minor_examples
    if re.search(r"regime[-\s]*stamp", e, re.I)
    and re.search(r"\bsection\b", e, re.I)
    and re.search(r"\btype\b|\bkind\b", e, re.I)
]
if not regime_minor:
    fail(
        2,
        "the minor change examples do not list 'a new regime stamp section type' "
        "(the ticket fixes that as minor, C-18)",
    )
    ac2 = False
if ac2:
    print(
        f"PASS AC2: minor and major changes defined, {len(minor_examples)} minor and "
        f"{len(major_examples)} major examples, regime stamp section type is minor"
    )

# ---------------------------------------------------------------------------
# AC 3 — receiver behaviour for the 5 cases
# ---------------------------------------------------------------------------
ac3 = True

CASES = {
    "same version": r"\bsame\s+version\b",
    "same major, higher minor": (
        r"\bhigher\s+minor\b|\bminor\s+(?:number\s+)?(?:is\s+)?higher\b|"
        r"\bnewer\s+minor\b"
    ),
    "same major, lower minor": (
        r"\blower\s+minor\b|\bminor\s+(?:number\s+)?(?:is\s+)?lower\b|"
        r"\bolder\s+minor\b"
    ),
    "different major": (
        r"\bdifferent\s+major\b|\bother\s+major\b|\banother\s+major\b|"
        r"\bmajor\s+(?:number\s+)?(?:is\s+)?different\b|\bfuture\s+major\b|"
        r"\bnewer\s+major\b"
    ),
    "bad magic": (
        r"\bbad\s+magic\b|\bwrong\s+magic\b|"
        r"\bmagic\s+(?:does\s+not\s+match|mismatch|is\s+wrong|is\s+not\s+the)\b"
    ),
}

ACCEPT = re.compile(
    r"\baccept(?:s|ed|ing)?\b|\bparse(?:s|d)?\s+(?:it\s+)?normally\b|"
    r"\bprocess(?:es|ed)?\s+(?:it\s+)?normally\b",
    re.I,
)
DROP = re.compile(r"\bdrop(?:s|ped|ping)?\b|\bdiscard(?:s|ed)?\b|\breject(?:s|ed|ing)?\b", re.I)
COUNT = re.compile(r"\bcount(?:s|ed|ing)?\b|\bincrement(?:s|ed|ing)?\b", re.I)
EVENT_NOUN = re.compile(r"\bevents?\b", re.I)
EVENT_RAISE = re.compile(
    r"(?<!not\s)(?<!never\s)(?<!no\s)\b(?:raise|raises|raised|raising|emit|emits|"
    r"emitted|signal|signals|signalled|signaled)\b",
    re.I,
)
NO_EVENT = re.compile(r"\bno\s+event\b|\bwithout\s+(?:an?\s+)?event\b|\bnever\s+raise\w*\b", re.I)

CASE_RE = {name: re.compile(pattern, re.I) for name, pattern in CASES.items()}
observed = {name: {"accept": False, "drop": False, "count": False, "event": False,
                   "no_event": False, "reasons": set(), "found": False}
            for name in CASES}


def assign(pos, spans):
    if not spans:
        return None
    best = None
    for start, end, case in spans:
        if start <= pos < end:
            return case
        distance = min(abs(pos - start), abs(pos - end))
        if best is None or distance < best[0]:
            best = (distance, case)
    return best[1]


OUTCOME_TOKENS = (
    ("accept", ACCEPT),
    ("drop", DROP),
    ("count", COUNT),
    ("event", EVENT_RAISE),
    ("no_event", NO_EVENT),
)

for statement in stmts:
    text = statement["text"]
    spans = []
    for case, regex in CASE_RE.items():
        for m in regex.finditer(text):
            spans.append((m.start(), m.end(), case))
    if not spans:
        continue
    spans.sort()
    for case in CASES:
        if any(c == case for _, _, c in spans):
            observed[case]["found"] = True

    for clause_start, clause_end in clause_spans(text):
        local = []
        for case, regex in CASE_RE.items():
            for m in regex.finditer(text, clause_start, clause_end):
                local.append((m.start(), m.end(), case))
        local.sort()
        target_spans = local or spans
        for field, regex in OUTCOME_TOKENS:
            for m in regex.finditer(text, clause_start, clause_end):
                case = assign(m.start(), target_spans)
                if case:
                    observed[case][field] = True
        for m in re.finditer(r"\b[A-Z][A-Za-z0-9]*\b", text[clause_start:clause_end]):
            name = m.group(0)
            if name in enum_names:
                case = assign(clause_start + m.start(), target_spans)
                if case:
                    observed[case]["reasons"].add(name)

# Rule 12: VersionMismatch is the only reason that raises an event.
for case, data in observed.items():
    if data["no_event"]:
        data["event"] = False
    if data["event"] and "VersionMismatch" not in data["reasons"]:
        fail(
            3,
            f"the {case} case raises an event but does not name VersionMismatch; "
            "VersionMismatch is the only rejection reason that raises a named "
            "event (rule 12)",
        )
        ac3 = False

for case, data in observed.items():
    if not data["found"]:
        fail(3, f"the receiver behaviour for '{case}' is not specified")
        ac3 = False
        continue
    if data["accept"] and data["drop"]:
        fail(3, f"the '{case}' case has an ambiguous outcome (both accept and drop)")
        ac3 = False
        continue
    if data["accept"] and not data["drop"]:
        if case in ("same version", "same major, higher minor"):
            continue
        if case in ("same major, lower minor",):
            continue
        if case == "bad magic":
            fail(3, "bad magic cannot be accepted; it is a rejection")
            ac3 = False
        elif case == "different major":
            fail(3, "a different major must be dropped, not accepted")
            ac3 = False
        continue
    if data["drop"]:
        if not data["count"]:
            fail(3, f"the '{case}' case drops the packet but does not count it")
            ac3 = False
        if not data["reasons"]:
            fail(
                3,
                f"the '{case}' case names no rejection reason; name a member of "
                "the 004-03 Reject enumeration",
            )
            ac3 = False
        if case == "different major":
            if "VersionMismatch" not in data["reasons"]:
                fail(
                    3,
                    "the different-major case must drop and count under "
                    "VersionMismatch",
                )
                ac3 = False
            if not data["event"]:
                fail(
                    3,
                    "the different-major case must raise the named VersionMismatch "
                    "event (rule 12: never a silent drop)",
                )
                ac3 = False
        elif case == "same version":
            fail(3, "the same-version case drops the packet; it must be accepted")
            ac3 = False
        elif case == "same major, higher minor":
            fail(
                3,
                "the same-major/higher-minor case drops the packet; it must be "
                "accepted (unknown optional parts are skipped by their length)",
            )
            ac3 = False
        if "VersionMismatch" in data["reasons"] and not data["event"]:
            fail(
                3,
                f"the '{case}' case counts VersionMismatch but does not raise the "
                "named event; VersionMismatch is never a silent drop (rule 12)",
            )
            ac3 = False
        if case == "bad magic" and "VersionMismatch" in data["reasons"]:
            warn(3, "bad magic is counted as VersionMismatch; BadMagic is the intended reason")
        continue
    fail(3, f"the '{case}' case states no outcome (accept, drop+count, or named event)")
    ac3 = False

# Work step 4: higher minor accepts and skips unknown optional parts by length.
higher = observed["same major, higher minor"]
if higher["found"] and higher["accept"]:
    length_skip = any(
        re.search(r"\bhigher\s+minor\b", s["text"], re.I)
        and re.search(r"\blength\b", s["text"], re.I)
        and re.search(r"\bskip|ignore|unknown\b", s["text"], re.I)
        for s in stmts
    )
    if not length_skip:
        warn(
            3,
            "the higher-minor case does not say unknown optional parts are "
            "skipped by their length (Work step 4; not an acceptance criterion)",
        )

# The hello carries the version, so the event is raised before a connection.
hello_ok = any(
    re.search(r"\bhello\b", s["text"], re.I)
    and re.search(r"\bVersionMismatch\b", s["text"])
    and EVENT_RAISE.search(s["text"])
    and re.search(r"\bbefore\b", s["text"], re.I)
    and re.search(r"\bconnect(?:ion|ed|ing)?\b|\bconnection\b", s["text"], re.I)
    for s in stmts
)
if not hello_ok:
    fail(
        3,
        "the section does not state that the hello raises the VersionMismatch "
        "event before a connection exists (the version travels in the hello)",
    )
    ac3 = False
if ac3:
    print(
        "PASS AC3: the 5 receiver cases have outcomes, named reasons are members "
        "of the Reject enumeration, the different-major case raises the event, "
        "and the hello raises it before a connection"
    )

# ---------------------------------------------------------------------------
# AC 4 — fixed offsets and an explicit immovability statement naming them
# ---------------------------------------------------------------------------
ac4 = True
OFFSET_UNIT_BITS = re.compile(r"offset\s*\([^)]*bits?[^)]*\)|\bbit\s+offsets?\b|offset\s+in\s+bits", re.I)
OFFSET_UNIT_BYTES = re.compile(
    r"offset\s*\([^)]*bytes?[^)]*\)|\bbyte\s+offsets?\b|offset\s+in\s+bytes", re.I
)


def statement_offset_unit(text):
    if OFFSET_UNIT_BITS.search(text):
        return "bits"
    if OFFSET_UNIT_BYTES.search(text):
        return "bytes"
    return "bytes"


def offset_patterns(text, word_re):
    """(value, position) offsets attached to a field word in one statement."""
    out = []
    for m in re.finditer(
        rf"{word_re}[^.;]{{0,70}}?offset\s*(?:\([^)]*\))?\s*(?:=|:|is|of|at)?\s*(\d+)",
        text,
        re.I,
    ):
        out.append((int(m.group(1)), m.start()))
    for m in re.finditer(
        rf"{word_re}\s*(?:field\s*)?(?:is\s+)?(?:at|starts\s+at)\s+(?:byte\s+|bit\s+)?(\d+)",
        text,
        re.I,
    ):
        out.append((int(m.group(1)), m.start()))
    for m in re.finditer(
        rf"{word_re}[^.;]{{0,30}}?(?:byte|bit)\s*(\d+)\b", text, re.I
    ):
        out.append((int(m.group(1)), m.start()))
    return out


magic_offset = version_offset = None
for statement in stmts:
    text = statement["text"]
    if is_retired(text):
        continue
    has_magic = bool(re.search(r"\bmagic\b", text, re.I))
    has_version = bool(re.search(r"\bversion\b", text, re.I))
    if not has_magic and not has_version:
        continue
    unit = statement_offset_unit(text)
    # A single sentence naming both fields and both offsets.
    pair = re.search(
        r"magic[^.;]{0,60}?offset[s]?\s*(?:=|:|is|at)?\s*(\d+)[^.;]{0,100}?"
        r"version[^.;]{0,60}?offset[s]?\s*(?:=|:|is|at)?\s*(\d+)",
        text,
        re.I,
    )
    if pair:
        magic_offset = (int(pair.group(1)), unit)
        version_offset = (int(pair.group(2)), unit)
        continue
    pair = re.search(
        r"version[^.;]{0,60}?offset[s]?\s*(?:=|:|is|at)?\s*(\d+)[^.;]{0,100}?"
        r"magic[^.;]{0,60}?offset[s]?\s*(?:=|:|is|at)?\s*(\d+)",
        text,
        re.I,
    )
    if pair:
        version_offset = (int(pair.group(1)), unit)
        magic_offset = (int(pair.group(2)), unit)
        continue
    pair = re.search(
        r"magic\s+and\s+version\s+(?:are\s+)?(?:at|in)\s+offsets?\s+(\d+)\s+and\s+(\d+)",
        text,
        re.I,
    )
    if pair:
        magic_offset = (int(pair.group(1)), unit)
        version_offset = (int(pair.group(2)), unit)
        continue
    pair = re.search(
        r"offsets?\s+(\d+)\s+and\s+(\d+)\s+(?:hold|carry|contain)\s+(?:the\s+)?magic\s+and\s+version",
        text,
        re.I,
    )
    if pair:
        magic_offset = (int(pair.group(1)), unit)
        version_offset = (int(pair.group(2)), unit)
        continue
    if has_magic:
        found = offset_patterns(text, r"\bmagic\b")
        if found:
            magic_offset = (found[0][0], unit)
    if has_version:
        found = offset_patterns(text, r"\bversion\b")
        if found:
            version_offset = (found[0][0], unit)

if magic_offset is None:
    fail(4, "the magic's fixed offset is not stated")
    ac4 = False
elif magic_offset[0] != 0:
    fail(4, f"the magic is at offset {magic_offset[0]}; it must be at offset 0 (checked before any other field)")
    ac4 = False

if version_offset is None:
    fail(4, "the version's fixed offset is not stated")
    ac4 = False
elif magic_offset is not None and magic_width is not None:
    magic_bytes = magic_width // 8
    expected = magic_bytes * 8 if version_offset[1] == "bits" else magic_bytes
    if version_offset[0] != expected:
        fail(
            4,
            f"the version is at offset {version_offset[0]} ({version_offset[1]}); "
            f"the {magic_bytes}-byte magic puts it at offset {expected}",
        )
        ac4 = False

immovable = False
for statement in stmts:
    text = statement["text"]
    if is_retired(text):
        continue
    immovable_language = re.search(
        r"(?:\bnever\b|\bmay\s+not\b|\bmust\s+not\b|\bshall\s+not\b|\bcannot\b|"
        r"\bcan\s+never\b|\bno\s+future\b)[^.;]{0,100}?"
        r"\b(?:move|moved|moving|relocat\w*|shift|change)\b"
        r"|\bimmovab\w*\b|\bfixed\s+offsets?\b",
        text,
        re.I,
    )
    if not immovable_language:
        continue
    fields = bool(re.search(r"\bmagic\b", text, re.I)) and bool(re.search(r"\bversion\b", text, re.I))
    if not fields and not re.search(r"\bboth\b|\bthese\s+(?:two\s+)?fields\b", text, re.I):
        continue
    numbers = [int(m.group(1)) for m in re.finditer(r"(?<![\w.])(\d+)(?![\d.])", text)]
    if len(numbers) < 2:
        continue
    immovable = True
    break
if not immovable:
    fail(
        4,
        "no explicit immovability statement names the magic and version offsets "
        "and says no future version may move them (e.g. 'no future version may "
        "move the magic or the version at offsets 0 and 2')",
    )
    ac4 = False

checked_first = any(
    re.search(r"\bmagic\b", s["text"], re.I)
    and re.search(r"\bversion\b", s["text"], re.I)
    and re.search(r"\b(?:checked|read|parsed)\b", s["text"], re.I)
    and re.search(r"\bbefore\s+any\s+other\s+field\b", s["text"], re.I)
    for s in stmts
)
if not checked_first:
    warn(
        4,
        "the section does not say the magic and version are checked before any "
        "other field (Work step 5; not an acceptance criterion)",
    )

if ac4:
    print(
        "PASS AC4: magic at offset 0, version immediately after it, and an "
        "immovability statement naming the offsets"
    )

# ---------------------------------------------------------------------------
# AC 5 — magic and version in wire.toml, value agreement, wire-check, tests
# ---------------------------------------------------------------------------
ac5 = True


def parse_int_value(raw):
    if raw is None:
        return None
    raw = raw.strip().strip("\"'")
    m = re.fullmatch(r"(0[xX][0-9a-fA-F_]+|0[bB][01_]+|0[oO][0-7_]+|\d[\d_]*)", raw)
    if not m:
        return None
    token = m.group(1).replace("_", "")
    if token.lower().startswith("0x"):
        return int(token, 16)
    if token.lower().startswith("0b"):
        return int(token, 2)
    if token.lower().startswith("0o"):
        return int(token, 8)
    return int(token, 10)


def parse_version_value(raw):
    """Return ('raw', n) or ('pair', major, minor) or None."""
    if raw is None:
        return None
    raw = raw.strip().strip("\"'")
    pair = re.fullmatch(r"(\d+)\s*\.\s*(\d+)", raw)
    if pair:
        return ("pair", int(pair.group(1)), int(pair.group(2)))
    value = parse_int_value(raw)
    if value is not None:
        return ("raw", value)
    return None


MAGIC_KEYS = {
    "magic", "magicvalue", "magicnumber", "protocolmagic", "wiremagic",
    "headermagic", "magicconstant",
}
VERSION_KEYS = {
    "version", "versionvalue", "versionnumber", "protocolversion", "wireversion",
    "header version", "versionconstant", "v1",
}


def is_magic_key(norm):
    # Exact names only: a Reject variant named "BadMagic" is not the wire magic.
    return norm in MAGIC_KEYS


def is_version_key(norm):
    return norm in VERSION_KEYS


def unquote(raw):
    return raw.strip().strip("\"'")


magic_entries = []
version_entries = []


def record_magic(raw, where):
    parsed = parse_int_value(raw)
    magic_entries.append({"value": parsed, "raw": raw, "where": where})


def record_version(raw, where):
    parsed = parse_version_value(raw)
    version_entries.append({"value": parsed, "raw": raw, "where": where})


for header, kv in toml_blocks_parsed:
    hnorm = norm_key(header or "")
    name = unquote(kv["name"]) if "name" in kv else None
    name_norm = norm_key(name) if name else None

    for key, raw in kv.items():
        kn = norm_key(key)
        if is_magic_key(kn):
            record_magic(raw, f"{header or 'top level'}.{key}")
        elif is_version_key(kn):
            record_version(raw, f"{header or 'top level'}.{key}")

    if name_norm and is_magic_key(name_norm):
        for key in ("value", "code", "id", "number"):
            if key in kv:
                record_magic(kv[key], f"{header}.{key} (name = {name})")
                break
        else:
            magic_entries.append({"value": None, "raw": None, "where": f"{header} name = {name}"})
    if name_norm and is_version_key(name_norm):
        for key in ("value", "code", "id", "number", "encoded"):
            if key in kv:
                record_version(kv[key], f"{header}.{key} (name = {name})")
                break
        else:
            # A definition-only entry (offset/width) still carries the field.
            version_entries.append({"value": None, "raw": None, "where": f"{header} name = {name}"})

    # A [versioning] / [version] table may carry the version as major + minor,
    # and a [magic] table may carry it as value.
    if hnorm in ("version", "versioning", "wireversion", "protocolversion"):
        if "value" in kv and not any(is_version_key(norm_key(k)) for k in kv):
            record_version(kv["value"], f"{header}.value")
    if hnorm in ("magic", "wiremagic", "protocolmagic"):
        if "value" in kv and not any(is_magic_key(norm_key(k)) for k in kv):
            record_magic(kv["value"], f"{header}.value")
    if "major" in kv and "minor" in kv:
        major = parse_int_value(kv["major"])
        minor = parse_int_value(kv["minor"])
        if major is not None and minor is not None:
            version_entries.append(
                {
                    "value": ("pair", major, minor),
                    "raw": f"{kv['major']}.{kv['minor']}",
                    "where": f"{header or 'top level'} major/minor",
                }
            )

if not magic_entries:
    fail(
        5,
        f"{tables_path} carries no magic entry (a [versioning] table, a direct "
        "magic key, a [[constant]] named magic, or a field named magic)",
    )
    ac5 = False
if not version_entries:
    fail(
        5,
        f"{tables_path} carries no version entry (a [versioning] table, a direct "
        "version key, a [[constant]] named version, or a field named version)",
    )
    ac5 = False

# Where both files state a value, they must agree.
if magic_entries and magic_value is not None:
    values = {e["value"] for e in magic_entries if e["value"] is not None}
    if len(values) > 1:
        fail(5, f"{tables_path} states more than one magic value: {sorted(values)}")
        ac5 = False
    elif values:
        toml_magic = next(iter(values))
        if toml_magic != magic_value:
            fail(
                5,
                f"magic disagreement: {wire_path} says 0x{magic_value:x}, "
                f"{tables_path} says 0x{toml_magic:x}",
            )
            ac5 = False

if version_entries and (v1_raw is not None or v1_pair is not None):
    raw_values = set()
    pair_values = set()
    for entry in version_entries:
        value = entry["value"]
        if value is None:
            continue
        if value[0] == "raw":
            raw_values.add(value[1])
        else:
            pair_values.add((value[1], value[2]))
    if len(raw_values) > 1 or len(pair_values) > 1:
        fail(
            5,
            f"{tables_path} states the version inconsistently: raw={sorted(raw_values)}, "
            f"major.minor={sorted(pair_values)}",
        )
        ac5 = False
    else:
        toml_raw = next(iter(raw_values)) if raw_values else None
        toml_pair = next(iter(pair_values)) if pair_values else None
        if toml_pair is not None and v1_pair is not None and toml_pair != v1_pair:
            fail(
                5,
                f"version disagreement: {wire_path} says v1 = "
                f"{v1_pair[0]}.{v1_pair[1]}, {tables_path} says "
                f"{toml_pair[0]}.{toml_pair[1]}",
            )
            ac5 = False
        if toml_raw is not None:
            expected = None
            if v1_raw is not None:
                expected = v1_raw
            elif v1_pair is not None:
                shift = layout["minor_bits"] if layout else (version_width or 16) // 2
                expected = (v1_pair[0] << shift) | v1_pair[1]
            if expected is not None and toml_raw != expected:
                fail(
                    5,
                    f"version disagreement: {wire_path} says v1 packs to "
                    f"0x{expected:x}, {tables_path} says 0x{toml_raw:x}",
                )
                ac5 = False

if overridden:
    print("NOTE AC5: document paths overridden; repository-level checks skipped")
    if ac5:
        print("PASS AC5: wire.toml carries the magic and version entries and agrees with wire.md (repository checks skipped)")
else:
    # 1. The 004-02 checker over the real file.
    try:
        proc = subprocess.run(
            ["cargo", "run", "--quiet", "--bin", "wire-check"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail(5, f"could not run cargo run --bin wire-check: {error}")
        ac5 = False
    if proc is not None and proc.returncode != 0:
        out = proc.stdout + proc.stderr
        tail = "\n".join(out.strip().splitlines()[-12:])
        fail(5, f"cargo run --bin wire-check failed:\n{tail}")
        ac5 = False

    # 2. The wire_tables tests stay 12 passed, 0 failed, 0 ignored.
    try:
        proc_tests = subprocess.run(
            ["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
            capture_output=True,
            text=True,
            timeout=900,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc_tests = None
        fail(5, f"could not run the wire_tables tests: {error}")
        ac5 = False
    if proc_tests is not None:
        out = proc_tests.stdout + proc_tests.stderr
        if proc_tests.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail(5, f"cargo test -p hadronyche-tools --test wire_tables failed:\n{tail}")
            ac5 = False
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if result is None:
            fail(5, "the wire_tables test run printed no 'test result:' line")
            ac5 = False
        else:
            m = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result)
            if not m:
                fail(5, f"could not read the test counts from: {result}")
                ac5 = False
            elif (int(m.group(1)), int(m.group(2)), int(m.group(3))) != (12, 0, 0):
                fail(
                    5,
                    "wire_tables must stay at 12 passed, 0 failed, 0 ignored; "
                    f"got: {result}",
                )
                ac5 = False
    if ac5:
        print(
            "PASS AC5: wire.toml carries the magic and version entries, agrees "
            "with wire.md, wire-check passes and wire_tables reports 12/0/0"
        )

# ---------------------------------------------------------------------------
# Definition of done — evidence tags, [decision] on the choices, open items
# ---------------------------------------------------------------------------
dod = True
prev_tagged = True
for block in blocks:
    content_lines = [
        line for line in block["text"].splitlines() if line.strip() and not line.lstrip().startswith("#")
    ]
    if not content_lines:
        prev_tagged = True
        continue
    body = "\n".join(content_lines).strip()
    if has_evidence(body):
        prev_tagged = True
        continue
    list_like = all(
        re.match(r"^\s*(?:[-*+]|\d+[.)])\s+\S", line) for line in content_lines if line.strip()
    )
    if block["kind"] in ("table", "code") and prev_tagged:
        prev_tagged = True
        continue
    if list_like and prev_tagged:
        # A bullet list may inherit the tag of the paragraph that introduces it.
        prev_tagged = True
        continue
    fail(
        -1,
        "this block of the Versioning section carries no evidence tag "
        "([code] [measured] [kernel-doc] [standard] [memory] [decision]): "
        + (body.splitlines()[0][:90] + (" …" if len(body.splitlines()[0]) > 90 else "")),
    )
    dod = False
    prev_tagged = False


def decisions_near(block_index):
    if block_index is None:
        return False
    candidates = [block_index]
    if block_index > 0:
        candidates.append(block_index - 1)
    return any(0 <= j < len(blocks) and DECISION.search(blocks[j]["text"]) for j in candidates)


decision_checks = []

# magic choice
if magic_choices:
    decision_checks.append(("magic value and width", magic_choices[0]["stmt"]["block"]))
# version width / scheme
if version_statements:
    decision_checks.append(("version field width and scheme", version_statements[0]["block"]))
# v1 value
if v1_candidates:
    decision_checks.append(("v1 version value", v1_candidates[0]["stmt"]["block"]))
# minor definition
minor_stmt = next((s for s in stmts if not s.get("heading") and re.search(r"\bminor\s+changes?\b", s["text"], re.I)), None)
if minor_stmt:
    decision_checks.append(("minor change definition", minor_stmt["block"]))
# major definition
major_stmt = next((s for s in stmts if not s.get("heading") and re.search(r"\bmajor\s+changes?\b", s["text"], re.I)), None)
if major_stmt:
    decision_checks.append(("major change definition", major_stmt["block"]))
# receiver behaviour
receiver_stmt = next(
    (s for s in stmts if any(regex.search(s["text"]) for regex in CASE_RE.values())),
    None,
)
if receiver_stmt:
    decision_checks.append(("receiver behaviour", receiver_stmt["block"]))
# immovability
if immovable:
    imm_stmt = next(
        (
            s
            for s in stmts
            if re.search(r"\bfixed\s+offsets?\b|\bnever\b|\bmay\s+not\b|\bimmovab\w*\b", s["text"], re.I)
            and re.search(r"\bmagic\b", s["text"], re.I)
            and re.search(r"\bversion\b", s["text"], re.I)
        ),
        None,
    )
    if imm_stmt:
        decision_checks.append(("fixed offsets", imm_stmt["block"]))

for label, block_index in decision_checks:
    if not decisions_near(block_index):
        fail(-1, f"the decision on {label} carries no [decision] tag")
        dod = False

# An open item must name an owner and a falsifier.
for block in blocks:
    text = block["text"]
    if re.search(r"\bopen\s+item\b|\bTBD\b|\bto\s+be\s+decided\b|\bunresolved\b|\bTODO\b", text, re.I):
        if not re.search(r"\bowner\b", text, re.I) or not re.search(r"\bfalsifier\b", text, re.I):
            fail(
                -1,
                "the section introduces an open item without an owner and a "
                "falsifier (DoD: no new open item without both)",
            )
            dod = False

# The design checker must print OK.
if overridden:
    print("NOTE DoD: document paths overridden; design checker skipped")
elif dod:
    try:
        proc_design = subprocess.run(
            ["python3", "doc/kickoff/check-design.py", "doc/design"],
            capture_output=True,
            text=True,
            timeout=600,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        proc_design = None
        fail(-1, f"could not run the design checker: {error}")
        dod = False
    if proc_design is not None:
        out = proc_design.stdout + proc_design.stderr
        if proc_design.returncode != 0 or not re.search(r"(?m)^OK\b", out):
            tail = "\n".join(out.strip().splitlines()[-8:])
            fail(-1, f"the design checker did not print OK:\n{tail}")
            dod = False
        elif dod:
            print("PASS DoD: evidence tags, [decision] tags, open items, design checker OK")
elif not overridden:
    print("NOTE DoD: design checker skipped because the DoD checks already failed")

# ---------------------------------------------------------------------------
for ac, msg in warns:
    print(f"NOTE AC{ac}: {msg}")

if fails:
    for ac, msg in fails:
        label = "DoD" if ac == -1 else f"AC{ac}"
        print(f"FAIL {label}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 5/5 acceptance criteria for 004-04")
PY
