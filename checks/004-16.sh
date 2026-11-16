#!/usr/bin/env bash
# 004-16 — hello message layout and the 32-member size bound.
#
# RED-phase document check. It validates the "Hello" section of
# doc/design/spec/wire.md and the hello message in doc/design/spec/wire.toml
# against every acceptance criterion of the ticket, and re-runs the wire-check
# binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either path is overridden the
# cargo checks are skipped with a NOTE, because they always target the real
# repository.
#
# The section is located by its level-2 heading "Hello"; its subsections are
# located by heading text. Numbers are read from doc/design/02-constants.md at
# run time (C-08, C-09, C-08 members, C-10, C-11, C-12, C-13, C-17, C-30, C-34),
# never copied into this file.
#
# The author contract (headings and table labels the check locates):
#
#   ### Common control header   a five-column field table (Offset (bytes) |
#       Width (bytes) | Type | Meaning | Valid range) of seven rows, in this
#       order: magic, protocol version, domain id, message type, authentication
#       scheme, message counter, authentication tag; the paragraph states the
#       header's size in bytes (32) and says 004-17, 004-18, 004-24 and 004-27
#       reuse it. Each Meaning cell begins with the field's name.
#   ## Hello body     the prose: fixed facts (C-30 period, C-100 IPv4 unicast and
#       multicast TTL 1, C-13 CS5 (40), binary not text ([code] pointer to the
#       retired POC gossip), C-34 64 peers, C-18 stamp in its own message) and
#       the sentence "The maximum hello size is N bytes" and the sentence
#       "... 32-member bundle ... N bytes", a paragraph starting "Verdict".
#   ### Hello layout   a field table whose first seven rows are the common
#       control header rows (same offsets, widths, types), then the hello rows,
#       each Meaning beginning with its content name (Module id, Instance nonce,
#       Name length, Name, Capability bits, Port count, Port, Election epoch,
#       Believed holder, Timeline epoch, Transport role, Alarm, Regime stamp
#       version number, and, when carried, Bundle count and Bundle). A counted
#       row has Width "E × max M" (E bytes per element, M the maximum count).
#   ### Bundle element   a field table of one bundle: group id, member count,
#       member list (stream numbers, "× max 32"), channel map ("× max 32").
#   ### Port element    a field table of one port: port id, content-type count,
#       content types ("× max T").
#   ### Alarm field     a table with one row per connection alarm of C-173
#       (refusal, interrupted, dead, demoted, cannot-meet-A, control-degraded),
#       each with a "bit N" or value cell; the six values are distinct.
#   ### Transport role a table of the proposed values, the word "proposal", and
#       an 'Open item' paragraph naming epic 005 as owner and the falsifier
#       "epic 005 needs a role the encoding cannot express".
#   ### Validity rules an invalid-form table with Form and Outcome columns, at
#       least ten rows; every outcome names a member of the closed enumeration
#       of wire.md's Rejection reasons (or argues a non-rejection) and the exact
#       reasons of the ticket's list are used.
#   wire.toml         one [[message]] named "hello": layer "control", dscp 40,
#       max_size equal to the markdown sum, one [[message.field]] per layout row
#       (offset and width in bits, max_count and count for counted rows),
#       rejection_reasons listing every reason the invalid-form rows name.
#
# What is checked, per acceptance criterion:
#   AC 1  every hello content has a row (name, width in bytes from the constants
#         where they fix it: module id 8, nonce 8, domain 2, timeline epoch 2,
#         scheme 1, counter 8, tag 16); capability bit count and holder-eligible bit;
#         port and content-type maxima; bundle group id, member list and channel
#         map; the liveness counter is the message counter (sentence); the regime
#         stamp version number row.
#   AC 2  the maximum hello size in bytes equals the sum of the layout rows (counted
#         rows at max count); the 32-member size equals the sum with one bundle
#         element at 32 members for every bundle (B from the bundle row or the
#         stated "at most N bundles"); members per bundle equal C-08's 32; a verdict
#         that fits requires the computed maximum at or below C-17's 1 252 bytes,
#         and a verdict that is falsified requires it above, with a fallback
#         (split or handshake) and a fallback layout at or below the bound.
#   AC 3  the six alarms of C-173, one bit or value each, distinct; the transport
#         role is a proposal with epic 005 as the confirming owner and its falsifier;
#         Work 3: the regime stamp version number's stale rule (equality, serial
#         comparison or wrap); C-18: its own message; behaviour restated as fact.
#   AC 4  at least ten invalid forms; the exact reasons of the ticket's list
#         (TooShort, BadMagic, VersionMismatch with a named event, ForeignDomain,
#         UnconfiguredDomain for domain 0, OversizeControl, BadLength for counts and
#         length, ReservedNonZero); the name and the counter rows use a member
#         reason or an argued non-rejection; the module-id collision is not a parse
#         rejection; no reason is invented.
#   AC 5  the common control header is defined once, sums to its stated size, the
#         hello's own table starts with it (same offsets, widths and types), and its
#         field order equals the landed sync header's order; a difference is excused only
#         by a paragraph in its subsection that says 'differs from the sync header' and
#         names the fields. The toml fields match the markdown rows by name as well.
#   AC 6  wire.toml hello: layer control, dscp 40 (C-13), max_size equal to the
#         markdown sum and at most C-17's bound, fields tile the layout one to one,
#         nested maxima stated, rejection_reasons complete; when the paths are not
#         overridden: `cargo run --bin wire-check` reports "checks clean" and
#         `cargo test -p hadronyche-tools --test wire_tables` stays 12 passed.
#   DoD   every prose paragraph of 5+ words in the section carries an evidence tag,
#         every table's subsection has one; any paragraph starting "Open item" names
#         an owner and a falsifier in the same paragraph; the regime stamp fields are
#         documented (version number) or stated not applicable.
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
# Constants: read from doc/design/02-constants.md (that file wins)
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


K = {
    "bound": const_int("C-17", r"at\s+most\s+(" + NUM + r")\s*bytes"),
    "members_max": const_int("C-08", r"at\s+most\s+(\d+)"),
    "streams_max": const_int("C-09", r"at\s+most\s+(\d+)"),
    "dscp_control": const_int("C-13", r"control\s+CS5\s*\((\d+)\)"),
    "module_bits": const_int("C-11", r"(\d+)\s*/\s*\d+\s*/\s*\d+\s*bits"),
    "nonce_bits": const_int("C-11", r"\d+\s*/\s*(\d+)\s*/\s*\d+\s*bits"),
    "domain_bits": const_int("C-11", r"\d+\s*/\s*\d+\s*/\s*(\d+)\s*bits"),
    "epoch_bits": const_int("C-10", r"\d+\s*/\s*\d+\s*/\s*(\d+)\s*bits"),
    "scheme_bits": const_int("C-12", r"scheme\s+(\d+)\s+bits"),
    "counter_bits": const_int("C-12", r"counter\s+(\d+)\s+bits"),
    "tag_bits": const_int("C-12", r"tag\s+(\d+)\s+bits"),
    "peers_max": const_int("C-34", r"at\s+most\s+(\d+)"),
    "hello_period_s": const_int("C-30", r"(\d+)\s*s\b"),
}
for key, value in K.items():
    if value is None:
        fail("DoD", f"constant for {key} could not be read from {CONSTANTS_PATH}")
if K["bound"] is None:
    K["bound"] = 0

# ---------------------------------------------------------------------------
# The closed enumeration, parsed from wire.md's "Rejection reasons" section
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
span = section_span(r"rejection[-\s]+reasons?")
if span is None:
    fail("AC4", "no 'Rejection reasons' section in wire.md: the closed enumeration cannot be read")
else:
    for _, _, rows in parse_tables("\n".join(lines[span[0]:span[1]])):
        for row in rows:
            if len(row) >= 2 and re.fullmatch(r"\d+", row[0].strip()):
                name = row[1].strip().strip("`*_ ")
                if re.fullmatch(r"[A-Z][A-Za-z0-9]+", name):
                    MEMBERS[name] = int(row[0].strip())
    if len(MEMBERS) != 13:
        fail("AC4", f"the closed enumeration parsed from wire.md has {len(MEMBERS)} names, not 13")
MEMBER_RE = re.compile(r"\b(" + "|".join(sorted(MEMBERS, key=len, reverse=True)) + r")\b") if MEMBERS else None
CAMEL = re.compile(r"\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b")

# ---------------------------------------------------------------------------
# The Hello section (located by its level-2 heading, never by position)
# ---------------------------------------------------------------------------
hello_span = section_span(r"^hello$", level=2)
if hello_span is None:
    fail(1, "wire.md has no '## Hello' section")
    HELLO_LINES = []
else:
    HELLO_LINES = lines[hello_span[0]:hello_span[1]]
HELLO = "\n".join(HELLO_LINES)
HELLO_PLAIN = strip_fences(HELLO)
SUBS = subsections(HELLO_LINES)
BLOCKS = prose_blocks(HELLO)
PARAS = [t for k, t in BLOCKS if k == "p"]


def para_with(*pats):
    for t in PARAS:
        if all(re.search(p, t, re.I) for p in pats):
            return t
    return None


# ---------------------------------------------------------------------------
# AC 5 (common control header) and the layout tables
# ---------------------------------------------------------------------------
KEY_RULES = [("magic", r"^magic"), ("version", r"^(protocol\s+)?version"), ("domain", r"^domain"),
             ("type", r"^message\s+type"), ("scheme", r"^(authentication\s+)?scheme"),
             ("counter", r"^(message\s+)?counter"), ("tag", r"^(authentication\s+)?tag")]


def key_of(meaning):
    m = clean_start(meaning).lower()
    for k, rx in KEY_RULES:
        if re.search(rx, m):
            return k
    return None


HDR_SUB = find_sub(SUBS, r"common\s+control\s+header")
HDR_TABLE = None
if not HDR_SUB:
    fail(5, "no '### Common control header' subsection in the Hello section")
else:
    HDR_TABLE = first_table(HDR_SUB[0][1], is_field_table)
    if HDR_TABLE is None:
        fail(5, "the common control header subsection has no five-column field table")
HDR = fields_of(HDR_TABLE, 5, "common control header")
HDR_KEYS = [key_of(f["meaning"]) for f in HDR]
EXPECT_HDR_KEYS = {"magic", "version", "domain", "type", "scheme", "counter", "tag"}
if HDR and set(HDR_KEYS) != EXPECT_HDR_KEYS:
    fail(5, f"the common control header must carry magic, version, domain, message type, "
         f"authentication scheme, counter and tag; it carries {HDR_KEYS}")
HDR_WANT = {"magic": 16, "version": 16, "domain": K["domain_bits"], "type": 8,
            "scheme": K["scheme_bits"], "counter": K["counter_bits"], "tag": K["tag_bits"]}
for f, k in zip(HDR, HDR_KEYS):
    if k in HDR_WANT and HDR_WANT[k] is not None and f["bits"] != HDR_WANT[k]:
        fail(5, f"common control header {k} is {f['bits']} bits; the constants fix {HDR_WANT[k]} bits")
HDR_TOTAL = 0
cur = 0
for f in HDR:
    if f["off"] != cur:
        fail(5, f"common control header: {f['meaning'][:30]} at byte {f['off']}, expected {cur}")
    cur += f["bits"] // 8
HDR_TOTAL = cur
if HDR and HDR_TOTAL != 32:
    fail(5, f"the common control header sums to {HDR_TOTAL} bytes (the sync header is 32)")
hdr_para = para_with(r"common\s+control\s+header", r"\d")
hdr_stated = None
if hdr_para:
    m = re.search(r"common\s+control\s+header[^\n]*?(" + NUM + r")\s*bytes", hdr_para, re.I)
    if m:
        hdr_stated = to_int(m.group(1))
if hdr_stated is None:
    fail(5, "no sentence states the common control header's size in bytes")
elif HDR and hdr_stated != HDR_TOTAL:
    fail(5, f"the common control header states {hdr_stated} bytes; its table sums to {HDR_TOTAL}")
if not para_with(r"reuse", r"004-17", r"004-18", r"004-24", r"004-27"):
    fail(5, "no sentence says that 004-17, 004-18, 004-24 and 004-27 reuse the common control header")

# The sync section's leading header: the order the hello header must match, or differ from, explicitly.
SYNC_SPAN = section_span(r"^sync\s+request\s+and\s+reply", level=2)
SYNC_KEYS = []
if SYNC_SPAN is None:
    fail(5, "no 'Sync request and reply' section in wire.md to compare the header with")
else:
    stext = "\n".join(lines[SYNC_SPAN[0]:SYNC_SPAN[1]])
    st = first_table(stext, is_field_table)
    if st is not None:
        SYNC_KEYS = [key_of(r[3]) for r in st[2][:7]]
KEY_LABEL = {"magic": r"magic", "version": r"version", "domain": r"domain", "type": r"message\s+type",
             "scheme": r"scheme", "counter": r"counter", "tag": r"\btag\b"}
if SYNC_KEYS and HDR_KEYS and SYNC_KEYS != HDR_KEYS:
    differing = [k for i, k in enumerate(HDR_KEYS) if i >= len(SYNC_KEYS) or SYNC_KEYS[i] != k]
    excuse = None
    for kind, t in prose_blocks(HDR_SUB[0][1] if HDR_SUB else ""):
        if kind == "p" and re.search(r"differs\s+from\s+the\s+sync\s+header", t, re.I):
            excuse = t
            break
    if excuse is None:
        fail(5, f"the hello header order {HDR_KEYS} differs from the sync header order {SYNC_KEYS}; "
             "the only excuse is a paragraph in the Common control header subsection that says "
             "'differs from the sync header' and names the fields")
    else:
        for k in differing:
            if not re.search(KEY_LABEL[k], excuse, re.I):
                fail(5, f"the 'differs from the sync header' paragraph does not name the differing field '{k}'")

# The layout: starts with the common header, rows tile from byte 0.
LAY_SUB = [(t, x) for t, x in find_sub(SUBS, r"layout")]
LAY_TABLE = None
if not LAY_SUB:
    fail(5, "no '### Hello layout' subsection with the hello's field table")
else:
    LAY_TABLE = first_table(LAY_SUB[0][1], is_field_table)
    if LAY_TABLE is None:
        fail(5, "the hello layout subsection has no five-column field table")
LAY = fields_of(LAY_TABLE, 1, "hello layout")
if LAY:
    if len(LAY) < 7 or [key_of(f["meaning"]) for f in LAY[:7]] != HDR_KEYS:
        fail(5, "the hello's own table does not start with the common control header rows")
    else:
        for a, b in zip(LAY[:7], HDR):
            if (a["off"], a["bits"], a["type"]) != (b["off"], b["bits"], b["type"]):
                fail(5, f"hello header row {b['meaning'][:30]} differs from the common control header "
                     "in offset, width or type")
cur = 0
for f in LAY:
    if f["off"] != cur:
        fail(5, f"hello layout: '{f['meaning'][:40]}' at byte {f['off']}, expected {cur} (rows must tile)")
    if size_bits(f) % 8:
        fail(5, f"hello layout: '{f['meaning'][:40]}' is not a whole number of bytes")
    cur = f["off"] + size_bits(f) // 8
H_BYTES = cur


def md_name(meaning):
    m = re.match(r"\s*([A-Za-z][A-Za-z ]*[A-Za-z]|[A-Za-z])", meaning)
    return m.group(1) if m else meaning


def norm_name(text):
    return re.sub(r"[\s_]+", " ", text.replace("-", " ").lower()).strip()


def row_for(pattern, counted=None):
    for f in LAY:
        if re.match(r"\s*" + pattern, f["meaning"], re.I) and (counted is None or (f["maxc"] is not None) == counted):
            return f
    return None


def need_row(name, pattern, bits=None, counted=None, ac=1):
    f = row_for(pattern, counted)
    if f is None:
        fail(ac, f"the hello layout has no row for {name} (its Meaning must begin with the name)")
        return None
    if bits is not None and f["bits"] != bits:
        fail(ac, f"{name} is {f['bits']} bits; the constants fix {bits} bits")
    return f


# AC 1: the seventeen hello contents, each as a row or a group of rows
need_row("module id (C-11)", r"module\s+id", K["module_bits"], ac=1)
need_row("instance nonce (C-11)", r"instance\s+nonce", K["nonce_bits"], ac=1)
need_row("protocol version", r"(protocol\s+)?version", 16, ac=1)
need_row("name length", r"name\s+length", 8, ac=1)
NAME_ROW = row_for(r"name\b", counted=True)
if NAME_ROW is None:
    fail(1, "the hello layout has no counted name row (its Meaning must begin with 'Name')")
elif not re.search(r"UTF-8|ASCII", NAME_ROW["meaning"] + " " + NAME_ROW["valid"]):
    fail(1, "the name row does not state its encoding (UTF-8 or ASCII)")
CAP_ROW = need_row("capability bits", r"capabilit", None, ac=1)
if CAP_ROW is not None:
    if CAP_ROW["bits"] != 32:
        fail(1, f"capability bits are {CAP_ROW['bits']} bits; the row must state its bit count (32)")
    if not re.search(r"holder[- ]eligible[^.;]*?\bbit\s+\d+", CAP_ROW["meaning"] + " " + CAP_ROW["valid"], re.I):
        fail(1, "the capability row does not fix the holder-eligible bit at a stated bit (for example 'bit 0')")
PORT_COUNT = need_row("port count", r"port\s+count", 8, ac=1)
PORT_ROW = row_for(r"ports?\b", counted=True)
if PORT_ROW is None:
    fail(1, "the hello layout has no counted port row (its Meaning must begin with 'Port')")
need_row("election epoch", r"election\s+epoch", None, ac=1)
need_row("believed holder (module id, 64 bits)", r"believed\s+holder", K["module_bits"], ac=1)
need_row("timeline epoch (C-10)", r"timeline\s+epoch", K["epoch_bits"], ac=1)
need_row("transport role", r"transport\s+role", None, ac=1)
need_row("alarm field", r"alarm", None, ac=1)
need_row("regime stamp version number (C-18)", r"regime\s+stamp", None, ac=1)
BUN_COUNT = row_for(r"bundle\s+count")
BUN_ROW = None
for f in LAY:
    if re.match(r"\s*bundle\b", f["meaning"], re.I) and not re.match(r"\s*bundle\s+count", f["meaning"], re.I):
        BUN_ROW = f
HAS_BUNDLES = any(re.match(r"\s*bundle", f["meaning"], re.I) for f in LAY)
if not HAS_BUNDLES and LAY:
    pass
if LAY and not any(re.search(r"liveness\s+counter", f["meaning"], re.I) for f in LAY[:7]) and \
        not para_with(r"message\s+counter", r"liveness\s+counter"):
    fail(1, "the message counter is not stated as the data-plane liveness counter (C-115)")
if not para_with(r"liveness\s+counter", r"message\s+counter|counter"):
    fail(1, "no sentence says that the message counter doubles as the liveness counter")

# bundle element sub-table (the 32-member description, in both outcomes)
BUNDLE_SUB = find_sub(SUBS, r"bundle\s+element")
BUN_FIELDS = []
if not BUNDLE_SUB:
    fail(1, "no '### Bundle element' subsection describing a bundle's group id, member list and channel map")
else:
    BUN_FIELDS = fields_of(first_table(BUNDLE_SUB[0][1], is_field_table), 2, "bundle element")
    text_b = " ".join(f["meaning"] for f in BUN_FIELDS).lower()
    for want in ("group id", "member", "channel map"):
        if want not in text_b:
            fail(1, f"the bundle element has no row for '{want}'")
E_BYTES = sum(size_bits(f) for f in BUN_FIELDS) // 8 if BUN_FIELDS else 0
bo = 0
for f in BUN_FIELDS:
    if f["off"] != bo:
        fail(2, f"bundle element: '{f['meaning'][:30]}' at byte {f['off']}, expected {bo}")
    bo = f["off"] + size_bits(f) // 8
members_max = K["members_max"]
member_rows = [f for f in BUN_FIELDS if f["maxc"] is not None]
for f in member_rows:
    if members_max is not None and f["maxc"] > members_max:
        fail(2, f"bundle element row '{f['meaning'][:40]}' allows {f['maxc']} members, above C-08's {members_max}")
    if re.search(r"member|stream|channel", f["meaning"], re.I) and members_max is not None and f["maxc"] != members_max:
        fail(2, f"bundle element row '{f['meaning'][:40]}' has max {f['maxc']}; C-08 fixes {members_max} members")
if members_max is not None and not any(f["maxc"] == members_max for f in member_rows):
    fail(2, f"the bundle element has no counted member row with the maximum of {members_max} members (C-08)")

# port element sub-table
PORT_SUB = find_sub(SUBS, r"port\s+element")
PORT_FIELDS = []
T_CT = None
if not PORT_SUB:
    fail(1, "no '### Port element' subsection describing a port and its content types")
else:
    PORT_FIELDS = fields_of(first_table(PORT_SUB[0][1], is_field_table), 1, "port element")
    ct = [f for f in PORT_FIELDS if re.search(r"content\s+type", f["meaning"], re.I) and f["maxc"]]
    if not ct:
        fail(1, "the port element has no counted content-type row")
    else:
        T_CT = ct[0]["maxc"]
    if PORT_ROW is not None and PORT_FIELDS:
        pe = sum(size_bits(f) for f in PORT_FIELDS)
        if PORT_ROW["bits"] != pe:
            fail(1, f"the port row is {PORT_ROW['bits']} bits per port; its element sums to {pe} bits")
E_PORT = sum(size_bits(f) for f in PORT_FIELDS) // 8 if PORT_FIELDS else 0
P_MAX = PORT_ROW["maxc"] if PORT_ROW is not None else None

# AC 2: the sizes, the verdict and the stated arithmetic
#   Bundles per hello (B) is stated whenever bundles are carried, and it agrees with the Bundle
#   row's max and the toml max_count. The absolute maximum (all bundles at B, every name and port
#   maximum) is the layout's sum; it must be stated and must fit for a "fits" verdict. The
#   one-bundle figure (one 32-member bundle, C-08) is stated as a sentence with "one" or "single",
#   "32" and "bundle", and must be correct; it is not read from the all-bundles sentence.
bundle_fallback_B = None
B_TEXT_M = re.search(r"at\s+most\s+(\d+)\s+bundles", HELLO_PLAIN, re.I)
B_TEXT = int(B_TEXT_M.group(1)) if B_TEXT_M else None
if HAS_BUNDLES:
    if BUN_ROW is None or BUN_ROW["maxc"] is None:
        fail(2, "the hello layout's bundle row is not a counted row 'E × max B'")
        B_MAX = None
    else:
        B_MAX = BUN_ROW["maxc"]
        if BUN_ROW["bits"] != E_BYTES * 8:
            fail(2, f"the bundle row is {BUN_ROW['bits']} bits per bundle; the bundle element sums to {E_BYTES * 8}")
    if BUN_COUNT is None or BUN_COUNT["bits"] != 8:
        fail(2, "the hello layout has no one-byte bundle count row before the bundles")
    if B_TEXT is None:
        fail(2, "bundles are carried but the Hello section states no bundles-per-hello maximum "
             "('at most N bundles')")
    elif B_MAX is not None and B_TEXT != B_MAX:
        fail(2, f"the stated bundles-per-hello maximum is {B_TEXT}; the bundle row says max {B_MAX}")
    if B_MAX is not None:
        more_p = [t for t in PARAS if re.search(
            r"(more|above|beyond|exceed\w*)\s+(than\s+)?(\d+|this|that)\s+bundles|bundles\s+(than|above|beyond)\b"
            r"|source\s+with\s+more\s+bundles", t, re.I)]
        ok_rule = any(
            re.search(r"\brule\b", t, re.I)
            or (clean_start(t).lower().startswith("open item") and re.search(r"owner", t, re.I)
                and re.search(r"falsifier", t, re.I))
            for t in more_p)
        if not ok_rule:
            fail(2, "no rule or 'Open item' (owner and falsifier) says what a source with more bundles "
                 "than the stated maximum does")
else:
    B_MAX = B_TEXT

NON_BUNDLE_BYTES = sum(size_bits(f) for f in LAY if not re.match(r"\s*bundle", f["meaning"], re.I)) // 8
ABS_ALL = None
ONE = None
if B_MAX is not None and E_BYTES:
    ABS_ALL = NON_BUNDLE_BYTES + 1 + B_MAX * E_BYTES
    ONE = NON_BUNDLE_BYTES + 1 + E_BYTES

if B_MAX is not None and K["streams_max"] and K["members_max"]:
    needed = -(-K["streams_max"] // K["members_max"])
    if B_MAX < needed and not para_with(r"\b%d\b" % K["streams_max"], r"bundle", r"rule|open\s+item"):
        fail(2, f"at most {B_MAX} bundles per hello cannot carry C-09's {K['streams_max']} streams "
             f"({needed} bundles of {K['members_max']}); no rule or open item says what a source with more does")

ONE_STATED = None
ONE_RX = re.compile(r"\b(?:one|single)\s+(?:32-member\s+bundle|bundle\s+of\s+32\s+members?|32-member\s+|bundle)\b", re.I)
for sent in re.split(r"(?<=[.;])\s+", HELLO_PLAIN.replace("\n", " ")):
    if not ONE_RX.search(sent):
        continue
    eq = re.search(r"=\s*(" + NUM + r")\s*bytes", sent)
    m1 = eq or re.search(r"(" + NUM + r")\s*bytes", sent)
    if m1:
        ONE_STATED = to_int(m1.group(1))
        break
if ONE is not None:
    if ONE_STATED is None:
        fail(2, "the one-bundle figure is not stated: a sentence with 'one' or 'single', '32' and 'bundle' "
             "and the size in bytes (C-08, the case C-17's falsifier names)")
    elif ONE_STATED != ONE:
        fail(2, f"the one-bundle figure is stated as {ONE_STATED} bytes; the tables compute {ONE}")

m_max = re.search(r"maximum\s+hello\s+size[^\n]*?(" + NUM + r")\s*bytes", HELLO_PLAIN, re.I)
STATED_MAX = to_int(m_max.group(1)) if m_max else None
if STATED_MAX is None:
    fail(2, "the maximum hello size is not stated in bytes ('maximum hello size ... N bytes')")

VERDICT_P = None
for t in PARAS:
    if re.match(r"\s*\*{0,2}\s*verdict\b", clean_start(t), re.I):
        VERDICT_P = t
        break

VERDICT_BODY = re.sub(r"(?:(?<=\s)|^)'[^']*'(?=\s|[.,;:)]|$)|\"[^\"]*\"|“[^”]*”|`[^`]*`", " ", VERDICT_P or "")


def classify(body):
    """'fits', 'falsified', 'ambiguous' or None, by the verdict's own statement (quoted text excluded)."""
    fits_n = len(re.findall(r"(?<!not )\bfits\b", body, re.I))
    fits_n += len(re.findall(r"\bC-17\s+(?:holds|stands)\b|\bdoes\s+not\s+hold\b|\bnot\s+falsified\b",
                             body, re.I))
    fals_n = len(re.findall(r"(?<!not )\bfalsified\b", body, re.I))
    fals_n += len(re.findall(r"\bdoes\s+not\s+fit\b|\bfalsifier\s+holds\b", body, re.I))
    if fits_n and fals_n:
        return "ambiguous"
    if fits_n:
        return "fits"
    if fals_n:
        return "falsified"
    return None


verdict = classify(VERDICT_BODY) if VERDICT_P else None
if VERDICT_P is None:
    fail(2, "no 'Verdict' paragraph says whether the hello fits the control bound or is falsified")
elif verdict is None:
    fail(2, "the verdict paragraph states neither 'fits' nor 'falsified' in its own words")
elif verdict == "ambiguous":
    fail(2, "the verdict paragraph says both 'fits' and 'falsified': state one verdict")
if VERDICT_P is not None and ONE is not None and ONE <= K["bound"] and \
        re.search(r"C-17[^.;]*?(falsified|falsifier\s+holds|holds\s+for)", VERDICT_BODY, re.I):
    fail(2, f"the verdict says C-17's falsifier holds, but that falsifier is about one 32-member bundle, "
         f"which computes {ONE} bytes (within {K['bound']}): C-17 is not falsified")

if verdict == "fits":
    if not HAS_BUNDLES:
        fail(2, "the verdict says it fits but the hello layout carries no bundle rows")
    if H_BYTES > K["bound"]:
        fail(2, f"the absolute maximum hello, with every stated maximum, is {H_BYTES} bytes, above "
             f"{K['bound']} (C-17), but the verdict says it fits")
    if STATED_MAX is not None and STATED_MAX != H_BYTES:
        fail(2, f"the maximum hello size is stated as {STATED_MAX} bytes; the table sums to {H_BYTES}")
elif verdict == "falsified":
    if HAS_BUNDLES:
        fail(2, "the verdict says falsified but the layout still carries the bundle rows: the table must be the fallback")
    if ABS_ALL is None:
        fail(2, "the all-bundles maximum cannot be computed: the bundle element or the bundles-per-hello maximum is missing")
    elif ABS_ALL <= K["bound"]:
        fail(2, f"the verdict says falsified but the all-bundles maximum is {ABS_ALL} bytes, not above {K['bound']}")
    if H_BYTES > K["bound"]:
        fail(2, f"the fallback hello is {H_BYTES} bytes, above {K['bound']}: the remaining message must fit")
    if STATED_MAX is not None and STATED_MAX != H_BYTES:
        fail(2, f"the maximum hello size is stated as {STATED_MAX} bytes; the fallback table sums to {H_BYTES}")
    if ABS_ALL is not None and ABS_ALL not in {to_int(x) for x in re.findall(r"(" + NUM + r")\s*bytes", HELLO_PLAIN)}:
        fail(2, f"the all-bundles size {ABS_ALL} bytes is not stated in bytes in the Hello section")
    claims_c17 = bool(re.search(r"C-17", VERDICT_BODY))
    if claims_c17 and (ONE is None or ONE <= K["bound"]):
        fail(2, f"the verdict says C-17 is falsified, but C-17's falsifier is about one 32-member bundle, "
             f"which computes {ONE} bytes (within {K['bound']}): C-17 holds")
    if not re.search(r"split|handshake", VERDICT_P + " " + HELLO_PLAIN, re.I):
        fail(3, "the falsified verdict does not name the fallback (split hello or handshake)")

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


# AC 3: the alarm field (six connection alarms of C-173, one bit or value each, distinct)
ALARMS = [("refusal", r"refus"), ("interrupted", r"interrupt"), ("dead", r"\bdead\b"),
          ("demoted", r"demot"), ("cannot-meet-A", r"cannot[\s_-]*meet"),
          ("control-degraded", r"control[\s_-]*degrad")]
ALARM_SUB = [(t, x) for t, x in SUBS if re.search(r"alarm", t, re.I)]
if not ALARM_SUB:
    fail(3, "no '### Alarm field' subsection with the encoding of the six connection alarms")
else:
    at = None
    for title, text in ALARM_SUB:
        for tb in parse_tables(text):
            if len(tb[2]) >= 6:
                at = tb
                break
        if at:
            break
    if at is None:
        fail(3, "the alarm subsection has no table with one row per alarm")
    else:
        values = {}
        for name, pat in ALARMS:
            hit = None
            for r in at[2]:
                if r and re.search(pat, r[0], re.I):
                    hit = r
                    break
            if hit is None:
                fail(3, f"the alarm table has no row for the alarm '{name}' (C-173)")
                continue
            rest = hit[1] if len(hit) > 1 else ""
            m_bit = re.search(r"bit\s+(\d+)", rest, re.I)
            m_hex = re.search(r"0x([0-9a-f]+)", rest, re.I)
            m_val = re.search(r"(?:value\s+)?\b(\d+)\b", rest, re.I)
            if m_bit:
                v = 1 << int(m_bit.group(1))
            elif m_hex:
                v = int(m_hex.group(1), 16)
            elif m_val:
                v = int(m_val.group(1))
            else:
                fail(3, f"the alarm '{name}' has no bit or value")
                continue
            values[name] = v
        if len(set(values.values())) != len(values):
            fail(3, f"the alarm encodings are not distinct: {values}")
        if any(v == 0 for v in values.values()):
            fail(3, "an alarm is encoded as zero, which means no alarm")
        ALARM_LAY = row_for(r"alarm")
        if ALARM_LAY is not None and values and max(values.values()).bit_length() > ALARM_LAY["bits"]:
            fail(3, "an alarm value does not fit the alarm field's width")

# AC 3: transport role, a proposal with epic 005 as confirming owner and its falsifier
ROLE_SUB = find_sub(SUBS, r"transport\s+role")
if not ROLE_SUB:
    fail(3, "no '### Transport role' subsection with the proposed role values")
else:
    rt = first_table(ROLE_SUB[0][1])
    if rt is None or len(rt[2]) < 2:
        fail(3, "the transport role subsection has no table with its values")
    if not re.search(r"proposal", ROLE_SUB[0][1], re.I):
        fail(3, "the transport role values are not recorded as a proposal")
ROLE_OPEN = None
for t in PARAS:
    if re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(t), re.I) and re.search(r"transport\s+role", t, re.I):
        ROLE_OPEN = t
        break
if ROLE_OPEN is None:
    fail(3, "no paragraph starting 'Open item' states the transport role open question")
else:
    if not re.search(r"\b005\b", ROLE_OPEN) or not re.search(r"owner", ROLE_OPEN, re.I):
        fail(3, "the transport role open item does not name epic 005 as its confirming owner")
    if not re.search(r"epic\s+005\s+needs\s+a\s+role\s+the\s+encoding\s+cannot\s+express", ROLE_OPEN, re.I):
        fail(3, "the transport role open item lacks its falsifier 'epic 005 needs a role the encoding cannot express'")

# AC 4: at least ten invalid forms with outcomes
NON_REJECTION = re.compile(r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|"
                           r"cannot\s+occur|ignored", re.I)
FORM_SUB = [(t, x) for t, x in SUBS if re.search(r"invalid|validity", t, re.I)]
FORM_ROWS = []
FORM_TABLE = None
for title, text in FORM_SUB:
    for tb in parse_tables(text):
        h = [c.lower() for c in tb[1]]
        if any("form" in c or "invalid" in c for c in h) and any("outcome" in c for c in h):
            FORM_TABLE = tb
            break
    if FORM_TABLE:
        break
if FORM_TABLE is None:
    fail(4, "no invalid-form table with 'Form' and 'Outcome' columns in the Hello section")
    FORM_ROWS = []
else:
    fi = next(i for i, c in enumerate(FORM_TABLE[1]) if re.search(r"form|invalid", c, re.I))
    oi = next(i for i, c in enumerate(FORM_TABLE[1]) if re.search(r"outcome", c, re.I))
    for r in FORM_TABLE[2]:
        if len(r) > max(fi, oi):
            FORM_ROWS.append((r[fi], r[oi]))
if len(FORM_ROWS) < 10:
    fail(4, f"{len(FORM_ROWS)} invalid form(s) with outcomes; at least 10 are required")


def cat_of(form):
    f = form.lower()
    if re.search(r"too\s+short|shorter|fewer\s+than|less\s+than\s+\d+\s*bytes", f):
        return "short"
    if "magic" in f:
        return "magic"
    if "version" in f:
        return "version"
    if re.search(r"collision|same\s+module\s+id", f):
        return "collision"
    if re.search(r"domain\s*(id\s*)?0\b|unconfigured", f):
        return "domain0"
    if re.search(r"foreign|different\s+non-?zero", f):
        return "foreign"
    if re.search(r"oversiz|larger\s+than|above\s+1", f):
        return "oversize"
    if re.search(r"(count|member|bundle|stream)[^|]*(above|exceed|more\s+than|beyond)", f):
        return "counts"
    if re.search(r"length|size", f):
        return "length"
    if re.search(r"counter", f):
        return "counter"
    if re.search(r"\bname\b|utf|encoding", f):
        return "name"
    if re.search(r"reserved|non-?zero|scheme|\btag\b", f):
        return "reserved"
    return None


EXACT = {"short": "TooShort", "magic": "BadMagic", "version": "VersionMismatch",
         "domain0": "UnconfiguredDomain", "foreign": "ForeignDomain", "oversize": "OversizeControl",
         "counts": "BadLength", "length": "BadLength", "reserved": "ReservedNonZero"}


def named_reasons(outcome):
    """Member names in the outcome that are not negated ('not X', 'never X')."""
    found = []
    for m in re.finditer(r"\b[A-Za-z]+\b", outcome):
        w = m.group(0)
        if w in MEMBERS:
            before = outcome[max(0, m.start() - 24):m.start()]
            if re.search(r"\b(not|never|nor|no|neither)\b[^|;.]*$", before, re.I):
                continue
            found.append(w)
    return found


FORM_EXACT_SEEN = set()
EXACT_OUTCOMES = {}
for form, outcome in FORM_ROWS:
    for name in CAMEL.findall(outcome):
        if name not in MEMBERS and not re.search(r"\b(not|never|nor)\b[^|;.]*" + name, outcome):
            fail(4, f"the outcome '{outcome[:60]}' names '{name}', which is not in the closed enumeration")
    cat = cat_of(form)
    if cat in EXACT:
        FORM_EXACT_SEEN.add(cat)
        EXACT_OUTCOMES[cat] = outcome
        if not re.search(r"\b" + EXACT[cat] + r"\b", outcome):
            fail(4, f"the form '{form[:50]}' must be counted under {EXACT[cat]}: {outcome[:70]}")
        if cat == "domain0" and "ForeignDomain" in named_reasons(outcome):
            fail(4, "domain 0 is mapped to ForeignDomain; it is ignored and counted as UnconfiguredDomain (C-173)")
        if cat == "version" and not re.search(r"\bevent\b", outcome, re.I):
            fail(4, "the version mismatch row does not say that it raises a named event")
    elif cat is None or cat in ("collision", "counter", "name"):
        if not (named_reasons(outcome) or NON_REJECTION.search(outcome)):
            fail(4, f"the form '{form[:50]}' names no member of the closed enumeration and argues no non-rejection")
    if cat in ("collision", "counter", "name") and not (named_reasons(outcome) or NON_REJECTION.search(outcome)):
        fail(4, f"the form '{form[:50]}' needs a member reason or an argued non-rejection")
for cat in ("short", "magic", "version", "domain0", "foreign", "oversize", "counts", "length", "reserved"):
    if cat not in FORM_EXACT_SEEN:
        fail(4, f"no invalid-form row covers the '{cat}' case of the ticket's list")
if not any(cat_of(f) == "counter" for f, _ in FORM_ROWS):
    fail(4, "no invalid-form row covers a liveness counter going backwards for the same instance nonce")
if not any(cat_of(f) == "name" for f, _ in FORM_ROWS):
    fail(4, "no invalid-form row covers a name that is not valid in the chosen encoding")
if not any(cat_of(f) == "collision" for f, _ in FORM_ROWS) and not para_with(r"collision", r"not\s+a\s+parse\s+rejection"):
    fail(4, "the module-id collision (C-46) is not stated as a non-rejection")
if not para_with(r"collision", r"not\s+a\s+parse\s+rejection"):
    fail(4, "no sentence says that a module-id collision (same module id, different instance nonce, C-46) "
         "is not a parse rejection (state-machine behaviour, epic 005)")
EXACT_REASONS = set()
for _, outcome in FORM_ROWS:
    EXACT_REASONS.update(named_reasons(outcome))

# ---------------------------------------------------------------------------
# AC 3: regime stamp version number (Work 3) and the behaviour restated as fact
# ---------------------------------------------------------------------------
if not para_with(r"regime\s+stamp", r"\bstale\b", r"\b(equal|equality|serial|wrap)"):
    fail("Work3", "no sentence gives the comparison rule of the regime stamp version number: a receiver "
         "treats its copy as stale by equality, serial comparison or wrap")
if not para_with(r"regime\s+stamp", r"version\s+number|not\s+applicable"):
    fail("DoD", "no sentence says the regime stamp fields the hello touches are documented (version number) "
         "or not applicable")
if not para_with(r"C-18", r"version\s+number", r"own"):
    fail("Work3", "no sentence states that the stamp travels in its own message and the hello carries only "
         "its version number (C-18)")
BEHAVIOUR = [
    ("hello every 1 s (C-30)", r"\b1\s*s\b", r"C-30"),
    ("multicast with TTL 1 and unicast over IPv4 (C-100)", r"TTL\s+1", r"C-100", r"unicast", r"IPv4"),
    ("DSCP CS5 (40) (C-13)", r"CS5\s*\(40\)", r"C-13"),
    ("binary, not the POC's text gossip ([code])", r"binary", r"\btext\b", r"\[code\]", r"HADRO/1|POC"),
    ("64-entry peer map (C-34)", r"\b64\b", r"C-34"),
]
for name, *pats in BEHAVIOUR:
    for p in pats:
        if not re.search(p, HELLO_PLAIN, re.I):
            fail("Behaviour", f"behaviour not restated as fact: {name} (missing '{p}')")
            break
if K["hello_period_s"] is not None and not re.search(r"\b%d\s*s\b" % K["hello_period_s"], HELLO_PLAIN):
    fail("Behaviour", "the hello period is not restated with the value of C-30")
if K["dscp_control"] is not None and not re.search(r"\(%d\)" % K["dscp_control"], HELLO_PLAIN):
    fail("Behaviour", "CS5 is not stated with the value of C-13")
if K["peers_max"] is not None and not re.search(r"\b%d\b" % K["peers_max"], HELLO_PLAIN):
    fail("Behaviour", "the peer map bound is not restated with the value of C-34")

def itertools_takewhile(ls):
    out = []
    for ln in ls:
        if re.match(r"^###\s", ln):
            break
        out.append(ln)
    return out


# ---------------------------------------------------------------------------
# DoD: evidence tags on prose and on every table's subsection; open items name owner and falsifier
# ---------------------------------------------------------------------------
TOP_TEXT = "\n".join(itertools_takewhile(HELLO_LINES))
for title, text in [("(section)", TOP_TEXT)] + SUBS:
    blocks = prose_blocks(text)
    has_tag_here = has_evidence(text)
    for kind, body in blocks:
        if kind == "p":
            words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", body)))
            if words >= 5 and not has_evidence(body):
                fail("DoD", f"a prose paragraph of '{title}' carries no evidence tag: {body.splitlines()[0][:80]}")
            if re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), re.I):
                if not re.search(r"owner", body, re.I) or not re.search(r"falsifier", body, re.I):
                    fail("DoD", f"an open item in '{title}' names no owner or no falsifier: {body.splitlines()[0][:80]}")
        elif kind == "t" and not has_tag_here:
            fail("DoD", f"a table of '{title}' follows no evidence-tagged paragraph")

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
HELLO_T = [m for m in msgs if isinstance(m, dict) and str(m.get("name", "")).strip().lower() == "hello"]
for m in msgs:
    if isinstance(m, dict) and re.match(r"hello", str(m.get("name", "")), re.I) and m.get("layer") != "control":
        fail(6, f"toml message '{m.get('name')}' is a hello message but its layer is not 'control'")
if not HELLO_T:
    fail(6, "wire.toml declares no [[message]] named 'hello'")
else:
    T = HELLO_T[0]
    if T.get("layer") != "control":
        fail(6, f"the toml hello has layer '{T.get('layer')}', not 'control'")
    if K["dscp_control"] is not None and T.get("dscp") != K["dscp_control"]:
        fail(6, f"the toml hello has dscp {T.get('dscp')}; C-13 fixes CS5 ({K['dscp_control']})")
    if T.get("max_size") != H_BYTES:
        fail(6, f"the toml hello has max_size {T.get('max_size')}; the markdown layout sums to {H_BYTES}")
    if K["bound"] and isinstance(T.get("max_size"), int) and T["max_size"] > K["bound"]:
        fail(6, f"the toml hello max_size {T['max_size']} exceeds the {K['bound']}-byte control bound (C-17)")
    tf = T.get("field") or []
    if len(tf) != len(LAY):
        fail(6, f"the toml hello has {len(tf)} fields; the markdown layout has {len(LAY)} rows: they must correspond one to one")
    PRIM = {"u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"}
    names = [f.get("name") for f in tf if isinstance(f, dict)]
    total_bits = 0
    for i, (mf, tfl) in enumerate(zip(LAY, tf)):
        if not isinstance(tfl, dict):
            continue
        if norm_name(md_name(mf["meaning"])) != norm_name(str(tfl.get("name", ""))):
            fail(6, f"toml hello field '{tfl.get('name')}' is not named after the markdown row "
                 f"'{md_name(mf['meaning'])}' (names are compared lower case, spaces and underscores equivalent)")
        if tfl.get("offset") != mf["off"] * 8:
            fail(6, f"toml hello field {tfl.get('name')}: offset {tfl.get('offset')} bits, "
                 f"markdown byte {mf['off']} (= {mf['off'] * 8} bits)")
        if mf["maxc"] is not None:
            if tfl.get("width") != mf["bits"]:
                fail(6, f"toml hello field {tfl.get('name')}: width {tfl.get('width')} bits per element; "
                     f"markdown element is {mf['bits']} bits")
            if tfl.get("max_count") != mf["maxc"]:
                fail(6, f"toml hello field {tfl.get('name')}: max_count {tfl.get('max_count')}; "
                     f"markdown says max {mf['maxc']}")
            if not tfl.get("count") or tfl.get("count") not in names[:i]:
                fail(6, f"toml hello field {tfl.get('name')}: its count must name an earlier field")
            total_bits += tfl.get("width", 0) * tfl.get("max_count", 0)
        else:
            if tfl.get("width") != mf["bits"]:
                fail(6, f"toml hello field {tfl.get('name')}: width {tfl.get('width')} bits; markdown {mf['bits']}")
            if mf["type"] in PRIM and tfl.get("type") != mf["type"]:
                fail(6, f"toml hello field {tfl.get('name')}: type {tfl.get('type')}; markdown {mf['type']}")
            total_bits += tfl.get("width", 0)
    if isinstance(T.get("max_size"), int) and total_bits != T["max_size"] * 8:
        fail(6, f"the toml hello fields compute {total_bits // 8} bytes; max_size is {T['max_size']}")
    # nested maxima (ports, content types, members) are stated in the element's toml text
    for mf, tfl in zip(LAY, tf):
        if not isinstance(tfl, dict):
            continue
        if PORT_ROW is not None and mf is PORT_ROW:
            txt = str(tfl.get("valid", "")) + str(tfl.get("meaning", ""))
            for v in (P_MAX, T_CT):
                if v is not None and not re.search(r"\b%d\b" % v, txt):
                    fail(6, f"the toml port field does not state the maximum {v} of its nested count")
        if BUN_ROW is not None and mf is BUN_ROW:
            txt = str(tfl.get("valid", "")) + str(tfl.get("meaning", ""))
            if members_max is not None and not re.search(r"\b%d\b" % members_max, txt):
                fail(6, f"the toml bundle field does not state the {members_max}-member maximum (C-08)")
    if EXACT_REASONS:
        have = set(T.get("rejection_reasons") or [])
        for r in sorted(EXACT_REASONS - have):
            fail(6, f"the toml hello rejection_reasons does not list '{r}', which the invalid-form rows name")
        for r in sorted(have - set(MEMBERS)):
            fail(6, f"the toml hello rejection_reasons names '{r}', which is not in the closed enumeration")

# ---------------------------------------------------------------------------
# AC 6 (repository level): the 004-02 checker and the wire_tables tests
# ---------------------------------------------------------------------------
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            fail(6, "cargo run --bin wire-check did not pass:\n" + "\n".join(out.strip().splitlines()[-8:]))
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(6, f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        result = [ln.strip() for ln in out.splitlines() if "test result:" in ln]
        if proc.returncode != 0 or not result:
            fail(6, "cargo test -p hadronyche-tools --test wire_tables failed:\n" + "\n".join(out.strip().splitlines()[-12:]))
        else:
            mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored", result[-1])
            if not mm or (int(mm.group(1)), int(mm.group(2)), int(mm.group(3))) != (12, 0, 0):
                fail(6, "wire_tables must stay 12 passed, 0 failed, 0 ignored; got: " + result[-1])
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(6, f"could not run the wire_tables tests: {error}")
else:
    print("NOTE AC6: document paths overridden; cargo checks (wire-check, wire_tables) skipped")

# ---------------------------------------------------------------------------
if fails:
    for ac, msg in fails:
        print(f"FAIL {label(ac)}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: hello layout, 32-member size bound, six alarms, ten invalid forms, common header, "
      "toml agreement and DoD tags present")
PY
